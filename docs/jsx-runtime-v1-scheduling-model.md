# JSX Runtime v1 Scheduling Model

This document freezes the minimum supported v1 wake / timer / invalidation model
shared by:

- `crates/clay-jsx-runtime/src/host_runtime_api.js`
- `crates/clay-jsx-runtime/src/runtime.rs`
- `crates/clay-jsx-egui-bridge/src/runtime.rs`

It defines how JS reports pending async work to the host and how the host must
drain that work without allowing unbounded callback bursts in one wake.

## Why this exists

The runtime already exposes several ways to ask for later work:

- `requestRepaint()`
- `setTimeout()` / `setInterval()`
- `requestAnimationFrame()`
- React effect work that schedules any of the above

v1 treats all of those as one explicit scheduling contract.

## Core invariant

The host only needs one scheduling bit: **`pending_host_wake`**.

If that bit is true, the host must schedule one drain / frame opportunity.

All cross-turn JS work converges through that same wake channel:

- timer worker enqueues due timer handles and requests a wake
- `requestAnimationFrame()` is implemented as a host-owned timer and therefore
  also requests a wake when due
- `requestRepaint()` requests a wake directly even when no timer callbacks are
  pending
- if a bounded drain cannot finish all queued timer / RAF callbacks in one
  batch, JS must request another wake before returning

## Stable v1 sources of wake requests

### Invalidation-only wake

`requestRepaint()` is the explicit invalidation primitive.

- it does **not** enqueue a timer callback
- the subsequent drain may invoke zero callbacks
- the host must still honor the wake and schedule one drain / repaint cycle

### Timer wake

`setTimeout()` and `setInterval()` are host-owned timers.

- Rust owns timer handles and due-time bookkeeping
- when timers become due, Rust queues the due handles and requests a wake
- repeating timers coalesce missed ticks into one callback after a long host
  stall so one stalled interval does not replay an unbounded burst

### RAF wake

`requestAnimationFrame()` is implemented as a one-shot host timer.

- it participates in the same bounded drain contract as normal timers
- one fired RAF callback consumes one callback slot in a drain batch

## Bounded drain contract

One host drain may invoke **at most 128 callbacks** in v1.

Source of truth:

- JS batch cap: `MAX_HOST_CALLBACKS_PER_DRAIN_CALL` in
  `crates/clay-jsx-runtime/src/host_runtime_api.js`
- Rust export: `HOST_CALLBACK_DRAIN_LIMIT` in
  `crates/clay-jsx-runtime/src/runtime.rs`

`RuntimeSession::drain_host_callbacks()` therefore performs **one bounded JS
drain batch per call**.

Its result tells the host:

- whether a wake was actually drained
- how many callbacks were invoked in that bounded batch
- whether another wake is already pending after the batch

## When JS must request another wake

JS must request another wake before returning from a bounded drain when either
of these is true:

1. timer / RAF handles remain queued after consuming the per-drain cap
2. callbacks executed in the current drain schedule new cross-turn work that is
   now pending

In v1 the JS helper `__clayDrainHostCallbacks()` is responsible for requesting
that follow-up wake when already-loaded due handles remain after a capped batch.

## When the host must schedule another drain

After every drain attempt, the host must inspect whether work is still pending.

The host must schedule another drain / frame when either of these is true:

- the drain result reports `pending_host_wake_after_drain = true`
- the bridge/session metrics still report `pending_host_wake = true`

This means convergence is intentionally multi-drain for worst-case bursts.

## Bridge-level contract

`JsxRuntimeSession::drain_pending_runtime_updates()` drains **one bounded wake**
and then materializes any queued commit batches into `RenderedJsx`.

Embedders must therefore treat it as a single scheduling step, not a
"drain everything forever" call.

If a drain returns a render while `pending_host_wake` is still true, the host
must render that result and schedule another drain.

## Non-goals for v1

These behaviors are intentionally out of scope for the frozen v1 model:

- unbounded callback draining in one host wake
- bypassing `pending_host_wake` with renderer-specific scheduling channels
- letting JS own native timer or repaint objects
- guaranteeing that one wake always converges all pending async work

## Checked-in coverage

The checked-in tests freeze this model with coverage for:

- effect-driven updates
- timer-driven updates
- RAF callbacks
- callback-free invalidation wakes via `requestRepaint()`
- bounded multi-drain convergence under callback bursts
- worst-case burst benchmarking with cap assertions
