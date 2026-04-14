use egui::{Color32, Rect, Response, Sense, Stroke, Ui};

#[derive(Debug, Clone, Copy)]
pub struct Swatch {
    pub fill: Color32,
    pub size: f32,
    pub stroke: Stroke,
    pub corner_radius: Option<u8>,
}

impl Swatch {
    pub fn new(fill: Color32) -> Self {
        Self {
            fill,
            size: 20.0,
            stroke: Stroke::NONE,
            corner_radius: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(1.0);
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    pub fn rounded(mut self, corner_radius: u8) -> Self {
        self.corner_radius = Some(corner_radius);
        self
    }
}

impl From<Color32> for Swatch {
    fn from(fill: Color32) -> Self {
        Self::new(fill)
    }
}

pub fn draw_swatch(ui: &mut Ui, props: Swatch) -> Response {
    let size = props.size.max(1.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
    paint_swatch(ui.painter(), rect, props);
    response
}

pub fn paint_swatch(painter: &egui::Painter, rect: Rect, props: Swatch) {
    if let Some(corner_radius) = props.corner_radius {
        painter.rect(
            rect,
            egui::CornerRadius::same(corner_radius),
            props.fill,
            props.stroke,
            egui::StrokeKind::Inside,
        );
        return;
    }

    let diameter = props.size.min(rect.width()).min(rect.height()).max(1.0);
    let stroke_width = props.stroke.width.max(0.0);
    let radius = (diameter * 0.5 - stroke_width * 0.5).max(0.0);
    painter.circle(rect.center(), radius, props.fill, props.stroke);
}
