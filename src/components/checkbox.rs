use super::api::ComponentUi;
use crate::ui::tokens;
use egui::{CornerRadius, CursorIcon, Response, RichText, Sense, Stroke, StrokeKind, Ui};

const CHECKBOX_CONTROL_SIZE: f32 = 16.0;
const CHECKBOX_CORNER_RADIUS: u8 = 4;

#[derive(Debug, Clone, Copy)]
pub struct Checkbox<'a> {
    pub label: Option<&'a str>,
}

impl<'a> Checkbox<'a> {
    pub fn new() -> Self {
        Self { label: None }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

impl<'a> Default for Checkbox<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<()> for Checkbox<'a> {
    fn from(_: ()) -> Self {
        Self::new()
    }
}

impl<'a> From<&'a str> for Checkbox<'a> {
    fn from(label: &'a str) -> Self {
        Self::new().label(label)
    }
}

impl ComponentUi<'_> {
    pub fn checkbox<'a>(&mut self, value: &mut bool, props: impl Into<Checkbox<'a>>) -> Response {
        draw_checkbox(self.raw_mut(), value, props.into())
    }
}

fn draw_checkbox(ui: &mut Ui, value: &mut bool, props: Checkbox<'_>) -> Response {
    let dark_mode = ui.visuals().dark_mode;
    match props.label {
        Some(label_text) => {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let mut control =
                    draw_checkbox_control(ui, value).on_hover_cursor(CursorIcon::PointingHand);
                let label = ui
                    .scope(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.add(
                            egui::Label::new(
                                RichText::new(label_text).color(tokens::text_primary(dark_mode)),
                            )
                            .selectable(false)
                            .sense(Sense::click()),
                        )
                    })
                    .inner
                    .on_hover_cursor(CursorIcon::PointingHand);
                if label.clicked() {
                    *value = !*value;
                    control.mark_changed();
                }
                control.union(label)
            })
            .inner
        }
        None => draw_checkbox_control(ui, value).on_hover_cursor(CursorIcon::PointingHand),
    }
}

fn draw_checkbox_control(ui: &mut Ui, value: &mut bool) -> Response {
    let dark_mode = ui.visuals().dark_mode;
    let (rect, mut response) = ui.allocate_exact_size(
        egui::vec2(CHECKBOX_CONTROL_SIZE, CHECKBOX_CONTROL_SIZE),
        Sense::click(),
    );

    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }

    let hovered = response.hovered();
    let pressed = response.is_pointer_button_down_on();
    let focused = response.has_focus();

    let (fill, mut stroke) = if *value {
        let checked_fill = if pressed {
            tokens::primary_active_bg(dark_mode)
        } else if hovered {
            tokens::primary_hover_bg(dark_mode)
        } else {
            tokens::primary_bg(dark_mode)
        };
        (checked_fill, Stroke::new(1.0, checked_fill))
    } else if pressed {
        (
            tokens::input_focus_background(dark_mode),
            Stroke::new(1.0, tokens::input_hover_border(dark_mode)),
        )
    } else if hovered {
        (
            tokens::input_hover_background(dark_mode),
            Stroke::new(1.0, tokens::input_hover_border(dark_mode)),
        )
    } else {
        (
            tokens::input_background(dark_mode),
            Stroke::new(1.0, tokens::input_border(dark_mode)),
        )
    };

    if focused {
        stroke = tokens::input_focus_stroke(dark_mode);
    }

    ui.painter().rect(
        rect,
        CornerRadius::same(CHECKBOX_CORNER_RADIUS),
        fill,
        stroke,
        StrokeKind::Outside,
    );

    if *value {
        let check_stroke = Stroke::new(1.8, tokens::primary_fg(dark_mode));
        let start = egui::pos2(rect.left() + rect.width() * 0.24, rect.center().y + 0.2);
        let middle = egui::pos2(
            rect.left() + rect.width() * 0.44,
            rect.bottom() - rect.height() * 0.28,
        );
        let end = egui::pos2(
            rect.right() - rect.width() * 0.22,
            rect.top() + rect.height() * 0.28,
        );
        ui.painter().line_segment([start, middle], check_stroke);
        ui.painter().line_segment([middle, end], check_stroke);
    }

    response
}
