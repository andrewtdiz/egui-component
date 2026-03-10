use super::{api::ComponentUi, Button, ButtonVariant, LabelTone};
use crate::ui::tokens;
use egui::CursorIcon;
use egui::Response;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TooltipPlacement {
    Auto,
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy)]
pub struct Tooltip<'a> {
    pub trigger_label: &'a str,
    pub text: &'a str,
    pub width: f32,
    pub delay_ms: u32,
    pub placement: TooltipPlacement,
}

impl<'a> Tooltip<'a> {
    pub fn new(trigger_label: &'a str, text: &'a str) -> Self {
        Self {
            trigger_label,
            text,
            width: 220.0,
            delay_ms: 0,
            placement: TooltipPlacement::Top,
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

    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }
}

impl ComponentUi<'_> {
    pub fn tooltip<'a>(&mut self, props: impl Into<Tooltip<'a>>) -> Response {
        let props = props.into();
        let overrides = self.overrides;
        self.raw_mut()
            .scope(|ui| {
                ui.style_mut().visuals.popup_shadow = tokens::tailwind_shadow_sm();
                ui.style_mut().spacing.menu_margin = egui::Margin::symmetric(6, 4);

                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let response = ui
                    .button(
                        Button::new(props.trigger_label)
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
                    match props.placement {
                        TooltipPlacement::Auto => {}
                        TooltipPlacement::Top => {
                            tooltip.popup = tooltip
                                .popup
                                .align(egui::RectAlign::TOP)
                                .align_alternatives(&[egui::RectAlign::TOP]);
                        }
                        TooltipPlacement::Right => {
                            tooltip.popup = tooltip
                                .popup
                                .align(egui::RectAlign::RIGHT)
                                .align_alternatives(&[egui::RectAlign::RIGHT]);
                        }
                        TooltipPlacement::Bottom => {
                            tooltip.popup = tooltip
                                .popup
                                .align(egui::RectAlign::BOTTOM)
                                .align_alternatives(&[egui::RectAlign::BOTTOM]);
                        }
                        TooltipPlacement::Left => {
                            tooltip.popup = tooltip
                                .popup
                                .align(egui::RectAlign::LEFT)
                                .align_alternatives(&[egui::RectAlign::LEFT]);
                        }
                    }

                    let _ = tooltip.show(|ui| {
                        let mut ui = ComponentUi::new(ui);
                        let _ = ui.label(
                            crate::components::Label::new(props.text).tone(LabelTone::Secondary),
                        );
                    });
                }

                response
            })
            .inner
    }
}
