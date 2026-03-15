use super::api::ComponentUi;
use crate::ui::{icons, tokens, typography};
use egui::{Align2, CornerRadius, CursorIcon, Id, Rect, RichText, Stroke, StrokeKind, Ui};

const INLINE_TAB_GAP: f32 = 3.0;
const INLINE_TAB_HEIGHT: f32 = 24.0;
const INLINE_TAB_MIN_WIDTH: f32 = 48.0;
const INLINE_TAB_PADDING_X: f32 = 11.0;
const INLINE_TAB_FRAME_PADDING: i8 = 3;
const STACKED_TAB_SIZE: egui::Vec2 = egui::vec2(80.0, 68.0);
const STACKED_TAB_GAP: f32 = 8.0;
const STACKED_TAB_ICON_SIZE: f32 = 18.0;
const RAIL_TAB_SIZE: egui::Vec2 = egui::vec2(62.0, 54.0);
const RAIL_TAB_GAP: f32 = 4.0;
const RAIL_TAB_ICON_SIZE: f32 = 16.0;
const RAIL_TAB_ICON_OFFSET_Y: f32 = -9.0;
const RAIL_TAB_LABEL_OFFSET_Y: f32 = 10.0;

#[derive(Debug, Clone, Copy)]
pub struct TabOption<'a> {
    pub value: usize,
    pub label: &'a str,
    pub icon: Option<&'a str>,
}

impl<'a> TabOption<'a> {
    pub const fn new(value: usize, label: &'a str) -> Self {
        Self {
            value,
            label,
            icon: None,
        }
    }

    pub const fn with_icon(value: usize, label: &'a str, icon: &'a str) -> Self {
        Self {
            value,
            label,
            icon: Some(icon),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TabsVariant {
    #[default]
    Underline,
    Segmented,
}

impl ComponentUi<'_> {
    pub fn tabs(&mut self, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
        draw_tabs(self.raw_mut(), id, current, options);
    }

    pub fn tabs_variant(
        &mut self,
        id: Id,
        current: &mut usize,
        options: &[TabOption<'_>],
        variant: TabsVariant,
    ) {
        match variant {
            TabsVariant::Underline => draw_tabs(self.raw_mut(), id, current, options),
            TabsVariant::Segmented => {
                let _ = draw_segmented_tabs(self.raw_mut(), id, current, options);
            }
        }
    }

    pub fn segmented_tabs(&mut self, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
        let _ = draw_segmented_tabs(self.raw_mut(), id, current, options);
    }

    pub fn stacked_tabs(&mut self, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
        let _ = draw_stacked_tabs(self.raw_mut(), id, current, options);
    }

    pub fn rail_tabs(&mut self, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
        let _ = draw_rail_tabs(self.raw_mut(), id, current, options);
    }

    pub fn toggle_rail_tabs(
        &mut self,
        id: Id,
        current: &mut Option<usize>,
        options: &[TabOption<'_>],
    ) {
        let _ = draw_toggle_rail_tabs(self.raw_mut(), id, current, options);
    }
}

fn draw_tabs(ui: &mut Ui, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
    if options.is_empty() {
        return;
    }
    let dark_mode = ui.visuals().dark_mode;

    ui.push_id(id, |ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.horizontal(|ui| {
            for option in options {
                let selected = *current == option.value;
                let response = ui
                    .add(
                        egui::Button::new(RichText::new(option.label).color(if selected {
                            tokens::row_selected_text(dark_mode)
                        } else {
                            tokens::text_secondary(dark_mode)
                        }))
                        .fill(tokens::TRANSPARENT)
                        .stroke(Stroke::new(1.0, tokens::TRANSPARENT))
                        .corner_radius(CornerRadius::ZERO)
                        .min_size(egui::vec2(0.0, ui.spacing().interact_size.y)),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand);

                if selected {
                    let y = response.rect.bottom() - 1.0;
                    ui.painter().line_segment(
                        [
                            egui::pos2(response.rect.left() + 4.0, y),
                            egui::pos2(response.rect.right() - 4.0, y),
                        ],
                        Stroke::new(2.6, tokens::text_secondary(dark_mode)),
                    );
                }

                if response.clicked() && !selected {
                    *current = option.value;
                }
            }
        });
    });
}

fn draw_segmented_tabs(
    ui: &mut Ui,
    id: Id,
    current: &mut usize,
    options: &[TabOption<'_>],
) -> Rect {
    if options.is_empty() {
        return Rect::NOTHING;
    }

    let dark_mode = ui.visuals().dark_mode;
    let label_font = typography::label_font();

    ui.push_id(id, |ui| {
        egui::Frame::new()
            .fill(inline_tabs_group_fill(dark_mode))
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(tokens::RADIUS_MD))
            .inner_margin(egui::Margin::same(INLINE_TAB_FRAME_PADDING))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = INLINE_TAB_GAP;
                ui.horizontal(|ui| {
                    for option in options {
                        let selected = *current == option.value;
                        let label_width = ui.fonts_mut(|fonts| {
                            fonts
                                .layout_no_wrap(
                                    option.label.to_owned(),
                                    label_font.clone(),
                                    tokens::text_primary(dark_mode),
                                )
                                .size()
                                .x
                        });
                        let width =
                            (label_width + (INLINE_TAB_PADDING_X * 2.0)).max(INLINE_TAB_MIN_WIDTH);
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(width, INLINE_TAB_HEIGHT),
                            egui::Sense::click(),
                        );

                        let fill = if selected {
                            inline_tabs_selected_fill(dark_mode)
                        } else if response.is_pointer_button_down_on() {
                            inline_tabs_pressed_fill(dark_mode)
                        } else if response.hovered() {
                            inline_tabs_hover_fill(dark_mode)
                        } else {
                            tokens::TRANSPARENT
                        };

                        ui.painter().rect(
                            rect,
                            CornerRadius::same(tokens::RADIUS_SM),
                            fill,
                            Stroke::NONE,
                            StrokeKind::Outside,
                        );

                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            option.label,
                            label_font.clone(),
                            if selected {
                                inline_tabs_selected_text(dark_mode)
                            } else {
                                tokens::text_secondary(dark_mode)
                            },
                        );

                        if response.clicked() && !selected {
                            *current = option.value;
                        }

                        let _ = response.on_hover_cursor(CursorIcon::PointingHand);
                    }
                });
            })
            .response
            .rect
    })
    .inner
}

