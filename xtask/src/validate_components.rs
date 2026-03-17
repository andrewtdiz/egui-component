use crate::support;
use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

const REQUIRED_MARKERS: &[(&str, &str, &str)] = &[
    (
        "src/components/mod.rs",
        "// xtask:component-modules:start",
        "// xtask:component-modules:end",
    ),
    (
        "src/components/mod.rs",
        "// xtask:component-exports:start",
        "// xtask:component-exports:end",
    ),
    (
        "src/lib.rs",
        "// xtask:prelude-exports:start",
        "// xtask:prelude-exports:end",
    ),
    (
        "src/catalog.rs",
        "// xtask:component-kinds:start",
        "// xtask:component-kinds:end",
    ),
    (
        "src/catalog.rs",
        "// xtask:component-definitions:start",
        "// xtask:component-definitions:end",
    ),
    (
        "src/dev/showcase/component_showcase.rs",
        "// xtask:showcase-metadata:start",
        "// xtask:showcase-metadata:end",
    ),
    (
        "src/dev/showcase/component_showcase.rs",
        "// xtask:showcase-render-arms:start",
        "// xtask:showcase-render-arms:end",
    ),
];

const LAYOUT_STATE_ALLOWLIST: &[&str] = &[
    "src/components/api.rs",
    "src/components/button.rs",
    "src/components/combobox.rs",
    "src/components/dialogue.rs",
    "src/components/field.rs",
    "src/components/image.rs",
    "src/components/image_tile.rs",
    "src/components/input.rs",
    "src/components/slider.rs",
    "src/components/tooltip.rs",
];

const DISALLOWED_LAYOUT_PATTERNS: &[&str] = &[
    "spacing_mut().item_spacing",
    "style_mut().spacing",
    "ui.spacing().interact_size",
    "ui.spacing().button_padding",
];

pub fn run(repo_root: &Path) -> Result<()> {
    for (path, start_marker, end_marker) in REQUIRED_MARKERS {
        support::ensure_markers(&repo_root.join(path), start_marker, end_marker)?;
    }

    validate_no_raw_colors(repo_root)?;
    validate_no_implicit_layout_state(repo_root)?;
    validate_registry_coverage(repo_root)?;
    support::run_checked(repo_root, "cargo", &["test", "--no-run"])?;
    support::run_checked(
        repo_root,
        "cargo",
        &["test", "--features", "showcase", "--no-run"],
    )?;
    Ok(())
}

fn validate_no_implicit_layout_state(repo_root: &Path) -> Result<()> {
    let components_dir = repo_root.join("src/components");
    let mut offenders = Vec::new();

    for entry in fs::read_dir(&components_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let relative_path = path
            .strip_prefix(repo_root)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        if LAYOUT_STATE_ALLOWLIST
            .iter()
            .any(|allowed| *allowed == relative_path)
        {
            continue;
        }

        let contents = support::read(&path)?;
        let runtime_only = contents
            .split("#[cfg(test)]")
            .next()
            .unwrap_or(contents.as_str());

        for pattern in DISALLOWED_LAYOUT_PATTERNS {
            if runtime_only.contains(pattern) {
                offenders.push(format!("{relative_path}: {pattern}"));
            }
        }
    }

    if offenders.is_empty() {
        Ok(())
    } else {
        bail!(
            "implicit layout state usage found outside the allowlist: {}",
            offenders.join(", ")
        )
    }
}

fn validate_no_raw_colors(repo_root: &Path) -> Result<()> {
    let components_dir = repo_root.join("src/components");
    let mut offenders = Vec::new();

    for entry in fs::read_dir(&components_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let contents = support::read(&path)?;
        let runtime_only = contents
            .split("#[cfg(test)]")
            .next()
            .unwrap_or(contents.as_str());
        if runtime_only.contains("Color32::from_") {
            offenders.push(path);
        }
    }

    if offenders.is_empty() {
        Ok(())
    } else {
        bail!(
            "raw Color32 constructors found in runtime component code: {}",
            offenders
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn validate_registry_coverage(repo_root: &Path) -> Result<()> {
    let definitions = support::parse_catalog_definitions(repo_root)?;
    let showcase = support::read(&repo_root.join("src/dev/showcase/component_showcase.rs"))?;

    for definition in &definitions {
        support::require_file(&support::component_stub_path(repo_root, &definition.id))?;
        support::parse_doc_stub(&support::component_stub_path(repo_root, &definition.id))?;

        let metadata_needle = format!("kind: ComponentKind::{}", definition.kind);
        if !showcase.contains(&metadata_needle) {
            bail!("showcase metadata missing {}", definition.kind);
        }

        let render_needle = format!("ComponentKind::{} =>", definition.kind);
        if !showcase.contains(&render_needle) {
            bail!("showcase render arm missing {}", definition.kind);
        }
    }

    Ok(())
}
