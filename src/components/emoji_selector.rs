use super::{
    api::{ComponentUi, ComponentUiExt},
    Button, ButtonVariant, Popover, PopoverAlign, PopoverSide, TextInput, Tooltip,
};
use crate::layout;
use crate::primitives::{
    content::muted_empty_state,
    control::{control_frame, ControlFrame},
};
use crate::ui::{tokens, twemoji, typography};
use egui::{Align2, CursorIcon, Id, Response, Sense, Stroke, StrokeKind, Ui};

const EMOJI_TRIGGER_SIZE: f32 = 38.0;
const EMOJI_TRIGGER_EMOJI_SIZE: f32 = 20.0;
const EMOJI_POPUP_WIDTH: f32 = 320.0;
const EMOJI_POPUP_MAX_HEIGHT: f32 = 360.0;
const EMOJI_CELL_SIZE: f32 = 32.0;
const EMOJI_CELL_EMOJI_SIZE: f32 = 20.0;
const EMOJI_CELL_GAP: f32 = 6.0;
const EMOJI_SECTION_GAP: f32 = 10.0;
const CATEGORY_BUTTON_SIZE: f32 = 30.0;
const CATEGORY_ICON_SIZE: f32 = 14.0;
const CATEGORY_BUTTON_GAP: f32 = 4.0;

#[derive(Debug, Clone, Copy)]
pub struct EmojiSelector<'a> {
    pub id: Id,
    pub popup_width: f32,
    pub popup_max_height: f32,
    pub placeholder: &'a str,
    pub trigger_variant: ButtonVariant,
}

impl<'a> EmojiSelector<'a> {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            popup_width: EMOJI_POPUP_WIDTH,
            popup_max_height: EMOJI_POPUP_MAX_HEIGHT,
            placeholder: "🙂",
            trigger_variant: ButtonVariant::Secondary,
        }
    }

    pub fn popup_width(mut self, popup_width: f32) -> Self {
        self.popup_width = popup_width.max(180.0);
        self
    }

    pub fn popup_max_height(mut self, popup_max_height: f32) -> Self {
        self.popup_max_height = popup_max_height.max(180.0);
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn trigger_variant(mut self, trigger_variant: ButtonVariant) -> Self {
        self.trigger_variant = trigger_variant;
        self
    }
}

impl From<Id> for EmojiSelector<'_> {
    fn from(id: Id) -> Self {
        Self::new(id)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum EmojiCategory {
    All,
    People,
    Nature,
    Food,
    Activity,
    Travel,
    Objects,
    Symbols,
    Flags,
}

impl Default for EmojiCategory {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Clone, Copy)]
struct EmojiCategoryTab {
    category: EmojiCategory,
    icon: &'static str,
}

const CATEGORY_TABS: [EmojiCategoryTab; 9] = [
    EmojiCategoryTab {
        category: EmojiCategory::All,
        icon: "grid-2x2",
    },
    EmojiCategoryTab {
        category: EmojiCategory::People,
        icon: "smile",
    },
    EmojiCategoryTab {
        category: EmojiCategory::Nature,
        icon: "leaf",
    },
    EmojiCategoryTab {
        category: EmojiCategory::Food,
        icon: "carrot",
    },
    EmojiCategoryTab {
        category: EmojiCategory::Activity,
        icon: "badge-check",
    },
    EmojiCategoryTab {
        category: EmojiCategory::Travel,
        icon: "plane",
    },
    EmojiCategoryTab {
        category: EmojiCategory::Objects,
        icon: "lightbulb",
    },
    EmojiCategoryTab {
        category: EmojiCategory::Symbols,
        icon: "sparkles",
    },
    EmojiCategoryTab {
        category: EmojiCategory::Flags,
        icon: "flag",
    },
];

#[derive(Debug, Clone, Copy)]
struct EmojiEntry {
    emoji: &'static str,
    label: &'static str,
    aliases: &'static [&'static str],
    category: EmojiCategory,
}

