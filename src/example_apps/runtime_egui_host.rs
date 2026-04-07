use std::{marker::PhantomData, path::PathBuf, ptr::NonNull};

use crate::{
    components::{
        Button, ButtonVariant, Card, ComponentUiExt, ControlSize, Label, LabelTone, LabelWeight,
        TextInput,
    },
    theme::{self, BaseColor, ThemeMode, ThemeSpec},
};
use egui::{CentralPanel, Context, Id, ScrollArea, Window};
use log::{debug, error, info, trace, warn};
use luau_runtime_core::{
    RootScriptWatcher, RuntimeConfig, RuntimeError, RuntimeHost, ScriptRuntime, ScriptSource,
    UiButtonOptions, UiButtonVariant, UiCardOptions, UiContainerOptions, UiControlSize,
    UiLabelOptions, UiLabelTone, UiLabelWeight, UiTextEditOptions, UiTextEditOutput,
    UiWindowOptions,
};

pub const WINDOW_TITLE: &str = "Luau Runtime egui Host";
pub const WINDOW_INNER_SIZE: [f32; 2] = [1360.0, 940.0];

pub struct RuntimeEguiHostApp {
    runtime: ScriptRuntime,
    root_path: PathBuf,
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
        Self {
            runtime: ScriptRuntime::new(RuntimeConfig::default()),
            root_path: root_path.into(),
            reload_watcher: None,
            initial_load_attempted: false,
            watcher_attempted: false,
            pending_reload: false,
            repaint_requested: false,
            host_error: None,
        }
    }

    fn process_frame_boundary(&mut self, ctx: &Context) {
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

        let mut host = RuntimeControlHost::new(ctx, &mut self.pending_reload, &mut self.repaint_requested);
        let had_root = self.runtime.config().root_source.is_some();
        let result = if had_root {
            self.runtime.reload_now_with_host(&mut host)
        } else {
            self.runtime
                .load_root_with_host(ScriptSource::path(self.root_path.clone()), &mut host)
        };
        let metrics = self.runtime.reload_metrics().clone();
        let duration_ms = metrics
            .last_duration
            .map(|duration| duration.as_millis())
            .unwrap_or(0);

        match result {
            Ok(()) => {
                info!(
                    "runtime_egui_host action={} script={} rebuilt={} affected={} dirty={} duration_ms={}",
                    if had_root { "reload" } else { "load" },
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
                info!("runtime_egui_host watching {}", watcher.watched_dir().display());
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

    fn render_script_surface(&mut self, ui: &mut egui::Ui) {
        if self.runtime.config().root_source.is_none() {
            ui.heading("No Luau script loaded");
            ui.label("Fix the script file and save to trigger reload.");
            return;
        }

        let ctx = ui.ctx().clone();
        let mut host = FrameUiHost::new(
            &ctx,
            ui,
            &mut self.pending_reload,
            &mut self.repaint_requested,
        );

        if let Err(err) = self.runtime.render_frame_with_host(&mut host) {
            warn!("runtime_egui_host render_failed error={err}");
        }
    }

    fn show_error_overlay(&mut self, ctx: &Context) {
        let host_error = self.host_error.as_deref();
        let compile_error = self.runtime.last_compile_error();
        let runtime_error = self.runtime.last_runtime_error();
        if host_error.is_none() && compile_error.is_none() && runtime_error.is_none() {
            return;
        }

        Window::new("Luau Runtime Error")
            .id(Id::new("runtime_egui_host.error_overlay"))
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
                    if compile_error.is_some() || runtime_error.is_some() {
                        ui.separator();
                    }
                }

                if let Some(error) = compile_error {
                    ui.label("Compile error");
                    ui.code(error);
                    if runtime_error.is_some() {
                        ui.separator();
                    }
                }

                if let Some(error) = runtime_error {
                    ui.label("Runtime error");
                    ui.code(error);
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
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            app.render_script_surface(ui);
        });
    });

    app.show_error_overlay(ctx);

    if app.repaint_requested {
        app.repaint_requested = false;
        ctx.request_repaint();
    }
}

fn default_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("runtime-luau")
        .join("demo.luau")
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

