use egui::{Color32, Shadow, Stroke};

pub(crate) const TRANSPARENT: Color32 = Color32::TRANSPARENT;
const TEXT_DESTRUCTIVE: Color32 = Color32::from_rgb(248, 113, 113);
#[cfg(feature = "showcase")]
pub(crate) const GAME_ENGINE_GREEN: Color32 = Color32::from_rgb(34, 197, 94);
#[cfg(feature = "showcase")]
pub(crate) const GAME_ENGINE_RED: Color32 = Color32::from_rgb(239, 68, 68);

pub(crate) const SPACING_ITEM_Y: f32 = 8.0;
pub(crate) const SPACING_BUTTON_PADDING_X: f32 = 12.0;
pub(crate) const SPACING_BUTTON_PADDING_Y: f32 = 7.0;
pub(crate) const SPACING_INTERACT_HEIGHT: f32 = 34.0;
pub(crate) const INPUT_PADDING_X: i8 = 10;
pub(crate) const INPUT_PADDING_Y: i8 = 6;

pub(crate) const RADIUS_SM: u8 = 6;
pub(crate) const RADIUS_MD: u8 = 8;
pub(crate) const RADIUS_LG: u8 = 10;

const LIGHT_APP_BACKGROUND: Color32 = Color32::from_rgb(248, 248, 249);
const LIGHT_CARD_BACKGROUND: Color32 = Color32::from_rgb(255, 255, 255);
const LIGHT_MUTED_SURFACE: Color32 = Color32::from_rgb(250, 250, 250);
const LIGHT_SEPARATOR: Color32 = Color32::from_rgb(212, 212, 216);
const LIGHT_ROW_HOVER_BG: Color32 = Color32::from_rgb(244, 244, 245);
const LIGHT_ROW_ACTIVE_BG: Color32 = Color32::from_rgb(228, 228, 231);
const LIGHT_TEXT_PRIMARY: Color32 = Color32::from_rgb(24, 24, 27);
const LIGHT_TEXT_SECONDARY: Color32 = Color32::from_rgb(63, 63, 70);
const LIGHT_TEXT_MUTED: Color32 = Color32::from_rgb(113, 113, 122);
const LIGHT_INPUT_BACKGROUND: Color32 = Color32::from_rgb(255, 255, 255);
const LIGHT_INPUT_HOVER_BACKGROUND: Color32 = Color32::from_rgb(250, 250, 250);
const LIGHT_INPUT_FOCUS_BACKGROUND: Color32 = Color32::from_rgb(255, 255, 255);
const LIGHT_INPUT_BORDER: Color32 = Color32::from_rgb(212, 212, 216);
const LIGHT_INPUT_HOVER_BORDER: Color32 = Color32::from_rgb(161, 161, 170);
const LIGHT_BUTTON_SECONDARY_BG: Color32 = Color32::from_rgb(255, 255, 255);
const LIGHT_BUTTON_SECONDARY_HOVER_BG: Color32 = Color32::from_rgb(250, 250, 250);
const LIGHT_BUTTON_SECONDARY_ACTIVE_BG: Color32 = Color32::from_rgb(244, 244, 245);
const LIGHT_BUTTON_SECONDARY_BORDER: Color32 = Color32::from_rgb(212, 212, 216);
const LIGHT_BUTTON_SECONDARY_HOVER_BORDER: Color32 = Color32::from_rgb(161, 161, 170);
const LIGHT_BUTTON_SECONDARY_ACTIVE_BORDER: Color32 = Color32::from_rgb(82, 82, 91);
const LIGHT_SWITCH_OFF_BG: Color32 = Color32::from_rgb(212, 212, 216);
const LIGHT_SWITCH_KNOB_OFF: Color32 = Color32::from_rgb(255, 255, 255);
const LIGHT_PRIMARY_BG: Color32 = Color32::from_rgb(24, 24, 27);
const LIGHT_PRIMARY_HOVER_BG: Color32 = Color32::from_rgb(39, 39, 42);
const LIGHT_PRIMARY_ACTIVE_BG: Color32 = Color32::from_rgb(63, 63, 70);
const LIGHT_PRIMARY_FG: Color32 = Color32::from_rgb(250, 250, 250);
const LIGHT_ROW_SELECTED_BG: Color32 = Color32::from_rgb(228, 228, 231);
const LIGHT_ROW_SELECTED_TEXT: Color32 = Color32::from_rgb(24, 24, 27);
const LIGHT_SLIDER_TRACK_INACTIVE: Color32 = Color32::from_rgb(221, 221, 226);
const LIGHT_SLIDER_TRACK_ACTIVE: Color32 = Color32::from_rgb(145, 145, 154);
const LIGHT_SLIDER_THUMB_FILL: Color32 = Color32::from_rgb(246, 246, 248);
const LIGHT_SLIDER_THUMB_HOVER_FILL: Color32 = Color32::from_rgb(251, 251, 252);
const LIGHT_SLIDER_THUMB_ACTIVE_FILL: Color32 = Color32::from_rgb(255, 255, 255);
const LIGHT_SLIDER_THUMB_BORDER: Color32 = Color32::from_rgb(182, 182, 190);
const LIGHT_SLIDER_THUMB_HOVER_BORDER: Color32 = Color32::from_rgb(161, 161, 170);
const LIGHT_SLIDER_THUMB_ACTIVE_BORDER: Color32 = Color32::from_rgb(140, 140, 149);
const LIGHT_INPUT_FOCUS_BORDER: Color32 = Color32::from_rgb(82, 82, 91);
const LIGHT_TEXT_SELECTION_BG: Color32 = Color32::from_rgba_premultiplied(37, 99, 235, 96);

