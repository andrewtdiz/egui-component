# Luau Runtime Architecture

This document describes the shipped architecture of the repository's Luau-driven, Rust-hosted immediate-mode UI runtime.

The canonical path is:

- runtime core: [`luau-runtime/src/runtime.rs`](/home/andy/Documents/egui-component/luau-runtime/src/runtime.rs)
- file watcher: [`luau-runtime/src/watch.rs`](/home/andy/Documents/egui-component/luau-runtime/src/watch.rs)
- `egui` host: [`src/example_apps/runtime_egui_host.rs`](/home/andy/Documents/egui-component/src/example_apps/runtime_egui_host.rs)
- example scripts: [`examples/runtime-luau/`](/home/andy/Documents/egui-component/examples/runtime-luau)

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

The most important seam in the runtime is not a value channel. It is the frame-local host bridge installed by [`with_host_scope`](/home/andy/Documents/egui-component/luau-runtime/src/runtime.rs).

The hot path is:

1. Rust opens the `egui` frame.
2. Rust creates a frame-local `RuntimeHost`.
3. Luau `render(state)` runs in the single shared VM.
4. Each `ui.*` call dispatches directly into the current `egui::Ui`.
5. Common widget results return immediately to Luau.
6. Rust closes the frame.

The most important methods on that bridge are:

- `ui.label(text, options)`
- `ui.separator()`
- `ui.button(id, text, options) -> clicked`
- `ui.text_edit(id, value, options) -> { value, changed }`
- `ui.horizontal(options, fn)`
- `ui.vertical(options, fn)`
- `ui.card(options, fn)`
- `ui.window(id, title, options, fn)`

That seam is where runtime performance, memory behavior, and correctness matter most.

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

## Frame Execution Flow

At a high level, a frame looks like this:

1. The host drains watcher notifications into the reload queue.
2. If needed, the runtime reloads affected modules before rendering.
3. The host enters `egui`.
4. The host installs scoped `app.*` and `ui.*` bindings for the current call.
5. Luau executes `render(state)`.
6. `ui.*` calls render widgets immediately.
7. Button clicks and text edits return results immediately.
8. The host shows any runtime or compile error overlay after the scripted surface.

Important invariants:

- there is one Luau VM on the main UI thread
- the watcher never touches the VM
- reload only happens at frame boundaries
- the last known-good graph stays active if reload fails
- frame-local `ui.*` bindings never outlive the current render call

## Runtime Core

[`ScriptRuntime`](/home/andy/Documents/egui-component/luau-runtime/src/runtime.rs) owns:

- the single `Lua` VM
- the active module graph
- the reload queue
- compile/runtime error state
- coarse reload metrics

Its responsibilities are:

- load the root script
- resolve `require(...)` within the project tree
- keep per-module persistent `state` tables
- install scoped `app.*` and `ui.*` bindings
- rebuild only affected modules on reload
- clone state into candidate modules before commit
- swap graphs only after a successful rebuild
- preserve the old graph on compile or runtime failure

The script contract stays narrow:

- optional `init(state)`
- required `render(state)`
- optional `reload(old_exports, state)`
- optional `shutdown(state)`

## Reload And Watcher Model

The watcher is intentionally simple.

[`RootScriptWatcher`](/home/andy/Documents/egui-component/luau-runtime/src/watch.rs) only:

- watches the Luau project tree
- normalizes changed paths
- enqueues dirty module paths
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
5. Commit only on success.
6. Call `shutdown` on replaced modules after the swap.

Manual reload with an empty dirty queue means: rebuild the active Luau project from disk.

## Performance-Critical Areas

The hot path is the direct `ui.*` dispatch into `egui`.

The most performance-sensitive areas are:

- scoped host callback dispatch
  This happens on every widget call and must stay small.
- frame-local `egui::Ui` ownership
  Nested container callbacks must preserve the current active `Ui` without leaking scope.
- immediate widget result return
  Buttons and text edits should flow straight back into Luau without extra queues.
- reload graph rebuild
  Not frame-hot, but expensive and correctness-critical when it runs.
- state cloning during reload
  This protects rollback and last-known-good behavior.

The architecture deliberately avoids a larger per-frame tax:

- no `RuntimeValue -> ContractTree` decode on every frame
- no `ContractEvent` re-encoding loop for ordinary controls
- no retained UI tree allocation in the runtime hot path

## Memory-Management Concerns

The highest-risk memory areas are:

- Lua table lifetime across graph swaps
- persistent state cloning during reload
- stale module exports staying reachable after failed rebuilds
- frame-local host state escaping the render call

The runtime manages those risks by:

- committing new graphs only after a full successful rebuild
- keeping the old graph alive until the replacement is known-good
- cloning script state before swap instead of mutating live state in place
- keeping the `RuntimeHost` scoped to the active runtime call
- keeping the `egui::Ui` pointer frame-local and restoring it with an RAII guard for nested containers

## Correctness Invariants

These invariants are core to the runtime:

- Luau only runs on the main UI thread.
- The watcher never touches the VM.
- Manual reload with no dirty queue rebuilds the active graph from disk.
- Unrelated `.luau` files do not churn the active graph.
- Failed reload never destroys the last known-good graph.
- Compile failures and runtime failures are tracked separately.
- Immediate widget results are not replayed on the next frame.
- Scoped `ui.*` bindings do not leak across runtime calls.

When these invariants fail, the runtime does not degrade gracefully. It becomes stale, duplicated, nondeterministic, or unsafe.

## Practical Rule For Future Work

If a change affects:

- `RuntimeHost`
- frame-local `ui.*` dispatch
- nested container scope handling
- graph swap behavior
- reload dependency resolution
- state cloning semantics

then it is touching the core runtime bridge and should be treated as performance-, memory-, and correctness-critical work.

If a change only adds new Luau-side composition helpers or richer style helpers on top of the existing typed `ui.*` surface, it is product work built on top of the bridge rather than a change to the bridge itself.
