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

#[cfg(feature = "showcase")]
pub(crate) fn set_debug_overlay(ctx: &Context, enabled: bool) {
    ctx.data_mut(|data| data.insert_temp(debug_overlay_id(), enabled));
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
    use super::{column, inset, row, sized_box, spacer, Align, Justify};
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
}
