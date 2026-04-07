use super::{api::ComponentUi, Card, ComponentUiExt, Label, LabelTone, LabelWeight, NumberInput};
use crate::layout;
use crate::ui::{tokens, typography};
use egui::{
    Align2, Color32, CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
};
use std::ops::RangeInclusive;

#[derive(Debug, Clone, Copy)]
pub struct CanvaInspectorCard {
    pub padding_x: i8,
    pub padding_y: i8,
}

impl CanvaInspectorCard {
    pub fn new() -> Self {
        Self {
            padding_x: 16,
            padding_y: 16,
        }
    }

    pub fn padding(mut self, x: i8, y: i8) -> Self {
        self.padding_x = x;
        self.padding_y = y;
        self
    }
}

impl Default for CanvaInspectorCard {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CanvaInspectorHeader<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> CanvaInspectorHeader<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            description: None,
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
}

#[derive(Debug, Clone)]
pub struct CanvaAxisField<'a> {
    pub id: Id,
    pub label: &'a str,
    pub tint: Color32,
    pub width: f32,
    pub range: Option<RangeInclusive<f64>>,
    pub speed: f64,
    pub decimals: usize,
    pub suffix: &'a str,
}

impl<'a> CanvaAxisField<'a> {
    pub fn new(id: Id, label: &'a str) -> Self {
        Self {
            id,
            label,
            tint: Color32::WHITE,
            width: 96.0,
            range: None,
            speed: 1.0,
            decimals: 0,
            suffix: "",
        }
    }

    pub fn tint(mut self, tint: Color32) -> Self {
        self.tint = tint;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn range(mut self, range: RangeInclusive<f64>) -> Self {
        self.range = Some(range);
        self
    }

    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = suffix;
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CanvaNinePoint {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy)]
pub struct CanvaColorStop {
    pub time: f32,
    pub color: Color32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CanvaTimelineAction {
    AddAtPointer,
}

impl ComponentUi<'_> {
    pub fn canva_inspector_card<R>(
        &mut self,
        props: impl Into<CanvaInspectorCard>,
        add: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        draw_canva_inspector_card(self.ui_mut(), props.into(), add)
    }

    pub fn canva_property_row<R>(
        &mut self,
        title: &str,
        label_width: f32,
        add: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        draw_canva_property_row(self.ui_mut(), title, label_width, add)
    }

    pub fn canva_inspector_header(
        &mut self,
        props: CanvaInspectorHeader<'_>,
        add_actions: impl FnOnce(&mut Ui),
    ) -> Response {
        draw_canva_inspector_header(self.ui_mut(), props, add_actions)
    }

    pub fn canva_choice_chips(&mut self, id: Id, selected: &mut usize, options: &[&str]) -> bool {
        draw_canva_choice_chips(self.ui_mut(), id, selected, options)
    }

    pub fn canva_axis_field(&mut self, value: &mut f64, props: CanvaAxisField<'_>) -> Response {
        draw_canva_axis_field(self.ui_mut(), value, props)
    }

    pub fn canva_anchor_picker(
        &mut self,
        id: Id,
        selected: &mut CanvaNinePoint,
        preview_origin: CanvaNinePoint,
        locked: bool,
    ) -> Response {
        draw_canva_anchor_picker(self.ui_mut(), id, selected, preview_origin, locked)
    }

    pub fn canva_origin_picker(
        &mut self,
        id: Id,
        parent_anchor: CanvaNinePoint,
        selected: &mut CanvaNinePoint,
        locked: &mut bool,
    ) -> Response {
        draw_canva_origin_picker(self.ui_mut(), id, parent_anchor, selected, locked)
    }

