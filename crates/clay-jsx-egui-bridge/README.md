# clay-jsx-egui-bridge

egui-component contract bridge for JSX, TS, and TSX authored surfaces.

The crate owns the egui-specific pieces on top of `clay-jsx-runtime`:

1. It configures the virtual `egui`, `egui/jsx-runtime`, `clay`, `clay/jsx-runtime`, `motion/react`, and `react/motion` modules on top of the shared React substrate from `clay-jsx-runtime`.
2. It lowers committed React host nodes into normalized egui contract descriptors and diffs those normalized descriptors into semantic `HostMutation` batches.
3. It installs `egui_component::contract` metadata into the JS runtime.
4. Rust applies those retained host mutations into a Rust-owned `HostTree`.
5. Rust exposes changed `ContractTree` values plus retained `MotionFrame` values for egui renderers to consume.
6. The bridge keeps one persistent React root per session, exports the supported React authoring hooks on the stable `egui` / `clay` surface, drains timer/effect callbacks on explicit host wake, and unmounts React cleanly on reload or teardown.

Minimal embedding shape:

```rust
use clay_jsx_egui_bridge::JsxRuntimeSession;

let (mut session, first_render) = JsxRuntimeSession::load("ui/app.tsx".as_ref())?;
if let Some(tree) = first_render.tree {
    // Render the ContractTree with your host.
}

let next_render = session.dispatch_events(&events)?;
if let Some(async_render) = session.drain_pending_runtime_updates()? {
    // Apply async/timer-driven React updates.
}
let motion_frame = session.tick_motion(now_secs)?.motion;
```

`JsxRuntimeSession::debug_metrics()` surfaces the retained-runtime readiness counters for ship gates and harnesses:

- React root/render and event-dispatch counts
- retained mutation batch and per-mutation-kind counts
- contract-tree materialization, no-op, motion-only, teardown, and unmount counts
- nested shared-runtime wake/timer/shutdown counters from `clay-jsx-runtime`

The eframe authoring shell remains in the repository example at `examples/runtime-jsx`.

Supported authoring exports now include `useState`, `useEffect`, `useReducer`, `useRef`, `useContext`, `useSyncExternalStore`, `startTransition`, `useDeferredValue`, `createContext`, `render`, `eventValue`, `log`, and `requestRepaint`.

Hot reload currently remounts cold on file changes. The bridge keeps `HotReloadState` as API compatibility, but hook-state restoration is deferred until a later pass. Reload and teardown still unmount React so effect cleanup runs before the session is dropped.
