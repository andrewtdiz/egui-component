use super::api::{ComponentOverride, ComponentOverrides, ComponentUi};
use crate::components::chrome::with_input_chrome;
use crate::ui::tokens;
use egui::{Align, CornerRadius, CursorIcon, Shape, StrokeKind, Ui};

#[derive(Debug, Clone, Copy)]
pub struct TextInput<'a> {
    pub width: f32,
    pub hint_text: Option<&'a str>,
}

impl<'a> TextInput<'a> {
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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TextInputOverride {
    pub width: Option<f32>,
}

impl TextInputOverride {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(1.0));
        self
    }

    fn apply<'a>(self, mut props: TextInput<'a>) -> TextInput<'a> {
        if let Some(width) = self.width {
            props.width = width;
        }
        props
    }
}

impl ComponentOverride for TextInputOverride {
    fn apply_to(self, overrides: &mut ComponentOverrides) {
        if let Some(width) = self.width {
            overrides.text_input.width = Some(width);
        }
    }
}

impl<'a> Default for TextInput<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<()> for TextInput<'a> {
    fn from(_: ()) -> Self {
        Self::new()
    }
}

impl<'a> From<&'a str> for TextInput<'a> {
    fn from(hint_text: &'a str) -> Self {
        Self::new().hint_text(hint_text)
    }
}

impl<'a> From<f32> for TextInput<'a> {
    fn from(width: f32) -> Self {
        Self::new().width(width)
    }
}

impl<'a> From<(f32, &'a str)> for TextInput<'a> {
    fn from((width, hint_text): (f32, &'a str)) -> Self {
        Self::new().width(width).hint_text(hint_text)
    }
}

impl ComponentUi<'_> {
    pub fn text_input<'a>(
        &mut self,
        value: &mut String,
        props: impl Into<TextInput<'a>>,
    ) -> egui::Response {
        let props = self.overrides.text_input.apply(props.into());
        draw_text_input(self.raw_mut(), value, props)
    }
}

fn draw_text_input(ui: &mut Ui, value: &mut String, props: TextInput<'_>) -> egui::Response {
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
        let focused = response.has_focus();
        let hovered = response.hovered();
        let fill = tokens::input_bg(focused, hovered);
        let stroke = tokens::input_stroke(dark_mode, focused, hovered);
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
