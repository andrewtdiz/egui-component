use std::{ops::RangeInclusive, path::PathBuf};

use crate::{
    components::{
        Button, ButtonGroup, ButtonVariant, Checkbox, ComponentUiExt, ControlSize, Label,
        LabelTone, LabelWeight, NumberInput, NumberInputAxis, Progress, Radio, Select,
        SelectVariant, Skeleton, Slider, Spinner, Switch, TabOption, TabsVariant, TextInput,
        Tooltip, TooltipPlacement,
    },
    primitives::surface::{surface_frame_builder, SurfaceFrame},
    theme::{self, BaseColor, ThemeMode, ThemeSpec},
    ui::tokens,
};
use egui::{Align, CentralPanel, Context, Id, Layout, ScrollArea, UiBuilder, Window};
use log::{debug, error, info, trace, warn};
use luau_runtime_core::{
    RootScriptWatcher, RuntimeAppHost, RuntimeConfig, RuntimeError, RuntimeRenderKind,
    RuntimeUiHost, ScriptRuntime, ScriptSource, SurfaceCapabilities, SurfaceId, SurfaceMount,
    UiBooleanOutput, UiButtonGroupOptions, UiButtonGroupOutput, UiButtonOptions, UiButtonVariant,
    UiCardOptions, UiCheckboxOptions, UiCollapsibleOptions, UiCollapsibleOutput,
    UiContainerOptions, UiControlSize, UiDropdownMenuEntry, UiDropdownMenuOptions,
    UiDropdownMenuOutput, UiLabelOptions, UiLabelTone, UiLabelWeight, UiNumberInputAxis,
    UiNumberInputOptions, UiNumberOutput, UiProgressOptions, UiRadioOptions, UiSelectOptions,
    UiSelectOutput, UiSelectVariant, UiSkeletonOptions, UiSkeletonShape, UiSliderOptions,
    UiSpinnerOptions, UiSwitchOptions, UiTabOption, UiTabsOptions, UiTabsOutput, UiTabsVariant,
    UiTextEditOptions, UiTextEditOutput, UiTooltipOptions, UiTooltipPlacement,
    UiVirtualListOptions, UiVirtualListOutput, WidgetIdentity,
};

pub const WINDOW_TITLE: &str = "Luau Runtime egui Host";
pub const WINDOW_INNER_SIZE: [f32; 2] = [1360.0, 940.0];

#[derive(Debug, Clone, Copy)]
pub struct RuntimeEguiHostUiOptions {
    pub show_diagnostics: bool,
    pub show_error_overlay: bool,
}

impl Default for RuntimeEguiHostUiOptions {
    fn default() -> Self {
        Self {
            show_diagnostics: true,
            show_error_overlay: true,
        }
    }
}

pub struct RuntimeEguiHostApp {
    runtime: ScriptRuntime,
    root_path: PathBuf,
    surface_mount: SurfaceMount,
    ui_options: RuntimeEguiHostUiOptions,
    reload_watcher: Option<RootScriptWatcher>,
    initial_load_attempted: bool,
    watcher_attempted: bool,
    pending_reload: bool,
    repaint_requested: bool,
    host_error: Option<String>,
}

impl Default for RuntimeEguiHostApp {
    fn default() -> Self {
        Self::new(default_script_path())
    }
}

impl RuntimeEguiHostApp {
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        Self::new_with_ui_options(root_path, RuntimeEguiHostUiOptions::default())
    }

    pub fn new_with_ui_options(
        root_path: impl Into<PathBuf>,
        ui_options: RuntimeEguiHostUiOptions,
    ) -> Self {
        Self {
            runtime: ScriptRuntime::new(RuntimeConfig::default()),
            root_path: root_path.into(),
            surface_mount: SurfaceMount::new(
                SurfaceId::new("runtime_egui_host.main"),
                SurfaceCapabilities {
                    log: true,
                    reload: true,
                    repaint: true,
                    ..SurfaceCapabilities::none()
                },
            ),
            ui_options,
            reload_watcher: None,
            initial_load_attempted: false,
            watcher_attempted: false,
            pending_reload: false,
            repaint_requested: false,
            host_error: None,
        }
    }

    pub fn new_presentational(root_path: impl Into<PathBuf>) -> Self {
        Self::new_with_ui_options(
            root_path,
            RuntimeEguiHostUiOptions {
                show_diagnostics: false,
                show_error_overlay: true,
            },
        )
    }

    pub(crate) fn process_frame_boundary(&mut self, ctx: &Context) {
        self.ensure_reload_watcher(ctx);
        self.queue_dirty_reload_if_needed(ctx);

        let needs_reload = self.pending_reload;
        let needs_initial_load = !self.initial_load_attempted;
        if !needs_initial_load && !needs_reload {
            return;
        }

        if needs_initial_load {
            self.initial_load_attempted = true;
        } else {
            self.pending_reload = false;
        }

        let mut host =
            RuntimeControlHost::new(ctx, &mut self.pending_reload, &mut self.repaint_requested);
        let had_root = self.runtime.config().root_source.is_some();
        let result = if had_root {
            self.runtime
                .reload_now_with_host(self.surface_mount.clone(), &mut host)
        } else {
            self.runtime.load_root_with_host(
                ScriptSource::path(self.root_path.clone()),
                self.surface_mount.clone(),
                &mut host,
            )
        };
        let lifecycle = self.runtime.lifecycle_metrics().clone();
        let metrics = self.runtime.reload_metrics().clone();
        let duration_ms = if had_root {
            lifecycle.reload.last_duration
        } else {
            lifecycle.load.last_duration
        }
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

        match result {
            Ok(()) => {
                info!(
                    "runtime_egui_host action={} script={} rebuilt={} affected={} dirty={} duration_ms={}",
                    if had_root { "reload_staged" } else { "load_staged" },
                    self.root_path.display(),
                    metrics.last_rebuilt_module_count,
                    metrics.last_affected_path_count,
                    metrics.last_dirty_path_count,
                    duration_ms
                );
                self.host_error = None;
            }
            Err(err) => {
                warn!(
                    "runtime_egui_host action={} script={} affected={} dirty={} duration_ms={} error={}",
                    if had_root { "reload" } else { "load" },
                    self.root_path.display(),
                    metrics.last_affected_path_count,
                    metrics.last_dirty_path_count,
                    duration_ms,
                    err
                );
                self.host_error = Some(err.to_string());
            }
        }
    }

    fn ensure_reload_watcher(&mut self, ctx: &Context) {
        if self.watcher_attempted {
            return;
        }

        self.watcher_attempted = true;
        let reload_queue = self.runtime.reload_queue();
        let repaint_ctx = ctx.clone();
        match RootScriptWatcher::new(self.root_path.clone(), reload_queue, move || {
            repaint_ctx.request_repaint();
        }) {
            Ok(watcher) => {
                info!(
                    "runtime_egui_host watching {}",
                    watcher.watched_dir().display()
                );
                self.reload_watcher = Some(watcher);
                self.host_error = None;
            }
            Err(err) => {
                warn!("runtime_egui_host watcher_start_failed error={err}");
                self.host_error = Some(format!("Failed to start watcher: {err}"));
            }
        }
    }

    fn queue_dirty_reload_if_needed(&mut self, ctx: &Context) {
        let dirty_count = self.runtime.reload_queue().len();
        if dirty_count == 0 {
            return;
        }

        self.pending_reload = true;
        self.repaint_requested = true;
        debug!(
            "runtime_egui_host dirty_detected script={} dirty_count={}",
            self.root_path.display(),
            dirty_count
        );
        ctx.request_repaint();
    }

    pub(crate) fn render_script_surface(&mut self, ui: &mut egui::Ui) {
        if !self.runtime.has_render_target() {
            ui.heading("No Luau script loaded");
            ui.label("Fix the script file and save to trigger reload.");
            return;
        }

        let ctx = ui.ctx().clone();
        let mut host = FrameUiHost::new(
            &ctx,
            self.surface_mount.surface_id.clone(),
            ui,
            &mut self.pending_reload,
            &mut self.repaint_requested,
        );

        if let Err(err) = self
            .runtime
            .render_frame_with_host(self.surface_mount.clone(), &mut host)
        {
            warn!("runtime_egui_host render_failed error={err}");
        } else {
            let render_metrics = &self.runtime.lifecycle_metrics().render;
            if let Some(kind) = render_metrics.last_kind {
                let duration_ms = match kind {
                    RuntimeRenderKind::FirstAfterLoad => {
                        render_metrics.first_after_load.last_duration
                    }
                    RuntimeRenderKind::FirstAfterReload => {
                        render_metrics.first_after_reload.last_duration
                    }
                    RuntimeRenderKind::SteadyState => render_metrics.steady_state.last_duration,
                }
                .map(|duration| duration.as_millis())
                .unwrap_or(0);
                debug!(
                    "runtime_egui_host action=render kind={} script={} duration_ms={}",
                    kind.as_str(),
                    self.root_path.display(),
                    duration_ms
                );
            }
        }
    }

    pub(crate) fn show_error_overlay(&mut self, ctx: &Context) {
        let host_error = self.host_error.as_deref();
        let failure = self.runtime.last_failure();
        let compile_error = self.runtime.last_compile_error();
        let runtime_error = self.runtime.last_runtime_error();
        if host_error.is_none()
            && failure.is_none()
            && compile_error.is_none()
            && runtime_error.is_none()
        {
            return;
        }

        Window::new("Luau Runtime Error")
            .id(Id::new("runtime_egui_host.error_overlay"))
            .interactable(false)
            .resizable(true)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(210, 112, 80),
                    "The Luau runtime host hit an error.",
                );
                ui.separator();

                if let Some(error) = host_error {
                    ui.label("Host error");
                    ui.code(error);
                    if failure.is_some() || compile_error.is_some() || runtime_error.is_some() {
                        ui.separator();
                    }
                }

                if let Some(failure) = failure {
                    ui.label("Failure class");
                    ui.code(failure.class.as_str());
                    ui.label("Failure stage");
                    ui.code(failure.stage.as_str());
                    ui.label("Rollback");
                    ui.code(if failure.rolled_back { "yes" } else { "no" });
                    if let Some(module_context) = failure.module_context.as_deref() {
                        ui.label("Module context");
                        ui.code(module_context);
                    }
                    if let Some(hook_or_api) = failure.hook_or_api.as_deref() {
                        ui.label("Hook/API");
                        ui.code(hook_or_api);
                    }
                    ui.label("Message");
                    ui.code(&failure.message);
                } else if let Some(error) = compile_error {
                    ui.label("Compile error");
                    ui.code(error);
                    if runtime_error.is_some() {
                        ui.separator();
                    }
                    if let Some(error) = runtime_error {
                        ui.label("Runtime error");
                        ui.code(error);
                    }
                } else if let Some(error) = runtime_error {
                    ui.label("Runtime error");
                    ui.code(error);
                }
            });
    }

    pub(crate) fn show_diagnostics_window(&mut self, ctx: &Context) {
        let memory = self.runtime.memory_metrics();
        let lifecycle = self.runtime.lifecycle_metrics();
        let reload = self.runtime.reload_metrics();

        Window::new("Luau Runtime Diagnostics")
            .id(Id::new("runtime_egui_host.diagnostics"))
            .resizable(true)
            .default_width(460.0)
            .default_pos(egui::pos2(884.0, 16.0))
            .show(ctx, |ui| {
                ui.label(format!(
                    "Heap bytes: {}",
                    format_bytes(memory.current_heap_bytes)
                ));
                ui.label(format!(
                    "Peak heap bytes: {}",
                    format_bytes(memory.peak_heap_bytes)
                ));
                ui.label(format!(
                    "Frame delta: {}",
                    format_optional_signed_bytes(memory.frame.totals.last_delta_bytes)
                ));
                ui.label(format!(
                    "Frame post-GC: {}",
                    format_optional_bytes(memory.frame.totals.last_post_gc_bytes)
                ));
                if let Some(kind) = memory.frame.last_render_kind {
                    ui.label(format!("Last frame kind: {}", kind.as_str()));
                }
                ui.separator();

                ui.label(format!(
                    "Reload delta: {}",
                    format_optional_signed_bytes(memory.reload.totals.last_delta_bytes)
                ));
                ui.label(format!(
                    "Reload post-GC: {}",
                    format_optional_bytes(memory.reload.totals.last_post_gc_bytes)
                ));
                if let Some(kind) = memory.reload.last_kind {
                    ui.label(format!("Last reload kind: {}", kind.as_str()));
                }
                ui.label(format!(
                    "Reload graph: rebuilt={} affected={} dirty={}",
                    reload.last_rebuilt_module_count,
                    reload.last_affected_path_count,
                    reload.last_dirty_path_count
                ));
                ui.separator();

                ui.label(format!(
                    "GC frame steps: {}/{}",
                    memory.gc.frame_step_completions, memory.gc.frame_step_attempts
                ));
                ui.label(format!(
                    "GC reload steps: {}/{}",
                    memory.gc.reload_step_completions, memory.gc.reload_step_attempts
                ));
                ui.label(format!("Full GC samples: {}", memory.gc.full_gc_runs));
                ui.label(format!(
                    "GC running: {}",
                    if memory.gc.is_running { "yes" } else { "no" }
                ));
                ui.separator();

                ui.label(format!(
                    "Load duration: {}",
                    format_optional_duration(lifecycle.load.last_duration)
                ));
                ui.label(format!(
                    "Reload duration: {}",
                    format_optional_duration(lifecycle.reload.last_duration)
                ));
                let steady_duration = lifecycle.render.steady_state.last_duration;
                ui.label(format!(
                    "Steady render duration: {}",
                    format_optional_duration(steady_duration)
                ));
                ui.separator();

                if let Some(sample) = memory.leak_detection.last_sample.as_ref() {
                    ui.label(format!(
                        "Leak sample: retained={} growth={} live_objects={} growth={} reloads={}",
                        format_bytes(sample.retained_bytes),
                        format_signed_bytes(sample.retained_growth_bytes),
                        sample.live_object_count,
                        sample.live_object_growth,
                        sample.committed_reload_count
                    ));
                } else {
                    ui.label("Leak sample: pending baseline");
                }
                if let Some(message) = memory.leak_detection.last_warning_message.as_deref() {
                    ui.colored_label(
                        egui::Color32::from_rgb(210, 112, 80),
                        format!("Leak warning: {message}"),
                    );
                }
            });
    }
}