    pub fn canva_color_timeline_editor(
        &mut self,
        id: Id,
        points: &[CanvaColorStop],
    ) -> (Response, Option<CanvaTimelineAction>) {
        draw_canva_color_timeline_editor(self.ui_mut(), id, points)
    }
}

fn draw_canva_inspector_card<R>(
    ui: &mut Ui,
    props: CanvaInspectorCard,
    add: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    let runtime = crate::theme::runtime_for_ui(ui);
    let fill = tokens::input_background(runtime);
    let stroke = Stroke::new(1.0, tokens::separator(runtime));
    ui.components().card(
        Card::new()
            .padding(props.padding_x, props.padding_y)
            .fill(fill)
            .stroke(stroke),
        add,
    )
}

fn draw_canva_property_row<R>(
    ui: &mut Ui,
    title: &str,
    label_width: f32,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        let _ = ui.allocate_ui_with_layout(
            egui::vec2(label_width, ui.spacing().interact_size.y.max(26.0)),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let _ = ui.components().label(
                    Label::new(title)
                        .tone(LabelTone::Muted)
                        .weight(LabelWeight::Semibold)
                        .size(11.0),
                );
            },
        );
        add(ui)
    })
    .inner
}

fn draw_canva_inspector_header(
    ui: &mut Ui,
    props: CanvaInspectorHeader<'_>,
    add_actions: impl FnOnce(&mut Ui),
) -> Response {
    layout::leading_trailing()
        .gap(8.0)
        .min_height(if props.description.is_some() {
            36.0
        } else {
            28.0
        })
        .align(layout::Align::Center)
        .show(
            ui,
            |ui| {
                ui.vertical(|ui| {
                    let _ = ui.components().label(
                        Label::new(props.title)
                            .tone(LabelTone::Primary)
                            .weight(LabelWeight::Semibold)
                            .size(13.0),
                    );
                    if let Some(description) = props.description {
                        ui.add_space(2.0);
                        let _ = ui
                            .components()
                            .label(Label::new(description).tone(LabelTone::Muted).size(11.0));
                    }
                });
            },
            add_actions,
        )
}

fn draw_canva_choice_chips(ui: &mut Ui, id: Id, selected: &mut usize, options: &[&str]) -> bool {
    let runtime = crate::theme::runtime_for_ui(ui);
    let mut changed = false;
    egui::ScrollArea::horizontal()
        .id_salt(id)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let _ = layout::row().gap(8.0).show(ui, |ui| {
                for (index, label) in options.iter().enumerate() {
                    let is_selected = *selected == index;
                    let width = ((*label).len() as f32 * 7.4 + 28.0).max(64.0);
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(width, 32.0), Sense::click());
                    let fill = if is_selected {
                        tokens::row_selected_bg(runtime).linear_multiply(0.24)
                    } else if response.hovered() {
                        tokens::button_secondary_hover_bg(runtime)
                    } else {
                        tokens::button_secondary_bg(runtime)
                    };
                    let stroke = if is_selected {
                        Stroke::new(1.0, tokens::row_selected_bg(runtime))
                    } else {
                        Stroke::new(1.0, tokens::button_secondary_border(runtime))
                    };
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(tokens::radius_md(runtime)),
                        fill,
                        stroke,
                        StrokeKind::Inside,
                    );
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        *label,
                        typography::semibold_font(12.0),
                        if is_selected {
                            tokens::text_primary(runtime)
                        } else {
                            tokens::text_secondary(runtime)
                        },
                    );
                    if response.clicked() && !is_selected {
                        *selected = index;
                        changed = true;
                    }
                }
            });
        });
    changed
}

fn draw_canva_axis_field(ui: &mut Ui, value: &mut f64, props: CanvaAxisField<'_>) -> Response {
    let mut edited = *value as f32;
    let mut input = NumberInput::new(props.id)
        .width(props.width)
        .speed(props.speed)
        .decimals(props.decimals)
        .prefix(props.label)
        .prefix_tint(props.tint)
        .prefix_align_left();
    if let Some(range) = props.range.as_ref() {
        input = input.range(*range.start() as f32..=*range.end() as f32);
    }
    if !props.suffix.is_empty() {
        input = input.suffix(props.suffix);
    }
    let response = ui.components().number_input(&mut edited, input);
    if response.changed() {
        let mut next = f64::from(edited);
        if let Some(range) = props.range {
            next = next.clamp(*range.start(), *range.end());
        }
        *value = next;
    }
    response
}

