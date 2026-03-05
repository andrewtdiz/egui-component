use crate::components::{button, ButtonProps, ButtonVariant};
use egui::{Id, Ui};

#[derive(Debug, Clone, Copy)]
pub struct ButtonGroupProps<'a> {
    pub id: Id,
    pub options: &'a [&'a str],
}

impl<'a> ButtonGroupProps<'a> {
    pub fn new(id: Id, options: &'a [&'a str]) -> Self {
        Self { id, options }
    }
}

pub fn button_group(ui: &mut Ui, selected_index: &mut usize, props: ButtonGroupProps<'_>) {
    if props.options.is_empty() {
        *selected_index = 0;
        return;
    }

    let clamped_index = (*selected_index).min(props.options.len() - 1);
    *selected_index = clamped_index;

    ui.push_id(props.id, |ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.horizontal(|ui| {
            for (index, label) in props.options.iter().copied().enumerate() {
                let variant = if index == *selected_index {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Secondary
                };
                if button(ui, ButtonProps::new(label).variant(variant)).clicked() {
                    *selected_index = index;
                }
            }
        });
    });
}
