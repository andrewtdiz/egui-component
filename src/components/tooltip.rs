use super::{api::ComponentUi, Button, ButtonStyle, LabelTone};
use crate::ui::tokens;
use egui::{CursorIcon, Response, Shadow};

#[derive(Debug, Clone, Copy)]
pub struct Tooltip<'a> {
    pub trigger_label: &'a str,
    pub text: &'a str,
    pub width: f32,
    pub delay_ms: u32,
    pub top_center: bool,
}

impl<'a> Tooltip<'a> {
    pub fn new(trigger_label: &'a str, text: &'a str) -> Self {
        Self {
            trigger_label,
            text,
            width: 220.0,
            delay_ms: 0,
            top_center: true,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn delay_ms(mut self, delay_ms: u32) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    pub fn top_center(mut self, top_center: bool) -> Self {
        self.top_center = top_center;
        self
    }
}

impl<'a> From<(&'a str, &'a str)> for Tooltip<'a> {
    fn from((trigger_label, text): (&'a str, &'a str)) -> Self {
        Self::new(trigger_label, text)
    }
}

impl<'a> From<(&'a str, &'a str, f32)> for Tooltip<'a> {
    fn from((trigger_label, text, width): (&'a str, &'a str, f32)) -> Self {
        Self::new(trigger_label, text).width(width)
    }
}

impl<'a> From<(&'a str, &'a str, f32, bool)> for Tooltip<'a> {
    fn from((trigger_label, text, width, top_center): (&'a str, &'a str, f32, bool)) -> Self {
        Self::new(trigger_label, text)
            .width(width)
            .top_center(top_center)
    }
}

impl ComponentUi<'_> {
    pub fn tooltip<'a>(&mut self, props: impl Into<Tooltip<'a>>) -> Response {
        let props = props.into();
        let overrides = self.overrides;
        self.raw_mut()
            .scope(|ui| {
                let dark_mode = ui.visuals().dark_mode;
                ui.style_mut().visuals.popup_shadow = Shadow {
                    offset: [2, 4],
                    blur: 4,
                    spread: 0,
                    color: tokens::tooltip_shadow(dark_mode),
                };

                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let response = ui
                    .button(
                        Button::from((props.trigger_label, ButtonStyle::Secondary))
                            .min_size(egui::vec2(props.width, ui.spacing().interact_size.y)),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand);

                let hover_started_id = response.id.with("tooltip_hover_started_at");
                let now = response.ctx.input(|input| input.time);
                let delay_secs = props.delay_ms as f64 / 1000.0;

                let show_tooltip = if response.enabled() && response.hovered() {
                    let hover_started_at = response.ctx.data_mut(|data| {
                        if let Some(hover_started_at) = data.get_temp::<f64>(hover_started_id) {
                            hover_started_at
                        } else {
                            data.insert_temp(hover_started_id, now);
                            now
                        }
                    });

                    let hovered_for = now - hover_started_at;
                    let should_show = hovered_for >= delay_secs;

                    if !should_show {
                        response
                            .ctx
                            .request_repaint_after_secs((delay_secs - hovered_for) as f32);
                    }

                    should_show
                } else {
                    response
                        .ctx
                        .data_mut(|data| data.remove::<f64>(hover_started_id));
                    false
                };

                if show_tooltip {
                    let mut tooltip = egui::Tooltip::for_widget(&response).gap(6.0);
                    if props.top_center {
                        tooltip.popup = tooltip
                            .popup
                            .align(egui::RectAlign::TOP)
                            .align_alternatives(&[egui::RectAlign::TOP]);
                    }

                    let _ = tooltip.show(|ui| {
                        let mut ui = ComponentUi::new(ui);
                        let _ = ui.label((props.text, LabelTone::Secondary));
                    });
                }

                response
            })
            .inner
    }
}
