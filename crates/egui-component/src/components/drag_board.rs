use super::api::ComponentUi;
use crate::ui::{tokens, typography};
use egui::{
    vec2, Align, Align2, CornerRadius, CursorIcon, Id, Response, RichText, Sense, Stroke,
    StrokeKind, Ui, UiBuilder, Vec2,
};

const DRAG_BOARD_DEFAULT_HEIGHT: f32 = 280.0;
const DRAG_BOARD_REGION_GAP: f32 = 12.0;
const DRAG_BOARD_REGION_PADDING: f32 = 12.0;
const DRAG_BOARD_ITEM_GAP: f32 = 8.0;
const DRAG_BOARD_PLACEHOLDER_HEIGHT: f32 = 72.0;
const DRAG_BOARD_PLACEHOLDER_LABEL: &str = "Drop card here";

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub enum DragBoardRegion {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct DragBoardItem<'a> {
    pub id: Option<Id>,
    pub title: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> DragBoardItem<'a> {
    pub const fn new(title: &'a str) -> Self {
        Self {
            id: None,
            title,
            description: None,
        }
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub const fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DragBoard<'a> {
    pub id: Id,
    pub left_title: &'a str,
    pub right_title: &'a str,
    pub items: &'a [DragBoardItem<'a>],
    pub height: f32,
}

impl<'a> DragBoard<'a> {
    pub fn new(
        id: Id,
        left_title: &'a str,
        right_title: &'a str,
        items: &'a [DragBoardItem<'a>],
    ) -> Self {
        Self {
            id,
            left_title,
            right_title,
            items,
            height: DRAG_BOARD_DEFAULT_HEIGHT,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(1.0);
        self
    }
}

impl ComponentUi<'_> {
    pub fn drag_board<'a>(
        &mut self,
        current_regions: &mut [DragBoardRegion],
        props: impl Into<DragBoard<'a>>,
    ) -> Response {
        draw_drag_board(self.ui_mut(), current_regions, props.into())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct DragBoardPayload {
    board_id: Id,
    item_index: usize,
    source: DragBoardRegion,
}

fn draw_drag_board(
    ui: &mut Ui,
    current_regions: &mut [DragBoardRegion],
    props: DragBoard<'_>,
) -> Response {
    let available_width = ui.available_width().max(DRAG_BOARD_REGION_GAP + 2.0);
    let region_width = ((available_width - DRAG_BOARD_REGION_GAP) / 2.0).max(1.0);
    let runtime = crate::theme::runtime_for_ui(ui);
    let mut combined_response: Option<Response> = None;
    let mut changed = false;

    let _ = ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = DRAG_BOARD_REGION_GAP;
        ui.horizontal(|ui| {
            for (region, title) in [
                (DragBoardRegion::Left, props.left_title),
                (DragBoardRegion::Right, props.right_title),
            ] {
                let response = draw_drag_board_region(
                    ui,
                    current_regions,
                    props,
                    region,
                    title,
                    region_width,
                    runtime,
                    &mut changed,
                );
                combined_response = Some(match combined_response.take() {
                    Some(previous) => previous.union(response),
                    None => response,
                });
            }
        })
    });

    let mut response =
        combined_response.unwrap_or_else(|| ui.allocate_response(Vec2::ZERO, Sense::hover()));
    if changed {
        response.mark_changed();
    }
    response
}

