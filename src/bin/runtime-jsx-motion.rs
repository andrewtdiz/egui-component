use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};

#[path = "../host/mod.rs"]
mod host;

use clay_jsx_egui_bridge::{
    extend_logs, JsxRuntimeDebugMetrics, JsxRuntimeSessionWorker, JsxRuntimeWorkerDebugSnapshot,
    JsxRuntimeWorkerEvent, JsxRuntimeWorkerReloadOutcome, MotionFrame, MotionProperty,
    MotionValues, RuntimeLogBuffer,
};
#[cfg(not(test))]
use eframe::egui::ViewportBuilder;
use eframe::egui::{
    self, pos2, vec2, Align2, CentralPanel, Color32, Context, CornerRadius, FontId, Pos2, Rect,
    Sense, Shape, Stroke, StrokeKind, TopBottomPanel, Vec2,
};
use egui_component::{
    contract::{render_tree, ContractEvent, ContractTree, NodeId},
    theme::{self, BaseColor, ThemeMode, ThemeSpec},
};
#[cfg(test)]
use host::ExampleHostDebugSnapshot;
use host::{request_host_repaint, ExampleHostMetricsTracker, ReloadWatcher};

#[cfg(not(test))]
const WINDOW_TITLE: &str = "egui-component JSX Motion Sync";
#[cfg(not(test))]
const WINDOW_INNER_SIZE: [f32; 2] = [1120.0, 780.0];

#[cfg(not(test))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size(WINDOW_INNER_SIZE),
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_TITLE,
        options,
        Box::new(|creation_context| {
            theme::install(
                &creation_context.egui_ctx,
                ThemeSpec::preset(BaseColor::Neutral),
                ThemeMode::System,
            );
            Ok(Box::new(MotionSyncApp::default()))
        }),
    )
}

#[derive(Debug)]
struct MotionSyncApp {
    entry_path: PathBuf,
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
    logs: RuntimeLogBuffer,
    watched_files: BTreeSet<PathBuf>,
    error: Option<String>,
    metrics: ExampleHostMetricsTracker,
    live_session_metrics: Option<JsxRuntimeDebugMetrics>,
    last_torn_down_session: Option<JsxRuntimeDebugMetrics>,
}

impl Default for MotionSyncApp {
    fn default() -> Self {
        let max_runtime_update_drains_per_frame = host::runtime_update_drain_frame_budget();
        Self {
            entry_path: default_entry_path(),
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
            logs: RuntimeLogBuffer::new(),
            watched_files: BTreeSet::new(),
            error: None,
            metrics: ExampleHostMetricsTracker::default(),
            live_session_metrics: None,
            last_torn_down_session: None,
        }
    }
}

impl eframe::App for MotionSyncApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_frame(ctx);
    }
}

