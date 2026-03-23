use super::{
    api::ComponentUi,
    button::{Button, ButtonVariant},
    common::ControlSize,
    label::{Label, LabelTone},
};
use crate::{
    layout,
    primitives::{row_chrome, RowChrome},
    ui::{tokens, typography},
};
use egui::{Align2, CornerRadius, Id, Response, Sense, Stroke, Vec2};

const PAGINATION_ITEM_GAP: f32 = 4.0;
const PAGINATION_PAGE_BUTTON_SIZE: Vec2 = egui::vec2(32.0, 32.0);
const PAGINATION_NAV_ICON_SIZE: f32 = 12.0;
const PAGINATION_ELLIPSIS_SIZE: f32 = 14.0;

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub id: Id,
    pub page_count: usize,
    pub sibling_count: usize,
}

impl Pagination {
    pub fn new(id: Id, page_count: usize) -> Self {
        Self {
            id,
            page_count,
            sibling_count: 1,
        }
    }

    pub fn sibling_count(mut self, sibling_count: usize) -> Self {
        self.sibling_count = sibling_count;
        self
    }
}

impl ComponentUi<'_> {
    pub fn pagination(
        &mut self,
        current_page: &mut usize,
        props: impl Into<Pagination>,
    ) -> Response {
        draw_pagination(self, current_page, props.into())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum PaginationItem {
    Page(usize),
    Ellipsis,
}

fn draw_pagination(
    ui: &mut ComponentUi<'_>,
    current_page: &mut usize,
    props: Pagination,
) -> Response {
    if props.page_count == 0 {
        return ui.ui_mut().allocate_response(Vec2::ZERO, Sense::hover());
    }

    *current_page = (*current_page).clamp(1, props.page_count);
    let items = pagination_items(*current_page, props.page_count, props.sibling_count);
    let overrides = ui.overrides();

    ui.ui_mut()
        .push_id(props.id, |ui| {
            ui.scope(|ui| {
                let runtime = crate::theme::runtime_for_ui(ui);
                apply_pagination_corner_radius(ui, runtime);

                layout::row()
                    .gap(PAGINATION_ITEM_GAP)
                    .show(ui, |ui| {
                        let mut ui = ComponentUi::with_overrides(ui, overrides);
                        if *current_page > 1
                            && ui
                                .button(
                                    Button::new("Previous")
                                        .variant(ButtonVariant::Ghost)
                                        .size(ControlSize::Sm)
                                        .leading_icon("chevron-left")
                                        .icon_size(PAGINATION_NAV_ICON_SIZE),
                                )
                                .clicked()
                        {
                            *current_page -= 1;
                        }

                        for item in items {
                            match item {
                                PaginationItem::Page(page) => {
                                    draw_page_button(&mut ui, current_page, page);
                                }
                                PaginationItem::Ellipsis => {
                                    let _ = ui.label(
                                        Label::new("...")
                                            .tone(LabelTone::Muted)
                                            .size(PAGINATION_ELLIPSIS_SIZE),
                                    );
                                }
                            }
                        }

                        if *current_page < props.page_count
                            && ui
                                .button(
                                    Button::new("Next")
                                        .variant(ButtonVariant::Ghost)
                                        .size(ControlSize::Sm)
                                        .trailing_icon("chevron-right")
                                        .icon_size(PAGINATION_NAV_ICON_SIZE),
                                )
                                .clicked()
                        {
                            *current_page += 1;
                        }
                    })
                    .response
            })
            .inner
        })
        .inner
}

fn draw_page_button(ui: &mut ComponentUi<'_>, current_page: &mut usize, page: usize) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let is_current = *current_page == page;
    let stroke = if is_current {
        Stroke::new(1.0, tokens::separator(runtime))
    } else {
        Stroke::NONE
    };
    let (rect, response) = row_chrome(
        ui.ui_mut(),
        RowChrome::new(PAGINATION_PAGE_BUTTON_SIZE)
            .corner_radius(tokens::radius_sm(runtime))
            .stroke(stroke),
        |response| {
            if is_current {
                tokens::card_background(runtime)
            } else if response.is_pointer_button_down_on() {
                tokens::button_secondary_active_bg(runtime)
            } else if response.hovered() {
                tokens::button_secondary_hover_bg(runtime)
            } else {
                tokens::TRANSPARENT
            }
        },
    );

    ui.ui_mut().painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        page.to_string(),
        typography::label_font(),
        tokens::text_primary(runtime),
    );

    if response.clicked() && !is_current {
        *current_page = page;
    }
}

