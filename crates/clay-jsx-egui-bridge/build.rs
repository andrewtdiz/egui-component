use std::{
    env, fs,
    path::{Path, PathBuf},
};

use deno_ast::{
    parse_module, EmitOptions, MediaType, ParseParams, SourceMapOption, TranspileModuleOptions,
    TranspileOptions,
};

const VIRTUAL_MODULES: [(&str, &str); 5] = [
    ("mod.ts", "mod.js"),
    ("jsx_runtime_api.ts", "jsx_runtime_api.js"),
    ("runtime_api.ts", "runtime_api.js"),
    ("lowering_api.ts", "lowering_api.js"),
    ("motion_api.ts", "motion_api.js"),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let source_dir = manifest_dir.join("src");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?).join("virtual_modules");
    fs::create_dir_all(&out_dir)?;

    for (source_name, output_name) in VIRTUAL_MODULES {
        transpile_module(&source_dir, &out_dir, source_name, output_name)?;
    }

    Ok(())
}

fn transpile_module(
    source_dir: &Path,
    out_dir: &Path,
    source_name: &str,
    output_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_path = source_dir.join(source_name);
    println!("cargo:rerun-if-changed={}", source_path.display());

    let code = fs::read_to_string(&source_path)?;
    let specifier = deno_ast::ModuleSpecifier::from_file_path(&source_path)
        .map_err(|_| format!("invalid module source path: {}", source_path.display()))?;
    let media_type = MediaType::from_path(&source_path);
    let parsed = parse_module(ParseParams {
        specifier,
        text: code.into(),
        media_type,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })?;

    let transpiled = parsed
        .transpile(
            &TranspileOptions::default(),
            &TranspileModuleOptions::default(),
            &EmitOptions {
                source_map: SourceMapOption::None,
                inline_sources: true,
                remove_comments: false,
                ..Default::default()
            },
        )?
        .into_source();

    let output_path = out_dir.join(output_name);
    fs::write(output_path, transpiled.text)?;
    Ok(())
}
