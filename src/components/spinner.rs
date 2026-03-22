use super::api::ComponentUi;
use crate::ui::tokens;
use egui::{Color32, Response, Sense, Stroke, Ui};

const SPINNER_SIZE: f32 = 16.0;
const SPINNER_STROKE_WIDTH: f32 = 2.0;
const SPINNER_ARC_SWEEP: f32 = std::f32::consts::TAU * 0.72;
const SPINNER_SEGMENTS: usize = 40;

#[derive(Debug, Clone, Copy)]
pub struct Spinner {
    pub size: f32,
    pub stroke_width: f32,
    pub color: Option<Color32>,
    pub speed: f32,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            size: SPINNER_SIZE,
            stroke_width: SPINNER_STROKE_WIDTH,
            color: None,
            speed: 0.9,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(1.0);
        self
    }

    pub fn stroke_width(mut self, stroke_width: f32) -> Self {
        self.stroke_width = stroke_width.max(1.0);
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed.max(0.0);
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl From<()> for Spinner {
    fn from(_: ()) -> Self {
        Self::new()
    }
}

impl From<f32> for Spinner {
    fn from(size: f32) -> Self {
        Self::new().size(size)
    }
}

impl ComponentUi<'_> {
    pub fn spinner(&mut self, props: impl Into<Spinner>) -> Response {
        draw_spinner(self.ui_mut(), props.into())
    }
}

fn draw_spinner(ui: &mut Ui, props: Spinner) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(props.size, props.size), Sense::hover());
    let radius = (rect.width().min(rect.height()) * 0.5) - props.stroke_width.max(1.0);

    if radius <= 0.0 {
        return response;
    }

    ui.ctx().request_repaint_after_secs(1.0 / 60.0);

    let color = props.color.unwrap_or(tokens::primary_bg(runtime));
    let track_color = tokens::input_border(runtime);
    let center = rect.center();
    let start_angle = (ui.input(|input| input.time) as f32) * props.speed * std::f32::consts::TAU;

    ui.painter()
        .circle_stroke(center, radius, Stroke::new(props.stroke_width, track_color));
    ui.painter().add(egui::Shape::line(
        spinner_arc_points(center, radius, start_angle, SPINNER_ARC_SWEEP),
        Stroke::new(props.stroke_width, color),
    ));

    response
}

fn spinner_arc_points(
    center: egui::Pos2,
    radius: f32,
    start_angle: f32,
    sweep: f32,
) -> Vec<egui::Pos2> {
    (0..=SPINNER_SEGMENTS)
        .map(|step| {
            let t = step as f32 / SPINNER_SEGMENTS as f32;
            let angle = start_angle + (t * sweep);
            egui::pos2(
                center.x + angle.cos() * radius,
                center.y + angle.sin() * radius,
            )
        })
        .collect()
}
