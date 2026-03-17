use super::{
    api::ComponentUi,
    dropdown_menu::{show_menu_entries_surface, DropdownMenuEntry},
};
use crate::ui::{tokens, typography};
use egui::{
    Align2, Color32, CornerRadius, CursorIcon, FontId, Id, Popup, Rect, Response, Stroke,
    StrokeKind, Ui,
};

const MENU_BAR_ITEM_HEIGHT: f32 = 28.0;
const MENU_BAR_ITEM_PADDING_X: f32 = 10.0;
const MENU_BAR_ITEM_GAP: f32 = 1.0;
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
    pub corner_radius: Option<u8>,
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
            corner_radius: None,
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
        self.corner_radius = Some(corner_radius);
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
        draw_menu_bar(self.ui_mut(), props.into())
    }
}

fn draw_menu_bar(ui: &mut Ui, props: MenuBar<'_>) -> (Response, MenuBarState) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let corner_radius = props.corner_radius.unwrap_or(tokens::radius_md(runtime));
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
    let item_widths = props
        .items
        .iter()
        .map(|item| menu_bar_item_width(ui, item.label, runtime))
        .collect::<Vec<_>>();
    let group_size = menu_bar_group_size(&item_widths, props.padding_x, props.padding_y);
    let (bar_rect, bar_response) = ui.allocate_exact_size(group_size, egui::Sense::hover());

    let shadow = props.shadow.unwrap_or(egui::Shadow::NONE);
    if shadow != egui::Shadow::NONE {
        ui.painter()
            .add(shadow.as_shape(bar_rect, CornerRadius::same(corner_radius)));
    }

    ui.painter().rect(
        bar_rect,
        CornerRadius::same(corner_radius),
        props.fill.unwrap_or(tokens::card_background(runtime)),
        props
            .stroke
            .unwrap_or(Stroke::new(1.0, tokens::separator(runtime))),
        StrokeKind::Outside,
    );

    for (index, (item, rect)) in props
        .items
        .iter()
        .copied()
        .zip(menu_bar_item_rects(
            bar_rect,
            &item_widths,
            props.padding_x,
            props.padding_y,
        ))
        .enumerate()
    {
        let response = draw_menu_bar_trigger(
            ui,
            props.id.with(("trigger", index)),
            rect,
            item.label,
            active_menu == Some(index) && item.can_open(),
            runtime,
        );
        if response.clicked() {
            clicked_trigger = Some(index);
        }
        trigger_rects.push(response.rect);
        trigger_responses.push(response);
    }

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
        let popup_response = Popup::menu(&trigger_responses[active_index])
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
            });

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

fn draw_menu_bar_trigger(
    ui: &mut Ui,
    id: Id,
    rect: Rect,
    label: &str,
    active: bool,
    runtime: crate::theme::ThemeRuntime,
) -> Response {
    let font_id: FontId = typography::proportional(MENU_BAR_TEXT_SIZE);
    let response = ui.interact(rect, id, egui::Sense::click());

    let fill = if response.is_pointer_button_down_on() {
        tokens::row_active_bg(runtime)
    } else if active || response.hovered() {
        tokens::row_selected_bg(runtime)
    } else {
        tokens::TRANSPARENT
    };

    ui.painter().rect(
        rect,
        CornerRadius::same(tokens::radius_sm(runtime)),
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
            tokens::text_primary(runtime)
        } else {
            tokens::text_secondary(runtime)
        },
    );

    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn menu_bar_item_width(ui: &mut Ui, label: &str, runtime: crate::theme::ThemeRuntime) -> f32 {
    let font_id: FontId = typography::proportional(MENU_BAR_TEXT_SIZE);
    let text_width = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(label.to_owned(), font_id, tokens::text_secondary(runtime))
            .size()
            .x
    });
    (text_width + MENU_BAR_ITEM_PADDING_X * 2.0).ceil()
}

fn menu_bar_group_size(item_widths: &[f32], padding_x: i8, padding_y: i8) -> egui::Vec2 {
    let items_width = item_widths.iter().sum::<f32>()
        + (MENU_BAR_ITEM_GAP * item_widths.len().saturating_sub(1) as f32);
    egui::vec2(
        items_width + (f32::from(padding_x) * 2.0),
        MENU_BAR_ITEM_HEIGHT + (f32::from(padding_y) * 2.0),
    )
}

fn menu_bar_item_rects(
    bar_rect: Rect,
    item_widths: &[f32],
    padding_x: i8,
    padding_y: i8,
) -> Vec<Rect> {
    let mut rects = Vec::with_capacity(item_widths.len());
    let mut left = bar_rect.left() + f32::from(padding_x);
    let top = bar_rect.top() + f32::from(padding_y);

    for width in item_widths.iter().copied() {
        let rect = Rect::from_min_size(
            egui::pos2(left, top),
            egui::vec2(width, MENU_BAR_ITEM_HEIGHT),
        );
        rects.push(rect);
        left = rect.right() + MENU_BAR_ITEM_GAP;
    }

    rects
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
    use super::{
        menu_bar_group_size, menu_bar_item_rects, menu_index_at_pointer, resolve_active_menu,
        MenuBar, MenuBarItem,
    };
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

    #[test]
    fn menu_bar_padding_is_uniform() {
        let widths = [40.0, 52.0, 60.0];
        let bar_rect = Rect::from_min_size(pos2(16.0, 20.0), menu_bar_group_size(&widths, 4, 3));
        let item_rects = menu_bar_item_rects(bar_rect, &widths, 4, 3);
        let first = item_rects
            .first()
            .copied()
            .expect("missing first menu item");
        let last = item_rects.last().copied().expect("missing last menu item");

        assert_eq!(first.left() - bar_rect.left(), 4.0);
        assert_eq!(bar_rect.right() - last.right(), 4.0);
        assert_eq!(first.top() - bar_rect.top(), 3.0);
        assert_eq!(bar_rect.bottom() - first.bottom(), 3.0);
    }
}