const EMOJI_ENTRIES: &[EmojiEntry] = &[
    EmojiEntry {
        emoji: "😀",
        label: "grinning face",
        aliases: &["smile", "happy"],
        category: EmojiCategory::People,
    },
    EmojiEntry {
        emoji: "😃",
        label: "grinning face with big eyes",
        aliases: &["joy"],
        category: EmojiCategory::People,
    },
    EmojiEntry {
        emoji: "😄",
        label: "grinning face with smiling eyes",
        aliases: &["cheerful"],
        category: EmojiCategory::People,
    },
    EmojiEntry {
        emoji: "😅",
        label: "grinning face with sweat",
        aliases: &["relief"],
        category: EmojiCategory::People,
    },
    EmojiEntry {
        emoji: "😂",
        label: "face with tears of joy",
        aliases: &["laugh"],
        category: EmojiCategory::People,
    },
    EmojiEntry {
        emoji: "🙂",
        label: "slightly smiling face",
        aliases: &["smile"],
        category: EmojiCategory::People,
    },
    EmojiEntry {
        emoji: "😊",
        label: "smiling face with smiling eyes",
        aliases: &["blush"],
        category: EmojiCategory::People,
    },
    EmojiEntry {
        emoji: "😉",
        label: "winking face",
        aliases: &["wink"],
        category: EmojiCategory::People,
    },
    EmojiEntry {
        emoji: "😍",
        label: "smiling face with heart-eyes",
        aliases: &["love"],
        category: EmojiCategory::People,
    },
    EmojiEntry {
        emoji: "🤔",
        label: "thinking face",
        aliases: &["hmm"],
        category: EmojiCategory::People,
    },
    EmojiEntry {
        emoji: "😎",
        label: "smiling face with sunglasses",
        aliases: &["cool"],
        category: EmojiCategory::People,
    },
    EmojiEntry {
        emoji: "😭",
        label: "loudly crying face",
        aliases: &["sad", "cry"],
        category: EmojiCategory::People,
    },
    EmojiEntry {
        emoji: "🐶",
        label: "dog face",
        aliases: &["puppy"],
        category: EmojiCategory::Nature,
    },
    EmojiEntry {
        emoji: "🐱",
        label: "cat face",
        aliases: &["kitty"],
        category: EmojiCategory::Nature,
    },
    EmojiEntry {
        emoji: "🐻",
        label: "bear",
        aliases: &[],
        category: EmojiCategory::Nature,
    },
    EmojiEntry {
        emoji: "🦊",
        label: "fox",
        aliases: &[],
        category: EmojiCategory::Nature,
    },
    EmojiEntry {
        emoji: "🐼",
        label: "panda",
        aliases: &[],
        category: EmojiCategory::Nature,
    },
    EmojiEntry {
        emoji: "🐸",
        label: "frog",
        aliases: &[],
        category: EmojiCategory::Nature,
    },
    EmojiEntry {
        emoji: "🌸",
        label: "cherry blossom",
        aliases: &["flower"],
        category: EmojiCategory::Nature,
    },
    EmojiEntry {
        emoji: "🌻",
        label: "sunflower",
        aliases: &["flower"],
        category: EmojiCategory::Nature,
    },
    EmojiEntry {
        emoji: "🌲",
        label: "evergreen tree",
        aliases: &["pine"],
        category: EmojiCategory::Nature,
    },
    EmojiEntry {
        emoji: "🌵",
        label: "cactus",
        aliases: &["desert"],
        category: EmojiCategory::Nature,
    },
    EmojiEntry {
        emoji: "🌈",
        label: "rainbow",
        aliases: &["weather"],
        category: EmojiCategory::Nature,
    },
    EmojiEntry {
        emoji: "🌙",
        label: "crescent moon",
        aliases: &["night"],
        category: EmojiCategory::Nature,
    },
    EmojiEntry {
        emoji: "🍎",
        label: "red apple",
        aliases: &["apple"],
        category: EmojiCategory::Food,
    },
    EmojiEntry {
        emoji: "🍌",
        label: "banana",
        aliases: &[],
        category: EmojiCategory::Food,
    },
    EmojiEntry {
        emoji: "🍇",
        label: "grapes",
        aliases: &[],
        category: EmojiCategory::Food,
    },
    EmojiEntry {
        emoji: "🍓",
        label: "strawberry",
        aliases: &["berry"],
        category: EmojiCategory::Food,
    },
    EmojiEntry {
        emoji: "🍕",
        label: "pizza",
        aliases: &["slice"],
        category: EmojiCategory::Food,
    },
    EmojiEntry {
        emoji: "🍔",
        label: "hamburger",
        aliases: &["burger"],
        category: EmojiCategory::Food,
    },
    EmojiEntry {
        emoji: "🍟",
        label: "french fries",
        aliases: &["fries"],
        category: EmojiCategory::Food,
    },
    EmojiEntry {
        emoji: "🌮",
        label: "taco",
        aliases: &[],
        category: EmojiCategory::Food,
    },
    EmojiEntry {
        emoji: "🍣",
        label: "sushi",
        aliases: &[],
        category: EmojiCategory::Food,
    },
    EmojiEntry {
        emoji: "☕",
        label: "hot beverage",
        aliases: &["coffee"],
        category: EmojiCategory::Food,
    },
    EmojiEntry {
        emoji: "🍰",
        label: "shortcake",
        aliases: &["cake"],
        category: EmojiCategory::Food,
    },
    EmojiEntry {
        emoji: "🍪",
        label: "cookie",
        aliases: &["dessert"],
        category: EmojiCategory::Food,
    },
    EmojiEntry {
        emoji: "⚽",
        label: "soccer ball",
        aliases: &["football"],
        category: EmojiCategory::Activity,
    },
    EmojiEntry {
        emoji: "🏀",
        label: "basketball",
        aliases: &[],
        category: EmojiCategory::Activity,
    },
    EmojiEntry {
        emoji: "🏈",
        label: "american football",
        aliases: &["football"],
        category: EmojiCategory::Activity,
    },
    EmojiEntry {
        emoji: "⚾",
        label: "baseball",
        aliases: &[],
        category: EmojiCategory::Activity,
    },
    EmojiEntry {
        emoji: "🎾",
        label: "tennis",
        aliases: &["racket"],
        category: EmojiCategory::Activity,
    },
    EmojiEntry {
        emoji: "🎲",
        label: "game die",
        aliases: &["dice"],
        category: EmojiCategory::Activity,
    },
    EmojiEntry {
        emoji: "🎮",
        label: "video game",
        aliases: &["controller"],
        category: EmojiCategory::Activity,
    },
    EmojiEntry {
        emoji: "🎯",
        label: "direct hit",
        aliases: &["target"],
        category: EmojiCategory::Activity,
    },
    EmojiEntry {
        emoji: "🎸",
        label: "guitar",
        aliases: &["music"],
        category: EmojiCategory::Activity,
    },
    EmojiEntry {
        emoji: "🎨",
        label: "artist palette",
        aliases: &["art", "paint"],
        category: EmojiCategory::Activity,
    },
    EmojiEntry {
        emoji: "🧩",
        label: "puzzle piece",
        aliases: &["puzzle"],
        category: EmojiCategory::Activity,
    },
    EmojiEntry {
        emoji: "🏆",
        label: "trophy",
        aliases: &["award"],
        category: EmojiCategory::Activity,
    },
    EmojiEntry {
        emoji: "🚗",
        label: "automobile",
        aliases: &["car"],
        category: EmojiCategory::Travel,
    },
    EmojiEntry {
        emoji: "🚕",
        label: "taxi",
        aliases: &["cab"],
        category: EmojiCategory::Travel,
    },
    EmojiEntry {
        emoji: "🚌",
        label: "bus",
        aliases: &[],
        category: EmojiCategory::Travel,
    },
    EmojiEntry {
        emoji: "🚲",
        label: "bicycle",
        aliases: &["bike"],
        category: EmojiCategory::Travel,
    },
    EmojiEntry {
        emoji: "✈️",
        label: "airplane",
        aliases: &["plane"],
        category: EmojiCategory::Travel,
    },
    EmojiEntry {
        emoji: "🚀",
        label: "rocket",
        aliases: &["space"],
        category: EmojiCategory::Travel,
    },
    EmojiEntry {
        emoji: "🚂",
        label: "locomotive",
        aliases: &["train"],
        category: EmojiCategory::Travel,
    },
    EmojiEntry {
        emoji: "🚢",
        label: "ship",
        aliases: &["boat"],
        category: EmojiCategory::Travel,
    },
    EmojiEntry {
        emoji: "🗺️",
        label: "world map",
        aliases: &["map"],
        category: EmojiCategory::Travel,
    },
    EmojiEntry {
        emoji: "🏝️",
        label: "desert island",
        aliases: &["island"],
        category: EmojiCategory::Travel,
    },
    EmojiEntry {
        emoji: "🗽",
        label: "statue of liberty",
        aliases: &["new york"],
        category: EmojiCategory::Travel,
    },
    EmojiEntry {
        emoji: "🧳",
        label: "luggage",
        aliases: &["travel"],
        category: EmojiCategory::Travel,
    },
    EmojiEntry {
        emoji: "💡",
        label: "light bulb",
        aliases: &["idea"],
        category: EmojiCategory::Objects,
    },
    EmojiEntry {
        emoji: "🔒",
        label: "locked",
        aliases: &["lock"],
        category: EmojiCategory::Objects,
    },
    EmojiEntry {
        emoji: "🔑",
        label: "key",
        aliases: &["unlock"],
        category: EmojiCategory::Objects,
    },
    EmojiEntry {
        emoji: "📌",
        label: "pushpin",
        aliases: &["pin"],
        category: EmojiCategory::Objects,
    },
    EmojiEntry {
        emoji: "📎",
        label: "paperclip",
        aliases: &["clip"],
        category: EmojiCategory::Objects,
    },
    EmojiEntry {
        emoji: "✏️",
        label: "pencil",
        aliases: &["write"],
        category: EmojiCategory::Objects,
    },
    EmojiEntry {
        emoji: "📷",
        label: "camera",
        aliases: &["photo"],
        category: EmojiCategory::Objects,
    },
    EmojiEntry {
        emoji: "📱",
        label: "mobile phone",
        aliases: &["phone"],
        category: EmojiCategory::Objects,
    },
    EmojiEntry {
        emoji: "💻",
        label: "laptop",
        aliases: &["computer"],
        category: EmojiCategory::Objects,
    },
    EmojiEntry {
        emoji: "⏰",
        label: "alarm clock",
        aliases: &["clock"],
        category: EmojiCategory::Objects,
    },
    EmojiEntry {
        emoji: "🎁",
        label: "wrapped gift",
        aliases: &["gift", "present"],
        category: EmojiCategory::Objects,
    },
    EmojiEntry {
        emoji: "🔋",
        label: "battery",
        aliases: &["power"],
        category: EmojiCategory::Objects,
    },
    EmojiEntry {
        emoji: "❤️",
        label: "red heart",
        aliases: &["heart", "love"],
        category: EmojiCategory::Symbols,
    },
    EmojiEntry {
        emoji: "⭐",
        label: "star",
        aliases: &["favorite"],
        category: EmojiCategory::Symbols,
    },
    EmojiEntry {
        emoji: "✅",
        label: "check mark button",
        aliases: &["check"],
        category: EmojiCategory::Symbols,
    },
    EmojiEntry {
        emoji: "❌",
        label: "cross mark",
        aliases: &["x"],
        category: EmojiCategory::Symbols,
    },
    EmojiEntry {
        emoji: "⚠️",
        label: "warning",
        aliases: &["alert"],
        category: EmojiCategory::Symbols,
    },
    EmojiEntry {
        emoji: "❓",
        label: "question mark",
        aliases: &["help"],
        category: EmojiCategory::Symbols,
    },
    EmojiEntry {
        emoji: "💯",
        label: "hundred points",
        aliases: &["100"],
        category: EmojiCategory::Symbols,
    },
    EmojiEntry {
        emoji: "🔔",
        label: "bell",
        aliases: &["notification"],
        category: EmojiCategory::Symbols,
    },
    EmojiEntry {
        emoji: "✨",
        label: "sparkles",
        aliases: &["shine"],
        category: EmojiCategory::Symbols,
    },
    EmojiEntry {
        emoji: "☮️",
        label: "peace symbol",
        aliases: &["peace"],
        category: EmojiCategory::Symbols,
    },
    EmojiEntry {
        emoji: "☯️",
        label: "yin yang",
        aliases: &["balance"],
        category: EmojiCategory::Symbols,
    },
    EmojiEntry {
        emoji: "♻️",
        label: "recycling symbol",
        aliases: &["recycle"],
        category: EmojiCategory::Symbols,
    },
    EmojiEntry {
        emoji: "🇺🇸",
        label: "flag: United States",
        aliases: &["usa", "america"],
        category: EmojiCategory::Flags,
    },
    EmojiEntry {
        emoji: "🇨🇦",
        label: "flag: Canada",
        aliases: &["canada"],
        category: EmojiCategory::Flags,
    },
    EmojiEntry {
        emoji: "🇬🇧",
        label: "flag: United Kingdom",
        aliases: &["uk", "britain"],
        category: EmojiCategory::Flags,
    },
    EmojiEntry {
        emoji: "🇯🇵",
        label: "flag: Japan",
        aliases: &["japan"],
        category: EmojiCategory::Flags,
    },
    EmojiEntry {
        emoji: "🇰🇷",
        label: "flag: South Korea",
        aliases: &["korea"],
        category: EmojiCategory::Flags,
    },
    EmojiEntry {
        emoji: "🇫🇷",
        label: "flag: France",
        aliases: &["france"],
        category: EmojiCategory::Flags,
    },
    EmojiEntry {
        emoji: "🇩🇪",
        label: "flag: Germany",
        aliases: &["germany"],
        category: EmojiCategory::Flags,
    },
    EmojiEntry {
        emoji: "🇪🇸",
        label: "flag: Spain",
        aliases: &["spain"],
        category: EmojiCategory::Flags,
    },
    EmojiEntry {
        emoji: "🇮🇹",
        label: "flag: Italy",
        aliases: &["italy"],
        category: EmojiCategory::Flags,
    },
    EmojiEntry {
        emoji: "🇧🇷",
        label: "flag: Brazil",
        aliases: &["brazil"],
        category: EmojiCategory::Flags,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct EmojiSelectorState {
    open: bool,
    query: String,
    category: EmojiCategory,
}

impl ComponentUi<'_> {
    pub fn emoji_selector<'a>(
        &mut self,
        value: &mut String,
        props: impl Into<EmojiSelector<'a>>,
    ) -> Response {
        draw_emoji_selector(self, value, props.into())
    }
}

