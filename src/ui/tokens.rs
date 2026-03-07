use egui::{Color32, Shadow, Stroke};

#[derive(Clone, Copy)]
pub(crate) struct ColorScale {
    pub(crate) c100: Color32,
    pub(crate) c200: Color32,
    pub(crate) c300: Color32,
    pub(crate) c400: Color32,
    pub(crate) c600: Color32,
    pub(crate) c700: Color32,
    pub(crate) c800: Color32,
    pub(crate) c900: Color32,
}

pub(crate) const NEUTRAL: ColorScale = ColorScale {
    c100: Color32::from_rgb(250, 250, 250),
    c200: Color32::from_rgb(228, 228, 231),
    c300: Color32::from_rgb(212, 212, 216),
    c400: Color32::from_rgb(161, 161, 170),
    c600: Color32::from_rgb(82, 82, 91),
    c700: Color32::from_rgb(63, 63, 70),
    c800: Color32::from_rgb(39, 39, 42),
    c900: Color32::from_rgb(24, 24, 27),
};

pub(crate) const TRANSPARENT: Color32 = Color32::TRANSPARENT;

pub(crate) const TEXT_DESTRUCTIVE: Color32 = Color32::from_rgb(248, 113, 113);
pub(crate) const GAME_ENGINE_GREEN: Color32 = Color32::from_rgb(34, 197, 94);
pub(crate) const GAME_ENGINE_RED: Color32 = Color32::from_rgb(239, 68, 68);

const APP_BACKGROUND_DARK: Color32 = Color32::from_rgb(9, 9, 11);
const APP_BACKGROUND_LIGHT: Color32 = Color32::from_rgb(248, 248, 249);
const CARD_BACKGROUND_DARK: Color32 = Color32::from_rgb(17, 17, 20);
const CARD_BACKGROUND_LIGHT: Color32 = Color32::from_rgb(255, 255, 255);
const MUTED_SURFACE_DARK: Color32 = Color32::from_rgb(24, 24, 27);
const MUTED_SURFACE_LIGHT: Color32 = Color32::from_rgb(250, 250, 250);
const SEPARATOR_DARK: Color32 = Color32::from_rgb(39, 39, 42);
const SEPARATOR_LIGHT: Color32 = Color32::from_rgb(212, 212, 216);

const ROW_HOVER_BG_DARK: Color32 = Color32::from_rgb(31, 31, 35);
const ROW_HOVER_BG_LIGHT: Color32 = Color32::from_rgb(244, 244, 245);
const ROW_ACTIVE_BG_DARK: Color32 = Color32::from_rgb(39, 39, 42);
const ROW_ACTIVE_BG_LIGHT: Color32 = Color32::from_rgb(228, 228, 231);

const TEXT_PRIMARY_DARK: Color32 = Color32::from_rgb(244, 244, 245);
const TEXT_PRIMARY_LIGHT: Color32 = Color32::from_rgb(24, 24, 27);
const TEXT_SECONDARY_DARK: Color32 = Color32::from_rgb(212, 212, 216);
const TEXT_SECONDARY_LIGHT: Color32 = Color32::from_rgb(63, 63, 70);
const TEXT_MUTED_DARK: Color32 = Color32::from_rgb(161, 161, 170);
const TEXT_MUTED_LIGHT: Color32 = Color32::from_rgb(113, 113, 122);

const INPUT_BACKGROUND_DARK: Color32 = Color32::from_rgb(24, 24, 27);
const INPUT_BACKGROUND_LIGHT: Color32 = Color32::from_rgb(255, 255, 255);
const INPUT_HOVER_BACKGROUND_DARK: Color32 = Color32::from_rgb(31, 31, 35);
const INPUT_HOVER_BACKGROUND_LIGHT: Color32 = Color32::from_rgb(250, 250, 250);
const INPUT_FOCUS_BACKGROUND_DARK: Color32 = Color32::from_rgb(14, 14, 17);
const INPUT_FOCUS_BACKGROUND_LIGHT: Color32 = Color32::from_rgb(255, 255, 255);
const INPUT_BORDER_DARK: Color32 = Color32::from_rgb(63, 63, 70);
const INPUT_BORDER_LIGHT: Color32 = Color32::from_rgb(212, 212, 216);
const INPUT_HOVER_BORDER_DARK: Color32 = Color32::from_rgb(82, 82, 91);
const INPUT_HOVER_BORDER_LIGHT: Color32 = Color32::from_rgb(161, 161, 170);

