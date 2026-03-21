mod component_showcase;

pub use component_showcase::ComponentShowcaseState;
use egui::Ui;

pub(crate) fn render_component_showcase(
    ui: &mut Ui,
    state: &mut ComponentShowcaseState,
    reload_generation: u64,
    runtime: crate::theme::ThemeRuntime,
) {
    component_showcase::render(ui, state, reload_generation, runtime);
}
