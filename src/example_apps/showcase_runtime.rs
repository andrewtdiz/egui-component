use std::path::PathBuf;

use egui::Context;

use super::{runtime_egui_host, showcase};

pub const WINDOW_TITLE: &str = "egui-component Showcase Runtime";
pub const WINDOW_INNER_SIZE: [f32; 2] = [1360.0, 940.0];

pub struct ShowcaseRuntimeApp {
    host: runtime_egui_host::RuntimeEguiHostApp,
}

impl Default for ShowcaseRuntimeApp {
    fn default() -> Self {
        Self {
            host: runtime_egui_host::RuntimeEguiHostApp::new_presentational(default_script_path()),
        }
    }
}

pub fn install_context(ctx: &Context) {
    showcase::install_context(ctx);
}

pub fn update(app: &mut ShowcaseRuntimeApp, ctx: &Context) {
    runtime_egui_host::update(&mut app.host, ctx);
}

fn default_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("runtime-luau")
        .join("apps")
        .join("showcase.luau")
}