const BUTTON_SECONDARY_BG_DARK: Color32 = Color32::from_rgb(24, 24, 27);
const BUTTON_SECONDARY_BG_LIGHT: Color32 = Color32::from_rgb(255, 255, 255);
const BUTTON_SECONDARY_HOVER_BG_DARK: Color32 = Color32::from_rgb(31, 31, 35);
const BUTTON_SECONDARY_HOVER_BG_LIGHT: Color32 = Color32::from_rgb(250, 250, 250);
const BUTTON_SECONDARY_ACTIVE_BG_DARK: Color32 = Color32::from_rgb(39, 39, 42);
const BUTTON_SECONDARY_ACTIVE_BG_LIGHT: Color32 = Color32::from_rgb(244, 244, 245);
const BUTTON_SECONDARY_BORDER_DARK: Color32 = Color32::from_rgb(63, 63, 70);
const BUTTON_SECONDARY_BORDER_LIGHT: Color32 = Color32::from_rgb(212, 212, 216);
const BUTTON_SECONDARY_HOVER_BORDER_DARK: Color32 = Color32::from_rgb(82, 82, 91);
const BUTTON_SECONDARY_HOVER_BORDER_LIGHT: Color32 = Color32::from_rgb(161, 161, 170);
const BUTTON_SECONDARY_ACTIVE_BORDER_DARK: Color32 = Color32::from_rgb(113, 113, 122);
const BUTTON_SECONDARY_ACTIVE_BORDER_LIGHT: Color32 = Color32::from_rgb(82, 82, 91);

const SWITCH_OFF_BG_DARK: Color32 = Color32::from_rgb(82, 82, 91);
const SWITCH_OFF_BG_LIGHT: Color32 = Color32::from_rgb(212, 212, 216);
const SWITCH_KNOB_OFF_DARK: Color32 = Color32::from_rgb(244, 244, 245);
const SWITCH_KNOB_OFF_LIGHT: Color32 = Color32::from_rgb(255, 255, 255);
const SLIDER_TRACK_INACTIVE_DARK: Color32 = Color32::from_rgb(55, 55, 61);
const SLIDER_TRACK_INACTIVE_LIGHT: Color32 = Color32::from_rgb(221, 221, 226);
const SLIDER_TRACK_ACTIVE_DARK: Color32 = Color32::from_rgb(133, 133, 144);
const SLIDER_TRACK_ACTIVE_LIGHT: Color32 = Color32::from_rgb(145, 145, 154);
const SLIDER_THUMB_FILL_DARK: Color32 = Color32::from_rgb(250, 250, 250);
const SLIDER_THUMB_FILL_LIGHT: Color32 = Color32::from_rgb(255, 255, 255);
const SLIDER_THUMB_HOVER_FILL_DARK: Color32 = Color32::from_rgb(244, 244, 245);
const SLIDER_THUMB_HOVER_FILL_LIGHT: Color32 = Color32::from_rgb(252, 252, 253);
const SLIDER_THUMB_BORDER_DARK: Color32 = Color32::from_rgb(82, 82, 91);
const SLIDER_THUMB_BORDER_LIGHT: Color32 = Color32::from_rgb(161, 161, 170);
const SLIDER_THUMB_HOVER_BORDER_DARK: Color32 = Color32::from_rgb(113, 113, 122);
const SLIDER_THUMB_HOVER_BORDER_LIGHT: Color32 = Color32::from_rgb(140, 140, 149);

pub(crate) const SPACING_ITEM_Y: f32 = 8.0;
pub(crate) const SPACING_BUTTON_PADDING_X: f32 = 12.0;
pub(crate) const SPACING_BUTTON_PADDING_Y: f32 = 7.0;
pub(crate) const SPACING_INTERACT_HEIGHT: f32 = 34.0;
pub(crate) const INPUT_PADDING_X: i8 = 10;
pub(crate) const INPUT_PADDING_Y: i8 = 6;

pub(crate) const RADIUS_SM: u8 = 6;
pub(crate) const RADIUS_MD: u8 = 8;
pub(crate) const RADIUS_LG: u8 = 10;

pub(crate) const fn app_background(dark_mode: bool) -> Color32 {
    if dark_mode {
        APP_BACKGROUND_DARK
    } else {
        APP_BACKGROUND_LIGHT
    }
}

