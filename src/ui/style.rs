use crate::ui::{tokens, typography};
use egui::{
    CornerRadius, FontData, FontDefinitions, FontFamily, Stroke, Style, TextStyle, Ui, Visuals,
};

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
    spacing.menu_margin = egui::Margin::symmetric(0, 2);
    spacing.menu_spacing = 0.0;

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
        typography::REGULAR_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI.TTF")).into(),
    );
    fonts.font_data.insert(
        typography::SEMIBOLD_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI-Semibold.TTF")).into(),
    );
    fonts.font_data.insert(
        typography::BOLD_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI-Bold.TTF")).into(),
    );
    fonts.font_data.insert(
        typography::ITALIC_DATA_KEY.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Segoe-UI-Italic.TTF")).into(),
    );

    fonts.families.insert(
        FontFamily::Name(typography::REGULAR_FAMILY.into()),
        vec![typography::REGULAR_DATA_KEY.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(typography::SEMIBOLD_FAMILY.into()),
        vec![typography::SEMIBOLD_DATA_KEY.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(typography::BOLD_FAMILY.into()),
        vec![typography::BOLD_DATA_KEY.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name(typography::ITALIC_FAMILY.into()),
        vec![typography::ITALIC_DATA_KEY.to_owned()],
    );

    if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
        proportional.insert(0, typography::REGULAR_DATA_KEY.to_owned());
    }

    fonts
}

fn apply_showcase_style_profile(style: &mut Style) {
    style
        .text_styles
        .insert(TextStyle::Body, typography::body_font());
    style
        .text_styles
        .insert(TextStyle::Button, typography::body_font());
    style
        .text_styles
        .insert(TextStyle::Heading, typography::heading_font());
    style
        .text_styles
        .insert(TextStyle::Small, typography::small_font());

    style.spacing.interact_size.y = tokens::SPACING_INTERACT_HEIGHT;
    style.spacing.item_spacing.y = tokens::SPACING_ITEM_Y;
    style.spacing.button_padding = egui::vec2(
        tokens::SPACING_BUTTON_PADDING_X,
        tokens::SPACING_BUTTON_PADDING_Y,
    );
    style.spacing.slider_width = 176.0;
    style.spacing.combo_width = 220.0;
    style.spacing.menu_margin = egui::Margin::symmetric(0, 2);
    style.spacing.menu_spacing = 0.0;
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

#[cfg(test)]
mod tests {
    use super::{setup_showcase_context, showcase_font_definitions};
    use crate::ui::typography;
    use egui::{Context, TextStyle};

    #[test]
    fn showcase_fonts_only_register_segoe_proportional_families() {
        let fonts = showcase_font_definitions();

        assert!(fonts.font_data.contains_key(typography::REGULAR_DATA_KEY));
        assert!(fonts.font_data.contains_key(typography::SEMIBOLD_DATA_KEY));
        assert!(fonts.font_data.contains_key(typography::BOLD_DATA_KEY));
        assert!(fonts.font_data.contains_key(typography::ITALIC_DATA_KEY));
        assert!(!fonts
            .font_data
            .keys()
            .any(|key| key.contains("geist") || key.contains("light")));
    }

    #[test]
    fn showcase_style_uses_shared_text_defaults() {
        let context = Context::default();
        setup_showcase_context(&context);

        let style = context.style();
        assert_eq!(style.text_styles[&TextStyle::Body], typography::body_font());
        assert_eq!(
            style.text_styles[&TextStyle::Button],
            typography::body_font()
        );
        assert_eq!(
            style.text_styles[&TextStyle::Heading],
            typography::heading_font()
        );
        assert_eq!(
            style.text_styles[&TextStyle::Small],
            typography::small_font()
        );
    }
}
