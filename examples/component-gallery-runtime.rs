use egui::ViewportBuilder;
use egui_component::demos::component_gallery_runtime;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(component_gallery_runtime::WINDOW_TITLE)
            .with_inner_size(component_gallery_runtime::WINDOW_INNER_SIZE),
        ..Default::default()
    };

    eframe::run_native(
        component_gallery_runtime::WINDOW_TITLE,
        options,
        Box::new(|creation_context| {
            component_gallery_runtime::install_context(&creation_context.egui_ctx);
            Ok(Box::<ComponentGalleryRuntimeWrapper>::default())
        }),
    )
}

#[derive(Default)]
struct ComponentGalleryRuntimeWrapper {
    state: component_gallery_runtime::ComponentGalleryRuntimeApp,
}

impl eframe::App for ComponentGalleryRuntimeWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        component_gallery_runtime::update(&mut self.state, ctx);
    }
}
