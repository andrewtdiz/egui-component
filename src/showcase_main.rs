fn main() -> egui_component::Result {
    match std::env::args().nth(1).as_deref() {
        Some("chat") => egui_component::dev::run_chat(),
        _ => egui_component::dev::run_showcase(),
    }
}
