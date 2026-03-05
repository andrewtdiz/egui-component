use crate::components::{label, text_input, LabelProps, LabelTone, LabelWeight, TextInputProps};
use egui::{Align, Layout, Ui};

#[derive(Debug, Clone, Copy)]
pub struct FieldProps<'a> {
    pub label: &'a str,
    pub helper_text: Option<&'a str>,
    pub width: f32,
    pub placeholder: Option<&'a str>,
}

impl<'a> FieldProps<'a> {
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

pub fn field(ui: &mut Ui, value: &mut String, props: FieldProps<'_>) -> egui::Response {
    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        let mut response = label(
            ui,
            LabelProps::new(props.label)
                .tone(LabelTone::Secondary)
                .weight(LabelWeight::Semibold),
        );

        let mut input_props = TextInputProps::new().width(props.width);
        if let Some(placeholder) = props.placeholder {
            input_props = input_props.hint_text(placeholder);
        }
        let input_response = text_input(ui, value, input_props);
        response = response.union(input_response);

        if let Some(helper_text) = props.helper_text {
            let helper_response = label(
                ui,
                LabelProps::new(helper_text)
                    .tone(LabelTone::Muted)
                    .size(11.0),
            );
            response = response.union(helper_response);
        }

        response
    })
    .inner
}
