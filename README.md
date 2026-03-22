# egui-component

`egui-component` is a focused component library for `egui`.

The crate is organized around three public layers:

- `egui_component::components::*` for typed component builders and `ui.components()`
- `egui_component::layout::*` for explicit row, column, inset, and spacer composition
- `egui_component::theme::*` for theme installation and scoped theme changes

## Install

```toml
[dependencies]
egui = "0.33.3"
eframe = "0.33.3"
egui-component = { path = "../egui-component-mainline" }
```

## Start A New Project

Create a new `eframe` app:

```bash
cargo new my-egui-app
cd my-egui-app
```

Then add `egui`, `eframe`, and this component library to `Cargo.toml`:

```toml
[dependencies]
egui = "0.33.3"
eframe = "0.33.3"
egui-component = { path = "../egui-component-mainline" }
```

Replace `src/main.rs` with a minimal app that installs the theme and renders components through `ui.components()`:

```rust
use egui::CentralPanel;
use egui_component::prelude::*;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            egui_component::theme::install(
                &cc.egui_ctx,
                ThemeSpec::preset(BaseColor::Neutral),
                ThemeMode::Dark,
            );
            Ok(Box::<MyApp>::default())
        }),
    )
}

#[derive(Default)]
struct MyApp {
    name: String,
    enabled: bool,
    status: Option<usize>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let mut ui = ui.components();

            let _ = ui.label("Project");
            let _ = ui.text_input(
                &mut self.name,
                TextInput::new()
                    .width(280.0)
                    .placeholder("Component name"),
            );
            let _ = ui.checkbox(&mut self.enabled, "Enabled");
            let _ = ui.select(
                &mut self.status,
                Select::from_id(
                    egui::Id::new("status"),
                    &["Draft", "Review", "Approved"],
                )
                .width(220.0),
            );
            let _ = ui.button(Button::new("Save").variant(ButtonVariant::Primary));
        });
    }
}
```

Run it with:

```bash
cargo run
```

From there, use:

- `egui_component::theme::*` to install or scope themes
- `egui_component::layout::*` for rows, columns, inset, and alignment
- `egui_component::prelude::*` for the typed component builders

## Basic Usage

Install the theme once during app startup:

```rust
use egui_component::{BaseColor, ThemeMode, ThemeSpec};

egui_component::theme::install(
    &cc.egui_ctx,
    ThemeSpec::preset(BaseColor::Neutral),
    ThemeMode::Dark,
);
```

Then render components through the typed facade:

```rust
use egui::Id;
use egui_component::prelude::*;

let mut ui = ui.components();

let _ = ui.label("Project");
let _ = ui.text_input(&mut name, TextInput::new().width(280.0).placeholder("Component name"));
let _ = ui.checkbox(&mut enabled, "Enabled");
let _ = ui.select(
    &mut selected_status,
    Select::from_id(Id::new("status"), &["Draft", "Review", "Approved"]).width(220.0),
);
let _ = ui.button(Button::new("Save").variant(ButtonVariant::Primary));
```

Use `layout::*` for composition and re-enter typed components inside closures:

```rust
use egui_component::layout;
use egui_component::prelude::*;

let _ = layout::row().gap(8.0).show(ui, |ui| {
    let mut ui = ui.components();
    let _ = ui.button(Button::new("Save").variant(ButtonVariant::Primary));
    let _ = ui.button(Button::new("Cancel").variant(ButtonVariant::Secondary));
});
```

## Examples

The repo currently ships these examples:

- `showcase`: broad catalog view for the component library
- `content-composition`: reusable `layout::*` composition with cards, labels, buttons, kbd, and scoped overrides
- `popup-patterns`: focused popup interactions with `Tooltip`, `Popover`, `DropdownMenu`, and `Dialogue`
- `theme-playground`: live `ThemeSpec`, `ThemeMode`, `theme::set_theme`, `theme::set_mode`, and `theme::with_theme`

Run the main showcase:

```bash
cargo run --example showcase
```

Run any focused example:

```bash
cargo run --example theme-playground
```

Replace `theme-playground` with `content-composition` or `popup-patterns`.

If you use the local alias for the showcase:

```bash
cargo showcase
```
