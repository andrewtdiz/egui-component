use crate::components::{
    button, label, ButtonProps, ButtonVariant, LabelProps, LabelTone, LabelWeight,
};
use egui::{Id, Response, Ui};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DialogueVariant {
    Default,
    Alert,
}

#[derive(Debug, Clone, Copy)]
pub struct DialogueModalProps {
    pub id: Id,
    pub width: f32,
}

impl DialogueModalProps {
    pub fn new(id: Id) -> Self {
        Self { id, width: 360.0 }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(1.0);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DialogueHeaderProps<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub variant: DialogueVariant,
}

impl<'a> DialogueHeaderProps<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            description: "",
            variant: DialogueVariant::Default,
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = description;
        self
    }

    pub fn variant(mut self, variant: DialogueVariant) -> Self {
        self.variant = variant;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DialogueProps<'a> {
    pub id: Id,
    pub title: &'a str,
    pub description: &'a str,
    pub trigger_label: &'a str,
    pub cancel_label: &'a str,
    pub confirm_label: &'a str,
    pub width: f32,
    pub variant: DialogueVariant,
}

impl<'a> DialogueProps<'a> {
    pub fn new(id: Id, title: &'a str) -> Self {
        Self {
            id,
            title,
            description: "",
            trigger_label: "Open",
            cancel_label: "Cancel",
            confirm_label: "Confirm",
            width: 360.0,
            variant: DialogueVariant::Default,
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
        self.width = width.max(1.0);
        self
    }

    pub fn variant(mut self, variant: DialogueVariant) -> Self {
        self.variant = variant;
        self
    }
}

pub fn dialogue_modal(
    ui: &mut Ui,
    open: &mut bool,
    props: DialogueModalProps,
    add_contents: impl FnOnce(&mut Ui, &mut bool),
) {
    if !*open {
        return;
    }

    let mut close_requested = false;
    let dialogue_modal_response = egui::Modal::new(props.id).show(ui.ctx(), |ui| {
        ui.set_min_width(props.width);
        add_contents(ui, &mut close_requested);
    });

    if close_requested || dialogue_modal_response.should_close() {
        *open = false;
    }
}

pub fn dialogue_title(ui: &mut Ui, title: &str, variant: DialogueVariant) -> Response {
    let tone = if variant == DialogueVariant::Alert {
        LabelTone::Destructive
    } else {
        LabelTone::Primary
    };

    label(
        ui,
        LabelProps::new(title)
            .tone(tone)
            .weight(LabelWeight::Semibold),
    )
}

pub fn dialogue_description(ui: &mut Ui, description: &str) -> Response {
    label(
        ui,
        LabelProps::new(description)
            .tone(LabelTone::Secondary)
            .size(11.0),
    )
}

pub fn dialogue_header(ui: &mut Ui, props: DialogueHeaderProps<'_>) {
    let _ = dialogue_title(ui, props.title, props.variant);
    if !props.description.is_empty() {
        ui.add_space(6.0);
        let _ = dialogue_description(ui, props.description);
    }
}

pub fn dialogue_body<R>(
    ui: &mut Ui,
    add_content: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.vertical(add_content)
}

pub fn dialogue_footer<R>(
    ui: &mut Ui,
    add_actions: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.horizontal(add_actions)
}

pub fn dialogue(ui: &mut Ui, open: &mut bool, props: DialogueProps<'_>) -> Response {
    let trigger_response = button(
        ui,
        ButtonProps::new(props.trigger_label).variant(ButtonVariant::Secondary),
    );
    if trigger_response.clicked() {
        *open = true;
    }

    dialogue_modal(
        ui,
        open,
        DialogueModalProps::new(props.id).width(props.width),
        |ui, close_requested| {
            dialogue_header(
                ui,
                DialogueHeaderProps::new(props.title)
                    .description(props.description)
                    .variant(props.variant),
            );
            ui.add_space(10.0);
            let _ = dialogue_footer(ui, |ui| {
                if button(
                    ui,
                    ButtonProps::new(props.cancel_label).variant(ButtonVariant::Secondary),
                )
                .clicked()
                {
                    *close_requested = true;
                }
                if button(
                    ui,
                    ButtonProps::new(props.confirm_label).variant(ButtonVariant::Primary),
                )
                .clicked()
                {
                    *close_requested = true;
                }
            });
        },
    );

    trigger_response
}