impl MotionSyncApp {
    fn update_frame(&mut self, ctx: &Context) {
        theme::set_theme(ctx, ThemeSpec::preset(BaseColor::Neutral));
        theme::set_mode(ctx, ThemeMode::System);
        self.reset_runtime_update_drain_frame_budget();
        self.ensure_runtime_worker(ctx);
        self.collect_worker_events(ctx);
        self.reload_external_changes(ctx);
        self.drive_runtime_update_drain_loop(ctx);
        self.schedule_tick_motion(ctx);
        self.collect_worker_events(ctx);

        TopBottomPanel::top("motion_sync_topbar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("JSX motion synced to egui");
                    ui.separator();
                    ui.label(self.entry_path.display().to_string());
                });
            });

        CentralPanel::default().show(ctx, |ui| {
            ui.add_space(12.0);
            ui.label(
                "The control below is authored in TSX. Clicking it updates JSX hook state in V8, \
                 commits retained motion specs back to Rust, and egui ticks the numeric values \
                 each frame.",
            );
            ui.add_space(10.0);

            self.render_jsx_controls(ui);
            ui.add_space(16.0);
            self.render_motion_grid(ui);
            ui.add_space(12.0);
            self.render_diagnostics(ui);
        });
    }

    fn render_jsx_controls(&mut self, ui: &mut egui::Ui) {
        self.collect_worker_events(ui.ctx());
        ui.horizontal(|ui| {
            if ui.button("Reload TSX").clicked() {
                self.request_reload_runtime(ui.ctx());
            }

            if self.motion.active {
                ui.label("motion active");
            } else {
                ui.label("motion idle");
            }
        });

        if let Some(error) = &self.error {
            ui.colored_label(Color32::from_rgb(185, 28, 28), error);
        }

        let events = self
            .rendered
            .as_ref()
            .map(|tree| render_tree(ui, tree))
            .unwrap_or_default();
        if !events.is_empty() {
            self.dispatch_events(&events, ui.ctx());
        }

        self.collect_worker_events(ui.ctx());
    }

    fn render_motion_grid(&self, ui: &mut egui::Ui) {
        let card_height = 178.0;
        ui.columns(2, |columns| {
            draw_motion_card(
                &mut columns[0],
                &self.motion,
                MotionCardSpec {
                    node_id: "motion-opacity",
                    title: "Opacity",
                    accent: Color32::from_rgb(67, 118, 196),
                    help: "alpha",
                },
                card_height,
            );
            draw_motion_card(
                &mut columns[1],
                &self.motion,
                MotionCardSpec {
                    node_id: "motion-translate",
                    title: "Transform translation",
                    accent: Color32::from_rgb(218, 96, 74),
                    help: "x/y offset",
                },
                card_height,
            );
        });
        ui.add_space(10.0);
        ui.columns(2, |columns| {
            draw_motion_card(
                &mut columns[0],
                &self.motion,
                MotionCardSpec {
                    node_id: "motion-scale",
                    title: "Scale",
                    accent: Color32::from_rgb(62, 150, 98),
                    help: "uniform scale",
                },
                card_height,
            );
            draw_motion_card(
                &mut columns[1],
                &self.motion,
                MotionCardSpec {
                    node_id: "motion-rotate",
                    title: "Rotation",
                    accent: Color32::from_rgb(168, 92, 172),
                    help: "radians",
                },
                card_height,
            );
        });
    }

    fn render_diagnostics(&self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "retained motion nodes: {}",
                self.motion.values.len()
            ));
            ui.separator();
            ui.label(format!("active: {}", self.motion.active));
        });
        for log in self.logs.iter().rev().take(4) {
            ui.label(log);
        }
    }

    fn reload_external_changes(&mut self, ctx: &Context) {
        self.ensure_reload_watcher(ctx);
        if self.live_session_metrics.is_none() && self.rendered.is_none() && self.error.is_none() {
            self.request_reload_runtime(ctx);
            return;
        }
        if self
            .reload_watcher
            .as_mut()
            .is_some_and(|watcher| watcher.take_pending_reload())
        {
            self.request_reload_runtime(ctx);
        }
    }

    #[cfg(test)]
    fn reload_runtime(&mut self, ctx: &Context) {
        self.ensure_runtime_worker(ctx);
        self.request_reload_runtime(ctx);
        self.wait_for_runtime_worker_idle(ctx, std::time::Duration::from_secs(10));
    }

    fn dispatch_events(&mut self, events: &[ContractEvent], ctx: &Context) {
        if events.is_empty() {
            return;
        }
        self.queued_dispatch_events.extend_from_slice(events);
        self.try_dispatch_queued_events(ctx);
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

    fn request_reload_runtime(&mut self, ctx: &Context) {
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

    fn schedule_tick_motion(&mut self, ctx: &Context) {
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
                        self.logs.clear();
                        self.apply_rendered_update(rendered, ctx);
                        self.error = None;
                        self.metrics.note_reload_success();
                    }
                    JsxRuntimeWorkerReloadOutcome::Failed(failure) => {
                        self.error = Some(failure.error.to_string());
                        self.set_watched_files(collect_failure_tracked_files(
                            failure.dependency_paths,
                            &self.entry_path,
                        ));
                        self.metrics.note_reload_failure();
                        if self.live_session_metrics.is_none() {
                            self.rendered = None;
                            self.motion = MotionFrame::default();
                            self.logs.clear();
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
        extend_logs(&mut self.logs, rendered.logs);
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
    fn debug_snapshot(&self) -> ExampleHostDebugSnapshot {
        ExampleHostDebugSnapshot {
            host: self.metrics.snapshot(),
            live_session: self.live_session_metrics.clone(),
            last_torn_down_session: self.last_torn_down_session.clone(),
        }
    }
}

impl Drop for MotionSyncApp {
    fn drop(&mut self) {
        self.shutdown_runtime_worker();
    }
}

#[derive(Clone, Copy)]
struct MotionCardSpec {
    node_id: &'static str,
    title: &'static str,
    accent: Color32,
    help: &'static str,
}

fn draw_motion_card(ui: &mut egui::Ui, frame: &MotionFrame, spec: MotionCardSpec, height: f32) {
    let width = ui.available_width().max(260.0);
    let (rect, _response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let painter = ui.painter_at(rect);
    let radius = CornerRadius::same(8);

    painter.rect_filled(rect, radius, Color32::from_rgb(248, 248, 246));
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, Color32::from_rgb(214, 218, 224)),
        StrokeKind::Inside,
    );

    painter.text(
        rect.left_top() + vec2(14.0, 12.0),
        Align2::LEFT_TOP,
        spec.title,
        FontId::proportional(16.0),
        Color32::from_rgb(28, 33, 40),
    );
    painter.text(
        rect.left_top() + vec2(14.0, 34.0),
        Align2::LEFT_TOP,
        spec.help,
        FontId::proportional(12.0),
        Color32::from_rgb(93, 101, 112),
    );

    let values = frame.values.get(&NodeId::from(spec.node_id));
    draw_reference_track(&painter, rect);
    draw_transformed_shape(&painter, rect, spec.accent, values);
    draw_motion_readout(&painter, rect, values);
}

fn draw_reference_track(painter: &egui::Painter, rect: Rect) {
    let center = rect.center() + vec2(0.0, 12.0);
    let left = center + vec2(-82.0, 0.0);
    let right = center + vec2(82.0, 0.0);
    painter.line_segment(
        [left, right],
        Stroke::new(1.0, Color32::from_rgb(190, 198, 207)),
    );
    painter.circle_filled(left, 3.0, Color32::from_rgb(190, 198, 207));
    painter.circle_filled(right, 3.0, Color32::from_rgb(190, 198, 207));
}

fn draw_transformed_shape(
    painter: &egui::Painter,
    rect: Rect,
    accent: Color32,
    values: Option<&MotionValues>,
) {
    let opacity = motion_value(values, MotionProperty::Opacity, 1.0).clamp(0.04, 1.0);
    let x = motion_value(values, MotionProperty::X, 0.0);
    let y = motion_value(values, MotionProperty::Y, 0.0);
    let scale = motion_value(values, MotionProperty::Scale, 1.0).max(0.05);
    let scale_x = motion_value(values, MotionProperty::ScaleX, scale).max(0.05);
    let scale_y = motion_value(values, MotionProperty::ScaleY, scale).max(0.05);
    let rotate = motion_value(values, MotionProperty::Rotate, 0.0);
    let center = rect.center() + vec2(x, y + 12.0);
    let half_size = vec2(36.0, 28.0);
    let corners = [
        vec2(-half_size.x, -half_size.y),
        vec2(half_size.x, -half_size.y),
        vec2(half_size.x, half_size.y),
        vec2(-half_size.x, half_size.y),
    ]
    .map(|local| transform_point(center, local, scale_x, scale_y, rotate))
    .to_vec();

    painter.add(Shape::convex_polygon(
        corners,
        accent.gamma_multiply(opacity),
        Stroke::new(
            2.0,
            Color32::from_rgba_unmultiplied(17, 24, 39, (190.0 * opacity) as u8),
        ),
    ));

    painter.circle_filled(
        center,
        3.0,
        Color32::from_rgba_unmultiplied(17, 24, 39, 180),
    );
}

fn draw_motion_readout(painter: &egui::Painter, rect: Rect, values: Option<&MotionValues>) {
    let readout = format!(
        "opacity {:.2}   x {:.0}   y {:.0}   scale {:.2}   rotate {:.2}",
        motion_value(values, MotionProperty::Opacity, 1.0),
        motion_value(values, MotionProperty::X, 0.0),
        motion_value(values, MotionProperty::Y, 0.0),
        motion_value(values, MotionProperty::Scale, 1.0),
        motion_value(values, MotionProperty::Rotate, 0.0),
    );
    painter.text(
        rect.left_bottom() + vec2(14.0, -14.0),
        Align2::LEFT_BOTTOM,
        readout,
        FontId::monospace(11.0),
        Color32::from_rgb(82, 89, 101),
    );
}

fn transform_point(center: Pos2, local: Vec2, scale_x: f32, scale_y: f32, rotate: f32) -> Pos2 {
    let scaled = vec2(local.x * scale_x, local.y * scale_y);
    let (sin, cos) = rotate.sin_cos();
    pos2(
        center.x + scaled.x * cos - scaled.y * sin,
        center.y + scaled.x * sin + scaled.y * cos,
    )
}

fn motion_value(values: Option<&MotionValues>, property: MotionProperty, fallback: f32) -> f32 {
    values
        .and_then(|values| values.get(property))
        .unwrap_or(fallback)
}

fn default_entry_path() -> PathBuf {
    host::default_entry_path().with_file_name("motion-sync.tsx")
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
    use std::time::Duration;

    use super::{default_entry_path, MotionSyncApp};
    use clay_jsx_egui_bridge::{JsxRuntimeSession, MotionProperty};
    use clay_jsx_runtime::contract::{ContractEvent, ContractNode, EventKind, NodeId};
    use eframe::egui::{pos2, CentralPanel, Event, Modifiers, PointerButton, RawInput};
    use egui_component::contract::ContractTree;
    use tempfile::tempdir;

    #[test]
    fn motion_sync_tsx_loads_and_retargets_all_demo_values() {
        let (mut session, rendered) =
            JsxRuntimeSession::load(&default_entry_path()).expect("motion sync TSX should render");
        assert!(rendered.tree.is_some());
        assert_eq!(
            rendered
                .motion
                .values
                .get(&NodeId::from("motion-opacity"))
                .and_then(|values| values.get(MotionProperty::Opacity)),
            Some(0.22)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("motion-toggle", EventKind::Clicked)])
            .expect("toggle should retarget motion values");
        assert!(!rendered.motion.active);
        assert!(rendered
            .motion
            .values
            .contains_key(&NodeId::from("motion-scale")));
        assert!(rendered
            .motion
            .values
            .contains_key(&NodeId::from("motion-rotate")));
        assert!(rendered
            .motion
            .values
            .contains_key(&NodeId::from("motion-translate")));
    }

    #[test]
    fn motion_host_metrics_track_reloads_and_repaint_requests() {
        let ctx = egui::Context::default();
        let mut app = MotionSyncApp::default();
        app.reload_runtime(&ctx);

        let before = app.debug_snapshot();
        assert_eq!(before.host.reload_attempt_count, 1);
        assert_eq!(before.host.reload_success_count, 1);

        app.dispatch_events(
            &[ContractEvent::new("motion-toggle", EventKind::Clicked)],
            &ctx,
        );
        app.wait_for_runtime_worker_idle(&ctx, std::time::Duration::from_secs(5));

        let after = app.debug_snapshot();
        assert!(
            after.host.repaint_request_count > before.host.repaint_request_count,
            "retargeting motion should request repaint"
        );
        let live = after
            .live_session
            .expect("motion host should retain a live session");
        assert!(live.contract_tree_materialization_count >= 1);
        assert_eq!(live.runtime.active_timer_count, 0);
    }

    #[test]
    fn async_burst_runtime_updates_converge_to_latest_state_within_frame_drain_budget() {
        const BURST_CALLBACK_COUNT: usize = 257;
        const FRAME_DRAIN_BUDGET: usize = 2;
        const MAX_FRAMES_TO_CONVERGE: usize = 3;

        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("motion-burst.tsx");
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

        let mut app = MotionSyncApp::default();
        app.entry_path = entry_path;
        app.max_runtime_update_drains_per_frame = FRAME_DRAIN_BUDGET;
        let ctx = egui::Context::default();
        app.reload_runtime(&ctx);

        let mut frame_statuses = Vec::new();
        let mut frame_drain_attempts = Vec::new();
        let mut converged_frame = None;

        for frame in 1..=MAX_FRAMES_TO_CONVERGE {
            let before = app.debug_snapshot().host;
            run_update_frame(&mut app, &ctx);
            let after = app.debug_snapshot().host;

            let frame_drain_attempts_this_frame = (after.runtime_update_drain_count
                - before.runtime_update_drain_count)
                + (after.runtime_update_drain_empty_count
                    - before.runtime_update_drain_empty_count);
            assert!(
                frame_drain_attempts_this_frame <= FRAME_DRAIN_BUDGET as u64,
                "frame {frame} exceeded the configured per-frame drain budget ({frame_drain_attempts_this_frame} > {FRAME_DRAIN_BUDGET})"
            );
            frame_drain_attempts.push(frame_drain_attempts_this_frame);

            let status = label_text(
                find_node(&rendered_tree(&app).root, "status")
                    .expect("status label should exist while advancing motion frames"),
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
            "expected rendered controls frames to drain async runtime work; observed attempts={frame_drain_attempts:?}, statuses={frame_statuses:?}"
        );

        let converged_frame = converged_frame.unwrap_or_else(|| {
            panic!(
                "motion controls did not converge to latest visible burst value within {MAX_FRAMES_TO_CONVERGE} rendered frames; statuses={frame_statuses:?}, drain attempts={frame_drain_attempts:?}"
            )
        });
        assert!(
            converged_frame <= MAX_FRAMES_TO_CONVERGE,
            "latest burst value should converge within {MAX_FRAMES_TO_CONVERGE} rendered controls frames"
        );
        assert_eq!(
            frame_statuses.last().map(String::as_str),
            Some("257"),
            "controls-rendered status text should show the latest burst value once convergence completes"
        );
    }

    #[test]
    fn failed_reload_keeps_previous_tree_visible_and_interactive_until_recovery() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("motion.tsx");
        let child_path = dir.path().join("copy.tsx");
        let extra_path = dir.path().join("nested").join("runtime").join("extra.tsx");

        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";
import { Copy } from "./copy.tsx";

function App() {
  const [active, setActive] = useState(false);
  return (
    <div id="root" data-slot="column">
      <button id="toggle" label="Toggle" onClick={() => setActive((value) => !value)} />
      <label id="status" text={active ? "On" : "Off"} />
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

        let mut app = MotionSyncApp::default();
        app.entry_path = entry_path;
        let ctx = egui::Context::default();
        app.reload_runtime(&ctx);
        app.ensure_reload_watcher(&ctx);
        assert!(
            app.reload_watcher.is_some(),
            "expected reload watcher to be active for missing-import fallback coverage"
        );

        app.dispatch_events(&[ContractEvent::new("toggle", EventKind::Clicked)], &ctx);
        app.wait_for_runtime_worker_idle(&ctx, std::time::Duration::from_secs(5));
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

        app.reload_runtime(&ctx);
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

        run_controls_frame(&mut app, &ctx, RawInput::default());
        assert!(
            app.error.is_some(),
            "render_jsx_controls should keep the reload error banner visible before interaction"
        );

        assert!(
            click_controls_toggle_until_status(&mut app, &ctx, "Off"),
            "render_jsx_controls should keep the stale tree interactive via UI-originated events while the reload error is shown"
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

        app.reload_runtime(&ctx);
        assert!(
            app.error.is_none(),
            "reload should recover once dependency exists"
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
    }

    fn rendered_tree(app: &MotionSyncApp) -> &ContractTree {
        app.rendered.as_ref().unwrap_or_else(|| {
            panic!(
                "expected app to have a rendered tree, error: {:?}",
                app.error
            )
        })
    }

    fn run_controls_frame(app: &mut MotionSyncApp, ctx: &egui::Context, input: RawInput) {
        let _ = ctx.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                app.render_jsx_controls(ui);
            });
        });
    }

    fn run_update_frame(app: &mut MotionSyncApp, ctx: &egui::Context) {
        let _ = ctx.run(RawInput::default(), |context| {
            app.update_frame(context);
        });
    }

    fn click_controls_toggle_until_status(
        app: &mut MotionSyncApp,
        ctx: &egui::Context,
        expected_status: &str,
    ) -> bool {
        run_controls_frame(app, ctx, RawInput::default());
        for y in (48..=600).step_by(24) {
            for x in (24..=600).step_by(24) {
                let position = pos2(x as f32, y as f32);
                run_controls_frame(app, ctx, press_at(position));
                run_controls_frame(app, ctx, release_at(position));
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
}
