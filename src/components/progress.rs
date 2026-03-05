use crate::ui::tokens;
use egui::{Response, Ui};

#[derive(Debug, Clone, Copy)]
pub struct ProgressProps {
    pub width: f32,
    pub height: f32,
}

impl ProgressProps {
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

pub fn progress(ui: &mut Ui, value: f32, props: ProgressProps) -> Response {
    ui.scope(|ui| {
        let dark_mode = ui.visuals().dark_mode;
        ui.visuals_mut().extreme_bg_color = tokens::INPUT_BACKGROUND;
        ui.add_sized(
            [props.width, props.height],
            egui::ProgressBar::new(value.clamp(0.0, 1.0))
                .text("")
                .fill(tokens::primary_bg(dark_mode)),
        )
    })
    .inner
}
