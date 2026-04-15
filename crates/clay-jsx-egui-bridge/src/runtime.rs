use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{mpsc, Arc},
    thread::JoinHandle,
};

use anyhow::{anyhow, bail, Context as AnyhowContext};
use clay_jsx_runtime::contract::{
    registry, schema_fingerprint, ContractChildPolicy, ContractEvent, ContractNode, ContractTree,
    CONTRACT_MODEL_VERSION,
};
use clay_jsx_runtime::{
    JsxRuntimeOptions, RuntimeCommitBatch, RuntimeDebugMetrics, RuntimeHostDebugCounters,
    RuntimeRecoveryCategory, RuntimeRecoveryDisposition, RuntimeRecoveryState, RuntimeSession,
};
use egui_component::contract::audit_tailwind_support;

use super::host_tree::{HostMutationBatch, HostTree};
use super::motion::MotionFrame;
use crate::{
    EGUI_JSX_RUNTIME_SOURCE, EGUI_LOWERING_SOURCE, EGUI_MODULE_SOURCE, EGUI_MOTION_REACT_SOURCE,
    EGUI_RUNTIME_SOURCE,
};

/// API-compatibility snapshot shape for reload callers.
///
/// V1 does not restore hook state. Successful reloads always construct a fresh
/// JS runtime and remount cold after the replacement entrypoint validates.
/// The shape remains so embedders can pass reload metadata without assuming it
/// changes live runtime behavior yet.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HotReloadState {
    #[serde(default)]
    pub hook_state: BTreeMap<String, Vec<serde_json::Value>>,
}

#[derive(Debug)]
pub struct RenderedJsx {
    pub tree: Option<ContractTree>,
    pub motion: MotionFrame,
    pub logs: clay_jsx_runtime::RuntimeLogBuffer,
}

#[derive(Debug)]
pub struct JsxRuntimeLoadFailure {
    pub dependency_paths: Vec<PathBuf>,
    pub error: anyhow::Error,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JsxRuntimeDebugMetrics {
    pub render_call_count: u64,
    pub dispatch_event_batch_count: u64,
    pub commit_protocol_rejected_batch_count: u64,
    pub commit_protocol_session_failure_count: u64,
    pub mutation_batch_count: u64,
    pub contract_tree_materialization_count: u64,
    pub contract_tree_noop_update_count: u64,
    pub motion_commit_update_count: u64,
    pub motion_only_update_count: u64,
    pub teardown_count: u64,
    pub unmount_count: u64,
    pub replace_root_count: u64,
    pub replace_subtree_count: u64,
    pub update_node_count: u64,
    pub insert_subtree_count: u64,
    pub remove_subtree_count: u64,
    pub set_children_count: u64,
    pub set_motion_count: u64,
    pub clear_motion_count: u64,
    pub runtime: RuntimeDebugMetrics,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct MutationBatchMetricsDelta {
    mutation_batch_count: u64,
    replace_root_count: u64,
    replace_subtree_count: u64,
    update_node_count: u64,
    insert_subtree_count: u64,
    remove_subtree_count: u64,
    set_children_count: u64,
    set_motion_count: u64,
    clear_motion_count: u64,
}

impl MutationBatchMetricsDelta {
    fn record_batch(&mut self, batch: &HostMutationBatch) {
        self.mutation_batch_count += 1;
        for mutation in &batch.mutations {
            match mutation {
                super::host_tree::HostMutation::ReplaceRoot { .. } => {
                    self.replace_root_count += 1;
                }
                super::host_tree::HostMutation::InsertSubtree { .. } => {
                    self.insert_subtree_count += 1;
                }
                super::host_tree::HostMutation::RemoveSubtree { .. } => {
                    self.remove_subtree_count += 1;
                }
                super::host_tree::HostMutation::ReplaceSubtree { .. } => {
                    self.replace_subtree_count += 1;
                }
                super::host_tree::HostMutation::UpdateNode { .. } => {
                    self.update_node_count += 1;
                }
                super::host_tree::HostMutation::SetChildren { .. } => {
                    self.set_children_count += 1;
                }
                super::host_tree::HostMutation::SetMotion { .. } => {
                    self.set_motion_count += 1;
                }
                super::host_tree::HostMutation::ClearMotion { .. } => {
                    self.clear_motion_count += 1;
                }
            }
        }
    }

    fn apply_to(self, metrics: &mut JsxRuntimeDebugMetrics) {
        metrics.mutation_batch_count += self.mutation_batch_count;
        metrics.replace_root_count += self.replace_root_count;
        metrics.replace_subtree_count += self.replace_subtree_count;
        metrics.update_node_count += self.update_node_count;
        metrics.insert_subtree_count += self.insert_subtree_count;
        metrics.remove_subtree_count += self.remove_subtree_count;
        metrics.set_children_count += self.set_children_count;
        metrics.set_motion_count += self.set_motion_count;
        metrics.clear_motion_count += self.clear_motion_count;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitProtocolState {
    Ready,
    Decoding {
        commit_batch_id: u32,
    },
    ValidatingEnvelope {
        commit_batch_id: u32,
    },
    Applying {
        commit_batch_id: u32,
    },
    ValidatingCommittedTree {
        last_applied_commit_batch_id: Option<u32>,
    },
    Acknowledging {
        commit_batch_ids: Vec<u32>,
    },
    Failed {
        rejected_commit_batch_id: Option<u32>,
        reason: String,
    },
}

impl Default for CommitProtocolState {
    fn default() -> Self {
        Self::Ready
    }
}

#[derive(Debug)]
pub enum JsxRuntimeLoadOutcome {
    Loaded {
        session: JsxRuntimeSession,
        rendered: RenderedJsx,
    },
    Failed(JsxRuntimeLoadFailure),
}

pub type JsxRuntimeHostWakeCallback = Arc<dyn Fn() + Send + Sync + 'static>;

#[derive(Debug)]
pub enum JsxRuntimeWorkerReloadOutcome {
    Loaded {
        rendered: RenderedJsx,
        dependency_paths: Vec<PathBuf>,
    },
    Failed(JsxRuntimeLoadFailure),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JsxRuntimeWorkerDebugSnapshot {
    pub live_session: Option<JsxRuntimeDebugMetrics>,
    pub last_torn_down_session: Option<JsxRuntimeDebugMetrics>,
    pub live_recovery_state: Option<RuntimeRecoveryState>,
    pub last_torn_down_recovery_state: Option<RuntimeRecoveryState>,
}

#[derive(Debug)]
pub enum JsxRuntimeWorkerEvent {
    ReloadCompleted {
        outcome: JsxRuntimeWorkerReloadOutcome,
        snapshot: JsxRuntimeWorkerDebugSnapshot,
    },
    DispatchCompleted {
        result: anyhow::Result<RenderedJsx>,
        snapshot: JsxRuntimeWorkerDebugSnapshot,
    },
    DrainCompleted {
        result: anyhow::Result<Option<RenderedJsx>>,
        snapshot: JsxRuntimeWorkerDebugSnapshot,
    },
    TickCompleted {
        result: anyhow::Result<RenderedJsx>,
        snapshot: JsxRuntimeWorkerDebugSnapshot,
    },
    TeardownCompleted {
        result: anyhow::Result<clay_jsx_runtime::RuntimeLogBuffer>,
        snapshot: JsxRuntimeWorkerDebugSnapshot,
    },
}

#[cfg(test)]
#[derive(Debug, Default)]
struct CommitProtocolTestHooks {
    force_post_apply_validation_failure: bool,
}

enum JsxRuntimeWorkerCommand {
    Reload {
        entry_path: PathBuf,
    },
    DispatchEvents {
        events: Vec<ContractEvent>,
    },
    DrainPendingRuntimeUpdates,
    TickMotion {
        now_secs: f64,
    },
    Teardown,
    Snapshot {
        reply: mpsc::Sender<JsxRuntimeWorkerDebugSnapshot>,
    },
    Shutdown,
}

#[derive(Debug)]
pub struct JsxRuntimeSessionWorker {
    command_tx: mpsc::Sender<JsxRuntimeWorkerCommand>,
    event_rx: mpsc::Receiver<JsxRuntimeWorkerEvent>,
    worker_handle: Option<JoinHandle<()>>,
}

impl JsxRuntimeSessionWorker {
    pub fn spawn(wake_callback: Option<JsxRuntimeHostWakeCallback>) -> anyhow::Result<Self> {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        let worker_handle = std::thread::Builder::new()
            .name("jsx-runtime-session-worker".to_owned())
            .spawn(move || runtime_worker_loop(command_rx, event_tx, wake_callback))
            .context("failed to spawn JSX runtime session worker thread")?;

        Ok(Self {
            command_tx,
            event_rx,
            worker_handle: Some(worker_handle),
        })
    }

    pub fn request_reload(&self, entry_path: impl Into<PathBuf>) -> anyhow::Result<()> {
        self.command_tx
            .send(JsxRuntimeWorkerCommand::Reload {
                entry_path: entry_path.into(),
            })
            .map_err(|error| anyhow!("failed to queue JSX runtime reload: {error}"))
    }

    pub fn request_dispatch_events(&self, events: Vec<ContractEvent>) -> anyhow::Result<()> {
        self.command_tx
            .send(JsxRuntimeWorkerCommand::DispatchEvents { events })
            .map_err(|error| anyhow!("failed to queue JSX runtime event dispatch: {error}"))
    }

    pub fn request_drain_pending_runtime_updates(&self) -> anyhow::Result<()> {
        self.command_tx
            .send(JsxRuntimeWorkerCommand::DrainPendingRuntimeUpdates)
            .map_err(|error| anyhow!("failed to queue JSX runtime update drain: {error}"))
    }

    pub fn request_tick_motion(&self, now_secs: f64) -> anyhow::Result<()> {
        self.command_tx
            .send(JsxRuntimeWorkerCommand::TickMotion { now_secs })
            .map_err(|error| anyhow!("failed to queue JSX runtime motion tick: {error}"))
    }

    pub fn request_teardown(&self) -> anyhow::Result<()> {
        self.command_tx
            .send(JsxRuntimeWorkerCommand::Teardown)
            .map_err(|error| anyhow!("failed to queue JSX runtime teardown: {error}"))
    }

    pub fn try_recv_event(&self) -> anyhow::Result<Option<JsxRuntimeWorkerEvent>> {
        match self.event_rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(anyhow!("JSX runtime worker event channel disconnected"))
            }
        }
    }

    pub fn debug_snapshot(&self) -> anyhow::Result<JsxRuntimeWorkerDebugSnapshot> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.command_tx
            .send(JsxRuntimeWorkerCommand::Snapshot { reply: reply_tx })
            .map_err(|error| anyhow!("failed to request JSX runtime worker snapshot: {error}"))?;
        reply_rx
            .recv()
            .map_err(|error| anyhow!("failed to receive JSX runtime worker snapshot: {error}"))
    }

    pub fn shutdown(&mut self) -> anyhow::Result<()> {
        let _ = self.command_tx.send(JsxRuntimeWorkerCommand::Shutdown);
        let Some(worker_handle) = self.worker_handle.take() else {
            return Ok(());
        };
        worker_handle.join().map_err(|panic_payload| {
            let reason = if let Some(reason) = panic_payload.downcast_ref::<&str>() {
                (*reason).to_owned()
            } else if let Some(reason) = panic_payload.downcast_ref::<String>() {
                reason.clone()
            } else {
                "unknown panic payload".to_owned()
            };
            anyhow!("JSX runtime worker panicked: {reason}")
        })
    }
}

impl Drop for JsxRuntimeSessionWorker {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

struct RuntimeWorkerState {
    wake_callback: Option<JsxRuntimeHostWakeCallback>,
    session: Option<JsxRuntimeSession>,
    last_torn_down_session: Option<JsxRuntimeDebugMetrics>,
    last_torn_down_recovery_state: Option<RuntimeRecoveryState>,
}

impl RuntimeWorkerState {
    fn new(wake_callback: Option<JsxRuntimeHostWakeCallback>) -> Self {
        Self {
            wake_callback,
            session: None,
            last_torn_down_session: None,
            last_torn_down_recovery_state: None,
        }
    }

    fn snapshot(&self) -> JsxRuntimeWorkerDebugSnapshot {
        JsxRuntimeWorkerDebugSnapshot {
            live_session: self.session.as_ref().map(JsxRuntimeSession::debug_metrics),
            last_torn_down_session: self.last_torn_down_session.clone(),
            live_recovery_state: self
                .session
                .as_ref()
                .map(JsxRuntimeSession::debug_recovery_state),
            last_torn_down_recovery_state: self.last_torn_down_recovery_state.clone(),
        }
    }

    fn reload(&mut self, entry_path: PathBuf) -> JsxRuntimeWorkerReloadOutcome {
        let hot_reload_state = self
            .session
            .as_mut()
            .and_then(|session| session.capture_hot_reload_state().ok());

        // V1 reload policy: validate the replacement session first, keep the
        // last good session live on validation failure, and only then tear down
        // the previous session before installing the validated cold remount.
        if self.session.is_some() {
            if let Err(failure) = validate_replacement_load_on_background_thread(
                entry_path.clone(),
                hot_reload_state.clone(),
            ) {
                return JsxRuntimeWorkerReloadOutcome::Failed(failure);
            }

            let _ = self.teardown_live_session();
        }

        match JsxRuntimeSession::load_with_hot_reload_state_outcome(
            &entry_path,
            hot_reload_state.as_ref(),
        ) {
            JsxRuntimeLoadOutcome::Loaded {
                mut session,
                rendered,
            } => {
                self.install_wake_callback(&mut session);
                let dependency_paths = session.dependency_paths();
                self.session = Some(session);
                JsxRuntimeWorkerReloadOutcome::Loaded {
                    rendered,
                    dependency_paths,
                }
            }
            JsxRuntimeLoadOutcome::Failed(failure) => {
                JsxRuntimeWorkerReloadOutcome::Failed(failure)
            }
        }
    }

    fn dispatch_events(&mut self, events: Vec<ContractEvent>) -> anyhow::Result<RenderedJsx> {
        let Some(session) = self.session.as_mut() else {
            bail!("JSX runtime session is not loaded");
        };
        session.dispatch_events(&events)
    }

    fn drain_pending_runtime_updates(&mut self) -> anyhow::Result<Option<RenderedJsx>> {
        let Some(session) = self.session.as_mut() else {
            bail!("JSX runtime session is not loaded");
        };
        session.drain_pending_runtime_updates()
    }

    fn tick_motion(&mut self, now_secs: f64) -> anyhow::Result<RenderedJsx> {
        let Some(session) = self.session.as_mut() else {
            bail!("JSX runtime session is not loaded");
        };
        session.tick_motion(now_secs)
    }

    fn teardown_live_session(&mut self) -> anyhow::Result<clay_jsx_runtime::RuntimeLogBuffer> {
        let Some(mut session) = self.session.take() else {
            return Ok(clay_jsx_runtime::RuntimeLogBuffer::new());
        };
        let result = session.teardown();
        self.last_torn_down_session = Some(session.debug_metrics());
        self.last_torn_down_recovery_state = Some(session.debug_recovery_state());
        result
    }

    fn install_wake_callback(&self, session: &mut JsxRuntimeSession) {
        let Some(wake_callback) = self.wake_callback.as_ref().map(Arc::clone) else {
            session.clear_wake_callback();
            return;
        };
        session.set_wake_callback(move || wake_callback());
    }
}

fn runtime_worker_loop(
    command_rx: mpsc::Receiver<JsxRuntimeWorkerCommand>,
    event_tx: mpsc::Sender<JsxRuntimeWorkerEvent>,
    wake_callback: Option<JsxRuntimeHostWakeCallback>,
) {
    let mut state = RuntimeWorkerState::new(wake_callback);

    while let Ok(command) = command_rx.recv() {
        let send_result = match command {
            JsxRuntimeWorkerCommand::Reload { entry_path } => {
                let outcome = state.reload(entry_path);
                event_tx.send(JsxRuntimeWorkerEvent::ReloadCompleted {
                    outcome,
                    snapshot: state.snapshot(),
                })
            }
            JsxRuntimeWorkerCommand::DispatchEvents { events } => {
                let result = state.dispatch_events(events);
                event_tx.send(JsxRuntimeWorkerEvent::DispatchCompleted {
                    result,
                    snapshot: state.snapshot(),
                })
            }
            JsxRuntimeWorkerCommand::DrainPendingRuntimeUpdates => {
                let result = state.drain_pending_runtime_updates();
                event_tx.send(JsxRuntimeWorkerEvent::DrainCompleted {
                    result,
                    snapshot: state.snapshot(),
                })
            }
            JsxRuntimeWorkerCommand::TickMotion { now_secs } => {
                let result = state.tick_motion(now_secs);
                event_tx.send(JsxRuntimeWorkerEvent::TickCompleted {
                    result,
                    snapshot: state.snapshot(),
                })
            }
            JsxRuntimeWorkerCommand::Teardown => {
                let result = state.teardown_live_session();
                event_tx.send(JsxRuntimeWorkerEvent::TeardownCompleted {
                    result,
                    snapshot: state.snapshot(),
                })
            }
            JsxRuntimeWorkerCommand::Snapshot { reply } => {
                let _ = reply.send(state.snapshot());
                continue;
            }
            JsxRuntimeWorkerCommand::Shutdown => {
                let _ = state.teardown_live_session();
                break;
            }
        };

        if send_result.is_err() {
            break;
        }
    }

    let _ = state.teardown_live_session();
}

pub struct JsxRuntimeSession {
    entry_path: PathBuf,
    host_tree: HostTree,
    runtime: RuntimeSession,
    metrics: JsxRuntimeDebugMetrics,
    recovery_state: RuntimeRecoveryState,
    commit_protocol_state: CommitProtocolState,
    #[cfg(test)]
    commit_protocol_test_hooks: CommitProtocolTestHooks,
    torn_down: bool,
}

impl std::fmt::Debug for JsxRuntimeSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JsxRuntimeSession")
            .field("entry_path", &self.entry_path)
            .finish_non_exhaustive()
    }
}

impl JsxRuntimeSession {
    pub fn load(entry_path: &Path) -> anyhow::Result<(Self, RenderedJsx)> {
        Self::load_with_hot_reload_state(entry_path, None)
    }

