use super::{
    api::{ComponentOverride, ComponentOverrides, ComponentUi},
    color::{paint_color, Color},
};
use crate::ui::{icons, tokens};
use egui::{Color32, CursorIcon, RichText, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy)]
pub struct Button<'a> {
    pub name: &'a str,
    pub style: ButtonStyle,
    pub icon: Option<&'a str>,
    pub icon_size: f32,
    pub color: Option<Color>,
    pub icon_tint: Option<Color32>,
    pub icon_only: bool,
    pub right_text: Option<&'a str>,
    pub right_text_weak: bool,
    pub selected: bool,
    pub min_size: Option<Vec2>,
}

impl<'a> Button<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            style: ButtonStyle::Primary,
            icon: None,
            icon_size: 14.0,
            color: None,
            icon_tint: None,
            icon_only: false,
            right_text: None,
            right_text_weak: false,
            selected: false,
            min_size: None,
        }
    }

    pub fn name(name: &'a str) -> Self {
        Self::new(name)
    }

    pub fn icon_only(icon: &'a str) -> Self {
        Self {
            name: "",
            style: ButtonStyle::Primary,
            icon: Some(icon),
            icon_size: 14.0,
            color: None,
            icon_tint: None,
            icon_only: true,
            right_text: None,
            right_text_weak: false,
            selected: false,
            min_size: None,
        }
    }

    pub fn color_only(color: Color) -> Self {
        Self {
            name: "",
            style: ButtonStyle::Ghost,
            icon: None,
            icon_size: 14.0,
            color: Some(color),
            icon_tint: None,
            icon_only: true,
            right_text: None,
            right_text_weak: false,
            selected: false,
            min_size: None,
        }
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = icon_size.max(1.0);
        self
    }

    pub fn icon_tint(mut self, icon_tint: Color32) -> Self {
        self.icon_tint = Some(icon_tint);
        self
    }

    pub fn right_text(mut self, right_text: &'a str) -> Self {
        self.right_text = Some(right_text);
        self.right_text_weak = false;
        self
    }

    pub fn right_text_weak(mut self, right_text: &'a str) -> Self {
        self.right_text = Some(right_text);
        self.right_text_weak = true;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = Some(min_size);
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ButtonOverride {
    pub style: Option<ButtonStyle>,
    pub icon_size: Option<f32>,
    pub icon_tint: Option<Color32>,
    pub icon_only: Option<bool>,
    pub selected: Option<bool>,
    pub min_size: Option<Vec2>,
}

impl ButtonOverride {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = Some(icon_size.max(1.0));
        self
    }

    pub fn icon_tint(mut self, icon_tint: Color32) -> Self {
        self.icon_tint = Some(icon_tint);
        self
    }

    pub fn icon_only(mut self, icon_only: bool) -> Self {
        self.icon_only = Some(icon_only);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = Some(min_size);
        self
    }

    fn apply<'a>(self, mut props: Button<'a>) -> Button<'a> {
        if let Some(style) = self.style {
            props.style = style;
        }
        if let Some(icon_size) = self.icon_size {
            props.icon_size = icon_size;
        }
        if let Some(icon_tint) = self.icon_tint {
            props.icon_tint = Some(icon_tint);
        }
        if let Some(icon_only) = self.icon_only {
            props.icon_only = icon_only;
        }
        if let Some(selected) = self.selected {
            props.selected = selected;
        }
        if let Some(min_size) = self.min_size {
            props.min_size = Some(min_size);
        }
        props
    }
}

impl ComponentOverride for ButtonOverride {
    fn apply_to(self, overrides: &mut ComponentOverrides) {
        if let Some(style) = self.style {
            overrides.button.style = Some(style);
        }
        if let Some(icon_size) = self.icon_size {
            overrides.button.icon_size = Some(icon_size);
        }
        if let Some(icon_tint) = self.icon_tint {
            overrides.button.icon_tint = Some(icon_tint);
        }
        if let Some(icon_only) = self.icon_only {
            overrides.button.icon_only = Some(icon_only);
        }
        if let Some(selected) = self.selected {
            overrides.button.selected = Some(selected);
        }
        if let Some(min_size) = self.min_size {
            overrides.button.min_size = Some(min_size);
        }
    }
}

