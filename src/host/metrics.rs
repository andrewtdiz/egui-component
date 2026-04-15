use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

use clay_jsx_egui_bridge::JsxRuntimeDebugMetrics;
use egui::Context;

#[derive(Debug, Clone, Default)]
pub struct ExampleHostMetricsTracker {
    inner: Arc<ExampleHostMetricsInner>,
}

#[derive(Debug, Default)]
struct ExampleHostMetricsInner {
    reload_attempt_count: AtomicU64,
    reload_success_count: AtomicU64,
    reload_failure_count: AtomicU64,
    reload_recovery_count: AtomicU64,
    repaint_request_count: AtomicU64,
    runtime_update_drain_count: AtomicU64,
    runtime_update_drain_empty_count: AtomicU64,
    last_reload_failed: AtomicBool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExampleHostMetrics {
    pub reload_attempt_count: u64,
    pub reload_success_count: u64,
    pub reload_failure_count: u64,
    pub reload_recovery_count: u64,
    pub repaint_request_count: u64,
    pub runtime_update_drain_count: u64,
    pub runtime_update_drain_empty_count: u64,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExampleHostDebugSnapshot {
    pub host: ExampleHostMetrics,
    pub live_session: Option<JsxRuntimeDebugMetrics>,
    pub last_torn_down_session: Option<JsxRuntimeDebugMetrics>,
}

impl ExampleHostMetricsTracker {
    pub fn note_reload_attempt(&self) {
        self.inner
            .reload_attempt_count
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn note_reload_success(&self) {
        self.inner
            .reload_success_count
            .fetch_add(1, Ordering::Relaxed);
        if self.inner.last_reload_failed.swap(false, Ordering::SeqCst) {
            self.inner
                .reload_recovery_count
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn note_reload_failure(&self) {
        self.inner
            .reload_failure_count
            .fetch_add(1, Ordering::Relaxed);
        self.inner.last_reload_failed.store(true, Ordering::SeqCst);
    }

    pub fn note_repaint_request(&self) {
        self.inner
            .repaint_request_count
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn note_runtime_update_drain(&self) {
        self.inner
            .runtime_update_drain_count
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn note_runtime_update_drain_empty(&self) {
        self.inner
            .runtime_update_drain_empty_count
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> ExampleHostMetrics {
        ExampleHostMetrics {
            reload_attempt_count: self.inner.reload_attempt_count.load(Ordering::Relaxed),
            reload_success_count: self.inner.reload_success_count.load(Ordering::Relaxed),
            reload_failure_count: self.inner.reload_failure_count.load(Ordering::Relaxed),
            reload_recovery_count: self.inner.reload_recovery_count.load(Ordering::Relaxed),
            repaint_request_count: self.inner.repaint_request_count.load(Ordering::Relaxed),
            runtime_update_drain_count: self
                .inner
                .runtime_update_drain_count
                .load(Ordering::Relaxed),
            runtime_update_drain_empty_count: self
                .inner
                .runtime_update_drain_empty_count
                .load(Ordering::Relaxed),
        }
    }
}

pub fn request_host_repaint(ctx: &Context, metrics: &ExampleHostMetricsTracker) {
    metrics.note_repaint_request();
    ctx.request_repaint();
}
