use crate::components::chrome::with_input_chrome;
use crate::components::{
    card, label, scroll_area, CardProps, LabelProps, LabelTone, ScrollAreaProps,
};
use crate::ui::tokens;
use egui::{Id, Response, RichText, Ui};

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
pub struct CommandProps<'a> {
    pub id: Id,
    pub width: f32,
    pub max_height: f32,
    pub placeholder: &'a str,
}

impl<'a> CommandProps<'a> {
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

pub fn command(
    ui: &mut Ui,
    query: &mut String,
    items: &[CommandItem<'_>],
    props: CommandProps<'_>,
) -> Response {
    let input_response = with_input_chrome(ui, |ui| {
        ui.add_sized(
            [props.width, ui.spacing().interact_size.y],
            egui::TextEdit::singleline(query).hint_text(props.placeholder),
        )
    });

    ui.add_space(6.0);

    card(ui, CardProps::new(), |ui| {
        scroll_area(
            ui,
            ScrollAreaProps::new(props.id)
                .max_height(props.max_height)
                .auto_shrink([false, false]),
            |ui| {
                let query_lower = query.to_ascii_lowercase();
                let mut shown = 0usize;

                for item in items {
                    let searchable = format!("{} {}", item.group, item.label).to_ascii_lowercase();
                    if !query_lower.is_empty() && !searchable.contains(query_lower.as_str()) {
                        continue;
                    }

                    shown += 1;
                    ui.horizontal(|ui| {
                        let _ = label(
                            ui,
                            LabelProps::new(item.group)
                                .tone(LabelTone::Muted)
                                .size(11.0),
                        );
                        ui.add_space(8.0);
                        let _ = ui.add(
                            egui::Label::new(RichText::new(item.label).color(tokens::TEXT_PRIMARY))
                                .selectable(false),
                        );
                    });
                }

                if shown == 0 {
                    let _ = label(ui, LabelProps::new("No commands").tone(LabelTone::Muted));
                }
            },
        );
    });

    input_response
}
