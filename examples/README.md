# Examples

Run examples from the repo root:

```bash
cargo run --example showcase
```

Use the local wrapper command for the showcase:

```bash
cargo example showcase
```

The wrapper commands are backed by the workspace tool crate at `tools/example-runner`.

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
  Optional host-driven declarative demo rendered entirely through `egui_component::contract::*`.
- `theme-playground`
  Live controls for `ThemeSpec`, `ThemeMode`, `theme::set_theme`, `theme::set_mode`, and `theme::with_theme`.
- `runtime-egui-host`
  Canonical Luau runtime example. Luau renders directly through a typed immediate `ui.*` bridge from `examples/runtime-luau/`, while Rust owns the `egui` frame, hot reload, and rollback.
- `showcase-runtime`
  Optional wrapper that boots the library-backed Luau showcase surface through the same direct typed runtime host.
- `component-gallery-runtime`
  Primitive Luau component gallery for the shadcn-like direct `ui.*` surface. Derived controls remain authored in Luau.

Example commands:

```bash
cargo run --example showcase
cargo run --example contract-showcase
cargo run --example theme-playground
cargo run --example runtime-egui-host
cargo run --example showcase-runtime
cargo run --example component-gallery-runtime
cargo example showcase
cargo example showcase --hot
cargo example snapshot --component canva-backgrounds
cargo example snapshot --all --theme dark
cargo screenshot --component canva-backgrounds
```

Hot mode watches the repo, rebuilds `showcase`, and relaunches the example process on change.
The window restarts on each rebuild instead of reloading code into the running process.

Snapshot mode is headless and crops the PNG to the component preview itself, without the outer showcase title/subtext/card wrapper.

For live Luau edits, run `runtime-egui-host` for the main demo surface, `showcase-runtime` for the recipe showcase, or `component-gallery-runtime` for the primitive component catalog. Runtime roots now live under `examples/runtime-luau/apps/`, while the reusable Luau library lives under `examples/runtime-luau/ui/` with one component directory at `examples/runtime-luau/ui/components/`. In all three cases Luau owns composition through the direct `ui.*` layer, while Rust owns `egui` frame execution, widget dispatch, hot reload, and rollback.

Use `contract-showcase` only when you explicitly want the alternate host-authored `ContractTree` path for declarative schema/tooling work. It is not required for the direct embedded Luau runtime path.
