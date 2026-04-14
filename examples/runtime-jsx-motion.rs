use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

#[path = "runtime-jsx/watch.rs"]
mod runtime_jsx_watch;

use clay_jsx_egui_bridge::{
    extend_logs, HotReloadState, JsxRuntimeLoadOutcome, JsxRuntimeSession, MotionFrame,
    MotionProperty, MotionValues, RuntimeLogBuffer,
};
use eframe::egui::{
    self, pos2, vec2, Align2, CentralPanel, Color32, Context, CornerRadius, FontId, Pos2, Rect,
    Sense, Shape, Stroke, StrokeKind, TopBottomPanel, Vec2, ViewportBuilder,
};
use egui_component::{
    contract::{render_tree, ContractEvent, ContractTree, NodeId},
    theme::{self, BaseColor, ThemeMode, ThemeSpec},
};
use runtime_jsx_watch::ReloadWatcher;

const WINDOW_TITLE: &str = "egui-component JSX Motion Sync";
const WINDOW_INNER_SIZE: [f32; 2] = [1120.0, 780.0];

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
    hot_reload_state: Option<HotReloadState>,
    reload_watcher: Option<ReloadWatcher>,
    session: Option<JsxRuntimeSession>,
    rendered: Option<ContractTree>,
    motion: MotionFrame,
    logs: RuntimeLogBuffer,
    watched_files: BTreeSet<PathBuf>,
    error: Option<String>,
}

impl Default for MotionSyncApp {
    fn default() -> Self {
        Self {
            entry_path: default_entry_path(),
            hot_reload_state: None,
            reload_watcher: None,
            session: None,
            rendered: None,
            motion: MotionFrame::default(),
            logs: RuntimeLogBuffer::new(),
            watched_files: BTreeSet::new(),
            error: None,
        }
    }
}

impl eframe::App for MotionSyncApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        theme::set_theme(ctx, ThemeSpec::preset(BaseColor::Neutral));
        theme::set_mode(ctx, ThemeMode::System);
        self.reload_external_changes(ctx);
        self.drain_pending_runtime_updates(ctx);
        self.tick_motion(ctx);

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
}

impl MotionSyncApp {
    fn render_jsx_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Reload TSX").clicked() {
                self.reload_runtime(ui.ctx());
            }

            if self.motion.active {
                ui.label("motion active");
            } else {
                ui.label("motion idle");
            }
        });

        if let Some(error) = &self.error {
            ui.colored_label(Color32::from_rgb(185, 28, 28), error);
            return;
        }

        let events = self
            .rendered
            .as_ref()
            .map(|tree| render_tree(ui, tree))
            .unwrap_or_default();
        if !events.is_empty() {
            self.dispatch_events(&events, ui.ctx());
        }
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
        if self.session.is_none() && self.rendered.is_none() && self.error.is_none() {
            self.reload_runtime(ctx);
            return;
        }
        if self
            .reload_watcher
            .as_mut()
            .is_some_and(|watcher| watcher.take_pending_reload())
        {
            self.reload_runtime(ctx);
        }
    }

    fn reload_runtime(&mut self, ctx: &Context) {
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
                install_session_wake_callback(&mut session, ctx);
                self.session = Some(session);
                self.rendered = rendered.tree;
                self.motion = rendered.motion;
                self.logs.clear();
                extend_logs(&mut self.logs, rendered.logs);
                self.error = None;
                if self.motion.active {
                    ctx.request_repaint();
                }
                self.drain_pending_runtime_updates(ctx);
            }
            JsxRuntimeLoadOutcome::Failed(failure) => {
                self.session = None;
                self.rendered = None;
                self.motion = MotionFrame::default();
                self.logs.clear();
                self.error = Some(failure.error.to_string());
                self.set_watched_files(collect_failure_tracked_files(
                    failure.dependency_paths,
                    &self.entry_path,
                ));
            }
        }
    }

    fn dispatch_events(&mut self, events: &[ContractEvent], ctx: &Context) {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        match session.dispatch_events(events) {
            Ok(rendered) => {
                if let Some(tree) = rendered.tree {
                    self.rendered = Some(tree);
                }
                self.motion = rendered.motion;
                extend_logs(&mut self.logs, rendered.logs);
                self.error = None;
                if self.motion.active {
                    ctx.request_repaint();
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
                if let Some(tree) = rendered.tree {
                    self.rendered = Some(tree);
                }
                self.motion = rendered.motion;
                extend_logs(&mut self.logs, rendered.logs);
                self.error = None;
                if self.motion.active {
                    ctx.request_repaint();
                }
            }
            Ok(None) => {}
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }

    fn tick_motion(&mut self, ctx: &Context) {
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
                extend_logs(&mut self.logs, rendered.logs);
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
        match ReloadWatcher::new(ctx) {
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
        }
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
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("runtime-jsx")
        .join("motion-sync.tsx")
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

fn install_session_wake_callback(session: &mut JsxRuntimeSession, ctx: &Context) {
    let ctx = ctx.clone();
    session.set_wake_callback(move || ctx.request_repaint());
}

#[cfg(test)]
mod tests {
    use super::default_entry_path;
    use clay_jsx_egui_bridge::{JsxRuntimeSession, MotionProperty};
    use egui_component::contract::{ContractEvent, EventKind, NodeId};

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
        assert!(rendered.motion.active);
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
}
