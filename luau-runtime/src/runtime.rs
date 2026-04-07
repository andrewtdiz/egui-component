use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    error::Error,
    fmt, fs,
    hash::{Hash, Hasher},
    marker::PhantomData,
    path::{Component, Path, PathBuf},
    ptr::NonNull,
    sync::Arc,
    time::{Duration, Instant},
};

use log::{info, warn};
use mlua::{Error as LuaError, Function, Lua, MultiValue, Table, Value};
use parking_lot::Mutex;

/// Configures the runtime before any script is loaded.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeConfig {
    /// Optional root script source that a future load step can consume.
    pub root_source: Option<ScriptSource>,
}

/// A Luau script source, either inline text or a file-backed path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptSource {
    /// Source code provided directly by the host.
    Inline { name: String, source: String },
    /// Source code loaded from disk.
    Path(PathBuf),
}

impl ScriptSource {
    /// Create an inline script source.
    pub fn inline(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self::Inline {
            name: name.into(),
            source: source.into(),
        }
    }

    /// Create a file-backed script source.
    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }
}

/// Thread-safe queue for file paths that need reload attention.
#[derive(Debug, Clone, Default)]
pub struct ReloadQueue {
    pending: Arc<Mutex<VecDeque<PathBuf>>>,
}

impl ReloadQueue {
    /// Create an empty reload queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueue a path for later reload handling.
    pub fn push(&self, path: impl Into<PathBuf>) {
        self.pending.lock().push_back(path.into());
    }

    /// Drain all queued paths in FIFO order.
    pub fn drain(&self) -> Vec<PathBuf> {
        self.pending.lock().drain(..).collect()
    }

    /// Return `true` when no paths are pending.
    pub fn is_empty(&self) -> bool {
        self.pending.lock().is_empty()
    }

    /// Return the number of queued paths.
    pub fn len(&self) -> usize {
        self.pending.lock().len()
    }
}

/// Error type used by the runtime facade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// Invalid configuration supplied by the host.
    Config(String),
    /// Filesystem or provider-level failure.
    Io(String),
    /// Luau backend failure surfaced as a string so the backend stays private.
    Backend(String),
}

impl RuntimeError {
    /// Create a configuration error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Create an IO error.
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io(message.into())
    }

    /// Create a backend error.
    pub fn backend(message: impl Into<String>) -> Self {
        Self::Backend(message.into())
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(message) => write!(f, "config error: {message}"),
            Self::Io(message) => write!(f, "io error: {message}"),
            Self::Backend(message) => write!(f, "backend error: {message}"),
        }
    }
}

