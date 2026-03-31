pub use crate::internal_taffy::{
    bg, taffy, tid, tui, virtual_tui, widgets, AsTuiBuilder, TaffyContainerUi, Tui, TuiBuilder,
    TuiBuilderLogic, TuiBuilderParamsAccess, TuiContainerResponse, TuiId, TuiInnerResponse,
    TuiWidget,
};

#[cfg(test)]
mod tests {
    use crate::layout::{taffy, tid, tui, TuiBuilderLogic};
    use egui::{CentralPanel, Context, RawInput, Rect};

    #[test]
    fn vendored_taffy_primitives_are_available_from_layout_module() {
        let context = Context::default();
        let mut child_rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = ui.scope(|ui| {
                    ui.set_width(160.0);
                    tui(ui, egui::Id::new("layout_public_taffy_smoke"))
                        .reserve_available_width()
                        .style(taffy::Style {
                            size: taffy::Size {
                                width: taffy::prelude::percent(1.0),
                                height: taffy::prelude::auto(),
                            },
                            ..Default::default()
                        })
                        .show(|tui| {
                            tui.id(tid("child"))
                                .style(taffy::Style {
                                    size: taffy::Size {
                                        width: taffy::prelude::length(48.0),
                                        height: taffy::prelude::length(20.0),
                                    },
                                    ..Default::default()
                                })
                                .ui(|ui| {
                                    child_rect = ui.max_rect();
                                });
                        });
                });
            });
        });

        assert_eq!(child_rect.width(), 48.0);
        assert_eq!(child_rect.height(), 20.0);
    }
}
