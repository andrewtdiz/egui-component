use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use clay_jsx_egui_bridge::{
    HotReloadState, JsxRuntimeDebugMetrics, JsxRuntimeLoadOutcome, JsxRuntimeSession, MotionFrame,
};
use egui::{CentralPanel, Context, RichText, TopBottomPanel};
use egui_component::{
    contract::{render_tree, ContractEvent, ContractTree},
    theme::{self, BaseColor, ThemeMode, ThemeSpec},
};

use super::{default_entry_path, request_host_repaint, ExampleHostMetricsTracker, ReloadWatcher};
#[cfg(test)]
use super::ExampleHostDebugSnapshot;

#[derive(Debug)]
pub struct RuntimeJsxApp {
    entry_path: PathBuf,
    hot_reload_state: Option<HotReloadState>,
    initialized: bool,
    reload_watcher: Option<ReloadWatcher>,
    session: Option<JsxRuntimeSession>,
    rendered: Option<ContractTree>,
    motion: MotionFrame,
    watched_files: BTreeSet<PathBuf>,
    error: Option<String>,
    metrics: ExampleHostMetricsTracker,
    last_torn_down_session: Option<JsxRuntimeDebugMetrics>,
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
            entry_path,
            hot_reload_state: None,
            initialized: false,
            reload_watcher: None,
            session: None,
            rendered: None,
            motion: MotionFrame::default(),
            watched_files: BTreeSet::new(),
            error: None,
            metrics: ExampleHostMetricsTracker::default(),
            last_torn_down_session: None,
        }
    }

    fn update_frame(&mut self, ctx: &Context) {
        self.reload_initial_or_external_changes(ctx);
        self.drain_pending_runtime_updates(ctx);
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
            self.dispatch_events_to_runtime(&frame_events, ui.ctx());
        }
    }

    fn reload_initial_or_external_changes(&mut self, ctx: &Context) {
        self.ensure_reload_watcher(ctx);
        if !self.initialized {
            self.initialized = true;
            self.reload_from_disk(ctx);
            return;
        }

        if self
            .reload_watcher
            .as_mut()
            .is_some_and(|watcher| watcher.take_pending_reload())
        {
            self.reload_from_disk(ctx);
        }
    }

    fn reload_from_disk(&mut self, ctx: &Context) {
        self.metrics.note_reload_attempt();
        self.capture_hot_reload_state();
        self.teardown_session();

        match JsxRuntimeSession::load_with_hot_reload_state_outcome(
            &self.entry_path,
            self.hot_reload_state.as_ref(),
        ) {
            JsxRuntimeLoadOutcome::Loaded {
                mut session,
                rendered,
            } => {
                self.set_watched_files(
                    session
                        .dependency_paths()
                        .into_iter()
                        .chain([self.entry_path.clone()]),
                );
                install_session_wake_callback(&mut session, ctx, self.metrics.clone());
                self.session = Some(session);
                self.rendered = rendered.tree;
                self.motion = rendered.motion;
                self.error = None;
                self.metrics.note_reload_success();
                if self.motion.active {
                    request_host_repaint(ctx, &self.metrics);
                }
                self.drain_pending_runtime_updates(ctx);
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
                self.set_watched_files(collect_failure_tracked_files(
                    failure.dependency_paths,
                    &self.entry_path,
                ));
                self.metrics.note_reload_failure();
            }
        }
    }

    fn dispatch_events_to_runtime(&mut self, events: &[ContractEvent], ctx: &Context) {
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
                if self.motion.active {
                    request_host_repaint(ctx, &self.metrics);
                }
            }
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }

    fn drain_pending_runtime_updates(&mut self, ctx: &Context) {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        match session.drain_pending_runtime_updates() {
            Ok(Some(rendered)) => {
                self.metrics.note_runtime_update_drain();
                if let Some(tree) = rendered.tree {
                    self.rendered = Some(tree);
                }
                self.motion = rendered.motion;
                self.error = None;
                if self.motion.active {
                    request_host_repaint(ctx, &self.metrics);
                }
            }
            Ok(None) => self.metrics.note_runtime_update_drain_empty(),
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
                    request_host_repaint(ctx, &self.metrics);
                }
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }

    fn capture_hot_reload_state(&mut self) {
        if self.error.is_some() {
            return;
        }
        let Some(session) = self.session.as_mut() else {
            return;
        };
        if let Ok(hot_reload_state) = session.capture_hot_reload_state() {
            self.hot_reload_state = Some(hot_reload_state);
        }
    }

    fn ensure_reload_watcher(&mut self, ctx: &Context) {
        if self.reload_watcher.is_some() {
            return;
        }
        let repaint_ctx = ctx.clone();
        let repaint_metrics = self.metrics.clone();
        match ReloadWatcher::new(move || request_host_repaint(&repaint_ctx, &repaint_metrics)) {
            Ok(mut watcher) => {
                if let Err(error) = watcher.set_tracked_files(self.watched_files.iter().cloned()) {
                    self.error = Some(format!("Failed to start JSX reload watcher: {error}"));
                    return;
                }
                self.reload_watcher = Some(watcher);
            }
            Err(error) => {
                self.error = Some(format!("Failed to start JSX reload watcher: {error}"));
            }
        }
    }

    fn set_watched_files(&mut self, files: impl IntoIterator<Item = PathBuf>) {
        self.watched_files = files.into_iter().collect();
        if let Some(watcher) = self.reload_watcher.as_mut() {
            if let Err(error) = watcher.set_tracked_files(self.watched_files.iter().cloned()) {
                self.error = Some(format!("Failed to update JSX reload watcher: {error}"));
            }
        }
    }

    fn teardown_session(&mut self) {
        if let Some(mut session) = self.session.take() {
            let _ = session.teardown();
            self.last_torn_down_session = Some(session.debug_metrics());
        }
    }

    #[cfg(test)]
    pub(crate) fn debug_snapshot(&self) -> ExampleHostDebugSnapshot {
        ExampleHostDebugSnapshot {
            host: self.metrics.snapshot(),
            live_session: self.session.as_ref().map(JsxRuntimeSession::debug_metrics),
            last_torn_down_session: self.last_torn_down_session.clone(),
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

fn collect_failure_tracked_files(
    dependency_paths: Vec<PathBuf>,
    entry_path: &Path,
) -> BTreeSet<PathBuf> {
    let paths = if dependency_paths.is_empty() {
        vec![entry_path.to_path_buf()]
    } else {
        dependency_paths
    };
    paths.into_iter().collect()
}

fn install_session_wake_callback(
    session: &mut JsxRuntimeSession,
    ctx: &Context,
    metrics: ExampleHostMetricsTracker,
) {
    let ctx = ctx.clone();
    session.set_wake_callback(move || request_host_repaint(&ctx, &metrics));
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use clay_jsx_runtime::contract::{ContractEvent, ContractNode, EventKind, EventValue};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn successful_load_tracks_imported_modules_for_reload_watch_targets() {
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
        assert!(
            session
                .dependency_paths()
                .into_iter()
                .collect::<BTreeSet<_>>()
                .contains(&child_path),
            "imported child should be tracked as a watched dependency"
        );
    }

    #[test]
    fn failed_reload_tracks_attempted_dependencies_and_recovers_with_cold_state() {
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
        let ctx = egui::Context::default();
        app.reload_from_disk(&ctx);

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

        std::fs::write(
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
        )
        .expect("updated child file should be written");

        app.reload_from_disk(&ctx);
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
            "the API-compatible hot reload snapshot should still be captured"
        );
        assert!(
            app.error.is_some(),
            "expected failed reload to surface an error"
        );
        assert!(
            app.watched_files.contains(&extra_path),
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
        app.reload_from_disk(&ctx);
        assert!(
            app.error.is_none(),
            "reload should recover once the dependency exists"
        );

        let tree = rendered_tree(&app);
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(false));
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("Off")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "copy").expect("copy label should exist")),
            Some("Reloaded copy")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "extra").expect("extra label should exist")),
            Some("Extra copy")
        );

        let snapshot = app.debug_snapshot();
        assert_eq!(snapshot.host.reload_attempt_count, 3);
        assert_eq!(snapshot.host.reload_success_count, 2);
        assert_eq!(snapshot.host.reload_failure_count, 1);
        assert_eq!(snapshot.host.reload_recovery_count, 1);
        let torn_down = snapshot
            .last_torn_down_session
            .expect("failed reload should capture the dead session metrics");
        assert_eq!(torn_down.teardown_count, 1);
        assert_eq!(torn_down.runtime.active_timer_count, 0);
        assert!(!torn_down.runtime.pending_host_wake);
        let live = snapshot
            .live_session
            .expect("recovered reload should have a live session");
        assert_eq!(live.render_call_count, 1);
    }

    #[test]
    fn timer_driven_runtime_update_requests_repaint_and_drains_one_visible_update() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("async.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useEffect, useState } from "egui";

function App() {
  const [status, setStatus] = useState("idle");
  useEffect(() => {
    const handle = setTimeout(() => setStatus("ready"), 15);
    return () => clearTimeout(handle);
  }, []);
  return <label id="status" text={status} />;
}

render(<App />);
"#,
        )
        .expect("entry file should be written");

        let mut app = RuntimeJsxApp::new(&entry_path);
        let ctx = egui::Context::default();
        app.reload_from_disk(&ctx);
        app.initialized = true;
        let before = app.debug_snapshot();

        wait_for(Duration::from_millis(100), || {
            app.drain_pending_runtime_updates(&ctx);
            label_text(
                find_node(&rendered_tree(&app).root, "status")
                    .expect("status label should exist while draining"),
            ) == Some("ready")
        });

        let snapshot = app.debug_snapshot();
        assert_eq!(
            snapshot.host.runtime_update_drain_count,
            before.host.runtime_update_drain_count + 1
        );
        assert!(
            snapshot.host.repaint_request_count >= 1,
            "timer wake should request at least one repaint"
        );
        let live = snapshot
            .live_session
            .expect("timer-driven update should leave a live session");
        assert!(live.runtime.host_wake_count >= 1);
        assert!(live.runtime.host_wake_callback_count >= 1);
        assert!(live.runtime.host_callback_drain_cycles >= 1);
        assert!(live.runtime.host_callbacks_invoked >= 1);
        let tree = rendered_tree(&app);
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("ready")
        );
    }

    #[test]
    fn idle_host_path_does_not_synthesize_reload_attempts_or_visible_runtime_updates() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("static.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

render(<label id="status" text="steady" />);
"#,
        )
        .expect("entry file should be written");

        let mut app = RuntimeJsxApp::new(&entry_path);
        let ctx = egui::Context::default();
        app.reload_from_disk(&ctx);
        app.initialized = true;
        let before = app.debug_snapshot();

        app.reload_initial_or_external_changes(&ctx);
        app.drain_pending_runtime_updates(&ctx);

        let after = app.debug_snapshot();
        assert_eq!(
            after.host.reload_attempt_count,
            before.host.reload_attempt_count
        );
        assert_eq!(
            after.host.runtime_update_drain_count,
            before.host.runtime_update_drain_count
        );
        assert_eq!(
            after.host.runtime_update_drain_empty_count,
            before.host.runtime_update_drain_empty_count + 1
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

    fn wait_for(timeout: Duration, mut predicate: impl FnMut() -> bool) {
        let deadline = Instant::now() + timeout;
        loop {
            if predicate() {
                return;
            }
            if Instant::now() >= deadline {
                panic!("timed out waiting for the condition");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
