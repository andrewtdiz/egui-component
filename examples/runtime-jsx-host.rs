#[path = "runtime-jsx/mod.rs"]
mod runtime_jsx;

fn main() -> eframe::Result {
    runtime_jsx::run_from_args()
}