fn apply_pagination_corner_radius(ui: &mut egui::Ui, runtime: crate::theme::ThemeRuntime) {
    let corner_radius = CornerRadius::same(tokens::radius_sm(runtime));
    let visuals = &mut ui.style_mut().visuals.widgets;
    visuals.noninteractive.corner_radius = corner_radius;
    visuals.inactive.corner_radius = corner_radius;
    visuals.hovered.corner_radius = corner_radius;
    visuals.active.corner_radius = corner_radius;
    visuals.open.corner_radius = corner_radius;
}

fn pagination_items(
    current_page: usize,
    page_count: usize,
    sibling_count: usize,
) -> Vec<PaginationItem> {
    if page_count == 0 {
        return Vec::new();
    }

    let current_page = current_page.clamp(1, page_count);
    let visible_page_slots = sibling_count.saturating_mul(2).saturating_add(5);

    if page_count <= visible_page_slots {
        return (1..=page_count).map(PaginationItem::Page).collect();
    }

    let left_sibling = current_page.saturating_sub(sibling_count).max(1);
    let right_sibling = current_page.saturating_add(sibling_count).min(page_count);
    let show_left_ellipsis = left_sibling > 2;
    let show_right_ellipsis = right_sibling < page_count.saturating_sub(1);

    match (show_left_ellipsis, show_right_ellipsis) {
        (false, false) => (1..=page_count).map(PaginationItem::Page).collect(),
        (false, true) => {
            let last_left_page = (sibling_count.saturating_mul(2).saturating_add(2))
                .min(page_count.saturating_sub(1));
            let mut items = (1..=last_left_page)
                .map(PaginationItem::Page)
                .collect::<Vec<_>>();
            items.push(PaginationItem::Ellipsis);
            items.push(PaginationItem::Page(page_count));
            items
        }
        (true, false) => {
            let start_page = page_count
                .saturating_sub(sibling_count.saturating_mul(2).saturating_add(1))
                .max(2);
            let mut items = vec![PaginationItem::Page(1), PaginationItem::Ellipsis];
            items.extend((start_page..=page_count).map(PaginationItem::Page));
            items
        }
        (true, true) => {
            let mut items = vec![PaginationItem::Page(1), PaginationItem::Ellipsis];
            items.extend((left_sibling..=right_sibling).map(PaginationItem::Page));
            items.push(PaginationItem::Ellipsis);
            items.push(PaginationItem::Page(page_count));
            items
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{pagination_items, Pagination, PaginationItem};
    use crate::components::ComponentUiExt;
    use egui::{CentralPanel, Context, Id, RawInput, Shape};

    fn collect_text_shapes(shape: &Shape, rendered_texts: &mut Vec<String>) {
        match shape {
            Shape::Text(text_shape) => {
                rendered_texts.push(text_shape.galley.job.text.clone());
            }
            Shape::Vec(shapes) => {
                for nested_shape in shapes {
                    collect_text_shapes(nested_shape, rendered_texts);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn pagination_items_insert_ellipsis_around_middle_window() {
        assert_eq!(
            pagination_items(5, 10, 1),
            vec![
                PaginationItem::Page(1),
                PaginationItem::Ellipsis,
                PaginationItem::Page(4),
                PaginationItem::Page(5),
                PaginationItem::Page(6),
                PaginationItem::Ellipsis,
                PaginationItem::Page(10),
            ]
        );
    }

    #[test]
    fn pagination_items_use_a_smaller_edge_window_by_default() {
        assert_eq!(
            pagination_items(2, 12, 1),
            vec![
                PaginationItem::Page(1),
                PaginationItem::Page(2),
                PaginationItem::Page(3),
                PaginationItem::Page(4),
                PaginationItem::Ellipsis,
                PaginationItem::Page(12),
            ]
        );
    }

    #[test]
    fn pagination_clamps_current_page_to_last_page() {
        let context = Context::default();
        let mut current_page = 99;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = ui.components().pagination(
                    &mut current_page,
                    Pagination::new(Id::new("pagination_clamp"), 5),
                );
            });
        });

        assert_eq!(current_page, 5);
    }

    #[test]
    fn first_page_hides_previous_button() {
        let context = Context::default();
        crate::theme::install(
            &context,
            crate::theme::ThemeSpec::default(),
            crate::theme::ThemeMode::Dark,
        );

        let frame_output = context.run(RawInput::default(), |context| {
            let mut current_page = 1;
            CentralPanel::default().show(context, |ui| {
                let _ = ui.components().pagination(
                    &mut current_page,
                    Pagination::new(Id::new("pagination_first_page"), 8),
                );
            });
        });

        let mut rendered_texts = Vec::new();
        for clipped_shape in frame_output.shapes {
            collect_text_shapes(&clipped_shape.shape, &mut rendered_texts);
        }

        assert!(!rendered_texts.iter().any(|text| text == "Previous"));
        assert!(rendered_texts.iter().any(|text| text == "Next"));
    }
}
