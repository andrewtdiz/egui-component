use super::{api::ComponentUi, Button, ButtonVariant};
use crate::primitives::{
    popup::{popup_panel, PopupPanel},
    row::{icon_label_row, row_chrome, IconLabelRow, RowChrome},
};
use crate::ui::{icons, tokens, typography};
use egui::{
    Align2, CornerRadius, CursorIcon, FontFamily, FontId, Margin, Popup, Rect, Response, Stroke,
    StrokeKind, Ui,
};

const MENU_MIN_WIDTH: f32 = 176.0;
const MENU_INNER_PADDING_X: i8 = 0;
const MENU_INNER_PADDING_Y: i8 = 0;
const MENU_ROW_HEIGHT: f32 = 40.0;
const MENU_ROW_PADDING_X: f32 = 10.0;
const MENU_LEADING_ICON_SIZE: f32 = 16.0;
const MENU_LEADING_ICON_GAP: f32 = 8.0;
const MENU_TRAILING_ICON_SIZE: f32 = 15.0;
const MENU_TRAILING_GAP: f32 = 6.0;
const MENU_SEPARATOR_MARGIN_Y: f32 = 4.0;
const MENU_TEXT_SIZE: f32 = 15.0;
const MENU_SHORTCUT_GAP: f32 = 2.0;
const MENU_KEYCAP_HEIGHT: f32 = 15.0;
const MENU_KEYCAP_MIN_WIDTH: f32 = 17.0;
const MENU_KEYCAP_TEXT_SIZE: f32 = 9.0;

#[derive(Debug, Clone, Copy)]
pub struct DropdownMenuAction<'a> {
    pub id: usize,
    pub label: &'a str,
    pub icon: Option<&'a str>,
    pub shortcut: Option<&'a str>,
}

impl<'a> DropdownMenuAction<'a> {
    pub const fn new(id: usize, label: &'a str) -> Self {
        Self {
            id,
            label,
            icon: None,
            shortcut: None,
        }
    }

    pub const fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub const fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DropdownMenuSubmenu<'a> {
    pub label: &'a str,
    pub icon: Option<&'a str>,
    pub entries: &'a [DropdownMenuEntry<'a>],
}

impl<'a> DropdownMenuSubmenu<'a> {
    pub const fn new(label: &'a str, entries: &'a [DropdownMenuEntry<'a>]) -> Self {
        Self {
            label,
            icon: None,
            entries,
        }
    }

    pub const fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DropdownMenuEntry<'a> {
    Action(DropdownMenuAction<'a>),
    Separator,
    Submenu(DropdownMenuSubmenu<'a>),
}

impl<'a> DropdownMenuEntry<'a> {
    pub const fn action(id: usize, label: &'a str) -> Self {
        Self::Action(DropdownMenuAction::new(id, label))
    }

    pub const fn action_with_shortcut(id: usize, label: &'a str, shortcut: &'a str) -> Self {
        Self::Action(DropdownMenuAction::new(id, label).shortcut(shortcut))
    }

    pub const fn action_with_icon(id: usize, label: &'a str, icon: &'a str) -> Self {
        Self::Action(DropdownMenuAction::new(id, label).icon(icon))
    }

    pub const fn action_with_icon_and_shortcut(
        id: usize,
        label: &'a str,
        icon: &'a str,
        shortcut: &'a str,
    ) -> Self {
        Self::Action(
            DropdownMenuAction::new(id, label)
                .icon(icon)
                .shortcut(shortcut),
        )
    }

    pub const fn separator() -> Self {
        Self::Separator
    }

    pub const fn submenu(label: &'a str, entries: &'a [DropdownMenuEntry<'a>]) -> Self {
        Self::Submenu(DropdownMenuSubmenu::new(label, entries))
    }

    pub const fn submenu_with_icon(
        label: &'a str,
        icon: &'a str,
        entries: &'a [DropdownMenuEntry<'a>],
    ) -> Self {
        Self::Submenu(DropdownMenuSubmenu::new(label, entries).icon(icon))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DropdownMenu<'a> {
    pub trigger_label: &'a str,
    pub options: &'a [&'a str],
    pub entries: &'a [DropdownMenuEntry<'a>],
    pub width: f32,
    pub trigger_variant: ButtonVariant,
}

impl<'a> DropdownMenu<'a> {
    pub fn new(trigger_label: &'a str) -> Self {
        Self {
            trigger_label,
            options: &[],
            entries: &[],
            width: 220.0,
            trigger_variant: ButtonVariant::Secondary,
        }
    }

    pub fn options(mut self, options: &'a [&'a str]) -> Self {
        self.options = options;
        self
    }

    pub fn entries(mut self, entries: &'a [DropdownMenuEntry<'a>]) -> Self {
        self.entries = entries;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(1.0);
        self
    }

