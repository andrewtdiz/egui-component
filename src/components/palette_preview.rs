use super::api::ComponentUi;
use egui::{Color32, Rect, Response, Sense, Stroke};

#[derive(Debug, Clone, Copy)]
pub struct PalettePreview<'a> {
    pub colors: &'a [Color32],
    pub width: f32,
    pub row_size: usize,
    pub max_rows: usize,
    pub row_height: f32,
}

impl<'a> PalettePreview<'a> {
    pub fn new(colors: &'a [Color32]) -> Self {
        Self {
            colors,
            width: 220.0,
            row_size: 8,
            max_rows: 4,
            row_height: 12.0,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(1.0);
        self
    }

    pub fn row_size(mut self, row_size: usize) -> Self {
        self.row_size = row_size.max(1);
        self
    }

    pub fn max_rows(mut self, max_rows: usize) -> Self {
        self.max_rows = max_rows.max(1);
        self
    }

    pub fn row_height(mut self, row_height: f32) -> Self {
        self.row_height = row_height.max(1.0);
        self
    }
}

impl ComponentUi<'_> {
    pub fn palette_preview<'a>(&mut self, props: PalettePreview<'a>) -> Response {
        draw_palette_preview(self.ui_mut(), props)
    }
}

fn draw_palette_preview(ui: &mut egui::Ui, props: PalettePreview<'_>) -> Response {
    let row_count = props
        .colors
        .len()
        .div_ceil(props.row_size)
        .min(props.max_rows)
        .max(1);
    let size = egui::vec2(props.width, props.row_height * row_count as f32);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let cell_width = rect.width() / props.row_size as f32;
    let stroke = Stroke::new(1.0, ui.visuals().widgets.inactive.bg_stroke.color);

    for (index, color) in props
        .colors
        .iter()
        .copied()
        .take(props.row_size * props.max_rows)
        .enumerate()
    {
        let row = index / props.row_size;
        let column = index % props.row_size;
        let min = egui::pos2(
            rect.left() + column as f32 * cell_width,
            rect.top() + row as f32 * props.row_height,
        );
        let max = egui::pos2(min.x + cell_width, min.y + props.row_height);
        ui.painter().rect(
            Rect::from_min_max(min, max),
            egui::CornerRadius::same(0),
            color,
            stroke,
            egui::StrokeKind::Inside,
        );
    }

    response
}
