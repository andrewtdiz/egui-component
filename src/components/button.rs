use crate::ui::{icons, tokens};
use egui::{Color32, CursorIcon, RichText, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonProps<'a> {
    pub label: &'a str,
    pub variant: ButtonVariant,
    pub icon: Option<&'a str>,
    pub icon_size: f32,
    pub icon_tint: Option<Color32>,
    pub icon_only: bool,
    pub right_text: Option<&'a str>,
    pub right_text_weak: bool,
    pub selected: bool,
    pub min_size: Option<Vec2>,
}

impl<'a> ButtonProps<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            variant: ButtonVariant::Primary,
            icon: None,
            icon_size: 14.0,
            icon_tint: None,
            icon_only: false,
            right_text: None,
            right_text_weak: false,
            selected: false,
            min_size: None,
        }
    }

    pub fn icon_only(icon: &'a str) -> Self {
        Self {
            label: "",
            variant: ButtonVariant::Primary,
            icon: Some(icon),
            icon_size: 14.0,
            icon_tint: None,
            icon_only: true,
            right_text: None,
            right_text_weak: false,
            selected: false,
            min_size: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
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

pub fn button(ui: &mut Ui, props: ButtonProps<'_>) -> egui::Response {
    ui.scope(|ui| {
        let dark_mode = ui.visuals().dark_mode;
        let visuals = &mut ui.style_mut().visuals.widgets;
        match props.variant {
            ButtonVariant::Primary => {
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
            ButtonVariant::Secondary => {
                visuals.inactive.bg_fill = tokens::BUTTON_SECONDARY_BG;
                visuals.inactive.weak_bg_fill = tokens::BUTTON_SECONDARY_BG;
                visuals.hovered.bg_fill = tokens::BUTTON_SECONDARY_HOVER_BG;
                visuals.hovered.weak_bg_fill = tokens::BUTTON_SECONDARY_HOVER_BG;
                visuals.active.bg_fill = tokens::BUTTON_SECONDARY_ACTIVE_BG;
                visuals.active.weak_bg_fill = tokens::BUTTON_SECONDARY_ACTIVE_BG;
                visuals.inactive.bg_stroke = Stroke::new(1.0, tokens::BUTTON_SECONDARY_BORDER);
                visuals.hovered.bg_stroke = Stroke::new(1.0, tokens::BUTTON_SECONDARY_HOVER_BORDER);
                visuals.active.bg_stroke = Stroke::new(1.0, tokens::BUTTON_SECONDARY_ACTIVE_BORDER);
            }
            ButtonVariant::Ghost => {
                visuals.inactive.bg_fill = Color32::TRANSPARENT;
                visuals.inactive.weak_bg_fill = Color32::TRANSPARENT;
                visuals.hovered.bg_fill = tokens::BUTTON_SECONDARY_HOVER_BG;
                visuals.hovered.weak_bg_fill = tokens::BUTTON_SECONDARY_HOVER_BG;
                visuals.active.bg_fill = tokens::BUTTON_SECONDARY_ACTIVE_BG;
                visuals.active.weak_bg_fill = tokens::BUTTON_SECONDARY_ACTIVE_BG;
                visuals.inactive.bg_stroke = Stroke::NONE;
                visuals.hovered.bg_stroke = Stroke::NONE;
                visuals.active.bg_stroke = Stroke::NONE;
            }
            ButtonVariant::Link => {
                visuals.inactive.bg_fill = Color32::TRANSPARENT;
                visuals.inactive.weak_bg_fill = Color32::TRANSPARENT;
                visuals.hovered.bg_fill = Color32::TRANSPARENT;
                visuals.hovered.weak_bg_fill = Color32::TRANSPARENT;
                visuals.active.bg_fill = Color32::TRANSPARENT;
                visuals.active.weak_bg_fill = Color32::TRANSPARENT;
                visuals.inactive.bg_stroke = Stroke::NONE;
                visuals.hovered.bg_stroke = Stroke::NONE;
                visuals.active.bg_stroke = Stroke::NONE;
                visuals.inactive.fg_stroke = Stroke::new(1.0, tokens::TEXT_SECONDARY);
                visuals.hovered.fg_stroke = Stroke::new(1.0, tokens::TEXT_PRIMARY);
                visuals.active.fg_stroke = Stroke::new(1.0, tokens::TEXT_PRIMARY);
            }
        }
        let button_label = match props.variant {
            ButtonVariant::Primary => {
                RichText::new(props.label).color(tokens::primary_fg(dark_mode))
            }
            _ => RichText::new(props.label),
        };
        let has_label = !props.label.is_empty();
        let icon_tint = props.icon_tint.unwrap_or(match props.variant {
            ButtonVariant::Primary => tokens::primary_fg(dark_mode),
            ButtonVariant::Link => tokens::TEXT_SECONDARY,
            _ => tokens::TEXT_PRIMARY,
        });

        let mut widget = match props
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
        };

        if let Some(right_text) = props.right_text {
            if props.right_text_weak {
                widget = widget.shortcut_text(right_text);
            } else {
                widget = widget.right_text(right_text);
            }
        }

        widget = widget.selected(props.selected);

        if props.variant == ButtonVariant::Link {
            widget = widget.frame(false);
        }
        if props.icon_only || (props.icon.is_some() && !has_label) {
            widget = widget.min_size(Vec2::splat(ui.spacing().interact_size.y));
        }
        if let Some(min_size) = props.min_size {
            widget = widget.min_size(min_size);
        }

        let response = ui.add(widget).on_hover_cursor(CursorIcon::PointingHand);
        if props.variant == ButtonVariant::Link
            && response.hovered()
            && has_label
            && props.icon.is_none()
            && !props.icon_only
        {
            let font_id = egui::TextStyle::Button.resolve(ui.style());
            let text_galley =
                ui.painter()
                    .layout_no_wrap(props.label.to_owned(), font_id, tokens::TEXT_PRIMARY);
            let text_half_width = text_galley.size().x * 0.5;
            let underline_y = response.rect.center().y + text_galley.size().y * 0.36;
            ui.painter().line_segment(
                [
                    egui::pos2(response.rect.center().x - text_half_width, underline_y),
                    egui::pos2(response.rect.center().x + text_half_width, underline_y),
                ],
                Stroke::new(1.0, tokens::TEXT_PRIMARY),
            );
        }

        response
    })
    .inner
}
