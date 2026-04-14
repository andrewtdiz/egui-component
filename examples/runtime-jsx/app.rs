use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::SystemTime,
};

use clay_jsx_egui_bridge::{HotReloadState, JsxRuntimeLoadOutcome, JsxRuntimeSession, MotionFrame};
use egui::{CentralPanel, Context, RichText, TopBottomPanel};
use egui_component::{
    contract::{render_tree, ContractEvent, ContractTree},
    theme::{self, BaseColor, ThemeMode, ThemeSpec},
};

use super::default_entry_path;

#[derive(Debug)]
pub struct RuntimeJsxApp {
    entry_path: PathBuf,
    dependency_stamps: Vec<DependencyStamp>,
    hot_reload_state: Option<HotReloadState>,
    initialized: bool,
    session: Option<JsxRuntimeSession>,
    rendered: Option<ContractTree>,
    motion: MotionFrame,
    error: Option<String>,
}

impl Default for RuntimeJsxApp {
    fn default() -> Self {
        Self::new(default_entry_path())
    }
}

impl RuntimeJsxApp {
    pub fn new(entry_path: impl Into<PathBuf>) -> Self {
        let entry_path = entry_path.into();
        Self {
            dependency_stamps: collect_dependency_stamps([entry_path.clone()]),
            entry_path,
            hot_reload_state: None,
            initialized: false,
            session: None,
            rendered: None,
            motion: MotionFrame::default(),
            error: None,
        }
    }

    fn update_frame(&mut self, ctx: &Context) {
        self.reload_initial_or_external_changes();
        self.tick_runtime_motion(ctx);

        TopBottomPanel::top("jsx_runtime_topbar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let entry_path = self.entry_path.display().to_string();
                    ui.label(RichText::new("JSX egui runtime").strong());
                    ui.label(RichText::new(entry_path.as_str()).small().weak());
                });
                ui.separator();
            });

        CentralPanel::default().show(ctx, |ui| self.render_preview(ui));
    }

    fn render_preview(&mut self, ui: &mut egui::Ui) {
        let mut frame_events = Vec::new();

        ui.add_space(16.0);
        if let Some(error) = &self.error {
            ui.label(
                RichText::new("Runtime error")
                    .strong()
                    .color(ui.visuals().error_fg_color),
            );
            ui.label(RichText::new(error.as_str()).small().weak());
        } else {
            match self.rendered.as_ref() {
                Some(tree) => {
                    let events = render_tree(ui, tree);
                    if !events.is_empty() {
                        frame_events = events;
                    }
                }
                None => {
                    ui.label(RichText::new("No JSX tree has been rendered yet.").weak());
                }
            }
        }

        if !frame_events.is_empty() {
            self.dispatch_events_to_runtime(&frame_events);
        }
    }

    fn reload_initial_or_external_changes(&mut self) {
        if !self.initialized {
            self.initialized = true;
            self.reload_from_disk();
            return;
        }

        if dependency_stamps_changed(&self.dependency_stamps) {
            self.reload_from_disk();
        }
    }

    fn reload_from_disk(&mut self) {
        if self.error.is_none() {
            if let Some(session) = self.session.as_mut() {
                if let Ok(hot_reload_state) = session.capture_hot_reload_state() {
                    self.hot_reload_state = Some(hot_reload_state);
                }
            }
        }

        match JsxRuntimeSession::load_with_hot_reload_state_outcome(
            &self.entry_path,
            self.hot_reload_state.as_ref(),
        ) {
            JsxRuntimeLoadOutcome::Loaded { session, rendered } => {
                self.dependency_stamps = collect_dependency_stamps(
                    session
                        .dependency_paths()
                        .into_iter()
                        .chain([self.entry_path.clone()]),
                );
                self.session = Some(session);
                self.rendered = rendered.tree;
                self.motion = rendered.motion;
                self.error = None;
            }
            JsxRuntimeLoadOutcome::Failed(failure) => {
                self.session = None;
                self.motion = MotionFrame::default();
                self.rendered = None;
                self.error = Some(format!(
                    "Failed to load {}: {error}",
                    self.entry_path.display(),
                    error = failure.error
                ));
                self.dependency_stamps =
                    collect_failure_dependency_stamps(failure.dependency_paths, &self.entry_path);
            }
        }
    }

    fn dispatch_events_to_runtime(&mut self, events: &[ContractEvent]) {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        match session.dispatch_events(events) {
            Ok(rendered) => {
                if let Some(tree) = rendered.tree {
                    self.rendered = Some(tree);
                }
                self.motion = rendered.motion;
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }

    fn tick_runtime_motion(&mut self, ctx: &Context) {
        if !self.motion.active {
            return;
        }
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let now_secs = ctx.input(|input| input.time);
        match session.tick_motion(now_secs) {
            Ok(rendered) => {
                self.motion = rendered.motion;
                if self.motion.active {
                    ctx.request_repaint();
                }
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }
}

impl eframe::App for RuntimeJsxApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        theme::set_theme(ctx, ThemeSpec::preset(BaseColor::Neutral));
        theme::set_mode(ctx, ThemeMode::System);
        self.update_frame(ctx);
    }
}

#[derive(Debug, Clone)]
struct DependencyStamp {
    path: PathBuf,
    modified: Option<SystemTime>,
}

fn dependency_stamps_changed(stamps: &[DependencyStamp]) -> bool {
    stamps
        .iter()
        .any(|stamp| modified_time(&stamp.path) != stamp.modified)
}

fn collect_dependency_stamps(paths: impl IntoIterator<Item = PathBuf>) -> Vec<DependencyStamp> {
    paths
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|path| DependencyStamp {
            modified: modified_time(&path),
            path,
        })
        .collect()
}

