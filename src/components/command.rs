use super::{api::ComponentUi, api::ComponentUiExt, Kbd, KbdGroup, TextInput};
use crate::layout;
use crate::primitives::{
    content::muted_empty_state,
    row::{row_chrome, RowChrome},
    surface::{surface_frame_builder, SurfaceFrame},
};
use crate::ui::{tokens, typography};
use egui::{
    containers::scroll_area::ScrollSource, Align, Align2, FontFamily, FontId, Id, Key, Layout,
    Rect, Response, Sense, Stroke, Ui, UiBuilder,
};

const COMMAND_PANEL_PADDING_X: i8 = 10;
const COMMAND_PANEL_PADDING_Y: i8 = 8;
const COMMAND_ROW_HEIGHT: f32 = 30.0;
const COMMAND_ROW_PADDING_X: f32 = 8.0;
const COMMAND_ROW_GAP: f32 = 8.0;
const COMMAND_PREVIEW_HEIGHT: f32 = 244.0;
const COMMAND_PREVIEW_TOP_OFFSET: f32 = 8.0;
const COMMAND_KEYCAP_HEIGHT: f32 = 18.0;
const COMMAND_KEYCAP_MIN_WIDTH: f32 = 18.0;
const COMMAND_KEYCAP_PADDING_X: f32 = 4.0;
const COMMAND_KEYCAP_TEXT_SIZE: f32 = 9.0;
const COMMAND_KEYCAP_GAP: f32 = 3.0;

#[derive(Debug, Clone, Copy, Default)]
struct CommandMemory {
    selected_index: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct CommandItem<'a> {
    pub group: &'a str,
    pub label: &'a str,
    pub shortcut: Option<&'a str>,
}

impl<'a> CommandItem<'a> {
    pub const fn new(group: &'a str, label: &'a str) -> Self {
        Self {
            group,
            label,
            shortcut: None,
        }
    }

    pub const fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Command<'a> {
    pub id: Id,
    pub width: f32,
    pub max_height: f32,
    pub placeholder: &'a str,
    pub preview: bool,
    pub preview_height: f32,
}

impl<'a> Command<'a> {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            width: 360.0,
            max_height: 216.0,
            placeholder: "Execute a command...",
            preview: false,
            preview_height: COMMAND_PREVIEW_HEIGHT,
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

    pub fn preview(mut self, preview: bool) -> Self {
        self.preview = preview;
        self
    }

    pub fn preview_height(mut self, preview_height: f32) -> Self {
        self.preview_height = preview_height.max(1.0);
        self
    }
}

impl<'a> From<Id> for Command<'a> {
    fn from(id: Id) -> Self {
        Self::new(id)
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
        if props.preview {
            draw_command_preview(self, query, items, props)
        } else {
            draw_command_panel(self, query, items, props)
        }
    }
}

fn draw_command_preview(
    ui: &mut ComponentUi<'_>,
    query: &mut String,
    items: &[CommandItem<'_>],
    props: Command<'_>,
) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let overrides = ui.overrides();
    surface_frame_builder(
        SurfaceFrame::new(
            command_preview_fill(runtime),
            Stroke::new(1.0, tokens::separator(runtime)),
        )
        .corner_radius(tokens::radius_md(runtime))
        .padding(12, 12),
    )
    .show(ui.ui_mut(), |ui| {
        let mut ui = ComponentUi::with_overrides(ui, overrides);
        let canvas_size = egui::vec2(ui.available_width(), props.preview_height);
        let (canvas_rect, _) = ui.allocate_exact_size(canvas_size, Sense::hover());
        let panel_outer_width = (props.width + (f32::from(COMMAND_PANEL_PADDING_X) * 2.0))
            .min(canvas_rect.width())
            .max(1.0);
        let palette_rect = Rect::from_min_size(
            egui::pos2(
                canvas_rect.center().x - (panel_outer_width * 0.5),
                canvas_rect.top() + COMMAND_PREVIEW_TOP_OFFSET,
            ),
            egui::vec2(
                panel_outer_width,
                (canvas_rect.height() - COMMAND_PREVIEW_TOP_OFFSET).max(1.0),
            ),
        );
        let mut preview_props = props;
        preview_props.width =
            (panel_outer_width - (f32::from(COMMAND_PANEL_PADDING_X) * 2.0)).max(1.0);
        let overrides = ui.overrides();
        ui.ui_mut()
            .scope_builder(
                UiBuilder::new()
                    .max_rect(palette_rect)
                    .layout(Layout::top_down(Align::Min)),
                |ui| {
                    let mut ui = ComponentUi::with_overrides(ui, overrides);
                    draw_command_panel(&mut ui, query, items, preview_props)
                },
            )
            .inner
    })
    .inner
}