fn draw_emoji_selector(
    ui: &mut ComponentUi<'_>,
    value: &mut String,
    props: EmojiSelector<'_>,
) -> Response {
    let mut state = load_emoji_selector_state(ui.ui(), props.id);
    let was_open = state.open;
    let mut open = state.open;
    let mut value_changed = false;
    let trigger_value = value.clone();
    let trigger_response = ui
        .popover(
            &mut open,
            Popover::new(props.id.with("popover"))
                .width(props.popup_width)
                .align(PopoverAlign::Start)
                .side(PopoverSide::Bottom)
                .side_offset(4.0)
                .padding(12, 12),
            |ui| draw_emoji_trigger(ui, props, trigger_value.as_str(), was_open),
            |ui, open| {
                draw_emoji_panel(
                    ui,
                    props,
                    value,
                    &mut state,
                    open,
                    !was_open,
                    &mut value_changed,
                );
            },
        )
        .trigger;
    state.open = open;

    if was_open && !state.open {
        state.query.clear();
    }

    store_emoji_selector_state(ui.ui_mut(), props.id, state);

    let mut response = trigger_response;
    if value_changed {
        response.mark_changed();
    }
    response
}

fn draw_emoji_trigger(
    ui: &mut Ui,
    props: EmojiSelector<'_>,
    value: &str,
    was_open: bool,
) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(EMOJI_TRIGGER_SIZE, EMOJI_TRIGGER_SIZE),
        Sense::click(),
    );
    let hovered = response.hovered();
    let pressed = response.is_pointer_button_down_on();
    let active = was_open || pressed;
    let (fill, stroke, text_color) =
        trigger_chrome(runtime, props.trigger_variant, active, hovered, pressed);
    control_frame(
        ui,
        rect,
        ControlFrame::new(fill, stroke)
            .corner_radius(tokens::radius_md(runtime))
            .stroke_kind(StrokeKind::Outside),
    );

    let display = if value.is_empty() {
        props.placeholder
    } else {
        value
    };
    paint_emoji_or_text(ui, rect, display, EMOJI_TRIGGER_EMOJI_SIZE, text_color);

    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    ui.components()
        .tooltip_for(&response, Tooltip::text("Choose emoji"));
    response
}

