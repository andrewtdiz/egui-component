use super::api::ComponentUi;
use crate::components::chrome::{with_input_chrome, with_slider_chrome};
use crate::ui::tokens;
use egui::{Align2, Color32, CursorIcon, FontFamily, FontId, Id, Response, Ui};
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub struct Slider {
    pub width: f32,
    pub range: RangeInclusive<f32>,
}

impl Slider {
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

impl From<RangeInclusive<f32>> for Slider {
    fn from(range: RangeInclusive<f32>) -> Self {
        Self::new(range)
    }
}

impl From<(RangeInclusive<f32>, f32)> for Slider {
    fn from((range, width): (RangeInclusive<f32>, f32)) -> Self {
        Self::new(range).width(width)
    }
}

impl ComponentUi<'_> {
    pub fn slider(&mut self, value: &mut f32, props: impl Into<Slider>) -> Response {
        draw_slider(self.raw_mut(), value, props.into())
    }

    pub fn number_input(&mut self, value: &mut f32, props: impl Into<NumberInput>) -> Response {
        draw_number_input(self.raw_mut(), value, props.into())
    }
}

fn draw_slider(ui: &mut Ui, value: &mut f32, props: Slider) -> Response {
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
pub struct NumberInput {
    pub id: Id,
    pub width: f32,
    pub range: RangeInclusive<f32>,
    pub speed: f64,
    pub decimals: usize,
    pub prefix: Option<String>,
    pub prefix_tint: Color32,
    pub prefix_align_left: bool,
    pub axis: NumberInputAxis,
}

impl NumberInput {
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
            axis: NumberInputAxis::Horizontal,
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

    pub fn axis(mut self, axis: NumberInputAxis) -> Self {
        self.axis = axis;
        self
    }
}

impl From<Id> for NumberInput {
    fn from(id: Id) -> Self {
        Self::new(id)
    }
}

impl From<(Id, RangeInclusive<f32>)> for NumberInput {
    fn from((id, range): (Id, RangeInclusive<f32>)) -> Self {
        Self::new(id).range(range)
    }
}

impl From<(Id, f32)> for NumberInput {
    fn from((id, width): (Id, f32)) -> Self {
        Self::new(id).width(width)
    }
}

fn draw_number_input(ui: &mut Ui, value: &mut f32, props: NumberInput) -> Response {
    with_input_chrome(ui, |ui| {
        let dark_mode = ui.visuals().dark_mode;
        ui.style_mut().visuals.selection.stroke = tokens::input_focus_stroke(dark_mode);
        ui.push_id(props.id, |ui| {
            let mut drag_value = egui::DragValue::new(value)
                .range(props.range)
                .speed(props.speed)
                .fixed_decimals(props.decimals);
            if let Some(prefix) = props.prefix.as_deref().filter(|_| !props.prefix_align_left) {
                drag_value = drag_value.prefix(format!("{prefix} "));
            }
            let cursor_icon = match props.axis {
                NumberInputAxis::Horizontal => CursorIcon::ResizeHorizontal,
                NumberInputAxis::Vertical => CursorIcon::ResizeVertical,
            };
            let response = ui
                .add_sized([props.width, ui.spacing().interact_size.y], drag_value)
                .on_hover_cursor(cursor_icon);
            if response.dragged() {
                ui.ctx().set_cursor_icon(cursor_icon);
            }

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NumberInputAxis {
    Horizontal,
    Vertical,
}
