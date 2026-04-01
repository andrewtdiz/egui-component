use super::{api::ComponentUi, LabelTone, LabelWeight, TextInput};
use crate::ui::typography;
use egui::Response;

#[derive(Debug, Clone, Copy)]
pub struct Field<'a> {
    pub label: &'a str,
    pub helper_text: Option<&'a str>,
    pub width: f32,
    pub placeholder: Option<&'a str>,
}

impl<'a> Field<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            helper_text: None,
            width: 220.0,
            placeholder: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn helper_text(mut self, helper_text: &'a str) -> Self {
        self.helper_text = Some(helper_text);
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }
}

impl<'a> From<&'a str> for Field<'a> {
    fn from(label: &'a str) -> Self {
        Self::new(label)
    }
}

impl ComponentUi<'_> {
    pub fn field<'a>(&mut self, value: &mut String, props: impl Into<Field<'a>>) -> Response {
        let props = props.into();
        let overrides = self.overrides();
        let parent_layout = *self.ui_mut().layout();
        let layout = egui::Layout::top_down(parent_layout.horizontal_placement())
            .with_cross_justify(parent_layout.cross_justify());
        self.ui_mut()
            .with_layout(layout, |ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let mut response = ui.label(
                    crate::components::Label::new(props.label)
                        .tone(LabelTone::Secondary)
                        .weight(LabelWeight::Semibold),
                );

                let mut input_props = TextInput::new().width(props.width);
                if let Some(placeholder) = props.placeholder {
                    input_props = input_props.placeholder(placeholder);
                }
                let input_response = ui.text_input(value, input_props);
                response = response.union(input_response);

                if let Some(helper_text) = props.helper_text {
                    let helper_response = ui.label(
                        crate::components::Label::new(helper_text)
                            .tone(LabelTone::Muted)
                            .size(typography::SMALL_SIZE),
                    );
                    response = response.union(helper_response);
                }

                response
            })
            .inner
    }
}

#[cfg(test)]
mod tests {
    use super::Field;
    use crate::{components::ComponentUiExt, theme, ThemeMode, ThemeSpec};
    use egui::{Align, CentralPanel, Context, Layout, RawInput};

    #[test]
    fn field_preserves_parent_horizontal_alignment() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Light);
        let mut field_rect = egui::Rect::NOTHING;
        let mut host_rect = egui::Rect::NOTHING;
        let mut value = String::new();

        let _ = context.run(RawInput::default(), |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                let _ = ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    host_rect = ui
                        .scope(|ui| {
                            ui.set_min_width(480.0);
                            ui.set_max_width(480.0);
                            field_rect = ui
                                .components()
                                .field(
                                    &mut value,
                                    Field::new("Material")
                                        .width(280.0)
                                        .helper_text("Assigned material"),
                                )
                                .rect;
                        })
                        .response
                        .rect;
                });
            });
        });

        assert_eq!(field_rect.width(), 280.0);
        assert_eq!(field_rect.center().x, host_rect.center().x);
    }
}