fn draw_emoji_panel(
    ui: &mut Ui,
    props: EmojiSelector<'_>,
    value: &mut String,
    state: &mut EmojiSelectorState,
    open: &mut bool,
    just_opened: bool,
    value_changed: &mut bool,
) {
    ui.set_min_width(props.popup_width);
    ui.set_max_width(props.popup_width);

    let content_width = ui.available_width();
    let mut components = ui.components();
    let input_response = components.text_input(
        &mut state.query,
        TextInput::new()
            .width(content_width)
            .placeholder("Search emoji")
            .leading_icon("search"),
    );
    if just_opened {
        input_response.request_focus();
    }

    ui.add_space(EMOJI_SECTION_GAP);

    let entries = filtered_entries(state.query.as_str(), state.category);
    let _ = egui::ScrollArea::vertical()
        .id_salt(props.id.with("emoji_scroll"))
        .max_height((props.popup_max_height - 104.0).max(96.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let body_width = ui.available_width();
            ui.set_min_width(body_width);
            ui.set_max_width(body_width);

            if entries.is_empty() {
                let _ = muted_empty_state(ui, "No emoji found");
                return;
            }

            let previous_spacing = ui.spacing().item_spacing;
            ui.spacing_mut().item_spacing = egui::vec2(EMOJI_CELL_GAP, EMOJI_CELL_GAP);

            let _ = ui.horizontal_wrapped(|ui| {
                ui.set_min_width(body_width);
                ui.set_max_width(body_width);
                for entry in entries {
                    let selected = value == entry.emoji;
                    let response = draw_emoji_cell(ui, entry, selected);
                    if response.clicked() {
                        value.clear();
                        value.push_str(entry.emoji);
                        state.query.clear();
                        *open = false;
                        *value_changed = true;
                    }
                }
            });

            ui.spacing_mut().item_spacing = previous_spacing;
        });

    ui.add_space(EMOJI_SECTION_GAP);
    draw_panel_separator(ui, content_width);
    ui.add_space(EMOJI_SECTION_GAP);
    draw_category_bar(ui, state, content_width);
}

