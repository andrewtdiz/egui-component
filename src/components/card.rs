use crate::ui::tokens;
use egui::{Color32, CornerRadius, Margin, Stroke, Ui};

#[derive(Debug, Clone, Copy)]
pub struct CardProps {
    pub fill: Color32,
    pub stroke: Stroke,
    pub corner_radius: u8,
    pub padding_x: i8,
    pub padding_y: i8,
}

impl CardProps {
    pub fn new() -> Self {
        Self {
            fill: tokens::MUTED_SURFACE,
            stroke: Stroke::new(1.0, tokens::SEPARATOR),
            corner_radius: tokens::RADIUS_LG,
            padding_x: 12,
            padding_y: 12,
        }
    }

    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = fill;
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    pub fn corner_radius(mut self, corner_radius: u8) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn padding(mut self, x: i8, y: i8) -> Self {
        self.padding_x = x;
        self.padding_y = y;
        self
    }
}

impl Default for CardProps {
    fn default() -> Self {
        Self::new()
    }
}

pub fn card<R>(
    ui: &mut Ui,
    props: CardProps,
    add: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Frame::new()
        .fill(props.fill)
        .stroke(props.stroke)
        .corner_radius(CornerRadius::same(props.corner_radius))
        .inner_margin(Margin::symmetric(props.padding_x, props.padding_y))
        .show(ui, add)
}
