use super::api::ComponentUi;
use crate::layout;
use crate::primitives::{
    control::{control_frame, ControlFrame},
    popup::{popup_panel, PopupPanel},
    row::{icon_label_row, row_chrome, IconLabelRow, RowChrome},
};
use crate::ui::{tokens, typography};
use egui::{CursorIcon, Id, Ui};

const MENU_INNER_PADDING_X: i8 = 3;
const MENU_INNER_PADDING_Y: i8 = 3;
const MENU_ROW_HEIGHT: f32 = 32.0;

#[derive(Debug, Clone, Copy)]
pub struct Select<'a> {
    pub trigger_id: Id,
    pub popup_id: Id,
    pub options: &'a [&'a str],
    pub width: f32,
    pub placeholder: &'a str,
}

impl<'a> Select<'a> {
    pub fn new(trigger_id: Id, popup_id: Id, options: &'a [&'a str]) -> Self {
        Self {
            trigger_id,
            popup_id,
            options,
            width: 220.0,
            placeholder: "Select an option",
        }
    }

    pub fn from_id(id: Id, options: &'a [&'a str]) -> Self {
        Self::new(id.with("trigger"), id.with("popup"), options)
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }
}

impl ComponentUi<'_> {
    pub fn select<'a>(
        &mut self,
        selected_index: &mut Option<usize>,
        props: impl Into<Select<'a>>,
    ) -> egui::Response {
        draw_select(self.ui_mut(), selected_index, props.into())
    }
}

#[derive(Debug, Clone, Copy)]
struct ResolvedSelectStyle {
    fill: egui::Color32,
    stroke: egui::Stroke,
    text_color: egui::Color32,
    icon_color: egui::Color32,
}

fn resolve_select_style(
    runtime: crate::theme::ThemeRuntime,
    focused: bool,
    hovered: bool,
    has_selection: bool,
) -> ResolvedSelectStyle {
    ResolvedSelectStyle {
        fill: tokens::input_bg(runtime, focused, hovered),
        stroke: tokens::input_stroke(runtime, focused, hovered),
        text_color: if has_selection {
            tokens::text_primary(runtime)
        } else {
            tokens::text_muted(runtime)
        },
        icon_color: tokens::text_secondary(runtime),
    }
}

fn draw_select(
    ui: &mut Ui,
    selected_index: &mut Option<usize>,
    props: Select<'_>,
) -> egui::Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    if selected_index.is_some_and(|index| index >= props.options.len()) {
        *selected_index = None;
    }

    let selected_text = selected_index.and_then(|index| props.options.get(index).copied());
    let current_selection = *selected_index;
    let mut trigger = draw_trigger(ui, props, selected_text);
    let mut next_selection = None;

    let _ = egui::Popup::menu(&trigger)
        .id(props.popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .gap(4.0)
        .show(|ui| {
            let row_width = (props.width - f32::from(MENU_INNER_PADDING_X * 2)).max(96.0);
            popup_panel(
                ui,
                PopupPanel::new(props.width).padding(MENU_INNER_PADDING_X, MENU_INNER_PADDING_Y),
                |ui| {
                    ui.set_min_width(row_width);
                    ui.set_max_width(row_width);
                    let _ = layout::column().gap(2.0).show(ui, |ui| {
                        for (index, label) in props.options.iter().copied().enumerate() {
                            let option_response = draw_option_row(
                                ui,
                                label,
                                current_selection == Some(index),
                                row_width,
                                runtime,
                            );
                            if option_response.clicked() {
                                next_selection = Some(index);
                                ui.close();
                            }
                        }
                    });
                },
            );
        });

    if let Some(next_index) = next_selection {
        if current_selection != Some(next_index) {
            trigger.mark_changed();
        }
        *selected_index = Some(next_index);
    }

    trigger
}

fn draw_trigger(ui: &mut Ui, props: Select<'_>, selected_text: Option<&str>) -> egui::Response {
    ui.push_id(props.trigger_id, |ui| {
        let desired_size = egui::vec2(props.width, tokens::SPACING_INTERACT_HEIGHT);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        let focused = response.has_focus() || egui::Popup::is_id_open(ui.ctx(), props.popup_id);
        let hovered = response.hovered();
        let runtime = crate::theme::runtime_for_ui(ui);
        let style = resolve_select_style(runtime, focused, hovered, selected_text.is_some());
        control_frame(ui, rect, ControlFrame::new(style.fill, style.stroke));
        let row = IconLabelRow::new(
            selected_text.unwrap_or(props.placeholder),
            typography::label_font(),
            style.text_color,
        )
        .trailing_icon("chevron-down")
        .trailing_icon_size(12.0)
        .trailing_icon_tint(style.icon_color);
        let _ = icon_label_row(ui, rect, &row);
        response.on_hover_cursor(CursorIcon::PointingHand)
    })
    .inner
}

fn draw_option_row(
    ui: &mut Ui,
    label: &str,
    selected: bool,
    row_width: f32,
    runtime: crate::theme::ThemeRuntime,
) -> egui::Response {
    let desired_size = egui::vec2(row_width.max(96.0), MENU_ROW_HEIGHT);
    let (rect, response) = row_chrome(
        ui,
        RowChrome::new(desired_size)
            .corner_radius(tokens::radius_sm(runtime))
            .stroke(egui::Stroke::NONE),
        |response| {
            tokens::row_bg(
                selected,
                response.is_pointer_button_down_on(),
                response.hovered(),
                runtime,
            )
        },
    );
    let row = IconLabelRow::new(
        label,
        typography::label_font(),
        if selected {
            tokens::row_selected_text(runtime)
        } else {
            tokens::text_secondary(runtime)
        },
    );
    let _ = icon_label_row(ui, rect, &row);
    response
}
