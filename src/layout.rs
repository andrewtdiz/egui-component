use crate::ui::tokens;
use egui::{
    Context, FontId, Id, InnerResponse, Layout as EguiLayout, Rect, Response, Sense, Stroke,
    StrokeKind, Ui,
};

const DEBUG_OVERLAY_ID: &str = "egui_component::layout_debug_overlay";
const DEBUG_LABEL_FONT_SIZE: f32 = 9.0;

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
            gap: tokens::LAYOUT_GAP_MD,
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
        let response = match (self.axis, self.justify, self.align) {
            (Axis::Row, Justify::Start, Align::Start) => {
                let scoped = ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.x = self.gap;
                    ui.horizontal_top(add)
                });
                scoped.inner
            }
            (Axis::Row, Justify::Start, Align::Center) => {
                let scoped = ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.x = self.gap;
                    ui.horizontal(add)
                });
                scoped.inner
            }
            (Axis::Column, Justify::Start, Align::Start) => {
                let scoped = ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.y = self.gap;
                    ui.vertical(add)
                });
                scoped.inner
            }
            (Axis::Column, Justify::Start, Align::Center) => {
                let scoped = ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.y = self.gap;
                    ui.vertical_centered(add)
                });
                scoped.inner
            }
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
        };

        paint_flow_overlay(ui, response.response.rect, self.axis, self.gap);
        response
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Inset {
    padding_x: i8,
    padding_y: i8,
}

impl Default for Inset {
    fn default() -> Self {
        Self {
            padding_x: tokens::LAYOUT_INSET_X,
            padding_y: tokens::LAYOUT_INSET_Y,
        }
    }
}

impl Inset {
    pub fn padding(mut self, x: i8, y: i8) -> Self {
        self.padding_x = x.max(0);
        self.padding_y = y.max(0);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let response = egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(self.padding_x, self.padding_y))
            .show(ui, add);

        let outer = response.response.rect;
        let inner = outer.shrink2(egui::vec2(
            f32::from(self.padding_x),
            f32::from(self.padding_y),
        ));
        paint_inset_overlay(ui, outer, inner, self.padding_x, self.padding_y);
        response
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AlignBox {
    justify: Justify,
    align: Align,
}

impl Default for AlignBox {
    fn default() -> Self {
        Self {
            justify: Justify::Start,
            align: Align::Start,
        }
    }
}

impl AlignBox {
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let layout = ui
            .layout()
            .clone()
            .with_main_align(self.justify.to_egui())
            .with_cross_align(self.align.to_egui());
        let scoped = ui.scope(|ui| {
            let inner = ui.with_layout(layout, add).inner;
            (inner, ui.min_rect())
        });
        let (inner, rect) = scoped.inner;
        let mut response = scoped.response;
        response.rect = rect;
        response.interact_rect = rect;
        paint_box_overlay(ui, rect, "align");
        InnerResponse { inner, response }
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
        let response = match (self.width, self.height) {
            (Some(width), Some(height)) => {
                let origin = ui.next_widget_position();
                let response =
                    ui.allocate_ui_with_layout(egui::vec2(width, height), ui.layout().clone(), add);
                let mut sized_response = response.response;
                let rect = Rect::from_min_size(origin, egui::vec2(width, height));
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
        };

        paint_box_overlay(ui, response.response.rect, "sized-box");
        response
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Spacer {
    width: f32,
    height: f32,
    flex: bool,
}

impl Default for Spacer {
    fn default() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            flex: true,
        }
    }
}

impl Spacer {
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(0.0);
        self.flex = false;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(0.0);
        self.flex = false;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width.max(0.0);
        self.height = height.max(0.0);
        self.flex = false;
        self
    }

    pub fn flex(mut self) -> Self {
        self.flex = true;
        self
    }