    pub fn load_with_hot_reload_state(
        entry_path: &Path,
        hot_reload_state: Option<&HotReloadState>,
    ) -> anyhow::Result<(Self, RenderedJsx)> {
        match Self::load_with_hot_reload_state_outcome(entry_path, hot_reload_state) {
            JsxRuntimeLoadOutcome::Loaded { session, rendered } => Ok((session, rendered)),
            JsxRuntimeLoadOutcome::Failed(failure) => Err(failure.error),
        }
    }

    pub fn load_with_hot_reload_state_outcome(
        entry_path: &Path,
        hot_reload_state: Option<&HotReloadState>,
    ) -> JsxRuntimeLoadOutcome {
        // This always creates a fresh runtime/session. V1 intentionally does
        // not reuse live JS hook state across reloads.
        let entry_path = match absolute_path(entry_path) {
            Ok(entry_path) => entry_path,
            Err(error) => {
                return JsxRuntimeLoadOutcome::Failed(JsxRuntimeLoadFailure {
                    dependency_paths: Vec::new(),
                    error,
                });
            }
        };
        let mut runtime = match RuntimeSession::new(egui_runtime_options()) {
            Ok(runtime) => runtime,
            Err(error) => {
                return JsxRuntimeLoadOutcome::Failed(JsxRuntimeLoadFailure {
                    dependency_paths: Vec::new(),
                    error,
                });
            }
        };

        if let Err(error) = install_contract_metadata(&mut runtime) {
            return load_failure(runtime, error);
        }
        if let Err(error) = install_hot_reload_state(&mut runtime, hot_reload_state) {
            return load_failure(runtime, error);
        }
        if let Err(error) = runtime.load_main_module(&entry_path) {
            return load_failure(runtime, error);
        }

        let mut session = Self {
            entry_path,
            host_tree: HostTree::default(),
            runtime,
            metrics: JsxRuntimeDebugMetrics::default(),
            recovery_state: RuntimeRecoveryState::default(),
            commit_protocol_state: CommitProtocolState::Ready,
            #[cfg(test)]
            commit_protocol_test_hooks: CommitProtocolTestHooks::default(),
            torn_down: false,
        };
        match session.take_rendered() {
            Ok(rendered) => JsxRuntimeLoadOutcome::Loaded { session, rendered },
            Err(error) => JsxRuntimeLoadOutcome::Failed(JsxRuntimeLoadFailure {
                dependency_paths: session.runtime.loaded_module_paths(),
                error,
            }),
        }
    }

    pub fn capture_hot_reload_state(&mut self) -> anyhow::Result<HotReloadState> {
        // V1 capture is intentionally empty. The API exists so embedders can
        // thread reload metadata through validation/swap paths without
        // implying that hook-state restoration is supported yet.
        Ok(HotReloadState::default())
    }

    pub fn dependency_paths(&self) -> Vec<PathBuf> {
        self.runtime.loaded_module_paths()
    }

    pub fn debug_metrics(&self) -> JsxRuntimeDebugMetrics {
        let mut metrics = self.metrics.clone();
        metrics.runtime = self.runtime.debug_metrics();
        metrics
    }

    pub fn debug_commit_protocol_state(&self) -> CommitProtocolState {
        self.commit_protocol_state.clone()
    }

    pub fn debug_recovery_state(&self) -> RuntimeRecoveryState {
        self.recovery_state.clone()
    }

    pub fn set_wake_callback<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.runtime.set_host_wake_callback(callback);
    }

    pub fn clear_wake_callback(&mut self) {
        self.runtime.clear_host_wake_callback();
    }

    pub fn drain_pending_runtime_updates(&mut self) -> anyhow::Result<Option<RenderedJsx>> {
        self.ensure_host_recovery_allows_work()?;
        self.ensure_commit_protocol_healthy()?;
        let drain_result = match self.runtime.drain_host_callbacks() {
            Ok(result) => result,
            Err(error) => {
                let error = anyhow!("failed to drain pending egui host callbacks: {error}");
                self.enter_fatal_runtime_failure(error.to_string());
                return Err(error);
            }
        };
        if !drain_result.drained_work() {
            return Ok(None);
        }
        self.take_rendered().map(Some)
    }

    pub fn dispatch_events(&mut self, events: &[ContractEvent]) -> anyhow::Result<RenderedJsx> {
        self.ensure_host_recovery_allows_work()?;
        self.ensure_commit_protocol_healthy()?;
        let events_json = serde_json::to_string(events)?;
        let source = format!("globalThis.__eguiDispatchEvents({events_json});");
        if let Err(error) = self
            .runtime
            .execute_script("[egui:dispatch-events]", source)
        {
            let error = anyhow!("failed to dispatch egui events into JSX runtime: {error}");
            self.enter_fatal_runtime_failure(error.to_string());
            return Err(error);
        }
        if let Err(error) = self
            .runtime
            .execute_script("[egui:post-dispatch-flush]", "undefined;")
        {
            let error = anyhow!("failed to flush egui runtime after event dispatch: {error}");
            self.enter_fatal_runtime_failure(error.to_string());
            return Err(error);
        }
        if self.runtime.debug_metrics().pending_host_wake {
            if let Err(error) = self.runtime.drain_host_callbacks() {
                let error =
                    anyhow!("failed to drain egui host callbacks after event dispatch: {error}");
                self.enter_fatal_runtime_failure(error.to_string());
                return Err(error);
            }
        }
        let rendered = self.take_rendered()?;
        self.metrics.dispatch_event_batch_count += 1;
        Ok(rendered)
    }

    pub fn tick_motion(&mut self, now_secs: f64) -> anyhow::Result<RenderedJsx> {
        self.ensure_host_recovery_allows_work()?;
        self.ensure_commit_protocol_healthy()?;
        if !self.host_tree.has_root() {
            bail!("{} did not call render(<... />)", self.entry_path.display());
        }
        let tick = self.host_tree.tick_motion(now_secs);
        let _changed = tick.changed;
        self.metrics.motion_only_update_count += 1;
        Ok(RenderedJsx {
            tree: None,
            motion: tick.frame,
            logs: clay_jsx_runtime::RuntimeLogBuffer::new(),
        })
    }

    pub fn teardown(&mut self) -> anyhow::Result<clay_jsx_runtime::RuntimeLogBuffer> {
        if self.torn_down {
            return Ok(clay_jsx_runtime::RuntimeLogBuffer::new());
        }
        self.torn_down = true;
        self.metrics.teardown_count += 1;
        let result = self
            .runtime
            .execute_script(
                "[egui:unmount-runtime]",
                "globalThis.__eguiUnmountRuntime == null ? undefined : globalThis.__eguiUnmountRuntime();",
            )
            .map_err(|error| anyhow!("failed to tear down egui JSX runtime: {error}"));
        self.runtime.shutdown_host_runtime();
        result?;
        let update = self.runtime.take_update();
        self.apply_runtime_update_metrics(update.host_debug_counters);
        Ok(update.logs)
    }

    fn take_rendered(&mut self) -> anyhow::Result<RenderedJsx> {
        self.ensure_host_recovery_allows_work()?;
        self.ensure_commit_protocol_healthy()?;
        let update = self.runtime.take_update();
        self.apply_runtime_update_metrics(update.host_debug_counters);
        self.sync_recovery_state_from_runtime();
        let mutation_batches = update.commit_batches;
        let mut logs = update.logs;

        if mutation_batches.is_empty() && !self.host_tree.has_root() {
            bail!("{} did not call render(<... />)", self.entry_path.display());
        }

        let mut next_host_tree = self.host_tree.clone();
        let mut tree_changed = false;
        let mut motion_only_commit_update = false;
        let mut metrics_delta = MutationBatchMetricsDelta::default();
        let mut applied_commit_batch_ids = Vec::new();
        for commit_batch in mutation_batches {
            let commit_batch_id = commit_batch.commit_batch_id;
            self.transition_commit_protocol_state(CommitProtocolState::Decoding {
                commit_batch_id,
            });
            let batch = match decode_mutation_batch(&self.entry_path, &commit_batch) {
                Ok(batch) => batch,
                Err(error) => return Err(self.reject_commit_batch(commit_batch_id, error)),
            };
            self.transition_commit_protocol_state(CommitProtocolState::ValidatingEnvelope {
                commit_batch_id,
            });
            let batch_changes_contract_tree = batch.changes_contract_tree();
            let batch_is_motion_only = batch.is_motion_only();
            metrics_delta.record_batch(&batch);
            if batch.version != CONTRACT_MODEL_VERSION {
                let error = anyhow!(
                    "{} returned contract model version {}, but this host supports version {}",
                    self.entry_path.display(),
                    batch.version,
                    CONTRACT_MODEL_VERSION
                );
                return Err(self.reject_commit_batch(commit_batch_id, error));
            }
            if batch.schema_fingerprint != schema_fingerprint() {
                let error = anyhow!(
                    "{} returned contract schema fingerprint {}, but this host supports {}",
                    self.entry_path.display(),
                    batch.schema_fingerprint,
                    schema_fingerprint()
                );
                return Err(self.reject_commit_batch(commit_batch_id, error));
            }
            if batch.mutations.is_empty() {
                applied_commit_batch_ids.push(commit_batch_id);
                continue;
            }
            self.transition_commit_protocol_state(CommitProtocolState::Applying {
                commit_batch_id,
            });
            tree_changed |= batch_changes_contract_tree;
            motion_only_commit_update |= batch_is_motion_only;
            if let Err(error) = next_host_tree.apply_mutations_in_place(batch.mutations) {
                return Err(self.reject_commit_batch(commit_batch_id, error));
            }
            applied_commit_batch_ids.push(commit_batch_id);
        }

        let rejected_commit_batch_id = applied_commit_batch_ids.last().copied();

        if !next_host_tree.has_root() {
            let error = anyhow!("{} did not call render(<... />)", self.entry_path.display());
            return Err(self.reject_or_fail_commit_protocol(rejected_commit_batch_id, error));
        }

        self.transition_commit_protocol_state(CommitProtocolState::ValidatingCommittedTree {
            last_applied_commit_batch_id: rejected_commit_batch_id,
        });
        let tree = if !tree_changed {
            None
        } else {
            let tree = next_host_tree.materialize().map_err(|error| {
                self.reject_or_fail_commit_protocol(rejected_commit_batch_id, error)
            })?;
            validate_contract_tree(&self.entry_path, &tree).map_err(|error| {
                self.reject_or_fail_commit_protocol(rejected_commit_batch_id, error)
            })?;
            self.maybe_fail_post_apply_validation().map_err(|error| {
                self.reject_or_fail_commit_protocol(rejected_commit_batch_id, error)
            })?;
            append_tailwind_diagnostics(&tree, &mut logs);
            Some(tree)
        };

        self.transition_commit_protocol_state(CommitProtocolState::Acknowledging {
            commit_batch_ids: applied_commit_batch_ids.clone(),
        });
        self.acknowledge_commit_batches(&applied_commit_batch_ids)
            .map_err(|error| self.fail_commit_protocol(error))?;

        self.transition_commit_protocol_state(CommitProtocolState::Ready);
        self.host_tree = next_host_tree;
        metrics_delta.apply_to(&mut self.metrics);
        if tree_changed {
            self.metrics.contract_tree_materialization_count += 1;
        } else if motion_only_commit_update {
            self.metrics.motion_commit_update_count += 1;
        } else {
            self.metrics.contract_tree_noop_update_count += 1;
        }

        Ok(RenderedJsx {
            tree,
            motion: self.host_tree.motion_frame(),
            logs,
        })
    }