fn draw_command_panel(
    ui: &mut ComponentUi<'_>,
    query: &mut String,
    items: &[CommandItem<'_>],
    props: Command<'_>,
) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let overrides = ui.overrides();
    surface_frame_builder(
        SurfaceFrame::new(
            tokens::card_background(runtime),
            Stroke::new(1.0, tokens::separator(runtime)),
        )
        .corner_radius(tokens::radius_lg(runtime))
        .padding(COMMAND_PANEL_PADDING_X, COMMAND_PANEL_PADDING_Y)
        .shadow(tokens::tailwind_shadow_lg(runtime)),
    )
    .show(ui.ui_mut(), |ui| {
        let mut ui = ComponentUi::with_overrides(ui, overrides);
        ui.set_min_width(props.width);
        ui.set_max_width(props.width);
        let input_response = ui.text_input(
            query,
            TextInput::new()
                .width(ui.available_width())
                .placeholder(props.placeholder),
        );
        ui.add_space(6.0);
        let _ = ui.separator();
        ui.add_space(2.0);
        draw_command_results(&mut ui, query, items, props, input_response.has_focus());
        input_response
    })
    .inner
}

fn draw_command_results(
    ui: &mut ComponentUi<'_>,
    query: &str,
    items: &[CommandItem<'_>],
    props: Command<'_>,
    input_focused: bool,
) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let query_lower = query.to_ascii_lowercase();
    let visible_items = items
        .iter()
        .copied()
        .filter(|item| command_item_matches(*item, query_lower.as_str()))
        .collect::<Vec<_>>();
    let show_group_column = visible_items.iter().any(|item| !item.group.is_empty());
    let group_color = tokens::text_muted(runtime);
    let group_font = typography::small_font();
    let group_width = if show_group_column {
        visible_items
            .iter()
            .map(|item| text_width(ui.ui_mut(), item.group, &group_font, group_color))
            .fold(60.0, f32::max)
            .min(96.0)
    } else {
        0.0
    };
    let shortcut_width = visible_items
        .iter()
        .filter_map(|item| item.shortcut)
        .map(|shortcut| shortcut_group_width(ui.ui_mut(), &parse_shortcut_keys(shortcut)))
        .fold(0.0, f32::max);
    let query_has_text = !query.trim().is_empty();
    let selected_index = resolve_selected_index(
        ui.ui_mut(),
        props.id,
        query_has_text,
        input_focused,
        visible_items.len(),
    );
    let overrides = ui.overrides();

    let _ = egui::ScrollArea::vertical()
        .id_salt(props.id)
        .scroll_source(ScrollSource {
            drag: false,
            ..ScrollSource::default()
        })
        .max_height(props.max_height)
        .auto_shrink([false, false])
        .show(ui.ui_mut(), |ui| {
            let mut ui = ComponentUi::with_overrides(ui, overrides);
            let _ = layout::column().gap(1.0).show(ui.ui_mut(), |ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                if visible_items.is_empty() {
                    let _ = layout::sized_box()
                        .width(ui.available_width())
                        .height(40.0)
                        .show(ui.ui_mut(), |ui| {
                            let _ = layout::align()
                                .justify(layout::Justify::Center)
                                .align(layout::Align::Center)
                                .show(ui, |ui| muted_empty_state(ui, "No commands"));
                        });
                    return;
                }

                for (index, item) in visible_items.iter().copied().enumerate() {
                    draw_command_row(
                        &mut ui,
                        item,
                        group_width,
                        shortcut_width,
                        show_group_column,
                        selected_index == Some(index),
                    );
                }
            });
        });
}

