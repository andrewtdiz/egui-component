use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::{anyhow, bail, Context as AnyhowContext};
use deno_ast::{
    DecoratorsTranspileOption, EmitOptions, ImportsNotUsedAsValues, JsxAutomaticOptions,
    JsxRuntime, MediaType, ParseParams, SourceMapOption, TranspileModuleOptions, TranspileOptions,
};
use deno_core::{
    extension, op2, resolve_import, JsRuntime, ModuleLoadOptions, ModuleLoadReferrer,
    ModuleLoadResponse, ModuleLoader, ModuleSource, ModuleSourceCode, ModuleSpecifier, ModuleType,
    OpState, ResolutionKind, RuntimeOptions,
};
use deno_error::JsErrorBox;
use egui_component::contract::{
    registry, ContractChildPolicy, ContractEvent, ContractNode, ContractTree,
    CONTRACT_MODEL_VERSION,
};

use super::diagnostics::{push_log, RuntimeLogBuffer};
use super::host_tree::{HostMutationBatch, HostTree};
use super::motion::MotionFrame;

const EGUI_MODULE_SPECIFIER: &str = "egui:mod";
const EGUI_JSX_RUNTIME_SPECIFIER: &str = "egui:jsx-runtime";
const EGUI_JSX_DEV_RUNTIME_SPECIFIER: &str = "egui:jsx-dev-runtime";
const EGUI_MOTION_REACT_SPECIFIER: &str = "egui:motion-react";
const EGUI_MODULE_SOURCE: &str = include_str!("mod.js");
const EGUI_JSX_RUNTIME_SOURCE: &str = include_str!("runtime_api.js");
const EGUI_MOTION_REACT_SOURCE: &str = include_str!("motion_api.js");

type SourceMapStore = Rc<RefCell<HashMap<String, Vec<u8>>>>;

#[derive(Debug, Default)]
struct RuntimeState {
    mutation_batches_json: Vec<String>,
    logs: RuntimeLogBuffer,
}

#[op2(fast)]
fn op_commit_mutations(
    state: &mut OpState,
    #[string] mutations_json: String,
) -> Result<(), JsErrorBox> {
    state
        .borrow_mut::<RuntimeState>()
        .mutation_batches_json
        .push(mutations_json);
    Ok(())
}

#[op2(fast)]
fn op_host_log(
    state: &mut OpState,
    #[string] level: String,
    #[string] message: String,
) -> Result<(), JsErrorBox> {
    let runtime_state = state.borrow_mut::<RuntimeState>();
    push_log(&mut runtime_state.logs, format!("{level}: {message}"));
    Ok(())
}

extension!(
    egui_jsx_host,
    ops = [op_commit_mutations, op_host_log],
    docs = "Ops used by the egui-component JSX/TSX host runtime."
);

#[derive(Debug)]
struct JsxModuleLoader {
    source_maps: SourceMapStore,
}

impl JsxModuleLoader {
    fn new() -> Self {
        Self {
            source_maps: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl ModuleLoader for JsxModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, deno_core::error::ModuleLoaderError> {
        match specifier {
            "egui" => ModuleSpecifier::parse(EGUI_MODULE_SPECIFIER).map_err(JsErrorBox::from_err),
            "egui/jsx-runtime" => {
                ModuleSpecifier::parse(EGUI_JSX_RUNTIME_SPECIFIER).map_err(JsErrorBox::from_err)
            }
            "egui/jsx-dev-runtime" => {
                ModuleSpecifier::parse(EGUI_JSX_DEV_RUNTIME_SPECIFIER).map_err(JsErrorBox::from_err)
            }
            "motion/react" | "react/motion" => {
                ModuleSpecifier::parse(EGUI_MOTION_REACT_SPECIFIER).map_err(JsErrorBox::from_err)
            }
            _ => resolve_import(specifier, referrer).map_err(JsErrorBox::from_err),
        }
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        ModuleLoadResponse::Sync(load_module(Rc::clone(&self.source_maps), module_specifier))
    }

    fn get_source_map(&self, specifier: &str) -> Option<Cow<'_, [u8]>> {
        self.source_maps
            .borrow()
            .get(specifier)
            .map(|source_map| source_map.clone().into())
    }
}

#[derive(Debug)]
pub struct RenderedJsx {
    pub tree: Option<ContractTree>,
    pub motion: MotionFrame,
    pub logs: RuntimeLogBuffer,
}

pub struct JsxRuntimeSession {
    entry_path: PathBuf,
    host_tree: HostTree,
    js_runtime: JsRuntime,
    tokio_runtime: tokio::runtime::Runtime,
}

impl std::fmt::Debug for JsxRuntimeSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JsxRuntimeSession")
            .field("entry_path", &self.entry_path)
            .finish_non_exhaustive()
    }
}

