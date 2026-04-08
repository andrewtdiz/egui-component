use egui::ViewportBuilder;
use egui_component::demos::showcase_runtime;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(showcase_runtime::WINDOW_TITLE)
            .with_inner_size(showcase_runtime::WINDOW_INNER_SIZE),
        ..Default::default()
    };

    eframe::run_native(
        showcase_runtime::WINDOW_TITLE,
        options,
        Box::new(|creation_context| {
            showcase_runtime::install_context(&creation_context.egui_ctx);
            Ok(Box::<ShowcaseRuntimeWrapper>::default())
        }),
    )
}

#[derive(Default)]
struct ShowcaseRuntimeWrapper {
    state: showcase_runtime::ShowcaseRuntimeApp,
}

impl eframe::App for ShowcaseRuntimeWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        showcase_runtime::update(&mut self.state, ctx);
    }
}
