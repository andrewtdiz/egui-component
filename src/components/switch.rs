use super::api::ComponentUi;
use crate::ui::tokens;
use egui::{Align, CornerRadius, Layout, Response, Stroke, StrokeKind, Ui};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SwitchSize {
    Default,
    Small,
}

#[derive(Debug, Clone, Copy)]
pub struct Switch<'a> {
    pub label: Option<&'a str>,
    pub size: SwitchSize,
}

impl<'a> Switch<'a> {
    pub fn new() -> Self {
        Self {
            label: None,
            size: SwitchSize::Default,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = SwitchSize::Small;
        self
    }
}

impl<'a> Default for Switch<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<()> for Switch<'a> {
    fn from(_: ()) -> Self {
        Self::new()
    }
}

impl<'a> From<&'a str> for Switch<'a> {
    fn from(label: &'a str) -> Self {
        Self::new().label(label)
    }
}

impl<'a> From<SwitchSize> for Switch<'a> {
    fn from(size: SwitchSize) -> Self {
        Self::new().size(size)
    }
}

impl<'a> From<(&'a str, SwitchSize)> for Switch<'a> {
    fn from((label, size): (&'a str, SwitchSize)) -> Self {
        Self::new().label(label).size(size)
    }
}

impl ComponentUi<'_> {
    pub fn switch<'a>(&mut self, value: &mut bool, props: impl Into<Switch<'a>>) -> Response {
        draw_switch(self.raw_mut(), value, props.into())
    }
}

fn draw_switch(ui: &mut Ui, value: &mut bool, props: Switch<'_>) -> Response {
    match props.label {
        Some(label) => {
            ui.horizontal(|ui| {
                let label_response = ui.add(
                    egui::Label::new(
                        egui::RichText::new(label)
                            .size(12.0)
                            .color(tokens::TEXT_PRIMARY),
                    )
                    .selectable(false),
                );
                let switch_response = ui
                    .with_layout(Layout::right_to_left(Align::Center), |ui| {
                        draw_switch_control(ui, value, props.size)
                    })
                    .inner;
                label_response.union(switch_response)
            })
            .inner
        }
        None => draw_switch_control(ui, value, props.size),
    }
}

fn draw_switch_control(ui: &mut Ui, value: &mut bool, size: SwitchSize) -> Response {
    let dark_mode = ui.visuals().dark_mode;
    let metrics = match size {
        SwitchSize::Default => SwitchMetrics {
            width: 42.0,
            height: 24.0,
            corner_radius: 12,
            knob_radius: 8.6,
            knob_inset: 11.2,
        },
        SwitchSize::Small => SwitchMetrics {
            width: 34.0,
            height: 20.0,
            corner_radius: 10,
            knob_radius: 7.0,
            knob_inset: 9.2,
        },
    };
    let desired_size = egui::vec2(metrics.width, metrics.height);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }

    let t = ui.ctx().animate_bool(response.id, *value);
    let on_fill = tokens::primary_bg(dark_mode);
    let fill = tokens::SWITCH_OFF_BG.lerp_to_gamma(on_fill, t);
    let stroke = if response.has_focus() {
        tokens::input_focus_stroke(dark_mode)
    } else if response.hovered() {
        Stroke::new(1.0, tokens::INPUT_HOVER_BORDER)
    } else {
        Stroke::new(1.0, tokens::SWITCH_BORDER)
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(metrics.corner_radius),
        fill,
        stroke,
        StrokeKind::Outside,
    );

    let knob_x = egui::lerp(
        (rect.left() + metrics.knob_inset)..=(rect.right() - metrics.knob_inset),
        t,
    );
    let knob_color = tokens::SWITCH_KNOB_OFF.lerp_to_gamma(tokens::primary_fg(dark_mode), t);
    ui.painter().circle_filled(
        egui::pos2(knob_x, rect.center().y),
        metrics.knob_radius,
        knob_color,
    );

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[derive(Debug, Clone, Copy)]
struct SwitchMetrics {
    width: f32,
    height: f32,
    corner_radius: u8,
    knob_radius: f32,
    knob_inset: f32,
}
