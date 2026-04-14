# Authoring Rules

Use these rules when you add or revise a component.

## Runtime Contract

- Keep the active public runtime surface centered on `theme::*`, `layout::*`, `contract::*`, and the JSX runtime crates under `crates/clay-jsx-runtime` and `crates/clay-jsx-egui-bridge`.
- Treat `ui.components()` and the typed builders as deprecated compatibility shims over the internal runtime renderer. Keep them working, but do not expand that public surface unless explicitly asked.
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

- Every internal runtime component must be exported from `src/components/mod.rs`.
- The deprecated `src/lib.rs::components` and `src/lib.rs::prelude` shims should continue to point at the same implementation unless the user explicitly requests a break.
- Every user-facing JSX surface must have migration-manifest coverage and a JSX catalog preview; keep the Rust showcase wiring only when maintaining the deprecated demo surface or internal renderer tests.