fn collect_failure_dependency_stamps(
    dependency_paths: Vec<PathBuf>,
    entry_path: &Path,
) -> Vec<DependencyStamp> {
    let paths = if dependency_paths.is_empty() {
        vec![entry_path.to_path_buf()]
    } else {
        dependency_paths
    };
    collect_dependency_stamps(paths)
}

fn modified_time(path: &std::path::Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use egui_component::contract::{ContractEvent, ContractNode, EventKind, EventValue};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn imported_module_changes_trigger_reload_from_dependency_stamps() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        let child_path = dir.path().join("copy.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";
import { Copy } from "./copy.tsx";

function App() {
  return (
    <div id="root" data-slot="column">
      <Copy />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("entry file should be written");
        std::fs::write(
            &child_path,
            r#"
export function Copy() {
  return <label id="copy" text="Old copy" />;
}
"#,
        )
        .expect("child file should be written");

        let (session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "copy").expect("copy label should exist")),
            Some("Old copy")
        );
        let dependency_stamps = collect_dependency_stamps(session.dependency_paths());

        write_with_newer_timestamp(
            &child_path,
            r#"
export function Copy() {
  return <label id="copy" text="New copy" />;
}
"#,
        );
        assert!(
            dependency_stamps_changed(&dependency_stamps),
            "imported child change should invalidate dependency stamps"
        );
    }

    #[test]
    fn failed_reload_keeps_last_good_snapshot_and_watches_attempted_dependencies() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        let child_path = dir.path().join("copy.tsx");
        let extra_path = dir.path().join("extra.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";
import { Copy } from "./copy.tsx";

function App() {
  const [checked, setChecked] = useState(false);
  return (
    <div id="root" data-slot="column">
      <input
        id="toggle"
        type="checkbox"
        checked={checked}
        onToggle={(event, value) => setChecked(Boolean(value))}
      />
      <label id="status" text={checked ? "On" : "Off"} />
      <Copy />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("entry file should be written");
        std::fs::write(
            &child_path,
            r#"
export function Copy() {
  return <label id="copy" text="Old copy" />;
}
"#,
        )
        .expect("child file should be written");

        let mut app = RuntimeJsxApp::new(&entry_path);
        app.reload_from_disk();

        let rendered = app
            .session
            .as_mut()
            .expect("session should be loaded")
            .dispatch_events(&[ContractEvent::new("toggle", EventKind::Toggled)
                .value(Some(EventValue::Boolean(true)))])
            .expect("toggle should update state");
        if let Some(tree) = rendered.tree {
            app.rendered = Some(tree);
        }
        app.motion = rendered.motion;

        let tree = rendered_tree(&app);
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(true));
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("On")
        );

        write_with_newer_timestamp(
            &child_path,
            r#"
import { Extra } from "./extra.tsx";

export function Copy() {
  return (
    <div id="copy-container" data-slot="column">
      <label id="copy" text="Reloaded copy" />
      <Extra />
    </div>
  );
}
"#,
        );

        app.reload_from_disk();
        assert!(
            app.session.is_none(),
            "failed reload should tear down the live session"
        );
        assert!(
            app.rendered.is_none(),
            "failed reload should clear the rendered tree"
        );
        assert!(
            app.hot_reload_state.is_some(),
            "last good hot reload state should survive a failed rebuild"
        );
        assert!(
            app.error.is_some(),
            "expected failed reload to surface an error"
        );
        assert!(
            app.dependency_stamps
                .iter()
                .any(|stamp| stamp.path == extra_path),
            "attempted dependency set should include the new missing import"
        );

        std::fs::write(
            &extra_path,
            r#"
export function Extra() {
  return <label id="extra" text="Extra copy" />;
}
"#,
        )
        .expect("missing dependency should be written");
        assert!(
            dependency_stamps_changed(&app.dependency_stamps),
            "creating the previously missing dependency should invalidate failure stamps"
        );

        app.initialized = true;
        app.reload_initial_or_external_changes();
        assert!(
            app.error.is_none(),
            "reload should recover once the dependency exists"
        );

        let tree = rendered_tree(&app);
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(true));
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("On")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "copy").expect("copy label should exist")),
            Some("Reloaded copy")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "extra").expect("extra label should exist")),
            Some("Extra copy")
        );
    }

    fn rendered_tree(app: &RuntimeJsxApp) -> &ContractTree {
        app.rendered.as_ref().unwrap_or_else(|| {
            panic!(
                "expected app to have a rendered tree, error: {:?}",
                app.error
            )
        })
    }

    fn write_with_newer_timestamp(path: &Path, contents: &str) {
        let previous = modified_time(path);
        for _ in 0..12 {
            std::fs::write(path, contents).expect("test file should be written");
            if modified_time(path) != previous {
                return;
            }
            sleep(Duration::from_millis(120));
        }
        panic!(
            "failed to advance modified timestamp for {}",
            path.display()
        );
    }

    fn checkbox_value(node: &ContractNode, node_id: &str) -> Option<bool> {
        if node.node_id().as_str() == node_id {
            if let ContractNode::Checkbox(checkbox) = node {
                return Some(checkbox.value);
            }
        }

        for child in contract_children(node) {
            if let Some(value) = checkbox_value(child, node_id) {
                return Some(value);
            }
        }

        None
    }

    fn find_node<'a>(node: &'a ContractNode, node_id: &str) -> Option<&'a ContractNode> {
        if node.node_id().as_str() == node_id {
            return Some(node);
        }

        for child in contract_children(node) {
            if let Some(found) = find_node(child, node_id) {
                return Some(found);
            }
        }

        None
    }

    fn contract_children(node: &ContractNode) -> &[ContractNode] {
        match node {
            ContractNode::Row(props) => &props.children,
            ContractNode::Column(props) => &props.children,
            ContractNode::Inset(props) => &props.children,
            ContractNode::SizedBox(props) => &props.children,
            ContractNode::Card(props) => &props.children,
            ContractNode::Sidebar(props) => &props.children,
            ContractNode::Toolbar(props) => &props.children,
            ContractNode::Collapsible(props) => &props.children,
            ContractNode::DialogueModal(props) => &props.children,
            ContractNode::Popover(props) => &props.children,
            ContractNode::ContextMenu(props) => &props.children,
            ContractNode::AudioPlayback(props) => &props.children,
            ContractNode::ImageTile(props) => &props.children,
            _ => &[],
        }
    }

    fn label_text(node: &ContractNode) -> Option<&str> {
        match node {
            ContractNode::Label(props) => Some(props.text.as_str()),
            _ => None,
        }
    }
}
