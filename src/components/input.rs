use super::{
    api::{ComponentOverride, ComponentOverrides, ComponentUi},
    common::{resolve_input_width, InputWidth},
};
use crate::primitives::control::with_input_chrome;
use crate::ui::tokens;
use crate::ui::typography;
use egui::{
    Align, Color32, CornerRadius, CursorIcon, Layout, Shape, Stroke, StrokeKind, Ui, UiBuilder,
};

#[derive(Debug, Clone, Copy)]
pub struct TextInput<'a> {
    pub width: f32,
    pub width_is_custom: bool,
    pub width_preset: Option<InputWidth>,
    pub placeholder: Option<&'a str>,
    pub leading_icon: Option<&'a str>,
    pub border_color: Option<Color32>,
    pub password: bool,
}

impl<'a> TextInput<'a> {
    pub fn new() -> Self {
        Self {
            width: 220.0,
            width_is_custom: false,
            width_preset: None,
            placeholder: None,
            leading_icon: None,
            border_color: None,
            password: false,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self.width_is_custom = true;
        self
    }

    pub fn width_preset(mut self, width_preset: InputWidth) -> Self {
        self.width_preset = Some(width_preset);
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

    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
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
            props.width_is_custom = true;
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
        let width = resolve_input_width(
            props.width,
            props.width_is_custom,
            props.width_preset,
            ui.available_width(),
        );
        let runtime = crate::theme::runtime_for_ui(ui);
        let leading_icon_size = 14.0;
        let leading_icon_extra_padding = if props.leading_icon.is_some() {
            leading_icon_size as i8 + 8
        } else {
            0
        };
        let text_font = typography::body_font();
        let text_edit_id = ui.next_auto_id();
        let mut text_edit = egui::TextEdit::singleline(value)
            .horizontal_align(Align::Min)
            .vertical_align(Align::Center)
            .font(text_font.clone())
            .id(text_edit_id)
            .frame(egui::Frame::NONE)
            .margin(egui::Margin::ZERO);
        if let Some(placeholder) = props.placeholder {
            text_edit = text_edit.hint_text(
                egui::RichText::new(placeholder)
                    .color(tokens::text_muted(runtime))
                    .weak(),
            );
        }
        let desired_size = egui::vec2(props.width, ui.spacing().interact_size.y);
        let (outer_rect, outer_response) =
            ui.allocate_exact_size(desired_size, egui::Sense::click());
        let text_row_height = ui.fonts_mut(|fonts| fonts.row_height(&text_font)).max(1.0);
        let left_inset = 1.0 + f32::from(tokens::INPUT_PADDING_X + leading_icon_extra_padding);
        let right_inset = 1.0 + f32::from(tokens::INPUT_PADDING_X);
        let inner_rect = egui::Rect::from_min_max(
            egui::pos2(
                outer_rect.left() + left_inset,
                outer_rect.center().y - text_row_height * 0.5,
            ),
            egui::pos2(
                (outer_rect.right() - right_inset).max(outer_rect.left() + left_inset + 1.0),
                outer_rect.center().y + text_row_height * 0.5,
            ),
        );

        if props.password {
            text_edit = text_edit.password(true);
        }
        let background_slot = ui.painter().add(Shape::Noop);
        let inner_response = ui
            .scope_builder(
                UiBuilder::new()
                    .max_rect(inner_rect)
                    .layout(Layout::left_to_right(Align::Center)),
                |ui| ui.add_sized(inner_rect.size(), text_edit),
            )
            .inner;
        if outer_response.clicked() {
            ui.memory_mut(|mem| mem.request_focus(text_edit_id));
        }

        let response = inner_response.union(outer_response);
        let focused = inner_response.has_focus();
        let hovered = inner_response.hovered() || response.hovered();
        let fill = tokens::input_bg(runtime, focused, hovered);
        let stroke = resolve_text_input_stroke(runtime, focused, hovered, props.border_color);
        ui.painter().set(
            background_slot,
            egui::epaint::RectShape::new(
                outer_rect,
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
                        outer_rect.left()
                            + f32::from(tokens::INPUT_PADDING_X)
                            + leading_icon_size * 0.5,
                        outer_rect.center().y,
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

#[cfg(test)]
mod tests {
    use super::TextInput;
    use crate::{components::ComponentUiExt, theme, ThemeMode, ThemeSpec};
    use egui::{CentralPanel, Context, RawInput, Rect, Shape};

    fn collect_text_rect(shape: &Shape, target: &str, text_rect: &mut Rect) {
        match shape {
            Shape::Text(text_shape) if text_shape.galley.job.text.contains(target) => {
                *text_rect = text_rect.union(text_shape.visual_bounding_rect());
            }
            Shape::Vec(shapes) => {
                for shape in shapes {
                    collect_text_rect(shape, target, text_rect);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn text_input_keeps_single_line_text_inside_the_control_chrome() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let mut input_rect = Rect::NOTHING;
        let mut value = String::from("Player_Robot");

        let output = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                input_rect = ui
                    .components()
                    .text_input(&mut value, TextInput::new().width(280.0))
                    .rect;
            });
        });

        let mut text_rect = Rect::NOTHING;
        for clipped_shape in output.shapes {
            collect_text_rect(&clipped_shape.shape, "Player_Robot", &mut text_rect);
        }

        assert!(text_rect.is_positive());
        assert!(text_rect.left() - input_rect.left() >= 8.0);
        assert!(text_rect.top() - input_rect.top() >= 8.0);
        assert!(input_rect.bottom() - text_rect.bottom() >= 6.0);
    }

    #[test]
    fn text_input_keeps_placeholder_inside_the_control_chrome() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let mut input_rect = Rect::NOTHING;
        let mut value = String::new();

        let output = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                input_rect = ui
                    .components()
                    .text_input(
                        &mut value,
                        TextInput::new()
                            .width(280.0)
                            .placeholder("Type component name"),
                    )
                    .rect;
            });
        });

        let mut text_rect = Rect::NOTHING;
        for clipped_shape in output.shapes {
            collect_text_rect(&clipped_shape.shape, "Type component name", &mut text_rect);
        }

        assert!(text_rect.is_positive());
        assert!(text_rect.left() - input_rect.left() >= 8.0);
        assert!(text_rect.top() - input_rect.top() >= 8.0);
        assert!(input_rect.bottom() - text_rect.bottom() >= 6.0);
    }
}
