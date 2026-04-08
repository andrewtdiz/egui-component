use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
    time::Instant,
};

use log::{debug, warn};
use mlua::{Error as LuaError, Lua};
use parking_lot::Mutex;

mod api;
mod graph;
mod host;
mod parse;
#[cfg(test)]
mod tests;
mod types;

use self::graph::{CommittedActiveRuntime, RequireMode, StagedRuntime};
use self::host::{HostBridgeCache, HostPhase, HostScopeToken, NullRuntimeHost};

pub use self::api::{runtime_api_luau_typings, runtime_api_reference_markdown};
pub use self::graph::ReloadKind;
pub use self::types::{
    ReloadMetrics, ReloadQueue, RuntimeAppHost, RuntimeConfig, RuntimeError, RuntimeFailureClass,
    RuntimeFailureMetrics, RuntimeFailureRecord, RuntimeFailureStage, RuntimeFrameMemoryMetrics,
    RuntimeGcMetrics, RuntimeInstrumentationConfig, RuntimeLeakMetrics, RuntimeLeakSample,
    RuntimeLifecycleMetrics, RuntimeMemoryMetrics, RuntimeMemoryOperationMetrics,
    RuntimeReloadMemoryMetrics, RuntimeRenderKind, RuntimeStageMetrics, RuntimeUiHost,
    ScriptSource, SurfaceCapabilities, SurfaceId, SurfaceMount, UiBooleanOutput,
    UiButtonGroupOptions, UiButtonGroupOutput, UiButtonOptions, UiButtonVariant, UiCardOptions,
    UiCheckboxOptions, UiCollapsibleOptions, UiCollapsibleOutput, UiContainerOptions,
    UiControlSize, UiDropdownMenuAction, UiDropdownMenuEntry, UiDropdownMenuOptions,
    UiDropdownMenuOutput, UiDropdownMenuSubmenu, UiLabelOptions, UiLabelTone, UiLabelWeight,
    UiNumberInputAxis, UiNumberInputOptions, UiNumberOutput, UiProgressOptions, UiRadioOptions,
    UiSelectOptions, UiSelectOutput, UiSelectVariant, UiSkeletonOptions, UiSkeletonShape,
    UiSliderOptions, UiSpinnerOptions, UiSwitchOptions, UiTabOption, UiTabsOptions, UiTabsOutput,
    UiTabsVariant, UiTextEditOptions, UiTextEditOutput, UiTooltipOptions, UiTooltipPlacement,
    UiVirtualListOptions, UiVirtualListOutput, WidgetIdentity,
};

/// Main runtime facade that owns the Luau VM and reload queue.
pub struct ScriptRuntime {
    config: RuntimeConfig,
    lua: Lua,
    reload_queue: ReloadQueue,
    active_runtime: Option<CommittedActiveRuntime>,
    pending_candidate: Option<StagedRuntime>,
    next_module_id: u64,
    next_host_scope_token: u64,
    require_mode: Arc<Mutex<RequireMode>>,
    last_compile_error: Option<String>,
    last_runtime_error: Option<String>,
    reload_metrics: ReloadMetrics,
    failure_metrics: RuntimeFailureMetrics,
    lifecycle_metrics: RuntimeLifecycleMetrics,
    memory_metrics: RuntimeMemoryMetrics,
    bridge_cache: Option<HostBridgeCache>,
    bridge_generation: u64,
}

impl ScriptRuntime {
    /// Construct a runtime with a fresh Luau VM.
    pub fn new(config: RuntimeConfig) -> Self {
        let lua = Lua::new();
        let initial_heap_bytes = lua.used_memory();
        let mut memory_metrics = RuntimeMemoryMetrics::default();
        memory_metrics.current_heap_bytes = initial_heap_bytes;
        memory_metrics.peak_heap_bytes = initial_heap_bytes;
        memory_metrics.gc.is_running = lua.gc_is_running();
        Self {
            config,
            lua,
            reload_queue: ReloadQueue::new(),
            active_runtime: None,
            pending_candidate: None,
            next_module_id: 1,
            next_host_scope_token: 1,
            require_mode: Arc::new(Mutex::new(RequireMode::Disabled)),
            last_compile_error: None,
            last_runtime_error: None,
            reload_metrics: ReloadMetrics::default(),
            failure_metrics: RuntimeFailureMetrics::default(),
            lifecycle_metrics: RuntimeLifecycleMetrics::default(),
            memory_metrics,
            bridge_cache: None,
            bridge_generation: 0,
        }
    }

