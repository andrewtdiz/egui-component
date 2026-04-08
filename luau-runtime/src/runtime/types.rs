use std::{collections::BTreeSet, error::Error, fmt, path::PathBuf, sync::Arc, time::Duration};

use super::graph::ReloadKind;
use parking_lot::Mutex;

/// Configures the runtime before any script is loaded.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeConfig {
    /// Optional root script source that a future load step can consume.
    pub root_source: Option<ScriptSource>,
    /// Runtime instrumentation policy for GC budgeting and memory diagnostics.
    pub instrumentation: RuntimeInstrumentationConfig,
}

/// Configures GC budgeting and leak sampling for the runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInstrumentationConfig {
    /// Incremental GC step budget to apply after each frame attempt.
    pub frame_gc_step_kbytes: usize,
    /// Incremental GC step budget to apply after each load/reload attempt.
    pub reload_gc_step_kbytes: usize,
    /// Take a full-GC leak sample after every N committed reloads.
    pub leak_sample_reload_interval: u64,
    /// Warn when retained heap growth exceeds this threshold between samples.
    pub leak_retained_growth_warning_bytes: usize,
}

impl Default for RuntimeInstrumentationConfig {
    fn default() -> Self {
        Self {
            frame_gc_step_kbytes: 64,
            reload_gc_step_kbytes: 512,
            leak_sample_reload_interval: 10,
            leak_retained_growth_warning_bytes: 64 * 1024,
        }
    }
}

/// Stable host-owned identity for a mounted scripted surface.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceId(Arc<str>);

impl SurfaceId {
    /// Construct a surface id from host-owned text.
    pub fn new(value: impl Into<String>) -> Self {
        Self(Arc::<str>::from(value.into()))
    }

    /// Borrow the surface id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SurfaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for SurfaceId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SurfaceId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Host capabilities granted to a mounted scripted surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SurfaceCapabilities {
    pub log: bool,
    pub reload: bool,
    pub repaint: bool,
}

impl SurfaceCapabilities {
    /// Deny every capability.
    pub const fn none() -> Self {
        Self {
            log: false,
            reload: false,
            repaint: false,
        }
    }
}

/// Host-owned mount contract for one scripted surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceMount {
    pub surface_id: SurfaceId,
    pub capabilities: SurfaceCapabilities,
}

impl SurfaceMount {
    /// Construct a new surface mount declaration.
    pub fn new(surface_id: SurfaceId, capabilities: SurfaceCapabilities) -> Self {
        Self {
            surface_id,
            capabilities,
        }
    }
}

/// Canonical scripted widget identity: `surface_id + explicit push_id stack + local_id`.
#[derive(Debug, Clone, Copy)]
pub struct WidgetIdentity<'a, Scope = String>
where
    Scope: AsRef<str>,
{
    surface_id: &'a SurfaceId,
    id_stack: &'a [Scope],
    local_id: &'a str,
}

impl<'a, Scope> WidgetIdentity<'a, Scope>
where
    Scope: AsRef<str>,
{
    /// Borrow the scripted identity parts without allocating.
    pub fn new(surface_id: &'a SurfaceId, id_stack: &'a [Scope], local_id: &'a str) -> Self {
        Self {
            surface_id,
            id_stack,
            local_id,
        }
    }

    /// Iterate the full identity path in order.
    pub fn segments(&self) -> impl Iterator<Item = &str> + '_ {
        std::iter::once(self.surface_id.as_str())
            .chain(self.id_stack.iter().map(|scope| scope.as_ref()))
            .chain(std::iter::once(self.local_id))
    }

    /// Render the identity as a stable debug path.
    pub fn debug_path(&self) -> String {
        self.segments().collect::<Vec<_>>().join("::")
    }
}

/// A Luau script source, either inline text or a file-backed path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptSource {
    /// Source code provided directly by the host.
    Inline { name: String, source: String },
    /// Source code loaded from disk.
    Path(PathBuf),
}

impl ScriptSource {
    /// Create an inline script source.
    pub fn inline(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self::Inline {
            name: name.into(),
            source: source.into(),
        }
    }

    /// Create a file-backed script source.
    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }
}

/// Thread-safe batch of distinct file paths that need reload attention.
#[derive(Debug, Clone, Default)]
pub struct ReloadQueue {
    pending: Arc<Mutex<BTreeSet<PathBuf>>>,
}

