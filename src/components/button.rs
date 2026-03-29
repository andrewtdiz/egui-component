use super::{
    api::{ComponentOverride, ComponentOverrides, ComponentUi},
    color::{paint_color, Color},
    common::ControlSize,
};
use crate::ui::{icons, tokens, typography};
use egui::{pos2, Color32, CursorIcon, FontId, Rect, RichText, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ButtonLabelWeight {
    Regular,
    Medium,
    Bold,
}

#[derive(Debug, Clone, Copy)]
pub struct Button<'a> {
    pub label: &'a str,
    pub variant: ButtonVariant,
    pub size: ControlSize,
    pub leading_icon: Option<&'a str>,
    pub icon_size: f32,
    pub color: Option<Color>,
    pub icon_tint: Option<Color32>,
    pub icon_only: bool,
    pub trailing_text: Option<&'a str>,
    pub trailing_hint: bool,
    pub trailing_icon: Option<&'a str>,
    pub selected: bool,
    pub min_size: Option<Vec2>,
    pub label_weight: ButtonLabelWeight,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            variant: ButtonVariant::Primary,
            size: ControlSize::Md,
            leading_icon: None,
            icon_size: 14.0,
            color: None,
            icon_tint: None,
            icon_only: false,
            trailing_text: None,
            trailing_hint: false,
            trailing_icon: None,
            selected: false,
            min_size: None,
            label_weight: ButtonLabelWeight::Regular,
        }
    }

    pub fn icon_only(icon: &'a str) -> Self {
        Self {
            label: "",
            variant: ButtonVariant::Primary,
            size: ControlSize::Md,
            leading_icon: Some(icon),
            icon_size: 14.0,
            color: None,
            icon_tint: None,
            icon_only: true,
            trailing_text: None,
            trailing_hint: false,
            trailing_icon: None,
            selected: false,
            min_size: None,
            label_weight: ButtonLabelWeight::Regular,
        }
    }

    pub fn color_only(color: Color) -> Self {
        Self {
            label: "",
            variant: ButtonVariant::Ghost,
            size: ControlSize::Md,
            leading_icon: None,
            icon_size: 14.0,
            color: Some(color),
            icon_tint: None,
            icon_only: true,
            trailing_text: None,
            trailing_hint: false,
            trailing_icon: None,
            selected: false,
            min_size: None,
            label_weight: ButtonLabelWeight::Regular,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn leading_icon(mut self, leading_icon: &'a str) -> Self {
        self.leading_icon = Some(leading_icon);
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

    pub fn trailing_text(mut self, trailing_text: &'a str) -> Self {
        self.trailing_text = Some(trailing_text);
        self.trailing_hint = false;
        self
    }

    pub fn trailing_hint(mut self, trailing_text: &'a str) -> Self {
        self.trailing_text = Some(trailing_text);
        self.trailing_hint = true;
        self
    }

    pub fn trailing_icon(mut self, trailing_icon: &'a str) -> Self {
        self.trailing_icon = Some(trailing_icon);
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

    pub fn label_weight(mut self, label_weight: ButtonLabelWeight) -> Self {
        self.label_weight = label_weight;
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ButtonOverride {
    pub variant: Option<ButtonVariant>,
    pub size: Option<ControlSize>,
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

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = Some(size);
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
        if let Some(variant) = self.variant {
            props.variant = variant;
        }
        if let Some(size) = self.size {
            props.size = size;
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
        if let Some(variant) = self.variant {
            overrides.button.variant = Some(variant);
        }
        if let Some(size) = self.size {
            overrides.button.size = Some(size);
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

impl ComponentUi<'_> {
    pub fn button<'a>(&mut self, props: impl Into<Button<'a>>) -> egui::Response {
        let props = self.overrides.button.apply(props.into());
        draw_button(self.ui_mut(), props)
    }
}

#[derive(Debug, Clone, Copy)]
struct ResolvedButtonStyle {
    inactive_fill: Color32,
    hovered_fill: Color32,
    active_fill: Color32,
    inactive_stroke: Stroke,
    hovered_stroke: Stroke,
    active_stroke: Stroke,
    inactive_fg: Stroke,
    hovered_fg: Stroke,
    active_fg: Stroke,
    label_color: Option<Color32>,
    icon_tint: Color32,
    frame: bool,
    button_padding: Vec2,
    min_size: Vec2,
}

fn active_button_stroke(style: &ResolvedButtonStyle, selected: bool) -> Stroke {
    if selected {
        style.hovered_stroke
    } else {
        style.active_stroke
    }
}

fn resolve_button_style(
    runtime: crate::theme::ThemeRuntime,
    variant: ButtonVariant,
    size: ControlSize,
    color_override: Option<Color>,
    icon_tint_override: Option<Color32>,
) -> ResolvedButtonStyle {
    let secondary_fg = Stroke::new(1.0, tokens::text_primary(runtime));
    let link_idle = Stroke::new(1.0, tokens::text_secondary(runtime));
    let link_active = Stroke::new(1.0, tokens::text_primary(runtime));
    let primary_fg = Stroke::new(1.0, tokens::primary_fg(runtime));
    let (button_padding, min_size) = match size {
        ControlSize::Sm => (
            egui::vec2(10.0, 5.0),
            egui::vec2(0.0, size.min_interact_height()),
        ),
        ControlSize::Md => (
            egui::vec2(
                tokens::SPACING_BUTTON_PADDING_X,
                tokens::SPACING_BUTTON_PADDING_Y,
            ),
            egui::vec2(0.0, size.min_interact_height()),
        ),
    };

    let mut resolved = match variant {
        ButtonVariant::Primary => ResolvedButtonStyle {
            inactive_fill: tokens::primary_bg(runtime),
            hovered_fill: tokens::primary_hover_bg(runtime),
            active_fill: tokens::primary_active_bg(runtime),
            inactive_stroke: Stroke::NONE,
            hovered_stroke: Stroke::NONE,
            active_stroke: Stroke::NONE,
            inactive_fg: primary_fg,
            hovered_fg: primary_fg,
            active_fg: primary_fg,
            label_color: Some(tokens::primary_fg(runtime)),
            icon_tint: tokens::primary_fg(runtime),
            frame: true,
            button_padding,
            min_size,
        },
        ButtonVariant::Secondary => ResolvedButtonStyle {
            inactive_fill: tokens::button_secondary_bg(runtime),
            hovered_fill: tokens::button_secondary_hover_bg(runtime),
            active_fill: tokens::button_secondary_active_bg(runtime),
            inactive_stroke: Stroke::new(1.0, tokens::button_secondary_border(runtime)),
            hovered_stroke: Stroke::new(1.0, tokens::button_secondary_hover_border(runtime)),
            active_stroke: Stroke::new(1.0, tokens::button_secondary_active_border(runtime)),
            inactive_fg: secondary_fg,
            hovered_fg: secondary_fg,
            active_fg: secondary_fg,
            label_color: None,
            icon_tint: tokens::text_primary(runtime),
            frame: true,
            button_padding,
            min_size,
        },
        ButtonVariant::Ghost => ResolvedButtonStyle {
            inactive_fill: tokens::TRANSPARENT,
            hovered_fill: tokens::button_secondary_hover_bg(runtime),
            active_fill: tokens::button_secondary_active_bg(runtime),
            inactive_stroke: Stroke::NONE,
            hovered_stroke: Stroke::NONE,
            active_stroke: Stroke::NONE,
            inactive_fg: secondary_fg,
            hovered_fg: secondary_fg,
            active_fg: secondary_fg,
            label_color: None,
            icon_tint: tokens::text_primary(runtime),
            frame: true,
            button_padding,
            min_size,
        },
        ButtonVariant::Link => ResolvedButtonStyle {
            inactive_fill: tokens::TRANSPARENT,
            hovered_fill: tokens::TRANSPARENT,
            active_fill: tokens::TRANSPARENT,
            inactive_stroke: Stroke::NONE,
            hovered_stroke: Stroke::NONE,
            active_stroke: Stroke::NONE,
            inactive_fg: link_idle,
            hovered_fg: link_active,
            active_fg: link_active,
            label_color: None,
            icon_tint: tokens::text_secondary(runtime),
            frame: false,
            button_padding,
            min_size,
        },
    };

    if let Some(icon_tint) = icon_tint_override {
        resolved.icon_tint = icon_tint;
    }

    if let Some(color) = color_override {
        if variant == ButtonVariant::Primary {
            let fill = color.fill;
            let hover_fill = fill.gamma_multiply(1.08);
            let active_fill = fill.gamma_multiply(0.92);
            resolved.inactive_fill = fill;
            resolved.hovered_fill = hover_fill;
            resolved.active_fill = active_fill;
            resolved.label_color = Some(tokens::button_primary_fill_fg(runtime));
            resolved.icon_tint = tokens::button_primary_fill_fg(runtime);
        }
    }

    resolved
}

fn draw_button(ui: &mut Ui, props: Button<'_>) -> egui::Response {
    ui.scope(|ui| {
        let runtime = crate::theme::runtime_for_ui(ui);
        let resolved = resolve_button_style(
            runtime,
            props.variant,
            props.size,
            props.color,
            props.icon_tint,
        );
        ui.spacing_mut().button_padding = resolved.button_padding;
        let visuals = &mut ui.style_mut().visuals.widgets;
        visuals.inactive.bg_fill = resolved.inactive_fill;
        visuals.inactive.weak_bg_fill = resolved.inactive_fill;
        visuals.hovered.bg_fill = resolved.hovered_fill;
        visuals.hovered.weak_bg_fill = resolved.hovered_fill;
        visuals.active.bg_fill = resolved.active_fill;
        visuals.active.weak_bg_fill = resolved.active_fill;
        visuals.inactive.bg_stroke = resolved.inactive_stroke;
        visuals.hovered.bg_stroke = resolved.hovered_stroke;
        visuals.active.bg_stroke = active_button_stroke(&resolved, props.selected);
        visuals.inactive.fg_stroke = resolved.inactive_fg;
        visuals.hovered.fg_stroke = resolved.hovered_fg;
        visuals.active.fg_stroke = resolved.active_fg;
        let button_label_font = resolve_button_label_font(props.label_weight);
        let label_color = resolved.label_color.unwrap_or(resolved.inactive_fg.color);
        let label_width = if props.label.is_empty() {
            0.0
        } else {
            ui.fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(
                        props.label.to_owned(),
                        button_label_font.clone(),
                        label_color,
                    )
                    .size()
                    .x
            })
        };
        let button_label = match resolved.label_color {
            Some(label_color) => RichText::new(props.label)
                .font(button_label_font.clone())
                .color(label_color),
            None => RichText::new(props.label).font(button_label_font.clone()),
        };
        let has_label = !props.label.is_empty();
        let swatch_only = props.color.is_some() && !has_label && props.leading_icon.is_none();

        let mut widget = if swatch_only {
            egui::Button::new("")
        } else {
            match props
                .leading_icon
                .and_then(|icon_name| icons::image(ui.ctx(), icon_name, props.icon_size))
            {
                Some(image) => {
                    let image = image.tint(resolved.icon_tint);
                    if props.icon_only || !has_label {
                        egui::Button::image(image)
                    } else {
                        egui::Button::image_and_text(image, button_label)
                    }
                }
                None => egui::Button::new(button_label),
            }
        };

        if props.trailing_icon.is_none() {
            if let Some(trailing_text) = props.trailing_text {
                if props.trailing_hint {
                    widget = widget.shortcut_text(trailing_text);
                } else {
                    widget = widget.right_text(trailing_text);
                }
            }
        }

        widget = widget.selected(props.selected);

        if !resolved.frame {
            widget = widget.frame(false);
        }
        let trailing_icon_extra_width = if props.trailing_icon.is_some() {
            props.icon_size + 12.0
        } else {
            0.0
        };
        if let Some(min_size) = props.min_size {
            widget = widget.min_size(egui::vec2(
                min_size.x.max(
                    label_width
                        + resolved.button_padding.x * 2.0
                        + trailing_icon_extra_width
                        + if props.leading_icon.is_some() && has_label {
                            props.icon_size + 8.0
                        } else {
                            0.0
                        },
                ),
                min_size.y,
            ));
        } else if props.icon_only || (props.leading_icon.is_some() && !has_label) || swatch_only {
            widget = widget.min_size(Vec2::splat(props.size.min_interact_height()));
        } else {
            widget = widget.min_size(egui::vec2(
                resolved.min_size.x.max(
                    label_width
                        + resolved.button_padding.x * 2.0
                        + trailing_icon_extra_width
                        + if props.leading_icon.is_some() && has_label {
                            props.icon_size + 8.0
                        } else {
                            0.0
                        },
                ),
                resolved.min_size.y,
            ));
        }

        let response = ui.add(widget).on_hover_cursor(CursorIcon::PointingHand);
        if let Some(color) = props.color.filter(|_| swatch_only) {
            paint_color(ui.painter(), response.rect, color);
        }
        if let Some(icon_name) = props.trailing_icon {
            paint_trailing_icon(
                ui,
                &response,
                icon_name,
                props.icon_size,
                resolved.button_padding.x,
                if !ui.is_enabled() {
                    tokens::text_muted(runtime)
                } else {
                    resolved.icon_tint
                },
            );
        }
        if props.variant == ButtonVariant::Link
            && response.hovered()
            && has_label
            && props.leading_icon.is_none()
            && !props.icon_only
        {
            let text_galley = ui.painter().layout_no_wrap(
                props.label.to_owned(),
                button_label_font.clone(),
                tokens::text_primary(runtime),
            );
            let text_half_width = text_galley.size().x * 0.5;
            let underline_y = response.rect.center().y + text_galley.size().y * 0.36;
            ui.painter().line_segment(
                [
                    egui::pos2(response.rect.center().x - text_half_width, underline_y),
                    egui::pos2(response.rect.center().x + text_half_width, underline_y),
                ],
                Stroke::new(1.0, tokens::text_primary(runtime)),
            );
        }

        response
    })
    .inner
}

fn paint_trailing_icon(
    ui: &mut Ui,
    response: &egui::Response,
    icon_name: &str,
    icon_size: f32,
    padding_x: f32,
    tint: Color32,
) {
    if let Some(image) = icons::image(ui.ctx(), icon_name, icon_size) {
        let icon_rect = Rect::from_center_size(
            pos2(
                response.rect.right() - padding_x - icon_size * 0.5,
                response.rect.center().y,
            ),
            egui::vec2(icon_size, icon_size),
        );
        let _ = image.tint(tint).paint_at(ui, icon_rect);
    }
}

fn resolve_button_label_font(weight: ButtonLabelWeight) -> FontId {
    match weight {
        ButtonLabelWeight::Regular => typography::label_font(),
        ButtonLabelWeight::Medium => typography::semibold_font(typography::LABEL_SIZE),
        ButtonLabelWeight::Bold => typography::bold_font(typography::LABEL_SIZE),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        active_button_stroke, resolve_button_label_font, Button, ButtonLabelWeight, ButtonVariant,
        ResolvedButtonStyle,
    };
    use crate::components::{Color, ComponentUiExt, ControlSize};
    use crate::ui::typography;
    use egui::{vec2, CentralPanel, Color32, Context, RawInput, Rect, Stroke};

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
                            .variant(ButtonVariant::Ghost)
                            .size(ControlSize::Sm),
                    )
                    .rect;
            });
        });

        assert!(rect.width() >= 18.0);
        assert!(rect.height() >= 18.0);
    }

    #[test]
    fn button_labels_use_label_text_size() {
        assert_eq!(
            resolve_button_label_font(ButtonLabelWeight::Regular),
            typography::label_font()
        );
        assert_eq!(
            resolve_button_label_font(ButtonLabelWeight::Medium).size,
            typography::LABEL_SIZE
        );
        assert_eq!(
            resolve_button_label_font(ButtonLabelWeight::Bold).size,
            typography::LABEL_SIZE
        );
    }

    #[test]
    fn selected_buttons_keep_hover_border_when_pressed() {
        let style = ResolvedButtonStyle {
            inactive_fill: Color32::TRANSPARENT,
            hovered_fill: Color32::TRANSPARENT,
            active_fill: Color32::TRANSPARENT,
            inactive_stroke: Stroke::new(1.0, Color32::BLACK),
            hovered_stroke: Stroke::new(1.0, Color32::WHITE),
            active_stroke: Stroke::new(1.0, Color32::RED),
            inactive_fg: Stroke::NONE,
            hovered_fg: Stroke::NONE,
            active_fg: Stroke::NONE,
            label_color: None,
            icon_tint: Color32::WHITE,
            frame: true,
            button_padding: vec2(0.0, 0.0),
            min_size: vec2(0.0, 0.0),
        };

        assert_eq!(active_button_stroke(&style, false), style.active_stroke);
        assert_eq!(active_button_stroke(&style, true), style.hovered_stroke);
    }
}
