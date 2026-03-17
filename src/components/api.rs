use super::{
    button::ButtonOverride, card::CardOverride, input::TextInputOverride, label::LabelOverride,
};
use egui::{Id, Rect, Ui, Vec2};

#[doc(hidden)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ComponentOverrides {
    pub(crate) button: ButtonOverride,
    pub(crate) card: CardOverride,
    pub(crate) label: LabelOverride,
    pub(crate) text_input: TextInputOverride,
}

#[doc(hidden)]
pub trait ComponentOverride {
    fn apply_to(self, overrides: &mut ComponentOverrides);
}

#[doc(hidden)]
pub trait ComponentOverrideSet {
    fn apply_to(self, overrides: &mut ComponentOverrides);
}

impl<T> ComponentOverrideSet for T
where
    T: ComponentOverride,
{
    fn apply_to(self, overrides: &mut ComponentOverrides) {
        ComponentOverride::apply_to(self, overrides);
    }
}

impl<A, B> ComponentOverrideSet for (A, B)
where
    A: ComponentOverrideSet,
    B: ComponentOverrideSet,
{
    fn apply_to(self, overrides: &mut ComponentOverrides) {
        self.0.apply_to(overrides);
        self.1.apply_to(overrides);
    }
}

impl<A, B, C> ComponentOverrideSet for (A, B, C)
where
    A: ComponentOverrideSet,
    B: ComponentOverrideSet,
    C: ComponentOverrideSet,
{
    fn apply_to(self, overrides: &mut ComponentOverrides) {
        self.0.apply_to(overrides);
        self.1.apply_to(overrides);
        self.2.apply_to(overrides);
    }
}

impl<A, B, C, D> ComponentOverrideSet for (A, B, C, D)
where
    A: ComponentOverrideSet,
    B: ComponentOverrideSet,
    C: ComponentOverrideSet,
    D: ComponentOverrideSet,
{
    fn apply_to(self, overrides: &mut ComponentOverrides) {
        self.0.apply_to(overrides);
        self.1.apply_to(overrides);
        self.2.apply_to(overrides);
        self.3.apply_to(overrides);
    }
}

pub struct ComponentUi<'ui> {
    ui: &'ui mut Ui,
    pub(crate) overrides: ComponentOverrides,
}

impl<'ui> ComponentUi<'ui> {
    pub(crate) fn new(ui: &'ui mut Ui) -> Self {
        let overrides = load_component_overrides(ui);
        Self { ui, overrides }
    }

    pub(crate) fn with_overrides(ui: &'ui mut Ui, overrides: ComponentOverrides) -> Self {
        Self { ui, overrides }
    }

    pub(crate) fn ui(&self) -> &Ui {
        self.ui
    }

    pub(crate) fn ui_mut(&mut self) -> &mut Ui {
        self.ui
    }

    pub(crate) fn overrides(&self) -> ComponentOverrides {
        self.overrides
    }

    pub fn with_override<R>(
        &mut self,
        override_set: impl ComponentOverrideSet,
        add: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        let mut scoped_overrides = self.overrides;
        override_set.apply_to(&mut scoped_overrides);
        with_component_overrides(self.ui, scoped_overrides, add)
    }

    pub(crate) fn spacing(&self) -> &egui::style::Spacing {
        self.ui.spacing()
    }

    #[cfg(feature = "showcase")]
    pub(crate) fn spacing_mut(&mut self) -> &mut egui::style::Spacing {
        self.ui.spacing_mut()
    }

    pub(crate) fn available_width(&self) -> f32 {
        self.ui.available_width()
    }

    pub(crate) fn allocate_exact_size(
        &mut self,
        desired_size: Vec2,
        sense: egui::Sense,
    ) -> (Rect, egui::Response) {
        self.ui.allocate_exact_size(desired_size, sense)
    }

    pub(crate) fn painter(&self) -> &egui::Painter {
        self.ui.painter()
    }

    pub(crate) fn ctx(&self) -> &egui::Context {
        self.ui.ctx()
    }

    pub(crate) fn add_space(&mut self, amount: f32) {
        self.ui.add_space(amount);
    }

    pub(crate) fn set_min_width(&mut self, width: f32) {
        self.ui.set_min_width(width);
    }

    pub(crate) fn set_max_width(&mut self, width: f32) {
        self.ui.set_max_width(width);
    }

    #[cfg(feature = "showcase")]
    pub(crate) fn style(&self) -> &egui::Style {
        self.ui.style()
    }

    pub(crate) fn close(&mut self) {
        self.ui.close();
    }
}

pub(crate) fn with_component_overrides<R>(
    ui: &mut Ui,
    overrides: ComponentOverrides,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let previous = store_component_overrides(ui, overrides);
    let result = add(ui);
    restore_component_overrides(ui, previous);
    result
}

