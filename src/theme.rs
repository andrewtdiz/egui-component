use crate::ui::{icons, style};

pub fn setup(context: &egui::Context) {
    setup_style(context);
    setup_icon_loader(context);
}

pub fn setup_style(context: &egui::Context) {
    style::setup_showcase_context(context);
}

pub fn setup_icon_loader(context: &egui::Context) {
    icons::setup(context);
}

pub fn apply_component_theme(ui: &mut egui::Ui) {
    style::apply_component_theme(ui);
}
