use super::{api::ComponentUi, ButtonStyle, LabelTone, LabelWeight};
use egui::{Id, Response};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DialogueStyle {
    Default,
    Alert,
}

#[derive(Debug, Clone, Copy)]
pub struct DialogueModal {
    pub id: Id,
    pub width: f32,
}

impl DialogueModal {
    pub fn new(id: Id) -> Self {
        Self { id, width: 360.0 }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(1.0);
        self
    }
}

impl From<Id> for DialogueModal {
    fn from(id: Id) -> Self {
        Self::new(id)
    }
}

impl From<(Id, f32)> for DialogueModal {
    fn from((id, width): (Id, f32)) -> Self {
        Self::new(id).width(width)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DialogueHeader<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub style: DialogueStyle,
}

impl<'a> DialogueHeader<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            description: "",
            style: DialogueStyle::Default,
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = description;
        self
    }

    pub fn style(mut self, style: DialogueStyle) -> Self {
        self.style = style;
        self
    }
}

impl<'a> From<&'a str> for DialogueHeader<'a> {
    fn from(title: &'a str) -> Self {
        Self::new(title)
    }
}

impl<'a> From<(&'a str, &'a str)> for DialogueHeader<'a> {
    fn from((title, description): (&'a str, &'a str)) -> Self {
        Self::new(title).description(description)
    }
}

impl<'a> From<(&'a str, &'a str, DialogueStyle)> for DialogueHeader<'a> {
    fn from((title, description, style): (&'a str, &'a str, DialogueStyle)) -> Self {
        Self::new(title).description(description).style(style)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Dialogue<'a> {
    pub id: Id,
    pub title: &'a str,
    pub description: &'a str,
    pub trigger_label: &'a str,
    pub cancel_label: &'a str,
    pub confirm_label: &'a str,
    pub width: f32,
    pub style: DialogueStyle,
}

impl<'a> Dialogue<'a> {
    pub fn new(id: Id, title: &'a str) -> Self {
        Self {
            id,
            title,
            description: "",
            trigger_label: "Open",
            cancel_label: "Cancel",
            confirm_label: "Confirm",
            width: 360.0,
            style: DialogueStyle::Default,
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

    pub fn style(mut self, style: DialogueStyle) -> Self {
        self.style = style;
        self
    }
}

impl<'a> From<(Id, &'a str)> for Dialogue<'a> {
    fn from((id, title): (Id, &'a str)) -> Self {
        Self::new(id, title)
    }
}

impl<'a> From<(Id, &'a str, &'a str)> for Dialogue<'a> {
    fn from((id, title, description): (Id, &'a str, &'a str)) -> Self {
        Self::new(id, title).description(description)
    }
}

impl<'a> From<(Id, &'a str, f32)> for Dialogue<'a> {
    fn from((id, title, width): (Id, &'a str, f32)) -> Self {
        Self::new(id, title).width(width)
    }
}

impl<'a> From<(Id, &'a str, &'a str, DialogueStyle)> for Dialogue<'a> {
    fn from((id, title, description, style): (Id, &'a str, &'a str, DialogueStyle)) -> Self {
        Self::new(id, title).description(description).style(style)
    }
}

impl ComponentUi<'_> {
    pub fn dialogue_modal(
        &mut self,
        open: &mut bool,
        props: impl Into<DialogueModal>,
        add_contents: impl FnOnce(&mut ComponentUi<'_>, &mut bool),
    ) {
        let props = props.into();
        if !*open {
            return;
        }

        let mut close_requested = false;
        let dialogue_modal_response = egui::Modal::new(props.id).show(self.ctx(), |ui| {
            ui.set_min_width(props.width);
            let mut components = ComponentUi::new(ui);
            add_contents(&mut components, &mut close_requested);
        });

        if close_requested || dialogue_modal_response.should_close() {
            *open = false;
        }
    }

    pub fn dialogue_title(&mut self, title: &str, style: DialogueStyle) -> Response {
        let tone = if style == DialogueStyle::Alert {
            LabelTone::Destructive
        } else {
            LabelTone::Primary
        };

        self.label((title, tone, LabelWeight::Semibold))
    }

    pub fn dialogue_description(&mut self, description: &str) -> Response {
        self.label((description, LabelTone::Secondary, 11.0))
    }

    pub fn dialogue_header<'a>(&mut self, props: impl Into<DialogueHeader<'a>>) {
        let props = props.into();
        let _ = self.dialogue_title(props.title, props.style);
        if !props.description.is_empty() {
            self.add_space(6.0);
            let _ = self.dialogue_description(props.description);
        }
    }

    pub fn dialogue<'a>(&mut self, open: &mut bool, props: impl Into<Dialogue<'a>>) -> Response {
        let props = props.into();
        let trigger_response = self.button((props.trigger_label, ButtonStyle::Secondary));
        if trigger_response.clicked() {
            *open = true;
        }

        self.dialogue_modal(open, (props.id, props.width), |ui, close_requested| {
            ui.dialogue_header((props.title, props.description, props.style));
            ui.add_space(10.0);
            let _ = ui.horizontal(|ui| {
                if ui
                    .button((props.cancel_label, ButtonStyle::Secondary))
                    .clicked()
                {
                    *close_requested = true;
                }
                if ui
                    .button((props.confirm_label, ButtonStyle::Primary))
                    .clicked()
                {
                    *close_requested = true;
                }
            });
        });

        trigger_response
    }
}
