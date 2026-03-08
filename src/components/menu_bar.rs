use super::{
    api::ComponentUi,
    dropdown_menu::{show_menu_entries_surface, DropdownMenuEntry},
};
use crate::ui::{tokens, typography};
use egui::{
    Align2, Color32, CornerRadius, CursorIcon, FontId, Id, Margin, Popup, Response, Stroke,
    StrokeKind, Ui,
};

const MENU_BAR_ITEM_HEIGHT: f32 = 28.0;
const MENU_BAR_ITEM_PADDING_X: f32 = 10.0;
const MENU_BAR_TEXT_SIZE: f32 = 13.0;
const MENU_BAR_MIN_MENU_WIDTH: f32 = 176.0;

#[derive(Debug, Clone, Copy)]
pub struct MenuBarItem<'a> {
    pub label: &'a str,
    pub entries: &'a [DropdownMenuEntry<'a>],
    pub width: f32,
}

impl<'a> MenuBarItem<'a> {
    pub const fn new(label: &'a str, entries: &'a [DropdownMenuEntry<'a>]) -> Self {
        Self {
            label,
            entries,
            width: 208.0,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(1.0);
        self
    }

    fn can_open(self) -> bool {
        !self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MenuBar<'a> {
    pub id: Id,
    pub items: &'a [MenuBarItem<'a>],
    pub fill: Option<Color32>,
    pub stroke: Option<Stroke>,
    pub corner_radius: u8,
    pub padding_x: i8,
    pub padding_y: i8,
    pub shadow: Option<egui::Shadow>,
}

impl<'a> MenuBar<'a> {
    pub fn new(id: Id, items: &'a [MenuBarItem<'a>]) -> Self {
        Self {
            id,
            items,
            fill: None,
            stroke: None,
            corner_radius: tokens::RADIUS_MD,
            padding_x: 4,
            padding_y: 3,
            shadow: None,
        }
    }

    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn corner_radius(mut self, corner_radius: u8) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn padding(mut self, x: i8, y: i8) -> Self {
        self.padding_x = x;
        self.padding_y = y;
        self
    }

    pub fn shadow(mut self, shadow: egui::Shadow) -> Self {
        self.shadow = Some(shadow);
        self
    }
}

impl<'a> From<(Id, &'a [MenuBarItem<'a>])> for MenuBar<'a> {
    fn from((id, items): (Id, &'a [MenuBarItem<'a>])) -> Self {
        Self::new(id, items)
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct MenuBarState {
    pub action: Option<usize>,
    pub active_menu: Option<usize>,
}

#[derive(Debug, Clone, Copy, Default)]
struct MenuBarMemory {
    active_menu: Option<usize>,
}

impl ComponentUi<'_> {
    pub fn menu_bar<'a>(&mut self, props: impl Into<MenuBar<'a>>) -> (Response, MenuBarState) {
        draw_menu_bar(self.raw_mut(), props.into())
    }
}

fn draw_menu_bar(ui: &mut Ui, props: MenuBar<'_>) -> (Response, MenuBarState) {
    let dark_mode = ui.visuals().dark_mode;
    let mut active_menu = load_menu_bar_memory(ui, props.id)
        .active_menu
        .filter(|index| *index < props.items.len());
    let openable = props
        .items
        .iter()
        .copied()
        .map(MenuBarItem::can_open)
        .collect::<Vec<_>>();
    let mut clicked_trigger = None;
    let mut trigger_rects = Vec::with_capacity(props.items.len());
    let mut trigger_responses = Vec::with_capacity(props.items.len());

    let bar_response = egui::Frame::new()
        .fill(props.fill.unwrap_or(tokens::card_background(dark_mode)))
        .stroke(
            props
                .stroke
                .unwrap_or(Stroke::new(1.0, tokens::separator(dark_mode))),
        )
        .corner_radius(CornerRadius::same(props.corner_radius))
        .inner_margin(Margin::symmetric(props.padding_x, props.padding_y))
        .shadow(props.shadow.unwrap_or(egui::Shadow::NONE))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            ui.spacing_mut().item_spacing.y = 0.0;

            let _ = ui.horizontal(|ui| {
                for (index, item) in props.items.iter().copied().enumerate() {
                    let response = draw_menu_bar_trigger(
                        ui,
                        item.label,
                        active_menu == Some(index) && item.can_open(),
                    );
                    if response.clicked() {
                        clicked_trigger = Some(index);
                    }
                    trigger_rects.push(response.rect);
                    trigger_responses.push(response);
                }
            });
        })
        .response;

    if let Some(index) = clicked_trigger {
        active_menu = resolve_active_menu(active_menu, Some(index), None, &openable);
    }

    if active_menu.is_some() {
        let hovered_trigger = menu_index_at_pointer(
            ui.ctx().pointer_hover_pos(),
            &trigger_rects,
            bar_response.rect,
        );
        active_menu = resolve_active_menu(active_menu, None, hovered_trigger, &openable);
    }

    let mut state = MenuBarState {
        action: None,
        active_menu,
    };

    if let Some(active_index) = active_menu {
        let mut popup_open = true;
        let popup_response = ui
            .scope(|ui| {
                ui.style_mut().spacing.menu_margin = Margin::symmetric(0, 2);
                Popup::menu(&trigger_responses[active_index])
                    .id(props.id.with("popup"))
                    .open_bool(&mut popup_open)
                    .gap(0.0)
                    .show(|ui| {
                        show_menu_entries_surface(
                            ui,
                            props.items[active_index].entries,
                            &mut state.action,
                            props.items[active_index].width.max(MENU_BAR_MIN_MENU_WIDTH),
                        );
                    })
            })
            .inner;

        if popup_response.is_none() || !popup_open || state.action.is_some() {
            active_menu = None;
        }
    }

    if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        active_menu = None;
    }

    state.active_menu = active_menu;
    store_menu_bar_memory(ui, props.id, MenuBarMemory { active_menu });

    (bar_response, state)
}

