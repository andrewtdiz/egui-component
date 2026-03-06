# Authoring Rules

This file is the implementation contract for contributors and coding agents.

Its goal is simple: add or edit components without drifting from the current design system.

## Primary Standard

Follow the spirit of shadcn component authoring:

- small public APIs
- predictable builder structs
- token-driven visuals
- composition from primitives
- restrained variants

In this repo that means:

- do not improvise colors
- do not improvise padding
- do not improvise radius values
- do not add variants unless the component truly needs them

## Required Runtime Entry Points

Use these APIs instead of rolling your own setup path:

- `egui_component::theme::setup(&Context)`
- `egui_component::theme::apply_component_theme(ui)`
- `ComponentUiExt::components()`

Do not create a parallel wrapper abstraction unless the user explicitly wants a redesign.

## Required Styling Sources

Before adding a new color, spacing rule, or shape rule, check these files first:

- `src/ui/tokens.rs`
- `src/ui/style.rs`
- `src/components/chrome.rs`

Prefer existing tokens in this order:

1. semantic text tokens like `TEXT_PRIMARY`, `TEXT_SECONDARY`, `TEXT_MUTED`, `TEXT_DESTRUCTIVE`
2. input state tokens like `INPUT_BACKGROUND`, `INPUT_HOVER_BACKGROUND`, `INPUT_BORDER`
3. row state helpers like `row_bg`, `row_stroke`, `row_selected_text`
4. primary action helpers like `primary_bg`, `primary_hover_bg`, `primary_fg`

## Fixed Visual Invariants

Unless the user asks for a design-system change, preserve these values:

| Token | Value |
| --- | --- |
| `SPACING_ITEM_Y` | `8.0` |
| `SPACING_BUTTON_PADDING_X` | `12.0` |
| `SPACING_BUTTON_PADDING_Y` | `7.0` |
| `SPACING_INTERACT_HEIGHT` | `34.0` |
| `INPUT_PADDING_X` | `10` |
| `INPUT_PADDING_Y` | `6` |
| `RADIUS_SM` | `6` |
| `RADIUS_MD` | `8` |
| `RADIUS_LG` | `10` |

Practical consequence:

- buttons should continue to feel like the current button system
- inputs should continue to use the current input chrome
- menus and popups should continue to match current row heights and corner treatment

## Public API Pattern

Every component in this repo follows roughly the same shape:

1. A small builder struct with public fields.
2. `new(...)` plus chainable setters.
3. A few `From<_>` shorthand implementations for common use.
4. A `ComponentUi` method that accepts `impl Into<Builder>`.
5. External state passed in by mutable reference when needed.

Prefer that pattern over custom traits or macro-heavy abstractions.

## Builder Rules

Use these rules when adding or editing builders:

- Make the builder easy to construct from the most common case.
- Clamp obviously invalid numeric values to a safe minimum when the existing code does that.
- Add shorthand `From<_>` forms only for common, obvious call sites.
- Do not create tuple shorthands that are hard to read or easy to misorder.
- Keep defaults close to current component defaults in this repo.

Good:

```rust
pub struct Tooltip<'a> {
    pub trigger_label: &'a str,
    pub text: &'a str,
    pub width: f32,
}
```

Bad:

```rust
pub struct Tooltip<'a> {
    pub a: &'a str,
    pub b: &'a str,
    pub style_mode: i32,
    pub custom_padding: Option<(f32, f32)>,
}
```

## State Ownership Rules

Interactive state stays with the caller unless `egui` itself needs temporary UI-local data.

Use caller-owned state for:

- selected indices
- open flags
- text input values
- boolean toggles
- numeric values

Use `Ui::data` or response-local state only for short-lived UI details such as:

- scoped overrides
- hover timestamps
- popup bookkeeping already tied to `egui::Id`

Do not hide durable component state inside global statics.

## Composition Rules

Prefer composition when a component is derived from existing primitives.

Current examples:

- `Field` = label + input + helper text
- `Combobox` = input + card + filterable rows
- `Command` = input + grouped list
- `Dialogue` = trigger button + modal + dialogue header/actions

If a new component can be built from existing primitives without awkward behavior, do that first.

## Override Rules

Scoped overrides already exist for:

- `Button`
- `Card`
- `Label`
- `TextInput`

Use the existing override mechanism in `src/components/api.rs` if the new behavior is truly a scoped styling override.

Do not add overrides for every component by default.

Add a new override type only when:

- the component is reused heavily
- scoped style changes improve ergonomics
- the override can stay small and predictable

## Interaction Rules

Mirror the current interaction conventions:

- clickable things use `CursorIcon::PointingHand`
- text inputs use `CursorIcon::Text`
- drag-based number inputs use axis resize cursors
- focus states come from input or row token helpers
- selected rows use `row_bg`, `row_stroke`, and `row_selected_text`

Do not invent custom hover/focus behavior per component unless there is a real interaction need.

## Popup And Modal Rules

For popups, menus, and modals:

- use stable `egui::Id` values
- keep width defaults aligned with existing components
- use cards/rows that visually match `Select`, `DropdownMenu`, `Combobox`, and `Command`
- close popups through normal `egui` close behavior when possible

Current default widths:

- input/select/dropdown/combobox/command: about `220.0`
- dialogue modal: `360.0`

## Typography Rules

The library uses the theme installed by `src/ui/style.rs`.

Practical rules:

- default labels are typically `12.0`
- body/button text is `14.0`
- helper text is usually `11.0`
- semibold labels use the named semibold font family already configured in the theme

Do not introduce random font families or one-off type scales.

## Chrome Rules

Reuse shared chrome helpers before writing component-local visuals:

- `with_input_chrome(ui, ...)`
- `with_slider_chrome(ui, ...)`

These helpers already encode the expected background, border, focus, and radius behavior.

If a new control belongs to the same family as input or slider controls, extend those helpers only if the change benefits multiple widgets.

## Do Not Do This

- Do not hardcode ad hoc colors when tokens exist.
- Do not change padding to "make it look nicer" without a broader design-system reason.
- Do not introduce a new radius just for one component.
- Do not use raw default `egui` widgets and accept their default visuals if the component is meant to match this library.
- Do not add a variant because another library has one.
- Do not overfit the public API to a single showcase example.
- Do not skip catalog/showcase updates for public components.

## New Component Checklist

Use this checklist before considering the work complete:

- The component lives in `src/components/<name>.rs`.
- It exports a focused builder struct.
- It has a `ComponentUi` method.
- It uses existing tokens and existing chrome helpers where appropriate.
- It owns no hidden durable state.
- It has only the minimum useful `From<_>` shorthands.
- It is exported from `src/components/mod.rs`.
- It is re-exported from `src/lib.rs::prelude` if appropriate.
- It is registered in `src/catalog.rs`.
- It has a showcase example.
- It does not change the system's color, spacing, or radius language.

## Editing Existing Components

When updating a current component, preserve these things unless the task explicitly says otherwise:

- default width
- default height
- default text size
- current variant names
- tuple shorthand call sites already used in the README/showcase
- token choices for hover, active, selected, and focus

If behavior must change, prefer changing composition or state handling before changing the visual contract.
