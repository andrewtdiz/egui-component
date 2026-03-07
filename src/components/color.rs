use super::api::ComponentUi;
use egui::{Color32, Rect, Response, Sense, Stroke, Ui};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub fill: Color32,
    pub size: f32,
    pub stroke: Stroke,
}

impl Color {
    pub fn new(fill: Color32) -> Self {
        Self {
            fill,
            size: 20.0,
            stroke: Stroke::NONE,
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
}

impl From<Color32> for Color {
    fn from(fill: Color32) -> Self {
        Self::new(fill)
    }
}

impl ComponentUi<'_> {
    pub fn color(&mut self, props: impl Into<Color>) -> Response {
        draw_color(self.raw_mut(), props.into())
    }
}

fn draw_color(ui: &mut Ui, props: Color) -> Response {
    let size = props.size.max(1.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
    paint_color(ui.painter(), rect, props);
    response
}

pub(crate) fn paint_color(painter: &egui::Painter, rect: Rect, props: Color) {
    let diameter = props.size.min(rect.width()).min(rect.height()).max(1.0);
    let stroke_width = props.stroke.width.max(0.0);
    let radius = (diameter * 0.5 - stroke_width * 0.5).max(0.0);
    painter.circle(rect.center(), radius, props.fill, props.stroke);
}

#[cfg(test)]
mod tests {
    use super::Color;
    use crate::components::ComponentUiExt;
    use egui::{CentralPanel, Color32, Context, RawInput, Rect, Stroke};

    #[test]
    fn renders_color_with_optional_border() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = ui
                    .components()
                    .color(
                        Color::new(Color32::from_rgb(34, 197, 94))
                            .stroke(Stroke::new(1.0, Color32::BLACK)),
                    )
                    .rect;
            });
        });

        assert_eq!(rect.width(), 20.0);
        assert_eq!(rect.height(), 20.0);
    }
}
