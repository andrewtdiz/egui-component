use std::path::PathBuf;

use clay_jsx_runtime::contract::write_typescript_declarations;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_path = manifest_dir.join("types/contract.generated.d.ts");
    write_typescript_declarations(&output_path)?;
    println!("wrote {}", output_path.display());
    Ok(())
}
