use super::api::ComponentUi;
use egui::{Color32, CornerRadius, Response, Sense, Ui, Vec2};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Image<'a> {
    widget: egui::Image<'a>,
}

impl<'a> Image<'a> {
    pub fn new(source: impl Into<egui::ImageSource<'a>>) -> Self {
        Self {
            widget: egui::Image::new(source),
        }
    }

    pub fn from_uri(uri: impl Into<Cow<'a, str>>) -> Self {
        Self {
            widget: egui::Image::from_uri(uri),
        }
    }

    pub fn from_bytes(
        uri: impl Into<Cow<'static, str>>,
        bytes: impl Into<egui::load::Bytes>,
    ) -> Self {
        Self {
            widget: egui::Image::from_bytes(uri, bytes),
        }
    }

    pub fn max_size(mut self, size: Vec2) -> Self {
        self.widget = self.widget.max_size(size);
        self
    }

    pub fn fit_to_exact_size(mut self, size: Vec2) -> Self {
        self.widget = self.widget.fit_to_exact_size(size);
        self
    }

    pub fn fit_to_original_size(mut self, scale: f32) -> Self {
        self.widget = self.widget.fit_to_original_size(scale);
        self
    }

    pub fn shrink_to_fit(mut self) -> Self {
        self.widget = self.widget.shrink_to_fit();
        self
    }

    pub fn maintain_aspect_ratio(mut self, value: bool) -> Self {
        self.widget = self.widget.maintain_aspect_ratio(value);
        self
    }

    pub fn sense(mut self, sense: Sense) -> Self {
        self.widget = self.widget.sense(sense);
        self
    }

    pub fn bg_fill(mut self, fill: impl Into<Color32>) -> Self {
        self.widget = self.widget.bg_fill(fill);
        self
    }

    pub fn tint(mut self, tint: impl Into<Color32>) -> Self {
        self.widget = self.widget.tint(tint);
        self
    }

    pub fn rotate(mut self, angle_radians: f32, origin: Vec2) -> Self {
        self.widget = self.widget.rotate(angle_radians, origin);
        self
    }

    pub fn corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.widget = self.widget.corner_radius(corner_radius);
        self
    }
}

impl<'a> From<egui::ImageSource<'a>> for Image<'a> {
    fn from(source: egui::ImageSource<'a>) -> Self {
        Self::new(source)
    }
}

impl<'a> From<&'a str> for Image<'a> {
    fn from(uri: &'a str) -> Self {
        Self::new(uri)
    }
}

impl<'a> From<&'a String> for Image<'a> {
    fn from(uri: &'a String) -> Self {
        Self::new(uri)
    }
}

impl From<String> for Image<'static> {
    fn from(uri: String) -> Self {
        Self::new(uri)
    }
}

impl<T> From<(&'static str, T)> for Image<'static>
where
    T: Into<egui::load::Bytes>,
{
    fn from((uri, bytes): (&'static str, T)) -> Self {
        Self::from_bytes(uri, bytes)
    }
}

impl ComponentUi<'_> {
    pub fn image<'a>(&mut self, props: impl Into<Image<'a>>) -> Response {
        draw_image(self.ui_mut(), props.into())
    }
}

fn draw_image(ui: &mut Ui, props: Image<'_>) -> Response {
    ui.add(props.widget)
}

#[cfg(test)]
mod tests {
    use super::Image;
    use crate::components::ComponentUiExt;
    use egui::{vec2, Align, CentralPanel, Context, Layout, RawInput, Rect};

    const SHOWCASE_PNG_BYTES: &[u8] = include_bytes!("../../assets/images/showcase-image.png");

    #[test]
    fn decodes_png_bytes_when_image_loader_is_enabled() {
        let decoded =
            egui_extras::image::load_image_bytes(SHOWCASE_PNG_BYTES).expect("PNG decode failed");

        assert_eq!(decoded.size, [96, 64]);
    }

    #[test]
    fn renders_image_widget_without_panic() {
        let context = Context::default();
        egui_extras::install_image_loaders(&context);

        let image = Image::from_bytes("bytes://tests/showcase-image.png", SHOWCASE_PNG_BYTES)
            .fit_to_exact_size(vec2(96.0, 64.0));
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = ui.allocate_ui_with_layout(
                    vec2(160.0, 120.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        rect = ui.components().image(image.clone()).rect;
                    },
                );
            });
        });

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
    }

    #[test]
    fn renders_rotated_image_widget_without_panic() {
        let context = Context::default();
        egui_extras::install_image_loaders(&context);

        let image = Image::from_bytes("bytes://tests/showcase-image.png", SHOWCASE_PNG_BYTES)
            .fit_to_exact_size(vec2(96.0, 64.0))
            .rotate(0.35, vec2(0.5, 0.5));
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = ui.allocate_ui_with_layout(
                    vec2(160.0, 120.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        rect = ui.components().image(image.clone()).rect;
                    },
                );
            });
        });

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
    }
}