const DARK_APP_BACKGROUND: Color32 = Color32::from_rgb(9, 9, 11);
const DARK_CARD_BACKGROUND: Color32 = Color32::from_rgb(17, 17, 20);
const DARK_MUTED_SURFACE: Color32 = Color32::from_rgb(24, 24, 27);
const DARK_SEPARATOR: Color32 = Color32::from_rgb(39, 39, 42);
const DARK_ROW_HOVER_BG: Color32 = Color32::from_rgb(31, 31, 35);
const DARK_ROW_ACTIVE_BG: Color32 = Color32::from_rgb(39, 39, 42);
const DARK_TEXT_PRIMARY: Color32 = Color32::from_rgb(244, 244, 245);
const DARK_TEXT_SECONDARY: Color32 = Color32::from_rgb(212, 212, 216);
const DARK_TEXT_MUTED: Color32 = Color32::from_rgb(161, 161, 170);
const DARK_INPUT_BACKGROUND: Color32 = Color32::from_rgb(24, 24, 27);
const DARK_INPUT_HOVER_BACKGROUND: Color32 = Color32::from_rgb(31, 31, 35);
const DARK_INPUT_FOCUS_BACKGROUND: Color32 = Color32::from_rgb(14, 14, 17);
const DARK_INPUT_BORDER: Color32 = Color32::from_rgb(63, 63, 70);
const DARK_INPUT_HOVER_BORDER: Color32 = Color32::from_rgb(82, 82, 91);
const DARK_BUTTON_SECONDARY_BG: Color32 = Color32::from_rgb(24, 24, 27);
const DARK_BUTTON_SECONDARY_HOVER_BG: Color32 = Color32::from_rgb(31, 31, 35);
const DARK_BUTTON_SECONDARY_ACTIVE_BG: Color32 = Color32::from_rgb(39, 39, 42);
const DARK_BUTTON_SECONDARY_BORDER: Color32 = Color32::from_rgb(63, 63, 70);
const DARK_BUTTON_SECONDARY_HOVER_BORDER: Color32 = Color32::from_rgb(82, 82, 91);
const DARK_BUTTON_SECONDARY_ACTIVE_BORDER: Color32 = Color32::from_rgb(113, 113, 122);
const DARK_SWITCH_OFF_BG: Color32 = Color32::from_rgb(82, 82, 91);
const DARK_SWITCH_KNOB_OFF: Color32 = Color32::from_rgb(244, 244, 245);
const DARK_PRIMARY_BG: Color32 = Color32::from_rgb(250, 250, 250);
const DARK_PRIMARY_HOVER_BG: Color32 = Color32::from_rgb(228, 228, 231);
const DARK_PRIMARY_ACTIVE_BG: Color32 = Color32::from_rgb(212, 212, 216);
const DARK_PRIMARY_FG: Color32 = Color32::from_rgb(24, 24, 27);
const DARK_ROW_SELECTED_BG: Color32 = Color32::from_rgb(63, 63, 70);
const DARK_ROW_SELECTED_TEXT: Color32 = Color32::from_rgb(250, 250, 250);
const DARK_SLIDER_TRACK_INACTIVE: Color32 = Color32::from_rgb(55, 55, 61);
const DARK_SLIDER_TRACK_ACTIVE: Color32 = Color32::from_rgb(133, 133, 144);
const DARK_SLIDER_THUMB_FILL: Color32 = Color32::from_rgb(232, 232, 236);
const DARK_SLIDER_THUMB_HOVER_FILL: Color32 = Color32::from_rgb(242, 242, 244);
const DARK_SLIDER_THUMB_ACTIVE_FILL: Color32 = Color32::from_rgb(250, 250, 250);
const DARK_SLIDER_THUMB_BORDER: Color32 = Color32::from_rgb(74, 74, 82);
const DARK_SLIDER_THUMB_HOVER_BORDER: Color32 = Color32::from_rgb(92, 92, 101);
const DARK_SLIDER_THUMB_ACTIVE_BORDER: Color32 = Color32::from_rgb(113, 113, 122);
const DARK_INPUT_FOCUS_BORDER: Color32 = Color32::from_rgb(161, 161, 170);
const DARK_TEXT_SELECTION_BG: Color32 = Color32::from_rgba_premultiplied(59, 130, 246, 120);

