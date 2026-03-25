use super::{
    api::{with_component_overrides, ComponentOverrides, ComponentUi},
    dropdown_menu::{show_menu_entries_surface, DropdownMenuEntry},
};
use crate::ui::tokens;
use egui::{
    vec2, Align, Color32, CornerRadius, Id, Layout, Popup, Rect, Response, Sense, Stroke,
    StrokeKind, Ui, UiBuilder, Vec2,
};

const CONTEXT_MENU_DEFAULT_WIDTH: f32 = 220.0;
const CONTEXT_MENU_DEFAULT_SIZE: Vec2 = egui::vec2(360.0, 176.0);
const CONTEXT_MENU_PADDING_X: i8 = 16;
const CONTEXT_MENU_PADDING_Y: i8 = 14;

#[derive(Debug, Clone, Copy)]
pub struct ContextMenu<'a> {
    pub id: Id,
    pub entries: &'a [DropdownMenuEntry<'a>],
    pub width: f32,
    pub size: Vec2,
    pub padding_x: i8,
    pub padding_y: i8,
}

impl<'a> ContextMenu<'a> {
    pub fn new(id: Id, entries: &'a [DropdownMenuEntry<'a>]) -> Self {
        Self {
            id,
            entries,
            width: CONTEXT_MENU_DEFAULT_WIDTH,
            size: CONTEXT_MENU_DEFAULT_SIZE,
            padding_x: CONTEXT_MENU_PADDING_X,
            padding_y: CONTEXT_MENU_PADDING_Y,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(1.0);
        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = vec2(size.x.max(1.0), size.y.max(1.0));
        self
    }

    pub fn padding(mut self, x: i8, y: i8) -> Self {
        self.padding_x = x;
        self.padding_y = y;
        self
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ContextMenuState {
    pub action: Option<usize>,
    pub open: bool,
    pub popup_rect: Option<Rect>,
}

impl ComponentUi<'_> {
    pub fn context_menu<'a>(
        &mut self,
        props: impl Into<ContextMenu<'a>>,
        add_region: impl FnOnce(&mut Ui),
    ) -> (Response, ContextMenuState) {
        let overrides = self.overrides();
        draw_context_menu_region(self.ui_mut(), overrides, props.into(), add_region)
    }

    pub fn context_menu_for<'a>(
        &mut self,
        response: &Response,
        props: impl Into<ContextMenu<'a>>,
    ) -> ContextMenuState {
        show_context_menu_for_response(response, props.into())
    }
}

fn draw_context_menu_region(
    ui: &mut Ui,
    overrides: ComponentOverrides,
    props: ContextMenu<'_>,
    add_region: impl FnOnce(&mut Ui),
) -> (Response, ContextMenuState) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let mut state = ContextMenuState::default();

    let response = ui
        .push_id(props.id, |ui| {
            let (rect, response) = ui.allocate_exact_size(props.size, Sense::hover());
            let response = response.interact(Sense::click());

            state = show_context_menu_for_response(&response, props);

            paint_context_menu_region(ui, rect, &response, state.open, runtime);

            let inner_rect =
                rect.shrink2(vec2(f32::from(props.padding_x), f32::from(props.padding_y)));
            let mut child_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(inner_rect)
                    .layout(Layout::top_down(Align::Min)),
            );
            with_component_overrides(&mut child_ui, overrides, add_region);

            response
        })
        .inner;

    (response, state)
}

fn show_context_menu_for_response(response: &Response, props: ContextMenu<'_>) -> ContextMenuState {
    let mut state = ContextMenuState::default();
    if props.entries.is_empty() {
        return state;
    }

    let popup_response = Popup::context_menu(response)
        .id(props.id)
        .show(|ui| show_menu_entries_surface(ui, props.entries, &mut state.action, props.width));

    state.open = popup_response.is_some();
    state.popup_rect = popup_response.as_ref().map(|inner| inner.response.rect);
    state
}

fn paint_context_menu_region(
    ui: &Ui,
    rect: Rect,
    response: &Response,
    menu_open: bool,
    runtime: crate::theme::ThemeRuntime,
) {
    let fill = context_menu_region_fill(response.hovered(), menu_open, runtime);
    let stroke = Stroke::new(
        1.0,
        if menu_open || response.hovered() {
            tokens::button_secondary_hover_border(runtime)
        } else {
            tokens::separator(runtime)
        },
    );
    ui.painter().rect(
        rect,
        CornerRadius::same(tokens::radius_lg(runtime)),
        fill,
        stroke,
        StrokeKind::Outside,
    );
}

fn context_menu_region_fill(
    hovered: bool,
    menu_open: bool,
    runtime: crate::theme::ThemeRuntime,
) -> Color32 {
    if menu_open {
        tokens::button_secondary_hover_bg(runtime)
    } else if hovered {
        tokens::card_background(runtime)
            .lerp_to_gamma(tokens::button_secondary_hover_bg(runtime), 0.42)
    } else {
        tokens::card_background(runtime)
    }
}