    pub fn trigger_variant(mut self, trigger_variant: ButtonVariant) -> Self {
        self.trigger_variant = trigger_variant;
        self
    }
}

impl<'a> From<&'a str> for DropdownMenu<'a> {
    fn from(trigger_label: &'a str) -> Self {
        Self::new(trigger_label)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DropdownMenuState {
    pub action: Option<usize>,
}

impl ComponentUi<'_> {
    pub fn dropdown_menu<'a>(
        &mut self,
        props: impl Into<DropdownMenu<'a>>,
    ) -> (Response, DropdownMenuState) {
        let props = props.into();
        let mut state = DropdownMenuState { action: None };

        if props.options.is_empty() && props.entries.is_empty() {
            let response = self
                .button(Button::new(props.trigger_label).variant(props.trigger_variant))
                .on_hover_cursor(CursorIcon::PointingHand);

            return (response, state);
        }

        let row_width = props.width.max(MENU_MIN_WIDTH);
        let response = self
            .button(Button::new(props.trigger_label).variant(props.trigger_variant))
            .on_hover_cursor(CursorIcon::PointingHand);

        let _ = self.raw_mut().scope(|ui| {
            ui.style_mut().spacing.menu_margin = Margin::symmetric(0, 2);
            Popup::menu(&response).show(|ui| {
                if props.entries.is_empty() {
                    let entries = props
                        .options
                        .iter()
                        .copied()
                        .enumerate()
                        .map(|(index, label)| {
                            DropdownMenuEntry::Action(DropdownMenuAction::new(index, label))
                        })
                        .collect::<Vec<_>>();
                    show_menu_entries_surface(ui, &entries, &mut state.action, row_width);
                } else {
                    show_menu_entries_surface(ui, props.entries, &mut state.action, row_width);
                }
            })
        });

        (response, state)
    }
}

