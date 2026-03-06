use super::api::ComponentUi;
use crate::ui::tokens;
use egui::{Response, Sense, Stroke, Ui};

impl ComponentUi<'_> {
    pub fn separator(&mut self) -> Response {
        draw_separator(self.raw_mut())
    }
}

fn draw_separator(ui: &mut Ui) -> Response {
    let width = ui.available_width().max(1.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 1.0), Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        Stroke::new(1.0, tokens::SEPARATOR),
    );
    response
}