#[cfg(test)]
mod tests {
    use super::{context_menu_region_fill, ContextMenu};
    use crate::components::{ComponentUiExt, DropdownMenuEntry, Label, LabelTone};
    use crate::theme::{self, ThemeMode};
    use crate::ui::tokens;
    use egui::{
        pos2, vec2, CentralPanel, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect,
        Sense,
    };

    const TEST_ENTRIES: [DropdownMenuEntry<'static>; 3] = [
        DropdownMenuEntry::action(0, "Rename"),
        DropdownMenuEntry::action(1, "Duplicate"),
        DropdownMenuEntry::action(2, "Delete"),
    ];

    #[test]
    fn region_fill_emphasizes_hover_and_open_state() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let runtime = theme::runtime_for_context(&context);

        let idle = context_menu_region_fill(false, false, runtime);
        let hovered = context_menu_region_fill(true, false, runtime);
        let open = context_menu_region_fill(false, true, runtime);

        assert_eq!(idle, tokens::card_background(runtime));
        assert_ne!(hovered, idle);
        assert_eq!(open, tokens::button_secondary_hover_bg(runtime));
    }

    #[test]
    fn secondary_click_opens_context_menu() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let (center, _) = render_context_menu(&context, RawInput::default());

        let _ = render_context_menu(
            &context,
            pointer_input(center, PointerButton::Secondary, true),
        );
        let (_, state) = render_context_menu(
            &context,
            pointer_input(center, PointerButton::Secondary, false),
        );

        assert!(state.open);
        assert!(state.popup_rect.is_some());
        assert_eq!(state.action, None);
    }

    #[test]
    fn selecting_menu_action_reports_selected_id() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let (center, _) = render_context_menu(&context, RawInput::default());

        let _ = render_context_menu(
            &context,
            pointer_input(center, PointerButton::Secondary, true),
        );
        let _ = render_context_menu(
            &context,
            pointer_input(center, PointerButton::Secondary, false),
        );
        let (_, state) = render_context_menu(&context, RawInput::default());
        let popup_rect = state.popup_rect.expect("context menu should open");
        let action_pos = pos2(popup_rect.left() + 64.0, popup_rect.top() + 24.0);

        let _ = render_context_menu(
            &context,
            pointer_input(action_pos, PointerButton::Primary, true),
        );
        let (_, state) = render_context_menu(
            &context,
            pointer_input(action_pos, PointerButton::Primary, false),
        );

        assert_eq!(state.action, Some(0));
    }

    #[test]
    fn context_menu_can_attach_to_existing_response_without_region_chrome() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let (center, _) = render_attached_context_menu(&context, RawInput::default());

        let _ = render_attached_context_menu(
            &context,
            pointer_input(center, PointerButton::Secondary, true),
        );
        let (_, state) = render_attached_context_menu(
            &context,
            pointer_input(center, PointerButton::Secondary, false),
        );

        assert!(state.open);
        assert!(state.popup_rect.is_some());
    }

    fn render_context_menu(context: &Context, input: RawInput) -> (Pos2, super::ContextMenuState) {
        let mut center = Pos2::ZERO;
        let mut state = None;

        let _ = context.run(input, |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                let (response, menu_state) = ui.components().context_menu(
                    ContextMenu::new(egui::Id::new("context_menu_test"), &TEST_ENTRIES),
                    |ui| {
                        let _ = ui.components().label(
                            Label::new("Right click inside this region").tone(LabelTone::Secondary),
                        );
                    },
                );
                center = response.rect.center();
                state = Some(menu_state);
            });
        });

        (
            center,
            state.expect("context menu state should be captured"),
        )
    }

    fn render_attached_context_menu(
        context: &Context,
        input: RawInput,
    ) -> (Pos2, super::ContextMenuState) {
        let mut center = Pos2::ZERO;
        let mut state = None;

        let _ = context.run(input, |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                let (rect, response) = ui.allocate_exact_size(vec2(240.0, 96.0), Sense::click());
                ui.painter().rect_stroke(
                    rect,
                    egui::CornerRadius::same(8),
                    egui::Stroke::new(1.0, tokens::separator(theme::runtime_for_ui(ui))),
                    egui::StrokeKind::Outside,
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Custom region",
                    egui::FontId::proportional(15.0),
                    tokens::text_secondary(theme::runtime_for_ui(ui)),
                );

                let menu_state = ui.components().context_menu_for(
                    &response,
                    ContextMenu::new(egui::Id::new("attached_context_menu_test"), &TEST_ENTRIES),
                );
                center = response.rect.center();
                state = Some(menu_state);
            });
        });

        (
            center,
            state.expect("attached context menu state should be captured"),
        )
    }

    fn pointer_input(position: Pos2, button: PointerButton, pressed: bool) -> RawInput {
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 360.0))),
            events: vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..Default::default()
        }
    }
}
