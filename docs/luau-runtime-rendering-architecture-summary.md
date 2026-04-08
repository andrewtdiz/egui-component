# Current Luau Runtime Rendering Architecture

This document summarizes the current render architecture implemented by:

- `luau-runtime/`
- the `examples/runtime-luau/` Luau helper layer
- the `runtime-egui-host` integration

The thin executable wrapper is `examples/runtime-egui-host.rs`. The actual host integration lives in `src/example_apps/runtime_egui_host.rs`.

## Scope Reviewed

- `luau-runtime/src/runtime/mod.rs`
- `luau-runtime/src/runtime/graph.rs`
- `luau-runtime/src/runtime/host.rs`
- `luau-runtime/src/runtime/parse.rs`
- `luau-runtime/src/runtime/types.rs`
- `luau-runtime/src/watch.rs`
- `src/example_apps/runtime_egui_host.rs`
- `examples/runtime-luau/apps/demo/main.luau`
- `examples/runtime-luau/ui.luau`
- `examples/runtime-luau/ui/core/bridge.luau`
- `examples/runtime-luau/ui/core/layout.luau`
- representative Luau helpers such as `ui/components/button.luau`, `ui/core/text.luau`, `ui/components/select.luau`, and `ui/core/virtual_list.luau`

## Architecture In One Sentence

Rust owns the frame, owns the single Luau VM, stages reloads into a candidate graph, validates staged first frames with `render`, runs steady-state `update` and `render` inside a frame-local host bridge, and translates each `ui.*` call directly into immediate `egui` widget work without building a retained intermediate tree.

## What The Architecture Is Not

- It is not a retained scene graph.
- It is not a serialized document renderer.
- It does not compile or mutate the Luau VM from the file watcher thread.
- It does not let `egui::Ui` or host trait objects escape the active frame.
- The `runtime-luau` helper library does not add a second renderer; it is a thin Luau composition layer over the same `ui.*` bridge.

## Core Runtime Objects

### `ScriptRuntime`

`ScriptRuntime` is the central owner of the runtime state in `luau-runtime/src/runtime/mod.rs:43-61`.

It owns:

- one long-lived `mlua::Lua` VM
- the active committed runtime graph
- an optional staged candidate graph
- the file-change `ReloadQueue`
- lifecycle, failure, reload, GC, and memory metrics
- a cached host bridge (`HostBridgeCache`)

Important implication:

- there is one Lua heap per `ScriptRuntime`, not one heap per reload
- active and staged graphs are sets of `mlua::Table` and `mlua::Function` handles into that same heap

### Active vs staged runtime

The graph layer models two runtime states in `luau-runtime/src/runtime/graph.rs:145-160`:

- `CommittedActiveRuntime`: the currently live graph, root hooks, root state, and active `require` snapshot
- `StagedRuntime`: a candidate produced by load or reload, plus metadata needed to commit and shut down replaced modules

This is the central rollback mechanism:

- load/reload only stages a candidate
- the candidate becomes active only after its first render succeeds

## End-To-End Render And Reload Flow

### 1. `eframe` enters the app

`examples/runtime-egui-host.rs` only wires `eframe::run_native(...)` to `runtime_egui_host::update(...)`.

The real host loop is `src/example_apps/runtime_egui_host.rs:436-448`.

Per frame it does:

1. `process_frame_boundary(ctx)`
2. render the script surface into the central panel
3. show diagnostics and error overlays
4. request another repaint if the runtime asked for one

### 2. Frame boundary work decides whether to load or reload

`RuntimeEguiHostApp::process_frame_boundary` in `src/example_apps/runtime_egui_host.rs:108-173` does the frame-boundary orchestration.

It:

- lazily starts a `RootScriptWatcher`
- checks the shared `ReloadQueue`
- marks `pending_reload` when the watcher has enqueued a coalesced dirty-path batch
- performs either:
  - initial `load_root_with_host(...)`, or
  - `reload_now_with_host(...)`

The host used here is `RuntimeControlHost` (`src/example_apps/runtime_egui_host.rs:506-544`), which supports:

- `app.log`
- `app.request_reload`
- `app.request_repaint`

This host does not render widgets. It only handles control-plane app calls during load/reload.

### 3. The watcher only marks dirtiness

`RootScriptWatcher` in `luau-runtime/src/watch.rs:15-85` watches the project root directory recursively.

Its responsibilities are intentionally narrow:

- normalize file paths
- ignore non-`.luau` files and metadata-only events
- coalesce dirty module paths before they reach `ReloadQueue`
- enqueue the watched root path on rescan-style events
- call the host-provided `on_dirty` callback so the next frame runs

It does not:

- compile Luau
- compute dependency order
- touch the Lua VM
- commit reloads

All real runtime work still happens on the UI thread at frame boundaries.

### 4. Load/reload builds a staged candidate graph

`ScriptRuntime::load_root_with_host` and `reload_now_with_host` live in `luau-runtime/src/runtime/mod.rs:158-205`.

Both go through `with_app_host_scope(...)` and then call:

- `load_root_inner(...)` for first load in `graph.rs:171-180`
- `reload_root_inner(...)` for reload in `graph.rs:182-192`

Those functions call `rebuild_graph(...)` in `graph.rs:194-331`.

`rebuild_graph(...)` does the heavy reload work:

1. clones the old runtime metadata for comparison and rollback bookkeeping
2. computes the affected path set with reverse-dependency expansion in `compute_affected_paths(...)` (`graph.rs:849-918`)
3. enters `RequireMode::Build(...)`
4. recursively builds module candidates via `build_module_candidate(...)` (`graph.rs:605-683`)
5. runs `init` for new modules or `reload` for replaced modules in build order via `apply_candidate_hooks(...)` (`graph.rs:333-402`)
6. assembles the new active graph, recalculates reverse deps, and rebuilds the `path_index` in `assemble_graph(...)` (`graph.rs:404-479`)
7. prepares a `CommittedActiveRuntime` snapshot in `prepare_runtime_snapshot(...)` (`graph.rs:577-602`)
8. stores the result as `pending_candidate`

Important detail:

- unaffected modules are reused by handle, not rebuilt, inside `build_module_candidate(...)` when the changed path set does not include them (`graph.rs:623-629`)

### 5. The script surface renders through a frame-local UI host

`RuntimeEguiHostApp::render_script_surface` in `src/example_apps/runtime_egui_host.rs:217-260` creates a `FrameUiHost` and calls:

- `ScriptRuntime::render_frame_with_host(...)` in `luau-runtime/src/runtime/mod.rs:214-264`

`FrameUiHost` is the actual `egui` adapter in `src/example_apps/runtime_egui_host.rs:560-678`.

It owns per-frame host state:

- the current `egui::Context`
- the mounted `SurfaceId`
- the root `egui::Ui`
- a stack of nested child scopes (`scope_stack`)
- the explicit script id stack (`id_stack`)

### 6. The runtime installs `app` and `ui` globals for that frame

The key bridge functions are:

- `with_app_host_scope(...)` in `luau-runtime/src/runtime/host.rs:1198-1232`
- `with_ui_host_scope(...)` in `luau-runtime/src/runtime/host.rs:1234-1304`

What they do:

- reuse the cached `app` and `ui` proxy tables from `HostBridgeCache` (`host.rs:370-402`)
- install them into `lua.globals()` with `ScopedHostGlobals` (`host.rs:886-910`)
- create a fresh `ActiveHostScope` token for the frame (`host.rs:695-778`)
- deactivate that scope automatically on drop (`host.rs:823-837`)

This is the safety boundary that prevents stale host callbacks from remaining valid after the frame closes.

### 7. `update` and `render` execute against either the candidate or the active graph

`render_frame_inner(...)` in `luau-runtime/src/runtime/host.rs:1306-1374` selects:

- the staged candidate when one exists
- otherwise the active committed runtime

During the call it temporarily switches `RequireMode` to `Active(require_snapshot)` so `require(...)` resolves against the runtime snapshot.

Execution order is:

1. staged candidate validation: required `render(state)` only
2. committed steady-state frame: optional `update(state, input)` and then required `render(state)`

