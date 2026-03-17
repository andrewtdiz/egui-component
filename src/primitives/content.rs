use crate::ui::{tokens, typography};
use egui::{Response, Ui};

pub(crate) fn muted_empty_state(ui: &mut Ui, text: &str) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    ui.add(
        egui::Label::new(
            egui::RichText::new(text)
                .color(tokens::text_muted(runtime))
                .font(typography::proportional(typography::SMALL_SIZE)),
        )
        .selectable(false),
    )
}
