use egui::{InnerResponse, Ui};

#[allow(dead_code)]
pub(crate) fn stack<R>(
    ui: &mut Ui,
    spacing_y: Option<f32>,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    ui.scope(|ui| {
        if let Some(spacing_y) = spacing_y {
            ui.spacing_mut().item_spacing.y = spacing_y;
        }
        ui.vertical(add).inner
    })
}

#[allow(dead_code)]
pub(crate) fn inline<R>(
    ui: &mut Ui,
    spacing_x: Option<f32>,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    ui.scope(|ui| {
        if let Some(spacing_x) = spacing_x {
            ui.spacing_mut().item_spacing.x = spacing_x;
        }
        ui.horizontal(add).inner
    })
}
