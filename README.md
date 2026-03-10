# egui-component

A focused component library for egui with one interactive showcase runtime for all components in `src/components`.
The default component typography uses Segoe UI.

## Library Surface

- `egui_component::catalog::*` exposes the component catalog and parser helpers.
- `egui_component::components::*` exposes the typed builder structs, typed scoped overrides, and the `ui.components()` wrapper entry point.
- `egui_component::prelude::*` re-exports `ComponentUiExt`, `ThemeMode`, and the component builder types for concise UI code.
- `egui_component::theme::*` exposes install and theme-swap helpers for shared fonts, visuals, and icon loading.
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
use egui_component::ThemeMode;

eframe::run_native(
    "App",
    eframe::NativeOptions::default(),
    Box::new(|cc| {
        egui_component::theme::install(&cc.egui_ctx, ThemeMode::Dark);
        Ok(Box::new(MyApp::default()))
    }),
)?;
```

Swap themes later with:

```rust
egui_component::theme::set_mode(ctx, ThemeMode::Light);
```

Then use components from the prelude in your UI code:

```rust
use egui_component::prelude::*;

let mut ui = ui.components();

let _ = ui.button("Save");
let _ = ui.button(Button::new("Cancel").variant(ButtonVariant::Secondary));
let _ = ui.icon(Icon::new("search").size(16.0));
```

Use builder structs for revised components and keep shorthand usage to the trivial cases:

```rust
let _ = ui.label("Material");
let _ = ui.tooltip(Tooltip::new("Hover this trigger", "Tooltip content example"));
let _ = ui.checkbox(&mut enabled, "Receive Shadows");
let _ = ui.text_input(&mut name, TextInput::new().width(280.0).placeholder("Component name"));
let _ = ui.field(
    &mut value,
    Field::new("Material")
        .width(280.0)
        .helper_text("Assigned material"),
);
let _ = ui.select(
    &mut selected,
    Select::from_id(Id::new("status"), &["Draft", "Review"]).width(220.0),
);
let _ = ui.dropdown_menu(DropdownMenu::new("Open").entries(&entries).width(220.0));
```

Nested component composition stays on the wrapper too:

```rust
ui.card((), |ui| {
    let _ = ui.label(Label::new("Material").tone(LabelTone::Secondary));
    let _ = ui.text_input(&mut name, "Component name");
    ui.horizontal(|ui| {
        let _ = ui.button(Button::new("Save").variant(ButtonVariant::Primary));
        let _ = ui.button(Button::new("Cancel").variant(ButtonVariant::Secondary));
    });
});
```

Scoped per-widget overrides live on the wrapper and compose by type:

```rust
ui.with_override(
    (
        ButtonOverride::new().variant(ButtonVariant::Secondary),
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

`ui.components()` now applies the component spacing, radius, and interaction profile automatically.

## Commands

Run the showcase window:

```bash
cargo showcase
```
