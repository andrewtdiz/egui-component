use super::{
    api::ComponentUi,
    button::ButtonVariant,
    common::ControlSize,
    dropdown_menu::{show_menu_entries_surface, DropdownMenuAction, DropdownMenuEntry},
};
use crate::ui::{icons, tokens, typography};
use egui::{
    vec2, Align2, Color32, CornerRadius, CursorIcon, FontId, Id, Popup, Pos2, Rect, RectAlign,
    Response, Sense, Stroke, StrokeKind, Ui,
};

const OPEN_WITH_ICON_GAP: f32 = 8.0;

#[derive(Debug, Clone, Copy)]
pub struct OpenWith<'a> {
    pub id: Id,
    pub entries: &'a [DropdownMenuEntry<'a>],
    pub width: f32,
    pub placeholder: &'a str,
    pub size: ControlSize,
    pub trigger_variant: ButtonVariant,
    pub icon_size: f32,
}

impl<'a> OpenWith<'a> {
    pub fn new(id: Id, entries: &'a [DropdownMenuEntry<'a>]) -> Self {
        Self {
            id,
            entries,
            width: 220.0,
            placeholder: "Open With",
            size: ControlSize::Md,
            trigger_variant: ButtonVariant::Secondary,
            icon_size: 14.0,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(1.0);
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn trigger_variant(mut self, trigger_variant: ButtonVariant) -> Self {
        self.trigger_variant = trigger_variant;
        self
    }

    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = icon_size.max(1.0);
        self
    }
}

