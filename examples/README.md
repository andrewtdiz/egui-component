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
- `theme-playground`
  Live controls for `ThemeSpec`, `ThemeMode`, `theme::set_theme`, `theme::set_mode`, and `theme::with_theme`.
- `runtime-core-demo`
  Standalone Luau runtime core demo with init, update, deferred callbacks, reload, and final.
  Uses the reusable `luau-runtime-core` crate directly.
- `runtime-egui-host`
  Tiny egui adapter that drives the shared `luau-runtime-core` package and hot-reloads `examples/runtime-luau/demo.luau` from filesystem events.

Example commands:

```bash
cargo run --example showcase
cargo run --example contract-showcase
cargo run --example theme-playground
cargo run --example runtime-core-demo
cargo run --example runtime-egui-host
cargo example showcase
cargo example showcase --hot
cargo example snapshot --component canva-backgrounds
cargo example snapshot --all --theme dark
cargo screenshot --component canva-backgrounds
```

Hot mode watches the repo, rebuilds `showcase`, and relaunches the example process on change.
The window restarts on each rebuild instead of reloading code into the running process.

Snapshot mode is headless and crops the PNG to the component preview itself, without the outer showcase title/subtext/card wrapper.

For live Luau edits, run `runtime-egui-host` and change `examples/runtime-luau/demo.luau` while the window is open.
