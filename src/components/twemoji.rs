use super::api::ComponentUi;
use crate::ui::twemoji;
use egui::{vec2, Response, Sense, Ui};

#[derive(Debug, Clone, Copy)]
pub struct Twemoji<'a> {
    pub emoji: &'a str,
    pub size: f32,
    pub sense: Sense,
}

impl<'a> Twemoji<'a> {
    pub fn new(emoji: &'a str) -> Self {
        Self {
            emoji,
            size: 18.0,
            sense: Sense::hover(),
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(1.0);
        self
    }

    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }
}

impl<'a> From<&'a str> for Twemoji<'a> {
    fn from(emoji: &'a str) -> Self {
        Self::new(emoji)
    }
}

impl ComponentUi<'_> {
    pub fn twemoji<'a>(&mut self, props: impl Into<Twemoji<'a>>) -> Response {
        draw_twemoji(self.ui_mut(), props.into())
    }
}

fn draw_twemoji(ui: &mut Ui, props: Twemoji<'_>) -> Response {
    if let Some(image) = twemoji::image(props.emoji, props.size) {
        ui.add(image.sense(props.sense))
    } else {
        let (_rect, response) = ui.allocate_exact_size(vec2(props.size, props.size), props.sense);
        response.on_hover_text("Unsupported Twemoji asset")
    }
}

#[cfg(test)]
mod tests {
    use super::Twemoji;
    use crate::components::ComponentUiExt;
    use crate::theme::{self, ThemeMode};
    use egui::{vec2, Align, CentralPanel, Layout, RawInput, Rect, Sense};

    #[test]
    fn renders_supported_twemoji_without_panic() {
        let context = egui::Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);

        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = ui.allocate_ui_with_layout(
                    vec2(160.0, 120.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        rect = ui.components().twemoji(Twemoji::new("👩‍💻").size(32.0)).rect;
                    },
                );
            });
        });

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
    }

    #[test]
    fn unsupported_twemoji_still_reserves_space() {
        let context = egui::Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);

        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = ui.allocate_ui_with_layout(
                    vec2(160.0, 120.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        rect = ui
                            .components()
                            .twemoji(
                                Twemoji::new("not-an-emoji")
                                    .size(28.0)
                                    .sense(Sense::click()),
                            )
                            .rect;
                    },
                );
            });
        });

        assert_eq!(rect.size(), vec2(28.0, 28.0));
    }
}
