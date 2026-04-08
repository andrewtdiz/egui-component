# Egui Component LLM Docs

This folder is the authoring guide for this crate.

Use it when you are:

- adding a new component
- editing an existing component
- generating examples
- asking an LLM to extend the library without breaking the current look and feel
- running the authoring workflow through `cargo xtask`

Read these files in order:

1. `architecture-overview.md`
2. `authoring-rules.md`
3. `components-reference.md`
4. `contract-architecture.md`
5. `contract-reference.md`
6. `../../defold_luau_gui_runtime_architecture.md`

## What This Library Is

`egui-component` is a small component library on top of `egui`.

The public runtime surface is built around three ideas:

- `theme::install(&Context, ThemeSpec, ThemeMode)` installs the shared fonts, visuals, semantic theme, and icon loading.
- `theme::set_theme`, `theme::set_mode`, and `theme::with_theme` are the only supported theme mutation paths.
- `layout::*` expresses flow layout with explicit gap, padding, alignment, and sizing primitives.
- `ui.components()` exposes the typed widget facade for a given `egui::Ui`.
- `contract::*` exposes an optional host-driven declarative contract layer, semantic event model, and schema export.
- The portable Luau embedding core is packaged separately as `luau-runtime-core`; see `../../defold_luau_gui_runtime_architecture.md` for the host-agnostic core/adapter split. File-backed providers can surface filesystem hot-reload events through the core API.
- Each component method accepts a typed builder; new work should keep shorthand forms minimal.

## Non-Negotiable Design Contract

When writing or editing components in this repo, keep these rules fixed unless the user explicitly asks to redesign the system:

- Do not invent new colors when an existing token already covers the state.
- Do not change default padding, spacing, radius, or widget heights just because a component looks slightly off.
- Do not bypass `src/theme.rs` for semantic theme source-of-truth.
- Treat `src/ui/tokens.rs` as internal semantic resolver glue, not a public theme contract.
- Do not bypass `src/ui/style.rs` for global spacing and runtime style defaults.
- Do not mutate `spacing_mut().item_spacing` or `style_mut().spacing` directly in component code when `layout::*` can express the intent.
- Keep components visually aligned with shadcn-style expectations: restrained variants, token-driven states, and composition over ad hoc styling.

Current global layout/style anchors:

- Item spacing Y: `8.0`
- Button padding: `12.0 x 7.0`
- Default interactive height: `34.0`
- Input padding: `10 x 6`
- Default root radius: `10`, derived radii `sm/md/lg/xl = 6 / 8 / 10 / 14`
- Default text sizes: body `14`, heading `16`, small `12`, most component labels `12`

## File Map

Use these files as the source of truth:

- `src/components/api.rs`
  Defines `ComponentUiExt`, the typed facade, and scoped override plumbing.
- `src/layout.rs`
  Public flow-layout helpers used for row, column, inset, alignment, sized boxes, and spacers.
- `src/primitives/*.rs`
  Public authoring primitives for chrome, popup framing, and exact-rect surface helpers.
- `src/components/*.rs`
  One file per component or component family.
- `src/theme.rs`
  Public Tailwind/shadcn semantic theme model, presets, scoped theme APIs, and source-of-truth colors.
- `src/ui/tokens.rs`
  Internal resolver layer that maps semantic theme roles onto component states and shared layout constants.
- `src/ui/style.rs`
  Global component-theme defaults.
- `src/catalog.rs`
  Public component registry used by the showcase and parser helpers.
- `src/contract/*.rs`
  Declarative contract model, registry, renderer, and schema export.
- `src/example_apps/showcase.rs`
  Canonical typed-component examples.
- `src/example_apps/contract_demo.rs`
  Canonical optional contract-layer example that renders every registered family from a host-authored tree.
- `docs/llm/components/*.md`
  Per-component authoring stubs used to build the generated reference.
- `docs/llm/contract-*.md`
  Declarative contract architecture and generated supported-family reference.
- `docs/llm/templates/*.md`
  Family guides and the component stub template used by `cargo xtask new-component`.

## Surface Summary

Primitive-ish building blocks:

- `Label`
- `Kbd`
- `Icon`
- `TextInput`
- `Field`
- `Button`
- `ButtonGroup`
- `Checkbox`
- `Switch`
- `Slider`
- `NumberInput`
- `Select`
- `Tabs`
- `Separator`
- `Card`
- `Progress`
- `Tooltip`

Composed or higher-level pieces:

- `Collapsible`
- `DropdownMenu`
- `Combobox`
- `Command`
- `Dialogue`

Wrapper utilities:

- `ComponentUiExt`
- `ButtonOverride`
- `CardOverride`
- `LabelOverride`
- `TextInputOverride`

## Typical Usage

```rust
use egui_component::prelude::*;

let mut ui = ui.components();

let _ = ui.label("Material");
let _ = ui.text_input(&mut name, TextInput::new().width(220.0).placeholder("Name"));
let _ = ui.button(Button::new("Save").variant(ButtonVariant::Primary));
```

Scoped overrides are opt-in and type-specific:

```rust
ui.with_override(
    (
        ButtonOverride::new().variant(ButtonVariant::Secondary),
        LabelOverride::new().tone(LabelTone::Muted),
    ),
    |ui| {
        let mut ui = ui.components();
        let _ = ui.button("Cancel");
        let _ = ui.label("Secondary copy");
    },
);
```

## Authoring Workflow

Use these commands:

- `cargo xtask new-component <name> --family <display|control|row-list|popup|composed>`
- `cargo xtask sync-llm-docs`
- `cargo xtask validate-components`

The expected implementation path is:

1. Run `cargo xtask new-component`.
2. Fill in the typed builder and keep shorthands minimal.
3. Compose `layout::*` for flow layout and `src/primitives/` helpers for chrome instead of mutating layout state inline.
4. Update the generated doc stub in `docs/llm/components/`.
5. Run sync and validation before finishing.

## If You Add A New Component

Do all of this in the same change:

1. Add `src/components/<name>.rs`.
2. Export it from `src/components/mod.rs`.
3. Re-export it from `src/lib.rs::prelude`.
4. Add it to `src/catalog.rs`.
5. Add a showcase example in `src/dev/showcase/component_showcase.rs`.
6. Add or refine the component stub in `docs/llm/components/`.
7. Reuse existing tokens and primitives before creating new visual rules.

## What To Read Next

- `authoring-rules.md` for implementation constraints.
- `components-reference.md` for the exact API and defaults of every existing component.
- `contract-reference.md` for the supported contract families, events, schema surface, and current support matrix.