    /// Access the runtime configuration.
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Get a shareable handle to the reload queue.
    pub fn reload_queue(&self) -> ReloadQueue {
        self.reload_queue.clone()
    }

    /// The last compile error recorded by the runtime, if any.
    pub fn last_compile_error(&self) -> Option<&str> {
        self.last_compile_error.as_deref()
    }

    /// The last runtime error recorded by the runtime, if any.
    pub fn last_runtime_error(&self) -> Option<&str> {
        self.last_runtime_error.as_deref()
    }

    /// Read-only reload metrics for the current runtime instance.
    pub fn reload_metrics(&self) -> &ReloadMetrics {
        &self.reload_metrics
    }

    /// Read-only failure counters and the currently surfaced failure.
    pub fn failure_metrics(&self) -> &RuntimeFailureMetrics {
        &self.failure_metrics
    }

    /// Read-only lifecycle counters for load, reload, and render buckets.
    pub fn lifecycle_metrics(&self) -> &RuntimeLifecycleMetrics {
        &self.lifecycle_metrics
    }

    /// Read-only GC and memory diagnostics for the current runtime instance.
    pub fn memory_metrics(&self) -> &RuntimeMemoryMetrics {
        &self.memory_metrics
    }

    /// The last structured failure surfaced by the runtime, if any.
    pub fn last_failure(&self) -> Option<&RuntimeFailureRecord> {
        self.failure_metrics.last_failure.as_ref()
    }

    /// Return `true` when the runtime has either an active committed graph or a staged candidate.
    pub fn has_render_target(&self) -> bool {
        self.active_runtime.is_some() || self.pending_candidate.is_some()
    }

    fn next_scope_token(&mut self) -> HostScopeToken {
        let token = HostScopeToken(self.next_host_scope_token);
        self.next_host_scope_token += 1;
        token
    }

    /// Load a root module from the provided source without a host API.
    pub fn load_root(
        &mut self,
        source: ScriptSource,
        mount: SurfaceMount,
    ) -> Result<(), RuntimeError> {
        let mut host = NullRuntimeHost;
        self.load_root_with_host(source, mount, &mut host)
    }

    /// Load a root module from the provided source with an active host API.
    pub fn load_root_with_host(
        &mut self,
        source: ScriptSource,
        mount: SurfaceMount,
        host: &mut dyn RuntimeAppHost,
    ) -> Result<(), RuntimeError> {
        let memory_before = self.lua.used_memory();
        let started_at = Instant::now();
        let result = self.with_app_host_scope(host, &mount, HostPhase::Load, move |runtime| {
            runtime.load_root_inner(source)
        });
        self.lifecycle_metrics
            .load
            .record(started_at.elapsed(), result.is_ok());
        self.instrument_reload_boundary(ReloadKind::Load, memory_before, result.is_ok());
        result
    }

    /// Reload the most recently loaded root source without a host API.
    pub fn reload_now(&mut self, mount: SurfaceMount) -> Result<(), RuntimeError> {
        let mut host = NullRuntimeHost;
        self.reload_now_with_host(mount, &mut host)
    }

    /// Reload the most recently loaded root source with an active host API.
    pub fn reload_now_with_host(
        &mut self,
        mount: SurfaceMount,
        host: &mut dyn RuntimeAppHost,
    ) -> Result<(), RuntimeError> {
        let memory_before = self.lua.used_memory();
        let started_at = Instant::now();
        let result = self.with_app_host_scope(host, &mount, HostPhase::Reload, |runtime| {
            let source = runtime.config.root_source.clone().ok_or_else(|| {
                runtime.record_compile_or_load_error(
                    RuntimeFailureStage::Reload,
                    None,
                    RuntimeError::config("no root source is loaded"),
                )
            })?;
            runtime.reload_root_inner(source)
        });
        self.lifecycle_metrics
            .reload
            .record(started_at.elapsed(), result.is_ok());
        self.instrument_reload_boundary(ReloadKind::Reload, memory_before, result.is_ok());
        result
    }