impl Error for RuntimeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLabelTone {
    #[default]
    Primary,
    Secondary,
    Muted,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLabelWeight {
    #[default]
    Regular,
    Semibold,
    Bold,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiLabelOptions {
    pub tone: UiLabelTone,
    pub weight: UiLabelWeight,
    pub size: Option<f32>,
}

impl Default for UiLabelOptions {
    fn default() -> Self {
        Self {
            tone: UiLabelTone::Primary,
            weight: UiLabelWeight::Regular,
            size: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiControlSize {
    Sm,
    #[default]
    Md,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiButtonOptions {
    pub variant: UiButtonVariant,
    pub size: UiControlSize,
    pub width: Option<f32>,
}

impl Default for UiButtonOptions {
    fn default() -> Self {
        Self {
            variant: UiButtonVariant::Primary,
            size: UiControlSize::Md,
            width: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiTextEditOptions {
    pub width: Option<f32>,
    pub placeholder: Option<String>,
    pub password: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTextEditOutput {
    pub value: String,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiContainerOptions {
    pub gap: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiCardOptions {
    pub width: Option<f32>,
    pub padding_x: f32,
    pub padding_y: f32,
}

impl Default for UiCardOptions {
    fn default() -> Self {
        Self {
            width: None,
            padding_x: 12.0,
            padding_y: 12.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiWindowOptions {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

/// Backend-agnostic host API used by script-facing `app.*` and `ui.*` bindings.
pub trait RuntimeHost {
    fn log(&mut self, level: &str, message: &str) -> Result<(), RuntimeError>;
    fn request_reload(&mut self) -> Result<(), RuntimeError>;
    fn request_repaint(&mut self) -> Result<(), RuntimeError>;
    fn label(&mut self, text: &str, options: UiLabelOptions) -> Result<(), RuntimeError>;
    fn separator(&mut self) -> Result<(), RuntimeError>;
    fn button(
        &mut self,
        id: &str,
        text: &str,
        options: UiButtonOptions,
    ) -> Result<bool, RuntimeError>;
    fn text_edit(
        &mut self,
        id: &str,
        value: &str,
        options: UiTextEditOptions,
    ) -> Result<UiTextEditOutput, RuntimeError>;
    fn horizontal(
        &mut self,
        options: UiContainerOptions,
        body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError>;
    fn vertical(
        &mut self,
        options: UiContainerOptions,
        body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError>;
    fn card(
        &mut self,
        options: UiCardOptions,
        body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError>;
    fn window(
        &mut self,
        id: &str,
        title: &str,
        options: UiWindowOptions,
        body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError>;
}

/// Coarse reload metrics for the runtime hot path.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReloadMetrics {
    pub attempts: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_kind: Option<ReloadKind>,
    pub last_dirty_path_count: usize,
    pub last_affected_path_count: usize,
    pub last_rebuilt_module_count: usize,
    pub last_active_module_count: usize,
    pub last_duration: Option<Duration>,
}

type ModuleId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorBucket {
    Compile,
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadKind {
    Load,
    Reload,
}

impl ReloadKind {
    fn as_str(self) -> &'static str {
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
struct ActiveModule {
    id: ModuleId,
    key: ModuleKey,
    exports: Table,
    render: Option<Function>,
    state: Table,
    dependencies: BTreeSet<ModuleKey>,
    reverse_deps: BTreeSet<ModuleKey>,
    #[allow(dead_code)]
    source_hash: u64,
    version: u64,
}

#[derive(Debug, Clone)]
struct ActiveGraph {
    root: ModuleKey,
    modules: HashMap<ModuleKey, ActiveModule>,
    path_index: HashMap<PathBuf, ModuleId>,
}

impl ActiveGraph {
    fn root_module(&self) -> Option<&ActiveModule> {
        self.modules.get(&self.root)
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
struct BuildDispatch {
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
struct ActiveRequireSnapshot {
    root_dir: Option<PathBuf>,
    exports_by_key: HashMap<ModuleKey, Table>,
}

#[derive(Debug, Clone)]
struct CommittedActiveRuntime {
    graph: ActiveGraph,
    root_render: Function,
    root_state: Table,
    require_snapshot: Arc<ActiveRequireSnapshot>,
}

#[derive(Debug)]
enum RequireMode {
    Disabled,
    Build(BuildDispatch),
    Active(Arc<ActiveRequireSnapshot>),
}

#[derive(Clone, Copy)]
struct ScopedRuntimeHostBridge<'host> {
    host: NonNull<dyn RuntimeHost + 'host>,
    marker: PhantomData<&'host mut dyn RuntimeHost>,
}

impl<'host> ScopedRuntimeHostBridge<'host> {
    fn new(host: &'host mut dyn RuntimeHost) -> Self {
        Self {
            host: NonNull::from(host),
            marker: PhantomData,
        }
    }

    fn with_host<R>(
        &self,
        f: impl FnOnce(&mut dyn RuntimeHost) -> Result<R, RuntimeError>,
    ) -> mlua::Result<R> {
        let mut host = self.host;
        unsafe { f(host.as_mut()).map_err(LuaError::external) }
    }
}

struct ScopedHostGlobals {
    globals: Table,
    previous_app: Value,
    previous_ui: Value,
}

impl ScopedHostGlobals {
    fn install(globals: Table, app: Table, ui: Table) -> mlua::Result<Self> {
        let previous_app = globals.get::<Value>("app")?;
        let previous_ui = globals.get::<Value>("ui")?;
        globals.set("app", app)?;
        globals.set("ui", ui)?;
        Ok(Self {
            globals,
            previous_app,
            previous_ui,
        })
    }
}

impl Drop for ScopedHostGlobals {
    fn drop(&mut self) {
        let _ = self.globals.set("app", self.previous_app.clone());
        let _ = self.globals.set("ui", self.previous_ui.clone());
    }
}

struct NullRuntimeHost;

impl RuntimeHost for NullRuntimeHost {
    fn log(&mut self, _level: &str, _message: &str) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("app.log"))
    }

    fn request_reload(&mut self) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("app.request_reload"))
    }

    fn request_repaint(&mut self) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("app.request_repaint"))
    }

    fn label(&mut self, _text: &str, _options: UiLabelOptions) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.label"))
    }

    fn separator(&mut self) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.separator"))
    }

    fn button(
        &mut self,
        _id: &str,
        _text: &str,
        _options: UiButtonOptions,
    ) -> Result<bool, RuntimeError> {
        Err(unavailable_host_api("ui.button"))
    }

    fn text_edit(
        &mut self,
        _id: &str,
        _value: &str,
        _options: UiTextEditOptions,
    ) -> Result<UiTextEditOutput, RuntimeError> {
        Err(unavailable_host_api("ui.text_edit"))
    }

    fn horizontal(
        &mut self,
        _options: UiContainerOptions,
        _body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.horizontal"))
    }

    fn vertical(
        &mut self,
        _options: UiContainerOptions,
        _body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.vertical"))
    }

    fn card(
        &mut self,
        _options: UiCardOptions,
        _body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.card"))
    }

    fn window(
        &mut self,
        _id: &str,
        _title: &str,
        _options: UiWindowOptions,
        _body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.window"))
    }
}

/// Main runtime facade that owns the Luau VM and reload queue.
pub struct ScriptRuntime {
    config: RuntimeConfig,
    lua: Lua,
    reload_queue: ReloadQueue,
    active_runtime: Option<CommittedActiveRuntime>,
    next_module_id: ModuleId,
    require_mode: Arc<Mutex<RequireMode>>,
    last_compile_error: Option<String>,
    last_runtime_error: Option<String>,
    reload_metrics: ReloadMetrics,
}

impl ScriptRuntime {
    /// Construct a runtime with a fresh Luau VM.
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            lua: Lua::new(),
            reload_queue: ReloadQueue::new(),
            active_runtime: None,
            next_module_id: 1,
            require_mode: Arc::new(Mutex::new(RequireMode::Disabled)),
            last_compile_error: None,
            last_runtime_error: None,
            reload_metrics: ReloadMetrics::default(),
        }
    }

    /// Access the runtime configuration.
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Get a shareable handle to the reload queue.
    pub fn reload_queue(&self) -> ReloadQueue {
        self.reload_queue.clone()
    }

    /// The last compile error recorded by the runtime, if any.
    pub fn last_compile_error(&self) -> Option<&str> {
        self.last_compile_error.as_deref()
    }

    /// The last runtime error recorded by the runtime, if any.
    pub fn last_runtime_error(&self) -> Option<&str> {
        self.last_runtime_error.as_deref()
    }

    /// Read-only reload metrics for the current runtime instance.
    pub fn reload_metrics(&self) -> &ReloadMetrics {
        &self.reload_metrics
    }

    /// Load a root module from the provided source without a host API.
    pub fn load_root(&mut self, source: ScriptSource) -> Result<(), RuntimeError> {
        let mut host = NullRuntimeHost;
        self.load_root_with_host(source, &mut host)
    }

    /// Load a root module from the provided source with an active host API.
    pub fn load_root_with_host(
        &mut self,
        source: ScriptSource,
        host: &mut dyn RuntimeHost,
    ) -> Result<(), RuntimeError> {
        self.with_host_scope(host, move |runtime| runtime.load_root_inner(source))
    }

    /// Reload the most recently loaded root source without a host API.
    pub fn reload_now(&mut self) -> Result<(), RuntimeError> {
        let mut host = NullRuntimeHost;
        self.reload_now_with_host(&mut host)
    }

    /// Reload the most recently loaded root source with an active host API.
    pub fn reload_now_with_host(&mut self, host: &mut dyn RuntimeHost) -> Result<(), RuntimeError> {
        self.with_host_scope(host, |runtime| {
            let source = runtime
                .config
                .root_source
                .clone()
                .ok_or_else(|| RuntimeError::config("no root source is loaded"))?;
            runtime.reload_root_inner(source)
        })
    }

    /// Render the active root for one frame without a host API.
    pub fn render_frame(&mut self) -> Result<(), RuntimeError> {
        let mut host = NullRuntimeHost;
        self.render_frame_with_host(&mut host)
    }

    /// Render the active root for one frame with an active host API.
    pub fn render_frame_with_host(
        &mut self,
        host: &mut dyn RuntimeHost,
    ) -> Result<(), RuntimeError> {
        self.with_host_scope(host, |runtime| runtime.render_frame_inner())
    }

    fn with_host_scope<T>(
        &mut self,
        host: &mut dyn RuntimeHost,
        f: impl FnOnce(&mut Self) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        let lua = self.lua.clone();
        let mut result = None;

        lua.scope(|scope| {
            let bridge = ScopedRuntimeHostBridge::new(host);
            let globals = lua.globals();
            let app = lua.create_table()?;
            let ui = lua.create_table()?;

            let bridge = bridge;
            app.set(
                "log",
                scope.create_function(move |_, (level, message): (String, String)| {
                    bridge.with_host(|host| host.log(&level, &message))
                })?,
            )?;

            let bridge = bridge;
            app.set(
                "request_reload",
                scope
                    .create_function(move |_, ()| bridge.with_host(|host| host.request_reload()))?,
            )?;

            let bridge = bridge;
            app.set(
                "request_repaint",
                scope.create_function(move |_, ()| {
                    bridge.with_host(|host| host.request_repaint())
                })?,
            )?;

            let bridge = bridge;
            ui.set(
                "label",
                scope.create_function(move |_, (text, props): (String, Option<Table>)| {
                    let props = parse_label_options(props)?;
                    bridge.with_host(|host| host.label(&text, props))
                })?,
            )?;

            let bridge = bridge;
            ui.set(
                "separator",
                scope.create_function(move |_, ()| bridge.with_host(|host| host.separator()))?,
            )?;

            let bridge = bridge;
            ui.set(
                "button",
                scope.create_function(move |_, args: MultiValue| {
                    let (id, text, props) = parse_button_call(args)?;
                    bridge.with_host(|host| host.button(&id, &text, props))
                })?,
            )?;

            let bridge = bridge;
            ui.set(
                "text_edit",
                scope.create_function(move |_, args: MultiValue| {
                    let (id, value, props) = parse_text_edit_call(args)?;
                    let output = bridge.with_host(|host| host.text_edit(&id, &value, props))?;
                    Ok((output.value, output.changed))
                })?,
            )?;

            let bridge = bridge;
            ui.set(
                "horizontal",
                scope.create_function(move |_, args: MultiValue| {
                    let (props, callback) = parse_container_call("ui.horizontal", args)?;
                    bridge.with_host(|host| {
                        let mut callback = || {
                            callback
                                .call::<()>(())
                                .map_err(runtime_error_from_lua_error)
                        };
                        host.horizontal(props, &mut callback)
                    })
                })?,
            )?;

            let bridge = bridge;
            ui.set(
                "vertical",
                scope.create_function(move |_, args: MultiValue| {
                    let (props, callback) = parse_container_call("ui.vertical", args)?;
                    bridge.with_host(|host| {
                        let mut callback = || {
                            callback
                                .call::<()>(())
                                .map_err(runtime_error_from_lua_error)
                        };
                        host.vertical(props, &mut callback)
                    })
                })?,
            )?;

            let bridge = bridge;
            ui.set(
                "card",
                scope.create_function(move |_, args: MultiValue| {
                    let (props, callback) = parse_card_call(args)?;
                    bridge.with_host(|host| {
                        let mut callback = || {
                            callback
                                .call::<()>(())
                                .map_err(runtime_error_from_lua_error)
                        };
                        host.card(props, &mut callback)
                    })
                })?,
            )?;

            let bridge = bridge;
            ui.set(
                "window",
                scope.create_function(move |_, args: MultiValue| {
                    let (id, title, props, callback) = parse_window_call(args)?;
                    bridge.with_host(|host| {
                        let mut callback = || {
                            callback
                                .call::<()>(())
                                .map_err(runtime_error_from_lua_error)
                        };
                        host.window(&id, &title, props, &mut callback)
                    })
                })?,
            )?;

            let _globals = ScopedHostGlobals::install(globals, app, ui)?;
            result = Some(f(self));
            Ok(())
        })
        .map_err(runtime_error_from_lua_error)?;

        result.expect("runtime host scope exited without producing a result")
    }

    fn load_root_inner(&mut self, source: ScriptSource) -> Result<(), RuntimeError> {
        let source = normalize_script_source(source)?;
        self.rebuild_graph(source, Vec::new(), ReloadKind::Load)
    }

    fn reload_root_inner(&mut self, source: ScriptSource) -> Result<(), RuntimeError> {
        let dirty_paths = self.reload_queue.drain();
        self.rebuild_graph(source, dirty_paths, ReloadKind::Reload)
    }

    fn render_frame_inner(&mut self) -> Result<(), RuntimeError> {
        let active_runtime = self
            .active_runtime
            .clone()
            .ok_or_else(|| RuntimeError::config("no root script has been loaded"))?;
        let render = active_runtime.root_render.clone();
        let state = active_runtime.root_state.clone();
        let previous = {
            let mut mode = self.require_mode.lock();
            std::mem::replace(
                &mut *mode,
                RequireMode::Active(Arc::clone(&active_runtime.require_snapshot)),
            )
        };
        let render_result = render.call::<()>(state);
        let current = {
            let mut mode = self.require_mode.lock();
            std::mem::replace(&mut *mode, previous)
        };
        match current {
            RequireMode::Active(_) => {}
            _ => unreachable!("require mode changed unexpectedly during render"),
        }

        render_result.map_err(|err| self.record_lua_error(err))?;
        self.last_runtime_error = None;
        Ok(())
    }

    fn rebuild_graph(
        &mut self,
        source: ScriptSource,
        dirty_paths: Vec<PathBuf>,
        kind: ReloadKind,
    ) -> Result<(), RuntimeError> {
        let root_name = script_source_display_name(&source);
        let dirty_count = dirty_paths.len();
        let started_at = Instant::now();

        let result = (|| -> Result<(usize, usize), RuntimeError> {
            let old_runtime = self.active_runtime.clone();
            let old_graph = old_runtime.as_ref().map(|runtime| runtime.graph.clone());
            let root_dir = root_directory_for_source(&source)?;
            let affected_paths = compute_affected_paths(&source, old_graph.as_ref(), dirty_paths)?;
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
                        .map_err(|err| runtime.record_lua_error(err))?;
                    runtime.apply_candidate_hooks()?;
                    Ok(root_key)
                })?;

            let rebuilt_count = build_dispatch.candidates.len();
            let candidate_keys: BTreeSet<_> = build_dispatch.candidates.keys().cloned().collect();
            let new_graph = self
                .assemble_graph(&root_key, old_graph.as_ref(), &mut build_dispatch)
                .map_err(|err| self.record_lua_error(err))?;
            let active_module_count = new_graph.modules.len();
            let old_modules_to_shutdown =
                collect_old_modules_to_shutdown(old_graph.as_ref(), &candidate_keys, &new_graph);
            let committed_runtime =
                commit_active_runtime(new_graph).map_err(|err| self.record_lua_error(err))?;

            self.active_runtime = Some(committed_runtime);
            self.config.root_source = Some(source);
            self.last_compile_error = None;
            self.last_runtime_error = None;

            if let Some(old_runtime) = old_runtime.as_ref() {
                let previous = {
                    let mut mode = self.require_mode.lock();
                    std::mem::replace(
                        &mut *mode,
                        RequireMode::Active(Arc::clone(&old_runtime.require_snapshot)),
                    )
                };
                let shutdown_result = (|| -> mlua::Result<()> {
                    for module in &old_modules_to_shutdown {
                        call_optional_hook(&module.exports, "shutdown", module.state.clone())?;
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
                shutdown_result.map_err(|err| self.record_lua_error(err))?;
            }

            Ok((rebuilt_count, active_module_count))
        })();

        self.reload_metrics.attempts += 1;
        self.reload_metrics.last_kind = Some(kind);
        self.reload_metrics.last_duration = Some(started_at.elapsed());

        match result {
            Ok((rebuilt_count, active_module_count)) => {
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
                Ok(())
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

    fn apply_candidate_hooks(&mut self) -> Result<(), RuntimeError> {
        let build_order = {
            let mode = self.require_mode.lock();
            match &*mode {
                RequireMode::Build(build) => build.build_order.clone(),
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
                .map_err(|err| self.record_lua_error(err))?;
            } else {
                call_optional_hook(&candidate.exports, "init", candidate.state.clone())
                    .map_err(|err| self.record_lua_error(err))?;
            }
        }

        Ok(())
    }

    fn assemble_graph(
        &mut self,
        root_key: &ModuleKey,
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

    fn record_lua_error(&mut self, err: LuaError) -> RuntimeError {
        match bucket_lua_error(&err) {
            ErrorBucket::Compile => {
                let message = format_lua_error(err);
                self.last_compile_error = Some(message.clone());
                RuntimeError::backend(message)
            }
            ErrorBucket::Runtime => {
                let message = format_lua_error(err);
                self.last_runtime_error = Some(message.clone());
                RuntimeError::backend(message)
            }
        }
    }
}

impl fmt::Debug for ScriptRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScriptRuntime")
            .field("config", &self.config)
            .field("reload_queue_len", &self.reload_queue.len())
            .field(
                "module_count",
                &self
                    .active_runtime
                    .as_ref()
                    .map(|runtime| runtime.graph.modules.len()),
            )
            .field("last_compile_error", &self.last_compile_error)
            .field("last_runtime_error", &self.last_runtime_error)
            .field("reload_metrics", &self.reload_metrics)
            .finish()
    }
}

fn script_source_display_name(source: &ScriptSource) -> String {
    match source {
        ScriptSource::Inline { name, .. } => name.clone(),
        ScriptSource::Path(path) => path.display().to_string(),
    }
}

fn commit_active_runtime(graph: ActiveGraph) -> mlua::Result<CommittedActiveRuntime> {
    let root_render = graph
        .root_module()
        .ok_or_else(|| runtime_lua_error("active runtime graph is missing the root module"))?
        .render
        .clone()
        .ok_or_else(|| runtime_lua_error("root module does not expose render(state)"))?;
    let root_state = graph
        .root_module()
        .expect("root module checked above")
        .state
        .clone();
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
            let target = resolve_require_target(&require_mode, &importer_key, &spec)?;
            dependencies.lock().insert(target.clone());
            build_module_candidate(lua, &require_mode, &target)
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
        ScriptSource::Path(path) => Ok(Some(
            path.parent()
                .ok_or_else(|| {
                    RuntimeError::config(format!(
                        "root script path `{}` has no parent directory",
                        path.display()
                    ))
                })?
                .to_path_buf(),
        )),
    }
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

    let root_dir = root_path.parent().ok_or_else(|| {
        RuntimeError::config(format!(
            "root script path `{}` has no parent directory",
            root_path.display()
        ))
    })?;

    let mut affected = BTreeSet::new();
    let mut stack = Vec::new();
    for dirty_path in dirty_paths {
        let normalized = normalize_script_path(&dirty_path)?;
        if !normalized.starts_with(root_dir) {
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
            "require path `{spec}` resolved outside the allowed script directory"
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

fn parse_label_options(table: Option<Table>) -> mlua::Result<UiLabelOptions> {
    let Some(table) = table else {
        return Ok(UiLabelOptions::default());
    };

    let mut options = UiLabelOptions::default();
    if let Some(tone) = raw_string_field(&table, "tone")? {
        options.tone = parse_label_tone(&tone)?;
    }
    if let Some(weight) = raw_string_field(&table, "weight")? {
        options.weight = parse_label_weight(&weight)?;
    }
    options.size = raw_number_field(&table, "size")?;
    Ok(options)
}

fn parse_button_call(args: MultiValue) -> mlua::Result<(String, String, UiButtonOptions)> {
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), "ui.button", 1)?;
    let text = expect_string_arg(args.next(), "ui.button", 2)?;
    let options = parse_button_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "ui.button expected an options table or nil as the third argument, got {other:?}"
            )))
        }
    })?;
    ensure_no_extra_args("ui.button", args)?;
    Ok((id, text, options))
}

fn parse_button_options(table: Option<Table>) -> mlua::Result<UiButtonOptions> {
    let Some(table) = table else {
        return Ok(UiButtonOptions::default());
    };

    let mut options = UiButtonOptions::default();
    if let Some(variant) = raw_string_field(&table, "variant")? {
        options.variant = parse_button_variant(&variant)?;
    }
    if let Some(size) = raw_string_field(&table, "size")? {
        options.size = parse_control_size(&size)?;
    }
    options.width = raw_number_field(&table, "width")?;
    Ok(options)
}

fn parse_text_edit_call(args: MultiValue) -> mlua::Result<(String, String, UiTextEditOptions)> {
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), "ui.text_edit", 1)?;
    let value = expect_string_arg(args.next(), "ui.text_edit", 2)?;
    let options = parse_text_edit_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "ui.text_edit expected an options table or nil as the third argument, got {other:?}"
            )))
        }
    })?;
    ensure_no_extra_args("ui.text_edit", args)?;
    Ok((id, value, options))
}

fn parse_text_edit_options(table: Option<Table>) -> mlua::Result<UiTextEditOptions> {
    let Some(table) = table else {
        return Ok(UiTextEditOptions::default());
    };

    Ok(UiTextEditOptions {
        width: raw_number_field(&table, "width")?,
        placeholder: raw_string_field(&table, "placeholder")?,
        password: raw_bool_field(&table, "password")?.unwrap_or(false),
    })
}

fn parse_container_call(
    fn_name: &str,
    args: MultiValue,
) -> mlua::Result<(UiContainerOptions, Function)> {
    let mut args = args.into_vec().into_iter();
    let first = args.next();
    let second = args.next();

    let (options, callback) = match (first, second) {
        (Some(Value::Function(callback)), None) => (UiContainerOptions::default(), callback),
        (Some(Value::Table(table)), Some(Value::Function(callback))) => {
            (parse_container_options(Some(table))?, callback)
        }
        (Some(Value::Nil), Some(Value::Function(callback))) => {
            (UiContainerOptions::default(), callback)
        }
        (Some(other), maybe_callback) => {
            let callback_desc = match maybe_callback {
                Some(value) => format!("{value:?}"),
                None => "missing".to_owned(),
            };
            return Err(runtime_lua_error(format!(
                "{fn_name} expected `(callback)` or `(options, callback)`, got first={other:?}, second={callback_desc}"
            )));
        }
        (None, _) => {
            return Err(runtime_lua_error(format!(
                "{fn_name} expected `(callback)` or `(options, callback)`"
            )))
        }
    };

    ensure_no_extra_args(fn_name, args)?;
    Ok((options, callback))
}

fn parse_container_options(table: Option<Table>) -> mlua::Result<UiContainerOptions> {
    let Some(table) = table else {
        return Ok(UiContainerOptions::default());
    };

    Ok(UiContainerOptions {
        gap: raw_number_field(&table, "gap")?,
    })
}

fn parse_card_call(args: MultiValue) -> mlua::Result<(UiCardOptions, Function)> {
    let (first, second, extras) = split_args(args, "ui.card")?;
    let (options, callback) = match (first, second) {
        (Some(Value::Function(callback)), None) => (UiCardOptions::default(), callback),
        (Some(Value::Table(table)), Some(Value::Function(callback))) => {
            (parse_card_options(Some(table))?, callback)
        }
        (Some(Value::Nil), Some(Value::Function(callback))) => {
            (UiCardOptions::default(), callback)
        }
        (Some(other), maybe_callback) => {
            let callback_desc = match maybe_callback {
                Some(value) => format!("{value:?}"),
                None => "missing".to_owned(),
            };
            return Err(runtime_lua_error(format!(
                "ui.card expected `(callback)` or `(options, callback)`, got first={other:?}, second={callback_desc}"
            )));
        }
        (None, _) => return Err(runtime_lua_error("ui.card expected a callback")),
    };
    ensure_no_extra_args("ui.card", extras.into_iter())?;
    Ok((options, callback))
}

fn parse_card_options(table: Option<Table>) -> mlua::Result<UiCardOptions> {
    let Some(table) = table else {
        return Ok(UiCardOptions::default());
    };

    let mut options = UiCardOptions::default();
    options.width = raw_number_field(&table, "width")?;
    if let Some(padding_x) = raw_number_field(&table, "padding_x")? {
        options.padding_x = padding_x;
    }
    if let Some(padding_y) = raw_number_field(&table, "padding_y")? {
        options.padding_y = padding_y;
    }
    Ok(options)
}

fn parse_window_call(
    args: MultiValue,
) -> mlua::Result<(String, String, UiWindowOptions, Function)> {
    let args = args.into_vec();
    match args.as_slice() {
        [Value::String(id), Value::String(title), Value::Function(callback)] => Ok((
            id.to_str()?.to_owned(),
            title.to_str()?.to_owned(),
            UiWindowOptions::default(),
            callback.clone(),
        )),
        [Value::String(id), Value::String(title), Value::Table(options), Value::Function(callback)] => Ok((
            id.to_str()?.to_owned(),
            title.to_str()?.to_owned(),
            parse_window_options(Some(options.clone()))?,
            callback.clone(),
        )),
        [Value::String(id), Value::String(title), Value::Nil, Value::Function(callback)] => Ok((
            id.to_str()?.to_owned(),
            title.to_str()?.to_owned(),
            UiWindowOptions::default(),
            callback.clone(),
        )),
        _ => Err(runtime_lua_error(
            "ui.window expected `(id, title, callback)` or `(id, title, options, callback)`",
        )),
    }
}

fn parse_window_options(table: Option<Table>) -> mlua::Result<UiWindowOptions> {
    let Some(table) = table else {
        return Ok(UiWindowOptions::default());
    };

    Ok(UiWindowOptions {
        width: raw_number_field(&table, "width")?,
        height: raw_number_field(&table, "height")?,
    })
}

fn split_args(
    args: MultiValue,
    _fn_name: &str,
) -> mlua::Result<(Option<Value>, Option<Value>, Vec<Value>)> {
    let mut args = args.into_vec().into_iter();
    Ok((args.next(), args.next(), args.collect()))
}

fn ensure_no_extra_args(
    fn_name: &str,
    mut extras: impl Iterator<Item = Value>,
) -> mlua::Result<()> {
    if let Some(extra) = extras.next() {
        return Err(runtime_lua_error(format!(
            "{fn_name} received an unexpected extra argument: {extra:?}"
        )));
    }
    Ok(())
}

fn expect_string_arg(value: Option<Value>, fn_name: &str, index: usize) -> mlua::Result<String> {
    match value {
        Some(Value::String(value)) => Ok(value.to_str()?.to_owned()),
        Some(other) => Err(runtime_lua_error(format!(
            "{fn_name} expected argument {index} to be a string, got {other:?}"
        ))),
        None => Err(runtime_lua_error(format!(
            "{fn_name} is missing required argument {index}"
        ))),
    }
}

fn raw_string_field(table: &Table, key: &str) -> mlua::Result<Option<String>> {
    match table.raw_get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::String(value) => Ok(Some(value.to_str()?.to_owned())),
        other => Err(runtime_lua_error(format!(
            "field `{key}` must be a string when present, got {other:?}"
        ))),
    }
}

fn raw_number_field(table: &Table, key: &str) -> mlua::Result<Option<f32>> {
    match table.raw_get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::Integer(value) => Ok(Some(value as f32)),
        Value::Number(value) => Ok(Some(value as f32)),
        other => Err(runtime_lua_error(format!(
            "field `{key}` must be a number when present, got {other:?}"
        ))),
    }
}

