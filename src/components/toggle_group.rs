use super::api::ComponentUi;
use crate::ui::{tokens, typography};
use egui::{Align2, CornerRadius, CursorIcon, FontId, Id, Response, Sense, Stroke, StrokeKind, Ui};

const TOGGLE_GROUP_HEIGHT: f32 = 32.0;
const TOGGLE_GROUP_MIN_SEGMENT_WIDTH: f32 = 56.0;
const TOGGLE_GROUP_PADDING_X: f32 = 14.0;

#[derive(Debug, Clone, Copy)]
pub struct ToggleGroup<'a> {
    pub id: Id,
    pub options: &'a [&'a str],
    pub min_segment_width: f32,
}

impl<'a> ToggleGroup<'a> {
    pub fn new(id: Id, options: &'a [&'a str]) -> Self {
        Self {
            id,
            options,
            min_segment_width: TOGGLE_GROUP_MIN_SEGMENT_WIDTH,
        }
    }

    pub fn min_segment_width(mut self, min_segment_width: f32) -> Self {
        self.min_segment_width = min_segment_width.max(1.0);
        self
    }
}

impl ComponentUi<'_> {
    pub fn toggle_group<'a>(
        &mut self,
        current: &mut usize,
        props: impl Into<ToggleGroup<'a>>,
    ) -> Response {
        draw_toggle_group(self.ui_mut(), current, props.into())
    }
}

fn draw_toggle_group(ui: &mut Ui, current: &mut usize, props: ToggleGroup<'_>) -> Response {
    if props.options.is_empty() {
        return ui.allocate_response(egui::Vec2::ZERO, Sense::hover());
    }

    let runtime = crate::theme::runtime_for_ui(ui);
    let label_font: FontId = typography::label_font();
    let mut combined_response: Option<Response> = None;
    *current = (*current).min(props.options.len().saturating_sub(1));

    ui.push_id(props.id, |ui| {
        let label_widths = props
            .options
            .iter()
            .copied()
            .map(|label| {
                ui.fonts_mut(|fonts| {
                    fonts
                        .layout_no_wrap(
                            label.to_owned(),
                            label_font.clone(),
                            tokens::text_primary(runtime),
                        )
                        .size()
                        .x
                })
            })
            .collect::<Vec<_>>();
        let segment_widths = label_widths
            .iter()
            .map(|label_width| {
                (label_width + (TOGGLE_GROUP_PADDING_X * 2.0)).max(props.min_segment_width)
            })
            .collect::<Vec<_>>();
        let total_width = segment_widths.iter().sum::<f32>();
        let (group_rect, _) =
            ui.allocate_exact_size(egui::vec2(total_width, TOGGLE_GROUP_HEIGHT), Sense::hover());
        let group_border = Stroke::new(1.0, tokens::button_secondary_border(runtime));
        ui.painter().rect(
            group_rect,
            CornerRadius::same(tokens::radius_md(runtime)),
            tokens::muted_surface(runtime),
            group_border,
            StrokeKind::Outside,
        );

        let mut segment_left = group_rect.left();
        for (index, label) in props.options.iter().copied().enumerate() {
            let width = segment_widths[index];
            let rect = egui::Rect::from_min_size(
                egui::pos2(segment_left, group_rect.top()),
                egui::vec2(width, TOGGLE_GROUP_HEIGHT),
            );
            let response = ui.interact(rect, props.id.with(index), Sense::click());
            let selected = *current == index;
            let fill = if selected {
                tokens::card_background(runtime)
            } else if response.is_pointer_button_down_on() {
                tokens::row_active_bg(runtime)
            } else if response.hovered() {
                tokens::row_hover_bg(runtime)
            } else {
                tokens::TRANSPARENT
            };

            let corner_radius = if props.options.len() == 1 {
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
                .rect(rect, corner_radius, fill, Stroke::NONE, StrokeKind::Outside);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                label_font.clone(),
                if selected {
                    tokens::text_primary(runtime)
                } else {
                    tokens::text_secondary(runtime)
                },
            );

            if response.clicked() && !selected {
                *current = index;
            }

            if index + 1 < props.options.len() {
                let x = rect.right();
                ui.painter().line_segment(
                    [
                        egui::pos2(x, group_rect.top() + 1.0),
                        egui::pos2(x, group_rect.bottom() - 1.0),
                    ],
                    group_border,
                );
            }

            combined_response = Some(match combined_response.take() {
                Some(previous) => previous.union(response),
                None => response,
            });
            segment_left = rect.right();
        }
    });

    combined_response
        .unwrap_or_else(|| ui.allocate_exact_size(egui::Vec2::ZERO, Sense::hover()).1)
        .on_hover_cursor(CursorIcon::PointingHand)
}

#[cfg(test)]
mod tests {
    use super::{ToggleGroup, TOGGLE_GROUP_HEIGHT};
    use crate::components::ComponentUiExt;
    use egui::{
        pos2, vec2, CentralPanel, Context, Event, Id, Modifiers, PointerButton, Pos2, RawInput,
    };

    const TEST_OPTIONS: &[&str] = &["Top", "Right", "Bottom"];

    #[test]
    fn empty_toggle_group_allocates_no_clickable_area() {
        let context = Context::default();
        let mut selected = 0;
        let mut rect = egui::Rect::ZERO;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = ui
                    .components()
                    .toggle_group(&mut selected, ToggleGroup::new(Id::new("empty"), &[]))
                    .rect;
            });
        });

        assert_eq!(rect.size(), egui::Vec2::ZERO);
        assert_eq!(selected, 0);
    }

    #[test]
    fn clicking_a_toggle_group_option_updates_the_selection() {
        let context = Context::default();
        let mut selected = 0;
        let (_, second_center) = render_toggle_group(&context, RawInput::default(), &mut selected);

        let _ = render_toggle_group(&context, pointer_input(second_center, true), &mut selected);
        let _ = render_toggle_group(&context, pointer_input(second_center, false), &mut selected);

        assert_eq!(selected, 1);
    }

    fn render_toggle_group(
        context: &Context,
        input: RawInput,
        selected: &mut usize,
    ) -> (Pos2, Pos2) {
        let mut first_center = Pos2::ZERO;
        let mut second_center = Pos2::ZERO;

        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                let origin = ui.next_widget_position();
                first_center = origin + vec2(56.0, TOGGLE_GROUP_HEIGHT) * 0.5;
                second_center = pos2(origin.x + 84.0, origin.y + TOGGLE_GROUP_HEIGHT * 0.5);
                let _ = ui.components().toggle_group(
                    selected,
                    ToggleGroup::new(Id::new("toggle_group_actions"), TEST_OPTIONS),
                );
            });
        });

        (first_center, second_center)
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