    /// Render the active root for one frame without a host API.
    pub fn render_frame(&mut self, mount: SurfaceMount) -> Result<(), RuntimeError> {
        let mut host = NullRuntimeHost;
        self.render_frame_with_host(mount, &mut host)
    }

    /// Render the active root for one frame with an active host API.
    pub fn render_frame_with_host(
        &mut self,
        mount: SurfaceMount,
        host: &mut dyn RuntimeUiHost,
    ) -> Result<(), RuntimeError> {
        let memory_before = self.lua.used_memory();
        let render_kind = self.current_render_kind();
        let render_stage = self.failure_stage_for_render_kind(render_kind);
        let rolled_back_on_failure =
            self.pending_candidate.is_some() && self.active_runtime.is_some();
        let attempting_candidate = self.pending_candidate.is_some();
        let started_at = Instant::now();
        let result = match self.with_ui_host_scope(
            host,
            &mount,
            render_stage,
            rolled_back_on_failure,
            attempting_candidate,
            |runtime, access| {
                runtime.render_frame_inner(access.as_ref(), render_stage, rolled_back_on_failure)
            },
        ) {
            Ok(()) => {
                if attempting_candidate {
                    self.clear_all_failure_state();
                } else {
                    self.clear_steady_state_frame_failure();
                }
                Ok(())
            }
            Err(err) => {
                if attempting_candidate {
                    self.discard_pending_bridge_generation();
                    self.pending_candidate = None;
                } else if self.last_runtime_error.is_none() {
                    self.last_runtime_error = Some(err.to_string());
                }
                Err(err)
            }
        };
        self.lifecycle_metrics
            .render
            .record(render_kind, started_at.elapsed(), result.is_ok());
        self.instrument_frame_boundary(
            render_kind,
            memory_before,
            result.is_ok(),
            attempting_candidate && result.is_ok(),
        );
        result
    }

    fn current_render_kind(&self) -> RuntimeRenderKind {
        match self
            .pending_candidate
            .as_ref()
            .map(|candidate| candidate.kind)
        {
            Some(ReloadKind::Load) => RuntimeRenderKind::FirstAfterLoad,
            Some(ReloadKind::Reload) => RuntimeRenderKind::FirstAfterReload,
            None => RuntimeRenderKind::SteadyState,
        }
    }

    fn failure_stage_for_reload_kind(&self, kind: ReloadKind) -> RuntimeFailureStage {
        match kind {
            ReloadKind::Load => RuntimeFailureStage::Load,
            ReloadKind::Reload => RuntimeFailureStage::Reload,
        }
    }

    fn failure_stage_for_render_kind(&self, kind: RuntimeRenderKind) -> RuntimeFailureStage {
        match kind {
            RuntimeRenderKind::FirstAfterLoad => RuntimeFailureStage::FirstAfterLoad,
            RuntimeRenderKind::FirstAfterReload => RuntimeFailureStage::FirstAfterReload,
            RuntimeRenderKind::SteadyState => RuntimeFailureStage::SteadyState,
        }
    }

    fn clear_all_failure_state(&mut self) {
        self.last_compile_error = None;
        self.last_runtime_error = None;
        self.failure_metrics.last_failure = None;
    }

    fn clear_steady_state_frame_failure(&mut self) {
        match self
            .failure_metrics
            .last_failure
            .as_ref()
            .map(|failure| failure.stage)
        {
            None | Some(RuntimeFailureStage::SteadyState) => {
                self.last_runtime_error = None;
                self.failure_metrics.last_failure = None;
            }
            Some(RuntimeFailureStage::Load)
            | Some(RuntimeFailureStage::Reload)
            | Some(RuntimeFailureStage::FirstAfterLoad)
            | Some(RuntimeFailureStage::FirstAfterReload) => {}
        }
    }

