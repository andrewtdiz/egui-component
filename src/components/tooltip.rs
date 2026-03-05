use super::{
    button::{button, ButtonProps, ButtonVariant},
    label::{label, LabelProps, LabelTone},
};
use egui::{Color32, CursorIcon, Response, Shadow, Ui};

#[derive(Debug, Clone, Copy)]
pub struct TooltipProps<'a> {
    pub trigger_label: &'a str,
    pub text: &'a str,
    pub width: f32,
    pub delay_ms: u32,
    pub top_center: bool,
}

impl<'a> TooltipProps<'a> {
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

pub fn tooltip(ui: &mut Ui, props: TooltipProps<'_>) -> Response {
    ui.scope(|ui| {
        let dark_mode = ui.visuals().dark_mode;
        ui.style_mut().visuals.popup_shadow = Shadow {
            offset: [2, 4],
            blur: 4,
            spread: 0,
            color: if dark_mode {
                Color32::from_black_alpha(56)
            } else {
                Color32::from_black_alpha(20)
            },
        };

        let response = button(
            ui,
            ButtonProps::new(props.trigger_label)
                .variant(ButtonVariant::Secondary)
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
                let _ = label(ui, LabelProps::new(props.text).tone(LabelTone::Secondary));
            });
        }

        response
    })
    .inner
}
