use crate::theme::{self, ColorRole, RadiusRole, ShadowRole, ThemeRuntime};
use egui::{Color32, Shadow, Stroke};

pub(crate) const TRANSPARENT: Color32 = Color32::TRANSPARENT;

pub(crate) const SPACING_ITEM_Y: f32 = 7.0;
pub(crate) const SPACING_BUTTON_PADDING_X: f32 = 12.0;
pub(crate) const SPACING_BUTTON_PADDING_Y: f32 = 7.0;
pub(crate) const SPACING_INTERACT_HEIGHT: f32 = 34.0;
pub(crate) const LAYOUT_GAP_XS: f32 = 4.0;
pub(crate) const LAYOUT_GAP_SM: f32 = 5.0;
pub(crate) const INPUT_PADDING_X: i8 = 10;

fn role(runtime: ThemeRuntime, role: ColorRole) -> Color32 {
    theme::resolved_color(runtime, role)
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    a.lerp_to_gamma(b, t)
}

fn alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

pub(crate) fn app_background(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Background)
}

pub(crate) fn card_background(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Card)
}

pub(crate) fn popover_background(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Popover)
}

pub(crate) fn muted_surface(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Muted)
}

pub(crate) fn separator(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Border)
}

pub(crate) fn row_hover_bg(runtime: ThemeRuntime) -> Color32 {
    mix(
        role(runtime, ColorRole::Accent),
        role(runtime, ColorRole::Background),
        if runtime.mode.is_dark() { 0.28 } else { 0.14 },
    )
}

#[cfg(test)]
pub(crate) fn row_active_bg(runtime: ThemeRuntime) -> Color32 {
    row_hover_bg(runtime)
}

pub(crate) fn text_primary(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Foreground)
}

pub(crate) fn text_secondary(runtime: ThemeRuntime) -> Color32 {
    mix(
        role(runtime, ColorRole::Foreground),
        role(runtime, ColorRole::MutedForeground),
        0.55,
    )
}

pub(crate) fn text_muted(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::MutedForeground)
}

pub(crate) fn text_destructive(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Destructive)
}

pub(crate) fn input_background(runtime: ThemeRuntime) -> Color32 {
    mix(
        role(runtime, ColorRole::Background),
        role(runtime, ColorRole::Card),
        if runtime.mode.is_dark() { 0.82 } else { 0.72 },
    )
}

pub(crate) fn input_hover_background(runtime: ThemeRuntime) -> Color32 {
    mix(
        input_background(runtime),
        role(runtime, ColorRole::Accent),
        if runtime.mode.is_dark() { 0.14 } else { 0.1 },
    )
}

pub(crate) fn input_focus_background(runtime: ThemeRuntime) -> Color32 {
    mix(
        input_background(runtime),
        role(runtime, ColorRole::Card),
        if runtime.mode.is_dark() { 0.18 } else { 0.12 },
    )
}

pub(crate) fn input_border(runtime: ThemeRuntime) -> Color32 {
    mix(
        role(runtime, ColorRole::Input),
        input_background(runtime),
        if runtime.mode.is_dark() { 0.56 } else { 0.4 },
    )
}

pub(crate) fn input_hover_border(runtime: ThemeRuntime) -> Color32 {
    mix(input_border(runtime), role(runtime, ColorRole::Ring), 0.18)
}

pub(crate) fn button_secondary_bg(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Secondary)
}

pub(crate) fn button_secondary_hover_bg(runtime: ThemeRuntime) -> Color32 {
    mix(
        role(runtime, ColorRole::Secondary),
        role(runtime, ColorRole::Foreground),
        0.025,
    )
}

pub(crate) fn button_secondary_active_bg(runtime: ThemeRuntime) -> Color32 {
    button_secondary_hover_bg(runtime)
}

pub(crate) fn button_secondary_border(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Border)
}

pub(crate) fn button_secondary_hover_border(runtime: ThemeRuntime) -> Color32 {
    mix(
        role(runtime, ColorRole::Border),
        role(runtime, ColorRole::Ring),
        0.18,
    )
}

pub(crate) fn button_secondary_active_border(runtime: ThemeRuntime) -> Color32 {
    button_secondary_hover_border(runtime)
}

pub(crate) fn switch_off_bg(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Input)
}

pub(crate) fn switch_knob_off(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Background)
}

pub(crate) fn primary_bg(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Primary)
}

pub(crate) fn primary_hover_bg(runtime: ThemeRuntime) -> Color32 {
    mix(
        role(runtime, ColorRole::Primary),
        role(runtime, ColorRole::Background),
        0.12,
    )
}

pub(crate) fn primary_active_bg(runtime: ThemeRuntime) -> Color32 {
    primary_hover_bg(runtime)
}

pub(crate) fn primary_fg(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::PrimaryForeground)
}

pub(crate) fn button_primary_fill_fg(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::PrimaryForeground)
}

pub(crate) fn row_selected_bg(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Accent)
}

pub(crate) fn row_selected_text(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::AccentForeground)
}

pub(crate) fn tailwind_shadow_sm(runtime: ThemeRuntime) -> Shadow {
    theme::resolved_shadow(runtime, ShadowRole::Sm)
}

pub(crate) fn tailwind_shadow_md(runtime: ThemeRuntime) -> Shadow {
    theme::resolved_shadow(runtime, ShadowRole::Md)
}

