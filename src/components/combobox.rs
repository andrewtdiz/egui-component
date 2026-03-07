use super::{api::ComponentUi, LabelTone, TextInput};
use crate::ui::tokens;
use egui::{
    containers::scroll_area::ScrollSource, CornerRadius, CursorIcon, Id, Response, StrokeKind, Ui,
};

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

impl<'a> From<(Id, &'a [&'a str])> for Combobox<'a> {
    fn from((id, options): (Id, &'a [&'a str])) -> Self {
        Self::new(id, options)
    }
}

impl<'a> From<(Id, &'a [&'a str], f32)> for Combobox<'a> {
    fn from((id, options, width): (Id, &'a [&'a str], f32)) -> Self {
        Self::new(id, options).width(width)
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
        let dark_mode = self.visuals().dark_mode;
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
                    let mut ui = ComponentUi::new(ui);
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
                        if draw_option_row(ui.raw_mut(), option, selected, dark_mode).clicked() {
                            *selected_index = index;
                        }
                    }

                    if shown == 0 {
                        let _ = ui.label(("No matches", LabelTone::Muted));
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

fn draw_option_row(ui: &mut Ui, text: &str, selected: bool, dark_mode: bool) -> Response {
    let desired_size = egui::vec2(ui.available_width().max(96.0), ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let fill = tokens::row_bg(
        selected,
        response.is_pointer_button_down_on(),
        response.hovered(),
        dark_mode,
    );
    ui.painter().rect(
        rect,
        CornerRadius::same(tokens::RADIUS_SM),
        fill,
        egui::Stroke::NONE,
        StrokeKind::Outside,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 10.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::new(12.0, egui::FontFamily::Proportional),
        if selected {
            tokens::row_selected_text(dark_mode)
        } else {
            tokens::text_secondary(dark_mode)
        },
    );

    response.on_hover_cursor(CursorIcon::PointingHand)
}
