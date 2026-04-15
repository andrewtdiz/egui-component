# JSX Runtime v1 Host-Config Contract

This document freezes the minimum supported v1 host-config surface for the
retained `clay-jsx-runtime` + `clay-jsx-egui-bridge` stack.

Source of truth:

- `crates/clay-jsx-egui-bridge/src/runtime_api.ts`
- targeted contract tests in `crates/clay-jsx-egui-bridge/src/runtime.rs`
- session-level commit protocol doc: `docs/jsx-runtime-v1-commit-protocol.md`
- session-level scheduling doc: `docs/jsx-runtime-v1-scheduling-model.md`

## Architectural mode

The v1 reconciler runs in **mutation mode**.

- `supportsMutation = true`
- `supportsMicrotasks = true`
- `supportsHydration = false`
- `supportsPersistence = false`

The JS reconciler owns only lightweight shadow nodes. Rust owns the retained
`HostTree` and any renderer-visible native state.

## JS shadow instances

`createInstance()` and `createTextInstance()` allocate **JS shadow nodes** used
for reconciliation and lowering only.

- `getPublicInstance(instance)` returns that same JS shadow object.
- `getInstanceFromNode()` returns `null`.
- `getInstanceFromScope()` returns `null`.

v1 therefore does **not** support direct native host refs, DOM-style node
lookups, or any API that would hand JS a retained/native renderer object.

## Frozen v1 host-config responsibilities

The checked-in v1 host-config supports only the following responsibilities:

### Structural mutation surface

- `createInstance(type, props)`
- `createTextInstance(text)`
- `appendInitialChild(parent, child)`
- `appendChild(parent, child)`
- `appendChildToContainer(container, child)`
- `insertBefore(parent, child, beforeChild)`
- `insertInContainerBefore(container, child, beforeChild)`
- `removeChild(parent, child)`
- `removeChildFromContainer(container, child)`
- `clearContainer(container)`
- `resetTextContent(instance)`

### Update surface

- `commitUpdate(instance, type, prevProps, nextProps)`
- `commitTextUpdate(textInstance, oldText, newText)`
- `hideInstance(instance)` / `unhideInstance(instance)`
- `hideTextInstance(textInstance)` / `unhideTextInstance(textInstance)`

### Scheduling surface

- `scheduleMicrotask(fn)`
- `requestPostPaintCallback(callback)`
- `scheduleTimeout(fn, delay)`
- `cancelTimeout(id)`

These scheduling hooks are lightweight JS entry points only. Timer ownership,
host wake delivery, bounded callback draining, and multi-drain convergence
remain Rust/runtime responsibilities. The frozen wake contract is documented in
`docs/jsx-runtime-v1-scheduling-model.md`.

## Commit boundary

The host-config never creates or mutates native renderer objects directly.

Instead it:

1. mutates only JS shadow nodes,
2. lowers committed shadow subtrees into declarative `HostMutationBatch`
   payloads, and
3. submits them through `enqueueCommitBatch()`.

`enqueueCommitBatch()` is the only retained-state mutation boundary.

- JSON is the default backend.
- typed commit transport is allowed only behind the same abstraction for tests
  and benchmarks.

The session-atomic commit state machine layered on top of that boundary is
frozen separately in `docs/jsx-runtime-v1-commit-protocol.md`.

## Frozen v1 semantics

The targeted host-config tests freeze these behavioral expectations:

- initial create + append-to-container emits `replace_root`
- append of a new materialized child emits `insert_subtree`
- insert-before of a new child emits indexed `insert_subtree`
- remove of a materialized child emits `remove_subtree`
- reorder of existing siblings emits `set_children`
- text updates emit `update_node`
- hide emits `remove_subtree`
- unhide emits `insert_subtree`

## Explicitly out of scope for v1

These features are intentionally unsupported and must not be treated as stable
contract surface for v1:

- hydration
- persistence mode
- direct native host refs / retained native instance handles in JS
- `getInstanceFromNode()` and `getInstanceFromScope()` lookups
- JS-side creation or mutation of native renderer objects outside committed
  batch transport
- expanding the host-config into a larger feature surface before the retained
  mutation contract is proven stable