pub fn install_context(ctx: &Context) {
    theme::install(ctx, ThemeSpec::preset(BaseColor::Slate), ThemeMode::System);
}

pub fn update(app: &mut RuntimeEguiHostApp, ctx: &Context) {
    app.process_frame_boundary(ctx);

    CentralPanel::default().show(ctx, |ui| {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                app.render_script_surface(ui);
            });
    });

    finish_embedded_frame(app, ctx);
}

pub(crate) fn finish_embedded_frame(app: &mut RuntimeEguiHostApp, ctx: &Context) {
    if app.ui_options.show_diagnostics {
        app.show_diagnostics_window(ctx);
    }
    if app.ui_options.show_error_overlay {
        app.show_error_overlay(ctx);
    }

    if app.repaint_requested {
        app.repaint_requested = false;
        ctx.request_repaint();
    }
}

fn default_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("runtime-luau")
        .join("apps")
        .join("demo.luau")
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

fn format_signed_bytes(bytes: i64) -> String {
    if bytes == 0 {
        return "0 B".to_owned();
    }
    let sign = if bytes < 0 { "-" } else { "+" };
    let magnitude = bytes.unsigned_abs() as usize;
    format!("{sign}{}", format_bytes(magnitude))
}

fn format_optional_bytes(bytes: Option<usize>) -> String {
    bytes.map(format_bytes).unwrap_or_else(|| "n/a".to_owned())
}

fn format_optional_signed_bytes(bytes: Option<i64>) -> String {
    bytes
        .map(format_signed_bytes)
        .unwrap_or_else(|| "n/a".to_owned())
}

fn format_optional_duration(duration: Option<std::time::Duration>) -> String {
    duration
        .map(|duration| format!("{} us", duration.as_micros()))
        .unwrap_or_else(|| "n/a".to_owned())
}

struct RuntimeControlHost<'a> {
    ctx: &'a Context,
    pending_reload: &'a mut bool,
    repaint_requested: &'a mut bool,
}

impl<'a> RuntimeControlHost<'a> {
    fn new(
        ctx: &'a Context,
        pending_reload: &'a mut bool,
        repaint_requested: &'a mut bool,
    ) -> Self {
        Self {
            ctx,
            pending_reload,
            repaint_requested,
        }
    }
}

impl RuntimeAppHost for RuntimeControlHost<'_> {
    fn log(&mut self, level: &str, message: &str) -> Result<(), RuntimeError> {
        log_host_message(level, message);
        Ok(())
    }

    fn request_reload(&mut self) -> Result<(), RuntimeError> {
        *self.pending_reload = true;
        *self.repaint_requested = true;
        self.ctx.request_repaint();
        Ok(())
    }

    fn request_repaint(&mut self) -> Result<(), RuntimeError> {
        *self.repaint_requested = true;
        self.ctx.request_repaint();
        Ok(())
    }
}

enum ScopeFrame {
    Container(egui::Ui),
    Card(egui::containers::frame::Prepared),
}

impl ScopeFrame {
    fn active_ui_mut(&mut self) -> &mut egui::Ui {
        match self {
            Self::Container(ui) => ui,
            Self::Card(prepared) => &mut prepared.content_ui,
        }
    }
}

struct FrameUiHost<'a> {
    ctx: &'a Context,
    surface_id: SurfaceId,
    pending_reload: &'a mut bool,
    repaint_requested: &'a mut bool,
    root_ui: &'a mut egui::Ui,
    scope_stack: Vec<ScopeFrame>,
    id_stack: Vec<String>,
    next_scope_id: u64,
}

impl<'a> FrameUiHost<'a> {
    fn new(
        ctx: &'a Context,
        surface_id: SurfaceId,
        ui: &'a mut egui::Ui,
        pending_reload: &'a mut bool,
        repaint_requested: &'a mut bool,
    ) -> Self {
        Self {
            ctx,
            surface_id,
            pending_reload,
            repaint_requested,
            root_ui: ui,
            scope_stack: Vec::new(),
            id_stack: Vec::new(),
            next_scope_id: 0,
        }
    }

    fn with_active_ui<R>(&mut self, f: impl FnOnce(&mut egui::Ui) -> R) -> R {
        f(self.active_ui_mut())
    }

    fn active_ui_mut(&mut self) -> &mut egui::Ui {
        match self.scope_stack.last_mut() {
            Some(scope) => scope.active_ui_mut(),
            None => self.root_ui,
        }
    }

    fn widget_identity<'b>(&'b self, local_id: &'b str) -> WidgetIdentity<'b, String> {
        WidgetIdentity::new(&self.surface_id, &self.id_stack, local_id)
    }

    fn begin_container(
        &mut self,
        layout: Layout,
        options: UiContainerOptions,
        gap_on_x_axis: bool,
    ) -> Result<(), RuntimeError> {
        let scope_id = self.next_scope_id;
        self.next_scope_id += 1;

        let child = {
            let parent = self.active_ui_mut();
            let mut child = parent.new_child(
                UiBuilder::new()
                    .id_salt(scope_id)
                    .max_rect(parent.available_rect_before_wrap())
                    .layout(layout),
            );
            if let Some(gap) = options.gap {
                if gap_on_x_axis {
                    child.spacing_mut().item_spacing.x = gap;
                } else {
                    child.spacing_mut().item_spacing.y = gap;
                }
            }
            child
        };
        self.scope_stack.push(ScopeFrame::Container(child));
        Ok(())
    }

    fn begin_card_scope(&mut self, options: UiCardOptions) {
        let prepared = {
            let parent = self.active_ui_mut();
            let runtime = theme::runtime_for_ui(parent);
            let mut prepared = surface_frame_builder(
                SurfaceFrame::new(
                    tokens::muted_surface(runtime),
                    egui::Stroke::new(1.0, tokens::separator(runtime)),
                )
                .corner_radius(tokens::radius_lg(runtime))
                .padding(options.padding_x as i8, options.padding_y as i8),
            )
            .begin(parent);

            if let Some(width) = options.width {
                prepared.content_ui.set_min_width(width);
                prepared.content_ui.set_max_width(width);
            }
            prepared
        };
        self.scope_stack.push(ScopeFrame::Card(prepared));
    }
}

