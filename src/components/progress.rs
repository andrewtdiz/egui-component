use super::api::ComponentUi;
use crate::ui::tokens;
use egui::{Response, Ui};

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
        draw_progress(self.raw_mut(), value, props.into())
    }
}

fn draw_progress(ui: &mut Ui, value: f32, props: Progress) -> Response {
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