    /// Fill the remaining space on the current axis.
    ///
    /// This is a terminal fill primitive, not a flexbox spacer. When used in a row,
    /// it consumes the current `available_width()`. Do not place trailing widgets after
    /// it unless you intentionally want them pushed out of the remaining space.
    pub fn show(self, ui: &mut Ui) -> Response {
        let size = if self.flex {
            match ui.layout().main_dir() {
                egui::Direction::LeftToRight | egui::Direction::RightToLeft => {
                    egui::vec2(ui.available_width().max(0.0), self.height)
                }
                egui::Direction::TopDown | egui::Direction::BottomUp => {
                    egui::vec2(self.width, ui.available_height().max(0.0))
                }
            }
        } else {
            egui::vec2(self.width, self.height)
        };

        ui.allocate_exact_size(size, Sense::hover()).1
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
            gap: tokens::LAYOUT_GAP_MD,
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
        let available_width = ui.available_width().max(0.0);
        let row_height = self
            .min_height
            .max(ui.text_style_height(&egui::TextStyle::Body))
            .max(1.0);
        let (row_rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, row_height), Sense::hover());

        let mut trailing_ui = ui.new_child(
            egui::UiBuilder::new().max_rect(row_rect).layout(
                EguiLayout::right_to_left(self.align.to_egui())
                    .with_main_align(egui::Align::Min)
                    .with_cross_align(self.align.to_egui()),
            ),
        );
        trailing(&mut trailing_ui);
        let trailing_rect = trailing_ui.min_rect();

        let leading_rect = Rect::from_min_max(
            row_rect.min,
            egui::pos2(
                (trailing_rect.left() - self.gap).max(row_rect.left()),
                row_rect.bottom(),
            ),
        );
        let mut leading_ui = ui.new_child(
            egui::UiBuilder::new().max_rect(leading_rect).layout(
                EguiLayout::left_to_right(self.align.to_egui())
                    .with_main_align(egui::Align::Min)
                    .with_cross_align(self.align.to_egui()),
            ),
        );
        leading(&mut leading_ui);

        paint_box_overlay(ui, row_rect, "leading-trailing");
        response
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TileGrid {
    min_tile_width: f32,
    columns: Option<usize>,
    column_gap: f32,
    row_gap: f32,
    tile_height: Option<f32>,
    aspect_ratio: Option<f32>,
}

impl Default for TileGrid {
    fn default() -> Self {
        Self {
            min_tile_width: 120.0,
            columns: None,
            column_gap: tokens::LAYOUT_GAP_MD,
            row_gap: tokens::LAYOUT_GAP_MD,
            tile_height: None,
            aspect_ratio: None,
        }
    }
}

impl TileGrid {
    pub fn min_tile_width(mut self, width: f32) -> Self {
        self.min_tile_width = width.max(1.0);
        self
    }

    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = Some(columns.max(1));
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        let gap = gap.max(0.0);
        self.column_gap = gap;
        self.row_gap = gap;
        self
    }

    pub fn column_gap(mut self, gap: f32) -> Self {
        self.column_gap = gap.max(0.0);
        self
    }

    pub fn row_gap(mut self, gap: f32) -> Self {
        self.row_gap = gap.max(0.0);
        self
    }

    pub fn tile_height(mut self, height: f32) -> Self {
        self.tile_height = Some(height.max(0.0));
        self
    }