fn draw_menu_bar_trigger(ui: &mut Ui, label: &str, active: bool) -> Response {
    let dark_mode = ui.visuals().dark_mode;
    let font_id: FontId = typography::proportional(MENU_BAR_TEXT_SIZE);
    let text_width = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(
                label.to_owned(),
                font_id.clone(),
                tokens::text_secondary(dark_mode),
            )
            .size()
            .x
    });
    let desired_size = egui::vec2(
        (text_width + MENU_BAR_ITEM_PADDING_X * 2.0).ceil(),
        MENU_BAR_ITEM_HEIGHT,
    );
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let fill = if response.is_pointer_button_down_on() {
        tokens::row_active_bg(dark_mode)
    } else if active || response.hovered() {
        tokens::row_selected_bg(dark_mode)
    } else {
        tokens::TRANSPARENT
    };

    ui.painter().rect(
        rect,
        CornerRadius::same(tokens::RADIUS_SM),
        fill,
        Stroke::NONE,
        StrokeKind::Outside,
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        font_id,
        if active || response.hovered() {
            tokens::text_primary(dark_mode)
        } else {
            tokens::text_secondary(dark_mode)
        },
    );

    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn resolve_active_menu(
    current: Option<usize>,
    clicked_trigger: Option<usize>,
    hovered_trigger: Option<usize>,
    openable: &[bool],
) -> Option<usize> {
    if let Some(clicked) =
        clicked_trigger.filter(|index| openable.get(*index).copied().unwrap_or(false))
    {
        return if current == Some(clicked) {
            None
        } else {
            Some(clicked)
        };
    }

    if current.is_some() {
        if let Some(hovered) =
            hovered_trigger.filter(|index| openable.get(*index).copied().unwrap_or(false))
        {
            return Some(hovered);
        }
    }

    current
}

