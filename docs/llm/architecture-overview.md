# Architecture Overview

`egui-component` is organized around a typed component layer on top of `egui`.

## Runtime Shape

- `theme::install` and `theme::set_mode` own shared fonts, visuals, and icon/image loader setup.
- `ui.components()` wraps `egui::Ui` in `ComponentUi`, which applies the component style profile and exposes typed widget methods.
- Each public widget lives in `src/components/*.rs` as a small builder plus a `ComponentUi` entry point.

## Styling Layers

- `src/ui/tokens.rs` is the source of truth for colors, radii, spacing, and shared state styling.
- `src/ui/style.rs` applies global defaults for spacing and interactive sizing.
- Components should resolve local widget states from tokens instead of hardcoding colors or repaint rules ad hoc.

## Composition Model

- `src/primitives/*.rs` contains reusable chrome helpers for rows, surfaces, popups, and layout framing.
- Primitive components stay narrow and typed.
- Composed components should build from existing primitives/components and use closures for optional regions such as headers, bodies, or footers.

## Registry And Showcase

- `src/catalog.rs` is the public component registry used by the showcase and parser helpers.
- `src/dev/showcase/component_showcase.rs` is the canonical live preview surface for every catalog entry.
- Every public component must also have a matching doc stub in `docs/llm/components/`.
