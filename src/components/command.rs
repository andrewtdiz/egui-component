use super::{api::ComponentUi, LabelTone, TextInput};
use crate::ui::tokens;
use egui::{containers::scroll_area::ScrollSource, Id, Response, RichText};

#[derive(Debug, Clone, Copy)]
pub struct CommandItem<'a> {
    pub group: &'a str,
    pub label: &'a str,
}

impl<'a> CommandItem<'a> {
    pub const fn new(group: &'a str, label: &'a str) -> Self {
        Self { group, label }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Command<'a> {
    pub id: Id,
    pub width: f32,
    pub max_height: f32,
    pub placeholder: &'a str,
}

impl<'a> Command<'a> {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            width: 220.0,
            max_height: 132.0,
            placeholder: "Type a command",
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }
}

impl<'a> From<Id> for Command<'a> {
    fn from(id: Id) -> Self {
        Self::new(id)
    }
}

impl<'a> From<(Id, f32)> for Command<'a> {
    fn from((id, width): (Id, f32)) -> Self {
        Self::new(id).width(width)
    }
}

impl<'a> From<(Id, f32, f32)> for Command<'a> {
    fn from((id, width, max_height): (Id, f32, f32)) -> Self {
        Self::new(id).width(width).max_height(max_height)
    }
}

impl ComponentUi<'_> {
    pub fn command<'a>(
        &mut self,
        query: &mut String,
        items: &[CommandItem<'_>],
        props: impl Into<Command<'a>>,
    ) -> Response {
        let props = props.into();
        let input_response = self.text_input(
            query,
            TextInput::new()
                .width(props.width)
                .hint_text(props.placeholder),
        );

        self.add_space(6.0);

        let _ = self.card((), |ui| {
            let _ = egui::ScrollArea::vertical()
                .id_salt(props.id)
                .scroll_source(ScrollSource {
                    drag: false,
                    ..ScrollSource::default()
                })
                .max_height(props.max_height)
                .auto_shrink([false, false])
                .show(ui.raw_mut(), |ui| {
                    let mut ui = super::api::ComponentUi::new(ui);
                    let dark_mode = ui.visuals().dark_mode;
                    let query_lower = query.to_ascii_lowercase();
                    let mut shown = 0usize;

                    for item in items {
                        let searchable =
                            format!("{} {}", item.group, item.label).to_ascii_lowercase();
                        if !query_lower.is_empty() && !searchable.contains(query_lower.as_str()) {
                            continue;
                        }

                        shown += 1;
                        ui.horizontal(|ui| {
                            let _ = ui.label((item.group, LabelTone::Muted, 11.0));
                            ui.add_space(8.0);
                            let _ = ui.add(
                                egui::Label::new(
                                    RichText::new(item.label)
                                        .color(tokens::text_primary(dark_mode)),
                                )
                                .selectable(false),
                            );
                        });
                    }

                    if shown == 0 {
                        let _ = ui.label(("No commands", LabelTone::Muted));
                    }
                });
        });

        input_response
    }
}