impl JsxRuntimeSession {
    pub fn load(entry_path: &Path) -> anyhow::Result<(Self, RenderedJsx)> {
        let entry_path = absolute_path(entry_path)?;
        let main_module = ModuleSpecifier::from_file_path(&entry_path).map_err(|_| {
            anyhow!(
                "entry path is not a valid file URL: {}",
                entry_path.display()
            )
        })?;

        let mut js_runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(Rc::new(JsxModuleLoader::new())),
            extensions: vec![egui_jsx_host::init()],
            ..Default::default()
        });
        js_runtime
            .op_state()
            .borrow_mut()
            .put(RuntimeState::default());
        install_contract_metadata(&mut js_runtime)?;

        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to build Tokio runtime for deno_core")?;

        let mut session = Self {
            entry_path,
            host_tree: HostTree::default(),
            js_runtime,
            tokio_runtime,
        };
        session.evaluate_main_module(&main_module)?;
        let rendered = session.take_rendered()?;
        Ok((session, rendered))
    }

    pub fn dispatch_events(&mut self, events: &[ContractEvent]) -> anyhow::Result<RenderedJsx> {
        let events_json = serde_json::to_string(events)?;
        let source = format!("globalThis.__eguiDispatchEvents({events_json});");
        self.js_runtime
            .execute_script("[egui:dispatch-events]", source)
            .map_err(|error| anyhow!("failed to dispatch egui events into JSX runtime: {error}"))?;
        self.run_event_loop()?;
        self.take_rendered()
    }

    pub fn tick_motion(&mut self, now_secs: f64) -> anyhow::Result<RenderedJsx> {
        if !self.host_tree.has_root() {
            bail!("{} did not call render(<... />)", self.entry_path.display());
        }
        let tick = self.host_tree.tick_motion(now_secs);
        let _changed = tick.changed;
        Ok(RenderedJsx {
            tree: None,
            motion: tick.frame,
            logs: RuntimeLogBuffer::new(),
        })
    }

    fn evaluate_main_module(&mut self, main_module: &ModuleSpecifier) -> anyhow::Result<()> {
        self.tokio_runtime.block_on(async {
            let module_id = self.js_runtime.load_main_es_module(main_module).await?;
            let result = self.js_runtime.mod_evaluate(module_id);
            self.js_runtime.run_event_loop(Default::default()).await?;
            result.await
        })?;
        Ok(())
    }

    fn run_event_loop(&mut self) -> anyhow::Result<()> {
        self.tokio_runtime
            .block_on(self.js_runtime.run_event_loop(Default::default()))?;
        Ok(())
    }

    fn take_rendered(&mut self) -> anyhow::Result<RenderedJsx> {
        let (mutation_batches_json, logs) = {
            let op_state = self.js_runtime.op_state();
            let mut op_state = op_state.borrow_mut();
            let state = op_state.borrow_mut::<RuntimeState>();
            let mutation_batches_json = std::mem::take(&mut state.mutation_batches_json);
            let logs = std::mem::take(&mut state.logs);
            (mutation_batches_json, logs)
        };

        if mutation_batches_json.is_empty() && !self.host_tree.has_root() {
            bail!("{} did not call render(<... />)", self.entry_path.display());
        }

        let mut tree_changed = false;
        for mutations_json in mutation_batches_json {
            let batch = decode_mutation_batch(&self.entry_path, &mutations_json)?;
            if batch.version != CONTRACT_MODEL_VERSION {
                bail!(
                    "{} returned contract model version {}, but this host supports version {}",
                    self.entry_path.display(),
                    batch.version,
                    CONTRACT_MODEL_VERSION
                );
            }
            if batch.mutations.is_empty() {
                continue;
            }
            tree_changed |= batch
                .mutations
                .iter()
                .any(|mutation| mutation.changes_contract_tree());
            self.host_tree.apply_mutations(batch.mutations)?;
        }

        if !self.host_tree.has_root() {
            bail!("{} did not call render(<... />)", self.entry_path.display());
        }
        if !tree_changed {
            return Ok(RenderedJsx {
                tree: None,
                motion: self.host_tree.motion_frame(),
                logs,
            });
        }

        let tree = self.host_tree.materialize()?;
        validate_contract_tree(&self.entry_path, &tree)?;

        Ok(RenderedJsx {
            tree: Some(tree),
            motion: self.host_tree.motion_frame(),
            logs,
        })
    }
}

