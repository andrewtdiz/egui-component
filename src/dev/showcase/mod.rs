mod component_showcase;

use crate::theme;
pub use component_showcase::ComponentShowcaseState;
use egui::Ui;

pub fn render_component_showcase(ui: &mut Ui, state: &mut ComponentShowcaseState) {
    let _ = ui.scope(|ui| {
        theme::apply_component_theme(ui);
        component_showcase::render(ui, state);
    });
}
