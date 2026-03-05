use egui::{CursorIcon, Response, Ui};

#[derive(Debug, Clone, Copy)]
pub struct DropdownMenuProps<'a> {
    pub trigger_label: &'a str,
    pub options: &'a [&'a str],
}

impl<'a> DropdownMenuProps<'a> {
    pub fn new(trigger_label: &'a str, options: &'a [&'a str]) -> Self {
        Self {
            trigger_label,
            options,
        }
    }
}

pub fn dropdown_menu(
    ui: &mut Ui,
    selected_index: &mut usize,
    props: DropdownMenuProps<'_>,
) -> Response {
    if props.options.is_empty() {
        *selected_index = 0;
        return ui.add(egui::Label::new("").selectable(false));
    }

    let response = ui
        .menu_button(props.trigger_label, |ui| {
            for (index, label) in props.options.iter().copied().enumerate() {
                if ui
                    .button(label)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    *selected_index = index;
                    ui.close();
                }
            }
        })
        .response
        .on_hover_cursor(CursorIcon::PointingHand);

    *selected_index = (*selected_index).min(props.options.len() - 1);
    response
}
