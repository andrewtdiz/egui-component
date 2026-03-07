use super::{api::ComponentUi, LabelTone, LabelWeight};
use crate::ui::tokens;
use egui::{Align, Color32, CornerRadius, Id, Layout, Response, Sense, StrokeKind};

#[derive(Debug, Clone, Copy)]
pub struct Collapsible<'a> {
    pub id: Id,
    pub title: &'a str,
    pub open: bool,
    pub leading_icon: Option<&'a str>,
    pub trailing_icon: Option<&'a str>,
    pub leading_icon_tint: Option<Color32>,
    pub trailing_icon_tint: Option<Color32>,
}

impl<'a> Collapsible<'a> {
    pub fn new(id: Id, title: &'a str) -> Self {
        Self {
            id,
            title,
            open: true,
            leading_icon: None,
            trailing_icon: None,
            leading_icon_tint: None,
            trailing_icon_tint: None,
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
        self.leading_icon_tint = Some(tint);
        self
    }

    pub fn trailing_icon_tint(mut self, tint: Color32) -> Self {
        self.trailing_icon_tint = Some(tint);
        self
    }
}

impl<'a> From<(Id, &'a str)> for Collapsible<'a> {
    fn from((id, title): (Id, &'a str)) -> Self {
        Self::new(id, title)
    }
}

impl<'a> From<(Id, &'a str, bool)> for Collapsible<'a> {
    fn from((id, title, open): (Id, &'a str, bool)) -> Self {
        Self::new(id, title).open(open)
    }
}

impl<'a> From<(Id, &'a str, bool, &'a str, &'a str)> for Collapsible<'a> {
    fn from(
        (id, title, open, leading_icon, trailing_icon): (Id, &'a str, bool, &'a str, &'a str),
    ) -> Self {
        Self::new(id, title)
            .open(open)
            .leading_icon(leading_icon)
            .trailing_icon(trailing_icon)
    }
}

impl ComponentUi<'_> {
    pub fn collapsible<'a, R>(
        &mut self,
        open: &mut bool,
        props: impl Into<Collapsible<'a>>,
        add: impl FnOnce(&mut ComponentUi<'_>) -> R,
    ) -> Response {
        draw_collapsible(self, open, props.into(), add)
    }
}

fn draw_collapsible<R>(
    ui: &mut ComponentUi<'_>,
    open: &mut bool,
    props: Collapsible<'_>,
    add: impl FnOnce(&mut ComponentUi<'_>) -> R,
) -> Response {
    if *open != props.open {
        *open = props.open;
    }
    let dark_mode = ui.visuals().dark_mode;

    let header_height = (ui.spacing().interact_size.y - 2.0).max(30.0);
    let desired_size = egui::vec2(ui.available_width(), header_height);
    let (rect, mut header_response) = ui.allocate_exact_size(desired_size, Sense::click());

    if header_response.clicked() {
        *open = !*open;
        header_response.mark_changed();
    }

    let fill = if header_response.is_pointer_button_down_on() {
        tokens::button_secondary_active_bg(dark_mode)
    } else if header_response.hovered() {
        tokens::button_secondary_hover_bg(dark_mode)
    } else {
        tokens::TRANSPARENT
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
            let _ = ui.icon((expand_icon, 12.0, tokens::text_muted(dark_mode)));

            if let Some(leading_icon) = props.leading_icon {
                let _ = ui.icon((
                    leading_icon,
                    13.0,
                    props
                        .leading_icon_tint
                        .unwrap_or(tokens::text_secondary(dark_mode)),
                ));
            }

            let _ = ui.label((props.title, LabelTone::Primary, LabelWeight::Semibold));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some(trailing_icon) = props.trailing_icon {
                    let _ = ui.icon((
                        trailing_icon,
                        13.0,
                        props
                            .trailing_icon_tint
                            .unwrap_or(tokens::text_muted(dark_mode)),
                    ));
                }
            });
        },
    );

    if *open {
        ui.add_space(6.0);
        let _ = add(ui);
    }

    header_response
}
