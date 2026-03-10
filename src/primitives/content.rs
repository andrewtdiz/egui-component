use crate::ui::{tokens, typography};
use egui::{Response, Ui};

pub(crate) fn muted_empty_state(ui: &mut Ui, text: &str) -> Response {
    let dark_mode = ui.visuals().dark_mode;
    ui.add(
        egui::Label::new(
            egui::RichText::new(text)
                .color(tokens::text_muted(dark_mode))
                .font(typography::proportional(typography::SMALL_SIZE)),
        )
        .selectable(false),
    )
}
