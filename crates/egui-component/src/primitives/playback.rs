use crate::ui::icons;
use egui::{CursorIcon, Rect, Response, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum PlaybackButtonState {
    Paused,
    Playing,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PlaybackButtonStyle {
    pub icon_size: f32,
    pub fill: egui::Color32,
    pub hover_fill: egui::Color32,
    pub stroke: Stroke,
    pub icon_tint: egui::Color32,
    pub paused_icon_name: &'static str,
    pub playing_icon_name: &'static str,
}

pub(crate) fn paint_playback_button(
    ui: &mut Ui,
    rect: Rect,
    response: &Response,
    state: PlaybackButtonState,
    style: PlaybackButtonStyle,
) {
    let fill = if response.is_pointer_button_down_on() {
        style.hover_fill
    } else if response.hovered() {
        style.hover_fill
    } else {
        style.fill
    };

    ui.painter()
        .circle_filled(rect.center(), rect.width() * 0.5, fill);
    ui.painter()
        .circle_stroke(rect.center(), rect.width() * 0.5, style.stroke);

    let icon_name = match state {
        PlaybackButtonState::Paused => style.paused_icon_name,
        PlaybackButtonState::Playing => style.playing_icon_name,
    };

    if let Some(icon) = icons::image(ui.ctx(), icon_name, style.icon_size) {
        let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(style.icon_size));
        let icon_rect = if state == PlaybackButtonState::Paused {
            icon_rect.translate(egui::vec2(1.0, 0.0))
        } else {
            icon_rect
        };
        let _ = icon.tint(style.icon_tint).paint_at(ui, icon_rect);
    }
}

pub(crate) fn playback_button_response(
    ui: &mut Ui,
    rect: Rect,
    id: egui::Id,
    state: PlaybackButtonState,
    style: PlaybackButtonStyle,
) -> Response {
    let response = ui.interact(rect, id, egui::Sense::click());
    paint_playback_button(ui, rect, &response, state, style);
    response.on_hover_cursor(CursorIcon::PointingHand)
}
