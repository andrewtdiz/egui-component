use crate::ui::{icons, tokens};
use egui::{Color32, CornerRadius, CursorIcon, Id, Stroke, StrokeKind, Ui};

const MENU_INNER_PADDING_X: i8 = 6;
const MENU_INNER_PADDING_Y: i8 = 6;
const MENU_ROW_HEIGHT: f32 = 32.0;

#[derive(Debug, Clone, Copy)]
pub struct SelectProps<'a> {
    pub trigger_id: Id,
    pub popup_id: Id,
    pub options: &'a [&'a str],
    pub width: f32,
    pub placeholder: &'a str,
}

impl<'a> SelectProps<'a> {
    pub fn new(trigger_id: Id, popup_id: Id, options: &'a [&'a str]) -> Self {
        Self {
            trigger_id,
            popup_id,
            options,
            width: 220.0,
            placeholder: "Select an option",
        }
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

pub fn select(
    ui: &mut Ui,
    selected_index: &mut Option<usize>,
    props: SelectProps<'_>,
) -> egui::Response {
    let dark_mode = ui.visuals().dark_mode;
    if selected_index.is_some_and(|index| index >= props.options.len()) {
        *selected_index = None;
    }

    let selected_text = selected_index.and_then(|index| props.options.get(index).copied());
    let current_selection = *selected_index;
    let mut trigger = draw_trigger(ui, props, selected_text, dark_mode);
    let mut next_selection = None;

    let _ = egui::Popup::menu(&trigger)
        .id(props.popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .gap(4.0)
        .show(|ui| {
            ui.style_mut().spacing.item_spacing.y = 2.0;
            ui.set_min_width(props.width);
            ui.set_max_width(props.width);

            let row_width = (props.width - f32::from(MENU_INNER_PADDING_X * 2)).max(96.0);
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(
                    MENU_INNER_PADDING_X,
                    MENU_INNER_PADDING_Y,
                ))
                .show(ui, |ui| {
                    ui.set_min_width(row_width);
                    ui.set_max_width(row_width);

                    for (index, label) in props.options.iter().copied().enumerate() {
                        let option_response = draw_option_row(
                            ui,
                            label,
                            current_selection == Some(index),
                            row_width,
                            dark_mode,
                        );
                        if option_response.clicked() {
                            next_selection = Some(index);
                            ui.close();
                        }
                    }
                });
        });

    if let Some(next_index) = next_selection {
        if current_selection != Some(next_index) {
            trigger.mark_changed();
        }
        *selected_index = Some(next_index);
    }

    trigger
}

fn draw_trigger(
    ui: &mut Ui,
    props: SelectProps<'_>,
    selected_text: Option<&str>,
    dark_mode: bool,
) -> egui::Response {
    ui.push_id(props.trigger_id, |ui| {
        let desired_size = egui::vec2(props.width, ui.spacing().interact_size.y);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        let focused = response.has_focus() || egui::Popup::is_id_open(ui.ctx(), props.popup_id);
        let fill = if focused {
            tokens::INPUT_FOCUS_BACKGROUND
        } else if response.hovered() {
            tokens::INPUT_HOVER_BACKGROUND
        } else {
            tokens::INPUT_BACKGROUND
        };
        let stroke = if focused {
            tokens::input_focus_stroke(dark_mode)
        } else if response.hovered() {
            Stroke::new(1.0, tokens::INPUT_HOVER_BORDER)
        } else {
            Stroke::new(1.0, tokens::INPUT_BORDER)
        };
        ui.painter().rect(
            rect,
            CornerRadius::same(tokens::RADIUS_MD),
            fill,
            stroke,
            StrokeKind::Outside,
        );
        ui.painter().text(
            egui::pos2(rect.left() + 10.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            selected_text.unwrap_or(props.placeholder),
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
            if selected_text.is_some() {
                tokens::TEXT_PRIMARY
            } else {
                tokens::TEXT_MUTED
            },
        );
        if let Some(image) = icons::image(ui.ctx(), "chevron-down", 12.0) {
            let icon_size = 12.0;
            let icon_rect = egui::Rect::from_center_size(
                egui::pos2(rect.right() - 12.0, rect.center().y),
                egui::vec2(icon_size, icon_size),
            );
            let _ = ui.put(icon_rect, image.tint(tokens::TEXT_SECONDARY));
        }
        response.on_hover_cursor(CursorIcon::PointingHand)
    })
    .inner
}

fn draw_option_row(
    ui: &mut Ui,
    label: &str,
    selected: bool,
    row_width: f32,
    dark_mode: bool,
) -> egui::Response {
    let desired_size = egui::vec2(row_width.max(96.0), MENU_ROW_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let fill = if selected {
        tokens::row_selected_bg(dark_mode)
    } else if response.hovered() {
        tokens::ROW_HOVER_BG
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(tokens::RADIUS_SM),
        fill,
        Stroke::NONE,
        StrokeKind::Outside,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 10.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::new(12.0, egui::FontFamily::Proportional),
        if selected {
            tokens::row_selected_text(dark_mode)
        } else {
            tokens::TEXT_SECONDARY
        },
    );
    response.on_hover_cursor(CursorIcon::PointingHand)
}