    fn apply_runtime_update_metrics(&mut self, counters: RuntimeHostDebugCounters) {
        self.metrics.render_call_count = counters.render_call_count;
        self.metrics.unmount_count = counters.unmount_count;
    }

    fn sync_recovery_state_from_runtime(&mut self) {
        if let Ok(next_state) = self.runtime.execute_script_as::<RuntimeRecoveryState>(
            "[egui:describe-recovery-state]",
            "globalThis.__eguiDescribeRecoveryStateForTest == null ? { category: null, disposition: 'continue', message: null, component_stack: '', error_boundary: null, rejected_commit_batch_id: null } : globalThis.__eguiDescribeRecoveryStateForTest()",
        ) {
            self.observe_recovery_state(next_state);
        }
    }

    fn observe_recovery_state(&mut self, next_state: RuntimeRecoveryState) {
        if recovery_disposition_priority(next_state.disposition)
            < recovery_disposition_priority(self.recovery_state.disposition)
        {
            return;
        }
        self.recovery_state = next_state;
    }

    fn record_fatal_runtime_error(&mut self, message: impl Into<String>) {
        if self.recovery_state.disposition == RuntimeRecoveryDisposition::Teardown
            && self.recovery_state.message.is_some()
        {
            return;
        }
        self.observe_recovery_state(RuntimeRecoveryState {
            category: Some(RuntimeRecoveryCategory::Fatal),
            disposition: RuntimeRecoveryDisposition::Teardown,
            message: Some(message.into()),
            ..RuntimeRecoveryState::default()
        });
    }

    fn enter_fatal_runtime_failure(&mut self, message: impl Into<String>) {
        let update = self.runtime.take_update();
        self.apply_runtime_update_metrics(update.host_debug_counters);
        self.sync_recovery_state_from_runtime();
        self.record_fatal_runtime_error(message);
    }

    fn ensure_host_recovery_allows_work(&self) -> anyhow::Result<()> {
        match self.recovery_state.disposition {
            RuntimeRecoveryDisposition::Continue | RuntimeRecoveryDisposition::BoundedFailure => {
                Ok(())
            }
            RuntimeRecoveryDisposition::ReloadRequired => bail!(
                "{}",
                self.recovery_state
                    .message
                    .as_deref()
                    .unwrap_or("JSX runtime session requires reload before it can continue.")
            ),
            RuntimeRecoveryDisposition::Teardown => bail!(
                "{}",
                self.recovery_state
                    .message
                    .as_deref()
                    .unwrap_or("JSX runtime session requires teardown before it can continue.")
            ),
        }
    }

    fn transition_commit_protocol_state(&mut self, next_state: CommitProtocolState) {
        if matches!(
            self.commit_protocol_state,
            CommitProtocolState::Failed { .. }
        ) {
            return;
        }
        self.commit_protocol_state = next_state;
    }

    #[cfg(test)]
    fn maybe_fail_post_apply_validation(&self) -> anyhow::Result<()> {
        if self
            .commit_protocol_test_hooks
            .force_post_apply_validation_failure
        {
            bail!("forced post-apply commit validation failure for test");
        }
        Ok(())
    }

