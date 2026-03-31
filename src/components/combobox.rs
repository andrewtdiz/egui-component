use super::{
    api::{ComponentUi, ComponentUiExt},
    TextInput,
};
use crate::primitives::{
    content::muted_empty_state,
    control::{control_frame, ControlFrame},
    popup::{popup_panel, PopupPanel},
    row::{icon_label_row, row_chrome, IconLabelRow, RowChrome},
};
use crate::ui::{tokens, typography};
use egui::{
    containers::scroll_area::ScrollSource, Align, CursorIcon, Id, Layout, Response, Stroke, Ui,
};

const MENU_INNER_PADDING_X: i8 = 3;
const MENU_INNER_PADDING_Y: i8 = 3;
const MENU_ROW_HEIGHT: f32 = 32.0;
const CHECKMARK_SLOT_WIDTH: f32 = 16.0;
const CHECKMARK_SIZE: f32 = 11.0;
const CHECKMARK_LABEL_GAP: f32 = 8.0;
const ROW_PADDING_X: f32 = 10.0;

#[derive(Debug, Clone, Copy)]
pub struct Combobox<'a> {
    pub id: Id,
    pub options: &'a [&'a str],
    pub width: f32,
    pub max_height: f32,
    pub placeholder: &'a str,
    pub filter_placeholder: &'a str,
    pub searchable: bool,
}

impl<'a> Combobox<'a> {
    pub fn new(id: Id, options: &'a [&'a str]) -> Self {
        Self {
            id,
            options,
            width: 220.0,
            max_height: 104.0,
            placeholder: "Select options",
            filter_placeholder: "Filter",
            searchable: true,
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

    pub fn filter_placeholder(mut self, filter_placeholder: &'a str) -> Self {
        self.filter_placeholder = filter_placeholder;
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
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
        let open_state_id = props.id.with("open");
        let was_open = load_combobox_open(self.ui_mut(), open_state_id);
        let summary_text = selection_summary(selected_indices, props.options, props.placeholder);
        let mut trigger_response = draw_trigger(
            self.ui_mut(),
            popup_id,
            props.width,
            summary_text.as_str(),
            !selected_indices.is_empty(),
        );
        let mut popup_open = was_open;

        if trigger_response.clicked() {
            popup_open = !was_open;
        }

        let just_opened = popup_open && !was_open;

        let _ = egui::Popup::from_response(&trigger_response)
            .id(popup_id)
            .kind(egui::PopupKind::Popup)
            .open_bool(&mut popup_open)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .gap(4.0)
            .layout(Layout::top_down(Align::Min))
            .show(|ui| {
                let row_width = (props.width - f32::from(MENU_INNER_PADDING_X * 2)).max(96.0);
                popup_panel(
                    ui,
                    PopupPanel::new(props.width)
                        .padding(MENU_INNER_PADDING_X, MENU_INNER_PADDING_Y),
                    |ui| {
                        ui.set_min_width(row_width);
                        ui.set_max_width(row_width);
                        if props.searchable {
                            let input_response = ui.components().text_input(
                                query,
                                TextInput::new()
                                    .width(row_width)
                                    .placeholder(props.filter_placeholder),
                            );
                            if just_opened {
                                input_response.request_focus();
                            }

                            ui.add_space(6.0);
                        }
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
                                let query_lower = if props.searchable {
                                    query.to_ascii_lowercase()
                                } else {
                                    String::new()
                                };
                                let mut shown = 0usize;
                                let _ = ui.scope(|ui| {
                                    ui.spacing_mut().item_spacing.y = 2.0;
                                    ui.vertical(|ui| {
                                        for (index, option) in
                                            props.options.iter().copied().enumerate()
                                        {
                                            if props.searchable
                                                && !query_lower.is_empty()
                                                && !option
                                                    .to_ascii_lowercase()
                                                    .contains(query_lower.as_str())
                                            {
                                                continue;
                                            }

                                            shown += 1;
                                            let selected = is_selected(selected_indices, index);
                                            if draw_option_row(
                                                ui, option, selected, row_width, runtime,
                                            )
                                            .clicked()
                                            {
                                                toggle_selected_index(selected_indices, index);
                                                trigger_response.mark_changed();
                                            }
                                        }

                                        if shown == 0 {
                                            let _ = muted_empty_state(ui, "No matches");
                                        }
                                    })
                                });
                            });
                    },
                );
            });

        store_combobox_open(self.ui_mut(), open_state_id, popup_open);

        trigger_response
    }
}