fn draw_canva_anchor_picker(
    ui: &mut Ui,
    id: Id,
    selected: &mut CanvaNinePoint,
    preview_origin: CanvaNinePoint,
    locked: bool,
) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let side = ui.available_width().clamp(96.0, 132.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), Sense::hover());
    let response = ui
        .interact(rect, id, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    let painter = ui.painter();
    painter.rect(
        rect,
        CornerRadius::same(tokens::radius_md(runtime)),
        tokens::button_secondary_bg(runtime),
        Stroke::new(1.0, tokens::button_secondary_border(runtime)),
        StrokeKind::Inside,
    );
    let cell_width = rect.width() / 3.0;
    let cell_height = rect.height() / 3.0;
    if response.clicked() {
        if let Some(pointer) = response.interact_pointer_pos() {
            let local = pointer - rect.min;
            let col = (local.x / cell_width).floor().clamp(0.0, 2.0) as usize;
            let row = (local.y / cell_height).floor().clamp(0.0, 2.0) as usize;
            *selected = point_from_grid(row, col);
        }
    }
    let selected_cell_rect = grid_cell_rect(rect, *selected).shrink(2.0);
    painter.rect(
        selected_cell_rect,
        CornerRadius::same(tokens::radius_md(runtime) / 2),
        tokens::row_selected_bg(runtime).linear_multiply(0.16),
        Stroke::new(1.0, tokens::row_selected_bg(runtime)),
        StrokeKind::Inside,
    );
    let grid_stroke = Stroke::new(1.0, tokens::separator(runtime).linear_multiply(0.7));
    for step in 1..3 {
        let x = rect.left() + cell_width * step as f32;
        let y = rect.top() + cell_height * step as f32;
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            grid_stroke,
        );
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            grid_stroke,
        );
    }
    paint_anchor_preview(
        ui,
        rect,
        *selected,
        preview_origin,
        locked,
        tokens::row_selected_bg(runtime),
    );
    response
}

fn draw_canva_origin_picker(
    ui: &mut Ui,
    id: Id,
    parent_anchor: CanvaNinePoint,
    selected: &mut CanvaNinePoint,
    locked: &mut bool,
) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let side = ui.available_width().clamp(72.0, 96.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), Sense::hover());
    let response = ui
        .interact(rect, id, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    let square_rect = rect;
    let painter = ui.painter();
    painter.rect(
        square_rect,
        CornerRadius::same(tokens::radius_md(runtime) / 2),
        tokens::muted_surface(runtime),
        Stroke::new(1.0, tokens::separator(runtime)),
        StrokeKind::Inside,
    );
    let positions = point_positions(square_rect);
    if response.clicked() {
        if let Some(pointer) = response.interact_pointer_pos() {
            let mut nearest = None;
            let mut nearest_distance = f32::MAX;
            for (point, position) in positions {
                let distance = pointer.distance_sq(position);
                if distance < nearest_distance {
                    nearest = Some(point);
                    nearest_distance = distance;
                }
            }
            if let Some(point) = nearest {
                if *locked && *selected == point {
                    *locked = false;
                } else {
                    *selected = point;
                    *locked = true;
                }
            }
        }
    }
    let active_point = if *locked { *selected } else { parent_anchor };
    for (point, position) in positions {
        let is_selected = point == active_point;
        let fill = if is_selected {
            if *locked {
                tokens::row_selected_bg(runtime)
            } else {
                Color32::WHITE
            }
        } else {
            tokens::text_muted(runtime)
        };
        let radius = if is_selected && *locked { 4.5 } else { 3.5 };
        painter.circle_filled(position, radius, fill);
    }
    response
}

