use super::api::{ComponentOverride, ComponentOverrides, ComponentUi};
use crate::ui::{tokens, typography};
use egui::{Color32, RichText, Ui};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LabelTone {
    Primary,
    Secondary,
    Muted,
    Destructive,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LabelWeight {
    Regular,
    Semibold,
    Bold,
}

#[derive(Debug, Clone, Copy)]
pub struct Label<'a> {
    pub text: &'a str,
    pub tone: LabelTone,
    pub weight: LabelWeight,
    pub size: f32,
    pub color_override: Option<Color32>,
}

impl<'a> Label<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            tone: LabelTone::Primary,
            weight: LabelWeight::Regular,
            size: 12.0,
            color_override: None,
        }
    }

    pub fn tone(mut self, tone: LabelTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn weight(mut self, weight: LabelWeight) -> Self {
        self.weight = weight;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color_override = Some(color);
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LabelOverride {
    pub tone: Option<LabelTone>,
    pub weight: Option<LabelWeight>,
    pub size: Option<f32>,
    pub color_override: Option<Color32>,
}

impl LabelOverride {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tone(mut self, tone: LabelTone) -> Self {
        self.tone = Some(tone);
        self
    }

    pub fn weight(mut self, weight: LabelWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size.max(1.0));
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color_override = Some(color);
        self
    }

    fn apply<'a>(self, mut props: Label<'a>) -> Label<'a> {
        if let Some(tone) = self.tone {
            props.tone = tone;
        }
        if let Some(weight) = self.weight {
            props.weight = weight;
        }
        if let Some(size) = self.size {
            props.size = size;
        }
        if let Some(color) = self.color_override {
            props.color_override = Some(color);
        }
        props
    }
}

impl ComponentOverride for LabelOverride {
    fn apply_to(self, overrides: &mut ComponentOverrides) {
        if let Some(tone) = self.tone {
            overrides.label.tone = Some(tone);
        }
        if let Some(weight) = self.weight {
            overrides.label.weight = Some(weight);
        }
        if let Some(size) = self.size {
            overrides.label.size = Some(size);
        }
        if let Some(color) = self.color_override {
            overrides.label.color_override = Some(color);
        }
    }
}

impl<'a> From<&'a str> for Label<'a> {
    fn from(text: &'a str) -> Self {
        Self::new(text)
    }
}

impl<'a> From<(&'a str, LabelTone)> for Label<'a> {
    fn from((text, tone): (&'a str, LabelTone)) -> Self {
        Self::new(text).tone(tone)
    }
}

impl<'a> From<(&'a str, LabelTone, f32)> for Label<'a> {
    fn from((text, tone, size): (&'a str, LabelTone, f32)) -> Self {
        Self::new(text).tone(tone).size(size)
    }
}

impl<'a> From<(&'a str, LabelTone, LabelWeight)> for Label<'a> {
    fn from((text, tone, weight): (&'a str, LabelTone, LabelWeight)) -> Self {
        Self::new(text).tone(tone).weight(weight)
    }
}

impl ComponentUi<'_> {
    pub fn label<'a>(&mut self, props: impl Into<Label<'a>>) -> egui::Response {
        let props = self.overrides.label.apply(props.into());
        draw_label(self.raw_mut(), props)
    }
}

fn draw_label(ui: &mut Ui, props: Label<'_>) -> egui::Response {
    let dark_mode = ui.visuals().dark_mode;
    let color = props.color_override.unwrap_or(match props.tone {
        LabelTone::Primary => tokens::text_primary(dark_mode),
        LabelTone::Secondary => tokens::text_secondary(dark_mode),
        LabelTone::Muted => tokens::text_muted(dark_mode),
        LabelTone::Destructive => tokens::text_destructive(dark_mode),
    });

    let font_id = match props.weight {
        LabelWeight::Regular => typography::proportional(props.size),
        LabelWeight::Semibold => typography::semibold_font(props.size),
        LabelWeight::Bold => typography::bold_font(props.size),
    };
    let text = RichText::new(props.text).font(font_id).color(color);

    ui.add(egui::Label::new(text).selectable(false))
}

#[cfg(test)]
mod tests {
    use super::{draw_label, Label, LabelTone, LabelWeight};
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

    #[test]
    fn theme_setup_supports_weighted_labels_and_helper_text() {
        let context = Context::default();
        crate::theme::install(&context, crate::theme::ThemeMode::Dark);

        let frame_output = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = draw_label(
                    ui,
                    Label::new("Section label").weight(LabelWeight::Semibold),
                );
                let _ = draw_label(
                    ui,
                    Label::new("Helper copy").tone(LabelTone::Muted).size(12.0),
                );
            });
        });

        let mut rendered_texts = Vec::new();
        for clipped_shape in frame_output.shapes {
            collect_text_shapes(&clipped_shape.shape, &mut rendered_texts);
        }

        assert!(rendered_texts.iter().any(|text| text == "Section label"));
        assert!(rendered_texts.iter().any(|text| text == "Helper copy"));
    }
}
