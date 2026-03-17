mod component_showcase;

pub use component_showcase::ComponentShowcaseState;
use egui::Ui;

pub fn render_component_showcase(ui: &mut Ui, state: &mut ComponentShowcaseState) {
    component_showcase::render(ui, state);
}
