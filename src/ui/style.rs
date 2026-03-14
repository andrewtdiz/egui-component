use crate::theme::ThemeMode;
use crate::ui::{tokens, typography};
use egui::{
    CornerRadius, FontData, FontDefinitions, FontFamily, Stroke, Style, TextStyle, Ui, Visuals,
};

pub(crate) fn install(context: &egui::Context, mode: ThemeMode) {
    context.set_fonts(component_font_definitions());
    set_mode(context, mode);
}

pub(crate) fn set_mode(context: &egui::Context, mode: ThemeMode) {
    let mut style = (*context.style()).clone();
    style.visuals = mode_visuals(mode);
    apply_typography(&mut style);
    apply_component_style_profile(&mut style);
    context.set_style(style);
}

pub(crate) fn apply_component_profile(ui: &mut Ui) {
    let style = ui.style_mut();
    apply_component_style_profile(style);
}

fn component_font_definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let proportional_fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
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
        component_font_family_with_fallback(typography::REGULAR_DATA_KEY, &proportional_fallback),
    );
    fonts.families.insert(
        FontFamily::Name(typography::SEMIBOLD_FAMILY.into()),
        component_font_family_with_fallback(typography::SEMIBOLD_DATA_KEY, &proportional_fallback),
    );
    fonts.families.insert(
        FontFamily::Name(typography::BOLD_FAMILY.into()),
        component_font_family_with_fallback(typography::BOLD_DATA_KEY, &proportional_fallback),
    );
    fonts.families.insert(
        FontFamily::Name(typography::ITALIC_FAMILY.into()),
        component_font_family_with_fallback(typography::ITALIC_DATA_KEY, &proportional_fallback),
    );

    if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
        proportional.insert(0, typography::REGULAR_DATA_KEY.to_owned());
    }

    fonts
}

fn component_font_family_with_fallback(
    primary_font_key: &str,
    fallback_family: &[String],
) -> Vec<String> {
    let mut family = Vec::with_capacity(1 + fallback_family.len());
    family.push(primary_font_key.to_owned());
    family.extend(fallback_family.iter().cloned());
    family
}

fn apply_typography(style: &mut Style) {
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
}

fn apply_component_style_profile(style: &mut Style) {
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
    style.visuals.selection.bg_fill = tokens::row_selected_bg(style.visuals.dark_mode);
    style.visuals.selection.stroke = Stroke::NONE;
    style.visuals.popup_shadow = tokens::tailwind_shadow_md();
    style.visuals.widgets.noninteractive.corner_radius = corner_radius;
    style.visuals.widgets.inactive.corner_radius = corner_radius;
    style.visuals.widgets.hovered.corner_radius = corner_radius;
    style.visuals.widgets.active.corner_radius = corner_radius;
    style.visuals.widgets.open.corner_radius = corner_radius;
    style.visuals.widgets.hovered.expansion = 0.0;
    style.visuals.widgets.active.expansion = 0.0;
    style.visuals.menu_corner_radius = corner_radius;
    style.visuals.handle_shape = egui::style::HandleShape::Circle;
    style.visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
}

fn mode_visuals(mode: ThemeMode) -> Visuals {
    let dark_mode = mode.is_dark();
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
    use super::{component_font_definitions, install, set_mode};
    use crate::theme::ThemeMode;
    use crate::ui::{tokens, typography};
    use egui::{Context, TextStyle};

    #[test]
    fn component_fonts_only_register_segoe_proportional_families() {
        let fonts = component_font_definitions();

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
    fn installed_style_uses_shared_text_defaults() {
        let context = Context::default();
        install(&context, ThemeMode::Dark);

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

    #[test]
    fn switching_theme_mode_updates_context_visuals() {
        let context = Context::default();
        install(&context, ThemeMode::Light);

        assert_eq!(
            context.style().visuals.panel_fill,
            tokens::app_background(false)
        );

        set_mode(&context, ThemeMode::Dark);

        assert_eq!(
            context.style().visuals.panel_fill,
            tokens::app_background(true)
        );
    }
}
