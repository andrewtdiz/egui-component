# clay-jsx-egui-bridge

egui-component contract bridge for JSX, TS, and TSX authored surfaces.

The crate owns the egui-specific pieces on top of `clay-jsx-runtime`:

1. It configures the virtual `egui`, `egui/jsx-runtime`, `clay`, `clay/jsx-runtime`, `motion/react`, and `react/motion` modules.
2. It installs `egui_component::contract` metadata into the JS runtime.
3. JS commits retained host mutations into a Rust-owned `HostTree`.
4. Rust exposes changed `ContractTree` values plus retained `MotionFrame` values for egui renderers to consume.

Minimal embedding shape:

```rust
use clay_jsx_egui_bridge::JsxRuntimeSession;

let (mut session, first_render) = JsxRuntimeSession::load("ui/app.tsx".as_ref())?;
if let Some(tree) = first_render.tree {
    // Render the ContractTree with your host.
}

let next_render = session.dispatch_events(&events)?;
let motion_frame = session.tick_motion(now_secs)?.motion;
```

The eframe authoring shell remains in the repository example at `examples/runtime-jsx`.
