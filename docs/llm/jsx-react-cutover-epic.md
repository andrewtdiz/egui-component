# JSX React Cutover Epic

## Problem Statement

The current JSX bridge works, but the egui-facing runtime is split across two different models:

- `crates/clay-jsx-runtime` already contains a host-neutral React and `react-reconciler` path.
- `crates/clay-jsx-egui-bridge/src/runtime_api.js` still runs a bespoke JSX runtime with its own `useState`, root rerender loop, and manual reconciliation.

That split is the wrong long-term architecture for this repo.

It duplicates runtime concerns, keeps egui on a custom state/reconcile path instead of the same React model we want authored code to target, and makes identity, repaint scheduling, and event semantics harder to reason about than they need to be.

The goal of this epic is to cut over the egui bridge to a React-backed retained-host architecture quickly and accurately, without trying to turn `egui` itself into a retained toolkit.

## Current Context

The repo already has the right Rust-side rendering boundary:

- `src/contract/*` is the active declarative render boundary.
- `crates/clay-jsx-egui-bridge/src/host_tree.rs` already owns a retained host tree and applies semantic `HostMutation` batches.
- `crates/clay-jsx-egui-bridge/src/runtime.rs` already materializes `ContractTree` from the retained host tree.
- `src/contract/renderer.rs` already replays the current `ContractTree` into egui each frame.
- `src/contract/renderer.rs` already derives stable `egui::Id` values from `node_id`.

The current mismatch is in the JS/runtime layer:

- `crates/clay-jsx-egui-bridge/src/runtime_api.js` implements a custom `jsx`, `useState`, event dispatch loop, and tree reconciliation.
- `crates/clay-jsx-runtime/src/clay_jsx_runtime.ts` already has a real `react-reconciler` host config and commit path, but the egui bridge is not using that model as its active runtime.

This means the repo is already structurally close to the target design. The cutover is primarily about replacing the bespoke egui-side runtime with the shared React-host model while keeping the Rust retained tree and egui renderer intact.

## Target Architecture

The end-state should be:

- `clay-jsx-runtime` is the only JS/V8 substrate.
- `clay-jsx-runtime` owns module loading, React, `react/jsx-runtime`, `react/jsx-dev-runtime`, `react-reconciler`, scheduler, and host ops.
- `clay-jsx-egui-bridge` becomes an egui-specific adapter on top of that substrate.
- `clay-jsx-egui-bridge` owns the `egui` authoring facade, contract-family normalization, event aliases, text lowering, and motion normalization.
- Rust keeps one retained model: `HostTree` plus motion sidecar, which materializes a `ContractTree`.
- `egui` stays immediate-mode and only renders the current retained `ContractTree` each frame.
- React owns component state and reconciliation.
- Rust owns retained host state, semantic event dispatch, motion/timer state, and egui-local transient widget memory.

The runtime data flow should be:

`TSX -> React -> react-reconciler host config -> HostMutationBatch -> Rust HostTree -> ContractTree -> egui frame -> ContractEvent -> React state update`

## Design Decisions

These decisions should be treated as fixed for this migration:

- Do not fork or redesign `egui` into a retained-mode toolkit.
- Do not add a second retained widget tree below `ContractTree`.
- Keep `ContractTree` and `ContractNode` as the retained host boundary.
- Keep `egui` as the frame renderer and interaction layer.
- `node_id` is the durable host identity.
- `key` is only for React sibling reconciliation.
- `egui::Id` continues to derive from `node_id`.
- The egui bridge keeps text lowering and HTML-ish sugar because the contract model does not have a native text node.
- Hot-reload hook-state restoration is not part of the first cutover pass.

## Non-Goals

- No attempt to patch already-rendered egui widgets in place.
- No broad component redesign during the runtime cutover.
- No large contract schema redesign unless a migration blocker forces it.
- No attempt to preserve every current hot-reload behavior before the React cutover is complete.
- No long-lived dual-runtime strategy. The legacy bespoke runtime should be removed once the new path clears the migration gates.

## Scope Rules

The migration should follow these scope constraints:

- Reuse the current Rust retained tree and renderer instead of rebuilding them.
- Reuse the current egui-side prop normalization, family aliasing, event aliasing, and motion parsing semantics unless they are wrong.
- Keep the authored TSX surface stable where possible: `import { render, useState, eventValue } from "egui"` can remain during the cutover.
- Treat path-derived fallback ids as transitional. Stateful and interactive widgets should have explicit stable `node_id` values.

## Phased Migration Plan

### Phase 1: Introduce a React-Backed Egui Runtime Beside the Legacy Path

Build a new egui runtime entrypoint that uses real React and `react-reconciler`, but leave the current bespoke runtime in place until parity is reached.

