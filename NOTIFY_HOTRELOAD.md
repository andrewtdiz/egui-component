# `notify` Hot Reload Plan For `luau-runtime-core`

This document turns [luau-egui-runtime-mvp.md](/home/andy/Documents/egui-component/luau-egui-runtime-mvp.md) into an implementation plan for this repository.

The intent is to ship a small, verifiable `luau-runtime-core` MVP first, then layer `notify`-driven hot reload on top without collapsing into a large one-shot design.

## Guiding Rules

- Keep one Luau VM on the main UI thread for the MVP.
- Reload only at frame boundaries.
- The watcher thread must never touch the VM directly.
- Start with one root script file before supporting module graphs.
- Keep the last known-good script version alive on compile or runtime failure.
- Keep `mlua` or any Luau backend private to the crate API.
- Support root viewport only in the MVP.
- Do not bind all of `egui-component`; start with a tiny typed host API.

## Current Repo Blockers

- Root `Cargo.toml` points to `luau-runtime-core`, but the crate directory is [luau-runtime](/home/andy/Documents/egui-component/luau-runtime).
- [luau-runtime/src/lib.rs](/home/andy/Documents/egui-component/luau-runtime/src/lib.rs) declares `mod runtime;`, but `runtime.rs` does not exist yet.
- Docs mention `runtime-core-demo` and `runtime-egui-host`, but those examples are not present yet.

Phase 0 should fix these before any runtime work starts.

## Phase 0: Buildable Scaffold

Goal: restore a clean workspace build and create the minimum crate skeleton.

Implement:

- Make the crate path consistent.
  Choose one:
  rename `luau-runtime/` to `luau-runtime-core/`, or update workspace and dependency paths to `luau-runtime/`.
- Add the missing runtime module and compile-safe public stubs.
- Define the first stable internal types:
  - `RuntimeConfig`
  - `ScriptRuntime`
  - `ScriptSource`
  - `ReloadQueue`
  - `RuntimeError`
- Add one smoke test that proves the crate links and the runtime can be constructed.

Do not implement yet:

- `notify`
- custom `require`
- egui bindings

Verification:

- `cargo check`
- `cargo test -p luau-runtime-core`

Exit criteria:

- The workspace builds from repo root.
- The runtime crate has a real module layout instead of stubs in docs only.

## Phase 1: Single-File Runtime With Manual Reload

Goal: prove Luau VM lifetime, script contract, and state persistence before adding filesystem complexity.

Implement:

- Load one root script from a file path or source string.
- Support the narrow module contract:
  - optional `init(state)`
  - required `render(state)`
  - optional `reload(old_exports, state)`
  - optional `shutdown(state)`
- Create one persistent root `state` table that survives reload.
- Add explicit host entry points:
  - `load_root(...)`
  - `reload_now()`
  - `render_frame(...)`
- Swap to new exports only after compile and load succeed.
- Preserve the previous version if compile or load fails.
- Store `last_compile_error` and `last_runtime_error` separately.

Implementation notes:

- Keep the public API backend-agnostic even if the MVP uses `mlua` internally.
- Do not add module dependency tracking yet.
- Do not add filesystem watching yet.

Verification:

- Add a smoke test with a root script that increments persistent state.
- Add a tiny manual reload path in a host example or test harness.
- Confirm that a syntax error does not destroy the previous running version.

Exit criteria:

- A single Luau file can load, render, reload, and preserve state.

## Phase 2: Minimal egui Host Adapter

Goal: prove the immediate-mode bridge works inside an actual `egui` frame.

Implement:

- Add a tiny host adapter example for this repo.
- Introduce a frame-local bridge similar to `FrameScope`.
- Run script `render(state)` from inside `egui_ctx.run(...)`.
- Start with a very small binding surface:
  - `app.log`
  - `app.request_reload`
  - `app.request_repaint`
  - `ui.window`
  - `ui.horizontal`
  - `ui.vertical`
  - `ui.label`
  - `ui.separator`
  - `ui.button`
  - `ui.text_edit`
- Add an in-app error overlay window for runtime and compile errors.

