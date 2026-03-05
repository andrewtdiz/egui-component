use egui::{Color32, Stroke};

#[derive(Clone, Copy)]
pub(crate) struct ColorScale {
    pub(crate) c100: Color32,
    pub(crate) c200: Color32,
    pub(crate) c300: Color32,
    pub(crate) c400: Color32,
    pub(crate) c500: Color32,
    pub(crate) c600: Color32,
    pub(crate) c700: Color32,
    pub(crate) c800: Color32,
    pub(crate) c900: Color32,
}

impl ColorScale {
    pub(crate) const fn at(self, level: u16) -> Color32 {
        match level {
            100 => self.c100,
            200 => self.c200,
            300 => self.c300,
            400 => self.c400,
            500 => self.c500,
            600 => self.c600,
            700 => self.c700,
            800 => self.c800,
            _ => self.c900,
        }
    }
}

pub(crate) const NEUTRAL: ColorScale = ColorScale {
    c100: Color32::from_rgb(250, 250, 250),
    c200: Color32::from_rgb(228, 228, 231),
    c300: Color32::from_rgb(212, 212, 216),
    c400: Color32::from_rgb(161, 161, 170),
    c500: Color32::from_rgb(113, 113, 122),
    c600: Color32::from_rgb(82, 82, 91),
    c700: Color32::from_rgb(63, 63, 70),
    c800: Color32::from_rgb(39, 39, 42),
    c900: Color32::from_rgb(24, 24, 27),
};

pub(crate) const APP_BACKGROUND: Color32 = Color32::from_rgb(9, 9, 11);
pub(crate) const CARD_BACKGROUND: Color32 = Color32::from_rgb(17, 17, 20);
pub(crate) const MUTED_SURFACE: Color32 = Color32::from_rgb(24, 24, 27);
pub(crate) const SEPARATOR: Color32 = Color32::from_rgb(39, 39, 42);

pub(crate) const ROW_HOVER_BG: Color32 = Color32::from_rgb(39, 39, 42);

pub(crate) const TEXT_PRIMARY: Color32 = Color32::from_rgb(244, 244, 245);
pub(crate) const TEXT_SECONDARY: Color32 = Color32::from_rgb(212, 212, 216);
pub(crate) const TEXT_MUTED: Color32 = Color32::from_rgb(161, 161, 170);
pub(crate) const TEXT_DESTRUCTIVE: Color32 = Color32::from_rgb(248, 113, 113);
pub(crate) const GAME_ENGINE_GREEN: Color32 = Color32::from_rgb(34, 197, 94);
pub(crate) const GAME_ENGINE_RED: Color32 = Color32::from_rgb(239, 68, 68);

pub(crate) const INPUT_BACKGROUND: Color32 = Color32::from_rgb(24, 24, 27);
pub(crate) const INPUT_HOVER_BACKGROUND: Color32 = Color32::from_rgb(31, 31, 35);
pub(crate) const INPUT_FOCUS_BACKGROUND: Color32 = Color32::from_rgb(14, 14, 17);
pub(crate) const INPUT_BORDER: Color32 = Color32::from_rgb(63, 63, 70);
pub(crate) const INPUT_HOVER_BORDER: Color32 = Color32::from_rgb(82, 82, 91);

pub(crate) const BUTTON_SECONDARY_BG: Color32 = Color32::from_rgb(24, 24, 27);
pub(crate) const BUTTON_SECONDARY_HOVER_BG: Color32 = Color32::from_rgb(31, 31, 35);
pub(crate) const BUTTON_SECONDARY_ACTIVE_BG: Color32 = Color32::from_rgb(39, 39, 42);
pub(crate) const BUTTON_SECONDARY_BORDER: Color32 = Color32::from_rgb(63, 63, 70);
pub(crate) const BUTTON_SECONDARY_HOVER_BORDER: Color32 = Color32::from_rgb(82, 82, 91);
pub(crate) const BUTTON_SECONDARY_ACTIVE_BORDER: Color32 = Color32::from_rgb(113, 113, 122);

pub(crate) const SWITCH_OFF_BG: Color32 = Color32::from_rgb(82, 82, 91);
pub(crate) const SWITCH_BORDER: Color32 = Color32::from_rgb(113, 113, 122);
pub(crate) const SWITCH_KNOB_OFF: Color32 = Color32::from_rgb(244, 244, 245);

pub(crate) const SPACING_ITEM_Y: f32 = 8.0;
pub(crate) const SPACING_BUTTON_PADDING_X: f32 = 12.0;
pub(crate) const SPACING_BUTTON_PADDING_Y: f32 = 7.0;
pub(crate) const SPACING_INTERACT_HEIGHT: f32 = 34.0;
pub(crate) const INPUT_PADDING_X: i8 = 10;
pub(crate) const INPUT_PADDING_Y: i8 = 6;

pub(crate) const RADIUS_SM: u8 = 6;
pub(crate) const RADIUS_MD: u8 = 8;
pub(crate) const RADIUS_LG: u8 = 10;

pub(crate) const fn neutral(level: u16) -> Color32 {
    NEUTRAL.at(level)
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

pub(crate) const fn row_selected_border(dark_mode: bool) -> Color32 {
    if dark_mode {
        NEUTRAL.c500
    } else {
        NEUTRAL.c400
    }
}

pub(crate) const fn row_selected_text(dark_mode: bool) -> Color32 {
    if dark_mode {
        NEUTRAL.c100
    } else {
        NEUTRAL.c900
    }
}

pub(crate) const fn text_selection_bg(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgba_premultiplied(59, 130, 246, 120)
    } else {
        Color32::from_rgba_premultiplied(37, 99, 235, 96)
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
