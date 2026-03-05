# egui-component

A focused component library for egui with one interactive showcase runtime for all components in `src/components`.
The default component typography uses Geist.

## Library Surface

- `egui_component::catalog::*` exposes the component catalog and parser helpers.
- `egui_component::components::*` exposes the custom, shadcn-style wrapper components used by the showcase.
- `egui_component::prelude::*` re-exports the full component surface for concise UI code.
- `egui_component::dev::showcase::render_component_showcase` renders the component gallery showcase.
- `egui_component::dev` includes the showcase runtime.

## Commands

Run the showcase window:

```bash
cargo showcase
```
