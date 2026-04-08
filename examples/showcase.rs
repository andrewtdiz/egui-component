use egui::ViewportBuilder;
use egui_component::demos::showcase;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(showcase::WINDOW_TITLE)
            .with_inner_size(showcase::WINDOW_INNER_SIZE),
        ..Default::default()
    };

    eframe::run_native(
        showcase::WINDOW_TITLE,
        options,
        Box::new(|creation_context| {
            showcase::install_context(&creation_context.egui_ctx);
            Ok(Box::<StaticShowcaseApp>::default())
        }),
    )
}

#[derive(Default)]
struct StaticShowcaseApp {
    state: showcase::ShowcaseApp,
}

impl eframe::App for StaticShowcaseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        showcase::update(&mut self.state, ctx);
    }
}
