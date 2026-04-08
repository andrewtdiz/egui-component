use std::{fs, io, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = workspace_root()?;
    let typings_path = workspace_root
        .join("examples")
        .join("runtime-luau")
        .join("ui")
        .join("core")
        .join("types.luau");
    let reference_path = workspace_root
        .join("docs")
        .join("luau-runtime-api-reference.md");

    fs::write(&typings_path, luau_runtime_core::runtime_api_luau_typings())?;
    fs::write(
        &reference_path,
        luau_runtime_core::runtime_api_reference_markdown(),
    )?;

    eprintln!("wrote {}", typings_path.display());
    eprintln!("wrote {}", reference_path.display());
    Ok(())
}

fn workspace_root() -> Result<PathBuf, io::Error> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|path| path.to_path_buf())
        .ok_or_else(|| io::Error::other("luau-runtime crate is not inside a workspace root"))
}
