use std::ops::RangeInclusive;

use super::api::ComponentUi;
use egui::{Color32, Id, Pos2, Rect, Response, Sense, Stroke};

#[derive(Debug, Clone, Copy)]
pub enum ColorStripKind {
    HueDelta { base_color: Color32 },
    SaturationDelta { base_color: Color32 },
    ValueDelta { base_color: Color32 },
    AlphaDelta { base_color: Color32 },
}

#[derive(Debug, Clone)]
pub struct ColorStrip {
    pub kind: ColorStripKind,
    pub range: RangeInclusive<f32>,
    pub id: Option<Id>,
    pub width: f32,
    pub height: f32,
}

impl ColorStrip {
    pub fn new(kind: ColorStripKind, range: RangeInclusive<f32>) -> Self {
        Self {
            kind,
            range,
            id: None,
            width: 220.0,
            height: 20.0,
        }
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(1.0);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(1.0);
        self
    }
}

impl ComponentUi<'_> {
    pub fn color_strip(&mut self, value: &mut f32, props: ColorStrip) -> Response {
        draw_color_strip(self.ui_mut(), value, props)
    }
}

fn draw_color_strip(ui: &mut egui::Ui, value: &mut f32, props: ColorStrip) -> Response {
    let id = props.id.unwrap_or_else(|| ui.next_auto_id());
    let desired_size = egui::vec2(props.width, props.height);
    let (rect, _) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
    let response = ui.interact(rect, id, Sense::click_and_drag());
    paint_strip(ui.painter(), rect, props.kind);

    if let Some(pointer) = response
        .interact_pointer_pos()
        .filter(|_| response.clicked() || response.dragged())
    {
        *value = value_for_pointer(pointer, rect, &props.range);
    }

    paint_indicator(
        ui.painter(),
        rect,
        t_for_value(*value, &props.range),
        ui.visuals().selection.stroke,
    );
    response
}

fn value_for_pointer(pointer: Pos2, rect: Rect, range: &RangeInclusive<f32>) -> f32 {
    let t = if rect.width() <= 0.0 {
        0.0
    } else {
        ((pointer.x - rect.left()) / rect.width()).clamp(0.0, 1.0)
    };
    let start = *range.start();
    let end = *range.end();
    start + (end - start) * t
}

fn t_for_value(value: f32, range: &RangeInclusive<f32>) -> f32 {
    let start = *range.start();
    let end = *range.end();
    let span = end - start;
    if span.abs() <= f32::EPSILON {
        0.0
    } else {
        ((value - start) / span).clamp(0.0, 1.0)
    }
}

fn paint_indicator(painter: &egui::Painter, rect: Rect, t: f32, stroke: Stroke) {
    let x = rect.left() + rect.width() * t;
    let indicator_rect = Rect::from_center_size(
        egui::pos2(x, rect.center().y),
        egui::vec2(8.0, rect.height() + 4.0),
    );
    painter.rect_stroke(
        indicator_rect,
        egui::CornerRadius::same(4),
        stroke,
        egui::StrokeKind::Inside,
    );
}

fn paint_strip(painter: &egui::Painter, rect: Rect, kind: ColorStripKind) {
    let steps = 48;
    let step_width = rect.width() / steps as f32;
    for index in 0..steps {
        let left = rect.left() + index as f32 * step_width;
        let right = if index + 1 == steps {
            rect.right()
        } else {
            left + step_width
        };
        let t = if steps <= 1 {
            0.0
        } else {
            index as f32 / (steps - 1) as f32
        };
        painter.rect_filled(
            Rect::from_min_max(
                egui::pos2(left, rect.top()),
                egui::pos2(right, rect.bottom()),
            ),
            0.0,
            strip_color(kind, t),
        );
    }
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(6),
        Stroke::new(
            1.0,
            painter
                .ctx()
                .style()
                .visuals
                .widgets
                .inactive
                .bg_stroke
                .color,
        ),
        egui::StrokeKind::Inside,
    );
}

fn strip_color(kind: ColorStripKind, t: f32) -> Color32 {
    match kind {
        ColorStripKind::HueDelta { .. } => Color32::from(egui::ecolor::Hsva::new(t, 1.0, 1.0, 1.0)),
        ColorStripKind::SaturationDelta { base_color } => {
            let mut hsva = egui::ecolor::Hsva::from(base_color);
            hsva.s = t;
            Color32::from(hsva)
        }
        ColorStripKind::ValueDelta { base_color } => {
            let mut hsva = egui::ecolor::Hsva::from(base_color);
            hsva.v = t;
            Color32::from(hsva)
        }
        ColorStripKind::AlphaDelta { base_color } => {
            let mut hsva = egui::ecolor::Hsva::from(base_color);
            hsva.a = t;
            Color32::from(hsva)
        }
    }
}
