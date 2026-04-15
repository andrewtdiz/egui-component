use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
        mpsc::{self, RecvTimeoutError, Sender},
        Arc, LazyLock, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context as AnyhowContext};
use deno_ast::{
    DecoratorsTranspileOption, EmitOptions, ImportsNotUsedAsValues, JsxAutomaticOptions,
    JsxRuntime, MediaType, ParseParams, SourceMapOption, TranspileModuleOptions, TranspileOptions,
};
use deno_core::{
    extension, op2, resolve_import, serde_v8, v8, JsRuntime, ModuleLoadOptions, ModuleLoadReferrer,
    ModuleLoadResponse, ModuleLoader, ModuleSource, ModuleSourceCode, ModuleSpecifier, ModuleType,
    OpState, ResolutionKind, RuntimeOptions as DenoRuntimeOptions,
};
use deno_error::JsErrorBox;
use serde::de::DeserializeOwned;

use crate::{
    diagnostics::{push_log, RuntimeLogBuffer},
    HOST_RUNTIME_SOURCE, REACT_JSX_DEV_RUNTIME_SOURCE, REACT_JSX_RUNTIME_SOURCE,
    REACT_RECONCILER_SOURCE, REACT_SOURCE, SCHEDULER_SOURCE,
};

type SourceMapStore = Rc<RefCell<HashMap<String, Vec<u8>>>>;

static HOST_TIME_ORIGIN: LazyLock<Instant> = LazyLock::new(Instant::now);

#[derive(Debug, Clone)]
pub struct JsxRuntimeOptions {
    pub jsx_import_source: String,
    pub virtual_modules: Vec<VirtualModule>,
}

impl JsxRuntimeOptions {
    pub fn new(jsx_import_source: impl Into<String>) -> Self {
        Self {
            jsx_import_source: jsx_import_source.into(),
            virtual_modules: Vec::new(),
        }
    }

    pub fn with_virtual_module(
        mut self,
        import_specifier: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        self.virtual_modules
            .push(VirtualModule::new(import_specifier, source));
        self
    }

    pub fn with_react_runtime_modules(self) -> Self {
        self.with_virtual_module("react", REACT_SOURCE)
            .with_virtual_module("react/jsx-runtime", REACT_JSX_RUNTIME_SOURCE)
            .with_virtual_module("react/jsx-dev-runtime", REACT_JSX_DEV_RUNTIME_SOURCE)
            .with_virtual_module("clay-internal:/react-reconciler", REACT_RECONCILER_SOURCE)
            .with_virtual_module("clay-internal:/scheduler", SCHEDULER_SOURCE)
    }
}

#[derive(Debug, Clone)]
pub struct VirtualModule {
    pub import_specifier: String,
    pub source: String,
}