Current `update` behavior:

- the runtime creates a fresh empty Lua table for `input` on committed steady-state frames (`host.rs:1331-1341`)
- `ui.*` calls are not allowed during `update`

### 8. Each `ui.*` call becomes immediate `egui` work

The dispatch path is:

1. Luau reads a member from the `ui` proxy table
2. the proxy `__index` lazily returns a cached wrapper function (`host.rs:1448-1465`, `host.rs:661-675`)
3. the wrapper validates:
   - active frame token
   - correct surface id
   - current phase
   - mount capabilities
4. the wrapper parses Lua arguments into Rust structs in `parse.rs`
5. the wrapper calls the active `RuntimeUiHost`
6. the `FrameUiHost` method renders the widget immediately against `egui`
7. the widget result is returned straight back to Luau

There is no retained tree between steps 4 and 6.

Examples:

- `ui.text_edit(...)` wrapper: `host.rs:1557-1565`, parse in `parse.rs:144-163`, egui host in `runtime_egui_host.rs:724-751`
- `ui.select(...)` wrapper: `host.rs:1601-1609`, parse in `parse.rs:250-270`, egui host in `runtime_egui_host.rs:874-908`
- `ui.tabs(...)` wrapper: `host.rs:1610-1618`, parse in `parse.rs:272-292`, egui host in `runtime_egui_host.rs:910-946`
- `ui.virtual_list(...)` wrapper: `host.rs:1692-1704`, parse in `parse.rs:432-451`, egui host in `runtime_egui_host.rs:1151-1230`

### 9. Layout scopes are balanced both in Luau and Rust

The Luau helper layer wraps scopes in `examples/runtime-luau/ui/core/layout.luau`.

It uses:

- `xpcall(...)` around the render callback
- `pcall(...)` around `end_scope()` and `pop_id()`
- explicit rethrow after cleanup

Rust still enforces the same invariant inside the bridge with `ScopedUiState` in `host.rs:913-962`, and `with_ui_host_scope(...)` rejects frames that return with unclosed containers or ids (`host.rs:1277-1286`).

This means:

- Luau helpers try to unwind correctly
- Rust still fails closed if a raw script misbalances scope operations

### 10. Deferred frame commands flush after successful render

`app.request_reload()` and `app.request_repaint()` are queued into `FrameCommandQueue` during render (`host.rs:1491-1517`, `host.rs:798-810`).

They are flushed only after render succeeds in `with_ui_host_scope(...)` via `apply_frame_commands(...)` (`host.rs:1287-1299`, `host.rs:1395-1423`).

`app.log(...)` remains immediate.

This avoids replaying frame-control side effects from partially failed renders.

### 11. Commit happens after the first successful frame

If the runtime is rendering a staged candidate, `with_ui_host_scope(...)` calls `commit_staged_candidate(...)` after a successful frame (`host.rs:1300-1302`).

`commit_staged_candidate(...)` in `graph.rs:504-557`:

1. takes `pending_candidate`
2. swaps `active_runtime` to the new runtime
3. promotes the bridge generation
4. runs `shutdown(state)` on replaced or removed modules using the old `require` snapshot

Actual rollback boundary:

- candidate build failure: old active runtime stays active
- candidate first-frame failure before commit: old active runtime stays active
- shutdown-hook failure after `commit_staged_candidate(...)` starts: the new runtime is already active

So rollback is guaranteed up to commit. It is not a post-commit transactional rollback.

## How `runtime-luau/` Integrates

The `runtime-luau` side is a thin typed library, not a second rendering architecture.

### Bridge layer

`examples/runtime-luau/ui/core/bridge.luau` exposes typed accessors:

- `current_app()`
- `current_ui()`

These just return the active frame-local globals `app` and `ui`.

### Helper library entrypoint

`examples/runtime-luau/ui.luau:1-47` is only a module aggregator for helper namespaces like:

- `stack`
- `text`
- `button`
- `select`
- `tabs`
- `list`

Those helpers still bottom out in `ui.*`.

### Helper modules are thin wrappers

