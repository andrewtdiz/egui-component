use super::api::ComponentUi;
use crate::ui::tokens;
use egui::{CornerRadius, Response, Sense, Stroke, StrokeKind, Ui};

const PROGRESS_ANIMATION_SECS: f32 = 0.2;
const PROGRESS_EPSILON: f32 = 0.000_1;

#[derive(Debug, Clone, Copy)]
struct ProgressAnimationState {
    from: f32,
    target: f32,
    started_at: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Progress {
    pub width: f32,
    pub height: f32,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            width: 188.0,
            height: 10.0,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl From<()> for Progress {
    fn from(_: ()) -> Self {
        Self::new()
    }
}

impl From<f32> for Progress {
    fn from(width: f32) -> Self {
        Self::new().width(width)
    }
}

impl From<(f32, f32)> for Progress {
    fn from((width, height): (f32, f32)) -> Self {
        Self::new().width(width).height(height)
    }
}

impl ComponentUi<'_> {
    pub fn progress(&mut self, value: f32, props: impl Into<Progress>) -> Response {
        draw_progress(self.ui_mut(), value, props.into())
    }
}

fn draw_progress(ui: &mut Ui, value: f32, props: Progress) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let target_value = value.clamp(0.0, 1.0);
    let animation_id = ui.auto_id_with("progress_value");
    let now = ui.input(|input| input.time);
    let animated_value = ui.ctx().data_mut(|data| {
        let mut state = data
            .get_temp::<ProgressAnimationState>(animation_id)
            .unwrap_or(ProgressAnimationState {
                from: target_value,
                target: target_value,
                started_at: now,
            });

        let current_value = progress_animation_value(state, now);
        if (state.target - target_value).abs() > PROGRESS_EPSILON {
            state = ProgressAnimationState {
                from: current_value,
                target: target_value,
                started_at: now,
            };
        }

        let value = progress_animation_value(state, now);
        data.insert_temp(animation_id, state);
        value
    });
    let desired_size = egui::vec2(props.width.max(1.0), props.height.max(2.0));
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
    let radius = ((rect.height() * 0.5).round()).clamp(0.0, 255.0) as u8;

    if (animated_value - target_value).abs() > PROGRESS_EPSILON {
        ui.ctx().request_repaint();
    }

    ui.painter().rect(
        rect,
        CornerRadius::same(radius),
        tokens::input_background(runtime),
        Stroke::new(1.0, tokens::input_border(runtime)),
        StrokeKind::Outside,
    );

    if animated_value > 0.0 {
        let fill_width = (rect.width() * animated_value).clamp(1.0, rect.width());
        let fill_rect = egui::Rect::from_min_max(
            rect.left_top(),
            egui::pos2(rect.left() + fill_width, rect.bottom()),
        );
        let fill_radius =
            ((fill_rect.width().min(fill_rect.height()) * 0.5).round()).clamp(0.0, 255.0) as u8;
        ui.painter().rect(
            fill_rect,
            CornerRadius::same(fill_radius),
            tokens::primary_bg(runtime),
            Stroke::NONE,
            StrokeKind::Outside,
        );
    }

    response
}

fn progress_animation_value(state: ProgressAnimationState, now: f64) -> f32 {
    let elapsed = ((now - state.started_at) as f32 / PROGRESS_ANIMATION_SECS).clamp(0.0, 1.0);
    let eased = quadratic_out(elapsed);
    egui::lerp(state.from..=state.target, eased)
}

fn quadratic_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}
