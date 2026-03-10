mod new_component;
mod support;
mod sync_llm_docs;
mod validate_components;

use anyhow::{bail, Result};

fn main() -> Result<()> {
    let repo_root = support::repo_root()?;
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let Some((command, rest)) = args.split_first() else {
        bail!("usage: cargo xtask <new-component|validate-components|sync-llm-docs> [args]");
    };

    match command.as_str() {
        "new-component" => new_component::run(&repo_root, rest),
        "validate-components" => validate_components::run(&repo_root),
        "sync-llm-docs" => sync_llm_docs::run(&repo_root),
        other => bail!("unknown xtask command: {other}"),
    }
}
