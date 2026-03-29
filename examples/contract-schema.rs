fn main() {
    match egui_component::contract::schema_json_pretty() {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("failed to export contract schema: {error}");
            std::process::exit(1);
        }
    }
}
