# Runtime A-Grade TASK List

Reference architecture notes captured from DeepWiki:
- `reference/deepwiki/react-three-fiber-runtime-architecture.md`

## Reliability-Critical Fixes

- [ ] Problem: Zero-interval repeating timers can busy-loop the timer worker.
  Context: `setInterval(..., 0)` currently allows an interval of `0ms`, which can cause non-progressing loops and catastrophic reliability failure.
  Context (Files): `crates/clay-jsx-runtime/src/host_runtime_api.js`, `crates/clay-jsx-runtime/src/runtime.rs`.
  Verification: Add `clay-jsx-runtime` tests proving `setInterval(0)` is clamped and callback draining remains responsive without CPU spin.

- [ ] Problem: The timer worker lacks a defensive guard for invalid repeating intervals.
  Context: Even if JS boundary clamps values, Rust-side guards are required to guarantee worker progress under malformed or future call paths.
  Context (Files): `crates/clay-jsx-runtime/src/runtime.rs`.
  Verification: Add a Rust unit test that injects invalid repeat intervals and asserts worker progress, bounded drain behavior, and clean shutdown.

- [ ] Problem: Timer catch-up replays can burst unbounded callbacks after stalls/sleep.
  Context: Replaying every missed interval tick can produce massive callback storms and runtime jitter.
  Context (Files): `crates/clay-jsx-runtime/src/runtime.rs`, `crates/clay-jsx-runtime/src/host_runtime_api.js`.
  Verification: Add tests simulating long stalls and assert per-drain callback caps/coalescing semantics.

- [ ] Problem: A single event may invoke the same handler twice via `node_id` and `action_id` routes.
  Context: Handler registration and dispatch currently permit duplicate invocation paths for one logical event.
  Context (Files): `crates/clay-jsx-egui-bridge/src/lowering_api.js`, `crates/clay-jsx-egui-bridge/src/runtime_api.js`, `src/contract/model.rs`.
  Verification: Add bridge integration tests dispatching events with both fields and assert exactly one handler invocation.

- [ ] Problem: Host-tree mutation application is not transactional.
  Context: Partial mutation application can persist if a later mutation in the batch fails, leaving inconsistent retained state.
  Context (Files): `crates/clay-jsx-egui-bridge/src/host_tree.rs`, `crates/clay-jsx-egui-bridge/src/runtime.rs`.
  Verification: Add rollback tests showing failed batches leave host tree state byte-for-byte identical to pre-batch state.

- [ ] Problem: JS->Rust commit flow has no explicit acknowledgment/recovery protocol.
  Context: If Rust rejects a commit, JS reconciler state can advance without deterministic host resync.
  Context (Files): `crates/clay-jsx-runtime/src/runtime.rs`, `crates/clay-jsx-egui-bridge/src/runtime_api.js`, `crates/clay-jsx-egui-bridge/src/runtime.rs`.
  Verification: Add integration tests that force commit rejection and assert deterministic recovery path (resync or controlled session fail).

- [ ] Problem: Reload is destructive before replacement validation.
  Context: Current reload flow tears down the live session before proving the new session can load/render.
  Context (Files): `examples/runtime-jsx/app.rs`, `examples/runtime-jsx-motion.rs`.
  Verification: Add app-level tests where reload fails and previous rendered tree remains visible and interactive.

- [ ] Problem: Reconciler errors are not surfaced as structured host errors.
  Context: Routing uncaught/caught/recoverable errors only to logging prevents robust host-level recovery policy.
  Context (Files): `crates/clay-jsx-runtime/src/clay_jsx_runtime.ts`, `crates/clay-jsx-egui-bridge/src/runtime_api.js`, `crates/clay-jsx-runtime/src/runtime.rs`.
  Verification: Add tests proving Rust receives structured error categories and host behavior is deterministic for each category.

## Reliability Hardening

- [ ] Problem: Runtime tree failures are not isolated by an explicit error-boundary containment policy.
  Context: A-grade reliability requires bounded failure scope and clear recovery semantics for runtime-authored exceptions.
  Context (Files): `crates/clay-jsx-egui-bridge/src/runtime_api.js`, `examples/runtime-jsx/app.rs`, `crates/clay-jsx-egui-bridge/src/lib.rs`.
  Verification: Add TSX failure fixtures (render/effect throw) and tests asserting bounded failure plus successful recovery after fix/reload.

- [ ] Problem: Watch target resolution can fail on missing nested import paths.
  Context: Missing intermediate directories can break watcher setup and degrade hot-reload resilience.
  Context (Files): `examples/runtime-jsx/watch.rs`, `examples/runtime-jsx/app.rs`, `examples/runtime-jsx-motion.rs`.
  Verification: Add watcher tests for nested missing imports and assert watch registration falls back to nearest existing ancestor.

