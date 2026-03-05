use crate::ui::tokens;
use egui::{Color32, CornerRadius, CursorIcon, Response, Stroke, StrokeKind, Ui};

const MENU_INNER_PADDING_X: i8 = 3;
const MENU_INNER_PADDING_Y: i8 = 3;
const MENU_ROW_HEIGHT: f32 = 32.0;

#[derive(Debug, Clone, Copy)]
pub struct ContextMenuProps<'a> {
    pub target_label: &'a str,
    pub width: f32,
}

impl<'a> ContextMenuProps<'a> {
    pub fn new(target_label: &'a str) -> Self {
        Self {
            target_label,
            width: 220.0,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ContextMenuAction {
    First,
    Second,
    Third,
}

#[derive(Debug, Clone, Copy)]
pub struct ContextMenuState {
    pub action: Option<ContextMenuAction>,
    pub toggled: bool,
}

pub fn context_menu(
    ui: &mut Ui,
    toggle_label: &str,
    toggle_value: &mut bool,
    props: ContextMenuProps<'_>,
) -> (Response, ContextMenuState) {
    let mut state = ContextMenuState {
        action: None,
        toggled: false,
    };

    let target = ui
        .add_sized(
            [props.width, ui.spacing().interact_size.y],
            egui::Button::new(props.target_label),
        )
        .on_hover_cursor(CursorIcon::PointingHand);

    let dark_mode = ui.visuals().dark_mode;
    egui::Popup::context_menu(&target).show(|ui| {
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

                if draw_menu_row(ui, "Rename", false, row_width, dark_mode).clicked() {
                    state.action = Some(ContextMenuAction::First);
                    ui.close();
                }

                if draw_menu_row(ui, "Duplicate", false, row_width, dark_mode).clicked() {
                    state.action = Some(ContextMenuAction::Second);
                    ui.close();
                }

                if draw_toggle_row(ui, toggle_label, *toggle_value, row_width, dark_mode).clicked()
                {
                    *toggle_value = !*toggle_value;
                    state.toggled = true;
                    ui.close();
                }

                if draw_menu_row(ui, "Delete", false, row_width, dark_mode).clicked() {
                    state.action = Some(ContextMenuAction::Third);
                    ui.close();
                }
            });
    });

    (target, state)
}

fn draw_menu_row(
    ui: &mut Ui,
    label: &str,
    selected: bool,
    row_width: f32,
    dark_mode: bool,
) -> Response {
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

fn draw_toggle_row(
    ui: &mut Ui,
    label: &str,
    selected: bool,
    row_width: f32,
    dark_mode: bool,
) -> Response {
    let response = draw_menu_row(ui, label, selected, row_width, dark_mode);
    let indicator = if selected { "On" } else { "Off" };
    let text_color = if selected {
        tokens::row_selected_text(dark_mode)
    } else {
        tokens::TEXT_MUTED
    };
    ui.painter().text(
        egui::pos2(response.rect.right() - 10.0, response.rect.center().y),
        egui::Align2::RIGHT_CENTER,
        indicator,
        egui::FontId::new(11.0, egui::FontFamily::Proportional),
        text_color,
    );
    response
}
