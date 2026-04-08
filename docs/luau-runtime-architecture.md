# Luau Runtime Architecture

This document describes the shipped architecture of the repository's Luau-driven, Rust-hosted immediate-mode UI runtime.

The canonical path is:

- runtime facade: [`luau-runtime/src/runtime/mod.rs`](/home/andy/Documents/egui-component/luau-runtime/src/runtime/mod.rs)
- runtime types and metrics: [`luau-runtime/src/runtime/types.rs`](/home/andy/Documents/egui-component/luau-runtime/src/runtime/types.rs)
- runtime host bridge: [`luau-runtime/src/runtime/host.rs`](/home/andy/Documents/egui-component/luau-runtime/src/runtime/host.rs)
- runtime graph and reload pipeline: [`luau-runtime/src/runtime/graph.rs`](/home/andy/Documents/egui-component/luau-runtime/src/runtime/graph.rs)
- runtime parsing and validation: [`luau-runtime/src/runtime/parse.rs`](/home/andy/Documents/egui-component/luau-runtime/src/runtime/parse.rs)
- file watcher: [`luau-runtime/src/watch.rs`](/home/andy/Documents/egui-component/luau-runtime/src/watch.rs)
- `egui` host: [`src/example_apps/runtime_egui_host.rs`](/home/andy/Documents/egui-component/src/example_apps/runtime_egui_host.rs)
- example scripts: [`examples/runtime-luau/`](/home/andy/Documents/egui-component/examples/runtime-luau)

## Non-Negotiable Invariants

The runtime assumes these invariants everywhere:

- one long-lived Luau VM
- main-thread VM execution only
- frame-boundary reload only
- last-known-good rollback
- no host UI pointer or frame-local `egui::Ui` state escapes into Luau

## Core Model

The runtime is built around one decision:

- Rust owns the frame.
- Luau runs inside that frame.
- Luau calls a narrow typed `ui.*` API directly.
- Rust executes those calls immediately against the current `egui::Ui`.

This is not a retained document renderer. It is an in-process immediate bridge.

The responsibilities are split like this:

- Luau owns composition, view helpers, styling choices, and hot-edited application logic.
- Rust owns `egui` frame lifetime, widget execution, reload orchestration, filesystem watching, and rollback.

## The Critical Bridge

The most important seam in the runtime is not a value channel. It is the frame-local host bridge installed by [`with_app_host_scope`](/home/andy/Documents/egui-component/luau-runtime/src/runtime/host.rs) and [`with_ui_host_scope`](/home/andy/Documents/egui-component/luau-runtime/src/runtime/host.rs).

The hot path is:

1. Rust opens the `egui` frame.
2. Rust mounts the surface with a `SurfaceMount` and creates a scoped `RuntimeAppHost` or `RuntimeUiHost`.
3. If the runtime is already committed, Luau optionally runs `update(state, input)` with a reserved empty input table.
4. Luau runs `render(state)` in the single shared VM.
5. Each `ui.*` call dispatches directly into the current `egui::Ui`.
6. Common widget results return immediately to Luau.
7. Deferred frame commands flush only after a successful frame.
8. Rust closes the frame.

The most important methods on that bridge are:

- `ui.label(text, options)`
- `ui.separator()`
- `ui.button(id, text, options) -> clicked`
- `ui.text_edit(id, value, options) -> { value, changed }`
- `ui.begin_row(options)`
- `ui.begin_column(options)`
- `ui.begin_card(options)`
- `ui.end_scope()`
- `ui.push_id(id)`
- `ui.pop_id()`

That seam is where runtime performance, memory behavior, and correctness matter most.

The canonical signatures, prop schemas, and enum tokens for this bridge are generated from Rust source of truth:

- [`docs/luau-runtime-api-reference.md`](/home/andy/Documents/egui-component/docs/luau-runtime-api-reference.md)
- [`ui/core/types.luau`](/home/andy/Documents/egui-component/examples/runtime-luau/ui/core/types.luau)

