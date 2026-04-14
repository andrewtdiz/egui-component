# egui-component

`egui-component` is now centered on a JSX/TSX-authored runtime for `egui`.

The active public surface is:

- `egui_component::contract::*` for serializable UI trees, semantic events, renderer entry points, and schema export
- `egui_component::layout::*` for vendored taffy flex/grid primitives and the minimal egui flow helpers used by the runtime
- `egui_component::theme::*` for theme installation and scoped theme changes

Direct Rust component authoring through `egui_component::components::*`, `egui_component::prelude::*`, and `ui.components()` is deprecated. Those APIs remain as compatibility shims over the internal runtime renderer implementation.

Host-authored Rust demos under `egui_component::demos::*` are also deprecated. The active example surface is `examples/runtime-jsx/`.

The host-neutral JSX/TSX runtime lives under `crates/clay-jsx-runtime/`; the egui contract bridge lives under `crates/clay-jsx-egui-bridge/`. The repo-local `examples/runtime-jsx/` app keeps the `egui` frame loop in Rust, calls the bridge crate, evaluates JSX/TSX with `deno_core`/V8 via `deno_ast`, and renders the result through `egui_component::contract::*`.

## Install

```toml
[dependencies]
egui = "0.33.3"
eframe = "0.33.3"
egui-component = { path = "../egui-component-mainline" }
```

No feature flag is required for taffy-backed layout.

## JSX Runtime

Run the active JSX runtime host from the repo root:

```bash
cargo run --example runtime-jsx-host
```

Pass a different authored entrypoint or open the component catalog:

```bash
cargo run --example runtime-jsx-host -- ./examples/runtime-jsx/app.jsx
cargo run --example runtime-jsx-host -- ./examples/runtime-jsx/catalog.tsx
```

The direct Rust builder facade remains below as a deprecated compatibility path.

## Legacy Direct Rust Authoring (Deprecated)

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
- `egui_component::prelude::*` only for legacy direct Rust authoring; it is deprecated
- `egui_component::contract::*` for the active declarative surface with schema export and semantic events

## Declarative Contract

The `contract` module is the active declarative render boundary. The JSX runtime example uses it as the Rust-side render boundary after V8 evaluates the authored JSX file.

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

## Legacy Rust Surface Details

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

- `showcase`: legacy broad catalog view for the deprecated Rust component facade
- `contract-showcase`: legacy host-authored declarative editor surface driven entirely through `egui_component::contract::*`
- `runtime-jsx-host`: embedded JSX/TSX runtime host using the `crates/clay-jsx-egui-bridge` bridge on top of the `crates/clay-jsx-runtime` `deno_core`/V8 runtime and the contract renderer
- `runtime-jsx-motion`: focused JSX/TSX motion sync demo for opacity, translation, scale, and rotation values driven by the runtime and drawn by egui
- `runtime-egui-host`: legacy compatibility alias for `runtime-jsx-host`
- `theme-playground`: legacy typed-Rust theme preview surface

Run the legacy Rust showcase:

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
cargo run --example runtime-jsx-host
cargo run --example runtime-jsx-motion
```

Replace `theme-playground` with `contract-showcase`.

For live JSX edits, run `runtime-jsx-host` and edit `examples/runtime-jsx/app.jsx` on disk. The Rust showcase and theme playground remain only as deprecated compatibility/demo surfaces. For a focused motion sync surface, run `runtime-jsx-motion` and edit `examples/runtime-jsx/motion-sync.tsx`. The host-neutral runtime is `clay-jsx-runtime` under `crates/clay-jsx-runtime`; the egui retained host tree and `ContractTree` materialization live in `clay-jsx-egui-bridge` under `crates/clay-jsx-egui-bridge`. `runtime-egui-host` remains as a legacy alias for the same host.

In hot mode the runner watches the repo, rebuilds `showcase`, and relaunches the example process on change.
The window is restarted on each rebuild rather than patched in place.

Snapshot mode renders offscreen without opening a native window and crops the output to the component preview itself rather than the outer showcase wrapper card.

## Twemoji Attribution

Twemoji graphics are included for the `Twemoji` component and are licensed under CC-BY 4.0.
Keep attribution to the Twemoji project in your app's README, About screen, or legal notices when you ship builds that use these graphics.
