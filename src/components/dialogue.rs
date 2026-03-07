use super::{api::ComponentUi, Button, ButtonStyle, Label, LabelTone, LabelWeight};
use crate::ui::tokens;
use egui::{Align, Color32, CornerRadius, Id, Layout, Margin, Response, Sense, Stroke, UiBuilder};

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
        let dark_mode = self.visuals().dark_mode;
        let frame = egui::Frame::popup(self.style())
            .fill(tokens::card_background(dark_mode))
            .stroke(Stroke::new(1.0, tokens::separator(dark_mode)))
            .corner_radius(CornerRadius::same(tokens::RADIUS_LG))
            .inner_margin(Margin::symmetric(12, 12))
            .shadow(tokens::tailwind_shadow_lg());
        let dialogue_modal_response = egui::Modal::new(props.id)
            .frame(frame)
            .backdrop_color(Color32::from_black_alpha(if dark_mode { 160 } else { 96 }))
            .show(self.ctx(), |ui| {
                ui.set_min_width(props.width);
                ui.set_max_width(props.width);
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

        self.label(
            Label::new(title)
                .tone(tone)
                .weight(LabelWeight::Bold)
                .size(16.0),
        )
    }

    pub fn dialogue_description(&mut self, description: &str) -> Response {
        self.label((description, LabelTone::Muted, 12.0))
    }

    pub fn dialogue_header<'a>(&mut self, props: impl Into<DialogueHeader<'a>>) {
        let props = props.into();
        let _ = self.dialogue_title(props.title, props.style);
        if !props.description.is_empty() {
            self.add_space(6.0);
            let _ = self.dialogue_description(props.description);
        }
    }

    pub fn dialogue_header_with_close<'a>(
        &mut self,
        props: impl Into<DialogueHeader<'a>>,
        close_requested: &mut bool,
    ) {
        let props = props.into();
        let close_button_size = 28.0;
        let header_gap = self.spacing().item_spacing.x;
        let row_width = self.available_width();
        let (header_rect, _) =
            self.allocate_exact_size(egui::vec2(row_width, close_button_size), Sense::hover());
        let close_rect = egui::Rect::from_min_size(
            egui::pos2(header_rect.right() - close_button_size, header_rect.top()),
            egui::vec2(close_button_size, close_button_size),
        );
        let title_rect = egui::Rect::from_min_max(
            header_rect.min,
            egui::pos2(
                (close_rect.left() - header_gap).max(header_rect.left()),
                header_rect.bottom(),
            ),
        );

        let _ = self.scope_builder(
            UiBuilder::new()
                .max_rect(title_rect)
                .layout(Layout::top_down(Align::Min)),
            |ui| {
                let _ = ui.dialogue_title(props.title, props.style);
            },
        );

        if self
            .scope_builder(
                UiBuilder::new()
                    .max_rect(close_rect)
                    .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
                |ui| {
                    ui.button(
                        Button::icon_only("x")
                            .style(ButtonStyle::Ghost)
                            .icon_size(14.0)
                            .min_size(egui::vec2(close_button_size, close_button_size)),
                    )
                },
            )
            .inner
            .clicked()
        {
            *close_requested = true;
        }

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
            ui.dialogue_header_with_close(
                (props.title, props.description, props.style),
                close_requested,
            );
            ui.add_space(12.0);
            let footer_size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
            let _ = ui.allocate_ui_with_layout(
                footer_size,
                Layout::right_to_left(Align::Center),
                |ui| {
                    if ui
                        .button((props.confirm_label, ButtonStyle::Primary))
                        .clicked()
                    {
                        *close_requested = true;
                    }
                    if ui
                        .button((props.cancel_label, ButtonStyle::Secondary))
                        .clicked()
                    {
                        *close_requested = true;
                    }
                },
            );
        });

        trigger_response
    }
}