const fn mode_color(dark_mode: bool, light: Color32, dark: Color32) -> Color32 {
    if dark_mode {
        dark
    } else {
        light
    }
}

pub(crate) fn app_background(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_APP_BACKGROUND, DARK_APP_BACKGROUND)
}

pub(crate) fn card_background(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_CARD_BACKGROUND, DARK_CARD_BACKGROUND)
}

pub(crate) fn muted_surface(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_MUTED_SURFACE, DARK_MUTED_SURFACE)
}

pub(crate) fn separator(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_SEPARATOR, DARK_SEPARATOR)
}

pub(crate) fn row_hover_bg(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_ROW_HOVER_BG, DARK_ROW_HOVER_BG)
}

pub(crate) fn row_active_bg(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_ROW_ACTIVE_BG, DARK_ROW_ACTIVE_BG)
}

pub(crate) fn text_primary(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_TEXT_PRIMARY, DARK_TEXT_PRIMARY)
}

pub(crate) fn text_secondary(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_TEXT_SECONDARY, DARK_TEXT_SECONDARY)
}

pub(crate) fn text_muted(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_TEXT_MUTED, DARK_TEXT_MUTED)
}

pub(crate) fn text_destructive(_: bool) -> Color32 {
    TEXT_DESTRUCTIVE
}

pub(crate) fn input_background(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_INPUT_BACKGROUND, DARK_INPUT_BACKGROUND)
}

pub(crate) fn input_hover_background(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_INPUT_HOVER_BACKGROUND,
        DARK_INPUT_HOVER_BACKGROUND,
    )
}

pub(crate) fn input_focus_background(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_INPUT_FOCUS_BACKGROUND,
        DARK_INPUT_FOCUS_BACKGROUND,
    )
}

pub(crate) fn input_border(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_INPUT_BORDER, DARK_INPUT_BORDER)
}

pub(crate) fn input_hover_border(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_INPUT_HOVER_BORDER, DARK_INPUT_HOVER_BORDER)
}

pub(crate) fn button_secondary_bg(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_BUTTON_SECONDARY_BG,
        DARK_BUTTON_SECONDARY_BG,
    )
}

pub(crate) fn button_secondary_hover_bg(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_BUTTON_SECONDARY_HOVER_BG,
        DARK_BUTTON_SECONDARY_HOVER_BG,
    )
}

pub(crate) fn button_secondary_active_bg(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_BUTTON_SECONDARY_ACTIVE_BG,
        DARK_BUTTON_SECONDARY_ACTIVE_BG,
    )
}

pub(crate) fn button_secondary_border(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_BUTTON_SECONDARY_BORDER,
        DARK_BUTTON_SECONDARY_BORDER,
    )
}

pub(crate) fn button_secondary_hover_border(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_BUTTON_SECONDARY_HOVER_BORDER,
        DARK_BUTTON_SECONDARY_HOVER_BORDER,
    )
}

pub(crate) fn button_secondary_active_border(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_BUTTON_SECONDARY_ACTIVE_BORDER,
        DARK_BUTTON_SECONDARY_ACTIVE_BORDER,
    )
}

pub(crate) fn switch_off_bg(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_SWITCH_OFF_BG, DARK_SWITCH_OFF_BG)
}

pub(crate) fn switch_knob_off(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_SWITCH_KNOB_OFF, DARK_SWITCH_KNOB_OFF)
}