pub(crate) const fn card_background(dark_mode: bool) -> Color32 {
    if dark_mode {
        CARD_BACKGROUND_DARK
    } else {
        CARD_BACKGROUND_LIGHT
    }
}

pub(crate) const fn muted_surface(dark_mode: bool) -> Color32 {
    if dark_mode {
        MUTED_SURFACE_DARK
    } else {
        MUTED_SURFACE_LIGHT
    }
}

pub(crate) const fn separator(dark_mode: bool) -> Color32 {
    if dark_mode {
        SEPARATOR_DARK
    } else {
        SEPARATOR_LIGHT
    }
}

pub(crate) const fn row_hover_bg(dark_mode: bool) -> Color32 {
    if dark_mode {
        ROW_HOVER_BG_DARK
    } else {
        ROW_HOVER_BG_LIGHT
    }
}

pub(crate) const fn row_active_bg(dark_mode: bool) -> Color32 {
    if dark_mode {
        ROW_ACTIVE_BG_DARK
    } else {
        ROW_ACTIVE_BG_LIGHT
    }
}

pub(crate) const fn text_primary(dark_mode: bool) -> Color32 {
    if dark_mode {
        TEXT_PRIMARY_DARK
    } else {
        TEXT_PRIMARY_LIGHT
    }
}

pub(crate) const fn text_secondary(dark_mode: bool) -> Color32 {
    if dark_mode {
        TEXT_SECONDARY_DARK
    } else {
        TEXT_SECONDARY_LIGHT
    }
}

pub(crate) const fn text_muted(dark_mode: bool) -> Color32 {
    if dark_mode {
        TEXT_MUTED_DARK
    } else {
        TEXT_MUTED_LIGHT
    }
}

pub(crate) const fn text_destructive(_: bool) -> Color32 {
    TEXT_DESTRUCTIVE
}

pub(crate) const fn input_background(dark_mode: bool) -> Color32 {
    if dark_mode {
        INPUT_BACKGROUND_DARK
    } else {
        INPUT_BACKGROUND_LIGHT
    }
}

pub(crate) const fn input_hover_background(dark_mode: bool) -> Color32 {
    if dark_mode {
        INPUT_HOVER_BACKGROUND_DARK
    } else {
        INPUT_HOVER_BACKGROUND_LIGHT
    }
}

pub(crate) const fn input_focus_background(dark_mode: bool) -> Color32 {
    if dark_mode {
        INPUT_FOCUS_BACKGROUND_DARK
    } else {
        INPUT_FOCUS_BACKGROUND_LIGHT
    }
}

pub(crate) const fn input_border(dark_mode: bool) -> Color32 {
    if dark_mode {
        INPUT_BORDER_DARK
    } else {
        INPUT_BORDER_LIGHT
    }
}

pub(crate) const fn input_hover_border(dark_mode: bool) -> Color32 {
    if dark_mode {
        INPUT_HOVER_BORDER_DARK
    } else {
        INPUT_HOVER_BORDER_LIGHT
    }
}

pub(crate) const fn button_secondary_bg(dark_mode: bool) -> Color32 {
    if dark_mode {
        BUTTON_SECONDARY_BG_DARK
    } else {
        BUTTON_SECONDARY_BG_LIGHT
    }
}

pub(crate) const fn button_secondary_hover_bg(dark_mode: bool) -> Color32 {
    if dark_mode {
        BUTTON_SECONDARY_HOVER_BG_DARK
    } else {
        BUTTON_SECONDARY_HOVER_BG_LIGHT
    }
}

pub(crate) const fn button_secondary_active_bg(dark_mode: bool) -> Color32 {
    if dark_mode {
        BUTTON_SECONDARY_ACTIVE_BG_DARK
    } else {
        BUTTON_SECONDARY_ACTIVE_BG_LIGHT
    }
}

pub(crate) const fn button_secondary_border(dark_mode: bool) -> Color32 {
    if dark_mode {
        BUTTON_SECONDARY_BORDER_DARK
    } else {
        BUTTON_SECONDARY_BORDER_LIGHT
    }
}

pub(crate) const fn button_secondary_hover_border(dark_mode: bool) -> Color32 {
    if dark_mode {
        BUTTON_SECONDARY_HOVER_BORDER_DARK
    } else {
        BUTTON_SECONDARY_HOVER_BORDER_LIGHT
    }
}

