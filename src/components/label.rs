use super::api::{ComponentOverride, ComponentOverrides, ComponentUi};
use crate::ui::tokens;
use egui::{Color32, FontFamily, FontId, RichText, Ui};

const SEMIBOLD_FONT: &str = "component-showcase-geist-semibold";
const BOLD_FONT: &str = "component-showcase-geist-bold";

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

    let mut text = RichText::new(props.text).size(props.size).color(color);
    match props.weight {
        LabelWeight::Regular => {}
        LabelWeight::Semibold => {
            text = text.font(FontId::new(
                props.size,
                FontFamily::Name(SEMIBOLD_FONT.into()),
            ));
        }
        LabelWeight::Bold => {
            text = text.font(FontId::new(props.size, FontFamily::Name(BOLD_FONT.into())));
        }
    }

    ui.add(egui::Label::new(text).selectable(false))
}
