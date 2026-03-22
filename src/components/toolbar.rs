use super::api::{with_component_overrides, ComponentUi};
use crate::layout;
use crate::primitives::surface::{surface_frame, SurfaceFrame};
use crate::ui::tokens;
use egui::{Align2, Color32, Id, Order, Pos2, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct Toolbar {
    pub id: Id,
    pub anchor: Align2,
    pub offset: Vec2,
    pub fill: Option<Color32>,
    pub stroke: Option<Stroke>,
    pub corner_radius: Option<u8>,
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
            corner_radius: None,
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
        self.corner_radius = Some(corner_radius);
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
        add: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let overrides = self.overrides();
        draw_toolbar(self.ui_mut(), props.into(), |ui| {
            with_component_overrides(ui, overrides, add)
        })
    }
}

fn draw_toolbar<R>(
    ui: &mut Ui,
    props: Toolbar,
    add: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    let runtime = crate::theme::runtime_for_ui(ui);
    let parent_rect = ui.max_rect();
    let anchor_pos = anchored_pos(parent_rect, props.anchor) + props.offset;
    let area_id = props.id.with("area");

    egui::Area::new(area_id)
        .order(Order::Foreground)
        .pivot(props.anchor)
        .fixed_pos(anchor_pos)
        .constrain_to(parent_rect)
        .fade_in(false)
        .show(ui.ctx(), |ui| {
            surface_frame(
                ui,
                SurfaceFrame::new(
                    props.fill.unwrap_or(tokens::card_background(runtime)),
                    props
                        .stroke
                        .unwrap_or(Stroke::new(1.0, tokens::separator(runtime))),
                )
                .corner_radius(
                    props
                        .corner_radius
                        .unwrap_or(crate::theme::radius(ui, crate::theme::RadiusRole::Xl)),
                )
                .padding(props.padding_x, props.padding_y)
                .shadow(props.shadow.unwrap_or(tokens::tailwind_shadow_sm(runtime))),
                |ui| layout::row().gap(tokens::LAYOUT_GAP_XS).show(ui, add).inner,
            )
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
    use crate::components::{Button, ButtonVariant, ComponentUiExt};
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
                            let mut ui = ui.components();
                            let _ = ui.button(Button::new("Edit").variant(ButtonVariant::Ghost));
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