fn draw_canva_color_timeline_editor(
    ui: &mut Ui,
    id: Id,
    points: &[CanvaColorStop],
) -> (Response, Option<CanvaTimelineAction>) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let width = ui.available_width().max(140.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 28.0), Sense::click());
    let response = response.union(ui.interact(rect, id, Sense::click()));
    let painter = ui.painter_at(rect);
    painter.rect_filled(
        rect,
        CornerRadius::same(4),
        tokens::button_secondary_bg(runtime),
    );
    let steps = 72;
    for step in 0..steps {
        let t0 = step as f32 / steps as f32;
        let t1 = (step + 1) as f32 / steps as f32;
        let color = sample_color(points, (t0 + t1) * 0.5);
        let x0 = rect.left() + rect.width() * t0;
        let x1 = rect.left() + rect.width() * t1;
        let segment = Rect::from_min_max(
            Pos2::new(x0, rect.top() + 4.0),
            Pos2::new(x1.max(x0 + 1.0), rect.bottom() - 4.0),
        );
        painter.rect_filled(segment, CornerRadius::ZERO, color);
    }
    painter.rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(1.0, tokens::button_secondary_border(runtime)),
        StrokeKind::Inside,
    );
    for point in points {
        let marker_x = rect.left() + rect.width() * point.time.clamp(0.0, 1.0);
        let marker = Pos2::new(marker_x, rect.center().y);
        painter.circle_filled(marker, 4.0, point.color);
        painter.circle_stroke(marker, 4.0, Stroke::new(1.0, Color32::from_gray(12)));
    }
    let action = if response.clicked() {
        Some(CanvaTimelineAction::AddAtPointer)
    } else {
        None
    };
    (response, action)
}

fn sample_color(points: &[CanvaColorStop], time: f32) -> Color32 {
    if points.is_empty() {
        return Color32::WHITE;
    }
    if points.len() == 1 {
        return points[0].color;
    }
    let time = time.clamp(0.0, 1.0);
    for pair in points.windows(2) {
        if time <= pair[1].time {
            let span = (pair[1].time - pair[0].time).max(f32::EPSILON);
            let t = ((time - pair[0].time) / span).clamp(0.0, 1.0);
            return pair[0].color.lerp_to_gamma(pair[1].color, t);
        }
    }
    points
        .last()
        .map(|point| point.color)
        .unwrap_or(Color32::WHITE)
}

fn paint_anchor_preview(
    ui: &Ui,
    rect: Rect,
    parent_anchor: CanvaNinePoint,
    origin: CanvaNinePoint,
    locked: bool,
    accent: Color32,
) {
    let preview_side = (rect.width() * 0.28).clamp(24.0, 38.0);
    let anchor_center = point_in_rect(rect, parent_anchor);
    let offset = point_offset(origin, preview_side);
    let preview_rect = Rect::from_center_size(
        Pos2::new(anchor_center.x - offset.x, anchor_center.y - offset.y),
        egui::vec2(preview_side, preview_side),
    );
    ui.painter().rect(
        preview_rect,
        CornerRadius::same(4),
        accent.linear_multiply(if locked { 0.16 } else { 0.08 }),
        Stroke::new(1.0, accent),
        StrokeKind::Inside,
    );
    ui.painter().circle_filled(anchor_center, 3.0, accent);
}

fn point_from_grid(row: usize, col: usize) -> CanvaNinePoint {
    match (row.min(2), col.min(2)) {
        (0, 0) => CanvaNinePoint::TopLeft,
        (0, 1) => CanvaNinePoint::TopCenter,
        (0, 2) => CanvaNinePoint::TopRight,
        (1, 0) => CanvaNinePoint::MiddleLeft,
        (1, 1) => CanvaNinePoint::MiddleCenter,
        (1, 2) => CanvaNinePoint::MiddleRight,
        (2, 0) => CanvaNinePoint::BottomLeft,
        (2, 1) => CanvaNinePoint::BottomCenter,
        (2, 2) => CanvaNinePoint::BottomRight,
        _ => CanvaNinePoint::MiddleCenter,
    }
}

fn grid_position(value: CanvaNinePoint) -> (usize, usize) {
    match value {
        CanvaNinePoint::TopLeft => (0, 0),
        CanvaNinePoint::TopCenter => (0, 1),
        CanvaNinePoint::TopRight => (0, 2),
        CanvaNinePoint::MiddleLeft => (1, 0),
        CanvaNinePoint::MiddleCenter => (1, 1),
        CanvaNinePoint::MiddleRight => (1, 2),
        CanvaNinePoint::BottomLeft => (2, 0),
        CanvaNinePoint::BottomCenter => (2, 1),
        CanvaNinePoint::BottomRight => (2, 2),
    }
}

