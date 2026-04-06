use egui::ViewportBuilder;
use egui_component::example_apps::contract_demo;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(contract_demo::WINDOW_TITLE)
            .with_inner_size(contract_demo::WINDOW_INNER_SIZE),
        ..Default::default()
    };

    eframe::run_native(
        contract_demo::WINDOW_TITLE,
        options,
        Box::new(|creation_context| {
            contract_demo::install_context(&creation_context.egui_ctx);
            Ok(Box::<ContractDemoWrapper>::default())
        }),
    )
}

#[derive(Default)]
struct ContractDemoWrapper {
    state: contract_demo::ContractDemoApp,
}

impl eframe::App for ContractDemoWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        contract_demo::update(&mut self.state, ctx);
    }
}
