# Architecture Overview

`egui-component` is organized around a typed component layer on top of `egui`.

## Runtime Shape

- `theme::install`, `theme::set_theme`, `theme::set_mode`, and `theme::with_theme` own shared fonts, visuals, icon/image loader setup, and the semantic theme contract.
- `layout::*` owns public flow-layout authoring for rows, columns, padding, alignment, and sized boxes.
- `ui.components()` exposes typed widget methods for the current `egui::Ui`.
- Each public widget lives in `src/components/*.rs` as a small builder plus a `ComponentUi` entry point.

## Styling Layers

- `src/theme.rs` is the source of truth for semantic colors, preset palettes, radii, shadows, and scoped theme behavior.
- `src/ui/tokens.rs` is internal resolver glue from semantic roles to component states plus shared layout constants.
- `src/ui/style.rs` applies global defaults for spacing, interactive sizing, and raw `egui::Visuals` derived from the active semantic theme.

## Composition Model

- `src/layout.rs` contains the public flow-layout layer used for explicit gap, padding, alignment, and sizing.
- `src/primitives/*.rs` contains reusable chrome helpers for surfaces, popups, and exact-rect control painting.
- Primitive components stay narrow and typed.
- Composed components should use `layout::*` for flow layout, exact rect math for geometry-sensitive controls, and re-enter `ui.components()` only when they need typed widgets inside closures.

## Registry And Showcase

- `src/catalog.rs` is the public component registry used by the showcase and parser helpers.
- `src/dev/showcase/component_showcase.rs` is the canonical live preview surface for every catalog entry.
- Every public component must also have a matching doc stub in `docs/llm/components/`.
