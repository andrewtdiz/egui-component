# `notify` Hot Reload Plan For `luau-runtime-core`

This document turns [luau-egui-runtime-mvp.md](/home/andy/Documents/egui-component/luau-egui-runtime-mvp.md) into an implementation plan for this repository.

The intent is to ship a small, verifiable `luau-runtime-core` MVP first, then layer `notify`-driven hot reload on top without collapsing into a large one-shot design.

## Status

As of 2026-04-06:

- Phase 0 is complete.
- Phase 1 is complete.
- Phase 2 is complete.
- Phase 3 is complete for the single-root-file path.
- Phase 4 is complete.
- Phase 5 is complete.
- Phase 6 has been replaced by the shipped immediate-mode `egui` host boundary in `runtime-egui-host`.

## Guiding Rules

- Keep one Luau VM on the main UI thread for the MVP.
- Reload only at frame boundaries.
- The watcher thread must never touch the VM directly.
- Start with one root script file before supporting module graphs.
- Keep the last known-good script version alive on compile or runtime failure.
- Keep `mlua` or any Luau backend private to the crate API.
- Support root viewport only in the MVP.
- Do not bind all of `egui-component` directly into the runtime core.
- Keep `egui-component` integration above the runtime crate, through a narrow typed host bridge.
- Keep the public runtime host surface limited to `app.*` control calls plus a small immediate `ui.*` surface.

## Historical Repo Blockers

- Root `Cargo.toml` points to `luau-runtime-core`, but the crate directory is [luau-runtime](/home/andy/Documents/egui-component/luau-runtime).
- [luau-runtime/src/lib.rs](/home/andy/Documents/egui-component/luau-runtime/src/lib.rs) declares `mod runtime;`, but `runtime.rs` does not exist yet.
- Docs mention `runtime-core-demo` and `runtime-egui-host`, but those examples are not present yet.

These were Phase 0 blockers and are now resolved.

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

Status: complete for `examples/runtime-luau/apps/demo.luau` in `runtime-egui-host`.

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

Status: complete for the `examples/runtime-luau/` project in `runtime-egui-host`.

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

## Phase 6: `egui-component` Contract Embedding

Goal: integrate the runtime host with the actual `egui-component` product surface without collapsing the crate boundary.

Status: complete for `examples/runtime-luau/` in `runtime-egui-host`.

Implement:

- Keep `luau-runtime-core` backend-agnostic and free of `egui-component` types.
- Add a generic structured value channel between Luau and the host.
- Publish `ContractTree` documents from Luau instead of binding `ComponentUi` directly.
- Render those trees on the Rust side with `egui_component::contract::render_tree`.
- Feed semantic `ContractEvent`s back into Luau on the next frame.
- Keep hot reload, dependency reload, and rollback behavior unchanged under this adapter.

Do not implement:

- direct bindings for the whole `components::*` builder API
- `egui-component` dependencies inside `luau-runtime-core`
- a reflection-heavy universal widget marshalling layer

Verification:

- Run `runtime-egui-host`.
- Edit `examples/runtime-luau/apps/panel.luau` and confirm the rendered card updates live.
- Click contract-driven controls and confirm Luau state updates through `ContractEvent`s.
- Save a broken leaf module and confirm the previous tree remains active until the file is fixed.

Exit criteria:

- The repository has a runnable proof that uses Luau for composition/styling and Rust for rendering/events at the `egui-component` boundary.

## Phase 7: Main Example Integration And `internal_taffy` Surface

Goal: move from the standalone `runtime-egui-host` proof into a reusable product surface that can drive real example UIs without collapsing Rust ownership of rendering, layout, and semantics.

Implement:

- Keep `examples/runtime-egui-host.rs` as the canonical low-noise runtime harness.
- Add a second host path for the main examples instead of replacing `showcase` immediately:
  - `examples/showcase-runtime.rs`, or
  - a `Runtime` section inside the existing showcase shell.
- Reuse the existing Rust chrome for top-level example framing, logs, theme install, and error overlays.
- Let Luau own only the inner example surface that publishes a runtime document each frame.
- Introduce a richer runtime document model that separates:
  - semantic component family
  - layout intent
  - class-like styling tokens
  - action bindings / callback intent
  - plain serializable view data
- Add a Rust-side adapter that renders layout-capable nodes through `internal_taffy` when requested.
- Keep the watcher and reload queue architecture unchanged; only the host adapter and document model should expand.

Do not implement:

- direct Luau access to `internal_taffy::Tui`, `ComponentUi`, or arbitrary Rust closures
- general-purpose reflection over all Rust structs
- moving the full static `showcase` into Luau in one migration

Verification:

- `runtime-egui-host` still covers the narrow runtime proof.
- `showcase-runtime` or the runtime showcase section can render a multi-component scripted surface.
- At least one scripted example uses:
  - contract events
  - class-like style strings
  - a non-trivial `internal_taffy` layout
  - hot reload of a leaf Luau module
- Static `showcase` remains available for snapshot/regression coverage during migration.

Exit criteria:

- The repository has both:
  - a stable Rust-authored showcase for regression testing
  - a runtime-backed showcase surface that proves the intended Luau authoring model

## Recommended Product Boundary For This Repository

The right balance is:

