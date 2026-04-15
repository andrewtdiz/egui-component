# Examples

Run examples from the repo root:

```bash
cargo run --example runtime-jsx-host
```

Available examples:

- `runtime-jsx-host`
  Embedded JSX runtime example. The host-neutral runtime lives in `crates/clay-jsx-runtime`, while the egui bridge lives in `crates/clay-jsx-egui-bridge`; JSX/TSX is transpiled with `deno_ast`, evaluated by `deno_core`/V8, committed into a Rust-owned retained host tree, and rendered by Rust-owned egui.
- `runtime-jsx-motion`
  Focused JSX motion sync example. The TSX file owns the motion specs and hook state; egui ticks the retained motion frame and draws opacity, translation, scale, and rotation values.

Example commands:

```bash
cargo run --example runtime-jsx-host
cargo run --example runtime-jsx-motion
```

For live JSX edits, run `runtime-jsx-host` and edit `examples/runtime-jsx/app.jsx` on disk. For motion sync verification, run `runtime-jsx-motion` and edit `examples/runtime-jsx/motion-sync.tsx`. The host-neutral runtime crate is `clay-jsx-runtime` at `crates/clay-jsx-runtime`; the egui bridge crate is `clay-jsx-egui-bridge` at `crates/clay-jsx-egui-bridge`.
