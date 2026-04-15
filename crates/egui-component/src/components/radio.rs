use super::api::ComponentUi;
use crate::ui::tokens;
use egui::{Align, CursorIcon, Id, Layout, Response, Sense, Stroke, Ui};

const RADIO_CONTROL_SIZE: f32 = 16.0;
const RADIO_DOT_RADIUS: f32 = 4.0;
const RADIO_GROUP_GAP: f32 = 8.0;
const RADIO_LABEL_GAP: f32 = 10.0;
const RADIO_TEXT_GAP: f32 = 2.0;

#[derive(Debug, Clone)]
pub struct Radio<'a> {
    pub id: Option<Id>,
    pub label: Option<&'a str>,
    pub description: Option<&'a str>,
}

impl<'a> Radio<'a> {
    pub fn new() -> Self {
        Self {
            id: None,
            label: None,
            description: None,
        }
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
}

impl<'a> Default for Radio<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<()> for Radio<'a> {
    fn from(_: ()) -> Self {
        Self::new()
    }
}

impl<'a> From<&'a str> for Radio<'a> {
    fn from(label: &'a str) -> Self {
        Self::new().label(label)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RadioOption<'a> {
    pub value: usize,
    pub label: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> RadioOption<'a> {
    pub const fn new(value: usize, label: &'a str) -> Self {
        Self {
            value,
            label,
            description: None,
        }
    }

    pub const fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RadioGroup<'a> {
    pub id: Id,
    pub options: &'a [RadioOption<'a>],
    pub gap: f32,
}

impl<'a> RadioGroup<'a> {
    pub fn new(id: Id, options: &'a [RadioOption<'a>]) -> Self {
        Self {
            id,
            options,
            gap: RADIO_GROUP_GAP,
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }
}

impl ComponentUi<'_> {
    pub fn radio<'a>(&mut self, value: &mut bool, props: impl Into<Radio<'a>>) -> Response {
        let mut response = draw_radio(self.ui_mut(), *value, props.into());
        if response.clicked() && !*value {
            *value = true;
            response.mark_changed();
        }
        response
    }

    pub fn radio_value<'a, T>(
        &mut self,
        current: &mut T,
        value: T,
        props: impl Into<Radio<'a>>,
    ) -> Response
    where
        T: Copy + PartialEq,
    {
        let mut response = draw_radio(self.ui_mut(), *current == value, props.into());
        if response.clicked() && *current != value {
            *current = value;
            response.mark_changed();
        }
        response
    }

    pub fn radio_group<'a>(
        &mut self,
        current: &mut Option<usize>,
        props: impl Into<RadioGroup<'a>>,
    ) -> Response {
        draw_radio_group(self.ui_mut(), current, props.into())
    }
}

fn draw_radio_group(ui: &mut Ui, current: &mut Option<usize>, props: RadioGroup<'_>) -> Response {
    let mut combined_response: Option<Response> = None;

    ui.push_id(props.id, |ui| {
        let _ = ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = props.gap;
            ui.vertical(|ui| {
                for option in props.options {
                    let mut response = draw_radio(
                        ui,
                        *current == Some(option.value),
                        Radio::new()
                            .label(option.label)
                            .description(option.description.unwrap_or("")),
                    );
                    if response.clicked() && *current != Some(option.value) {
                        *current = Some(option.value);
                        response.mark_changed();
                    }

                    combined_response = Some(match combined_response.take() {
                        Some(previous) => previous.union(response),
                        None => response,
                    });
                }
            })
            .inner;
        });
    });

    combined_response.unwrap_or_else(|| ui.allocate_exact_size(egui::Vec2::ZERO, Sense::hover()).1)
}

fn draw_radio(ui: &mut Ui, selected: bool, props: Radio<'_>) -> Response {
    if let Some(id) = props.id.clone() {
        return ui
            .push_id(id, |ui| draw_radio_inner(ui, selected, props))
            .inner;
    }

    draw_radio_inner(ui, selected, props)
}

