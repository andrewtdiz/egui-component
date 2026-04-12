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
- `runtime-jsx-host`
  Embedded JSX runtime example. The host-neutral runtime lives in `crates/clay-jsx-runtime`, while the egui bridge lives in `crates/clay-jsx-egui-bridge`; JSX/TSX is transpiled with `deno_ast`, evaluated by `deno_core`/V8, committed into a Rust-owned retained host tree, and rendered by Rust-owned egui.
- `runtime-jsx-motion`
  Focused JSX motion sync example. The TSX file owns the motion specs and hook state; egui ticks the retained motion frame and draws opacity, translation, scale, and rotation values.
- `runtime-egui-host`
  Legacy compatibility alias for `runtime-jsx-host`.

Example commands:

```bash
cargo run --example showcase
cargo run --example contract-showcase
cargo run --example theme-playground
cargo run --example runtime-jsx-host
cargo run --example runtime-jsx-motion
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

For live JSX edits, run `runtime-jsx-host` and edit `examples/runtime-jsx/app.jsx` on disk. For motion sync verification, run `runtime-jsx-motion` and edit `examples/runtime-jsx/motion-sync.tsx`. The host-neutral runtime crate is `clay-jsx-runtime` at `crates/clay-jsx-runtime`; the egui bridge crate is `clay-jsx-egui-bridge` at `crates/clay-jsx-egui-bridge`. `runtime-egui-host` remains as a legacy alias for the same host.

Use `contract-showcase` when you explicitly want a Rust-authored `ContractTree` path for declarative schema/tooling work. Use `runtime-jsx-host` when you want the authored JSX-to-egui runtime path.
