# Architecture Overview

`egui-component` is organized around a typed component layer on top of `egui`, plus a curated declarative contract layer for host-driven editor UX.

## Runtime Shape

- `theme::install`, `theme::set_theme`, `theme::set_mode`, and `theme::with_theme` own shared fonts, visuals, icon/image loader setup, and the semantic theme contract.
- `layout::*` owns public flow-layout authoring for rows, columns, padding, alignment, and sized boxes.
- `ui.components()` exposes typed widget methods for the current `egui::Ui`.
- Each public widget lives in `src/components/*.rs` as a small builder plus a `ComponentUi` entry point.
- `contract::*` owns serializable node trees, semantic events, schema export, and the host renderer that translates declarative nodes into the typed builders.
- `contract::*` is the egui-side declarative contract layer, not a scripting VM embedding core.
- The reusable authored-runtime crate lives in `crates/runtime-jsx`, and the active example in `examples/runtime-jsx` feeds this layer through a host adapter.

## Styling Layers

- `src/theme.rs` is the source of truth for semantic colors, preset palettes, radii, shadows, and scoped theme behavior.
- `src/ui/tokens.rs` is internal resolver glue from semantic roles to component states plus shared layout constants.
- `src/ui/style.rs` applies global defaults for spacing, interactive sizing, and raw `egui::Visuals` derived from the active semantic theme.

## Composition Model

- `src/layout.rs` contains the public flow-layout layer used for explicit gap, padding, alignment, and sizing.
- `src/primitives/*.rs` contains reusable chrome helpers for surfaces, popups, and exact-rect control painting.
- Primitive components stay narrow and typed.
- Composed components should use `layout::*` for flow layout, exact rect math for geometry-sensitive controls, and re-enter `ui.components()` only when they need typed widgets inside closures.
- The declarative renderer should reuse the typed builders where possible and only compose around them when it needs to recover stable semantic events such as confirm/cancel or command invocation.

## Registry And Showcase

- `src/catalog.rs` is the public component registry used by the showcase and parser helpers.
- `src/contract/registry.rs` is the declarative source of truth for supported families, props, variants, events, and generated reference output.
- Runtime embedding code is intentionally separate from this registry; it should not be folded into the declarative contract surface.
- `src/example_apps/showcase.rs` is the canonical live preview surface for typed components.
- `src/example_apps/contract_demo.rs` is the declarative contract demo surface and should visibly exercise every registered contract family.
- Every public component must also have a matching doc stub in `docs/llm/components/`.
