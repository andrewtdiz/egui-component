use crate::ui::tokens;
use egui::{Align, CornerRadius, Layout, Response, Stroke, StrokeKind, Ui};

#[derive(Debug, Clone, Copy)]
pub struct SwitchProps<'a> {
    pub label: Option<&'a str>,
}

impl<'a> SwitchProps<'a> {
    pub fn new() -> Self {
        Self { label: None }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

pub fn switch(ui: &mut Ui, value: &mut bool, props: SwitchProps<'_>) -> Response {
    match props.label {
        Some(label) => {
            ui.horizontal(|ui| {
                let label_response = ui.label(
                    egui::RichText::new(label)
                        .size(12.0)
                        .color(tokens::TEXT_PRIMARY),
                );
                let switch_response = ui
                    .with_layout(Layout::right_to_left(Align::Center), |ui| {
                        draw_switch_control(ui, value)
                    })
                    .inner;
                label_response.union(switch_response)
            })
            .inner
        }
        None => draw_switch_control(ui, value),
    }
}

fn draw_switch_control(ui: &mut Ui, value: &mut bool) -> Response {
    let dark_mode = ui.visuals().dark_mode;
    let desired_size = egui::vec2(42.0, 24.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }

    let t = ui.ctx().animate_bool(response.id, *value);
    let on_fill = tokens::primary_bg(dark_mode);
    let fill = tokens::SWITCH_OFF_BG.lerp_to_gamma(on_fill, t);
    let stroke = Stroke::new(1.0, tokens::SWITCH_BORDER);
    ui.painter().rect(
        rect,
        CornerRadius::same(12),
        fill,
        stroke,
        StrokeKind::Outside,
    );

    let knob_radius = 7.5;
    let knob_x = egui::lerp((rect.left() + 12.0)..=(rect.right() - 12.0), t);
    let knob_color = tokens::SWITCH_KNOB_OFF.lerp_to_gamma(tokens::primary_fg(dark_mode), t);
    ui.painter()
        .circle_filled(egui::pos2(knob_x, rect.center().y), knob_radius, knob_color);

    response
}