    pub fn aspect_ratio(mut self, aspect_ratio: f32) -> Self {
        self.aspect_ratio = Some(aspect_ratio.max(f32::EPSILON));
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
        item_count: usize,
        mut add: impl FnMut(&mut Ui, usize, Rect),
    ) -> Response {
        let available_width = ui.available_width().max(0.0);
        let columns = self.resolve_columns(available_width);
        let tile_width = self.resolve_tile_width(available_width, columns);
        let tile_height = self.resolve_tile_height(tile_width);
        let tile_size = egui::vec2(tile_width, tile_height);

        let scoped = ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

            if item_count == 0 {
                return Rect::from_min_size(ui.next_widget_position(), egui::Vec2::ZERO);
            }

            for row_start in (0..item_count).step_by(columns) {
                let row_end = (row_start + columns).min(item_count);
                let _ = row()
                    .gap(self.column_gap)
                    .align(Align::Start)
                    .show(ui, |ui| {
                        for index in row_start..row_end {
                            let (rect, _) = ui.allocate_exact_size(tile_size, Sense::hover());
                            add(ui, index, rect);
                        }
                    });

                if row_end < item_count {
                    ui.add_space(self.row_gap);
                }
            }

            ui.min_rect()
        });

        let mut response = scoped.response;
        response.rect = scoped.inner;
        response.interact_rect = scoped.inner;
        paint_tile_grid_overlay(ui, response.rect, columns, self.column_gap, self.row_gap);
        response
    }

    fn resolve_columns(self, available_width: f32) -> usize {
        if let Some(columns) = self.columns {
            return columns.max(1);
        }

        let slot_width = (self.min_tile_width + self.column_gap).max(1.0);
        ((available_width + self.column_gap) / slot_width)
            .floor()
            .max(1.0) as usize
    }

    fn resolve_tile_width(self, available_width: f32, columns: usize) -> f32 {
        let total_gap = self.column_gap * columns.saturating_sub(1) as f32;
        ((available_width - total_gap) / columns as f32)
            .floor()
            .max(0.0)
    }

    fn resolve_tile_height(self, tile_width: f32) -> f32 {
        if let Some(height) = self.tile_height {
            height
        } else if let Some(aspect_ratio) = self.aspect_ratio {
            (tile_width / aspect_ratio).max(0.0)
        } else {
            tile_width
        }
    }
}

pub fn row() -> Flow {
    Flow::new(Axis::Row)
}

pub fn column() -> Flow {
    Flow::new(Axis::Column)
}

pub fn inset() -> Inset {
    Inset::default()
}

pub fn align() -> AlignBox {
    AlignBox::default()
}

pub fn sized_box() -> SizedBox {
    SizedBox::default()
}

pub fn spacer() -> Spacer {
    Spacer::default()
}

pub fn leading_trailing() -> LeadingTrailing {
    LeadingTrailing::default()
}

pub fn tile_grid() -> TileGrid {
    TileGrid::default()
}

fn debug_overlay_enabled(ctx: &Context) -> bool {
    ctx.data(|data| data.get_temp::<bool>(debug_overlay_id()).unwrap_or(false))
}

fn debug_overlay_id() -> Id {
    Id::new(DEBUG_OVERLAY_ID)
}

fn paint_flow_overlay(ui: &Ui, rect: Rect, axis: Axis, gap: f32) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let label = match axis {
        Axis::Row => format!("row gap={gap:.0}"),
        Axis::Column => format!("column gap={gap:.0}"),
    };
    paint_overlay(ui, rect, &label, flow_overlay_stroke(runtime));
}

fn paint_inset_overlay(ui: &Ui, outer: Rect, inner: Rect, padding_x: i8, padding_y: i8) {
    if !debug_overlay_enabled(ui.ctx()) {
        return;
    }

    let runtime = crate::theme::runtime_for_ui(ui);
    let outer_stroke = Stroke::new(1.0, tokens::layout_debug_outer(runtime));
    let inner_stroke = Stroke::new(1.0, tokens::layout_debug_inner(runtime));
    let painter = ui.painter();
    painter.rect_stroke(
        outer,
        egui::CornerRadius::same(tokens::radius_sm(runtime)),
        outer_stroke,
        StrokeKind::Inside,
    );
    painter.rect_stroke(
        inner,
        egui::CornerRadius::same(tokens::radius_sm(runtime)),
        inner_stroke,
        StrokeKind::Inside,
    );
    paint_overlay_label(
        ui,
        outer.left_top() + egui::vec2(4.0, 4.0),
        &format!("inset {padding_x}x{padding_y}"),
        outer_stroke.color,
    );
}

fn paint_box_overlay(ui: &Ui, rect: Rect, label: &str) {
    let runtime = crate::theme::runtime_for_ui(ui);
    paint_overlay(ui, rect, label, box_overlay_stroke(runtime));
}

fn paint_tile_grid_overlay(ui: &Ui, rect: Rect, columns: usize, column_gap: f32, row_gap: f32) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let label = if (column_gap - row_gap).abs() < f32::EPSILON {
        format!("tile-grid cols={columns} gap={column_gap:.0}")
    } else {
        format!("tile-grid cols={columns} x={column_gap:.0} y={row_gap:.0}")
    };
    paint_overlay(ui, rect, &label, box_overlay_stroke(runtime));
}

