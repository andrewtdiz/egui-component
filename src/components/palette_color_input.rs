use super::{
    api::{ComponentUi, ComponentUiExt},
    Button, ButtonVariant, Color, ColorInput, Label, LabelTone, LabelWeight, Popover, PopoverAlign,
    PopoverSide,
};
use crate::theme::{self, ColorRole};
use egui::{vec2, Color32, Id, Response, Stroke, Ui};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
struct PaletteColorInputState {
    open: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct PaletteColorInput<'a> {
    pub id: Id,
    pub palette: &'a [Color32],
    pub palette_columns: usize,
    pub trigger_size: f32,
    pub swatch_size: f32,
    pub popover_width: f32,
    pub allow_custom: bool,
}

impl<'a> PaletteColorInput<'a> {
    pub fn new(id: Id, palette: &'a [Color32]) -> Self {
        Self {
            id,
            palette,
            palette_columns: 8,
            trigger_size: 20.0,
            swatch_size: 28.0,
            popover_width: 280.0,
            allow_custom: true,
        }
    }

    pub fn palette_columns(mut self, palette_columns: usize) -> Self {
        self.palette_columns = palette_columns.max(1);
        self
    }

    pub fn trigger_size(mut self, trigger_size: f32) -> Self {
        self.trigger_size = trigger_size.max(1.0);
        self
    }

    pub fn swatch_size(mut self, swatch_size: f32) -> Self {
        self.swatch_size = swatch_size.max(1.0);
        self
    }

    pub fn popover_width(mut self, popover_width: f32) -> Self {
        self.popover_width = popover_width.max(1.0);
        self
    }

    pub fn allow_custom(mut self, allow_custom: bool) -> Self {
        self.allow_custom = allow_custom;
        self
    }
}

impl ComponentUi<'_> {
    pub fn palette_color_input<'a>(
        &mut self,
        value: &mut Color32,
        props: impl Into<PaletteColorInput<'a>>,
    ) -> Response {
        draw_palette_color_input(self, value, props.into())
    }
}

fn draw_palette_color_input(
    ui: &mut ComponentUi<'_>,
    value: &mut Color32,
    props: PaletteColorInput<'_>,
) -> Response {
    let mut state = load_palette_color_input_state(ui.ui(), props.id);
    let mut open = state.open;
    let mut swatch_changed = false;
    let mut custom_changed = false;
    let trigger_value = *value;
    let trigger_open = open;
    let popover = ui.popover(
        &mut open,
        Popover::new(props.id.with("popover"))
            .width(props.popover_width)
            .align(PopoverAlign::End)
            .side(PopoverSide::Bottom)
            .side_offset(4.0),
        |ui| draw_palette_color_trigger(ui, props, trigger_value, trigger_open),
        |ui, open| {
            draw_palette_color_panel(
                ui,
                props,
                value,
                open,
                &mut swatch_changed,
                &mut custom_changed,
            )
        },
    );
    state.open = open;
    store_palette_color_input_state(ui.ui_mut(), props.id, state);

    let mut response = popover.trigger;
    if swatch_changed || custom_changed {
        response.mark_changed();
    }
    response
}

fn draw_palette_color_trigger(
    ui: &mut Ui,
    props: PaletteColorInput<'_>,
    value: Color32,
    open: bool,
) -> Response {
    let stroke = if open {
        Stroke::new(2.0, theme::color(ui, ColorRole::Ring))
    } else {
        Stroke::new(1.0, theme::color(ui, ColorRole::Border))
    };
    ui.components().button(
        Button::color_only(Color::new(value).size(props.trigger_size).stroke(stroke))
            .variant(ButtonVariant::Secondary)
            .selected(open)
            .min_size(vec2(props.trigger_size + 14.0, props.trigger_size + 14.0)),
    )
}

