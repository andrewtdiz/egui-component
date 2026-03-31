use super::{
    api::{with_component_overrides, ComponentUi, ComponentUiExt},
    Button, ButtonVariant, Label, LabelTone, LabelWeight,
};
use crate::primitives::surface::{surface_frame, SurfaceFrame};
use crate::theme::ColorRole;
use crate::ui::tokens;
use egui::{Align, Align2, Color32, Id, Layout, Order, Rect, Stroke, Ui, Vec2};

const DEFAULT_TOAST_WIDTH: f32 = 320.0;
const DEFAULT_TOAST_DURATION_SECS: f32 = 4.0;
const TOAST_FRAME_PADDING_X: i8 = 10;
const TOAST_FRAME_PADDING_Y: i8 = 10;
const TOAST_CLOSE_BUTTON_SIZE: f32 = 22.0;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToastIntent {
    #[default]
    Neutral,
    Success,
    Destructive,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToastPlacement {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    #[default]
    BottomRight,
}

impl ToastPlacement {
    fn anchor(self) -> Align2 {
        match self {
            Self::TopLeft => Align2::LEFT_TOP,
            Self::TopCenter => Align2::CENTER_TOP,
            Self::TopRight => Align2::RIGHT_TOP,
            Self::CenterLeft => Align2::LEFT_CENTER,
            Self::Center => Align2::CENTER_CENTER,
            Self::CenterRight => Align2::RIGHT_CENTER,
            Self::BottomLeft => Align2::LEFT_BOTTOM,
            Self::BottomCenter => Align2::CENTER_BOTTOM,
            Self::BottomRight => Align2::RIGHT_BOTTOM,
        }
    }

    fn area_layout(self) -> Layout {
        match self {
            Self::TopLeft => Layout::top_down(Align::Min),
            Self::TopCenter => Layout::top_down(Align::Center),
            Self::TopRight => Layout::top_down(Align::Max),
            Self::CenterLeft => Layout::top_down(Align::Min),
            Self::Center => Layout::top_down(Align::Center),
            Self::CenterRight => Layout::top_down(Align::Max),
            Self::BottomLeft => Layout::bottom_up(Align::Min),
            Self::BottomCenter => Layout::bottom_up(Align::Center),
            Self::BottomRight => Layout::bottom_up(Align::Max),
        }
    }

    fn anchor_offset(self, margin: Vec2) -> Vec2 {
        match self {
            Self::TopLeft => margin,
            Self::TopCenter => egui::vec2(0.0, margin.y),
            Self::TopRight => egui::vec2(-margin.x, margin.y),
            Self::CenterLeft => egui::vec2(margin.x, 0.0),
            Self::Center => Vec2::ZERO,
            Self::CenterRight => egui::vec2(-margin.x, 0.0),
            Self::BottomLeft => egui::vec2(margin.x, -margin.y),
            Self::BottomCenter => egui::vec2(0.0, -margin.y),
            Self::BottomRight => egui::vec2(-margin.x, -margin.y),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub title: String,
    pub description: Option<String>,
    pub intent: ToastIntent,
    pub duration_secs: f32,
}

impl Toast {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            intent: ToastIntent::Neutral,
            duration_secs: DEFAULT_TOAST_DURATION_SECS,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn intent(mut self, intent: ToastIntent) -> Self {
        self.intent = intent;
        self
    }

    pub fn duration(mut self, duration_secs: f32) -> Self {
        self.duration_secs = duration_secs.max(0.0);
        self
    }
}

impl From<&str> for Toast {
    fn from(title: &str) -> Self {
        Self::new(title)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ToastViewport {
    pub id: Id,
    pub placement: ToastPlacement,
    pub width: f32,
    pub margin: Vec2,
    pub gap: f32,
    pub overlap: f32,
    pub max_visible: usize,
}

impl ToastViewport {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            placement: ToastPlacement::BottomRight,
            width: DEFAULT_TOAST_WIDTH,
            margin: egui::vec2(16.0, 16.0),
            gap: 8.0,
            overlap: 0.0,
            max_visible: 4,
        }
    }

    pub fn placement(mut self, placement: ToastPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(1.0);
        self
    }

    pub fn margin(mut self, margin: Vec2) -> Self {
        self.margin = egui::vec2(margin.x.max(0.0), margin.y.max(0.0));
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    pub fn overlap(mut self, overlap: f32) -> Self {
        self.overlap = overlap.max(0.0);
        self
    }

    pub fn max_visible(mut self, max_visible: usize) -> Self {
        self.max_visible = max_visible.max(1);
        self
    }
}

impl From<Id> for ToastViewport {
    fn from(id: Id) -> Self {
        Self::new(id)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ToastStack {
    next_id: u64,
    entries: Vec<ToastEntry>,
}

impl ToastStack {
    pub fn push(&mut self, toast: impl Into<Toast>) {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.entries.push(ToastEntry {
            id,
            toast: toast.into(),
            shown_at: None,
        });
    }

    pub fn neutral(&mut self, title: impl Into<String>) {
        self.push(Toast::new(title));
    }

    pub fn success(&mut self, title: impl Into<String>) {
        self.push(Toast::new(title).intent(ToastIntent::Success));
    }

    pub fn destructive(&mut self, title: impl Into<String>) {
        self.push(Toast::new(title).intent(ToastIntent::Destructive));
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone)]
struct ToastEntry {
    id: u64,
    toast: Toast,
    shown_at: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
struct ToastPalette {
    fill: Color32,
    stroke: Stroke,
}

impl ComponentUi<'_> {
    pub fn toast_viewport(&mut self, stack: &mut ToastStack, props: impl Into<ToastViewport>) {
        let _ = draw_toast_viewport(self, stack, props.into());
    }
}

fn draw_toast_viewport(
    ui: &mut ComponentUi<'_>,
    stack: &mut ToastStack,
    props: ToastViewport,
) -> Rect {
    if stack.entries.is_empty() {
        return Rect::NOTHING;
    }

    let ctx = ui.ctx().clone();
    let runtime = crate::theme::runtime_for_ui(ui.ui());
    let host_rect = ui.ui().max_rect();
    let now = ctx.input(|input| input.time);
    let mut next_expiration: Option<f64> = None;

    for entry in &mut stack.entries {
        if entry.shown_at.is_none() {
            entry.shown_at = Some(now);
        }
        if entry.toast.duration_secs > 0.0 {
            let expiry = entry.shown_at.unwrap_or(now) + f64::from(entry.toast.duration_secs);
            next_expiration = Some(match next_expiration {
                Some(current) => current.min(expiry),
                None => expiry,
            });
        }
    }

    stack.entries.retain(|entry| !toast_expired(entry, now));
    if stack.entries.is_empty() {
        return Rect::NOTHING;
    }

    if let Some(expiry) = next_expiration {
        let remaining = (expiry - now).max(0.0) as f32;
        ctx.request_repaint_after_secs(remaining.max(0.05));
    }

    let mut dismiss_ids = Vec::new();
    let visible_entries = stack
        .entries
        .iter()
        .rev()
        .take(props.max_visible)
        .collect::<Vec<_>>();
    let overrides = ui.overrides();
    let viewport_area_id = props.id.with("viewport");

    let area_response = egui::Area::new(viewport_area_id)
        .order(Order::Foreground)
        .anchor(
            props.placement.anchor(),
            props.placement.anchor_offset(props.margin),
        )
        .constrain_to(host_rect)
        .layout(props.placement.area_layout())
        .show(&ctx, |ui| {
            ui.set_min_width(props.width);
            ui.set_max_width(props.width);
            ui.spacing_mut().item_spacing.y = props.gap;

            let _ = ui.with_layout(props.placement.area_layout(), |ui| {
                for (depth, entry) in visible_entries.iter().enumerate() {
                    let _ = ui.push_id(entry.id, |ui| {
                        let result = draw_toast(
                            ui,
                            overrides,
                            entry.toast.clone(),
                            runtime,
                            props.width,
                            depth,
                        );
                        if result.dismissed {
                            dismiss_ids.push(entry.id);
                        }
                    });

                    if depth + 1 < visible_entries.len() && props.overlap > 0.0 {
                        ui.add_space(-(props.gap + props.overlap));
                    }
                }
            });
        });

    stack
        .entries
        .retain(|entry| !dismiss_ids.iter().any(|dismissed| *dismissed == entry.id));
    area_response.response.rect
}

#[derive(Debug, Clone, Copy)]
struct ToastDrawResult {
    rect: Rect,
    dismissed: bool,
}

fn draw_toast(
    ui: &mut Ui,
    overrides: super::api::ComponentOverrides,
    toast: Toast,
    runtime: crate::theme::ThemeRuntime,
    width: f32,
    depth: usize,
) -> ToastDrawResult {
    let palette = toast_palette(runtime, toast.intent, depth);
    let mut result = ToastDrawResult {
        rect: Rect::NOTHING,
        dismissed: false,
    };
    let shadow = toast_shadow(runtime, depth);
    let inner_width = (width - f32::from(TOAST_FRAME_PADDING_X * 2)).max(1.0);

    let response = ui.with_layout(Layout::top_down(Align::Min), |ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);

        let frame_response = surface_frame(
            ui,
            SurfaceFrame::new(palette.fill, palette.stroke)
                .corner_radius(tokens::radius_lg(runtime))
                .padding(TOAST_FRAME_PADDING_X, TOAST_FRAME_PADDING_Y)
                .shadow(shadow),
            |ui| {
                ui.set_min_width(inner_width);
                ui.set_max_width(inner_width);
                with_component_overrides(ui, overrides, |ui| {
                    let _ = ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        let width = ui.available_width();
                        let _ = ui.allocate_ui_with_layout(
                            egui::vec2(width, TOAST_CLOSE_BUTTON_SIZE),
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                ui.spacing_mut().item_spacing.x = 8.0;
                                let _ = ui.components().label(
                                    Label::new(toast.title.as_str())
                                        .tone(LabelTone::Primary)
                                        .weight(LabelWeight::Semibold),
                                );

                                let trailing_width = ui.available_width().max(0.0);
                                let _ = ui.allocate_ui_with_layout(
                                    egui::vec2(trailing_width, TOAST_CLOSE_BUTTON_SIZE),
                                    Layout::right_to_left(Align::Center),
                                    |ui| {
                                        if ui
                                            .components()
                                            .button(
                                                Button::icon_only("x")
                                                    .variant(ButtonVariant::Ghost)
                                                    .size(super::ControlSize::Sm)
                                                    .icon_size(12.0)
                                                    .icon_tint(tokens::text_muted(runtime))
                                                    .min_size(egui::vec2(
                                                        TOAST_CLOSE_BUTTON_SIZE,
                                                        TOAST_CLOSE_BUTTON_SIZE,
                                                    )),
                                            )
                                            .clicked()
                                        {
                                            result.dismissed = true;
                                        }
                                    },
                                );
                            },
                        );

                        if let Some(description) = toast.description.as_deref() {
                            ui.add_space(4.0);
                            let _ = ui
                                .components()
                                .label(Label::new(description).tone(LabelTone::Muted).size(12.0));
                        }
                    });
                });
            },
        );
        frame_response.response.rect
    });
    result.rect = response.inner;

    result
}

fn toast_expired(entry: &ToastEntry, now: f64) -> bool {
    entry.toast.duration_secs > 0.0
        && entry
            .shown_at
            .is_some_and(|shown_at| now >= shown_at + f64::from(entry.toast.duration_secs))
}

fn toast_palette(
    runtime: crate::theme::ThemeRuntime,
    intent: ToastIntent,
    depth: usize,
) -> ToastPalette {
    let card_fill = tokens::card_background(runtime);
    let border = tokens::separator(runtime);
    let mut palette = match intent {
        ToastIntent::Neutral => ToastPalette {
            fill: card_fill,
            stroke: Stroke::new(1.0, border),
        },
        ToastIntent::Success => {
            let accent = Color32::from_rgb(34, 197, 94);
            ToastPalette {
                fill: card_fill
                    .lerp_to_gamma(accent, if runtime.mode.is_dark() { 0.18 } else { 0.1 }),
                stroke: Stroke::new(1.0, border.lerp_to_gamma(accent, 0.55)),
            }
        }
        ToastIntent::Destructive => {
            let accent = crate::theme::resolved_color(runtime, ColorRole::Destructive);
            ToastPalette {
                fill: card_fill
                    .lerp_to_gamma(accent, if runtime.mode.is_dark() { 0.2 } else { 0.12 }),
                stroke: Stroke::new(1.0, border.lerp_to_gamma(accent, 0.6)),
            }
        }
    };

    let shade = (depth as f32 * 0.075).clamp(0.0, 0.28);
    let stroke_shade = (depth as f32 * 0.05).clamp(0.0, 0.18);
    palette.fill = palette
        .fill
        .lerp_to_gamma(tokens::muted_surface(runtime), shade * 0.8)
        .gamma_multiply(1.0 - (shade * 0.22));
    palette.stroke.color = palette
        .stroke
        .color
        .lerp_to_gamma(tokens::input_border(runtime), stroke_shade);
    palette
}

fn toast_shadow(runtime: crate::theme::ThemeRuntime, depth: usize) -> egui::Shadow {
    let mut shadow = tokens::tailwind_shadow_md(runtime);
    let fade = (1.0 - (depth as f32 * 0.18)).clamp(0.3, 1.0);
    shadow.color = shadow.color.gamma_multiply(fade);
    shadow.blur = (shadow.blur as f32 * (1.0 - (depth as f32 * 0.12)).clamp(0.45, 1.0)) as u8;
    shadow
}

#[cfg(test)]
mod tests {
    use super::{
        draw_toast, draw_toast_viewport, toast_shadow, Toast, ToastIntent, ToastPlacement,
        ToastStack, ToastViewport,
    };
    use crate::components::api::ComponentOverrides;
    use crate::components::ComponentUiExt;
    use crate::ui::tokens;
    use egui::{Align, CentralPanel, Context, Id, Layout, RawInput, Rect, UiBuilder};

    #[test]
    fn stack_pushes_ordered_toasts() {
        let mut stack = ToastStack::default();
        stack.push(Toast::new("First"));
        stack.push(Toast::new("Second").intent(ToastIntent::Success));

        assert_eq!(stack.entries.len(), 2);
        assert_eq!(stack.entries[0].toast.title, "First");
        assert_eq!(stack.entries[1].toast.intent, ToastIntent::Success);
    }

    #[test]
    fn toast_respects_requested_width_with_close_button() {
        let context = Context::default();
        let mut toast_rect = Rect::NOTHING;
        crate::theme::install(
            &context,
            crate::theme::ThemeSpec::default(),
            crate::theme::ThemeMode::Dark,
        );

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let runtime = crate::theme::runtime_for_ui(ui);
                toast_rect = draw_toast(
                    ui,
                    ComponentOverrides::default(),
                    Toast::new("Build failed")
                        .description("A component export is missing a required icon mapping.")
                        .intent(ToastIntent::Destructive),
                    runtime,
                    280.0,
                    0,
                )
                .rect;
            });
        });

        assert!(toast_rect.width() >= 280.0);
        assert!(toast_rect.width() <= 282.0);
    }

    #[test]
    fn toast_stays_compact_inside_bottom_up_layout() {
        let context = Context::default();
        let mut toast_rect = Rect::NOTHING;
        let host_rect = Rect::from_min_size(egui::pos2(40.0, 30.0), egui::vec2(320.0, 240.0));
        crate::theme::install(
            &context,
            crate::theme::ThemeSpec::default(),
            crate::theme::ThemeMode::Dark,
        );

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = ui.scope_builder(
                    UiBuilder::new()
                        .max_rect(host_rect)
                        .layout(Layout::bottom_up(Align::Center)),
                    |ui| {
                        let runtime = crate::theme::runtime_for_ui(ui);
                        toast_rect = draw_toast(
                            ui,
                            ComponentOverrides::default(),
                            Toast::new("Changes saved")
                                .description("Your layout tokens were published successfully.")
                                .intent(ToastIntent::Success)
                                .duration(0.0),
                            runtime,
                            280.0,
                            0,
                        )
                        .rect;
                    },
                );
            });
        });

        assert!(toast_rect.width() >= 280.0);
        assert!(toast_rect.width() <= 282.0);
        assert!(toast_rect.height() < 120.0);
    }

    #[test]
    fn top_right_toast_viewport_stays_within_host_rect() {
        let context = Context::default();
        let mut viewport_rect = Rect::NOTHING;
        let host_rect = Rect::from_min_size(egui::pos2(40.0, 30.0), egui::vec2(320.0, 240.0));
        let mut stack = ToastStack::default();
        crate::theme::install(
            &context,
            crate::theme::ThemeSpec::default(),
            crate::theme::ThemeMode::Dark,
        );
        stack.push(
            Toast::new("Build failed")
                .description("A component export is missing a required icon mapping.")
                .intent(ToastIntent::Destructive)
                .duration(0.0),
        );
        stack.push(
            Toast::new("Build failed")
                .description("A component export is missing a required icon mapping.")
                .intent(ToastIntent::Destructive)
                .duration(0.0),
        );

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = ui.scope_builder(UiBuilder::new().max_rect(host_rect), |ui| {
                    let mut components = ui.components();
                    viewport_rect = draw_toast_viewport(
                        &mut components,
                        &mut stack,
                        ToastViewport::new(Id::new("toast_viewport_top_right_test"))
                            .placement(ToastPlacement::TopRight)
                            .width(280.0),
                    );
                });
            });
        });

        assert!(viewport_rect.right() <= host_rect.right());
        assert!(viewport_rect.left() >= host_rect.left());
    }

    #[test]
    fn toast_shadow_keeps_md_offset_and_fades_with_depth() {
        let context = Context::default();
        crate::theme::install(
            &context,
            crate::theme::ThemeSpec::default(),
            crate::theme::ThemeMode::Dark,
        );

        let runtime = crate::theme::runtime_for_context(&context);
        let base = tokens::tailwind_shadow_md(runtime);

        assert_eq!(toast_shadow(runtime, 0), base);

        let stacked = toast_shadow(runtime, 2);
        assert_eq!(stacked.offset, base.offset);
        assert!(stacked.blur < base.blur);
        assert!(stacked.color.a() < base.color.a());
    }
}
