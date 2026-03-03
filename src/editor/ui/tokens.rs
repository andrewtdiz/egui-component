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
    c100: Color32::from_rgb(245, 245, 245),
    c200: Color32::from_rgb(229, 229, 229),
    c300: Color32::from_rgb(212, 212, 212),
    c400: Color32::from_rgb(163, 163, 163),
    c500: Color32::from_rgb(115, 115, 115),
    c600: Color32::from_rgb(82, 82, 82),
    c700: Color32::from_rgb(64, 64, 64),
    c800: Color32::from_rgb(38, 38, 38),
    c900: Color32::from_rgb(23, 23, 23),
};

pub(crate) const BLUE: ColorScale = ColorScale {
    c100: Color32::from_rgb(219, 234, 254),
    c200: Color32::from_rgb(191, 219, 254),
    c300: Color32::from_rgb(147, 197, 253),
    c400: Color32::from_rgb(96, 165, 250),
    c500: Color32::from_rgb(59, 130, 246),
    c600: Color32::from_rgb(37, 99, 235),
    c700: Color32::from_rgb(29, 78, 216),
    c800: Color32::from_rgb(30, 64, 175),
    c900: Color32::from_rgb(30, 58, 138),
};

pub(crate) const GREEN: ColorScale = ColorScale {
    c100: Color32::from_rgb(220, 252, 231),
    c200: Color32::from_rgb(187, 247, 208),
    c300: Color32::from_rgb(134, 239, 172),
    c400: Color32::from_rgb(74, 222, 128),
    c500: Color32::from_rgb(34, 197, 94),
    c600: Color32::from_rgb(22, 163, 74),
    c700: Color32::from_rgb(21, 128, 61),
    c800: Color32::from_rgb(22, 101, 52),
    c900: Color32::from_rgb(20, 83, 45),
};

pub(crate) const APP_BACKGROUND: Color32 = NEUTRAL.c900;
pub(crate) const SIDEBAR_BACKGROUND: Color32 = Color32::from_rgb(39, 39, 42);
pub(crate) const CARD_BACKGROUND: Color32 = Color32::from_rgb(41, 41, 45);
pub(crate) const CARD_BORDER: Color32 = Color32::from_rgb(58, 58, 64);
pub(crate) const MUTED_SURFACE: Color32 = Color32::from_rgb(36, 36, 40);

pub(crate) const TEXT_PRIMARY: Color32 = Color32::from_rgb(234, 234, 237);
pub(crate) const TEXT_SECONDARY: Color32 = Color32::from_rgb(186, 186, 192);
pub(crate) const TEXT_MUTED: Color32 = Color32::from_rgb(145, 145, 151);

pub(crate) const INPUT_BACKGROUND: Color32 = Color32::from_gray(47);
pub(crate) const INPUT_HOVER_BACKGROUND: Color32 = Color32::from_gray(47);
pub(crate) const INPUT_FOCUS_BACKGROUND: Color32 = Color32::from_rgb(36, 36, 36);
pub(crate) const INPUT_BORDER: Color32 = Color32::from_rgb(74, 74, 80);
pub(crate) const INPUT_HOVER_BORDER: Color32 = Color32::from_rgb(88, 88, 95);
pub(crate) const INPUT_FOCUS_BORDER: Color32 = Color32::from_rgb(13, 153, 255);

pub(crate) const fn neutral(level: u16) -> Color32 {
    NEUTRAL.at(level)
}

pub(crate) const fn blue(level: u16) -> Color32 {
    BLUE.at(level)
}

pub(crate) const fn green(level: u16) -> Color32 {
    GREEN.at(level)
}

pub(crate) fn input_focus_stroke() -> Stroke {
    Stroke::new(1.0, INPUT_FOCUS_BORDER)
}