fn draw_palette_color_panel(
    ui: &mut Ui,
    props: PaletteColorInput<'_>,
    value: &mut Color32,
    open: &mut bool,
    swatch_changed: &mut bool,
    custom_changed: &mut bool,
) {
    let _ = ui.components().label(
        Label::new("Default solid colors")
            .weight(LabelWeight::Semibold)
            .tone(LabelTone::Primary),
    );
    ui.add_space(8.0);

    egui::Grid::new(props.id.with("grid"))
        .num_columns(props.palette_columns)
        .spacing(vec2(8.0, 8.0))
        .show(ui, |ui| {
            for (index, swatch) in props.palette.iter().copied().enumerate() {
                let selected = *value == swatch;
                let stroke = if selected {
                    Stroke::new(2.0, theme::color(ui, ColorRole::Ring))
                } else {
                    Stroke::new(1.0, theme::color(ui, ColorRole::Border))
                };
                let response = ui.components().button(
                    Button::color_only(Color::new(swatch).size(20.0).stroke(stroke))
                        .variant(ButtonVariant::Ghost)
                        .selected(selected)
                        .min_size(vec2(props.swatch_size, props.swatch_size)),
                );
                if response.clicked() {
                    apply_palette_swatch_selection(value, swatch, open, swatch_changed);
                }
                if (index + 1) % props.palette_columns == 0 {
                    ui.end_row();
                }
            }
        });

    if props.allow_custom {
        ui.add_space(10.0);
        let _ = ui.components().label(
            Label::new("Custom")
                .tone(LabelTone::Muted)
                .size(11.0)
                .weight(LabelWeight::Semibold),
        );
        ui.add_space(6.0);
        let response = ui
            .components()
            .color_input(value, ColorInput::new().id(props.id.with("custom")));
        if response.changed() {
            *custom_changed = true;
        }
    }
}

fn load_palette_color_input_state(ui: &Ui, id: Id) -> PaletteColorInputState {
    ui.data(|data| {
        data.get_temp::<PaletteColorInputState>(id.with("palette_color_input_state"))
            .unwrap_or_default()
    })
}

fn store_palette_color_input_state(ui: &mut Ui, id: Id, state: PaletteColorInputState) {
    ui.data_mut(|data| {
        if state == PaletteColorInputState::default() {
            data.remove::<PaletteColorInputState>(id.with("palette_color_input_state"));
        } else {
            data.insert_temp(id.with("palette_color_input_state"), state);
        }
    });
}

fn apply_palette_swatch_selection(
    value: &mut Color32,
    swatch: Color32,
    open: &mut bool,
    swatch_changed: &mut bool,
) {
    *value = swatch;
    *swatch_changed = true;
    *open = false;
}

#[cfg(test)]
mod tests {
    use super::{
        apply_palette_swatch_selection, store_palette_color_input_state, PaletteColorInput,
        PaletteColorInputState,
    };
    use crate::components::ComponentUiExt;
    use crate::theme::{self, ThemeMode};
    use egui::{
        pos2, vec2, CentralPanel, Context, Event, Id, Modifiers, PointerButton, Pos2, RawInput,
        Rect, Response,
    };

    #[derive(Debug, Default)]
    struct PaletteColorPanelDiagnostics {
        selected_swatch_index: Option<usize>,
        custom_rect: Option<Rect>,
    }

    struct PaletteColorInputFrame {
        response: Response,
        open: bool,
        popover_rect: Option<Rect>,
        selected_swatch_index: Option<usize>,
        custom_rect: Option<Rect>,
    }

    const TEST_ID: &str = "palette_color_input_test";
    const TEST_PALETTE: [egui::Color32; 4] = [
        egui::Color32::from_rgb(190, 74, 47),
        egui::Color32::from_rgb(215, 118, 67),
        egui::Color32::from_rgb(234, 212, 170),
        egui::Color32::from_rgb(228, 166, 114),
    ];

    #[test]
    fn trigger_click_opens_palette_popover() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let mut value = TEST_PALETTE[0];
        let frame =
            render_test_palette_color_input(&context, RawInput::default(), None, &mut value);

        let _ = render_test_palette_color_input(
            &context,
            pointer_input(frame.response.rect.center(), true),
            None,
            &mut value,
        );
        let frame = render_test_palette_color_input(
            &context,
            pointer_input(frame.response.rect.center(), false),
            None,
            &mut value,
        );

