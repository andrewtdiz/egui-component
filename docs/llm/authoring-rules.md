# Authoring Rules

Use these rules when you add or revise a component.

## Runtime Contract

- Keep the typed runtime surface centered on `theme::install`, `theme::set_mode`, `ui.components()`, and typed component builders.
- Keep existing shipped builder entry points working unless the user explicitly asks for a public API break.
- Do not add new multi-field tuple shorthand permutations. Prefer one dominant shorthand at most.

## Authoring Contract

- Use `src/primitives/` before adding new paint or layout helpers inside a component.
- Add a new primitive only when at least two components need the same behavior.
- Keep style resolution local to the component with a private `resolve_*_style` helper.
- Use named semantic fields for new work: `variant`, `size`, `tone`, `intent`, `leading_icon`, `trailing_text`, and closures for compound sections when they fit.

## Visual Contract

- Use `src/ui/tokens.rs` for colors, spacing, radii, and shared state values.
- Use `src/ui/style.rs` for global defaults.
- Do not hardcode runtime `Color32::from_*` values inside `src/components/*.rs`.
- Reuse the shared popup, row, control, and surface primitives before inventing new chrome.

## Tooling Contract

- Scaffold with `cargo xtask new-component`.
- Regenerate the LLM reference with `cargo xtask sync-llm-docs`.
- Finish with `cargo xtask validate-components`.

## Registry Contract

- Every public component must be exported from `src/components/mod.rs`.
- Every public component must be re-exported from `src/lib.rs::prelude`.
- Every public component must have a catalog definition, a showcase metadata entry, a showcase render arm, and a `docs/llm/components/<id>.md` stub.