fn draw_trigger(
    ui: &mut Ui,
    popup_id: Id,
    width: f32,
    summary_text: &str,
    has_selection: bool,
) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let desired_size = egui::vec2(width, tokens::SPACING_INTERACT_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let focused = response.has_focus() || egui::Popup::is_id_open(ui.ctx(), popup_id);
    let hovered = response.hovered();
    let fill = tokens::input_bg(runtime, focused, hovered);
    let stroke = tokens::input_stroke(runtime, focused, hovered);

    control_frame(ui, rect, ControlFrame::new(fill, stroke));
    let row = IconLabelRow::new(
        summary_text,
        typography::label_font(),
        if has_selection {
            tokens::text_primary(runtime)
        } else {
            tokens::text_muted(runtime)
        },
    )
    .padding_x(10.0)
    .trailing_icon("chevron-down")
    .trailing_icon_size(12.0)
    .trailing_icon_tint(tokens::text_secondary(runtime));
    let _ = icon_label_row(ui, rect, &row);

    response.on_hover_cursor(CursorIcon::PointingHand)
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

    let checkmark_rect = egui::Rect::from_center_size(
        egui::pos2(
            rect.left() + ROW_PADDING_X + CHECKMARK_SLOT_WIDTH * 0.5,
            rect.center().y,
        ),
        egui::vec2(CHECKMARK_SIZE, CHECKMARK_SIZE),
    );
    draw_checkmark(ui, checkmark_rect, selected, runtime);

    let row = IconLabelRow::new(
        text,
        typography::label_font(),
        if selected {
            tokens::row_selected_text(runtime)
        } else {
            tokens::text_secondary(runtime)
        },
    )
    .padding_x(ROW_PADDING_X + CHECKMARK_SLOT_WIDTH + CHECKMARK_LABEL_GAP);
    let _ = icon_label_row(ui, rect, &row);

    response
}

fn draw_checkmark(
    ui: &mut Ui,
    rect: egui::Rect,
    selected: bool,
    runtime: crate::theme::ThemeRuntime,
) {
    if !selected {
        return;
    }

    let check_stroke = Stroke::new(1.8, tokens::row_selected_text(runtime));
    let start = egui::pos2(rect.left() + rect.width() * 0.08, rect.center().y + 0.1);
    let middle = egui::pos2(
        rect.left() + rect.width() * 0.38,
        rect.bottom() - rect.height() * 0.18,
    );
    let end = egui::pos2(
        rect.right() - rect.width() * 0.04,
        rect.top() + rect.height() * 0.12,
    );
    ui.painter().line_segment([start, middle], check_stroke);
    ui.painter().line_segment([middle, end], check_stroke);
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

fn selection_summary(selected_indices: &[usize], options: &[&str], placeholder: &str) -> String {
    match selected_indices.len() {
        0 => placeholder.to_owned(),
        1 => options
            .get(selected_indices[0])
            .copied()
            .unwrap_or(placeholder)
            .to_owned(),
        count => format!("{count} selected"),
    }
}

fn load_combobox_open(ui: &mut Ui, id: Id) -> bool {
    ui.data(|data| data.get_temp::<bool>(id).unwrap_or(false))
}

fn store_combobox_open(ui: &mut Ui, id: Id, open: bool) {
    ui.data_mut(|data| {
        if open {
            data.insert_temp(id, true);
        } else {
            data.remove::<bool>(id);
        }
    });
}
