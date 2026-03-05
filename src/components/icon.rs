use crate::ui::{icons, tokens};
use egui::{Color32, Response, Sense, Ui};

#[derive(Debug, Clone, Copy)]
pub struct IconProps<'a> {
    pub name: &'a str,
    pub size: f32,
    pub tint: Color32,
}

impl<'a> IconProps<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            size: 14.0,
            tint: tokens::TEXT_SECONDARY,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(1.0);
        self
    }

    pub fn tint(mut self, tint: Color32) -> Self {
        self.tint = tint;
        self
    }
}

pub fn icon(ui: &mut Ui, props: IconProps<'_>) -> Response {
    if let Some(image) = icons::image(ui.ctx(), props.name, props.size) {
        ui.add(image.tint(props.tint))
    } else {
        let (_rect, response) =
            ui.allocate_exact_size(egui::vec2(props.size, props.size), Sense::hover());
        response
    }
}