fn draw_category_bar(ui: &mut Ui, state: &mut EmojiSelectorState, content_width: f32) {
    let button_width = ((content_width
        - CATEGORY_BUTTON_GAP * (CATEGORY_TABS.len().saturating_sub(1) as f32))
        / CATEGORY_TABS.len() as f32)
        .max(1.0);
    let _ = layout::sized_box().width(content_width).show(ui, |ui| {
        let _ = layout::row().gap(CATEGORY_BUTTON_GAP).show(ui, |ui| {
            for tab in CATEGORY_TABS {
                let selected = state.category == tab.category;
                let response = ui.components().button(
                    Button::icon_only(tab.icon)
                        .variant(if selected {
                            ButtonVariant::Secondary
                        } else {
                            ButtonVariant::Ghost
                        })
                        .icon_size(CATEGORY_ICON_SIZE)
                        .min_size(egui::vec2(button_width, CATEGORY_BUTTON_SIZE)),
                );
                if response.clicked() && !selected {
                    state.category = tab.category;
                }
            }
        });
    });
}

fn draw_panel_separator(ui: &mut Ui, width: f32) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), Sense::hover());
    ui.painter().line_segment(
        [
            egui::pos2(rect.left(), rect.center().y),
            egui::pos2(rect.right(), rect.center().y),
        ],
        Stroke::new(1.0, tokens::separator(runtime)),
    );
}

