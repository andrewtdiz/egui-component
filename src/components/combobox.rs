use super::{api::ComponentUi, TextInput};
use crate::primitives::{
    content::muted_empty_state,
    row::{icon_label_row, row_chrome, IconLabelRow, RowChrome},
};
use crate::ui::{tokens, typography};
use egui::{containers::scroll_area::ScrollSource, Id, Response, Ui};

#[derive(Debug, Clone, Copy)]
pub struct Combobox<'a> {
    pub id: Id,
    pub options: &'a [&'a str],
    pub width: f32,
    pub max_height: f32,
    pub placeholder: &'a str,
}

impl<'a> Combobox<'a> {
    pub fn new(id: Id, options: &'a [&'a str]) -> Self {
        Self {
            id,
            options,
            width: 220.0,
            max_height: 104.0,
            placeholder: "Filter",
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

impl ComponentUi<'_> {
    pub fn combobox<'a>(
        &mut self,
        query: &mut String,
        selected_index: &mut usize,
        props: impl Into<Combobox<'a>>,
    ) -> Response {
        let props = props.into();
        let input_response = self.text_input(
            query,
            TextInput::new()
                .width(props.width)
                .placeholder(props.placeholder),
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
                .show(ui, |ui| {
                    let runtime = crate::theme::runtime_for_ui(ui);
                    let query_lower = query.to_ascii_lowercase();
                    let mut shown = 0usize;

                    for (index, option) in props.options.iter().copied().enumerate() {
                        if !query_lower.is_empty()
                            && !option.to_ascii_lowercase().contains(query_lower.as_str())
                        {
                            continue;
                        }

                        shown += 1;
                        let selected = *selected_index == index;
                        if draw_option_row(ui, option, selected, runtime).clicked() {
                            *selected_index = index;
                        }
                    }

                    if shown == 0 {
                        let _ = muted_empty_state(ui, "No matches");
                    }
                });
        });

        if props.options.is_empty() {
            *selected_index = 0;
        } else {
            *selected_index = (*selected_index).min(props.options.len() - 1);
        }

        input_response
    }
}

fn draw_option_row(
    ui: &mut Ui,
    text: &str,
    selected: bool,
    runtime: crate::theme::ThemeRuntime,
) -> Response {
    let desired_size = egui::vec2(ui.available_width().max(96.0), ui.spacing().interact_size.y);
    let (rect, response) = row_chrome(
        ui,
        RowChrome::new(desired_size)
            .corner_radius(tokens::radius_sm(runtime))
            .stroke(egui::Stroke::NONE),
        |response| {
            tokens::row_bg(
                selected,
                response.is_pointer_button_down_on(),
                response.hovered(),
                runtime,
            )
        },
    );
    let row = IconLabelRow::new(
        text,
        typography::label_font(),
        if selected {
            tokens::row_selected_text(runtime)
        } else {
            tokens::text_secondary(runtime)
        },
    );
    let _ = icon_label_row(ui, rect, &row);

    response
}
