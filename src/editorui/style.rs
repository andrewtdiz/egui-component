use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, Style, TextStyle, Visuals};

const EDITOR_UI_FONT_REGULAR_FAMILY: &str = "clay-editor-segoe-regular";
const EDITOR_UI_FONT_SEMIBOLD_FAMILY: &str = "clay-editor-segoe-semibold";
const EDITOR_UI_FONT_BOLD_FAMILY: &str = "clay-editor-segoe-bold";
const EDITOR_UI_FONT_ITALIC_FAMILY: &str = "clay-editor-segoe-italic";
const EDITOR_UI_FONT_LIGHT_FAMILY: &str = "clay-editor-segoe-light";
const EDITOR_UI_FONT_REGULAR_DATA_KEY: &str = "clay-editor-segoe-regular-data";
const EDITOR_UI_FONT_SEMIBOLD_DATA_KEY: &str = "clay-editor-segoe-semibold-data";
const EDITOR_UI_FONT_BOLD_DATA_KEY: &str = "clay-editor-segoe-bold-data";
const EDITOR_UI_FONT_ITALIC_DATA_KEY: &str = "clay-editor-segoe-italic-data";
const EDITOR_UI_FONT_LIGHT_DATA_KEY: &str = "clay-editor-segoe-light-data";

pub(crate) fn setup_editor_context(context: &egui::Context) {
    context.set_fonts(editor_font_definitions());
    let base_style = (*context.style()).clone();
    let mut style = base_style;
    style.visuals = neutral_grayscale_visuals();
    apply_editor_style_profile(&mut style);
    context.set_style(style);
}

fn editor_font_definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        EDITOR_UI_FONT_REGULAR_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI.TTF")).into(),
    );
    fonts.font_data.insert(
        EDITOR_UI_FONT_SEMIBOLD_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI-Semibold.TTF")).into(),
    );
    fonts.font_data.insert(
        EDITOR_UI_FONT_BOLD_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI-Bold.TTF")).into(),
    );
    fonts.font_data.insert(
        EDITOR_UI_FONT_ITALIC_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI-Italic.TTF")).into(),
    );
    fonts.font_data.insert(
        EDITOR_UI_FONT_LIGHT_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI-Light.TTF")).into(),
    );

    fonts.families.insert(
        FontFamily::Name(EDITOR_UI_FONT_REGULAR_FAMILY.into()),
        vec![EDITOR_UI_FONT_REGULAR_DATA_KEY.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(EDITOR_UI_FONT_SEMIBOLD_FAMILY.into()),
        vec![EDITOR_UI_FONT_SEMIBOLD_DATA_KEY.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(EDITOR_UI_FONT_BOLD_FAMILY.into()),
        vec![EDITOR_UI_FONT_BOLD_DATA_KEY.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(EDITOR_UI_FONT_ITALIC_FAMILY.into()),
        vec![EDITOR_UI_FONT_ITALIC_DATA_KEY.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(EDITOR_UI_FONT_LIGHT_FAMILY.into()),
        vec![EDITOR_UI_FONT_LIGHT_DATA_KEY.to_owned()],
    );

    if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
        proportional.insert(0, EDITOR_UI_FONT_REGULAR_DATA_KEY.to_owned());
    }

    fonts
}

fn apply_editor_style_profile(style: &mut Style) {
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(12.0, FontFamily::Proportional));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::new(12.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(14.0, FontFamily::Name(EDITOR_UI_FONT_BOLD_FAMILY.into())),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(10.0, FontFamily::Name(EDITOR_UI_FONT_LIGHT_FAMILY.into())),
    );

    style.spacing.interact_size.y = 28.0;
    style.spacing.item_spacing.y = 5.0;
    style.spacing.button_padding = egui::vec2(8.0, 2.0);
    style.spacing.slider_width = 128.0;
    style.spacing.combo_width = 128.0;

    let corner_radius = CornerRadius::same(5);
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
    visuals.override_text_color = Some(Color32::from_gray(232));
    visuals.hyperlink_color = Color32::from_gray(200);
    visuals.faint_bg_color = Color32::from_gray(38);
    visuals.extreme_bg_color = Color32::from_gray(18);
    visuals.code_bg_color = Color32::from_gray(30);
    visuals.warn_fg_color = Color32::from_gray(220);
    visuals.error_fg_color = Color32::from_gray(210);
    visuals.window_fill = Color32::from_rgb(43, 43, 43);
    visuals.panel_fill = Color32::from_rgb(43, 43, 43);
    visuals.window_stroke = Stroke::new(1.0, Color32::from_gray(86));
    visuals.selection.bg_fill = Color32::from_gray(88);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_gray(230));
    visuals.widgets.noninteractive.bg_fill = Color32::from_gray(32);
    visuals.widgets.noninteractive.weak_bg_fill = Color32::from_gray(32);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_gray(68));
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_gray(206));
    visuals.widgets.inactive.bg_fill = Color32::from_gray(44);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(44);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_gray(84));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_gray(210));
    visuals.widgets.hovered.bg_fill = Color32::from_gray(62);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_gray(62);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_gray(102));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::from_gray(224));
    visuals.widgets.active.bg_fill = Color32::from_gray(78);
    visuals.widgets.active.weak_bg_fill = Color32::from_gray(78);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_gray(116));
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::from_gray(232));
    visuals.widgets.open.bg_fill = Color32::from_gray(54);
    visuals.widgets.open.weak_bg_fill = Color32::from_gray(54);
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, Color32::from_gray(94));
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, Color32::from_gray(220));
    visuals
}