fn draw_command_row(
    ui: &mut ComponentUi<'_>,
    item: CommandItem<'_>,
    group_width: f32,
    shortcut_width: f32,
    show_group_column: bool,
    selected: bool,
) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let shortcut_keys = item.shortcut.map(parse_shortcut_keys).unwrap_or_default();
    let desired_size = egui::vec2(ui.available_width(), COMMAND_ROW_HEIGHT);
    let (rect, response) = row_chrome(
        ui.ui_mut(),
        RowChrome::new(desired_size).corner_radius(tokens::radius_md(runtime)),
        |response| {
            if selected {
                tokens::primary_bg(runtime)
            } else {
                tokens::row_bg(
                    false,
                    response.is_pointer_button_down_on(),
                    response.hovered(),
                    runtime,
                )
            }
        },
    );
    let emphasized = selected || response.hovered() || response.is_pointer_button_down_on();
    let content_rect = Rect::from_min_max(
        egui::pos2(rect.left() + COMMAND_ROW_PADDING_X, rect.top()),
        egui::pos2(rect.right() - COMMAND_ROW_PADDING_X, rect.bottom()),
    );
    let trailing_width = if shortcut_keys.is_empty() {
        0.0
    } else {
        shortcut_width
    };
    let group_slot_width = if show_group_column { group_width } else { 0.0 };
    let shortcut_rect = if trailing_width > 0.0 {
        Some(Rect::from_min_max(
            egui::pos2(content_rect.right() - trailing_width, content_rect.top()),
            content_rect.right_bottom(),
        ))
    } else {
        None
    };
    let group_rect = if show_group_column {
        Some(Rect::from_min_max(
            content_rect.min,
            egui::pos2(
                content_rect.left() + group_slot_width,
                content_rect.bottom(),
            ),
        ))
    } else {
        None
    };
    let label_left = group_rect
        .map(|rect| rect.right() + COMMAND_ROW_GAP)
        .unwrap_or(content_rect.left());
    let label_right = shortcut_rect
        .map(|rect| rect.left() - COMMAND_ROW_GAP)
        .unwrap_or(content_rect.right())
        .max(label_left + 1.0);
    let label_rect = Rect::from_min_max(
        egui::pos2(label_left, content_rect.top()),
        egui::pos2(label_right, content_rect.bottom()),
    );

    if let Some(group_rect) = group_rect {
        let group_color = if selected {
            tokens::primary_fg(runtime)
        } else if emphasized {
            tokens::text_secondary(runtime)
        } else {
            tokens::text_muted(runtime)
        };
        ui.painter().with_clip_rect(group_rect).text(
            egui::pos2(group_rect.left(), group_rect.center().y),
            Align2::LEFT_CENTER,
            item.group,
            typography::proportional(11.0),
            group_color,
        );
    }

    let label_color = if selected {
        tokens::primary_fg(runtime)
    } else if emphasized {
        tokens::text_primary(runtime)
    } else {
        tokens::text_secondary(runtime)
    };
    ui.painter().with_clip_rect(label_rect).text(
        egui::pos2(label_rect.left(), label_rect.center().y),
        Align2::LEFT_CENTER,
        item.label,
        typography::proportional(14.0),
        label_color,
    );

    if let Some(shortcut_rect) = shortcut_rect {
        let overrides = ui.overrides();
        let _ = ui
            .ui_mut()
            .scope_builder(UiBuilder::new().max_rect(shortcut_rect), |ui| {
                ui.set_min_width(shortcut_rect.width());
                ui.set_max_width(shortcut_rect.width());
                let _ = layout::align()
                    .justify(layout::Justify::End)
                    .align(layout::Align::Center)
                    .show(ui, |ui| {
                        let mut ui = ComponentUi::with_overrides(ui, overrides);
                        let _ =
                            render_shortcut_keycaps(&mut ui, &shortcut_keys, selected, emphasized);
                    });
            });
    }
}

fn command_item_matches(item: CommandItem<'_>, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let searchable = format!("{} {}", item.group, item.label).to_ascii_lowercase();
    searchable.contains(query)
}

fn command_preview_fill(runtime: crate::theme::ThemeRuntime) -> egui::Color32 {
    if runtime.mode.is_dark() {
        tokens::app_background(runtime)
    } else {
        tokens::muted_surface(runtime)
    }
}

fn text_width(ui: &mut Ui, text: &str, font_id: &FontId, color: egui::Color32) -> f32 {
    ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font_id.clone(), color)
            .size()
            .x
    })
}

