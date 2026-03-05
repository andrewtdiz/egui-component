use crate::ui::tokens;
use egui::{Color32, FontFamily, FontId, RichText, Ui};

const SEMIBOLD_FONT: &str = "component-showcase-geist-semibold";

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
}

#[derive(Debug, Clone, Copy)]
pub struct LabelProps<'a> {
    pub text: &'a str,
    pub tone: LabelTone,
    pub weight: LabelWeight,
    pub size: f32,
    pub color_override: Option<Color32>,
}

impl<'a> LabelProps<'a> {
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

pub fn label(ui: &mut Ui, props: LabelProps<'_>) -> egui::Response {
    let color = props.color_override.unwrap_or(match props.tone {
        LabelTone::Primary => tokens::TEXT_PRIMARY,
        LabelTone::Secondary => tokens::TEXT_SECONDARY,
        LabelTone::Muted => tokens::TEXT_MUTED,
        LabelTone::Destructive => tokens::TEXT_DESTRUCTIVE,
    });

    let mut text = RichText::new(props.text).size(props.size).color(color);
    if props.weight == LabelWeight::Semibold {
        text = text.font(FontId::new(
            props.size,
            FontFamily::Name(SEMIBOLD_FONT.into()),
        ));
    }

    ui.label(text)
}
