Target

The right end-state is:

- clay-jsx-runtime is the only JS/V8 substrate. It owns module loading, React, react/jsx-runtime, react-reconciler,
  scheduler, logging, and host-op plumbing.
- clay-jsx-egui-bridge is an egui-specific renderer adapter on top of that substrate. It owns the egui authoring facade,
  contract-family normalization, semantic event codec, motion specs, and the host config that turns React commits into Rust
  HostMutations.
- Rust keeps one authoritative retained tree: the current contract/host tree, not raw egui widgets. In this repo that means
  keeping crates/clay-jsx-egui-bridge/src/host_tree.rs:55 and src/contract/model.rs:615 as the retained model.
- egui stays immediate-mode. Each frame, src/contract/renderer.rs:21 replays the current retained tree, and egui keeps only
  its normal transient memory keyed from stable ids like src/contract/renderer.rs:2914.
- React owns semantic UI state. Rust owns only retained host state, motion/timer sidecars, and egui-local transient
  interaction memory.
- Repaints come from exactly three sources: React commits, egui/input-driven interaction, and timers/animation. Nothing
  should poll continuously in the steady state.

Two important design rules:

- key is for React sibling identity. node_id is for host semantics and egui::Id stability. Do not treat tree-position
  fallback ids as the long-term identity model.
- React commits should emit semantic HostMutations directly. Do not make the final egui bridge protocol a DOM-like {create,
  insert, remove} stream with numeric runtime ids.

Path

1. Lock current behavior with tests before changing architecture.
    Verification: add snapshot and roundtrip tests for initial tree, event-driven updates, keyed reorder/state preservation,
    motion ticks, and tricky families like text input, popover, select, hierarchy, and drag board.
2. Unify the JS substrate without changing behavior.
    Keep the current egui behavior, but run it through clay-jsx-runtime as the single runtime/session layer. The goal here is
    to remove runtime duplication, not to switch rendering semantics yet.
    Verification: existing JsxRuntimeSession tests and example surfaces produce the same RenderedJsx { tree, motion } outputs
    as before.
3. Build a new React-backed egui renderer in parallel.
    Replace the bespoke hook/reconcile engine in crates/clay-jsx-egui-bridge/src/runtime_api.js:428 and crates/clay-jsx-egui-
    bridge/src/runtime_api.js:490 with a react-reconciler host config that commits directly into HostMutationBatch.
    Verification: same TSX file plus same event stream must yield the same materialized ContractTree and MotionFrame as the
    legacy path.
4. Keep the authoring API stable while swapping internals.
    import { render, useState } from "egui" can remain, but useState should become a React re-export and render should
    create/update the React root. egui/jsx-runtime should become a React-compatible runtime facade.
    Verification: examples compile unchanged, or with only mechanical import edits if you choose to expose react directly.
5. Fix repaint orchestration as a first-class subsystem.
    Replace per-frame dependency stamp polling with a watcher/wake path. The session should expose “commit happened” and
    “next timer deadline” signals so the shell can call request_repaint() or request_repaint_after(...) explicitly.
    Verification: idle surfaces stay idle, file edits wake the UI immediately, and motion/timed widgets animate without
    continuous polling.
6. Audit family-by-family state ownership.
    For every contract family, specify whether value/open/selection is JS-authoritative and which transient bits remain in
    egui memory. Text input, IME, focus, popup open state, scroll retention, and drag interactions need explicit tests.
    Verification: stable node_id preserves widget memory across re-renders; changing identity resets it when expected.
7. Remove the legacy runtime only after differential parity is closed.
    The current custom hook snapshotting in runtime_api.js is not a good long-term foundation. If hot-reload state
    preservation still matters after the React cutover, add an explicit persisted-state API instead of serializing React
    internals.
    Verification: catalog and example apps run only on the new path, and the legacy engine can be deleted.
