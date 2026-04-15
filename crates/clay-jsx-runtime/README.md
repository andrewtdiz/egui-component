# clay-jsx-runtime

Configurable `deno_core`/V8 runtime primitives for Clay JSX/TSX host bridges.

Frozen minimal v1 host-config contract:
`docs/jsx-runtime-v1-host-config-contract.md`

Frozen minimal v1 commit protocol:
`docs/jsx-runtime-v1-commit-protocol.md`

Frozen minimal v1 scheduling model:
`docs/jsx-runtime-v1-scheduling-model.md`

That checked-in v1 surface freezes mutation mode, JS shadow public instances,
microtask/timer scheduling hooks, and explicit non-support for hydration,
persistence, and direct native host refs.

The checked-in commit protocol freezes the session-atomic states and
transitions for enqueue, decode, validate, acknowledge, reject, and fail.

## Ownership boundary

`clay-jsx-runtime` owns the embedded `JsRuntime`, shared React runtime modules,
and the generic host ops that JS can call.

- JS owns only ephemeral reconciler state inside V8.
- JS can cross into Rust-owned renderer state only by enqueueing commit batches
  through the runtime commit transport abstraction.
- The default backend uses `Deno.core.ops.op_commit_mutations(...)` with JSON,
  while typed transport experiments stay behind the same interface.
- The other host ops are intentionally non-renderer-specific: diagnostics,
  reconciler telemetry, timers, and wake scheduling.
- Renderer crates must decode, validate, acknowledge/reject, and apply commit
  batches into their own retained structures.

This crate therefore does **not** expose JS APIs for directly creating,
retaining, or mutating native renderer objects.

This crate owns the host-neutral pieces:

1. `RuntimeSession` creates and retains a `deno_core::JsRuntime`.
2. `deno_ast` transpiles `.jsx`, `.tsx`, and `.ts` modules with a caller-provided automatic JSX import source.
3. Callers register virtual modules such as `clay`, `clay/jsx-runtime`, or renderer-specific aliases.
4. `JsxRuntimeOptions::with_react_runtime_modules()` installs the shared React, JSX runtime, reconciler, and scheduler modules for renderer bridges that want the standard React substrate.
5. It installs host-owned timer and wake globals before user modules load: `setTimeout`, `clearTimeout`, `setInterval`, `clearInterval`, `requestAnimationFrame`, and `requestRepaint`.
6. `RuntimeSession` exposes pending host wake signals and performs one bounded callback-drain batch per wake so embedders can process effect-driven, timer-driven, RAF-driven, and invalidation-only React work explicitly.
7. JS can commit JSON batches through `Deno.core.ops.op_commit_mutations(...)` and write diagnostics through `op_host_log(...)`.
8. Renderer-specific crates interpret the committed JSON and own their contract model, validation, retained tree, event dispatch semantics, and teardown behavior.

The v1 scheduling model freezes these wake rules:

- `pending_host_wake` is the one host-visible async-work bit
- one drain may invoke at most `HOST_CALLBACK_DRAIN_LIMIT` callbacks
- if more callbacks remain after a bounded batch, JS requests another wake
- if `pending_host_wake` remains true after a drain, the host must schedule another frame/drain

For ship-readiness and test harnesses, `RuntimeSession::debug_metrics()` plus `RuntimeHostCallbackDrainResult` expose host wake, callback-drain, timer, active-timer, and shutdown counters so bridges can assert cleanup and repaint behavior with hard gates instead of behavioral guesses.

The egui-component bridge lives in `crates/clay-jsx-egui-bridge`.