        assert!(frame.open);
        assert!(frame.popover_rect.is_some());
    }

    #[test]
    fn selecting_palette_swatch_updates_color_and_closes_popover() {
        let mut value = TEST_PALETTE[0];
        let mut open = true;
        let mut changed = false;

        apply_palette_swatch_selection(&mut value, TEST_PALETTE[1], &mut open, &mut changed);

        assert_eq!(value, TEST_PALETTE[1]);
        assert!(changed);
        assert!(!open);
    }

    #[test]
    fn non_palette_current_colors_render_with_no_selected_swatch() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let mut value = egui::Color32::from_rgb(12, 34, 56);
        let frame =
            render_test_palette_color_input(&context, RawInput::default(), Some(true), &mut value);

        assert_eq!(frame.selected_swatch_index, None);
    }

    #[test]
    fn custom_fallback_renders_inside_popover() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let mut value = TEST_PALETTE[0];
        let frame =
            render_test_palette_color_input(&context, RawInput::default(), Some(true), &mut value);

        assert!(frame.custom_rect.is_some());
    }

    fn render_test_palette_color_input(
        context: &Context,
        input: RawInput,
        seed_open: Option<bool>,
        value: &mut egui::Color32,
    ) -> PaletteColorInputFrame {
        let mut current = *value;
        let mut response = None;
        let mut open = false;
        let mut popover_rect = None;
        let mut diagnostics = PaletteColorPanelDiagnostics::default();

        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                ui.set_width(320.0);
                if let Some(open) = seed_open {
                    store_palette_color_input_state(
                        ui,
                        Id::new(TEST_ID),
                        PaletteColorInputState { open },
                    );
                }
                let mut trigger_response = None;
                let mut next_open =
                    super::load_palette_color_input_state(ui, Id::new(TEST_ID)).open;
                let mut swatch_changed = false;
                let mut custom_changed = false;
                let trigger_value = current;
                let trigger_open = next_open;
                let popover = ui.components().popover(
                    &mut next_open,
                    super::Popover::new(Id::new(TEST_ID).with("popover"))
                        .width(220.0)
                        .align(super::PopoverAlign::End)
                        .side(super::PopoverSide::Bottom)
                        .side_offset(4.0),
                    |ui| {
                        let response = super::draw_palette_color_trigger(
                            ui,
                            PaletteColorInput::new(Id::new(TEST_ID), &TEST_PALETTE)
                                .palette_columns(2),
                            trigger_value,
                            trigger_open,
                        );
                        trigger_response = Some(response.clone());
                        response
                    },
                    |ui, open| {
                        diagnostics = draw_test_palette_color_panel(
                            ui,
                            PaletteColorInput::new(Id::new(TEST_ID), &TEST_PALETTE)
                                .palette_columns(2)
                                .popover_width(220.0),
                            &mut current,
                            open,
                            &mut swatch_changed,
                            &mut custom_changed,
                        );
                    },
                );
                super::store_palette_color_input_state(
                    ui,
                    Id::new(TEST_ID),
                    PaletteColorInputState { open: next_open },
                );
                open = next_open;
                popover_rect = popover
                    .content
                    .as_ref()
                    .map(|content| content.response.rect);
                response = Some(trigger_response.expect("palette trigger should render"));
            });
        });

        *value = current;
        PaletteColorInputFrame {
            response: response.expect("palette color input should render"),
            open,
            popover_rect,
            selected_swatch_index: diagnostics.selected_swatch_index,
            custom_rect: diagnostics.custom_rect,
        }
    }

    fn draw_test_palette_color_panel(
        ui: &mut egui::Ui,
        props: PaletteColorInput<'_>,
        value: &mut egui::Color32,
        open: &mut bool,
        swatch_changed: &mut bool,
        custom_changed: &mut bool,
    ) -> PaletteColorPanelDiagnostics {
        let mut diagnostics = PaletteColorPanelDiagnostics::default();

        let _ = ui.components().label(
            super::Label::new("Default solid colors")
                .weight(super::LabelWeight::Semibold)
                .tone(super::LabelTone::Primary),
        );
        ui.add_space(8.0);

        egui::Grid::new(props.id.with("grid"))
            .num_columns(props.palette_columns)
            .spacing(vec2(8.0, 8.0))
            .show(ui, |ui| {
                for (index, swatch) in props.palette.iter().copied().enumerate() {
                    let selected = *value == swatch;
                    if selected {
                        diagnostics.selected_swatch_index = Some(index);
                    }
                    let stroke = if selected {
                        super::Stroke::new(2.0, super::theme::color(ui, super::ColorRole::Ring))
                    } else {
                        super::Stroke::new(1.0, super::theme::color(ui, super::ColorRole::Border))
                    };
                    let response = ui.components().button(
                        super::Button::color_only(
                            super::Color::new(swatch).size(20.0).stroke(stroke),
                        )
                        .variant(super::ButtonVariant::Ghost)
                        .selected(selected)
                        .min_size(vec2(props.swatch_size, props.swatch_size)),
                    );
                    if response.clicked() {
                        super::apply_palette_swatch_selection(value, swatch, open, swatch_changed);
                    }
                    if (index + 1) % props.palette_columns == 0 {
                        ui.end_row();
                    }
                }
            });

        if props.allow_custom {
            ui.add_space(10.0);
            let _ = ui.components().label(
                super::Label::new("Custom")
                    .tone(super::LabelTone::Muted)
                    .size(11.0)
                    .weight(super::LabelWeight::Semibold),
            );
            ui.add_space(6.0);
            let response = ui
                .components()
                .color_input(value, super::ColorInput::new().id(props.id.with("custom")));
            diagnostics.custom_rect = Some(response.rect);
            if response.changed() {
                *custom_changed = true;
            }
        }

        diagnostics
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
}
