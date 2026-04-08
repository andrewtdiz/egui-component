# Luau Runtime Cutover Plan

This document turns the architecture feedback into a smaller set of cutover workstreams for the current codebase.

## Goal

Keep the current architecture shape:

- one engine-owned Luau VM
- frame-scoped `app` and `ui` host bridges
- staged load/reload with commit after first successful frame
- immediate-mode `egui` rendering with no retained widget tree

Change the implementation priorities:

- harden reload correctness first
- make the public rollback and state contract explicit
- clean up watcher behavior
- only optimize the bridge after we measure it

## Non-Goals

- do not replace the single-VM model
- do not introduce a retained renderer above `egui`
- do not split the runtime into multiple heaps as part of this cutover
- do not build a typed fast lane before bench data justifies it

## Workstream 1: Reliability Cutover Now

This is the work we should actively cut over to now. It groups the immediate correctness and contract fixes into one landing sequence.

### Scope

- fix the public contract around rollback, shared heap behavior, and reloadable state
- reduce mutation during staged validation
- make file watching behave like a coalesced editor signal instead of a raw append-only queue
- add tests that lock the new behavior in place

### Tasks

- Update docs and inline comments so they say rollback is guaranteed for runtime selection, not for full VM-state isolation.
- Document that active and staged graphs share one Lua heap.
- Document that unaffected modules may be reused by handle during reload.
- Promote the reloadable state rule to a public contract:
  - `state` is plain-data tables only
  - supported values are tables, booleans, integers, numbers, and strings
  - functions, userdata, threads, and similar values are not reloadable state
- Change staged first-frame validation to `render`-only.
- Skip `update(state, input)` while validating `pending_candidate`.
- Keep normal `update + render` behavior for committed steady-state frames.
- Replace the raw FIFO dirty-path queue with frame-level coalescing and path deduplication.
- Preserve the ability to force a root reload on rescan-style watcher events.
- Add tests for staged render-only validation, rollback on candidate failure, steady-state update behavior, shared-state caveats, queue coalescing, and current shutdown behavior.

### Current Touch Points

- `docs/luau-runtime-rendering-architecture-summary.md`
- `docs/luau-runtime-architecture.md`
- `examples/runtime-luau/README.md`
- `luau-runtime/src/runtime/host.rs`
- `luau-runtime/src/runtime/graph.rs`
- `luau-runtime/src/runtime/types.rs`
- `luau-runtime/src/watch.rs`
- `src/example_apps/runtime_egui_host.rs`
- `luau-runtime/src/runtime/tests.rs`

### Exit Criteria

- No doc or inline comment claims full transactional rollback across the shared VM.
- The plain-data state rule is visible to Luau authors.
- A staged candidate validates with `render` only.
- Repeated file saves are coalesced before rebuild work starts.
- The repo has tests for the new cutover behavior.

## Workstream 2: Isolation And Lifecycle Hardening

This is the next block of work after the immediate cutover lands. It is about making the remaining shared-heap limits explicit and deciding how strict we want the runtime to become.

### Scope

- tighten the authoring model around mutable state
- reduce accidental side effects through reused modules and shared exports
- decide whether current post-commit shutdown semantics are the long-term contract

### Tasks

- Document the intended module discipline:
  - mutable reloadable data belongs in `state`
  - module exports should be treated as immutable
  - shared helpers should be stateless where possible
- Add tests that explicitly demonstrate the remaining shared-heap caveats:
  - side effects from `reload`
  - side effects through reused unaffected modules
  - mutations via shared exports or module-level caches
- Keep this phase at docs plus tests only; defer stronger guardrails such as readonly exports or runtime assertions to a later phase.
- Treat current post-commit `shutdown` behavior as the long-term contract for now:
  - document it clearly
  - keep the current ordering
  - add tests that show shutdown failure does not roll back the committed runtime

### Current Touch Points

- `luau-runtime/src/runtime/graph.rs`
- `luau-runtime/src/runtime/tests.rs`
- `examples/runtime-luau/README.md`
- architecture docs

### Exit Criteria

- Module authors have a clear rule for where mutable state is allowed.
- The repo has tests that make the remaining isolation limits explicit.
- The shutdown contract is explicit, stable, and covered by tests.

## Workstream 3: Performance Follow-Up

This stays grouped as one measurement-driven track. We should not start it until Workstream 1 is landed.

### Scope

- measure the current bridge cost
- identify the true hot widgets and hot allocations
- only then decide whether a small typed fast lane is worth the complexity

### Tasks

- Run and extend the existing benchmark coverage for steady-state render and first-frame-after-reload.
- Capture baseline numbers for `label`, `button`, `text_edit`, `select`, common container-heavy surfaces, and `virtual_list`.
- Add focused profiling around wrapper dispatch, argument parsing, temporary allocations, and `egui_widget_id` construction.
- If the data justifies it, add a small typed fast lane for the hottest widgets while keeping the current dynamic bridge as the general safe path.
- Start with the smallest useful typed set only: `label`, `button`, `text_edit`, and the most common layout/container calls.

### Current Touch Points

- `luau-runtime/benches/runtime_scale.rs`
- `luau-runtime/src/runtime/host.rs`
- `luau-runtime/src/runtime/parse.rs`
- `src/example_apps/runtime_egui_host.rs`

### Exit Criteria

- We have baseline numbers for bridge overhead and frame categories.
- Fast-path work is prioritized from measured hotspots rather than intuition.
- Any added typed path is small, targeted, and measurably useful.

## Immediate Cutover Scope

The current cutover should include everything in Workstream 1 and nothing beyond it.