fn raw_bool_field(table: &Table, key: &str) -> mlua::Result<Option<bool>> {
    match table.raw_get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::Boolean(value) => Ok(Some(value)),
        other => Err(runtime_lua_error(format!(
            "field `{key}` must be a boolean when present, got {other:?}"
        ))),
    }
}

fn parse_label_tone(value: &str) -> mlua::Result<UiLabelTone> {
    match value.to_ascii_lowercase().as_str() {
        "primary" => Ok(UiLabelTone::Primary),
        "secondary" => Ok(UiLabelTone::Secondary),
        "muted" => Ok(UiLabelTone::Muted),
        "destructive" => Ok(UiLabelTone::Destructive),
        _ => Err(runtime_lua_error(format!("unknown label tone `{value}`"))),
    }
}

fn parse_label_weight(value: &str) -> mlua::Result<UiLabelWeight> {
    match value.to_ascii_lowercase().as_str() {
        "regular" => Ok(UiLabelWeight::Regular),
        "semibold" => Ok(UiLabelWeight::Semibold),
        "bold" => Ok(UiLabelWeight::Bold),
        _ => Err(runtime_lua_error(format!("unknown label weight `{value}`"))),
    }
}

fn parse_button_variant(value: &str) -> mlua::Result<UiButtonVariant> {
    match value.to_ascii_lowercase().as_str() {
        "primary" => Ok(UiButtonVariant::Primary),
        "secondary" => Ok(UiButtonVariant::Secondary),
        "ghost" => Ok(UiButtonVariant::Ghost),
        "link" => Ok(UiButtonVariant::Link),
        _ => Err(runtime_lua_error(format!("unknown button variant `{value}`"))),
    }
}