    fn note_failure(&mut self, failure: RuntimeFailureRecord) {
        match failure.class {
            RuntimeFailureClass::CompileLoad => {
                self.last_compile_error = Some(failure.message.clone());
            }
            RuntimeFailureClass::InitReload
            | RuntimeFailureClass::Update
            | RuntimeFailureClass::Render
            | RuntimeFailureClass::HostCallbackPanic
            | RuntimeFailureClass::CommandFlush => {
                self.last_runtime_error = Some(failure.message.clone());
            }
        }
        self.failure_metrics.record(failure);
    }

    fn record_compile_or_load_error(
        &mut self,
        stage: RuntimeFailureStage,
        module_context: Option<String>,
        err: RuntimeError,
    ) -> RuntimeError {
        self.note_failure(RuntimeFailureRecord {
            class: RuntimeFailureClass::CompileLoad,
            stage,
            message: err.to_string(),
            module_context,
            hook_or_api: None,
            rolled_back: false,
        });
        err
    }

    fn record_compile_or_load_lua_failure(
        &mut self,
        stage: RuntimeFailureStage,
        module_context: Option<String>,
        err: LuaError,
    ) -> RuntimeError {
        let message = format_lua_error(err);
        self.note_failure(RuntimeFailureRecord {
            class: RuntimeFailureClass::CompileLoad,
            stage,
            message: message.clone(),
            module_context,
            hook_or_api: None,
            rolled_back: false,
        });
        RuntimeError::backend(message)
    }

    fn record_init_reload_lua_failure(
        &mut self,
        stage: RuntimeFailureStage,
        module_context: String,
        hook: &str,
        err: LuaError,
    ) -> RuntimeError {
        let message = format_lua_error(err);
        let (class, hook_or_api) = match parse_host_callback_panic_api(&message) {
            Some(api_name) => (RuntimeFailureClass::HostCallbackPanic, Some(api_name)),
            None => (RuntimeFailureClass::InitReload, Some(hook.to_owned())),
        };
        self.note_failure(RuntimeFailureRecord {
            class,
            stage,
            message: message.clone(),
            module_context: Some(module_context),
            hook_or_api,
            rolled_back: false,
        });
        RuntimeError::backend(message)
    }

    fn record_frame_lua_failure(
        &mut self,
        stage: RuntimeFailureStage,
        phase: HostPhase,
        module_context: Option<String>,
        err: LuaError,
        rolled_back: bool,
    ) -> RuntimeError {
        let message = format_lua_error(err);
        let (class, hook_or_api) = match parse_host_callback_panic_api(&message) {
            Some(api_name) => (RuntimeFailureClass::HostCallbackPanic, Some(api_name)),
            None => match phase {
                HostPhase::Update => (RuntimeFailureClass::Update, None),
                HostPhase::Render => (RuntimeFailureClass::Render, None),
                HostPhase::Load | HostPhase::Reload => (RuntimeFailureClass::InitReload, None),
            },
        };
        self.note_failure(RuntimeFailureRecord {
            class,
            stage,
            message: message.clone(),
            module_context,
            hook_or_api,
            rolled_back,
        });
        RuntimeError::backend(message)
    }

    fn record_frame_runtime_error(
        &mut self,
        stage: RuntimeFailureStage,
        phase: HostPhase,
        module_context: Option<String>,
        err: RuntimeError,
        rolled_back: bool,
    ) -> RuntimeError {
        let class = match phase {
            HostPhase::Update => RuntimeFailureClass::Update,
            HostPhase::Render => RuntimeFailureClass::Render,
            HostPhase::Load | HostPhase::Reload => RuntimeFailureClass::InitReload,
        };
        self.note_failure(RuntimeFailureRecord {
            class,
            stage,
            message: err.to_string(),
            module_context,
            hook_or_api: None,
            rolled_back,
        });
        err
    }

    fn record_command_flush_failure(
        &mut self,
        stage: RuntimeFailureStage,
        hook_or_api: Option<String>,
        err: RuntimeError,
        rolled_back: bool,
    ) -> RuntimeError {
        self.note_failure(RuntimeFailureRecord {
            class: RuntimeFailureClass::CommandFlush,
            stage,
            message: err.to_string(),
            module_context: None,
            hook_or_api,
            rolled_back,
        });
        err
    }

