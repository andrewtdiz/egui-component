use super::api::{ComponentOverride, ComponentOverrides, ComponentUi};
use crate::primitives::surface::{surface_frame, SurfaceFrame};
use crate::ui::tokens;
use egui::{Color32, Stroke, Ui};

#[derive(Debug, Clone, Copy)]
pub struct Card {
    pub fill: Option<Color32>,
    pub stroke: Option<Stroke>,
    pub corner_radius: u8,
    pub padding_x: i8,
    pub padding_y: i8,
}

impl Card {
    pub fn new() -> Self {
        Self {
            fill: None,
            stroke: None,
            corner_radius: tokens::RADIUS_LG,
            padding_x: 12,
            padding_y: 12,
        }
    }

    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn corner_radius(mut self, corner_radius: u8) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn padding(mut self, x: i8, y: i8) -> Self {
        self.padding_x = x;
        self.padding_y = y;
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CardOverride {
    pub fill: Option<Color32>,
    pub stroke: Option<Stroke>,
    pub corner_radius: Option<u8>,
    pub padding_x: Option<i8>,
    pub padding_y: Option<i8>,
}

impl CardOverride {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn corner_radius(mut self, corner_radius: u8) -> Self {
        self.corner_radius = Some(corner_radius);
        self
    }

    pub fn padding(mut self, x: i8, y: i8) -> Self {
        self.padding_x = Some(x);
        self.padding_y = Some(y);
        self
    }

    fn apply(self, mut props: Card) -> Card {
        if let Some(fill) = self.fill {
            props.fill = Some(fill);
        }
        if let Some(stroke) = self.stroke {
            props.stroke = Some(stroke);
        }
        if let Some(corner_radius) = self.corner_radius {
            props.corner_radius = corner_radius;
        }
        if let Some(padding_x) = self.padding_x {
            props.padding_x = padding_x;
        }
        if let Some(padding_y) = self.padding_y {
            props.padding_y = padding_y;
        }
        props
    }
}

impl ComponentOverride for CardOverride {
    fn apply_to(self, overrides: &mut ComponentOverrides) {
        if let Some(fill) = self.fill {
            overrides.card.fill = Some(fill);
        }
        if let Some(stroke) = self.stroke {
            overrides.card.stroke = Some(stroke);
        }
        if let Some(corner_radius) = self.corner_radius {
            overrides.card.corner_radius = Some(corner_radius);
        }
        if let Some(padding_x) = self.padding_x {
            overrides.card.padding_x = Some(padding_x);
        }
        if let Some(padding_y) = self.padding_y {
            overrides.card.padding_y = Some(padding_y);
        }
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl From<()> for Card {
    fn from(_: ()) -> Self {
        Self::new()
    }
}

impl From<Color32> for Card {
    fn from(fill: Color32) -> Self {
        Self::new().fill(fill)
    }
}

impl From<(Color32, Stroke)> for Card {
    fn from((fill, stroke): (Color32, Stroke)) -> Self {
        Self::new().fill(fill).stroke(stroke)
    }
}

impl ComponentUi<'_> {
    pub fn card<R>(
        &mut self,
        props: impl Into<Card>,
        add: impl FnOnce(&mut ComponentUi<'_>) -> R,
    ) -> egui::InnerResponse<R> {
        let overrides = self.overrides;
        let props = overrides.card.apply(props.into());
        draw_card(self.raw_mut(), props, |ui| {
            let mut components = ComponentUi::with_overrides(ui, overrides);
            add(&mut components)
        })
    }
}

fn draw_card<R>(
    ui: &mut Ui,
    props: Card,
    add: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    let dark_mode = ui.visuals().dark_mode;
    surface_frame(
        ui,
        SurfaceFrame::new(
            props.fill.unwrap_or(tokens::muted_surface(dark_mode)),
            props
                .stroke
                .unwrap_or(Stroke::new(1.0, tokens::separator(dark_mode))),
        )
        .corner_radius(props.corner_radius)
        .padding(props.padding_x, props.padding_y),
        add,
    )
}
