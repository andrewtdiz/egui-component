use super::api::ComponentUi;
use crate::layout;
use crate::ui::tokens;
use egui::{CursorIcon, Id, Response, Sense, Stroke, Ui};

const RADIO_CONTROL_SIZE: f32 = 16.0;
const RADIO_DOT_RADIUS: f32 = 4.0;
const RADIO_GROUP_GAP: f32 = 8.0;
const RADIO_TEXT_GAP: f32 = 2.0;

#[derive(Debug, Clone, Copy)]
pub struct Radio<'a> {
    pub label: Option<&'a str>,
    pub description: Option<&'a str>,
}

impl<'a> Radio<'a> {
    pub fn new() -> Self {
        Self {
            label: None,
            description: None,
        }
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
        let _ = layout::column().gap(props.gap).show(ui, |ui| {
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
        });
    });

    combined_response.unwrap_or_else(|| ui.allocate_exact_size(egui::Vec2::ZERO, Sense::hover()).1)
}

fn draw_radio(ui: &mut Ui, selected: bool, props: Radio<'_>) -> Response {
    let has_text = props.label.is_some() || props.description.is_some();
    if !has_text {
        return draw_radio_control(ui, selected).on_hover_cursor(CursorIcon::PointingHand);
    }

    let click_id = ui.next_auto_id();
    let content = layout::row().gap(10.0).show(ui, |ui| {
        let control = draw_radio_control(ui, selected).on_hover_cursor(CursorIcon::PointingHand);
        let label = ui
            .scope(|ui| {
                ui.style_mut().interaction.selectable_labels = false;
                let _ = layout::column().gap(RADIO_TEXT_GAP).show(ui, |ui| {
                    let runtime = crate::theme::runtime_for_ui(ui);
                    if let Some(label_text) = props.label {
                        let _ = ui.add(
                            egui::Label::new(
                                egui::RichText::new(label_text)
                                    .color(tokens::text_primary(runtime)),
                            )
                            .selectable(false),
                        );
                    }
                    if let Some(description) = props.description.filter(|text| !text.is_empty()) {
                        let _ = ui.add(
                            egui::Label::new(
                                egui::RichText::new(description)
                                    .color(tokens::text_muted(runtime))
                                    .size(12.0),
                            )
                            .selectable(false),
                        );
                    }
                });
            })
            .response
            .on_hover_cursor(CursorIcon::PointingHand);

        control.union(label)
    });

    content
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
            tokens::input_focus_background(runtime),
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
    use super::{RadioGroup, RadioOption};
    use crate::components::ComponentUiExt;
    use crate::theme::{self, ThemeMode};
    use egui::{CentralPanel, Context, Id, RawInput};

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
}