fn install_contract_metadata(js_runtime: &mut JsRuntime) -> anyhow::Result<()> {
    let families = registry()
        .iter()
        .map(|family| family.id.as_str())
        .collect::<Vec<_>>();
    let families_with_children = registry()
        .iter()
        .filter(|family| family.child_policy != ContractChildPolicy::None)
        .map(|family| family.id.as_str())
        .collect::<Vec<_>>();
    let metadata = serde_json::json!({
        "version": CONTRACT_MODEL_VERSION,
        "families": families,
        "familiesWithChildren": families_with_children,
    });
    let source = format!(
        "globalThis.__eguiContract = Object.freeze({});",
        serde_json::to_string(&metadata)?
    );
    js_runtime
        .execute_script("[egui:contract-metadata]", source)
        .map_err(|error| anyhow!("failed to install egui contract metadata: {error}"))?;
    Ok(())
}

fn load_module(
    source_maps: SourceMapStore,
    module_specifier: &ModuleSpecifier,
) -> Result<ModuleSource, deno_core::error::ModuleLoaderError> {
    match module_specifier.as_str() {
        EGUI_MODULE_SPECIFIER => {
            return Ok(module_source(
                module_specifier,
                EGUI_MODULE_SOURCE.to_owned(),
            ));
        }
        EGUI_JSX_RUNTIME_SPECIFIER | EGUI_JSX_DEV_RUNTIME_SPECIFIER => {
            return Ok(module_source(
                module_specifier,
                EGUI_JSX_RUNTIME_SOURCE.to_owned(),
            ));
        }
        EGUI_MOTION_REACT_SPECIFIER => {
            return Ok(module_source(
                module_specifier,
                EGUI_MOTION_REACT_SOURCE.to_owned(),
            ));
        }
        _ => {}
    }

    let path = module_specifier
        .to_file_path()
        .map_err(|_| JsErrorBox::generic("Only file:// and egui: modules are supported."))?;
    let media_type = MediaType::from_path(&path);
    let (module_type, should_transpile) = match media_type {
        MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => (ModuleType::JavaScript, false),
        MediaType::Jsx
        | MediaType::TypeScript
        | MediaType::Mts
        | MediaType::Cts
        | MediaType::Dts
        | MediaType::Dmts
        | MediaType::Dcts
        | MediaType::Tsx => (ModuleType::JavaScript, true),
        MediaType::Json => (ModuleType::Json, false),
        _ => {
            return Err(JsErrorBox::generic(format!(
                "Unsupported module extension for {}",
                path.display()
            )));
        }
    };

    let code = std::fs::read_to_string(&path).map_err(JsErrorBox::from_err)?;
    let code = if should_transpile {
        let transpiled = transpile_module(module_specifier, media_type, code)
            .map_err(|error| JsErrorBox::generic(error.to_string()))?;
        if let Some(source_map) = transpiled.source_map {
            source_maps
                .borrow_mut()
                .insert(module_specifier.to_string(), source_map);
        }
        transpiled.code
    } else {
        code
    };

    Ok(ModuleSource::new(
        module_type,
        ModuleSourceCode::String(code.into()),
        module_specifier,
        None,
    ))
}

struct TranspiledModule {
    code: String,
    source_map: Option<Vec<u8>>,
}

