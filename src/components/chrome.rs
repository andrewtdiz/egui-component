use crate::ui::tokens;
use egui::{Stroke, Ui};

pub(crate) fn with_input_chrome<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        let dark_mode = ui.visuals().dark_mode;
        let style = ui.style_mut();
        style.visuals.text_edit_bg_color = Some(tokens::INPUT_BACKGROUND);
        style.visuals.code_bg_color = tokens::INPUT_BACKGROUND;
        style.visuals.selection.bg_fill = tokens::text_selection_bg(dark_mode);
        style.visuals.selection.stroke = Stroke::new(1.0, tokens::TEXT_PRIMARY);
        let visuals = &mut style.visuals.widgets;
        visuals.noninteractive.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.inactive.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.hovered.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.active.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.open.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.inactive.bg_fill = tokens::INPUT_BACKGROUND;
        visuals.inactive.weak_bg_fill = tokens::INPUT_BACKGROUND;
        visuals.hovered.bg_fill = tokens::INPUT_HOVER_BACKGROUND;
        visuals.hovered.weak_bg_fill = tokens::INPUT_HOVER_BACKGROUND;
        visuals.active.bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.active.weak_bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.open.bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.open.weak_bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.inactive.bg_stroke = Stroke::new(1.0, tokens::INPUT_BORDER);
        visuals.hovered.bg_stroke = Stroke::new(1.0, tokens::INPUT_HOVER_BORDER);
        visuals.active.bg_stroke = tokens::input_focus_stroke(dark_mode);
        visuals.open.bg_stroke = tokens::input_focus_stroke(dark_mode);
        add(ui)
    })
    .inner
}

pub(crate) fn with_slider_chrome<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        let dark_mode = ui.visuals().dark_mode;
        let style = ui.style_mut();
        style.spacing.slider_rail_height = style.spacing.slider_rail_height.max(4.0);
        style.visuals.selection.bg_fill = tokens::row_selected_bg(dark_mode);
        style.visuals.selection.stroke = Stroke::new(1.0, tokens::row_selected_border(dark_mode));
        let visuals = &mut style.visuals.widgets;
        visuals.noninteractive.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.inactive.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.hovered.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.active.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.open.corner_radius = egui::CornerRadius::same(tokens::RADIUS_MD);
        visuals.inactive.bg_fill = tokens::neutral(600);
        visuals.inactive.weak_bg_fill = tokens::neutral(600);
        visuals.hovered.bg_fill = tokens::neutral(500);
        visuals.hovered.weak_bg_fill = tokens::neutral(500);
        visuals.active.bg_fill = tokens::neutral(500);
        visuals.active.weak_bg_fill = tokens::neutral(500);
        visuals.open.bg_fill = tokens::neutral(500);
        visuals.open.weak_bg_fill = tokens::neutral(500);
        visuals.inactive.bg_stroke = Stroke::new(1.0, tokens::INPUT_BORDER);
        visuals.hovered.bg_stroke = Stroke::new(1.0, tokens::INPUT_HOVER_BORDER);
        visuals.active.bg_stroke = Stroke::new(1.0, tokens::neutral(400));
        visuals.open.bg_stroke = Stroke::new(1.0, tokens::neutral(400));
        visuals.inactive.fg_stroke = Stroke::new(1.0, tokens::INPUT_BORDER);
        visuals.hovered.fg_stroke = Stroke::new(1.0, tokens::INPUT_HOVER_BORDER);
        visuals.active.fg_stroke = Stroke::new(1.0, tokens::neutral(400));
        visuals.open.fg_stroke = Stroke::new(1.0, tokens::neutral(400));
        add(ui)
    })
    .inner
}
