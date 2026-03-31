use super::{
    api::ComponentUi,
    common::{resolve_input_width, InputWidth},
};
use crate::primitives::control::{with_input_chrome, with_slider_chrome};
use crate::ui::{tokens, typography};
use egui::{Align2, Color32, CornerRadius, CursorIcon, Id, Rect, Response, Stroke, Ui};
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
        draw_slider(self.ui_mut(), value, props.into())
    }

    pub fn number_input(&mut self, value: &mut f32, props: impl Into<NumberInput>) -> Response {
        draw_number_input(self.ui_mut(), value, props.into())
    }
}

fn draw_slider(ui: &mut Ui, value: &mut f32, props: Slider) -> Response {
    with_slider_chrome(ui, |ui| {
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = 24.0;
            let response = ui.add_sized(
                [props.width, ui.spacing().interact_size.y],
                egui::Slider::new(value, props.range.clone()).show_value(false),
            );
            paint_slider(ui, response.rect, &response, *value, &props.range);
            response
        })
        .inner
    })
}

fn paint_slider(ui: &Ui, rect: Rect, response: &Response, value: f32, range: &RangeInclusive<f32>) {
    if !ui.is_rect_visible(rect) {
        return;
    }

    let runtime = crate::theme::runtime_for_ui(ui);
    let handle_travel_inset = rect.height() / 2.5;
    let thumb_radius = (handle_travel_inset - 0.8).max(6.0);
    let rail_height = 4.0;
    let rail_rect = Rect::from_min_max(
        egui::pos2(
            rect.left() + handle_travel_inset,
            rect.center().y - (rail_height * 0.5),
        ),
        egui::pos2(
            rect.right() - handle_travel_inset,
            rect.center().y + (rail_height * 0.5),
        ),
    );
    let rail_radius = CornerRadius::same((rail_height * 0.5).round() as u8);
    let normalized = normalized_slider_value(value, range);
    let thumb_x = egui::lerp(rail_rect.x_range(), normalized);
    let thumb_center = egui::pos2(thumb_x, rail_rect.center().y);

    ui.painter().rect_filled(
        rail_rect,
        rail_radius,
        tokens::slider_track_inactive(runtime),
    );

    let active_width = (thumb_x - rail_rect.left()).max(rail_height);
    let active_rect = Rect::from_min_size(
        rail_rect.min,
        egui::vec2(active_width.min(rail_rect.width()), rail_rect.height()),
    );
    ui.painter().rect_filled(
        active_rect,
        rail_radius,
        tokens::slider_track_active(runtime),
    );

    let thumb_pressed = response.is_pointer_button_down_on() || response.dragged();
    let thumb_hovered = response.hovered() || response.has_focus();
    let thumb_fill = if thumb_pressed {
        tokens::slider_thumb_active_fill(runtime)
    } else if thumb_hovered {
        tokens::slider_thumb_hover_fill(runtime)
    } else {
        tokens::slider_thumb_fill(runtime)
    };
    let thumb_border = if thumb_pressed {
        tokens::slider_thumb_active_border(runtime)
    } else if thumb_hovered {
        tokens::slider_thumb_hover_border(runtime)
    } else {
        tokens::slider_thumb_border(runtime)
    };
    let thumb_border_width = slider_thumb_border_width(thumb_hovered, thumb_pressed);
    ui.painter().circle(
        thumb_center,
        thumb_radius,
        thumb_fill,
        Stroke::new(thumb_border_width, thumb_border),
    );
}

fn slider_thumb_border_width(thumb_hovered: bool, thumb_pressed: bool) -> f32 {
    if thumb_hovered || thumb_pressed {
        1.5
    } else {
        1.0
    }
}

