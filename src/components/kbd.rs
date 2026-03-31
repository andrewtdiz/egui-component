use super::api::{with_component_overrides, ComponentUi};
use crate::ui::tokens;
use egui::{Align2, Color32, CornerRadius, FontFamily, FontId, Sense, Stroke, StrokeKind, Ui};

#[derive(Debug, Clone, Copy)]
pub struct Kbd<'a> {
    pub text: &'a str,
    pub min_width: f32,
    pub height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub corner_radius: Option<u8>,
    pub fill: Option<Color32>,
    pub stroke: Option<Stroke>,
    pub text_color: Option<Color32>,
    pub text_size: f32,
}

impl<'a> Kbd<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            min_width: 20.0,
            height: 20.0,
            padding_x: 4.0,
            padding_y: 2.0,
            corner_radius: None,
            fill: None,
            stroke: None,
            text_color: None,
            text_size: 10.0,
        }
    }

    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = min_width.max(1.0);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(1.0);
        self
    }

    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.padding_x = x.max(0.0);
        self.padding_y = y.max(0.0);
        self
    }

    pub fn corner_radius(mut self, corner_radius: u8) -> Self {
        self.corner_radius = Some(corner_radius);
        self
    }

    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn text_color(mut self, text_color: Color32) -> Self {
        self.text_color = Some(text_color);
        self
    }

    pub fn text_size(mut self, text_size: f32) -> Self {
        self.text_size = text_size.max(1.0);
        self
    }
}

impl<'a> From<&'a str> for Kbd<'a> {
    fn from(text: &'a str) -> Self {
        Self::new(text)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KbdGroup {
    pub gap: f32,
}

impl KbdGroup {
    pub fn new() -> Self {
        Self { gap: 4.0 }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }
}

impl Default for KbdGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl From<()> for KbdGroup {
    fn from(_: ()) -> Self {
        Self::new()
    }
}

impl From<f32> for KbdGroup {
    fn from(gap: f32) -> Self {
        Self::new().gap(gap)
    }
}

impl ComponentUi<'_> {
    pub fn kbd<'a>(&mut self, props: impl Into<Kbd<'a>>) -> egui::Response {
        draw_kbd(self.ui_mut(), props.into())
    }

    pub fn kbd_group<R>(
        &mut self,
        props: impl Into<KbdGroup>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let overrides = self.overrides();
        draw_kbd_group(self.ui_mut(), props.into(), |ui| {
            with_component_overrides(ui, overrides, add_contents)
        })
    }
}

fn draw_kbd(ui: &mut Ui, props: Kbd<'_>) -> egui::Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let text_color = props.text_color.unwrap_or(tokens::text_muted(runtime));
    let font_id = FontId::new(props.text_size, FontFamily::Monospace);
    let text_width = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(props.text.to_owned(), font_id.clone(), text_color)
            .size()
            .x
    });
    let width = (text_width + (props.padding_x * 2.0)).max(props.min_width);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, props.height), Sense::hover());

    ui.painter().rect(
        rect,
        CornerRadius::same(props.corner_radius.unwrap_or(tokens::radius_sm(runtime))),
        props.fill.unwrap_or(tokens::input_background(runtime)),
        props
            .stroke
            .unwrap_or(Stroke::new(1.0, tokens::input_border(runtime))),
        StrokeKind::Outside,
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        props.text,
        font_id,
        text_color,
    );

    response
}

fn draw_kbd_group<R>(
    ui: &mut Ui,
    props: KbdGroup,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = props.gap;
        ui.horizontal(add_contents).inner
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{CentralPanel, Context, RawInput, Shape};

    fn collect_text_shapes(shape: &Shape, rendered_texts: &mut Vec<String>) {
        match shape {
            Shape::Text(text_shape) => {
                rendered_texts.push(text_shape.galley.job.text.clone());
            }
            Shape::Vec(shapes) => {
                for nested_shape in shapes {
                    collect_text_shapes(nested_shape, rendered_texts);
                }
            }
            _ => {}
        }
    }

    fn headless_rendered_texts(texts: &[&str]) -> Vec<String> {
        let context = Context::default();
        let frame_output = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = draw_kbd_group(ui, KbdGroup::new(), |ui| {
                    for text in texts {
                        let _ = draw_kbd(ui, Kbd::new(*text));
                    }
                });
            });
        });

        let mut rendered_texts = Vec::new();
        for clipped_shape in frame_output.shapes {
            collect_text_shapes(&clipped_shape.shape, &mut rendered_texts);
        }
        rendered_texts
    }

    #[test]
    fn headless_rendering_preserves_symbol_characters() {
        let expected = ["⌘", "⇧", "⌥", "⌃", "→", "Esc"];
        let rendered_texts = headless_rendered_texts(&expected);

        for expected_text in expected {
            assert!(
                rendered_texts.iter().any(|text| text == expected_text),
                "missing {expected_text:?}; rendered text shapes: {rendered_texts:?}"
            );
        }
    }

    #[test]
    fn honors_min_width_for_single_character_keycap() {
        let context = Context::default();
        let mut rendered_width = 0.0;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rendered_width = draw_kbd(ui, Kbd::new("A").min_width(36.0)).rect.width();
            });
        });

        assert!(
            rendered_width >= 36.0,
            "expected width >= 36.0, got {rendered_width}"
        );
    }
}