Work:

- Add a new egui runtime module that exposes `render`, `useState`, `eventValue`, and the JSX runtime through real React.
- Wire `clay-jsx-egui-bridge/src/runtime.rs` to register the React modules exported by `clay-jsx-runtime`.
- Keep the current author-facing `"egui"` import surface stable.
- Do not try to preserve the legacy custom hook implementation.

Outcome:

- The egui bridge can run authored TSX through React instead of the bespoke hook/reconcile engine.

### Phase 2: Move Egui Lowering Into Pure Bridge Helpers

Extract the egui-specific lowering rules from the current bespoke runtime into reusable pure helpers.

Work:

- Extract family aliasing, prop aliasing, layout hoisting, event aliasing, child flattening, text lowering, and motion normalization out of the current `runtime_api.js` flow.
- Reuse those helpers from the new React-backed runtime so behavior stays stable while internals change.
- Keep contract-node materialization in the egui bridge, not the shared runtime crate.

Outcome:

- The egui bridge keeps its current authored semantics without depending on the bespoke hook engine.

### Phase 3: Commit Directly Into `HostMutationBatch`

The new React host config should emit the Rust-side semantic mutations that the bridge already understands.

Work:

- Make the React host config queue `replace_root`, `insert_subtree`, `remove_subtree`, `replace_subtree`, `update_node`, `set_children`, `set_motion`, and `clear_motion`.
- Feed those mutations into the existing Rust `HostTree`.
- Avoid introducing a DOM-like low-level command stream as the long-term egui protocol.

Outcome:

- React commits mutate the same retained Rust tree that the current bridge already renders.

### Phase 4: Tighten Identity and Stateful Widget Rules

Identity needs to be explicit enough for React state and egui widget memory to both stay correct.

Work:

- Define which contract families must have explicit stable `node_id` values.
- Keep `key` as React-only sibling identity.
- Continue deriving `egui::Id` from `node_id`.
- Restrict fallback path-derived ids to stateless layout and text sugar.

Outcome:

- Focus, popup memory, text editing state, and other egui-local transient state remain stable across normal rerenders.

### Phase 5: Repaint and Wake-Up Cleanup

The host shell should repaint only when it actually needs another frame.

Work:

- Schedule repaint on React commits that change the retained tree.
- Schedule repaint for active motion and timer-driven behavior with `request_repaint()` or `request_repaint_after(...)`.
- Replace passive polling where practical with explicit wake-up paths for file changes and runtime-triggered commits.

Outcome:

- Idle surfaces stay idle.
- Content updates, motion, and interaction still wake the next frame immediately.

### Phase 6: Switch the Examples and Delete the Legacy Runtime

After the React-backed path is working, make it the only path.

Work:

- Point `examples/runtime-jsx` and `examples/runtime-jsx-motion` at the new runtime by default.
- Remove the bespoke `useState`, root rerender loop, and manual reconciliation code from the egui bridge.
- Update docs that still describe the legacy runtime behavior.

Outcome:

- There is one active JSX runtime model in the repo, not two.

## Minimum Migration Gates

This migration does not need broad ceremonial lock-down, but it does need a small set of hard gates before the legacy runtime is removed.

The new runtime path must satisfy all of these:

- `examples/runtime-jsx/app.jsx` loads and interactive state updates correctly.
- `examples/runtime-jsx/catalog.tsx` renders without obvious family or prop regressions.
- `examples/runtime-jsx/motion-sync.tsx` still commits motion specs and animates correctly.
- A keyed reorder case preserves identity correctly.
- A text input case preserves focus and edit behavior across normal rerenders.
- Idle surfaces do not continuously repaint when nothing is changing.

If any of those gates fail, the cutover is not complete.

## Risks

The migration risks are concentrated in a few areas:

- Identity drift between React `key`, contract `node_id`, and egui widget ids.
- Stateful controls such as text input, popups, selection widgets, and drag interactions.
- Motion timing and repaint scheduling.
- Behavioral drift while porting the current egui-specific lowering rules into the React-backed runtime.

These are the areas to verify aggressively during the cutover. Most other behavior should fall out of the existing retained Rust tree and contract renderer.

## Epic Completion Criteria

This epic is complete when all of the following are true:

- The egui bridge runs authored TSX on top of React and `react-reconciler`.
- The egui bridge commits directly into the existing Rust `HostTree`.
- The current `ContractTree` renderer remains the active egui render boundary.
- The legacy bespoke egui runtime has been removed.
- The runtime only repaints on commits, interaction, and timer-driven work.

At that point the repo has a single coherent architecture:

- React owns the retained authored tree and state model.
- Rust owns the retained host tree and egui bridge.
- egui owns frame-time rendering and transient interaction memory.