impl VirtualModule {
    pub fn new(import_specifier: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            import_specifier: import_specifier.into(),
            source: source.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct RuntimeUpdate {
    pub commit_batches: Vec<RuntimeCommitBatch>,
    pub reconciler_errors: Vec<RuntimeReconcilerError>,
    pub logs: RuntimeLogBuffer,
    pub host_debug_counters: RuntimeHostDebugCounters,
}

impl RuntimeUpdate {
    pub fn commit_batch_ids(&self) -> Vec<u32> {
        self.commit_batches
            .iter()
            .map(|batch| batch.commit_batch_id)
            .collect()
    }

    pub fn commit_batches_json(&self) -> anyhow::Result<Vec<String>> {
        self.commit_batches
            .iter()
            .map(RuntimeCommitBatch::payload_json)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCommitTransportKind {
    /// The frozen v1 production transport. All supported embedder flows must be
    /// able to rely on JSON as the stable commit boundary.
    Json,
    /// Test/benchmark-only experimental transport. Production transport
    /// replacement is intentionally deferred in v1.
    Typed,
}

/// Frozen v1 production transport posture: JSON is the only supported commit
/// transport for non-experimental sessions.
pub const V1_PRODUCTION_COMMIT_TRANSPORT: RuntimeCommitTransportKind =
    RuntimeCommitTransportKind::Json;

impl RuntimeCommitTransportKind {
    pub fn is_deferred_for_v1_production(self) -> bool {
        self != V1_PRODUCTION_COMMIT_TRANSPORT
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "transport", content = "payload", rename_all = "snake_case")]
pub enum RuntimeCommitBatchPayload {
    Json(String),
    Typed(serde_json::Value),
}

impl RuntimeCommitBatchPayload {
    fn transport_kind(&self) -> RuntimeCommitTransportKind {
        match self {
            Self::Json(_) => RuntimeCommitTransportKind::Json,
            Self::Typed(_) => RuntimeCommitTransportKind::Typed,
        }
    }

    fn payload_json(&self) -> anyhow::Result<String> {
        match self {
            Self::Json(json) => Ok(json.clone()),
            Self::Typed(value) => serde_json::to_string(value)
                .map_err(|error| anyhow!("failed to encode typed commit batch as JSON: {error}")),
        }
    }

    fn payload_bytes(&self) -> anyhow::Result<usize> {
        Ok(self.payload_json()?.len())
    }

    fn decode_as<T: DeserializeOwned>(&self) -> anyhow::Result<T> {
        match self {
            Self::Json(json) => serde_json::from_str(json)
                .map_err(|error| anyhow!("failed to decode JSON commit batch: {error}")),
            Self::Typed(value) => serde_json::from_value(value.clone())
                .map_err(|error| anyhow!("failed to decode typed commit batch: {error}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeCommitBatch {
    pub commit_batch_id: u32,
    pub payload: RuntimeCommitBatchPayload,
}

impl RuntimeCommitBatch {
    pub fn transport_kind(&self) -> RuntimeCommitTransportKind {
        self.payload.transport_kind()
    }

    pub fn payload_json(&self) -> anyhow::Result<String> {
        self.payload.payload_json()
    }

    pub fn decode_as<T: DeserializeOwned>(&self) -> anyhow::Result<T> {
        self.payload.decode_as()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeReconcilerErrorCategory {
    Uncaught,
    Caught,
    Recoverable,
}

impl RuntimeReconcilerErrorCategory {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "uncaught" => Some(Self::Uncaught),
            "caught" => Some(Self::Caught),
            "recoverable" => Some(Self::Recoverable),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Uncaught => "uncaught",
            Self::Caught => "caught",
            Self::Recoverable => "recoverable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeReconcilerError {
    pub category: RuntimeReconcilerErrorCategory,
    pub message: String,
    #[serde(default)]
    pub component_stack: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_boundary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeRecoveryCategory {
    Recoverable,
    BoundaryContained,
    Fatal,
    ProtocolFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeRecoveryDisposition {
    Continue,
    BoundedFailure,
    ReloadRequired,
    Teardown,
}

impl Default for RuntimeRecoveryDisposition {
    fn default() -> Self {
        Self::Continue
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeRecoveryState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<RuntimeRecoveryCategory>,
    #[serde(default)]
    pub disposition: RuntimeRecoveryDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default)]
    pub component_stack: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_boundary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejected_commit_batch_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeHostDebugCounters {
    pub render_call_count: u64,
    pub unmount_count: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeHostCallbackDrainResult {
    pub had_pending_wake: bool,
    pub callbacks_invoked: u64,
    pub pending_host_wake_after_drain: bool,
}

impl RuntimeHostCallbackDrainResult {
    pub fn drained_work(self) -> bool {
        self.had_pending_wake
    }

    pub fn needs_another_drain(self) -> bool {
        self.pending_host_wake_after_drain
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeDebugMetrics {
    pub host_wake_count: u64,
    pub host_wake_callback_count: u64,
    pub host_callback_drain_cycles: u64,
    pub host_callbacks_invoked: u64,
    pub host_callback_drain_noop_count: u64,
    pub timer_schedule_count: u64,
    pub timer_cancel_count: u64,
    pub timer_fire_count: u64,
    pub timer_repeat_fire_count: u64,
    pub active_timer_count: u64,
    pub active_timer_high_water: u64,
    pub shutdown_count: u64,
    pub pending_host_wake: bool,
    pub pending_mutation_batch_count: u64,
    pub pending_mutation_batch_bytes: u64,
    pub pending_mutation_batch_high_water: u64,
    pub pending_mutation_batch_bytes_high_water: u64,
    pub pending_mutation_batch_overflow_count: u64,
    pub pending_mutation_batch_limit: u64,
    pub pending_mutation_batch_byte_limit: u64,
}

const MAX_PENDING_MUTATION_BATCHES: usize = 256;
const MAX_PENDING_MUTATION_BATCH_BYTES: usize = 4 * 1024 * 1024;
pub const HOST_CALLBACK_DRAIN_LIMIT: usize = 128;

#[derive(Debug, Clone, Default)]
struct HostRuntimeBridge {
    inner: Arc<HostRuntimeBridgeInner>,
}

struct HostRuntimeBridgeInner {
    is_shutdown: AtomicBool,
    wake: RuntimeWakeState,
    next_timer_handle: AtomicU32,
    pending_due_timers: Mutex<Vec<u32>>,
    pending_due_timer_set: Mutex<HashSet<u32>>,
    active_timers: Mutex<HashSet<u32>>,
    metrics: RuntimeDebugCounterState,
    timer_command_tx: Mutex<Option<Sender<TimerCommand>>>,
    timer_worker: Mutex<Option<JoinHandle<()>>>,
}

#[derive(Default)]
struct RuntimeWakeState {
    pending: AtomicBool,
    callback: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

#[derive(Debug, Clone, Copy)]
struct TimerEntry {
    handle: u32,
    next_fire_at: Instant,
    interval: Option<Duration>,
}

#[derive(Debug, Clone, Copy)]
enum TimerCommand {
    Schedule(TimerEntry),
    Cancel(u32),
    Shutdown,
}

impl Default for HostRuntimeBridgeInner {
    fn default() -> Self {
        Self {
            is_shutdown: AtomicBool::new(false),
            wake: RuntimeWakeState::default(),
            next_timer_handle: AtomicU32::new(1),
            pending_due_timers: Mutex::new(Vec::new()),
            pending_due_timer_set: Mutex::new(HashSet::new()),
            active_timers: Mutex::new(HashSet::new()),
            metrics: RuntimeDebugCounterState::default(),
            timer_command_tx: Mutex::new(None),
            timer_worker: Mutex::new(None),
        }
    }
}

impl std::fmt::Debug for HostRuntimeBridgeInner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HostRuntimeBridgeInner")
            .field("next_timer_handle", &self.next_timer_handle)
            .finish_non_exhaustive()
    }
}

impl HostRuntimeBridge {
    fn new() -> Self {
        let (timer_command_tx, timer_command_rx) = mpsc::channel();
        let inner = Arc::new(HostRuntimeBridgeInner {
            timer_command_tx: Mutex::new(Some(timer_command_tx)),
            ..Default::default()
        });

        let worker_inner = Arc::clone(&inner);
        let worker = std::thread::Builder::new()
            .name("clay-jsx-runtime-timers".to_owned())
            .spawn(move || run_timer_worker(worker_inner, timer_command_rx))
            .expect("timer worker should spawn");
        *inner
            .timer_worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(worker);

        Self { inner }
    }

    fn schedule_timer(&self, delay_ms: u64, interval_ms: Option<u64>) -> u32 {
        // Repeating timers must always advance time. Clamp to 1ms so an
        // interval of 0 cannot trap the timer worker in a non-progressing loop.
        let interval_ms = interval_ms.map(|interval_ms| interval_ms.max(1));
        let handle = self.inner.next_timer_handle.fetch_add(1, Ordering::Relaxed);
        self.inner
            .metrics
            .timer_schedule_count
            .fetch_add(1, Ordering::Relaxed);
        self.inner.register_active_timer(handle);
        if delay_ms == 0 && interval_ms.is_none() {
            self.inner.note_timer_fire(false);
            self.inner.unregister_active_timer(handle);
            let _ = self.inner.enqueue_due_timers([handle]);
            self.inner.wake.trigger(&self.inner.metrics);
            return handle;
        }
        let entry = TimerEntry {
            handle,
            next_fire_at: Instant::now() + Duration::from_millis(delay_ms),
            interval: interval_ms.map(Duration::from_millis),
        };
        if let Some(timer_command_tx) = self
            .inner
            .timer_command_tx
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .cloned()
        {
            let _ = timer_command_tx.send(TimerCommand::Schedule(entry));
        }
        handle
    }

    fn cancel_timer(&self, handle: u32) {
        self.inner
            .metrics
            .timer_cancel_count
            .fetch_add(1, Ordering::Relaxed);
        self.inner.unregister_active_timer(handle);
        self.inner.remove_due_timer(handle);
        if let Some(timer_command_tx) = self
            .inner
            .timer_command_tx
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .cloned()
        {
            let _ = timer_command_tx.send(TimerCommand::Cancel(handle));
        }
    }

    fn take_due_timers(&self) -> Vec<u32> {
        self.inner.take_due_timers()
    }

    fn request_wake(&self) {
        self.inner.wake.trigger(&self.inner.metrics);
    }

    fn set_wake_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        *self
            .inner
            .wake
            .callback
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Arc::new(callback));
    }

    fn clear_wake_callback(&self) {
        *self
            .inner
            .wake
            .callback
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }

    fn take_pending_wake(&self) -> bool {
        self.inner.wake.take_pending()
    }

    fn shutdown(&self) {
        self.shutdown_with_metrics(true);
    }

    fn abort(&self) {
        self.shutdown_with_metrics(false);
    }

    fn shutdown_with_metrics(&self, count_shutdown_metric: bool) {
        if self.inner.is_shutdown.swap(true, Ordering::SeqCst) {
            return;
        }
        if count_shutdown_metric {
            self.inner
                .metrics
                .shutdown_count
                .fetch_add(1, Ordering::Relaxed);
        }
        let timer_command_tx = self
            .inner
            .timer_command_tx
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        if let Some(timer_command_tx) = timer_command_tx {
            let _ = timer_command_tx.send(TimerCommand::Shutdown);
        }
        if let Some(worker) = self
            .inner
            .timer_worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            let _ = worker.join();
        }
        self.clear_wake_callback();
        let _ = self.take_pending_wake();
        self.inner.clear_due_timers();
        self.inner.clear_active_timers();
    }

    fn note_callback_drain_cycle(&self) {
        self.inner
            .metrics
            .host_callback_drain_cycles
            .fetch_add(1, Ordering::Relaxed);
    }

    fn note_callback_drain_noop(&self) {
        self.inner
            .metrics
            .host_callback_drain_noop_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn note_callbacks_invoked(&self, count: u64) {
        self.inner
            .metrics
            .host_callbacks_invoked
            .fetch_add(count, Ordering::Relaxed);
    }

    fn debug_metrics(&self) -> RuntimeDebugMetrics {
        self.inner.metrics.snapshot(
            self.inner.wake.pending.load(Ordering::SeqCst),
            self.inner.active_timer_count(),
        )
    }
}

impl RuntimeWakeState {
    fn trigger(&self, metrics: &RuntimeDebugCounterState) {
        metrics.host_wake_count.fetch_add(1, Ordering::Relaxed);
        self.pending.store(true, Ordering::SeqCst);
        let callback = self
            .callback
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(callback) = callback {
            metrics
                .host_wake_callback_count
                .fetch_add(1, Ordering::Relaxed);
            callback();
        }
    }

    fn take_pending(&self) -> bool {
        self.pending.swap(false, Ordering::SeqCst)
    }
}

fn run_timer_worker(
    inner: Arc<HostRuntimeBridgeInner>,
    timer_command_rx: mpsc::Receiver<TimerCommand>,
) {
    let mut timers = HashMap::<u32, TimerEntry>::new();
    loop {
        let now = Instant::now();
        let next_deadline = timers.values().map(|timer| timer.next_fire_at).min();
        let command = match next_deadline {
            Some(next_deadline) if next_deadline > now => {
                match timer_command_rx.recv_timeout(next_deadline.saturating_duration_since(now)) {
                    Ok(command) => Some(command),
                    Err(RecvTimeoutError::Timeout) => None,
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
            Some(_) => None,
            None => match timer_command_rx.recv() {
                Ok(command) => Some(command),
                Err(_) => break,
            },
        };

        match command {
            Some(TimerCommand::Schedule(mut entry)) => {
                if let Some(interval) = entry.interval {
                    // Defensively clamp malformed repeating intervals that may
                    // bypass higher-level scheduling paths.
                    entry.interval = Some(interval.max(Duration::from_millis(1)));
                }
                timers.insert(entry.handle, entry);
                continue;
            }
            Some(TimerCommand::Cancel(handle)) => {
                timers.remove(&handle);
                continue;
            }
            Some(TimerCommand::Shutdown) => break,
            None => {}
        }

        let now = Instant::now();
        let mut due_handles = Vec::new();
        let mut completed_one_shots = Vec::new();
        for timer in timers.values_mut() {
            if timer.next_fire_at > now {
                continue;
            }
            due_handles.push(timer.handle);
            if let Some(interval) = timer.interval {
                let interval = interval.max(Duration::from_millis(1));
                // Coalesce missed repeat ticks into a single callback so long
                // stalls don't replay an unbounded burst in one drain.
                timer.next_fire_at = now + interval;
                timer.interval = Some(interval);
                inner.note_timer_fire(true);
            } else {
                inner.note_timer_fire(false);
                inner.unregister_active_timer(timer.handle);
                completed_one_shots.push(timer.handle);
            }
        }
        for handle in completed_one_shots {
            timers.remove(&handle);
        }
        if due_handles.is_empty() {
            continue;
        }

        if inner.enqueue_due_timers(due_handles) > 0 {
            inner.wake.trigger(&inner.metrics);
        }
    }
}

#[derive(Debug, Default)]
struct RuntimeDebugCounterState {
    host_wake_count: AtomicU64,
    host_wake_callback_count: AtomicU64,
    host_callback_drain_cycles: AtomicU64,
    host_callbacks_invoked: AtomicU64,
    host_callback_drain_noop_count: AtomicU64,
    timer_schedule_count: AtomicU64,
    timer_cancel_count: AtomicU64,
    timer_fire_count: AtomicU64,
    timer_repeat_fire_count: AtomicU64,
    active_timer_high_water: AtomicU64,
    shutdown_count: AtomicU64,
}

impl RuntimeDebugCounterState {
    fn snapshot(&self, pending_host_wake: bool, active_timer_count: u64) -> RuntimeDebugMetrics {
        RuntimeDebugMetrics {
            host_wake_count: self.host_wake_count.load(Ordering::Relaxed),
            host_wake_callback_count: self.host_wake_callback_count.load(Ordering::Relaxed),
            host_callback_drain_cycles: self.host_callback_drain_cycles.load(Ordering::Relaxed),
            host_callbacks_invoked: self.host_callbacks_invoked.load(Ordering::Relaxed),
            host_callback_drain_noop_count: self
                .host_callback_drain_noop_count
                .load(Ordering::Relaxed),
            timer_schedule_count: self.timer_schedule_count.load(Ordering::Relaxed),
            timer_cancel_count: self.timer_cancel_count.load(Ordering::Relaxed),
            timer_fire_count: self.timer_fire_count.load(Ordering::Relaxed),
            timer_repeat_fire_count: self.timer_repeat_fire_count.load(Ordering::Relaxed),
            active_timer_count,
            active_timer_high_water: self.active_timer_high_water.load(Ordering::Relaxed),
            shutdown_count: self.shutdown_count.load(Ordering::Relaxed),
            pending_host_wake,
            pending_mutation_batch_count: 0,
            pending_mutation_batch_bytes: 0,
            pending_mutation_batch_high_water: 0,
            pending_mutation_batch_bytes_high_water: 0,
            pending_mutation_batch_overflow_count: 0,
            pending_mutation_batch_limit: MAX_PENDING_MUTATION_BATCHES as u64,
            pending_mutation_batch_byte_limit: MAX_PENDING_MUTATION_BATCH_BYTES as u64,
        }
    }
}

impl HostRuntimeBridgeInner {
    fn enqueue_due_timers<I>(&self, due_handles: I) -> usize
    where
        I: IntoIterator<Item = u32>,
    {
        let mut pending_due_timers = self
            .pending_due_timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut pending_due_timer_set = self
            .pending_due_timer_set
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let mut inserted = 0;
        for handle in due_handles {
            if pending_due_timer_set.insert(handle) {
                pending_due_timers.push(handle);
                inserted += 1;
            }
        }
        inserted
    }

    fn take_due_timers(&self) -> Vec<u32> {
        let mut pending_due_timers = self
            .pending_due_timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut pending_due_timer_set = self
            .pending_due_timer_set
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let due_timers = std::mem::take(&mut *pending_due_timers);
        pending_due_timer_set.clear();
        due_timers
    }

    fn remove_due_timer(&self, handle: u32) {
        let mut pending_due_timers = self
            .pending_due_timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut pending_due_timer_set = self
            .pending_due_timer_set
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        pending_due_timers.retain(|pending_handle| *pending_handle != handle);
        pending_due_timer_set.remove(&handle);
    }

    fn clear_due_timers(&self) {
        let mut pending_due_timers = self
            .pending_due_timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut pending_due_timer_set = self
            .pending_due_timer_set
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        pending_due_timers.clear();
        pending_due_timer_set.clear();
    }

    fn active_timer_count(&self) -> u64 {
        self.active_timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len() as u64
    }

    fn register_active_timer(&self, handle: u32) {
        let active_count = {
            let mut active_timers = self
                .active_timers
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            active_timers.insert(handle);
            active_timers.len() as u64
        };
        self.metrics
            .active_timer_high_water
            .fetch_max(active_count, Ordering::Relaxed);
    }

    fn unregister_active_timer(&self, handle: u32) {
        let _ = self
            .active_timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&handle);
    }

    fn clear_active_timers(&self) {
        self.active_timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    fn note_timer_fire(&self, repeat: bool) {
        self.metrics
            .timer_fire_count
            .fetch_add(1, Ordering::Relaxed);
        if repeat {
            self.metrics
                .timer_repeat_fire_count
                .fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[derive(Debug, Default)]
struct RuntimeState {
    commit_batches: Vec<RuntimeCommitBatch>,
    pending_mutation_batch_bytes: usize,
    pending_mutation_batch_high_water: usize,
    pending_mutation_batch_bytes_high_water: usize,
    pending_mutation_batch_overflow_count: u64,
    reconciler_errors: Vec<RuntimeReconcilerError>,
    next_commit_batch_id: u32,
    logs: RuntimeLogBuffer,
    host_runtime: HostRuntimeBridge,
    host_debug_counters: RuntimeHostDebugCounters,
}

impl RuntimeState {
    fn can_enqueue_commit_batch(&self, payload_bytes: usize) -> bool {
        self.commit_batches.len() < MAX_PENDING_MUTATION_BATCHES
            && self
                .pending_mutation_batch_bytes
                .saturating_add(payload_bytes)
                <= MAX_PENDING_MUTATION_BATCH_BYTES
    }

    fn note_commit_batch_enqueued(&mut self, payload_bytes: usize) {
        self.pending_mutation_batch_bytes = self
            .pending_mutation_batch_bytes
            .saturating_add(payload_bytes);
        self.pending_mutation_batch_high_water = self
            .pending_mutation_batch_high_water
            .max(self.commit_batches.len());
        self.pending_mutation_batch_bytes_high_water = self
            .pending_mutation_batch_bytes_high_water
            .max(self.pending_mutation_batch_bytes);
    }

    fn note_commit_batch_overflow(&mut self, payload_bytes: usize) {
        self.pending_mutation_batch_overflow_count =
            self.pending_mutation_batch_overflow_count.saturating_add(1);
        push_log(
            &mut self.logs,
            format!(
                "error: runtime mutation queue overflow (pending batches: {}, pending bytes: {}, limit: {} batches / {} bytes, rejected payload bytes: {payload_bytes})",
                self.commit_batches.len(),
                self.pending_mutation_batch_bytes,
                MAX_PENDING_MUTATION_BATCHES,
                MAX_PENDING_MUTATION_BATCH_BYTES,
            ),
        );
    }

    fn take_pending_commit_update(&mut self) -> Vec<RuntimeCommitBatch> {
        self.pending_mutation_batch_bytes = 0;
        std::mem::take(&mut self.commit_batches)
    }

    fn apply_mutation_queue_metrics(&self, metrics: &mut RuntimeDebugMetrics) {
        metrics.pending_mutation_batch_count = self.commit_batches.len() as u64;
        metrics.pending_mutation_batch_bytes = self.pending_mutation_batch_bytes as u64;
        metrics.pending_mutation_batch_high_water = self.pending_mutation_batch_high_water as u64;
        metrics.pending_mutation_batch_bytes_high_water =
            self.pending_mutation_batch_bytes_high_water as u64;
        metrics.pending_mutation_batch_overflow_count = self.pending_mutation_batch_overflow_count;
        metrics.pending_mutation_batch_limit = MAX_PENDING_MUTATION_BATCHES as u64;
        metrics.pending_mutation_batch_byte_limit = MAX_PENDING_MUTATION_BATCH_BYTES as u64;
    }

    fn enqueue_commit_batch(
        &mut self,
        payload: RuntimeCommitBatchPayload,
    ) -> Result<u32, JsErrorBox> {
        let payload_bytes = payload.payload_bytes().map_err(|error| {
            JsErrorBox::generic(format!(
                "failed to measure commit batch payload bytes: {error:#}"
            ))
        })?;
        if !self.can_enqueue_commit_batch(payload_bytes) {
            self.note_commit_batch_overflow(payload_bytes);
            return Err(JsErrorBox::generic(format!(
                "runtime mutation queue overflow: reached {} batches / {} bytes pending",
                MAX_PENDING_MUTATION_BATCHES, MAX_PENDING_MUTATION_BATCH_BYTES
            )));
        }

        let commit_batch_id = self
            .next_commit_batch_id
            .checked_add(1)
            .ok_or_else(|| JsErrorBox::generic("commit batch id overflow"))?;
        self.next_commit_batch_id = commit_batch_id;
        self.commit_batches.push(RuntimeCommitBatch {
            commit_batch_id,
            payload,
        });
        self.note_commit_batch_enqueued(payload_bytes);
        Ok(commit_batch_id)
    }
}

#[op2(fast)]
fn op_commit_mutations(
    state: &mut OpState,
    #[string] mutations_json: String,
) -> Result<u32, JsErrorBox> {
    state
        .borrow_mut::<RuntimeState>()
        .enqueue_commit_batch(RuntimeCommitBatchPayload::Json(mutations_json))
}

#[op2]
fn op_commit_mutations_typed(
    state: &mut OpState,
    #[serde] mutations: serde_json::Value,
) -> Result<u32, JsErrorBox> {
    state
        .borrow_mut::<RuntimeState>()
        .enqueue_commit_batch(RuntimeCommitBatchPayload::Typed(mutations))
}

#[op2(fast)]
fn op_host_log(
    state: &mut OpState,
    #[string] level: String,
    #[string] message: String,
) -> Result<(), JsErrorBox> {
    let runtime_state = state.borrow_mut::<RuntimeState>();
    push_log(&mut runtime_state.logs, format!("{level}: {message}"));
    Ok(())
}

#[op2(fast)]
fn op_host_report_reconciler_error(
    state: &mut OpState,
    #[string] category: String,
    #[string] message: String,
    #[string] component_stack: String,
    #[string] error_boundary: String,
) -> Result<(), JsErrorBox> {
    let Some(category) = RuntimeReconcilerErrorCategory::parse(category.as_str()) else {
        return Err(JsErrorBox::generic(format!(
            "unknown reconciler error category {}",
            category
        )));
    };

    let runtime_state = state.borrow_mut::<RuntimeState>();
    runtime_state
        .reconciler_errors
        .push(RuntimeReconcilerError {
            category,
            message,
            component_stack,
            error_boundary: (!error_boundary.trim().is_empty()).then_some(error_boundary),
        });

    let last_message = runtime_state
        .reconciler_errors
        .last()
        .map(|error| error.message.clone())
        .unwrap_or_default();
    push_log(
        &mut runtime_state.logs,
        format!("error: reconciler:{}: {}", category.as_str(), last_message),
    );
    Ok(())
}

#[op2(fast)]
fn op_host_note_render(state: &mut OpState) -> Result<(), JsErrorBox> {
    state
        .borrow_mut::<RuntimeState>()
        .host_debug_counters
        .render_call_count += 1;
    Ok(())
}

#[op2(fast)]
fn op_host_note_unmount(state: &mut OpState) -> Result<(), JsErrorBox> {
    state
        .borrow_mut::<RuntimeState>()
        .host_debug_counters
        .unmount_count += 1;
    Ok(())
}

#[op2(fast)]
fn op_host_schedule_timer(
    state: &mut OpState,
    delay_ms: i32,
    interval_ms: i32,
) -> Result<u32, JsErrorBox> {
    let host_runtime = state.borrow::<RuntimeState>().host_runtime.clone();
    let delay_ms = delay_ms.max(0) as u64;
    let interval_ms = (interval_ms >= 0).then_some(interval_ms as u64);
    Ok(host_runtime.schedule_timer(delay_ms, interval_ms))
}

#[op2(fast)]
fn op_host_cancel_timer(state: &mut OpState, handle: u32) -> Result<(), JsErrorBox> {
    state
        .borrow::<RuntimeState>()
        .host_runtime
        .cancel_timer(handle);
    Ok(())
}

#[op2(fast)]
fn op_host_request_wake(state: &mut OpState) -> Result<(), JsErrorBox> {
    state.borrow::<RuntimeState>().host_runtime.request_wake();
    Ok(())
}

#[op2]
#[serde]
fn op_host_take_due_timers(state: &mut OpState) -> Result<Vec<u32>, JsErrorBox> {
    Ok(state
        .borrow::<RuntimeState>()
        .host_runtime
        .take_due_timers())
}

#[op2(fast)]
fn op_host_now_ms() -> Result<f64, JsErrorBox> {
    Ok(HOST_TIME_ORIGIN.elapsed().as_secs_f64() * 1_000.0)
}

extension!(
    clay_jsx_host,
    ops = [
        op_commit_mutations,
        op_commit_mutations_typed,
        op_host_log,
        op_host_report_reconciler_error,
        op_host_note_render,
        op_host_note_unmount,
        op_host_schedule_timer,
        op_host_cancel_timer,
        op_host_request_wake,
        op_host_take_due_timers,
        op_host_now_ms
    ],
    docs = "Ops used by clay JSX host runtimes."
);

#[derive(Debug)]
struct JsxModuleLoader {
    import_aliases: HashMap<String, String>,
    virtual_sources: HashMap<String, String>,
    jsx_import_source: String,
    source_maps: SourceMapStore,
    loaded_modules: Rc<RefCell<BTreeSet<PathBuf>>>,
}

impl JsxModuleLoader {
    fn new(options: JsxRuntimeOptions) -> anyhow::Result<Self> {
        if options.jsx_import_source.trim().is_empty() {
            return Err(anyhow!("JSX import source must not be empty"));
        }

        let mut import_aliases = HashMap::new();
        let mut virtual_sources = HashMap::new();
        for (index, module) in options.virtual_modules.into_iter().enumerate() {
            if module.import_specifier.trim().is_empty() {
                return Err(anyhow!("virtual module import specifier must not be empty"));
            }
            let virtual_specifier = format!("clay-jsx-runtime:///virtual/{index}");
            import_aliases.insert(module.import_specifier, virtual_specifier.clone());
            virtual_sources.insert(virtual_specifier, module.source);
        }

        Ok(Self {
            import_aliases,
            virtual_sources,
            jsx_import_source: options.jsx_import_source,
            source_maps: Rc::new(RefCell::new(HashMap::new())),
            loaded_modules: Rc::new(RefCell::new(BTreeSet::new())),
        })
    }

    fn loaded_module_paths(&self) -> Vec<PathBuf> {
        self.loaded_modules.borrow().iter().cloned().collect()
    }
}

impl ModuleLoader for JsxModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, deno_core::error::ModuleLoaderError> {
        if let Some(virtual_specifier) = self.import_aliases.get(specifier) {
            return ModuleSpecifier::parse(virtual_specifier).map_err(JsErrorBox::from_err);
        }

        resolve_import(specifier, referrer).map_err(JsErrorBox::from_err)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        ModuleLoadResponse::Sync(load_module(
            Rc::clone(&self.source_maps),
            Rc::clone(&self.loaded_modules),
            &self.virtual_sources,
            &self.jsx_import_source,
            module_specifier,
        ))
    }

    fn get_source_map(&self, specifier: &str) -> Option<Cow<'_, [u8]>> {
        self.source_maps
            .borrow()
            .get(specifier)
            .map(|source_map| source_map.clone().into())
    }
}

pub struct RuntimeSession {
    entry_path: Option<PathBuf>,
    host_runtime: HostRuntimeBridge,
    module_loader: Rc<JsxModuleLoader>,
    js_runtime: JsRuntime,
    tokio_runtime: tokio::runtime::Runtime,
}

impl std::fmt::Debug for RuntimeSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeSession")
            .field("entry_path", &self.entry_path)
            .finish_non_exhaustive()
    }
}

impl RuntimeSession {
    pub fn new(options: JsxRuntimeOptions) -> anyhow::Result<Self> {
        let module_loader = Rc::new(JsxModuleLoader::new(options)?);
        let host_runtime = HostRuntimeBridge::new();
        let mut js_runtime = JsRuntime::new(DenoRuntimeOptions {
            module_loader: Some(module_loader.clone()),
            extensions: vec![clay_jsx_host::init()],
            ..Default::default()
        });
        js_runtime.op_state().borrow_mut().put(RuntimeState {
            host_runtime: host_runtime.clone(),
            ..Default::default()
        });
        js_runtime
            .execute_script("[clay:host-runtime]", HOST_RUNTIME_SOURCE)
            .map_err(|error| anyhow!("failed to install clay host runtime globals: {error}"))?;

        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to build Tokio runtime for deno_core")?;

        Ok(Self {
            entry_path: None,
            host_runtime,
            module_loader,
            js_runtime,
            tokio_runtime,
        })
    }

    pub fn load(
        entry_path: &Path,
        options: JsxRuntimeOptions,
    ) -> anyhow::Result<(Self, RuntimeUpdate)> {
        let mut session = Self::new(options)?;
        session.load_main_module(entry_path)?;
        let update = session.take_update();
        Ok((session, update))
    }

    pub fn load_main_module(&mut self, entry_path: &Path) -> anyhow::Result<()> {
        let entry_path = normalize_file_path(entry_path)?;
        let main_module = ModuleSpecifier::from_file_path(&entry_path).map_err(|_| {
            anyhow!(
                "entry path is not a valid file URL: {}",
                entry_path.display()
            )
        })?;

        self.evaluate_main_module(&main_module)?;
        self.entry_path = Some(entry_path);
        Ok(())
    }

    pub fn execute_script(
        &mut self,
        name: &'static str,
        source: impl Into<String>,
    ) -> anyhow::Result<()> {
        self.js_runtime
            .execute_script(name, source.into())
            .map_err(|error| anyhow!("failed to execute script {name}: {error}"))?;
        self.run_event_loop()?;
        Ok(())
    }

    pub fn execute_script_as<T: DeserializeOwned>(
        &mut self,
        name: &'static str,
        source: impl Into<String>,
    ) -> anyhow::Result<T> {
        let value = self
            .js_runtime
            .execute_script(name, source.into())
            .map_err(|error| anyhow!("failed to execute script {name}: {error}"))?;
        self.run_event_loop()?;

        deno_core::scope!(scope, &mut self.js_runtime);
        let local = v8::Local::new(scope, value);
        serde_v8::from_v8(scope, local)
            .map_err(|error| anyhow!("failed to deserialize script result {name}: {error}"))
    }

    pub fn execute_json_expression_as<T: DeserializeOwned>(
        &mut self,
        name: &'static str,
        expression: &str,
    ) -> anyhow::Result<T> {
        let source = format!(
            "(() => {{ const __clayResult = ({expression}); return JSON.stringify(__clayResult); }})()"
        );
        let value = self
            .js_runtime
            .execute_script(name, source)
            .map_err(|error| anyhow!("failed to execute script {name}: {error}"))?;
        self.run_event_loop()?;

        deno_core::scope!(scope, &mut self.js_runtime);
        let local = value.open(scope);
        let json = local
            .to_string(scope)
            .ok_or_else(|| anyhow!("script result {name} could not be stringified"))?
            .to_rust_string_lossy(scope);
        serde_json::from_str(&json)
            .map_err(|error| anyhow!("failed to deserialize JSON script result {name}: {error}"))
    }

    pub fn take_update(&mut self) -> RuntimeUpdate {
        let op_state = self.js_runtime.op_state();
        let mut op_state = op_state.borrow_mut();
        let state = op_state.borrow_mut::<RuntimeState>();
        let commit_batches = state.take_pending_commit_update();
        RuntimeUpdate {
            commit_batches,
            reconciler_errors: std::mem::take(&mut state.reconciler_errors),
            logs: std::mem::take(&mut state.logs),
            host_debug_counters: state.host_debug_counters,
        }
    }

    pub fn entry_path(&self) -> Option<&Path> {
        self.entry_path.as_deref()
    }

    pub fn loaded_module_paths(&self) -> Vec<PathBuf> {
        self.module_loader.loaded_module_paths()
    }

    pub fn set_host_wake_callback<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.host_runtime.set_wake_callback(callback);
    }

    pub fn clear_host_wake_callback(&mut self) {
        self.host_runtime.clear_wake_callback();
    }

    pub fn take_pending_host_wake(&self) -> bool {
        self.host_runtime.take_pending_wake()
    }

    pub fn drain_host_callbacks(&mut self) -> anyhow::Result<RuntimeHostCallbackDrainResult> {
        if !self.host_runtime.take_pending_wake() {
            self.host_runtime.note_callback_drain_noop();
            return Ok(RuntimeHostCallbackDrainResult {
                had_pending_wake: false,
                callbacks_invoked: 0,
                pending_host_wake_after_drain: false,
            });
        }

        self.host_runtime.note_callback_drain_cycle();
        let invoked_callbacks = self.execute_script_as::<u64>(
            "[clay:drain-host-callbacks]",
            "globalThis.__clayDrainHostCallbacks == null ? 0 : globalThis.__clayDrainHostCallbacks()",
        )?;
        self.host_runtime.note_callbacks_invoked(invoked_callbacks);
        Ok(RuntimeHostCallbackDrainResult {
            had_pending_wake: true,
            callbacks_invoked: invoked_callbacks,
            pending_host_wake_after_drain: self.debug_metrics().pending_host_wake,
        })
    }

    pub fn shutdown_host_runtime(&mut self) {
        self.host_runtime.shutdown();
    }

    pub fn abort_host_runtime(&mut self) {
        self.host_runtime.abort();
    }

    pub fn debug_metrics(&self) -> RuntimeDebugMetrics {
        let mut metrics = self.host_runtime.debug_metrics();
        let op_state = self.js_runtime.op_state();
        let op_state = op_state.borrow();
        let state = op_state.borrow::<RuntimeState>();
        state.apply_mutation_queue_metrics(&mut metrics);
        metrics
    }

    fn evaluate_main_module(&mut self, main_module: &ModuleSpecifier) -> anyhow::Result<()> {
        self.tokio_runtime.block_on(async {
            let module_id = self.js_runtime.load_main_es_module(main_module).await?;
            let result = self.js_runtime.mod_evaluate(module_id);
            self.js_runtime.run_event_loop(Default::default()).await?;
            result.await
        })?;
        Ok(())
    }

    fn run_event_loop(&mut self) -> anyhow::Result<()> {
        self.tokio_runtime
            .block_on(self.js_runtime.run_event_loop(Default::default()))?;
        Ok(())
    }
}

impl Drop for RuntimeSession {
    fn drop(&mut self) {
        self.shutdown_host_runtime();
    }
}

fn load_module(
    source_maps: SourceMapStore,
    loaded_modules: Rc<RefCell<BTreeSet<PathBuf>>>,
    virtual_sources: &HashMap<String, String>,
    jsx_import_source: &str,
    module_specifier: &ModuleSpecifier,
) -> Result<ModuleSource, deno_core::error::ModuleLoaderError> {
    if let Some(source) = virtual_sources.get(module_specifier.as_str()) {
        return Ok(module_source(module_specifier, source.clone()));
    }

    let path = module_specifier.to_file_path().map_err(|_| {
        JsErrorBox::generic("Only file:// and configured virtual modules are supported.")
    })?;
    let path =
        normalize_file_path(&path).map_err(|error| JsErrorBox::generic(error.to_string()))?;
    loaded_modules.borrow_mut().insert(path.clone());
    let media_type = MediaType::from_path(&path);
    let (module_type, should_transpile) = match media_type {
        MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => (ModuleType::JavaScript, false),
        MediaType::Jsx
        | MediaType::TypeScript
        | MediaType::Mts
        | MediaType::Cts
        | MediaType::Dts
        | MediaType::Dmts
        | MediaType::Dcts
        | MediaType::Tsx => (ModuleType::JavaScript, true),
        MediaType::Json => (ModuleType::Json, false),
        _ => {
            return Err(JsErrorBox::generic(format!(
                "Unsupported module extension for {}",
                path.display()
            )));
        }
    };

    let code = std::fs::read_to_string(&path).map_err(JsErrorBox::from_err)?;
    let code = if should_transpile {
        let transpiled = transpile_module(module_specifier, media_type, code, jsx_import_source)
            .map_err(|error| JsErrorBox::generic(error.to_string()))?;
        if let Some(source_map) = transpiled.source_map {
            source_maps
                .borrow_mut()
                .insert(module_specifier.to_string(), source_map);
        }
        transpiled.code
    } else {
        code
    };

    Ok(ModuleSource::new(
        module_type,
        ModuleSourceCode::String(code.into()),
        module_specifier,
        None,
    ))
}

struct TranspiledModule {
    code: String,
    source_map: Option<Vec<u8>>,
}

fn transpile_module(
    module_specifier: &ModuleSpecifier,
    media_type: MediaType,
    code: String,
    jsx_import_source: &str,
) -> anyhow::Result<TranspiledModule> {
    let parsed = deno_ast::parse_module(ParseParams {
        specifier: module_specifier.clone(),
        text: code.into(),
        media_type,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })?;

    let emitted = parsed
        .transpile(
            &TranspileOptions {
                decorators: DecoratorsTranspileOption::Ecma,
                imports_not_used_as_values: ImportsNotUsedAsValues::Remove,
                jsx: Some(JsxRuntime::Automatic(JsxAutomaticOptions {
                    import_source: Some(jsx_import_source.to_owned()),
                    development: false,
                })),
                ..Default::default()
            },
            &TranspileModuleOptions::default(),
            &EmitOptions {
                source_map: SourceMapOption::Separate,
                inline_sources: true,
                remove_comments: true,
                ..Default::default()
            },
        )?
        .into_source();

    Ok(TranspiledModule {
        code: emitted.text,
        source_map: emitted.source_map.map(|source_map| source_map.into_bytes()),
    })
}

fn module_source(module_specifier: &ModuleSpecifier, source: String) -> ModuleSource {
    ModuleSource::new(
        ModuleType::JavaScript,
        ModuleSourceCode::String(source.into()),
        module_specifier,
        None,
    )
}

fn absolute_path(path: &Path) -> anyhow::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()
            .context("failed to resolve current directory")?
            .join(path))
    }
}

fn normalize_file_path(path: &Path) -> anyhow::Result<PathBuf> {
    let path = absolute_path(path)?;
    Ok(std::fs::canonicalize(&path).unwrap_or(path))
}

#[cfg(test)]
mod tests {
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        hint::black_box,
        sync::mpsc,
        sync::{
            atomic::{AtomicBool, AtomicU64, Ordering},
            Arc,
        },
        time::{Duration, Instant},
    };

    use deno_ast::MediaType;
    use deno_core::ModuleSpecifier;
    use tempfile::tempdir;

    use super::{
        HostRuntimeBridge, HostRuntimeBridgeInner, JsxRuntimeOptions, RuntimeCommitTransportKind,
        RuntimeReconcilerErrorCategory, RuntimeRecoveryCategory, RuntimeRecoveryDisposition,
        RuntimeRecoveryState, RuntimeSession, TimerCommand, TimerEntry, HOST_CALLBACK_DRAIN_LIMIT,
        MAX_PENDING_MUTATION_BATCHES, MAX_PENDING_MUTATION_BATCH_BYTES,
    };

    struct TestCountingAllocator;

    static TRACK_TEST_ALLOCATIONS: AtomicBool = AtomicBool::new(false);
    static TEST_ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
    static TEST_ALLOCATION_BYTES: AtomicU64 = AtomicU64::new(0);

    #[global_allocator]
    static TEST_ALLOCATOR: TestCountingAllocator = TestCountingAllocator;

    unsafe impl GlobalAlloc for TestCountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let pointer = unsafe { System.alloc(layout) };
            if TRACK_TEST_ALLOCATIONS.load(Ordering::Relaxed) && !pointer.is_null() {
                TEST_ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
                TEST_ALLOCATION_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
            }
            pointer
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            let pointer = unsafe { System.alloc_zeroed(layout) };
            if TRACK_TEST_ALLOCATIONS.load(Ordering::Relaxed) && !pointer.is_null() {
                TEST_ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
                TEST_ALLOCATION_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
            }
            pointer
        }

        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            unsafe { System.dealloc(pointer, layout) }
        }

        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
            if TRACK_TEST_ALLOCATIONS.load(Ordering::Relaxed) && !new_pointer.is_null() {
                TEST_ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
                TEST_ALLOCATION_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
            }
            new_pointer
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct AllocationSnapshot {
        count: u64,
        bytes: u64,
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct DueTimerTransferBenchmark {
        elapsed: Duration,
        allocations: u64,
        allocated_bytes: u64,
        checksum: u64,
    }

    #[derive(Debug, Clone, Copy)]
    enum DueTimerTransferMode {
        JsonRoundTripBaseline,
        Typed,
    }

    fn allocation_snapshot() -> AllocationSnapshot {
        AllocationSnapshot {
            count: TEST_ALLOCATION_COUNT.load(Ordering::Relaxed),
            bytes: TEST_ALLOCATION_BYTES.load(Ordering::Relaxed),
        }
    }

    fn with_allocation_tracking<T>(operation: impl FnOnce() -> T) -> (T, AllocationSnapshot) {
        struct AllocationTrackingGuard;
        impl Drop for AllocationTrackingGuard {
            fn drop(&mut self) {
                TRACK_TEST_ALLOCATIONS.store(false, Ordering::SeqCst);
            }
        }

        let before = allocation_snapshot();
        TRACK_TEST_ALLOCATIONS.store(true, Ordering::SeqCst);
        let tracking_guard = AllocationTrackingGuard;
        let result = operation();
        drop(tracking_guard);
        let after = allocation_snapshot();

        (
            result,
            AllocationSnapshot {
                count: after.count.saturating_sub(before.count),
                bytes: after.bytes.saturating_sub(before.bytes),
            },
        )
    }

    fn benchmark_due_timer_transfer(
        mode: DueTimerTransferMode,
        iterations: usize,
        handles_per_batch: usize,
    ) -> DueTimerTransferBenchmark {
        let host_runtime = HostRuntimeBridge {
            inner: Arc::new(HostRuntimeBridgeInner::default()),
        };
        let handles: Vec<u32> = (1..=handles_per_batch as u32).collect();

        let ((checksum, elapsed), allocation_delta) = with_allocation_tracking(|| {
            let started = Instant::now();
            let mut checksum = 0u64;
            for _ in 0..iterations {
                let inserted = host_runtime
                    .inner
                    .enqueue_due_timers(handles.iter().copied());
                assert_eq!(
                    inserted, handles_per_batch,
                    "due timer benchmark should insert one full batch per cycle"
                );

                let due_handles = host_runtime.take_due_timers();
                let transferred = match mode {
                    DueTimerTransferMode::JsonRoundTripBaseline => {
                        let payload = serde_json::to_string(&due_handles)
                            .expect("json baseline should encode");
                        serde_json::from_str::<Vec<u32>>(&payload)
                            .expect("json baseline should decode")
                    }
                    DueTimerTransferMode::Typed => due_handles,
                };
                checksum = checksum.wrapping_add(transferred.len() as u64);
                checksum = checksum.wrapping_add(transferred.first().copied().unwrap_or(0) as u64);
                checksum = checksum.wrapping_add(transferred.last().copied().unwrap_or(0) as u64);
                black_box(transferred);
            }
            (checksum, started.elapsed())
        });

        DueTimerTransferBenchmark {
            elapsed,
            allocations: allocation_delta.count,
            allocated_bytes: allocation_delta.bytes,
            checksum,
        }
    }

    #[derive(Debug, serde::Deserialize)]
    struct RuntimeAction {
        ok: bool,
        code: Option<String>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct MutationQueueProbeResult {
        accepted: u64,
        overflow: String,
    }

    #[test]
    fn tsx_entrypoint_uses_configured_virtual_modules() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "clay";

render(<box answer={42} />);
"#,
        )
        .expect("tsx file should be written");

        let (_session, update) = RuntimeSession::load(&entry_path, test_options())
            .expect("tsx should transpile and render");

        assert_eq!(update.commit_batches.len(), 1);
        assert_eq!(
            update.commit_batches[0].transport_kind(),
            RuntimeCommitTransportKind::Json
        );
        assert_eq!(
            update.commit_batches[0]
                .decode_as::<serde_json::Value>()
                .expect("commit should decode"),
            serde_json::json!({ "type": "box", "props": { "answer": 42 } })
        );
        assert_eq!(update.commit_batch_ids(), vec![1]);
    }

    #[test]
    fn commit_batches_expose_monotonic_commit_batch_ids() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");

        session
            .execute_script(
                "[test:commit-batch-ids]",
                r#"
Deno.core.ops.op_commit_mutations(JSON.stringify({ kind: "first" }));
Deno.core.ops.op_commit_mutations(JSON.stringify({ kind: "second" }));
"#,
            )
            .expect("commits should enqueue");

        let update = session.take_update();
        assert_eq!(update.commit_batch_ids(), vec![1, 2]);
        assert_eq!(update.commit_batches.len(), 2);
    }

    #[test]
    fn typed_commit_batches_decode_through_the_transport_abstraction() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");

        session
            .execute_script(
                "[test:typed-commit-batch]",
                r#"
Deno.core.ops.op_commit_mutations_typed({
  kind: "typed",
  nested: { answer: 42 },
});
"#,
            )
            .expect("typed commit should enqueue");

        let update = session.take_update();
        assert_eq!(update.commit_batch_ids(), vec![1]);
        assert_eq!(update.commit_batches.len(), 1);
        assert_eq!(
            update.commit_batches[0].transport_kind(),
            RuntimeCommitTransportKind::Typed
        );
        assert_eq!(
            update.commit_batches[0]
                .decode_as::<serde_json::Value>()
                .expect("typed commit should decode"),
            serde_json::json!({
                "kind": "typed",
                "nested": { "answer": 42 },
            })
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                &update.commit_batches[0]
                    .payload_json()
                    .expect("typed payload should remain serializable")
            )
            .expect("typed payload json should parse"),
            serde_json::json!({
                "kind": "typed",
                "nested": { "answer": 42 },
            })
        );
    }

    #[test]
    fn mutation_queue_backpressure_caps_pending_batches_and_reports_overflow() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        let attempts = MAX_PENDING_MUTATION_BATCHES + 64;

        let probe = session
            .execute_script_as::<MutationQueueProbeResult>(
                "[test:mutation-queue-backpressure]",
                format!(
                    r#"
(() => {{
  let accepted = 0;
  let overflow = "";
  for (let index = 0; index < {attempts}; index += 1) {{
    try {{
      Deno.core.ops.op_commit_mutations(JSON.stringify({{ kind: "queue-probe", index }}));
      accepted += 1;
    }} catch (error) {{
      overflow = error instanceof Error ? error.message : String(error);
      break;
    }}
  }}
  return {{ accepted, overflow }};
}})()
"#,
                    attempts = attempts,
                ),
            )
            .expect("queue backpressure probe should complete");

        assert_eq!(probe.accepted as usize, MAX_PENDING_MUTATION_BATCHES);
        assert!(
            probe.overflow.contains("runtime mutation queue overflow"),
            "expected overflow message from backpressure guard, got: {}",
            probe.overflow
        );

        let metrics = session.debug_metrics();
        assert_eq!(
            metrics.pending_mutation_batch_count,
            MAX_PENDING_MUTATION_BATCHES as u64
        );
        assert_eq!(
            metrics.pending_mutation_batch_high_water,
            MAX_PENDING_MUTATION_BATCHES as u64
        );
        assert!(metrics.pending_mutation_batch_bytes > 0);
        assert_eq!(
            metrics.pending_mutation_batch_bytes,
            metrics.pending_mutation_batch_bytes_high_water
        );
        assert_eq!(metrics.pending_mutation_batch_overflow_count, 1);
        assert_eq!(
            metrics.pending_mutation_batch_limit,
            MAX_PENDING_MUTATION_BATCHES as u64
        );

        let update = session.take_update();
        let commit_batch_ids = update.commit_batch_ids();
        assert_eq!(commit_batch_ids.len(), MAX_PENDING_MUTATION_BATCHES);
        assert_eq!(update.commit_batches.len(), MAX_PENDING_MUTATION_BATCHES);
        assert_eq!(commit_batch_ids.first().copied(), Some(1));
        assert_eq!(
            commit_batch_ids.last().copied(),
            Some(MAX_PENDING_MUTATION_BATCHES as u32)
        );
        assert!(
            update
                .logs
                .iter()
                .any(|entry| entry.contains("runtime mutation queue overflow")),
            "expected overflow logging in runtime update"
        );

        let metrics_after_drain = session.debug_metrics();
        assert_eq!(metrics_after_drain.pending_mutation_batch_count, 0);
        assert_eq!(metrics_after_drain.pending_mutation_batch_bytes, 0);
        assert_eq!(metrics_after_drain.pending_mutation_batch_overflow_count, 1);

        session
            .execute_script(
                "[test:mutation-queue-backpressure-after-drain]",
                "Deno.core.ops.op_commit_mutations(JSON.stringify({ kind: 'after-drain' }));",
            )
            .expect("queue should accept commits again after draining");
        let update = session.take_update();
        assert_eq!(
            update.commit_batch_ids(),
            vec![MAX_PENDING_MUTATION_BATCHES as u32 + 1]
        );
        assert_eq!(update.commit_batches.len(), 1);
    }

    #[test]
    fn mutation_queue_backpressure_enforces_pending_byte_budget() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");

        let probe = session
            .execute_script_as::<MutationQueueProbeResult>(
                "[test:mutation-queue-byte-budget]",
                format!(
                    r#"
(() => {{
  const payload = "x".repeat({payload_bytes});
  const batchJson = JSON.stringify({{ kind: "byte-budget-probe", payload }});
  let accepted = 0;
  let overflow = "";
  for (let index = 0; index < 8; index += 1) {{
    try {{
      Deno.core.ops.op_commit_mutations(batchJson);
      accepted += 1;
    }} catch (error) {{
      overflow = error instanceof Error ? error.message : String(error);
      break;
    }}
  }}
  return {{ accepted, overflow }};
}})()
"#,
                    payload_bytes = MAX_PENDING_MUTATION_BATCH_BYTES / 2,
                ),
            )
            .expect("byte-budget probe should complete");

        assert!(probe.accepted >= 1);
        assert!(
            probe.accepted < MAX_PENDING_MUTATION_BATCHES as u64,
            "byte budget should overflow before batch-count budget"
        );
        assert!(
            probe.overflow.contains("runtime mutation queue overflow"),
            "expected overflow message from byte-budget guard, got: {}",
            probe.overflow
        );

        let metrics = session.debug_metrics();
        assert_eq!(metrics.pending_mutation_batch_count, probe.accepted);
        assert!(
            metrics.pending_mutation_batch_bytes <= MAX_PENDING_MUTATION_BATCH_BYTES as u64,
            "pending byte budget should never exceed hard limit"
        );
        assert_eq!(metrics.pending_mutation_batch_overflow_count, 1);
        assert_eq!(
            metrics.pending_mutation_batch_byte_limit,
            MAX_PENDING_MUTATION_BATCH_BYTES as u64
        );

        let update = session.take_update();
        assert_eq!(update.commit_batches.len(), probe.accepted as usize);
        assert!(
            update
                .logs
                .iter()
                .any(|entry| entry.contains("runtime mutation queue overflow")),
            "expected overflow logging in runtime update"
        );
    }

    #[test]
    fn reconciler_error_categories_are_structured_and_runtime_actions_are_deterministic() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("runtime-setup.ts");
        std::fs::write(
            &entry_path,
            r#"
import React from "react";
import { createClayJsxRuntime } from "clay-internal:/jsx-runtime";

globalThis.__clayRuntimeForTest = createClayJsxRuntime({
  hostTags: new Map([["box", "box"]]),
  eventProps: new Map(),
  commitPatch: () => ({ ok: true, code: null }),
});
globalThis.__clayReactForTest = React;
"#,
        )
        .expect("setup file should be written");

        let mut session =
            RuntimeSession::new(clay_runtime_test_options()).expect("runtime should be created");
        session
            .load_main_module(&entry_path)
            .expect("setup module should load");

        let cases = [
            (
                "uncaught",
                "uncaught exploded",
                RuntimeReconcilerErrorCategory::Uncaught,
                Some(RuntimeRecoveryCategory::Fatal),
                RuntimeRecoveryDisposition::Teardown,
                false,
                Some("reconciler:uncaught:uncaught exploded"),
            ),
            (
                "caught",
                "caught exploded",
                RuntimeReconcilerErrorCategory::Caught,
                Some(RuntimeRecoveryCategory::BoundaryContained),
                RuntimeRecoveryDisposition::BoundedFailure,
                false,
                Some("reconciler:caught:caught exploded"),
            ),
            (
                "recoverable",
                "recoverable wobble",
                RuntimeReconcilerErrorCategory::Recoverable,
                Some(RuntimeRecoveryCategory::Recoverable),
                RuntimeRecoveryDisposition::Continue,
                true,
                None,
            ),
        ];

        for (
            index,
            (
                category,
                message,
                expected_category,
                expected_recovery_category,
                expected_recovery_disposition,
                expected_ok,
                expected_code,
            ),
        ) in cases.into_iter().enumerate()
        {
            let render_action = session
                .execute_script_as::<RuntimeAction>(
                    "[test:reconciler-category-case]",
                    format!(
                        r#"
(() => {{
  const React = globalThis.__clayReactForTest;
  const runtime = globalThis.__clayRuntimeForTest;
  function Trigger() {{
    runtime.__reportReconcilerErrorForTest(
      "{category}",
      "{message}",
      "<Trigger />",
      "BoundaryShell",
    );
    return React.createElement("box", {{ id: "root" }});
  }}
  return runtime.render(React.createElement(Trigger));
}})()
"#,
                    ),
                )
                .expect("render action should deserialize");

            let recovery_state = session
                .execute_script_as::<RuntimeRecoveryState>(
                    "[test:reconciler-recovery-state]",
                    "globalThis.__clayRuntimeForTest.__describeRecoveryStateForTest()",
                )
                .expect("runtime recovery state should deserialize");

            assert_eq!(
                render_action.ok, expected_ok,
                "unexpected runtime action result for category {category}"
            );
            assert_eq!(
                recovery_state.category, expected_recovery_category,
                "unexpected recovery category for case {index} ({category})"
            );
            assert_eq!(
                recovery_state.disposition, expected_recovery_disposition,
                "unexpected recovery disposition for case {index} ({category})"
            );
            assert_eq!(recovery_state.message.as_deref(), Some(message));
            match expected_code {
                Some(expected_code) => {
                    let code = render_action
                        .code
                        .expect("fatal reconciler category should produce an action code");
                    assert_eq!(code, expected_code);
                }
                None => {
                    assert!(
                        render_action.code.is_none(),
                        "recoverable reconciler category should preserve successful action semantics"
                    );
                }
            }

            let update = session.take_update();
            assert_eq!(
                update.reconciler_errors.len(),
                1,
                "expected exactly one structured reconciler error for case {index} ({category})"
            );
            let error = &update.reconciler_errors[0];
            assert_eq!(error.category, expected_category);
            assert_eq!(error.message, message);
            assert_eq!(error.component_stack, "<Trigger />");
            if expected_category == RuntimeReconcilerErrorCategory::Caught {
                assert_eq!(error.error_boundary.as_deref(), Some("BoundaryShell"));
            } else {
                assert!(error.error_boundary.is_none());
            }
        }
    }

