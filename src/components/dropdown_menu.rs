use super::{api::ComponentUi, Button, ButtonStyle, Kbd, KbdGroup};
use egui::{containers::menu::SubMenuButton, Align, CursorIcon, Layout, Rect, Response, Ui};

const MENU_MIN_WIDTH: f32 = 160.0;
const MENU_ROW_HEIGHT: f32 = 28.0;

#[derive(Debug, Clone, Copy)]
pub struct DropdownMenuAction<'a> {
    pub id: usize,
    pub label: &'a str,
    pub shortcut: Option<&'a str>,
}

impl<'a> DropdownMenuAction<'a> {
    pub const fn new(id: usize, label: &'a str) -> Self {
        Self {
            id,
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
pub struct DropdownMenuSubmenu<'a> {
    pub label: &'a str,
    pub entries: &'a [DropdownMenuEntry<'a>],
}

impl<'a> DropdownMenuSubmenu<'a> {
    pub const fn new(label: &'a str, entries: &'a [DropdownMenuEntry<'a>]) -> Self {
        Self { label, entries }
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

    pub const fn separator() -> Self {
        Self::Separator
    }

    pub const fn submenu(label: &'a str, entries: &'a [DropdownMenuEntry<'a>]) -> Self {
        Self::Submenu(DropdownMenuSubmenu::new(label, entries))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DropdownMenu<'a> {
    pub trigger_label: &'a str,
    pub options: &'a [&'a str],
    pub entries: &'a [DropdownMenuEntry<'a>],
    pub width: f32,
    pub trigger_style: ButtonStyle,
}

impl<'a> DropdownMenu<'a> {
    pub fn new(trigger_label: &'a str, options: &'a [&'a str]) -> Self {
        Self {
            trigger_label,
            options,
            entries: &[],
            width: 220.0,
            trigger_style: ButtonStyle::Secondary,
        }
    }

    pub fn with_entries(trigger_label: &'a str, entries: &'a [DropdownMenuEntry<'a>]) -> Self {
        Self {
            trigger_label,
            options: &[],
            entries,
            width: 220.0,
            trigger_style: ButtonStyle::Secondary,
        }
    }

    pub fn entries(mut self, entries: &'a [DropdownMenuEntry<'a>]) -> Self {
        self.entries = entries;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(1.0);
        self
    }

    pub fn trigger_style(mut self, trigger_style: ButtonStyle) -> Self {
        self.trigger_style = trigger_style;
        self
    }
}

impl<'a> From<(&'a str, &'a [&'a str])> for DropdownMenu<'a> {
    fn from((trigger_label, options): (&'a str, &'a [&'a str])) -> Self {
        Self::new(trigger_label, options)
    }
}

impl<'a> From<&'a str> for DropdownMenu<'a> {
    fn from(trigger_label: &'a str) -> Self {
        Self::new(trigger_label, &[])
    }
}

impl<'a> From<(&'a str, &'a [DropdownMenuEntry<'a>])> for DropdownMenu<'a> {
    fn from((trigger_label, entries): (&'a str, &'a [DropdownMenuEntry<'a>])) -> Self {
        Self::with_entries(trigger_label, entries)
    }
}

impl<'a> From<(&'a str, &'a [&'a str], f32)> for DropdownMenu<'a> {
    fn from((trigger_label, options, width): (&'a str, &'a [&'a str], f32)) -> Self {
        Self::new(trigger_label, options).width(width)
    }
}

impl<'a> From<(&'a str, &'a [DropdownMenuEntry<'a>], f32)> for DropdownMenu<'a> {
    fn from((trigger_label, entries, width): (&'a str, &'a [DropdownMenuEntry<'a>], f32)) -> Self {
        Self::with_entries(trigger_label, entries).width(width)
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
                .button((props.trigger_label, props.trigger_style))
                .on_hover_cursor(CursorIcon::PointingHand);

            return (response, state);
        }

        let row_width = props.width.max(MENU_MIN_WIDTH);
        let response = self
            .button((props.trigger_label, props.trigger_style))
            .on_hover_cursor(CursorIcon::PointingHand);

        let _ = egui::Popup::menu(&response).show(|ui| {
            let mut ui = ComponentUi::new(ui);
            ui.style_mut().spacing.item_spacing.y = 1.0;
            ui.set_min_width(row_width);
            ui.set_max_width(row_width);

            if props.entries.is_empty() {
                for (index, label) in props.options.iter().copied().enumerate() {
                    let action = DropdownMenuAction::new(index, label);
                    draw_action_row(&mut ui, action, &mut state, row_width);
                }
            } else {
                draw_entries(&mut ui, props.entries, &mut state, row_width);
            }
        });

        (response, state)
    }
}

fn draw_entries(
    ui: &mut ComponentUi<'_>,
    entries: &[DropdownMenuEntry<'_>],
    state: &mut DropdownMenuState,
    row_width: f32,
) {
    for entry in entries {
        match entry {
            DropdownMenuEntry::Action(action) => {
                draw_action_row(ui, *action, state, row_width);
            }
            DropdownMenuEntry::Separator => {
                ui.add_space(2.0);
                let _ = ui.separator();
                ui.add_space(2.0);
            }
            DropdownMenuEntry::Submenu(submenu) => {
                let submenu_button = ui
                    .button(
                        Button::name(submenu.label)
                            .style(ButtonStyle::Ghost)
                            .right_text(SubMenuButton::RIGHT_ARROW)
                            .min_size(egui::vec2(row_width, MENU_ROW_HEIGHT)),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand);
                let _ = egui::containers::menu::SubMenu::new().show(
                    ui.raw_mut(),
                    &submenu_button,
                    |ui| {
                        let mut ui = ComponentUi::new(ui);
                        ui.style_mut().spacing.item_spacing.y = 1.0;
                        ui.set_min_width(row_width);
                        ui.set_max_width(row_width);
                        draw_entries(&mut ui, submenu.entries, state, row_width);
                    },
                );
            }
        }
    }
}

fn draw_action_row(
    ui: &mut ComponentUi<'_>,
    action: DropdownMenuAction<'_>,
    state: &mut DropdownMenuState,
    row_width: f32,
) {
    let props = Button::name(action.label)
        .style(ButtonStyle::Ghost)
        .right_text("")
        .min_size(egui::vec2(row_width, MENU_ROW_HEIGHT));

    let response = ui.button(props).on_hover_cursor(CursorIcon::PointingHand);
    if let Some(shortcut) = action.shortcut {
        draw_shortcut_keycaps(ui.raw_mut(), response.rect, shortcut);
    }
    if response.clicked() {
        state.action = Some(action.id);
        ui.close();
    }
}

fn draw_shortcut_keycaps(ui: &mut Ui, row_rect: Rect, shortcut: &str) {
    let keys = parse_shortcut_keys(shortcut);
    if keys.is_empty() {
        return;
    }

    let _ = ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(row_rect.shrink2(egui::vec2(8.0, 4.0)))
            .layout(Layout::right_to_left(Align::Center)),
        |ui| {
            let mut ui = ComponentUi::new(ui);
            let _ = ui.kbd_group(KbdGroup::new().gap(3.0), |ui| {
                for key in keys {
                    let _ = ui.kbd(Kbd::new(key).height(18.0).text_size(9.5));
                }
            });
        },
    );
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

#[cfg(test)]
mod tests {
    use super::parse_shortcut_keys;

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
}
