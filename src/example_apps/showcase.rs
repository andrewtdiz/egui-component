use crate::internal_taffy::{
    taffy,
    taffy::prelude::{auto, fr, length, percent},
    tid, tui, TuiBuilderLogic,
};
use crate::primitives::{draw_swatch, surface_frame, ScrollAreaExt, SurfaceFrame, Swatch};
use crate::runtime_components::*;
use crate::theme::{self, BaseColor, ColorRole, RadiusRole, ThemeMode, ThemeSpec};
use crate::ui::tokens;
use crate::{component_definitions, ComponentDefinition, ComponentGroup, ComponentKind};
use egui::{
    vec2, Align2, CentralPanel, Color32, CornerRadius, CursorIcon, Id, InnerResponse, Layout, Rect,
    Response, ScrollArea, Sense, SidePanel, Stroke, TopBottomPanel, Ui, UiBuilder,
};

pub(crate) fn show_row<R>(
    ui: &mut Ui,
    gap: f32,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = gap.max(0.0);
        ui.horizontal(add)
    })
    .inner
}

pub(crate) fn show_column<R>(
    ui: &mut Ui,
    gap: f32,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = gap.max(0.0);
        ui.vertical(add)
    })
    .inner
}

pub(crate) fn show_inset<R>(
    ui: &mut Ui,
    padding_x: i8,
    padding_y: i8,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(padding_x.max(0), padding_y.max(0)))
        .show(ui, add)
}

pub(crate) fn showcase_card<R>(
    ui: &mut Ui,
    fill: Option<Color32>,
    stroke: Option<Stroke>,
    corner_radius: Option<u8>,
    padding_x: i8,
    padding_y: i8,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let runtime = theme::runtime_for_ui(ui);
    surface_frame(
        ui,
        SurfaceFrame::new(
            fill.unwrap_or(tokens::muted_surface(runtime)),
            stroke.unwrap_or(Stroke::new(1.0, tokens::separator(runtime))),
        )
        .corner_radius(corner_radius.unwrap_or(tokens::radius_lg(runtime)))
        .padding(padding_x, padding_y),
        add,
    )
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ShowcaseTabOption<'a> {
    pub value: usize,
    pub label: &'a str,
    pub icon: Option<&'a str>,
    pub icon_only: bool,
}

impl<'a> ShowcaseTabOption<'a> {
    pub const fn new(value: usize, label: &'a str) -> Self {
        Self {
            value,
            label,
            icon: None,
            icon_only: false,
        }
    }

    pub const fn with_icon(value: usize, label: &'a str, icon: &'a str) -> Self {
        Self {
            value,
            label,
            icon: Some(icon),
            icon_only: false,
        }
    }