fn render_shortcut_keycaps(
    ui: &mut ComponentUi<'_>,
    keys: &[&str],
    selected: bool,
    emphasized: bool,
) -> egui::InnerResponse<()> {
    let runtime = crate::theme::runtime_for_ui(ui);
    let text_color = shortcut_text_color(ui, selected, emphasized);
    let key_fill = if selected {
        tokens::primary_fg(runtime).gamma_multiply(0.14)
    } else if emphasized {
        tokens::input_hover_background(runtime)
    } else {
        tokens::input_background(runtime)
    };
    let key_stroke = if selected {
        Stroke::new(1.0, tokens::primary_fg(runtime).gamma_multiply(0.28))
    } else if emphasized {
        Stroke::new(1.0, tokens::input_hover_border(runtime))
    } else {
        Stroke::new(1.0, tokens::input_border(runtime))
    };
    ui.kbd_group(KbdGroup::new().gap(COMMAND_KEYCAP_GAP), |ui| {
        let mut ui = ui.components();
        for key in keys {
            let _ = ui.kbd(
                Kbd::new(key)
                    .min_width(COMMAND_KEYCAP_MIN_WIDTH)
                    .height(COMMAND_KEYCAP_HEIGHT)
                    .padding(COMMAND_KEYCAP_PADDING_X, 0.0)
                    .text_size(COMMAND_KEYCAP_TEXT_SIZE)
                    .fill(key_fill)
                    .stroke(key_stroke)
                    .text_color(text_color),
            );
        }
    })
}

fn shortcut_text_color(ui: &ComponentUi<'_>, selected: bool, emphasized: bool) -> egui::Color32 {
    let runtime = crate::theme::runtime_for_ui(ui);
    if selected {
        tokens::primary_fg(runtime)
    } else if emphasized {
        tokens::text_secondary(runtime)
    } else {
        tokens::text_muted(runtime)
    }
}

fn shortcut_group_width(ui: &mut Ui, keys: &[&str]) -> f32 {
    if keys.is_empty() {
        return 0.0;
    }

    keys.iter()
        .enumerate()
        .map(|(index, key)| {
            let width = command_keycap_width(ui, key);
            if index == 0 {
                width
            } else {
                COMMAND_KEYCAP_GAP + width
            }
        })
        .sum()
}

fn command_keycap_width(ui: &mut Ui, text: &str) -> f32 {
    let text_color = tokens::text_muted(crate::theme::runtime_for_ui(ui));
    let font_id = FontId::new(COMMAND_KEYCAP_TEXT_SIZE, FontFamily::Monospace);
    let width = text_width(ui, text, &font_id, text_color);
    (width + (COMMAND_KEYCAP_PADDING_X * 2.0)).max(COMMAND_KEYCAP_MIN_WIDTH)
}

fn parse_shortcut_keys(shortcut: &str) -> Vec<&str> {
    shortcut
        .split('+')
        .flat_map(|segment| segment.split_whitespace())
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(normalize_shortcut_key)
        .collect()
}

fn normalize_shortcut_key(key: &str) -> &str {
    if key.eq_ignore_ascii_case("command") || key.eq_ignore_ascii_case("cmd") {
        "Cmd"
    } else if key.eq_ignore_ascii_case("control") || key.eq_ignore_ascii_case("ctrl") {
        "Ctrl"
    } else if key.eq_ignore_ascii_case("option") || key.eq_ignore_ascii_case("alt") {
        "Alt"
    } else if key.eq_ignore_ascii_case("shift") {
        "Shift"
    } else {
        key
    }
}

fn resolve_selected_index(
    ui: &mut Ui,
    id: Id,
    query_has_text: bool,
    input_focused: bool,
    visible_item_count: usize,
) -> Option<usize> {
    let mut memory = load_command_memory(ui, id);
    if !query_has_text || !input_focused || visible_item_count == 0 {
        memory.selected_index = None;
        store_command_memory(ui, id, memory);
        return None;
    }

    if let Some(selected_index) = memory.selected_index {
        if selected_index >= visible_item_count {
            memory.selected_index = Some(visible_item_count - 1);
        }
    }

    let move_down = ui.input(|input| input.key_pressed(Key::ArrowDown));
    let move_up = ui.input(|input| input.key_pressed(Key::ArrowUp));

    if move_down {
        memory.selected_index = Some(match memory.selected_index {
            Some(selected_index) => (selected_index + 1).min(visible_item_count - 1),
            None => 0,
        });
    } else if move_up {
        memory.selected_index = Some(match memory.selected_index {
            Some(selected_index) => selected_index.saturating_sub(1),
            None => 0,
        });
    }

    store_command_memory(ui, id, memory);
    memory.selected_index
}

fn load_command_memory(ui: &Ui, id: Id) -> CommandMemory {
    ui.data(|data| {
        data.get_temp::<CommandMemory>(id.with("command_state"))
            .unwrap_or_default()
    })
}

fn store_command_memory(ui: &mut Ui, id: Id, memory: CommandMemory) {
    ui.data_mut(|data| {
        if memory.selected_index.is_some() {
            data.insert_temp(id.with("command_state"), memory);
        } else {
            data.remove::<CommandMemory>(id.with("command_state"));
        }
    });
}
