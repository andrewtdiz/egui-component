use egui::{FontFamily, FontId};

pub(crate) const REGULAR_FAMILY: &str = "egui-component-segoe-regular";
pub(crate) const SEMIBOLD_FAMILY: &str = "egui-component-segoe-semibold";
pub(crate) const BOLD_FAMILY: &str = "egui-component-segoe-bold";
pub(crate) const ITALIC_FAMILY: &str = "egui-component-segoe-italic";

pub(crate) const REGULAR_DATA_KEY: &str = "egui-component-segoe-regular-data";
pub(crate) const SEMIBOLD_DATA_KEY: &str = "egui-component-segoe-semibold-data";
pub(crate) const BOLD_DATA_KEY: &str = "egui-component-segoe-bold-data";
pub(crate) const ITALIC_DATA_KEY: &str = "egui-component-segoe-italic-data";

pub(crate) const BODY_SIZE: f32 = 14.0;
pub(crate) const HEADING_SIZE: f32 = 16.0;
pub(crate) const LABEL_SIZE: f32 = 12.0;
pub(crate) const SMALL_SIZE: f32 = 12.0;

pub(crate) fn proportional(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub(crate) fn body_font() -> FontId {
    proportional(BODY_SIZE)
}

pub(crate) fn heading_font() -> FontId {
    bold_font(HEADING_SIZE)
}

pub(crate) fn label_font() -> FontId {
    proportional(LABEL_SIZE)
}

pub(crate) fn small_font() -> FontId {
    proportional(SMALL_SIZE)
}

pub(crate) fn semibold_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD_FAMILY.into()))
}

pub(crate) fn bold_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(BOLD_FAMILY.into()))
}
