use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context as AnyhowContext};
use clay_jsx_runtime::{JsxRuntimeOptions, RuntimeSession};
use egui_component::contract::{
    registry, ContractChildPolicy, ContractEvent, ContractNode, ContractTree,
    CONTRACT_MODEL_VERSION,
};

use super::host_tree::{HostMutationBatch, HostTree};
use super::motion::MotionFrame;
use crate::{EGUI_JSX_RUNTIME_SOURCE, EGUI_MODULE_SOURCE, EGUI_MOTION_REACT_SOURCE};

#[derive(Debug)]
pub struct RenderedJsx {
    pub tree: Option<ContractTree>,
    pub motion: MotionFrame,
    pub logs: clay_jsx_runtime::RuntimeLogBuffer,
}

pub struct JsxRuntimeSession {
    entry_path: PathBuf,
    host_tree: HostTree,
    runtime: RuntimeSession,
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
        let mut runtime = RuntimeSession::new(egui_runtime_options())?;
        install_contract_metadata(&mut runtime)?;
        runtime.load_main_module(&entry_path)?;

        let mut session = Self {
            entry_path,
            host_tree: HostTree::default(),
            runtime,
        };
        let rendered = session.take_rendered()?;
        Ok((session, rendered))
    }

    pub fn dispatch_events(&mut self, events: &[ContractEvent]) -> anyhow::Result<RenderedJsx> {
        let events_json = serde_json::to_string(events)?;
        let source = format!("globalThis.__eguiDispatchEvents({events_json});");
        self.runtime
            .execute_script("[egui:dispatch-events]", source)
            .map_err(|error| anyhow!("failed to dispatch egui events into JSX runtime: {error}"))?;
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
            logs: clay_jsx_runtime::RuntimeLogBuffer::new(),
        })
    }

    fn take_rendered(&mut self) -> anyhow::Result<RenderedJsx> {
        let update = self.runtime.take_update();
        let mutation_batches_json = update.commit_batches_json;
        let logs = update.logs;

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

fn egui_runtime_options() -> JsxRuntimeOptions {
    JsxRuntimeOptions::new("egui")
        .with_virtual_module("egui", EGUI_MODULE_SOURCE)
        .with_virtual_module("clay", EGUI_MODULE_SOURCE)
        .with_virtual_module("egui/jsx-runtime", EGUI_JSX_RUNTIME_SOURCE)
        .with_virtual_module("clay/jsx-runtime", EGUI_JSX_RUNTIME_SOURCE)
        .with_virtual_module("egui/jsx-dev-runtime", EGUI_JSX_RUNTIME_SOURCE)
        .with_virtual_module("clay/jsx-dev-runtime", EGUI_JSX_RUNTIME_SOURCE)
        .with_virtual_module("motion/react", EGUI_MOTION_REACT_SOURCE)
        .with_virtual_module("react/motion", EGUI_MOTION_REACT_SOURCE)
}

fn install_contract_metadata(runtime: &mut RuntimeSession) -> anyhow::Result<()> {
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
    runtime
        .execute_script("[egui:contract-metadata]", source)
        .map_err(|error| anyhow!("failed to install egui contract metadata: {error}"))?;
    Ok(())
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

fn absolute_path(path: &Path) -> anyhow::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()
            .context("failed to resolve current directory")?
            .join(path))
    }
}
