use super::api::ComponentUi;
use crate::layout;
use crate::ui::{icons, tokens, typography};
use egui::{Align2, CornerRadius, CursorIcon, Id, Rect, RichText, Stroke, StrokeKind, Ui};

const INLINE_TAB_GAP: f32 = 3.0;
const INLINE_TAB_HEIGHT: f32 = 24.0;
const INLINE_TAB_MIN_WIDTH: f32 = 48.0;
const INLINE_TAB_PADDING_X: f32 = 11.0;
const INLINE_TAB_GROUP_PADDING: f32 = 4.0;
const STACKED_TAB_SIZE: egui::Vec2 = egui::vec2(80.0, 68.0);
const STACKED_TAB_GAP: f32 = 8.0;
const STACKED_TAB_ICON_SIZE: f32 = 18.0;
const RAIL_TAB_SIZE: egui::Vec2 = egui::vec2(62.0, 54.0);
const RAIL_TAB_GAP: f32 = 4.0;
const RAIL_TAB_ICON_SIZE: f32 = 16.0;
const RAIL_TAB_ICON_OFFSET_Y: f32 = -9.0;
const RAIL_TAB_LABEL_OFFSET_Y: f32 = 10.0;
const BLENDER_TAB_HEIGHT: f32 = 28.0;
const BLENDER_TAB_GAP: f32 = 1.0;
const BLENDER_TAB_MIN_WIDTH: f32 = 72.0;
const BLENDER_TAB_PADDING_X: f32 = 14.0;

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
    BlenderTopbar,
}