impl RuntimeHost for RuntimeControlHost<'_> {
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

    fn label(&mut self, _text: &str, _options: UiLabelOptions) -> Result<(), RuntimeError> {
        Err(RuntimeError::backend(
            "ui.* is only available during the frame render call".to_owned(),
        ))
    }

    fn separator(&mut self) -> Result<(), RuntimeError> {
        Err(RuntimeError::backend(
            "ui.* is only available during the frame render call".to_owned(),
        ))
    }

    fn button(
        &mut self,
        _id: &str,
        _text: &str,
        _options: UiButtonOptions,
    ) -> Result<bool, RuntimeError> {
        Err(RuntimeError::backend(
            "ui.* is only available during the frame render call".to_owned(),
        ))
    }

    fn text_edit(
        &mut self,
        _id: &str,
        _value: &str,
        _options: UiTextEditOptions,
    ) -> Result<UiTextEditOutput, RuntimeError> {
        Err(RuntimeError::backend(
            "ui.* is only available during the frame render call".to_owned(),
        ))
    }

    fn horizontal(
        &mut self,
        _options: UiContainerOptions,
        _body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        Err(RuntimeError::backend(
            "ui.* is only available during the frame render call".to_owned(),
        ))
    }

    fn vertical(
        &mut self,
        _options: UiContainerOptions,
        _body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        Err(RuntimeError::backend(
            "ui.* is only available during the frame render call".to_owned(),
        ))
    }

    fn card(
        &mut self,
        _options: UiCardOptions,
        _body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        Err(RuntimeError::backend(
            "ui.* is only available during the frame render call".to_owned(),
        ))
    }

    fn window(
        &mut self,
        _id: &str,
        _title: &str,
        _options: UiWindowOptions,
        _body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        Err(RuntimeError::backend(
            "ui.* is only available during the frame render call".to_owned(),
        ))
    }
}

struct FrameUiHost<'a> {
    ctx: &'a Context,
    pending_reload: &'a mut bool,
    repaint_requested: &'a mut bool,
    current_ui: NonNull<egui::Ui>,
    marker: PhantomData<&'a mut egui::Ui>,
}

impl<'a> FrameUiHost<'a> {
    fn new(
        ctx: &'a Context,
        ui: &'a mut egui::Ui,
        pending_reload: &'a mut bool,
        repaint_requested: &'a mut bool,
    ) -> Self {
        Self {
            ctx,
            pending_reload,
            repaint_requested,
            current_ui: NonNull::from(ui),
            marker: PhantomData,
        }
    }

    fn with_active_ui<R>(&mut self, f: impl FnOnce(&mut egui::Ui) -> R) -> R {
        let mut ui = self.current_ui;
        unsafe { f(ui.as_mut()) }
    }
}

struct UiScopeGuard<'a> {
    current_ui: &'a mut NonNull<egui::Ui>,
    previous: NonNull<egui::Ui>,
}

impl Drop for UiScopeGuard<'_> {
    fn drop(&mut self) {
        *self.current_ui = self.previous;
    }
}

fn run_scoped_body(
    current_ui: &mut NonNull<egui::Ui>,
    ui: &mut egui::Ui,
    body: &mut dyn FnMut() -> Result<(), RuntimeError>,
) -> Result<(), RuntimeError> {
    let previous = *current_ui;
    *current_ui = NonNull::from(ui);
    let _guard = UiScopeGuard {
        current_ui,
        previous,
    };
    body()
}

