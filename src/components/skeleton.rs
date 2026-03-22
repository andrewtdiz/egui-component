use super::api::ComponentUi;
use crate::ui::tokens;
use egui::{CornerRadius, Response, Sense, StrokeKind, Ui, Vec2};

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
    let highlight = tokens::card_background(runtime);
    let fill = if props.animated {
        let pulse = ((ui.input(|input| input.time) as f32 * 2.6).sin() + 1.0) * 0.5;
        ui.ctx().request_repaint_after_secs(1.0 / 30.0);
        base.lerp_to_gamma(highlight, 0.12 + (pulse * 0.16))
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
