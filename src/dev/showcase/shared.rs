use crate::ui::tokens;
use egui::{CornerRadius, Stroke, Ui};

pub(super) fn apply_showcase_component_theme(ui: &mut Ui) {
    let dark_mode = ui.visuals().dark_mode;
    let spacing = ui.spacing_mut();
    spacing.item_spacing.y = tokens::SPACING_ITEM_Y;
    spacing.button_padding = egui::vec2(
        tokens::SPACING_BUTTON_PADDING_X,
        tokens::SPACING_BUTTON_PADDING_Y,
    );
    spacing.interact_size.y = tokens::SPACING_INTERACT_HEIGHT;

    let style = ui.style_mut();
    style.interaction.selectable_labels = false;
    style.interaction.multi_widget_text_select = false;
    let visuals = &mut style.visuals;
    visuals.selection.bg_fill = tokens::row_selected_bg(dark_mode);
    visuals.selection.stroke = Stroke::new(1.0, tokens::row_selected_border(dark_mode));
    let corner = CornerRadius::same(tokens::RADIUS_MD);
    visuals.widgets.noninteractive.corner_radius = corner;
    visuals.widgets.inactive.corner_radius = corner;
    visuals.widgets.hovered.corner_radius = corner;
    visuals.widgets.active.corner_radius = corner;
    visuals.widgets.open.corner_radius = corner;
    visuals.menu_corner_radius = corner;
}
