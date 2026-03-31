# Examples

Run examples from the repo root:

```bash
cargo run --example showcase
```

Use the local wrapper command for the showcase:

```bash
cargo example showcase
```

Generate isolated showcase snapshots:

```bash
cargo example snapshot --component canva-position
cargo example snapshot --all
cargo screenshot --component canva-position
```

Enable showcase hot restart:

```bash
cargo example showcase --hot
```

Available examples:

- `showcase`
  Broad component catalog and API preview surface.
- `contract-showcase`
  Host-driven declarative demo rendered entirely through `egui_component::contract::*`.
- `contract-schema`
  Prints the machine-readable contract schema JSON for Luau/codegen consumers.
- `contract-reference`
  Prints the generated human-readable contract reference from the same Rust registry.
- `content-composition`
  Reusable composition patterns with native egui stacks, vendored taffy primitives, `Card`, `Label`, `Button`, `Kbd`, and scoped overrides.
- `popup-patterns`
  Popup interaction patterns for `Tooltip`, `Popover`, `DropdownMenu`, and `Dialogue`.
- `theme-playground`
  Live controls for `ThemeSpec`, `ThemeMode`, `theme::set_theme`, `theme::set_mode`, and `theme::with_theme`.

Example commands:

```bash
cargo run --example showcase
cargo run --example contract-showcase
cargo run --example contract-schema
cargo run --example contract-reference
cargo run --example content-composition
cargo run --example popup-patterns
cargo run --example theme-playground
cargo example showcase
cargo example showcase --hot
cargo example snapshot --component canva-backgrounds
cargo example snapshot --all --theme dark
cargo screenshot --component canva-backgrounds
```

Hot mode watches the repo, rebuilds `showcase`, and relaunches the example process on change.
The window restarts on each rebuild instead of reloading code into the running process.

Snapshot mode is headless and crops the PNG to the component preview itself, without the outer showcase title/subtext/card wrapper.