fn draw_radio_inner(ui: &mut Ui, selected: bool, props: Radio<'_>) -> Response {
    let has_text = props.label.is_some() || props.description.is_some();
    if !has_text {
        return draw_radio_control(ui, selected).on_hover_cursor(CursorIcon::PointingHand);
    }

    let click_id = ui.next_auto_id();
    let content = ui.scope(|ui| {
        let label_slot_width =
            (ui.available_width() - RADIO_CONTROL_SIZE - RADIO_LABEL_GAP).max(0.0);
        ui.spacing_mut().item_spacing.x = RADIO_LABEL_GAP;
        ui.horizontal(|ui| {
            let control = ui
                .allocate_ui_with_layout(
                    egui::vec2(RADIO_CONTROL_SIZE, 0.0),
                    Layout::top_down(Align::Min),
                    |ui| draw_radio_control(ui, selected),
                )
                .inner
                .on_hover_cursor(CursorIcon::PointingHand);
            let label = ui
                .scope(|ui| {
                    ui.set_min_width(label_slot_width);
                    ui.set_max_width(label_slot_width);
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.spacing_mut().item_spacing.y = RADIO_TEXT_GAP;
                    ui.vertical(|ui| {
                        let runtime = crate::theme::runtime_for_ui(ui);
                        let mut combined: Option<Response> = None;
                        if let Some(label_text) = props.label {
                            let response = ui.add(
                                egui::Label::new(
                                    egui::RichText::new(label_text)
                                        .color(tokens::text_primary(runtime)),
                                )
                                .selectable(false)
                                .wrap(),
                            );
                            combined = Some(response);
                        }
                        if let Some(description) = props.description.filter(|text| !text.is_empty())
                        {
                            let response = ui.add(
                                egui::Label::new(
                                    egui::RichText::new(description)
                                        .color(tokens::text_muted(runtime))
                                        .size(12.0),
                                )
                                .selectable(false)
                                .wrap(),
                            );
                            combined = Some(match combined.take() {
                                Some(previous) => previous.union(response),
                                None => response,
                            });
                        }
                        combined.unwrap_or_else(|| {
                            ui.allocate_exact_size(egui::Vec2::ZERO, Sense::hover()).1
                        })
                    })
                    .inner
                })
                .inner
                .on_hover_cursor(CursorIcon::PointingHand);

            control.union(label)
        })
    });

    content
        .inner
        .inner
        .union(ui.interact(content.response.rect, click_id, Sense::click()))
        .on_hover_cursor(CursorIcon::PointingHand)
}

fn draw_radio_control(ui: &mut Ui, selected: bool) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(RADIO_CONTROL_SIZE, RADIO_CONTROL_SIZE),
        Sense::click(),
    );
    let hovered = response.hovered();
    let pressed = response.is_pointer_button_down_on();
    let focused = response.has_focus();
    let center = rect.center();
    let radius = rect.width() * 0.5;

    let (fill, mut stroke) = if selected {
        let selected_fill = if pressed {
            tokens::primary_active_bg(runtime)
        } else if hovered {
            tokens::primary_hover_bg(runtime)
        } else {
            tokens::primary_bg(runtime)
        };
        (selected_fill, Stroke::new(1.0, selected_fill))
    } else if pressed {
        (
            tokens::input_hover_background(runtime),
            Stroke::new(1.0, tokens::input_hover_border(runtime)),
        )
    } else if hovered {
        (
            tokens::input_hover_background(runtime),
            Stroke::new(1.0, tokens::input_hover_border(runtime)),
        )
    } else {
        (
            tokens::input_background(runtime),
            Stroke::new(1.0, tokens::input_border(runtime)),
        )
    };

    if focused {
        stroke = tokens::input_focus_stroke(runtime);
    }

    ui.painter().circle_filled(center, radius, fill);
    ui.painter().circle_stroke(center, radius, stroke);

    if selected {
        ui.painter()
            .circle_filled(center, RADIO_DOT_RADIUS, tokens::primary_fg(runtime));
    }

    response
}

#[cfg(test)]
mod tests {
    use super::{Radio, RadioGroup, RadioOption};
    use crate::runtime_components::ComponentUiExt;
    use crate::theme::{self, ThemeMode};
    use egui::{Align, CentralPanel, Context, Id, Layout, Pos2, RawInput, Response};

    #[test]
    fn radio_group_preserves_unselected_state_until_user_choice() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let mut selected = None;
        let options = [RadioOption::new(0, "A"), RadioOption::new(1, "B")];

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = ui.components().radio_group(
                    &mut selected,
                    RadioGroup::new(Id::new("radio_group_test"), &options),
                );
            });
        });

        assert_eq!(selected, None);
    }

    #[test]
    fn labeled_radio_uses_available_width_for_text_slot() {
        let context = Context::default();
        theme::install(&context, theme::ThemeSpec::default(), ThemeMode::Dark);
        let mut row_response = None;
        let mut slot_right = 0.0;
        let mut selected = false;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let origin: Pos2 = ui.next_widget_position();
                slot_right = origin.x + 280.0;
                row_response = Some(
                    ui.scope(|ui| {
                        ui.set_min_width(280.0);
                        ui.set_max_width(280.0);
                        ui.with_layout(Layout::top_down(Align::Min), |ui| {
                            ui.components().radio(
                                &mut selected,
                                Radio::new()
                                    .label("Starter")
                                    .description("Explicit checked state"),
                            )
                        })
                        .inner
                    })
                    .inner,
                );
            });
        });

        let row_response: Response = row_response.expect("radio row should render");
        assert!(row_response.rect.right() <= slot_right + 0.5);
        assert!(row_response.rect.width() >= 160.0);
    }
}
