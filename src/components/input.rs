use crate::components::chrome::with_input_chrome;
use crate::ui::tokens;
use egui::{Align, CursorIcon, Ui};

#[derive(Debug, Clone, Copy)]
pub struct TextInputProps<'a> {
    pub width: f32,
    pub hint_text: Option<&'a str>,
}

impl<'a> TextInputProps<'a> {
    pub fn new() -> Self {
        Self {
            width: 220.0,
            hint_text: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn hint_text(mut self, hint_text: &'a str) -> Self {
        self.hint_text = Some(hint_text);
        self
    }
}

pub fn text_input(ui: &mut Ui, value: &mut String, props: TextInputProps<'_>) -> egui::Response {
    with_input_chrome(ui, |ui| {
        let mut text_edit = egui::TextEdit::singleline(value)
            .horizontal_align(Align::Min)
            .vertical_align(Align::Center)
            .margin(egui::Margin::symmetric(
                tokens::INPUT_PADDING_X,
                tokens::INPUT_PADDING_Y,
            ));
        if let Some(hint_text) = props.hint_text {
            text_edit = text_edit.hint_text(hint_text);
        }
        ui.add_sized([props.width, ui.spacing().interact_size.y], text_edit)
            .on_hover_cursor(CursorIcon::Text)
    })
}
