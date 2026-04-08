use egui::ViewportBuilder;
use egui_component::demos::runtime_egui_host;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(runtime_egui_host::WINDOW_TITLE)
            .with_inner_size(runtime_egui_host::WINDOW_INNER_SIZE),
        ..Default::default()
    };

    eframe::run_native(
        runtime_egui_host::WINDOW_TITLE,
        options,
        Box::new(|creation_context| {
            runtime_egui_host::install_context(&creation_context.egui_ctx);
            Ok(Box::<RuntimeEguiHostWrapper>::default())
        }),
    )
}

#[derive(Default)]
struct RuntimeEguiHostWrapper {
    state: runtime_egui_host::RuntimeEguiHostApp,
}

impl eframe::App for RuntimeEguiHostWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        runtime_egui_host::update(&mut self.state, ctx);
    }
}