- [ ] Problem: Runtime mutation buffering is unbounded under delayed host drains.
  Context: Unbounded queues risk memory growth and latency spikes during host stalls.
  Context (Files): `crates/clay-jsx-runtime/src/runtime.rs`, `crates/clay-jsx-egui-bridge/src/runtime.rs`.
  Verification: Add stress tests proving hard queue limits/backpressure behavior and explicit overflow metrics/logging.

- [ ] Problem: Lifecycle leak guarantees are not enforced at stronger endurance thresholds.
  Context: A-grade reliability needs high-confidence proof across repeated mount/reload/unmount cycles.
  Context (Files): `examples/runtime-jsx/mod.rs`, `crates/clay-jsx-egui-bridge/src/lib.rs`.
  Verification: Extend readiness harness to 100+ cycles and assert zero active timers, zero pending wakes, and consistent teardown counters.

## Performance-Critical Fixes

- [ ] Problem: Runtime execution competes with UI rendering on the egui frame thread.
  Context: Load/dispatch/drain/tick work on the UI thread can cause frame drops under heavier workloads.
  Context (Files): `examples/runtime-jsx/app.rs`, `examples/runtime-jsx-motion.rs`, `crates/clay-jsx-egui-bridge/src/runtime.rs`.
  Verification: Introduce worker/session-thread architecture and benchmark proving improved UI responsiveness during burst workloads.

- [ ] Problem: Pending runtime updates are drained only once per frame.
  Context: Single-drain behavior can leave backlog and multi-frame convergence lag during async bursts.
  Context (Files): `examples/runtime-jsx/app.rs`, `examples/runtime-jsx-motion.rs`, `crates/clay-jsx-egui-bridge/src/runtime.rs`.
  Verification: Add bounded-loop drain tests asserting latest-state convergence within configured frame budgets.

- [ ] Problem: Due-timer transfer uses repeated JSON stringify/parse on hot paths.
  Context: Serialization overhead adds avoidable allocation and CPU cost during frequent wake cycles.
  Context (Files): `crates/clay-jsx-runtime/src/runtime.rs`, `crates/clay-jsx-runtime/src/host_runtime_api.js`.
  Verification: Replace with typed op transfer and benchmark reduced allocations/latency versus baseline.

- [ ] Problem: Tailwind class parsing/effective-layout derivation repeats per node and per path.
  Context: Repeated parse work scales poorly with large trees and duplicate Taffy traversal.
  Context (Files): `src/contract/renderer.rs`, `src/ui/tailwind/parse.rs`, `src/ui/tailwind/parse_layout.rs`.
  Verification: Add cache keyed by stable identity/hash and microbenchmarks showing lower `render_tree` CPU time on large trees.

- [ ] Problem: Motion mutation payloads include unnecessary `clear_motion` operations.
  Context: Emitting clears for nodes that never had motion inflates mutation payload size and apply work.
  Context (Files): `crates/clay-jsx-egui-bridge/src/runtime_api.js`, `crates/clay-jsx-egui-bridge/src/host_tree.rs`, `crates/clay-jsx-egui-bridge/src/runtime.rs`.
  Verification: Add mutation diff tests and benchmarks showing reduced payload size and apply latency on non-motion-heavy screens.

- [ ] Problem: Reorder/index operations use repeated sibling scans in hot commit paths.
  Context: O(n^2)-like behavior can surface under large sibling lists and frequent reorders.
  Context (Files): `crates/clay-jsx-egui-bridge/src/runtime_api.js`.
  Verification: Add deep-reorder benchmarks proving improved p95 commit time after indexed parent-child mapping.

## A-Grade Evidence Gates

- [ ] Problem: No standardized, reproducible benchmark contract defines A-grade performance.
  Context: Performance claims require stable p50/p95 metrics for core runtime operations on representative tree sizes.
  Context (Files): `examples/runtime-jsx/mod.rs`, `reference/benchmarks/`.
  Verification: Add benchmark target and artifacts tracking `dispatch_events`, `drain_pending_runtime_updates`, and `render_tree` latency.

- [ ] Problem: Reliability soak/stress gates are not mandatory in automated release validation.
  Context: A-grade reliability requires enforced CI gating on leak/recovery stability, not optional/manual runs.
  Context (Files): `examples/runtime-jsx/mod.rs`, `.github/workflows/`.
  Verification: Add CI jobs for soak/stress harnesses with machine-readable artifact checks and failing exit codes on gate violations.

- [ ] Problem: Release criteria for A-grade are not codified as enforceable thresholds.
  Context: Without explicit thresholds, quality grade remains subjective and can regress unnoticed.
  Context (Files): `TASK.md`, `scripts/`, `docs/`.
  Verification: Add checked-in release checklist + automated gate script requiring zero critical/high known defects and passing latency/reliability thresholds.

