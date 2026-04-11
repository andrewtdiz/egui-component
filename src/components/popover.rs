use super::api::{with_component_overrides, ComponentUi};
use crate::ui::tokens;
use egui::{
    Align, Id, InnerResponse, Layout, Margin, Popup, PopupCloseBehavior, PopupKind, RectAlign,
    Response, Stroke, Ui,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PopoverSide {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PopoverAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy)]
pub struct PopoverPanel {
    pub id: Id,
    pub side: PopoverSide,
    pub align: PopoverAlign,
    pub side_offset: f32,
    pub width: Option<f32>,
    pub padding_x: i8,
    pub padding_y: i8,
}

impl PopoverPanel {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            side: PopoverSide::Bottom,
            align: PopoverAlign::Center,
            side_offset: 8.0,
            width: None,
            padding_x: 12,
            padding_y: 12,
        }
    }

    pub fn side(mut self, side: PopoverSide) -> Self {
        self.side = side;
        self
    }

    pub fn align(mut self, align: PopoverAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side_offset(mut self, side_offset: f32) -> Self {
        self.side_offset = side_offset.max(0.0);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(1.0));
        self
    }

    pub fn padding(mut self, x: i8, y: i8) -> Self {
        self.padding_x = x;
        self.padding_y = y;
        self
    }
}

impl From<Id> for PopoverPanel {
    fn from(id: Id) -> Self {
        Self::new(id)
    }
}

pub type Popover = PopoverPanel;

pub struct PopoverResponse<R> {
    pub trigger: Response,
    pub content: Option<InnerResponse<R>>,
}

impl ComponentUi<'_> {
    pub fn popover<R>(
        &mut self,
        open: &mut bool,
        props: impl Into<Popover>,
        trigger: impl FnOnce(&mut Ui) -> Response,
        content: impl FnOnce(&mut Ui, &mut bool) -> R,
    ) -> PopoverResponse<R> {
        let props = props.into();
        let overrides = self.overrides();
        let runtime = crate::theme::runtime_for_ui(self);
        let trigger_response = with_component_overrides(self.ui_mut(), overrides, |ui| trigger(ui));

        if trigger_response.clicked() {
            *open = !*open;
        }

        let mut popup_open = *open;
        let mut content_open = *open;
        let mut popup = Popup::from_response(&trigger_response)
            .id(props.id)
            .kind(PopupKind::Popup)
            .open_bool(&mut popup_open)
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .align(popover_rect_align(props.side, props.align))
            .gap(props.side_offset)
            .layout(Layout::top_down(Align::Min))
            .frame(popover_frame(self.style(), runtime, props));

        if let Some(width) = props.width {
            popup = popup.width(width);
        }

        let content_response = popup.show(|ui| {
            with_component_overrides(ui, overrides, |ui| content(ui, &mut content_open))
        });

        *open = popup_open && content_open;

        PopoverResponse {
            trigger: trigger_response,
            content: content_response,
        }
    }
}

fn popover_frame(
    style: &egui::Style,
    runtime: crate::theme::ThemeRuntime,
    props: Popover,
) -> egui::Frame {
    egui::Frame::popup(style)
        .fill(tokens::popover_background(runtime))
        .stroke(Stroke::new(1.0, tokens::separator(runtime)))
        .corner_radius(tokens::radius_lg(runtime))
        .inner_margin(Margin::symmetric(props.padding_x, props.padding_y))
        .shadow(tokens::tailwind_shadow_md(runtime))
}

