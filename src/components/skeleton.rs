use super::api::ComponentUi;
use crate::ui::tokens;
use egui::{CornerRadius, Response, Sense, StrokeKind, Ui, Vec2};

const SKELETON_PULSE_DURATION_SECS: f32 = 2.0;
const SKELETON_MIN_OPACITY: f32 = 0.5;
const SKELETON_EASING_X1: f32 = 0.4;
const SKELETON_EASING_Y1: f32 = 0.0;
const SKELETON_EASING_X2: f32 = 0.6;
const SKELETON_EASING_Y2: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
pub struct Skeleton {
    pub size: Vec2,
    pub corner_radius: Option<u8>,
    pub animated: bool,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            size: egui::vec2(120.0, 16.0),
            corner_radius: None,
            animated: true,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.size.x = width.max(1.0);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.size.y = height.max(1.0);
        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size.x = size.x.max(1.0);
        self.size.y = size.y.max(1.0);
        self
    }

    pub fn corner_radius(mut self, corner_radius: u8) -> Self {
        self.corner_radius = Some(corner_radius);
        self
    }

    pub fn circle(mut self, diameter: f32) -> Self {
        let diameter = diameter.max(1.0);
        self.size = egui::vec2(diameter, diameter);
        self.corner_radius = Some((diameter * 0.5).round().clamp(0.0, 255.0) as u8);
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl From<()> for Skeleton {
    fn from(_: ()) -> Self {
        Self::new()
    }
}

impl From<(f32, f32)> for Skeleton {
    fn from((width, height): (f32, f32)) -> Self {
        Self::new().width(width).height(height)
    }
}

impl ComponentUi<'_> {
    pub fn skeleton(&mut self, props: impl Into<Skeleton>) -> Response {
        draw_skeleton(self.ui_mut(), props.into())
    }
}

fn draw_skeleton(ui: &mut Ui, props: Skeleton) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let (rect, response) = ui.allocate_exact_size(props.size, Sense::hover());
    let base = tokens::muted_surface(runtime);
    let fill = if props.animated {
        let phase = (ui.input(|input| input.time) as f32 / SKELETON_PULSE_DURATION_SECS).fract();
        let segment_progress = if phase < 0.5 {
            phase * 2.0
        } else {
            (phase - 0.5) * 2.0
        };
        let eased = cubic_bezier_ease(
            segment_progress,
            SKELETON_EASING_X1,
            SKELETON_EASING_Y1,
            SKELETON_EASING_X2,
            SKELETON_EASING_Y2,
        );
        let opacity = if phase < 0.5 {
            1.0 - ((1.0 - SKELETON_MIN_OPACITY) * eased)
        } else {
            SKELETON_MIN_OPACITY + ((1.0 - SKELETON_MIN_OPACITY) * eased)
        };
        ui.ctx().request_repaint_after_secs(1.0 / 30.0);
        base.gamma_multiply(opacity)
    } else {
        base
    };

    ui.painter().rect(
        rect,
        CornerRadius::same(props.corner_radius.unwrap_or(tokens::radius_md(runtime))),
        fill,
        egui::Stroke::NONE,
        StrokeKind::Outside,
    );

    response
}

fn cubic_bezier_ease(progress: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    let mut parameter = progress;

    for _ in 0..5 {
        let slope = cubic_bezier_slope(parameter, x1, x2);
        if slope.abs() < 1e-5 {
            break;
        }

        let error = cubic_bezier_axis(parameter, x1, x2) - progress;
        parameter = (parameter - (error / slope)).clamp(0.0, 1.0);
    }

    cubic_bezier_axis(parameter, y1, y2)
}

fn cubic_bezier_axis(parameter: f32, p1: f32, p2: f32) -> f32 {
    let inverse = 1.0 - parameter;
    (3.0 * inverse * inverse * parameter * p1)
        + (3.0 * inverse * parameter * parameter * p2)
        + (parameter * parameter * parameter)
}

fn cubic_bezier_slope(parameter: f32, p1: f32, p2: f32) -> f32 {
    let inverse = 1.0 - parameter;
    (3.0 * inverse * inverse * p1)
        + (6.0 * inverse * parameter * (p2 - p1))
        + (3.0 * parameter * parameter * (1.0 - p2))
}