impl RuntimeAppHost for FrameUiHost<'_> {
    fn log(&mut self, level: &str, message: &str) -> Result<(), RuntimeError> {
        log_host_message(level, message);
        Ok(())
    }

    fn request_reload(&mut self) -> Result<(), RuntimeError> {
        *self.pending_reload = true;
        *self.repaint_requested = true;
        self.ctx.request_repaint();
        Ok(())
    }

    fn request_repaint(&mut self) -> Result<(), RuntimeError> {
        *self.repaint_requested = true;
        self.ctx.request_repaint();
        Ok(())
    }
}

impl RuntimeUiHost for FrameUiHost<'_> {
    fn label(&mut self, text: &str, options: UiLabelOptions) -> Result<(), RuntimeError> {
        self.with_active_ui(|ui| {
            let mut components = ui.components();
            let mut label = Label::new(text)
                .tone(map_label_tone(options.tone))
                .weight(map_label_weight(options.weight));
            if let Some(size) = options.size {
                label = label.size(size);
            }
            let _ = components.label(label);
        });
        Ok(())
    }

    fn separator(&mut self) -> Result<(), RuntimeError> {
        self.with_active_ui(|ui| {
            let mut components = ui.components();
            let _ = components.separator();
        });
        Ok(())
    }

    fn button(
        &mut self,
        id: &str,
        text: &str,
        options: UiButtonOptions,
    ) -> Result<bool, RuntimeError> {
        let widget_id = egui_widget_id(&self.widget_identity(id));
        let clicked = self.with_active_ui(|ui| {
            let mut components = ui.components();
            let mut button = Button::new(text)
                .id(widget_id)
                .variant(map_button_variant(options.variant))
                .size(map_control_size(options.size))
                .selected(options.selected);
            if let Some(width) = options.width {
                button = button.min_size(egui::vec2(width.max(1.0), 0.0));
            }
            if let Some(icon) = options.leading_icon.as_deref() {
                button = button.leading_icon(icon);
            }
            if let Some(icon) = options.trailing_icon.as_deref() {
                button = button.trailing_icon(icon);
            }
            if let Some(icon_size) = options.icon_size {
                button = button.icon_size(icon_size);
            }
            if options.icon_only {
                button.icon_only = true;
            }
            components.button(button).clicked()
        });
        Ok(clicked)
    }

    fn text_edit(
        &mut self,
        id: &str,
        value: &str,
        options: UiTextEditOptions,
    ) -> Result<UiTextEditOutput, RuntimeError> {
        let widget_id = egui_widget_id(&self.widget_identity(id));
        let output = self.with_active_ui(|ui| {
            let mut components = ui.components();
            let mut value = value.to_owned();
            let mut input = TextInput::new().id(widget_id);
            if let Some(width) = options.width {
                input = input.width(width);
            }
            if let Some(placeholder) = options.placeholder.as_deref() {
                input = input.placeholder(placeholder);
            }
            if let Some(icon) = options.leading_icon.as_deref() {
                input = input.leading_icon(icon);
            }
            if options.password {
                input = input.password(true);
            }
            let response = components.text_input(&mut value, input);
            UiTextEditOutput {
                value,
                changed: response.changed(),
            }
        });
        Ok(output)
    }

    fn checkbox(
        &mut self,
        id: &str,
        checked: bool,
        options: UiCheckboxOptions,
    ) -> Result<UiBooleanOutput, RuntimeError> {
        let widget_id = egui_widget_id(&self.widget_identity(id));
        let output = self.with_active_ui(|ui| {
            let mut value = checked;
            let changed = ui
                .push_id(widget_id, |ui| {
                    let mut components = ui.components();
                    let props = options
                        .label
                        .as_deref()
                        .map(|label| Checkbox::new().label(label))
                        .unwrap_or_else(Checkbox::new);
                    components.checkbox(&mut value, props).changed()
                })
                .inner;
            UiBooleanOutput { value, changed }
        });
        Ok(output)
    }

    fn switch(
        &mut self,
        id: &str,
        checked: bool,
        options: UiSwitchOptions,
    ) -> Result<UiBooleanOutput, RuntimeError> {
        let widget_id = egui_widget_id(&self.widget_identity(id));
        let output = self.with_active_ui(|ui| {
            let mut value = checked;
            let changed = ui
                .push_id(widget_id, |ui| {
                    let mut components = ui.components();
                    let mut props = Switch::new().size(map_control_size(options.size));
                    if let Some(label) = options.label.as_deref() {
                        props = props.label(label);
                    }
                    components.switch(&mut value, props).changed()
                })
                .inner;
            UiBooleanOutput { value, changed }
        });
        Ok(output)
    }

    fn slider(
        &mut self,
        id: &str,
        value: f32,
        options: UiSliderOptions,
    ) -> Result<UiNumberOutput, RuntimeError> {
        let widget_id = egui_widget_id(&self.widget_identity(id));
        let output = self.with_active_ui(|ui| {
            let mut current = value.clamp(options.min, options.max);
            let changed = ui
                .push_id(widget_id, |ui| {
                    let mut components = ui.components();
                    let mut props = Slider::new(options.min..=options.max);
                    if let Some(width) = options.width {
                        props = props.width(width);
                    }
                    components.slider(&mut current, props).changed()
                })
                .inner;
            UiNumberOutput {
                value: current,
                changed,
            }
        });
        Ok(output)
    }

    fn number_input(
        &mut self,
        id: &str,
        value: f32,
        options: UiNumberInputOptions,
    ) -> Result<UiNumberOutput, RuntimeError> {
        let widget_id = egui_widget_id(&self.widget_identity(id));
        let output = self.with_active_ui(|ui| {
            let mut current = value;
            let mut props = NumberInput::new(widget_id);
            if let Some(width) = options.width {
                props = props.width(width);
            }
            let range = number_input_range(&options);
            current = current.clamp(*range.start(), *range.end());
            props = props.range(range);
            if let Some(speed) = options.speed {
                props = props.speed(speed);
            }
            if let Some(fine_speed) = options.fine_speed {
                props = props.fine_speed(fine_speed);
            }
            if let Some(decimals) = options.decimals {
                props = props.decimals(decimals);
            }
            if let Some(fine_decimals) = options.fine_decimals {
                props = props.fine_decimals(fine_decimals);
            }
            if let Some(prefix) = options.prefix.as_deref() {
                props = props.prefix(prefix);
            }
            if let Some(suffix) = options.suffix.as_deref() {
                props = props.suffix(suffix);
            }
            if let Some(prefix_tint) = options.prefix_tint.as_deref() {
                if let Some(prefix_tint) = resolve_ui_color(ui, prefix_tint) {
                    props = props.prefix_tint(prefix_tint);
                }
            }
            if options.prefix_align_left {
                props = props.prefix_align_left();
            }
            props = props.axis(map_number_input_axis(options.axis));
            let mut components = ui.components();
            let changed = components.number_input(&mut current, props).changed();
            UiNumberOutput {
                value: current,
                changed,
            }
        });
        Ok(output)
    }

    fn select(
        &mut self,
        id: &str,
        selected_index: usize,
        items: &[String],
        options: UiSelectOptions,
    ) -> Result<UiSelectOutput, RuntimeError> {
        let widget_id = egui_widget_id(&self.widget_identity(id));
        let output = self.with_active_ui(|ui| {
            let mut current = if selected_index == 0 {
                None
            } else {
                Some(
                    selected_index
                        .saturating_sub(1)
                        .min(items.len().saturating_sub(1)),
                )
            };
            let borrowed = items.iter().map(String::as_str).collect::<Vec<_>>();
            let mut props =
                Select::from_id(widget_id, &borrowed).variant(map_select_variant(options.variant));
            if let Some(width) = options.width {
                props = props.width(width);
            }
            if let Some(placeholder) = options.placeholder.as_deref() {
                props = props.placeholder(placeholder);
            }
            let changed = ui.components().select(&mut current, props).changed();
            UiSelectOutput {
                selected_index: current.map(|index| index + 1).unwrap_or(0),
                changed,
            }
        });
        Ok(output)
    }

    fn tabs(
        &mut self,
        id: &str,
        selected_index: usize,
        options: &[UiTabOption],
        props: UiTabsOptions,
    ) -> Result<UiTabsOutput, RuntimeError> {
        let widget_id = egui_widget_id(&self.widget_identity(id));
        let output = self.with_active_ui(|ui| {
            let mut current = selected_index;
            let tab_options = options
                .iter()
                .enumerate()
                .map(|(index, option)| {
                    let value = index + 1;
                    match (option.icon.as_deref(), option.icon_only) {
                        (Some(icon), true) => TabOption::icon_only(value, &option.label, icon),
                        (Some(icon), false) => TabOption::with_icon(value, &option.label, icon),
                        (None, _) => TabOption::new(value, &option.label),
                    }
                    .tooltip(option.tooltip.as_deref().unwrap_or(""))
                })
                .collect::<Vec<_>>();
            let previous = current;
            ui.components().tabs_variant(
                widget_id,
                &mut current,
                &tab_options,
                map_tabs_variant(props.variant),
            );
            UiTabsOutput {
                selected_index: current,
                changed: current != previous,
            }
        });
        Ok(output)
    }

    fn progress(&mut self, value: f32, options: UiProgressOptions) -> Result<(), RuntimeError> {
        self.with_active_ui(|ui| {
            let mut components = ui.components();
            let mut props = Progress::new();
            if let Some(width) = options.width {
                props = props.width(width);
            }
            if let Some(height) = options.height {
                props = props.height(height);
            }
            let _ = components.progress(value, props);
        });
        Ok(())
    }

    fn radio(
        &mut self,
        id: &str,
        selected: bool,
        options: UiRadioOptions,
    ) -> Result<UiBooleanOutput, RuntimeError> {
        let widget_id = egui_widget_id(&self.widget_identity(id));
        let output = self.with_active_ui(|ui| {
            let mut value = selected;
            let changed = ui
                .push_id(widget_id, |ui| {
                    let mut components = ui.components();
                    let mut props = Radio::new();
                    if let Some(label) = options.label.as_deref() {
                        props = props.label(label);
                    }
                    if let Some(description) = options.description.as_deref() {
                        props = props.description(description);
                    }
                    components.radio(&mut value, props).changed()
                })
                .inner;
            UiBooleanOutput { value, changed }
        });
        Ok(output)
    }

    fn button_group(
        &mut self,
        id: &str,
        options: &[String],
        _props: UiButtonGroupOptions,
    ) -> Result<UiButtonGroupOutput, RuntimeError> {
        let widget_id = egui_widget_id(&self.widget_identity(id));
        let output = self.with_active_ui(|ui| {
            let borrowed = options.iter().map(String::as_str).collect::<Vec<_>>();
            let clicked = ui
                .components()
                .button_group(ButtonGroup::new(widget_id, &borrowed));
            UiButtonGroupOutput {
                clicked_index: clicked.map(|index| index + 1).unwrap_or(0),
                changed: clicked.is_some(),
            }
        });
        Ok(output)
    }

    fn begin_collapsible(
        &mut self,
        id: &str,
        title: &str,
        open: bool,
        options: UiCollapsibleOptions,
    ) -> Result<UiCollapsibleOutput, RuntimeError> {
        let widget_id = egui_widget_id(&self.widget_identity(id));
        let mut next_open = options.open.unwrap_or(open);
        let visible = {
            let scope_id = self.next_scope_id;
            self.next_scope_id += 1;
            let mut visible = false;
            let child = {
                let parent = self.active_ui_mut();
                let mut trigger = Button::new(title)
                    .variant(ButtonVariant::Secondary)
                    .trailing_icon(if next_open {
                        "chevron-down"
                    } else {
                        "chevron-right"
                    })
                    .icon_size(12.0)
                    .min_size(egui::vec2(parent.available_width().max(120.0), 0.0));
                if let Some(icon) = options.leading_icon.as_deref() {
                    trigger = trigger.leading_icon(icon);
                }
                if let Some(icon) = options.trailing_icon.as_deref() {
                    trigger = trigger.trailing_icon(icon);
                }
                if parent
                    .push_id(widget_id, |ui| ui.components().button(trigger).clicked())
                    .inner
                {
                    next_open = !next_open;
                }
                if next_open {
                    parent.add_space(6.0);
                    visible = true;
                    Some(
                        parent.new_child(
                            UiBuilder::new()
                                .id_salt(scope_id)
                                .max_rect(parent.available_rect_before_wrap())
                                .layout(Layout::top_down(Align::Min)),
                        ),
                    )
                } else {
                    None
                }
            };
            if let Some(child) = child {
                self.scope_stack.push(ScopeFrame::Container(child));
            }
            visible
        };
        Ok(UiCollapsibleOutput {
            open: next_open,
            visible,
        })
    }

    fn dropdown_menu(
        &mut self,
        id: &str,
        trigger_label: &str,
        entries: &[UiDropdownMenuEntry],
        options: UiDropdownMenuOptions,
    ) -> Result<UiDropdownMenuOutput, RuntimeError> {
        let widget_id = egui_widget_id(&self.widget_identity(id));
        let output = self.with_active_ui(|ui| {
            let trigger = Button::new(trigger_label)
                .variant(map_button_variant(options.trigger_variant))
                .min_size(egui::vec2(options.width.unwrap_or(180.0).max(120.0), 0.0));
            let response = ui
                .push_id(widget_id, |ui| ui.components().button(trigger))
                .inner;
            let mut action = None;
            let menu_width = options.width.unwrap_or(220.0).max(176.0);
            let _ = egui::Popup::menu(&response).show(|ui| {
                render_runtime_dropdown_entries(ui, entries, &mut action, menu_width);
            });
            UiDropdownMenuOutput {
                action_id: action.unwrap_or(0),
                changed: action.is_some(),
            }
        });
        Ok(output)
    }

    fn tooltip(
        &mut self,
        trigger_label: &str,
        text: &str,
        options: UiTooltipOptions,
    ) -> Result<(), RuntimeError> {
        self.with_active_ui(|ui| {
            let mut components = ui.components();
            let mut props = Tooltip::new(trigger_label, text)
                .placement(map_tooltip_placement(options.placement));
            if let Some(width) = options.width {
                props = props.width(width);
            }
            if let Some(delay_ms) = options.delay_ms {
                props = props.delay_ms(delay_ms);
            }
            let _ = components.tooltip(props);
        });
        Ok(())
    }

    fn spinner(&mut self, options: UiSpinnerOptions) -> Result<(), RuntimeError> {
        self.with_active_ui(|ui| {
            let mut props = Spinner::new();
            if let Some(size) = options.size {
                props = props.size(size);
            }
            if let Some(stroke_width) = options.stroke_width {
                props = props.stroke_width(stroke_width);
            }
            if let Some(speed) = options.speed {
                props = props.speed(speed);
            }
            if let Some(color) = options
                .color
                .as_deref()
                .and_then(|color| resolve_ui_color(ui, color))
            {
                props = props.color(color);
            }
            let mut components = ui.components();
            let _ = components.spinner(props);
        });
        Ok(())
    }

    fn skeleton(&mut self, options: UiSkeletonOptions) -> Result<(), RuntimeError> {
        self.with_active_ui(|ui| {
            let mut components = ui.components();
            let mut props = Skeleton::new();
            if let Some(width) = options.width {
                props = props.width(width);
            }
            if let Some(height) = options.height {
                props = props.height(height);
            }
            if matches!(options.shape, UiSkeletonShape::Circle) {
                props = props.circle(options.width.or(options.height).unwrap_or(24.0));
            }
            let _ = components.skeleton(props);
        });
        Ok(())
    }

    fn virtual_list(
        &mut self,
        id: &str,
        items: &[String],
        options: UiVirtualListOptions,
    ) -> Result<UiVirtualListOutput, RuntimeError> {
        #[cfg(test)]
        TEST_VIRTUAL_LIST_INVOCATIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let widget_id = egui_widget_id(&self.widget_identity(id));
        let mut output = UiVirtualListOutput {
            selected_index: if options.selected_index <= items.len() {
                options.selected_index
            } else {
                0
            },
            changed: false,
        };

        self.with_active_ui(|ui| {
            let runtime = theme::runtime_for_ui(ui);
            let width = options.width.unwrap_or(ui.available_width()).max(1.0);
            let height = options
                .height
                .unwrap_or((options.row_height * 8.0).max(options.row_height))
                .max(options.row_height.max(1.0));
            let row_height = options.row_height.max(18.0);

            let _ = surface_frame_builder(
                SurfaceFrame::new(
                    tokens::input_background(runtime),
                    egui::Stroke::new(1.0, tokens::separator(runtime)),
                )
                .corner_radius(tokens::radius_md(runtime))
                .padding(0, 0),
            )
            .show(ui, |ui| {
                ui.set_min_width(width);
                ui.set_max_width(width);
                ScrollArea::vertical()
                    .id_salt(widget_id)
                    .max_height(height)
                    .auto_shrink([false, false])
                    .show_rows(ui, row_height, items.len(), |ui, row_range| {
                        ui.set_width(width);
                        for row in row_range {
                            let row_index = row + 1;
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), row_height),
                                egui::Sense::click(),
                            );
                            let selected = output.selected_index == row_index;
                            let fill = if selected {
                                ui.visuals().selection.bg_fill
                            } else if response.hovered() {
                                ui.visuals().widgets.hovered.weak_bg_fill
                            } else {
                                egui::Color32::TRANSPARENT
                            };
                            if fill != egui::Color32::TRANSPARENT {
                                ui.painter().rect_filled(rect, 4.0, fill);
                            }
                            ui.painter().text(
                                egui::pos2(rect.left() + 8.0, rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                &items[row],
                                egui::TextStyle::Body.resolve(ui.style()),
                                tokens::text_primary(runtime),
                            );
                            if response.clicked() && output.selected_index != row_index {
                                output.selected_index = row_index;
                                output.changed = true;
                            }
                        }
                    });
            });
        });

        Ok(output)
    }

    fn begin_row(&mut self, options: UiContainerOptions) -> Result<(), RuntimeError> {
        self.begin_container(Layout::left_to_right(Align::Min), options, true)
    }

    fn begin_column(&mut self, options: UiContainerOptions) -> Result<(), RuntimeError> {
        self.begin_container(Layout::top_down(Align::Min), options, false)
    }

    fn begin_card(&mut self, options: UiCardOptions) -> Result<(), RuntimeError> {
        self.begin_card_scope(options);
        Ok(())
    }

    fn end(&mut self) -> Result<(), RuntimeError> {
        let scope = self.scope_stack.pop().ok_or_else(|| {
            RuntimeError::backend("ui.end_scope called without an open container scope".to_owned())
        })?;

        match scope {
            ScopeFrame::Container(ui) => {
                let rect = ui.min_rect();
                drop(ui);
                let _ = self.active_ui_mut().advance_cursor_after_rect(rect);
            }
            ScopeFrame::Card(prepared) => {
                let _ = prepared.end(self.active_ui_mut());
            }
        }
        Ok(())
    }

    fn push_id(&mut self, id: &str) -> Result<(), RuntimeError> {
        self.id_stack.push(id.to_owned());
        Ok(())
    }

    fn pop_id(&mut self) -> Result<(), RuntimeError> {
        self.id_stack.pop().ok_or_else(|| {
            RuntimeError::backend("ui.pop_id called without a pushed id scope".to_owned())
        })?;
        Ok(())
    }
}