    #[cfg(not(test))]
    fn maybe_fail_post_apply_validation(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn acknowledge_commit_batches(&mut self, commit_batch_ids: &[u32]) -> anyhow::Result<()> {
        if commit_batch_ids.is_empty() {
            return Ok(());
        }
        let commit_batch_ids_json = serde_json::to_string(commit_batch_ids)?;
        let source = format!(
            "globalThis.__eguiAcknowledgeCommits == null ? undefined : globalThis.__eguiAcknowledgeCommits({commit_batch_ids_json});"
        );
        self.runtime
            .execute_script("[egui:acknowledge-commits]", source)
            .map_err(|error| {
                anyhow!("failed to acknowledge applied egui commit batches: {error}")
            })?;
        Ok(())
    }

    fn ensure_commit_protocol_healthy(&self) -> anyhow::Result<()> {
        match &self.commit_protocol_state {
            CommitProtocolState::Failed {
                rejected_commit_batch_id: Some(commit_batch_id),
                reason,
            } => bail!(
                "{} entered controlled session failure after commit batch {} was rejected: {}. Reload the session to recover.",
                self.entry_path.display(),
                commit_batch_id,
                reason
            ),
            CommitProtocolState::Failed {
                rejected_commit_batch_id: None,
                reason,
            } => bail!(
                "{} entered controlled session failure after a commit protocol error: {}. Reload the session to recover.",
                self.entry_path.display(),
                reason
            ),
            _ => Ok(()),
        }
    }

    fn reject_commit_batch(&mut self, commit_batch_id: u32, error: anyhow::Error) -> anyhow::Error {
        let reason = error.to_string();
        self.enter_commit_protocol_failure(Some(commit_batch_id), &reason);
        anyhow!(
            "{} rejected commit batch {}: {}",
            self.entry_path.display(),
            commit_batch_id,
            reason
        )
    }

    fn reject_or_fail_commit_protocol(
        &mut self,
        commit_batch_id: Option<u32>,
        error: anyhow::Error,
    ) -> anyhow::Error {
        if let Some(commit_batch_id) = commit_batch_id {
            return self.reject_commit_batch(commit_batch_id, error);
        }
        self.fail_commit_protocol(error)
    }

    fn fail_commit_protocol(&mut self, error: anyhow::Error) -> anyhow::Error {
        let reason = error.to_string();
        self.enter_commit_protocol_failure(None, &reason);
        anyhow!(
            "{} entered controlled session failure due to a commit protocol error: {}",
            self.entry_path.display(),
            reason
        )
    }

    fn enter_commit_protocol_failure(
        &mut self,
        rejected_commit_batch_id: Option<u32>,
        reason: &str,
    ) {
        let first_failure = !matches!(
            self.commit_protocol_state,
            CommitProtocolState::Failed { .. }
        );
        if first_failure {
            self.metrics.commit_protocol_session_failure_count += 1;
            if rejected_commit_batch_id.is_some() {
                self.metrics.commit_protocol_rejected_batch_count += 1;
            }
            self.commit_protocol_state = CommitProtocolState::Failed {
                rejected_commit_batch_id,
                reason: reason.to_owned(),
            };
        }

        let reason_json = serde_json::to_string(reason)
            .unwrap_or_else(|_| "\"commit protocol failure\"".to_owned());
        let source = if let Some(commit_batch_id) = rejected_commit_batch_id {
            format!(
                "globalThis.__eguiRejectCommit == null ? undefined : globalThis.__eguiRejectCommit({commit_batch_id}, {reason_json});"
            )
        } else {
            format!(
                "globalThis.__eguiFailCommitProtocol == null ? undefined : globalThis.__eguiFailCommitProtocol({reason_json});"
            )
        };
        let _ = self
            .runtime
            .execute_script("[egui:commit-protocol-failure]", source);
        self.sync_recovery_state_from_runtime();
        self.observe_recovery_state(RuntimeRecoveryState {
            category: Some(RuntimeRecoveryCategory::ProtocolFailed),
            disposition: RuntimeRecoveryDisposition::ReloadRequired,
            message: Some(match rejected_commit_batch_id {
                Some(commit_batch_id) => format!(
                    "{} entered controlled session failure after commit batch {} was rejected: {}. Reload the session to recover.",
                    self.entry_path.display(),
                    commit_batch_id,
                    reason
                ),
                None => format!(
                    "{} entered controlled session failure after a commit protocol error: {}. Reload the session to recover.",
                    self.entry_path.display(),
                    reason
                ),
            }),
            rejected_commit_batch_id,
            ..RuntimeRecoveryState::default()
        });
        self.runtime.abort_host_runtime();
    }
}

fn egui_runtime_options() -> JsxRuntimeOptions {
    JsxRuntimeOptions::new("egui")
        .with_react_runtime_modules()
        .with_virtual_module("egui", EGUI_MODULE_SOURCE)
        .with_virtual_module("clay", EGUI_MODULE_SOURCE)
        .with_virtual_module("egui/jsx-runtime", EGUI_JSX_RUNTIME_SOURCE)
        .with_virtual_module("clay/jsx-runtime", EGUI_JSX_RUNTIME_SOURCE)
        .with_virtual_module("egui/jsx-dev-runtime", EGUI_JSX_RUNTIME_SOURCE)
        .with_virtual_module("clay/jsx-dev-runtime", EGUI_JSX_RUNTIME_SOURCE)
        .with_virtual_module("clay-internal:/egui-runtime", EGUI_RUNTIME_SOURCE)
        .with_virtual_module("clay-internal:/egui-lowering", EGUI_LOWERING_SOURCE)
        .with_virtual_module("motion/react", EGUI_MOTION_REACT_SOURCE)
        .with_virtual_module("react/motion", EGUI_MOTION_REACT_SOURCE)
}

fn recovery_disposition_priority(disposition: RuntimeRecoveryDisposition) -> u8 {
    match disposition {
        RuntimeRecoveryDisposition::Continue => 0,
        RuntimeRecoveryDisposition::BoundedFailure => 1,
        RuntimeRecoveryDisposition::Teardown => 2,
        RuntimeRecoveryDisposition::ReloadRequired => 3,
    }
}

fn install_contract_metadata(runtime: &mut RuntimeSession) -> anyhow::Result<()> {
    let families = registry()
        .iter()
        .map(|family| family.id.as_str())
        .collect::<Vec<_>>();
    let family_props = registry()
        .iter()
        .map(|family| {
            (
                family.id.as_str(),
                family
                    .props
                    .iter()
                    .map(|prop| prop.name)
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let families_with_children = registry()
        .iter()
        .filter(|family| family.child_policy != ContractChildPolicy::None)
        .map(|family| family.id.as_str())
        .collect::<Vec<_>>();
    let identity_sensitive_families = registry()
        .iter()
        .filter(|family| family.id.is_identity_sensitive())
        .map(|family| family.id.as_str())
        .collect::<Vec<_>>();
    let fallback_node_id_families = registry()
        .iter()
        .filter(|family| family.id.allows_fallback_node_id())
        .map(|family| family.id.as_str())
        .collect::<Vec<_>>();
    let metadata = serde_json::json!({
        "version": CONTRACT_MODEL_VERSION,
        "schema_fingerprint": schema_fingerprint(),
        "families": families,
        "familyProps": family_props,
        "familiesWithChildren": families_with_children,
        "identitySensitiveFamilies": identity_sensitive_families,
        "fallbackNodeIdFamilies": fallback_node_id_families,
    });
    let source = format!(
        "globalThis.__eguiContract = Object.freeze({});",
        serde_json::to_string(&metadata)?
    );
    runtime
        .execute_script("[egui:contract-metadata]", source)
        .map_err(|error| anyhow!("failed to install egui contract metadata: {error}"))?;
    Ok(())
}

fn install_hot_reload_state(
    runtime: &mut RuntimeSession,
    hot_reload_state: Option<&HotReloadState>,
) -> anyhow::Result<()> {
    let Some(hot_reload_state) = hot_reload_state else {
        return Ok(());
    };
    // V1 preserves the input shape for compatibility/debuggability, but the JS
    // runtime deletes and ignores it because reload still remounts cold.
    let source = format!(
        "globalThis.__eguiHotReloadState = {};",
        serde_json::to_string(hot_reload_state)?
    );
    runtime
        .execute_script("[egui:hot-reload-state]", source)
        .map_err(|error| anyhow!("failed to install egui hot reload state: {error}"))?;
    Ok(())
}

fn decode_mutation_batch(
    entry_path: &Path,
    commit_batch: &RuntimeCommitBatch,
) -> anyhow::Result<HostMutationBatch> {
    commit_batch.decode_as().with_context(|| {
        format!(
            "{} returned invalid host mutations through {:?} commit transport",
            entry_path.display(),
            commit_batch.transport_kind()
        )
    })
}

fn validate_contract_tree(entry_path: &Path, tree: &ContractTree) -> anyhow::Result<()> {
    let root_family = tree.root.family_id();
    if !registry().iter().any(|spec| spec.id == root_family) {
        bail!(
            "{} returned unknown root family {}",
            entry_path.display(),
            root_family.as_str()
        );
    }
    validate_contract_node(entry_path, &tree.root)?;

    Ok(())
}

fn validate_contract_node(entry_path: &Path, node: &ContractNode) -> anyhow::Result<()> {
    let family = node.family_id();
    if !registry().iter().any(|spec| spec.id == family) {
        bail!(
            "{} returned unregistered family {} at node {}",
            entry_path.display(),
            family.as_str(),
            node.node_id()
        );
    }

    match node {
        ContractNode::Row(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Column(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Inset(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::SizedBox(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Card(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Sidebar(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Toolbar(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Collapsible(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::DialogueModal(props) => {
            validate_contract_children(entry_path, &props.children)
        }
        ContractNode::Popover(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::ContextMenu(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::AudioPlayback(props) => {
            validate_contract_children(entry_path, &props.children)
        }
        ContractNode::ImageTile(props) => validate_contract_children(entry_path, &props.children),
        ContractNode::Spacer(_)
        | ContractNode::MenuBar(_)
        | ContractNode::Tabs(_)
        | ContractNode::Label(_)
        | ContractNode::Button(_)
        | ContractNode::ButtonGroup(_)
        | ContractNode::Input(_)
        | ContractNode::NumberInput(_)
        | ContractNode::Checkbox(_)
        | ContractNode::Switch(_)
        | ContractNode::Select(_)
        | ContractNode::Field(_)
        | ContractNode::Separator(_)
        | ContractNode::Hierarchy(_)
        | ContractNode::Spinner(_)
        | ContractNode::Progress(_)
        | ContractNode::ToastViewport(_)
        | ContractNode::Color(_)
        | ContractNode::Icon(_)
        | ContractNode::Image(_)
        | ContractNode::Twemoji(_)
        | ContractNode::Kbd(_)
        | ContractNode::Skeleton(_)
        | ContractNode::Slider(_)
        | ContractNode::Radio(_)
        | ContractNode::RadioGroup(_)
        | ContractNode::Combobox(_)
        | ContractNode::EmojiSelector(_)
        | ContractNode::Pagination(_)
        | ContractNode::Tooltip(_)
        | ContractNode::DropdownMenu(_)
        | ContractNode::OpenWith(_)
        | ContractNode::CollabCursor(_)
        | ContractNode::IconToolbar(_)
        | ContractNode::FileTree(_)
        | ContractNode::DragBoard(_)
        | ContractNode::Command(_) => Ok(()),
    }
}

fn validate_contract_children(entry_path: &Path, children: &[ContractNode]) -> anyhow::Result<()> {
    for child in children {
        validate_contract_node(entry_path, child)?;
    }
    Ok(())
}

fn append_tailwind_diagnostics(tree: &ContractTree, logs: &mut clay_jsx_runtime::RuntimeLogBuffer) {
    for diagnostic in audit_tailwind_support(tree) {
        clay_jsx_runtime::push_log(
            logs,
            format!(
                "warn: Tailwind {} at node \"{}\" ({}) for token \"{}\"",
                diagnostic.reason.as_str(),
                diagnostic.node_id,
                diagnostic.family,
                diagnostic.token,
            ),
        );
    }
}

fn absolute_path(path: &Path) -> anyhow::Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("failed to resolve current directory")?
            .join(path)
    };
    Ok(std::fs::canonicalize(&path).unwrap_or(path))
}

fn load_failure(runtime: RuntimeSession, error: anyhow::Error) -> JsxRuntimeLoadOutcome {
    JsxRuntimeLoadOutcome::Failed(JsxRuntimeLoadFailure {
        dependency_paths: runtime.loaded_module_paths(),
        error,
    })
}

fn validate_replacement_load_on_background_thread(
    entry_path: PathBuf,
    hot_reload_state: Option<HotReloadState>,
) -> Result<(), JsxRuntimeLoadFailure> {
    let fallback_entry_path = entry_path.clone();
    let handle = std::thread::Builder::new()
        .name("jsx-runtime-worker-reload-validate".to_owned())
        .spawn(move || {
            match JsxRuntimeSession::load_with_hot_reload_state_outcome(
                &entry_path,
                hot_reload_state.as_ref(),
            ) {
                JsxRuntimeLoadOutcome::Loaded { mut session, .. } => {
                    let _ = session.teardown();
                    Ok(())
                }
                JsxRuntimeLoadOutcome::Failed(failure) => Err(failure),
            }
        })
        .map_err(|error| JsxRuntimeLoadFailure {
            dependency_paths: vec![fallback_entry_path.clone()],
            error: anyhow!("failed to spawn JSX runtime reload validator thread: {error}"),
        })?;

    handle.join().map_err(|panic_payload| {
        let reason = if let Some(reason) = panic_payload.downcast_ref::<&str>() {
            (*reason).to_owned()
        } else if let Some(reason) = panic_payload.downcast_ref::<String>() {
            reason.clone()
        } else {
            "unknown panic payload".to_owned()
        };
        JsxRuntimeLoadFailure {
            dependency_paths: vec![fallback_entry_path],
            error: anyhow!("JSX runtime reload validator panicked: {reason}"),
        }
    })?
}

impl Drop for JsxRuntimeSession {
    fn drop(&mut self) {
        let _ = self.teardown();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;
    use std::time::{Duration, Instant};

    use clay_jsx_runtime::contract::{ContractEvent, ContractNode, EventKind};
    use clay_jsx_runtime::{RuntimeCommitBatch, RuntimeCommitTransportKind, RuntimeSession};
    use serde_json::{json, Value};
    use tempfile::tempdir;

    use super::{
        decode_mutation_batch, egui_runtime_options, install_contract_metadata,
        CommitProtocolState, HostMutationBatch, HostTree, JsxRuntimeDebugMetrics,
        JsxRuntimeSession, JsxRuntimeSessionWorker, JsxRuntimeWorkerEvent,
        JsxRuntimeWorkerReloadOutcome,
    };

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct HostConfigContractSnapshot {
        get_instance_from_node_result: Option<Value>,
        get_instance_from_scope_result: Option<Value>,
        public_host_instance_keys: Vec<String>,
        public_host_instance_kind: String,
        public_host_instance_same_object: bool,
        public_text_instance_keys: Vec<String>,
        public_text_instance_kind: String,
        public_text_instance_same_object: bool,
        supports_hydration: bool,
        supports_microtasks: bool,
        supports_mutation: bool,
        supports_persistence: bool,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct EventHandlerRegistrySnapshot {
        key_counts: BTreeMap<String, usize>,
        keys: Vec<String>,
        registration_slot_count: usize,
        unique_registration_count: usize,
    }

    #[test]
    fn mutation_queue_backpressure_is_bounded_and_logged_in_bridge_session() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("queue-backpressure.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [active, setActive] = useState(false);
  return (
    <div id="root" data-slot="column">
      <button
        id="toggle"
        label="Toggle"
        onClick={() => setActive((value) => !value)}
      />
      <label id="status" text={active ? "on" : "off"} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let baseline_metrics = session.debug_metrics();

        session
            .runtime
            .execute_script(
                "[test:bridge-mutation-queue-stall]",
                r#"
globalThis.__bridgeMutationFloodAccepted = 0;
globalThis.__bridgeMutationFloodOverflow = "";
const version = globalThis.__eguiContract?.version ?? 1;
const schemaFingerprint = globalThis.__eguiContract?.schema_fingerprint ?? "";
const batchJson = JSON.stringify({ version, schema_fingerprint: schemaFingerprint, mutations: [] });
for (let index = 0; index < 4096; index += 1) {
  try {
    Deno.core.ops.op_commit_mutations(batchJson);
    globalThis.__bridgeMutationFloodAccepted += 1;
  } catch (error) {
    globalThis.__bridgeMutationFloodOverflow = error instanceof Error ? error.message : String(error);
    break;
  }
}
"#,
            )
            .expect("flooding dispatches without draining should complete");

        let accepted = session
            .runtime
            .execute_json_expression_as::<u64>(
                "[test:bridge-mutation-queue-stall-accepted]",
                "globalThis.__bridgeMutationFloodAccepted",
            )
            .expect("accepted flood count should deserialize");
        let overflow = session
            .runtime
            .execute_json_expression_as::<String>(
                "[test:bridge-mutation-queue-stall-overflow]",
                "globalThis.__bridgeMutationFloodOverflow",
            )
            .expect("overflow reason should deserialize");
        assert!(
            overflow.contains("runtime mutation queue overflow"),
            "expected queue overflow during stalled drains, got: {overflow}"
        );

        let pre_drain_metrics = session.debug_metrics();
        let pre_drain_runtime = &pre_drain_metrics.runtime;
        assert_eq!(accepted, pre_drain_runtime.pending_mutation_batch_limit);
        assert_eq!(
            pre_drain_runtime.pending_mutation_batch_count,
            pre_drain_runtime.pending_mutation_batch_limit
        );
        assert_eq!(pre_drain_runtime.pending_mutation_batch_overflow_count, 1);

        let rendered = session.runtime.take_update();
        assert!(
            rendered
                .logs
                .iter()
                .any(|entry| entry.contains("runtime mutation queue overflow")),
            "expected overflow logging when stalled updates are drained"
        );
        assert_eq!(rendered.commit_batches.len(), accepted as usize);

        let post_drain_metrics = session.debug_metrics();
        let post_drain_runtime = &post_drain_metrics.runtime;
        assert_eq!(post_drain_runtime.pending_mutation_batch_count, 0);
        assert_eq!(post_drain_runtime.pending_mutation_batch_bytes, 0);
        assert_eq!(
            post_drain_runtime.pending_mutation_batch_high_water,
            post_drain_runtime.pending_mutation_batch_limit
        );
        assert!(
            post_drain_runtime.pending_mutation_batch_bytes_high_water
                <= post_drain_runtime.pending_mutation_batch_byte_limit,
            "mutation queue byte high-water should stay within the configured limit"
        );
        assert_eq!(post_drain_runtime.pending_mutation_batch_overflow_count, 1);
        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle", EventKind::Clicked)])
            .expect("bridge should remain interactive after overflow recovery");
        let tree = rendered
            .tree
            .expect("toggle dispatch should materialize an updated tree");
        assert_eq!(status_label_text(&tree.root), Some("on"));

        let final_metrics = session.debug_metrics();
        assert_eq!(final_metrics.runtime.pending_mutation_batch_count, 0);
        assert_eq!(
            final_metrics.runtime.pending_mutation_batch_overflow_count,
            1
        );
        assert_eq!(
            final_metrics.mutation_batch_count - baseline_metrics.mutation_batch_count,
            1,
            "only real bridge-applied batches should count toward host-tree mutation metrics"
        );
    }

    #[test]
    fn host_config_contract_freezes_mutation_mode_and_shadow_public_instances() {
        let mut runtime = load_host_config_test_runtime();
        let snapshot = runtime
            .execute_script_as::<HostConfigContractSnapshot>(
                "[test:describe-host-config]",
                "globalThis.__eguiDescribeHostConfigForTest();",
            )
            .expect("host-config contract snapshot should deserialize");

        assert!(
            snapshot.supports_mutation,
            "v1 host-config must stay in mutation mode"
        );
        assert!(
            snapshot.supports_microtasks,
            "v1 host-config must continue to support microtask scheduling"
        );
        assert!(
            !snapshot.supports_hydration,
            "hydration must remain explicitly unsupported in v1"
        );
        assert!(
            !snapshot.supports_persistence,
            "persistence mode must remain explicitly unsupported in v1"
        );
        assert_eq!(snapshot.get_instance_from_node_result, None);
        assert_eq!(snapshot.get_instance_from_scope_result, None);

        assert!(
            snapshot.public_host_instance_same_object,
            "getPublicInstance() must return the same JS shadow host object"
        );
        assert_eq!(snapshot.public_host_instance_kind, "host");
        assert!(
            snapshot
                .public_host_instance_keys
                .iter()
                .any(|key| key == "kind"),
            "public host instance should expose JS shadow-node fields"
        );
        assert!(
            !snapshot
                .public_host_instance_keys
                .iter()
                .any(|key| { key.contains("native") || key == "nativeHandle" || key == "ptr" }),
            "public host instance must not expose a native renderer handle"
        );

        assert!(
            snapshot.public_text_instance_same_object,
            "getPublicInstance() must return the same JS shadow text object"
        );
        assert_eq!(snapshot.public_text_instance_kind, "text");
        assert!(
            snapshot
                .public_text_instance_keys
                .iter()
                .any(|key| key == "kind"),
            "public text instance should expose JS shadow-node fields"
        );

        runtime.shutdown_host_runtime();
    }

    #[test]
    fn host_config_create_append_insert_remove_reorder_hide_unhide_emit_frozen_mutations() {
        let mut runtime = load_host_config_test_runtime();
        let mut host_tree = HostTree::default();

        runtime
            .execute_script(
                "[test:host-config-initial-create-append]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
const root = host.createInstance("div", { id: "root", "data-slot": "column" });
const alpha = host.createInstance("label", { id: "alpha", text: "Alpha" });
const gamma = host.createInstance("label", { id: "gamma", text: "Gamma" });
globalThis.__hostConfigRoot = root;
globalThis.__hostConfigAlpha = alpha;
globalThis.__hostConfigGamma = gamma;
host.appendInitialChild(root, alpha);
host.appendInitialChild(root, gamma);
host.prepareForCommit(container);
host.appendChildToContainer(container, root);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("initial create/append host-config script should execute");
        let initial_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&initial_batches),
            vec!["replace_root"]
        );
        apply_runtime_commit_batches(&mut host_tree, &initial_batches);
        assert_eq!(
            column_child_node_ids(&host_tree.materialize().unwrap().root),
            ["alpha", "gamma"]
        );

        runtime
            .execute_script(
                "[test:host-config-append-child]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
const root = globalThis.__hostConfigRoot;
const delta = host.createInstance("label", { id: "delta", text: "Delta" });
globalThis.__hostConfigDelta = delta;
host.prepareForCommit(container);
host.appendChild(root, delta);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("appendChild host-config script should execute");
        let append_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&append_batches),
            vec!["insert_subtree"]
        );
        apply_runtime_commit_batches(&mut host_tree, &append_batches);
        assert_eq!(
            column_child_node_ids(&host_tree.materialize().unwrap().root),
            ["alpha", "gamma", "delta"]
        );

        runtime
            .execute_script(
                "[test:host-config-insert-before]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
const root = globalThis.__hostConfigRoot;
const gamma = globalThis.__hostConfigGamma;
const beta = host.createInstance("label", { id: "beta", text: "Beta" });
globalThis.__hostConfigBeta = beta;
host.prepareForCommit(container);
host.insertBefore(root, beta, gamma);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("insertBefore host-config script should execute");
        let insert_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&insert_batches),
            vec!["insert_subtree"]
        );
        apply_runtime_commit_batches(&mut host_tree, &insert_batches);
        assert_eq!(
            column_child_node_ids(&host_tree.materialize().unwrap().root),
            ["alpha", "beta", "gamma", "delta"]
        );

        runtime
            .execute_script(
                "[test:host-config-remove-child]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
const root = globalThis.__hostConfigRoot;
const beta = globalThis.__hostConfigBeta;
host.prepareForCommit(container);
host.removeChild(root, beta);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("removeChild host-config script should execute");
        let remove_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&remove_batches),
            vec!["remove_subtree"]
        );
        apply_runtime_commit_batches(&mut host_tree, &remove_batches);
        assert_eq!(
            column_child_node_ids(&host_tree.materialize().unwrap().root),
            ["alpha", "gamma", "delta"]
        );

        runtime
            .execute_script(
                "[test:host-config-reorder-existing-child]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
const root = globalThis.__hostConfigRoot;
const delta = globalThis.__hostConfigDelta;
const alpha = globalThis.__hostConfigAlpha;
host.prepareForCommit(container);
host.insertBefore(root, delta, alpha);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("reorder host-config script should execute");
        let reorder_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&reorder_batches),
            vec!["set_children"]
        );
        apply_runtime_commit_batches(&mut host_tree, &reorder_batches);
        assert_eq!(
            column_child_node_ids(&host_tree.materialize().unwrap().root),
            ["delta", "alpha", "gamma"]
        );

        runtime
            .execute_script(
                "[test:host-config-hide-instance]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
const gamma = globalThis.__hostConfigGamma;
host.prepareForCommit(container);
host.hideInstance(gamma);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("hideInstance host-config script should execute");
        let hide_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&hide_batches),
            vec!["remove_subtree"]
        );
        apply_runtime_commit_batches(&mut host_tree, &hide_batches);
        assert_eq!(
            column_child_node_ids(&host_tree.materialize().unwrap().root),
            ["delta", "alpha"]
        );

        runtime
            .execute_script(
                "[test:host-config-unhide-instance]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
const gamma = globalThis.__hostConfigGamma;
host.prepareForCommit(container);
host.unhideInstance(gamma);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("unhideInstance host-config script should execute");
        let unhide_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&unhide_batches),
            vec!["insert_subtree"]
        );
        apply_runtime_commit_batches(&mut host_tree, &unhide_batches);
        assert_eq!(
            column_child_node_ids(&host_tree.materialize().unwrap().root),
            ["delta", "alpha", "gamma"]
        );

        runtime.shutdown_host_runtime();
    }

    #[test]
    fn host_config_create_text_and_commit_text_update_emit_frozen_mutations() {
        let mut runtime = load_host_config_test_runtime();
        let mut host_tree = HostTree::default();

        runtime
            .execute_script(
                "[test:host-config-create-text]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
const root = host.createInstance("div", { id: "root", "data-slot": "column" });
const text = host.createTextInstance("Hello");
globalThis.__hostConfigTextRoot = root;
globalThis.__hostConfigTextNode = text;
host.appendInitialChild(root, text);
host.prepareForCommit(container);
host.appendChildToContainer(container, root);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("createTextInstance host-config script should execute");
        let initial_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&initial_batches),
            vec!["replace_root"]
        );
        apply_runtime_commit_batches(&mut host_tree, &initial_batches);
        assert_eq!(
            column_child_label_texts(&host_tree.materialize().unwrap().root),
            ["Hello"]
        );

        runtime
            .execute_script(
                "[test:host-config-commit-text-update]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
const text = globalThis.__hostConfigTextNode;
host.prepareForCommit(container);
host.commitTextUpdate(text, "Hello", "World");
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("commitTextUpdate host-config script should execute");
        let text_update_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&text_update_batches),
            vec!["update_node"]
        );
        apply_runtime_commit_batches(&mut host_tree, &text_update_batches);
        assert_eq!(
            column_child_label_texts(&host_tree.materialize().unwrap().root),
            ["World"]
        );

        runtime.shutdown_host_runtime();
    }

    #[test]
    fn event_handler_routing_supports_node_action_and_wildcard_routes_without_duplicate_dispatch() {
        let mut runtime = load_host_config_test_runtime();

        runtime
            .execute_script(
                "[test:event-routing-setup]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
globalThis.__nodeOnlyCount = 0;
globalThis.__actionOnlyCount = 0;
globalThis.__wildcardCount = 0;
globalThis.__dedupeCount = 0;

const root = host.createInstance("div", { id: "root", "data-slot": "column" });
const nodeOnly = host.createInstance("button", {
  id: "node-only",
  label: "Node Only",
  onClick: () => {
    globalThis.__nodeOnlyCount += 1;
  },
});
const actionOnly = host.createInstance("button", {
  id: "action-node",
  actionId: "action.only",
  label: "Action Only",
  onClick: () => {
    globalThis.__actionOnlyCount += 1;
  },
});
const wildcard = host.createInstance("button", {
  id: "wildcard-node",
  label: "Wildcard",
  onEvent: () => {
    globalThis.__wildcardCount += 1;
  },
});
const dedupe = host.createInstance("button", {
  id: "dedupe-node",
  actionId: "action.dedupe",
  label: "Dedupe",
  onClick: () => {
    globalThis.__dedupeCount += 1;
  },
});

host.appendInitialChild(root, nodeOnly);
host.appendInitialChild(root, actionOnly);
host.appendInitialChild(root, wildcard);
host.appendInitialChild(root, dedupe);

host.prepareForCommit(container);
host.appendChildToContainer(container, root);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("event routing setup script should execute");
        let initial_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&initial_batches),
            vec!["replace_root"]
        );

        let snapshot = describe_event_handler_registry(&mut runtime);
        assert_eq!(snapshot.unique_registration_count, 4);
        assert_eq!(snapshot.registration_slot_count, 6);
        assert_eq!(snapshot.key_counts.get("node-only:clicked"), Some(&1));
        assert_eq!(snapshot.key_counts.get("action-node:clicked"), Some(&1));
        assert_eq!(
            snapshot.key_counts.get("action:action.only:clicked"),
            Some(&1)
        );
        assert_eq!(snapshot.key_counts.get("wildcard-node:*"), Some(&1));
        assert_eq!(snapshot.key_counts.get("dedupe-node:clicked"), Some(&1));
        assert_eq!(
            snapshot.key_counts.get("action:action.dedupe:clicked"),
            Some(&1)
        );

        dispatch_test_event(
            &mut runtime,
            json!({ "node_id": "node-only", "kind": "clicked" }),
        );
        dispatch_test_event(
            &mut runtime,
            json!({
                "node_id": "missing-node-route",
                "action_id": "action.only",
                "kind": "clicked"
            }),
        );
        dispatch_test_event(
            &mut runtime,
            json!({ "node_id": "wildcard-node", "kind": "toggled" }),
        );
        dispatch_test_event(
            &mut runtime,
            json!({
                "node_id": "dedupe-node",
                "action_id": "action.dedupe",
                "kind": "clicked"
            }),
        );

        assert_eq!(
            runtime
                .execute_json_expression_as::<u64>(
                    "[test:event-routing-node-count]",
                    "globalThis.__nodeOnlyCount",
                )
                .expect("node-only count should deserialize"),
            1
        );
        assert_eq!(
            runtime
                .execute_json_expression_as::<u64>(
                    "[test:event-routing-action-count]",
                    "globalThis.__actionOnlyCount",
                )
                .expect("action-only count should deserialize"),
            1
        );
        assert_eq!(
            runtime
                .execute_json_expression_as::<u64>(
                    "[test:event-routing-wildcard-count]",
                    "globalThis.__wildcardCount",
                )
                .expect("wildcard count should deserialize"),
            1
        );
        assert_eq!(
            runtime
                .execute_json_expression_as::<u64>(
                    "[test:event-routing-dedupe-count]",
                    "globalThis.__dedupeCount",
                )
                .expect("dedupe count should deserialize"),
            1,
            "one logical handler registration must not run twice when node and action routes both match"
        );

        runtime.shutdown_host_runtime();
    }

    #[test]
    fn event_handler_registrations_are_removed_on_clear_replace_hide_and_unmount() {
        let mut runtime = load_host_config_test_runtime();

        runtime
            .execute_script(
                "[test:event-handler-removal-setup]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
globalThis.__clearCount = 0;
globalThis.__replaceOldCount = 0;
globalThis.__replaceNewCount = 0;
globalThis.__hideCount = 0;

const root = host.createInstance("div", { id: "root", "data-slot": "column" });
const clearParent = host.createInstance("div", { id: "clear-parent", "data-slot": "column" });
const clearChild = host.createInstance("button", {
  id: "clear-child",
  label: "Clear Child",
  onClick: () => {
    globalThis.__clearCount += 1;
  },
});
const replaceTarget = host.createInstance("button", {
  id: "replace-old",
  actionId: "action.replace.old",
  label: "Replace Old",
  onClick: () => {
    globalThis.__replaceOldCount += 1;
  },
});
const hideTarget = host.createInstance("button", {
  id: "hide-target",
  actionId: "action.hide",
  label: "Hide Target",
  onClick: () => {
    globalThis.__hideCount += 1;
  },
});

globalThis.__clearParent = clearParent;
globalThis.__replaceTarget = replaceTarget;
globalThis.__hideTarget = hideTarget;

host.appendInitialChild(clearParent, clearChild);
host.appendInitialChild(root, clearParent);
host.appendInitialChild(root, replaceTarget);
host.appendInitialChild(root, hideTarget);

host.prepareForCommit(container);
host.appendChildToContainer(container, root);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("event handler removal setup script should execute");
        let initial_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&initial_batches),
            vec!["replace_root"]
        );

        let initial_snapshot = describe_event_handler_registry(&mut runtime);
        assert!(
            initial_snapshot
                .keys
                .contains(&"clear-child:clicked".to_owned()),
            "initial routing table should include the clear child handler"
        );
        assert!(
            initial_snapshot
                .keys
                .contains(&"replace-old:clicked".to_owned()),
            "initial routing table should include the replace target node route"
        );
        assert!(
            initial_snapshot
                .keys
                .contains(&"action:action.replace.old:clicked".to_owned()),
            "initial routing table should include the replace target action route"
        );
        assert!(
            initial_snapshot
                .keys
                .contains(&"hide-target:clicked".to_owned()),
            "initial routing table should include the hide target route"
        );

        dispatch_test_event(
            &mut runtime,
            json!({ "node_id": "clear-child", "kind": "clicked" }),
        );
        assert_eq!(
            runtime
                .execute_json_expression_as::<u64>(
                    "[test:event-handler-removal-clear-before]",
                    "globalThis.__clearCount",
                )
                .expect("clear count should deserialize"),
            1
        );

        runtime
            .execute_script(
                "[test:event-handler-removal-clear-subtree]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
const clearParent = globalThis.__clearParent;
host.prepareForCommit(container);
host.resetTextContent(clearParent);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("clear subtree script should execute");
        let clear_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&clear_batches),
            vec!["remove_subtree"]
        );
        let after_clear = describe_event_handler_registry(&mut runtime);
        assert!(
            !after_clear.keys.contains(&"clear-child:clicked".to_owned()),
            "subtree clear must remove child handler registrations"
        );
        dispatch_test_event(
            &mut runtime,
            json!({ "node_id": "clear-child", "kind": "clicked" }),
        );
        assert_eq!(
            runtime
                .execute_json_expression_as::<u64>(
                    "[test:event-handler-removal-clear-after]",
                    "globalThis.__clearCount",
                )
                .expect("clear count should deserialize after removal"),
            1,
            "cleared subtrees must not retain stale event handlers"
        );

        runtime
            .execute_script(
                "[test:event-handler-removal-replace]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
const replaceTarget = globalThis.__replaceTarget;
host.prepareForCommit(container);
host.commitUpdate(
  replaceTarget,
  "button",
  replaceTarget.props,
  {
    id: "replace-new",
    actionId: "action.replace.new",
    label: "Replace New",
    onClick: () => {
      globalThis.__replaceNewCount += 1;
    },
  },
);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("replace script should execute");
        let replace_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&replace_batches),
            vec!["replace_subtree"]
        );
        let after_replace = describe_event_handler_registry(&mut runtime);
        assert!(
            !after_replace
                .keys
                .contains(&"replace-old:clicked".to_owned()),
            "replaced nodes must remove their old node-id route"
        );
        assert!(
            !after_replace
                .keys
                .contains(&"action:action.replace.old:clicked".to_owned()),
            "replaced nodes must remove their old action-id route"
        );
        assert!(
            after_replace
                .keys
                .contains(&"replace-new:clicked".to_owned()),
            "replacement node should install its new node-id route"
        );
        assert!(
            after_replace
                .keys
                .contains(&"action:action.replace.new:clicked".to_owned()),
            "replacement node should install its new action-id route"
        );
        dispatch_test_event(
            &mut runtime,
            json!({
                "node_id": "replace-old",
                "action_id": "action.replace.old",
                "kind": "clicked"
            }),
        );
        dispatch_test_event(
            &mut runtime,
            json!({
                "node_id": "replace-new",
                "action_id": "action.replace.new",
                "kind": "clicked"
            }),
        );
        assert_eq!(
            runtime
                .execute_json_expression_as::<u64>(
                    "[test:event-handler-removal-replace-old-count]",
                    "globalThis.__replaceOldCount",
                )
                .expect("old replace count should deserialize"),
            0,
            "old replacement routes must not stay active after a subtree replace"
        );
        assert_eq!(
            runtime
                .execute_json_expression_as::<u64>(
                    "[test:event-handler-removal-replace-new-count]",
                    "globalThis.__replaceNewCount",
                )
                .expect("new replace count should deserialize"),
            1,
            "replacement routes should dispatch exactly once"
        );

        dispatch_test_event(
            &mut runtime,
            json!({
                "node_id": "hide-target",
                "action_id": "action.hide",
                "kind": "clicked"
            }),
        );
        assert_eq!(
            runtime
                .execute_json_expression_as::<u64>(
                    "[test:event-handler-removal-hide-before]",
                    "globalThis.__hideCount",
                )
                .expect("hide count should deserialize"),
            1
        );

        runtime
            .execute_script(
                "[test:event-handler-removal-hide]",
                r#"
(() => {
const host = globalThis.__eguiHostConfigForTest;
const container = globalThis.__eguiContainerForTest;
const hideTarget = globalThis.__hideTarget;
host.prepareForCommit(container);
host.hideInstance(hideTarget);
host.resetAfterCommit(container);
})();
"#,
            )
            .expect("hide script should execute");
        let hide_batches = runtime.take_update().commit_batches;
        assert_eq!(
            mutation_kinds_in_batches(&hide_batches),
            vec!["remove_subtree"]
        );
        let after_hide = describe_event_handler_registry(&mut runtime);
        assert!(
            !after_hide.keys.contains(&"hide-target:clicked".to_owned()),
            "hidden subtrees must remove node-id handler routes"
        );
        assert!(
            !after_hide
                .keys
                .contains(&"action:action.hide:clicked".to_owned()),
            "hidden subtrees must remove action-id handler routes"
        );
        dispatch_test_event(
            &mut runtime,
            json!({
                "node_id": "hide-target",
                "action_id": "action.hide",
                "kind": "clicked"
            }),
        );
        assert_eq!(
            runtime
                .execute_json_expression_as::<u64>(
                    "[test:event-handler-removal-hide-after]",
                    "globalThis.__hideCount",
                )
                .expect("hide count should deserialize after removal"),
            1,
            "hidden subtrees must not retain stale event handlers"
        );

        runtime
            .execute_script(
                "[test:event-handler-removal-unmount]",
                "globalThis.__eguiUnmountRuntime();",
            )
            .expect("unmount script should execute");
        let _ = runtime.take_update();
        let after_unmount = describe_event_handler_registry(&mut runtime);
        assert_eq!(after_unmount.keys, Vec::<String>::new());
        assert_eq!(after_unmount.registration_slot_count, 0);
        assert_eq!(after_unmount.unique_registration_count, 0);
        dispatch_test_event(
            &mut runtime,
            json!({
                "node_id": "replace-new",
                "action_id": "action.replace.new",
                "kind": "clicked"
            }),
        );
        assert_eq!(
            runtime
                .execute_json_expression_as::<u64>(
                    "[test:event-handler-removal-unmount-count]",
                    "globalThis.__replaceNewCount",
                )
                .expect("replacement count should deserialize after unmount"),
            1,
            "unmounted runtimes must clear all event handler registrations"
        );

        runtime.shutdown_host_runtime();
    }

    #[test]
    fn failed_runtime_batch_sequence_rolls_back_host_tree_state_byte_for_byte() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return (
    <div id="root" data-slot="column">
      <button
        id="trigger"
        label="Trigger"
        onClick={() => {
          const version = globalThis.__eguiContract?.version ?? 1;
          const schemaFingerprint = globalThis.__eguiContract?.schema_fingerprint ?? "";
          const validBatch = {
            version,
            schema_fingerprint: schemaFingerprint,
            mutations: [
              {
                kind: "update_node",
                node: { family: "label", node_id: "status", text: "mutated" },
              },
            ],
          };
          const invalidBatch = {
            version,
            schema_fingerprint: schemaFingerprint,
            mutations: [{ kind: "remove_subtree", node_id: "missing" }],
          };
          // @ts-ignore
          Deno.core.ops.op_commit_mutations(JSON.stringify(validBatch));
          // @ts-ignore
          Deno.core.ops.op_commit_mutations(JSON.stringify(invalidBatch));
        }}
      />
      <label id="status" text="stable" />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let initial_tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(status_label_text(&initial_tree.root), Some("stable"));

        let checkpoint = host_tree_snapshot_bytes(&session);
        let error = session
            .dispatch_events(&[ContractEvent::new("trigger", EventKind::Clicked)])
            .expect_err("invalid batch sequence should fail");
        assert!(
            error.to_string().contains("missing subtree root missing"),
            "unexpected dispatch error: {error:#}"
        );

        assert_eq!(host_tree_snapshot_bytes(&session), checkpoint);
        let restored_tree = session
            .host_tree
            .materialize()
            .expect("host tree should remain valid after rollback");
        assert_eq!(status_label_text(&restored_tree.root), Some("stable"));
    }

    #[test]
    fn decode_failure_rejects_the_session_atomically() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("decode-failure.tsx");
        write_commit_protocol_failure_fixture(
            &entry_path,
            r#"
// @ts-ignore
Deno.core.ops.op_commit_mutations("not valid json");
"#,
        );

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("fixture should load successfully");

        assert_failed_commit_is_session_atomic(
            &mut session,
            "returned invalid host mutations",
            Some(2),
        );
    }

    #[test]
    fn version_mismatch_rejects_the_session_atomically() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("version-mismatch.tsx");
        write_commit_protocol_failure_fixture(
            &entry_path,
            r#"
const invalidBatch = {
  version: version + 1,
  schema_fingerprint: schemaFingerprint,
  mutations: [
    {
      kind: "update_node",
      node: { family: "label", node_id: "status", text: "mutated" },
    },
  ],
};
// @ts-ignore
Deno.core.ops.op_commit_mutations(JSON.stringify(invalidBatch));
"#,
        );

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("fixture should load successfully");

        assert_failed_commit_is_session_atomic(
            &mut session,
            "returned contract model version",
            Some(2),
        );
    }

