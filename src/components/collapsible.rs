use crate::components::{icon, label, IconProps, LabelProps, LabelTone, LabelWeight};
use crate::ui::tokens;
use egui::{Align, Color32, CornerRadius, Id, Layout, Response, Sense, StrokeKind, Ui};

#[derive(Debug, Clone, Copy)]
pub struct CollapsibleProps<'a> {
    pub id: Id,
    pub title: &'a str,
    pub open: bool,
    pub leading_icon: Option<&'a str>,
    pub trailing_icon: Option<&'a str>,
    pub leading_icon_tint: Color32,
    pub trailing_icon_tint: Color32,
}

impl<'a> CollapsibleProps<'a> {
    pub fn new(id: Id, title: &'a str) -> Self {
        Self {
            id,
            title,
            open: true,
            leading_icon: None,
            trailing_icon: None,
            leading_icon_tint: tokens::TEXT_SECONDARY,
            trailing_icon_tint: tokens::TEXT_MUTED,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn leading_icon(mut self, icon_name: &'a str) -> Self {
        self.leading_icon = Some(icon_name);
        self
    }

    pub fn trailing_icon(mut self, icon_name: &'a str) -> Self {
        self.trailing_icon = Some(icon_name);
        self
    }

    pub fn leading_icon_tint(mut self, tint: Color32) -> Self {
        self.leading_icon_tint = tint;
        self
    }

    pub fn trailing_icon_tint(mut self, tint: Color32) -> Self {
        self.trailing_icon_tint = tint;
        self
    }
}

pub fn collapsible<R>(
    ui: &mut Ui,
    open: &mut bool,
    props: CollapsibleProps<'_>,
    add: impl FnOnce(&mut Ui) -> R,
) -> Response {
    if *open != props.open {
        *open = props.open;
    }

    let header_height = (ui.spacing().interact_size.y - 2.0).max(30.0);
    let desired_size = egui::vec2(ui.available_width(), header_height);
    let (rect, mut header_response) = ui.allocate_exact_size(desired_size, Sense::click());

    if header_response.clicked() {
        *open = !*open;
        header_response.mark_changed();
    }

    let fill = if header_response.hovered() {
        tokens::INPUT_HOVER_BACKGROUND
    } else {
        egui::Color32::TRANSPARENT
    };

    ui.painter().rect(
        rect,
        CornerRadius::same(tokens::RADIUS_SM),
        fill,
        egui::Stroke::NONE,
        StrokeKind::Outside,
    );

    let _ = ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;

            let expand_icon = if *open {
                "chevron-down"
            } else {
                "chevron-right"
            };
            let _ = icon(
                ui,
                IconProps::new(expand_icon)
                    .size(12.0)
                    .tint(tokens::TEXT_MUTED),
            );

            if let Some(leading_icon) = props.leading_icon {
                let _ = icon(
                    ui,
                    IconProps::new(leading_icon)
                        .size(13.0)
                        .tint(props.leading_icon_tint),
                );
            }

            let _ = label(
                ui,
                LabelProps::new(props.title)
                    .tone(LabelTone::Primary)
                    .weight(LabelWeight::Semibold),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some(trailing_icon) = props.trailing_icon {
                    let _ = icon(
                        ui,
                        IconProps::new(trailing_icon)
                            .size(13.0)
                            .tint(props.trailing_icon_tint),
                    );
                }
            });
        },
    );

    if *open {
        ui.add_space(6.0);
        add(ui);
    }

    header_response
}
