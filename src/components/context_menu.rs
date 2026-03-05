use super::button::{button, ButtonProps, ButtonVariant};
use egui::{CursorIcon, Response, Ui};

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

    let target = button(
        ui,
        ButtonProps::new(props.target_label)
            .variant(ButtonVariant::Secondary)
            .min_size(egui::vec2(props.width, ui.spacing().interact_size.y)),
    )
    .on_hover_cursor(CursorIcon::PointingHand);

    egui::Popup::context_menu(&target).show(|ui| {
        ui.style_mut().spacing.item_spacing.y = 2.0;
        ui.set_min_width(props.width);
        ui.set_max_width(props.width);

        let row_width = props.width.max(96.0);
        if draw_menu_row(ui, "Rename", false, row_width).clicked() {
            state.action = Some(ContextMenuAction::First);
            ui.close();
        }

        if draw_menu_row(ui, "Duplicate", false, row_width).clicked() {
            state.action = Some(ContextMenuAction::Second);
            ui.close();
        }

        if draw_toggle_row(ui, toggle_label, *toggle_value, row_width).clicked() {
            *toggle_value = !*toggle_value;
            state.toggled = true;
            ui.close();
        }

        if draw_menu_row(ui, "Delete", false, row_width).clicked() {
            state.action = Some(ContextMenuAction::Third);
            ui.close();
        }
    });

    (target, state)
}

fn draw_menu_row(ui: &mut Ui, label: &str, selected: bool, row_width: f32) -> Response {
    button(
        ui,
        ButtonProps::new(label)
            .variant(ButtonVariant::Ghost)
            .selected(selected)
            .right_text("")
            .min_size(egui::vec2(row_width.max(96.0), MENU_ROW_HEIGHT)),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
}

fn draw_toggle_row(ui: &mut Ui, label: &str, selected: bool, row_width: f32) -> Response {
    let indicator = if selected { "On" } else { "Off" };
    button(
        ui,
        ButtonProps::new(label)
            .variant(ButtonVariant::Ghost)
            .selected(selected)
            .right_text_weak(indicator)
            .min_size(egui::vec2(row_width.max(96.0), MENU_ROW_HEIGHT)),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
}
