use crate::ui::tokens;
use egui::{Stroke, Ui};

pub(crate) fn with_input_chrome<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        let dark_mode = ui.visuals().dark_mode;
        let style = ui.style_mut();
        style.visuals.text_edit_bg_color = Some(tokens::input_background(dark_mode));
        style.visuals.code_bg_color = tokens::input_background(dark_mode);
        style.visuals.selection.bg_fill = tokens::text_selection_bg(dark_mode);
        style.visuals.selection.stroke = Stroke::new(1.0, tokens::text_primary(dark_mode));
        let visuals = &mut style.visuals.widgets;
        visuals.noninteractive.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.inactive.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.hovered.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.active.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.open.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.inactive.bg_fill = tokens::input_background(dark_mode);
        visuals.inactive.weak_bg_fill = tokens::input_background(dark_mode);
        visuals.hovered.bg_fill = tokens::input_hover_background(dark_mode);
        visuals.hovered.weak_bg_fill = tokens::input_hover_background(dark_mode);
        visuals.active.bg_fill = tokens::input_focus_background(dark_mode);
        visuals.active.weak_bg_fill = tokens::input_focus_background(dark_mode);
        visuals.open.bg_fill = tokens::input_focus_background(dark_mode);
        visuals.open.weak_bg_fill = tokens::input_focus_background(dark_mode);
        visuals.inactive.bg_stroke = Stroke::new(1.0, tokens::input_border(dark_mode));
        visuals.hovered.bg_stroke = Stroke::new(1.0, tokens::input_hover_border(dark_mode));
        visuals.active.bg_stroke = tokens::input_focus_stroke(dark_mode);
        visuals.open.bg_stroke = tokens::input_focus_stroke(dark_mode);
        add(ui)
    })
    .inner
}

pub(crate) fn with_slider_chrome<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        let style = ui.style_mut();
        style.spacing.slider_rail_height = 4.0;
        style.visuals.selection.bg_fill = tokens::TRANSPARENT;
        style.visuals.selection.stroke = Stroke::NONE;
        let visuals = &mut style.visuals.widgets;
        visuals.noninteractive.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.inactive.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.hovered.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.active.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.open.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.inactive.bg_fill = tokens::TRANSPARENT;
        visuals.inactive.weak_bg_fill = tokens::TRANSPARENT;
        visuals.hovered.bg_fill = tokens::TRANSPARENT;
        visuals.hovered.weak_bg_fill = tokens::TRANSPARENT;
        visuals.active.bg_fill = tokens::TRANSPARENT;
        visuals.active.weak_bg_fill = tokens::TRANSPARENT;
        visuals.open.bg_fill = tokens::TRANSPARENT;
        visuals.open.weak_bg_fill = tokens::TRANSPARENT;
        visuals.noninteractive.bg_fill = tokens::TRANSPARENT;
        visuals.noninteractive.weak_bg_fill = tokens::TRANSPARENT;
        visuals.inactive.bg_stroke = Stroke::NONE;
        visuals.hovered.bg_stroke = Stroke::NONE;
        visuals.active.bg_stroke = Stroke::NONE;
        visuals.open.bg_stroke = Stroke::NONE;
        visuals.noninteractive.bg_stroke = Stroke::NONE;
        visuals.inactive.fg_stroke = Stroke::NONE;
        visuals.hovered.fg_stroke = Stroke::NONE;
        visuals.active.fg_stroke = Stroke::NONE;
        visuals.open.fg_stroke = Stroke::NONE;
        add(ui)
    })
    .inner
}
