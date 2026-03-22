use super::api::ComponentUi;
use crate::ui::tokens;
use egui::{CornerRadius, Response, Sense, Stroke, StrokeKind, Ui};

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
    let value = value.clamp(0.0, 1.0);
    let desired_size = egui::vec2(props.width.max(1.0), props.height.max(2.0));
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
    let radius = ((rect.height() * 0.5).round()).clamp(0.0, 255.0) as u8;

    ui.painter().rect(
        rect,
        CornerRadius::same(radius),
        tokens::input_background(runtime),
        Stroke::new(1.0, tokens::input_border(runtime)),
        StrokeKind::Outside,
    );

    if value > 0.0 {
        let fill_width = (rect.width() * value).clamp(1.0, rect.width());
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
