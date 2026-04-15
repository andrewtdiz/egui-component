#![doc = "Configurable deno_core-backed TS/TSX JSX runtime primitives for Clay host bridges."]

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
    JsxRuntimeOptions, RuntimeDebugMetrics, RuntimeHostDebugCounters, RuntimeSession,
    RuntimeUpdate, VirtualModule,
};
