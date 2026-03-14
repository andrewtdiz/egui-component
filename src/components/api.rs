use super::{
    button::ButtonOverride, card::CardOverride, input::TextInputOverride, label::LabelOverride,
};
use crate::ui::style;
use egui::{pos2, Id, InnerResponse, Layout, Rect, Ui, UiBuilder, Vec2};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

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
        style::apply_component_profile(ui);
        let overrides = load_component_overrides(ui);
        Self { ui, overrides }
    }

    pub(crate) fn with_overrides(ui: &'ui mut Ui, overrides: ComponentOverrides) -> Self {
        Self { ui, overrides }
    }

    pub fn raw(&self) -> &Ui {
        self.ui
    }

    pub fn raw_mut(&mut self) -> &mut Ui {
        self.ui
    }

    pub fn with_override<R>(
        &mut self,
        override_set: impl ComponentOverrideSet,
        add: impl FnOnce(&mut ComponentUi<'_>) -> R,
    ) -> R {
        let mut scoped_overrides = self.overrides;
        override_set.apply_to(&mut scoped_overrides);

        let previous = store_component_overrides(self.ui, scoped_overrides);
        let result = {
            let mut scoped = ComponentUi::with_overrides(&mut *self.ui, scoped_overrides);
            add(&mut scoped)
        };
        restore_component_overrides(self.ui, previous);
        result
    }

    pub fn scope<R>(&mut self, add: impl FnOnce(&mut ComponentUi<'_>) -> R) -> InnerResponse<R> {
        let overrides = self.overrides;
        self.ui.scope(|ui| {
            let mut components = ComponentUi::with_overrides(ui, overrides);
            add(&mut components)
        })
    }

    pub fn scope_builder<R>(
        &mut self,
        builder: UiBuilder,
        add: impl FnOnce(&mut ComponentUi<'_>) -> R,
    ) -> InnerResponse<R> {
        let overrides = self.overrides;
        self.ui.scope_builder(builder, |ui| {
            let mut components = ComponentUi::with_overrides(ui, overrides);
            add(&mut components)
        })
    }

    pub fn centered_lane<R>(
        &mut self,
        width: f32,
        layout: Layout,
        add: impl FnOnce(&mut ComponentUi<'_>) -> R,
    ) -> InnerResponse<R> {
        let available_rect = self.ui.available_rect_before_wrap();
        let lane_width = width.max(0.0).min(available_rect.width().max(0.0));
        let top = self.ui.cursor().top().max(available_rect.top());
        let bottom = available_rect.bottom().max(top);
        let lane_rect = Rect::from_min_max(
            pos2(available_rect.center().x - lane_width * 0.5, top),
            pos2(available_rect.center().x + lane_width * 0.5, bottom),
        );
        let overrides = self.overrides;
        let mut used_rect = Rect::from_min_size(lane_rect.min, Vec2::ZERO);
        let response =
            self.ui
                .scope_builder(UiBuilder::new().max_rect(lane_rect).layout(layout), |ui| {
                    let mut components = ComponentUi::with_overrides(ui, overrides);
                    let inner = add(&mut components);
                    used_rect = components.min_rect();
                    inner
                });
        self.ui.advance_cursor_after_rect(used_rect);
        response
    }

    pub fn push_id<R>(
        &mut self,
        id_salt: impl Hash,
        add: impl FnOnce(&mut ComponentUi<'_>) -> R,
    ) -> InnerResponse<R> {
        let overrides = self.overrides;
        self.ui.push_id(id_salt, |ui| {
            let mut components = ComponentUi::with_overrides(ui, overrides);
            add(&mut components)
        })
    }

    pub fn horizontal<R>(
        &mut self,
        add: impl FnOnce(&mut ComponentUi<'_>) -> R,
    ) -> InnerResponse<R> {
        let overrides = self.overrides;
        self.ui.horizontal(|ui| {
            let mut components = ComponentUi::with_overrides(ui, overrides);
            add(&mut components)
        })
    }

    pub fn vertical<R>(&mut self, add: impl FnOnce(&mut ComponentUi<'_>) -> R) -> InnerResponse<R> {
        let overrides = self.overrides;
        self.ui.vertical(|ui| {
            let mut components = ComponentUi::with_overrides(ui, overrides);
            add(&mut components)
        })
    }

    pub fn with_layout<R>(
        &mut self,
        layout: Layout,
        add: impl FnOnce(&mut ComponentUi<'_>) -> R,
    ) -> InnerResponse<R> {
        let overrides = self.overrides;
        self.ui.with_layout(layout, |ui| {
            let mut components = ComponentUi::with_overrides(ui, overrides);
            add(&mut components)
        })
    }

    pub fn allocate_ui_with_layout<R>(
        &mut self,
        desired_size: Vec2,
        layout: Layout,
        add: impl FnOnce(&mut ComponentUi<'_>) -> R,
    ) -> InnerResponse<R> {
        let overrides = self.overrides;
        self.ui.allocate_ui_with_layout(desired_size, layout, |ui| {
            let mut components = ComponentUi::with_overrides(ui, overrides);
            add(&mut components)
        })
    }
}

impl Deref for ComponentUi<'_> {
    type Target = Ui;

    fn deref(&self) -> &Self::Target {
        self.ui
    }
}

impl DerefMut for ComponentUi<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ui
    }
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
        Label,
    };
    use crate::theme::{self, ThemeMode};
    use crate::ui::tokens;
    use egui::{Align, CentralPanel, Color32, Layout, RawInput, Stroke};

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
    fn components_wrapper_applies_component_style_profile() {
        let context = egui::Context::default();
        theme::install(&context, ThemeMode::Light);

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let components = ui.components();
                assert_eq!(components.spacing().item_spacing.y, tokens::SPACING_ITEM_Y);
                assert_eq!(
                    components.spacing().button_padding.x,
                    tokens::SPACING_BUTTON_PADDING_X
                );
                assert_eq!(
                    components.spacing().button_padding.y,
                    tokens::SPACING_BUTTON_PADDING_Y
                );
                assert_eq!(
                    components.spacing().interact_size.y,
                    tokens::SPACING_INTERACT_HEIGHT
                );
                assert_eq!(components.spacing().menu_spacing, 0.0);
                assert_eq!(components.visuals().selection.stroke, Stroke::NONE);
            });
        });
    }

    #[test]
    fn centered_lane_centers_and_advances_parent_cursor() {
        let context = egui::Context::default();
        theme::install(&context, ThemeMode::Light);

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let parent_center_x = ui.max_rect().center().x;
                let cursor_top_before = ui.cursor().top();
                let lane = {
                    let mut components = ui.components();
                    components.centered_lane(180.0, Layout::top_down(Align::Min), |ui| {
                        let lane_rect = ui.max_rect();
                        let label_rect = ui.label(Label::new("Centered lane")).rect;
                        (lane_rect, label_rect)
                    })
                };
                let cursor_top_after = ui.cursor().top();
                let (lane_rect, label_rect) = lane.inner;

                assert!((lane_rect.center().x - parent_center_x).abs() <= 0.5);
                assert!(cursor_top_after >= label_rect.bottom());
                assert!(cursor_top_after < lane_rect.bottom());
                assert!(cursor_top_after > cursor_top_before);
            });
        });
    }
}