fn draw_drag_board_region(
    ui: &mut Ui,
    current_regions: &mut [DragBoardRegion],
    props: DragBoard<'_>,
    region: DragBoardRegion,
    title: &str,
    width: f32,
    runtime: crate::theme::ThemeRuntime,
    changed: &mut bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, props.height), Sense::hover());
    let hovered_payload = hovered_payload_for_board(ui.ctx(), &response, props.id);
    let accepting_drop = hovered_payload.is_some_and(|payload| payload.source != region);

    let fill = if accepting_drop {
        tokens::button_secondary_hover_bg(runtime)
    } else {
        tokens::muted_surface(runtime)
    };
    let stroke = Stroke::new(
        1.0,
        if accepting_drop {
            tokens::button_secondary_hover_border(runtime)
        } else {
            tokens::separator(runtime)
        },
    );
    let corner_radius = CornerRadius::same(tokens::radius_lg(runtime));
    ui.painter()
        .rect(rect, corner_radius, fill, stroke, StrokeKind::Outside);

    let inner_rect = rect.shrink2(vec2(DRAG_BOARD_REGION_PADDING, DRAG_BOARD_REGION_PADDING));
    let mut child_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(inner_rect)
            .layout(egui::Layout::top_down(Align::Min)),
    );

    let _ = child_ui.add(
        egui::Label::new(
            RichText::new(title)
                .font(typography::label_font())
                .color(tokens::text_primary(runtime)),
        )
        .selectable(false),
    );
    child_ui.add_space(10.0);

    let mut item_count = 0usize;
    for (index, item) in props.items.iter().enumerate() {
        if current_regions.get(index).copied() != Some(region) {
            continue;
        }

        let payload = DragBoardPayload {
            board_id: props.id,
            item_index: index,
            source: region,
        };
        let drag_id = item.id.unwrap_or_else(|| props.id.with(("card", index)));
        let _ = child_ui.dnd_drag_source(drag_id, payload, |ui| {
            draw_drag_board_item(ui, *item, runtime, inner_rect.width())
        });
        item_count += 1;
        if item_count < region_item_count(current_regions, region) {
            child_ui.add_space(DRAG_BOARD_ITEM_GAP);
        }
    }

    if item_count == 0 {
        let placeholder_rect = child_ui
            .allocate_exact_size(
                vec2(
                    inner_rect.width(),
                    DRAG_BOARD_PLACEHOLDER_HEIGHT.min(child_ui.available_height()),
                ),
                Sense::hover(),
            )
            .0;
        ui.painter().rect(
            placeholder_rect,
            CornerRadius::same(tokens::radius_md(runtime)),
            tokens::card_background(runtime),
            Stroke::new(1.0, tokens::separator(runtime)),
            StrokeKind::Outside,
        );
        ui.painter().text(
            placeholder_rect.center(),
            Align2::CENTER_CENTER,
            DRAG_BOARD_PLACEHOLDER_LABEL,
            typography::label_font(),
            tokens::text_muted(runtime),
        );
    }

    if let Some(payload) = released_payload_for_board(ui.ctx(), &response, props.id) {
        if payload.source != region {
            if let Some(target) = current_regions.get_mut(payload.item_index) {
                *target = region;
                *changed = true;
            }
        }
    }

    response
}

fn draw_drag_board_item(
    ui: &mut Ui,
    item: DragBoardItem<'_>,
    runtime: crate::theme::ThemeRuntime,
    width: f32,
) {
    let _ = ui.scope(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);
        let frame = egui::Frame::new()
            .fill(tokens::card_background(runtime))
            .stroke(Stroke::new(1.0, tokens::separator(runtime)))
            .corner_radius(CornerRadius::same(tokens::radius_md(runtime)))
            .inner_margin(egui::Margin::same(12));
        let response = frame.show(ui, |ui| {
            let _ = ui.scope(|ui| {
                ui.spacing_mut().item_spacing.y = DRAG_BOARD_ITEM_GAP;
                ui.vertical(|ui| {
                    let _ = ui.add(
                        egui::Label::new(
                            RichText::new(item.title)
                                .font(typography::label_font())
                                .color(tokens::text_primary(runtime)),
                        )
                        .selectable(false),
                    );
                    if let Some(description) = item.description.filter(|text| !text.is_empty()) {
                        let _ = ui.add(
                            egui::Label::new(
                                RichText::new(description)
                                    .size(12.0)
                                    .color(tokens::text_muted(runtime)),
                            )
                            .selectable(false),
                        );
                    }
                })
            });
        });
        let _ = response.response.on_hover_cursor(CursorIcon::Grab);
    });
}

fn region_item_count(current_regions: &[DragBoardRegion], region: DragBoardRegion) -> usize {
    current_regions
        .iter()
        .filter(|value| **value == region)
        .count()
}

fn hovered_payload_for_board(
    ctx: &egui::Context,
    response: &Response,
    board_id: Id,
) -> Option<DragBoardPayload> {
    if !response.contains_pointer() {
        return None;
    }

    egui::DragAndDrop::payload::<DragBoardPayload>(ctx)
        .map(|payload| *payload)
        .filter(|payload| payload.board_id == board_id)
}

fn released_payload_for_board(
    ctx: &egui::Context,
    response: &Response,
    board_id: Id,
) -> Option<DragBoardPayload> {
    if !response.contains_pointer() || !ctx.input(|input| input.pointer.any_released()) {
        return None;
    }

    let payload = egui::DragAndDrop::payload::<DragBoardPayload>(ctx)
        .map(|payload| *payload)
        .filter(|payload| payload.board_id == board_id)?;
    let _ = egui::DragAndDrop::take_payload::<DragBoardPayload>(ctx);
    Some(payload)
}

#[cfg(test)]
mod tests {
    use super::{
        DragBoard, DragBoardItem, DragBoardPayload, DragBoardRegion, DRAG_BOARD_REGION_GAP,
    };
    use crate::runtime_components::ComponentUiExt;
    use crate::theme::{self, ThemeMode};
    use egui::{pos2, vec2, CentralPanel, Context, Event, Id, Modifiers, PointerButton, RawInput};