Representative examples:

- `ui/components/button.luau` forwards to `bridge.current_ui().button(...)`
- `ui/core/text.luau` forwards to `bridge.current_ui().label(...)`
- `ui/components/select.luau` forwards to `bridge.current_ui().select(...)`
- `ui/core/virtual_list.luau` forwards to `bridge.current_ui().virtual_list(...)`

The helper layer can add convenience work before the bridge:

- merge variant props in Luau via `variant.merge(...)` (`ui/core/variant.luau`)
- wrap scope begin/end in protected callbacks (`ui/core/layout.luau`)

But it does not:

- build a widget tree
- batch bridge calls
- retain host layout state

### Root scripts use the runtime hooks directly

`examples/runtime-luau/apps/demo/main.luau` shows the intended script contract:

- `init(state)`
- `update(state, input)`
- `reload(old_exports, state)`
- `render(state)`

`update(state, input)` is steady-state only. Staged first-frame validation after load or reload runs `render(state)` without calling `update`.

Inside `render`, the demo calls `kit.stack.column(...)`, `kit.text.*(...)`, and component helpers such as `profile_panel.render(...)`, which eventually call the same `ui.*` bridge.

## Most Performance-Critical Locations

### Steady-state frame hot path

#### 1. Per-widget host bridge dispatch

This is the hottest repeated path in normal rendering:

- wrapper lookup through proxy tables: `host.rs:1425-1465`
- frame validation: `host.rs:723-778`
- argument parsing and schema validation: `parse.rs:39-432`, `parse.rs:1043-1084`, `parse.rs:1179-1189`
- host method dispatch: `host.rs:1522-1798`

Why it is hot:

- it runs for every widget call
- it performs dynamic Lua-to-Rust conversion on every call
- it often allocates owned `String` and `Vec` values
- it enforces closed-schema prop validation by iterating option tables

#### 2. `FrameUiHost` widget adapters

The egui-side adapter methods in `src/example_apps/runtime_egui_host.rs:680-1274` are the next hot layer.

Notable per-call costs:

- `text_edit(...)` clones the incoming text into a new owned `String` every frame (`runtime_egui_host.rs:731-747`)
- `select(...)` builds a temporary `Vec<&str>` from `&[String]` every call (`runtime_egui_host.rs:892-901`)
- `tabs(...)` builds a temporary `Vec<TabOption>` every call (`runtime_egui_host.rs:920-939`)
- `button_group(...)` builds a temporary `Vec<&str>` every call (`runtime_egui_host.rs:997-1005`)
- `egui_widget_id(...)` reconstructs the `egui::Id` chain from `surface_id + id_stack + local_id` every time (`runtime_egui_host.rs:1276-1285`)

These are not architectural bugs. They are the current cost model of the immediate bridge.

#### 3. The Luau helper layer can add temporary tables and callback overhead

The Luau helper layer is thin, but not free:

- `variant.merge(...)` allocates a new merged props table (`ui/core/variant.luau`)
- `stack.column(...)`, `stack.row(...)`, `stack.card(...)`, and `with_id(...)` wrap every scoped callback with `xpcall`/`pcall` (`ui/core/layout.luau`)

For shallow UIs this is minor. For very deeply nested or helper-heavy UIs, it becomes measurable Lua-side overhead before the bridge call even happens.

### Large-list special case

#### 4. `ui.virtual_list(...)` is explicitly optimized

This is the one path with dedicated cross-frame caching:

- parse-side string snapshot cache in `host.rs:455-617`
- dense array snapshot and change detection in `parse.rs:882-983`
- visible-row-only rendering via `ScrollArea::show_rows(...)` in `runtime_egui_host.rs:1190-1225`

Why this matters:

- without caching, a large Lua string table would be copied into Rust every frame
- without `show_rows`, `egui` would attempt to build and paint every row every frame

Current strategy:

- cache `Arc<[String]>` by Lua table identity
- reuse cached data when the table pointer and string element pointers have not changed
- cap cache size to 16 entries or 16 MiB and evict LRU entries
- prune entries when bridge generation changes so stale candidate data does not leak across failed reloads