fn parse_control_size(value: &str) -> mlua::Result<UiControlSize> {
    match value.to_ascii_lowercase().as_str() {
        "sm" | "small" => Ok(UiControlSize::Sm),
        "md" | "medium" => Ok(UiControlSize::Md),
        _ => Err(runtime_lua_error(format!("unknown control size `{value}`"))),
    }
}

fn source_hash(source_text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source_text.hash(&mut hasher);
    hasher.finish()
}

fn unavailable_host_api(name: &str) -> RuntimeError {
    RuntimeError::backend(format!(
        "host API `{name}` is not available during this runtime call"
    ))
}

fn runtime_lua_error(message: impl Into<String>) -> LuaError {
    LuaError::external(RuntimeError::backend(message.into()))
}

fn bucket_lua_error(err: &LuaError) -> ErrorBucket {
    match err {
        LuaError::SyntaxError { .. } => ErrorBucket::Compile,
        LuaError::CallbackError { cause, .. } => bucket_lua_error(cause.as_ref()),
        _ => ErrorBucket::Runtime,
    }
}

fn format_lua_error(err: LuaError) -> String {
    match err {
        LuaError::SyntaxError { message, .. } => message,
        LuaError::CallbackError { cause, .. } => format_lua_error((*cause).clone()),
        other => other.to_string(),
    }
}

