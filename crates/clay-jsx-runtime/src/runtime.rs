use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
        mpsc::{self, RecvTimeoutError, Sender},
        Arc, Mutex,
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
    pub commit_batches_json: Vec<String>,
    pub logs: RuntimeLogBuffer,
    pub host_debug_counters: RuntimeHostDebugCounters,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeHostDebugCounters {
    pub render_call_count: u64,
    pub unmount_count: u64,
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
}

#[derive(Debug, Clone, Default)]
struct HostRuntimeBridge {
    inner: Arc<HostRuntimeBridgeInner>,
}

struct HostRuntimeBridgeInner {
    is_shutdown: AtomicBool,
    wake: RuntimeWakeState,
    next_timer_handle: AtomicU32,
    pending_due_timers: Mutex<Vec<u32>>,
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
            self.inner
                .pending_due_timers
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(handle);
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
        self.inner
            .pending_due_timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain(|pending_handle| *pending_handle != handle);
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

    fn take_due_timers_json(&self) -> String {
        let mut pending_due_timers = self
            .inner
            .pending_due_timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        serde_json::to_string(&std::mem::take(&mut *pending_due_timers))
            .unwrap_or_else(|_| "[]".to_owned())
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
        if self.inner.is_shutdown.swap(true, Ordering::SeqCst) {
            return;
        }
        self.inner
            .metrics
            .shutdown_count
            .fetch_add(1, Ordering::Relaxed);
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
        self.inner
            .pending_due_timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
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
            Some(TimerCommand::Schedule(entry)) => {
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
            while timer.next_fire_at <= now {
                due_handles.push(timer.handle);
                if let Some(interval) = timer.interval {
                    inner.note_timer_fire(true);
                    timer.next_fire_at += interval;
                } else {
                    inner.note_timer_fire(false);
                    inner.unregister_active_timer(timer.handle);
                    completed_one_shots.push(timer.handle);
                    break;
                }
            }
        }
        for handle in completed_one_shots {
            timers.remove(&handle);
        }
        if due_handles.is_empty() {
            continue;
        }

        inner
            .pending_due_timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .extend(due_handles);
        inner.wake.trigger(&inner.metrics);
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
        }
    }
}

impl HostRuntimeBridgeInner {
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
    commit_batches_json: Vec<String>,
    logs: RuntimeLogBuffer,
    host_runtime: HostRuntimeBridge,
    host_debug_counters: RuntimeHostDebugCounters,
}

#[op2(fast)]
fn op_commit_mutations(
    state: &mut OpState,
    #[string] mutations_json: String,
) -> Result<(), JsErrorBox> {
    state
        .borrow_mut::<RuntimeState>()
        .commit_batches_json
        .push(mutations_json);
    Ok(())
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
#[string]
fn op_host_take_due_timers(state: &mut OpState) -> Result<String, JsErrorBox> {
    Ok(state
        .borrow::<RuntimeState>()
        .host_runtime
        .take_due_timers_json())
}

extension!(
    clay_jsx_host,
    ops = [
        op_commit_mutations,
        op_host_log,
        op_host_note_render,
        op_host_note_unmount,
        op_host_schedule_timer,
        op_host_cancel_timer,
        op_host_request_wake,
        op_host_take_due_timers
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
        RuntimeUpdate {
            commit_batches_json: std::mem::take(&mut state.commit_batches_json),
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

    pub fn drain_host_callbacks(&mut self) -> anyhow::Result<bool> {
        if !self.host_runtime.take_pending_wake() {
            self.host_runtime.note_callback_drain_noop();
            return Ok(false);
        }

        self.host_runtime.note_callback_drain_cycle();
        let mut iterations = 0usize;
        let mut invoked_callbacks = 0u64;
        loop {
            iterations += 1;
            if iterations > 256 {
                return Err(anyhow!("host callback drain exceeded 256 iterations"));
            }
            invoked_callbacks += self.execute_script_as::<u64>(
                "[clay:drain-host-callbacks]",
                "globalThis.__clayDrainHostCallbacks == null ? 0 : globalThis.__clayDrainHostCallbacks()",
            )?;
            if !self.host_runtime.take_pending_wake() {
                break;
            }
        }
        self.host_runtime.note_callbacks_invoked(invoked_callbacks);
        Ok(true)
    }

    pub fn shutdown_host_runtime(&mut self) {
        self.host_runtime.shutdown();
    }

    pub fn debug_metrics(&self) -> RuntimeDebugMetrics {
        self.host_runtime.debug_metrics()
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
    use std::time::{Duration, Instant};

    use tempfile::tempdir;

    use super::{JsxRuntimeOptions, RuntimeSession};

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

        assert_eq!(update.commit_batches_json.len(), 1);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&update.commit_batches_json[0])
                .expect("commit should be json"),
            serde_json::json!({ "type": "box", "props": { "answer": 42 } })
        );
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

        assert!(session
            .drain_host_callbacks()
            .expect("draining callbacks should succeed"));
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
        assert!(session
            .drain_host_callbacks()
            .expect("raf callbacks should drain"));

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
    fn draining_host_callbacks_without_work_is_counted_as_a_noop() {
        let mut session =
            RuntimeSession::new(JsxRuntimeOptions::new("clay")).expect("runtime should be created");
        assert!(!session
            .drain_host_callbacks()
            .expect("noop drain should succeed"));
        let metrics = session.debug_metrics();
        assert_eq!(metrics.host_callback_drain_noop_count, 1);
        assert_eq!(metrics.host_callback_drain_cycles, 0);
        assert_eq!(metrics.host_callbacks_invoked, 0);
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
