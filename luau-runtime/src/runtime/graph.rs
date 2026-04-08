use std::{
    collections::{BTreeSet, HashMap},
    fs,
    hash::Hash,
    panic::{catch_unwind, AssertUnwindSafe},
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use log::{info, warn};
use mlua::{Error as LuaError, Function, Lua, Table, Value};
use parking_lot::Mutex;

use super::{runtime_lua_error, source_hash, RuntimeError, ScriptRuntime, ScriptSource};

type ModuleId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadKind {
    Load,
    Reload,
}

impl ReloadKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Reload => "reload",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ModuleKey {
    RootInline,
    Path(PathBuf),
}

impl ModuleKey {
    fn path(&self) -> Option<&Path> {
        match self {
            Self::RootInline => None,
            Self::Path(path) => Some(path.as_path()),
        }
    }

    fn display_name(&self, inline_name: Option<&str>) -> String {
        match self {
            Self::RootInline => inline_name.unwrap_or("<inline root>").to_owned(),
            Self::Path(path) => path.display().to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ActiveModule {
    id: ModuleId,
    key: ModuleKey,
    exports: Table,
    render: Option<Function>,
    pub(super) state: Table,
    dependencies: BTreeSet<ModuleKey>,
    reverse_deps: BTreeSet<ModuleKey>,
    #[allow(dead_code)]
    source_hash: u64,
    version: u64,
}

#[derive(Debug, Clone)]
pub(super) struct ActiveGraph {
    root: ModuleKey,
    root_context: String,
    modules: HashMap<ModuleKey, ActiveModule>,
    path_index: HashMap<PathBuf, ModuleId>,
}

impl ActiveGraph {
    pub(super) fn root_module(&self) -> Option<&ActiveModule> {
        self.modules.get(&self.root)
    }

    pub(super) fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub(super) fn root_context(&self) -> &str {
        &self.root_context
    }

    fn root_dir(&self) -> Option<PathBuf> {
        self.root
            .path()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
    }
}

#[derive(Debug, Clone)]
struct CandidateModule {
    key: ModuleKey,
    exports: Table,
    state: Table,
    dependencies: BTreeSet<ModuleKey>,
    source_hash: u64,
    version: u64,
}

#[derive(Debug)]
pub(super) struct BuildDispatch {
    root_source: ScriptSource,
    old_graph: Option<ActiveGraph>,
    root_dir: Option<PathBuf>,
    affected_paths: BTreeSet<PathBuf>,
    candidates: HashMap<ModuleKey, CandidateModule>,
    build_order: Vec<ModuleKey>,
    loading: Vec<ModuleKey>,
}

impl BuildDispatch {
    fn new(
        root_source: ScriptSource,
        old_graph: Option<ActiveGraph>,
        root_dir: Option<PathBuf>,
        affected_paths: BTreeSet<PathBuf>,
    ) -> Self {
        Self {
            root_source,
            old_graph,
            root_dir,
            affected_paths,
            candidates: HashMap::new(),
            build_order: Vec::new(),
            loading: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub(super) struct ActiveRequireSnapshot {
    root_dir: Option<PathBuf>,
    exports_by_key: HashMap<ModuleKey, Table>,
}

#[derive(Debug, Clone)]
pub(super) struct CommittedActiveRuntime {
    pub(super) graph: ActiveGraph,
    pub(super) root_update: Option<Function>,
    pub(super) root_render: Function,
    pub(super) root_state: Table,
    pub(super) require_snapshot: Arc<ActiveRequireSnapshot>,
}

#[derive(Debug)]
pub(super) struct StagedRuntime {
    pub(super) runtime: CommittedActiveRuntime,
    pub(super) source: ScriptSource,
    pub(super) kind: ReloadKind,
    pub(super) old_modules_to_shutdown: Vec<ActiveModule>,
    pub(super) old_require_snapshot: Option<Arc<ActiveRequireSnapshot>>,
}

#[derive(Debug)]
pub(super) enum RequireMode {
    Disabled,
    Build(BuildDispatch),
    Active(Arc<ActiveRequireSnapshot>),
}

impl ScriptRuntime {
    pub(super) fn load_root_inner(&mut self, source: ScriptSource) -> Result<(), RuntimeError> {
        let stage = self.failure_stage_for_reload_kind(ReloadKind::Load);
        let root_name = script_source_display_name(&source);
        let source = normalize_script_source(source)
            .map_err(|err| self.record_compile_or_load_error(stage, Some(root_name), err))?;
        self.rebuild_graph(source, Vec::new(), ReloadKind::Load)
            .map(|staged| {
                self.pending_candidate = Some(staged);
            })
    }

    pub(super) fn reload_root_inner(&mut self, source: ScriptSource) -> Result<(), RuntimeError> {
        let stage = self.failure_stage_for_reload_kind(ReloadKind::Reload);
        let root_name = script_source_display_name(&source);
        let dirty_paths = self.reload_queue.drain();
        let source = normalize_script_source(source)
            .map_err(|err| self.record_compile_or_load_error(stage, Some(root_name), err))?;
        self.rebuild_graph(source, dirty_paths, ReloadKind::Reload)
            .map(|staged| {
                self.pending_candidate = Some(staged);
            })
    }

    fn rebuild_graph(
        &mut self,
        source: ScriptSource,
        dirty_paths: Vec<PathBuf>,
        kind: ReloadKind,
    ) -> Result<StagedRuntime, RuntimeError> {
        let root_name = script_source_display_name(&source);
        let stage = self.failure_stage_for_reload_kind(kind);
        let dirty_count = dirty_paths.len();
        let started_at = Instant::now();

        let result = (|| -> Result<(StagedRuntime, usize, usize), RuntimeError> {
            let old_runtime = self.active_runtime.clone();
            let old_graph = old_runtime.as_ref().map(|runtime| runtime.graph.clone());
            let root_dir = root_directory_for_source(&source).map_err(|err| {
                self.record_compile_or_load_error(stage, Some(root_name.clone()), err)
            })?;
            let affected_paths = compute_affected_paths(&source, old_graph.as_ref(), dirty_paths)
                .map_err(|err| {
                self.record_compile_or_load_error(stage, Some(root_name.clone()), err)
            })?;
            let affected_count = affected_paths.len();
            self.reload_metrics.last_dirty_path_count = dirty_count;
            self.reload_metrics.last_affected_path_count = affected_count;
            self.reload_metrics.last_rebuilt_module_count = 0;
            info!(
                "luau_runtime event=reload_scheduled kind={} root={} dirty_paths={} affected_paths={}",
                kind.as_str(),
                root_name,
                dirty_count,
                affected_count
            );

            let build_dispatch =
                BuildDispatch::new(source.clone(), old_graph.clone(), root_dir, affected_paths);

            let (root_key, mut build_dispatch) =
                self.with_build_dispatch(build_dispatch, |runtime| {
                    let root_key = module_key_for_source(&source);
                    build_module_candidate(&runtime.lua, &runtime.require_mode, &root_key)
                        .map_err(|err| {
                            runtime.record_compile_or_load_lua_failure(
                                stage,
                                Some(root_key.display_name(root_inline_name(&source).as_deref())),
                                err,
                            )
                        })?;
                    runtime.apply_candidate_hooks(stage)?;
                    Ok(root_key)
                })?;

            let rebuilt_count = build_dispatch.candidates.len();
            let candidate_keys: BTreeSet<_> = build_dispatch.candidates.keys().cloned().collect();
            let new_graph = self
                .assemble_graph(
                    &root_key,
                    &root_name,
                    old_graph.as_ref(),
                    &mut build_dispatch,
                )
                .map_err(|err| {
                    self.record_compile_or_load_lua_failure(stage, Some(root_name.clone()), err)
                })?;
            let active_module_count = new_graph.modules.len();
            let old_modules_to_shutdown =
                collect_old_modules_to_shutdown(old_graph.as_ref(), &candidate_keys, &new_graph);
            let prepared_runtime = prepare_runtime_snapshot(new_graph).map_err(|err| {
                self.record_compile_or_load_lua_failure(stage, Some(root_name.clone()), err)
            })?;

            self.config.root_source = Some(source);
            self.last_compile_error = None;
            self.last_runtime_error = None;
            self.failure_metrics.last_failure = None;

            Ok((
                StagedRuntime {
                    runtime: prepared_runtime,
                    source: self
                        .config
                        .root_source
                        .clone()
                        .expect("root source set for staged runtime"),
                    kind,
                    old_modules_to_shutdown,
                    old_require_snapshot: old_runtime
                        .as_ref()
                        .map(|runtime| Arc::clone(&runtime.require_snapshot)),
                },
                rebuilt_count,
                active_module_count,
            ))
        })();

        self.reload_metrics.attempts += 1;
        self.reload_metrics.last_kind = Some(kind);
        self.reload_metrics.last_duration = Some(started_at.elapsed());

        match result {
            Ok((staged, rebuilt_count, active_module_count)) => {
                self.reload_metrics.successes += 1;
                self.reload_metrics.last_rebuilt_module_count = rebuilt_count;
                self.reload_metrics.last_active_module_count = active_module_count;
                info!(
                    "luau_runtime event=reload_succeeded kind={} root={} rebuilt_modules={} active_modules={} duration_ms={}",
                    kind.as_str(),
                    root_name,
                    rebuilt_count,
                    active_module_count,
                    self.reload_metrics
                        .last_duration
                        .expect("reload duration set")
                        .as_millis()
                );
                Ok(staged)
            }
            Err(err) => {
                self.reload_metrics.failures += 1;
                self.reload_metrics.last_rebuilt_module_count = 0;
                self.reload_metrics.last_active_module_count = self
                    .active_runtime
                    .as_ref()
                    .map(|runtime| runtime.graph.modules.len())
                    .unwrap_or(0);
                warn!(
                    "luau_runtime event=reload_failed kind={} root={} duration_ms={} error={}",
                    kind.as_str(),
                    root_name,
                    self.reload_metrics
                        .last_duration
                        .expect("reload duration set")
                        .as_millis(),
                    err
                );
                Err(err)
            }
        }
    }

    fn apply_candidate_hooks(
        &mut self,
        stage: super::RuntimeFailureStage,
    ) -> Result<(), RuntimeError> {
        let (build_order, inline_name) = {
            let mode = self.require_mode.lock();
            match &*mode {
                RequireMode::Build(build) => (
                    build.build_order.clone(),
                    root_inline_name(&build.root_source),
                ),
                _ => unreachable!("candidate hooks require build mode"),
            }
        };

        for key in build_order {
            let candidate = {
                let mode = self.require_mode.lock();
                match &*mode {
                    RequireMode::Build(build) => build
                        .candidates
                        .get(&key)
                        .cloned()
                        .expect("build order referenced missing candidate"),
                    _ => unreachable!("candidate hooks require build mode"),
                }
            };

            let old_exports = {
                let mode = self.require_mode.lock();
                match &*mode {
                    RequireMode::Build(build) => build
                        .old_graph
                        .as_ref()
                        .and_then(|graph| graph.modules.get(&key))
                        .map(|module| module.exports.clone()),
                    _ => unreachable!("candidate hooks require build mode"),
                }
            };

            if old_exports.is_some() {
                call_optional_hook(
                    &candidate.exports,
                    "reload",
                    (old_exports.unwrap(), candidate.state.clone()),
                )
                .map_err(|err| {
                    self.record_init_reload_lua_failure(
                        stage,
                        candidate.key.display_name(inline_name.as_deref()),
                        "reload",
                        err,
                    )
                })?;
            } else {
                call_optional_hook(&candidate.exports, "init", candidate.state.clone()).map_err(
                    |err| {
                        self.record_init_reload_lua_failure(
                            stage,
                            candidate.key.display_name(inline_name.as_deref()),
                            "init",
                            err,
                        )
                    },
                )?;
            }
        }

        Ok(())
    }

    fn assemble_graph(
        &mut self,
        root_key: &ModuleKey,
        root_context: &str,
        old_graph: Option<&ActiveGraph>,
        build_dispatch: &mut BuildDispatch,
    ) -> mlua::Result<ActiveGraph> {
        let mut modules = old_graph
            .map(|graph| graph.modules.clone())
            .unwrap_or_default();

        for candidate in build_dispatch.candidates.values() {
            let id = old_graph
                .and_then(|graph| graph.modules.get(&candidate.key))
                .map(|module| module.id)
                .unwrap_or_else(|| {
                    let id = self.next_module_id;
                    self.next_module_id += 1;
                    id
                });

            let render = if &candidate.key == root_key {
                Some(require_function(&candidate.exports, "render")?)
            } else {
                None
            };

            modules.insert(
                candidate.key.clone(),
                ActiveModule {
                    id,
                    key: candidate.key.clone(),
                    exports: candidate.exports.clone(),
                    render,
                    state: candidate.state.clone(),
                    dependencies: candidate.dependencies.clone(),
                    reverse_deps: BTreeSet::new(),
                    source_hash: candidate.source_hash,
                    version: candidate.version,
                },
            );
        }

        let reachable = collect_reachable_modules(root_key, &modules);
        modules.retain(|key, _| reachable.contains(key));

        for module in modules.values_mut() {
            module.reverse_deps.clear();
        }

        let edge_list: Vec<_> = modules
            .iter()
            .map(|(key, module)| (key.clone(), module.dependencies.clone()))
            .collect();
        for (module_key, dependencies) in edge_list {
            for dependency_key in dependencies {
                if let Some(dependency) = modules.get_mut(&dependency_key) {
                    dependency.reverse_deps.insert(module_key.clone());
                }
            }
        }

        let mut path_index = HashMap::new();
        for module in modules.values() {
            if let Some(path) = module.key.path() {
                path_index.insert(path.to_path_buf(), module.id);
            }
        }

        Ok(ActiveGraph {
            root: root_key.clone(),
            root_context: root_context.to_owned(),
            modules,
            path_index,
        })
    }

    fn with_build_dispatch<T>(
        &mut self,
        build_dispatch: BuildDispatch,
        f: impl FnOnce(&mut Self) -> Result<T, RuntimeError>,
    ) -> Result<(T, BuildDispatch), RuntimeError> {
        let previous = {
            let mut mode = self.require_mode.lock();
            std::mem::replace(&mut *mode, RequireMode::Build(build_dispatch))
        };

        let result = f(self);

        let current = {
            let mut mode = self.require_mode.lock();
            std::mem::replace(&mut *mode, previous)
        };

        match current {
            RequireMode::Build(build_dispatch) => result.map(|value| (value, build_dispatch)),
            _ => unreachable!("require mode changed unexpectedly during build"),
        }
    }

    pub(super) fn commit_staged_candidate(
        &mut self,
        stage: super::RuntimeFailureStage,
    ) -> Result<(), RuntimeError> {
        let Some(staged) = self.pending_candidate.take() else {
            return Err(RuntimeError::config(
                "no staged runtime candidate is available for commit",
            ));
        };

        // Commit swaps the active runtime selection first. Any later shutdown failure happens
        // after the new runtime is live, so rollback is not transactional past this boundary.
        self.active_runtime = Some(staged.runtime);
        self.promote_bridge_generation();

        if staged.old_modules_to_shutdown.is_empty() {
            return Ok(());
        }

        let Some(old_require_snapshot) = staged.old_require_snapshot else {
            return Ok(());
        };

        let previous = {
            let mut mode = self.require_mode.lock();
            std::mem::replace(&mut *mode, RequireMode::Active(old_require_snapshot))
        };

        let shutdown_result = (|| -> Result<(), RuntimeError> {
            let inline_name = root_inline_name(&staged.source);
            for module in &staged.old_modules_to_shutdown {
                call_optional_hook(&module.exports, "shutdown", module.state.clone()).map_err(
                    |err| {
                        self.record_init_reload_lua_failure(
                            stage,
                            module.key.display_name(inline_name.as_deref()),
                            "shutdown",
                            err,
                        )
                    },
                )?;
            }
            Ok(())
        })();

        let current = {
            let mut mode = self.require_mode.lock();
            std::mem::replace(&mut *mode, previous)
        };
        match current {
            RequireMode::Active(_) => {}
            _ => unreachable!("require mode changed unexpectedly during shutdown"),
        }

        shutdown_result
    }
}

fn script_source_display_name(source: &ScriptSource) -> String {
    match source {
        ScriptSource::Inline { name, .. } => name.clone(),
        ScriptSource::Path(path) => path.display().to_string(),
    }
}

fn panic_payload_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_owned()
    }
}

fn prepare_runtime_snapshot(graph: ActiveGraph) -> mlua::Result<CommittedActiveRuntime> {
    let root_module = graph
        .root_module()
        .ok_or_else(|| runtime_lua_error("active runtime graph is missing the root module"))?;
    let root_update = optional_function(&root_module.exports, "update")?;
    let root_render = root_module
        .render
        .clone()
        .ok_or_else(|| runtime_lua_error("root module does not expose render(state)"))?;
    let root_state = root_module.state.clone();
    let require_snapshot = Arc::new(ActiveRequireSnapshot {
        root_dir: graph.root_dir(),
        exports_by_key: graph
            .modules
            .iter()
            .map(|(key, module)| (key.clone(), module.exports.clone()))
            .collect(),
    });

    Ok(CommittedActiveRuntime {
        graph,
        root_update,
        root_render,
        root_state,
        require_snapshot,
    })
}

fn build_module_candidate(
    lua: &Lua,
    require_mode: &Arc<Mutex<RequireMode>>,
    key: &ModuleKey,
) -> mlua::Result<Table> {
    let existing = {
        let mut mode = require_mode.lock();
        match &mut *mode {
            RequireMode::Build(build) => {
                if let Some(candidate) = build.candidates.get(key) {
                    return Ok(candidate.exports.clone());
                }
                if build.loading.contains(key) {
                    return Err(runtime_lua_error(format!(
                        "cyclic require detected while loading {}",
                        key.display_name(root_inline_name(&build.root_source).as_deref())
                    )));
                }
                if let Some(path) = key.path() {
                    if !build.affected_paths.contains(path) {
                        if let Some(old_graph) = &build.old_graph {
                            if let Some(module) = old_graph.modules.get(key) {
                                // Unaffected modules are reused by handle. Active and staged
                                // graphs still point into the same Lua heap, so this is runtime
                                // selection rollback, not full VM-state isolation.
                                return Ok(module.exports.clone());
                            }
                        }
                    }
                }
                build.loading.push(key.clone());
                build.root_source.clone()
            }
            RequireMode::Active(active) => {
                return active.exports_by_key.get(key).cloned().ok_or_else(|| {
                    runtime_lua_error(format!("module {} is not active", key.display_name(None)))
                });
            }
            RequireMode::Disabled => {
                return Err(runtime_lua_error(
                    "require is not available during this runtime call",
                ));
            }
        }
    };

    let result = (|| -> mlua::Result<CandidateModule> {
        let (source_name, source_text) = read_module_source(&existing, key)?;
        let dependencies = Arc::new(Mutex::new(BTreeSet::new()));
        let environment =
            create_module_environment(lua, require_mode, key.clone(), Arc::clone(&dependencies))?;
        let exports = load_exports(lua, &source_name, &source_text, Some(environment))?;
        let state = previous_module_state(lua, require_mode, key)?;
        let version = previous_module_version(require_mode, key) + 1;
        let dependencies = dependencies.lock().clone();
        Ok(CandidateModule {
            key: key.clone(),
            exports,
            state,
            dependencies,
            source_hash: source_hash(&source_text),
            version,
        })
    })();

    let mut mode = require_mode.lock();
    match &mut *mode {
        RequireMode::Build(build) => {
            build.loading.retain(|loading_key| loading_key != key);
            match result {
                Ok(candidate) => {
                    let exports = candidate.exports.clone();
                    build.candidates.insert(key.clone(), candidate);
                    build.build_order.push(key.clone());
                    Ok(exports)
                }
                Err(err) => Err(err),
            }
        }
        _ => unreachable!("module candidates can only be built while building"),
    }
}

fn create_module_environment(
    lua: &Lua,
    require_mode: &Arc<Mutex<RequireMode>>,
    importer: ModuleKey,
    dependencies: Arc<Mutex<BTreeSet<ModuleKey>>>,
) -> mlua::Result<Table> {
    let env = lua.create_table()?;
    let mt = lua.create_table()?;
    mt.set("__index", lua.globals())?;
    env.set_metatable(Some(mt))?;

    let require_mode = Arc::clone(require_mode);
    let importer_key = importer.clone();
    env.set(
        "require",
        lua.create_function(move |lua, spec: String| {
            match catch_unwind(AssertUnwindSafe(|| {
                let target = resolve_require_target(&require_mode, &importer_key, &spec)?;
                dependencies.lock().insert(target.clone());
                build_module_candidate(lua, &require_mode, &target)
            })) {
                Ok(result) => result,
                Err(payload) => Err(runtime_lua_error(format!(
                    "require callback panic while loading from {}: {}",
                    importer_key.display_name(None),
                    panic_payload_message(payload),
                ))),
            }
        })?,
    )?;

    Ok(env)
}

fn resolve_require_target(
    require_mode: &Arc<Mutex<RequireMode>>,
    importer: &ModuleKey,
    spec: &str,
) -> mlua::Result<ModuleKey> {
    let root_dir = {
        let mode = require_mode.lock();
        match &*mode {
            RequireMode::Build(build) => build.root_dir.clone(),
            RequireMode::Active(active) => active.root_dir.clone(),
            RequireMode::Disabled => None,
        }
    };

    let root_dir = root_dir.ok_or_else(|| {
        runtime_lua_error(
            "require is only available for file-backed projects with a root directory",
        )
    })?;

    resolve_require_path(&root_dir, importer, spec)
        .map(ModuleKey::Path)
        .map_err(LuaError::external)
}

fn previous_module_state(
    lua: &Lua,
    require_mode: &Arc<Mutex<RequireMode>>,
    key: &ModuleKey,
) -> mlua::Result<Table> {
    let old_state = {
        let mode = require_mode.lock();
        match &*mode {
            RequireMode::Build(build) => build
                .old_graph
                .as_ref()
                .and_then(|graph| graph.modules.get(key))
                .map(|module| module.state.clone()),
            _ => None,
        }
    };

    match old_state {
        // Rebuilt modules receive cloned plain-data state. Unsupported Lua values fail reload
        // instead of being shared or mutated in place.
        Some(state) => clone_state_table(lua, &state),
        None => lua.create_table(),
    }
}

fn previous_module_version(require_mode: &Arc<Mutex<RequireMode>>, key: &ModuleKey) -> u64 {
    let mode = require_mode.lock();
    match &*mode {
        RequireMode::Build(build) => build
            .old_graph
            .as_ref()
            .and_then(|graph| graph.modules.get(key))
            .map(|module| module.version)
            .unwrap_or(0),
        _ => 0,
    }
}

fn collect_old_modules_to_shutdown(
    old_graph: Option<&ActiveGraph>,
    candidate_keys: &BTreeSet<ModuleKey>,
    new_graph: &ActiveGraph,
) -> Vec<ActiveModule> {
    let Some(old_graph) = old_graph else {
        return Vec::new();
    };

    old_graph
        .modules
        .iter()
        .filter(|(key, _)| candidate_keys.contains(*key) || !new_graph.modules.contains_key(*key))
        .map(|(_, module)| module.clone())
        .collect()
}

fn collect_reachable_modules(
    root_key: &ModuleKey,
    modules: &HashMap<ModuleKey, ActiveModule>,
) -> BTreeSet<ModuleKey> {
    let mut reachable = BTreeSet::new();
    let mut stack = vec![root_key.clone()];

    while let Some(key) = stack.pop() {
        if !reachable.insert(key.clone()) {
            continue;
        }

        if let Some(module) = modules.get(&key) {
            for dependency in &module.dependencies {
                stack.push(dependency.clone());
            }
        }
    }

    reachable
}

fn module_key_for_source(source: &ScriptSource) -> ModuleKey {
    match source {
        ScriptSource::Inline { .. } => ModuleKey::RootInline,
        ScriptSource::Path(path) => ModuleKey::Path(path.clone()),
    }
}

fn root_directory_for_source(source: &ScriptSource) -> Result<Option<PathBuf>, RuntimeError> {
    match source {
        ScriptSource::Inline { .. } => Ok(None),
        ScriptSource::Path(path) => Ok(Some(script_project_root(path)?)),
    }
}

fn script_project_root(path: &Path) -> Result<PathBuf, RuntimeError> {
    let root_parent = path.parent().ok_or_else(|| {
        RuntimeError::config(format!(
            "root script path `{}` has no parent directory",
            path.display()
        ))
    })?;

    let mut current = Some(root_parent);
    while let Some(candidate) = current {
        if candidate.join(".luaurc").is_file() {
            return Ok(candidate.to_path_buf());
        }
        current = candidate.parent();
    }

    Ok(root_parent.to_path_buf())
}

fn root_inline_name(source: &ScriptSource) -> Option<String> {
    match source {
        ScriptSource::Inline { name, .. } => Some(name.clone()),
        ScriptSource::Path(_) => None,
    }
}

fn compute_affected_paths(
    source: &ScriptSource,
    old_graph: Option<&ActiveGraph>,
    dirty_paths: Vec<PathBuf>,
) -> Result<BTreeSet<PathBuf>, RuntimeError> {
    let ScriptSource::Path(root_path) = source else {
        return Ok(BTreeSet::new());
    };

    let root_path = normalize_script_path(root_path)?;

    let Some(old_graph) = old_graph else {
        let mut affected = BTreeSet::new();
        affected.insert(root_path);
        return Ok(affected);
    };

    if dirty_paths.is_empty() {
        let mut affected = BTreeSet::new();
        affected.extend(
            old_graph
                .modules
                .keys()
                .filter_map(ModuleKey::path)
                .map(Path::to_path_buf),
        );
        return Ok(affected);
    }

    let root_dir = script_project_root(&root_path)?;

    let mut affected = BTreeSet::new();
    let mut stack = Vec::new();
    for dirty_path in dirty_paths {
        let normalized = normalize_script_path(&dirty_path)?;
        if !normalized.starts_with(&root_dir) {
            continue;
        }
        if normalized == root_path || old_graph.path_index.contains_key(&normalized) {
            if affected.insert(normalized.clone()) {
                stack.push(ModuleKey::Path(normalized));
            }
        }
    }

    if affected.is_empty() {
        return Ok(affected);
    }

    while let Some(module_key) = stack.pop() {
        if let Some(module) = old_graph.modules.get(&module_key) {
            for reverse_dep in &module.reverse_deps {
                if let Some(path) = reverse_dep.path() {
                    let path = path.to_path_buf();
                    if affected.insert(path.clone()) {
                        stack.push(reverse_dep.clone());
                    }
                }
            }
        }
    }

    affected.insert(root_path);
    Ok(affected)
}

fn normalize_script_source(source: ScriptSource) -> Result<ScriptSource, RuntimeError> {
    match source {
        ScriptSource::Inline { .. } => Ok(source),
        ScriptSource::Path(path) => Ok(ScriptSource::Path(normalize_script_path(&path)?)),
    }
}

fn normalize_script_path(path: &Path) -> Result<PathBuf, RuntimeError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|err| RuntimeError::io(format!("failed to resolve current directory: {err}")))?
            .join(path)
    };
    let lexical = normalize_lexical_path(&absolute);
    match fs::canonicalize(&lexical) {
        Ok(canonical) => Ok(canonical),
        Err(_) => Ok(lexical),
    }
}

fn normalize_lexical_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

fn resolve_require_path(
    root_dir: &Path,
    importer: &ModuleKey,
    spec: &str,
) -> Result<PathBuf, RuntimeError> {
    if spec.is_empty() {
        return Err(RuntimeError::backend(
            "require path must not be empty".to_owned(),
        ));
    }

    let raw = Path::new(spec);
    if raw.is_absolute() {
        return Err(RuntimeError::backend(format!(
            "require path `{spec}` must be relative to the script project root"
        )));
    }

    let base_dir = if spec.starts_with("./") || spec.starts_with("../") {
        importer
            .path()
            .and_then(Path::parent)
            .ok_or_else(|| {
                RuntimeError::backend(
                    "relative require is not available from an inline root module".to_owned(),
                )
            })?
            .to_path_buf()
    } else {
        root_dir.to_path_buf()
    };

    let mut candidate = normalize_lexical_path(&base_dir.join(raw));
    if candidate.extension().is_none() {
        candidate.set_extension("luau");
    }
    let candidate = normalize_script_path(&candidate)?;

    if !candidate.starts_with(root_dir) {
        return Err(RuntimeError::backend(format!(
            "require path `{spec}` resolved outside the allowed script project root"
        )));
    }

    Ok(candidate)
}

fn read_module_source(
    source: &ScriptSource,
    key: &ModuleKey,
) -> Result<(String, String), LuaError> {
    match key {
        ModuleKey::RootInline => match source {
            ScriptSource::Inline { name, source } => Ok((name.clone(), source.clone())),
            ScriptSource::Path(_) => Err(runtime_lua_error("inline root source is not available")),
        },
        ModuleKey::Path(path) => fs::read_to_string(path)
            .map(|contents| (path.display().to_string(), contents))
            .map_err(|err| {
                LuaError::external(RuntimeError::io(format!(
                    "failed to read {}: {}",
                    path.display(),
                    err
                )))
            }),
    }
}

fn load_exports(
    lua: &Lua,
    source_name: &str,
    source_text: &str,
    environment: Option<Table>,
) -> mlua::Result<Table> {
    let chunk = lua.load(source_text).set_name(source_name);
    let chunk = if let Some(environment) = environment {
        chunk.set_environment(environment)
    } else {
        chunk
    };
    chunk.call::<Table>(())
}

fn require_function(exports: &Table, name: &str) -> mlua::Result<Function> {
    match exports.raw_get::<Value>(name)? {
        Value::Function(function) => Ok(function),
        Value::Nil => Err(runtime_lua_error(format!(
            "required hook `{name}` is missing"
        ))),
        other => Err(runtime_lua_error(format!(
            "required hook `{name}` must be a function, got {other:?}"
        ))),
    }
}

fn optional_function(exports: &Table, name: &str) -> mlua::Result<Option<Function>> {
    match exports.raw_get::<Value>(name)? {
        Value::Nil => Ok(None),
        Value::Function(function) => Ok(Some(function)),
        other => Err(runtime_lua_error(format!(
            "hook `{name}` must be a function when present, got {other:?}"
        ))),
    }
}

fn call_optional_hook<A>(exports: &Table, name: &str, args: A) -> mlua::Result<()>
where
    A: mlua::IntoLuaMulti,
{
    match exports.raw_get::<Value>(name)? {
        Value::Nil => Ok(()),
        Value::Function(function) => function.call::<()>(args),
        other => Err(runtime_lua_error(format!(
            "hook `{name}` must be a function when present, got {other:?}"
        ))),
    }
}

fn clone_state_table(lua: &Lua, state: &Table) -> mlua::Result<Table> {
    let mut memo = HashMap::new();
    clone_table_inner(lua, state, &mut memo)
}

fn clone_table_inner(
    lua: &Lua,
    table: &Table,
    memo: &mut HashMap<usize, Table>,
) -> mlua::Result<Table> {
    let key = Value::Table(table.clone()).to_pointer() as usize;
    if let Some(existing) = memo.get(&key) {
        return Ok(existing.clone());
    }

    let cloned = lua.create_table()?;
    memo.insert(key, cloned.clone());

    for pair in table.pairs::<Value, Value>() {
        let (raw_key, raw_value) = pair?;
        let cloned_key = clone_value(lua, raw_key, memo)?;
        let cloned_value = clone_value(lua, raw_value, memo)?;
        cloned.raw_set(cloned_key, cloned_value)?;
    }

    Ok(cloned)
}

fn clone_value(lua: &Lua, value: Value, memo: &mut HashMap<usize, Table>) -> mlua::Result<Value> {
    match value {
        Value::Nil => Ok(Value::Nil),
        Value::Boolean(value) => Ok(Value::Boolean(value)),
        Value::Integer(value) => Ok(Value::Integer(value)),
        Value::Number(value) => Ok(Value::Number(value)),
        Value::String(value) => Ok(Value::String(value)),
        Value::Table(table) => Ok(Value::Table(clone_table_inner(lua, &table, memo)?)),
        other => Err(runtime_lua_error(format!(
            "state contains unsupported value type {other:?}"
        ))),
    }
}
