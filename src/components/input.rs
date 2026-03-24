use super::api::{ComponentOverride, ComponentOverrides, ComponentUi};
use crate::primitives::control::with_input_chrome;
use crate::ui::tokens;
use egui::{Align, Color32, CornerRadius, CursorIcon, Shape, Stroke, StrokeKind, Ui};

#[derive(Debug, Clone, Copy)]
pub struct TextInput<'a> {
    pub width: f32,
    pub placeholder: Option<&'a str>,
    pub leading_icon: Option<&'a str>,
    pub border_color: Option<Color32>,
}

impl<'a> TextInput<'a> {
    pub fn new() -> Self {
        Self {
            width: 220.0,
            placeholder: None,
            leading_icon: None,
            border_color: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    pub fn leading_icon(mut self, leading_icon: &'a str) -> Self {
        self.leading_icon = Some(leading_icon);
        self
    }

    pub fn border_color(mut self, border_color: Color32) -> Self {
        self.border_color = Some(border_color);
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
    fn from(placeholder: &'a str) -> Self {
        Self::new().placeholder(placeholder)
    }
}

impl ComponentUi<'_> {
    pub fn text_input<'a>(
        &mut self,
        value: &mut String,
        props: impl Into<TextInput<'a>>,
    ) -> egui::Response {
        let props = self.overrides.text_input.apply(props.into());
        draw_text_input(self.ui_mut(), value, props)
    }
}

fn draw_text_input(ui: &mut Ui, value: &mut String, props: TextInput<'_>) -> egui::Response {
    with_input_chrome(ui, |ui| {
        let runtime = crate::theme::runtime_for_ui(ui);
        let leading_icon_size = 14.0;
        let leading_icon_extra_padding = if props.leading_icon.is_some() {
            leading_icon_size as i8 + 8
        } else {
            0
        };
        let mut text_edit = egui::TextEdit::singleline(value)
            .horizontal_align(Align::Min)
            .vertical_align(Align::Center)
            .frame(false)
            .margin(egui::Margin {
                left: tokens::INPUT_PADDING_X + leading_icon_extra_padding,
                right: tokens::INPUT_PADDING_X,
                top: tokens::INPUT_PADDING_Y,
                bottom: tokens::INPUT_PADDING_Y,
            });
        if let Some(placeholder) = props.placeholder {
            text_edit = text_edit.hint_text(
                egui::RichText::new(placeholder)
                    .color(tokens::text_muted(runtime))
                    .weak(),
            );
        }
        let background_slot = ui.painter().add(Shape::Noop);
        let response = ui.add_sized([props.width, ui.spacing().interact_size.y], text_edit);
        let focused = response.has_focus();
        let hovered = response.hovered();
        let fill = tokens::input_bg(runtime, focused, hovered);
        let stroke = resolve_text_input_stroke(runtime, focused, hovered, props.border_color);
        ui.painter().set(
            background_slot,
            egui::epaint::RectShape::new(
                response.rect,
                CornerRadius::same(tokens::radius_md(runtime)),
                fill,
                stroke,
                StrokeKind::Inside,
            ),
        );

        if let Some(icon_name) = props.leading_icon {
            if let Some(image) = crate::icons::image(ui.ctx(), icon_name, leading_icon_size) {
                let icon_rect = egui::Rect::from_center_size(
                    egui::pos2(
                        response.rect.left()
                            + f32::from(tokens::INPUT_PADDING_X)
                            + leading_icon_size * 0.5,
                        response.rect.center().y,
                    ),
                    egui::vec2(leading_icon_size, leading_icon_size),
                );
                let _ = image
                    .tint(tokens::text_muted(runtime))
                    .paint_at(ui, icon_rect);
            }
        }

        response.on_hover_cursor(CursorIcon::Text)
    })
}

fn resolve_text_input_stroke(
    runtime: crate::theme::ThemeRuntime,
    focused: bool,
    hovered: bool,
    border_override: Option<Color32>,
) -> Stroke {
    let base = tokens::input_stroke(runtime, focused, hovered);
    if let Some(border) = border_override {
        let color = if focused {
            border
        } else if hovered {
            base.color.lerp_to_gamma(border, 0.35)
        } else {
            base.color.lerp_to_gamma(border, 0.18)
        };
        Stroke::new(base.width.max(1.0), color)
    } else {
        base
    }
}
