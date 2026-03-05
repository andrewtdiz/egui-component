mod entity_editor;
mod shared;

use egui::Ui;
pub use entity_editor::EntityEditorStory;

pub fn render_entity_components_editor(ui: &mut Ui, story: &mut EntityEditorStory) {
    let _ = ui.scope(|ui| {
        shared::apply_preview_component_theme(ui);
        entity_editor::render(ui, story);
    });
}
