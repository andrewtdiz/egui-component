# egui-component

`egui-component` is a focused component library for `egui`.

The crate is organized around three public layers:

- `egui_component::components::*` for typed component builders and `ui.components()`
- `egui_component::layout::*` for vendored taffy flex/grid primitives
- `egui_component::theme::*` for theme installation and scoped theme changes

The crate also ships an optional host-owned declarative layer for serialized or host-driven editor surfaces:

- `egui_component::contract::*` for serializable UI trees, semantic events, renderer entry points, and schema export

Repo-local demos and showcase helpers live under `egui_component::demos::*`. The older
`egui_component::example_apps::*` and `egui_component::catalog::*` module paths are deprecated shims.

The portable Luau embedding core lives in the reusable `luau-runtime-core` crate. `egui-component` does not expose that runtime surface; new hosts should depend on `luau-runtime-core` directly. The core includes filesystem hot-reload support for file-backed script providers.

## Install

```toml
[dependencies]
egui = "0.33.3"
eframe = "0.33.3"
egui-component = { path = "../egui-component-mainline" }
```

No feature flag is required for taffy-backed layout.

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
                ThemeMode::System,
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
- `egui_component::layout::*` for direct taffy layout via `tui`, `tid`, and `taffy`
- `egui_component::prelude::*` for the typed component builders
- `egui_component::contract::*` when the host needs an optional declarative surface with schema export and semantic events

## Declarative Contract

The `contract` module is an optional host-driven declarative layer. It is not the default embedded Luau runtime boundary.

It provides:

- serde-serializable `ContractTree` and `ContractNode` models
- `render_tree(&mut egui::Ui, &ContractTree) -> Vec<ContractEvent>`
- registry and schema export helpers via `contract::registry()`, `contract::schema()`, and `contract::schema_json_pretty()`
- `contract::reference_markdown()` for a generated human-readable family reference

Export the schema as JSON with:

```rust
egui_component::contract::schema_json_pretty()
```

Export the generated reference with:

```rust
egui_component::contract::reference_markdown()
```

## Basic Usage

Install the theme once during app startup:

```rust
use egui_component::{BaseColor, ThemeMode, ThemeSpec};

egui_component::theme::install(
    &cc.egui_ctx,
    ThemeSpec::preset(BaseColor::Neutral),
    ThemeMode::System,
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

Use egui's native `ui.horizontal` / `ui.vertical` for simple composition, and `layout::*` when you need flex or grid behavior from taffy:

```rust
use egui::Id;
use egui_component::layout::{taffy, tid, tui, TuiBuilderLogic};
use egui_component::prelude::*;
use egui_component::layout::taffy::prelude::{auto, length, percent};

tui(ui, Id::new("actions"))
    .reserve_available_width()
    .style(taffy::Style {
        flex_direction: taffy::FlexDirection::Row,
        gap: length(8.0),
        size: taffy::Size {
            width: percent(1.0),
            height: auto(),
        },
        ..Default::default()
    })
    .show(|tui| {
        tui.id(tid("save")).ui(|ui| {
            let _ = ui.components().button(Button::new("Save").variant(ButtonVariant::Primary));
        });
        tui.id(tid("cancel")).ui(|ui| {
            let _ = ui.components().button(Button::new("Cancel").variant(ButtonVariant::Secondary));
        });
    });

let _ = ui.horizontal(|ui| {
    let mut ui = ui.components();
    let _ = ui.button(Button::new("Preview").variant(ButtonVariant::Secondary));
    let _ = ui.button(Button::new("Duplicate").variant(ButtonVariant::Ghost));
});
```

Twemoji rendering is available through the typed facade:

```rust
use egui_component::prelude::*;

let _ = ui.components().twemoji(Twemoji::new("🧑🏽‍🚀").size(28.0));
```

## Examples

The repo currently ships these examples:

- `showcase`: broad catalog view for the component library
- `contract-showcase`: optional declarative editor surface driven entirely through `egui_component::contract::*`
- `runtime-egui-host`: canonical embedded Luau runtime host using the direct typed `app.*` / `ui.*` bridge
- `showcase-runtime`: convenience wrapper that boots the Luau showcase surface through the same direct runtime host
- `component-gallery-runtime`: Luau primitive component catalog for the shadcn-like direct `ui.*` surface
- `theme-playground`: live `ThemeSpec`, `ThemeMode`, `theme::set_theme`, `theme::set_mode`, and `theme::with_theme`

Run the main showcase:

```bash
cargo run --example showcase
```

Run the local wrapper command for the showcase:

```bash
cargo example showcase
```

The wrapper commands live in the workspace tool crate at `tools/example-runner`.

Generate isolated showcase snapshots:

```bash
cargo example snapshot --component canva-position
cargo screenshot --component canva-position
```

Enable hot restart for the showcase:

```bash
cargo example showcase --hot
```

Run any focused example:

```bash
cargo run --example theme-playground
cargo run --example runtime-egui-host
cargo run --example showcase-runtime
cargo run --example component-gallery-runtime
```

Replace `theme-playground` with `contract-showcase`.

For live Luau edits, use `runtime-egui-host` for the main demo surface, `showcase-runtime` for the library-backed recipe showcase, and `component-gallery-runtime` for the primitive Luau component catalog. The component gallery now uses the same Rust-owned showcase shell pattern as the original catalog while each selected preview is rendered from a focused Luau root. The direct typed Luau path is the supported product-development model. The alternate `contract-showcase` path remains optional.

In hot mode the runner watches the repo, rebuilds `showcase`, and relaunches the example process on change.
The window is restarted on each rebuild rather than patched in place.

Snapshot mode renders offscreen without opening a native window and crops the output to the component preview itself rather than the outer showcase wrapper card.

## Twemoji Attribution

Twemoji graphics are included for the `Twemoji` component and are licensed under CC-BY 4.0.
Keep attribution to the Twemoji project in your app's README, About screen, or legal notices when you ship builds that use these graphics.