impl ComponentUi<'_> {
    pub fn open_with<'a>(
        &mut self,
        selected_action: &mut Option<usize>,
        props: impl Into<OpenWith<'a>>,
    ) -> Response {
        draw_open_with(self.ui_mut(), selected_action, props.into())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SegmentState {
    Idle,
    Hovered,
    Active,
}

#[derive(Debug, Clone)]
struct OpenWithStyle {
    height: f32,
    padding_x: f32,
    arrow_width: f32,
    corner_radius: u8,
    fill_idle: Color32,
    fill_hover: Color32,
    fill_active: Color32,
    stroke_idle: Stroke,
    stroke_hover: Stroke,
    stroke_active: Stroke,
    label_idle: Color32,
    label_hover: Color32,
    label_active: Color32,
    icon_idle: Color32,
    icon_hover: Color32,
    icon_active: Color32,
    arrow_idle: Color32,
    arrow_hover: Color32,
    arrow_active: Color32,
    label_font: FontId,
}

impl OpenWithStyle {
    fn fill(&self, state: SegmentState) -> Color32 {
        match state {
            SegmentState::Idle => self.fill_idle,
            SegmentState::Hovered => self.fill_hover,
            SegmentState::Active => self.fill_active,
        }
    }

    fn stroke(&self, state: SegmentState) -> Stroke {
        match state {
            SegmentState::Idle => self.stroke_idle,
            SegmentState::Hovered => self.stroke_hover,
            SegmentState::Active => self.stroke_active,
        }
    }

    fn label_color(&self, state: SegmentState) -> Color32 {
        match state {
            SegmentState::Idle => self.label_idle,
            SegmentState::Hovered => self.label_hover,
            SegmentState::Active => self.label_active,
        }
    }

    fn icon_tint(&self, state: SegmentState) -> Color32 {
        match state {
            SegmentState::Idle => self.icon_idle,
            SegmentState::Hovered => self.icon_hover,
            SegmentState::Active => self.icon_active,
        }
    }

    fn arrow_tint(&self, state: SegmentState) -> Color32 {
        match state {
            SegmentState::Idle => self.arrow_idle,
            SegmentState::Hovered => self.arrow_hover,
            SegmentState::Active => self.arrow_active,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct OpenWithMetrics {
    left_width: f32,
    arrow_width: f32,
    total_width: f32,
}

fn draw_open_with(
    ui: &mut Ui,
    selected_action: &mut Option<usize>,
    props: OpenWith<'_>,
) -> Response {
    ui.push_id(props.id, |ui| {
        let runtime = crate::theme::runtime_for_ui(ui);
        let current_action = resolve_open_with_action(props.entries, *selected_action)
            .or_else(|| first_open_with_action(props.entries));
        let current_action_id = current_action.map(|action| action.id);
        let current_label = current_action
            .map(|action| action.label)
            .unwrap_or(props.placeholder);
        let current_icon = current_action.and_then(|action| action.icon);
        let style = resolve_open_with_style(runtime, props.size, props.trigger_variant);
        let metrics =
            resolve_open_with_metrics(ui, current_label, current_icon, props.icon_size, &style);

        let (group_rect, _) =
            ui.allocate_exact_size(vec2(metrics.total_width, style.height), Sense::hover());
        let left_rect = Rect::from_min_size(group_rect.min, vec2(metrics.left_width, style.height));
        let right_rect = Rect::from_min_size(
            Pos2::new(left_rect.right(), group_rect.top()),
            vec2(metrics.arrow_width, style.height),
        );

        let left_response = ui.interact(left_rect, ui.id().with("left"), Sense::click());
        let right_response = ui.interact(right_rect, ui.id().with("right"), Sense::click());
        let group_hover = ui.interact(group_rect, ui.id().with("group"), Sense::hover());

        let popup_id = props.id.with("popup");
        let popup_open = Popup::is_id_open(ui.ctx(), popup_id);

        paint_open_with_split(
            ui,
            group_rect,
            left_rect,
            right_rect,
            current_label,
            current_icon,
            props.icon_size,
            &style,
            left_response.is_pointer_button_down_on(),
            left_response.hovered(),
            right_response.is_pointer_button_down_on() || right_response.clicked() || popup_open,
            right_response.hovered(),
        );

        let menu_entries = selected_open_with_entries(props.entries, current_action_id);
        let mut next_action = None;
        if !props.entries.is_empty() {
            let _ = Popup::menu(&right_response)
                .id(popup_id)
                .align(RectAlign::BOTTOM_END)
                .show(|ui| {
                    show_menu_entries_surface(
                        ui,
                        menu_entries.as_slice(),
                        &mut next_action,
                        props.width,
                    );
                });
        }

        let mut response = right_response
            .union(group_hover)
            .on_hover_cursor(CursorIcon::PointingHand);

        if let Some(action_id) = next_action {
            if *selected_action != Some(action_id) {
                response.mark_changed();
            }
            *selected_action = Some(action_id);
        }

        response
    })
    .inner
}

fn resolve_open_with_style(
    runtime: crate::theme::ThemeRuntime,
    size: ControlSize,
    variant: ButtonVariant,
) -> OpenWithStyle {
    let (padding_x, height) = match size {
        ControlSize::Sm => (10.0, 28.0),
        ControlSize::Md => (12.0, 34.0),
    };
    let arrow_width = height;
    let corner_radius = tokens::radius_md(runtime);
    let label_font = typography::semibold_font(typography::LABEL_SIZE);

    match variant {
        ButtonVariant::Primary => OpenWithStyle {
            height,
            padding_x,
            arrow_width,
            corner_radius,
            fill_idle: tokens::primary_bg(runtime),
            fill_hover: tokens::primary_hover_bg(runtime),
            fill_active: tokens::primary_active_bg(runtime),
            stroke_idle: Stroke::NONE,
            stroke_hover: Stroke::NONE,
            stroke_active: Stroke::NONE,
            label_idle: tokens::primary_fg(runtime),
            label_hover: tokens::primary_fg(runtime),
            label_active: tokens::primary_fg(runtime),
            icon_idle: tokens::primary_fg(runtime),
            icon_hover: tokens::primary_fg(runtime),
            icon_active: tokens::primary_fg(runtime),
            arrow_idle: tokens::primary_fg(runtime),
            arrow_hover: tokens::primary_fg(runtime),
            arrow_active: tokens::primary_fg(runtime),
            label_font,
        },
        ButtonVariant::Ghost => OpenWithStyle {
            height,
            padding_x,
            arrow_width,
            corner_radius,
            fill_idle: tokens::TRANSPARENT,
            fill_hover: tokens::button_secondary_hover_bg(runtime),
            fill_active: tokens::button_secondary_active_bg(runtime),
            stroke_idle: Stroke::NONE,
            stroke_hover: Stroke::NONE,
            stroke_active: Stroke::NONE,
            label_idle: tokens::text_primary(runtime),
            label_hover: tokens::text_primary(runtime),
            label_active: tokens::text_primary(runtime),
            icon_idle: tokens::text_primary(runtime),
            icon_hover: tokens::text_primary(runtime),
            icon_active: tokens::text_primary(runtime),
            arrow_idle: tokens::text_primary(runtime),
            arrow_hover: tokens::text_primary(runtime),
            arrow_active: tokens::text_primary(runtime),
            label_font,
        },
        ButtonVariant::Link => OpenWithStyle {
            height,
            padding_x,
            arrow_width,
            corner_radius,
            fill_idle: tokens::TRANSPARENT,
            fill_hover: tokens::TRANSPARENT,
            fill_active: tokens::TRANSPARENT,
            stroke_idle: Stroke::NONE,
            stroke_hover: Stroke::NONE,
            stroke_active: Stroke::NONE,
            label_idle: tokens::text_secondary(runtime),
            label_hover: tokens::text_primary(runtime),
            label_active: tokens::text_primary(runtime),
            icon_idle: tokens::text_secondary(runtime),
            icon_hover: tokens::text_secondary(runtime),
            icon_active: tokens::text_secondary(runtime),
            arrow_idle: tokens::text_secondary(runtime),
            arrow_hover: tokens::text_primary(runtime),
            arrow_active: tokens::text_primary(runtime),
            label_font,
        },
        ButtonVariant::Secondary => OpenWithStyle {
            height,
            padding_x,
            arrow_width,
            corner_radius,
            fill_idle: tokens::button_secondary_bg(runtime),
            fill_hover: tokens::button_secondary_hover_bg(runtime),
            fill_active: tokens::button_secondary_active_bg(runtime),
            stroke_idle: Stroke::new(1.0, tokens::button_secondary_border(runtime)),
            stroke_hover: Stroke::new(1.0, tokens::button_secondary_hover_border(runtime)),
            stroke_active: Stroke::new(1.0, tokens::button_secondary_active_border(runtime)),
            label_idle: tokens::text_primary(runtime),
            label_hover: tokens::text_primary(runtime),
            label_active: tokens::text_primary(runtime),
            icon_idle: tokens::text_primary(runtime),
            icon_hover: tokens::text_primary(runtime),
            icon_active: tokens::text_primary(runtime),
            arrow_idle: tokens::text_secondary(runtime),
            arrow_hover: tokens::text_primary(runtime),
            arrow_active: tokens::text_primary(runtime),
            label_font,
        },
    }
}

fn resolve_open_with_metrics(
    ui: &mut Ui,
    current_label: &str,
    current_icon: Option<&str>,
    icon_size: f32,
    style: &OpenWithStyle,
) -> OpenWithMetrics {
    let label_width = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(
                current_label.to_owned(),
                style.label_font.clone(),
                Color32::WHITE,
            )
            .size()
            .x
    });
    let left_width = style.padding_x * 2.0
        + label_width
        + if current_icon.is_some() {
            icon_size + OPEN_WITH_ICON_GAP
        } else {
            0.0
        };

    let left_width = left_width.max(style.height);

    OpenWithMetrics {
        left_width,
        arrow_width: style.arrow_width,
        total_width: left_width + style.arrow_width,
    }
}

fn paint_open_with_split(
    ui: &mut Ui,
    group_rect: Rect,
    left_rect: Rect,
    right_rect: Rect,
    current_label: &str,
    current_icon: Option<&str>,
    icon_size: f32,
    style: &OpenWithStyle,
    left_pressed: bool,
    left_hovered: bool,
    right_active: bool,
    right_hovered: bool,
) {
    let left_state = if left_pressed {
        SegmentState::Active
    } else if left_hovered {
        SegmentState::Hovered
    } else {
        SegmentState::Idle
    };
    let right_state = if right_active {
        SegmentState::Active
    } else if right_hovered {
        SegmentState::Hovered
    } else {
        SegmentState::Idle
    };
    let group_state = match (left_state, right_state) {
        (SegmentState::Active, _) | (_, SegmentState::Active) => SegmentState::Active,
        (SegmentState::Hovered, _) | (_, SegmentState::Hovered) => SegmentState::Hovered,
        _ => SegmentState::Idle,
    };

    let painter = ui.painter();
    painter.rect(
        group_rect,
        CornerRadius::same(style.corner_radius),
        style.fill_idle,
        style.stroke(group_state),
        StrokeKind::Outside,
    );

    let left_corner = CornerRadius {
        nw: style.corner_radius,
        ne: 0,
        sw: style.corner_radius,
        se: 0,
    };
    let right_corner = CornerRadius {
        nw: 0,
        ne: style.corner_radius,
        sw: 0,
        se: style.corner_radius,
    };

    painter.rect(
        left_rect,
        left_corner,
        style.fill(left_state),
        Stroke::NONE,
        StrokeKind::Outside,
    );
    painter.rect(
        right_rect,
        right_corner,
        style.fill(right_state),
        Stroke::NONE,
        StrokeKind::Outside,
    );

    let label_x = left_rect.left()
        + style.padding_x
        + current_icon.map_or(0.0, |_| icon_size + OPEN_WITH_ICON_GAP);
    if let Some(icon_name) = current_icon {
        if let Some(image) = icons::image(ui.ctx(), icon_name, icon_size) {
            let icon_rect = Rect::from_center_size(
                Pos2::new(
                    left_rect.left() + style.padding_x + icon_size * 0.5,
                    left_rect.center().y,
                ),
                vec2(icon_size, icon_size),
            );
            image
                .tint(style.icon_tint(left_state))
                .paint_at(ui, icon_rect);
        }
    }

    painter.text(
        Pos2::new(label_x, left_rect.center().y),
        Align2::LEFT_CENTER,
        current_label,
        style.label_font.clone(),
        style.label_color(left_state),
    );

    if let Some(image) = icons::image(ui.ctx(), "chevron-down", 12.0) {
        let arrow_rect = Rect::from_center_size(right_rect.center(), vec2(12.0, 12.0));
        image
            .tint(style.arrow_tint(right_state))
            .paint_at(ui, arrow_rect);
    }

    let divider_x = left_rect.right();
    let divider_stroke = style.stroke(group_state);
    if divider_stroke != Stroke::NONE {
        painter.line_segment(
            [
                Pos2::new(divider_x, group_rect.top() + 1.0),
                Pos2::new(divider_x, group_rect.bottom() - 1.0),
            ],
            divider_stroke,
        );
    }
}

fn resolve_open_with_action<'a>(
    entries: &'a [DropdownMenuEntry<'a>],
    selected_action: Option<usize>,
) -> Option<DropdownMenuAction<'a>> {
    if let Some(selected_action) = selected_action {
        for entry in entries {
            if let DropdownMenuEntry::Action(action) = entry {
                if action.id == selected_action {
                    return Some(*action);
                }
            }
        }
    }

    first_open_with_action(entries)
}