    fn current_root_module_context(&self) -> Option<String> {
        self.pending_candidate
            .as_ref()
            .map(|candidate| candidate.runtime.graph.root_context().to_owned())
            .or_else(|| {
                self.active_runtime
                    .as_ref()
                    .map(|runtime| runtime.graph.root_context().to_owned())
            })
    }

    fn current_module_count(&self) -> usize {
        self.pending_candidate
            .as_ref()
            .map(|candidate| candidate.runtime.graph.module_count())
            .or_else(|| {
                self.active_runtime
                    .as_ref()
                    .map(|runtime| runtime.graph.module_count())
            })
            .unwrap_or(0)
    }

    fn record_heap_bytes(&mut self, heap_bytes: usize) {
        self.memory_metrics.current_heap_bytes = heap_bytes;
        self.memory_metrics.peak_heap_bytes = self.memory_metrics.peak_heap_bytes.max(heap_bytes);
    }

    fn instrument_reload_boundary(&mut self, kind: ReloadKind, before_bytes: usize, success: bool) {
        let after_bytes = self.lua.used_memory();
        self.record_heap_bytes(after_bytes);
        self.memory_metrics.reload.last_kind = Some(kind);
        self.memory_metrics
            .reload
            .totals
            .record_attempt(before_bytes, after_bytes, success);

        let post_gc_bytes = self.gc_step_boundary(
            GcBoundaryKind::Reload,
            self.config.instrumentation.reload_gc_step_kbytes,
        );
        if let Some(post_gc_bytes) = post_gc_bytes {
            self.memory_metrics
                .reload
                .totals
                .record_post_gc(post_gc_bytes, before_bytes);
        }
        let post_gc_recorded = post_gc_bytes.is_some();

        debug!(
            "luau_runtime event=memory_sample boundary=reload kind={} success={} before_bytes={} after_bytes={} post_gc_bytes={} post_gc_recorded={}",
            kind.as_str(),
            success,
            before_bytes,
            after_bytes,
            post_gc_bytes.unwrap_or(0),
            post_gc_recorded
        );
    }

    fn instrument_frame_boundary(
        &mut self,
        kind: RuntimeRenderKind,
        before_bytes: usize,
        success: bool,
        committed_candidate: bool,
    ) {
        let after_bytes = self.lua.used_memory();
        self.record_heap_bytes(after_bytes);
        self.memory_metrics.frame.last_render_kind = Some(kind);
        self.memory_metrics
            .frame
            .totals
            .record_attempt(before_bytes, after_bytes, success);

        let mut post_gc_bytes = self.gc_step_boundary(
            GcBoundaryKind::Frame,
            self.config.instrumentation.frame_gc_step_kbytes,
        );

        if committed_candidate {
            if let Some(sampled_bytes) = self.maybe_sample_leak_after_commit(kind) {
                post_gc_bytes = Some(sampled_bytes);
            }
        }

        if let Some(post_gc_bytes) = post_gc_bytes {
            self.memory_metrics
                .frame
                .totals
                .record_post_gc(post_gc_bytes, before_bytes);
        }
        let post_gc_recorded = post_gc_bytes.is_some();

        debug!(
            "luau_runtime event=memory_sample boundary=frame kind={} success={} committed_candidate={} before_bytes={} after_bytes={} post_gc_bytes={} post_gc_recorded={}",
            kind.as_str(),
            success,
            committed_candidate,
            before_bytes,
            after_bytes,
            post_gc_bytes.unwrap_or(0),
            post_gc_recorded
        );
    }

