use egui::{Id, Ui};

#[derive(Debug, Clone, Copy)]
pub struct ScrollAreaProps {
    pub id: Id,
    pub max_height: Option<f32>,
    pub auto_shrink: [bool; 2],
}

impl ScrollAreaProps {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            max_height: None,
            auto_shrink: [false, false],
        }
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    pub fn auto_shrink(mut self, auto_shrink: [bool; 2]) -> Self {
        self.auto_shrink = auto_shrink;
        self
    }
}

pub fn scroll_area<R>(ui: &mut Ui, props: ScrollAreaProps, add: impl FnOnce(&mut Ui) -> R) {
    let mut area = egui::ScrollArea::vertical()
        .id_salt(props.id)
        .auto_shrink(props.auto_shrink);
    if let Some(max_height) = props.max_height {
        area = area.max_height(max_height);
    }
    let _ = area.show(ui, add);
}
