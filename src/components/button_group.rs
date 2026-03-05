use crate::ui::tokens;
use egui::{Align2, Color32, CornerRadius, CursorIcon, FontId, Id, Sense, Stroke, StrokeKind, Ui};

#[derive(Debug, Clone, Copy)]
pub struct ButtonGroupProps<'a> {
    pub id: Id,
    pub options: &'a [&'a str],
}

impl<'a> ButtonGroupProps<'a> {
    pub fn new(id: Id, options: &'a [&'a str]) -> Self {
        Self { id, options }
    }
}

pub fn button_group(ui: &mut Ui, selected_index: &mut usize, props: ButtonGroupProps<'_>) {
    if props.options.is_empty() {
        *selected_index = 0;
        return;
    }

    let clamped_index = (*selected_index).min(props.options.len() - 1);
    *selected_index = clamped_index;

    ui.push_id(props.id, |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.horizontal(|ui| {
            let button_padding_x = ui.spacing().button_padding.x;
            let text_font = FontId::new(12.0, egui::FontFamily::Proportional);
            let height = (ui.spacing().interact_size.y - 4.0).max(24.0);
            let mut segment_rects = Vec::with_capacity(props.options.len());

            for (index, label) in props.options.iter().copied().enumerate() {
                let galley_width = ui.fonts_mut(|fonts| {
                    fonts
                        .layout_no_wrap(label.to_owned(), text_font.clone(), tokens::TEXT_PRIMARY)
                        .size()
                        .x
                });
                let width = (galley_width + (button_padding_x * 2.0)).max(58.0);
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
                segment_rects.push(rect);

                let fill = if response.is_pointer_button_down_on() {
                    tokens::BUTTON_SECONDARY_ACTIVE_BG
                } else if response.hovered() {
                    tokens::BUTTON_SECONDARY_HOVER_BG
                } else {
                    tokens::BUTTON_SECONDARY_BG
                };
                let corner = if props.options.len() == 1 {
                    CornerRadius::same(tokens::RADIUS_MD)
                } else if index == 0 {
                    CornerRadius {
                        nw: tokens::RADIUS_MD,
                        ne: 0,
                        sw: tokens::RADIUS_MD,
                        se: 0,
                    }
                } else if index == props.options.len() - 1 {
                    CornerRadius {
                        nw: 0,
                        ne: tokens::RADIUS_MD,
                        sw: 0,
                        se: tokens::RADIUS_MD,
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
                    tokens::TEXT_PRIMARY,
                );

                if response.clicked() {
                    *selected_index = index;
                }

                let _ = response.on_hover_cursor(CursorIcon::PointingHand);
            }

            if let (Some(first), Some(last)) = (segment_rects.first(), segment_rects.last()) {
                let group_rect = first.union(*last);
                let border = Stroke::new(1.0, tokens::BUTTON_SECONDARY_BORDER);
                ui.painter().rect(
                    group_rect,
                    CornerRadius::same(tokens::RADIUS_MD),
                    Color32::TRANSPARENT,
                    border,
                    StrokeKind::Outside,
                );

                for rect in segment_rects
                    .iter()
                    .take(segment_rects.len().saturating_sub(1))
                {
                    let x = rect.right();
                    ui.painter().line_segment(
                        [
                            egui::pos2(x, group_rect.top() + 1.0),
                            egui::pos2(x, group_rect.bottom() - 1.0),
                        ],
                        border,
                    );
                }
            }
        });
    });
}
