# Contract Architecture

`egui_component::contract::*` is the optional host-owned declarative layer for serialized and host-driven UI surfaces.

## Design Goals

- Keep the public boundary serializable and introspectable.
- Keep the surface curated and semantic rather than mirroring all immediate-mode builders.
- Preserve `egui-component` as the rendering substrate instead of exposing raw `egui`.
- Return normalized semantic events so the host can stay authoritative for state.

## Relationship To Authored Runtimes

- `egui_component::contract::*` is an egui-side declarative surface that a host can drive from any authored runtime.
- The `clay-jsx-runtime` crate in `crates/clay-jsx-runtime` owns host-neutral V8 lifetime, module loading, JSX/TSX transpilation through `deno_ast`, and commit/log host ops. The `clay-jsx-egui-bridge` crate in `crates/clay-jsx-egui-bridge` owns incremental retained host-tree commits and egui contract materialization.
- This layer only renders host-authored trees and returns semantic events; it does not own script state or dispatch.
- The runtime lowers JSX into Rust-owned host nodes, materializes `ContractTree` for rendering, and keeps Rust as the egui renderer.

## Runtime Shape

- `ContractTree` and `ContractNode` are serde-friendly owned data structures.
- Every node carries a stable `node_id`, plus shared `visible`, `enabled`, `class`, `class_list`, `slot_classes`, `actions`, and `layout` scaffolding.
- Interactive nodes expose stable string `action_id`s rather than callback handles.
- `render_tree` and `render_component_tree` translate the declarative tree into the existing typed builders and return `Vec<ContractEvent>` for the current frame.
- These functions are host adapter entry points, not scripting runtime entry points.

## State Ownership

- The host owns authoritative values such as text, selection, open state, and hierarchy expansion.
- The renderer copies those values into local mutable variables only for the current frame.
- When a rendered value changes, the renderer emits a semantic event and the host decides how to update authoritative state for the next frame.

## Schema And Metadata

- `contract::registry()` is the Rust source of truth for the supported families.
- `contract::schema()` and `contract::schema_json_pretty()` expose machine-readable metadata for code generation.
- `contract::reference_markdown()` exposes a generated human-readable family reference from the same source of truth.
- Shared object and enum types are described alongside family metadata so runtime bridges do not need to infer nested item shapes.

## V1 Scope

- Layout: `row`, `column`, `inset`, `sized-box`, `spacer`
- Surfaces and shell: `card`, `sidebar`, `toolbar`, `menu-bar`, `dialogue-modal`
- Navigation and content: `tabs`, `label`, `separator`, `collapsible`, `hierarchy`
- Controls: `button`, `button-group`, `input`, `number-input`, `checkbox`, `switch`, `select`, `field`
- Feedback: `spinner`, `progress`, `toast-viewport`

## When To Add A Family

- Add a family when the host needs a stable semantic surface that maps cleanly onto existing typed builders.
- Add a family when state can stay host-owned and the renderer can return meaningful semantic events without leaking immediate-mode internals.
- Add a family when the Rust registry can describe the props, variants, and events clearly enough for schema-driven runtime bridges.

## When Not To Add A Family

- Do not add components that are still mostly paint knobs, geometry-heavy one-offs, or product-specific workflow shells.
- Do not add components that would require imperative per-widget scripting calls every frame.
- Do not add components whose useful public shape would expose raw `egui` ids, styling primitives, or transient UI memory details.

## Explicit Non-Goals For V1

- No raw `egui::Id`, `Color32`, `Stroke`, `Shadow`, or low-level paint overrides in the public contract
- No third-party extension system
- No authoritative state ownership in the renderer
- No `combobox` in the first declarative release
- No hierarchy drag-reorder event contract in v1
- No scripting VM ownership, script loading, or reload orchestration in this layer
