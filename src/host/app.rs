use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};

#[cfg(test)]
use clay_jsx_egui_bridge::JsxRuntimeSession;
use clay_jsx_egui_bridge::{
    JsxRuntimeDebugMetrics, JsxRuntimeSessionWorker, JsxRuntimeWorkerDebugSnapshot,
    JsxRuntimeWorkerEvent, JsxRuntimeWorkerReloadOutcome, MotionFrame,
};
use egui::{CentralPanel, Context, RichText, TopBottomPanel};
use egui_component::{
    contract::{render_tree, ContractEvent, ContractTree},
    theme::{self, BaseColor, ThemeMode, ThemeSpec},
};

#[cfg(test)]
use super::ExampleHostDebugSnapshot;
use super::{
    default_entry_path, request_host_repaint, runtime_update_drain_frame_budget,
    ExampleHostMetricsTracker, ReloadWatcher,
};

#[derive(Debug)]
pub struct RuntimeJsxApp {
    entry_path: PathBuf,
    initialized: bool,
    reload_watcher: Option<ReloadWatcher>,
    runtime_worker: Option<JsxRuntimeSessionWorker>,
    reload_in_flight: bool,
    dispatch_in_flight: bool,
    drain_in_flight: bool,
    tick_in_flight: bool,
    queued_dispatch_events: Vec<ContractEvent>,
    rendered: Option<ContractTree>,
    motion: MotionFrame,
    max_runtime_update_drains_per_frame: usize,
    remaining_runtime_update_drains_this_frame: usize,
    watched_files: BTreeSet<PathBuf>,
    error: Option<String>,
    metrics: ExampleHostMetricsTracker,
    live_session_metrics: Option<JsxRuntimeDebugMetrics>,
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
        let max_runtime_update_drains_per_frame = runtime_update_drain_frame_budget();
        Self {
            entry_path,
            initialized: false,
            reload_watcher: None,
            runtime_worker: None,
            reload_in_flight: false,
            dispatch_in_flight: false,
            drain_in_flight: false,
            tick_in_flight: false,
            queued_dispatch_events: Vec::new(),
            rendered: None,
            motion: MotionFrame::default(),
            max_runtime_update_drains_per_frame,
            remaining_runtime_update_drains_this_frame: max_runtime_update_drains_per_frame,
            watched_files: BTreeSet::new(),
            error: None,
            metrics: ExampleHostMetricsTracker::default(),
            live_session_metrics: None,
            last_torn_down_session: None,
        }
    }

    fn update_frame(&mut self, ctx: &Context) {
        self.reset_runtime_update_drain_frame_budget();
        self.ensure_runtime_worker(ctx);
        self.collect_worker_events(ctx);
        self.reload_initial_or_external_changes(ctx);
        self.drive_runtime_update_drain_loop(ctx);
        self.schedule_tick_runtime_motion(ctx);
        self.collect_worker_events(ctx);

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
        self.collect_worker_events(ui.ctx());
        let mut frame_events = Vec::new();

        ui.add_space(16.0);
        if let Some(error) = &self.error {
            ui.label(
                RichText::new("Runtime error")
                    .strong()
                    .color(ui.visuals().error_fg_color),
            );
            ui.label(RichText::new(error.as_str()).small().weak());
        }

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

        if !frame_events.is_empty() {
            self.dispatch_events_to_runtime(&frame_events, ui.ctx());
        }

        self.collect_worker_events(ui.ctx());
    }

    fn reload_initial_or_external_changes(&mut self, ctx: &Context) {
        self.ensure_reload_watcher(ctx);
        if !self.initialized {
            self.initialized = true;
            self.request_reload_from_disk(ctx);
            return;
        }

        if self
            .reload_watcher
            .as_mut()
            .is_some_and(|watcher| watcher.take_pending_reload())
        {
            self.request_reload_from_disk(ctx);
        }
    }

    #[cfg(test)]
    fn reload_from_disk(&mut self, ctx: &Context) {
        self.ensure_runtime_worker(ctx);
        self.request_reload_from_disk(ctx);
        self.wait_for_runtime_worker_idle(ctx, std::time::Duration::from_secs(10));
    }

    fn dispatch_events_to_runtime(&mut self, events: &[ContractEvent], ctx: &Context) {
        if events.is_empty() {
            return;
        }
        self.queued_dispatch_events.extend_from_slice(events);
        self.try_dispatch_queued_events(ctx);
    }

    #[cfg(test)]
    fn drain_pending_runtime_updates(&mut self, ctx: &Context) {
        self.reset_runtime_update_drain_frame_budget();
        self.drive_runtime_update_drain_loop(ctx);
        self.wait_for_runtime_worker_idle(ctx, std::time::Duration::from_secs(2));
    }

    fn ensure_runtime_worker(&mut self, ctx: &Context) {
        if self.runtime_worker.is_some() {
            return;
        }
        let repaint_ctx = ctx.clone();
        let repaint_metrics = self.metrics.clone();
        let wake_callback: Arc<dyn Fn() + Send + Sync + 'static> = Arc::new(move || {
            request_host_repaint(&repaint_ctx, &repaint_metrics);
        });
        match JsxRuntimeSessionWorker::spawn(Some(wake_callback)) {
            Ok(worker) => {
                self.runtime_worker = Some(worker);
                self.error = None;
            }
            Err(error) => {
                self.error = Some(format!("Failed to start JSX runtime worker: {error}"));
            }
        }
    }

    fn request_reload_from_disk(&mut self, ctx: &Context) {
        self.ensure_runtime_worker(ctx);
        if self.reload_in_flight {
            return;
        }
        let Some(worker) = self.runtime_worker.as_ref() else {
            return;
        };

        self.metrics.note_reload_attempt();
        match worker.request_reload(self.entry_path.clone()) {
            Ok(()) => {
                self.reload_in_flight = true;
                request_host_repaint(ctx, &self.metrics);
            }
            Err(error) => {
                self.error = Some(format!("Failed to queue JSX runtime reload: {error}"));
            }
        }
    }

    fn try_dispatch_queued_events(&mut self, ctx: &Context) {
        if self.dispatch_in_flight || self.queued_dispatch_events.is_empty() {
            return;
        }
        if self.live_session_metrics.is_none() {
            return;
        }
        let Some(worker) = self.runtime_worker.as_ref() else {
            return;
        };

        let events = std::mem::take(&mut self.queued_dispatch_events);
        match worker.request_dispatch_events(events) {
            Ok(()) => {
                self.dispatch_in_flight = true;
                request_host_repaint(ctx, &self.metrics);
            }
            Err(error) => {
                self.error = Some(format!(
                    "Failed to queue JSX runtime event dispatch: {error}"
                ));
            }
        }
    }

    fn reset_runtime_update_drain_frame_budget(&mut self) {
        self.remaining_runtime_update_drains_this_frame = self.max_runtime_update_drains_per_frame;
    }

    fn drive_runtime_update_drain_loop(&mut self, ctx: &Context) {
        if self.schedule_drain_pending_runtime_updates(ctx) {
            self.collect_worker_events(ctx);
        }
    }

    fn schedule_drain_pending_runtime_updates(&mut self, ctx: &Context) -> bool {
        if self.drain_in_flight || self.live_session_metrics.is_none() {
            return false;
        }
        if self.remaining_runtime_update_drains_this_frame == 0 {
            return false;
        }
        let Some(worker) = self.runtime_worker.as_ref() else {
            return false;
        };
        match worker.request_drain_pending_runtime_updates() {
            Ok(()) => {
                self.drain_in_flight = true;
                self.remaining_runtime_update_drains_this_frame -= 1;
                request_host_repaint(ctx, &self.metrics);
                true
            }
            Err(error) => {
                self.error = Some(format!("Failed to queue JSX runtime update drain: {error}"));
                false
            }
        }
    }

    fn schedule_tick_runtime_motion(&mut self, ctx: &Context) {
        if !self.motion.active || self.tick_in_flight || self.live_session_metrics.is_none() {
            return;
        }
        let Some(worker) = self.runtime_worker.as_ref() else {
            return;
        };
        let now_secs = ctx.input(|input| input.time);
        match worker.request_tick_motion(now_secs) {
            Ok(()) => {
                self.tick_in_flight = true;
                request_host_repaint(ctx, &self.metrics);
            }
            Err(error) => {
                self.error = Some(format!("Failed to queue JSX runtime motion tick: {error}"));
            }
        }
    }

    fn collect_worker_events(&mut self, ctx: &Context) {
        loop {
            let next_event = {
                let Some(worker) = self.runtime_worker.as_ref() else {
                    return;
                };
                worker.try_recv_event()
            };

            match next_event {
                Ok(Some(event)) => self.handle_worker_event(event, ctx),
                Ok(None) => break,
                Err(error) => {
                    self.error = Some(error.to_string());
                    self.runtime_worker = None;
                    self.clear_in_flight_requests();
                    break;
                }
            }
        }
    }

    fn handle_worker_event(&mut self, event: JsxRuntimeWorkerEvent, ctx: &Context) {
        match event {
            JsxRuntimeWorkerEvent::ReloadCompleted { outcome, snapshot } => {
                self.reload_in_flight = false;
                self.apply_worker_snapshot(snapshot);
                match outcome {
                    JsxRuntimeWorkerReloadOutcome::Loaded {
                        rendered,
                        dependency_paths,
                    } => {
                        self.set_watched_files(
                            dependency_paths
                                .into_iter()
                                .chain([self.entry_path.clone()]),
                        );
                        self.apply_rendered_update(rendered, ctx);
                        self.error = None;
                        self.metrics.note_reload_success();
                    }
                    JsxRuntimeWorkerReloadOutcome::Failed(failure) => {
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
                        if self.live_session_metrics.is_none() {
                            self.motion = MotionFrame::default();
                            self.rendered = None;
                        }
                    }
                }
            }
            JsxRuntimeWorkerEvent::DispatchCompleted { result, snapshot } => {
                self.dispatch_in_flight = false;
                self.apply_worker_snapshot(snapshot);
                match result {
                    Ok(rendered) => {
                        self.apply_rendered_update(rendered, ctx);
                        self.error = None;
                    }
                    Err(error) => {
                        self.error = Some(error.to_string());
                    }
                }
                self.try_dispatch_queued_events(ctx);
            }
            JsxRuntimeWorkerEvent::DrainCompleted { result, snapshot } => {
                self.drain_in_flight = false;
                self.apply_worker_snapshot(snapshot);
                let mut should_queue_follow_up = false;
                match result {
                    Ok(Some(rendered)) => {
                        self.metrics.note_runtime_update_drain();
                        self.apply_rendered_update(rendered, ctx);
                        self.error = None;
                        should_queue_follow_up = true;
                    }
                    Ok(None) => {
                        self.metrics.note_runtime_update_drain_empty();
                    }
                    Err(error) => {
                        self.error = Some(error.to_string());
                    }
                }
                if should_queue_follow_up {
                    let _ = self.schedule_drain_pending_runtime_updates(ctx);
                }
            }
            JsxRuntimeWorkerEvent::TickCompleted { result, snapshot } => {
                self.tick_in_flight = false;
                self.apply_worker_snapshot(snapshot);
                match result {
                    Ok(rendered) => {
                        self.apply_rendered_update(rendered, ctx);
                        self.error = None;
                    }
                    Err(error) => {
                        self.error = Some(error.to_string());
                    }
                }
            }
            JsxRuntimeWorkerEvent::TeardownCompleted { result, snapshot } => {
                self.clear_in_flight_requests();
                self.apply_worker_snapshot(snapshot);
                if let Err(error) = result {
                    self.error = Some(error.to_string());
                }
            }
        }
    }

    fn apply_worker_snapshot(&mut self, snapshot: JsxRuntimeWorkerDebugSnapshot) {
        self.live_session_metrics = snapshot.live_session;
        self.last_torn_down_session = snapshot.last_torn_down_session;
    }

    fn apply_rendered_update(
        &mut self,
        rendered: clay_jsx_egui_bridge::RenderedJsx,
        ctx: &Context,
    ) {
        if let Some(tree) = rendered.tree {
            self.rendered = Some(tree);
        }
        self.motion = rendered.motion;
        if self.motion.active {
            request_host_repaint(ctx, &self.metrics);
        }
    }

    fn clear_in_flight_requests(&mut self) {
        self.reload_in_flight = false;
        self.dispatch_in_flight = false;
        self.drain_in_flight = false;
        self.tick_in_flight = false;
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

    #[cfg(test)]
    fn has_pending_worker_requests(&self) -> bool {
        self.reload_in_flight
            || self.dispatch_in_flight
            || self.drain_in_flight
            || self.tick_in_flight
    }

    #[cfg(test)]
    fn wait_for_runtime_worker_idle(&mut self, ctx: &Context, timeout: std::time::Duration) {
        let deadline = std::time::Instant::now() + timeout;
        while self.has_pending_worker_requests() && std::time::Instant::now() < deadline {
            self.collect_worker_events(ctx);
            if self.has_pending_worker_requests() {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        self.collect_worker_events(ctx);
        if self.has_pending_worker_requests() {
            self.error = Some(format!(
                "timed out waiting for JSX runtime worker after {:?}",
                timeout
            ));
        }
    }

    fn shutdown_runtime_worker(&mut self) {
        let Some(mut worker) = self.runtime_worker.take() else {
            return;
        };
        let _ = worker.request_teardown();
        let _ = worker.shutdown();
    }

    #[cfg(test)]
    pub(crate) fn debug_snapshot(&self) -> ExampleHostDebugSnapshot {
        ExampleHostDebugSnapshot {
            host: self.metrics.snapshot(),
            live_session: self.live_session_metrics.clone(),
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

impl Drop for RuntimeJsxApp {
    fn drop(&mut self) {
        self.shutdown_runtime_worker();
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

#[cfg(test)]
mod tests {
    use std::{
        path::Path,
        time::{Duration, Instant},
    };

    use clay_jsx_runtime::contract::{ContractEvent, ContractNode, EventKind};
    use egui::{pos2, CentralPanel, Event, Modifiers, PointerButton, RawInput};
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
    fn failed_reload_keeps_live_tree_visible_and_interactive_until_recovery() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        let child_path = dir.path().join("copy.tsx");
        let extra_path = dir.path().join("nested").join("runtime").join("extra.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";
import { Copy } from "./copy.tsx";

function App() {
  const [checked, setChecked] = useState(false);
  return (
    <div id="root" data-slot="column">
      <button id="toggle" label="Toggle" onClick={() => setChecked((value) => !value)} />
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
        app.ensure_reload_watcher(&ctx);
        assert!(
            app.reload_watcher.is_some(),
            "expected reload watcher to be active for missing-import fallback coverage"
        );

        app.dispatch_events_to_runtime(&[ContractEvent::new("toggle", EventKind::Clicked)], &ctx);
        app.wait_for_runtime_worker_idle(&ctx, Duration::from_secs(5));

        let tree = rendered_tree(&app);
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("On")
        );

        std::fs::write(
            &child_path,
            r#"
import { Extra } from "./nested/runtime/extra.tsx";

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
            app.debug_snapshot().live_session.is_some(),
            "failed reload should keep the previous live session"
        );
        assert!(
            app.rendered.is_some(),
            "failed reload should keep the previous rendered tree visible"
        );
        assert!(
            app.error.is_some(),
            "expected failed reload to surface an error"
        );
        let reload_error = app
            .error
            .as_deref()
            .expect("expected failed reload to preserve the runtime load error");
        assert!(
            !reload_error.contains("Failed to update JSX reload watcher"),
            "missing nested imports should register a nearest-existing watch ancestor, got watcher failure: {reload_error}"
        );
        assert!(
            app.watched_files.contains(&extra_path),
            "attempted dependency set should include the new missing import"
        );
        assert!(
            app.debug_snapshot().last_torn_down_session.is_none(),
            "failed reload should not tear down the previous live session"
        );

        let tree = rendered_tree(&app);
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("On")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "copy").expect("copy label should exist")),
            Some("Old copy")
        );

        run_preview_frame(&mut app, &ctx, RawInput::default());
        assert!(
            app.error.is_some(),
            "render_preview should keep the reload error banner visible before interaction"
        );

        assert!(
            click_preview_toggle_until_status(&mut app, &ctx, "Off"),
            "render_preview should keep the stale tree interactive via UI-originated events while the reload error is shown"
        );
        assert!(
            app.error.is_none(),
            "successful UI-originated event dispatch should clear the reload error"
        );

        let tree = rendered_tree(&app);
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("Off")
        );

        std::fs::create_dir_all(
            extra_path
                .parent()
                .expect("nested missing dependency should have a parent directory"),
        )
        .expect("missing dependency directories should be created");
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
            .expect("successful replacement should capture the previous session metrics");
        assert_eq!(torn_down.teardown_count, 1);
        assert_eq!(torn_down.runtime.active_timer_count, 0);
        assert!(!torn_down.runtime.pending_host_wake);
        let live = snapshot
            .live_session
            .expect("recovered reload should have a live session");
        assert_eq!(live.render_call_count, 1);
    }

    #[test]
    fn failed_reload_from_render_throw_keeps_live_tree_visible_and_recovers_after_fix() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        let panel_path = dir.path().join("panel.tsx");
        write_runtime_failure_entry_fixture(&entry_path);
        write_panel_fixture_healthy(&panel_path, "Healthy panel");

        let mut app = RuntimeJsxApp::new(&entry_path);
        let ctx = egui::Context::default();
        app.reload_from_disk(&ctx);

        app.dispatch_events_to_runtime(&[ContractEvent::new("toggle", EventKind::Clicked)], &ctx);
        app.wait_for_runtime_worker_idle(&ctx, Duration::from_secs(5));

        let tree = rendered_tree(&app);
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("On")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "panel").expect("panel label should exist")),
            Some("Healthy panel")
        );

        write_panel_fixture_render_throw(&panel_path);
        app.reload_from_disk(&ctx);

        assert!(
            app.debug_snapshot().live_session.is_some(),
            "failed reload should keep the previous live session"
        );
        assert!(
            app.rendered.is_some(),
            "failed reload should keep the previous rendered tree visible"
        );
        let error = app
            .error
            .as_deref()
            .expect("expected failed reload to surface an error");
        assert!(
            error.contains("render reload boom"),
            "expected render throw reason in reload error: {error}"
        );

        let tree = rendered_tree(&app);
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("On")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "panel").expect("panel label should exist")),
            Some("Healthy panel")
        );

        assert!(
            click_preview_toggle_until_status(&mut app, &ctx, "Off"),
            "render_preview should keep the stale tree interactive after a render-throw reload failure"
        );

        write_panel_fixture_healthy(&panel_path, "Recovered panel");
        app.reload_from_disk(&ctx);
        assert!(
            app.error.is_none(),
            "reload should recover once the runtime render throw is fixed"
        );

        let tree = rendered_tree(&app);
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("Off")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "panel").expect("panel label should exist")),
            Some("Recovered panel")
        );
    }

    #[test]
    fn failed_reload_from_effect_throw_keeps_live_tree_visible_and_recovers_after_fix() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        let panel_path = dir.path().join("panel.tsx");
        write_runtime_failure_entry_fixture(&entry_path);
        write_panel_fixture_healthy(&panel_path, "Healthy panel");

        let mut app = RuntimeJsxApp::new(&entry_path);
        let ctx = egui::Context::default();
        app.reload_from_disk(&ctx);

        app.dispatch_events_to_runtime(&[ContractEvent::new("toggle", EventKind::Clicked)], &ctx);
        app.wait_for_runtime_worker_idle(&ctx, Duration::from_secs(5));

        let tree = rendered_tree(&app);
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("On")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "panel").expect("panel label should exist")),
            Some("Healthy panel")
        );

        write_panel_fixture_effect_throw(&panel_path);
        app.reload_from_disk(&ctx);

        assert!(
            app.debug_snapshot().live_session.is_some(),
            "failed reload should keep the previous live session"
        );
        assert!(
            app.rendered.is_some(),
            "failed reload should keep the previous rendered tree visible"
        );
        let error = app
            .error
            .as_deref()
            .expect("expected failed reload to surface an error");
        assert!(
            error.contains("effect reload boom"),
            "expected effect throw reason in reload error: {error}"
        );

        let tree = rendered_tree(&app);
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("On")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "panel").expect("panel label should exist")),
            Some("Healthy panel")
        );

        assert!(
            click_preview_toggle_until_status(&mut app, &ctx, "Off"),
            "render_preview should keep the stale tree interactive after an effect-throw reload failure"
        );

        write_panel_fixture_healthy(&panel_path, "Recovered panel");
        app.reload_from_disk(&ctx);
        assert!(
            app.error.is_none(),
            "reload should recover once the runtime effect throw is fixed"
        );

        let tree = rendered_tree(&app);
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("Off")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "panel").expect("panel label should exist")),
            Some("Recovered panel")
        );
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
        assert!(
            snapshot.host.runtime_update_drain_count >= before.host.runtime_update_drain_count + 1,
            "timer-driven state should produce at least one visible drain"
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
    fn async_burst_runtime_updates_converge_to_latest_state_within_frame_drain_budget() {
        const BURST_CALLBACK_COUNT: usize = 257;
        const FRAME_DRAIN_BUDGET: usize = 2;
        const MAX_FRAMES_TO_CONVERGE: usize = 3;

        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("burst-async.tsx");
        std::fs::write(
            &entry_path,
            format!(
                r#"
import {{ render, useEffect, useState }} from "egui";

function App() {{
  const [count, setCount] = useState(0);
  useEffect(() => {{
    for (let next = 1; next <= {BURST_CALLBACK_COUNT}; next += 1) {{
      setTimeout(() => setCount(next), 0);
    }}
  }}, []);

  return <label id="status" text={{String(count)}} />;
}}

render(<App />);
"#,
            ),
        )
        .expect("entry file should be written");

        let mut app = RuntimeJsxApp::new(&entry_path);
        app.max_runtime_update_drains_per_frame = FRAME_DRAIN_BUDGET;
        let ctx = egui::Context::default();
        app.reload_from_disk(&ctx);
        app.initialized = true;

        let mut frame_statuses = Vec::new();
        let mut frame_drain_attempts = Vec::new();
        let mut converged_frame = None;

        for frame in 1..=MAX_FRAMES_TO_CONVERGE {
            let before = app.debug_snapshot().host;
            run_update_frame(&mut app, &ctx);
            let after = app.debug_snapshot().host;

            let frame_drain_attempts_this_frame = drain_attempt_delta(&before, &after);
            assert!(
                frame_drain_attempts_this_frame <= FRAME_DRAIN_BUDGET as u64,
                "frame {frame} exceeded the configured per-frame drain budget ({frame_drain_attempts_this_frame} > {FRAME_DRAIN_BUDGET})"
            );
            frame_drain_attempts.push(frame_drain_attempts_this_frame);

            let status = label_text(
                find_node(&rendered_tree(&app).root, "status")
                    .expect("status label should exist while advancing update frames"),
            )
            .unwrap_or("<missing>")
            .to_owned();
            frame_statuses.push(status.clone());
            if status == BURST_CALLBACK_COUNT.to_string() {
                converged_frame = Some(frame);
                break;
            }

            std::thread::sleep(Duration::from_millis(5));
        }

        assert!(
            frame_drain_attempts.iter().any(|attempts| *attempts > 0),
            "expected rendered preview frames to drain async runtime work; observed attempts={frame_drain_attempts:?}, statuses={frame_statuses:?}"
        );

        let converged_frame = converged_frame.unwrap_or_else(|| {
            panic!(
                "runtime did not converge to latest visible burst value within {MAX_FRAMES_TO_CONVERGE} rendered frames; statuses={frame_statuses:?}, drain attempts={frame_drain_attempts:?}"
            )
        });
        assert!(
            converged_frame <= MAX_FRAMES_TO_CONVERGE,
            "latest burst value should converge within {MAX_FRAMES_TO_CONVERGE} rendered frames"
        );
        assert_eq!(
            frame_statuses.last().map(String::as_str),
            Some("257"),
            "preview-rendered status text should show the latest burst value once convergence completes"
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

    fn run_preview_frame(app: &mut RuntimeJsxApp, ctx: &egui::Context, input: RawInput) {
        let _ = ctx.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                app.render_preview(ui);
            });
        });
    }

    fn run_update_frame(app: &mut RuntimeJsxApp, ctx: &egui::Context) {
        let _ = ctx.run(RawInput::default(), |context| {
            app.update_frame(context);
        });
    }

    fn drain_attempt_delta(
        before: &super::super::metrics::ExampleHostMetrics,
        after: &super::super::metrics::ExampleHostMetrics,
    ) -> u64 {
        (after.runtime_update_drain_count - before.runtime_update_drain_count)
            + (after.runtime_update_drain_empty_count - before.runtime_update_drain_empty_count)
    }

    fn click_preview_toggle_until_status(
        app: &mut RuntimeJsxApp,
        ctx: &egui::Context,
        expected_status: &str,
    ) -> bool {
        run_preview_frame(app, ctx, RawInput::default());
        for y in (56..=360).step_by(24) {
            for x in (24..=360).step_by(24) {
                let position = pos2(x as f32, y as f32);
                run_preview_frame(app, ctx, press_at(position));
                run_preview_frame(app, ctx, release_at(position));
                app.wait_for_runtime_worker_idle(ctx, Duration::from_secs(2));

                let status = label_text(
                    find_node(&rendered_tree(app).root, "status")
                        .expect("status label should exist while probing UI clicks"),
                );
                if status == Some(expected_status) {
                    return true;
                }
            }
        }

        false
    }

    fn press_at(position: egui::Pos2) -> RawInput {
        RawInput {
            events: vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..RawInput::default()
        }
    }

    fn release_at(position: egui::Pos2) -> RawInput {
        RawInput {
            events: vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..RawInput::default()
        }
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

    fn write_runtime_failure_entry_fixture(entry_path: &Path) {
        std::fs::write(
            entry_path,
            r#"
import { render, useState } from "egui";
import { FaultPanel } from "./panel.tsx";

function App() {
  const [enabled, setEnabled] = useState(false);
  return (
    <div id="root" data-slot="column">
      <button id="toggle" label="Toggle" onClick={() => setEnabled((value) => !value)} />
      <label id="status" text={enabled ? "On" : "Off"} />
      <FaultPanel />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("entry fixture should be written");
    }

    fn write_panel_fixture_healthy(panel_path: &Path, text: &str) {
        std::fs::write(
            panel_path,
            format!(
                r#"
export function FaultPanel() {{
  return <label id="panel" text={text:?} />;
}}
"#,
            ),
        )
        .expect("healthy panel fixture should be written");
    }

    fn write_panel_fixture_render_throw(panel_path: &Path) {
        std::fs::write(
            panel_path,
            r#"
export function FaultPanel() {
  throw new Error("render reload boom");
}
"#,
        )
        .expect("render-throw panel fixture should be written");
    }

    fn write_panel_fixture_effect_throw(panel_path: &Path) {
        std::fs::write(
            panel_path,
            r#"
import { useEffect } from "egui";

export function FaultPanel() {
  useEffect(() => {
    throw new Error("effect reload boom");
  }, []);
  return <label id="panel" text="Broken panel" />;
}
"#,
        )
        .expect("effect-throw panel fixture should be written");
    }
}
