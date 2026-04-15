#[path = "../host/mod.rs"]
mod host;

fn main() -> eframe::Result {
    host::run_from_args()
}