fn egui_widget_id<Scope>(identity: &WidgetIdentity<'_, Scope>) -> Id
where
    Scope: AsRef<str>,
{
    let mut segments = identity.segments();
    let mut id = Id::new(segments.next().unwrap_or_default());
    for segment in segments {
        id = id.with(segment);
    }
    id
}

fn map_label_tone(tone: UiLabelTone) -> LabelTone {
    match tone {
        UiLabelTone::Primary => LabelTone::Primary,
        UiLabelTone::Secondary => LabelTone::Secondary,
        UiLabelTone::Muted => LabelTone::Muted,
        UiLabelTone::Destructive => LabelTone::Destructive,
    }
}

fn map_label_weight(weight: UiLabelWeight) -> LabelWeight {
    match weight {
        UiLabelWeight::Regular => LabelWeight::Regular,
        UiLabelWeight::Semibold => LabelWeight::Semibold,
        UiLabelWeight::Bold => LabelWeight::Bold,
    }
}

fn map_button_variant(variant: UiButtonVariant) -> ButtonVariant {
    match variant {
        UiButtonVariant::Primary => ButtonVariant::Primary,
        UiButtonVariant::Secondary => ButtonVariant::Secondary,
        UiButtonVariant::Ghost => ButtonVariant::Ghost,
        UiButtonVariant::Link => ButtonVariant::Link,
    }
}

fn map_control_size(size: UiControlSize) -> ControlSize {
    match size {
        UiControlSize::Sm => ControlSize::Sm,
        UiControlSize::Md => ControlSize::Md,
    }
}

fn map_number_input_axis(axis: UiNumberInputAxis) -> NumberInputAxis {
    match axis {
        UiNumberInputAxis::Horizontal => NumberInputAxis::Horizontal,
        UiNumberInputAxis::Vertical => NumberInputAxis::Vertical,
    }
}

fn number_input_range(options: &UiNumberInputOptions) -> RangeInclusive<f32> {
    let min = options.min.unwrap_or(f32::MIN);
    let max = options.max.unwrap_or(f32::MAX);
    min..=max
}

fn resolve_ui_color(ui: &egui::Ui, value: &str) -> Option<egui::Color32> {
    match value {
        "foreground" => return Some(theme::color(ui, theme::ColorRole::Foreground)),
        "primary_text" | "text_primary" => {
            return Some(tokens::text_primary(theme::runtime_for_ui(ui)))
        }
        "secondary_text" | "text_secondary" => {
            return Some(tokens::text_secondary(theme::runtime_for_ui(ui)))
        }
        "muted_text" | "text_muted" => return Some(tokens::text_muted(theme::runtime_for_ui(ui))),
        _ => {}
    }

    parse_hex_color(value)
}

fn parse_hex_color(value: &str) -> Option<egui::Color32> {
    let trimmed = value.trim();
    let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
    match hex.len() {
        6 => {
            let rgb = u32::from_str_radix(hex, 16).ok()?;
            Some(egui::Color32::from_rgb(
                ((rgb >> 16) & 0xff) as u8,
                ((rgb >> 8) & 0xff) as u8,
                (rgb & 0xff) as u8,
            ))
        }
        8 => {
            let rgba = u32::from_str_radix(hex, 16).ok()?;
            Some(egui::Color32::from_rgba_unmultiplied(
                ((rgba >> 24) & 0xff) as u8,
                ((rgba >> 16) & 0xff) as u8,
                ((rgba >> 8) & 0xff) as u8,
                (rgba & 0xff) as u8,
            ))
        }
        _ => None,
    }
}

fn map_select_variant(variant: UiSelectVariant) -> SelectVariant {
    match variant {
        UiSelectVariant::Default => SelectVariant::Default,
        UiSelectVariant::Secondary => SelectVariant::Secondary,
    }
}

fn map_tabs_variant(variant: UiTabsVariant) -> TabsVariant {
    match variant {
        UiTabsVariant::Underline => TabsVariant::Underline,
        UiTabsVariant::Segmented => TabsVariant::Segmented,
        UiTabsVariant::Stacked => TabsVariant::Stacked,
        UiTabsVariant::Rail => TabsVariant::Rail,
        UiTabsVariant::BlenderTopbar => TabsVariant::BlenderTopbar,
    }
}

fn map_tooltip_placement(placement: UiTooltipPlacement) -> TooltipPlacement {
    match placement {
        UiTooltipPlacement::Auto => TooltipPlacement::Auto,
        UiTooltipPlacement::Top => TooltipPlacement::Top,
        UiTooltipPlacement::Right => TooltipPlacement::Right,
        UiTooltipPlacement::Bottom => TooltipPlacement::Bottom,
        UiTooltipPlacement::Left => TooltipPlacement::Left,
    }
}

fn render_runtime_dropdown_entries(
    ui: &mut egui::Ui,
    entries: &[UiDropdownMenuEntry],
    selected_action: &mut Option<usize>,
    row_width: f32,
) {
    ui.set_min_width(row_width);
    ui.set_max_width(row_width);
    ui.spacing_mut().item_spacing.y = 2.0;

    for entry in entries {
        match entry {
            UiDropdownMenuEntry::Action(action) => {
                let mut label = String::new();
                if action.selected {
                    label.push_str("✓ ");
                }
                label.push_str(&action.label);
                if let Some(shortcut) = action.shortcut.as_deref() {
                    label.push_str("    ");
                    label.push_str(shortcut);
                }
                let button = egui::Button::new(label).frame(false);
                let response = ui.add_enabled_ui(action.enabled, |ui| {
                    ui.add_sized([row_width - 8.0, 32.0], button)
                });
                if response.inner.clicked() && action.enabled {
                    *selected_action = Some(action.id);
                    ui.close();
                }
            }
            UiDropdownMenuEntry::Separator => {
                ui.add_space(4.0);
                let _ = ui.components().separator();
                ui.add_space(4.0);
            }
            UiDropdownMenuEntry::Submenu(submenu) => {
                let button = ui.add_sized(
                    [row_width - 8.0, 32.0],
                    egui::Button::new(&submenu.label).frame(false),
                );
                let _ = egui::containers::menu::SubMenu::new().show(ui, &button, |ui| {
                    render_runtime_dropdown_entries(
                        ui,
                        &submenu.entries,
                        selected_action,
                        row_width,
                    );
                });
            }
        }
    }
}

fn log_host_message(level: &str, message: &str) {
    let normalized = level.to_ascii_lowercase();
    match normalized.as_str() {
        "trace" => trace!("{message}"),
        "debug" => debug!("{message}"),
        "warn" => warn!("{message}"),
        "error" => error!("{message}"),
        _ => info!("{message}"),
    }
}

#[cfg(test)]
static TEST_VIRTUAL_LIST_INVOCATIONS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{
        pos2, vec2, widgets::text_edit::TextEditState, Event, Modifiers, PointerButton, Pos2,
        RawInput, Rect, Shape,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::Ordering,
        thread,
        time::{Duration, Instant},
    };
    use tempfile::TempDir;

    #[test]
    fn number_input_range_defaults_to_unbounded_when_bounds_are_omitted() {
        let range = number_input_range(&UiNumberInputOptions::default());

        assert_eq!(*range.start(), f32::MIN);
        assert_eq!(*range.end(), f32::MAX);
    }

    #[test]
    fn number_input_range_supports_single_sided_bounds() {
        let min_only = number_input_range(&UiNumberInputOptions {
            min: Some(-12.5),
            ..UiNumberInputOptions::default()
        });
        let max_only = number_input_range(&UiNumberInputOptions {
            max: Some(42.0),
            ..UiNumberInputOptions::default()
        });

        assert_eq!(*min_only.start(), -12.5);
        assert_eq!(*min_only.end(), f32::MAX);
        assert_eq!(*max_only.start(), f32::MIN);
        assert_eq!(*max_only.end(), 42.0);
    }

    #[test]
    fn renders_immediate_mode_script_without_panic() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    ui.begin_column({ gap = 12 })
                    ui.label("Hello from immediate host", {
                        weight = "Bold",
                        size = 20,
                    })
                    ui.button("demo.increment", "Increment", {
                        variant = "Primary",
                    })
                    ui.end_scope()
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);

        let texts = run_frame(&context, &mut app);
        run_frame(&context, &mut app);

        assert_eq!(app.runtime.last_compile_error(), None);
        assert_eq!(app.runtime.last_runtime_error(), None);
        assert!(texts.iter().any(|text| text == "Hello from immediate host"));
        assert!(texts.iter().any(|text| text == "Increment"));
    }

    #[test]
    fn increment_button_click_updates_count() {
        let script_path = default_script_path();
        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);

        let initial = run_frame(&context, &mut app);
        assert!(initial.iter().any(|text| text == "Count: 0"));

        let increment_center = find_text_center(
            &run_frame_output(&context, &mut app, RawInput::default()).shapes,
            "Increment",
        )
        .expect("expected Increment button text");

        run_frame_output(&context, &mut app, pointer_input(increment_center, true));
        let clicked = collect_rendered_texts(
            &run_frame_output(&context, &mut app, pointer_input(increment_center, false)).shapes,
        );

        assert!(
            clicked.iter().any(|text| text == "Count: 1"),
            "expected count to increment after clicking Increment; rendered texts: {clicked:?}"
        );
    }

    #[test]
    fn reload_button_click_does_not_increment_count() {
        let script_path = default_script_path();
        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);

        let initial = run_frame(&context, &mut app);
        assert!(initial.iter().any(|text| text == "Count: 0"));

        let reload_center = find_text_center(
            &run_frame_output(&context, &mut app, RawInput::default()).shapes,
            "Reload Script",
        )
        .expect("expected Reload Script button text");

        run_frame_output(&context, &mut app, pointer_input(reload_center, true));
        let clicked = collect_rendered_texts(
            &run_frame_output(&context, &mut app, pointer_input(reload_center, false)).shapes,
        );

        assert!(
            clicked.iter().any(|text| text == "Count: 0"),
            "did not expect reload click to increment count; rendered texts: {clicked:?}"
        );
        assert!(
            clicked.iter().any(|text| text == "Last event: reload"),
            "expected reload click to update last_event; rendered texts: {clicked:?}"
        );

        let post_reload = run_frame(&context, &mut app);
        assert!(
            post_reload.iter().any(|text| text == "Count: 0"),
            "did not expect deferred reload to increment count; rendered texts: {post_reload:?}"
        );
    }

    #[test]
    fn showcase_library_surface_renders_recipe_modules() {
        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(showcase_script_path());

        let rendered = run_frame(&context, &mut app);

        assert!(rendered.iter().any(|text| text == "Showcase Runtime Shell"));
        assert!(rendered.iter().any(|text| text == "Workspace Settings"));
        assert!(rendered.iter().any(|text| text == "Editor Sidebar"));
        assert!(rendered.iter().any(|text| text == "src/lib.rs"));
        assert_eq!(app.runtime.last_compile_error(), None);
        assert_eq!(app.runtime.last_runtime_error(), None);
    }

    #[test]
    fn component_gallery_surface_renders_initial_content_section() {
        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(component_gallery_script_path());

        let rendered = run_frame(&context, &mut app);

        assert!(rendered
            .iter()
            .any(|text| text == "Luau Primitive Component Gallery"));
        assert!(rendered.iter().any(|text| text == "Field"));
        assert!(rendered.iter().any(|text| text == "Material"));
        assert_eq!(app.runtime.last_compile_error(), None);
        assert_eq!(app.runtime.last_runtime_error(), None);
    }

    #[test]
    fn component_gallery_sidebar_switches_to_checkbox_preview() {
        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(component_gallery_script_path());

        run_frame(&context, &mut app);

        let inputs_center = find_text_center(
            &run_frame_output(&context, &mut app, RawInput::default()).shapes,
            "Checkbox",
        )
        .expect("expected Checkbox sidebar item");

        run_frame_output(&context, &mut app, pointer_input(inputs_center, true));
        let clicked = collect_rendered_texts(
            &run_frame_output(&context, &mut app, pointer_input(inputs_center, false)).shapes,
        );
        let post_click = run_frame(&context, &mut app);

        assert!(
            clicked.iter().any(|text| text == "Checkbox")
                || post_click.iter().any(|text| text == "Checkbox"),
            "expected component gallery to switch into the Checkbox preview; clicked={clicked:?} post_click={post_click:?}"
        );
        assert!(
            post_click.iter().any(|text| text == "Receive Shadows"),
            "expected checkbox preview after sidebar selection; rendered texts: {post_click:?}"
        );
    }

    #[test]
    fn library_backed_settings_form_identity_survives_reload() {
        let temp_dir = TempDir::new().unwrap();
        copy_example_runtime_library(temp_dir.path());
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                --!strict

                local settings_form = require("./ui/recipes/settings_form")
                local module = {}

                function module.render(state)
                    settings_form.render(state)
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path.clone());

        run_frame(&context, &mut app);

        let identity = WidgetIdentity::new(&app.surface_mount.surface_id, &["workspace"], "input");
        let text_id = egui_widget_id(&identity);
        let debug_path = identity.debug_path();

        context.memory_mut(|mem| mem.request_focus(text_id));
        run_frame(&context, &mut app);
        assert!(
            context.memory(|mem| mem.has_focus(text_id)),
            "expected pre-reload focus for {debug_path}"
        );

        write_script(
            &script_path,
            r#"
                --!strict

                local kit = require("./ui.luau")
                local settings_form = require("./ui/recipes/settings_form")
                local module = {}

                function module.render(state)
                    kit.text.meta("Reloaded settings form")
                    settings_form.render(state)
                end

                return module
            "#,
        );
        app.runtime.reload_queue().push(script_path);
        let rendered = run_frame(&context, &mut app);

        assert!(
            rendered.iter().any(|text| text == "Reloaded settings form"),
            "expected reloaded settings form marker; rendered texts: {rendered:?}"
        );
        assert!(
            context.memory(|mem| mem.has_focus(text_id)),
            "expected focus continuity after reload for {debug_path}"
        );
        assert!(
            TextEditState::load(&context, text_id).is_some(),
            "expected persisted text edit state after reload for {debug_path}"
        );
    }

    #[test]
    fn library_backed_editor_sidebar_uses_virtual_list_and_updates_selection() {
        let temp_dir = TempDir::new().unwrap();
        copy_example_runtime_library(temp_dir.path());
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                --!strict

                local editor_sidebar = require("./ui/recipes/editor_sidebar")
                local module = {}

                function module.render(state)
                    editor_sidebar.render(state)
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);
        reset_test_virtual_list_invocations();

        let initial = run_frame(&context, &mut app);
        assert!(initial.iter().any(|text| text == "Editor Sidebar"));
        assert!(initial
            .iter()
            .any(|text| text == "Focused file: src/lib.rs"));
        assert!(
            test_virtual_list_invocations() > 0,
            "expected library-backed editor sidebar to use the native virtual list primitive"
        );

        let target_file = "examples/runtime-luau/apps/demo.luau";
        let target_center = find_text_center(
            &run_frame_output(&context, &mut app, RawInput::default()).shapes,
            target_file,
        )
        .expect("expected library-backed virtual list row text");

        run_frame_output(&context, &mut app, pointer_input(target_center, true));
        let clicked = collect_rendered_texts(
            &run_frame_output(&context, &mut app, pointer_input(target_center, false)).shapes,
        );
        let post_click = run_frame(&context, &mut app);

        assert!(
            clicked.iter().any(|text| text == target_file),
            "expected clicked frame to render {target_file}; rendered texts: {clicked:?}"
        );
        assert!(
            post_click
                .iter()
                .any(|text| text == &format!("Focused file: {target_file}")),
            "expected library-backed editor sidebar selection to update; rendered texts: {post_click:?}"
        );
    }

    #[test]
    fn library_helpers_unwind_scopes_after_callback_errors() {
        let temp_dir = TempDir::new().unwrap();
        copy_example_runtime_library(temp_dir.path());
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                --!strict

                local kit = require("./ui")
                local module = {}

                function module.render(state)
                    state.frames = (state.frames or 0) + 1

                    local column_ok = pcall(function()
                        kit.stack.column(function()
                            kit.text.label("Inside broken column")
                            error("column boom")
                        end)
                    end)

                    local card_ok = pcall(function()
                        kit.card.surface(function()
                            kit.text.label("Inside broken card")
                            error("card boom")
                        end)
                    end)

                    local id_ok = pcall(function()
                        kit.stack.with_id("danger", function()
                            error("id boom")
                        end)
                    end)

                    kit.text.label("column_ok=" .. tostring(column_ok))
                    kit.text.label("card_ok=" .. tostring(card_ok))
                    kit.text.label("id_ok=" .. tostring(id_ok))
                    kit.text.label("After helper errors")
                    kit.text.label("Frames: " .. tostring(state.frames))
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);

        let first = run_frame(&context, &mut app);
        let second = run_frame(&context, &mut app);

        for rendered in [&first, &second] {
            assert!(rendered.iter().any(|text| text == "column_ok=false"));
            assert!(rendered.iter().any(|text| text == "card_ok=false"));
            assert!(rendered.iter().any(|text| text == "id_ok=false"));
            assert!(rendered.iter().any(|text| text == "After helper errors"));
        }
        assert!(second.iter().any(|text| text == "Frames: 2"));
        assert_eq!(app.runtime.last_compile_error(), None);
        assert_eq!(app.runtime.last_runtime_error(), None);
    }

    #[test]
    fn example_host_exposes_phase0_lifecycle_metrics() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    ui.label("Metrics")
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path.clone());

        run_frame(&context, &mut app);
        run_frame(&context, &mut app);

        let metrics = app.runtime.lifecycle_metrics();
        assert_eq!(metrics.load.attempts, 1);
        assert_eq!(metrics.load.successes, 1);
        assert_eq!(metrics.load.failures, 0);
        assert!(metrics.load.last_duration.is_some());
        assert_eq!(metrics.render.first_after_load.attempts, 1);
        assert_eq!(metrics.render.first_after_load.successes, 1);
        assert_eq!(metrics.render.steady_state.attempts, 1);
        assert_eq!(metrics.render.steady_state.successes, 1);
        eprintln!(
            "phase0_example_host_baseline load_us={} first_after_load_us={} steady_us={}",
            metrics
                .load
                .last_duration
                .expect("load duration captured")
                .as_micros(),
            metrics
                .render
                .first_after_load
                .last_duration
                .expect("first-after-load duration captured")
                .as_micros(),
            metrics
                .render
                .steady_state
                .last_duration
                .expect("steady render duration captured")
                .as_micros(),
        );
    }

    #[test]
    fn example_host_exposes_phase8_memory_diagnostics() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    state.lines = state.lines or { "Alpha", "Beta", "Gamma" }
                    for _, text in ipairs(state.lines) do
                        ui.label(text)
                    end
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);

        let first = run_frame(&context, &mut app);
        let second = run_frame(&context, &mut app);
        let metrics = app.runtime.memory_metrics();

        assert!(metrics.current_heap_bytes > 0);
        assert!(metrics.peak_heap_bytes >= metrics.current_heap_bytes);
        assert_eq!(metrics.frame.totals.attempts, 2);
        assert_eq!(metrics.gc.frame_step_attempts, 2);
        assert_eq!(metrics.leak_detection.sample_count, 1);
        assert!(metrics.leak_detection.last_sample.is_some());

        let rendered = [first, second].concat();
        assert!(rendered.iter().any(|text| text.contains("Heap bytes:")));
        assert!(rendered.iter().any(|text| text.contains("GC frame steps:")));
        assert!(rendered.iter().any(|text| text.contains("Leak sample:")));
    }

    #[test]
    fn virtual_list_click_updates_selection() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    state.selected_index = state.selected_index or 0
                    ui.label("Selected: " .. tostring(state.selected_index))
                    local selected_index, changed = ui.virtual_list("demo.files", {
                        "Cargo.toml",
                        "README.md",
                        "src/lib.rs",
                    }, {
                        height = 88,
                        row_height = 22,
                        selected_index = state.selected_index,
                    })
                    if changed then
                        state.selected_index = selected_index
                    end
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);

        let initial = run_frame(&context, &mut app);
        assert!(initial.iter().any(|text| text == "Selected: 0"));

        let item_center = find_text_center(
            &run_frame_output(&context, &mut app, RawInput::default()).shapes,
            "README.md",
        )
        .expect("expected virtual list row text");

        run_frame_output(&context, &mut app, pointer_input(item_center, true));
        let clicked = collect_rendered_texts(
            &run_frame_output(&context, &mut app, pointer_input(item_center, false)).shapes,
        );
        let post_click = run_frame(&context, &mut app);

        assert!(
            clicked.iter().any(|text| text == "README.md"),
            "expected clicked frame to render the selected row text; rendered texts: {clicked:?}"
        );
        assert!(
            post_click.iter().any(|text| text == "Selected: 2"),
            "expected virtual list click to update selection on the next frame; rendered texts: {post_click:?}"
        );
    }

    #[test]
    fn leaf_file_reload_updates_rendered_text() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        let leaf_path = temp_dir.path().join("panel.luau");
        write_script(
            &leaf_path,
            r#"
                local module = {}

                function module.render()
                    ui.label("Panel v1")
                end

                return module
            "#,
        );
        write_script(
            &script_path,
            r#"
                local panel = require("./panel")
                local module = {}

                function module.render(state)
                    panel.render(state)
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path.clone());

        let first = run_frame(&context, &mut app);
        assert!(first.iter().any(|text| text == "Panel v1"));

        write_script(
            &leaf_path,
            r#"
                local module = {}

                function module.render()
                    ui.label("Panel v2")
                end

                return module
            "#,
        );
        app.runtime.reload_queue().push(leaf_path);
        let second = run_frame(&context, &mut app);

        assert!(second.iter().any(|text| text == "Panel v2"));
    }

    #[test]
    fn denied_reload_capability_surfaces_runtime_error_and_keeps_host_healthy() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    app.request_reload()
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);
        app.surface_mount = SurfaceMount::new(
            SurfaceId::new("runtime_egui_host.denied_reload"),
            SurfaceCapabilities {
                log: true,
                reload: false,
                repaint: true,
                ..SurfaceCapabilities::none()
            },
        );

        run_frame(&context, &mut app);
        let _ = run_frame(&context, &mut app);

        assert_eq!(app.pending_reload, false);
        assert_eq!(
            app.runtime.last_runtime_error(),
            Some(
                "backend error: surface `runtime_egui_host.denied_reload` is not allowed to call `app.request_reload` because capability `reload` is disabled"
            )
        );
    }

    #[test]
    fn deferred_reload_applies_only_after_successful_frame() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.update(state, input)
                    app.request_reload()
                end

                function module.render(state)
                    ui.label("Frame succeeded")
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);

        let rendered = run_frame(&context, &mut app);
        assert!(rendered.iter().any(|text| text == "Frame succeeded"));
        assert_eq!(app.pending_reload, false);

        let rendered = run_frame(&context, &mut app);
        assert!(rendered.iter().any(|text| text == "Frame succeeded"));
        assert_eq!(app.pending_reload, true);
        assert_eq!(app.runtime.last_runtime_error(), None);
    }

    #[test]
    fn deferred_reload_is_dropped_when_render_fails() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.update(state, input)
                    app.request_reload()
                end

                function module.render(state)
                    state.frames = (state.frames or 0) + 1
                    if state.frames > 1 then
                        error("boom")
                    end
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);

        let _ = run_frame(&context, &mut app);
        let _ = run_frame(&context, &mut app);

        assert_eq!(app.pending_reload, false);
        assert!(app
            .runtime
            .last_runtime_error()
            .is_some_and(|message| message.contains("boom")));
    }

    #[test]
    fn watcher_reloads_after_leaf_file_change() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        let leaf_path = temp_dir.path().join("panel.luau");
        write_script(
            &leaf_path,
            r#"
                local module = {}

                function module.render()
                    ui.label("Watcher v1")
                end

                return module
            "#,
        );
        write_script(
            &script_path,
            r#"
                local panel = require("./panel")
                local module = {}

                function module.render(state)
                    panel.render(state)
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path.clone());

        run_frame(&context, &mut app);
        write_script(
            &leaf_path,
            r#"
                local module = {}

                function module.render()
                    ui.label("Watcher v2")
                end

                return module
            "#,
        );

        wait_for(&context, &mut app, Duration::from_secs(3), |app, texts| {
            app.runtime.last_compile_error().is_none()
                && texts.iter().any(|text| text == "Watcher v2")
        });
    }

    #[test]
    fn broken_leaf_reload_keeps_previous_ui_and_recovers_after_fix() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        let leaf_path = temp_dir.path().join("panel.luau");
        write_script(
            &leaf_path,
            r#"
                local module = {}

                function module.render()
                    ui.label("Stable panel")
                end

                return module
            "#,
        );
        write_script(
            &script_path,
            r#"
                local panel = require("./panel")
                local module = {}

                function module.render(state)
                    panel.render(state)
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);

        let first = run_frame(&context, &mut app);
        assert!(first.iter().any(|text| text == "Stable panel"));

        write_script(
            &leaf_path,
            r#"
                local module = {
            "#,
        );
        app.runtime.reload_queue().push(leaf_path.clone());
        let broken = run_frame(&context, &mut app);

        assert!(broken.iter().any(|text| text == "Stable panel"));
        assert!(app.runtime.last_compile_error().is_some());

        write_script(
            &leaf_path,
            r#"
                local module = {}

                function module.render()
                    ui.label("Recovered panel")
                end

                return module
            "#,
        );
        app.runtime.reload_queue().push(leaf_path);
        let recovered = run_frame(&context, &mut app);

        assert!(recovered.iter().any(|text| text == "Recovered panel"));
        assert_eq!(app.runtime.last_compile_error(), None);
    }

    #[test]
    fn failed_reload_candidate_rolls_back_on_next_frame_and_overlay_shows_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    ui.label("Stable panel")
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path.clone());

        let first = run_frame(&context, &mut app);
        assert!(first.iter().any(|text| text == "Stable panel"));

        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    ui.label("Candidate panel")
                    error("candidate render boom")
                end

                return module
            "#,
        );
        app.runtime.reload_queue().push(script_path.clone());

        let failing = run_frame(&context, &mut app);
        let failure = app
            .runtime
            .last_failure()
            .expect("candidate failure recorded");
        assert_eq!(failure.stage.as_str(), "first_after_reload");
        assert!(failure.rolled_back);
        assert_eq!(failure.class.as_str(), "render");
        assert_eq!(
            failure.module_context.as_deref(),
            Some(script_path.display().to_string().as_str())
        );
        assert!(!failing.iter().any(|text| text == "Stable panel"));

        wait_for(&context, &mut app, Duration::from_secs(1), |app, texts| {
            app.runtime
                .last_failure()
                .is_some_and(|failure| failure.rolled_back)
                && texts.iter().any(|text| text == "Stable panel")
        });

        let recovered = run_frame(&context, &mut app);
        assert!(recovered.iter().any(|text| text == "Stable panel"));
        assert!(app
            .runtime
            .last_failure()
            .is_some_and(|failure| failure.rolled_back));
    }

    #[test]
    fn failed_reload_candidate_keeps_library_backed_surface_interactive() {
        let temp_dir = TempDir::new().unwrap();
        copy_example_runtime_library(temp_dir.path());
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                --!strict

                local profile_panel = require("./ui/recipes/profile_panel")
                local module = {}

                function module.init(state)
                    state.name = state.name or "Luau"
                    state.count = state.count or 0
                    state.last_event = state.last_event or "boot"
                    state.reloads = state.reloads or 0
                    state.variant = state.variant or "library-panel"
                end

                function module.render(state)
                    profile_panel.render(state)
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path.clone());

        let initial = run_frame(&context, &mut app);
        assert!(
            initial.iter().any(|text| text.contains("Count: 0")),
            "expected initial library-backed panel render; rendered texts: {initial:?}"
        );

        let increment_center = {
            let output = run_frame_output(&context, &mut app, RawInput::default());
            find_text_center(&output.shapes, "Increment").expect("expected increment button text")
        };

        run_frame_output(&context, &mut app, pointer_input(increment_center, true));
        let clicked = collect_rendered_texts(
            &run_frame_output(&context, &mut app, pointer_input(increment_center, false)).shapes,
        );
        let post_click = run_frame(&context, &mut app);
        assert!(
            clicked.iter().any(|text| text.contains("Count: 1"))
                || post_click.iter().any(|text| text.contains("Count: 1")),
            "expected first increment to update the library-backed panel; clicked texts: {clicked:?}; post-click texts: {post_click:?}"
        );

        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    ui.label("Candidate panel")
                    error("candidate render boom")
                end

                return module
            "#,
        );
        app.runtime.reload_queue().push(script_path.clone());
        wait_for(&context, &mut app, Duration::from_secs(1), |app, texts| {
            app.runtime
                .last_failure()
                .is_some_and(|failure| failure.rolled_back)
                && texts.iter().any(|text| text.contains("Count: 1"))
        });

        let recovered = run_frame(&context, &mut app);
        assert!(
            recovered.iter().any(|text| text.contains("Count: 1")),
            "expected the last-known-good library-backed UI to recover after failed reload; rendered texts: {recovered:?}"
        );

        let settled_recovered = run_frame(&context, &mut app);
        assert!(
            settled_recovered
                .iter()
                .any(|text| text.contains("Count: 1")),
            "expected recovered library-backed UI to remain stable on the next frame; rendered texts: {settled_recovered:?}"
        );

        let identity = WidgetIdentity::new(
            &app.surface_mount.surface_id,
            &["profile", "name_field"],
            "input",
        );
        let text_id = egui_widget_id(&identity);
        context.memory_mut(|mem| mem.request_focus(text_id));
        let focused = run_frame(&context, &mut app);

        assert!(
            context.memory(|mem| mem.has_focus(text_id)),
            "expected recovered surface to accept focus on the profile name field; rendered texts: {focused:?}"
        );
        assert!(
            TextEditState::load(&context, text_id).is_some(),
            "expected recovered text input state to remain available after interaction"
        );
    }

    #[test]
    fn initial_candidate_failure_leaves_no_render_target_and_overlay_shows_hook_context() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.init(state)
                    error("init boom")
                end

                function module.render(state)
                    ui.label("Should not render")
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path.clone());

        let first = run_frame(&context, &mut app);
        assert!(first.iter().any(|text| text == "No Luau script loaded"));
        let failure = app
            .runtime
            .last_failure()
            .expect("initial failure recorded");
        assert_eq!(failure.class.as_str(), "init_reload");
        assert_eq!(failure.hook_or_api.as_deref(), Some("init"));
        assert!(!failure.rolled_back);
        assert_eq!(app.runtime.has_render_target(), false);
        assert_eq!(
            failure.module_context.as_deref(),
            Some(script_path.display().to_string().as_str())
        );
    }

    #[test]
    fn duplicate_dirty_entries_reload_once_per_frame_boundary() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        let leaf_path = temp_dir.path().join("panel.luau");
        write_script(
            &leaf_path,
            r#"
                local module = {}
                local module_state = nil

                function module.init(state)
                    module_state = state
                    state.reloads = state.reloads or 0
                end

                function module.reload(old_exports, state)
                    module_state = state
                    state.reloads = (state.reloads or 0) + 1
                end

                function module.render()
                    ui.label("Panel v1 / reloads=" .. tostring(module_state.reloads))
                end

                return module
            "#,
        );
        write_script(
            &script_path,
            r#"
                local panel = require("./panel")
                local module = {}

                function module.render(state)
                    panel.render(state)
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);

        let first = run_frame(&context, &mut app);
        assert!(first.iter().any(|text| text == "Panel v1 / reloads=0"));

        write_script(
            &leaf_path,
            r#"
                local module = {}
                local module_state = nil

                function module.init(state)
                    module_state = state
                    state.reloads = state.reloads or 0
                end

                function module.reload(old_exports, state)
                    module_state = state
                    state.reloads = (state.reloads or 0) + 1
                end

                function module.render()
                    ui.label("Panel v2 / reloads=" .. tostring(module_state.reloads))
                end

                return module
            "#,
        );

        app.runtime.reload_queue().push(leaf_path.clone());
        app.runtime.reload_queue().push(leaf_path.clone());
        app.runtime.reload_queue().push(leaf_path);
        let second = run_frame(&context, &mut app);

        assert!(second.iter().any(|text| text == "Panel v2 / reloads=1"));
        assert_eq!(app.runtime.reload_metrics().last_dirty_path_count, 1);
        assert_eq!(app.runtime.reload_metrics().last_affected_path_count, 2);
        assert_eq!(app.runtime.reload_metrics().last_rebuilt_module_count, 2);
    }

    #[test]
    fn text_edit_identity_survives_rerender_with_stable_id_path() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    state.name = state.name or "Andy"
                    ui.push_id("profile")
                    state.name = select(1, ui.text_edit("name", state.name, {
                        placeholder = "Name",
                        width = 220,
                    }))
                    ui.pop_id()
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path);

        run_frame(&context, &mut app);

        let identity = WidgetIdentity::new(&app.surface_mount.surface_id, &["profile"], "name");
        let text_id = egui_widget_id(&identity);
        let debug_path = identity.debug_path();

        assert!(
            TextEditState::load(&context, text_id).is_some(),
            "expected persisted text edit state for {debug_path}"
        );

        context.memory_mut(|mem| mem.request_focus(text_id));
        assert!(
            context.memory(|mem| mem.has_focus(text_id)),
            "expected requested focus for {debug_path}"
        );

        run_frame(&context, &mut app);

        assert!(
            context.memory(|mem| mem.has_focus(text_id)),
            "expected focus continuity for {debug_path}"
        );
        assert!(
            TextEditState::load(&context, text_id).is_some(),
            "expected persisted text edit state after rerender for {debug_path}"
        );
    }

    #[test]
    fn text_edit_identity_survives_reload_with_stable_id_path() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    state.name = state.name or "Andy"
                    ui.push_id("profile")
                    ui.label("Version A")
                    state.name = select(1, ui.text_edit("name", state.name, {
                        placeholder = "Name",
                        width = 220,
                    }))
                    ui.pop_id()
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path.clone());

        run_frame(&context, &mut app);

        let identity = WidgetIdentity::new(&app.surface_mount.surface_id, &["profile"], "name");
        let text_id = egui_widget_id(&identity);
        let debug_path = identity.debug_path();

        context.memory_mut(|mem| mem.request_focus(text_id));
        run_frame(&context, &mut app);
        assert!(
            context.memory(|mem| mem.has_focus(text_id)),
            "expected pre-reload focus for {debug_path}"
        );

        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    state.name = state.name or "Andy"
                    ui.push_id("profile")
                    ui.label("Version B")
                    state.name = select(1, ui.text_edit("name", state.name, {
                        placeholder = "Name",
                        width = 220,
                    }))
                    ui.pop_id()
                end

                return module
            "#,
        );
        app.runtime.reload_queue().push(script_path);
        run_frame(&context, &mut app);

        assert!(
            context.memory(|mem| mem.has_focus(text_id)),
            "expected focus continuity after reload for {debug_path}"
        );
        assert!(
            TextEditState::load(&context, text_id).is_some(),
            "expected persisted text edit state after reload for {debug_path}"
        );
        assert_eq!(app.runtime.last_compile_error(), None);
        assert_eq!(app.runtime.last_runtime_error(), None);
    }

    #[test]
    fn text_edit_identity_breakage_does_not_transfer_focus_to_new_path() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    state.name = state.name or "Andy"
                    ui.push_id("profile")
                    state.name = select(1, ui.text_edit("name", state.name, {
                        placeholder = "Name",
                        width = 220,
                    }))
                    ui.pop_id()
                end

                return module
            "#,
        );

        let context = Context::default();
        install_context(&context);
        let mut app = RuntimeEguiHostApp::new(script_path.clone());

        run_frame(&context, &mut app);

        let old_identity = WidgetIdentity::new(&app.surface_mount.surface_id, &["profile"], "name");
        let old_id = egui_widget_id(&old_identity);
        let old_debug_path = old_identity.debug_path();
        context.memory_mut(|mem| mem.request_focus(old_id));
        run_frame(&context, &mut app);
        assert!(
            context.memory(|mem| mem.has_focus(old_id)),
            "expected pre-reload focus for {old_debug_path}"
        );

        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    state.name = state.name or "Andy"
                    ui.push_id("settings")
                    state.name = select(1, ui.text_edit("name", state.name, {
                        placeholder = "Name",
                        width = 220,
                    }))
                    ui.pop_id()
                end

                return module
            "#,
        );
        app.runtime.reload_queue().push(script_path);
        run_frame(&context, &mut app);

        let new_identity =
            WidgetIdentity::new(&app.surface_mount.surface_id, &["settings"], "name");
        let new_id = egui_widget_id(&new_identity);
        let new_debug_path = new_identity.debug_path();
        assert_ne!(old_id, new_id);
        assert!(
            TextEditState::load(&context, new_id).is_some(),
            "expected persisted text edit state for {new_debug_path}"
        );
        assert!(
            !context.memory(|mem| mem.has_focus(new_id)),
            "did not expect focus continuity for new id path {new_debug_path}"
        );
    }

    fn showcase_script_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("runtime-luau")
            .join("apps")
            .join("showcase.luau")
    }

    fn component_gallery_script_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("runtime-luau")
            .join("apps")
            .join("gallery.luau")
    }

    fn copy_example_runtime_library(destination: &Path) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("runtime-luau");
        copy_path_tree(&root.join(".luaurc"), &destination.join(".luaurc"));
        copy_path_tree(&root.join("ui.luau"), &destination.join("ui.luau"));
        copy_path_tree(&root.join("ui"), &destination.join("ui"));
        copy_path_tree(&root.join("apps"), &destination.join("apps"));
    }

    fn copy_path_tree(source: &Path, destination: &Path) {
        if source.is_dir() {
            fs::create_dir_all(destination).unwrap();
            for entry in fs::read_dir(source).unwrap() {
                let entry = entry.unwrap();
                copy_path_tree(&entry.path(), &destination.join(entry.file_name()));
            }
            return;
        }

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::copy(source, destination).unwrap();
    }

    fn write_script(path: &Path, source: &str) {
        fs::write(path, source).unwrap();
    }

    fn run_frame(context: &Context, app: &mut RuntimeEguiHostApp) -> Vec<String> {
        let output = run_frame_output(context, app, RawInput::default());
        collect_rendered_texts(&output.shapes)
    }

    fn run_frame_output(
        context: &Context,
        app: &mut RuntimeEguiHostApp,
        mut input: RawInput,
    ) -> egui::FullOutput {
        input.screen_rect = Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1360.0, 940.0)));
        context.run(input, |ctx| update(app, ctx))
    }

    fn wait_for(
        context: &Context,
        app: &mut RuntimeEguiHostApp,
        timeout: Duration,
        predicate: impl Fn(&RuntimeEguiHostApp, &[String]) -> bool,
    ) {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let texts = run_frame(context, app);
            if predicate(app, &texts) {
                return;
            }
            thread::sleep(Duration::from_millis(25));
        }

        panic!("timed out waiting for watcher-driven reload");
    }

    fn collect_rendered_texts(shapes: &[egui::epaint::ClippedShape]) -> Vec<String> {
        let mut rendered = Vec::new();
        for shape in shapes {
            collect_texts_from_shape(&shape.shape, &mut rendered);
        }
        rendered
    }

    fn collect_texts_from_shape(shape: &Shape, rendered: &mut Vec<String>) {
        match shape {
            Shape::Text(text) => rendered.push(text.galley.job.text.clone()),
            Shape::Vec(shapes) => {
                for shape in shapes {
                    collect_texts_from_shape(shape, rendered);
                }
            }
            _ => {}
        }
    }

    fn find_text_center(shapes: &[egui::epaint::ClippedShape], target: &str) -> Option<Pos2> {
        let mut text_rect = Rect::NOTHING;
        for clipped_shape in shapes {
            collect_text_rect(&clipped_shape.shape, target, &mut text_rect);
        }
        text_rect.is_positive().then(|| text_rect.center())
    }

    fn collect_text_rect(shape: &Shape, target: &str, text_rect: &mut Rect) {
        match shape {
            Shape::Text(text_shape) if text_shape.galley.job.text.contains(target) => {
                *text_rect = text_rect.union(text_shape.visual_bounding_rect());
            }
            Shape::Vec(shapes) => {
                for shape in shapes {
                    collect_text_rect(shape, target, text_rect);
                }
            }
            _ => {}
        }
    }

    fn pointer_input(position: Pos2, pressed: bool) -> RawInput {
        RawInput {
            events: vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..Default::default()
        }
    }

    fn reset_test_virtual_list_invocations() {
        TEST_VIRTUAL_LIST_INVOCATIONS.store(0, Ordering::Relaxed);
    }

    fn test_virtual_list_invocations() -> usize {
        TEST_VIRTUAL_LIST_INVOCATIONS.load(Ordering::Relaxed)
    }
}