    #[test]
    fn schema_mismatch_rejects_the_session_atomically() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("schema-mismatch.tsx");
        write_commit_protocol_failure_fixture(
            &entry_path,
            r#"
const invalidBatch = {
  version,
  schema_fingerprint: "schema-mismatch-for-test",
  mutations: [
    {
      kind: "update_node",
      node: { family: "label", node_id: "status", text: "mutated" },
    },
  ],
};
// @ts-ignore
Deno.core.ops.op_commit_mutations(JSON.stringify(invalidBatch));
"#,
        );

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("fixture should load successfully");

        assert_failed_commit_is_session_atomic(
            &mut session,
            "returned contract schema fingerprint",
            Some(2),
        );
    }

    #[test]
    fn invalid_mutations_reject_the_session_atomically() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("invalid-mutation.tsx");
        write_commit_protocol_failure_fixture(
            &entry_path,
            r#"
const invalidBatch = {
  version,
  schema_fingerprint: schemaFingerprint,
  mutations: [{ kind: "remove_subtree", node_id: "missing" }],
};
// @ts-ignore
Deno.core.ops.op_commit_mutations(JSON.stringify(invalidBatch));
"#,
        );

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("fixture should load successfully");

        assert_failed_commit_is_session_atomic(
            &mut session,
            "missing subtree root missing",
            Some(2),
        );
    }

    #[test]
    fn post_apply_validation_failure_rejects_the_session_atomically() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("post-apply-validation-failure.tsx");
        write_commit_protocol_failure_fixture(
            &entry_path,
            r#"
const invalidBatch = {
  version,
  schema_fingerprint: schemaFingerprint,
  mutations: [
    {
      kind: "update_node",
      node: { family: "label", node_id: "status", text: "mutated" },
    },
  ],
};
// @ts-ignore
Deno.core.ops.op_commit_mutations(JSON.stringify(invalidBatch));
"#,
        );

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("fixture should load successfully");
        session
            .commit_protocol_test_hooks
            .force_post_apply_validation_failure = true;

        assert_failed_commit_is_session_atomic(
            &mut session,
            "forced post-apply commit validation failure for test",
            Some(2),
        );
    }

    #[test]
    fn non_motion_root_commit_omits_redundant_clear_motion_mutations() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("non-motion-root.tsx");
        write_non_motion_fixture(&entry_path, 320);

        let optimized_batch_json = capture_replace_root_commit_batch(&entry_path, None)
            .payload_json()
            .expect("replace_root batch should remain serializable as JSON");
        let batch_value: Value = serde_json::from_str(&optimized_batch_json)
            .expect("runtime commit batch should be valid JSON");
        let mutations = batch_value
            .get("mutations")
            .and_then(Value::as_array)
            .expect("commit batch should expose mutations array");

        let replace_root_count = mutation_kind_count(mutations, "replace_root");
        let set_motion_count = mutation_kind_count(mutations, "set_motion");
        let clear_motion_count = mutation_kind_count(mutations, "clear_motion");

        assert_eq!(replace_root_count, 1);
        assert_eq!(set_motion_count, 0);
        assert_eq!(clear_motion_count, 0);
    }

    #[test]
    fn motion_removal_emits_exactly_one_required_clear_motion_without_redundant_clears() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("motion-clear-diff.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";
import { motion } from "motion/react";

function App() {
  const [animated, setAnimated] = useState(true);

  return (
    <div id="root" data-slot="column">
      <button id="toggle" label="Toggle" onClick={() => setAnimated(false)} />
      <motion.label
        id="status"
        text="Status"
        animate={animated ? { opacity: 1, x: 0 } : undefined}
        transition={{ duration: 1, ease: "linear" }}
      />
      <label id="static" text="Static" />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx fixture should be written");

        let mut runtime =
            RuntimeSession::new(egui_runtime_options()).expect("runtime session should initialize");
        install_contract_metadata(&mut runtime).expect("contract metadata should install");
        runtime
            .load_main_module(&entry_path)
            .expect("fixture module should load");

        let initial_batches = runtime.take_update().commit_batches;
        assert_eq!(
            count_mutation_kind_in_batches(&initial_batches, "clear_motion"),
            0,
            "initial render should not emit redundant clear_motion operations"
        );
        assert_eq!(
            count_mutation_kind_for_node_in_batches(&initial_batches, "set_motion", "status"),
            1,
            "initial render should emit one set_motion for the animated status node"
        );

        let dispatch_batches = dispatch_events_and_take_commit_batches(
            &mut runtime,
            &[ContractEvent::new("toggle", EventKind::Clicked)],
        );

        assert_eq!(
            count_mutation_kind_for_node_in_batches(&dispatch_batches, "clear_motion", "status"),
            1,
            "motion removal should emit one clear_motion for status"
        );
        assert_eq!(
            count_mutation_kind_in_batches(&dispatch_batches, "clear_motion"),
            1,
            "motion removal diff should not emit redundant clear_motion operations"
        );

        runtime.shutdown_host_runtime();
    }

    #[test]
    fn typed_commit_transport_requires_explicit_test_opt_in() {
        let mut runtime = load_host_config_test_runtime();

        let error = runtime
            .execute_script(
                "[test:typed-transport-without-opt-in]",
                "globalThis.__eguiSetCommitTransportModeForTest('typed');",
            )
            .expect_err("typed commit transport should stay deferred by default in v1");
        let error_text = format!("{error:#}");
        assert!(
            error_text.contains("Typed egui commit transport is deferred in v1"),
            "unexpected typed transport deferral error: {error_text}"
        );

        enable_experimental_typed_commit_transport(&mut runtime);
        let active_mode = runtime
            .execute_script_as::<String>(
                "[test:typed-transport-with-opt-in]",
                "globalThis.__eguiSetCommitTransportModeForTest('typed');\n globalThis.__eguiGetCommitTransportModeForTest();",
            )
            .expect("typed commit transport should become available after explicit opt-in");
        assert_eq!(active_mode, "typed");
    }

    #[test]
    fn typed_commit_transport_spike_preserves_bridge_batch_semantics() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("typed-commit-transport.tsx");
        write_non_motion_fixture(&entry_path, 24);

        let json_batch = capture_replace_root_commit_batch(&entry_path, None);
        let typed_batch = capture_replace_root_commit_batch(&entry_path, Some("typed"));

        assert_eq!(
            json_batch.transport_kind(),
            RuntimeCommitTransportKind::Json
        );
        assert_eq!(
            typed_batch.transport_kind(),
            RuntimeCommitTransportKind::Typed
        );

        assert_eq!(
            json_batch
                .decode_as::<Value>()
                .expect("json transport batch should decode into a JSON value"),
            typed_batch
                .decode_as::<Value>()
                .expect("typed transport batch should decode into a JSON value"),
            "typed transport should preserve the committed batch shape"
        );

        let json_decoded = decode_mutation_batch(&entry_path, &json_batch)
            .expect("json transport batch should decode");
        let typed_decoded = decode_mutation_batch(&entry_path, &typed_batch)
            .expect("typed transport batch should decode");
        assert_eq!(
            apply_batch_and_snapshot(&json_decoded),
            apply_batch_and_snapshot(&typed_decoded),
            "typed transport must preserve retained host-tree semantics"
        );
    }

    #[test]
    #[ignore = "performance benchmark; excluded from default test path because wall-clock improvement assertions are host-dependent"]
    fn clear_motion_elision_benchmark_reduces_payload_and_apply_latency() {
        const LABEL_COUNT: usize = 1200;
        const BENCH_SAMPLES: usize = 7;
        const ITERATIONS_PER_SAMPLE: usize = 32;

        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("non-motion-benchmark.tsx");
        write_non_motion_fixture(&entry_path, LABEL_COUNT);

        let optimized_batch_json = capture_replace_root_commit_batch(&entry_path, None)
            .payload_json()
            .expect("replace_root batch should remain serializable as JSON");
        let optimized_batch_value: Value = serde_json::from_str(&optimized_batch_json)
            .expect("optimized batch should deserialize");
        let replace_root_subtree = replace_root_subtree(&optimized_batch_value)
            .expect("replace_root batch should include subtree");

        let mut non_motion_node_ids = Vec::new();
        collect_subtree_node_ids(replace_root_subtree, &mut non_motion_node_ids);
        assert!(
            !non_motion_node_ids.is_empty(),
            "benchmark fixture should contain lowerable nodes"
        );

        let mut legacy_batch_value = optimized_batch_value.clone();
        let legacy_mutations = legacy_batch_value
            .get_mut("mutations")
            .and_then(Value::as_array_mut)
            .expect("legacy batch should expose mutable mutations array");
        for node_id in &non_motion_node_ids {
            legacy_mutations.push(json!({
                "kind": "clear_motion",
                "node_id": node_id,
            }));
        }
        let legacy_batch_json =
            serde_json::to_string(&legacy_batch_value).expect("legacy batch should serialize");

        let optimized_batch: HostMutationBatch =
            serde_json::from_str(&optimized_batch_json).expect("optimized batch should decode");
        let legacy_batch: HostMutationBatch =
            serde_json::from_str(&legacy_batch_json).expect("legacy batch should decode");

        let optimized_payload_bytes = optimized_batch_json.len();
        let legacy_payload_bytes = legacy_batch_json.len();
        assert!(
            optimized_payload_bytes < legacy_payload_bytes,
            "expected optimized payload ({optimized_payload_bytes} bytes) to be smaller than legacy baseline ({legacy_payload_bytes} bytes)"
        );

        assert_eq!(
            apply_batch_and_snapshot(&optimized_batch),
            apply_batch_and_snapshot(&legacy_batch),
            "clear-motion elision must preserve retained host-tree state"
        );

        let optimized_apply = benchmark_batch_apply_only_median_duration(
            &optimized_batch,
            BENCH_SAMPLES,
            ITERATIONS_PER_SAMPLE,
        );
        let legacy_apply = benchmark_batch_apply_only_median_duration(
            &legacy_batch,
            BENCH_SAMPLES,
            ITERATIONS_PER_SAMPLE,
        );

        let legacy_ns = legacy_apply.as_nanos();
        let optimized_ns = optimized_apply.as_nanos();
        let improvement = if legacy_ns > 0 {
            ((legacy_ns.saturating_sub(optimized_ns) as f64) * 100.0) / legacy_ns as f64
        } else {
            0.0
        };

        println!(
            "clear-motion elision apply-only benchmark (non-motion screen, labels={LABEL_COUNT}, median of {BENCH_SAMPLES} samples × {ITERATIONS_PER_SAMPLE} apply iterations): payload baseline={legacy_payload_bytes}B, optimized={optimized_payload_bytes}B, apply baseline={legacy_apply:?}, optimized={optimized_apply:?}, improvement={improvement:.2}%"
        );

        assert!(
            optimized_apply < legacy_apply,
            "expected optimized apply latency ({optimized_apply:?}) to beat legacy baseline ({legacy_apply:?})"
        );
    }

    #[test]
    #[ignore = "performance benchmark; excluded from default test path because wall-clock measurements are host-dependent"]
    fn motion_only_commit_benchmark_is_separate_from_structural_commit_benchmarks() {
        const MOTION_LABEL_COUNT: usize = 720;
        const COMMITS_PER_SAMPLE: usize = 4;
        const WARMUP_SAMPLES: usize = 12;
        const BENCH_SAMPLES: usize = 48;

        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("motion-heavy-benchmark.tsx");
        write_motion_heavy_fixture(&entry_path, MOTION_LABEL_COUNT);

        let samples = benchmark_motion_only_commit_samples(
            &entry_path,
            MOTION_LABEL_COUNT,
            COMMITS_PER_SAMPLE,
            WARMUP_SAMPLES,
            BENCH_SAMPLES,
        );
        let p95 = percentile_duration(&samples, 0.95);

        println!(
            "motion-only commit benchmark (labels={MOTION_LABEL_COUNT}, commits/sample={COMMITS_PER_SAMPLE}, warmup={WARMUP_SAMPLES}, samples={BENCH_SAMPLES}, total commits={}): p95={p95:?}",
            samples.len() * COMMITS_PER_SAMPLE
        );

        assert_eq!(samples.len(), BENCH_SAMPLES);
    }

    #[test]
    #[ignore = "performance benchmark; excluded from default test path because wall-clock improvement assertions are host-dependent"]
    fn indexed_parent_child_mapping_benchmark_improves_deep_reorder_commit_p95() {
        const REORDER_DEPTH: usize = 6;
        const REORDER_SIBLINGS: usize = 720;
        const COMMITS_PER_SAMPLE: usize = 4;
        const WARMUP_SAMPLES: usize = 20;
        const BENCH_SAMPLES: usize = 96;

        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("deep-reorder-benchmark.tsx");
        write_deep_reorder_fixture(&entry_path, REORDER_DEPTH, REORDER_SIBLINGS);

        let legacy_samples = benchmark_deep_reorder_commit_samples(
            &entry_path,
            false,
            COMMITS_PER_SAMPLE,
            WARMUP_SAMPLES,
            BENCH_SAMPLES,
        );
        let indexed_samples = benchmark_deep_reorder_commit_samples(
            &entry_path,
            true,
            COMMITS_PER_SAMPLE,
            WARMUP_SAMPLES,
            BENCH_SAMPLES,
        );

        let legacy_p95 = percentile_duration(&legacy_samples, 0.95);
        let indexed_p95 = percentile_duration(&indexed_samples, 0.95);
        let legacy_ns = legacy_p95.as_nanos();
        let indexed_ns = indexed_p95.as_nanos();
        let improvement = if legacy_ns > 0 {
            ((legacy_ns.saturating_sub(indexed_ns) as f64) * 100.0) / legacy_ns as f64
        } else {
            0.0
        };

        println!(
            "deep-reorder indexed parent-child commit-only benchmark (depth={REORDER_DEPTH}, siblings={REORDER_SIBLINGS}, commits/sample={COMMITS_PER_SAMPLE}, warmup={WARMUP_SAMPLES}, samples={BENCH_SAMPLES}, total commits/mode={}): legacy p95={legacy_p95:?}, indexed p95={indexed_p95:?}, improvement={improvement:.2}%",
            legacy_samples.len() * COMMITS_PER_SAMPLE
        );

        assert!(
            indexed_p95 < legacy_p95,
            "expected indexed parent-child mapping p95 commit latency ({indexed_p95:?}) to beat sibling-scan baseline ({legacy_p95:?})"
        );
    }

    #[test]
    #[ignore = "performance benchmark; excluded from default test path because wall-clock improvement assertions are host-dependent"]
    fn session_worker_thread_benchmark_improves_ui_responsiveness_under_burst_dispatch() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("burst.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function burn(ms) {
  const deadline = Date.now() + ms;
  while (Date.now() < deadline) {
    // Busy loop to simulate heavier runtime work per dispatch.
  }
}

function App() {
  const [count, setCount] = useState(0);
  return (
    <button
      id="burst"
      label={String(count)}
      onClick={() => {
        burn(8);
        setCount((value) => value + 1);
      }}
    />
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        const FRAME_SAMPLES: usize = 48;
        let burst_event = ContractEvent::new("burst", EventKind::Clicked);

        let (mut sync_session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("sync session should load");
        let mut sync_frame_costs = Vec::with_capacity(FRAME_SAMPLES);
        for _ in 0..FRAME_SAMPLES {
            let frame_start = Instant::now();
            sync_session
                .dispatch_events(&[burst_event.clone()])
                .expect("sync dispatch should complete");
            sync_frame_costs.push(frame_start.elapsed());
        }

        let worker = JsxRuntimeSessionWorker::spawn(None).expect("worker session should spawn");
        worker
            .request_reload(&entry_path)
            .expect("worker reload should queue");
        wait_for_worker_reload(&worker, Duration::from_secs(15));

        let mut worker_frame_costs = Vec::with_capacity(FRAME_SAMPLES);
        for _ in 0..FRAME_SAMPLES {
            let frame_start = Instant::now();
            worker
                .request_dispatch_events(vec![burst_event.clone()])
                .expect("worker dispatch should queue");
            drain_worker_events_non_blocking(&worker);
            worker_frame_costs.push(frame_start.elapsed());
        }
        wait_for_worker_dispatches(&worker, FRAME_SAMPLES, Duration::from_secs(30));

        let sync_p90 = percentile_duration(&sync_frame_costs, 0.90);
        let worker_p90 = percentile_duration(&worker_frame_costs, 0.90);
        println!("session worker benchmark: sync p90={sync_p90:?}, worker p90={worker_p90:?}");
        assert!(
            sync_p90 >= Duration::from_millis(6),
            "expected synchronous frame cost to include burst runtime work, observed p90={sync_p90:?}"
        );
        assert!(
            worker_p90.as_nanos().saturating_mul(3) < sync_p90.as_nanos(),
            "expected worker-thread frame p90 {worker_p90:?} to be meaningfully below synchronous p90 {sync_p90:?}"
        );
    }

    fn status_label_text(root: &ContractNode) -> Option<&str> {
        let ContractNode::Column(props) = root else {
            return None;
        };
        props
            .children
            .iter()
            .find(|child| child.node_id().as_str() == "status")
            .and_then(|node| match node {
                ContractNode::Label(props) => Some(props.text.as_str()),
                _ => None,
            })
    }

    fn load_host_config_test_runtime() -> RuntimeSession {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("host-config-contract.ts");
        std::fs::write(&entry_path, "import \"egui\";\n")
            .expect("host-config fixture should be written");

        let mut runtime =
            RuntimeSession::new(egui_runtime_options()).expect("runtime session should initialize");
        install_contract_metadata(&mut runtime).expect("contract metadata should install");
        runtime
            .load_main_module(&entry_path)
            .expect("host-config fixture should load");
        let update = runtime.take_update();
        assert!(
            update.commit_batches.is_empty(),
            "loading the host-config contract fixture should not emit commit batches"
        );
        runtime
    }

    fn describe_event_handler_registry(
        runtime: &mut RuntimeSession,
    ) -> EventHandlerRegistrySnapshot {
        runtime
            .execute_script_as::<EventHandlerRegistrySnapshot>(
                "[test:describe-event-handler-registry]",
                "globalThis.__eguiDescribeEventHandlerRegistryForTest();",
            )
            .expect("event handler registry snapshot should deserialize")
    }

    fn dispatch_test_event(runtime: &mut RuntimeSession, event: Value) {
        let event_json = serde_json::to_string(&event).expect("test event should serialize");
        let source = format!("globalThis.__eguiDispatchEventForTest({event_json});");
        runtime
            .execute_script("[test:dispatch-event-for-test]", source)
            .expect("test event dispatch should execute");
    }

    fn mutation_kinds_in_batches(commit_batches: &[RuntimeCommitBatch]) -> Vec<String> {
        let mut kinds = Vec::new();
        for commit_batch in commit_batches {
            let batch: Value = commit_batch
                .decode_as()
                .expect("runtime commit batch should deserialize");
            let Some(mutations) = batch.get("mutations").and_then(Value::as_array) else {
                continue;
            };
            for mutation in mutations {
                let kind = mutation
                    .get("kind")
                    .and_then(Value::as_str)
                    .expect("runtime mutation should expose a string kind");
                kinds.push(kind.to_owned());
            }
        }
        kinds
    }

    fn apply_runtime_commit_batches(
        host_tree: &mut HostTree,
        commit_batches: &[RuntimeCommitBatch],
    ) {
        for commit_batch in commit_batches {
            let batch = decode_mutation_batch(Path::new("[host-config-contract]"), commit_batch)
                .expect("runtime commit batch should decode through the transport abstraction");
            host_tree
                .apply_mutations_in_place(batch.mutations)
                .expect("decoded host-config batch should apply cleanly");
        }
    }

    fn column_child_node_ids(root: &ContractNode) -> Vec<&str> {
        let ContractNode::Column(props) = root else {
            panic!("expected a column root contract node");
        };
        props
            .children
            .iter()
            .map(|child| child.node_id().as_str())
            .collect()
    }

    fn column_child_label_texts(root: &ContractNode) -> Vec<&str> {
        let ContractNode::Column(props) = root else {
            panic!("expected a column root contract node");
        };
        props
            .children
            .iter()
            .map(|child| match child {
                ContractNode::Label(props) => props.text.as_str(),
                _ => panic!("expected a lowered label child"),
            })
            .collect()
    }

    fn write_commit_protocol_failure_fixture(entry_path: &Path, on_click_body: &str) {
        let source = format!(
            r#"
import {{ render }} from "egui";

function App() {{
  return (
    <div id="root" data-slot="column">
      <button
        id="trigger"
        label="Trigger"
        onClick={{() => {{
          const version = globalThis.__eguiContract?.version ?? 1;
          const schemaFingerprint = globalThis.__eguiContract?.schema_fingerprint ?? "";
          {on_click_body}
        }}}}
      />
      <label id="status" text="stable" />
    </div>
  );
}}

render(<App />);
"#,
        );
        std::fs::write(entry_path, source).expect("commit protocol fixture should be written");
    }

    fn assert_failed_commit_is_session_atomic(
        session: &mut JsxRuntimeSession,
        expected_error_substring: &str,
        expected_rejected_commit_batch_id: Option<u32>,
    ) {
        let metrics_before = session.debug_metrics();
        let checkpoint = host_tree_snapshot_bytes(session);

        let error = session
            .dispatch_events(&[ContractEvent::new("trigger", EventKind::Clicked)])
            .expect_err("commit protocol violation should fail the dispatch atomically");
        let error_string = error.to_string();
        assert!(
            error_string.contains(expected_error_substring),
            "unexpected commit protocol error: {error:#}"
        );

        assert_eq!(host_tree_snapshot_bytes(session), checkpoint);
        let restored_tree = session
            .host_tree
            .materialize()
            .expect("host tree should remain materializable after rejected commit");
        assert_eq!(status_label_text(&restored_tree.root), Some("stable"));

        let metrics_after = session.debug_metrics();
        assert_metrics_unchanged_except_commit_failure_counters(&metrics_before, &metrics_after);

        match session.debug_commit_protocol_state() {
            CommitProtocolState::Failed {
                rejected_commit_batch_id,
                reason,
            } => {
                assert_eq!(rejected_commit_batch_id, expected_rejected_commit_batch_id);
                assert!(
                    reason.contains(expected_error_substring),
                    "unexpected commit protocol failure reason: {reason}"
                );
            }
            state => panic!("expected failed commit protocol state, got {state:?}"),
        }

        let follow_up_error = session
            .dispatch_events(&[ContractEvent::new("trigger", EventKind::Clicked)])
            .expect_err("failed sessions should remain in controlled failure mode");
        assert!(
            follow_up_error
                .to_string()
                .contains("entered controlled session failure"),
            "expected controlled-failure guard after rejected commit, got: {follow_up_error:#}"
        );
    }

    fn assert_metrics_unchanged_except_commit_failure_counters(
        before: &JsxRuntimeDebugMetrics,
        after: &JsxRuntimeDebugMetrics,
    ) {
        assert_eq!(
            after.commit_protocol_rejected_batch_count,
            before.commit_protocol_rejected_batch_count + 1,
            "rejected batch counter should advance exactly once"
        );
        assert_eq!(
            after.commit_protocol_session_failure_count,
            before.commit_protocol_session_failure_count + 1,
            "session failure counter should advance exactly once"
        );

        let mut normalized_after = after.clone();
        normalized_after.commit_protocol_rejected_batch_count =
            before.commit_protocol_rejected_batch_count;
        normalized_after.commit_protocol_session_failure_count =
            before.commit_protocol_session_failure_count;
        assert_eq!(
            &normalized_after, before,
            "only commit protocol failure counters should change after a rejected batch"
        );
        assert_eq!(
            after.runtime.shutdown_count, before.runtime.shutdown_count,
            "rejected commits must not change exposed runtime shutdown metrics"
        );
    }

    fn host_tree_snapshot_bytes(session: &JsxRuntimeSession) -> Vec<u8> {
        session.host_tree.retained_state_snapshot_bytes()
    }

    fn write_non_motion_fixture(entry_path: &Path, label_count: usize) {
        let source = format!(
            r#"
import {{ render }} from "egui";

function App() {{
  const children = [];
  for (let index = 0; index < {label_count}; index += 1) {{
    children.push(<label key={{index}} id={{`item.${{index}}`}} text={{`Item ${{index}}`}} />);
  }}

  return (
    <div id="root" data-slot="column">
      {{children}}
    </div>
  );
}}

render(<App />);
"#,
        );
        std::fs::write(entry_path, source).expect("tsx fixture should be written");
    }

    fn write_motion_heavy_fixture(entry_path: &Path, label_count: usize) {
        let source = format!(
            r#"
import {{ render }} from "egui";
import {{ motion }} from "motion/react";

function appTree(open) {{
  const children = [];
  for (let index = 0; index < {label_count}; index += 1) {{
    children.push(
      <motion.label
        key={{index}}
        id={{`status.${{index}}`}}
        text={{`Status ${{index}}`}}
        animate={{{{ opacity: open ? 1 : 0, x: open ? 0 : -10 }}}}
        transition={{{{ duration: 1, ease: "linear" }}}}
      />,
    );
  }}

  return (
    <div id="root" data-slot="column">
      {{children}}
    </div>
  );
}}

let open = false;
globalThis.__eguiCommitBenchmarkRenderStepForTest = () => {{
  open = !open;
  return appTree(open);
}};

render(appTree(open));
"#,
        );
        std::fs::write(entry_path, source).expect("motion-heavy tsx fixture should be written");
    }

    fn write_deep_reorder_fixture(entry_path: &Path, depth: usize, sibling_count: usize) {
        let source = format!(
            r#"
import {{ render }} from "egui";

function nestedContent(depth, child) {{
  let node = child;
  for (let level = 0; level < depth; level += 1) {{
    node = (
      <div id={{`layer.${{level}}`}} data-slot="column">
        {{node}}
      </div>
    );
  }}
  return node;
}}

function reorderParent(swapTail, revision) {{
  const items = [];
  for (let index = 0; index < {sibling_count}; index += 1) {{
    let itemId = index;
    if (swapTail && index === {sibling_count} - 2) {{
      itemId = {sibling_count} - 1;
    }} else if (swapTail && index === {sibling_count} - 1) {{
      itemId = {sibling_count} - 2;
    }}

    items.push(
      <label
        key={{itemId}}
        id={{`item.${{itemId}}`}}
        text={{`r${{revision}} item ${{itemId}}`}}
      />,
    );
  }}

  return (
    <div id="reorder-parent" data-slot="column">
      {{items}}
    </div>
  );
}}

function appTree(swapTail, revision) {{
  return (
    <div id="root" data-slot="column">
      <label id="mode" text={{`${{swapTail ? "swap-tail" : "stable-tail"}}:${{revision}}`}} />
      {{nestedContent(
        {depth},
        reorderParent(swapTail, revision),
      )}}
    </div>
  );
}}

let swapTail = false;
let revision = 0;
globalThis.__eguiCommitBenchmarkRenderStepForTest = () => {{
  swapTail = !swapTail;
  revision += 1;
  return appTree(swapTail, revision);
}};

render(appTree(swapTail, revision));
"#,
        );

        std::fs::write(entry_path, source).expect("deep reorder tsx fixture should be written");
    }

    fn benchmark_deep_reorder_commit_samples(
        entry_path: &Path,
        indexed_mapping_enabled: bool,
        commits_per_sample: usize,
        warmup_samples: usize,
        measured_samples: usize,
    ) -> Vec<Duration> {
        assert!(
            measured_samples > 0,
            "benchmark sample count should be non-zero"
        );
        assert!(
            commits_per_sample > 0,
            "benchmark commits-per-sample should be non-zero"
        );

        let mut runtime =
            RuntimeSession::new(egui_runtime_options()).expect("runtime session should initialize");
        install_contract_metadata(&mut runtime).expect("contract metadata should install");
        runtime
            .load_main_module(entry_path)
            .expect("deep reorder benchmark fixture should load");
        let _ = runtime.take_update();

        let mode = if indexed_mapping_enabled {
            "true"
        } else {
            "false"
        };
        runtime
            .execute_script(
                "[test:set-indexed-parent-child-mapping-mode]",
                format!("globalThis.__eguiSetIndexedParentChildMappingEnabledForTest({mode});"),
            )
            .expect("benchmark should set indexed parent-child mapping mode");

        let benchmark_expression =
            format!("globalThis.__eguiBenchmarkCommitOnlyForTest({commits_per_sample})");
        for _ in 0..warmup_samples {
            let warmup_commit_durations_ms = runtime
                .execute_json_expression_as::<Vec<f64>>(
                    "[test:deep-reorder-warmup-commit-only]",
                    &benchmark_expression,
                )
                .expect("deep reorder warmup commit benchmark should execute");
            assert_eq!(
                warmup_commit_durations_ms.len(),
                commits_per_sample,
                "warmup commit benchmark should return one duration per commit"
            );

            let warmup_update = runtime.take_update();
            assert!(
                !warmup_update.commit_batches.is_empty(),
                "deep reorder warmup commit benchmark should emit mutation batches (durations={warmup_commit_durations_ms:?}, logs={:?})",
                warmup_update.logs
            );
        }

        let mut samples = Vec::with_capacity(measured_samples);
        for _ in 0..measured_samples {
            let commit_durations_ms = runtime
                .execute_json_expression_as::<Vec<f64>>(
                    "[test:deep-reorder-commit-only]",
                    &benchmark_expression,
                )
                .expect("deep reorder commit benchmark should execute");
            assert!(
                commit_durations_ms.len() == commits_per_sample,
                "deep reorder commit benchmark should return one duration per commit"
            );

            let update = runtime.take_update();
            assert!(
                !update.commit_batches.is_empty(),
                "deep reorder commit benchmark should emit mutation batches (durations={commit_durations_ms:?}, logs={:?})",
                update.logs
            );

            let mut total_elapsed_ms = 0.0_f64;
            for elapsed_ms in commit_durations_ms {
                assert!(
                    elapsed_ms.is_finite() && elapsed_ms >= 0.0,
                    "deep reorder commit benchmark returned invalid duration {elapsed_ms}"
                );
                total_elapsed_ms += elapsed_ms;
            }

            let average_elapsed_ms = total_elapsed_ms / commits_per_sample as f64;
            samples.push(Duration::from_secs_f64(average_elapsed_ms / 1_000.0));
        }

        runtime.shutdown_host_runtime();
        samples
    }

    fn benchmark_motion_only_commit_samples(
        entry_path: &Path,
        motion_label_count: usize,
        commits_per_sample: usize,
        warmup_samples: usize,
        measured_samples: usize,
    ) -> Vec<Duration> {
        assert!(
            measured_samples > 0,
            "benchmark sample count should be non-zero"
        );
        assert!(
            commits_per_sample > 0,
            "benchmark commits-per-sample should be non-zero"
        );

        let mut runtime =
            RuntimeSession::new(egui_runtime_options()).expect("runtime session should initialize");
        install_contract_metadata(&mut runtime).expect("contract metadata should install");
        runtime
            .load_main_module(entry_path)
            .expect("motion-heavy benchmark fixture should load");
        let _ = runtime.take_update();

        let benchmark_expression =
            format!("globalThis.__eguiBenchmarkCommitOnlyForTest({commits_per_sample})");
        for _ in 0..warmup_samples {
            let warmup_commit_durations_ms = runtime
                .execute_json_expression_as::<Vec<f64>>(
                    "[test:motion-heavy-warmup-commit-only]",
                    &benchmark_expression,
                )
                .expect("motion-heavy warmup benchmark should execute");
            assert_eq!(
                warmup_commit_durations_ms.len(),
                commits_per_sample,
                "motion-heavy warmup benchmark should return one duration per commit"
            );

            let warmup_update = runtime.take_update();
            assert_motion_only_commit_batches(
                &warmup_update.commit_batches,
                motion_label_count * commits_per_sample,
            );
        }

        let mut samples = Vec::with_capacity(measured_samples);
        for _ in 0..measured_samples {
            let commit_durations_ms = runtime
                .execute_json_expression_as::<Vec<f64>>(
                    "[test:motion-heavy-commit-only]",
                    &benchmark_expression,
                )
                .expect("motion-heavy benchmark should execute");
            assert_eq!(
                commit_durations_ms.len(),
                commits_per_sample,
                "motion-heavy benchmark should return one duration per commit"
            );

            let update = runtime.take_update();
            assert_motion_only_commit_batches(
                &update.commit_batches,
                motion_label_count * commits_per_sample,
            );

            let mut total_elapsed_ms = 0.0_f64;
            for elapsed_ms in commit_durations_ms {
                assert!(
                    elapsed_ms.is_finite() && elapsed_ms >= 0.0,
                    "motion-heavy benchmark returned invalid duration {elapsed_ms}"
                );
                total_elapsed_ms += elapsed_ms;
            }

            let average_elapsed_ms = total_elapsed_ms / commits_per_sample as f64;
            samples.push(Duration::from_secs_f64(average_elapsed_ms / 1_000.0));
        }

        runtime.shutdown_host_runtime();
        samples
    }

    fn capture_replace_root_commit_batch(
        entry_path: &Path,
        transport_mode: Option<&str>,
    ) -> RuntimeCommitBatch {
        let mut runtime =
            RuntimeSession::new(egui_runtime_options()).expect("runtime session should initialize");
        install_contract_metadata(&mut runtime).expect("contract metadata should install");
        if let Some(mode) = transport_mode {
            set_commit_transport_mode(&mut runtime, mode);
        }
        runtime
            .load_main_module(entry_path)
            .expect("fixture module should load");
        let update = runtime.take_update();
        runtime.shutdown_host_runtime();

        let mut replace_root_batch = None;
        for commit_batch in update.commit_batches {
            let value: Value = commit_batch
                .decode_as()
                .expect("runtime commit batch should decode into JSON value");
            let has_replace_root = value
                .get("mutations")
                .and_then(Value::as_array)
                .map(|mutations| {
                    mutations.iter().any(|mutation| {
                        mutation
                            .get("kind")
                            .and_then(Value::as_str)
                            .is_some_and(|kind| kind == "replace_root")
                    })
                })
                .unwrap_or(false);
            if has_replace_root {
                replace_root_batch = Some(commit_batch);
            }
        }

        replace_root_batch.expect("runtime should emit a replace_root commit batch")
    }

    fn mutation_kind_count(mutations: &[Value], kind: &str) -> usize {
        mutations
            .iter()
            .filter(|mutation| {
                mutation
                    .get("kind")
                    .and_then(Value::as_str)
                    .is_some_and(|value| value == kind)
            })
            .count()
    }

    fn count_mutation_kind_in_batches(commit_batches: &[RuntimeCommitBatch], kind: &str) -> usize {
        commit_batches
            .iter()
            .map(|commit_batch| {
                let batch: Value = commit_batch
                    .decode_as()
                    .expect("runtime commit batch should deserialize");
                batch
                    .get("mutations")
                    .and_then(Value::as_array)
                    .map(|mutations| mutation_kind_count(mutations, kind))
                    .unwrap_or(0)
            })
            .sum()
    }

    fn count_mutation_kind_for_node_in_batches(
        commit_batches: &[RuntimeCommitBatch],
        kind: &str,
        node_id: &str,
    ) -> usize {
        commit_batches
            .iter()
            .map(|commit_batch| {
                let batch: Value = commit_batch
                    .decode_as()
                    .expect("runtime commit batch should deserialize");
                batch
                    .get("mutations")
                    .and_then(Value::as_array)
                    .map(|mutations| {
                        mutations
                            .iter()
                            .filter(|mutation| {
                                mutation
                                    .get("kind")
                                    .and_then(Value::as_str)
                                    .is_some_and(|value| value == kind)
                                    && mutation
                                        .get("node_id")
                                        .and_then(Value::as_str)
                                        .is_some_and(|value| value == node_id)
                            })
                            .count()
                    })
                    .unwrap_or(0)
            })
            .sum()
    }

    fn assert_motion_only_commit_batches(
        commit_batches: &[RuntimeCommitBatch],
        expected_set_motion_count: usize,
    ) {
        assert!(
            !commit_batches.is_empty(),
            "motion-only benchmark should emit commit batches"
        );
        assert_eq!(
            count_mutation_kind_in_batches(commit_batches, "set_motion"),
            expected_set_motion_count,
            "motion-only benchmark should retarget every animated node once per commit"
        );
        assert_eq!(
            count_mutation_kind_in_batches(commit_batches, "clear_motion"),
            0,
            "motion-only benchmark should not clear retained motion state"
        );
        for structural_kind in [
            "replace_root",
            "insert_subtree",
            "remove_subtree",
            "replace_subtree",
            "update_node",
            "set_children",
        ] {
            assert_eq!(
                count_mutation_kind_in_batches(commit_batches, structural_kind),
                0,
                "motion-only benchmark should not emit structural mutations of kind {structural_kind}"
            );
        }
    }

    fn replace_root_subtree(batch: &Value) -> Option<&Value> {
        let mutations = batch.get("mutations")?.as_array()?;
        for mutation in mutations {
            if mutation
                .get("kind")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == "replace_root")
            {
                return mutation.get("subtree");
            }
        }
        None
    }

    fn collect_subtree_node_ids(node: &Value, output: &mut Vec<String>) {
        if let Some(node_id) = node.get("node_id").and_then(Value::as_str) {
            output.push(node_id.to_owned());
        }

        if let Some(children) = node.get("children").and_then(Value::as_array) {
            for child in children {
                collect_subtree_node_ids(child, output);
            }
        }
    }

    fn apply_batch_and_snapshot(batch: &HostMutationBatch) -> Vec<u8> {
        let mut tree = HostTree::default();
        tree.apply_mutations_in_place(batch.mutations.clone())
            .expect("benchmark batch should apply cleanly");
        tree.retained_state_snapshot_bytes()
    }

    fn benchmark_batch_apply_only_median_duration(
        batch: &HostMutationBatch,
        samples: usize,
        iterations_per_sample: usize,
    ) -> Duration {
        assert!(samples > 0, "benchmark samples should be non-zero");
        assert!(
            iterations_per_sample > 0,
            "benchmark iterations should be non-zero"
        );

        let mut sample_durations = Vec::with_capacity(samples);
        for _ in 0..samples {
            let mut apply_duration = Duration::ZERO;
            for _ in 0..iterations_per_sample {
                let mut tree = HostTree::default();
                let mutations = batch.mutations.clone();
                let start = Instant::now();
                tree.apply_mutations_in_place(mutations)
                    .expect("benchmark batch should apply cleanly");
                apply_duration += start.elapsed();
            }
            sample_durations.push(apply_duration);
        }

        sample_durations.sort_unstable();
        sample_durations[sample_durations.len() / 2]
    }

    fn dispatch_events_and_take_commit_batches(
        runtime: &mut RuntimeSession,
        events: &[ContractEvent],
    ) -> Vec<RuntimeCommitBatch> {
        let events_json = serde_json::to_string(events).expect("events should serialize");
        let source = format!("globalThis.__eguiDispatchEvents({events_json});");
        runtime
            .execute_script("[test:dispatch-events]", source)
            .expect("dispatch script should execute");
        runtime
            .execute_script("[test:post-dispatch-flush]", "undefined;")
            .expect("post-dispatch flush should execute");
        let _ = runtime
            .drain_host_callbacks()
            .expect("host callbacks should drain after dispatch");
        runtime.take_update().commit_batches
    }

    fn set_commit_transport_mode(runtime: &mut RuntimeSession, mode: &str) {
        let mode_json = serde_json::to_string(mode).expect("transport mode should serialize");
        let source = format!(
            "globalThis.__eguiAllowExperimentalCommitTransportForTest = true;\nglobalThis.__eguiCommitTransportModeForTest = {mode_json};\nif (typeof globalThis.__eguiSetCommitTransportModeForTest === 'function') {{\n  globalThis.__eguiSetCommitTransportModeForTest({mode_json});\n}}"
        );
        runtime
            .execute_script("[test:set-commit-transport-mode]", source)
            .expect("commit transport mode should be configurable for tests");
    }

    fn enable_experimental_typed_commit_transport(runtime: &mut RuntimeSession) {
        runtime
            .execute_script(
                "[test:enable-experimental-typed-transport]",
                "globalThis.__eguiAllowExperimentalCommitTransportForTest = true;",
            )
            .expect("typed transport test opt-in should be configurable");
    }

    fn wait_for_worker_reload(worker: &JsxRuntimeSessionWorker, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            match worker
                .try_recv_event()
                .expect("worker reload polling should succeed")
            {
                Some(JsxRuntimeWorkerEvent::ReloadCompleted { outcome, .. }) => match outcome {
                    JsxRuntimeWorkerReloadOutcome::Loaded { .. } => return,
                    JsxRuntimeWorkerReloadOutcome::Failed(failure) => {
                        panic!("worker reload failed during benchmark: {:#}", failure.error)
                    }
                },
                Some(_) => continue,
                None => std::thread::sleep(Duration::from_millis(1)),
            }
        }

        panic!("timed out waiting for worker reload benchmark initialization");
    }

    fn wait_for_worker_dispatches(
        worker: &JsxRuntimeSessionWorker,
        expected_dispatches: usize,
        timeout: Duration,
    ) {
        let mut completed = 0usize;
        let deadline = Instant::now() + timeout;
        while completed < expected_dispatches && Instant::now() < deadline {
            match worker
                .try_recv_event()
                .expect("worker dispatch polling should succeed")
            {
                Some(JsxRuntimeWorkerEvent::DispatchCompleted { result, .. }) => {
                    result.expect("worker dispatch should complete successfully");
                    completed += 1;
                }
                Some(_) => continue,
                None => std::thread::sleep(Duration::from_millis(1)),
            }
        }

        assert_eq!(
            completed, expected_dispatches,
            "worker dispatch benchmark should complete all queued dispatches before timeout"
        );
    }

    fn drain_worker_events_non_blocking(worker: &JsxRuntimeSessionWorker) {
        loop {
            match worker
                .try_recv_event()
                .expect("worker event drain should succeed")
            {
                Some(JsxRuntimeWorkerEvent::DispatchCompleted { result, .. }) => {
                    result.expect("worker dispatch should complete successfully");
                }
                Some(_) => continue,
                None => break,
            }
        }
    }

    fn percentile_duration(samples: &[Duration], percentile: f64) -> Duration {
        assert!(
            !samples.is_empty(),
            "percentile samples should not be empty"
        );
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let index = ((sorted.len() - 1) as f64 * percentile)
            .round()
            .clamp(0.0, (sorted.len() - 1) as f64) as usize;
        sorted[index]
    }
}