    fn gc_step_boundary(
        &mut self,
        boundary: GcBoundaryKind,
        budget_kbytes: usize,
    ) -> Option<usize> {
        if budget_kbytes == 0 {
            self.memory_metrics.gc.is_running = self.lua.gc_is_running();
            return None;
        }

        let budget_kbytes = budget_kbytes.min(i32::MAX as usize) as i32;
        match boundary {
            GcBoundaryKind::Frame => {
                self.memory_metrics.gc.frame_step_attempts += 1;
            }
            GcBoundaryKind::Reload => {
                self.memory_metrics.gc.reload_step_attempts += 1;
            }
        }
        self.memory_metrics.gc.last_step_kbytes = Some(budget_kbytes as usize);

        match self.lua.gc_step_kbytes(budget_kbytes) {
            Ok(finished_cycle) => {
                if finished_cycle {
                    match boundary {
                        GcBoundaryKind::Frame => {
                            self.memory_metrics.gc.frame_step_completions += 1;
                        }
                        GcBoundaryKind::Reload => {
                            self.memory_metrics.gc.reload_step_completions += 1;
                        }
                    }
                }
                self.memory_metrics.gc.last_cycle_finished = Some(finished_cycle);
                self.memory_metrics.gc.is_running = self.lua.gc_is_running();
                let post_gc_bytes = self.lua.used_memory();
                self.record_heap_bytes(post_gc_bytes);
                Some(post_gc_bytes)
            }
            Err(err) => {
                self.memory_metrics.gc.last_cycle_finished = None;
                self.memory_metrics.gc.is_running = self.lua.gc_is_running();
                warn!(
                    "luau_runtime event=gc_step_failed boundary={} budget_kbytes={} error={}",
                    boundary.as_str(),
                    budget_kbytes,
                    err
                );
                None
            }
        }
    }

    fn maybe_sample_leak_after_commit(&mut self, kind: RuntimeRenderKind) -> Option<usize> {
        match kind {
            RuntimeRenderKind::SteadyState => return None,
            RuntimeRenderKind::FirstAfterLoad => {}
            RuntimeRenderKind::FirstAfterReload => {
                self.memory_metrics.leak_detection.committed_reload_count += 1;
            }
        }

        let sample_interval = self.config.instrumentation.leak_sample_reload_interval;
        let should_sample = matches!(kind, RuntimeRenderKind::FirstAfterLoad)
            || (sample_interval > 0
                && self.memory_metrics.leak_detection.committed_reload_count % sample_interval
                    == 0);
        if !should_sample {
            return None;
        }

        let retained_bytes = match self.collect_full_gc_sample() {
            Some(retained_bytes) => retained_bytes,
            None => return None,
        };
        let heap_dump = match self.lua.heap_dump() {
            Ok(heap_dump) => heap_dump,
            Err(err) => {
                warn!("luau_runtime event=heap_dump_failed error={err}");
                return Some(retained_bytes);
            }
        };

        let live_object_count = heap_dump
            .size_by_type(None)
            .values()
            .map(|(count, _)| *count)
            .sum::<usize>();
        let retained_bytes = heap_dump.size().min(usize::MAX as u64) as usize;
        let previous_sample = self
            .memory_metrics
            .leak_detection
            .last_sample
            .as_ref()
            .cloned();
        let retained_growth_bytes = previous_sample
            .as_ref()
            .map(|sample| signed_delta(retained_bytes, sample.retained_bytes))
            .unwrap_or(0);
        let live_object_growth = previous_sample
            .as_ref()
            .map(|sample| signed_delta(live_object_count, sample.live_object_count))
            .unwrap_or(0);
        let module_count = self.current_module_count();
        let warning_emitted = previous_sample.as_ref().is_some_and(|previous| {
            retained_growth_bytes
                > self
                    .config
                    .instrumentation
                    .leak_retained_growth_warning_bytes as i64
                && module_count <= previous.module_count
        });

        self.memory_metrics.leak_detection.sample_count += 1;
        let sample = RuntimeLeakSample {
            sample_index: self.memory_metrics.leak_detection.sample_count,
            committed_reload_count: self.memory_metrics.leak_detection.committed_reload_count,
            retained_bytes,
            retained_growth_bytes,
            live_object_count,
            live_object_growth,
            module_count,
            warning_emitted,
        };
        self.memory_metrics.leak_detection.last_sample = Some(sample.clone());
        self.record_heap_bytes(retained_bytes);
        self.memory_metrics.gc.is_running = self.lua.gc_is_running();

        if warning_emitted {
            let message = format!(
                "retained heap grew by {} bytes after {} committed reloads without module-count growth",
                retained_growth_bytes,
                sample.committed_reload_count
            );
            warn!(
                "luau_runtime event=leak_warning retained_bytes={} retained_growth_bytes={} live_object_count={} live_object_growth={} module_count={} message={}",
                sample.retained_bytes,
                sample.retained_growth_bytes,
                sample.live_object_count,
                sample.live_object_growth,
                sample.module_count,
                message
            );
            self.memory_metrics.leak_detection.last_warning_message = Some(message);
        } else {
            debug!(
                "luau_runtime event=leak_sample sample_index={} retained_bytes={} retained_growth_bytes={} live_object_count={} live_object_growth={} module_count={}",
                sample.sample_index,
                sample.retained_bytes,
                sample.retained_growth_bytes,
                sample.live_object_count,
                sample.live_object_growth,
                sample.module_count
            );
        }

        Some(retained_bytes)
    }

