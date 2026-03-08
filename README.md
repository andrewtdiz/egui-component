# egui-component

A focused component library for egui with one interactive showcase runtime for all components in `src/components`.
The default component typography uses Segoe UI.

## Library Surface

- `egui_component::catalog::*` exposes the component catalog and parser helpers.
- `egui_component::components::*` exposes the typed builder structs, typed scoped overrides, and the `ui.components()` wrapper entry point.
- `egui_component::prelude::*` re-exports `ComponentUiExt` and the component builder types for concise UI code.
- `egui_component::theme::*` exposes setup helpers for style, icon loading, and per-`Ui` component theme application.
- `egui_component::dev::showcase::render_component_showcase` renders the component gallery showcase.
- `egui_component::dev` includes the showcase runtime.

## Integration in Another `egui` Project

Add the crate to your app (path dependency shown here):

```toml
[dependencies]
eframe = "0.33.3"
egui = "0.33.3"
egui-component = { path = "../egui-component" }
```

Initialize the theme once in your app startup:

```rust
eframe::run_native(
    "App",
    eframe::NativeOptions::default(),
    Box::new(|cc| {
        egui_component::theme::setup(&cc.egui_ctx);
        Ok(Box::new(MyApp::default()))
    }),
)?;
```

Then use components from the prelude in your UI code:

```rust
use egui_component::prelude::*;

let mut ui = ui.components();

let _ = ui.button("Save");
let _ = ui.button(("Cancel", ButtonStyle::Secondary));
let _ = ui.icon(("search", 16.0));
```

Most component methods accept shorthand values via `Into<ComponentBuilder>` in addition to explicit builder structs:

```rust
let _ = ui.label("Material");
let _ = ui.tooltip(("Hover this trigger", "Tooltip content example"));
let _ = ui.checkbox(&mut enabled, "Receive Shadows");
let _ = ui.text_input(&mut name, (280.0, "Component name"));
let _ = ui.field(&mut value, ("Material", 280.0, "Assigned material"));
let _ = ui.select(
    &mut selected,
    (Id::new("status"), &["Draft", "Review"], 220.0),
);
let _ = ui.dropdown_menu(("Open", &entries[..], 220.0));
```

Nested component composition stays on the wrapper too:

```rust
ui.card((), |ui| {
    let _ = ui.label(("Material", LabelTone::Secondary));
    let _ = ui.text_input(&mut name, "Component name");
    ui.horizontal(|ui| {
        let _ = ui.button(("Save", ButtonStyle::Primary));
        let _ = ui.button(("Cancel", ButtonStyle::Secondary));
    });
});
```

Scoped per-widget overrides live on the wrapper and compose by type:

```rust
ui.with_override(
    (
        ButtonOverride::new().style(ButtonStyle::Secondary),
        LabelOverride::new().tone(LabelTone::Muted),
        CardOverride::new().padding(16, 16),
    ),
    |ui| {
        let _ = ui.button("Save");
        let _ = ui.label("Scoped helper copy");
        let _ = ui.card((), |ui| {
            let _ = ui.button("Nested secondary action");
        });
    },
);
```

If you only want component spacing/interaction styling inside one region, wrap with:

```rust
egui_component::theme::apply_component_theme(ui);
```

## Commands

Run the showcase window:

```bash
cargo showcase
```