fn runtime_error_from_lua_error(err: LuaError) -> RuntimeError {
    RuntimeError::backend(format_lua_error(err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[derive(Debug, Default)]
    struct RecordingHost {
        events: Vec<String>,
        logs: Vec<(String, String)>,
        reload_requests: usize,
        repaint_requests: usize,
        labels: Vec<(String, UiLabelOptions)>,
        buttons: Vec<(String, String, UiButtonOptions)>,
        text_edits: Vec<(String, String, UiTextEditOptions)>,
        button_results: BTreeMap<String, bool>,
        text_edit_results: BTreeMap<String, UiTextEditOutput>,
        container_events: Vec<String>,
    }

    impl RuntimeHost for RecordingHost {
        fn log(&mut self, level: &str, message: &str) -> Result<(), RuntimeError> {
            self.events.push(format!("log:{level}:{message}"));
            self.logs.push((level.to_owned(), message.to_owned()));
            Ok(())
        }

        fn request_reload(&mut self) -> Result<(), RuntimeError> {
            self.events.push("request_reload".to_owned());
            self.reload_requests += 1;
            Ok(())
        }

        fn request_repaint(&mut self) -> Result<(), RuntimeError> {
            self.events.push("request_repaint".to_owned());
            self.repaint_requests += 1;
            Ok(())
        }

        fn label(&mut self, text: &str, options: UiLabelOptions) -> Result<(), RuntimeError> {
            self.events.push(format!("label:{text}"));
            self.labels.push((text.to_owned(), options));
            Ok(())
        }

        fn separator(&mut self) -> Result<(), RuntimeError> {
            self.events.push("separator".to_owned());
            Ok(())
        }

        fn button(
            &mut self,
            id: &str,
            text: &str,
            options: UiButtonOptions,
        ) -> Result<bool, RuntimeError> {
            self.events.push(format!("button:{id}:{text}"));
            self.buttons
                .push((id.to_owned(), text.to_owned(), options));
            Ok(self.button_results.remove(id).unwrap_or(false))
        }

        fn text_edit(
            &mut self,
            id: &str,
            value: &str,
            options: UiTextEditOptions,
        ) -> Result<UiTextEditOutput, RuntimeError> {
            self.events.push(format!("text_edit:{id}"));
            self.text_edits
                .push((id.to_owned(), value.to_owned(), options));
            Ok(self.text_edit_results.remove(id).unwrap_or(UiTextEditOutput {
                value: value.to_owned(),
                changed: false,
            }))
        }

        fn horizontal(
            &mut self,
            options: UiContainerOptions,
            body: &mut dyn FnMut() -> Result<(), RuntimeError>,
        ) -> Result<(), RuntimeError> {
            self.container_events
                .push(format!("horizontal:start:{:?}", options.gap));
            body()?;
            self.container_events.push("horizontal:end".to_owned());
            Ok(())
        }

        fn vertical(
            &mut self,
            options: UiContainerOptions,
            body: &mut dyn FnMut() -> Result<(), RuntimeError>,
        ) -> Result<(), RuntimeError> {
            self.container_events
                .push(format!("vertical:start:{:?}", options.gap));
            body()?;
            self.container_events.push("vertical:end".to_owned());
            Ok(())
        }

        fn card(
            &mut self,
            options: UiCardOptions,
            body: &mut dyn FnMut() -> Result<(), RuntimeError>,
        ) -> Result<(), RuntimeError> {
            self.container_events.push(format!(
                "card:start:{:?}:{:.1}:{:.1}",
                options.width, options.padding_x, options.padding_y
            ));
            body()?;
            self.container_events.push("card:end".to_owned());
            Ok(())
        }

        fn window(
            &mut self,
            id: &str,
            title: &str,
            options: UiWindowOptions,
            body: &mut dyn FnMut() -> Result<(), RuntimeError>,
        ) -> Result<(), RuntimeError> {
            self.container_events.push(format!(
                "window:start:{id}:{title}:{:?}:{:?}",
                options.width, options.height
            ));
            body()?;
            self.container_events.push("window:end".to_owned());
            Ok(())
        }
    }

    #[test]
    fn constructs_runtime_and_processes_reload_queue() {
        let runtime = ScriptRuntime::new(RuntimeConfig::default());

        assert_eq!(runtime.config(), &RuntimeConfig::default());
        assert_eq!(runtime.last_compile_error(), None);
        assert_eq!(runtime.last_runtime_error(), None);

        let queue = runtime.reload_queue();
        queue.push("scripts/root.luau");
        assert_eq!(queue.drain(), vec![PathBuf::from("scripts/root.luau")]);
        assert!(queue.is_empty());
    }

    #[test]
    fn persistent_state_survives_multiple_frames() {
        let source = ScriptSource::inline(
            "root.luau",
            r#"
                local module = {}

                function module.init(state)
                    if state.count == nil then
                        state.count = 0
                    end
                end

                function module.render(state)
                    if state.count == nil then
                        state.count = 0
                    end
                    state.count = state.count + 1
                end

                return module
            "#,
        );

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        runtime.load_root(source).unwrap();
        runtime.render_frame().unwrap();
        runtime.render_frame().unwrap();

        assert_eq!(runtime.state_i64("count"), Some(2));
        assert_eq!(runtime.last_compile_error(), None);
        assert_eq!(runtime.last_runtime_error(), None);
    }

    #[test]
    fn reload_failure_keeps_last_known_good_version() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            r#"
                local module = {{}}

                function module.render(state)
                    state.count = (state.count or 0) + 1
                end

                return module
            "#
        )
        .unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path().to_path_buf();
        let source = ScriptSource::path(path.clone());
        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());

        runtime.load_root(source).unwrap();
        runtime.render_frame().unwrap();
        assert_eq!(runtime.state_i64("count"), Some(1));

        fs::write(
            &path,
            r#"
                local module = {
            "#,
        )
        .unwrap();

        runtime.reload_queue().push(path);
        let reload_result = runtime.reload_now();
        assert!(reload_result.is_err());
        assert!(runtime.last_compile_error().is_some());

        runtime.render_frame().unwrap();
        assert_eq!(runtime.state_i64("count"), Some(2));
        assert_eq!(runtime.last_runtime_error(), None);
    }

    #[test]
    fn host_bindings_route_app_calls() {
        let source = ScriptSource::inline(
            "host_demo.luau",
            r#"
                local module = {}

                function module.init(state)
                    app.log("info", "init")
                end

                function module.render(state)
                    app.log("debug", "render")
                    app.request_reload()
                    app.request_repaint()
                end

                return module
            "#,
        );

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut host = RecordingHost::default();

        runtime.load_root_with_host(source, &mut host).unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();

        assert_eq!(
            host.logs,
            vec![
                ("info".to_owned(), "init".to_owned()),
                ("debug".to_owned(), "render".to_owned()),
            ]
        );
        assert_eq!(host.reload_requests, 1);
        assert_eq!(host.repaint_requests, 1);
    }

    #[test]
    fn direct_ui_bindings_return_immediate_results() {
        let source = ScriptSource::inline(
            "direct_ui_demo.luau",
            r#"
                local module = {}

                function module.render(state)
                    state.count = state.count or 0
                    state.name = state.name or "Luau"

                    ui.label("Runtime Showcase", {
                        tone = "Muted",
                        size = 12,
                    })

                    if ui.button("demo.increment", "Increment", {
                        variant = "Primary",
                        width = 180,
                    }) then
                        state.count = state.count + 1
                    end

                    local next_value, changed = ui.text_edit("demo.name", state.name, {
                        placeholder = "Enter a name",
                        width = 240,
                    })
                    if changed then
                        state.name = next_value
                    end

                    app.log("info", state.name)
                end

                return module
            "#,
        );

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut host = RecordingHost::default();
        host.button_results
            .insert("demo.increment".to_owned(), true);
        host.text_edit_results.insert(
            "demo.name".to_owned(),
            UiTextEditOutput {
                value: "Andy".to_owned(),
                changed: true,
            },
        );

        runtime.load_root_with_host(source, &mut host).unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();

        assert!(host.events.contains(&"label:Runtime Showcase".to_owned()));
        assert!(host
            .events
            .contains(&"button:demo.increment:Increment".to_owned()));
        assert!(host.events.contains(&"text_edit:demo.name".to_owned()));
        assert_eq!(
            host.logs,
            vec![("info".to_owned(), "Andy".to_owned())]
        );
        assert_eq!(runtime.state_i64("count"), Some(1));
        assert_eq!(runtime.state_string("name"), Some("Andy".to_owned()));
    }

    #[test]
    fn ui_namespace_is_unavailable_without_host() {
        let source = ScriptSource::inline(
            "ui_requires_host_demo.luau",
            r#"
                local module = {}

                function module.render(state)
                    ui.label("legacy")
                end

                return module
            "#,
        );

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        runtime.load_root(source).unwrap();
        let render = runtime.render_frame();

        assert!(render.is_err());
        assert!(runtime.last_runtime_error().is_some());
    }

    #[test]
    fn host_scopes_do_not_leak_between_frames() {
        let source = ScriptSource::inline(
            "host_scope_demo.luau",
            r#"
                local module = {}

                function module.render(state)
                    app.log("info", "render")
                end

                return module
            "#,
        );

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut first_host = RecordingHost::default();
        let mut second_host = RecordingHost::default();

        runtime
            .load_root_with_host(source, &mut first_host)
            .unwrap();
        runtime.render_frame_with_host(&mut first_host).unwrap();
        runtime.render_frame_with_host(&mut second_host).unwrap();

        assert_eq!(
            first_host.logs,
            vec![("info".to_owned(), "render".to_owned())]
        );
        assert_eq!(
            second_host.logs,
            vec![("info".to_owned(), "render".to_owned())]
        );
    }

    #[test]
    fn host_globals_restore_after_runtime_error() {
        let source = ScriptSource::inline(
            "callback_error_demo.luau",
            r#"
                local module = {}

                function module.render(state)
                    if not state.failed_once then
                        state.failed_once = true
                        error("boom")
                    end

                    app.log("info", "recovered")
                end

                return module
            "#,
        );

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut failing_host = RecordingHost::default();

        runtime
            .load_root_with_host(source, &mut failing_host)
            .unwrap();
        let first_render = runtime.render_frame_with_host(&mut failing_host);
        assert!(first_render.is_err());

        let mut recovered_host = RecordingHost::default();
        runtime.render_frame_with_host(&mut recovered_host).unwrap();

        assert_eq!(
            recovered_host.logs,
            vec![("info".to_owned(), "recovered".to_owned())]
        );
        assert_eq!(runtime.last_runtime_error(), None);
    }

    #[test]
    fn nested_container_callbacks_execute_in_order() {
        let source = ScriptSource::inline(
            "nested_container_demo.luau",
            r#"
                local module = {}

                function module.render(state)
                    ui.vertical({ gap = 16 }, function()
                        ui.label("outer")
                        ui.card({ width = 280, padding_x = 18, padding_y = 20 }, function()
                            ui.horizontal({ gap = 8 }, function()
                                ui.label("inner")
                            end)
                        end)
                    end)
                end

                return module
            "#,
        );

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut host = RecordingHost::default();

        runtime.load_root_with_host(source, &mut host).unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();

        assert_eq!(
            host.container_events,
            vec![
                "vertical:start:Some(16.0)".to_owned(),
                "card:start:Some(280.0):18.0:20.0".to_owned(),
                "horizontal:start:Some(8.0)".to_owned(),
                "horizontal:end".to_owned(),
                "card:end".to_owned(),
                "vertical:end".to_owned(),
            ]
        );
        assert_eq!(
            host.labels,
            vec![
                ("outer".to_owned(), UiLabelOptions::default()),
                ("inner".to_owned(), UiLabelOptions::default()),
            ]
        );
    }

    #[test]
    fn leaf_module_reload_updates_dependents_and_preserves_module_state() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("demo.luau");
        let leaf_path = temp_dir.path().join("leaf.luau");

        fs::write(
            &leaf_path,
            r#"
                local module = {}
                local module_state = nil

                function module.init(state)
                    module_state = state
                    state.reloads = state.reloads or 0
                end

                function module.reload(old_exports, state)
                    module_state = state
                    state.reloads = (state.reloads or 0) + 1
                    app.log("info", "leaf reloaded")
                end

                function module.message()
                    return "leaf v1 / reloads=" .. tostring(module_state.reloads)
                end

                return module
            "#,
        )
        .unwrap();
        fs::write(
            &root_path,
            r#"
                local leaf = require("./leaf")
                local module = {}

                function module.render(state)
                    app.log("info", leaf.message())
                end

                return module
            "#,
        )
        .unwrap();

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut host = RecordingHost::default();

        runtime
            .load_root_with_host(ScriptSource::path(root_path.clone()), &mut host)
            .unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();
        assert!(host
            .logs
            .iter()
            .any(|(_, message)| message == "leaf v1 / reloads=0"));

        fs::write(
            &leaf_path,
            r#"
                local module = {}
                local module_state = nil

                function module.init(state)
                    module_state = state
                    state.reloads = state.reloads or 0
                end

                function module.reload(old_exports, state)
                    module_state = state
                    state.reloads = (state.reloads or 0) + 1
                    app.log("info", "leaf reloaded")
                end

                function module.message()
                    return "leaf v2 / reloads=" .. tostring(module_state.reloads)
                end

                return module
            "#,
        )
        .unwrap();

        runtime.reload_queue().push(leaf_path);
        runtime.reload_now_with_host(&mut host).unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();

        assert!(host
            .logs
            .iter()
            .any(|(_, message)| message == "leaf reloaded"));
        assert!(host
            .logs
            .iter()
            .any(|(_, message)| message == "leaf v2 / reloads=1"));
    }

    #[test]
    fn broken_leaf_module_keeps_previous_graph_alive() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("demo.luau");
        let leaf_path = temp_dir.path().join("leaf.luau");

        fs::write(
            &leaf_path,
            r#"
                local module = {}

                function module.message()
                    return "leaf stable"
                end

                return module
            "#,
        )
        .unwrap();
        fs::write(
            &root_path,
            r#"
                local leaf = require("./leaf")
                local module = {}

                function module.render(state)
                    app.log("info", leaf.message())
                end

                return module
            "#,
        )
        .unwrap();

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut host = RecordingHost::default();

        runtime
            .load_root_with_host(ScriptSource::path(root_path.clone()), &mut host)
            .unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();
        assert!(host
            .logs
            .iter()
            .any(|(_, message)| message == "leaf stable"));

        fs::write(
            &leaf_path,
            r#"
                local module = {
            "#,
        )
        .unwrap();

        runtime.reload_queue().push(leaf_path);
        let reload = runtime.reload_now_with_host(&mut host);
        assert!(reload.is_err());
        assert!(runtime.last_compile_error().is_some());

        runtime.render_frame_with_host(&mut host).unwrap();
        let stable_logs = host
            .logs
            .iter()
            .filter(|(_, message)| message == "leaf stable")
            .count();
        assert!(stable_logs >= 2);
    }

    #[test]
    fn dependency_reload_hooks_run_in_dependency_order() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("demo.luau");
        let middle_path = temp_dir.path().join("middle.luau");
        let leaf_path = temp_dir.path().join("leaf.luau");

        fs::write(
            &leaf_path,
            r#"
                local module = {}

                function module.message()
                    return "leaf-v1"
                end

                function module.reload(old_exports, state)
                    app.log("info", "leaf reload")
                end

                return module
            "#,
        )
        .unwrap();
        fs::write(
            &middle_path,
            r#"
                local leaf = require("./leaf")
                local module = {}

                function module.message()
                    return "middle -> " .. leaf.message()
                end

                function module.reload(old_exports, state)
                    app.log("info", "middle reload")
                end

                return module
            "#,
        )
        .unwrap();
        fs::write(
            &root_path,
            r#"
                local middle = require("./middle")
                local module = {}

                function module.render(state)
                    app.log("info", middle.message())
                end

                return module
            "#,
        )
        .unwrap();

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut host = RecordingHost::default();

        runtime
            .load_root_with_host(ScriptSource::path(root_path.clone()), &mut host)
            .unwrap();
        host.logs.clear();

        fs::write(
            &leaf_path,
            r#"
                local module = {}

                function module.message()
                    return "leaf-v2"
                end

                function module.reload(old_exports, state)
                    app.log("info", "leaf reload")
                end

                return module
            "#,
        )
        .unwrap();

        runtime.reload_queue().push(leaf_path);
        runtime.reload_now_with_host(&mut host).unwrap();

        let leaf_index = host
            .logs
            .iter()
            .position(|(_, message)| message == "leaf reload")
            .expect("leaf reload hook logged");
        let middle_index = host
            .logs
            .iter()
            .position(|(_, message)| message == "middle reload")
            .expect("middle reload hook logged");
        assert!(leaf_index < middle_index);
    }

    #[test]
    fn reload_metrics_track_success_failure_and_dirty_queue_shape() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("demo.luau");
        let leaf_path = temp_dir.path().join("leaf.luau");

        fs::write(
            &leaf_path,
            r#"
                local module = {}

                function module.message()
                    return "leaf-v1"
                end

                return module
            "#,
        )
        .unwrap();
        fs::write(
            &root_path,
            r#"
                local leaf = require("./leaf")
                local module = {}

                function module.render(state)
                    app.log("info", leaf.message())
                end

                return module
            "#,
        )
        .unwrap();

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut host = RecordingHost::default();

        runtime
            .load_root_with_host(ScriptSource::path(root_path.clone()), &mut host)
            .unwrap();
        assert_eq!(
            runtime.reload_metrics(),
            &ReloadMetrics {
                attempts: 1,
                successes: 1,
                failures: 0,
                last_kind: Some(ReloadKind::Load),
                last_dirty_path_count: 0,
                last_affected_path_count: 1,
                last_rebuilt_module_count: 2,
                last_active_module_count: 2,
                last_duration: runtime.reload_metrics().last_duration,
            }
        );
        assert!(runtime.reload_metrics().last_duration.is_some());

        fs::write(
            &leaf_path,
            r#"
                local module = {}

                function module.message()
                    return "leaf-v2"
                end

                return module
            "#,
        )
        .unwrap();

        runtime.reload_now_with_host(&mut host).unwrap();

        let metrics = runtime.reload_metrics();
        assert_eq!(metrics.attempts, 2);
        assert_eq!(metrics.successes, 2);
        assert_eq!(metrics.failures, 0);
        assert_eq!(metrics.last_kind, Some(ReloadKind::Reload));
        assert_eq!(metrics.last_dirty_path_count, 0);
        assert_eq!(metrics.last_affected_path_count, 2);
        assert_eq!(metrics.last_rebuilt_module_count, 2);
        assert_eq!(metrics.last_active_module_count, 2);
        assert!(metrics.last_duration.is_some());

        fs::write(
            &leaf_path,
            r#"
                local module = {
            "#,
        )
        .unwrap();
        runtime.reload_queue().push(leaf_path);
        assert!(runtime.reload_now_with_host(&mut host).is_err());

        let metrics = runtime.reload_metrics();
        assert_eq!(metrics.attempts, 3);
        assert_eq!(metrics.successes, 2);
        assert_eq!(metrics.failures, 1);
        assert_eq!(metrics.last_kind, Some(ReloadKind::Reload));
        assert_eq!(metrics.last_dirty_path_count, 1);
        assert_eq!(metrics.last_affected_path_count, 2);
        assert_eq!(metrics.last_rebuilt_module_count, 0);
        assert_eq!(metrics.last_active_module_count, 2);
        assert!(metrics.last_duration.is_some());
    }

    #[test]
    fn manual_reload_rebuilds_required_leaf_modules_when_queue_is_empty() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("demo.luau");
        let leaf_path = temp_dir.path().join("leaf.luau");

        fs::write(
            &leaf_path,
            r#"
                local module = {}

                function module.message()
                    return "leaf-v1"
                end

                return module
            "#,
        )
        .unwrap();
        fs::write(
            &root_path,
            r#"
                local leaf = require("./leaf")
                local module = {}

                function module.render(state)
                    app.log("info", leaf.message())
                end

                return module
            "#,
        )
        .unwrap();

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut host = RecordingHost::default();

        runtime
            .load_root_with_host(ScriptSource::path(root_path), &mut host)
            .unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();
        assert!(host.logs.iter().any(|(_, message)| message == "leaf-v1"));

        fs::write(
            &leaf_path,
            r#"
                local module = {}

                function module.message()
                    return "leaf-v2"
                end

                return module
            "#,
        )
        .unwrap();

        host.logs.clear();
        runtime.reload_now_with_host(&mut host).unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();

        assert!(host.logs.iter().any(|(_, message)| message == "leaf-v2"));
        let metrics = runtime.reload_metrics();
        assert_eq!(metrics.last_dirty_path_count, 0);
        assert_eq!(metrics.last_affected_path_count, 2);
        assert_eq!(metrics.last_rebuilt_module_count, 2);
    }

    #[test]
    fn render_time_require_uses_committed_snapshot_across_frames_and_reload() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("demo.luau");
        let leaf_path = temp_dir.path().join("leaf.luau");

        fs::write(
            &leaf_path,
            r#"
                local module = {}

                function module.message()
                    return "leaf-v1"
                end

                return module
            "#,
        )
        .unwrap();
        fs::write(
            &root_path,
            r#"
                local initial_leaf = require("./leaf")
                local module = {}

                function module.render(state)
                    local leaf = require("./leaf")
                    app.log("info", leaf.message())
                    app.log("debug", initial_leaf.message())
                end

                return module
            "#,
        )
        .unwrap();

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut host = RecordingHost::default();

        runtime
            .load_root_with_host(ScriptSource::path(root_path.clone()), &mut host)
            .unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();

        let leaf_v1_logs = host
            .logs
            .iter()
            .filter(|(_, message)| message == "leaf-v1")
            .count();
        assert_eq!(leaf_v1_logs, 4);

        fs::write(
            &leaf_path,
            r#"
                local module = {}

                function module.message()
                    return "leaf-v2"
                end

                return module
            "#,
        )
        .unwrap();

        runtime.reload_queue().push(leaf_path);
        runtime.reload_now_with_host(&mut host).unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();

        assert!(host.logs.iter().any(|(_, message)| message == "leaf-v2"));
    }

    #[test]
    fn shutdown_uses_old_committed_snapshot_after_swap() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("demo.luau");
        let leaf_path = temp_dir.path().join("leaf.luau");
        let shared_path = temp_dir.path().join("shared.luau");

        fs::write(
            &shared_path,
            r#"
                local module = {}

                function module.value()
                    return "old-shared"
                end

                return module
            "#,
        )
        .unwrap();
        fs::write(
            &leaf_path,
            r#"
                local shared = require("./shared")
                local module = {}

                function module.message()
                    return "leaf-v1"
                end

                function module.shutdown(state)
                    local shared = require("./shared")
                    app.log("info", "shutdown:" .. shared.value())
                end

                return module
            "#,
        )
        .unwrap();
        fs::write(
            &root_path,
            r#"
                local initial_leaf = require("./leaf")
                local module = {}

                function module.render(state)
                    local leaf = require("./leaf")
                    app.log("info", leaf.message())
                    app.log("debug", initial_leaf.message())
                end

                return module
            "#,
        )
        .unwrap();

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut host = RecordingHost::default();

        runtime
            .load_root_with_host(ScriptSource::path(root_path.clone()), &mut host)
            .unwrap();
        host.logs.clear();

        fs::write(
            &shared_path,
            r#"
                local module = {}

                function module.value()
                    return "new-shared"
                end

                return module
            "#,
        )
        .unwrap();
        fs::write(
            &leaf_path,
            r#"
                local shared = require("./shared")
                local module = {}

                function module.message()
                    return "leaf-v2"
                end

                function module.shutdown(state)
                    local shared = require("./shared")
                    app.log("info", "shutdown:" .. shared.value())
                end

                return module
            "#,
        )
        .unwrap();

        runtime.reload_queue().push(shared_path);
        runtime.reload_queue().push(leaf_path);
        runtime.reload_now_with_host(&mut host).unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();

        assert!(host
            .logs
            .iter()
            .any(|(_, message)| message == "shutdown:old-shared"));
        assert!(host.logs.iter().any(|(_, message)| message == "leaf-v2"));
        assert!(!host
            .logs
            .iter()
            .any(|(_, message)| message == "shutdown:new-shared"));
    }

    #[test]
    fn unrelated_dirty_luau_file_does_not_reload_active_graph() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("demo.luau");
        let leaf_path = temp_dir.path().join("leaf.luau");
        let scratch_path = temp_dir.path().join("scratch.luau");

        fs::write(
            &leaf_path,
            r#"
                local module = {}

                function module.reload(old_exports, state)
                    app.log("info", "leaf reloaded")
                end

                function module.message()
                    return "leaf stable"
                end

                return module
            "#,
        )
        .unwrap();
        fs::write(
            &root_path,
            r#"
                local leaf = require("./leaf")
                local module = {}

                function module.render(state)
                    app.log("info", leaf.message())
                end

                return module
            "#,
        )
        .unwrap();
        fs::write(
            &scratch_path,
            r#"
                return {
                    note = "scratch"
                }
            "#,
        )
        .unwrap();

        let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
        let mut host = RecordingHost::default();

        runtime
            .load_root_with_host(ScriptSource::path(root_path), &mut host)
            .unwrap();
        host.logs.clear();

        runtime.reload_queue().push(scratch_path);
        runtime.reload_now_with_host(&mut host).unwrap();
        runtime.render_frame_with_host(&mut host).unwrap();

        assert!(!host
            .logs
            .iter()
            .any(|(_, message)| message == "leaf reloaded"));
        assert!(host
            .logs
            .iter()
            .any(|(_, message)| message == "leaf stable"));

        let metrics = runtime.reload_metrics();
        assert_eq!(metrics.last_dirty_path_count, 1);
        assert_eq!(metrics.last_affected_path_count, 0);
        assert_eq!(metrics.last_rebuilt_module_count, 0);
        assert_eq!(metrics.last_active_module_count, 2);
    }

    impl ScriptRuntime {
        fn state_i64(&self, key: &str) -> Option<i64> {
            let root = self.active_runtime.as_ref()?.graph.root_module()?;
            match root.state.raw_get::<Value>(key).ok()? {
                Value::Integer(value) => Some(value),
                Value::Number(value) => Some(value as i64),
                _ => None,
            }
        }

        fn state_string(&self, key: &str) -> Option<String> {
            let root = self.active_runtime.as_ref()?.graph.root_module()?;
            match root.state.raw_get::<Value>(key).ok()? {
                Value::String(value) => Some(value.to_str().ok()?.to_string()),
                _ => None,
            }
        }
    }
}
