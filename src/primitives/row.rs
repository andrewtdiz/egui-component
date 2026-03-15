use crate::icons;
use egui::{
    Align2, Color32, CornerRadius, CursorIcon, FontId, Rect, Response, Sense, Stroke, StrokeKind,
    Ui, Vec2,
};

#[derive(Debug, Clone, Copy)]
pub struct RowChrome {
    size: Vec2,
    corner_radius: u8,
    stroke: Stroke,
    cursor: CursorIcon,
}

impl RowChrome {
    pub fn new(size: Vec2) -> Self {
        Self {
            size,
            corner_radius: 0,
            stroke: Stroke::NONE,
            cursor: CursorIcon::PointingHand,
        }
    }

    pub fn corner_radius(mut self, corner_radius: u8) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    pub fn cursor(mut self, cursor: CursorIcon) -> Self {
        self.cursor = cursor;
        self
    }
}

pub fn row_chrome(
    ui: &mut Ui,
    chrome: RowChrome,
    fill: impl FnOnce(&Response) -> Color32,
) -> (Rect, Response) {
    let (rect, response) = ui.allocate_exact_size(chrome.size, Sense::click());
    paint_row_chrome(
        ui,
        rect,
        fill(&response),
        chrome.corner_radius,
        chrome.stroke,
    );
    (rect, response.on_hover_cursor(chrome.cursor))
}

pub fn paint_row_chrome(ui: &mut Ui, rect: Rect, fill: Color32, corner_radius: u8, stroke: Stroke) {
    ui.painter().rect(
        rect,
        CornerRadius::same(corner_radius),
        fill,
        stroke,
        StrokeKind::Outside,
    );
}

#[derive(Debug, Clone)]
pub struct IconLabelRow<'a> {
    padding_x: f32,
    label: &'a str,
    label_font: FontId,
    label_color: Color32,
    leading_icon: Option<&'a str>,
    leading_icon_size: f32,
    leading_gap: f32,
    trailing_text: Option<&'a str>,
    trailing_text_font: FontId,
    trailing_text_color: Color32,
    trailing_icon: Option<&'a str>,
    trailing_icon_size: f32,
    trailing_icon_tint: Color32,
    trailing_gap: f32,
}

impl<'a> IconLabelRow<'a> {
    pub fn new(label: &'a str, label_font: FontId, label_color: Color32) -> Self {
        Self {
            padding_x: 10.0,
            label,
            label_font,
            label_color,
            leading_icon: None,
            leading_icon_size: 16.0,
            leading_gap: 8.0,
            trailing_text: None,
            trailing_text_font: FontId::default(),
            trailing_text_color: label_color,
            trailing_icon: None,
            trailing_icon_size: 14.0,
            trailing_icon_tint: label_color,
            trailing_gap: 6.0,
        }
    }

    pub fn padding_x(mut self, padding_x: f32) -> Self {
        self.padding_x = padding_x;
        self
    }

    pub fn leading_icon(mut self, leading_icon: &'a str) -> Self {
        self.leading_icon = Some(leading_icon);
        self
    }

    pub fn leading_icon_size(mut self, leading_icon_size: f32) -> Self {
        self.leading_icon_size = leading_icon_size;
        self
    }

    pub fn leading_gap(mut self, leading_gap: f32) -> Self {
        self.leading_gap = leading_gap;
        self
    }

    pub fn trailing_text(
        mut self,
        trailing_text: &'a str,
        trailing_text_font: FontId,
        trailing_text_color: Color32,
    ) -> Self {
        self.trailing_text = Some(trailing_text);
        self.trailing_text_font = trailing_text_font;
        self.trailing_text_color = trailing_text_color;
        self
    }

    pub fn trailing_icon(mut self, trailing_icon: &'a str) -> Self {
        self.trailing_icon = Some(trailing_icon);
        self
    }

    pub fn trailing_icon_size(mut self, trailing_icon_size: f32) -> Self {
        self.trailing_icon_size = trailing_icon_size;
        self
    }

    pub fn trailing_icon_tint(mut self, trailing_icon_tint: Color32) -> Self {
        self.trailing_icon_tint = trailing_icon_tint;
        self
    }

    pub fn trailing_gap(mut self, trailing_gap: f32) -> Self {
        self.trailing_gap = trailing_gap;
        self
    }
}

pub fn icon_label_row(ui: &mut Ui, rect: Rect, row: &IconLabelRow<'_>) -> Option<Rect> {
    let leading_width = if row.leading_icon.is_some() {
        row.leading_icon_size + row.leading_gap
    } else {
        0.0
    };
    let trailing_width = if let Some(trailing_text) = row.trailing_text {
        ui.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(
                    trailing_text.to_owned(),
                    row.trailing_text_font.clone(),
                    row.trailing_text_color,
                )
                .size()
                .x
        })
    } else if row.trailing_icon.is_some() {
        row.trailing_icon_size
    } else {
        0.0
    };
    let trailing_rect = if trailing_width > 0.0 {
        Some(Rect::from_min_max(
            egui::pos2(rect.right() - row.padding_x - trailing_width, rect.top()),
            egui::pos2(rect.right() - row.padding_x, rect.bottom()),
        ))
    } else {
        None
    };
    let label_right = trailing_rect
        .map(|trailing_rect| trailing_rect.left() - row.trailing_gap)
        .unwrap_or(rect.right() - row.padding_x);
    let label_x = rect.left() + row.padding_x + leading_width;

    if let Some(icon_name) = row.leading_icon {
        if let Some(image) = icons::image(ui.ctx(), icon_name, row.leading_icon_size) {
            image.tint(row.label_color).paint_at(
                ui,
                Rect::from_center_size(
                    egui::pos2(
                        rect.left() + row.padding_x + row.leading_icon_size * 0.5,
                        rect.center().y,
                    ),
                    egui::vec2(row.leading_icon_size, row.leading_icon_size),
                ),
            );
        }
    }

    ui.painter().text(
        egui::pos2(label_x.min(label_right), rect.center().y),
        Align2::LEFT_CENTER,
        row.label,
        row.label_font.clone(),
        row.label_color,
    );

    if let (Some(icon_name), Some(trailing_rect)) = (row.trailing_icon, trailing_rect) {
        if let Some(image) = icons::image(ui.ctx(), icon_name, row.trailing_icon_size) {
            image.tint(row.trailing_icon_tint).paint_at(
                ui,
                Rect::from_center_size(
                    trailing_rect.center(),
                    egui::vec2(row.trailing_icon_size, row.trailing_icon_size),
                ),
            );
        }
    }

    trailing_rect
}
