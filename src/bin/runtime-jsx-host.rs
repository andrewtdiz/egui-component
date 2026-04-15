#[path = "../host/mod.rs"]
mod host;

#[cfg(not(test))]
fn main() -> eframe::Result {
    host::run_from_args()
}