pub trait ComponentUiExt {
    fn components(&mut self) -> ComponentUi<'_>;
}

impl ComponentUiExt for Ui {
    fn components(&mut self) -> ComponentUi<'_> {
        ComponentUi::new(self)
    }
}

fn load_component_overrides(ui: &Ui) -> ComponentOverrides {
    ui.data(|data| {
        data.get_temp::<ComponentOverrides>(component_overrides_id())
            .unwrap_or_default()
    })
}

fn store_component_overrides(
    ui: &mut Ui,
    overrides: ComponentOverrides,
) -> Option<ComponentOverrides> {
    let previous = ui.data(|data| data.get_temp::<ComponentOverrides>(component_overrides_id()));
    ui.data_mut(|data| data.insert_temp(component_overrides_id(), overrides));
    previous
}

fn restore_component_overrides(ui: &mut Ui, previous: Option<ComponentOverrides>) {
    ui.data_mut(|data| {
        if let Some(previous) = previous {
            data.insert_temp(component_overrides_id(), previous);
        } else {
            data.remove::<ComponentOverrides>(component_overrides_id());
        }
    });
}

fn component_overrides_id() -> Id {
    Id::new("egui_component::component_overrides")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{
        button::{ButtonOverride, ButtonVariant},
        card::CardOverride,
        label::{LabelOverride, LabelTone},
    };
    use crate::theme::{self, ThemeMode};
    use crate::ui::tokens;
    use egui::{CentralPanel, Color32, RawInput, Stroke};

    #[test]
    fn tuple_override_sets_merge_by_widget_type() {
        let mut overrides = ComponentOverrides::default();
        (
            ButtonOverride::new().variant(ButtonVariant::Secondary),
            LabelOverride::new().tone(LabelTone::Muted),
            CardOverride::new().fill(Color32::WHITE),
        )
            .apply_to(&mut overrides);

        assert_eq!(overrides.button.variant, Some(ButtonVariant::Secondary));
        assert_eq!(overrides.label.tone, Some(LabelTone::Muted));
        assert_eq!(overrides.card.fill, Some(Color32::WHITE));
    }

    #[test]
    fn later_tuple_overrides_win_for_same_widget() {
        let mut overrides = ComponentOverrides::default();
        (
            ButtonOverride::new().variant(ButtonVariant::Secondary),
            ButtonOverride::new()
                .variant(ButtonVariant::Ghost)
                .icon_size(18.0),
            CardOverride::new().stroke(Stroke::NONE),
        )
            .apply_to(&mut overrides);

        assert_eq!(overrides.button.variant, Some(ButtonVariant::Ghost));
        assert_eq!(overrides.button.icon_size, Some(18.0));
        assert_eq!(overrides.card.stroke, Some(Stroke::NONE));
    }

    #[test]
    fn theme_install_applies_component_style_profile() {
        let context = egui::Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Light);

        let style = context.style();
        assert_eq!(style.spacing.item_spacing.y, tokens::SPACING_ITEM_Y);
        assert_eq!(
            style.spacing.button_padding.x,
            tokens::SPACING_BUTTON_PADDING_X
        );
        assert_eq!(
            style.spacing.button_padding.y,
            tokens::SPACING_BUTTON_PADDING_Y
        );
        assert_eq!(
            style.spacing.interact_size.y,
            tokens::SPACING_INTERACT_HEIGHT
        );
        assert_eq!(style.spacing.menu_spacing, 0.0);
        assert_eq!(style.visuals.selection.stroke, Stroke::NONE);
    }

    #[test]
    fn components_wrapper_preserves_existing_ui_style() {
        let context = egui::Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Light);

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                ui.spacing_mut().item_spacing.y = 123.0;
                ui.spacing_mut().button_padding.x = 19.0;
                ui.spacing_mut().button_padding.y = 11.0;
                ui.spacing_mut().interact_size.y = 41.0;
                let components = ui.components();
                assert_eq!(components.ui().spacing().item_spacing.y, 123.0);
                assert_eq!(components.ui().spacing().button_padding.x, 19.0);
                assert_eq!(components.ui().spacing().button_padding.y, 11.0);
                assert_eq!(components.ui().spacing().interact_size.y, 41.0);
            });
        });
    }

    #[test]
    fn with_override_propagates_to_nested_components_wrapper() {
        let context = egui::Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Light);

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let mut components = ui.components();
                components.with_override(
                    ButtonOverride::new().variant(ButtonVariant::Secondary),
                    |ui| {
                        let nested = ui.components();
                        assert_eq!(
                            nested.overrides().button.variant,
                            Some(ButtonVariant::Secondary)
                        );
                    },
                );
            });
        });
    }
}
