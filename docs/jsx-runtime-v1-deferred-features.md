# JSX runtime v1 deferred features

This document freezes the features that are **intentionally out of scope** for
the reliable v1 retained-renderer architecture.

The rule for follow-up work is simple: if a feature appears in this list, it is
not part of the supported v1 contract until code, docs, tests, and host-facing
recovery semantics are updated together.

## Deferred in v1

- **Hydration**
  - `supportsHydration = false`
  - The retained renderer does not reconcile against pre-existing native/UI
    objects.

- **Persistence mode**
  - `supportsPersistence = false`
  - The bridge supports the mutation host-config path only.

- **Direct native host refs / public native instances**
  - `getInstanceFromNode()` and `getInstanceFromScope()` return `null`
  - `getPublicInstance(instance) === instance` only for JS shadow nodes
  - JS must not hold or expose native egui renderer objects as public instances.

- **Hook-state restoration across reload**
  - `HotReloadState` remains API-compatibility only in v1
  - reload is a validated cold remount after deterministic teardown
  - partial hook-state restoration is deferred work, not an implied guarantee

- **Production transport replacement**
  - JSON is the only production-supported commit transport in v1
  - typed commit transport exists only as an explicit test/benchmark experiment
  - embedders must not assume that switching production sessions away from JSON
    is supported or stable in v1

## Guard policy

When code touches one of these deferred areas, v1 requires an explicit guard:

- reject unsupported modes loudly instead of silently pretending they work
- document the deferral at the API boundary where callers would otherwise infer
  support
- keep experiments behind test/benchmark-only flags or helper surfaces
- add or update checked-in docs before broadening the contract

## Review checklist for follow-up architecture work

Before merging a feature that touches the runtime boundary, verify:

1. it is not accidentally expanding one of the deferred v1 surfaces above
2. JS still behaves as a declarative mutation producer rather than a native-host
   owner
3. the host-visible recovery/commit/scheduling contracts still match the docs
4. any newly supported surface is documented and covered by checked-in tests
