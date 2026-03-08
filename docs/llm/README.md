# Egui Component LLM Docs

This folder is the authoring guide for this crate.

Use it when you are:

- adding a new component
- editing an existing component
- generating examples
- asking an LLM to extend the library without breaking the current look and feel

Read these files in order:

1. `authoring-rules.md`
2. `components-reference.md`

## What This Library Is

`egui-component` is a small component library on top of `egui`.

The public surface is built around three ideas:

- `theme::install(&Context, ThemeMode)` installs the shared fonts, visuals, and icon loading.
- `ui.components()` wraps an `egui::Ui` in `ComponentUi`.
- Each component method accepts a typed builder and usually a few `Into<Builder>` shorthand forms.

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
- `src/components/*.rs`
  One file per component or component family.
- `src/ui/tokens.rs`
  Color, spacing, radius, and state tokens.
- `src/ui/style.rs`
  Global component-theme defaults.
- `src/components/chrome.rs`
  Shared input and slider chrome helpers.
- `src/theme.rs`
  Theme setup entry points.
- `src/catalog.rs`
  Public component registry used by the showcase and parser helpers.
- `src/dev/showcase/component_showcase.rs`
  Canonical live examples.

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
let _ = ui.text_input(&mut name, (220.0, "Name"));
let _ = ui.button(("Save", ButtonStyle::Primary));
```

Scoped overrides are opt-in and type-specific:

```rust
ui.with_override(
    (
        ButtonOverride::new().style(ButtonStyle::Secondary),
        LabelOverride::new().tone(LabelTone::Muted),
    ),
    |ui| {
        let _ = ui.button("Cancel");
        let _ = ui.label("Secondary copy");
    },
);
```

## If You Add A New Component

Do all of this in the same change:

1. Add `src/components/<name>.rs`.
2. Export it from `src/components/mod.rs`.
3. Re-export it from `src/lib.rs::prelude` if it belongs in the public prelude.
4. Add it to `src/catalog.rs`.
5. Add a showcase example in `src/dev/showcase/component_showcase.rs`.
6. Reuse existing tokens and chrome helpers before creating new visual rules.

## What To Read Next

- `authoring-rules.md` for implementation constraints.
- `components-reference.md` for the exact API and defaults of every existing component.