impl ReloadQueue {
    /// Create an empty reload queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueue a path for later reload handling.
    ///
    /// Duplicate paths coalesce until the next drain so one save burst becomes one reload batch.
    pub fn push(&self, path: impl Into<PathBuf>) {
        self.pending.lock().insert(path.into());
    }

    /// Drain all queued paths as one distinct, deterministic batch.
    pub fn drain(&self) -> Vec<PathBuf> {
        let mut pending = self.pending.lock();
        let drained = pending.iter().cloned().collect();
        pending.clear();
        drained
    }

    /// Return `true` when no paths are pending.
    pub fn is_empty(&self) -> bool {
        self.pending.lock().is_empty()
    }

    /// Return the number of distinct queued paths.
    pub fn len(&self) -> usize {
        self.pending.lock().len()
    }
}

/// Error type used by the runtime facade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// Invalid configuration supplied by the host.
    Config(String),
    /// Filesystem or provider-level failure.
    Io(String),
    /// Luau backend failure surfaced as a string so the backend stays private.
    Backend(String),
}

impl RuntimeError {
    /// Create a configuration error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Create an IO error.
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io(message.into())
    }

    /// Create a backend error.
    pub fn backend(message: impl Into<String>) -> Self {
        Self::Backend(message.into())
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(message) => write!(f, "config error: {message}"),
            Self::Io(message) => write!(f, "io error: {message}"),
            Self::Backend(message) => write!(f, "backend error: {message}"),
        }
    }
}

