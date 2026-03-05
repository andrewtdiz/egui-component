use crate::components::chrome::with_input_chrome;
use crate::ui::tokens;
use egui::{Align, CornerRadius, CursorIcon, Shape, Stroke, StrokeKind, Ui};

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

impl<'a> Default for TextInputProps<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn text_input(ui: &mut Ui, value: &mut String, props: TextInputProps<'_>) -> egui::Response {
    with_input_chrome(ui, |ui| {
        let dark_mode = ui.visuals().dark_mode;
        let mut text_edit = egui::TextEdit::singleline(value)
            .horizontal_align(Align::Min)
            .vertical_align(Align::Center)
            .frame(false)
            .margin(egui::Margin::symmetric(
                tokens::INPUT_PADDING_X,
                tokens::INPUT_PADDING_Y,
            ));
        if let Some(hint_text) = props.hint_text {
            text_edit = text_edit.hint_text(
                egui::RichText::new(hint_text)
                    .color(tokens::TEXT_MUTED)
                    .weak(),
            );
        }
        let background_slot = ui.painter().add(Shape::Noop);
        let response = ui.add_sized([props.width, ui.spacing().interact_size.y], text_edit);
        let fill = if response.has_focus() {
            tokens::INPUT_FOCUS_BACKGROUND
        } else if response.hovered() {
            tokens::INPUT_HOVER_BACKGROUND
        } else {
            tokens::INPUT_BACKGROUND
        };
        let stroke = if response.has_focus() {
            tokens::input_focus_stroke(dark_mode)
        } else if response.hovered() {
            Stroke::new(1.0, tokens::INPUT_HOVER_BORDER)
        } else {
            Stroke::new(1.0, tokens::INPUT_BORDER)
        };
        ui.painter().set(
            background_slot,
            egui::epaint::RectShape::new(
                response.rect,
                CornerRadius::same(tokens::RADIUS_MD),
                fill,
                stroke,
                StrokeKind::Inside,
            ),
        );

        response.on_hover_cursor(CursorIcon::Text)
    })
}
