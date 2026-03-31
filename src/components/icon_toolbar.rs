use super::{api::ComponentUi, common::ControlSize, tooltip::attach_text_tooltip};
use crate::ui::icons;
use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Id, Rect, Response, Sense, Stroke, StrokeKind,
    Ui,
};

const ICON_TOOLBAR_GAP: f32 = 4.0;
const ICON_TOOLBAR_PADDING_X: i8 = 6;
const ICON_TOOLBAR_PADDING_Y: i8 = 4;
const ICON_TOOLBAR_CORNER_RADIUS: u8 = 12;
const ICON_TOOLBAR_ITEM_CORNER_RADIUS: u8 = 8;

#[derive(Debug, Clone, Copy)]
pub struct IconToolbarItem<'a> {
    pub icon: &'a str,
    pub tooltip: Option<&'a str>,
    pub badge_fill: Option<Color32>,
}

impl<'a> IconToolbarItem<'a> {
    pub const fn new(icon: &'a str) -> Self {
        Self {
            icon,
            tooltip: None,
            badge_fill: None,
        }
    }

    pub const fn tooltip(mut self, tooltip: &'a str) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    pub const fn badge_fill(mut self, badge_fill: Color32) -> Self {
        self.badge_fill = Some(badge_fill);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IconToolbar<'a> {
    pub id: Id,
    pub items: &'a [IconToolbarItem<'a>],
    pub size: ControlSize,
    pub fill: Option<Color32>,
    pub stroke: Option<Stroke>,
    pub corner_radius: Option<u8>,
    pub item_corner_radius: Option<u8>,
    pub padding_x: i8,
    pub padding_y: i8,
    pub gap: f32,
    pub shadow: Option<egui::Shadow>,
    pub icon_size: Option<f32>,
    pub icon_tint: Option<Color32>,
    pub selected_fill: Option<Color32>,
    pub selected_icon_tint: Option<Color32>,
}

impl<'a> IconToolbar<'a> {
    pub fn new(id: Id, items: &'a [IconToolbarItem<'a>]) -> Self {
        Self {
            id,
            items,
            size: ControlSize::Md,
            fill: None,
            stroke: None,
            corner_radius: None,
            item_corner_radius: None,
            padding_x: ICON_TOOLBAR_PADDING_X,
            padding_y: ICON_TOOLBAR_PADDING_Y,
            gap: ICON_TOOLBAR_GAP,
            shadow: None,
            icon_size: None,
            icon_tint: None,
            selected_fill: None,
            selected_icon_tint: None,
        }
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
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

    pub fn item_corner_radius(mut self, item_corner_radius: u8) -> Self {
        self.item_corner_radius = Some(item_corner_radius);
        self
    }

    pub fn padding(mut self, x: i8, y: i8) -> Self {
        self.padding_x = x;
        self.padding_y = y;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    pub fn shadow(mut self, shadow: egui::Shadow) -> Self {
        self.shadow = Some(shadow);
        self
    }

    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = Some(icon_size.max(1.0));
        self
    }

    pub fn icon_tint(mut self, icon_tint: Color32) -> Self {
        self.icon_tint = Some(icon_tint);
        self
    }

    pub fn selected_fill(mut self, selected_fill: Color32) -> Self {
        self.selected_fill = Some(selected_fill);
        self
    }

    pub fn selected_icon_tint(mut self, selected_icon_tint: Color32) -> Self {
        self.selected_icon_tint = Some(selected_icon_tint);
        self
    }
}

impl ComponentUi<'_> {
    pub fn icon_toolbar<'a>(
        &mut self,
        current: &mut usize,
        props: impl Into<IconToolbar<'a>>,
    ) -> Response {
        draw_icon_toolbar(self.ui_mut(), current, props.into())
    }
}