pub(crate) fn tailwind_shadow_lg(runtime: ThemeRuntime) -> Shadow {
    theme::resolved_shadow(runtime, ShadowRole::Lg)
}

pub(crate) fn text_selection_bg(runtime: ThemeRuntime) -> Color32 {
    let dark_mode = runtime.mode.is_dark();
    alpha(
        role(runtime, ColorRole::Ring),
        if dark_mode { 120 } else { 96 },
    )
}

pub(crate) fn dialogue_backdrop(runtime: ThemeRuntime) -> Color32 {
    if runtime.mode.is_dark() {
        Color32::from_black_alpha(160)
    } else {
        Color32::from_black_alpha(96)
    }
}

pub(crate) fn input_bg(runtime: ThemeRuntime, focused: bool, hovered: bool) -> Color32 {
    if focused {
        input_focus_background(runtime)
    } else if hovered {
        input_hover_background(runtime)
    } else {
        input_background(runtime)
    }
}

pub(crate) fn input_stroke(runtime: ThemeRuntime, focused: bool, hovered: bool) -> Stroke {
    if focused {
        input_focus_stroke(runtime)
    } else if hovered {
        Stroke::new(1.0, input_hover_border(runtime))
    } else {
        Stroke::new(1.0, input_border(runtime))
    }
}

pub(crate) fn row_bg(
    selected: bool,
    pressed: bool,
    hovered: bool,
    runtime: ThemeRuntime,
) -> Color32 {
    if selected {
        row_selected_bg(runtime)
    } else if pressed || hovered {
        row_hover_bg(runtime)
    } else {
        TRANSPARENT
    }
}

pub(crate) fn slider_track_inactive(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Border)
}

pub(crate) fn slider_track_active(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Primary)
}

pub(crate) fn slider_thumb_fill(runtime: ThemeRuntime) -> Color32 {
    mix(
        role(runtime, ColorRole::Muted),
        role(runtime, ColorRole::Foreground),
        if runtime.mode.is_dark() { 0.18 } else { 0.12 },
    )
}

pub(crate) fn slider_thumb_hover_fill(runtime: ThemeRuntime) -> Color32 {
    slider_thumb_fill(runtime)
}

pub(crate) fn slider_thumb_active_fill(runtime: ThemeRuntime) -> Color32 {
    slider_thumb_hover_fill(runtime)
}

pub(crate) fn slider_thumb_border(runtime: ThemeRuntime) -> Color32 {
    mix(
        role(runtime, ColorRole::Border),
        role(runtime, ColorRole::Foreground),
        if runtime.mode.is_dark() { 0.1 } else { 0.05 },
    )
}

pub(crate) fn slider_thumb_hover_border(runtime: ThemeRuntime) -> Color32 {
    mix(
        role(runtime, ColorRole::Border),
        role(runtime, ColorRole::Foreground),
        if runtime.mode.is_dark() { 0.24 } else { 0.12 },
    )
}

pub(crate) fn slider_thumb_active_border(runtime: ThemeRuntime) -> Color32 {
    slider_thumb_hover_border(runtime)
}

pub(crate) fn input_focus_border(runtime: ThemeRuntime) -> Color32 {
    role(runtime, ColorRole::Ring)
}

pub(crate) fn input_focus_stroke(runtime: ThemeRuntime) -> Stroke {
    Stroke::new(1.0, input_focus_border(runtime))
}

pub(crate) fn radius_sm(runtime: ThemeRuntime) -> u8 {
    theme::resolved_radius(runtime, RadiusRole::Sm)
}

pub(crate) fn radius_md(runtime: ThemeRuntime) -> u8 {
    theme::resolved_radius(runtime, RadiusRole::Md)
}

pub(crate) fn radius_lg(runtime: ThemeRuntime) -> u8 {
    theme::resolved_radius(runtime, RadiusRole::Lg)
}

#[cfg(test)]
mod tests {
    use super::{
        button_secondary_active_bg, button_secondary_active_border, button_secondary_hover_bg,
        button_secondary_hover_border, primary_active_bg, primary_hover_bg, row_active_bg,
        row_hover_bg, slider_thumb_active_border, slider_thumb_active_fill, slider_thumb_fill,
        slider_thumb_hover_border, slider_thumb_hover_fill,
    };
    use crate::theme::ThemeRuntime;

    #[test]
    fn button_secondary_active_border_matches_hover_border() {
        let runtime = ThemeRuntime::default();

        assert_eq!(
            button_secondary_active_border(runtime),
            button_secondary_hover_border(runtime)
        );
    }

    #[test]
    fn active_background_tokens_match_hover_tokens() {
        let runtime = ThemeRuntime::default();

        assert_eq!(
            button_secondary_active_bg(runtime),
            button_secondary_hover_bg(runtime)
        );
        assert_eq!(primary_active_bg(runtime), primary_hover_bg(runtime));
        assert_eq!(row_active_bg(runtime), row_hover_bg(runtime));
        assert_eq!(
            slider_thumb_active_fill(runtime),
            slider_thumb_hover_fill(runtime)
        );
        assert_eq!(
            slider_thumb_active_border(runtime),
            slider_thumb_hover_border(runtime)
        );
    }

    #[test]
    fn slider_thumb_fill_matches_hover_fill() {
        let runtime = ThemeRuntime::default();

        assert_eq!(slider_thumb_fill(runtime), slider_thumb_hover_fill(runtime));
    }
}
