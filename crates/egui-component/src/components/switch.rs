use super::{api::ComponentUi, common::ControlSize};
use crate::ui::tokens;
use egui::{Align, CornerRadius, Id, Layout, Response, Stroke, StrokeKind, Ui};

const SWITCH_LABEL_GAP: f32 = 10.0;

#[derive(Debug, Clone)]
pub struct Switch<'a> {
    pub id: Option<Id>,
    pub label: Option<&'a str>,
    pub size: ControlSize,
}

impl<'a> Switch<'a> {
    pub fn new() -> Self {
        Self {
            id: None,
            label: None,
            size: ControlSize::Md,
        }
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = ControlSize::Sm;
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

impl ComponentUi<'_> {
    pub fn switch<'a>(&mut self, value: &mut bool, props: impl Into<Switch<'a>>) -> Response {
        draw_switch(self.ui_mut(), value, props.into())
    }
}

fn draw_switch(ui: &mut Ui, value: &mut bool, props: Switch<'_>) -> Response {
    if let Some(id) = props.id.clone() {
        return ui
            .push_id(id, |ui| draw_switch_inner(ui, value, props))
            .inner;
    }

    draw_switch_inner(ui, value, props)
}

fn draw_switch_inner(ui: &mut Ui, value: &mut bool, props: Switch<'_>) -> Response {
    match props.label {
        Some(label) => {
            let control_slot_width = switch_metrics(props.size).width;
            let label_slot_width =
                (ui.available_width() - control_slot_width - SWITCH_LABEL_GAP).max(0.0);
            ui.scope(|ui| {
                ui.spacing_mut().item_spacing.x = SWITCH_LABEL_GAP;
                ui.horizontal(|ui| {
                    let label_response = ui
                        .scope(|ui| {
                            ui.set_min_width(label_slot_width);
                            ui.set_max_width(label_slot_width);
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(label).size(12.0).color(
                                        tokens::text_primary(crate::theme::runtime_for_ui(ui)),
                                    ),
                                )
                                .selectable(false)
                                .wrap(),
                            )
                        })
                        .inner;
                    let switch_response = ui
                        .allocate_ui_with_layout(
                            egui::vec2(control_slot_width, 0.0),
                            Layout::top_down(Align::Min),
                            |ui| draw_switch_control(ui, value, props.size),
                        )
                        .inner;
                    label_response.union(switch_response)
                })
                .inner
            })
            .inner
        }
        None => draw_switch_control(ui, value, props.size),
    }
}

fn draw_switch_control(ui: &mut Ui, value: &mut bool, size: ControlSize) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let metrics = switch_metrics(size);
    let desired_size = egui::vec2(metrics.width, metrics.height);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }

    let t = ui.ctx().animate_bool(response.id, *value);
    let on_fill = tokens::primary_bg(runtime);
    let fill = tokens::switch_off_bg(runtime).lerp_to_gamma(on_fill, t);
    ui.painter().rect(
        rect,
        CornerRadius::same(metrics.corner_radius),
        fill,
        Stroke::NONE,
        StrokeKind::Outside,
    );

    let knob_x = egui::lerp(
        (rect.left() + metrics.knob_inset)..=(rect.right() - metrics.knob_inset),
        t,
    );
    let knob_color = tokens::switch_knob_off(runtime).lerp_to_gamma(tokens::primary_fg(runtime), t);
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

fn switch_metrics(size: ControlSize) -> SwitchMetrics {
    match size {
        ControlSize::Md => SwitchMetrics {
            width: 42.0,
            height: 24.0,
            corner_radius: 12,
            knob_radius: 8.6,
            knob_inset: 11.2,
        },
        ControlSize::Sm => SwitchMetrics {
            width: 34.0,
            height: 20.0,
            corner_radius: 10,
            knob_radius: 7.0,
            knob_inset: 9.2,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::Switch;
    use crate::runtime_components::{ComponentUiExt, ControlSize};
    use egui::{Align, CentralPanel, Context, Layout, Pos2, RawInput, Response};

    #[test]
    fn compact_labeled_switch_stays_within_preview_width() {
        let context = Context::default();
        let mut row_response = None;
        let mut slot_right = 0.0;
        let mut value = false;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let origin: Pos2 = ui.next_widget_position();
                slot_right = origin.x + 280.0;
                row_response = Some(
                    ui.scope(|ui| {
                        ui.set_min_width(280.0);
                        ui.set_max_width(280.0);
                        ui.with_layout(Layout::top_down(Align::Min), |ui| {
                            ui.components().switch(
                                &mut value,
                                Switch::new()
                                    .label("Use Compact Handles")
                                    .size(ControlSize::Sm),
                            )
                        })
                        .inner
                    })
                    .inner,
                );
            });
        });

        let row_response: Response = row_response.expect("switch row should render");
        assert!(row_response.rect.right() <= slot_right + 0.5);
    }
}