### Reload-only hot path

#### 5. Graph rebuild is the dominant reload cost

`rebuild_graph(...)` and its helpers are the expensive reload path:

- clone old runtime metadata: `graph.rs:205-208`
- compute affected paths through reverse dependencies: `graph.rs:849-918`
- recursively read and execute modules: `graph.rs:605-683`, `graph.rs:1005-1039`
- clone state tables for rebuilt modules: `graph.rs:744-764`, `graph.rs:1076-1105`
- rebuild reverse dependency sets and path index: `graph.rs:447-478`
- rebuild the active `require` snapshot map: `graph.rs:587-594`

Important nuance:

- unaffected Lua modules are reused by handle
- but runtime metadata is still cloned and rebuilt at the Rust `HashMap`/`BTreeSet` level during reload work

#### 6. First frame after load/reload is more expensive than steady state

The first successful frame after a staged load/reload includes:

- the normal `update` and `render`
- scope-balance verification
- deferred command flush
- candidate commit
- post-commit `shutdown` hooks for old modules
- frame GC step
- optional full-GC leak sample and heap dump

So `first_after_load` and `first_after_reload` are intentionally heavier than steady-state frames.

This distinction is reflected in `RuntimeRenderKind` and separate lifecycle buckets in `mod.rs:267-276` and `types.rs:842-900`.

### Benchmark coverage

There is already a benchmark harness for the main scale-sensitive paths in `luau-runtime/benches/runtime_scale.rs`.

It explicitly exercises:

- label flood surfaces
- mixed label/button/text-input surfaces
- virtual lists
- both steady-state frames and first-frame-after-reload

See `runtime_scale.rs:16`, `runtime_scale.rs:266-325`, and `runtime_scale.rs:416-491`.

## Memory Management Model

### 1. One Lua heap, many handles

`ScriptRuntime::new(...)` creates one `Lua` instance and records its initial heap size (`mod.rs:65-89`).

All of these are handles into that single heap:

- module export tables
- root state tables
- render/update functions
- `require` snapshots
- bridge proxy tables
- cached virtual-list Lua tables

Reload does not create a second VM.

### 2. Active graph, staged graph, and rollback ownership

Ownership is split like this:

- `active_runtime`: the committed live graph
- `pending_candidate`: a staged graph waiting for first-frame success

Until commit:

- the old active graph remains reachable
- the candidate graph is additional live memory

That is the price of rollback safety.

After commit:

- the new graph becomes active
- old replaced modules are still kept long enough to run `shutdown`

### 3. Module state is cloned for rebuilt modules

When a module is rebuilt, the candidate gets a cloned copy of the old module state via `previous_module_state(...)` and `clone_state_table(...)` (`graph.rs:744-764`, `graph.rs:1076-1105`).

What the clone supports:

- nested tables
- booleans
- integers
- numbers
- strings

What it rejects:

- functions
- userdata
- threads
- other unsupported Lua value types

If unsupported values are stored in `state`, reload fails rather than mutating state in place.

That is an important memory invariant:

- runtime state must be cloneable plain-data tables if it is expected to survive reloads

### 4. Unaffected modules are reused, affected modules are duplicated temporarily

For reloads:

- unaffected modules reuse existing export/state handles
- affected modules get rebuilt export tables and cloned state

So reload-time memory pressure scales mostly with:

- number of affected modules
- size of cloned `state` tables
- size of temporary Rust-side graph metadata

### 5. The bridge cache is long-lived, but bounded where it matters

`HostBridgeCache` is retained on the runtime for reuse (`host.rs:370-402`, `host.rs:1156-1167`).

Long-lived cached objects:

- proxy `app` table
- proxy `ui` table
- bridge session object
- virtual list cache

The high-risk cache is bounded:

- `VirtualListCache` max entries: 16
- `VirtualListCache` max payload: 16 MiB

See `host.rs:455-617`.

### 6. Frame-local host pointers are non-owning and guarded

The bridge stores host trait objects as `NonNull<dyn RuntimeAppHost>` and `NonNull<dyn RuntimeUiHost>` in:

