use egui::{CursorIcon, Response, Ui};

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

    egui::Popup::context_menu(&target).show(|ui| {
        if ui
            .button("Rename")
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            state.action = Some(ContextMenuAction::First);
            ui.close();
        }
        if ui
            .button("Duplicate")
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            state.action = Some(ContextMenuAction::Second);
            ui.close();
        }
        if ui
            .checkbox(toggle_value, toggle_label)
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            state.toggled = true;
            ui.close();
        }
        if ui
            .button("Delete")
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            state.action = Some(ContextMenuAction::Third);
            ui.close();
        }
    });

    (target, state)
}
