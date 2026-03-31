use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const ICON_FAMILIES: [(&str, &str); 2] = [("Lucide", "lucide"), ("Bootstrap", "bootstrap")];

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let icons_dir = manifest_dir.join("assets").join("icons");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let embedded_icons_path = out_dir.join("embedded_icons.rs");

    println!("cargo:rerun-if-changed={}", icons_dir.display());

    let mut icons = Vec::new();
    for (family_variant, subdirectory) in ICON_FAMILIES {
        let family_dir = icons_dir.join(subdirectory);
        println!("cargo:rerun-if-changed={}", family_dir.display());
        collect_icons(
            manifest_dir.as_path(),
            family_dir.as_path(),
            family_variant,
            &mut icons,
        )?;
    }

    icons.sort_by(|left, right| {
        left.family_variant
            .cmp(right.family_variant)
            .then_with(|| left.name.cmp(&right.name))
    });

    fs::write(embedded_icons_path, render_embedded_icons(icons.as_slice()))?;

    Ok(())
}

struct IconAsset {
    family_variant: &'static str,
    name: String,
    relative_path: String,
}

fn collect_icons(
    manifest_dir: &Path,
    directory: &Path,
    family_variant: &'static str,
    icons: &mut Vec<IconAsset>,
) -> Result<(), Box<dyn Error>> {
    let mut entries = fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();

    for path in entries {
        if path.is_dir() {
            collect_icons(manifest_dir, path.as_path(), family_variant, icons)?;
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) != Some("svg") {
            continue;
        }

        println!("cargo:rerun-if-changed={}", path.display());

        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };

        let relative_path = path
            .strip_prefix(manifest_dir)?
            .to_string_lossy()
            .replace('\\', "/");

        icons.push(IconAsset {
            family_variant,
            name: stem.to_owned(),
            relative_path,
        });
    }

    Ok(())
}

fn render_embedded_icons(icons: &[IconAsset]) -> String {
    let mut output = String::from(
        "pub(super) fn svg_source(family: super::IconFamily, name: &str) -> Option<&'static str> {\n",
    );
    output.push_str("    match (family, name) {\n");

    for icon in icons {
        let include_path = format!("/{}", icon.relative_path);
        output.push_str(
            format!(
                "        (super::IconFamily::{}, {:?}) => Some(include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), {:?}))),\n",
                icon.family_variant, icon.name, include_path
            )
            .as_str(),
        );
    }

    output.push_str("        _ => None,\n");
    output.push_str("    }\n");
    output.push_str("}\n");
    output
}