- `ScopedRuntimeAppHostBridge` (`host.rs:839-861`)
- `ScopedRuntimeUiHostBridge` (`host.rs:863-884`)

Those pointers are safe only because:

- they are installed for the duration of a scoped call
- `ScopedActiveHostScope` invalidates the frame token on drop (`host.rs:823-837`)
- `ScopedHostGlobals` restores previous globals on drop (`host.rs:886-910`)
- every host callback revalidates the active token and surface id (`host.rs:723-778`)

This is explicit lifetime management, not ownership transfer.

### 7. `FrameUiHost` owns only per-frame Rust-side layout state

`FrameUiHost` stores:

- `scope_stack: Vec<ScopeFrame>`
- `id_stack: Vec<String>`

See `runtime_egui_host.rs:560-589`.

These vectors are per frame and dropped when the host adapter is dropped. They do not retain data across frames.

### 8. The watcher queue is shared and coalesced until drained

`ReloadQueue` is a shared distinct-path batch in `types.rs:170-207`.

This keeps watcher behavior editor-like instead of append-only:

- repeated saves of the same file collapse to one pending path before rebuild work starts
- rescan-style watcher events still force a root-path reload by enqueuing the watched root once

The host still requests a repaint as soon as dirtiness is observed.

### 9. GC and leak instrumentation are integrated into runtime boundaries

Instrumentation policy is configured in `RuntimeInstrumentationConfig` (`types.rs:15-37`).

Default behavior:

- frame GC step budget: 64 KiB
- reload GC step budget: 512 KiB
- leak sample interval: every 10 committed reloads
- warning threshold: 64 KiB retained growth

Runtime memory accounting is recorded at:

- reload boundaries in `instrument_reload_boundary(...)` (`mod.rs:490-520`)
- frame boundaries in `instrument_frame_boundary(...)` (`mod.rs:522-566`)

Leak detection does the expensive work only after committed load/reload events:

- optional full GC twice in `collect_full_gc_sample(...)` (`mod.rs:727-742`)
- optional `heap_dump()` sampling in `maybe_sample_leak_after_commit(...)` (`mod.rs:621-725`)

Operational implication:

- steady-state frames pay only incremental GC stepping by default
- first load and some reloads can pay for full GC and heap dump analysis

## Concrete Invariants To Keep In Mind

- Rust owns the frame boundary and the only live `egui::Ui`.
- Luau owns composition and state mutation inside `update` and `render`.
- `runtime-luau` is a thin helper layer over `ui.*`, not a second renderer.
- Reload is staged first, committed only after a successful first frame.
- Rollback is guaranteed before commit, not after post-commit shutdown starts.
- Widget identity is `surface_id + push_id stack + local_id`.
- Layout scopes do not affect widget identity.
- Scope balance is enforced in both Luau helpers and Rust bridge state.
- Every `ui.*` call is parsed and validated at runtime.
- Large virtual lists are the one path with a dedicated cross-frame bridge cache.

## Authoring Discipline

- Put mutable reload-persistent data in `state`.
- Treat module exports as immutable by convention after load/reload.
- Prefer stateless shared helpers; module-local caches and mutated exports remain shared heap memory.
- `reload` can create observable side effects outside `state` when it mutates shared modules or exports.
- Current lifecycle contract is post-commit shutdown: old-module `shutdown` runs after the new runtime is active, and shutdown failure does not roll back that commit.

## Practical Summary

The current system is an immediate-mode Luau-to-egui bridge with a staged-reload runtime around it.

The hottest steady-state work is:

- Lua helper execution
- proxy lookup and wrapper dispatch
- argument parsing and schema validation
- egui host adapter execution

The hottest reload work is:

- recursive module rebuild
- cloned state creation
- graph metadata reconstruction
- first-frame commit and optional leak sampling

Memory is managed by keeping one Lua VM alive, staging new graphs beside the active graph until first-frame success, cloning only reloadable module state, bounding the only cross-frame data cache that can grow significantly (`VirtualListCache`), and collecting GC/leak telemetry at frame and reload boundaries.
