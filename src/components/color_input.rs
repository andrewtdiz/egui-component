use super::api::ComponentUi;
use egui::{Color32, Id, Response, Ui};

#[derive(Debug, Clone, Copy, Default)]
pub struct ColorInput {
    pub id: Option<Id>,
}

impl ColorInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }
}

impl ComponentUi<'_> {
    pub fn color_input(&mut self, value: &mut Color32, props: impl Into<ColorInput>) -> Response {
        draw_color_input(self.raw_mut(), value, props.into())
    }
}

fn draw_color_input(ui: &mut Ui, value: &mut Color32, props: ColorInput) -> Response {
    if let Some(id) = props.id {
        ui.push_id(id, |ui| ui.color_edit_button_srgba(value)).inner
    } else {
        ui.color_edit_button_srgba(value)
    }
}

#[cfg(test)]
mod tests {
    use super::ColorInput;
    use crate::components::ComponentUiExt;
    use egui::{CentralPanel, Color32, Context, RawInput, Rect};

    #[test]
    fn renders_color_input() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let mut color = Color32::from_rgb(34, 197, 94);
                rect = ui
                    .components()
                    .color_input(&mut color, ColorInput::new())
                    .rect;
            });
        });

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
    }
}
