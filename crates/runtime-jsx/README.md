# egui-component-runtime-jsx

Reusable `deno_core`/V8 host for authoring egui-component contract trees from JSX, TS, and TSX.

The crate owns the embeddable runtime pieces:

1. `JsxRuntimeSession` creates a retained `deno_core::JsRuntime`.
2. `deno_ast` transpiles `.jsx`, `.tsx`, and `.ts` modules with automatic JSX runtime imports.
3. The virtual `egui`, `egui/jsx-runtime`, `motion/react`, and `react/motion` modules are loaded from embedded JS.
4. JS commits host mutations into a Rust-owned retained `HostTree`.
5. Rust exposes changed `ContractTree` values plus retained `MotionFrame` values for renderers to consume.

Minimal embedding shape:

```rust
use egui_component_runtime_jsx::JsxRuntimeSession;

let (mut session, first_render) = JsxRuntimeSession::load("ui/app.tsx".as_ref())?;
if let Some(tree) = first_render.tree {
    // Render the ContractTree with your host.
}

let next_render = session.dispatch_events(&events)?;
let motion_frame = session.tick_motion(now_secs)?.motion;
```

The eframe authoring shell remains in the repository example at `examples/runtime-jsx`.