pub(crate) const fn button_secondary_active_border(dark_mode: bool) -> Color32 {
    if dark_mode {
        BUTTON_SECONDARY_ACTIVE_BORDER_DARK
    } else {
        BUTTON_SECONDARY_ACTIVE_BORDER_LIGHT
    }
}

pub(crate) const fn switch_off_bg(dark_mode: bool) -> Color32 {
    if dark_mode {
        SWITCH_OFF_BG_DARK
    } else {
        SWITCH_OFF_BG_LIGHT
    }
}

pub(crate) const fn switch_knob_off(dark_mode: bool) -> Color32 {
    if dark_mode {
        SWITCH_KNOB_OFF_DARK
    } else {
        SWITCH_KNOB_OFF_LIGHT
    }
}

pub(crate) const fn primary_bg(dark_mode: bool) -> Color32 {
    if dark_mode {
        NEUTRAL.c100
    } else {
        NEUTRAL.c900
    }
}

pub(crate) const fn primary_hover_bg(dark_mode: bool) -> Color32 {
    if dark_mode {
        NEUTRAL.c200
    } else {
        NEUTRAL.c800
    }
}

pub(crate) const fn primary_active_bg(dark_mode: bool) -> Color32 {
    if dark_mode {
        NEUTRAL.c300
    } else {
        NEUTRAL.c700
    }
}

pub(crate) const fn primary_fg(dark_mode: bool) -> Color32 {
    if dark_mode {
        NEUTRAL.c900
    } else {
        NEUTRAL.c100
    }
}

pub(crate) const fn row_selected_bg(dark_mode: bool) -> Color32 {
    if dark_mode {
        NEUTRAL.c700
    } else {
        NEUTRAL.c200
    }
}

pub(crate) const fn row_selected_text(dark_mode: bool) -> Color32 {
    if dark_mode {
        NEUTRAL.c100
    } else {
        NEUTRAL.c900
    }
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

pub(crate) const fn text_selection_bg(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgba_premultiplied(59, 130, 246, 120)
    } else {
        Color32::from_rgba_premultiplied(37, 99, 235, 96)
    }
}

pub(crate) const fn input_bg(dark_mode: bool, focused: bool, hovered: bool) -> Color32 {
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

pub(crate) const fn row_bg(
    selected: bool,
    pressed: bool,
    hovered: bool,
    dark_mode: bool,
) -> Color32 {
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

pub(crate) const fn slider_track_inactive(dark_mode: bool) -> Color32 {
    if dark_mode {
        SLIDER_TRACK_INACTIVE_DARK
    } else {
        SLIDER_TRACK_INACTIVE_LIGHT
    }
}

pub(crate) const fn slider_track_active(dark_mode: bool) -> Color32 {
    if dark_mode {
        SLIDER_TRACK_ACTIVE_DARK
    } else {
        SLIDER_TRACK_ACTIVE_LIGHT
    }
}

pub(crate) const fn slider_thumb_fill(dark_mode: bool) -> Color32 {
    if dark_mode {
        SLIDER_THUMB_FILL_DARK
    } else {
        SLIDER_THUMB_FILL_LIGHT
    }
}

pub(crate) const fn slider_thumb_hover_fill(dark_mode: bool) -> Color32 {
    if dark_mode {
        SLIDER_THUMB_HOVER_FILL_DARK
    } else {
        SLIDER_THUMB_HOVER_FILL_LIGHT
    }
}

pub(crate) const fn slider_thumb_border(dark_mode: bool) -> Color32 {
    if dark_mode {
        SLIDER_THUMB_BORDER_DARK
    } else {
        SLIDER_THUMB_BORDER_LIGHT
    }
}

pub(crate) const fn slider_thumb_hover_border(dark_mode: bool) -> Color32 {
    if dark_mode {
        SLIDER_THUMB_HOVER_BORDER_DARK
    } else {
        SLIDER_THUMB_HOVER_BORDER_LIGHT
    }
}

pub(crate) const fn input_focus_border(dark_mode: bool) -> Color32 {
    if dark_mode {
        NEUTRAL.c400
    } else {
        NEUTRAL.c600
    }
}

pub(crate) fn input_focus_stroke(dark_mode: bool) -> Stroke {
    Stroke::new(1.1, input_focus_border(dark_mode))
}