pub(crate) fn primary_bg(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_PRIMARY_BG, DARK_PRIMARY_BG)
}

pub(crate) fn primary_hover_bg(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_PRIMARY_HOVER_BG, DARK_PRIMARY_HOVER_BG)
}

pub(crate) fn primary_active_bg(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_PRIMARY_ACTIVE_BG, DARK_PRIMARY_ACTIVE_BG)
}

pub(crate) fn primary_fg(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_PRIMARY_FG, DARK_PRIMARY_FG)
}

pub(crate) fn row_selected_bg(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_ROW_SELECTED_BG, DARK_ROW_SELECTED_BG)
}

pub(crate) fn row_selected_text(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_ROW_SELECTED_TEXT, DARK_ROW_SELECTED_TEXT)
}

pub(crate) const fn tailwind_shadow_sm() -> Shadow {
    Shadow {
        offset: [0, 1],
        blur: 2,
        spread: 0,
        color: Color32::from_rgba_premultiplied(0, 0, 0, 10),
    }
}

pub(crate) const fn tailwind_shadow_md() -> Shadow {
    Shadow {
        offset: [0, 2],
        blur: 4,
        spread: 0,
        color: Color32::from_rgba_premultiplied(0, 0, 0, 18),
    }
}

pub(crate) const fn tailwind_shadow_lg() -> Shadow {
    Shadow {
        offset: [0, 6],
        blur: 10,
        spread: 0,
        color: Color32::from_rgba_premultiplied(0, 0, 0, 22),
    }
}

pub(crate) fn text_selection_bg(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_TEXT_SELECTION_BG, DARK_TEXT_SELECTION_BG)
}

pub(crate) fn input_bg(dark_mode: bool, focused: bool, hovered: bool) -> Color32 {
    if focused {
        input_focus_background(dark_mode)
    } else if hovered {
        input_hover_background(dark_mode)
    } else {
        input_background(dark_mode)
    }
}

pub(crate) fn input_stroke(dark_mode: bool, focused: bool, hovered: bool) -> Stroke {
    if focused {
        input_focus_stroke(dark_mode)
    } else if hovered {
        Stroke::new(1.0, input_hover_border(dark_mode))
    } else {
        Stroke::new(1.0, input_border(dark_mode))
    }
}

pub(crate) fn row_bg(selected: bool, pressed: bool, hovered: bool, dark_mode: bool) -> Color32 {
    if selected {
        row_selected_bg(dark_mode)
    } else if pressed {
        row_active_bg(dark_mode)
    } else if hovered {
        row_hover_bg(dark_mode)
    } else {
        TRANSPARENT
    }
}

pub(crate) fn slider_track_inactive(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_SLIDER_TRACK_INACTIVE,
        DARK_SLIDER_TRACK_INACTIVE,
    )
}

pub(crate) fn slider_track_active(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_SLIDER_TRACK_ACTIVE,
        DARK_SLIDER_TRACK_ACTIVE,
    )
}

pub(crate) fn slider_thumb_fill(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_SLIDER_THUMB_FILL, DARK_SLIDER_THUMB_FILL)
}

pub(crate) fn slider_thumb_hover_fill(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_SLIDER_THUMB_HOVER_FILL,
        DARK_SLIDER_THUMB_HOVER_FILL,
    )
}

pub(crate) fn slider_thumb_active_fill(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_SLIDER_THUMB_ACTIVE_FILL,
        DARK_SLIDER_THUMB_ACTIVE_FILL,
    )
}

pub(crate) fn slider_thumb_border(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_SLIDER_THUMB_BORDER,
        DARK_SLIDER_THUMB_BORDER,
    )
}

pub(crate) fn slider_thumb_hover_border(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_SLIDER_THUMB_HOVER_BORDER,
        DARK_SLIDER_THUMB_HOVER_BORDER,
    )
}

pub(crate) fn slider_thumb_active_border(dark_mode: bool) -> Color32 {
    mode_color(
        dark_mode,
        LIGHT_SLIDER_THUMB_ACTIVE_BORDER,
        DARK_SLIDER_THUMB_ACTIVE_BORDER,
    )
}

pub(crate) fn input_focus_border(dark_mode: bool) -> Color32 {
    mode_color(dark_mode, LIGHT_INPUT_FOCUS_BORDER, DARK_INPUT_FOCUS_BORDER)
}

pub(crate) fn input_focus_stroke(dark_mode: bool) -> Stroke {
    Stroke::new(1.1, input_focus_border(dark_mode))
}
