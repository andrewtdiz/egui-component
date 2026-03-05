use crate::components::{
    button, label, ButtonProps, ButtonVariant, LabelProps, LabelTone, LabelWeight,
};
use egui::{Id, Response, Ui};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DialogVariant {
    Default,
    Alert,
}

#[derive(Debug, Clone, Copy)]
pub struct DialogProps<'a> {
    pub id: Id,
    pub title: &'a str,
    pub description: &'a str,
    pub trigger_label: &'a str,
    pub cancel_label: &'a str,
    pub confirm_label: &'a str,
    pub width: f32,
    pub variant: DialogVariant,
}

impl<'a> DialogProps<'a> {
    pub fn new(id: Id, title: &'a str) -> Self {
        Self {
            id,
            title,
            description: "",
            trigger_label: "Open",
            cancel_label: "Cancel",
            confirm_label: "Confirm",
            width: 360.0,
            variant: DialogVariant::Default,
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = description;
        self
    }

    pub fn trigger_label(mut self, trigger_label: &'a str) -> Self {
        self.trigger_label = trigger_label;
        self
    }

    pub fn cancel_label(mut self, cancel_label: &'a str) -> Self {
        self.cancel_label = cancel_label;
        self
    }

    pub fn confirm_label(mut self, confirm_label: &'a str) -> Self {
        self.confirm_label = confirm_label;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn variant(mut self, variant: DialogVariant) -> Self {
        self.variant = variant;
        self
    }
}

pub fn dialog(ui: &mut Ui, open: &mut bool, props: DialogProps<'_>) -> Response {
    let trigger_response = button(
        ui,
        ButtonProps::new(props.trigger_label).variant(ButtonVariant::Secondary),
    );
    if trigger_response.clicked() {
        *open = true;
    }

    if *open {
        egui::Window::new(props.title)
            .id(props.id)
            .collapsible(false)
            .resizable(false)
            .open(open)
            .default_width(props.width)
            .show(ui.ctx(), |ui| {
                let tone = if props.variant == DialogVariant::Alert {
                    LabelTone::Destructive
                } else {
                    LabelTone::Primary
                };
                let _ = label(
                    ui,
                    LabelProps::new(props.title)
                        .tone(tone)
                        .weight(LabelWeight::Semibold),
                );
                ui.add_space(6.0);
                let _ = label(
                    ui,
                    LabelProps::new(props.description).tone(LabelTone::Secondary),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if button(
                        ui,
                        ButtonProps::new(props.cancel_label).variant(ButtonVariant::Secondary),
                    )
                    .clicked()
                    {
                        ui.close();
                    }
                    if button(
                        ui,
                        ButtonProps::new(props.confirm_label).variant(ButtonVariant::Primary),
                    )
                    .clicked()
                    {
                        ui.close();
                    }
                });
            });
    }

    trigger_response
}
