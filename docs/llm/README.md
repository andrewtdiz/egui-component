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

## What This Library Is

`egui-component` is a small component library on top of `egui`.

The public runtime surface is built around three ideas:

- `theme::install(&Context, ThemeMode)` installs the shared fonts, visuals, and icon loading.
- `ui.components()` wraps an `egui::Ui` in `ComponentUi`.
- Each component method accepts a typed builder; new work should keep shorthand forms minimal.

## Non-Negotiable Design Contract

When writing or editing components in this repo, keep these rules fixed unless the user explicitly asks to redesign the system:

- Do not invent new colors when an existing token already covers the state.
- Do not change default padding, spacing, radius, or widget heights just because a component looks slightly off.
- Do not bypass `src/ui/tokens.rs` for interactive colors.
- Do not bypass `src/ui/style.rs` for global spacing and radius defaults.
- Keep components visually aligned with shadcn-style expectations: restrained variants, token-driven states, and composition over ad hoc styling.

Current global layout/style anchors:

- Item spacing Y: `8.0`
- Button padding: `12.0 x 7.0`
- Default interactive height: `34.0`
- Input padding: `10 x 6`
- Radius sm/md/lg: `6 / 8 / 10`
- Default text sizes: body `14`, heading `16`, small `12`, most component labels `12`

## File Map

Use these files as the source of truth:

- `src/components/api.rs`
  Defines `ComponentUi`, `ComponentUiExt`, and scoped override plumbing.
- `src/primitives/*.rs`
  Public authoring primitives for chrome, popup framing, row layouts, and surface helpers.
- `src/components/*.rs`
  One file per component or component family.
- `src/ui/tokens.rs`
  Color, spacing, radius, and state tokens.
- `src/ui/style.rs`
  Global component-theme defaults.
- `src/theme.rs`
  Theme setup entry points.
- `src/catalog.rs`
  Public component registry used by the showcase and parser helpers.
- `src/dev/showcase/component_showcase.rs`
  Canonical live examples.
- `docs/llm/components/*.md`
  Per-component authoring stubs used to build the generated reference.
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

- `ComponentUi`
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
3. Compose `src/primitives/` helpers instead of painting new chrome inline.
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