Do not implement yet:

- full `egui-component` surface area
- multi-window viewport support
- arbitrary callback storage

Verification:

- Add `runtime-egui-host` as the canonical demo.
- Use a small script with a counter and text field.
- Confirm nested Luau callbacks can enter egui container closures without stack corruption.

Exit criteria:

- A Luau-authored UI can run each frame in egui and survive manual reloads.

## Phase 3: `notify` Hot Reload For One Root File

Goal: add reliable file-backed hot reload without introducing module-graph complexity yet.

Implement:

- Add a file-backed script provider rooted at one canonical script path.
- Add a watcher thread using `notify`.
- Normalize watched paths before enqueueing them.
- Push dirty paths into a thread-safe queue only.
- Drain the queue on the main thread before the next frame.
- Coalesce duplicate events per frame.
- Trigger `reload_now()` only from the main thread.
- Request repaint when a dirty event arrives.
- Keep the last known-good script version running if reload fails.

Recommended simplifications:

- Treat create, modify, rename, and remove as "root became dirty".
- Use a simple debounce or per-frame coalescing step.
- Log ignored events rather than overfitting to every `notify` edge case.

Verification:

- Run `runtime-egui-host`.
- Edit the root Luau file and confirm the UI updates live.
- Save an invalid file and confirm:
  - the old UI keeps running
  - the error overlay updates
  - fixing the file recovers cleanly

Exit criteria:

- One script file hot reloads reliably from filesystem changes during an egui session.

## Phase 4: Custom `require` And Dependency-Aware Reload

Goal: move from a demo loop to a real multi-file Luau project shape.

Implement:

- Add a custom `require` rooted to an allowed script directory.
- Build a module cache with:
  - `ModuleId`
  - path index
  - dependency set
  - reverse dependency set
  - per-module state
- Track which modules were loaded by which importer.
- On file change, compute the affected reload set.
- Compile the entire affected subgraph before committing the swap.
- Keep old modules alive if any file in the reload batch fails.
- Call module `reload(...)` hooks after a successful swap.

Do not implement yet:

- background script execution
- native module loading
- generic host object reflection

Verification:

- Add a root module plus at least one leaf dependency.
- Edit only the leaf file and confirm the correct dependents reload.
- Save a broken dependency and confirm the previous graph remains active.

Exit criteria:

- Multi-file Luau UIs hot reload coherently and preserve per-module state.

## Phase 5: Hardening And Team Adoption

Goal: make the MVP usable and maintainable for daily engineering work.

Implement:

- Add integration tests for:
  - state preservation across reload
  - failure rollback
  - dependency reload ordering
  - watcher queue coalescing
- Add reload metrics and structured logs:
  - file changed
  - reload scheduled
  - reload succeeded
  - reload failed
  - reload duration
- Add example scripts under `examples/runtime-luau/`.
- Document the script contract and host API next to the example.
- Keep the runtime API narrow until repeated product needs justify expansion.

Verification:

- Team can run one command, edit one script file, and see live UI updates.
- Failure cases are visible in the app and in logs.

Exit criteria:

- The runtime is stable enough for engineering use on editor/debug surfaces.

## Recommended File Layout

Keep the structure small and obvious:

```text
luau-runtime/
  src/
    lib.rs
    runtime.rs
    backend.rs
    module.rs
    reload.rs
    watcher.rs
    bindings/
      app.rs
      ui.rs
examples/
  runtime-egui-host.rs
  runtime-core-demo.rs
  runtime-luau/
    demo.luau
    widgets.luau
```

This is a suggestion, not a hard requirement. The key point is to isolate:

- backend-specific Luau code
- reload orchestration
- watcher integration
- egui bindings

## Scope Discipline

If a decision slows Phase 1 through Phase 3, defer it.

Specifically defer:

- full `egui-component` component coverage
- generic schema-driven widget binding
- worker-thread script execution
- multi-viewport egui support
- deep host object reflection

The first success condition is not "complete scripting support". It is:

- one running egui demo
- one watched Luau file
- reliable hot reload
- preserved state
- safe rollback on errors