- Luau owns composition, reusable components, view-model projection, action routing, and style class selection.
- Rust owns component rendering, `ContractEvent` emission, `internal_taffy` layout execution, class parsing, theme tokens, and all hot-reload orchestration.

That means Luau should feel like `shadcn` + Tailwind authoring, but it should not own:

- low-level widget drawing
- the taffy engine itself
- direct egui response objects
- host callback closures
- native state that must remain authoritative in Rust

In practice:

- Luau returns pure data.
- Rust interprets that data against authoritative widget + layout implementations.
- Rust emits semantic events.
- Luau handles those events and builds the next frame's document.

## Runtime Document Model Needed For The Tailwind-Like Use Case

The current `ContractTree` is a good base, but the runtime-facing model needs three more first-class concepts.

### 1. Common class-like styling fields

Every node should support:

- `class`: optional whitespace-delimited string
- `class_list`: optional normalized string array
- `slot_classes`: optional map of slot name to class string for multi-part widgets
- `variant`: optional semantic variant name chosen by Luau

Recommended rule:

- Luau authors `class` strings.
- Rust normalizes and parses them.
- Decoded style values should not round-trip back into Luau every frame.

### 2. Generic action binding fields

Move toward a common action shape on interactive nodes:

- `actions.click`
- `actions.change`
- `actions.submit`
- `actions.select`
- `actions.open`
- `actions.close`
- `actions.confirm`
- `actions.cancel`

Keep emitting typed `ContractEvent`s from Rust. Luau should dispatch those events by action id or handler key.

This keeps callbacks "wired through Luau" without requiring host-managed closures.

### 3. Layout fields that map onto `internal_taffy`

Do not expose raw Rust `taffy::Style` objects. Add a serializable layout DSL instead.

Recommended common fields:

- `layout.display`: `flow | flex | grid | overlay`
- `layout.direction`: `row | column`
- `layout.grow`, `layout.shrink`, `layout.basis`
- `layout.width`, `layout.height`, `layout.min_width`, `layout.min_height`
- `layout.max_width`, `layout.max_height`
- `layout.gap_x`, `layout.gap_y`
- `layout.padding`, `layout.margin`
- `layout.align`, `layout.justify`
- `layout.wrap`
- `layout.columns` for grid tracks
- `layout.rows` for explicit row tracks
- `layout.col_span`, `layout.row_span`
- `layout.overflow_x`, `layout.overflow_y`
- `layout.anchor`, `layout.offset_x`, `layout.offset_y` for overlay/tooling surfaces

Recommended architecture:

- simple row/column/inset/card nodes can continue using the current typed fields for ergonomic authoring
- all nodes may additionally carry an optional `layout` block
- when `layout` is absent, the existing renderer path remains active
- when `layout` is present for supported families, the Rust renderer routes that subtree through an `internal_taffy` adapter

This preserves the current API while opening a real layout system for scripted surfaces.

## How To Bring This Into The Main Examples

Do this in layers instead of converting `showcase` wholesale.

1. Keep `showcase` Rust-authored.
2. Add `showcase-runtime` that reuses the same outer app shell but renders one scripted preview surface.
3. Move only a small curated set of showcase surfaces into Luau first:
   - `button`
   - `input`
   - `card`
   - `tabs`
   - one `internal_taffy` grid/flex-heavy example
4. Add a script-side component or recipe library under `examples/runtime-luau/ui/`.
5. Add script-side class presets under `examples/runtime-luau/styles/`.
6. Keep the static showcase as the truth source for snapshots until runtime parity is good enough.

This keeps the migration reversible and avoids turning the main showcase into the first place where runtime breakage is discovered.

## Script Authoring Model To Target

The intended Luau ergonomics should look closer to this:

```lua
local ui = require("./ui")
local cx = require("./cx")

return function(props)
    return ui.card({
        id = props.id,
        class = cx("w-full gap-3 rounded-lg border bg-panel p-4", {
            "ring-2 ring-brand-9": props.selected,
        }),
        layout = {
            display = "flex",
            direction = "column",
            gap_y = 12,
        },
    }, {
        ui.label({
            id = props.id .. ".title",
            class = "text-lg font-semibold tracking-tight",
            text = props.title,
        }),
        ui.input({
            id = props.id .. ".name",
            class = "w-72",
            value = props.value,
            actions = {
                change = "profile.name.changed",
            },
        }),
        ui.button({
            id = props.id .. ".submit",
            class = "btn-primary",
            label = "Save",
            actions = {
                click = "profile.save",
            },
        }),
    })
end
```

The important property is not the exact syntax. It is that Luau authors:

- tree composition
- reusable components
- nearby class strings
- action ids
- plain serializable props

And Rust authors:

- how classes become tokens
- how tokens become component visuals
- how layout is executed
- how events are emitted

## Recommended File Layout After This Expansion

```text
examples/
  runtime-egui-host.rs
  showcase.rs
  showcase-runtime.rs
  runtime-luau/
    app.luau
    showcase/
      main.luau
      sections/
      components/
      styles/
      data/
src/
  contract/
    model.rs
    renderer.rs
    runtime.rs
    layout.rs
    styling.rs
  example_apps/
    runtime_egui_host.rs
    showcase.rs
    showcase_runtime.rs
```

This keeps the runtime crate generic, the rendering host in `egui-component`, and the scripted example surface clearly separate from the static showcase.

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
