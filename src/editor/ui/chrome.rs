#![allow(dead_code)]

use crate::editor::ui::tokens;
use egui::{Button, Checkbox, ComboBox, CornerRadius, CursorIcon, Id, Response, Stroke, Ui};

pub(crate) fn draw_enum_combo_str(
    ui: &mut Ui,
    id: Id,
    current_value: &str,
    options: &[&str],
    min_width: f32,
) -> Option<String> {
    let display = if current_value.is_empty() {
        "<select>"
    } else {
        current_value
    };
    let current_index = options
        .iter()
        .position(|candidate| *candidate == current_value)
        .unwrap_or(usize::MAX);
    draw_enum_combo_indexed(
        ui,
        id,
        current_index,
        display,
        min_width,
        options.len(),
        |index| (index, options[index]),
    )
    .map(|index| options[index].to_owned())
}

pub(crate) fn draw_enum_combo_u32(
    ui: &mut Ui,
    id: Id,
    current_value: u32,
    options: &[(u32, &'static str)],
    min_width: f32,
) -> Option<u32> {
    let current_text = options
        .iter()
        .find_map(|(candidate, text)| {
            if *candidate == current_value {
                Some(*text)
            } else {
                None
            }
        })
        .unwrap_or("<select>");
    draw_enum_combo_indexed(
        ui,
        id,
        current_value,
        current_text,
        min_width,
        options.len(),
        |index| {
            let (candidate, text) = options[index];
            (candidate, text)
        },
    )
}

fn draw_enum_combo_indexed<'a, T: Copy + PartialEq>(
    ui: &mut Ui,
    id: Id,
    current_value: T,
    selected_text: &str,
    min_width: f32,
    option_count: usize,
    mut option_at: impl FnMut(usize) -> (T, &'a str),
) -> Option<T> {
    let mut selection = None;
    with_select_chrome(ui, |ui| {
        enum_popup_style(ui.style_mut());
        let combo = ComboBox::from_id_salt(id)
            .width(min_width.max(96.0))
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                ui.add_space(4.0);
                for index in 0..option_count {
                    let (candidate, text) = option_at(index);
                    let option_response = draw_enum_popup_row(ui, text, candidate == current_value);
                    if option_response.clicked() {
                        selection = Some(candidate);
                    }
                }
            });
        combo.response.on_hover_cursor(CursorIcon::PointingHand);
    });
    selection
}

fn draw_enum_popup_row(ui: &mut Ui, text: &str, selected: bool) -> Response {
    ui.scope(|ui| {
        let visuals = &mut ui.style_mut().visuals.widgets;
        visuals.inactive.bg_stroke = Stroke::NONE;
        visuals.hovered.bg_stroke = Stroke::NONE;
        visuals.active.bg_stroke = Stroke::NONE;
        visuals.open.bg_stroke = Stroke::NONE;
        ui.add_sized(
            [ui.available_width().max(96.0), ui.spacing().interact_size.y],
            Button::new(text).selected(selected),
        )
    })
    .inner
    .on_hover_cursor(CursorIcon::PointingHand)
}

fn enum_popup_style(style: &mut egui::Style) {
    style.spacing.menu_margin = egui::Margin::symmetric(2, 2);
    style.spacing.button_padding = egui::vec2(6.0, 1.0);
    style.spacing.item_spacing.y = 0.0;
    style.visuals.selection.bg_fill = tokens::neutral(700);
    style.visuals.selection.stroke = Stroke::new(1.0, tokens::neutral(600));
    style.visuals.popup_shadow.offset = [0, 2];
}

pub(crate) fn styled_checkbox(ui: &mut Ui, value: &mut bool) -> Response {
    let checkbox_control = |ui: &mut Ui, value: &mut bool| {
        ui.add_sized(
            [ui.spacing().interact_size.y, ui.spacing().interact_size.y],
            Checkbox::new(value, ""),
        )
    };

    if *value {
        return ui
            .scope(|ui| {
                let visuals = &mut ui.style_mut().visuals.widgets;
                let checked = tokens::blue(500);
                let checked_hovered = tokens::blue(400);
                let checked_active = tokens::blue(600);
                visuals.inactive.corner_radius = CornerRadius::same(3);
                visuals.hovered.corner_radius = CornerRadius::same(3);
                visuals.active.corner_radius = CornerRadius::same(3);
                visuals.open.corner_radius = CornerRadius::same(3);
                visuals.inactive.bg_fill = checked;
                visuals.inactive.weak_bg_fill = checked;
                visuals.hovered.bg_fill = checked_hovered;
                visuals.hovered.weak_bg_fill = checked_hovered;
                visuals.active.bg_fill = checked_active;
                visuals.active.weak_bg_fill = checked_active;
                visuals.inactive.bg_stroke = Stroke::new(1.0, checked);
                visuals.hovered.bg_stroke = Stroke::new(1.0, checked_hovered);
                visuals.active.bg_stroke = Stroke::new(1.0, checked_active);
                visuals.inactive.fg_stroke = Stroke::new(1.5, tokens::neutral(100));
                visuals.hovered.fg_stroke = Stroke::new(1.5, tokens::neutral(100));
                visuals.active.fg_stroke = Stroke::new(1.5, tokens::neutral(100));
                checkbox_control(ui, value)
            })
            .inner;
    }

    ui.scope(|ui| {
        let visuals = &mut ui.style_mut().visuals.widgets;
        visuals.inactive.corner_radius = CornerRadius::same(3);
        visuals.hovered.corner_radius = CornerRadius::same(3);
        visuals.active.corner_radius = CornerRadius::same(3);
        visuals.open.corner_radius = CornerRadius::same(3);
        visuals.inactive.bg_fill = tokens::INPUT_BACKGROUND;
        visuals.inactive.weak_bg_fill = tokens::INPUT_BACKGROUND;
        visuals.hovered.bg_fill = tokens::INPUT_HOVER_BACKGROUND;
        visuals.hovered.weak_bg_fill = tokens::INPUT_HOVER_BACKGROUND;
        visuals.active.bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.active.weak_bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.inactive.bg_stroke = Stroke::new(1.0, tokens::INPUT_BORDER);
        visuals.hovered.bg_stroke = Stroke::new(1.0, tokens::INPUT_HOVER_BORDER);
        visuals.active.bg_stroke = tokens::input_focus_stroke();
        checkbox_control(ui, value)
    })
    .inner
}