    #[test]
    fn tracks_transitive_loaded_module_paths() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        let child_path = dir.path().join("child.ts");
        let grandchild_path = dir.path().join("grandchild.ts");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "clay";
import { answer } from "./child.ts";

render(<box answer={answer} />);
"#,
        )
        .expect("entry file should be written");
        std::fs::write(
            &child_path,
            r#"
import { answerValue } from "./grandchild.ts";

export const answer = answerValue;
"#,
        )
        .expect("child file should be written");
        std::fs::write(&grandchild_path, "export const answerValue = 42;")
            .expect("grandchild file should be written");

        let (session, _update) =
            RuntimeSession::load(&entry_path, test_options()).expect("tsx should load");
        assert_eq!(
            session.loaded_module_paths(),
            vec![entry_path, child_path, grandchild_path]
        );
    }

    #[test]
    fn failed_load_tracks_attempted_dependency_paths_for_missing_transitive_imports() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        let child_path = dir.path().join("child.ts");
        let missing_path = dir.path().join("missing.ts");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "clay";
import { answer } from "./child.ts";

render(<box answer={answer} />);
"#,
        )
        .expect("entry file should be written");
        std::fs::write(
            &child_path,
            r#"
import { answerValue } from "./missing.ts";

export const answer = answerValue;
"#,
        )
        .expect("child file should be written");

        let mut session = RuntimeSession::new(test_options()).expect("runtime should be created");
        let error = session
            .load_main_module(&entry_path)
            .expect_err("missing transitive import should fail");
        assert!(
            !error.to_string().trim().is_empty(),
            "expected missing transitive import to produce an error"
        );
        assert_eq!(
            session.loaded_module_paths(),
            vec![entry_path, child_path, missing_path]
        );
    }

    #[test]
    fn zero_delay_timeout_schedules_a_wake_and_drains_exactly_one_callback() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        session.set_host_wake_callback(|| {});
        session
            .execute_script(
                "[test:zero-timeout]",
                r#"
globalThis.timeoutCount = 0;
setTimeout(() => {
  globalThis.timeoutCount += 1;
}, 0);
"#,
            )
            .expect("zero-delay timeout should schedule");

        wait_for_pending_wake(&session, Duration::from_millis(100));
        let metrics = session.debug_metrics();
        assert_eq!(metrics.host_wake_count, 1);
        assert_eq!(metrics.host_wake_callback_count, 1);
        assert_eq!(metrics.timer_schedule_count, 1);
        assert_eq!(metrics.timer_fire_count, 1);
        assert!(metrics.pending_host_wake);
        assert_eq!(metrics.active_timer_count, 0);

        let drain = session
            .drain_host_callbacks()
            .expect("draining callbacks should succeed");
        assert!(drain.drained_work());
        assert_eq!(drain.callbacks_invoked, 1);
        assert!(!drain.needs_another_drain());
        assert_eq!(
            session
                .execute_json_expression_as::<u32>(
                    "[test:zero-timeout-count]",
                    "globalThis.timeoutCount",
                )
                .expect("timeout count should deserialize"),
            1
        );

        let metrics = session.debug_metrics();
        assert_eq!(metrics.host_callback_drain_cycles, 1);
        assert_eq!(metrics.host_callbacks_invoked, 1);
        assert_eq!(metrics.host_callback_drain_noop_count, 0);
        assert!(!metrics.pending_host_wake);
    }

    #[test]
    fn set_interval_zero_is_clamped_before_scheduling() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        session
            .execute_script(
                "[test:set-interval-zero-clamped]",
                r#"
globalThis.capturedSchedules = [];
const originalScheduleTimer = Deno.core.ops.op_host_schedule_timer;
Deno.core.ops.op_host_schedule_timer = (delayMs, intervalMs) => {
  globalThis.capturedSchedules.push([delayMs, intervalMs]);
  return originalScheduleTimer(delayMs, intervalMs);
};

const handle = setInterval(() => {}, 0);
clearInterval(handle);

Deno.core.ops.op_host_schedule_timer = originalScheduleTimer;
"#,
            )
            .expect("setInterval(0) should schedule");

        let captured = session
            .execute_json_expression_as::<Vec<Vec<i32>>>(
                "[test:set-interval-zero-clamped-captured]",
                "globalThis.capturedSchedules",
            )
            .expect("captured timer schedules should deserialize");

        assert_eq!(captured, vec![vec![0, 1]]);
    }

    #[test]
    fn set_interval_zero_keeps_worker_and_callback_draining_responsive() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        session
            .execute_script(
                "[test:set-interval-zero-responsiveness]",
                r#"
globalThis.zeroIntervalCount = 0;
globalThis.shortTimeoutFired = false;

globalThis.zeroIntervalHandle = setInterval(() => {
  globalThis.zeroIntervalCount += 1;
  if (globalThis.shortTimeoutFired && globalThis.zeroIntervalCount >= 3) {
    clearInterval(globalThis.zeroIntervalHandle);
    globalThis.zeroIntervalHandle = null;
  }
}, 0);

setTimeout(() => {
  globalThis.shortTimeoutFired = true;
}, 10);
"#,
            )
            .expect("setInterval(0) + timeout should schedule");

        wait_for_expression(
            &mut session,
            Duration::from_millis(250),
            "globalThis.shortTimeoutFired",
            |fired: bool| fired,
        );
        wait_for_expression(
            &mut session,
            Duration::from_millis(250),
            "globalThis.zeroIntervalHandle === null",
            |cleared: bool| cleared,
        );

        let interval_count = session
            .execute_json_expression_as::<u32>(
                "[test:set-interval-zero-count]",
                "globalThis.zeroIntervalCount",
            )
            .expect("interval count should deserialize");
        assert!(interval_count >= 3);

        let metrics = session.debug_metrics();
        assert!(metrics.host_callback_drain_cycles >= 1);
        assert!(metrics.host_callbacks_invoked >= 4);
        assert!(metrics.timer_fire_count >= 4);
        assert!(metrics.timer_repeat_fire_count >= 3);
        assert_eq!(metrics.timer_cancel_count, 1);
        assert_eq!(metrics.active_timer_count, 0);
    }

    #[test]
    fn runtime_defensively_clamps_zero_repeat_interval_from_host_ops() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        session
            .execute_script(
                "[test:forced-zero-repeat-interval]",
                r#"
globalThis.forcedZeroIntervalCount = 0;
globalThis.forcedZeroTimeoutFired = false;

const originalScheduleTimer = Deno.core.ops.op_host_schedule_timer;
Deno.core.ops.op_host_schedule_timer = (delayMs, intervalMs) => {
  if (intervalMs >= 0) {
    // Force a raw 0ms repeat interval through the host op boundary.
    return originalScheduleTimer(delayMs, 0);
  }
  return originalScheduleTimer(delayMs, intervalMs);
};

globalThis.forcedZeroHandle = setInterval(() => {
  globalThis.forcedZeroIntervalCount += 1;
  if (globalThis.forcedZeroTimeoutFired && globalThis.forcedZeroIntervalCount >= 3) {
    clearInterval(globalThis.forcedZeroHandle);
    globalThis.forcedZeroHandle = null;
  }
}, 0);

setTimeout(() => {
  globalThis.forcedZeroTimeoutFired = true;
}, 10);

Deno.core.ops.op_host_schedule_timer = originalScheduleTimer;
"#,
            )
            .expect("forced 0ms repeat interval should remain responsive");

        wait_for_expression(
            &mut session,
            Duration::from_millis(250),
            "globalThis.forcedZeroTimeoutFired",
            |fired: bool| fired,
        );
        wait_for_expression(
            &mut session,
            Duration::from_millis(250),
            "globalThis.forcedZeroHandle === null",
            |cleared: bool| cleared,
        );

        let interval_count = session
            .execute_json_expression_as::<u32>(
                "[test:forced-zero-repeat-interval-count]",
                "globalThis.forcedZeroIntervalCount",
            )
            .expect("forced-zero interval count should deserialize");
        assert!(interval_count >= 3);

        let metrics = session.debug_metrics();
        assert!(metrics.host_callback_drain_cycles >= 1);
        assert!(metrics.host_callbacks_invoked >= 4);
        assert!(metrics.timer_fire_count >= 4);
        assert!(metrics.timer_repeat_fire_count >= 3);
        assert_eq!(metrics.timer_cancel_count, 1);
        assert_eq!(metrics.active_timer_count, 0);
    }

    #[test]
    fn timer_worker_defensively_clamps_injected_invalid_repeat_intervals() {
        let host_runtime = HostRuntimeBridge::new();
        let timer_handle = 77;

        host_runtime.inner.register_active_timer(timer_handle);
        let timer_command_tx = host_runtime
            .inner
            .timer_command_tx
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .cloned()
            .expect("timer worker command channel should exist");

        timer_command_tx
            .send(TimerCommand::Schedule(TimerEntry {
                handle: timer_handle,
                next_fire_at: Instant::now(),
                interval: Some(Duration::ZERO),
            }))
            .expect("malformed timer entry should send");

        let mut total_due = 0usize;
        let mut max_batch = 0usize;
        let collect_deadline = Instant::now() + Duration::from_millis(80);
        while Instant::now() < collect_deadline && total_due < 4 {
            let due_handles = host_runtime.take_due_timers();
            if !due_handles.is_empty() {
                total_due += due_handles.len();
                max_batch = max_batch.max(due_handles.len());
            }
            let _ = host_runtime.take_pending_wake();
            std::thread::sleep(Duration::from_millis(2));
        }

        assert!(
            total_due >= 3,
            "worker should continue firing a malformed repeating timer after defensive clamp"
        );
        assert!(
            total_due <= 200,
            "defensive clamp should prevent runaway callback accumulation (saw {total_due})"
        );
        assert!(
            max_batch <= 128,
            "timer draining batches should stay bounded after clamping malformed intervals (max batch: {max_batch})"
        );

        timer_command_tx
            .send(TimerCommand::Cancel(timer_handle))
            .expect("timer cancel command should send");
        host_runtime.inner.unregister_active_timer(timer_handle);

        let (shutdown_done_tx, shutdown_done_rx) = mpsc::channel();
        let shutdown_runtime = host_runtime.clone();
        std::thread::spawn(move || {
            shutdown_runtime.shutdown();
            let _ = shutdown_done_tx.send(());
        });

        shutdown_done_rx
            .recv_timeout(Duration::from_millis(500))
            .expect("timer worker should accept shutdown and exit");

        let metrics = host_runtime.debug_metrics();
        assert!(metrics.timer_repeat_fire_count >= 3);
        assert_eq!(metrics.shutdown_count, 1);
        assert_eq!(metrics.active_timer_count, 0);
        assert!(!metrics.pending_host_wake);
    }

    #[test]
    fn typed_due_timer_transfer_reduces_allocation_and_latency_vs_json_baseline() {
        const ITERATIONS: usize = 8_192;
        const HANDLES_PER_BATCH: usize = 128;

        let baseline = benchmark_due_timer_transfer(
            DueTimerTransferMode::JsonRoundTripBaseline,
            ITERATIONS,
            HANDLES_PER_BATCH,
        );
        let typed = benchmark_due_timer_transfer(
            DueTimerTransferMode::Typed,
            ITERATIONS,
            HANDLES_PER_BATCH,
        );

        println!(
            "due-timer transfer benchmark: baseline elapsed={:?}, allocations={}, bytes={} | typed elapsed={:?}, allocations={}, bytes={}",
            baseline.elapsed,
            baseline.allocations,
            baseline.allocated_bytes,
            typed.elapsed,
            typed.allocations,
            typed.allocated_bytes,
        );

        assert_eq!(
            baseline.checksum, typed.checksum,
            "benchmark paths should process equivalent due timer payloads"
        );
        assert!(
            typed.allocations < baseline.allocations,
            "typed transfer should allocate less than JSON baseline (typed={}, baseline={})",
            typed.allocations,
            baseline.allocations
        );
        assert!(
            typed.allocated_bytes < baseline.allocated_bytes,
            "typed transfer should allocate fewer bytes than JSON baseline (typed={}, baseline={})",
            typed.allocated_bytes,
            baseline.allocated_bytes
        );
        assert!(
            typed.elapsed < baseline.elapsed,
            "typed transfer latency should beat JSON baseline (typed={:?}, baseline={:?})",
            typed.elapsed,
            baseline.elapsed
        );
    }

    #[test]
    fn host_callback_burst_benchmark_respects_the_per_drain_cap() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        const SCHEDULED_TIMEOUTS: u64 = 4_096;
        const TIMEOUT_DELAY_MS: u64 = 5;
        const STALL_AFTER_WAKE_MS: u64 = 80;

        session
            .execute_script(
                "[test:drain-cap-schedule]",
                format!(
                    r#"
globalThis.cappedDrainInvocations = 0;
for (let i = 0; i < {scheduled}; i++) {{
  setTimeout(() => {{
    globalThis.cappedDrainInvocations += 1;
  }}, {timeout_delay_ms});
}}
"#,
                    scheduled = SCHEDULED_TIMEOUTS,
                    timeout_delay_ms = TIMEOUT_DELAY_MS,
                ),
            )
            .expect("zero-delay timeouts should schedule");

        // Simulate a host stall: wait for timers to start waking, then avoid
        // draining while callbacks accumulate.
        wait_for_pending_wake(&session, Duration::from_millis(250));
        std::thread::sleep(Duration::from_millis(STALL_AFTER_WAKE_MS));

        let scheduling_snapshot = session
            .execute_json_expression_as::<serde_json::Value>(
                "[test:drain-cap-limit]",
                "globalThis.__clayDescribeHostSchedulingForTest()",
            )
            .expect("host scheduling snapshot should deserialize");
        let drain_limit = scheduling_snapshot["drainLimit"]
            .as_u64()
            .expect("drain limit should be numeric");
        assert_eq!(drain_limit as usize, HOST_CALLBACK_DRAIN_LIMIT);

        let started_at = Instant::now();
        let mut drains = 0u64;
        let mut total_invoked = 0u64;
        let mut max_batch = 0u64;
        while session.debug_metrics().pending_host_wake {
            let drain = session
                .drain_host_callbacks()
                .expect("draining host callbacks should succeed");
            if !drain.drained_work() {
                break;
            }
            let batch = drain.callbacks_invoked;
            assert!(
                batch <= drain_limit,
                "post-stall drain batch {batch} exceeded cap {drain_limit}"
            );
            drains += 1;
            total_invoked += batch;
            max_batch = max_batch.max(batch);
        }
        let elapsed = started_at.elapsed();

        let callback_invocations = session
            .execute_json_expression_as::<u64>(
                "[test:drain-cap-invocations]",
                "globalThis.cappedDrainInvocations",
            )
            .expect("callback count should deserialize");

        assert_eq!(callback_invocations, SCHEDULED_TIMEOUTS);
        assert_eq!(total_invoked, SCHEDULED_TIMEOUTS);
        assert!(
            drains >= 2,
            "expected capped draining to require multiple batches"
        );
        assert!(
            max_batch <= drain_limit,
            "a single drain call should not process more than the configured cap"
        );
        println!(
            "host callback burst benchmark: callbacks={SCHEDULED_TIMEOUTS}, drains={drains}, cap={drain_limit}, max_batch={max_batch}, elapsed={elapsed:?}"
        );
    }

    #[test]
    fn request_repaint_is_a_callback_free_invalidation_wake() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        session
            .execute_script(
                "[test:request-repaint]",
                r#"
globalThis.repaintCount = (globalThis.repaintCount ?? 0) + 1;
requestRepaint();
"#,
            )
            .expect("requestRepaint should execute");

        wait_for_pending_wake(&session, Duration::from_millis(100));
        let drain = session
            .drain_host_callbacks()
            .expect("requestRepaint wake should drain");
        assert!(drain.had_pending_wake);
        assert_eq!(drain.callbacks_invoked, 0);
        assert!(!drain.pending_host_wake_after_drain);

        let metrics = session.debug_metrics();
        assert_eq!(metrics.host_wake_count, 1);
        assert_eq!(metrics.host_callback_drain_cycles, 1);
        assert_eq!(metrics.host_callbacks_invoked, 0);
        assert!(!metrics.pending_host_wake);
    }

    #[test]
    fn repeating_timer_backlog_is_coalesced_after_host_stall() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        const INTERVAL_MS: u64 = 80;
        const STALL_MS: u64 = 320;
        session
            .execute_script(
                "[test:stall-coalescing-schedule]",
                format!(
                    r#"
globalThis.stallCoalescingCount = 0;
globalThis.stallCoalescingHandle = setInterval(() => {{
  globalThis.stallCoalescingCount += 1;
}}, {interval_ms});
"#,
                    interval_ms = INTERVAL_MS,
                ),
            )
            .expect("repeating timer should schedule");

        wait_for_pending_wake(&session, Duration::from_millis(250));
        std::thread::sleep(Duration::from_millis(STALL_MS));

        assert!(session
            .drain_host_callbacks()
            .expect("first post-stall drain should process one coalesced callback")
            .drained_work());

        let count_after_first_drain = session
            .execute_json_expression_as::<u64>(
                "[test:stall-coalescing-count-after-first-drain]",
                "globalThis.stallCoalescingCount",
            )
            .expect("coalescing count should deserialize after first drain");
        assert_eq!(
            count_after_first_drain, 1,
            "expected only one coalesced repeat callback after a long stall"
        );

        assert!(!session
            .drain_host_callbacks()
            .expect("immediate follow-up drain should be idle")
            .drained_work());
        let count_after_second_drain = session
            .execute_json_expression_as::<u64>(
                "[test:stall-coalescing-count-after-second-drain]",
                "globalThis.stallCoalescingCount",
            )
            .expect("coalescing count should deserialize after second drain");
        assert_eq!(count_after_second_drain, count_after_first_drain);

        std::thread::sleep(Duration::from_millis(INTERVAL_MS / 4));
        assert!(!session
            .drain_host_callbacks()
            .expect("drain before next interval should be idle")
            .drained_work());
        let count_before_next_interval = session
            .execute_json_expression_as::<u64>(
                "[test:stall-coalescing-count-before-next-interval]",
                "globalThis.stallCoalescingCount",
            )
            .expect("coalescing count should deserialize before next interval");
        assert_eq!(count_before_next_interval, count_after_first_drain);

        wait_for_pending_wake(&session, Duration::from_millis(250));
        assert!(session
            .drain_host_callbacks()
            .expect("drain after real interval advancement should process work")
            .drained_work());
        let count_after_real_interval = session
            .execute_json_expression_as::<u64>(
                "[test:stall-coalescing-count-after-real-interval]",
                "globalThis.stallCoalescingCount",
            )
            .expect("coalescing count should deserialize after real interval");
        assert_eq!(
            count_after_real_interval,
            count_after_first_drain + 1,
            "expected exactly one additional callback once real time advanced by another interval"
        );

        session
            .execute_script(
                "[test:stall-coalescing-clear]",
                "clearInterval(globalThis.stallCoalescingHandle);",
            )
            .expect("interval should cancel");

        let metrics = session.debug_metrics();
        assert!(metrics.timer_repeat_fire_count >= 1);
        assert_eq!(metrics.timer_cancel_count, 1);
        assert_eq!(metrics.active_timer_count, 0);
    }

    #[test]
    fn interval_repeats_until_cancelled_and_active_timer_count_returns_to_zero() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        session
            .execute_script(
                "[test:interval]",
                r#"
globalThis.intervalCount = 0;
globalThis.intervalHandle = setInterval(() => {
  globalThis.intervalCount += 1;
}, 5);
"#,
            )
            .expect("interval should schedule");

        wait_for_expression(
            &mut session,
            Duration::from_millis(200),
            "globalThis.intervalCount",
            |count: u32| count >= 2,
        );

        let metrics = session.debug_metrics();
        assert!(metrics.timer_fire_count >= 2);
        assert!(metrics.timer_repeat_fire_count >= 2);
        assert_eq!(metrics.active_timer_count, 1);
        assert!(metrics.active_timer_high_water >= 1);

        session
            .execute_script(
                "[test:interval-cancel]",
                "clearInterval(globalThis.intervalHandle);",
            )
            .expect("interval should cancel");
        let metrics = session.debug_metrics();
        assert_eq!(metrics.timer_cancel_count, 1);
        assert_eq!(metrics.active_timer_count, 0);
    }

    #[test]
    fn request_animation_frame_is_a_one_shot_callback() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        session
            .execute_script(
                "[test:raf]",
                r#"
globalThis.rafTimestamp = -1;
requestAnimationFrame((timestamp) => {
  globalThis.rafTimestamp = timestamp;
});
"#,
            )
            .expect("requestAnimationFrame should schedule");

        wait_for_pending_wake(&session, Duration::from_millis(200));
        let drain = session
            .drain_host_callbacks()
            .expect("raf callbacks should drain");
        assert!(drain.drained_work());
        assert_eq!(drain.callbacks_invoked, 1);
        assert!(!drain.needs_another_drain());

        let timestamp = session
            .execute_json_expression_as::<f64>("[test:raf-ts]", "globalThis.rafTimestamp")
            .expect("raf timestamp should deserialize");
        assert!(timestamp >= 0.0);

        let metrics = session.debug_metrics();
        assert_eq!(metrics.timer_schedule_count, 1);
        assert_eq!(metrics.timer_fire_count, 1);
        assert_eq!(metrics.active_timer_count, 0);
        assert_eq!(metrics.host_callbacks_invoked, 1);
    }

    #[test]
    fn shutdown_clears_timers_and_pending_wake_state() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        session
            .execute_script(
                "[test:shutdown]",
                r#"
globalThis.shutdownInterval = setInterval(() => {}, 1000);
setTimeout(() => {}, 0);
"#,
            )
            .expect("timers should schedule");

        wait_for_pending_wake(&session, Duration::from_millis(100));
        session.shutdown_host_runtime();

        let metrics = session.debug_metrics();
        assert_eq!(metrics.shutdown_count, 1);
        assert_eq!(metrics.active_timer_count, 0);
        assert!(!metrics.pending_host_wake);
    }

    #[test]
    fn abort_clears_timers_and_pending_wake_without_incrementing_shutdown_metrics() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        session
            .execute_script(
                "[test:abort]",
                r#"
globalThis.abortInterval = setInterval(() => {}, 1000);
setTimeout(() => {}, 0);
"#,
            )
            .expect("timers should schedule");

        wait_for_pending_wake(&session, Duration::from_millis(100));
        session.abort_host_runtime();

        let metrics = session.debug_metrics();
        assert_eq!(metrics.shutdown_count, 0);
        assert_eq!(metrics.active_timer_count, 0);
        assert!(!metrics.pending_host_wake);
    }

    #[test]
    fn draining_host_callbacks_without_work_is_counted_as_a_noop() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        assert!(!session
            .drain_host_callbacks()
            .expect("noop drain should succeed")
            .drained_work());
        let metrics = session.debug_metrics();
        assert_eq!(metrics.host_callback_drain_noop_count, 1);
        assert_eq!(metrics.host_callback_drain_cycles, 0);
        assert_eq!(metrics.host_callbacks_invoked, 0);
    }

    fn clay_runtime_test_options() -> JsxRuntimeOptions {
        let runtime_specifier =
            ModuleSpecifier::parse("clay-internal:/jsx-runtime").expect("specifier should parse");
        let transpiled = super::transpile_module(
            &runtime_specifier,
            MediaType::TypeScript,
            crate::CLAY_JSX_RUNTIME_SOURCE.to_owned(),
            "clay",
        )
        .expect("clay runtime test module should transpile");

        JsxRuntimeOptions::new("clay")
            .with_react_runtime_modules()
            .with_virtual_module("clay-internal:/jsx-runtime", transpiled.code)
    }

    fn test_options() -> JsxRuntimeOptions {
        JsxRuntimeOptions::new("clay")
            .with_virtual_module(
                "clay",
                r#"
export function render(element) {
  Deno.core.ops.op_commit_mutations(JSON.stringify(element));
}
"#,
            )
            .with_virtual_module(
                "clay/jsx-runtime",
                r#"
export const Fragment = Symbol.for("test.fragment");
export function jsx(type, props) {
  return { type, props: props ?? {} };
}
export const jsxs = jsx;
export const jsxDEV = jsx;
"#,
            )
    }

    fn wait_for_pending_wake(session: &RuntimeSession, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        loop {
            if session.debug_metrics().pending_host_wake {
                return;
            }
            if Instant::now() >= deadline {
                panic!("timed out waiting for a pending host wake");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn wait_for_expression<T>(
        session: &mut RuntimeSession,
        timeout: Duration,
        expression: &str,
        predicate: impl Fn(T) -> bool,
    ) where
        T: serde::de::DeserializeOwned,
    {
        let deadline = Instant::now() + timeout;
        loop {
            if session.debug_metrics().pending_host_wake {
                let _ = session
                    .drain_host_callbacks()
                    .expect("draining callbacks while waiting should succeed");
            }
            let value = session
                .execute_json_expression_as::<T>("[test:wait-expression]", expression)
                .expect("expression should deserialize");
            if predicate(value) {
                return;
            }
            if Instant::now() >= deadline {
                panic!("timed out waiting for expression {expression}");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
