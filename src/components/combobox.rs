use crate::components::chrome::with_input_chrome;
use crate::components::{
    card, label, scroll_area, CardProps, LabelProps, LabelTone, ScrollAreaProps,
};
use crate::ui::tokens;
use egui::{Color32, CornerRadius, CursorIcon, Id, Response, Stroke, StrokeKind, Ui};

#[derive(Debug, Clone, Copy)]
pub struct ComboboxProps<'a> {
    pub id: Id,
    pub options: &'a [&'a str],
    pub width: f32,
    pub max_height: f32,
    pub placeholder: &'a str,
}

impl<'a> ComboboxProps<'a> {
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

pub fn combobox(
    ui: &mut Ui,
    query: &mut String,
    selected_index: &mut usize,
    props: ComboboxProps<'_>,
) -> Response {
    let dark_mode = ui.visuals().dark_mode;
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

                for (index, option) in props.options.iter().copied().enumerate() {
                    if !query_lower.is_empty()
                        && !option.to_ascii_lowercase().contains(query_lower.as_str())
                    {
                        continue;
                    }

                    shown += 1;
                    let selected = *selected_index == index;
                    if draw_option_row(ui, option, selected, dark_mode).clicked() {
                        *selected_index = index;
                    }
                }

                if shown == 0 {
                    let _ = label(ui, LabelProps::new("No matches").tone(LabelTone::Muted));
                }
            },
        );
    });

    if props.options.is_empty() {
        *selected_index = 0;
    } else {
        *selected_index = (*selected_index).min(props.options.len() - 1);
    }

    input_response
}

fn draw_option_row(ui: &mut Ui, text: &str, selected: bool, dark_mode: bool) -> Response {
    let desired_size = egui::vec2(ui.available_width().max(96.0), ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let fill = if selected {
        tokens::row_selected_bg(dark_mode)
    } else if response.hovered() {
        tokens::ROW_HOVER_BG
    } else {
        Color32::TRANSPARENT
    };
    let stroke = if selected {
        Stroke::new(1.0, tokens::row_selected_border(dark_mode))
    } else {
        Stroke::NONE
    };

    ui.painter().rect(
        rect,
        CornerRadius::same(tokens::RADIUS_SM),
        fill,
        stroke,
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
            tokens::TEXT_SECONDARY
        },
    );

    response.on_hover_cursor(CursorIcon::PointingHand)
}