pub(crate) fn with_slider_thumb_chrome<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        let style = ui.style_mut();
        style.spacing.slider_rail_height = style.spacing.slider_rail_height.max(4.0);
        let visuals = &mut style.visuals.widgets;
        let inactive_fill = tokens::neutral(600);
        let hovered_fill = tokens::neutral(500);
        let active_fill = tokens::neutral(500);
        let open_fill = tokens::neutral(500);

        let inactive_stroke = Stroke::new(1.0, tokens::neutral(500));
        let hovered_stroke = Stroke::new(1.0, tokens::neutral(400));
        let active_stroke = Stroke::new(1.0, tokens::neutral(300));
        let open_stroke = Stroke::new(1.0, tokens::neutral(400));

        visuals.inactive.bg_fill = inactive_fill;
        visuals.inactive.weak_bg_fill = inactive_fill;
        visuals.hovered.bg_fill = hovered_fill;
        visuals.hovered.weak_bg_fill = hovered_fill;
        visuals.active.bg_fill = active_fill;
        visuals.active.weak_bg_fill = active_fill;
        visuals.open.bg_fill = open_fill;
        visuals.open.weak_bg_fill = open_fill;

        visuals.inactive.bg_stroke = inactive_stroke;
        visuals.hovered.bg_stroke = hovered_stroke;
        visuals.active.bg_stroke = active_stroke;
        visuals.open.bg_stroke = open_stroke;

        visuals.inactive.fg_stroke = inactive_stroke;
        visuals.hovered.fg_stroke = hovered_stroke;
        visuals.active.fg_stroke = active_stroke;
        visuals.open.fg_stroke = open_stroke;
        add(ui)
    })
    .inner
}

pub(crate) fn with_input_chrome<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        let style = ui.style_mut();
        style.visuals.text_edit_bg_color = Some(tokens::INPUT_BACKGROUND);
        style.visuals.code_bg_color = tokens::INPUT_BACKGROUND;
        style.visuals.selection.stroke = Stroke::NONE;
        let visuals = &mut style.visuals.widgets;
        visuals.inactive.bg_fill = tokens::INPUT_BACKGROUND;
        visuals.inactive.weak_bg_fill = tokens::INPUT_BACKGROUND;
        visuals.hovered.bg_fill = tokens::INPUT_HOVER_BACKGROUND;
        visuals.hovered.weak_bg_fill = tokens::INPUT_HOVER_BACKGROUND;
        visuals.active.bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.active.weak_bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.open.bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.open.weak_bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.inactive.bg_stroke = Stroke::NONE;
        visuals.hovered.bg_stroke = Stroke::NONE;
        visuals.active.bg_stroke = tokens::input_focus_stroke();
        visuals.open.bg_stroke = tokens::input_focus_stroke();
        add(ui)
    })
    .inner
}

pub(crate) fn with_select_chrome<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        let style = ui.style_mut();
        style.spacing.button_padding = egui::vec2(7.0, 1.0);
        style.spacing.menu_margin = egui::Margin::symmetric(2, 2);
        style.visuals.popup_shadow.offset = [0, 2];
        style.visuals.selection.bg_fill = tokens::neutral(700);
        style.visuals.selection.stroke = Stroke::new(1.0, tokens::neutral(600));
        let visuals = &mut style.visuals.widgets;
        visuals.inactive.bg_fill = tokens::INPUT_BACKGROUND;
        visuals.inactive.weak_bg_fill = tokens::INPUT_BACKGROUND;
        visuals.hovered.bg_fill = tokens::INPUT_HOVER_BACKGROUND;
        visuals.hovered.weak_bg_fill = tokens::INPUT_HOVER_BACKGROUND;
        visuals.active.bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.active.weak_bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.inactive.bg_stroke = Stroke::NONE;
        visuals.hovered.bg_stroke = Stroke::NONE;
        visuals.active.bg_stroke = tokens::input_focus_stroke();
        add(ui)
    })
    .inner
}
