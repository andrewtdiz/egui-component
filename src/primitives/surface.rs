use egui::{Color32, CornerRadius, InnerResponse, Margin, Shadow, Stroke, Ui};

#[derive(Debug, Clone, Copy)]
pub struct SurfaceFrame {
    fill: Color32,
    stroke: Stroke,
    corner_radius: u8,
    padding_x: i8,
    padding_y: i8,
    shadow: Shadow,
}

impl SurfaceFrame {
    pub fn new(fill: Color32, stroke: Stroke) -> Self {
        Self {
            fill,
            stroke,
            corner_radius: 0,
            padding_x: 0,
            padding_y: 0,
            shadow: Shadow::NONE,
        }
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

    pub fn shadow(mut self, shadow: Shadow) -> Self {
        self.shadow = shadow;
        self
    }
}

pub fn surface_frame<R>(
    ui: &mut Ui,
    frame: SurfaceFrame,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    surface_frame_builder(frame).show(ui, add)
}

pub fn surface_frame_builder(frame: SurfaceFrame) -> egui::Frame {
    egui::Frame::new()
        .fill(frame.fill)
        .stroke(frame.stroke)
        .corner_radius(CornerRadius::same(frame.corner_radius))
        .inner_margin(Margin::symmetric(frame.padding_x, frame.padding_y))
        .shadow(frame.shadow)
}