fn inline_tabs_group_fill(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        tokens::row_active_bg(dark_mode)
    } else {
        tokens::muted_surface(dark_mode)
    }
}

fn inline_tabs_selected_fill(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        tokens::card_background(dark_mode)
    } else {
        tokens::primary_bg(dark_mode)
    }
}

fn inline_tabs_hover_fill(dark_mode: bool) -> egui::Color32 {
    tokens::row_hover_bg(dark_mode)
}

fn inline_tabs_pressed_fill(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        tokens::input_background(dark_mode)
    } else {
        tokens::row_active_bg(dark_mode)
    }
}

fn inline_tabs_selected_text(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        tokens::text_primary(dark_mode)
    } else {
        tokens::primary_fg(dark_mode)
    }
}

fn draw_stacked_tabs(ui: &mut Ui, id: Id, current: &mut usize, options: &[TabOption<'_>]) -> Rect {
    if options.is_empty() {
        return Rect::NOTHING;
    }

    let dark_mode = ui.visuals().dark_mode;
    *current = (*current).min(options.len().saturating_sub(1));

    ui.push_id(id, |ui| {
        ui.spacing_mut().item_spacing.x = STACKED_TAB_GAP;
        ui.horizontal(|ui| {
            for option in options {
                let selected = *current == option.value;
                let (rect, response) =
                    ui.allocate_exact_size(STACKED_TAB_SIZE, egui::Sense::click());

                let fill = if response.is_pointer_button_down_on() {
                    tokens::button_secondary_active_bg(dark_mode)
                } else if selected || response.hovered() {
                    tokens::button_secondary_hover_bg(dark_mode)
                } else {
                    tokens::TRANSPARENT
                };
                ui.painter().rect(
                    rect,
                    CornerRadius::same(tokens::RADIUS_SM),
                    fill,
                    Stroke::NONE,
                    StrokeKind::Outside,
                );

                if let Some(icon) = option.icon {
                    if let Some(image) = icons::image(ui.ctx(), icon, STACKED_TAB_ICON_SIZE) {
                        let icon_rect = Rect::from_center_size(
                            egui::pos2(rect.center().x, rect.top() + 18.0),
                            egui::vec2(STACKED_TAB_ICON_SIZE, STACKED_TAB_ICON_SIZE),
                        );
                        // Paint directly so the icon never participates in layout after the
                        // tile rect has already been allocated.
                        image
                            .tint(if selected {
                                tokens::text_primary(dark_mode)
                            } else {
                                tokens::text_secondary(dark_mode)
                            })
                            .paint_at(ui, icon_rect);
                    }
                }

                ui.painter().text(
                    egui::pos2(rect.center().x, rect.bottom() - 18.0),
                    Align2::CENTER_CENTER,
                    option.label,
                    typography::label_font(),
                    if selected {
                        tokens::text_primary(dark_mode)
                    } else {
                        tokens::text_secondary(dark_mode)
                    },
                );

                if response.clicked() && !selected {
                    *current = option.value;
                }

                let _ = response.on_hover_cursor(CursorIcon::PointingHand);
            }
        })
        .response
        .rect
    })
    .inner
}

fn draw_rail_tabs(ui: &mut Ui, id: Id, current: &mut usize, options: &[TabOption<'_>]) -> Rect {
    if options.is_empty() {
        return Rect::NOTHING;
    }

    let dark_mode = ui.visuals().dark_mode;
    *current = (*current).min(options.len().saturating_sub(1));

    ui.push_id(id, |ui| {
        ui.spacing_mut().item_spacing.y = RAIL_TAB_GAP;
        ui.vertical(|ui| {
            for option in options {
                let selected = *current == option.value;
                let (rect, response) = ui.allocate_exact_size(RAIL_TAB_SIZE, egui::Sense::click());
                let icon_and_text_color = if selected {
                    tokens::text_primary(dark_mode)
                } else if response.hovered() {
                    tokens::text_secondary(dark_mode)
                } else {
                    tokens::text_muted(dark_mode)
                };

                let fill = if response.is_pointer_button_down_on() {
                    tokens::button_secondary_active_bg(dark_mode)
                } else if selected {
                    tokens::row_active_bg(dark_mode)
                } else if response.hovered() {
                    tokens::button_secondary_hover_bg(dark_mode)
                } else {
                    tokens::TRANSPARENT
                };

                ui.painter().rect(
                    rect,
                    CornerRadius::same(tokens::RADIUS_SM),
                    fill,
                    Stroke::NONE,
                    StrokeKind::Outside,
                );

                if let Some(icon) = option.icon {
                    if let Some(image) = icons::image(ui.ctx(), icon, RAIL_TAB_ICON_SIZE) {
                        let icon_rect = Rect::from_center_size(
                            egui::pos2(rect.center().x, rect.center().y + RAIL_TAB_ICON_OFFSET_Y),
                            egui::vec2(RAIL_TAB_ICON_SIZE, RAIL_TAB_ICON_SIZE),
                        );
                        image.tint(icon_and_text_color).paint_at(ui, icon_rect);
                    }
                }

                ui.painter().text(
                    egui::pos2(rect.center().x, rect.center().y + RAIL_TAB_LABEL_OFFSET_Y),
                    Align2::CENTER_CENTER,
                    option.label,
                    egui::FontId::new(10.0, egui::FontFamily::Proportional),
                    icon_and_text_color,
                );

                if response.clicked() && !selected {
                    *current = option.value;
                }

                let _ = response.on_hover_cursor(CursorIcon::PointingHand);
            }
        })
        .response
        .rect
    })
    .inner
}

fn draw_toggle_rail_tabs(
    ui: &mut Ui,
    id: Id,
    current: &mut Option<usize>,
    options: &[TabOption<'_>],
) -> Rect {
    if options.is_empty() {
        return Rect::NOTHING;
    }

    let dark_mode = ui.visuals().dark_mode;

    if let Some(selected) = current.as_mut() {
        *selected = (*selected).min(options.len().saturating_sub(1));
    }

    ui.push_id(id, |ui| {
        ui.spacing_mut().item_spacing.y = RAIL_TAB_GAP;
        ui.vertical(|ui| {
            for option in options {
                let selected = *current == Some(option.value);
                let (rect, response) = ui.allocate_exact_size(RAIL_TAB_SIZE, egui::Sense::click());
                let icon_and_text_color = if selected {
                    tokens::text_primary(dark_mode)
                } else if response.hovered() {
                    tokens::text_secondary(dark_mode)
                } else {
                    tokens::text_muted(dark_mode)
                };

                let fill = if response.is_pointer_button_down_on() {
                    tokens::button_secondary_active_bg(dark_mode)
                } else if selected {
                    tokens::row_active_bg(dark_mode)
                } else if response.hovered() {
                    tokens::button_secondary_hover_bg(dark_mode)
                } else {
                    tokens::TRANSPARENT
                };

                ui.painter().rect(
                    rect,
                    CornerRadius::same(tokens::RADIUS_SM),
                    fill,
                    Stroke::NONE,
                    StrokeKind::Outside,
                );

                if let Some(icon) = option.icon {
                    if let Some(image) = icons::image(ui.ctx(), icon, RAIL_TAB_ICON_SIZE) {
                        let icon_rect = Rect::from_center_size(
                            egui::pos2(rect.center().x, rect.center().y + RAIL_TAB_ICON_OFFSET_Y),
                            egui::vec2(RAIL_TAB_ICON_SIZE, RAIL_TAB_ICON_SIZE),
                        );
                        image.tint(icon_and_text_color).paint_at(ui, icon_rect);
                    }
                }

                ui.painter().text(
                    egui::pos2(rect.center().x, rect.center().y + RAIL_TAB_LABEL_OFFSET_Y),
                    Align2::CENTER_CENTER,
                    option.label,
                    egui::FontId::new(10.0, egui::FontFamily::Proportional),
                    icon_and_text_color,
                );

                if response.clicked() {
                    if selected {
                        *current = None;
                    } else {
                        *current = Some(option.value);
                    }
                }

                let _ = response.on_hover_cursor(CursorIcon::PointingHand);
            }
        })
        .response
        .rect
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::{
        draw_rail_tabs, draw_segmented_tabs, draw_stacked_tabs, draw_toggle_rail_tabs, TabOption,
        INLINE_TAB_FRAME_PADDING, INLINE_TAB_HEIGHT, RAIL_TAB_GAP, RAIL_TAB_SIZE, STACKED_TAB_GAP,
        STACKED_TAB_SIZE,
    };
    use egui::{CentralPanel, Context, Id, RawInput, Rect};

    #[test]
    fn renders_stacked_tabs() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;
        let options = [
            TabOption::with_icon(0, "Templates", "layout-template"),
            TabOption::with_icon(1, "Assets", "panel-top"),
        ];
        let mut current = 0;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = draw_stacked_tabs(ui, Id::new("stacked_tabs_test"), &mut current, &options);
            });
        });

        let expected_width = (STACKED_TAB_SIZE.x * options.len() as f32)
            + (STACKED_TAB_GAP * options.len().saturating_sub(1) as f32);
        assert_eq!(rect.width(), expected_width);
        assert_eq!(rect.height(), STACKED_TAB_SIZE.y);
    }

    #[test]
    fn renders_segmented_tabs() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;
        let options = [TabOption::new(0, "All mail"), TabOption::new(1, "Unread")];
        let mut current = 0;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect =
                    draw_segmented_tabs(ui, Id::new("segmented_tabs_test"), &mut current, &options);
            });
        });

        assert!(rect.width() > 0.0);
        assert_eq!(
            rect.height(),
            INLINE_TAB_HEIGHT + (INLINE_TAB_FRAME_PADDING as f32 * 2.0)
        );
    }

    #[test]
    fn renders_rail_tabs() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;
        let options = [
            TabOption::with_icon(0, "World", "globe"),
            TabOption::with_icon(1, "Assets", "folder-open"),
        ];
        let mut current = 0;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = draw_rail_tabs(ui, Id::new("rail_tabs_test"), &mut current, &options);
            });
        });

        let expected_height = (RAIL_TAB_SIZE.y * options.len() as f32)
            + (RAIL_TAB_GAP * options.len().saturating_sub(1) as f32);
        assert_eq!(rect.width(), RAIL_TAB_SIZE.x);
        assert_eq!(rect.height(), expected_height);
    }

    #[test]
    fn rail_tabs_clamp_current_to_last_visible_option() {
        let context = Context::default();
        let options = [
            TabOption::with_icon(0, "World", "globe"),
            TabOption::with_icon(1, "Assets", "folder-open"),
        ];
        let mut current = 99;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = draw_rail_tabs(ui, Id::new("rail_tabs_clamp_test"), &mut current, &options);
            });
        });

        assert_eq!(current, options.len() - 1);
    }

    #[test]
    fn renders_toggle_rail_tabs_with_no_selection() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;
        let options = [
            TabOption::with_icon(0, "World", "globe"),
            TabOption::with_icon(1, "Assets", "folder-open"),
        ];
        let mut current = None;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = draw_toggle_rail_tabs(
                    ui,
                    Id::new("toggle_rail_tabs_test"),
                    &mut current,
                    &options,
                );
            });
        });

        let expected_height = (RAIL_TAB_SIZE.y * options.len() as f32)
            + (RAIL_TAB_GAP * options.len().saturating_sub(1) as f32);
        assert_eq!(rect.width(), RAIL_TAB_SIZE.x);
        assert_eq!(rect.height(), expected_height);
        assert_eq!(current, None);
    }
}
