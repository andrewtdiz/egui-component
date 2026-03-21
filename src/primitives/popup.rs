use egui::{InnerResponse, Margin, Ui};

#[derive(Debug, Clone, Copy)]
pub struct PopupPanel {
    width: f32,
    padding_x: i8,
    padding_y: i8,
}

impl PopupPanel {
    pub fn new(width: f32) -> Self {
        Self {
            width,
            padding_x: 0,
            padding_y: 0,
        }
    }

    pub fn padding(mut self, x: i8, y: i8) -> Self {
        self.padding_x = x;
        self.padding_y = y;
        self
    }
}

pub fn popup_panel<R>(
    ui: &mut Ui,
    panel: PopupPanel,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    ui.set_min_width(panel.width);
    ui.set_max_width(panel.width);
    egui::Frame::new()
        .inner_margin(Margin::symmetric(panel.padding_x, panel.padding_y))
        .show(ui, add)
}