impl RuntimeHost for FrameUiHost<'_> {
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
        let clicked = self.with_active_ui(|ui| {
            ui.push_id(id, |ui| {
                let mut components = ui.components();
                let mut button = Button::new(text)
                    .variant(map_button_variant(options.variant))
                    .size(map_control_size(options.size));
                if let Some(width) = options.width {
                    button = button.min_size(egui::vec2(width.max(1.0), 0.0));
                }
                components.button(button).clicked()
            })
            .inner
        });
        Ok(clicked)
    }

    fn text_edit(
        &mut self,
        id: &str,
        value: &str,
        options: UiTextEditOptions,
    ) -> Result<UiTextEditOutput, RuntimeError> {
        let output = self.with_active_ui(|ui| {
            ui.push_id(id, |ui| {
                let mut components = ui.components();
                let mut value = value.to_owned();
                let mut input = TextInput::new();
                if let Some(width) = options.width {
                    input = input.width(width);
                }
                if let Some(placeholder) = options.placeholder.as_deref() {
                    input = input.placeholder(placeholder);
                }
                if options.password {
                    input = input.password(true);
                }
                let response = components.text_input(&mut value, input);
                UiTextEditOutput {
                    value,
                    changed: response.changed(),
                }
            })
            .inner
        });
        Ok(output)
    }

    fn horizontal(
        &mut self,
        options: UiContainerOptions,
        body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let current_ui = &mut self.current_ui as *mut NonNull<egui::Ui>;
        self.with_active_ui(|ui| {
            ui.scope(|ui| {
                if let Some(gap) = options.gap {
                    ui.spacing_mut().item_spacing.x = gap;
                }
                ui.horizontal(|ui| unsafe { run_scoped_body(&mut *current_ui, ui, body) })
                    .inner
            })
            .inner
        })
    }

    fn vertical(
        &mut self,
        options: UiContainerOptions,
        body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let current_ui = &mut self.current_ui as *mut NonNull<egui::Ui>;
        self.with_active_ui(|ui| {
            ui.scope(|ui| {
                if let Some(gap) = options.gap {
                    ui.spacing_mut().item_spacing.y = gap;
                }
                ui.vertical(|ui| unsafe { run_scoped_body(&mut *current_ui, ui, body) })
                    .inner
            })
            .inner
        })
    }

    fn card(
        &mut self,
        options: UiCardOptions,
        body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let current_ui = &mut self.current_ui as *mut NonNull<egui::Ui>;
        self.with_active_ui(|ui| {
            let mut components = ui.components();
            components
                .card(
                    Card::new().padding(options.padding_x as i8, options.padding_y as i8),
                    |ui| {
                        if let Some(width) = options.width {
                            ui.set_min_width(width);
                            ui.set_max_width(width);
                        }
                        unsafe { run_scoped_body(&mut *current_ui, ui, body) }
                    },
                )
                .inner
        })
    }

    fn window(
        &mut self,
        id: &str,
        title: &str,
        options: UiWindowOptions,
        body: &mut dyn FnMut() -> Result<(), RuntimeError>,
    ) -> Result<(), RuntimeError> {
        let mut window = Window::new(title).id(Id::new(id));
        if let Some(width) = options.width {
            window = window.default_width(width);
        }
        if let Some(height) = options.height {
            window = window.default_height(height);
        }

        let current_ui = &mut self.current_ui as *mut NonNull<egui::Ui>;
        if let Some(response) = window.show(
            self.ctx,
            |ui| unsafe { run_scoped_body(&mut *current_ui, ui, body) },
        ) {
            if let Some(result) = response.inner {
                result?;
            }
        }
        Ok(())
    }
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
mod tests {
    use super::*;
    use egui::{pos2, vec2, RawInput, Rect, Shape};
    use std::{
        fs,
        path::Path,
        thread,
        time::{Duration, Instant},
    };
    use tempfile::TempDir;

    #[test]
    fn renders_immediate_mode_script_without_panic() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("app.luau");
        write_script(
            &script_path,
            r#"
                local module = {}

                function module.render(state)
                    ui.vertical({ gap = 12 }, function()
                        ui.label("Hello from immediate host", {
                            weight = "Bold",
                            size = 20,
                        })
                        ui.button("demo.increment", "Increment", {
                            variant = "Primary",
                        })
                    end)
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
        assert!(app.runtime.reload_metrics().last_dirty_path_count >= 3);
        assert_eq!(app.runtime.reload_metrics().last_affected_path_count, 2);
        assert_eq!(app.runtime.reload_metrics().last_rebuilt_module_count, 2);
    }

    fn write_script(path: &Path, source: &str) {
        fs::write(path, source).unwrap();
    }

    fn run_frame(context: &Context, app: &mut RuntimeEguiHostApp) -> Vec<String> {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1360.0, 940.0))),
            ..Default::default()
        };
        let output = context.run(input, |ctx| update(app, ctx));
        collect_rendered_texts(&output.shapes)
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
}
