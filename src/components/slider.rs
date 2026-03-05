use crate::components::chrome::{with_input_chrome, with_slider_chrome};
use crate::ui::tokens;
use egui::{Align2, Color32, CursorIcon, FontFamily, FontId, Id, Response, Ui};
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub struct SliderProps {
    pub width: f32,
    pub range: RangeInclusive<f32>,
}

impl SliderProps {
    pub fn new(range: RangeInclusive<f32>) -> Self {
        Self {
            width: 156.0,
            range,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

pub fn slider(ui: &mut Ui, value: &mut f32, props: SliderProps) -> Response {
    with_slider_chrome(ui, |ui| {
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = 24.0;
            ui.add_sized(
                [props.width, ui.spacing().interact_size.y],
                egui::Slider::new(value, props.range).show_value(false),
            )
        })
        .inner
    })
}

#[derive(Debug, Clone)]
pub struct NumberInputProps {
    pub id: Id,
    pub width: f32,
    pub range: RangeInclusive<f32>,
    pub speed: f64,
    pub decimals: usize,
    pub prefix: Option<String>,
    pub prefix_tint: Color32,
    pub prefix_align_left: bool,
}

impl NumberInputProps {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            width: 58.0,
            range: 0.0..=100.0,
            speed: 0.2,
            decimals: 1,
            prefix: None,
            prefix_tint: tokens::TEXT_SECONDARY,
            prefix_align_left: false,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn prefix_tint(mut self, tint: Color32) -> Self {
        self.prefix_tint = tint;
        self
    }

    pub fn prefix_align_left(mut self) -> Self {
        self.prefix_align_left = true;
        self
    }
}

pub fn number_input(ui: &mut Ui, value: &mut f32, props: NumberInputProps) -> Response {
    with_input_chrome(ui, |ui| {
        ui.push_id(props.id, |ui| {
            let mut drag_value = egui::DragValue::new(value)
                .range(props.range)
                .speed(props.speed)
                .fixed_decimals(props.decimals);
            if let Some(prefix) = props.prefix.as_deref().filter(|_| !props.prefix_align_left) {
                drag_value = drag_value.prefix(format!("{prefix} "));
            }
            let response = ui
                .add_sized([props.width, ui.spacing().interact_size.y], drag_value)
                .on_hover_cursor(CursorIcon::Text);

            if props.prefix_align_left {
                if let Some(prefix) = props.prefix.as_deref() {
                    ui.painter().text(
                        egui::pos2(response.rect.left() + 10.0, response.rect.center().y),
                        Align2::LEFT_CENTER,
                        prefix,
                        FontId::new(12.0, FontFamily::Proportional),
                        props.prefix_tint,
                    );
                }
            }

            response
        })
        .inner
    })
}