    const TEST_WIDTH: f32 = 560.0;
    const TEST_ITEMS: [DragBoardItem<'static>; 3] = [
        DragBoardItem::new("Polish header spacing").description("Shared toolbar chrome"),
        DragBoardItem::new("Tune sidebar spacing").description("Examples rail"),
        DragBoardItem::new("Ship drag board").description("Trello-style preview"),
    ];

    #[test]
    fn dropping_on_other_board_instance_is_ignored() {
        let context = test_context();
        let mut current_regions = [
            DragBoardRegion::Left,
            DragBoardRegion::Left,
            DragBoardRegion::Right,
        ];
        let board = test_board();
        let layout = render_layout(&context, RawInput::default(), &mut current_regions, board);

        egui::DragAndDrop::set_payload(
            &context,
            DragBoardPayload {
                board_id: Id::new("different"),
                item_index: 0,
                source: DragBoardRegion::Left,
            },
        );

        let _ = render_layout(
            &context,
            pointer_release_input(layout.right_center),
            &mut current_regions,
            board,
        );

        assert_eq!(
            current_regions,
            [
                DragBoardRegion::Left,
                DragBoardRegion::Left,
                DragBoardRegion::Right,
            ]
        );
    }

    #[test]
    fn dropping_on_opposite_region_updates_selected_card_state() {
        let context = test_context();
        let mut current_regions = [
            DragBoardRegion::Left,
            DragBoardRegion::Left,
            DragBoardRegion::Right,
        ];
        let board = test_board();
        let layout = render_layout(&context, RawInput::default(), &mut current_regions, board);

        egui::DragAndDrop::set_payload(
            &context,
            DragBoardPayload {
                board_id: test_board_id(),
                item_index: 0,
                source: DragBoardRegion::Left,
            },
        );

        let _ = render_layout(
            &context,
            pointer_release_input(layout.right_center),
            &mut current_regions,
            board,
        );

        assert_eq!(
            current_regions,
            [
                DragBoardRegion::Right,
                DragBoardRegion::Left,
                DragBoardRegion::Right,
            ]
        );
    }

    #[test]
    fn dropping_on_same_region_keeps_state() {
        let context = test_context();
        let mut current_regions = [
            DragBoardRegion::Left,
            DragBoardRegion::Left,
            DragBoardRegion::Right,
        ];
        let board = test_board();
        let layout = render_layout(&context, RawInput::default(), &mut current_regions, board);

        egui::DragAndDrop::set_payload(
            &context,
            DragBoardPayload {
                board_id: test_board_id(),
                item_index: 0,
                source: DragBoardRegion::Left,
            },
        );

        let _ = render_layout(
            &context,
            pointer_release_input(layout.left_center),
            &mut current_regions,
            board,
        );

        assert_eq!(
            current_regions,
            [
                DragBoardRegion::Left,
                DragBoardRegion::Left,
                DragBoardRegion::Right,
            ]
        );
    }

    #[test]
    fn rendering_multiple_items_does_not_panic() {
        let context = test_context();
        let mut current_regions = [
            DragBoardRegion::Left,
            DragBoardRegion::Left,
            DragBoardRegion::Right,
        ];

        let _ = render_layout(
            &context,
            RawInput::default(),
            &mut current_regions,
            test_board(),
        );

        assert_eq!(current_regions[0], DragBoardRegion::Left);
    }

    struct BoardLayout {
        left_center: egui::Pos2,
        right_center: egui::Pos2,
    }

    fn render_layout(
        context: &Context,
        input: RawInput,
        current_regions: &mut [DragBoardRegion],
        props: DragBoard<'_>,
    ) -> BoardLayout {
        let mut board_rect = egui::Rect::NOTHING;

        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = ui.scope(|ui| {
                    ui.set_min_width(TEST_WIDTH);
                    ui.set_max_width(TEST_WIDTH);
                    board_rect = ui.components().drag_board(current_regions, props).rect;
                });
            });
        });

        let left_width = ((board_rect.width() - DRAG_BOARD_REGION_GAP) / 2.0).max(1.0);
        let left_rect =
            egui::Rect::from_min_size(board_rect.min, vec2(left_width, board_rect.height()));
        let right_rect = egui::Rect::from_min_size(
            pos2(left_rect.right() + DRAG_BOARD_REGION_GAP, board_rect.top()),
            vec2(left_width, board_rect.height()),
        );

        BoardLayout {
            left_center: left_rect.center(),
            right_center: right_rect.center(),
        }
    }

    fn pointer_release_input(position: egui::Pos2) -> RawInput {
        RawInput {
            events: vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..RawInput::default()
        }
    }

    fn test_context() -> Context {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        context
    }

    fn test_board_id() -> Id {
        Id::new("drag_board_test")
    }

    fn test_board() -> DragBoard<'static> {
        DragBoard::new(test_board_id(), "Backlog", "Done", &TEST_ITEMS)
    }
}
