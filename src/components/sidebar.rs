use super::{
    api::{with_component_overrides, ComponentUi, ComponentUiExt},
    Button, ButtonVariant, Label, LabelTone, LabelWeight,
};
use crate::layout;
use crate::theme::ColorRole;
use crate::ui::tokens;
use egui::{Color32, CornerRadius, Id, Key, Margin, Order, Stroke, StrokeKind, Ui};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct Sidebar<'a> {
    pub id: Id,
    pub title: Option<&'a str>,
    pub side: SidebarSide,
    pub width: f32,
    pub fill: Option<Color32>,
    pub stroke: Option<Stroke>,
    pub padding_x: i8,
    pub padding_y: i8,
    pub shadow: Option<egui::Shadow>,
    pub backdrop: bool,
    pub close_on_outside_click: bool,
    pub close_on_escape: bool,
}

impl<'a> Sidebar<'a> {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            title: None,
            side: SidebarSide::Left,
            width: 280.0,
            fill: None,
            stroke: None,
            padding_x: 12,
            padding_y: 12,
            shadow: None,
            backdrop: true,
            close_on_outside_click: true,
            close_on_escape: true,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn side(mut self, side: SidebarSide) -> Self {
        self.side = side;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(1.0);
        self
    }

    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn padding(mut self, x: i8, y: i8) -> Self {
        self.padding_x = x;
        self.padding_y = y;
        self
    }

    pub fn shadow(mut self, shadow: egui::Shadow) -> Self {
        self.shadow = Some(shadow);
        self
    }

    pub fn backdrop(mut self, backdrop: bool) -> Self {
        self.backdrop = backdrop;
        self
    }

    pub fn close_on_outside_click(mut self, close_on_outside_click: bool) -> Self {
        self.close_on_outside_click = close_on_outside_click;
        self
    }

    pub fn close_on_escape(mut self, close_on_escape: bool) -> Self {
        self.close_on_escape = close_on_escape;
        self
    }
}

impl<'a> From<Id> for Sidebar<'a> {
    fn from(id: Id) -> Self {
        Self::new(id)
    }
}

impl ComponentUi<'_> {
    pub fn sidebar<'a>(
        &mut self,
        open: &mut bool,
        props: impl Into<Sidebar<'a>>,
        add: impl FnOnce(&mut Ui, &mut bool),
    ) {
        let host_rect = self.ui().max_rect();
        draw_sidebar_in(self, host_rect, open, props.into(), add);
    }

    pub fn sidebar_in<'a>(
        &mut self,
        host_rect: egui::Rect,
        open: &mut bool,
        props: impl Into<Sidebar<'a>>,
        add: impl FnOnce(&mut Ui, &mut bool),
    ) {
        draw_sidebar_in(self, host_rect, open, props.into(), add);
    }
}

fn draw_sidebar_in(
    component_ui: &mut ComponentUi<'_>,
    host_rect: egui::Rect,
    open: &mut bool,
    props: Sidebar<'_>,
    add: impl FnOnce(&mut Ui, &mut bool),
) {
    let ctx = component_ui.ctx().clone();
    let runtime = crate::theme::runtime_for_ui(component_ui.ui());
    let overrides = component_ui.overrides();
    let animation_id = props.id.with("open");
    let panel_area_id = props.id.with("panel");
    let backdrop_area_id = props.id.with("backdrop");
    let openness = ctx.animate_bool_responsive(animation_id, *open);

    if !*open && openness <= 0.0 {
        return;
    }

    let fill = props
        .fill
        .unwrap_or(crate::theme::resolved_color(runtime, ColorRole::Sidebar));
    let stroke = props.stroke.unwrap_or(Stroke::new(
        1.0,
        crate::theme::resolved_color(runtime, ColorRole::SidebarBorder),
    ));
    let shadow = props.shadow.unwrap_or(tokens::tailwind_shadow_lg(runtime));
    let panel_rect = sidebar_panel_rect(host_rect, props.side, props.width, openness);
    let corner_radius = sidebar_corner_radius(props.side, tokens::radius_lg(runtime));
    let mut close_requested = false;

    if props.backdrop && openness > 0.0 {
        let backdrop_fill =
            tokens::dialogue_backdrop(runtime).gamma_multiply((0.85 * openness).clamp(0.0, 1.0));
        let backdrop_response = egui::Area::new(backdrop_area_id)
            .order(Order::Foreground)
            .fixed_pos(host_rect.left_top())
            .constrain_to(host_rect)
            .interactable(true)
            .show(&ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(host_rect.size(), egui::Sense::click());
                ui.painter().rect(
                    rect,
                    CornerRadius::ZERO,
                    backdrop_fill,
                    egui::Stroke::NONE,
                    StrokeKind::Outside,
                );
                response
            })
            .inner;

        if props.close_on_outside_click && backdrop_response.clicked() {
            close_requested = true;
        }
    }

    let _ = egui::Area::new(panel_area_id)
        .order(Order::Foreground)
        .fixed_pos(panel_rect.min)
        .constrain_to(host_rect)
        .show(&ctx, |ui| {
            ui.set_min_width(props.width);
            ui.set_max_width(props.width);
            ui.set_min_height(host_rect.height());
            ui.set_max_height(host_rect.height());

            let inner_height = (host_rect.height() - f32::from(props.padding_y * 2)).max(1.0);
            let _ = egui::Frame::new()
                .fill(fill)
                .stroke(stroke)
                .corner_radius(corner_radius)
                .inner_margin(Margin::symmetric(props.padding_x, props.padding_y))
                .shadow(shadow)
                .show(ui, |ui| {
                    ui.set_min_height(inner_height);
                    with_component_overrides(ui, overrides, |ui| {
                        if let Some(title) = props.title {
                            draw_sidebar_header(ui, title, runtime, &mut close_requested);
                            ui.add_space(10.0);
                            let _ = ui.components().separator();
                            ui.add_space(10.0);
                        }
                        let body_max_height = ui.available_height().max(1.0);
                        let _ = egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .max_height(body_max_height)
                            .show(ui, |ui| add(ui, &mut close_requested));
                    });
                });
        });

    if props.close_on_escape && ctx.input(|input| input.key_pressed(Key::Escape)) {
        close_requested = true;
    }

    if close_requested {
        *open = false;
    }
}

