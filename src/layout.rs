pub use crate::internal_taffy::{
    bg, taffy, tid, tui, virtual_tui, widgets, AsTuiBuilder, TaffyContainerUi, Tui, TuiBuilder,
    TuiBuilderLogic, TuiBuilderParamsAccess, TuiContainerResponse, TuiId, TuiInnerResponse,
    TuiWidget,
};

use crate::ui::tokens;
use egui::{InnerResponse, Layout as EguiLayout, Response, Sense, Ui};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
}

impl Justify {
    fn to_egui(self) -> egui::Align {
        match self {
            Self::Start => egui::Align::Min,
            Self::Center => egui::Align::Center,
            Self::End => egui::Align::Max,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

impl Align {
    fn to_egui(self) -> egui::Align {
        match self {
            Self::Start => egui::Align::Min,
            Self::Center => egui::Align::Center,
            Self::End => egui::Align::Max,
        }
    }

    fn to_taffy(self) -> taffy::AlignItems {
        match self {
            Self::Start => taffy::AlignItems::Start,
            Self::Center => taffy::AlignItems::Center,
            Self::End => taffy::AlignItems::End,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Axis {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy)]
pub struct Flow {
    axis: Axis,
    gap: f32,
    justify: Justify,
    align: Align,
}

impl Flow {
    fn new(axis: Axis) -> Self {
        Self {
            axis,
            gap: tokens::LAYOUT_GAP_SM,
            justify: Justify::Start,
            align: match axis {
                Axis::Row => Align::Center,
                Axis::Column => Align::Start,
            },
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        match (self.axis, self.justify, self.align) {
            (Axis::Row, Justify::Start, Align::Center) => ui.scope(|ui| {
                ui.spacing_mut().item_spacing.x = self.gap;
                ui.horizontal(add).inner
            }),
            (Axis::Row, Justify::Start, Align::Start) => ui.scope(|ui| {
                ui.spacing_mut().item_spacing.x = self.gap;
                ui.horizontal_top(add).inner
            }),
            (Axis::Column, Justify::Start, Align::Start) => ui.scope(|ui| {
                ui.spacing_mut().item_spacing.y = self.gap;
                ui.vertical(add).inner
            }),
            (Axis::Column, Justify::Start, Align::Center) => ui.scope(|ui| {
                ui.spacing_mut().item_spacing.y = self.gap;
                ui.vertical_centered(add).inner
            }),
            _ => {
                let layout = match self.axis {
                    Axis::Row => EguiLayout::left_to_right(self.align.to_egui())
                        .with_main_align(self.justify.to_egui())
                        .with_cross_align(self.align.to_egui()),
                    Axis::Column => EguiLayout::top_down(self.align.to_egui())
                        .with_main_align(self.justify.to_egui())
                        .with_cross_align(self.align.to_egui()),
                };
                let scoped = ui.scope(|ui| {
                    match self.axis {
                        Axis::Row => ui.spacing_mut().item_spacing.x = self.gap,
                        Axis::Column => ui.spacing_mut().item_spacing.y = self.gap,
                    }
                    let inner = ui.with_layout(layout, add).inner;
                    (inner, ui.min_rect())
                });
                let (inner, rect) = scoped.inner;
                let mut response = scoped.response;
                response.rect = rect;
                response.interact_rect = rect;
                InnerResponse { inner, response }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SizedBox {
    width: Option<f32>,
    height: Option<f32>,
}

impl SizedBox {
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.0));
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height.max(0.0));
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        match (self.width, self.height) {
            (Some(width), Some(height)) => {
                let origin = ui.next_widget_position();
                let response =
                    ui.allocate_ui_with_layout(egui::vec2(width, height), ui.layout().clone(), add);
                let rect = egui::Rect::from_min_size(origin, egui::vec2(width, height));
                let mut sized_response = response.response;
                sized_response.rect = rect;
                sized_response.interact_rect = rect;
                InnerResponse {
                    inner: response.inner,
                    response: sized_response,
                }
            }
            (Some(width), None) => {
                let scoped = ui.scope(|ui| {
                    ui.set_min_width(width);
                    ui.set_max_width(width);
                    let inner = add(ui);
                    (inner, ui.min_rect())
                });
                let (inner, rect) = scoped.inner;
                let mut response = scoped.response;
                response.rect = rect;
                response.interact_rect = rect;
                InnerResponse { inner, response }
            }
            (None, Some(height)) => {
                let scoped = ui.scope(|ui| {
                    ui.set_min_height(height);
                    ui.set_max_height(height);
                    let inner = add(ui);
                    (inner, ui.min_rect())
                });
                let (inner, rect) = scoped.inner;
                let mut response = scoped.response;
                response.rect = rect;
                response.interact_rect = rect;
                InnerResponse { inner, response }
            }
            (None, None) => ui.scope(add),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LeadingTrailing {
    gap: f32,
    min_height: f32,
    align: Align,
}

impl Default for LeadingTrailing {
    fn default() -> Self {
        Self {
            gap: tokens::LAYOUT_GAP_SM,
            min_height: 0.0,
            align: Align::Center,
        }
    }
}

impl LeadingTrailing {
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    pub fn min_height(mut self, min_height: f32) -> Self {
        self.min_height = min_height.max(0.0);
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
        leading: impl FnOnce(&mut Ui),
        trailing: impl FnOnce(&mut Ui),
    ) -> Response {
        use taffy::prelude::{auto, length, percent};

        let row_height = self
            .min_height
            .max(ui.text_style_height(&egui::TextStyle::Body))
            .max(1.0);
        let scoped = ui.scope(|ui| {
            tui(ui, ui.auto_id_with("layout_leading_trailing"))
                .reserve_available_width()
                .style(taffy::Style {
                    flex_direction: taffy::FlexDirection::Row,
                    align_items: Some(self.align.to_taffy()),
                    gap: length(self.gap),
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
                            EguiLayout::left_to_right(self.align.to_egui())
                                .with_main_align(egui::Align::Min)
                                .with_cross_align(self.align.to_egui()),
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
                            EguiLayout::right_to_left(self.align.to_egui())
                                .with_main_align(egui::Align::Min)
                                .with_cross_align(self.align.to_egui()),
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
}

pub fn row() -> Flow {
    Flow::new(Axis::Row)
}

pub fn column() -> Flow {
    Flow::new(Axis::Column)
}

pub fn sized_box() -> SizedBox {
    SizedBox::default()
}

pub fn leading_trailing() -> LeadingTrailing {
    LeadingTrailing::default()
}

#[cfg(test)]
mod tests {
    use crate::layout::{taffy, tid, tui, TuiBuilderLogic};
    use egui::{CentralPanel, Context, RawInput, Rect, Sense};

    use super::{column, leading_trailing, row, sized_box, Align};

    #[test]
    fn vendored_taffy_primitives_are_available_from_layout_module() {
        let context = Context::default();
        let mut child_rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = ui.scope(|ui| {
                    ui.set_width(160.0);
                    tui(ui, egui::Id::new("layout_public_taffy_smoke"))
                        .reserve_available_width()
                        .style(taffy::Style {
                            size: taffy::Size {
                                width: taffy::prelude::percent(1.0),
                                height: taffy::prelude::auto(),
                            },
                            ..Default::default()
                        })
                        .show(|tui| {
                            tui.id(tid("child"))
                                .style(taffy::Style {
                                    size: taffy::Size {
                                        width: taffy::prelude::length(48.0),
                                        height: taffy::prelude::length(20.0),
                                    },
                                    ..Default::default()
                                })
                                .ui(|ui| {
                                    child_rect = ui.max_rect();
                                });
                        });
                });
            });
        });

        assert_eq!(child_rect.width(), 48.0);
        assert_eq!(child_rect.height(), 20.0);
    }

    #[test]
    fn row_gap_positions_children_explicitly() {
        let context = Context::default();
        let mut first = Rect::NOTHING;
        let mut second = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = row().gap(12.0).show(ui, |ui| {
                    first = ui
                        .allocate_exact_size(egui::vec2(20.0, 10.0), Sense::hover())
                        .0;
                    second = ui
                        .allocate_exact_size(egui::vec2(20.0, 10.0), Sense::hover())
                        .0;
                });
            });
        });

        assert_eq!(second.left() - first.right(), 12.0);
    }

    #[test]
    fn column_gap_positions_children_explicitly() {
        let context = Context::default();
        let mut first = Rect::NOTHING;
        let mut second = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = column().gap(7.0).show(ui, |ui| {
                    first = ui
                        .allocate_exact_size(egui::vec2(10.0, 12.0), Sense::hover())
                        .0;
                    second = ui
                        .allocate_exact_size(egui::vec2(10.0, 12.0), Sense::hover())
                        .0;
                });
            });
        });

        assert_eq!(second.top() - first.bottom(), 7.0);
    }

    #[test]
    fn sized_box_sets_explicit_width() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = sized_box().width(84.0).show(ui, |ui| {
                    rect = ui.max_rect();
                });
            });
        });

        assert_eq!(rect.width(), 84.0);
    }

    #[test]
    fn leading_trailing_reserves_full_width() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                ui.set_width(240.0);
                rect = leading_trailing()
                    .gap(8.0)
                    .min_height(28.0)
                    .align(Align::Center)
                    .show(
                        ui,
                        |ui| {
                            ui.label("Left");
                        },
                        |ui| {
                            ui.label("Right");
                        },
                    )
                    .rect;
            });
        });

        assert!(rect.width() >= 240.0);
    }
}
