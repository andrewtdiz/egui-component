use super::{api::ComponentUi, Button, ButtonVariant, LabelTone};
use crate::ui::tokens;
use egui::util::IdTypeMap;
use egui::{CursorIcon, Response};

const TOOLTIP_GAP: f32 = 6.0;
const TOOLTIP_PADDING_X: i8 = 6;
const TOOLTIP_PADDING_Y: i8 = 4;

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
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
        self.ui_mut()
            .scope(|ui| {
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
                    let hover_started_at = response.ctx.data_mut(|data: &mut IdTypeMap| {
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
                        .data_mut(|data: &mut IdTypeMap| data.remove::<f64>(hover_started_id));
                    false
                };

                if show_tooltip {
                    let tooltip_frame =
                        tooltip_frame(ui.style(), crate::theme::runtime_for_ui(&ui));
                    let mut tooltip = egui::Tooltip::for_widget(&response).gap(TOOLTIP_GAP);
                    tooltip.popup = tooltip.popup.frame(tooltip_frame);
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

fn tooltip_frame(style: &egui::Style, runtime: crate::theme::ThemeRuntime) -> egui::Frame {
    egui::Frame::popup(style)
        .corner_radius(tokens::radius_sm(runtime))
        .inner_margin(egui::Margin::symmetric(
            TOOLTIP_PADDING_X,
            TOOLTIP_PADDING_Y,
        ))
        .shadow(tokens::tailwind_shadow_sm(runtime))
}

#[cfg(test)]
mod tests {
    use super::{tooltip_frame, TOOLTIP_PADDING_X, TOOLTIP_PADDING_Y};
    use crate::theme::ThemeMode;
    use crate::ui::tokens;
    use egui::{Context, CornerRadius, Margin};

    #[test]
    fn tooltip_frame_overrides_popup_padding_and_shadow() {
        let context = Context::default();
        crate::theme::install(
            &context,
            crate::theme::ThemeSpec::default(),
            ThemeMode::Dark,
        );
        let style = context.style();
        let popup_frame = egui::Frame::popup(&style);
        let frame = tooltip_frame(&style, crate::theme::runtime_for_context(&context));

        assert_eq!(
            frame.inner_margin,
            Margin::symmetric(TOOLTIP_PADDING_X, TOOLTIP_PADDING_Y)
        );
        assert_eq!(
            frame.shadow,
            tokens::tailwind_shadow_sm(crate::theme::runtime_for_context(&context))
        );
        assert_eq!(frame.fill, popup_frame.fill);
        assert_eq!(frame.stroke, popup_frame.stroke);
        assert_eq!(
            frame.corner_radius,
            CornerRadius::same(tokens::radius_sm(crate::theme::runtime_for_context(
                &context
            )))
        );
    }
}
