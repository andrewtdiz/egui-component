mod component_run_options;
mod component_showcase;
mod headless;
mod style;
mod theme_snapshot;
mod views;

use std::path::PathBuf;

use crate::Result;
use component_run_options::ComponentRunOptions;

const SNAPSHOT_PATH: &str = "ui/editorui.theme.ron";

pub(crate) fn run_components() -> Result {
    let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SNAPSHOT_PATH);
    views::run_components_window(snapshot_path)
}

pub(crate) fn run_component(args: &[String]) -> Result {
    let options = ComponentRunOptions::parse(args)?;
    if options.headless {
        headless::run_single_component_headless(options)
    } else {
        views::run_single_component_window(options)
    }
}