    pub const fn icon_only(value: usize, label: &'a str, icon: &'a str) -> Self {
        Self {
            value,
            label,
            icon: Some(icon),
            icon_only: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ShowcaseTabsStyle {
    Underline,
    Segmented,
    Stacked,
    Rail,
    BlenderTopbar,
}

pub(crate) fn showcase_tabs_variant(
    ui: &mut Ui,
    id: Id,
    current: &mut usize,
    options: &[ShowcaseTabOption<'_>],
    style: ShowcaseTabsStyle,
) {
    if options.is_empty() {
        return;
    }

    ui.push_id(id, |ui| {
        let vertical = matches!(style, ShowcaseTabsStyle::Stacked | ShowcaseTabsStyle::Rail);
        let gap = match style {
            ShowcaseTabsStyle::BlenderTopbar => 1.0,
            ShowcaseTabsStyle::Stacked => 8.0,
            ShowcaseTabsStyle::Rail => 6.0,
            ShowcaseTabsStyle::Segmented => 4.0,
            ShowcaseTabsStyle::Underline => 6.0,
        };
        let draw = |ui: &mut Ui| {
            let mut components = ui.components();
            for option in options {
                let selected = *current == option.value;
                let mut button = if option.icon_only {
                    Button::icon_only(option.icon.unwrap_or("circle"))
                } else {
                    Button::new(option.label)
                };
                if let Some(icon) = option.icon.filter(|_| !option.icon_only) {
                    button = button.leading_icon(icon);
                }
                button = button
                    .variant(match style {
                        ShowcaseTabsStyle::Underline => {
                            if selected {
                                ButtonVariant::Link
                            } else {
                                ButtonVariant::Ghost
                            }
                        }
                        _ => {
                            if selected {
                                ButtonVariant::Secondary
                            } else {
                                ButtonVariant::Ghost
                            }
                        }
                    })
                    .selected(selected);
                if vertical {
                    button = button.min_size(egui::vec2(96.0, 40.0));
                }
                if components.button(button).clicked() {
                    *current = option.value;
                }
            }
        };
        if vertical {
            let _ = show_column(ui, gap, draw);
        } else {
            let _ = show_row(ui, gap, draw);
        }
    });
}

pub(crate) fn showcase_swatch(ui: &mut Ui, swatch: Swatch) -> Response {
    draw_swatch(ui, swatch)
}

pub(crate) fn showcase_menu_entries(
    ui: &mut Ui,
    entries: &[DropdownMenuEntry<'_>],
    action: &mut Option<usize>,
) {
    for entry in entries {
        match entry {
            DropdownMenuEntry::Action(item) => {
                let mut label = item.label.to_owned();
                if let Some(shortcut) = item.shortcut {
                    label.push_str("    ");
                    label.push_str(shortcut);
                }
                let response = ui.add_enabled(item.enabled, egui::Button::new(label));
                if response.clicked() {
                    *action = Some(item.id);
                    ui.close();
                }
            }
            DropdownMenuEntry::Separator => {
                ui.separator();
            }
            DropdownMenuEntry::Submenu(submenu) => {
                ui.menu_button(submenu.label, |ui| {
                    showcase_menu_entries(ui, submenu.entries, action)
                });
            }
        }
    }
}

pub(crate) fn show_width<R>(
    ui: &mut Ui,
    width: f32,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let scoped = ui.scope(|ui| {
        ui.set_width(width.max(0.0));
        let inner = add(ui);
        (inner, ui.min_rect())
    });
    let (inner, rect) = scoped.inner;
    let mut response = scoped.response;
    response.rect = rect;
    response.interact_rect = rect;
    InnerResponse { inner, response }
}

fn show_leading_trailing(
    ui: &mut Ui,
    gap: f32,
    min_height: f32,
    align: egui::Align,
    leading: impl FnOnce(&mut Ui),
    trailing: impl FnOnce(&mut Ui),
) -> Response {
    let row_height = min_height
        .max(ui.text_style_height(&egui::TextStyle::Body))
        .max(1.0);
    let scoped = ui.scope(|ui| {
        tui(ui, ui.auto_id_with("showcase_leading_trailing"))
            .reserve_available_width()
            .style(taffy::Style {
                flex_direction: taffy::FlexDirection::Row,
                align_items: Some(match align {
                    egui::Align::Min => taffy::AlignItems::Start,
                    egui::Align::Center => taffy::AlignItems::Center,
                    egui::Align::Max => taffy::AlignItems::End,
                }),
                gap: length(gap.max(0.0)),
                size: taffy::Size {
                    width: percent(1.0),
                    height: auto(),
                },
                min_size: taffy::Size {
                    width: auto(),
                    height: length(row_height),
                },
                ..Default::default()
            })
            .show(|tui| {
                tui.id("leading")
                    .style(taffy::Style {
                        flex_grow: 1.0,
                        flex_basis: length(0.0),
                        size: taffy::Size {
                            width: percent(1.0),
                            height: auto(),
                        },
                        min_size: taffy::Size {
                            width: length(0.0),
                            height: length(row_height),
                        },
                        ..Default::default()
                    })
                    .egui_layout(
                        Layout::left_to_right(align)
                            .with_main_align(egui::Align::Min)
                            .with_cross_align(align),
                    )
                    .ui(leading);

                tui.id("trailing")
                    .style(taffy::Style {
                        min_size: taffy::Size {
                            width: auto(),
                            height: length(row_height),
                        },
                        ..Default::default()
                    })
                    .egui_layout(
                        Layout::right_to_left(align)
                            .with_main_align(egui::Align::Min)
                            .with_cross_align(align),
                    )
                    .ui(trailing);
            });

        ui.min_rect()
    });

    let mut response = scoped.response;
    response.rect = scoped.inner;
    response.interact_rect = scoped.inner;
    response
}

fn show_tile_grid(
    ui: &mut Ui,
    columns: Option<usize>,
    min_tile_width: f32,
    gap: f32,
    tile_height: Option<f32>,
    aspect_ratio: Option<f32>,
    item_count: usize,
    mut add: impl FnMut(&mut Ui, usize, Rect),
) -> Response {
    if item_count == 0 {
        return ui.allocate_exact_size(egui::Vec2::ZERO, Sense::hover()).1;
    }

    let available_width = ui.available_width().max(0.0);
    let resolved_columns = columns.unwrap_or_else(|| {
        let slot_width = (min_tile_width.max(1.0) + gap.max(0.0)).max(1.0);
        ((available_width + gap.max(0.0)) / slot_width)
            .floor()
            .max(1.0) as usize
    });

    let scoped = ui.scope(|ui| {
        let total_gap = gap.max(0.0) * resolved_columns.saturating_sub(1) as f32;
        let tile_width = ((ui.available_width().max(0.0) - total_gap) / resolved_columns as f32)
            .floor()
            .max(0.0);
        let resolved_tile_height = tile_height.unwrap_or_else(|| {
            aspect_ratio
                .map(|ratio| (tile_width / ratio.max(f32::EPSILON)).max(0.0))
                .unwrap_or(tile_width)
        });

        tui(ui, ui.auto_id_with("showcase_tile_grid"))
            .reserve_available_width()
            .style(taffy::Style {
                display: taffy::Display::Grid,
                grid_template_columns: vec![fr(1.0); resolved_columns],
                gap: taffy::Size {
                    width: length(gap.max(0.0)),
                    height: length(gap.max(0.0)),
                },
                size: taffy::Size {
                    width: percent(1.0),
                    height: auto(),
                },
                align_items: Some(taffy::AlignItems::Stretch),
                justify_items: Some(taffy::AlignItems::Stretch),
                ..Default::default()
            })
            .show(|tui| {
                for index in 0..item_count {
                    tui.id(tid(index))
                        .style(taffy::Style {
                            size: taffy::Size {
                                width: auto(),
                                height: length(resolved_tile_height),
                            },
                            ..Default::default()
                        })
                        .ui(|ui| add(ui, index, ui.max_rect()));
                }
            });

        ui.min_rect()
    });

    let mut response = scoped.response;
    response.rect = scoped.inner;
    response.interact_rect = scoped.inner;
    response
}

include!("showcase/data.rs");
include!("showcase/chrome.rs");
include!("showcase/previews.rs");
include!("showcase/helpers.rs");
include!("showcase/brand_kit_layout.rs");
include!("showcase/metadata.rs");

#[cfg(test)]
mod tests;
