use super::{api::ComponentUi, LabelTone, LabelWeight, TextInput};
use crate::ui::typography;
use egui::{Align, Response};

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
        self.with_layout(egui::Layout::top_down(Align::Min), |ui| {
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
