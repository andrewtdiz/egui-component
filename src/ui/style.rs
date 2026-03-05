use crate::ui::tokens;
use egui::{
    CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, Style, TextStyle, Visuals,
};

const SHOWCASE_FONT_REGULAR_FAMILY: &str = "component-showcase-geist-regular";
const SHOWCASE_FONT_SEMIBOLD_FAMILY: &str = "component-showcase-geist-semibold";
const SHOWCASE_FONT_BOLD_FAMILY: &str = "component-showcase-geist-bold";
const SHOWCASE_FONT_ITALIC_FAMILY: &str = "component-showcase-geist-italic";
const SHOWCASE_FONT_LIGHT_FAMILY: &str = "component-showcase-geist-light";
const SHOWCASE_FONT_REGULAR_DATA_KEY: &str = "component-showcase-geist-regular-data";
const SHOWCASE_FONT_SEMIBOLD_DATA_KEY: &str = "component-showcase-geist-semibold-data";
const SHOWCASE_FONT_BOLD_DATA_KEY: &str = "component-showcase-geist-bold-data";
const SHOWCASE_FONT_ITALIC_DATA_KEY: &str = "component-showcase-geist-italic-data";
const SHOWCASE_FONT_LIGHT_DATA_KEY: &str = "component-showcase-geist-light-data";

pub(crate) fn setup_showcase_context(context: &egui::Context) {
    context.set_fonts(showcase_font_definitions());
    let mut style = (*context.style()).clone();
    style.visuals = neutral_grayscale_visuals();
    apply_showcase_style_profile(&mut style);
    context.set_style(style);
}

fn showcase_font_definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        SHOWCASE_FONT_REGULAR_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Geist-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        SHOWCASE_FONT_SEMIBOLD_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Geist-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        SHOWCASE_FONT_BOLD_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Geist-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        SHOWCASE_FONT_ITALIC_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Geist-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        SHOWCASE_FONT_LIGHT_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Geist-Regular.ttf")).into(),
    );

    fonts.families.insert(
        FontFamily::Name(SHOWCASE_FONT_REGULAR_FAMILY.into()),
        vec![SHOWCASE_FONT_REGULAR_DATA_KEY.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(SHOWCASE_FONT_SEMIBOLD_FAMILY.into()),
        vec![SHOWCASE_FONT_SEMIBOLD_DATA_KEY.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(SHOWCASE_FONT_BOLD_FAMILY.into()),
        vec![SHOWCASE_FONT_BOLD_DATA_KEY.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(SHOWCASE_FONT_ITALIC_FAMILY.into()),
        vec![SHOWCASE_FONT_ITALIC_DATA_KEY.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(SHOWCASE_FONT_LIGHT_FAMILY.into()),
        vec![SHOWCASE_FONT_LIGHT_DATA_KEY.to_owned()],
    );

    if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
        proportional.insert(0, SHOWCASE_FONT_REGULAR_DATA_KEY.to_owned());
    }

    fonts
}

fn apply_showcase_style_profile(style: &mut Style) {
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(16.0, FontFamily::Name(SHOWCASE_FONT_BOLD_FAMILY.into())),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, FontFamily::Name(SHOWCASE_FONT_LIGHT_FAMILY.into())),
    );

    style.spacing.interact_size.y = tokens::SPACING_INTERACT_HEIGHT;
    style.spacing.item_spacing.y = tokens::SPACING_ITEM_Y;
    style.spacing.button_padding = egui::vec2(
        tokens::SPACING_BUTTON_PADDING_X,
        tokens::SPACING_BUTTON_PADDING_Y,
    );
    style.spacing.slider_width = 176.0;
    style.spacing.combo_width = 220.0;

    let corner_radius = CornerRadius::same(tokens::RADIUS_MD);
    style.visuals.widgets.noninteractive.corner_radius = corner_radius;
    style.visuals.widgets.inactive.corner_radius = corner_radius;
    style.visuals.widgets.hovered.corner_radius = corner_radius;
    style.visuals.widgets.active.corner_radius = corner_radius;
    style.visuals.widgets.open.corner_radius = corner_radius;
    style.visuals.widgets.hovered.expansion = 0.0;
    style.visuals.widgets.active.expansion = 0.0;
    style.visuals.handle_shape = egui::style::HandleShape::Circle;
    style.visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
}

fn neutral_grayscale_visuals() -> Visuals {
    let mut visuals = Visuals::dark();
    let dark_mode = visuals.dark_mode;
    visuals.override_text_color = Some(tokens::TEXT_PRIMARY);
    visuals.hyperlink_color = tokens::TEXT_SECONDARY;
    visuals.faint_bg_color = tokens::MUTED_SURFACE;
    visuals.extreme_bg_color = tokens::APP_BACKGROUND;
    visuals.code_bg_color = tokens::MUTED_SURFACE;
    visuals.warn_fg_color = tokens::TEXT_PRIMARY;
    visuals.error_fg_color = tokens::TEXT_DESTRUCTIVE;
    visuals.window_fill = tokens::CARD_BACKGROUND;
    visuals.panel_fill = tokens::APP_BACKGROUND;
    visuals.window_stroke = Stroke::new(1.0, tokens::SEPARATOR);
    visuals.selection.bg_fill = tokens::row_selected_bg(dark_mode);
    visuals.selection.stroke = Stroke::new(1.0, tokens::row_selected_border(dark_mode));
    visuals.widgets.noninteractive.bg_fill = tokens::CARD_BACKGROUND;
    visuals.widgets.noninteractive.weak_bg_fill = tokens::CARD_BACKGROUND;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, tokens::SEPARATOR);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, tokens::TEXT_SECONDARY);
    visuals.widgets.inactive.bg_fill = tokens::INPUT_BACKGROUND;
    visuals.widgets.inactive.weak_bg_fill = tokens::INPUT_BACKGROUND;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, tokens::INPUT_BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, tokens::TEXT_PRIMARY);
    visuals.widgets.hovered.bg_fill = tokens::INPUT_HOVER_BACKGROUND;
    visuals.widgets.hovered.weak_bg_fill = tokens::INPUT_HOVER_BACKGROUND;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, tokens::INPUT_HOVER_BORDER);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, tokens::TEXT_PRIMARY);
    visuals.widgets.active.bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
    visuals.widgets.active.weak_bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
    visuals.widgets.active.bg_stroke = tokens::input_focus_stroke(dark_mode);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, tokens::TEXT_PRIMARY);
    visuals.widgets.open.bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
    visuals.widgets.open.weak_bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
    visuals.widgets.open.bg_stroke = tokens::input_focus_stroke(dark_mode);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, tokens::TEXT_PRIMARY);
    visuals.menu_corner_radius = CornerRadius::same(tokens::RADIUS_MD);
    visuals.handle_shape = egui::style::HandleShape::Rect { aspect_ratio: 0.85 };
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
    visuals
}
