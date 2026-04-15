use crate::ui::tailwind::types::TextSize;

pub const SPACING_UNIT: f32 = 4.0;
pub const DIMENSION_UNIT: f32 = 4.0;
pub const BORDER_WIDTH_DEFAULT: f32 = 1.0;
pub const Z_INDEX_DEFAULT: i16 = 0;

#[derive(Clone, Copy)]
pub struct RadiusToken {
    pub token: &'static str,
    pub radius: f32,
}

pub const RADIUS_TOKENS: &[RadiusToken] = &[
    RadiusToken {
        token: "rounded-none",
        radius: 0.0,
    },
    RadiusToken {
        token: "rounded-sm",
        radius: 2.0,
    },
    RadiusToken {
        token: "rounded",
        radius: 4.0,
    },
    RadiusToken {
        token: "rounded-md",
        radius: 6.0,
    },
    RadiusToken {
        token: "rounded-lg",
        radius: 8.0,
    },
    RadiusToken {
        token: "rounded-xl",
        radius: 12.0,
    },
    RadiusToken {
        token: "rounded-2xl",
        radius: 16.0,
    },
    RadiusToken {
        token: "rounded-3xl",
        radius: 24.0,
    },
    RadiusToken {
        token: "rounded-full",
        radius: 9999.0,
    },
];

#[derive(Clone, Copy)]
pub struct ZLayerToken {
    pub token: &'static str,
    pub value: i16,
}

pub const Z_LAYER_TOKENS: &[ZLayerToken] = &[
    ZLayerToken {
        token: "base",
        value: 0,
    },
    ZLayerToken {
        token: "dropdown",
        value: 100,
    },
    ZLayerToken {
        token: "overlay",
        value: 200,
    },
    ZLayerToken {
        token: "modal",
        value: 300,
    },
    ZLayerToken {
        token: "popover",
        value: 400,
    },
    ZLayerToken {
        token: "tooltip",
        value: 500,
    },
];

#[derive(Clone, Copy)]
pub struct TypographyToken {
    pub token: &'static str,
    pub text_size: TextSize,
}

pub const TYPOGRAPHY_TOKENS: &[TypographyToken] = &[
    TypographyToken {
        token: "text-xs",
        text_size: TextSize::Xs,
    },
    TypographyToken {
        token: "text-sm",
        text_size: TextSize::Sm,
    },
    TypographyToken {
        token: "text-base",
        text_size: TextSize::Base,
    },
    TypographyToken {
        token: "text-lg",
        text_size: TextSize::Lg,
    },
    TypographyToken {
        token: "text-xl",
        text_size: TextSize::Xl,
    },
    TypographyToken {
        token: "text-2xl",
        text_size: TextSize::X2l,
    },
    TypographyToken {
        token: "text-3xl",
        text_size: TextSize::X3l,
    },
];