fn popover_rect_align(side: PopoverSide, align: PopoverAlign) -> RectAlign {
    match (side, align) {
        (PopoverSide::Top, PopoverAlign::Start) => RectAlign::TOP_START,
        (PopoverSide::Top, PopoverAlign::Center) => RectAlign::TOP,
        (PopoverSide::Top, PopoverAlign::End) => RectAlign::TOP_END,
        (PopoverSide::Right, PopoverAlign::Start) => RectAlign::RIGHT_START,
        (PopoverSide::Right, PopoverAlign::Center) => RectAlign::RIGHT,
        (PopoverSide::Right, PopoverAlign::End) => RectAlign::RIGHT_END,
        (PopoverSide::Bottom, PopoverAlign::Start) => RectAlign::BOTTOM_START,
        (PopoverSide::Bottom, PopoverAlign::Center) => RectAlign::BOTTOM,
        (PopoverSide::Bottom, PopoverAlign::End) => RectAlign::BOTTOM_END,
        (PopoverSide::Left, PopoverAlign::Start) => RectAlign::LEFT_START,
        (PopoverSide::Left, PopoverAlign::Center) => RectAlign::LEFT,
        (PopoverSide::Left, PopoverAlign::End) => RectAlign::LEFT_END,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        popover_frame, popover_rect_align, Popover, PopoverAlign, PopoverResponse, PopoverSide,
    };
    use crate::components::{Button, ButtonVariant, ComponentUiExt, Label, LabelTone};
    use crate::theme::{self, ColorRole, ThemeMode};
    use crate::ui::tokens;
    use egui::{
        pos2, vec2, CentralPanel, Context, CornerRadius, Event, Id, Key, Margin, Modifiers,
        PointerButton, Pos2, RawInput, Rect,
    };

    #[test]
    fn placement_mapping_matches_expected_rect_aligns() {
        assert_eq!(
            popover_rect_align(PopoverSide::Top, PopoverAlign::Start),
            egui::RectAlign::TOP_START
        );
        assert_eq!(
            popover_rect_align(PopoverSide::Top, PopoverAlign::Center),
            egui::RectAlign::TOP
        );
        assert_eq!(
            popover_rect_align(PopoverSide::Top, PopoverAlign::End),
            egui::RectAlign::TOP_END
        );
        assert_eq!(
            popover_rect_align(PopoverSide::Right, PopoverAlign::Start),
            egui::RectAlign::RIGHT_START
        );
        assert_eq!(
            popover_rect_align(PopoverSide::Right, PopoverAlign::Center),
            egui::RectAlign::RIGHT
        );
        assert_eq!(
            popover_rect_align(PopoverSide::Right, PopoverAlign::End),
            egui::RectAlign::RIGHT_END
        );
        assert_eq!(
            popover_rect_align(PopoverSide::Bottom, PopoverAlign::Start),
            egui::RectAlign::BOTTOM_START
        );
        assert_eq!(
            popover_rect_align(PopoverSide::Bottom, PopoverAlign::Center),
            egui::RectAlign::BOTTOM
        );
        assert_eq!(
            popover_rect_align(PopoverSide::Bottom, PopoverAlign::End),
            egui::RectAlign::BOTTOM_END
        );
        assert_eq!(
            popover_rect_align(PopoverSide::Left, PopoverAlign::Start),
            egui::RectAlign::LEFT_START
        );
        assert_eq!(
            popover_rect_align(PopoverSide::Left, PopoverAlign::Center),
            egui::RectAlign::LEFT
        );
        assert_eq!(
            popover_rect_align(PopoverSide::Left, PopoverAlign::End),
            egui::RectAlign::LEFT_END
        );
    }

    #[test]
    fn popover_frame_uses_popover_fill_padding_and_shadow_defaults() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let style = context.style();
        let runtime = theme::runtime_for_context(&context);
        let props = Popover::new(Id::new("popover_frame_test"));
        let frame = popover_frame(&style, runtime, props);

        assert_eq!(
            frame.fill,
            theme::resolved_color(runtime, ColorRole::Popover)
        );
        assert_eq!(
            frame.stroke,
            egui::Stroke::new(1.0, tokens::separator(runtime))
        );
        assert_eq!(frame.shadow, tokens::tailwind_shadow_md(runtime));
        assert_eq!(frame.inner_margin, Margin::symmetric(12, 12));
        assert_eq!(
            frame.corner_radius,
            CornerRadius::same(tokens::radius_lg(runtime))
        );
    }

    #[test]
    fn respects_controlled_open_state() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let (_, _, response) = render_popover(&context, RawInput::default(), false, false);
        assert!(response.content.is_none());

        let (_, _, response) = render_popover(&context, RawInput::default(), true, false);
        assert!(response.content.is_some());
    }

    #[test]
    fn trigger_click_toggles_popover_open() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let (trigger_center, _, _) = render_popover(&context, RawInput::default(), false, false);

        let _ = render_popover(&context, pointer_input(trigger_center, true), false, false);
        let (_, open, response) =
            render_popover(&context, pointer_input(trigger_center, false), false, false);

        assert!(open);
        assert!(response.content.is_some());
    }

    #[test]
    fn outside_click_closes_popover() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let mut open = true;
        let (_, next_open, response) = render_popover(&context, RawInput::default(), open, false);
        open = next_open;
        let outside = response
            .content
            .as_ref()
            .map(|_| pos2(520.0, 240.0))
            .expect("popover content should be visible");

        let (_, next_open, _) = render_popover(&context, pointer_input(outside, true), open, false);
        open = next_open;
        let (_, next_open, _) =
            render_popover(&context, pointer_input(outside, false), open, false);
        open = next_open;
        let (_, final_open, response) = render_popover(&context, RawInput::default(), open, false);

        assert!(!final_open);
        assert!(response.content.is_none());
    }

    #[test]
    fn escape_closes_popover() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let mut open = true;
        let (_, next_open, _) = render_popover(&context, RawInput::default(), open, false);
        open = next_open;
        let (_, next_open, _) = render_popover(&context, escape_input(), open, false);
        open = next_open;
        let (_, final_open, response) = render_popover(&context, RawInput::default(), open, false);

        assert!(!final_open);
        assert!(response.content.is_none());
    }

    #[test]
    fn clicking_inside_content_keeps_popover_open() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let (_, _, response) = render_popover(&context, RawInput::default(), true, false);
        let inside = response
            .content
            .as_ref()
            .map(|content| content.response.rect.center())
            .expect("popover content should be visible");

        let _ = render_popover(&context, pointer_input(inside, true), true, false);
        let (_, open, response) =
            render_popover(&context, pointer_input(inside, false), true, false);

        assert!(open);
        assert!(response.content.is_some());
    }

    #[test]
    fn content_can_request_close() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let (_, open, response) = render_popover(&context, RawInput::default(), true, true);

        assert!(!open);
        assert!(response.content.is_some());
    }

    #[test]
    fn renders_typed_components_inside_popover_content() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let (_, _, response) = render_popover(&context, RawInput::default(), true, false);
        let content = response.content.expect("popover content should render");

        assert!(content.response.rect.width() > 0.0);
        assert!(content.response.rect.height() > 0.0);
    }

    fn render_popover(
        context: &Context,
        input: RawInput,
        initial_open: bool,
        close_from_content: bool,
    ) -> (Pos2, bool, PopoverResponse<()>) {
        let mut open = initial_open;
        let mut trigger_center = Pos2::ZERO;
        let mut response = None;

        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                ui.set_width(320.0);
                let popover_response = ui.components().popover(
                    &mut open,
                    Popover::new(Id::new("popover_test")).width(220.0),
                    |ui| {
                        let response = ui
                            .components()
                            .button(Button::new("Open Popover").variant(ButtonVariant::Secondary));
                        trigger_center = response.rect.center();
                        response
                    },
                    |ui, open| {
                        let _ = ui
                            .components()
                            .label(Label::new("Popover content").tone(LabelTone::Secondary));
                        let _ = ui
                            .components()
                            .button(Button::new("Apply").variant(ButtonVariant::Primary));
                        if close_from_content {
                            *open = false;
                        }
                    },
                );
                response = Some(popover_response);
            });
        });

        (
            trigger_center,
            open,
            response.expect("popover response should be captured"),
        )
    }

    fn pointer_input(position: Pos2, pressed: bool) -> RawInput {
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 360.0))),
            events: vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..Default::default()
        }
    }

    fn escape_input() -> RawInput {
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 360.0))),
            events: vec![Event::Key {
                key: Key::Escape,
                physical_key: Some(Key::Escape),
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        }
    }
}