fn draw_emoji_cell(ui: &mut Ui, entry: &EmojiEntry, selected: bool) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(EMOJI_CELL_SIZE, EMOJI_CELL_SIZE), Sense::click());
    let hovered = response.hovered();
    let pressed = response.is_pointer_button_down_on();
    let fill = tokens::row_bg(selected, pressed, hovered, runtime);
    let stroke = if selected {
        Stroke::new(1.0, tokens::button_secondary_hover_border(runtime))
    } else {
        Stroke::NONE
    };
    control_frame(
        ui,
        rect,
        ControlFrame::new(fill, stroke)
            .corner_radius(tokens::radius_sm(runtime))
            .stroke_kind(StrokeKind::Inside),
    );
    paint_emoji_or_text(
        ui,
        rect,
        entry.emoji,
        EMOJI_CELL_EMOJI_SIZE,
        if selected {
            tokens::row_selected_text(runtime)
        } else {
            tokens::text_primary(runtime)
        },
    );

    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    ui.components()
        .tooltip_for(&response, Tooltip::text(entry.label));
    response
}

fn paint_emoji_or_text(
    ui: &mut Ui,
    rect: egui::Rect,
    text: &str,
    emoji_size: f32,
    fallback_color: egui::Color32,
) {
    if let Some(image) = twemoji::image(text, emoji_size) {
        let image_rect =
            egui::Rect::from_center_size(rect.center(), egui::vec2(emoji_size, emoji_size));
        let _ = image.paint_at(ui, image_rect);
    } else {
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            typography::label_font(),
            fallback_color,
        );
    }
}