pub(crate) fn show_menu_entries_surface(
    ui: &mut Ui,
    entries: &[DropdownMenuEntry<'_>],
    action: &mut Option<usize>,
    row_width: f32,
) {
    let mut ui = ComponentUi::new(ui);
    ui.style_mut().spacing.item_spacing.y = 0.0;
    ui.set_min_width(row_width);
    ui.set_max_width(row_width);

    popup_panel(
        ui.raw_mut(),
        PopupPanel::new(row_width).padding(MENU_INNER_PADDING_X, MENU_INNER_PADDING_Y),
        |ui| {
            let mut ui = ComponentUi::new(ui);
            let inner_width = inner_row_width(row_width);
            ui.set_min_width(inner_width);
            ui.set_max_width(inner_width);
            draw_entries(&mut ui, entries, action, row_width);
        },
    );
}

fn draw_entries(
    ui: &mut ComponentUi<'_>,
    entries: &[DropdownMenuEntry<'_>],
    selected_action: &mut Option<usize>,
    row_width: f32,
) {
    for entry in entries {
        match entry {
            DropdownMenuEntry::Action(menu_action) => {
                draw_action_row(ui, *menu_action, selected_action, row_width);
            }
            DropdownMenuEntry::Separator => {
                ui.add_space(MENU_SEPARATOR_MARGIN_Y);
                let _ = ui.separator();
                ui.add_space(MENU_SEPARATOR_MARGIN_Y);
            }
            DropdownMenuEntry::Submenu(submenu) => {
                let (submenu_button, _) = draw_menu_row(
                    ui.raw_mut(),
                    submenu.label,
                    submenu.icon,
                    row_width,
                    None,
                    true,
                );
                let _ = egui::containers::menu::SubMenu::new().show(
                    ui.raw_mut(),
                    &submenu_button,
                    |ui| {
                        let mut ui = ComponentUi::new(ui);
                        ui.style_mut().spacing.item_spacing.y = 0.0;
                        popup_panel(
                            ui.raw_mut(),
                            PopupPanel::new(row_width)
                                .padding(MENU_INNER_PADDING_X, MENU_INNER_PADDING_Y),
                            |ui| {
                                let mut ui = ComponentUi::new(ui);
                                let inner_width = inner_row_width(row_width);
                                ui.set_min_width(inner_width);
                                ui.set_max_width(inner_width);
                                draw_entries(&mut ui, submenu.entries, selected_action, row_width);
                            },
                        );
                    },
                );
            }
        }
    }
}

fn draw_action_row(
    ui: &mut ComponentUi<'_>,
    action: DropdownMenuAction<'_>,
    selected_action: &mut Option<usize>,
    row_width: f32,
) {
    let (response, trailing_rect) = draw_menu_row(
        ui.raw_mut(),
        action.label,
        action.icon,
        row_width,
        action.shortcut,
        false,
    );
    if let (Some(shortcut), Some(trailing_rect)) = (action.shortcut, trailing_rect) {
        draw_shortcut_keycaps(ui.raw_mut(), trailing_rect, shortcut);
    }
    if response.clicked() {
        *selected_action = Some(action.id);
        ui.close();
    }
}

fn draw_menu_row(
    ui: &mut Ui,
    label: &str,
    icon: Option<&str>,
    menu_width: f32,
    shortcut: Option<&str>,
    submenu: bool,
) -> (Response, Option<Rect>) {
    let dark_mode = ui.visuals().dark_mode;
    let desired_size = egui::vec2(inner_row_width(menu_width), MENU_ROW_HEIGHT);
    let (rect, response) = row_chrome(
        ui,
        RowChrome::new(desired_size).stroke(egui::Stroke::NONE),
        |response| {
            tokens::row_bg(
                false,
                response.is_pointer_button_down_on(),
                response.hovered(),
                dark_mode,
            )
        },
    );

    let label_color = if response.hovered() || response.is_pointer_button_down_on() {
        tokens::text_primary(dark_mode)
    } else {
        tokens::text_secondary(dark_mode)
    };
    let trailing_width = if submenu {
        MENU_TRAILING_ICON_SIZE
    } else {
        shortcut
            .map(|shortcut| shortcut_group_width(ui, shortcut))
            .unwrap_or(0.0)
    };
    let trailing_rect = if trailing_width > 0.0 {
        Some(Rect::from_min_max(
            egui::pos2(
                rect.right() - MENU_ROW_PADDING_X - trailing_width,
                rect.top(),
            ),
            egui::pos2(rect.right() - MENU_ROW_PADDING_X, rect.bottom()),
        ))
    } else {
        None
    };
    let label_right = trailing_rect
        .map(|rect| rect.left() - MENU_TRAILING_GAP)
        .unwrap_or(rect.right() - MENU_ROW_PADDING_X);
    let row = {
        let row = IconLabelRow::new(label, typography::proportional(MENU_TEXT_SIZE), label_color)
            .padding_x(MENU_ROW_PADDING_X)
            .leading_icon_size(MENU_LEADING_ICON_SIZE)
            .leading_gap(MENU_LEADING_ICON_GAP);
        if let Some(icon) = icon {
            row.leading_icon(icon)
        } else {
            row
        }
    };
    let _ = label_right;
    let _ = icon_label_row(ui, rect, &row);

    if submenu {
        if let Some(trailing_rect) = trailing_rect {
            draw_submenu_indicator(ui, trailing_rect);
        }
    }

    (
        response.on_hover_cursor(CursorIcon::PointingHand),
        trailing_rect,
    )
}

fn draw_shortcut_keycaps(ui: &mut Ui, trailing_rect: Rect, shortcut: &str) {
    let keycaps = measure_shortcut_keycaps(ui, shortcut);
    if keycaps.is_empty() {
        return;
    }

    let dark_mode = ui.visuals().dark_mode;
    let text_color = tokens::text_muted(dark_mode);
    let font_id = FontId::new(MENU_KEYCAP_TEXT_SIZE, FontFamily::Monospace);
    let total_width = shortcut_group_width_from_keycaps(&keycaps);
    let mut left = trailing_rect.right() - total_width;
    let top = trailing_rect.center().y - (MENU_KEYCAP_HEIGHT * 0.5);

    // Paint keycaps directly so shortcut rows never participate in menu layout.
    for (index, keycap) in keycaps.iter().enumerate() {
        let key_rect = Rect::from_min_size(
            egui::pos2(left, top),
            egui::vec2(keycap.width, MENU_KEYCAP_HEIGHT),
        );
        ui.painter().rect(
            key_rect,
            CornerRadius::same(tokens::RADIUS_SM),
            tokens::input_background(dark_mode),
            Stroke::new(1.0, tokens::input_border(dark_mode)),
            StrokeKind::Outside,
        );
        ui.painter().text(
            key_rect.center(),
            Align2::CENTER_CENTER,
            keycap.label,
            font_id.clone(),
            text_color,
        );

        left += keycap.width;
        if index + 1 < keycaps.len() {
            left += MENU_SHORTCUT_GAP;
        }
    }
}
fn draw_submenu_indicator(ui: &mut Ui, trailing_rect: Rect) {
    let dark_mode = ui.visuals().dark_mode;
    if let Some(image) = icons::image(ui.ctx(), "chevron-right", MENU_TRAILING_ICON_SIZE) {
        let icon_rect = Rect::from_center_size(
            trailing_rect.center(),
            egui::vec2(MENU_TRAILING_ICON_SIZE, MENU_TRAILING_ICON_SIZE),
        );
        // Paint directly so the submenu indicator never participates in layout after the row
        // itself has already been allocated.
        image
            .tint(tokens::text_muted(dark_mode))
            .paint_at(ui, icon_rect);
    }
}

fn shortcut_group_width(ui: &mut Ui, shortcut: &str) -> f32 {
    let keycaps = measure_shortcut_keycaps(ui, shortcut);
    shortcut_group_width_from_keycaps(&keycaps)
}

fn shortcut_group_width_from_keycaps(keycaps: &[ShortcutKeycap<'_>]) -> f32 {
    if keycaps.is_empty() {
        return 0.0;
    }

    keycaps
        .iter()
        .enumerate()
        .map(|(index, keycap)| {
            if index == 0 {
                keycap.width
            } else {
                MENU_SHORTCUT_GAP + keycap.width
            }
        })
        .sum()
}

fn measure_shortcut_keycaps<'a>(ui: &mut Ui, shortcut: &'a str) -> Vec<ShortcutKeycap<'a>> {
    let keys = parse_shortcut_keys(shortcut);
    if keys.is_empty() {
        return Vec::new();
    }

    let text_color = tokens::text_muted(ui.visuals().dark_mode);
    let font_id = FontId::new(MENU_KEYCAP_TEXT_SIZE, FontFamily::Monospace);
    keys.into_iter()
        .map(|key| {
            let width = ui.fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(key.to_owned(), font_id.clone(), text_color)
                    .size()
                    .x
            });
            ShortcutKeycap {
                label: key,
                width: (width + 8.0).max(MENU_KEYCAP_MIN_WIDTH),
            }
        })
        .collect()
}