fn first_open_with_action<'a>(
    entries: &'a [DropdownMenuEntry<'a>],
) -> Option<DropdownMenuAction<'a>> {
    entries.iter().find_map(|entry| match entry {
        DropdownMenuEntry::Action(action) => Some(*action),
        _ => None,
    })
}

fn selected_open_with_entries<'a>(
    entries: &'a [DropdownMenuEntry<'a>],
    selected_action_id: Option<usize>,
) -> Vec<DropdownMenuEntry<'a>> {
    entries
        .iter()
        .copied()
        .map(|entry| match entry {
            DropdownMenuEntry::Action(action) => DropdownMenuEntry::Action(
                action.selected(selected_action_id.is_some_and(|id| id == action.id)),
            ),
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        first_open_with_action, resolve_open_with_action, resolve_open_with_metrics,
        resolve_open_with_style, ButtonVariant, ControlSize, OpenWith,
    };
    use crate::components::{ComponentUiExt, DropdownMenuEntry};
    use crate::theme::{self, ThemeMode, ThemeSpec};
    use egui::{
        pos2, CentralPanel, Context, Event, Id, Modifiers, PointerButton, Popup, Pos2, RawInput,
    };

    const TEST_ENTRIES: [DropdownMenuEntry<'static>; 1] =
        [DropdownMenuEntry::action_with_icon(0, "Codex", "codex")];

    #[test]
    fn open_with_renders_without_panic() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let mut selected_action = None;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let response = ui.components().open_with(
                    &mut selected_action,
                    OpenWith::new(Id::new("open_with_test"), &TEST_ENTRIES),
                );

                assert!(response.rect.width() > 0.0);
                assert!(response.rect.height() > 0.0);
            });
        });

        assert_eq!(selected_action, None);
    }

    #[test]
    fn left_segment_does_not_open_the_dropdown() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let mut selected_action = None;
        let popup_id = Id::new("open_with_test").with("popup");

        let (_, left_center, _) =
            render_open_with(&context, RawInput::default(), &mut selected_action);

        let _ = render_open_with(
            &context,
            pointer_input(left_center, true),
            &mut selected_action,
        );
        let _ = render_open_with(
            &context,
            pointer_input(left_center, false),
            &mut selected_action,
        );

        assert!(!Popup::is_id_open(&context, popup_id));
        assert_eq!(selected_action, None);
    }

    #[test]
    fn right_segment_opens_the_dropdown() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let mut selected_action = None;
        let popup_id = Id::new("open_with_test").with("popup");

        let (_, _, right_center) =
            render_open_with(&context, RawInput::default(), &mut selected_action);

        let _ = render_open_with(
            &context,
            pointer_input(right_center, true),
            &mut selected_action,
        );
        let _ = render_open_with(
            &context,
            pointer_input(right_center, false),
            &mut selected_action,
        );

        assert!(Popup::is_id_open(&context, popup_id));
    }

    #[test]
    fn caret_segment_matches_control_height() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                for size in [ControlSize::Sm, ControlSize::Md] {
                    let style = resolve_open_with_style(
                        theme::runtime_for_ui(ui),
                        size,
                        ButtonVariant::Secondary,
                    );
                    assert_eq!(style.arrow_width, style.height);
                }
            });
        });
    }

    fn render_open_with(
        context: &Context,
        input: RawInput,
        selected_action: &mut Option<usize>,
    ) -> (egui::Rect, Pos2, Pos2) {
        let mut response_rect = egui::Rect::NOTHING;
        let mut left_center = Pos2::ZERO;
        let mut right_center = Pos2::ZERO;

        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                let props = OpenWith::new(Id::new("open_with_test"), &TEST_ENTRIES);
                let current_action = resolve_open_with_action(props.entries, *selected_action)
                    .or_else(|| first_open_with_action(props.entries));
                let style = resolve_open_with_style(
                    theme::runtime_for_ui(ui),
                    props.size,
                    props.trigger_variant,
                );
                let metrics = resolve_open_with_metrics(
                    ui,
                    current_action
                        .map(|action| action.label)
                        .unwrap_or(props.placeholder),
                    current_action.and_then(|action| action.icon),
                    props.icon_size,
                    &style,
                );
                let response = ui.components().open_with(selected_action, props);
                response_rect = response.rect;
                left_center = pos2(
                    response_rect.left() + metrics.left_width * 0.5,
                    response_rect.center().y,
                );
                right_center = pos2(
                    response_rect.left() + metrics.left_width + metrics.arrow_width * 0.5,
                    response_rect.center().y,
                );
            });
        });

        (response_rect, left_center, right_center)
    }

    fn pointer_input(position: Pos2, pressed: bool) -> RawInput {
        RawInput {
            events: vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..RawInput::default()
        }
    }
}