impl<'a> From<&'a str> for Button<'a> {
    fn from(label: &'a str) -> Self {
        Self::new(label)
    }
}

impl<'a> From<(&'a str, ButtonStyle)> for Button<'a> {
    fn from((label, style): (&'a str, ButtonStyle)) -> Self {
        Self::new(label).style(style)
    }
}

impl<'a> From<(&'a str, &'a str)> for Button<'a> {
    fn from((label, icon): (&'a str, &'a str)) -> Self {
        Self::new(label).icon(icon)
    }
}

impl<'a> From<(&'a str, &'a str, ButtonStyle)> for Button<'a> {
    fn from((label, icon, style): (&'a str, &'a str, ButtonStyle)) -> Self {
        Self::new(label).icon(icon).style(style)
    }
}

impl<'a> From<(&'a str, ButtonStyle, &'a str)> for Button<'a> {
    fn from((label, style, icon): (&'a str, ButtonStyle, &'a str)) -> Self {
        Self::new(label).style(style).icon(icon)
    }
}

impl ComponentUi<'_> {
    pub fn button<'a>(&mut self, props: impl Into<Button<'a>>) -> egui::Response {
        let props = self.overrides.button.apply(props.into());
        draw_button(self.raw_mut(), props)
    }
}

fn draw_button(ui: &mut Ui, props: Button<'_>) -> egui::Response {
    ui.scope(|ui| {
        let dark_mode = ui.visuals().dark_mode;
        let visuals = &mut ui.style_mut().visuals.widgets;
        match props.style {
            ButtonStyle::Primary => {
                visuals.inactive.bg_fill = tokens::primary_bg(dark_mode);
                visuals.inactive.weak_bg_fill = tokens::primary_bg(dark_mode);
                visuals.hovered.bg_fill = tokens::primary_hover_bg(dark_mode);
                visuals.hovered.weak_bg_fill = tokens::primary_hover_bg(dark_mode);
                visuals.active.bg_fill = tokens::primary_active_bg(dark_mode);
                visuals.active.weak_bg_fill = tokens::primary_active_bg(dark_mode);
                visuals.inactive.bg_stroke = Stroke::NONE;
                visuals.hovered.bg_stroke = Stroke::NONE;
                visuals.active.bg_stroke = Stroke::NONE;
                let fg = Stroke::new(1.0, tokens::primary_fg(dark_mode));
                visuals.inactive.fg_stroke = fg;
                visuals.hovered.fg_stroke = fg;
                visuals.active.fg_stroke = fg;
            }
            ButtonStyle::Secondary => {
                visuals.inactive.bg_fill = tokens::button_secondary_bg(dark_mode);
                visuals.inactive.weak_bg_fill = tokens::button_secondary_bg(dark_mode);
                visuals.hovered.bg_fill = tokens::button_secondary_hover_bg(dark_mode);
                visuals.hovered.weak_bg_fill = tokens::button_secondary_hover_bg(dark_mode);
                visuals.active.bg_fill = tokens::button_secondary_active_bg(dark_mode);
                visuals.active.weak_bg_fill = tokens::button_secondary_active_bg(dark_mode);
                visuals.inactive.bg_stroke =
                    Stroke::new(1.0, tokens::button_secondary_border(dark_mode));
                visuals.hovered.bg_stroke =
                    Stroke::new(1.0, tokens::button_secondary_hover_border(dark_mode));
                visuals.active.bg_stroke =
                    Stroke::new(1.0, tokens::button_secondary_active_border(dark_mode));
            }
            ButtonStyle::Ghost => {
                visuals.inactive.bg_fill = tokens::TRANSPARENT;
                visuals.inactive.weak_bg_fill = tokens::TRANSPARENT;
                visuals.hovered.bg_fill = tokens::button_secondary_hover_bg(dark_mode);
                visuals.hovered.weak_bg_fill = tokens::button_secondary_hover_bg(dark_mode);
                visuals.active.bg_fill = tokens::button_secondary_active_bg(dark_mode);
                visuals.active.weak_bg_fill = tokens::button_secondary_active_bg(dark_mode);
                visuals.inactive.bg_stroke = Stroke::NONE;
                visuals.hovered.bg_stroke = Stroke::NONE;
                visuals.active.bg_stroke = Stroke::NONE;
            }
            ButtonStyle::Link => {
                visuals.inactive.bg_fill = tokens::TRANSPARENT;
                visuals.inactive.weak_bg_fill = tokens::TRANSPARENT;
                visuals.hovered.bg_fill = tokens::TRANSPARENT;
                visuals.hovered.weak_bg_fill = tokens::TRANSPARENT;
                visuals.active.bg_fill = tokens::TRANSPARENT;
                visuals.active.weak_bg_fill = tokens::TRANSPARENT;
                visuals.inactive.bg_stroke = Stroke::NONE;
                visuals.hovered.bg_stroke = Stroke::NONE;
                visuals.active.bg_stroke = Stroke::NONE;
                visuals.inactive.fg_stroke = Stroke::new(1.0, tokens::text_secondary(dark_mode));
                visuals.hovered.fg_stroke = Stroke::new(1.0, tokens::text_primary(dark_mode));
                visuals.active.fg_stroke = Stroke::new(1.0, tokens::text_primary(dark_mode));
            }
        }
        let button_label = match props.style {
            ButtonStyle::Primary => RichText::new(props.name).color(tokens::primary_fg(dark_mode)),
            _ => RichText::new(props.name),
        };
        let has_label = !props.name.is_empty();
        let swatch_only = props.color.is_some() && !has_label && props.icon.is_none();
        let icon_tint = props.icon_tint.unwrap_or(match props.style {
            ButtonStyle::Primary => tokens::primary_fg(dark_mode),
            ButtonStyle::Link => tokens::text_secondary(dark_mode),
            _ => tokens::text_primary(dark_mode),
        });

        let mut widget = if swatch_only {
            egui::Button::new("")
        } else {
            match props
                .icon
                .and_then(|icon_name| icons::image(ui.ctx(), icon_name, props.icon_size))
            {
                Some(image) => {
                    let image = image.tint(icon_tint);
                    if props.icon_only || !has_label {
                        egui::Button::image(image)
                    } else {
                        egui::Button::image_and_text(image, button_label)
                    }
                }
                None => egui::Button::new(button_label),
            }
        };

        if let Some(right_text) = props.right_text {
            if props.right_text_weak {
                widget = widget.shortcut_text(right_text);
            } else {
                widget = widget.right_text(right_text);
            }
        }

        widget = widget.selected(props.selected);

        if props.style == ButtonStyle::Link {
            widget = widget.frame(false);
        }
        if props.icon_only || (props.icon.is_some() && !has_label) || swatch_only {
            widget = widget.min_size(Vec2::splat(ui.spacing().interact_size.y));
        }
        if let Some(min_size) = props.min_size {
            widget = widget.min_size(min_size);
        }

        let response = ui.add(widget).on_hover_cursor(CursorIcon::PointingHand);
        if let Some(color) = props.color.filter(|_| swatch_only) {
            paint_color(ui.painter(), response.rect, color);
        }
        if props.style == ButtonStyle::Link
            && response.hovered()
            && has_label
            && props.icon.is_none()
            && !props.icon_only
        {
            let font_id = egui::TextStyle::Button.resolve(ui.style());
            let text_galley = ui.painter().layout_no_wrap(
                props.name.to_owned(),
                font_id,
                tokens::text_primary(dark_mode),
            );
            let text_half_width = text_galley.size().x * 0.5;
            let underline_y = response.rect.center().y + text_galley.size().y * 0.36;
            ui.painter().line_segment(
                [
                    egui::pos2(response.rect.center().x - text_half_width, underline_y),
                    egui::pos2(response.rect.center().x + text_half_width, underline_y),
                ],
                Stroke::new(1.0, tokens::text_primary(dark_mode)),
            );
        }

        response
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::{Button, ButtonStyle};
    use crate::components::{Color, ComponentUiExt};
    use egui::{CentralPanel, Color32, Context, RawInput, Rect};

    #[test]
    fn renders_color_only_button() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = ui
                    .components()
                    .button(
                        Button::color_only(Color::new(Color32::from_rgb(17, 24, 39)).size(18.0))
                            .style(ButtonStyle::Ghost),
                    )
                    .rect;
            });
        });

        assert!(rect.width() >= 18.0);
        assert!(rect.height() >= 18.0);
    }
}