fn transpile_module(
    module_specifier: &ModuleSpecifier,
    media_type: MediaType,
    code: String,
) -> anyhow::Result<TranspiledModule> {
    let parsed = deno_ast::parse_module(ParseParams {
        specifier: module_specifier.clone(),
        text: code.into(),
        media_type,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })?;

    let emitted = parsed
        .transpile(
            &TranspileOptions {
                decorators: DecoratorsTranspileOption::Ecma,
                imports_not_used_as_values: ImportsNotUsedAsValues::Remove,
                jsx: Some(JsxRuntime::Automatic(JsxAutomaticOptions {
                    import_source: Some("egui".to_owned()),
                    development: false,
                })),
                ..Default::default()
            },
            &TranspileModuleOptions::default(),
            &EmitOptions {
                source_map: SourceMapOption::Separate,
                inline_sources: true,
                remove_comments: true,
                ..Default::default()
            },
        )?
        .into_source();

    Ok(TranspiledModule {
        code: emitted.text,
        source_map: emitted.source_map.map(|source_map| source_map.into_bytes()),
    })
}

fn decode_mutation_batch(
    entry_path: &Path,
    mutations_json: &str,
) -> anyhow::Result<HostMutationBatch> {
    serde_json::from_str(mutations_json)
        .with_context(|| format!("{} returned invalid host mutations", entry_path.display()))
}

fn validate_contract_tree(entry_path: &Path, tree: &ContractTree) -> anyhow::Result<()> {
    let root_family = tree.root.family_id();
    if !registry().iter().any(|spec| spec.id == root_family) {
        bail!(
            "{} returned unknown root family {}",
            entry_path.display(),
            root_family.as_str()
        );
    }
    validate_contract_node(entry_path, &tree.root)?;

    Ok(())
}

fn validate_contract_node(entry_path: &Path, node: &ContractNode) -> anyhow::Result<()> {
    let family = node.family_id();
    if !registry().iter().any(|spec| spec.id == family) {
        bail!(
            "{} returned unregistered family {} at node {}",
            entry_path.display(),
            family.as_str(),
            node.node_id()
        );
    }

    match node {
        ContractNode::Row(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Column(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Inset(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::SizedBox(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Card(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Sidebar(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Toolbar(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Collapsible(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::DialogueModal(props) => {
            validate_contract_children(entry_path, &props.children)
        }
        ContractNode::Popover(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::ContextMenu(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::AudioPlayback(props) => {
            validate_contract_children(entry_path, &props.children)
        }
        ContractNode::ImageTile(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Spacer(_)
        | ContractNode::MenuBar(_)
        | ContractNode::Tabs(_)
        | ContractNode::Label(_)
        | ContractNode::Button(_)
        | ContractNode::ButtonGroup(_)
        | ContractNode::Input(_)
        | ContractNode::NumberInput(_)
        | ContractNode::Checkbox(_)
        | ContractNode::Switch(_)
        | ContractNode::Select(_)
        | ContractNode::Field(_)
        | ContractNode::Separator(_)
        | ContractNode::Hierarchy(_)
        | ContractNode::Spinner(_)
        | ContractNode::Progress(_)
        | ContractNode::ToastViewport(_)
        | ContractNode::Color(_)
        | ContractNode::Icon(_)
        | ContractNode::Image(_)
        | ContractNode::Twemoji(_)
        | ContractNode::Kbd(_)
        | ContractNode::Skeleton(_)
        | ContractNode::Slider(_)
        | ContractNode::Radio(_)
        | ContractNode::RadioGroup(_)
        | ContractNode::Combobox(_)
        | ContractNode::EmojiSelector(_)
        | ContractNode::Pagination(_)
        | ContractNode::Tooltip(_)
        | ContractNode::DropdownMenu(_)
        | ContractNode::OpenWith(_)
        | ContractNode::CollabCursor(_)
        | ContractNode::IconToolbar(_)
        | ContractNode::FileTree(_)
        | ContractNode::DragBoard(_)
        | ContractNode::Command(_) => Ok(()),
    }
}

fn validate_contract_children(entry_path: &Path, children: &[ContractNode]) -> anyhow::Result<()> {
    for child in children {
        validate_contract_node(entry_path, child)?;
    }
    Ok(())
}

fn module_source(module_specifier: &ModuleSpecifier, source: String) -> ModuleSource {
    ModuleSource::new(
        ModuleType::JavaScript,
        ModuleSourceCode::String(source.into()),
        module_specifier,
        None,
    )
}

fn absolute_path(path: &Path) -> anyhow::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()
            .context("failed to resolve current directory")?
            .join(path))
    }
}