`ui.end_scope()` is used instead of `ui.end()` because `end` is a reserved Luau keyword.

Identity and props are now part of the enforced runtime contract:

- every stateful widget must take an explicit local id
- full widget identity is `surface_id + explicit push_id stack + local_id`
- layout scopes such as row, column, and card do not participate in widget identity
- current `props` tables are closed schemas, and unknown keys are runtime errors

## Why This Bridge Matters Most

Every frame crosses it. Every interaction crosses it. Every hot-reloaded script eventually executes through it.

If this bridge is wrong, the runtime fails in the worst possible place:

- per-frame overhead climbs
- widget state becomes inconsistent
- nested UI scopes break
- reload safety becomes irrelevant because the frame path is already unstable

The core runtime therefore treats this bridge as the most constrained surface in the system:

- no general reflective widget API
- no per-frame document serialization
- no next-frame feedback loop for ordinary controls
- no host object graph escaping the frame
- no callback-retained container scopes in the shipped MVP path
- no open-ended prop dictionaries in hot widget calls

## Frame Execution Flow

At a high level, a frame looks like this:

1. The host drains watcher notifications into the reload queue.
2. If needed, the runtime stages a candidate graph before rendering.
3. The host enters `egui`.
4. The host installs scoped `app.*` and `ui.*` bindings for the mounted surface and active phase.
5. Luau validates staged candidates with `render(state)` only; committed steady-state frames run optional `update(state, input)` and then `render(state)`.
6. `ui.*` calls render widgets immediately.
7. Button clicks and text edits return results immediately.
8. Deferred frame commands flush only after a successful frame.
9. If a candidate was staged, the runtime commits it only after that first frame succeeds.
10. The host shows any structured runtime failure overlay after the scripted surface.

Important invariants:

- there is one Luau VM on the main UI thread
- the watcher never touches the VM
- reload only happens at frame boundaries
- the last known-good graph stays active until a staged candidate survives its first frame
- frame-local `ui.*` bindings are guarded by an active frame token and mounted surface id
- stable widget continuity depends only on `surface_id`, explicit `ui.push_id(...)` scopes, and the widget's local id

## Runtime Core

[`ScriptRuntime`](/home/andy/Documents/egui-component/luau-runtime/src/runtime/mod.rs) owns:

- the single `Lua` VM
- the active module graph
- the staged candidate graph
- the reload queue
- compile/runtime error state
- structured failure telemetry
- coarse reload metrics
- lifecycle timing/counter metrics for load, reload, first-after-load render, first-after-reload render, and steady-state render

Its responsibilities are:

- load the root script
- resolve `require(...)` within the project tree
- keep per-module persistent `state` tables
- install scoped `app.*` and `ui.*` bindings
- validate active-frame liveness and capability policy on every bound host callback
- commit or roll back staged candidates after their first rendered frame
- classify compile/load, init/reload, update, render, host-callback panic, and command-flush failures
- enforce the current public capability contract for `log`, `reload`, and `repaint`
- rebuild only affected modules on reload
- clone state into candidate modules before commit
- swap graphs only after a successful rebuild
- preserve the old graph on compile or runtime failure

Important contract details:

- rollback guarantees runtime selection, not full VM-state isolation
- active and staged graphs share one long-lived Lua heap
- unaffected modules may be reused by handle during reload
- reloadable `state` must be plain-data tables containing only tables, booleans, integers, numbers, and strings
- functions, userdata, threads, and similar values are not reloadable state
- mutable reload-persistent data belongs in `state`
- module exports should be treated as immutable by convention
- shared helpers should be stateless where practical because module-local caches live in shared heap memory

The script contract stays narrow:

- optional `init(state)`
- optional `update(state, input)`
- required `render(state)`
- optional `reload(old_exports, state)`
- optional `shutdown(state)`

Current phase policy:

- staged first-frame validation runs `render` only
- `update` receives a fresh reserved empty table as `input` on committed steady-state frames
- `ui.*` is render-only
- `app.request_reload()` and `app.request_repaint()` are deferred frame commands
- `app.log(...)` remains immediate