fn paint_overlay(ui: &Ui, rect: Rect, label: &str, stroke: Stroke) {
    let runtime = crate::theme::runtime_for_ui(ui);
    if !debug_overlay_enabled(ui.ctx()) || !rect.is_positive() {
        return;
    }

    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(tokens::radius_sm(runtime)),
        stroke,
        StrokeKind::Inside,
    );
    paint_overlay_label(
        ui,
        rect.left_top() + egui::vec2(4.0, 4.0),
        label,
        stroke.color,
    );
}

fn paint_overlay_label(ui: &Ui, pos: egui::Pos2, label: &str, color: egui::Color32) {
    ui.painter().text(
        pos,
        egui::Align2::LEFT_TOP,
        label,
        FontId::monospace(DEBUG_LABEL_FONT_SIZE),
        color,
    );
}

fn flow_overlay_stroke(runtime: crate::theme::ThemeRuntime) -> Stroke {
    Stroke::new(1.0, tokens::layout_debug_flow(runtime))
}

fn box_overlay_stroke(runtime: crate::theme::ThemeRuntime) -> Stroke {
    Stroke::new(1.0, tokens::layout_debug_box(runtime))
}

#[cfg(test)]
mod tests {
    use super::{
        column, inset, leading_trailing, row, sized_box, spacer, tile_grid, Align, Justify,
    };
    use egui::{CentralPanel, Context, RawInput, Rect, Sense};

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
    fn inset_adds_uniform_padding() {
        let context = Context::default();
        let mut outer = Rect::NOTHING;
        let mut inner = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let response = inset().padding(6, 4).show(ui, |ui| {
                    inner = ui
                        .allocate_exact_size(egui::vec2(20.0, 10.0), Sense::hover())
                        .0;
                });
                outer = response.response.rect;
            });
        });

        assert_eq!(inner.left() - outer.left(), 6.0);
        assert_eq!(outer.right() - inner.right(), 6.0);
        assert_eq!(inner.top() - outer.top(), 4.0);
        assert_eq!(outer.bottom() - inner.bottom(), 4.0);
    }

    #[test]
    fn sized_box_uses_explicit_dimensions() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = sized_box()
                    .width(120.0)
                    .height(32.0)
                    .show(ui, |ui| {
                        let _ = super::align()
                            .justify(Justify::Center)
                            .align(Align::Center)
                            .show(ui, |ui| {
                                let _ =
                                    ui.allocate_exact_size(egui::vec2(40.0, 16.0), Sense::hover());
                            });
                    })
                    .response
                    .rect;
            });
        });

        assert_eq!(rect.width(), 120.0);
        assert_eq!(rect.height(), 32.0);
    }

    #[test]
    fn flex_spacer_consumes_remaining_row_width() {
        let context = Context::default();
        let mut spacer_rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                ui.set_width(200.0);
                let _ = row().gap(0.0).show(ui, |ui| {
                    let _ = ui.allocate_exact_size(egui::vec2(20.0, 10.0), Sense::hover());
                    spacer_rect = spacer().show(ui).rect;
                    let _ = ui.allocate_exact_size(egui::vec2(30.0, 10.0), Sense::hover());
                });
            });
        });

        assert!(spacer_rect.width() >= 149.0);
    }

    #[test]
    fn leading_trailing_places_trailing_content_at_row_end() {
        let context = Context::default();
        let mut row_rect = Rect::NOTHING;
        let mut leading_rect = Rect::NOTHING;
        let mut trailing_rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let response = sized_box().width(240.0).show(ui, |ui| {
                    leading_trailing().gap(12.0).show(
                        ui,
                        |ui| {
                            leading_rect = ui
                                .allocate_exact_size(egui::vec2(72.0, 18.0), Sense::hover())
                                .0;
                        },
                        |ui| {
                            trailing_rect = ui
                                .allocate_exact_size(egui::vec2(48.0, 18.0), Sense::hover())
                                .0;
                        },
                    )
                });
                row_rect = response.response.rect;
            });
        });

        assert_eq!(row_rect.width(), 240.0);
        assert_eq!(leading_rect.left(), row_rect.left());
        assert!(trailing_rect.right() <= row_rect.right());
        assert_eq!(trailing_rect.right(), row_rect.right());
        assert!(trailing_rect.left() - leading_rect.right() >= 12.0);
    }

    #[test]
    fn leading_trailing_respects_min_height() {
        let context = Context::default();
        let mut row_rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                row_rect = leading_trailing()
                    .min_height(36.0)
                    .show(
                        ui,
                        |ui| {
                            let _ = ui.allocate_exact_size(egui::vec2(40.0, 12.0), Sense::hover());
                        },
                        |ui| {
                            let _ = ui.allocate_exact_size(egui::vec2(16.0, 12.0), Sense::hover());
                        },
                    )
                    .rect;
            });
        });

        assert!(row_rect.height() >= 36.0);
    }

    #[test]
    fn tile_grid_uses_responsive_columns_and_equal_gaps() {
        let context = Context::default();
        let mut rects = Vec::new();

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = sized_box().width(320.0).show(ui, |ui| {
                    let _ =
                        tile_grid()
                            .min_tile_width(100.0)
                            .gap(10.0)
                            .show(ui, 4, |ui, _, rect| {
                                rects.push(rect);
                                ui.painter().rect_filled(
                                    rect,
                                    egui::CornerRadius::ZERO,
                                    egui::Color32::WHITE,
                                );
                            });
                });
            });
        });

        assert_eq!(rects.len(), 4);
        assert_eq!(rects[1].left() - rects[0].right(), 10.0);
        assert_eq!(rects[2].left() - rects[1].right(), 10.0);
        assert_eq!(rects[3].top() - rects[0].bottom(), 10.0);
        assert_eq!(rects[0].width(), 100.0);
        assert_eq!(rects[0].height(), 100.0);
    }

    #[test]
    fn tile_grid_fixed_columns_preserve_equal_widths() {
        let context = Context::default();
        let mut rects = Vec::new();

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = sized_box().width(208.0).show(ui, |ui| {
                    let _ = tile_grid().columns(2).gap(8.0).tile_height(40.0).show(
                        ui,
                        3,
                        |ui, _, rect| {
                            rects.push(rect);
                            ui.painter().rect_filled(
                                rect,
                                egui::CornerRadius::ZERO,
                                egui::Color32::WHITE,
                            );
                        },
                    );
                });
            });
        });

        assert_eq!(rects.len(), 3);
        assert_eq!(rects[0].width(), 100.0);
        assert_eq!(rects[1].width(), 100.0);
        assert_eq!(rects[2].width(), 100.0);
        assert_eq!(rects[1].left() - rects[0].right(), 8.0);
        assert_eq!(rects[2].top() - rects[0].bottom(), 8.0);
    }

    #[test]
    fn tile_grid_aspect_ratio_sets_tile_height() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = sized_box().width(210.0).show(ui, |ui| {
                    let _ = tile_grid().columns(2).gap(10.0).aspect_ratio(2.0).show(
                        ui,
                        1,
                        |ui, _, tile_rect| {
                            rect = tile_rect;
                            ui.painter().rect_filled(
                                tile_rect,
                                egui::CornerRadius::ZERO,
                                egui::Color32::WHITE,
                            );
                        },
                    );
                });
            });
        });

        assert_eq!(rect.width(), 100.0);
        assert_eq!(rect.height(), 50.0);
    }

    #[test]
    fn tile_grid_fixed_height_overrides_aspect_ratio() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = sized_box().width(210.0).show(ui, |ui| {
                    let _ = tile_grid()
                        .columns(2)
                        .gap(10.0)
                        .aspect_ratio(2.0)
                        .tile_height(60.0)
                        .show(ui, 1, |ui, _, tile_rect| {
                            rect = tile_rect;
                            ui.painter().rect_filled(
                                tile_rect,
                                egui::CornerRadius::ZERO,
                                egui::Color32::WHITE,
                            );
                        });
                });
            });
        });

        assert_eq!(rect.width(), 100.0);
        assert_eq!(rect.height(), 60.0);
    }
}
