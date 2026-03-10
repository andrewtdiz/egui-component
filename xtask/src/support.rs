use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CatalogGroup {
    Primitive,
    Composed,
}

impl CatalogGroup {
    pub fn title(self) -> &'static str {
        match self {
            Self::Primitive => "Primitive",
            Self::Composed => "Composed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CatalogDefinition {
    pub kind: String,
    pub id: String,
    pub label: String,
    pub group: CatalogGroup,
}

#[derive(Debug, Clone)]
pub struct DocStub {
    pub id: String,
    pub label: String,
    pub family: String,
    pub summary: String,
    pub path: PathBuf,
}

pub fn repo_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir().context("failed to resolve current directory")?;
    loop {
        if current.join("Cargo.toml").is_file() && current.join("src/lib.rs").is_file() {
            return Ok(current);
        }
        if !current.pop() {
            bail!("failed to locate repo root");
        }
    }
}

pub fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

pub fn write(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}

pub fn write_if_changed(path: &Path, contents: &str) -> Result<bool> {
    match fs::read_to_string(path) {
        Ok(existing) if existing == contents => Ok(false),
        Ok(_) | Err(_) => {
            write(path, contents)?;
            Ok(true)
        }
    }
}

pub fn insert_before_marker(path: &Path, end_marker: &str, entry: &str) -> Result<bool> {
    let mut contents = read(path)?;
    if contents.contains(entry.trim()) {
        return Ok(false);
    }

    let Some(index) = contents.find(end_marker) else {
        bail!("missing marker `{end_marker}` in {}", path.display());
    };

    contents.insert_str(index, entry);
    write(path, &contents)?;
    Ok(true)
}

pub fn ensure_markers(path: &Path, start_marker: &str, end_marker: &str) -> Result<()> {
    let contents = read(path)?;
    if !contents.contains(start_marker) || !contents.contains(end_marker) {
        bail!(
            "missing marker pair `{start_marker}` / `{end_marker}` in {}",
            path.display()
        );
    }
    Ok(())
}

pub fn run_checked(repo_root: &Path, program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .current_dir(repo_root)
        .status()
        .with_context(|| format!("failed to run `{program} {}`", args.join(" ")))?;

    if status.success() {
        Ok(())
    } else {
        bail!("`{program} {}` exited with {status}", args.join(" "));
    }
}

pub fn parse_catalog_definitions(repo_root: &Path) -> Result<Vec<CatalogDefinition>> {
    let path = repo_root.join("src/catalog.rs");
    let contents = read(&path)?;
    let mut definitions = Vec::new();
    let mut current_kind = None::<String>;
    let mut current_id = None::<String>;
    let mut current_label = None::<String>;
    let mut current_group = None::<CatalogGroup>;
    let mut in_definition = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed == "ComponentDefinition {" {
            in_definition = true;
            current_kind = None;
            current_id = None;
            current_label = None;
            current_group = None;
            continue;
        }
        if !in_definition {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("kind: ComponentKind::") {
            current_kind = Some(rest.trim_end_matches(',').to_owned());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("id: \"") {
            current_id = Some(rest.trim_end_matches("\",").to_owned());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("label: \"") {
            current_label = Some(rest.trim_end_matches("\",").to_owned());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("group: ComponentGroup::") {
            current_group = Some(match rest.trim_end_matches(',') {
                "Primitive" => CatalogGroup::Primitive,
                "Composed" => CatalogGroup::Composed,
                other => bail!("unknown component group `{other}`"),
            });
            continue;
        }
        if trimmed == "}," {
            if let (Some(kind), Some(id), Some(label), Some(group)) = (
                current_kind.take(),
                current_id.take(),
                current_label.take(),
                current_group.take(),
            ) {
                definitions.push(CatalogDefinition {
                    kind,
                    id,
                    label,
                    group,
                });
            }
            in_definition = false;
        }
    }

    if definitions.is_empty() {
        bail!(
            "failed to parse component definitions from {}",
            path.display()
        );
    }

    Ok(definitions)
}

pub fn parse_doc_stub(path: &Path) -> Result<DocStub> {
    let contents = read(path)?;
    let mut id = None::<String>;
    let mut label = None::<String>;
    let mut family = None::<String>;
    let mut summary = None::<String>;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let value = value.trim().to_owned();
            match key.trim() {
                "id" => id = Some(value),
                "label" => label = Some(value),
                "family" => family = Some(value),
                "summary" => summary = Some(value),
                _ => {}
            }
        }
    }

    Ok(DocStub {
        id: id.ok_or_else(|| anyhow!("missing `id` header in {}", path.display()))?,
        label: label.ok_or_else(|| anyhow!("missing `label` header in {}", path.display()))?,
        family: family.ok_or_else(|| anyhow!("missing `family` header in {}", path.display()))?,
        summary: summary
            .ok_or_else(|| anyhow!("missing `summary` header in {}", path.display()))?,
        path: path.to_path_buf(),
    })
}

pub fn component_stub_path(repo_root: &Path, id: &str) -> PathBuf {
    repo_root
        .join("docs/llm/components")
        .join(format!("{id}.md"))
}

pub fn default_family(definition: &CatalogDefinition) -> &'static str {
    match definition.id.as_str() {
        "label" | "color" | "image" | "icon" | "kbd" => "display",
        "select" | "tabs" | "button-group" | "menu-bar" | "collapsible" => "row-list",
        "tooltip" | "dropdown-menu" => "popup",
        _ if definition.group == CatalogGroup::Composed => "composed",
        _ => "control",
    }
}

pub fn default_summary(definition: &CatalogDefinition) -> String {
    format!("{} authoring notes.", definition.label)
}

pub fn render_template(template: &str, replacements: &[(&str, &str)]) -> String {
    replacements
        .iter()
        .fold(template.to_owned(), |output, (key, value)| {
            output.replace(key, value)
        })
}

pub fn split_words(input: &str) -> Vec<String> {
    let mut normalized = String::with_capacity(input.len() * 2);
    let mut previous_is_lower = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && previous_is_lower {
                normalized.push(' ');
            }
            normalized.push(ch.to_ascii_lowercase());
            previous_is_lower = ch.is_ascii_lowercase();
        } else {
            normalized.push(' ');
            previous_is_lower = false;
        }
    }

    normalized
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect()
}

pub fn pascal_case(words: &[String]) -> String {
    words
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

pub fn title_case(words: &[String]) -> String {
    words
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn require_file(path: &Path) -> Result<()> {
    if path.is_file() {
        Ok(())
    } else {
        bail!("missing {}", path.display())
    }
}