Current shutdown policy:

- commit happens before old-module `shutdown`
- `shutdown` runs against the old require snapshot
- shutdown failure is surfaced, but it does not roll back the committed runtime

## Reload And Watcher Model

The watcher is intentionally simple.

[`RootScriptWatcher`](/home/andy/Documents/egui-component/luau-runtime/src/watch.rs) only:

- watches the Luau project tree
- normalizes changed paths
- coalesces dirty module paths into one distinct batch per reload boundary
- preserves root reloads for rescan-style watcher events
- requests repaint

It never:

- compiles Luau
- mutates the VM
- decides reload order
- commits module swaps

All meaningful reload work stays on the main thread.

The reload path is:

1. Drain queued dirty paths.
2. Compute the affected module set from the active dependency graph.
3. Build a candidate graph for the affected modules.
4. Re-run `reload` / `init` hooks on the candidate modules.
5. Validate the candidate with a render-only first frame.
6. Commit only after that frame succeeds.
7. Call `shutdown` on replaced modules after the swap.

Manual reload with an empty dirty queue means: rebuild the active Luau project from disk.

## Performance-Critical Areas

The hot path is the direct `ui.*` dispatch into `egui`.

The most performance-sensitive areas are:

- scoped host API dispatch
  This happens on every widget call and must stay small.
- frame-local `egui::Ui` ownership
  The explicit scope stack must preserve the current active `Ui` without leaking scope or retaining frame-local containers.
- immediate widget result return
  Buttons and text edits should flow straight back into Luau without extra queues.
- reload graph rebuild
  Not frame-hot, but expensive and correctness-critical when it runs.
- state cloning during reload
  This protects rollback and last-known-good behavior.

The current local baseline capture and reproduction commands live in [`docs/luau-runtime-phase0-baseline.md`](/home/andy/Documents/egui-component/docs/luau-runtime-phase0-baseline.md).

The architecture deliberately avoids a larger per-frame tax:

- no `RuntimeValue -> ContractTree` decode on every frame
- no `ContractEvent` re-encoding loop for ordinary controls
- no retained UI tree allocation in the runtime hot path

## Memory-Management Concerns

The highest-risk memory areas are:

- Lua table lifetime across graph swaps
- persistent state cloning during reload
- stale module exports staying reachable after failed rebuilds
- stale bound callbacks escaping the render call

The runtime manages those risks by:

- committing new graphs only after a full successful rebuild
- keeping the old graph alive until the replacement is known-good
- cloning script state before swap instead of mutating live state in place
- invalidating the active frame token before scoped globals are restored
- keeping the `egui::Ui` scope stack frame-local inside the render host

Because the runtime stays inside one Lua heap, this is last-known-good runtime rollback rather than transactional VM rollback.

## Correctness Invariants

These invariants are core to the runtime:

- Luau only runs on the main UI thread.
- The watcher never touches the VM.
- Manual reload with no dirty queue rebuilds the active graph from disk.
- Unrelated `.luau` files do not churn the active graph.
- Failed reload never destroys the last known-good graph.
- Compile failures and runtime failures are tracked separately.
- Immediate widget results are not replayed on the next frame.
- Scoped `ui.*` bindings fail closed after the active frame is invalidated.

When these invariants fail, the runtime does not degrade gracefully. It becomes stale, duplicated, nondeterministic, or unsafe.

## Practical Rule For Future Work

If a change affects:

- `RuntimeAppHost` / `RuntimeUiHost`
- frame-local `ui.*` dispatch
- nested container scope handling
- `SurfaceMount` capability policy
- graph swap behavior
- reload dependency resolution
- state cloning semantics

then it is touching the core runtime bridge and should be treated as performance-, memory-, and correctness-critical work.

If a change only adds new Luau-side composition helpers or richer style helpers on top of the existing typed `ui.*` surface, it is product work built on top of the bridge rather than a change to the bridge itself.
