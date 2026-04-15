# clay-jsx-egui-bridge

egui-component contract bridge for JSX, TS, and TSX authored surfaces.

Frozen minimal v1 host-config contract:
`docs/jsx-runtime-v1-host-config-contract.md`

Frozen minimal v1 commit protocol:
`docs/jsx-runtime-v1-commit-protocol.md`

Frozen minimal v1 scheduling model:
`docs/jsx-runtime-v1-scheduling-model.md`

Frozen v1 deferred-features list:
`docs/jsx-runtime-v1-deferred-features.md`

## Frozen v1 host-config surface

The v1 reconciler contract is intentionally small and explicit:

- mutation mode only (`supportsMutation = true`)
- microtasks enabled (`supportsMicrotasks = true`)
- hydration unsupported (`supportsHydration = false`)
- persistence unsupported (`supportsPersistence = false`)
- JS public instances are shadow nodes only (`getPublicInstance(instance) === instance`)
- direct native host refs are unsupported (`getInstanceFromNode()` / `getInstanceFromScope()` return `null`)
- structural host-config verbs are limited to create / append / insert / remove / reorder / text / hide / unhide plus microtask/timer scheduling hooks

Anything outside that surface is out of scope for v1 until the retained commit
protocol is proven stable.

## Explicit v1 deferrals

These features are intentionally deferred so they do not leak into the reliable
core architecture by accident:

- hydration
- persistence mode
- direct native host refs / public native instances
- hook-state restoration across reload
- production commit-transport replacement

See `docs/jsx-runtime-v1-deferred-features.md` for the frozen list plus the
review checklist for follow-up architecture changes.

The checked-in commit protocol freezes the session-atomic states and legal
transitions for enqueue, decode, validate, acknowledge, reject, and fail.

## Ownership contract

The retained-renderer boundary is intentional and should be treated as an
architecture invariant:

- `runtime_api.ts` owns an ephemeral React reconciler tree plus event-handler
  routing metadata in JS.
- JS does **not** own native egui objects and must not behave as if it does.
- JS can propose renderer-state changes only by sending `HostMutationBatch`
  payloads through the bridge commit transport abstraction.
- JSON remains the only production-supported backend in v1, and typed transport
  experiments stay behind explicit test/benchmark-only opt-in.
- Rust owns the retained `HostTree`, validates batches, applies them
  transactionally, and materializes `ContractTree` / `MotionFrame` outputs for
  the renderer host.
- Rust acknowledges a commit batch only after successful validation and apply;
  rejected batches force controlled session failure instead of partial native
  mutation.

The crate owns the egui-specific pieces on top of `clay-jsx-runtime`:

1. It configures the virtual `egui`, `egui/jsx-runtime`, `clay`, `clay/jsx-runtime`, `motion/react`, and `react/motion` modules on top of the shared React substrate from `clay-jsx-runtime`.
2. It lowers committed React host nodes into normalized egui contract descriptors and diffs those normalized descriptors into semantic `HostMutation` batches.
3. It installs `clay_jsx_runtime::contract` metadata into the JS runtime.
4. Rust applies those retained host mutations into a Rust-owned `HostTree`.
5. Rust exposes changed `ContractTree` values plus retained `MotionFrame` values for egui renderers to consume.
6. The bridge keeps one persistent React root per session, exports the supported React authoring hooks on the stable `egui` / `clay` surface, drains timer/effect callbacks on explicit host wake, and unmounts React cleanly on reload or teardown.

The frozen v1 scheduling model treats `drain_pending_runtime_updates()` as one
bounded wake step, not an unbounded flush loop:

- each call drains at most one wake worth of host callbacks
- `requestRepaint`, timers, RAF callbacks, and effect-scheduled async work all converge through `pending_host_wake`
- if `session.debug_metrics().runtime.pending_host_wake` remains true after a drain, the embedder must schedule another frame/drain

Retained motion is structurally separate from contract-tree updates:

- motion-only commit batches update `MotionFrame` state without forcing a fresh `ContractTree`
- `tick_motion()` advances motion state with `tree: None`
- structural commits preserve retained motion values for unaffected nodes until those nodes are replaced or removed

Performance-comparison benchmarks that assert wall-clock improvements are kept
as ignored tests so the default `cargo test` path stays reproducible across
different host machines. Run them explicitly with `cargo test -- --ignored` when
you want benchmark coverage.

The checked-in architecture gates are:

- end-to-end session flow: `cargo test -p clay-jsx-egui-bridge architecture_v1_integration_matrix_covers_load_dispatch_async_motion_reload_and_teardown -- --nocapture`
- representative-tree hot-path benchmark: `cargo run --example runtime-jsx-benchmark`
- strict CI performance gate: `CLAY_JSX_BENCH_ENFORCE_A_GRADE=1 cargo run --example runtime-jsx-benchmark`

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
- contract-tree materialization, structural no-op, motion-commit-only, motion-tick-only, teardown, and unmount counts
- nested shared-runtime wake/timer/shutdown counters from `clay-jsx-runtime`

The eframe authoring shell remains in the repository host at `src/host` with TSX/JSX entrypoints in `src/showcase`.

Supported authoring exports now include `useState`, `useEffect`, `useReducer`, `useRef`, `useContext`, `useSyncExternalStore`, `startTransition`, `useDeferredValue`, `createContext`, `render`, `eventValue`, `log`, and `requestRepaint`.

## Hot reload / state-restoration policy

V1 hot reload is intentionally conservative:

- reload validates the replacement entrypoint in an isolated runtime before the live session is touched
- if validation fails, the last good session stays live and interactive
- if validation succeeds, the previous session is deterministically torn down and unmounted, then the replacement session is installed as a cold remount
- effect cleanup from the previous session runs during that teardown before the old session is discarded
- `HotReloadState` is retained for API compatibility only; v1 capture returns the default empty shape and restore input is ignored
- hook-state restoration is explicitly deferred work, not a supported v1 guarantee

Callers should treat reload as a validated session swap, not as partial state preservation.

## Error containment / host-visible recovery policy

The bridge normalizes session-affecting failures into one host-visible recovery
contract:

- recoverable reconciler issues => `continue`
- error-boundary-contained render/effect failures => `bounded_failure`
- fatal runtime callback failures or uncaught reconciler failures => `teardown`
- commit-protocol rejection/failure => `reload_required`

`JsxRuntimeSession::debug_recovery_state()` exposes the current category,
disposition, message, component stack / boundary metadata when present, and the
rejected commit batch id for protocol failures.