fn filtered_entries(query: &str, category: EmojiCategory) -> Vec<&'static EmojiEntry> {
    let query = query.trim().to_ascii_lowercase();
    EMOJI_ENTRIES
        .iter()
        .filter(|entry| category == EmojiCategory::All || entry.category == category)
        .filter(|entry| query.is_empty() || emoji_entry_matches(entry, query.as_str()))
        .collect()
}

fn emoji_entry_matches(entry: &EmojiEntry, query: &str) -> bool {
    if entry.emoji == query {
        return true;
    }

    if entry.label.to_ascii_lowercase().contains(query) {
        return true;
    }

    entry
        .aliases
        .iter()
        .any(|alias| alias.to_ascii_lowercase().contains(query))
}

fn trigger_chrome(
    runtime: crate::theme::ThemeRuntime,
    variant: ButtonVariant,
    active: bool,
    hovered: bool,
    pressed: bool,
) -> (egui::Color32, Stroke, egui::Color32) {
    match variant {
        ButtonVariant::Primary => (
            if pressed {
                tokens::primary_active_bg(runtime)
            } else if active || hovered {
                tokens::primary_hover_bg(runtime)
            } else {
                tokens::primary_bg(runtime)
            },
            Stroke::NONE,
            tokens::primary_fg(runtime),
        ),
        ButtonVariant::Secondary => (
            if pressed {
                tokens::button_secondary_active_bg(runtime)
            } else if active || hovered {
                tokens::button_secondary_hover_bg(runtime)
            } else {
                tokens::button_secondary_bg(runtime)
            },
            Stroke::new(
                1.0,
                if pressed {
                    tokens::button_secondary_active_border(runtime)
                } else if active || hovered {
                    tokens::button_secondary_hover_border(runtime)
                } else {
                    tokens::button_secondary_border(runtime)
                },
            ),
            tokens::text_primary(runtime),
        ),
        ButtonVariant::Ghost | ButtonVariant::Link => (
            if pressed {
                tokens::button_secondary_active_bg(runtime)
            } else if active || hovered {
                tokens::button_secondary_hover_bg(runtime)
            } else {
                tokens::TRANSPARENT
            },
            Stroke::NONE,
            tokens::text_primary(runtime),
        ),
    }
}