impl ComponentUi<'_> {
    pub fn tabs(&mut self, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
        draw_tabs(self.ui_mut(), id, current, options);
    }

    pub fn tabs_variant(
        &mut self,
        id: Id,
        current: &mut usize,
        options: &[TabOption<'_>],
        variant: TabsVariant,
    ) {
        match variant {
            TabsVariant::Underline => draw_tabs(self.ui_mut(), id, current, options),
            TabsVariant::Segmented => {
                let _ = draw_segmented_tabs(self.ui_mut(), id, current, options);
            }
            TabsVariant::BlenderTopbar => {
                let _ = draw_blender_topbar_tabs(self.ui_mut(), id, current, options);
            }
        }
    }

    pub fn segmented_tabs(&mut self, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
        let _ = draw_segmented_tabs(self.ui_mut(), id, current, options);
    }

    pub fn stacked_tabs(&mut self, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
        let _ = draw_stacked_tabs(self.ui_mut(), id, current, options);
    }

    pub fn blender_topbar_tabs(&mut self, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
        let _ = draw_blender_topbar_tabs(self.ui_mut(), id, current, options);
    }

    pub fn rail_tabs(&mut self, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
        let _ = draw_rail_tabs(self.ui_mut(), id, current, options);
    }

    pub fn toggle_rail_tabs(
        &mut self,
        id: Id,
        current: &mut Option<usize>,
        options: &[TabOption<'_>],
    ) {
        let _ = draw_toggle_rail_tabs(self.ui_mut(), id, current, options);
    }
}

fn draw_tabs(ui: &mut Ui, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
    if options.is_empty() {
        return;
    }
    let runtime = crate::theme::runtime_for_ui(ui);

    ui.push_id(id, |ui| {
        let _ = layout::row().gap(6.0).show(ui, |ui| {
            for option in options {
                let selected = *current == option.value;
                let response = ui
                    .add(
                        egui::Button::new(RichText::new(option.label).color(if selected {
                            tokens::row_selected_text(runtime)
                        } else {
                            tokens::text_secondary(runtime)
                        }))
                        .fill(tokens::TRANSPARENT)
                        .stroke(Stroke::new(1.0, tokens::TRANSPARENT))
                        .corner_radius(CornerRadius::ZERO)
                        .min_size(egui::vec2(0.0, tokens::SPACING_INTERACT_HEIGHT)),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand);

                if selected {
                    let y = response.rect.bottom() - 1.0;
                    ui.painter().line_segment(
                        [
                            egui::pos2(response.rect.left() + 4.0, y),
                            egui::pos2(response.rect.right() - 4.0, y),
                        ],
                        Stroke::new(2.6, tokens::text_secondary(runtime)),
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

    let runtime = crate::theme::runtime_for_ui(ui);
    let label_font = typography::label_font();

    ui.push_id(id, |ui| {
        let tab_widths = options
            .iter()
            .map(|option| {
                let label_width = ui.fonts_mut(|fonts| {
                    fonts
                        .layout_no_wrap(
                            option.label.to_owned(),
                            label_font.clone(),
                            tokens::text_primary(runtime),
                        )
                        .size()
                        .x
                });
                (label_width + (INLINE_TAB_PADDING_X * 2.0)).max(INLINE_TAB_MIN_WIDTH)
            })
            .collect::<Vec<_>>();
        let group_size = segmented_tabs_group_size(&tab_widths);
        let (group_rect, _) = ui.allocate_exact_size(group_size, egui::Sense::hover());

        ui.painter().rect(
            group_rect,
            CornerRadius::same(tokens::radius_md(runtime)),
            inline_tabs_group_fill(runtime),
            Stroke::NONE,
            StrokeKind::Outside,
        );

        for (option, rect) in options
            .iter()
            .zip(segmented_tabs_rects(group_rect, &tab_widths))
        {
            let response = ui.interact(rect, ui.id().with(option.value), egui::Sense::click());
            let selected = *current == option.value;
            let fill = if selected {
                inline_tabs_selected_fill(runtime)
            } else if response.is_pointer_button_down_on() {
                inline_tabs_pressed_fill(runtime)
            } else if response.hovered() {
                inline_tabs_hover_fill(runtime)
            } else {
                tokens::TRANSPARENT
            };

            ui.painter().rect(
                rect,
                CornerRadius::same(tokens::radius_sm(runtime)),
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
                    inline_tabs_selected_text(runtime)
                } else {
                    tokens::text_secondary(runtime)
                },
            );

            if response.clicked() && !selected {
                *current = option.value;
            }

            let _ = response.on_hover_cursor(CursorIcon::PointingHand);
        }

        group_rect
    })
    .inner
}

fn segmented_tabs_group_size(tab_widths: &[f32]) -> egui::Vec2 {
    let tabs_width = tab_widths.iter().sum::<f32>()
        + (INLINE_TAB_GAP * tab_widths.len().saturating_sub(1) as f32);
    egui::vec2(
        tabs_width + (INLINE_TAB_GROUP_PADDING * 2.0),
        INLINE_TAB_HEIGHT + (INLINE_TAB_GROUP_PADDING * 2.0),
    )
}

fn segmented_tabs_rects(group_rect: Rect, tab_widths: &[f32]) -> Vec<Rect> {
    let mut rects = Vec::with_capacity(tab_widths.len());
    let mut left = group_rect.left() + INLINE_TAB_GROUP_PADDING;
    let top = group_rect.top() + INLINE_TAB_GROUP_PADDING;

    for width in tab_widths.iter().copied() {
        let rect = Rect::from_min_size(egui::pos2(left, top), egui::vec2(width, INLINE_TAB_HEIGHT));
        rects.push(rect);
        left = rect.right() + INLINE_TAB_GAP;
    }

    rects
}

fn inline_tabs_group_fill(runtime: crate::theme::ThemeRuntime) -> egui::Color32 {
    if runtime.mode.is_dark() {
        tokens::row_active_bg(runtime)
    } else {
        tokens::muted_surface(runtime)
    }
}

fn inline_tabs_selected_fill(runtime: crate::theme::ThemeRuntime) -> egui::Color32 {
    if runtime.mode.is_dark() {
        tokens::card_background(runtime)
    } else {
        tokens::primary_bg(runtime)
    }
}

fn inline_tabs_hover_fill(runtime: crate::theme::ThemeRuntime) -> egui::Color32 {
    tokens::row_hover_bg(runtime)
}

fn inline_tabs_pressed_fill(runtime: crate::theme::ThemeRuntime) -> egui::Color32 {
    if runtime.mode.is_dark() {
        tokens::input_background(runtime)
    } else {
        tokens::row_active_bg(runtime)
    }
}

fn inline_tabs_selected_text(runtime: crate::theme::ThemeRuntime) -> egui::Color32 {
    if runtime.mode.is_dark() {
        tokens::text_primary(runtime)
    } else {
        tokens::primary_fg(runtime)
    }
}

fn draw_stacked_tabs(ui: &mut Ui, id: Id, current: &mut usize, options: &[TabOption<'_>]) -> Rect {
    if options.is_empty() {
        return Rect::NOTHING;
    }

    let runtime = crate::theme::runtime_for_ui(ui);
    *current = (*current).min(options.len().saturating_sub(1));

    ui.push_id(id, |ui| {
        layout::row()
            .gap(STACKED_TAB_GAP)
            .show(ui, |ui| {
                for option in options {
                    let selected = *current == option.value;
                    let (rect, response) =
                        ui.allocate_exact_size(STACKED_TAB_SIZE, egui::Sense::click());

                    let fill = if response.is_pointer_button_down_on() {
                        tokens::button_secondary_active_bg(runtime)
                    } else if selected || response.hovered() {
                        tokens::button_secondary_hover_bg(runtime)
                    } else {
                        tokens::TRANSPARENT
                    };
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(tokens::radius_sm(runtime)),
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
                                    tokens::text_primary(runtime)
                                } else {
                                    tokens::text_secondary(runtime)
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
                            tokens::text_primary(runtime)
                        } else {
                            tokens::text_secondary(runtime)
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

fn draw_blender_topbar_tabs(
    ui: &mut Ui,
    id: Id,
    current: &mut usize,
    options: &[TabOption<'_>],
) -> Rect {
    if options.is_empty() {
        return Rect::NOTHING;
    }

    let runtime = crate::theme::runtime_for_ui(ui);
    let label_font = typography::label_font();

    ui.push_id(id, |ui| {
        let tab_widths = options
            .iter()
            .map(|option| {
                let label_width = ui.fonts_mut(|fonts| {
                    fonts
                        .layout_no_wrap(
                            option.label.to_owned(),
                            label_font.clone(),
                            tokens::text_primary(runtime),
                        )
                        .size()
                        .x
                });
                (label_width + (BLENDER_TAB_PADDING_X * 2.0)).max(BLENDER_TAB_MIN_WIDTH)
            })
            .collect::<Vec<_>>();

        let tabs_width = tab_widths.iter().sum::<f32>()
            + (BLENDER_TAB_GAP * tab_widths.len().saturating_sub(1) as f32);
        let group_size = egui::vec2(tabs_width, BLENDER_TAB_HEIGHT);
        let (group_rect, _) = ui.allocate_exact_size(group_size, egui::Sense::hover());
        let mut left = group_rect.left();

        for (option, width) in options.iter().zip(tab_widths.iter().copied()) {
            let rect = Rect::from_min_size(
                egui::pos2(left, group_rect.top()),
                egui::vec2(width, BLENDER_TAB_HEIGHT),
            );
            let response = ui.interact(rect, ui.id().with(option.value), egui::Sense::click());
            let selected = *current == option.value;
            let fill = if selected {
                blender_topbar_selected_fill(runtime)
            } else if response.is_pointer_button_down_on() {
                blender_topbar_pressed_fill(runtime)
            } else if response.hovered() {
                blender_topbar_hover_fill(runtime)
            } else {
                blender_topbar_idle_fill(runtime)
            };
            let corner_radius = blender_topbar_corner_radius(selected);

            ui.painter()
                .rect(rect, corner_radius, fill, Stroke::NONE, StrokeKind::Outside);
            paint_blender_topbar_border(ui, rect, runtime, corner_radius);

            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                option.label,
                label_font.clone(),
                if selected {
                    tokens::text_primary(runtime)
                } else {
                    blender_topbar_idle_text(runtime)
                },
            );

            if response.clicked() && !selected {
                *current = option.value;
            }

            let _ = response.on_hover_cursor(CursorIcon::PointingHand);
            left = rect.right() + BLENDER_TAB_GAP;
        }

        group_rect
    })
    .inner
}

fn blender_topbar_corner_radius(selected: bool) -> CornerRadius {
    let radius = if selected { 4 } else { 3 };
    CornerRadius {
        nw: radius,
        ne: radius,
        sw: 0,
        se: 0,
    }
}

fn paint_blender_topbar_border(
    ui: &Ui,
    rect: Rect,
    runtime: crate::theme::ThemeRuntime,
    corner_radius: CornerRadius,
) {
    ui.painter().rect_stroke(
        rect,
        corner_radius,
        Stroke::new(1.0, blender_topbar_border(runtime)),
        StrokeKind::Inside,
    );
}

fn blender_topbar_idle_fill(runtime: crate::theme::ThemeRuntime) -> egui::Color32 {
    if runtime.mode.is_dark() {
        tokens::muted_surface(runtime).linear_multiply(0.7)
    } else {
        tokens::muted_surface(runtime)
    }
}

fn blender_topbar_hover_fill(runtime: crate::theme::ThemeRuntime) -> egui::Color32 {
    if runtime.mode.is_dark() {
        tokens::button_secondary_hover_bg(runtime).linear_multiply(0.92)
    } else {
        tokens::button_secondary_hover_bg(runtime)
    }
}

fn blender_topbar_pressed_fill(runtime: crate::theme::ThemeRuntime) -> egui::Color32 {
    if runtime.mode.is_dark() {
        tokens::button_secondary_active_bg(runtime).linear_multiply(0.9)
    } else {
        tokens::button_secondary_active_bg(runtime)
    }
}

fn blender_topbar_selected_fill(runtime: crate::theme::ThemeRuntime) -> egui::Color32 {
    if runtime.mode.is_dark() {
        tokens::card_background(runtime).linear_multiply(1.06)
    } else {
        tokens::card_background(runtime)
    }
}

fn blender_topbar_border(runtime: crate::theme::ThemeRuntime) -> egui::Color32 {
    if runtime.mode.is_dark() {
        tokens::separator(runtime).linear_multiply(0.78)
    } else {
        tokens::separator(runtime)
    }
}

fn blender_topbar_idle_text(runtime: crate::theme::ThemeRuntime) -> egui::Color32 {
    if runtime.mode.is_dark() {
        tokens::text_secondary(runtime).linear_multiply(0.95)
    } else {
        tokens::text_secondary(runtime)
    }
}

fn draw_rail_tabs(ui: &mut Ui, id: Id, current: &mut usize, options: &[TabOption<'_>]) -> Rect {
    if options.is_empty() {
        return Rect::NOTHING;
    }

    let runtime = crate::theme::runtime_for_ui(ui);
    *current = (*current).min(options.len().saturating_sub(1));

    ui.push_id(id, |ui| {
        layout::column()
            .gap(RAIL_TAB_GAP)
            .show(ui, |ui| {
                for option in options {
                    let selected = *current == option.value;
                    let (rect, response) =
                        ui.allocate_exact_size(RAIL_TAB_SIZE, egui::Sense::click());
                    let icon_and_text_color = if selected {
                        tokens::text_primary(runtime)
                    } else if response.hovered() {
                        tokens::text_secondary(runtime)
                    } else {
                        tokens::text_muted(runtime)
                    };

                    let fill = if response.is_pointer_button_down_on() {
                        tokens::button_secondary_active_bg(runtime)
                    } else if selected {
                        tokens::row_active_bg(runtime)
                    } else if response.hovered() {
                        tokens::button_secondary_hover_bg(runtime)
                    } else {
                        tokens::TRANSPARENT
                    };

                    ui.painter().rect(
                        rect,
                        CornerRadius::same(tokens::radius_sm(runtime)),
                        fill,
                        Stroke::NONE,
                        StrokeKind::Outside,
                    );

                    if let Some(icon) = option.icon {
                        if let Some(image) = icons::image(ui.ctx(), icon, RAIL_TAB_ICON_SIZE) {
                            let icon_rect = Rect::from_center_size(
                                egui::pos2(
                                    rect.center().x,
                                    rect.center().y + RAIL_TAB_ICON_OFFSET_Y,
                                ),
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

    let runtime = crate::theme::runtime_for_ui(ui);

    if let Some(selected) = current.as_mut() {
        *selected = (*selected).min(options.len().saturating_sub(1));
    }

    ui.push_id(id, |ui| {
        layout::column()
            .gap(RAIL_TAB_GAP)
            .show(ui, |ui| {
                for option in options {
                    let selected = *current == Some(option.value);
                    let (rect, response) =
                        ui.allocate_exact_size(RAIL_TAB_SIZE, egui::Sense::click());
                    let icon_and_text_color = if selected {
                        tokens::text_primary(runtime)
                    } else if response.hovered() {
                        tokens::text_secondary(runtime)
                    } else {
                        tokens::text_muted(runtime)
                    };

                    let fill = if response.is_pointer_button_down_on() {
                        tokens::button_secondary_active_bg(runtime)
                    } else if selected {
                        tokens::row_active_bg(runtime)
                    } else if response.hovered() {
                        tokens::button_secondary_hover_bg(runtime)
                    } else {
                        tokens::TRANSPARENT
                    };

                    ui.painter().rect(
                        rect,
                        CornerRadius::same(tokens::radius_sm(runtime)),
                        fill,
                        Stroke::NONE,
                        StrokeKind::Outside,
                    );

                    if let Some(icon) = option.icon {
                        if let Some(image) = icons::image(ui.ctx(), icon, RAIL_TAB_ICON_SIZE) {
                            let icon_rect = Rect::from_center_size(
                                egui::pos2(
                                    rect.center().x,
                                    rect.center().y + RAIL_TAB_ICON_OFFSET_Y,
                                ),
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
        blender_topbar_corner_radius, draw_blender_topbar_tabs, draw_rail_tabs,
        draw_segmented_tabs, draw_stacked_tabs, draw_toggle_rail_tabs, segmented_tabs_group_size,
        segmented_tabs_rects, TabOption, BLENDER_TAB_GAP, BLENDER_TAB_HEIGHT,
        INLINE_TAB_GROUP_PADDING, INLINE_TAB_HEIGHT, RAIL_TAB_GAP, RAIL_TAB_SIZE, STACKED_TAB_GAP,
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
            INLINE_TAB_HEIGHT + (INLINE_TAB_GROUP_PADDING * 2.0)
        );
    }

    #[test]
    fn segmented_tabs_group_padding_is_uniform() {
        let tab_widths = [58.0, 64.0, 72.0];
        let group_rect = Rect::from_min_size(
            egui::pos2(12.0, 24.0),
            segmented_tabs_group_size(&tab_widths),
        );
        let tab_rects = segmented_tabs_rects(group_rect, &tab_widths);
        let first = tab_rects
            .first()
            .copied()
            .expect("missing first segmented tab");
        let last = tab_rects
            .last()
            .copied()
            .expect("missing last segmented tab");

        assert_eq!(first.left() - group_rect.left(), INLINE_TAB_GROUP_PADDING);
        assert_eq!(group_rect.right() - last.right(), INLINE_TAB_GROUP_PADDING);
        assert_eq!(first.top() - group_rect.top(), INLINE_TAB_GROUP_PADDING);
        assert_eq!(
            group_rect.bottom() - first.bottom(),
            INLINE_TAB_GROUP_PADDING
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
    fn renders_blender_topbar_tabs() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;
        let options = [
            TabOption::new(0, "Layout"),
            TabOption::new(1, "Modeling"),
            TabOption::new(2, "Sculpting"),
        ];
        let mut current = 1;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = draw_blender_topbar_tabs(
                    ui,
                    Id::new("blender_topbar_tabs_test"),
                    &mut current,
                    &options,
                );
            });
        });

        assert!(rect.width() > 0.0);
        assert_eq!(rect.height(), BLENDER_TAB_HEIGHT);
        assert!(rect.width() >= (BLENDER_TAB_GAP * options.len().saturating_sub(1) as f32));
    }

    #[test]
    fn blender_topbar_tabs_only_round_top_corners() {
        let selected = blender_topbar_corner_radius(true);
        let idle = blender_topbar_corner_radius(false);

        assert_eq!(selected.sw, 0);
        assert_eq!(selected.se, 0);
        assert_eq!(idle.sw, 0);
        assert_eq!(idle.se, 0);
        assert!(selected.nw > idle.nw);
        assert!(selected.ne > idle.ne);
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