fn draw_icon_toolbar(ui: &mut Ui, current: &mut usize, props: IconToolbar<'_>) -> Response {
    if props.items.is_empty() {
        return ui.allocate_response(egui::Vec2::ZERO, Sense::hover());
    }

    let item_extent = item_extent(props.size);
    let icon_size = props
        .icon_size
        .unwrap_or(default_icon_size(props.size))
        .max(1.0);
    let group_size = icon_toolbar_size(
        props.items.len(),
        item_extent,
        props.gap,
        props.padding_x,
        props.padding_y,
    );
    let (group_rect, _) = ui.allocate_exact_size(group_size, Sense::hover());
    let group_corner_radius =
        CornerRadius::same(props.corner_radius.unwrap_or(ICON_TOOLBAR_CORNER_RADIUS));
    let item_corner_radius = CornerRadius::same(
        props
            .item_corner_radius
            .unwrap_or(ICON_TOOLBAR_ITEM_CORNER_RADIUS),
    );
    let fill = props.fill.unwrap_or(default_toolbar_fill());
    let stroke = props.stroke.unwrap_or(default_toolbar_stroke());
    let shadow = props.shadow.unwrap_or(default_toolbar_shadow());
    let hover_fill = default_hover_fill();
    let active_fill = default_active_fill();
    let selected_fill = props.selected_fill.unwrap_or(default_selected_fill());
    let selected_hover_fill = default_selected_hover_fill();
    let selected_active_fill = default_selected_active_fill();
    let icon_tint = props.icon_tint.unwrap_or(default_icon_tint());
    let selected_icon_tint = props.selected_icon_tint.unwrap_or(Color32::WHITE);
    let item_size = vec2(item_extent, item_extent);
    *current = (*current).min(props.items.len().saturating_sub(1));

    if shadow != egui::Shadow::NONE {
        ui.painter()
            .add(shadow.as_shape(group_rect, group_corner_radius));
    }

    ui.painter().rect(
        group_rect,
        group_corner_radius,
        fill,
        stroke,
        StrokeKind::Outside,
    );

    let mut combined_response: Option<Response> = None;
    let mut item_left = group_rect.left() + f32::from(props.padding_x);
    let item_top = group_rect.top() + f32::from(props.padding_y);

    for (index, item) in props.items.iter().copied().enumerate() {
        let item_rect = Rect::from_min_size(pos2(item_left, item_top), item_size);
        let selected = *current == index;
        let response = ui
            .interact(item_rect, props.id.with(index), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);

        if let Some(tooltip) = item.tooltip {
            attach_text_tooltip(ui, &response, tooltip);
        }

        let item_fill = if selected {
            if response.is_pointer_button_down_on() {
                selected_active_fill
            } else if response.hovered() {
                selected_hover_fill
            } else {
                selected_fill
            }
        } else if response.is_pointer_button_down_on() {
            active_fill
        } else if response.hovered() {
            hover_fill
        } else {
            Color32::TRANSPARENT
        };

        if selected || response.hovered() || response.is_pointer_button_down_on() {
            ui.painter().rect(
                item_rect,
                item_corner_radius,
                item_fill,
                Stroke::NONE,
                StrokeKind::Outside,
            );
        }

        if let Some(image) = icons::image(ui.ctx(), item.icon, icon_size) {
            let icon_rect = Rect::from_center_size(item_rect.center(), vec2(icon_size, icon_size));
            image
                .tint(if selected {
                    selected_icon_tint
                } else {
                    icon_tint
                })
                .paint_at(ui, icon_rect);

            if let Some(badge_fill) = item.badge_fill {
                let badge_radius = if props.size.is_small() { 3.0 } else { 3.5 };
                let badge_center = pos2(
                    icon_rect.right() - badge_radius * 0.4,
                    icon_rect.bottom() - badge_radius * 0.25,
                );
                ui.painter().circle(
                    badge_center,
                    badge_radius,
                    badge_fill,
                    Stroke::new(1.0, fill),
                );
            }
        }

        if response.clicked() && !selected {
            *current = index;
        }

        combined_response = Some(match combined_response.take() {
            Some(previous) => previous.union(response),
            None => response,
        });
        item_left = item_rect.right() + props.gap;
    }

    combined_response.unwrap_or_else(|| ui.allocate_response(egui::Vec2::ZERO, Sense::hover()))
}

fn item_extent(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 28.0,
        ControlSize::Md => 32.0,
    }
}

