use super::{api::ComponentUi, TextInput};
use crate::primitives::{
    content::muted_empty_state,
    popup::{popup_panel, PopupPanel},
    row::{icon_label_row, row_chrome, IconLabelRow, RowChrome},
};
use crate::ui::{tokens, typography};
use egui::{
    containers::scroll_area::ScrollSource, CornerRadius, Id, Response, Stroke, StrokeKind, Ui,
};

const MENU_INNER_PADDING_X: i8 = 3;
const MENU_INNER_PADDING_Y: i8 = 3;
const MENU_ROW_HEIGHT: f32 = 32.0;
const CHECKBOX_SIZE: f32 = 16.0;
const CHECKBOX_CORNER_RADIUS: u8 = 4;
const CHECKBOX_LABEL_GAP: f32 = 8.0;
const ROW_PADDING_X: f32 = 10.0;

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
        self.width = width.max(1.0);
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height.max(1.0);
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
        selected_indices: &mut Vec<usize>,
        props: impl Into<Combobox<'a>>,
    ) -> Response {
        let props = props.into();
        sanitize_selected_indices(selected_indices, props.options.len());

        let popup_id = props.id.with("popup");
        let mut input_response = self.text_input(
            query,
            TextInput::new()
                .width(props.width)
                .placeholder(props.placeholder),
        );
        let mut popup_open =
            input_response.has_focus() || egui::Popup::is_id_open(self.ctx(), popup_id);

        if input_response.clicked() || input_response.changed() {
            popup_open = true;
        }

        let _ = egui::Popup::menu(&input_response)
            .id(popup_id)
            .open_bool(&mut popup_open)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .gap(4.0)
            .show(|ui| {
                let row_width = (props.width - f32::from(MENU_INNER_PADDING_X * 2)).max(96.0);
                popup_panel(
                    ui,
                    PopupPanel::new(props.width)
                        .padding(MENU_INNER_PADDING_X, MENU_INNER_PADDING_Y),
                    |ui| {
                        ui.set_min_width(row_width);
                        ui.set_max_width(row_width);
                        let _ = egui::ScrollArea::vertical()
                            .id_salt(props.id.with("scroll"))
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
                                        && !option
                                            .to_ascii_lowercase()
                                            .contains(query_lower.as_str())
                                    {
                                        continue;
                                    }

                                    shown += 1;
                                    let selected = is_selected(selected_indices, index);
                                    if draw_option_row(ui, option, selected, row_width, runtime)
                                        .clicked()
                                    {
                                        let changed =
                                            toggle_selected_index(selected_indices, index);
                                        if changed {
                                            input_response.mark_changed();
                                        }
                                    }
                                }

                                if shown == 0 {
                                    let _ = muted_empty_state(ui, "No matches");
                                }
                            });
                    },
                );
            });

        input_response
    }
}

fn draw_option_row(
    ui: &mut Ui,
    text: &str,
    selected: bool,
    row_width: f32,
    runtime: crate::theme::ThemeRuntime,
) -> Response {
    let desired_size = egui::vec2(row_width.max(96.0), MENU_ROW_HEIGHT);
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

    let checkbox_rect = egui::Rect::from_center_size(
        egui::pos2(
            rect.left() + ROW_PADDING_X + CHECKBOX_SIZE * 0.5,
            rect.center().y,
        ),
        egui::vec2(CHECKBOX_SIZE, CHECKBOX_SIZE),
    );
    draw_checkbox(ui, checkbox_rect, selected, runtime, &response);

    let row = IconLabelRow::new(
        text,
        typography::label_font(),
        if selected {
            tokens::row_selected_text(runtime)
        } else {
            tokens::text_secondary(runtime)
        },
    )
    .padding_x(ROW_PADDING_X + CHECKBOX_SIZE + CHECKBOX_LABEL_GAP);
    let _ = icon_label_row(ui, rect, &row);

    response
}

fn draw_checkbox(
    ui: &mut Ui,
    rect: egui::Rect,
    selected: bool,
    runtime: crate::theme::ThemeRuntime,
    response: &Response,
) {
    let hovered = response.hovered();
    let pressed = response.is_pointer_button_down_on();

    let (fill, stroke) = if selected {
        let checked_fill = if pressed {
            tokens::primary_active_bg(runtime)
        } else if hovered {
            tokens::primary_hover_bg(runtime)
        } else {
            tokens::primary_bg(runtime)
        };
        (checked_fill, Stroke::new(1.0, checked_fill))
    } else if pressed {
        (
            tokens::input_focus_background(runtime),
            Stroke::new(1.0, tokens::input_hover_border(runtime)),
        )
    } else if hovered {
        (
            tokens::input_hover_background(runtime),
            Stroke::new(1.0, tokens::input_hover_border(runtime)),
        )
    } else {
        (
            tokens::input_background(runtime),
            Stroke::new(1.0, tokens::input_border(runtime)),
        )
    };

    ui.painter().rect(
        rect,
        CornerRadius::same(CHECKBOX_CORNER_RADIUS),
        fill,
        stroke,
        StrokeKind::Outside,
    );

    if selected {
        let check_stroke = Stroke::new(1.8, tokens::primary_fg(runtime));
        let start = egui::pos2(rect.left() + rect.width() * 0.24, rect.center().y + 0.2);
        let middle = egui::pos2(
            rect.left() + rect.width() * 0.44,
            rect.bottom() - rect.height() * 0.28,
        );
        let end = egui::pos2(
            rect.right() - rect.width() * 0.22,
            rect.top() + rect.height() * 0.28,
        );
        ui.painter().line_segment([start, middle], check_stroke);
        ui.painter().line_segment([middle, end], check_stroke);
    }
}

fn sanitize_selected_indices(selected_indices: &mut Vec<usize>, option_count: usize) {
    selected_indices.retain(|index| *index < option_count);
    selected_indices.sort_unstable();
    selected_indices.dedup();
}

fn is_selected(selected_indices: &[usize], index: usize) -> bool {
    selected_indices.binary_search(&index).is_ok()
}

fn toggle_selected_index(selected_indices: &mut Vec<usize>, index: usize) -> bool {
    match selected_indices.binary_search(&index) {
        Ok(position) => {
            selected_indices.remove(position);
            true
        }
        Err(position) => {
            selected_indices.insert(position, index);
            true
        }
    }
}
