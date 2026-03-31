use super::api::ComponentUi;
use crate::ui::{tokens, typography};
use egui::{Align2, CornerRadius, CursorIcon, FontId, Id, Sense, Stroke, StrokeKind, Ui};

#[derive(Debug, Clone, Copy)]
pub struct ButtonGroup<'a> {
    pub id: Id,
    pub options: &'a [&'a str],
}

impl<'a> ButtonGroup<'a> {
    pub fn new(id: Id, options: &'a [&'a str]) -> Self {
        Self { id, options }
    }
}

impl ComponentUi<'_> {
    pub fn button_group<'a>(&mut self, props: impl Into<ButtonGroup<'a>>) -> Option<usize> {
        draw_button_group(self.ui_mut(), props.into())
    }
}

fn draw_button_group(ui: &mut Ui, props: ButtonGroup<'_>) -> Option<usize> {
    if props.options.is_empty() {
        return None;
    }

    let runtime = crate::theme::runtime_for_ui(ui);
    let mut clicked_index = None;

    let button_padding_x = tokens::SPACING_BUTTON_PADDING_X;
    let text_font: FontId = typography::label_font();
    let height = (tokens::SPACING_INTERACT_HEIGHT - 4.0).max(24.0);
    let segment_widths = props
        .options
        .iter()
        .copied()
        .map(|label| {
            let galley_width = ui.fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(
                        label.to_owned(),
                        text_font.clone(),
                        tokens::text_primary(runtime),
                    )
                    .size()
                    .x
            });
            (galley_width + (button_padding_x * 2.0)).max(58.0)
        })
        .collect::<Vec<_>>();
    let total_width = segment_widths.iter().sum::<f32>();

    ui.push_id(props.id, |ui| {
        let (group_rect, _) =
            ui.allocate_exact_size(egui::vec2(total_width, height), Sense::hover());
        let border = Stroke::new(1.0, tokens::button_secondary_border(runtime));
        let mut segment_left = group_rect.left();

        for (index, label) in props.options.iter().copied().enumerate() {
            let rect = egui::Rect::from_min_size(
                egui::pos2(segment_left, group_rect.top()),
                egui::vec2(segment_widths[index], height),
            );
            let response = ui.interact(rect, ui.id().with(index), Sense::click());
            let fill = if response.is_pointer_button_down_on() {
                tokens::button_secondary_active_bg(runtime)
            } else if response.hovered() {
                tokens::button_secondary_hover_bg(runtime)
            } else {
                tokens::button_secondary_bg(runtime)
            };
            let corner = if props.options.len() == 1 {
                CornerRadius::same(tokens::radius_md(runtime))
            } else if index == 0 {
                CornerRadius {
                    nw: tokens::radius_md(runtime),
                    ne: 0,
                    sw: tokens::radius_md(runtime),
                    se: 0,
                }
            } else if index == props.options.len() - 1 {
                CornerRadius {
                    nw: 0,
                    ne: tokens::radius_md(runtime),
                    sw: 0,
                    se: tokens::radius_md(runtime),
                }
            } else {
                CornerRadius::ZERO
            };

            ui.painter()
                .rect(rect, corner, fill, Stroke::NONE, StrokeKind::Outside);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                text_font.clone(),
                tokens::text_primary(runtime),
            );

            if response.clicked() {
                clicked_index = Some(index);
            }

            if index + 1 < props.options.len() {
                let x = rect.right();
                ui.painter().line_segment(
                    [
                        egui::pos2(x, group_rect.top() + 1.0),
                        egui::pos2(x, group_rect.bottom() - 1.0),
                    ],
                    border,
                );
            }

            let _ = response.on_hover_cursor(CursorIcon::PointingHand);
            segment_left = rect.right();
        }

        ui.painter().rect(
            group_rect,
            CornerRadius::same(tokens::radius_md(runtime)),
            tokens::TRANSPARENT,
            border,
            StrokeKind::Outside,
        );
    });

    clicked_index
}

#[cfg(test)]
mod tests {
    use super::ButtonGroup;
    use crate::components::ComponentUiExt;
    use crate::ui::tokens;
    use egui::{
        pos2, vec2, CentralPanel, Context, Event, Id, Modifiers, PointerButton, Pos2, RawInput,
    };

    const TEST_OPTIONS: &[&str] = &["One", "Two", "Three"];

    #[test]
    fn empty_button_group_has_no_action() {
        let context = Context::default();
        let mut clicked = Some(0);

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                clicked = ui
                    .components()
                    .button_group(ButtonGroup::new(Id::new("button_group_empty"), &[]));
            });
        });

        assert_eq!(clicked, None);
    }

    #[test]
    fn button_group_returns_clicked_action_index() {
        let context = Context::default();
        let (_, second_button_center, _) = render_button_group(&context, RawInput::default());

        let _ = render_button_group(&context, pointer_input(second_button_center, true));
        let (_, _, clicked) =
            render_button_group(&context, pointer_input(second_button_center, false));

        assert_eq!(clicked, Some(1));
    }

    #[test]
    fn button_group_advances_layout_as_a_single_row() {
        let context = Context::default();
        let mut before = Pos2::ZERO;
        let mut after = Pos2::ZERO;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                before = ui.next_widget_position();
                let _ = ui.components().button_group(ButtonGroup::new(
                    Id::new("button_group_row_layout"),
                    TEST_OPTIONS,
                ));
                after = ui.next_widget_position();
            });
        });

        let height = (tokens::SPACING_INTERACT_HEIGHT - 4.0).max(24.0);
        assert!(after.y - before.y < height * 2.0);
    }

    fn render_button_group(context: &Context, input: RawInput) -> (Pos2, Pos2, Option<usize>) {
        let mut first_button_center = Pos2::ZERO;
        let mut second_button_center = Pos2::ZERO;
        let mut clicked = None;

        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                let origin = ui.next_widget_position();
                let button_size = vec2(58.0, (tokens::SPACING_INTERACT_HEIGHT - 4.0).max(24.0));
                first_button_center = origin + button_size * 0.5;
                second_button_center = pos2(
                    origin.x + button_size.x * 1.5,
                    origin.y + button_size.y * 0.5,
                );
                clicked = ui.components().button_group(ButtonGroup::new(
                    Id::new("button_group_actions"),
                    TEST_OPTIONS,
                ));
            });
        });

        (first_button_center, second_button_center, clicked)
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