    fn collect_full_gc_sample(&mut self) -> Option<usize> {
        self.memory_metrics.gc.full_gc_runs += 1;

        if let Err(err) = self.lua.gc_collect() {
            warn!("luau_runtime event=full_gc_failed pass=1 error={err}");
            return None;
        }
        if let Err(err) = self.lua.gc_collect() {
            warn!("luau_runtime event=full_gc_failed pass=2 error={err}");
            return None;
        }

        let retained_bytes = self.lua.used_memory();
        self.record_heap_bytes(retained_bytes);
        Some(retained_bytes)
    }
}

impl fmt::Debug for ScriptRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScriptRuntime")
            .field("config", &self.config)
            .field("reload_queue_len", &self.reload_queue.len())
            .field(
                "module_count",
                &self
                    .active_runtime
                    .as_ref()
                    .map(|runtime| runtime.graph.module_count()),
            )
            .field("has_pending_candidate", &self.pending_candidate.is_some())
            .field("last_compile_error", &self.last_compile_error)
            .field("last_runtime_error", &self.last_runtime_error)
            .field("reload_metrics", &self.reload_metrics)
            .field("failure_metrics", &self.failure_metrics)
            .field("lifecycle_metrics", &self.lifecycle_metrics)
            .field("memory_metrics", &self.memory_metrics)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GcBoundaryKind {
    Frame,
    Reload,
}

impl GcBoundaryKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Frame => "frame",
            Self::Reload => "reload",
        }
    }
}

fn source_hash(source_text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source_text.hash(&mut hasher);
    hasher.finish()
}

fn unavailable_host_api(name: &str) -> RuntimeError {
    RuntimeError::backend(format!(
        "host API `{name}` is not available during this runtime call"
    ))
}

fn runtime_lua_error(message: impl Into<String>) -> LuaError {
    LuaError::external(RuntimeError::backend(message.into()))
}

fn format_lua_error(err: LuaError) -> String {
    match err {
        LuaError::SyntaxError { message, .. } => message,
        LuaError::CallbackError { cause, .. } => format_lua_error((*cause).clone()),
        other => other.to_string(),
    }
}

fn runtime_error_from_lua_error(err: LuaError) -> RuntimeError {
    RuntimeError::backend(format_lua_error(err))
}

fn signed_delta(current: usize, previous: usize) -> i64 {
    match current.cmp(&previous) {
        std::cmp::Ordering::Greater => {
            current.saturating_sub(previous).min(i64::MAX as usize) as i64
        }
        std::cmp::Ordering::Less => {
            -((previous.saturating_sub(current).min(i64::MAX as usize)) as i64)
        }
        std::cmp::Ordering::Equal => 0,
    }
}

fn parse_host_callback_panic_api(message: &str) -> Option<String> {
    parse_backticked_suffix(message, "host callback panic in `")
}

fn parse_backticked_suffix(message: &str, marker: &str) -> Option<String> {
    let start = message.find(marker)? + marker.len();
    let tail = &message[start..];
    let end = tail.find('`')?;
    Some(tail[..end].to_owned())
}
