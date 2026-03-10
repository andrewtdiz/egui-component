use crate::support;
use anyhow::{bail, Context, Result};
use std::path::Path;

const MODULES_END: &str = "// xtask:component-modules:end";
const EXPORTS_END: &str = "// xtask:component-exports:end";
const PRELUDE_END: &str = "// xtask:prelude-exports:end";
const KINDS_END: &str = "// xtask:component-kinds:end";
const DEFINITIONS_END: &str = "// xtask:component-definitions:end";
const SHOWCASE_METADATA_END: &str = "// xtask:showcase-metadata:end";
const SHOWCASE_RENDER_END: &str = "// xtask:showcase-render-arms:end";

pub fn run(repo_root: &Path, args: &[String]) -> Result<()> {
    let (name, family) = parse_args(args)?;
    let words = support::split_words(&name);
    if words.is_empty() {
        bail!("component name must contain at least one alphanumeric character");
    }

    let module_name = words.join("_");
    let component_id = words.join("-");
    let type_name = support::pascal_case(&words);
    let label = support::title_case(&words);
    let method_name = module_name.clone();
    let component_path = repo_root
        .join("src/components")
        .join(format!("{module_name}.rs"));

    if component_path.exists() {
        bail!(
            "component file already exists: {}",
            component_path.display()
        );
    }

    let definitions = support::parse_catalog_definitions(repo_root)?;
    if definitions
        .iter()
        .any(|definition| definition.id == component_id)
    {
        bail!("component id already exists in catalog: {component_id}");
    }

    let family_template = format!("family-{family}.md");
    let summary = format!("{label} component preview stub.");

    support::insert_before_marker(
        &repo_root.join("src/components/mod.rs"),
        MODULES_END,
        &format!("mod {module_name};\n"),
    )?;
    support::insert_before_marker(
        &repo_root.join("src/components/mod.rs"),
        EXPORTS_END,
        &format!("pub use {module_name}::{type_name};\n"),
    )?;
    support::insert_before_marker(
        &repo_root.join("src/lib.rs"),
        PRELUDE_END,
        &format!("        {type_name},\n"),
    )?;
    support::insert_before_marker(
        &repo_root.join("src/catalog.rs"),
        KINDS_END,
        &format!("    {type_name},\n"),
    )?;
    support::insert_before_marker(
        &repo_root.join("src/catalog.rs"),
        DEFINITIONS_END,
        &format!(
            "    ComponentDefinition {{\n        kind: ComponentKind::{type_name},\n        id: \"{component_id}\",\n        label: \"{label}\",\n        group: ComponentGroup::{},\n    }},\n",
            if family == "composed" {
                "Composed"
            } else {
                "Primitive"
            }
        ),
    )?;
    support::insert_before_marker(
        &repo_root.join("src/dev/showcase/component_showcase.rs"),
        SHOWCASE_METADATA_END,
        &format!(
            "    ShowcaseMetadata {{\n        kind: ComponentKind::{type_name},\n        description: \"{summary}\",\n        section_override: {},\n    }},\n",
            if family == "composed" {
                "None"
            } else {
                "None"
            }
        ),
    )?;
    support::insert_before_marker(
        &repo_root.join("src/dev/showcase/component_showcase.rs"),
        SHOWCASE_RENDER_END,
        &format!(
            "        ComponentKind::{type_name} => {{\n            let _ = ui.label(\n                Label::new(\"{label} preview pending\")\n                    .tone(LabelTone::Muted)\n                    .size(typography::SMALL_SIZE),\n            );\n        }}\n",
        ),
    )?;

    let component_source = format!(
        "use super::api::ComponentUi;\nuse egui::Response;\n\n#[derive(Debug, Clone, Copy)]\npub struct {type_name}<'a> {{\n    pub label: &'a str,\n}}\n\nimpl<'a> {type_name}<'a> {{\n    pub fn new(label: &'a str) -> Self {{\n        Self {{ label }}\n    }}\n}}\n\nimpl<'a> From<&'a str> for {type_name}<'a> {{\n    fn from(label: &'a str) -> Self {{\n        Self::new(label)\n    }}\n}}\n\nimpl ComponentUi<'_> {{\n    pub fn {method_name}<'a>(&mut self, props: impl Into<{type_name}<'a>>) -> Response {{\n        draw_{module_name}(self.raw_mut(), props.into())\n    }}\n}}\n\nfn draw_{module_name}(ui: &mut egui::Ui, props: {type_name}<'_>) -> Response {{\n    // Compose primitives here instead of painting raw shapes inline.\n    ui.label(props.label)\n}}\n\n#[cfg(test)]\nmod tests {{\n    use super::{type_name};\n    use crate::components::ComponentUiExt;\n    use egui::{{CentralPanel, Context, RawInput}};\n\n    #[test]\n    fn renders_{module_name}_without_panic() {{\n        let context = Context::default();\n        let _ = context.run(RawInput::default(), |context| {{\n            CentralPanel::default().show(context, |ui| {{\n                let _ = ui.components().{method_name}({type_name}::new(\"{label}\"));\n            }});\n        }});\n    }}\n}}\n"
    );
    support::write(&component_path, &component_source)?;

    let template = support::read(&repo_root.join("docs/llm/templates/component.md"))
        .context("missing docs/llm/templates/component.md")?;
    let stub = support::render_template(
        &template,
        &[
            ("{{id}}", &component_id),
            ("{{label}}", &label),
            ("{{family}}", &family),
            ("{{summary}}", &summary),
            ("{{family_template}}", &family_template),
        ],
    );
    support::write(
        &support::component_stub_path(repo_root, &component_id),
        &stub,
    )?;

    crate::sync_llm_docs::run(repo_root)
}

fn parse_args(args: &[String]) -> Result<(String, String)> {
    if args.len() < 3 {
        bail!("usage: cargo xtask new-component <name> --family <display|control|row-list|popup|composed>");
    }

    let name = args[0].clone();
    if args[1] != "--family" {
        bail!("expected `--family` after component name");
    }
    let family = args[2].as_str();
    match family {
        "display" | "control" | "row-list" | "popup" | "composed" => {}
        other => bail!("unknown component family `{other}`"),
    }

    Ok((name, args[2].clone()))
}
