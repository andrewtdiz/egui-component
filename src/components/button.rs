use crate::ui::tokens;
use egui::{Color32, CursorIcon, RichText, Stroke, Ui};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonProps<'a> {
    pub label: &'a str,
    pub variant: ButtonVariant,
}

impl<'a> ButtonProps<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            variant: ButtonVariant::Primary,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
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
        }
        let button_label = match props.variant {
            ButtonVariant::Primary => {
                RichText::new(props.label).color(tokens::primary_fg(dark_mode))
            }
            _ => RichText::new(props.label),
        };

        ui.button(button_label)
            .on_hover_cursor(CursorIcon::PointingHand)
    })
    .inner
}
