# egui-component

A focused component library for egui with one showcase runtime for game-entity editing UI.
The default component typography now uses Geist.

## Library Surface

- `egui_component::catalog::*` exposes the component catalog and parser helpers.
- `egui_component::components::*` exposes the custom, shadcn-style wrapper components used by the showcase.
- `egui_component::prelude::*` re-exports the full component surface for concise UI code.
- `egui_component::dev::showcase::render_entity_components_editor` renders the single composed showcase story.
- `egui_component::dev` includes the showcase runtime.

## Commands

Run the showcase window:

```bash
cargo showcase
```
