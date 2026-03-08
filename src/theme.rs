use crate::ui::{icons, style};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
}

impl ThemeMode {
    pub const fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }
}

pub fn install(context: &egui::Context, mode: ThemeMode) {
    style::install(context, mode);
    icons::setup(context);
}

pub fn set_mode(context: &egui::Context, mode: ThemeMode) {
    style::set_mode(context, mode);
}

#[cfg(feature = "showcase")]
pub(crate) fn apply_component_theme(ui: &mut egui::Ui) {
    style::apply_component_profile(ui);
}
