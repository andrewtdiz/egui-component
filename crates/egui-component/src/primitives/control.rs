use crate::ui::tokens;
use egui::{Color32, CornerRadius, FontId, Rect, Stroke, StrokeKind, Ui, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct ControlFrame {
    fill: Color32,
    stroke: Stroke,
    corner_radius: Option<u8>,
    stroke_kind: StrokeKind,
}

impl ControlFrame {
    pub fn new(fill: Color32, stroke: Stroke) -> Self {
        Self {
            fill,
            stroke,
            corner_radius: None,
            stroke_kind: StrokeKind::Outside,
        }
    }

    pub fn corner_radius(mut self, corner_radius: u8) -> Self {
        self.corner_radius = Some(corner_radius);
        self
    }

    pub fn stroke_kind(mut self, stroke_kind: StrokeKind) -> Self {
        self.stroke_kind = stroke_kind;
        self
    }

    pub fn paint(self, ui: &mut Ui, rect: Rect) {
        control_frame(ui, rect, self);
    }
}

pub(crate) fn with_input_chrome<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        let runtime = crate::theme::runtime_for_ui(ui);
        let style = ui.style_mut();
        style.visuals.text_edit_bg_color = Some(tokens::input_background(runtime));
        style.visuals.code_bg_color = tokens::input_background(runtime);
        style.visuals.selection.bg_fill = tokens::text_selection_bg(runtime);
        style.visuals.selection.stroke = Stroke::new(1.0, tokens::text_primary(runtime));
        let visuals = &mut style.visuals.widgets;
        visuals.noninteractive.corner_radius = CornerRadius::same(tokens::radius_md(runtime));
        visuals.inactive.corner_radius = CornerRadius::same(tokens::radius_md(runtime));
        visuals.hovered.corner_radius = CornerRadius::same(tokens::radius_md(runtime));
        visuals.active.corner_radius = CornerRadius::same(tokens::radius_md(runtime));
        visuals.open.corner_radius = CornerRadius::same(tokens::radius_md(runtime));
        visuals.inactive.bg_fill = tokens::input_background(runtime);
        visuals.inactive.weak_bg_fill = tokens::input_background(runtime);
        visuals.hovered.bg_fill = tokens::input_hover_background(runtime);
        visuals.hovered.weak_bg_fill = tokens::input_hover_background(runtime);
        visuals.active.bg_fill = tokens::input_hover_background(runtime);
        visuals.active.weak_bg_fill = tokens::input_hover_background(runtime);
        visuals.open.bg_fill = tokens::input_focus_background(runtime);
        visuals.open.weak_bg_fill = tokens::input_focus_background(runtime);
        visuals.inactive.bg_stroke = Stroke::new(1.0, tokens::input_border(runtime));
        visuals.hovered.bg_stroke = Stroke::new(1.0, tokens::input_hover_border(runtime));
        visuals.active.bg_stroke = Stroke::new(1.0, tokens::input_hover_border(runtime));
        visuals.open.bg_stroke = tokens::input_focus_stroke(runtime);
        add(ui)
    })
    .inner
}

pub(crate) fn with_slider_chrome<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        let runtime = crate::theme::runtime_for_ui(ui);
        let style = ui.style_mut();
        style.spacing.slider_rail_height = 4.0;
        style.visuals.selection.bg_fill = tokens::TRANSPARENT;
        style.visuals.selection.stroke = Stroke::NONE;
        let visuals = &mut style.visuals.widgets;
        visuals.noninteractive.corner_radius = CornerRadius::same(tokens::radius_md(runtime));
        visuals.inactive.corner_radius = CornerRadius::same(tokens::radius_md(runtime));
        visuals.hovered.corner_radius = CornerRadius::same(tokens::radius_md(runtime));
        visuals.active.corner_radius = CornerRadius::same(tokens::radius_md(runtime));
        visuals.open.corner_radius = CornerRadius::same(tokens::radius_md(runtime));
        visuals.inactive.bg_fill = tokens::TRANSPARENT;
        visuals.inactive.weak_bg_fill = tokens::TRANSPARENT;
        visuals.hovered.bg_fill = tokens::TRANSPARENT;
        visuals.hovered.weak_bg_fill = tokens::TRANSPARENT;
        visuals.active.bg_fill = tokens::TRANSPARENT;
        visuals.active.weak_bg_fill = tokens::TRANSPARENT;
        visuals.open.bg_fill = tokens::TRANSPARENT;
        visuals.open.weak_bg_fill = tokens::TRANSPARENT;
        visuals.noninteractive.bg_fill = tokens::TRANSPARENT;
        visuals.noninteractive.weak_bg_fill = tokens::TRANSPARENT;
        visuals.inactive.bg_stroke = Stroke::NONE;
        visuals.hovered.bg_stroke = Stroke::NONE;
        visuals.active.bg_stroke = Stroke::NONE;
        visuals.open.bg_stroke = Stroke::NONE;
        visuals.noninteractive.bg_stroke = Stroke::NONE;
        visuals.inactive.fg_stroke = Stroke::NONE;
        visuals.hovered.fg_stroke = Stroke::NONE;
        visuals.active.fg_stroke = Stroke::NONE;
        visuals.open.fg_stroke = Stroke::NONE;
        add(ui)
    })
    .inner
}

pub(crate) fn centered_input_vertical_padding(ui: &mut Ui, font_id: &FontId) -> f32 {
    let row_height = ui.fonts_mut(|fonts| fonts.row_height(font_id));
    ((ui.spacing().interact_size.y - row_height).max(0.0) * 0.5).round()
}

pub(crate) fn centered_input_button_padding(
    ui: &mut Ui,
    font_id: &FontId,
    horizontal_padding: f32,
) -> Vec2 {
    egui::vec2(
        horizontal_padding.max(0.0),
        centered_input_vertical_padding(ui, font_id),
    )
}

pub fn control_frame(ui: &mut Ui, rect: Rect, frame: ControlFrame) {
    let runtime = crate::theme::runtime_for_ui(ui);
    ui.painter().rect(
        rect,
        CornerRadius::same(frame.corner_radius.unwrap_or(tokens::radius_md(runtime))),
        frame.fill,
        frame.stroke,
        frame.stroke_kind,
    );
}