impl Error for RuntimeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLabelTone {
    #[default]
    Primary,
    Secondary,
    Muted,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLabelWeight {
    #[default]
    Regular,
    Semibold,
    Bold,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiLabelOptions {
    pub tone: UiLabelTone,
    pub weight: UiLabelWeight,
    pub size: Option<f32>,
}

impl Default for UiLabelOptions {
    fn default() -> Self {
        Self {
            tone: UiLabelTone::Primary,
            weight: UiLabelWeight::Regular,
            size: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiControlSize {
    Sm,
    #[default]
    Md,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiButtonOptions {
    pub variant: UiButtonVariant,
    pub size: UiControlSize,
    pub width: Option<f32>,
    pub leading_icon: Option<String>,
    pub trailing_icon: Option<String>,
    pub icon_size: Option<f32>,
    pub icon_only: bool,
    pub selected: bool,
}

impl Default for UiButtonOptions {
    fn default() -> Self {
        Self {
            variant: UiButtonVariant::Primary,
            size: UiControlSize::Md,
            width: None,
            leading_icon: None,
            trailing_icon: None,
            icon_size: None,
            icon_only: false,
            selected: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiTextEditOptions {
    pub width: Option<f32>,
    pub placeholder: Option<String>,
    pub leading_icon: Option<String>,
    pub password: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTextEditOutput {
    pub value: String,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiBooleanOutput {
    pub value: bool,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiCheckboxOptions {
    pub label: Option<String>,
}

impl Default for UiCheckboxOptions {
    fn default() -> Self {
        Self { label: None }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiSwitchOptions {
    pub label: Option<String>,
    pub size: UiControlSize,
}

impl Default for UiSwitchOptions {
    fn default() -> Self {
        Self {
            label: None,
            size: UiControlSize::Md,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiSliderOptions {
    pub min: f32,
    pub max: f32,
    pub width: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiNumberOutput {
    pub value: f32,
    pub changed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiNumberInputAxis {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiNumberInputOptions {
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub width: Option<f32>,
    pub speed: Option<f64>,
    pub fine_speed: Option<f64>,
    pub decimals: Option<usize>,
    pub fine_decimals: Option<usize>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub prefix_tint: Option<String>,
    pub prefix_align_left: bool,
    pub axis: UiNumberInputAxis,
}

impl Default for UiNumberInputOptions {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            width: None,
            speed: None,
            fine_speed: None,
            decimals: None,
            fine_decimals: None,
            prefix: None,
            suffix: None,
            prefix_tint: None,
            prefix_align_left: false,
            axis: UiNumberInputAxis::Horizontal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiSelectVariant {
    #[default]
    Default,
    Secondary,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiSelectOptions {
    pub width: Option<f32>,
    pub placeholder: Option<String>,
    pub variant: UiSelectVariant,
}

impl Default for UiSelectOptions {
    fn default() -> Self {
        Self {
            width: None,
            placeholder: None,
            variant: UiSelectVariant::Default,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiSelectOutput {
    /// 1-based option index. `0` means no active selection.
    pub selected_index: usize,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTabOption {
    pub label: String,
    pub icon: Option<String>,
    pub icon_only: bool,
    pub tooltip: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiTabsVariant {
    #[default]
    Underline,
    Segmented,
    Stacked,
    Rail,
    BlenderTopbar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiTabsOptions {
    pub variant: UiTabsVariant,
}

impl Default for UiTabsOptions {
    fn default() -> Self {
        Self {
            variant: UiTabsVariant::Underline,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTabsOutput {
    /// 1-based option index. `0` means no active selection.
    pub selected_index: usize,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiProgressOptions {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Default for UiProgressOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiRadioOptions {
    pub label: Option<String>,
    pub description: Option<String>,
}

impl Default for UiRadioOptions {
    fn default() -> Self {
        Self {
            label: None,
            description: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiButtonGroupOptions {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiButtonGroupOutput {
    /// 1-based option index. `0` means no click occurred.
    pub clicked_index: usize,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiCollapsibleOptions {
    pub open: Option<bool>,
    pub leading_icon: Option<String>,
    pub trailing_icon: Option<String>,
}

impl Default for UiCollapsibleOptions {
    fn default() -> Self {
        Self {
            open: None,
            leading_icon: None,
            trailing_icon: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiCollapsibleOutput {
    pub open: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiDropdownMenuAction {
    pub id: usize,
    pub label: String,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiDropdownMenuSubmenu {
    pub label: String,
    pub icon: Option<String>,
    pub entries: Vec<UiDropdownMenuEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiDropdownMenuEntry {
    Action(UiDropdownMenuAction),
    Separator,
    Submenu(UiDropdownMenuSubmenu),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiDropdownMenuOptions {
    pub width: Option<f32>,
    pub trigger_variant: UiButtonVariant,
}

impl Default for UiDropdownMenuOptions {
    fn default() -> Self {
        Self {
            width: None,
            trigger_variant: UiButtonVariant::Secondary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiDropdownMenuOutput {
    /// Action id. `0` means no action fired.
    pub action_id: usize,
    pub changed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiTooltipPlacement {
    #[default]
    Auto,
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiTooltipOptions {
    pub width: Option<f32>,
    pub delay_ms: Option<u32>,
    pub placement: UiTooltipPlacement,
}

impl Default for UiTooltipOptions {
    fn default() -> Self {
        Self {
            width: None,
            delay_ms: None,
            placement: UiTooltipPlacement::Auto,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiSpinnerOptions {
    pub size: Option<f32>,
    pub stroke_width: Option<f32>,
    pub speed: Option<f32>,
    pub color: Option<String>,
}

impl Default for UiSpinnerOptions {
    fn default() -> Self {
        Self {
            size: None,
            stroke_width: None,
            speed: None,
            color: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiSkeletonShape {
    #[default]
    Rect,
    Circle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiSkeletonOptions {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub shape: UiSkeletonShape,
}

impl Default for UiSkeletonOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            shape: UiSkeletonShape::Rect,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiVirtualListOptions {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub row_height: f32,
    /// 1-based row index. `0` means no active selection.
    pub selected_index: usize,
}

impl Default for UiVirtualListOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: Some(220.0),
            row_height: 24.0,
            selected_index: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiVirtualListOutput {
    /// 1-based row index. `0` means no active selection.
    pub selected_index: usize,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiContainerOptions {
    pub gap: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiCardOptions {
    pub width: Option<f32>,
    pub padding_x: f32,
    pub padding_y: f32,
}

impl Default for UiCardOptions {
    fn default() -> Self {
        Self {
            width: None,
            padding_x: 12.0,
            padding_y: 12.0,
        }
    }
}

/// Backend-agnostic host API used by script-facing `app.*` bindings.
pub trait RuntimeAppHost {
    fn log(&mut self, level: &str, message: &str) -> Result<(), RuntimeError>;
    fn request_reload(&mut self) -> Result<(), RuntimeError>;
    fn request_repaint(&mut self) -> Result<(), RuntimeError>;
}

/// Backend-agnostic host API used by script-facing `ui.*` bindings during render.
pub trait RuntimeUiHost: RuntimeAppHost {
    fn label(&mut self, text: &str, options: UiLabelOptions) -> Result<(), RuntimeError>;
    fn separator(&mut self) -> Result<(), RuntimeError>;
    fn button(
        &mut self,
        id: &str,
        text: &str,
        options: UiButtonOptions,
    ) -> Result<bool, RuntimeError>;
    fn text_edit(
        &mut self,
        id: &str,
        value: &str,
        options: UiTextEditOptions,
    ) -> Result<UiTextEditOutput, RuntimeError>;
    fn checkbox(
        &mut self,
        id: &str,
        checked: bool,
        options: UiCheckboxOptions,
    ) -> Result<UiBooleanOutput, RuntimeError>;
    fn switch(
        &mut self,
        id: &str,
        checked: bool,
        options: UiSwitchOptions,
    ) -> Result<UiBooleanOutput, RuntimeError>;
    fn slider(
        &mut self,
        id: &str,
        value: f32,
        options: UiSliderOptions,
    ) -> Result<UiNumberOutput, RuntimeError>;
    fn number_input(
        &mut self,
        id: &str,
        value: f32,
        options: UiNumberInputOptions,
    ) -> Result<UiNumberOutput, RuntimeError>;
    fn select(
        &mut self,
        id: &str,
        selected_index: usize,
        items: &[String],
        options: UiSelectOptions,
    ) -> Result<UiSelectOutput, RuntimeError>;
    fn tabs(
        &mut self,
        id: &str,
        selected_index: usize,
        options: &[UiTabOption],
        props: UiTabsOptions,
    ) -> Result<UiTabsOutput, RuntimeError>;
    fn progress(&mut self, value: f32, options: UiProgressOptions) -> Result<(), RuntimeError>;
    fn radio(
        &mut self,
        id: &str,
        selected: bool,
        options: UiRadioOptions,
    ) -> Result<UiBooleanOutput, RuntimeError>;
    fn button_group(
        &mut self,
        id: &str,
        options: &[String],
        props: UiButtonGroupOptions,
    ) -> Result<UiButtonGroupOutput, RuntimeError>;
    fn begin_collapsible(
        &mut self,
        id: &str,
        title: &str,
        open: bool,
        options: UiCollapsibleOptions,
    ) -> Result<UiCollapsibleOutput, RuntimeError>;
    fn dropdown_menu(
        &mut self,
        id: &str,
        trigger_label: &str,
        entries: &[UiDropdownMenuEntry],
        options: UiDropdownMenuOptions,
    ) -> Result<UiDropdownMenuOutput, RuntimeError>;
    fn tooltip(
        &mut self,
        trigger_label: &str,
        text: &str,
        options: UiTooltipOptions,
    ) -> Result<(), RuntimeError>;
    fn spinner(&mut self, options: UiSpinnerOptions) -> Result<(), RuntimeError>;
    fn skeleton(&mut self, options: UiSkeletonOptions) -> Result<(), RuntimeError>;
    fn virtual_list(
        &mut self,
        id: &str,
        items: &[String],
        options: UiVirtualListOptions,
    ) -> Result<UiVirtualListOutput, RuntimeError>;
    fn begin_row(&mut self, options: UiContainerOptions) -> Result<(), RuntimeError>;
    fn begin_column(&mut self, options: UiContainerOptions) -> Result<(), RuntimeError>;
    fn begin_card(&mut self, options: UiCardOptions) -> Result<(), RuntimeError>;
    fn end(&mut self) -> Result<(), RuntimeError>;
    fn push_id(&mut self, id: &str) -> Result<(), RuntimeError>;
    fn pop_id(&mut self) -> Result<(), RuntimeError>;
}

/// Per-operation memory deltas recorded around a runtime boundary.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeMemoryOperationMetrics {
    pub attempts: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_before_bytes: Option<usize>,
    pub last_after_bytes: Option<usize>,
    pub last_delta_bytes: Option<i64>,
    pub last_post_gc_bytes: Option<usize>,
    pub last_post_gc_delta_bytes: Option<i64>,
}

impl RuntimeMemoryOperationMetrics {
    pub(crate) fn record_attempt(
        &mut self,
        before_bytes: usize,
        after_bytes: usize,
        success: bool,
    ) {
        self.attempts += 1;
        self.last_before_bytes = Some(before_bytes);
        self.last_after_bytes = Some(after_bytes);
        self.last_delta_bytes = Some(signed_byte_delta(after_bytes, before_bytes));
        if success {
            self.successes += 1;
        } else {
            self.failures += 1;
        }
    }

    pub(crate) fn record_post_gc(&mut self, post_gc_bytes: usize, before_bytes: usize) {
        self.last_post_gc_bytes = Some(post_gc_bytes);
        self.last_post_gc_delta_bytes = Some(signed_byte_delta(post_gc_bytes, before_bytes));
    }
}

/// Per-frame memory metrics.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeFrameMemoryMetrics {
    pub last_render_kind: Option<RuntimeRenderKind>,
    pub totals: RuntimeMemoryOperationMetrics,
}

/// Per-load/reload memory metrics.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeReloadMemoryMetrics {
    pub last_kind: Option<ReloadKind>,
    pub totals: RuntimeMemoryOperationMetrics,
}

/// GC progress counters captured by the runtime instrumentation layer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeGcMetrics {
    pub frame_step_attempts: u64,
    pub frame_step_completions: u64,
    pub reload_step_attempts: u64,
    pub reload_step_completions: u64,
    pub full_gc_runs: u64,
    pub last_step_kbytes: Option<usize>,
    pub last_cycle_finished: Option<bool>,
    pub is_running: bool,
}

/// Summary from a forced-GC leak sample.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLeakSample {
    pub sample_index: u64,
    pub committed_reload_count: u64,
    pub retained_bytes: usize,
    pub retained_growth_bytes: i64,
    pub live_object_count: usize,
    pub live_object_growth: i64,
    pub module_count: usize,
    pub warning_emitted: bool,
}

/// Leak-detection history and the last surfaced warning.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeLeakMetrics {
    pub committed_reload_count: u64,
    pub sample_count: u64,
    pub last_sample: Option<RuntimeLeakSample>,
    pub last_warning_message: Option<String>,
}

/// GC and heap diagnostics for the current runtime instance.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeMemoryMetrics {
    pub current_heap_bytes: usize,
    pub peak_heap_bytes: usize,
    pub frame: RuntimeFrameMemoryMetrics,
    pub reload: RuntimeReloadMemoryMetrics,
    pub gc: RuntimeGcMetrics,
    pub leak_detection: RuntimeLeakMetrics,
}

fn signed_byte_delta(after_bytes: usize, before_bytes: usize) -> i64 {
    match after_bytes.cmp(&before_bytes) {
        std::cmp::Ordering::Greater => after_bytes
            .saturating_sub(before_bytes)
            .min(i64::MAX as usize) as i64,
        std::cmp::Ordering::Less => {
            -((before_bytes
                .saturating_sub(after_bytes)
                .min(i64::MAX as usize)) as i64)
        }
        std::cmp::Ordering::Equal => 0,
    }
}

/// Coarse reload metrics for the runtime hot path.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReloadMetrics {
    pub attempts: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_kind: Option<ReloadKind>,
    /// Distinct dirty paths drained for the last reload boundary.
    pub last_dirty_path_count: usize,
    pub last_affected_path_count: usize,
    pub last_rebuilt_module_count: usize,
    pub last_active_module_count: usize,
    pub last_duration: Option<Duration>,
}

/// Coarse counters and timing for one runtime operation bucket.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeStageMetrics {
    pub attempts: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_duration: Option<Duration>,
}

impl RuntimeStageMetrics {
    pub(crate) fn record(&mut self, duration: Duration, success: bool) {
        self.attempts += 1;
        self.last_duration = Some(duration);
        if success {
            self.successes += 1;
        } else {
            self.failures += 1;
        }
    }

    pub(crate) fn record_failure(&mut self) {
        self.attempts += 1;
        self.failures += 1;
    }
}

/// The render bucket used for per-frame timing and failure counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRenderKind {
    FirstAfterLoad,
    FirstAfterReload,
    SteadyState,
}

impl RuntimeRenderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FirstAfterLoad => "first_after_load",
            Self::FirstAfterReload => "first_after_reload",
            Self::SteadyState => "steady_state",
        }
    }
}

/// Render counters split by first-frame and steady-state paths.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeRenderMetrics {
    pub first_after_load: RuntimeStageMetrics,
    pub first_after_reload: RuntimeStageMetrics,
    pub steady_state: RuntimeStageMetrics,
    pub last_kind: Option<RuntimeRenderKind>,
}

impl RuntimeRenderMetrics {
    pub(crate) fn record(&mut self, kind: RuntimeRenderKind, duration: Duration, success: bool) {
        self.last_kind = Some(kind);
        match kind {
            RuntimeRenderKind::FirstAfterLoad => self.first_after_load.record(duration, success),
            RuntimeRenderKind::FirstAfterReload => {
                self.first_after_reload.record(duration, success)
            }
            RuntimeRenderKind::SteadyState => self.steady_state.record(duration, success),
        }
    }
}

/// Baseline counters for the current runtime lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeLifecycleMetrics {
    pub load: RuntimeStageMetrics,
    pub reload: RuntimeStageMetrics,
    pub render: RuntimeRenderMetrics,
}

/// The high-level failure class recorded by the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeFailureClass {
    CompileLoad,
    InitReload,
    Update,
    Render,
    HostCallbackPanic,
    CommandFlush,
}

impl RuntimeFailureClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CompileLoad => "compile_load",
            Self::InitReload => "init_reload",
            Self::Update => "update",
            Self::Render => "render",
            Self::HostCallbackPanic => "host_callback_panic",
            Self::CommandFlush => "command_flush",
        }
    }
}

/// The lifecycle stage where a failure occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeFailureStage {
    Load,
    Reload,
    FirstAfterLoad,
    FirstAfterReload,
    SteadyState,
}

impl RuntimeFailureStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Reload => "reload",
            Self::FirstAfterLoad => "first_after_load",
            Self::FirstAfterReload => "first_after_reload",
            Self::SteadyState => "steady_state",
        }
    }
}

/// Structured information about the most recent runtime failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFailureRecord {
    pub class: RuntimeFailureClass,
    pub stage: RuntimeFailureStage,
    pub message: String,
    pub module_context: Option<String>,
    pub hook_or_api: Option<String>,
    pub rolled_back: bool,
}

/// Failure counters for the runtime lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeFailureMetrics {
    pub compile_load: RuntimeStageMetrics,
    pub init_reload: RuntimeStageMetrics,
    pub first_frame_update: RuntimeStageMetrics,
    pub first_frame_render: RuntimeStageMetrics,
    pub steady_state_update: RuntimeStageMetrics,
    pub steady_state_render: RuntimeStageMetrics,
    pub host_callback_panic: RuntimeStageMetrics,
    pub command_flush: RuntimeStageMetrics,
    pub last_failure: Option<RuntimeFailureRecord>,
}

impl RuntimeFailureMetrics {
    pub(crate) fn record(&mut self, record: RuntimeFailureRecord) {
        let bucket = match record.class {
            RuntimeFailureClass::CompileLoad => &mut self.compile_load,
            RuntimeFailureClass::InitReload => &mut self.init_reload,
            RuntimeFailureClass::Update => match record.stage {
                RuntimeFailureStage::FirstAfterLoad | RuntimeFailureStage::FirstAfterReload => {
                    &mut self.first_frame_update
                }
                RuntimeFailureStage::SteadyState => &mut self.steady_state_update,
                RuntimeFailureStage::Load | RuntimeFailureStage::Reload => {
                    &mut self.first_frame_update
                }
            },
            RuntimeFailureClass::Render => match record.stage {
                RuntimeFailureStage::FirstAfterLoad | RuntimeFailureStage::FirstAfterReload => {
                    &mut self.first_frame_render
                }
                RuntimeFailureStage::SteadyState => &mut self.steady_state_render,
                RuntimeFailureStage::Load | RuntimeFailureStage::Reload => {
                    &mut self.first_frame_render
                }
            },
            RuntimeFailureClass::HostCallbackPanic => &mut self.host_callback_panic,
            RuntimeFailureClass::CommandFlush => &mut self.command_flush,
        };
        bucket.record_failure();
        self.last_failure = Some(record);
    }
}
