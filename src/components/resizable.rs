use egui::{Id, Ui, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct ResizableProps {
    pub id: Id,
    pub default_size: Vec2,
    pub min_size: Vec2,
    pub max_size: Vec2,
}

impl ResizableProps {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            default_size: egui::vec2(228.0, 108.0),
            min_size: egui::vec2(168.0, 78.0),
            max_size: egui::vec2(320.0, 180.0),
        }
    }

    pub fn default_size(mut self, default_size: Vec2) -> Self {
        self.default_size = default_size;
        self
    }

    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    pub fn max_size(mut self, max_size: Vec2) -> Self {
        self.max_size = max_size;
        self
    }
}

pub fn resizable<R>(ui: &mut Ui, props: ResizableProps, add: impl FnOnce(&mut Ui) -> R) {
    let _ = egui::Resize::default()
        .id_salt(props.id)
        .default_size(props.default_size)
        .min_size(props.min_size)
        .max_size(props.max_size)
        .show(ui, add);
}