fn sidebar_panel_rect(
    host_rect: egui::Rect,
    side: SidebarSide,
    width: f32,
    openness: f32,
) -> egui::Rect {
    let panel_left = match side {
        SidebarSide::Left => host_rect.left() - (width * (1.0 - openness)),
        SidebarSide::Right => host_rect.right() - (width * openness),
    };
    egui::Rect::from_min_size(
        egui::pos2(panel_left, host_rect.top()),
        egui::vec2(width, host_rect.height()),
    )
}

fn draw_sidebar_header(
    ui: &mut Ui,
    title: &str,
    runtime: crate::theme::ThemeRuntime,
    close_requested: &mut bool,
) {
    let _ = layout::leading_trailing().gap(8.0).min_height(28.0).show(
        ui,
        |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new(title)
                    .tone(LabelTone::Primary)
                    .weight(LabelWeight::Semibold)
                    .size(16.0),
            );
        },
        |ui| {
            if ui
                .components()
                .button(
                    Button::icon_only("x")
                        .variant(ButtonVariant::Ghost)
                        .size(super::ControlSize::Sm)
                        .icon_size(12.0)
                        .icon_tint(crate::theme::resolved_color(
                            runtime,
                            ColorRole::SidebarForeground,
                        ))
                        .min_size(egui::vec2(28.0, 28.0)),
                )
                .clicked()
            {
                *close_requested = true;
            }
        },
    );
}

fn sidebar_corner_radius(side: SidebarSide, radius: u8) -> CornerRadius {
    match side {
        SidebarSide::Left => CornerRadius {
            nw: 0,
            ne: radius,
            sw: 0,
            se: radius,
        },
        SidebarSide::Right => CornerRadius {
            nw: radius,
            ne: 0,
            sw: radius,
            se: 0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{sidebar_panel_rect, Sidebar, SidebarSide};
    use crate::components::{Button, ButtonVariant, ComponentUiExt};
    use crate::theme::{self, ThemeMode};
    use egui::{pos2, vec2, CentralPanel, Context, Id, RawInput, Rect, SidePanel};

    #[test]
    fn derived_area_ids_avoid_colliding_with_host_panel_ids() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let mut open = true;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = context.run(RawInput::default(), |context| {
                SidePanel::left("sidebar_collision").show(context, |ui| {
                    let _ = ui
                        .components()
                        .button(Button::new("Host").variant(ButtonVariant::Ghost));
                });

                CentralPanel::default().show(context, |ui| {
                    ui.components().sidebar(
                        &mut open,
                        Sidebar::new(Id::new("sidebar_collision"))
                            .title("Workspace")
                            .side(SidebarSide::Left),
                        |ui, _open| {
                            let _ = ui
                                .components()
                                .button(Button::new("Inner").variant(ButtonVariant::Primary));
                        },
                    );
                });
            });
        }));

        assert!(result.is_ok());
    }

    #[test]
    fn sidebar_panel_rect_anchors_to_host_edges_when_open() {
        let host_rect = Rect::from_min_size(pos2(24.0, 48.0), vec2(640.0, 320.0));

        let left = sidebar_panel_rect(host_rect, SidebarSide::Left, 240.0, 1.0);
        let right = sidebar_panel_rect(host_rect, SidebarSide::Right, 240.0, 1.0);

        assert_eq!(left.left(), host_rect.left());
        assert_eq!(left.width(), 240.0);
        assert_eq!(left.height(), host_rect.height());

        assert_eq!(right.right(), host_rect.right());
        assert_eq!(right.width(), 240.0);
        assert_eq!(right.height(), host_rect.height());
    }

    #[test]
    fn sidebar_panel_rect_starts_outside_host_when_closed() {
        let host_rect = Rect::from_min_size(pos2(24.0, 48.0), vec2(640.0, 320.0));

        let left = sidebar_panel_rect(host_rect, SidebarSide::Left, 240.0, 0.0);
        let right = sidebar_panel_rect(host_rect, SidebarSide::Right, 240.0, 0.0);

        assert_eq!(left.right(), host_rect.left());
        assert_eq!(right.left(), host_rect.right());
    }
}
