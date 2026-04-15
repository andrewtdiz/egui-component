#![doc = r#"
Configurable `deno_core`-backed TS/TSX JSX runtime primitives for Clay host bridges.

## Ownership boundary

This crate owns the embedded `JsRuntime`, the shared React substrate, and the
host-neutral ops that renderer bridges install into JS. It does **not** own any
renderer-specific retained UI objects.

- JS owns the ephemeral reconciler state that lives inside V8.
- JS may only push renderer state across the boundary by enqueueing commit
  batches through the runtime commit transport abstraction.
- The default backend uses `Deno.core.ops.op_commit_mutations(...)` with JSON,
  and typed transport experiments remain behind that same abstraction.
- The remaining ops in this crate are deliberately limited to diagnostics,
  reconciler telemetry, timers, and host wake scheduling.
- Renderer-specific crates own commit-batch decoding, validation, retained-tree
  mutation, materialization, and recovery policy.

In other words: `clay-jsx-runtime` provides a host-neutral queue and scheduling
surface, not a JS API for creating or mutating native renderer objects.

The frozen minimal v1 host-config contract is checked in at
`docs/jsx-runtime-v1-host-config-contract.md`.

The frozen minimal v1 commit protocol is checked in at
`docs/jsx-runtime-v1-commit-protocol.md`.

The frozen minimal v1 scheduling model is checked in at
`docs/jsx-runtime-v1-scheduling-model.md`.

That checked-in v1 surface freezes mutation mode, JS shadow public instances,
microtask/timer scheduling hooks, and explicit non-support for hydration,
persistence, and direct native host refs.

The checked-in commit protocol freezes the session-atomic states and
transitions for enqueue, decode, validate, acknowledge, reject, and fail.

The checked-in scheduling model freezes one host-visible async-work bit
(`pending_host_wake`), a bounded per-drain callback cap, and the rule that the
host must schedule another drain when a bounded batch leaves work pending.
"#]

pub mod contract;
pub mod diagnostics;
mod runtime;
mod runtime_components;

pub const CLAY_JSX_RUNTIME_SPECIFIER: &str = "clay-internal:/jsx-runtime";
pub const CLAY_JSX_RUNTIME_SOURCE_PATH: &str = "crates/clay-jsx-runtime/src/clay_jsx_runtime.ts";
pub const CLAY_JSX_RUNTIME_SOURCE: &str = include_str!("clay_jsx_runtime.ts");
pub const HOST_RUNTIME_SOURCE: &str = include_str!("host_runtime_api.js");

pub const REACT_SOURCE: &str = include_str!("vendor/react_19_2_5.mjs");
pub const REACT_JSX_RUNTIME_SOURCE: &str = include_str!("vendor/react_jsx_runtime_19_2_5.mjs");
pub const REACT_JSX_DEV_RUNTIME_SOURCE: &str =
    include_str!("vendor/react_jsx_dev_runtime_19_2_5.mjs");
pub const REACT_RECONCILER_SOURCE: &str = include_str!("vendor/react_reconciler_0_33_0.mjs");
pub const SCHEDULER_SOURCE: &str = include_str!("vendor/scheduler_0_27_0.mjs");

pub use diagnostics::{
    extend_logs, push_log, RuntimeLogBuffer, LOG_HISTORY_LIMIT, LOG_MESSAGE_LIMIT_BYTES,
};
pub use runtime::{
    JsxRuntimeOptions, RuntimeCommitBatch, RuntimeCommitBatchPayload, RuntimeCommitTransportKind,
    RuntimeDebugMetrics, RuntimeHostCallbackDrainResult, RuntimeHostDebugCounters,
    RuntimeReconcilerError, RuntimeReconcilerErrorCategory, RuntimeRecoveryCategory,
    RuntimeRecoveryDisposition, RuntimeRecoveryState, RuntimeSession, RuntimeUpdate, VirtualModule,
    HOST_CALLBACK_DRAIN_LIMIT,
};
