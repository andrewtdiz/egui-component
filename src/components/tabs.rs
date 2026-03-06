use super::api::ComponentUi;
use crate::ui::tokens;
use egui::{CornerRadius, CursorIcon, Id, RichText, Stroke, Ui};

#[derive(Debug, Clone, Copy)]
pub struct TabOption<'a> {
    pub value: usize,
    pub label: &'a str,
}

impl<'a> TabOption<'a> {
    pub const fn new(value: usize, label: &'a str) -> Self {
        Self { value, label }
    }
}

impl ComponentUi<'_> {
    pub fn tabs(&mut self, id: Id, current: &mut usize, options: &[TabOption<'_>]) {
        draw_tabs(self.raw_mut(), id, current, options);
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
                            tokens::TEXT_SECONDARY
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
                        Stroke::new(2.6, tokens::TEXT_SECONDARY),
                    );
                }

                if response.clicked() && !selected {
                    *current = option.value;
                }
            }
        });
    });
}