fn menu_index_at_pointer(
    pointer_pos: Option<egui::Pos2>,
    trigger_rects: &[egui::Rect],
    bar_rect: egui::Rect,
) -> Option<usize> {
    let pointer_pos = pointer_pos?;
    if trigger_rects.is_empty()
        || pointer_pos.y < bar_rect.top()
        || pointer_pos.y > bar_rect.bottom()
    {
        return None;
    }

    for (index, rect) in trigger_rects.iter().enumerate() {
        let left = if index == 0 {
            rect.left()
        } else {
            (trigger_rects[index - 1].center().x + rect.center().x) * 0.5
        };
        let right = if index + 1 == trigger_rects.len() {
            rect.right()
        } else {
            (rect.center().x + trigger_rects[index + 1].center().x) * 0.5
        };

        if pointer_pos.x >= left && pointer_pos.x <= right {
            return Some(index);
        }
    }

    None
}

fn load_menu_bar_memory(ui: &Ui, id: Id) -> MenuBarMemory {
    ui.data(|data| {
        data.get_temp::<MenuBarMemory>(id.with("menu_bar_state"))
            .unwrap_or_default()
    })
}

fn store_menu_bar_memory(ui: &mut Ui, id: Id, state: MenuBarMemory) {
    ui.data_mut(|data| {
        if state.active_menu.is_some() {
            data.insert_temp(id.with("menu_bar_state"), state);
        } else {
            data.remove::<MenuBarMemory>(id.with("menu_bar_state"));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{menu_index_at_pointer, resolve_active_menu, MenuBar, MenuBarItem};
    use crate::components::{ComponentUiExt, DropdownMenuEntry};
    use egui::{pos2, vec2, CentralPanel, Context, Id, RawInput, Rect};

    const FILE_ENTRIES: [DropdownMenuEntry<'static>; 1] =
        [DropdownMenuEntry::action(0, "New File")];
    const EDIT_ENTRIES: [DropdownMenuEntry<'static>; 1] = [DropdownMenuEntry::action(1, "Undo")];
    const MENU_ITEMS: [MenuBarItem<'static>; 2] = [
        MenuBarItem::new("File", &FILE_ENTRIES),
        MenuBarItem::new("Edit", &EDIT_ENTRIES),
    ];

    #[test]
    fn pointer_regions_switch_by_horizontal_position() {
        let trigger_rects = [
            Rect::from_min_size(pos2(10.0, 10.0), vec2(40.0, 28.0)),
            Rect::from_min_size(pos2(54.0, 10.0), vec2(40.0, 28.0)),
            Rect::from_min_size(pos2(98.0, 10.0), vec2(50.0, 28.0)),
        ];
        let bar_rect = Rect::from_min_max(pos2(8.0, 8.0), pos2(152.0, 42.0));

        assert_eq!(
            menu_index_at_pointer(Some(pos2(18.0, 20.0)), &trigger_rects, bar_rect),
            Some(0)
        );
        assert_eq!(
            menu_index_at_pointer(Some(pos2(76.0, 20.0)), &trigger_rects, bar_rect),
            Some(1)
        );
        assert_eq!(
            menu_index_at_pointer(Some(pos2(132.0, 20.0)), &trigger_rects, bar_rect),
            Some(2)
        );
        assert_eq!(
            menu_index_at_pointer(Some(pos2(76.0, 50.0)), &trigger_rects, bar_rect),
            None
        );
    }

    #[test]
    fn hovering_new_trigger_switches_open_menu() {
        let openable = [true, true, false];

        assert_eq!(
            resolve_active_menu(Some(0), None, Some(1), &openable),
            Some(1)
        );
        assert_eq!(
            resolve_active_menu(Some(0), None, Some(2), &openable),
            Some(0)
        );
        assert_eq!(resolve_active_menu(Some(0), Some(0), None, &openable), None);
        assert_eq!(resolve_active_menu(None, Some(1), None, &openable), Some(1));
    }

    #[test]
    fn renders_menu_bar_without_panic() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = ui
                    .components()
                    .menu_bar(MenuBar::new(Id::new("menu_bar_test"), &MENU_ITEMS))
                    .0
                    .rect;
            });
        });

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
    }
}
