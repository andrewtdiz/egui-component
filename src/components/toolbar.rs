use super::api::ComponentUi;
use crate::ui::tokens;
use egui::{
    Align, Align2, Color32, CornerRadius, Id, Layout, Margin, Order, Pos2, Stroke, Ui, Vec2,
};

#[derive(Debug, Clone, Copy)]
pub struct Toolbar {
    pub id: Id,
    pub anchor: Align2,
    pub offset: Vec2,
    pub fill: Option<Color32>,
    pub stroke: Option<Stroke>,
    pub corner_radius: u8,
    pub padding_x: i8,
    pub padding_y: i8,
    pub shadow: Option<egui::Shadow>,
}

impl Toolbar {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            anchor: Align2::CENTER_TOP,
            offset: egui::vec2(0.0, 8.0),
            fill: None,
            stroke: None,
            corner_radius: 14,
            padding_x: 8,
            padding_y: 6,
            shadow: None,
        }
    }

    pub fn anchor(mut self, anchor: Align2) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn corner_radius(mut self, corner_radius: u8) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn padding(mut self, x: i8, y: i8) -> Self {
        self.padding_x = x;
        self.padding_y = y;
        self
    }

    pub fn shadow(mut self, shadow: egui::Shadow) -> Self {
        self.shadow = Some(shadow);
        self
    }
}

impl From<Id> for Toolbar {
    fn from(id: Id) -> Self {
        Self::new(id)
    }
}

impl ComponentUi<'_> {
    pub fn toolbar<R>(
        &mut self,
        props: impl Into<Toolbar>,
        add: impl FnOnce(&mut ComponentUi<'_>) -> R,
    ) -> egui::InnerResponse<R> {
        let overrides = self.overrides;
        draw_toolbar(self.raw_mut(), props.into(), |ui| {
            let mut components = ComponentUi::with_overrides(ui, overrides);
            add(&mut components)
        })
    }
}

fn draw_toolbar<R>(
    ui: &mut Ui,
    props: Toolbar,
    add: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    let dark_mode = ui.visuals().dark_mode;
    let parent_rect = ui.max_rect();
    let anchor_pos = anchored_pos(parent_rect, props.anchor) + props.offset;

    egui::Area::new(props.id)
        .order(Order::Foreground)
        .pivot(props.anchor)
        .fixed_pos(anchor_pos)
        .constrain_to(parent_rect)
        .fade_in(false)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(props.fill.unwrap_or(tokens::card_background(dark_mode)))
                .stroke(
                    props
                        .stroke
                        .unwrap_or(Stroke::new(1.0, tokens::separator(dark_mode))),
                )
                .corner_radius(CornerRadius::same(props.corner_radius))
                .inner_margin(Margin::symmetric(props.padding_x, props.padding_y))
                .shadow(props.shadow.unwrap_or(tokens::tailwind_shadow_sm()))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.with_layout(Layout::left_to_right(Align::Center), add)
                        .inner
                })
                .inner
        })
}

fn anchored_pos(rect: egui::Rect, anchor: Align2) -> Pos2 {
    egui::pos2(
        rect.left() + rect.width() * anchor.x().to_factor(),
        rect.top() + rect.height() * anchor.y().to_factor(),
    )
}

#[cfg(test)]
mod tests {
    use super::Toolbar;
    use crate::components::{ButtonStyle, ComponentUiExt};
    use egui::{CentralPanel, Context, Id, RawInput, Rect};

    #[test]
    fn renders_toolbar_inside_parent_area() {
        let context = Context::default();
        let mut rect = Rect::NOTHING;

        for _ in 0..2 {
            let _ = context.run(RawInput::default(), |context| {
                CentralPanel::default().show(context, |ui| {
                    rect = ui
                        .components()
                        .toolbar(Toolbar::new(Id::new("toolbar_test")), |ui| {
                            let _ = ui.button(("Edit", ButtonStyle::Ghost));
                        })
                        .response
                        .rect;
                });
            });
        }

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
    }
}
