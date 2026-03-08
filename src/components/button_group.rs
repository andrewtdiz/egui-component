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

impl<'a> From<(Id, &'a [&'a str])> for ButtonGroup<'a> {
    fn from((id, options): (Id, &'a [&'a str])) -> Self {
        Self::new(id, options)
    }
}

impl ComponentUi<'_> {
    pub fn button_group<'a>(
        &mut self,
        selected_index: &mut usize,
        props: impl Into<ButtonGroup<'a>>,
    ) {
        draw_button_group(self.raw_mut(), selected_index, props.into());
    }
}

fn draw_button_group(ui: &mut Ui, selected_index: &mut usize, props: ButtonGroup<'_>) {
    if props.options.is_empty() {
        *selected_index = 0;
        return;
    }

    let clamped_index = (*selected_index).min(props.options.len() - 1);
    *selected_index = clamped_index;
    let dark_mode = ui.visuals().dark_mode;

    ui.push_id(props.id, |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.horizontal(|ui| {
            let button_padding_x = ui.spacing().button_padding.x;
            let text_font: FontId = typography::label_font();
            let height = (ui.spacing().interact_size.y - 4.0).max(24.0);
            let mut segment_rects = Vec::with_capacity(props.options.len());

            for (index, label) in props.options.iter().copied().enumerate() {
                let selected = index == *selected_index;
                let galley_width = ui.fonts_mut(|fonts| {
                    fonts
                        .layout_no_wrap(
                            label.to_owned(),
                            text_font.clone(),
                            tokens::text_primary(dark_mode),
                        )
                        .size()
                        .x
                });
                let width = (galley_width + (button_padding_x * 2.0)).max(58.0);
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
                segment_rects.push(rect);

                let fill = if selected {
                    tokens::row_selected_bg(dark_mode)
                } else if response.is_pointer_button_down_on() {
                    tokens::button_secondary_active_bg(dark_mode)
                } else if response.hovered() {
                    tokens::button_secondary_hover_bg(dark_mode)
                } else {
                    tokens::button_secondary_bg(dark_mode)
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
                let fill_rect = rect;
                ui.painter()
                    .rect(fill_rect, corner, fill, Stroke::NONE, StrokeKind::Outside);
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    label,
                    text_font.clone(),
                    if selected {
                        tokens::row_selected_text(dark_mode)
                    } else {
                        tokens::text_primary(dark_mode)
                    },
                );

                if response.clicked() {
                    *selected_index = index;
                }

                let _ = response.on_hover_cursor(CursorIcon::PointingHand);
            }

            if let (Some(first), Some(last)) = (segment_rects.first(), segment_rects.last()) {
                let group_rect = first.union(*last);
                let border = Stroke::new(1.0, tokens::button_secondary_border(dark_mode));
                ui.painter().rect(
                    group_rect,
                    CornerRadius::same(tokens::RADIUS_MD),
                    tokens::TRANSPARENT,
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