fn load_emoji_selector_state(ui: &Ui, id: Id) -> EmojiSelectorState {
    ui.data(|data| {
        data.get_temp::<EmojiSelectorState>(id.with("emoji_selector_state"))
            .unwrap_or_default()
    })
}

fn store_emoji_selector_state(ui: &mut Ui, id: Id, state: EmojiSelectorState) {
    ui.data_mut(|data| {
        if state == EmojiSelectorState::default() {
            data.remove::<EmojiSelectorState>(id.with("emoji_selector_state"));
        } else {
            data.insert_temp(id.with("emoji_selector_state"), state);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{filtered_entries, EmojiCategory, EmojiSelector, EmojiSelectorState};
    use crate::components::ComponentUiExt;
    use crate::theme::{self, ThemeMode};
    use egui::{CentralPanel, Context, Event, Id, Modifiers, PointerButton, RawInput};

    #[test]
    fn query_matches_labels_aliases_and_direct_emoji() {
        let people = filtered_entries("laugh", EmojiCategory::People);
        assert!(people.iter().any(|entry| entry.emoji == "😂"));

        let symbols = filtered_entries("notification", EmojiCategory::Symbols);
        assert!(symbols.iter().any(|entry| entry.emoji == "🔔"));

        let direct = filtered_entries("🍕", EmojiCategory::Food);
        assert!(direct.iter().any(|entry| entry.emoji == "🍕"));
    }

    #[test]
    fn category_filter_limits_results() {
        let people = filtered_entries("", EmojiCategory::People);
        assert!(people
            .iter()
            .all(|entry| entry.category == EmojiCategory::People));
        assert!(people.iter().any(|entry| entry.emoji == "🙂"));
        assert!(!people.iter().any(|entry| entry.emoji == "🍕"));
    }

    #[test]
    fn trigger_click_opens_internal_popover_state() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let id = Id::new("emoji_selector_trigger_test");
        let mut value = String::new();
        let mut trigger_center = egui::Pos2::ZERO;

        let _ = context.run(RawInput::default(), |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                let response = ui
                    .components()
                    .emoji_selector(&mut value, EmojiSelector::new(id));
                trigger_center = response.rect.center();
            });
        });

        let _ = context.run(pointer_input(trigger_center, true), |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                let _ = ui
                    .components()
                    .emoji_selector(&mut value, EmojiSelector::new(id));
            });
        });
        let _ = context.run(pointer_input(trigger_center, false), |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                let _ = ui
                    .components()
                    .emoji_selector(&mut value, EmojiSelector::new(id));
            });
        });

        let state = context.data(|data| {
            data.get_temp::<EmojiSelectorState>(id.with("emoji_selector_state"))
                .unwrap_or_default()
        });
        assert!(state.open);
    }

    #[test]
    fn render_without_supported_twemoji_still_allocates_space() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let mut value = "not-in-picker".to_owned();
        let mut width = 0.0;

        let _ = context.run(RawInput::default(), |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                width = ui
                    .components()
                    .emoji_selector(
                        &mut value,
                        EmojiSelector::new(Id::new("emoji_selector_fallback_test")),
                    )
                    .rect
                    .width();
            });
        });

        assert!(width > 0.0);
    }

    fn pointer_input(position: egui::Pos2, pressed: bool) -> RawInput {
        RawInput {
            events: vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..RawInput::default()
        }
    }
}