fn grid_cell_rect(rect: Rect, value: CanvaNinePoint) -> Rect {
    let (row, col) = grid_position(value);
    let cell_width = rect.width() / 3.0;
    let cell_height = rect.height() / 3.0;
    Rect::from_min_max(
        Pos2::new(
            rect.left() + cell_width * col as f32,
            rect.top() + cell_height * row as f32,
        ),
        Pos2::new(
            rect.left() + cell_width * (col + 1) as f32,
            rect.top() + cell_height * (row + 1) as f32,
        ),
    )
}

fn point_positions(rect: Rect) -> [(CanvaNinePoint, Pos2); 9] {
    let xs = [rect.left(), rect.center().x, rect.right()];
    let ys = [rect.top(), rect.center().y, rect.bottom()];
    [
        (CanvaNinePoint::TopLeft, Pos2::new(xs[0], ys[0])),
        (CanvaNinePoint::TopCenter, Pos2::new(xs[1], ys[0])),
        (CanvaNinePoint::TopRight, Pos2::new(xs[2], ys[0])),
        (CanvaNinePoint::MiddleLeft, Pos2::new(xs[0], ys[1])),
        (CanvaNinePoint::MiddleCenter, Pos2::new(xs[1], ys[1])),
        (CanvaNinePoint::MiddleRight, Pos2::new(xs[2], ys[1])),
        (CanvaNinePoint::BottomLeft, Pos2::new(xs[0], ys[2])),
        (CanvaNinePoint::BottomCenter, Pos2::new(xs[1], ys[2])),
        (CanvaNinePoint::BottomRight, Pos2::new(xs[2], ys[2])),
    ]
}

fn point_in_rect(rect: Rect, value: CanvaNinePoint) -> Pos2 {
    let (row, col) = grid_position(value);
    let x = match col {
        0 => rect.left() + rect.width() * 0.16,
        1 => rect.center().x,
        _ => rect.right() - rect.width() * 0.16,
    };
    let y = match row {
        0 => rect.top() + rect.height() * 0.16,
        1 => rect.center().y,
        _ => rect.bottom() - rect.height() * 0.16,
    };
    Pos2::new(x, y)
}

fn point_offset(value: CanvaNinePoint, side: f32) -> egui::Vec2 {
    match value {
        CanvaNinePoint::TopLeft => egui::vec2(-side * 0.5, -side * 0.5),
        CanvaNinePoint::TopCenter => egui::vec2(0.0, -side * 0.5),
        CanvaNinePoint::TopRight => egui::vec2(side * 0.5, -side * 0.5),
        CanvaNinePoint::MiddleLeft => egui::vec2(-side * 0.5, 0.0),
        CanvaNinePoint::MiddleCenter => egui::vec2(0.0, 0.0),
        CanvaNinePoint::MiddleRight => egui::vec2(side * 0.5, 0.0),
        CanvaNinePoint::BottomLeft => egui::vec2(-side * 0.5, side * 0.5),
        CanvaNinePoint::BottomCenter => egui::vec2(0.0, side * 0.5),
        CanvaNinePoint::BottomRight => egui::vec2(side * 0.5, side * 0.5),
    }
}

#[cfg(test)]
mod tests {
    use super::{point_from_grid, sample_color, CanvaColorStop, CanvaNinePoint};
    use crate::components::ComponentUiExt;
    use crate::theme::{self, ThemeMode};
    use egui::{CentralPanel, Context, Id, RawInput};

    #[test]
    fn grid_mapping_covers_bottom_right() {
        assert_eq!(point_from_grid(2, 2), CanvaNinePoint::BottomRight);
    }

    #[test]
    fn sample_color_interpolates_between_points() {
        let color = sample_color(
            &[
                CanvaColorStop {
                    time: 0.0,
                    color: egui::Color32::BLACK,
                },
                CanvaColorStop {
                    time: 1.0,
                    color: egui::Color32::WHITE,
                },
            ],
            0.5,
        );
        assert!(color.r() > 0);
        assert!(color.r() < 255);
    }

    #[test]
    fn canva_choice_chips_keeps_selected_index_when_not_clicked() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let mut selected = 1;
        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let mut components = ui.components();
                assert!(!components.canva_choice_chips(
                    Id::new("chips"),
                    &mut selected,
                    &["One", "Two", "Three"],
                ));
            });
        });
        assert_eq!(selected, 1);
    }
}
