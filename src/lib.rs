mod editor;
mod editorui;
mod error;

pub use error::{ClayError, Result};

pub fn run_components() -> Result {
    editorui::run_components()
}

pub fn run_component_cli() -> Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    editorui::run_component(args.as_slice())
}