fn normalized_slider_value(value: f32, range: &RangeInclusive<f32>) -> f32 {
    let min = *range.start();
    let max = *range.end();
    let span = max - min;
    if span.abs() <= f32::EPSILON {
        0.0
    } else {
        ((value - min) / span).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct NumberInput {
    pub id: Id,
    pub width: f32,
    pub width_is_custom: bool,
    pub width_preset: Option<InputWidth>,
    pub range: RangeInclusive<f32>,
    pub speed: f64,
    pub fine_speed: Option<f64>,
    pub decimals: usize,
    pub fine_decimals: Option<usize>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub prefix_tint: Option<Color32>,
    pub prefix_align_left: bool,
    pub axis: NumberInputAxis,
}

impl NumberInput {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            width: 58.0,
            width_is_custom: false,
            width_preset: None,
            range: 0.0..=100.0,
            speed: 0.2,
            fine_speed: None,
            decimals: 1,
            fine_decimals: None,
            prefix: None,
            suffix: None,
            prefix_tint: None,
            prefix_align_left: false,
            axis: NumberInputAxis::Horizontal,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self.width_is_custom = true;
        self
    }

    pub fn width_preset(mut self, width_preset: InputWidth) -> Self {
        self.width_preset = Some(width_preset);
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

    pub fn fine_speed(mut self, fine_speed: f64) -> Self {
        self.fine_speed = Some(fine_speed.max(f64::EPSILON));
        self
    }

    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn fine_decimals(mut self, fine_decimals: usize) -> Self {
        self.fine_decimals = Some(fine_decimals);
        self
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    pub fn prefix_tint(mut self, tint: Color32) -> Self {
        self.prefix_tint = Some(tint);
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
        let width = resolve_input_width(
            props.width,
            props.width_is_custom,
            props.width_preset,
            ui.available_width(),
        );
        let runtime = crate::theme::runtime_for_ui(ui);
        let drag_value_text_style = ui.style().drag_value_text_style.clone();
        let drag_value_font = drag_value_text_style.resolve(ui.style());
        ui.style_mut().visuals.selection.stroke = tokens::input_focus_stroke(runtime);
        ui.spacing_mut().button_padding = crate::primitives::control::centered_input_button_padding(
            ui,
            &drag_value_font,
            f32::from(tokens::INPUT_PADDING_X),
        );
        ui.push_id(props.id, |ui| {
            let fine_adjustment = ui.input(|input| input.modifiers.shift);
            let decimals = if fine_adjustment {
                props.fine_decimals.unwrap_or(props.decimals)
            } else {
                props.decimals
            };
            let speed = if fine_adjustment {
                props.fine_speed.unwrap_or(props.speed)
            } else {
                props.speed
            };
            let mut drag_value = egui::DragValue::new(value)
                .range(props.range)
                .speed(speed)
                .fixed_decimals(decimals);
            if let Some(prefix) = props.prefix.as_deref().filter(|_| !props.prefix_align_left) {
                drag_value = drag_value.prefix(format!("{prefix} "));
            }
            if let Some(suffix) = props.suffix.as_deref() {
                drag_value = drag_value.suffix(format!(" {suffix}"));
            }
            let cursor_icon = match props.axis {
                NumberInputAxis::Horizontal => CursorIcon::ResizeHorizontal,
                NumberInputAxis::Vertical => CursorIcon::ResizeVertical,
            };
            let response = ui
                .add_sized([width, ui.spacing().interact_size.y], drag_value)
                .on_hover_cursor(cursor_icon);
            if response.dragged() {
                ui.ctx().set_cursor_icon(cursor_icon);
            }

            if props.prefix_align_left {
                if let Some(prefix) = props.prefix.as_deref() {
                    ui.painter().text(
                        egui::pos2(
                            response.rect.left() + f32::from(tokens::INPUT_PADDING_X),
                            response.rect.center().y,
                        ),
                        Align2::LEFT_CENTER,
                        prefix,
                        typography::label_font(),
                        props.prefix_tint.unwrap_or(tokens::text_secondary(runtime)),
                    );
                }
            }

            response
        })
        .inner
    })
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum NumberInputAxis {
    Horizontal,
    Vertical,
}

#[cfg(test)]
mod tests {
    use super::{normalized_slider_value, slider_thumb_border_width};

    #[test]
    fn slider_thumb_border_is_emphasized_on_hover() {
        assert_eq!(slider_thumb_border_width(false, false), 1.0);
        assert_eq!(slider_thumb_border_width(true, false), 1.5);
        assert_eq!(slider_thumb_border_width(false, true), 1.5);
    }

    #[test]
    fn normalized_slider_value_clamps_to_range() {
        assert_eq!(normalized_slider_value(-10.0, &(0.0..=100.0)), 0.0);
        assert_eq!(normalized_slider_value(50.0, &(0.0..=100.0)), 0.5);
        assert_eq!(normalized_slider_value(120.0, &(0.0..=100.0)), 1.0);
    }
}
