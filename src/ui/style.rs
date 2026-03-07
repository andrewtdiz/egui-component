use crate::ui::tokens;
use egui::{
    CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, Style, TextStyle, Ui,
    Visuals,
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
    set_showcase_dark_mode(context, true);
}

pub(crate) fn set_showcase_dark_mode(context: &egui::Context, dark_mode: bool) {
    let mut style = (*context.style()).clone();
    style.visuals = neutral_grayscale_visuals(dark_mode);
    apply_showcase_style_profile(&mut style);
    context.set_style(style);
}

pub(crate) fn apply_component_theme(ui: &mut Ui) {
    let dark_mode = ui.visuals().dark_mode;
    let spacing = ui.spacing_mut();
    spacing.item_spacing.y = tokens::SPACING_ITEM_Y;
    spacing.button_padding = egui::vec2(
        tokens::SPACING_BUTTON_PADDING_X,
        tokens::SPACING_BUTTON_PADDING_Y,
    );
    spacing.interact_size.y = tokens::SPACING_INTERACT_HEIGHT;
    spacing.menu_margin = egui::Margin::same(4);

    let style = ui.style_mut();
    style.interaction.selectable_labels = false;
    style.interaction.multi_widget_text_select = false;
    let visuals = &mut style.visuals;
    visuals.selection.bg_fill = tokens::row_selected_bg(dark_mode);
    visuals.selection.stroke = Stroke::NONE;
    visuals.popup_shadow = tokens::tailwind_shadow_md();
    let corner = CornerRadius::same(tokens::RADIUS_MD);
    visuals.widgets.noninteractive.corner_radius = corner;
    visuals.widgets.inactive.corner_radius = corner;
    visuals.widgets.hovered.corner_radius = corner;
    visuals.widgets.active.corner_radius = corner;
    visuals.widgets.open.corner_radius = corner;
    visuals.menu_corner_radius = corner;
}

fn showcase_font_definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        SHOWCASE_FONT_REGULAR_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Geist-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        SHOWCASE_FONT_SEMIBOLD_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI-Semibold.TTF")).into(),
    );
    fonts.font_data.insert(
        SHOWCASE_FONT_BOLD_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI-Bold.TTF")).into(),
    );
    fonts.font_data.insert(
        SHOWCASE_FONT_ITALIC_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI-Italic.TTF")).into(),
    );
    fonts.font_data.insert(
        SHOWCASE_FONT_LIGHT_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI-Light.TTF")).into(),
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
    style.spacing.menu_margin = egui::Margin::same(4);
    style.interaction.selectable_labels = false;
    style.interaction.multi_widget_text_select = false;

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

fn neutral_grayscale_visuals(dark_mode: bool) -> Visuals {
    let mut visuals = if dark_mode {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    visuals.override_text_color = Some(tokens::text_primary(dark_mode));
    visuals.hyperlink_color = tokens::text_secondary(dark_mode);
    visuals.faint_bg_color = tokens::muted_surface(dark_mode);
    visuals.extreme_bg_color = tokens::app_background(dark_mode);
    visuals.code_bg_color = tokens::muted_surface(dark_mode);
    visuals.warn_fg_color = tokens::text_primary(dark_mode);
    visuals.error_fg_color = tokens::text_destructive(dark_mode);
    visuals.window_fill = tokens::card_background(dark_mode);
    visuals.panel_fill = tokens::app_background(dark_mode);
    visuals.window_stroke = Stroke::new(1.0, tokens::separator(dark_mode));
    visuals.selection.bg_fill = tokens::row_selected_bg(dark_mode);
    visuals.selection.stroke = Stroke::NONE;
    visuals.popup_shadow = tokens::tailwind_shadow_md();
    visuals.widgets.noninteractive.bg_fill = tokens::card_background(dark_mode);
    visuals.widgets.noninteractive.weak_bg_fill = tokens::card_background(dark_mode);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, tokens::separator(dark_mode));
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, tokens::text_secondary(dark_mode));
    visuals.widgets.inactive.bg_fill = tokens::input_background(dark_mode);
    visuals.widgets.inactive.weak_bg_fill = tokens::input_background(dark_mode);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, tokens::input_border(dark_mode));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, tokens::text_primary(dark_mode));
    visuals.widgets.hovered.bg_fill = tokens::input_hover_background(dark_mode);
    visuals.widgets.hovered.weak_bg_fill = tokens::input_hover_background(dark_mode);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, tokens::input_hover_border(dark_mode));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, tokens::text_primary(dark_mode));
    visuals.widgets.active.bg_fill = tokens::input_focus_background(dark_mode);
    visuals.widgets.active.weak_bg_fill = tokens::input_focus_background(dark_mode);
    visuals.widgets.active.bg_stroke = tokens::input_focus_stroke(dark_mode);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, tokens::text_primary(dark_mode));
    visuals.widgets.open.bg_fill = tokens::input_focus_background(dark_mode);
    visuals.widgets.open.weak_bg_fill = tokens::input_focus_background(dark_mode);
    visuals.widgets.open.bg_stroke = tokens::input_focus_stroke(dark_mode);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, tokens::text_primary(dark_mode));
    visuals.menu_corner_radius = CornerRadius::same(tokens::RADIUS_MD);
    visuals.handle_shape = egui::style::HandleShape::Rect { aspect_ratio: 0.85 };
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
    visuals
}
