# Architecture Overview

`egui-component` is organized around a JSX-first declarative contract runtime on top of `egui`. The typed component layer in `src/components` remains as the internal renderer implementation and a deprecated compatibility shim for old Rust-authored callers.

## Runtime Shape

- `theme::install`, `theme::set_theme`, `theme::set_mode`, and `theme::with_theme` own shared fonts, visuals, icon/image loader setup, and the semantic theme contract.
- `layout::*` owns public flow-layout authoring for rows, columns, padding, alignment, and sized boxes.
- `contract::*` owns serializable node trees, semantic events, schema export, and the renderer entry point used by the JSX runtime.
- `src/components/*.rs` contains the internal Rust runtime implementations that the contract renderer delegates to.
- `ui.components()` remains as a deprecated direct-Rust compatibility facade over those same internal implementations.
- `contract::*` is the egui-side declarative contract layer, not a scripting VM embedding core.
- The host-neutral authored-runtime crate lives in `crates/clay-jsx-runtime`, the egui bridge lives in `crates/clay-jsx-egui-bridge`, and the active example in `examples/runtime-jsx` feeds this layer through the bridge.

## Styling Layers

- `src/theme.rs` is the source of truth for semantic colors, preset palettes, radii, shadows, and scoped theme behavior.
- `src/ui/tokens.rs` is internal resolver glue from semantic roles to component states plus shared layout constants.
- `src/ui/style.rs` applies global defaults for spacing, interactive sizing, and raw `egui::Visuals` derived from the active semantic theme.

## Composition Model

- `src/layout.rs` contains the public flow-layout layer used for explicit gap, padding, alignment, and sizing.
- `src/primitives/*.rs` contains reusable chrome helpers for surfaces, popups, and exact-rect control painting.
- Primitive components stay narrow and typed.
- Internal runtime components should use `layout::*` for flow layout, exact rect math for geometry-sensitive controls, and re-enter `ComponentUi` only when they need other runtime widgets inside closures.
- The declarative renderer should reuse the internal runtime components where possible and only compose around them when it needs to recover stable semantic events such as confirm/cancel or command invocation.

## Registry And Showcase

- `src/catalog.rs` is the legacy component registry used by the deprecated Rust showcase and parser helpers.
- `src/contract/registry.rs` is the declarative source of truth for supported families, props, variants, events, and generated reference output.
- Runtime embedding code is intentionally separate from this registry; it should not be folded into the declarative contract surface.
- `src/example_apps/showcase.rs` is the legacy live preview surface for the deprecated Rust component facade.
- `src/example_apps/contract_demo.rs` is the legacy host-authored contract demo surface and should visibly exercise every registered contract family.
- Every internal runtime component that remains user-relevant should also have a matching doc stub in `docs/llm/components/`.
