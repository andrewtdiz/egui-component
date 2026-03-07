use super::api::ComponentUi;
use crate::ui::{icons, tokens};
use egui::{Align2, CornerRadius, CursorIcon, Id, Rect, RichText, Stroke, StrokeKind, Ui};

const STACKED_TAB_SIZE: egui::Vec2 = egui::vec2(80.0, 68.0);
const STACKED_TAB_GAP: f32 = 8.0;
const STACKED_TAB_ICON_SIZE: f32 = 18.0;

#[derive(Debug, Clone, Copy)]
pub struct TabOption<'a> {
    pub value: usize,
    pub label: &'a str,
    pub icon: Option<&'a str>,
}

impl<'a> TabOption<'a> {
    pub const fn new(value: usize, label: &'a str) -> Self {
        Self {
            value,
            label,
            icon: None,
        }
    }

    pub const fn with_icon(value: usize, label: &'a str, icon: &'a str) -> Self {
        Self {
            value,
            label,
            icon: Some(icon),
        }
    }
}

impl ComponentUi<'_> {
    pub fn tabs(&mut self, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
        draw_tabs(self.raw_mut(), id, current, options);
    }

    pub fn stacked_tabs(&mut self, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
        let _ = draw_stacked_tabs(self.raw_mut(), id, current, options);
    }
}

fn draw_tabs(ui: &mut Ui, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
    if options.is_empty() {
        return;
    }
    let dark_mode = ui.visuals().dark_mode;

    ui.push_id(id, |ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.horizontal(|ui| {
            for option in options {
                let selected = *current == option.value;
                let response = ui
                    .add(
                        egui::Button::new(RichText::new(option.label).color(if selected {
                            tokens::row_selected_text(dark_mode)
                        } else {
                            tokens::text_secondary(dark_mode)
                        }))
                        .fill(tokens::TRANSPARENT)
                        .stroke(Stroke::new(1.0, tokens::TRANSPARENT))
                        .corner_radius(CornerRadius::ZERO)
                        .min_size(egui::vec2(0.0, ui.spacing().interact_size.y)),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand);

                if selected {
                    let y = response.rect.bottom() - 1.0;
                    ui.painter().line_segment(
                        [
                            egui::pos2(response.rect.left() + 4.0, y),
                            egui::pos2(response.rect.right() - 4.0, y),
                        ],
                        Stroke::new(2.6, tokens::text_secondary(dark_mode)),
                    );
                }

                if response.clicked() && !selected {
                    *current = option.value;
                }
            }
        });
    });
}

fn draw_stacked_tabs(ui: &mut Ui, id: Id, current: &mut usize, options: &[TabOption<'_>]) -> Rect {
    if options.is_empty() {
        return Rect::NOTHING;
    }

    let dark_mode = ui.visuals().dark_mode;
    *current = (*current).min(options.len().saturating_sub(1));

    ui.push_id(id, |ui| {
        ui.spacing_mut().item_spacing.x = STACKED_TAB_GAP;
        ui.horizontal(|ui| {
            for option in options {
                let selected = *current == option.value;
                let (rect, response) =
                    ui.allocate_exact_size(STACKED_TAB_SIZE, egui::Sense::click());

                let fill = if response.is_pointer_button_down_on() {
                    tokens::button_secondary_active_bg(dark_mode)
                } else if selected || response.hovered() {
                    tokens::button_secondary_hover_bg(dark_mode)
                } else {
                    tokens::TRANSPARENT
                };
                ui.painter().rect(
                    rect,
                    CornerRadius::same(tokens::RADIUS_SM),
                    fill,
                    Stroke::NONE,
                    StrokeKind::Outside,
                );

                if let Some(icon) = option.icon {
                    if let Some(image) = icons::image(ui.ctx(), icon, STACKED_TAB_ICON_SIZE) {
                        let icon_rect = Rect::from_center_size(
                            egui::pos2(rect.center().x, rect.top() + 18.0),
                            egui::vec2(STACKED_TAB_ICON_SIZE, STACKED_TAB_ICON_SIZE),
                        );
                        // Paint directly so the icon never participates in layout after the
                        // tile rect has already been allocated.
                        image
                            .tint(if selected {
                                tokens::text_primary(dark_mode)
                            } else {
                                tokens::text_secondary(dark_mode)
                            })
                            .paint_at(ui, icon_rect);
                    }
                }

                ui.painter().text(
                    egui::pos2(rect.center().x, rect.bottom() - 18.0),
                    Align2::CENTER_CENTER,
                    option.label,
                    egui::FontId::new(12.0, egui::FontFamily::Proportional),
                    if selected {
                        tokens::text_primary(dark_mode)
                    } else {
                        tokens::text_secondary(dark_mode)
                    },
                );

                if response.clicked() && !selected {
                    *current = option.value;
                }

                let _ = response.on_hover_cursor(CursorIcon::PointingHand);
            }
        })
        .response
        .rect
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::{draw_stacked_tabs, TabOption, STACKED_TAB_GAP, STACKED_TAB_SIZE};
    use egui::{CentralPanel, Context, Id, RawInput, Rect};

    #[test]
    fn renders_stacked_tabs() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;
        let options = [
            TabOption::with_icon(0, "Templates", "layout-template"),
            TabOption::with_icon(1, "Assets", "panel-top"),
        ];
        let mut current = 0;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = draw_stacked_tabs(ui, Id::new("stacked_tabs_test"), &mut current, &options);
            });
        });

        let expected_width = (STACKED_TAB_SIZE.x * options.len() as f32)
            + (STACKED_TAB_GAP * options.len().saturating_sub(1) as f32);
        assert_eq!(rect.width(), expected_width);
        assert_eq!(rect.height(), STACKED_TAB_SIZE.y);
    }
}