fn inner_row_width(menu_width: f32) -> f32 {
    (menu_width - f32::from(MENU_INNER_PADDING_X * 2)).max(MENU_MIN_WIDTH - 8.0)
}

fn parse_shortcut_keys(shortcut: &str) -> Vec<&str> {
    shortcut
        .split('+')
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(normalize_shortcut_key)
        .collect()
}

fn normalize_shortcut_key(key: &str) -> &str {
    if key.eq_ignore_ascii_case("cmd") || key.eq_ignore_ascii_case("command") {
        "⌘"
    } else if key.eq_ignore_ascii_case("ctrl") || key.eq_ignore_ascii_case("control") {
        "⌃"
    } else if key.eq_ignore_ascii_case("alt") || key.eq_ignore_ascii_case("option") {
        "⌥"
    } else if key.eq_ignore_ascii_case("shift") {
        "⇧"
    } else {
        key
    }
}

#[derive(Debug, Clone, Copy)]
struct ShortcutKeycap<'a> {
    label: &'a str,
    width: f32,
}

#[cfg(test)]
mod tests {
    use super::{
        draw_shortcut_keycaps, draw_submenu_indicator, parse_shortcut_keys, MENU_ROW_HEIGHT,
    };
    use egui::{pos2, vec2, CentralPanel, Context, RawInput, Rect};

    #[test]
    fn parses_and_normalizes_shortcut_segments() {
        assert_eq!(parse_shortcut_keys("Shift+Cmd+P"), vec!["⇧", "⌘", "P"]);
        assert_eq!(parse_shortcut_keys(" Ctrl + Alt + B "), vec!["⌃", "⌥", "B"]);
    }

    #[test]
    fn drops_empty_shortcut_segments() {
        assert_eq!(parse_shortcut_keys("++Cmd++K++"), vec!["⌘", "K"]);
        assert!(parse_shortcut_keys(" +  + ").is_empty());
    }

    #[test]
    fn drawing_shortcut_keycaps_does_not_advance_layout() {
        let context = Context::default();
        let mut before = Rect::NOTHING;
        let mut after = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                before = ui.min_rect();
                draw_shortcut_keycaps(
                    ui,
                    Rect::from_min_size(pos2(24.0, 24.0), vec2(72.0, MENU_ROW_HEIGHT)),
                    "Shift+Cmd+Q",
                );
                after = ui.min_rect();
            });
        });

        assert_eq!(before, after);
    }

    #[test]
    fn drawing_submenu_indicator_does_not_advance_layout() {
        let context = Context::default();
        let mut before = Rect::NOTHING;
        let mut after = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                crate::icons::setup(context);
                before = ui.min_rect();
                draw_submenu_indicator(
                    ui,
                    Rect::from_min_size(pos2(24.0, 24.0), vec2(16.0, MENU_ROW_HEIGHT)),
                );
                after = ui.min_rect();
            });
        });

        assert_eq!(before, after);
    }
}
