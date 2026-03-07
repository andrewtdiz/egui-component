use super::api::ComponentUi;
use crate::ui::{icons, tokens};
use egui::{Color32, Response, Sense, Ui};

#[derive(Debug, Clone, Copy)]
pub struct Icon<'a> {
    pub name: &'a str,
    pub size: f32,
    pub tint: Option<Color32>,
}

impl<'a> Icon<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            size: 14.0,
            tint: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(1.0);
        self
    }

    pub fn tint(mut self, tint: Color32) -> Self {
        self.tint = Some(tint);
        self
    }
}

impl<'a> From<&'a str> for Icon<'a> {
    fn from(name: &'a str) -> Self {
        Self::new(name)
    }
}

impl<'a> From<(&'a str, f32)> for Icon<'a> {
    fn from((name, size): (&'a str, f32)) -> Self {
        Self::new(name).size(size)
    }
}

impl<'a> From<(&'a str, f32, Color32)> for Icon<'a> {
    fn from((name, size, tint): (&'a str, f32, Color32)) -> Self {
        Self::new(name).size(size).tint(tint)
    }
}

impl ComponentUi<'_> {
    pub fn icon<'a>(&mut self, props: impl Into<Icon<'a>>) -> Response {
        draw_icon(self.raw_mut(), props.into())
    }
}

fn draw_icon(ui: &mut Ui, props: Icon<'_>) -> Response {
    if let Some(image) = icons::image(ui.ctx(), props.name, props.size) {
        let tint = props
            .tint
            .unwrap_or(tokens::text_secondary(ui.visuals().dark_mode));
        ui.add(image.tint(tint))
    } else {
        let (_rect, response) =
            ui.allocate_exact_size(egui::vec2(props.size, props.size), Sense::hover());
        response
    }
}