fn icon_toolbar_size(
    item_count: usize,
    item_extent: f32,
    gap: f32,
    padding_x: i8,
    padding_y: i8,
) -> egui::Vec2 {
    vec2(
        f32::from(padding_x) * 2.0
            + item_extent * item_count as f32
            + gap * item_count.saturating_sub(1) as f32,
        f32::from(padding_y) * 2.0 + item_extent,
    )
}

fn default_icon_size(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 14.0,
        ControlSize::Md => 15.5,
    }
}

fn default_toolbar_fill() -> Color32 {
    Color32::from_rgb(34, 36, 40)
}

fn default_toolbar_stroke() -> Stroke {
    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 30))
}

fn default_toolbar_shadow() -> egui::Shadow {
    egui::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: Color32::from_rgba_unmultiplied(0, 0, 0, 64),
    }
}

fn default_hover_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, 16)
}

fn default_active_fill() -> Color32 {
    default_hover_fill()
}

fn default_selected_fill() -> Color32 {
    Color32::from_rgb(40, 150, 255)
}

fn default_selected_hover_fill() -> Color32 {
    Color32::from_rgb(56, 160, 255)
}

fn default_selected_active_fill() -> Color32 {
    default_selected_hover_fill()
}

fn default_icon_tint() -> Color32 {
    Color32::from_rgb(225, 229, 235)
}

#[cfg(test)]
mod tests {
    use super::{
        item_extent, IconToolbar, IconToolbarItem, ICON_TOOLBAR_GAP, ICON_TOOLBAR_PADDING_X,
        ICON_TOOLBAR_PADDING_Y,
    };
    use crate::components::{ComponentUiExt, ControlSize};
    use egui::{
        pos2, CentralPanel, Context, Event, Id, Modifiers, PointerButton, Pos2, RawInput, Rect,
    };

    const TEST_ITEMS: &[IconToolbarItem<'_>] = &[
        IconToolbarItem::new("mouse-pointer-2").tooltip("Select"),
        IconToolbarItem::new("move").tooltip("Move"),
        IconToolbarItem::new("rotate-ccw").tooltip("Rotate"),
    ];

    #[test]
    fn empty_icon_toolbar_allocates_no_clickable_area() {
        let context = Context::default();
        let mut selected = 0;
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = ui
                    .components()
                    .icon_toolbar(
                        &mut selected,
                        IconToolbar::new(Id::new("empty"), &[] as &[IconToolbarItem<'_>]),
                    )
                    .rect;
            });
        });

        assert_eq!(rect.size(), egui::Vec2::ZERO);
        assert_eq!(selected, 0);
    }

    #[test]
    fn clicking_icon_toolbar_item_updates_selection() {
        let context = Context::default();
        let mut selected = 0;
        let second_center = render_toolbar(&context, RawInput::default(), &mut selected);

        let _ = render_toolbar(&context, pointer_input(second_center, true), &mut selected);
        let _ = render_toolbar(&context, pointer_input(second_center, false), &mut selected);

        assert_eq!(selected, 1);
    }

    #[test]
    fn icon_toolbar_clamps_selection_to_last_item() {
        let context = Context::default();
        let mut selected = 99;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = ui.components().icon_toolbar(
                    &mut selected,
                    IconToolbar::new(Id::new("clamped"), TEST_ITEMS),
                );
            });
        });

        assert_eq!(selected, TEST_ITEMS.len() - 1);
    }

    fn render_toolbar(context: &Context, input: RawInput, selected: &mut usize) -> Pos2 {
        let mut second_center = Pos2::ZERO;

        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                let origin = ui.next_widget_position();
                second_center = pos2(
                    origin.x
                        + f32::from(ICON_TOOLBAR_PADDING_X)
                        + item_extent(ControlSize::Md)
                        + ICON_TOOLBAR_GAP
                        + item_extent(ControlSize::Md) * 0.5,
                    origin.y
                        + f32::from(ICON_TOOLBAR_PADDING_Y)
                        + item_extent(ControlSize::Md) * 0.5,
                );
                let _ = ui.components().icon_toolbar(
                    selected,
                    IconToolbar::new(Id::new("toolbar_actions"), TEST_ITEMS),
                );
            });
        });

        second_center
    }

    fn pointer_input(position: Pos2, pressed: bool) -> RawInput {
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
