use egui::{CursorIcon, Response, Ui};

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
            delay_ms: 75,
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
        ui.style_mut().interaction.tooltip_delay = props.delay_ms as f32 / 1000.0;
        ui.style_mut().interaction.show_tooltips_only_when_still = false;

        let response = ui
            .add_sized(
                [props.width, ui.spacing().interact_size.y],
                egui::Button::new(props.trigger_label),
            )
            .on_hover_cursor(CursorIcon::PointingHand);

        if props.top_center {
            let mut tooltip = egui::Tooltip::for_enabled(&response).gap(6.0);
            tooltip.popup = tooltip
                .popup
                .align(egui::RectAlign::TOP)
                .align_alternatives(&[egui::RectAlign::TOP]);
            let _ = tooltip.show(|ui| {
                ui.label(props.text);
            });
            response
        } else {
            response.on_hover_text(props.text)
        }
    })
    .inner
}
