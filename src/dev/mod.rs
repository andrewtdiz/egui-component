mod runtime;
pub mod showcase;

use crate::Result;

pub fn run_showcase() -> Result {
    runtime::run_showcase_window()
}
