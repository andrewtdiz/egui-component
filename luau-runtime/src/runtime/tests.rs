use std::{collections::BTreeMap, fs, io::Write, path::PathBuf};

use mlua::Value;
use tempfile::{NamedTempFile, TempDir};

use super::*;

const TEST_SURFACE_ID: &str = "tests.runtime";

#[derive(Debug)]
struct RecordingHost {
    surface_id: SurfaceId,
    events: Vec<String>,
    logs: Vec<(String, String)>,
    reload_requests: usize,
    repaint_requests: usize,
    panic_log: bool,
    panic_label: bool,
    panic_request_reload: bool,
    panic_request_repaint: bool,
    fail_request_reload: Option<RuntimeError>,
    fail_request_repaint: Option<RuntimeError>,
    labels: Vec<(String, UiLabelOptions)>,
    buttons: Vec<(String, String, UiButtonOptions)>,
    text_edits: Vec<(String, String, UiTextEditOptions)>,
    checkboxes: Vec<(String, bool, UiCheckboxOptions)>,
    switches: Vec<(String, bool, UiSwitchOptions)>,
    sliders: Vec<(String, f32, UiSliderOptions)>,
    number_inputs: Vec<(String, f32, UiNumberInputOptions)>,
    selects: Vec<(String, usize, Vec<String>, UiSelectOptions)>,
    tabs: Vec<(String, usize, Vec<UiTabOption>, UiTabsOptions)>,
    progresses: Vec<(f32, UiProgressOptions)>,
    radios: Vec<(String, bool, UiRadioOptions)>,
    button_groups: Vec<(String, Vec<String>, UiButtonGroupOptions)>,
    collapsibles: Vec<(String, String, bool, UiCollapsibleOptions)>,
    dropdown_menus: Vec<(
        String,
        String,
        Vec<UiDropdownMenuEntry>,
        UiDropdownMenuOptions,
    )>,
    tooltips: Vec<(String, String, UiTooltipOptions)>,
    spinners: Vec<UiSpinnerOptions>,
    skeletons: Vec<UiSkeletonOptions>,
    virtual_lists: Vec<(String, Vec<String>, UiVirtualListOptions)>,
    boolean_results: BTreeMap<String, UiBooleanOutput>,
    number_results: BTreeMap<String, UiNumberOutput>,
    select_results: BTreeMap<String, UiSelectOutput>,
    tabs_results: BTreeMap<String, UiTabsOutput>,
    button_group_results: BTreeMap<String, UiButtonGroupOutput>,
    collapsible_results: BTreeMap<String, UiCollapsibleOutput>,
    dropdown_menu_results: BTreeMap<String, UiDropdownMenuOutput>,
    button_results: BTreeMap<String, bool>,
    text_edit_results: BTreeMap<String, UiTextEditOutput>,
    virtual_list_results: BTreeMap<String, UiVirtualListOutput>,
    container_events: Vec<String>,
    container_stack: Vec<&'static str>,
    id_events: Vec<String>,
    id_stack: Vec<String>,
}

impl Default for RecordingHost {
    fn default() -> Self {
        Self {
            surface_id: SurfaceId::new(TEST_SURFACE_ID),
            events: Vec::new(),
            logs: Vec::new(),
            reload_requests: 0,
            repaint_requests: 0,
            panic_log: false,
            panic_label: false,
            panic_request_reload: false,
            panic_request_repaint: false,
            fail_request_reload: None,
            fail_request_repaint: None,
            labels: Vec::new(),
            buttons: Vec::new(),
            text_edits: Vec::new(),
            checkboxes: Vec::new(),
            switches: Vec::new(),
            sliders: Vec::new(),
            number_inputs: Vec::new(),
            selects: Vec::new(),
            tabs: Vec::new(),
            progresses: Vec::new(),
            radios: Vec::new(),
            button_groups: Vec::new(),
            collapsibles: Vec::new(),
            dropdown_menus: Vec::new(),
            tooltips: Vec::new(),
            spinners: Vec::new(),
            skeletons: Vec::new(),
            virtual_lists: Vec::new(),
            boolean_results: BTreeMap::new(),
            number_results: BTreeMap::new(),
            select_results: BTreeMap::new(),
            tabs_results: BTreeMap::new(),
            button_group_results: BTreeMap::new(),
            collapsible_results: BTreeMap::new(),
            dropdown_menu_results: BTreeMap::new(),
            button_results: BTreeMap::new(),
            text_edit_results: BTreeMap::new(),
            virtual_list_results: BTreeMap::new(),
            container_events: Vec::new(),
            container_stack: Vec::new(),
            id_events: Vec::new(),
            id_stack: Vec::new(),
        }
    }
}

impl RecordingHost {
    fn scoped_id(&self, id: &str) -> String {
        WidgetIdentity::new(&self.surface_id, &self.id_stack, id).debug_path()
    }
}

impl RuntimeAppHost for RecordingHost {
    fn log(&mut self, level: &str, message: &str) -> Result<(), RuntimeError> {
        assert!(!self.panic_log, "recording host panic in app.log");
        self.events.push(format!("log:{level}:{message}"));
        self.logs.push((level.to_owned(), message.to_owned()));
        Ok(())
    }

    fn request_reload(&mut self) -> Result<(), RuntimeError> {
        assert!(
            !self.panic_request_reload,
            "recording host panic in app.request_reload"
        );
        if let Some(err) = self.fail_request_reload.clone() {
            return Err(err);
        }
        self.events.push("request_reload".to_owned());
        self.reload_requests += 1;
        Ok(())
    }

    fn request_repaint(&mut self) -> Result<(), RuntimeError> {
        assert!(
            !self.panic_request_repaint,
            "recording host panic in app.request_repaint"
        );
        if let Some(err) = self.fail_request_repaint.clone() {
            return Err(err);
        }
        self.events.push("request_repaint".to_owned());
        self.repaint_requests += 1;
        Ok(())
    }
}

impl RuntimeUiHost for RecordingHost {
    fn label(&mut self, text: &str, options: UiLabelOptions) -> Result<(), RuntimeError> {
        assert!(!self.panic_label, "recording host panic in ui.label");
        self.events.push(format!("label:{text}"));
        self.labels.push((text.to_owned(), options));
        Ok(())
    }

    fn separator(&mut self) -> Result<(), RuntimeError> {
        self.events.push("separator".to_owned());
        Ok(())
    }

    fn button(
        &mut self,
        id: &str,
        text: &str,
        options: UiButtonOptions,
    ) -> Result<bool, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events.push(format!("button:{scoped_id}:{text}"));
        self.buttons
            .push((scoped_id.clone(), text.to_owned(), options));
        Ok(self.button_results.remove(&scoped_id).unwrap_or(false))
    }

    fn text_edit(
        &mut self,
        id: &str,
        value: &str,
        options: UiTextEditOptions,
    ) -> Result<UiTextEditOutput, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events.push(format!("text_edit:{scoped_id}"));
        self.text_edits
            .push((scoped_id.clone(), value.to_owned(), options));
        Ok(self
            .text_edit_results
            .remove(&scoped_id)
            .unwrap_or(UiTextEditOutput {
                value: value.to_owned(),
                changed: false,
            }))
    }

    fn checkbox(
        &mut self,
        id: &str,
        checked: bool,
        options: UiCheckboxOptions,
    ) -> Result<UiBooleanOutput, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events.push(format!("checkbox:{scoped_id}"));
        self.checkboxes.push((scoped_id.clone(), checked, options));
        Ok(self
            .boolean_results
            .remove(&scoped_id)
            .unwrap_or(UiBooleanOutput {
                value: checked,
                changed: false,
            }))
    }

    fn switch(
        &mut self,
        id: &str,
        checked: bool,
        options: UiSwitchOptions,
    ) -> Result<UiBooleanOutput, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events.push(format!("switch:{scoped_id}"));
        self.switches.push((scoped_id.clone(), checked, options));
        Ok(self
            .boolean_results
            .remove(&scoped_id)
            .unwrap_or(UiBooleanOutput {
                value: checked,
                changed: false,
            }))
    }

    fn slider(
        &mut self,
        id: &str,
        value: f32,
        options: UiSliderOptions,
    ) -> Result<UiNumberOutput, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events.push(format!("slider:{scoped_id}"));
        self.sliders.push((scoped_id.clone(), value, options));
        Ok(self
            .number_results
            .remove(&scoped_id)
            .unwrap_or(UiNumberOutput {
                value,
                changed: false,
            }))
    }

    fn number_input(
        &mut self,
        id: &str,
        value: f32,
        options: UiNumberInputOptions,
    ) -> Result<UiNumberOutput, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events.push(format!("number_input:{scoped_id}"));
        self.number_inputs.push((scoped_id.clone(), value, options));
        Ok(self
            .number_results
            .remove(&scoped_id)
            .unwrap_or(UiNumberOutput {
                value,
                changed: false,
            }))
    }

    fn select(
        &mut self,
        id: &str,
        selected_index: usize,
        items: &[String],
        options: UiSelectOptions,
    ) -> Result<UiSelectOutput, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events.push(format!("select:{scoped_id}"));
        self.selects
            .push((scoped_id.clone(), selected_index, items.to_vec(), options));
        Ok(self
            .select_results
            .remove(&scoped_id)
            .unwrap_or(UiSelectOutput {
                selected_index,
                changed: false,
            }))
    }

    fn tabs(
        &mut self,
        id: &str,
        selected_index: usize,
        options: &[UiTabOption],
        props: UiTabsOptions,
    ) -> Result<UiTabsOutput, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events.push(format!("tabs:{scoped_id}"));
        self.tabs
            .push((scoped_id.clone(), selected_index, options.to_vec(), props));
        Ok(self
            .tabs_results
            .remove(&scoped_id)
            .unwrap_or(UiTabsOutput {
                selected_index,
                changed: false,
            }))
    }

    fn progress(&mut self, value: f32, options: UiProgressOptions) -> Result<(), RuntimeError> {
        self.events.push("progress".to_owned());
        self.progresses.push((value, options));
        Ok(())
    }

    fn radio(
        &mut self,
        id: &str,
        selected: bool,
        options: UiRadioOptions,
    ) -> Result<UiBooleanOutput, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events.push(format!("radio:{scoped_id}"));
        self.radios.push((scoped_id.clone(), selected, options));
        Ok(self
            .boolean_results
            .remove(&scoped_id)
            .unwrap_or(UiBooleanOutput {
                value: selected,
                changed: false,
            }))
    }

    fn button_group(
        &mut self,
        id: &str,
        options: &[String],
        props: UiButtonGroupOptions,
    ) -> Result<UiButtonGroupOutput, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events.push(format!("button_group:{scoped_id}"));
        self.button_groups
            .push((scoped_id.clone(), options.to_vec(), props));
        Ok(self
            .button_group_results
            .remove(&scoped_id)
            .unwrap_or(UiButtonGroupOutput {
                clicked_index: 0,
                changed: false,
            }))
    }

    fn begin_collapsible(
        &mut self,
        id: &str,
        title: &str,
        open: bool,
        options: UiCollapsibleOptions,
    ) -> Result<UiCollapsibleOutput, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events
            .push(format!("begin_collapsible:{scoped_id}:{title}"));
        self.collapsibles
            .push((scoped_id.clone(), title.to_owned(), open, options));
        let output = self
            .collapsible_results
            .remove(&scoped_id)
            .unwrap_or(UiCollapsibleOutput {
                open,
                visible: open,
            });
        if output.visible {
            self.container_stack.push("collapsible");
            self.container_events
                .push(format!("collapsible:begin:{scoped_id}:{title}"));
        }
        Ok(output)
    }

    fn dropdown_menu(
        &mut self,
        id: &str,
        trigger_label: &str,
        entries: &[UiDropdownMenuEntry],
        options: UiDropdownMenuOptions,
    ) -> Result<UiDropdownMenuOutput, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events
            .push(format!("dropdown_menu:{scoped_id}:{trigger_label}"));
        self.dropdown_menus.push((
            scoped_id.clone(),
            trigger_label.to_owned(),
            entries.to_vec(),
            options,
        ));
        Ok(self
            .dropdown_menu_results
            .remove(&scoped_id)
            .unwrap_or(UiDropdownMenuOutput {
                action_id: 0,
                changed: false,
            }))
    }

    fn tooltip(
        &mut self,
        trigger_label: &str,
        text: &str,
        options: UiTooltipOptions,
    ) -> Result<(), RuntimeError> {
        self.events.push(format!("tooltip:{trigger_label}:{text}"));
        self.tooltips
            .push((trigger_label.to_owned(), text.to_owned(), options));
        Ok(())
    }

    fn spinner(&mut self, options: UiSpinnerOptions) -> Result<(), RuntimeError> {
        self.events.push("spinner".to_owned());
        self.spinners.push(options);
        Ok(())
    }

    fn skeleton(&mut self, options: UiSkeletonOptions) -> Result<(), RuntimeError> {
        self.events.push("skeleton".to_owned());
        self.skeletons.push(options);
        Ok(())
    }

    fn virtual_list(
        &mut self,
        id: &str,
        items: &[String],
        options: UiVirtualListOptions,
    ) -> Result<UiVirtualListOutput, RuntimeError> {
        let scoped_id = self.scoped_id(id);
        self.events.push(format!("virtual_list:{scoped_id}"));
        self.virtual_lists
            .push((scoped_id.clone(), items.to_vec(), options.clone()));
        Ok(self
            .virtual_list_results
            .remove(&scoped_id)
            .unwrap_or(UiVirtualListOutput {
                selected_index: options.selected_index,
                changed: false,
            }))
    }

    fn begin_row(&mut self, options: UiContainerOptions) -> Result<(), RuntimeError> {
        self.container_events
            .push(format!("row:begin:{:?}", options.gap));
        self.container_stack.push("row");
        Ok(())
    }

    fn begin_column(&mut self, options: UiContainerOptions) -> Result<(), RuntimeError> {
        self.container_events
            .push(format!("column:begin:{:?}", options.gap));
        self.container_stack.push("column");
        Ok(())
    }

    fn begin_card(&mut self, options: UiCardOptions) -> Result<(), RuntimeError> {
        self.container_events.push(format!(
            "card:begin:{:?}:{:.1}:{:.1}",
            options.width, options.padding_x, options.padding_y
        ));
        self.container_stack.push("card");
        Ok(())
    }

    fn end(&mut self) -> Result<(), RuntimeError> {
        let kind = self.container_stack.pop().ok_or_else(|| {
            RuntimeError::backend("ui.end_scope called without an open container scope".to_owned())
        })?;
        self.container_events.push(format!("{kind}:end"));
        Ok(())
    }

    fn push_id(&mut self, id: &str) -> Result<(), RuntimeError> {
        self.id_stack.push(id.to_owned());
        self.id_events.push(format!("push:{id}"));
        Ok(())
    }

    fn pop_id(&mut self) -> Result<(), RuntimeError> {
        let id = self.id_stack.pop().ok_or_else(|| {
            RuntimeError::backend("ui.pop_id called without a pushed id scope".to_owned())
        })?;
        self.id_events.push(format!("pop:{id}"));
        Ok(())
    }
}

fn test_mount() -> SurfaceMount {
    SurfaceMount::new(
        SurfaceId::new(TEST_SURFACE_ID),
        SurfaceCapabilities {
            log: true,
            reload: true,
            repaint: true,
            ..SurfaceCapabilities::none()
        },
    )
}

fn test_mount_with(capabilities: SurfaceCapabilities) -> SurfaceMount {
    SurfaceMount::new(SurfaceId::new(TEST_SURFACE_ID), capabilities)
}

fn test_runtime_config() -> RuntimeConfig {
    RuntimeConfig::default()
}

fn instrumented_runtime_config(
    leak_sample_reload_interval: u64,
    leak_retained_growth_warning_bytes: usize,
) -> RuntimeConfig {
    let mut config = RuntimeConfig::default();
    config.instrumentation.leak_sample_reload_interval = leak_sample_reload_interval;
    config.instrumentation.leak_retained_growth_warning_bytes = leak_retained_growth_warning_bytes;
    config
}

#[test]
fn constructs_runtime_and_processes_reload_queue() {
    let runtime = ScriptRuntime::new(test_runtime_config());

    assert_eq!(runtime.config(), &test_runtime_config());
    assert_eq!(runtime.last_compile_error(), None);
    assert_eq!(runtime.last_runtime_error(), None);

    let queue = runtime.reload_queue();
    queue.push("scripts/root.luau");
    assert_eq!(queue.drain(), vec![PathBuf::from("scripts/root.luau")]);
    assert!(queue.is_empty());
}

#[test]
fn persistent_state_survives_multiple_frames() {
    let source = ScriptSource::inline(
        "root.luau",
        r#"
            local module = {}

            function module.init(state)
                if state.count == nil then
                    state.count = 0
                end
            end

            function module.render(state)
                if state.count == nil then
                    state.count = 0
                end
                state.count = state.count + 1
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(test_runtime_config());
    runtime.load_root(source, test_mount()).unwrap();
    runtime.render_frame(test_mount()).unwrap();
    runtime.render_frame(test_mount()).unwrap();

    assert_eq!(runtime.state_i64("count"), Some(2));
    assert_eq!(runtime.last_compile_error(), None);
    assert_eq!(runtime.last_runtime_error(), None);
}

#[test]
fn reload_failure_keeps_last_known_good_version() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(
        temp_file,
        r#"
            local module = {{}}

            function module.render(state)
                state.count = (state.count or 0) + 1
            end

            return module
        "#
    )
    .unwrap();
    temp_file.flush().unwrap();

    let path = temp_file.path().to_path_buf();
    let source = ScriptSource::path(path.clone());
    let mut runtime = ScriptRuntime::new(test_runtime_config());

    runtime.load_root(source, test_mount()).unwrap();
    runtime.render_frame(test_mount()).unwrap();
    assert_eq!(runtime.state_i64("count"), Some(1));

    fs::write(
        &path,
        r#"
            local module = {
        "#,
    )
    .unwrap();

    runtime.reload_queue().push(path);
    let reload_result = runtime.reload_now(test_mount());
    assert!(reload_result.is_err());
    assert!(runtime.last_compile_error().is_some());

    runtime.render_frame(test_mount()).unwrap();
    assert_eq!(runtime.state_i64("count"), Some(2));
    assert_eq!(runtime.last_runtime_error(), None);
}

#[test]
fn host_bindings_route_app_calls() {
    let source = ScriptSource::inline(
        "host_demo.luau",
        r#"
            local module = {}

            function module.init(state)
                app.log("info", "init")
            end

            function module.render(state)
                app.log("debug", "render")
                app.request_reload()
                app.request_repaint()
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(
        host.logs,
        vec![
            ("info".to_owned(), "init".to_owned()),
            ("debug".to_owned(), "render".to_owned()),
        ]
    );
    assert_eq!(host.reload_requests, 1);
    assert_eq!(host.repaint_requests, 1);
}

#[test]
fn staged_first_frame_skips_update_on_initial_load() {
    let source = ScriptSource::inline(
        "first_frame_skips_update.luau",
        r#"
            local module = {}

            function module.update(state, input)
                state.update_calls = (state.update_calls or 0) + 1
                app.log("info", "update")
            end

            function module.render(state)
                state.render_calls = (state.render_calls or 0) + 1
                app.log("debug", "render:" .. tostring(state.render_calls))
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(host.logs, vec![("debug".to_owned(), "render:1".to_owned())]);
    assert_eq!(runtime.state_i64("update_calls"), None);
    assert_eq!(runtime.state_i64("render_calls"), Some(1));
}

#[test]
fn update_runs_before_render_and_receives_reserved_input_table_in_steady_state() {
    let source = ScriptSource::inline(
        "update_order_demo.luau",
        r#"
            local module = {}

            function module.update(state, input)
                if next(input) ~= nil then
                    error("expected reserved input table to be empty")
                end

                state.sequence = (state.sequence or "") .. "u"
                app.log("info", "update")
            end

            function module.render(state)
                state.sequence = (state.sequence or "") .. "r"
                app.log("debug", state.sequence)
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    host.logs.clear();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(
        host.logs,
        vec![
            ("info".to_owned(), "update".to_owned()),
            ("debug".to_owned(), "rur".to_owned()),
        ]
    );
    assert_eq!(runtime.state_string("sequence"), Some("rur".to_owned()));
}

#[test]
fn deferred_frame_commands_apply_after_successful_render() {
    let source = ScriptSource::inline(
        "deferred_frame_commands_demo.luau",
        r#"
            local module = {}

            function module.update(state, input)
                app.request_reload()
                app.log("info", "update")
            end

            function module.render(state)
                app.request_repaint()
                app.log("debug", "render")
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    host.events.clear();
    host.logs.clear();
    host.reload_requests = 0;
    host.repaint_requests = 0;
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(
        host.events,
        vec![
            "log:info:update".to_owned(),
            "log:debug:render".to_owned(),
            "request_reload".to_owned(),
            "request_repaint".to_owned(),
        ]
    );
    assert_eq!(host.reload_requests, 1);
    assert_eq!(host.repaint_requests, 1);
}

#[test]
fn queued_commands_are_dropped_when_render_fails_after_update() {
    let source = ScriptSource::inline(
        "drop_queued_commands_after_update_demo.luau",
        r#"
            local module = {}

            function module.update(state, input)
                app.request_reload()
                app.request_repaint()
                app.log("info", "queued")
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

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    host.logs.clear();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(host.reload_requests, 0);
    assert_eq!(host.repaint_requests, 0);
    assert_eq!(host.logs, vec![("info".to_owned(), "queued".to_owned())]);
    assert!(runtime
        .last_runtime_error()
        .is_some_and(|message| message.contains("boom")));
}

#[test]
fn queued_commands_are_dropped_when_render_fails_after_enqueueing() {
    let source = ScriptSource::inline(
        "drop_queued_commands_in_render_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                app.request_reload()
                app.request_repaint()
                error("boom")
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(host.reload_requests, 0);
    assert_eq!(host.repaint_requests, 0);
    assert!(runtime
        .last_runtime_error()
        .is_some_and(|message| message.contains("boom")));
}

#[test]
fn direct_ui_bindings_return_immediate_results() {
    let source = ScriptSource::inline(
        "direct_ui_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                state.count = state.count or 0
                state.name = state.name or "Luau"

                ui.label("Runtime Showcase", {
                    tone = "Muted",
                    size = 12,
                })

                if ui.button("demo.increment", "Increment", {
                    variant = "Primary",
                    width = 180,
                }) then
                    state.count = state.count + 1
                end

                local next_value, changed = ui.text_edit("demo.name", state.name, {
                    placeholder = "Enter a name",
                    width = 240,
                })
                if changed then
                    state.name = next_value
                end

                app.log("info", state.name)
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();
    let no_scopes: [&str; 0] = [];
    host.button_results.insert(
        WidgetIdentity::new(&host.surface_id, &no_scopes, "demo.increment").debug_path(),
        true,
    );
    host.text_edit_results.insert(
        WidgetIdentity::new(&host.surface_id, &no_scopes, "demo.name").debug_path(),
        UiTextEditOutput {
            value: "Andy".to_owned(),
            changed: true,
        },
    );

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert!(host.events.contains(&"label:Runtime Showcase".to_owned()));
    assert!(host
        .events
        .contains(&"button:tests.runtime::demo.increment:Increment".to_owned()));
    assert!(host
        .events
        .contains(&"text_edit:tests.runtime::demo.name".to_owned()));
    assert_eq!(host.logs, vec![("info".to_owned(), "Andy".to_owned())]);
    assert_eq!(runtime.state_i64("count"), Some(1));
    assert_eq!(runtime.state_string("name"), Some("Andy".to_owned()));
}

#[test]
fn extended_ui_bindings_return_immediate_results() {
    let source = ScriptSource::inline(
        "extended_ui_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                state.checked = state.checked or false
                state.slider = state.slider or 0
                state.select_index = state.select_index or 0
                state.tab_index = state.tab_index or 1
                state.group_index = state.group_index or 0
                state.open = state.open or false
                state.menu_action = state.menu_action or 0

                local checked, checked_changed = ui.checkbox("demo.checkbox", state.checked, {
                    label = "Enabled",
                })
                if checked_changed then
                    state.checked = checked
                end

                local slider_value, slider_changed = ui.slider("demo.slider", state.slider, {
                    min = 0,
                    max = 100,
                    width = 180,
                })
                if slider_changed then
                    state.slider = slider_value
                end

                local select_index, select_changed = ui.select("demo.select", state.select_index, {
                    { label = "Alpha" },
                    { label = "Beta" },
                }, {
                    placeholder = "Choose",
                })
                if select_changed then
                    state.select_index = select_index
                end

                local tab_index, tab_changed = ui.tabs("demo.tabs", state.tab_index, {
                    { label = "Overview" },
                    { label = "Files" },
                }, {
                    variant = "Segmented",
                })
                if tab_changed then
                    state.tab_index = tab_index
                end

                local group_index, group_changed = ui.button_group("demo.group", {
                    { label = "Preview" },
                    { label = "Ship" },
                })
                if group_changed then
                    state.group_index = group_index
                end

                local open, visible = ui.begin_collapsible(
                    "demo.collapsible",
                    "Advanced",
                    state.open
                )
                state.open = open
                if visible then
                    ui.label("Inside collapsible")
                    ui.end_scope()
                end

                local action_id, action_changed = ui.dropdown_menu("demo.menu", "Open menu", {
                    { kind = "action", id = 7, label = "Rename" },
                    {
                        kind = "submenu",
                        label = "Move to",
                        entries = {
                            { kind = "action", id = 8, label = "Archive" },
                        },
                    },
                })
                if action_changed then
                    state.menu_action = action_id
                end

                ui.progress(0.5, {
                    width = 160,
                    height = 8,
                })
                ui.tooltip("Tooltip trigger", "Tooltip body", {
                    placement = "Top",
                })
                ui.spinner({
                    size = 18,
                })
                ui.skeleton({
                    width = 80,
                    height = 12,
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();
    let no_scopes: [&str; 0] = [];
    host.boolean_results.insert(
        WidgetIdentity::new(&host.surface_id, &no_scopes, "demo.checkbox").debug_path(),
        UiBooleanOutput {
            value: true,
            changed: true,
        },
    );
    host.number_results.insert(
        WidgetIdentity::new(&host.surface_id, &no_scopes, "demo.slider").debug_path(),
        UiNumberOutput {
            value: 42.0,
            changed: true,
        },
    );
    host.select_results.insert(
        WidgetIdentity::new(&host.surface_id, &no_scopes, "demo.select").debug_path(),
        UiSelectOutput {
            selected_index: 2,
            changed: true,
        },
    );
    host.tabs_results.insert(
        WidgetIdentity::new(&host.surface_id, &no_scopes, "demo.tabs").debug_path(),
        UiTabsOutput {
            selected_index: 2,
            changed: true,
        },
    );
    host.button_group_results.insert(
        WidgetIdentity::new(&host.surface_id, &no_scopes, "demo.group").debug_path(),
        UiButtonGroupOutput {
            clicked_index: 2,
            changed: true,
        },
    );
    host.collapsible_results.insert(
        WidgetIdentity::new(&host.surface_id, &no_scopes, "demo.collapsible").debug_path(),
        UiCollapsibleOutput {
            open: true,
            visible: true,
        },
    );
    host.dropdown_menu_results.insert(
        WidgetIdentity::new(&host.surface_id, &no_scopes, "demo.menu").debug_path(),
        UiDropdownMenuOutput {
            action_id: 8,
            changed: true,
        },
    );

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(host.checkboxes.len(), 1);
    assert_eq!(host.sliders.len(), 1);
    assert_eq!(host.selects.len(), 1);
    assert_eq!(host.tabs.len(), 1);
    assert_eq!(host.button_groups.len(), 1);
    assert_eq!(host.collapsibles.len(), 1);
    assert_eq!(host.dropdown_menus.len(), 1);
    assert_eq!(host.progresses.len(), 1);
    assert_eq!(host.tooltips.len(), 1);
    assert_eq!(host.spinners.len(), 1);
    assert_eq!(host.skeletons.len(), 1);
    assert!(host.events.contains(&"label:Inside collapsible".to_owned()));
    assert_eq!(runtime.state_bool("checked"), Some(true));
    assert_eq!(runtime.state_i64("slider"), Some(42));
    assert_eq!(runtime.state_i64("select_index"), Some(2));
    assert_eq!(runtime.state_i64("tab_index"), Some(2));
    assert_eq!(runtime.state_i64("group_index"), Some(2));
    assert_eq!(runtime.state_bool("open"), Some(true));
    assert_eq!(runtime.state_i64("menu_action"), Some(8));
}

#[test]
fn virtual_list_returns_immediate_selection_result() {
    let source = ScriptSource::inline(
        "virtual_list_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                state.selection = state.selection or 0
                local selected_index, changed = ui.virtual_list("demo.files", {
                    "Cargo.toml",
                    "README.md",
                    "src/lib.rs",
                }, {
                    selected_index = state.selection,
                    height = 96,
                    row_height = 22,
                })
                if changed then
                    state.selection = selected_index
                end
                app.log("info", tostring(state.selection))
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(test_runtime_config());
    let mut host = RecordingHost::default();
    let no_scopes: [&str; 0] = [];
    host.virtual_list_results.insert(
        WidgetIdentity::new(&host.surface_id, &no_scopes, "demo.files").debug_path(),
        UiVirtualListOutput {
            selected_index: 2,
            changed: true,
        },
    );

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert!(host
        .events
        .contains(&"virtual_list:tests.runtime::demo.files".to_owned()));
    assert_eq!(host.virtual_lists.len(), 1);
    assert_eq!(
        host.virtual_lists[0].1,
        vec![
            "Cargo.toml".to_owned(),
            "README.md".to_owned(),
            "src/lib.rs".to_owned()
        ]
    );
    assert_eq!(host.virtual_lists[0].2.selected_index, 0);
    assert_eq!(host.logs, vec![("info".to_owned(), "2".to_owned())]);
    assert_eq!(runtime.state_i64("selection"), Some(2));
}

#[test]
fn virtual_list_reuses_cached_items_on_steady_frames() {
    let source = ScriptSource::inline(
        "virtual_list_cache_hit_demo.luau",
        r#"
            local module = {}

            function module.init(state)
                state.items = {
                    "Cargo.toml",
                    "README.md",
                    "src/lib.rs",
                }
            end

            function module.render(state)
                ui.virtual_list("demo.files", state.items, {
                    height = 96,
                    row_height = 22,
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(test_runtime_config());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert_eq!(runtime.test_virtual_list_cache_stats(), (0, 0, 1));

    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(runtime.test_virtual_list_cache_stats(), (1, 0, 1));
    assert_eq!(host.virtual_lists.len(), 2);
    assert_eq!(
        host.virtual_lists[1].1,
        vec![
            "Cargo.toml".to_owned(),
            "README.md".to_owned(),
            "src/lib.rs".to_owned(),
        ]
    );
}

#[test]
fn virtual_list_rebuilds_after_cached_table_mutation() {
    let source = ScriptSource::inline(
        "virtual_list_mutation_demo.luau",
        r#"
            local module = {}

            function module.init(state)
                state.frame = 0
                state.items = {
                    "Cargo.toml",
                    "README.md",
                }
            end

            function module.render(state)
                state.frame += 1
                if state.frame == 2 then
                    state.items[2] = "src/lib.rs"
                end
                ui.virtual_list("demo.files", state.items, {
                    height = 96,
                    row_height = 22,
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(test_runtime_config());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert_eq!(runtime.test_virtual_list_cache_stats(), (0, 0, 1));

    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert_eq!(runtime.test_virtual_list_cache_stats(), (0, 0, 2));
    assert_eq!(
        host.virtual_lists
            .last()
            .expect("expected second-frame virtual list call")
            .1,
        vec!["Cargo.toml".to_owned(), "src/lib.rs".to_owned()]
    );

    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert_eq!(runtime.test_virtual_list_cache_stats(), (1, 0, 2));
}

#[test]
fn virtual_list_readonly_tables_use_readonly_fast_path() {
    let source = ScriptSource::inline(
        "virtual_list_readonly_demo.luau",
        r#"
            local FILES = table.freeze({
                "Cargo.toml",
                "README.md",
                "src/lib.rs",
            })

            local module = {}

            function module.render(state)
                ui.virtual_list("demo.files", FILES, {
                    height = 96,
                    row_height = 22,
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(test_runtime_config());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert_eq!(runtime.test_virtual_list_cache_stats(), (0, 0, 1));

    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(runtime.test_virtual_list_cache_stats(), (1, 1, 1));
}

#[test]
fn ui_namespace_is_unavailable_without_host() {
    let source = ScriptSource::inline(
        "ui_requires_host_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.label("legacy")
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    runtime.load_root(source, test_mount()).unwrap();
    let render = runtime.render_frame(test_mount());

    assert!(render.is_err());
    assert!(runtime.last_runtime_error().is_some());
}

#[test]
fn host_scopes_do_not_leak_between_frames() {
    let source = ScriptSource::inline(
        "host_scope_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                app.log("info", "render")
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut first_host = RecordingHost::default();
    let mut second_host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut first_host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut first_host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut second_host)
        .unwrap();

    assert_eq!(
        first_host.logs,
        vec![("info".to_owned(), "render".to_owned())]
    );
    assert_eq!(
        second_host.logs,
        vec![("info".to_owned(), "render".to_owned())]
    );
}

#[test]
fn host_globals_restore_after_runtime_error() {
    let source = ScriptSource::inline(
        "callback_error_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                state.frame = (state.frame or 0) + 1
                if state.frame == 2 then
                    error("boom")
                end

                if state.frame >= 3 then
                    app.log("info", "recovered")
                end
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut failing_host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut failing_host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut failing_host)
        .unwrap();
    let first_render = runtime.render_frame_with_host(test_mount(), &mut failing_host);
    assert!(first_render.is_err());

    let mut recovered_host = RecordingHost::default();
    runtime
        .render_frame_with_host(test_mount(), &mut recovered_host)
        .unwrap();

    assert_eq!(
        recovered_host.logs,
        vec![("info".to_owned(), "recovered".to_owned())]
    );
    assert_eq!(runtime.last_runtime_error(), None);
}

#[test]
fn stashed_app_function_fails_after_frame_closes() {
    let source = ScriptSource::inline(
        "stashed_app_function.luau",
        r#"
            local module = {}
            local stale_repaint = nil

            function module.render(state)
                state.frame = (state.frame or 0) + 1

                if state.frame == 1 then
                    stale_repaint = app.request_repaint
                    return
                end

                stale_repaint()
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(host.repaint_requests, 0);
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: surface `tests.runtime` attempted to call `app.request_repaint` after the frame closed"
        )
    );
}

#[test]
fn stashed_ui_function_fails_after_frame_closes() {
    let source = ScriptSource::inline(
        "stashed_ui_function.luau",
        r#"
            local module = {}
            local stale_label = nil

            function module.render(state)
                state.frame = (state.frame or 0) + 1

                if state.frame == 1 then
                    stale_label = ui.label
                    ui.label("fresh")
                    return
                end

                stale_label("stale")
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(host.labels.len(), 1);
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: surface `tests.runtime` attempted to call `ui.label` after the frame closed"
        )
    );
}

#[test]
fn app_and_ui_tables_are_stable_across_frames() {
    let source = ScriptSource::inline(
        "stable_bridge_tables.luau",
        r#"
            local module = {}

            function module.render(state)
                state.frame = (state.frame or 0) + 1

                if state.frame == 1 then
                    state.first_app = app
                    state.first_ui = ui
                    return
                end

                state.same_app = state.first_app == app
                state.same_ui = state.first_ui == ui
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(runtime.state_bool("same_app"), Some(true));
    assert_eq!(runtime.state_bool("same_ui"), Some(true));
}

#[test]
fn api_function_identity_refreshes_each_frame() {
    let source = ScriptSource::inline(
        "scoped_bridge_functions.luau",
        r#"
            local module = {}

            function module.render(state)
                state.frame = (state.frame or 0) + 1

                if state.frame == 1 then
                    state.first_repaint = app.request_repaint
                    state.first_label = ui.label
                    return
                end

                state.same_repaint = state.first_repaint == app.request_repaint
                state.same_label = state.first_label == ui.label
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(runtime.state_bool("same_repaint"), Some(false));
    assert_eq!(runtime.state_bool("same_label"), Some(false));
}

#[test]
fn bridge_materializes_only_used_wrappers_per_scope() {
    let source = ScriptSource::inline(
        "lazy_bridge_wrappers.luau",
        r#"
            local module = {}

            function module.render(state)
                local first = ui.label
                local second = ui.label
                state.same_label_wrapper = first == second
                first("one")
                second("two")
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(runtime.state_bool("same_label_wrapper"), Some(true));
    assert_eq!(runtime.test_bridge_wrapper_creations(), 1);
}

#[test]
fn app_log_requires_capability() {
    let source = ScriptSource::inline(
        "log_capability_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                app.log("info", "blocked")
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();
    let mount = test_mount_with(SurfaceCapabilities {
        log: false,
        reload: true,
        repaint: true,
        ..SurfaceCapabilities::none()
    });

    runtime
        .load_root_with_host(source, mount.clone(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(mount, &mut host);

    assert!(render.is_err());
    assert!(host.logs.is_empty());
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: surface `tests.runtime` is not allowed to call `app.log` because capability `log` is disabled"
        )
    );
}

#[test]
fn app_request_reload_requires_capability() {
    let source = ScriptSource::inline(
        "reload_capability_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                app.request_reload()
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();
    let mount = test_mount_with(SurfaceCapabilities {
        log: true,
        reload: false,
        repaint: true,
        ..SurfaceCapabilities::none()
    });

    runtime
        .load_root_with_host(source, mount.clone(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(mount, &mut host);

    assert!(render.is_err());
    assert_eq!(host.reload_requests, 0);
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: surface `tests.runtime` is not allowed to call `app.request_reload` because capability `reload` is disabled"
        )
    );
}

#[test]
fn app_request_repaint_requires_capability() {
    let source = ScriptSource::inline(
        "repaint_capability_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                app.request_repaint()
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();
    let mount = test_mount_with(SurfaceCapabilities {
        log: true,
        reload: true,
        repaint: false,
        ..SurfaceCapabilities::none()
    });

    runtime
        .load_root_with_host(source, mount.clone(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(mount, &mut host);

    assert!(render.is_err());
    assert_eq!(host.repaint_requests, 0);
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: surface `tests.runtime` is not allowed to call `app.request_repaint` because capability `repaint` is disabled"
        )
    );
}

#[test]
fn ui_calls_fail_during_update() {
    let source = ScriptSource::inline(
        "ui_in_update_demo.luau",
        r#"
            local module = {}

            function module.update(state, input)
                ui.label("not allowed")
            end

            function module.render(state)
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert!(host.labels.is_empty());
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: surface `tests.runtime` attempted to call render-only API `ui.label` during update"
        )
    );
}

#[test]
fn ui_calls_fail_during_init() {
    let source = ScriptSource::inline(
        "ui_in_init_demo.luau",
        r#"
            local module = {}

            function module.init(state)
                ui.label("not allowed")
            end

            function module.render(state)
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    let load = runtime.load_root_with_host(source, test_mount(), &mut host);

    assert!(load.is_err());
    assert!(host.labels.is_empty());
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: surface `tests.runtime` attempted to call render-only API `ui.label` during load"
        )
    );
}

#[test]
fn app_request_reload_fails_during_init() {
    let source = ScriptSource::inline(
        "reload_in_init_demo.luau",
        r#"
            local module = {}

            function module.init(state)
                app.request_reload()
            end

            function module.render(state)
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    let load = runtime.load_root_with_host(source, test_mount(), &mut host);

    assert!(load.is_err());
    assert_eq!(host.reload_requests, 0);
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: surface `tests.runtime` attempted to call frame-only API `app.request_reload` during load"
        )
    );
}

#[test]
fn ui_calls_fail_during_reload() {
    let source = ScriptSource::inline(
        "ui_in_reload_demo.luau",
        r#"
            local module = {}

            function module.reload(old_exports, state)
                ui.label("not allowed")
            end

            function module.render(state)
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    let reload = runtime.reload_now_with_host(test_mount(), &mut host);

    assert!(reload.is_err());
    assert!(host.labels.is_empty());
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: surface `tests.runtime` attempted to call render-only API `ui.label` during reload"
        )
    );
}

#[test]
fn app_request_repaint_fails_during_reload() {
    let source = ScriptSource::inline(
        "repaint_in_reload_demo.luau",
        r#"
            local module = {}

            function module.reload(old_exports, state)
                app.request_repaint()
            end

            function module.render(state)
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    let reload = runtime.reload_now_with_host(test_mount(), &mut host);

    assert!(reload.is_err());
    assert_eq!(host.repaint_requests, 0);
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: surface `tests.runtime` attempted to call frame-only API `app.request_repaint` during reload"
        )
    );
}

#[test]
fn nested_explicit_scopes_execute_in_order() {
    let source = ScriptSource::inline(
        "nested_scope_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.begin_column({ gap = 16 })
                ui.label("outer")
                ui.begin_card({ width = 280, padding_x = 18, padding_y = 20 })
                ui.begin_row({ gap = 8 })
                ui.label("inner")
                ui.end_scope()
                ui.end_scope()
                ui.end_scope()
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(
        host.container_events,
        vec![
            "column:begin:Some(16.0)".to_owned(),
            "card:begin:Some(280.0):18.0:20.0".to_owned(),
            "row:begin:Some(8.0)".to_owned(),
            "row:end".to_owned(),
            "card:end".to_owned(),
            "column:end".to_owned(),
        ]
    );
    assert_eq!(
        host.labels,
        vec![
            ("outer".to_owned(), UiLabelOptions::default()),
            ("inner".to_owned(), UiLabelOptions::default()),
        ]
    );
}

#[test]
fn push_id_scopes_stateful_widget_ids() {
    let source = ScriptSource::inline(
        "push_id_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.push_id("profile")
                ui.button("increment", "Increment", {
                    variant = "Primary",
                })
                ui.pop_id()
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(
        host.id_events,
        vec!["push:profile".to_owned(), "pop:profile".to_owned()]
    );
    assert_eq!(
        host.buttons,
        vec![(
            "tests.runtime::profile::increment".to_owned(),
            "Increment".to_owned(),
            UiButtonOptions {
                variant: UiButtonVariant::Primary,
                ..UiButtonOptions::default()
            },
        )]
    );
}

#[test]
fn layout_containers_do_not_affect_widget_ids() {
    let source = ScriptSource::inline(
        "container_identity_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.begin_column({ gap = 12 })
                ui.begin_card({ width = 280, padding_x = 16, padding_y = 16 })
                ui.begin_row({ gap = 8 })
                ui.button("save", "Save")
                ui.end_scope()
                ui.end_scope()
                ui.end_scope()
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert!(host.id_events.is_empty());
    assert_eq!(
        host.buttons,
        vec![(
            "tests.runtime::save".to_owned(),
            "Save".to_owned(),
            UiButtonOptions::default(),
        )]
    );
}

#[test]
fn label_props_reject_unknown_keys() {
    let source = ScriptSource::inline(
        "label_unknown_prop_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.label("Hello", {
                    tone = "Primary",
                    color = "red",
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: ui.label received unknown prop `color`; allowed keys: tone, weight, size"
        )
    );
}

#[test]
fn button_props_reject_unknown_keys() {
    let source = ScriptSource::inline(
        "button_unknown_prop_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.button("save", "Save", {
                    variant = "Primary",
                    padding = 12,
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: ui.button received unknown prop `padding`; allowed keys: variant, size, width, leading_icon, trailing_icon, icon_size, icon_only, selected"
        )
    );
}

#[test]
fn text_edit_props_reject_unknown_keys() {
    let source = ScriptSource::inline(
        "text_edit_unknown_prop_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.text_edit("name", "", {
                    placeholder = "Name",
                    multiline = true,
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: ui.text_edit received unknown prop `multiline`; allowed keys: width, placeholder, leading_icon, password"
        )
    );
}

#[test]
fn checkbox_props_reject_unknown_keys() {
    let source = ScriptSource::inline(
        "checkbox_unknown_prop_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.checkbox("enabled", true, {
                    label = "Enabled",
                    tone = "muted",
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some("backend error: ui.checkbox received unknown prop `tone`; allowed keys: label")
    );
}

#[test]
fn select_props_reject_unknown_keys() {
    let source = ScriptSource::inline(
        "select_unknown_prop_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.select("theme", 0, {
                    { label = "Light" },
                    { label = "Dark" },
                }, {
                    width = 220,
                    size = "sm",
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: ui.select received unknown prop `size`; allowed keys: width, placeholder, variant"
        )
    );
}

#[test]
fn virtual_list_props_reject_unknown_keys() {
    let source = ScriptSource::inline(
        "virtual_list_unknown_prop_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.virtual_list("files", {
                    "Cargo.toml",
                    "README.md",
                }, {
                    row_height = 22,
                    padding = 10,
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(test_runtime_config());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: ui.virtual_list received unknown prop `padding`; allowed keys: width, height, row_height, selected_index"
        )
    );
}

#[test]
fn virtual_list_rejects_sparse_or_non_string_items() {
    let sparse_source = ScriptSource::inline(
        "virtual_list_sparse_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.virtual_list("files", {
                    [2] = "README.md",
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(test_runtime_config());
    let mut host = RecordingHost::default();
    runtime
        .load_root_with_host(sparse_source, test_mount(), &mut host)
        .unwrap();
    let sparse_render = runtime.render_frame_with_host(test_mount(), &mut host);
    assert!(sparse_render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: ui.virtual_list expected argument 2 to use only consecutive integer keys 1..0, got 2"
        )
    );

    let wrong_type_source = ScriptSource::inline(
        "virtual_list_wrong_type_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.virtual_list("files", {
                    "Cargo.toml",
                    true,
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(test_runtime_config());
    let mut host = RecordingHost::default();
    runtime
        .load_root_with_host(wrong_type_source, test_mount(), &mut host)
        .unwrap();
    let wrong_type_render = runtime.render_frame_with_host(test_mount(), &mut host);
    assert!(wrong_type_render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: ui.virtual_list expected argument 2 to contain only strings, got Boolean(true) at index 2"
        )
    );
}

#[test]
fn container_props_reject_unknown_keys() {
    let source = ScriptSource::inline(
        "container_unknown_prop_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.begin_row({
                    gap = 8,
                    padding = 10,
                })
                ui.end_scope()
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some("backend error: ui.begin_row received unknown prop `padding`; allowed keys: gap")
    );
}

#[test]
fn column_props_reject_unknown_keys() {
    let source = ScriptSource::inline(
        "column_unknown_prop_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.begin_column({
                    gap = 8,
                    padding = 10,
                })
                ui.end_scope()
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some("backend error: ui.begin_column received unknown prop `padding`; allowed keys: gap")
    );
}

#[test]
fn card_props_reject_unknown_keys() {
    let source = ScriptSource::inline(
        "card_unknown_prop_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.begin_card({
                    width = 280,
                    radius = 12,
                })
                ui.end_scope()
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some(
            "backend error: ui.begin_card received unknown prop `radius`; allowed keys: width, padding_x, padding_y"
        )
    );
}

#[test]
fn unclosed_container_scopes_fail_the_frame() {
    let source = ScriptSource::inline(
        "unclosed_scope_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.begin_column({ gap = 12 })
                ui.label("oops")
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some("backend error: render exited with 1 unclosed container scope(s)")
    );
}

#[test]
fn popping_id_without_push_fails_the_frame() {
    let source = ScriptSource::inline(
        "pop_id_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.pop_id()
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert_eq!(
        runtime.last_runtime_error(),
        Some("backend error: ui.pop_id called without a pushed id scope")
    );
}

#[test]
fn leaf_module_reload_updates_dependents_and_preserves_module_state() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let leaf_path = temp_dir.path().join("leaf.luau");

    fs::write(
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
                app.log("info", "leaf reloaded")
            end

            function module.message()
                return "leaf v1 / reloads=" .. tostring(module_state.reloads)
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local leaf = require("./leaf")
            local module = {}

            function module.render(state)
                app.log("info", leaf.message())
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(host
        .logs
        .iter()
        .any(|(_, message)| message == "leaf v1 / reloads=0"));

    fs::write(
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
                app.log("info", "leaf reloaded")
            end

            function module.message()
                return "leaf v2 / reloads=" .. tostring(module_state.reloads)
            end

            return module
        "#,
    )
    .unwrap();

    runtime.reload_queue().push(leaf_path);
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert!(host
        .logs
        .iter()
        .any(|(_, message)| message == "leaf reloaded"));
    assert!(host
        .logs
        .iter()
        .any(|(_, message)| message == "leaf v2 / reloads=1"));
}

#[test]
fn unaffected_modules_are_reused_by_handle_across_reload() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let shared_path = temp_dir.path().join("shared.luau");

    fs::write(
        &shared_path,
        r#"
            local counter = 0
            local module = {}

            function module.bump()
                counter += 1
                return counter
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local shared = require("./shared")
            local module = {}

            function module.render(state)
                app.log("info", "root-v1:" .. tostring(shared.bump()))
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(host.logs.iter().any(|(_, message)| message == "root-v1:1"));

    fs::write(
        &root_path,
        r#"
            local shared = require("./shared")
            local module = {}

            function module.render(state)
                app.log("info", "root-v2:" .. tostring(shared.bump()))
            end

            return module
        "#,
    )
    .unwrap();

    host.logs.clear();
    runtime.reload_queue().push(root_path);
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert!(host.logs.iter().any(|(_, message)| message == "root-v2:2"));
}

#[test]
fn shared_export_mutation_is_not_reload_isolated() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let shared_path = temp_dir.path().join("shared.luau");

    fs::write(
        &shared_path,
        r#"
            local module = {
                counter = 0,
            }

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local shared = require("./shared")
            local module = {}

            function module.render(state)
                shared.counter = shared.counter + 1
                app.log("info", "root-v1:" .. tostring(shared.counter))
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(host.logs.iter().any(|(_, message)| message == "root-v1:1"));

    fs::write(
        &root_path,
        r#"
            local shared = require("./shared")
            local module = {}

            function module.render(state)
                shared.counter = shared.counter + 1
                app.log("info", "root-v2:" .. tostring(shared.counter))
            end

            return module
        "#,
    )
    .unwrap();

    host.logs.clear();
    runtime.reload_queue().push(root_path);
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert!(host.logs.iter().any(|(_, message)| message == "root-v2:2"));
}

#[test]
fn reload_hook_side_effects_escape_state_via_shared_helper() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let leaf_path = temp_dir.path().join("leaf.luau");
    let shared_path = temp_dir.path().join("shared.luau");

    fs::write(
        &shared_path,
        r#"
            local module = {
                count = 0,
                last = "none",
            }

            function module.note(tag)
                module.count = module.count + 1
                module.last = tag
            end

            function module.summary()
                return tostring(module.count) .. ":" .. module.last
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &leaf_path,
        r#"
            local shared = require("./shared")
            local module = {}

            function module.message()
                return "leaf-v1 / shared=" .. shared.summary()
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local leaf = require("./leaf")
            local module = {}

            function module.render(state)
                app.log("info", leaf.message())
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(host
        .logs
        .iter()
        .any(|(_, message)| message == "leaf-v1 / shared=0:none"));

    fs::write(
        &leaf_path,
        r#"
            local shared = require("./shared")
            local module = {}

            function module.reload(old_exports, state)
                shared.note("leaf-reload-v2")
            end

            function module.message()
                return "leaf-v2 / shared=" .. shared.summary()
            end

            return module
        "#,
    )
    .unwrap();

    host.logs.clear();
    runtime.reload_queue().push(leaf_path);
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert!(host
        .logs
        .iter()
        .any(|(_, message)| message == "leaf-v2 / shared=1:leaf-reload-v2"));
}

#[test]
fn broken_leaf_module_keeps_previous_graph_alive() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let leaf_path = temp_dir.path().join("leaf.luau");

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.message()
                return "leaf stable"
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local leaf = require("./leaf")
            local module = {}

            function module.render(state)
                app.log("info", leaf.message())
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(host
        .logs
        .iter()
        .any(|(_, message)| message == "leaf stable"));

    fs::write(
        &leaf_path,
        r#"
            local module = {
        "#,
    )
    .unwrap();

    runtime.reload_queue().push(leaf_path);
    let reload = runtime.reload_now_with_host(test_mount(), &mut host);
    assert!(reload.is_err());
    assert!(runtime.last_compile_error().is_some());

    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    let stable_logs = host
        .logs
        .iter()
        .filter(|(_, message)| message == "leaf stable")
        .count();
    assert!(stable_logs >= 2);
}

#[test]
fn dependency_reload_hooks_run_in_dependency_order() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let middle_path = temp_dir.path().join("middle.luau");
    let leaf_path = temp_dir.path().join("leaf.luau");

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.message()
                return "leaf-v1"
            end

            function module.reload(old_exports, state)
                app.log("info", "leaf reload")
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &middle_path,
        r#"
            local leaf = require("./leaf")
            local module = {}

            function module.message()
                return "middle -> " .. leaf.message()
            end

            function module.reload(old_exports, state)
                app.log("info", "middle reload")
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local middle = require("./middle")
            local module = {}

            function module.render(state)
                app.log("info", middle.message())
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    host.logs.clear();

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.message()
                return "leaf-v2"
            end

            function module.reload(old_exports, state)
                app.log("info", "leaf reload")
            end

            return module
        "#,
    )
    .unwrap();

    runtime.reload_queue().push(leaf_path);
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();

    let leaf_index = host
        .logs
        .iter()
        .position(|(_, message)| message == "leaf reload")
        .expect("leaf reload hook logged");
    let middle_index = host
        .logs
        .iter()
        .position(|(_, message)| message == "middle reload")
        .expect("middle reload hook logged");
    assert!(leaf_index < middle_index);
}

#[test]
fn reload_metrics_track_success_failure_and_dirty_queue_shape() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let leaf_path = temp_dir.path().join("leaf.luau");

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.message()
                return "leaf-v1"
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local leaf = require("./leaf")
            local module = {}

            function module.render(state)
                app.log("info", leaf.message())
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    assert_eq!(
        runtime.reload_metrics(),
        &ReloadMetrics {
            attempts: 1,
            successes: 1,
            failures: 0,
            last_kind: Some(ReloadKind::Load),
            last_dirty_path_count: 0,
            last_affected_path_count: 1,
            last_rebuilt_module_count: 2,
            last_active_module_count: 2,
            last_duration: runtime.reload_metrics().last_duration,
        }
    );
    assert!(runtime.reload_metrics().last_duration.is_some());
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.message()
                return "leaf-v2"
            end

            return module
        "#,
    )
    .unwrap();

    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();

    let metrics = runtime.reload_metrics();
    assert_eq!(metrics.attempts, 2);
    assert_eq!(metrics.successes, 2);
    assert_eq!(metrics.failures, 0);
    assert_eq!(metrics.last_kind, Some(ReloadKind::Reload));
    assert_eq!(metrics.last_dirty_path_count, 0);
    assert_eq!(metrics.last_affected_path_count, 2);
    assert_eq!(metrics.last_rebuilt_module_count, 2);
    assert_eq!(metrics.last_active_module_count, 2);
    assert!(metrics.last_duration.is_some());

    fs::write(
        &leaf_path,
        r#"
            local module = {
        "#,
    )
    .unwrap();
    runtime.reload_queue().push(leaf_path);
    assert!(runtime
        .reload_now_with_host(test_mount(), &mut host)
        .is_err());

    let metrics = runtime.reload_metrics();
    assert_eq!(metrics.attempts, 3);
    assert_eq!(metrics.successes, 2);
    assert_eq!(metrics.failures, 1);
    assert_eq!(metrics.last_kind, Some(ReloadKind::Reload));
    assert_eq!(metrics.last_dirty_path_count, 1);
    assert_eq!(metrics.last_affected_path_count, 2);
    assert_eq!(metrics.last_rebuilt_module_count, 0);
    assert_eq!(metrics.last_active_module_count, 2);
    assert!(metrics.last_duration.is_some());
}

#[test]
fn memory_metrics_track_load_reload_render_and_gc_samples() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("memory_demo.luau");

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.render(state)
                state.payload = state.payload or "v1"
                ui.label(state.payload)
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(instrumented_runtime_config(1, usize::MAX));
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.render(state)
                state.payload = state.payload or string.rep("v", 4096)
                ui.label("reloaded")
            end

            return module
        "#,
    )
    .unwrap();

    runtime.reload_queue().push(root_path);
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    let metrics = runtime.memory_metrics();
    assert!(metrics.current_heap_bytes > 0);
    assert!(metrics.peak_heap_bytes >= metrics.current_heap_bytes);
    assert_eq!(metrics.reload.last_kind, Some(ReloadKind::Reload));
    assert_eq!(metrics.reload.totals.attempts, 2);
    assert_eq!(metrics.reload.totals.successes, 2);
    assert!(metrics.reload.totals.last_delta_bytes.is_some());
    assert!(metrics.reload.totals.last_post_gc_bytes.is_some());
    assert_eq!(
        metrics.frame.last_render_kind,
        Some(RuntimeRenderKind::FirstAfterReload)
    );
    assert_eq!(metrics.frame.totals.attempts, 2);
    assert_eq!(metrics.frame.totals.successes, 2);
    assert!(metrics.frame.totals.last_post_gc_bytes.is_some());
    assert_eq!(metrics.gc.reload_step_attempts, 2);
    assert_eq!(metrics.gc.frame_step_attempts, 2);
    assert_eq!(metrics.gc.full_gc_runs, 2);
    assert_eq!(metrics.leak_detection.sample_count, 2);
    assert_eq!(metrics.leak_detection.committed_reload_count, 1);
    assert_eq!(
        metrics
            .leak_detection
            .last_sample
            .as_ref()
            .map(|sample| sample.committed_reload_count),
        Some(1)
    );
}

#[test]
fn memory_metrics_record_failed_frame_attempts() {
    let source = ScriptSource::inline(
        "memory_failure_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                error("frame boom")
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(test_runtime_config());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);

    assert!(render.is_err());
    assert!(runtime
        .last_runtime_error()
        .is_some_and(|message| message.contains("frame boom")));

    let metrics = runtime.memory_metrics();
    assert_eq!(
        metrics.frame.last_render_kind,
        Some(RuntimeRenderKind::FirstAfterLoad)
    );
    assert_eq!(metrics.frame.totals.attempts, 1);
    assert_eq!(metrics.frame.totals.successes, 0);
    assert_eq!(metrics.frame.totals.failures, 1);
    assert!(metrics.frame.totals.last_delta_bytes.is_some());
    assert_eq!(metrics.gc.frame_step_attempts, 1);
}

#[test]
fn leak_detection_warns_when_retained_heap_grows_across_reload() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("leak_warning_demo.luau");

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.render(state)
                state.payload = "small"
                ui.label("baseline")
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(instrumented_runtime_config(1, 1024));
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.render(state)
                state.payload = string.rep("x", 128 * 1024)
                ui.label("grown")
            end

            return module
        "#,
    )
    .unwrap();

    runtime.reload_queue().push(root_path);
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    let metrics = runtime.memory_metrics();
    let sample = metrics
        .leak_detection
        .last_sample
        .as_ref()
        .expect("leak sample recorded after committed reload");
    assert!(sample.warning_emitted);
    assert!(sample.retained_growth_bytes > 1024);
    assert_eq!(sample.module_count, 1);
    assert!(metrics
        .leak_detection
        .last_warning_message
        .as_ref()
        .is_some_and(|message| message.contains("retained heap grew")));
}

#[test]
fn lifecycle_metrics_track_load_reload_and_render_buckets() {
    let source = ScriptSource::inline(
        "lifecycle_metrics_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                state.frames = (state.frames or 0) + 1
                app.log("info", tostring(state.frames))
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    let metrics = runtime.lifecycle_metrics();
    assert_eq!(metrics.load.attempts, 1);
    assert_eq!(metrics.load.successes, 1);
    assert_eq!(metrics.load.failures, 0);
    assert!(metrics.load.last_duration.is_some());
    assert_eq!(metrics.reload.attempts, 0);

    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    let metrics = runtime.lifecycle_metrics();
    assert_eq!(
        metrics.render.last_kind,
        Some(RuntimeRenderKind::FirstAfterLoad)
    );
    assert_eq!(metrics.render.first_after_load.attempts, 1);
    assert_eq!(metrics.render.first_after_load.successes, 1);
    assert_eq!(metrics.render.first_after_load.failures, 0);
    assert!(metrics.render.first_after_load.last_duration.is_some());
    assert_eq!(metrics.render.steady_state.attempts, 0);

    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    let metrics = runtime.lifecycle_metrics();
    assert_eq!(
        metrics.render.last_kind,
        Some(RuntimeRenderKind::SteadyState)
    );
    assert_eq!(metrics.render.steady_state.attempts, 1);
    assert_eq!(metrics.render.steady_state.successes, 1);
    assert_eq!(metrics.render.steady_state.failures, 0);
    assert!(metrics.render.steady_state.last_duration.is_some());

    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    let metrics = runtime.lifecycle_metrics();
    assert_eq!(metrics.reload.attempts, 1);
    assert_eq!(metrics.reload.successes, 1);
    assert_eq!(metrics.reload.failures, 0);
    assert!(metrics.reload.last_duration.is_some());

    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    let metrics = runtime.lifecycle_metrics();
    assert_eq!(
        metrics.render.last_kind,
        Some(RuntimeRenderKind::FirstAfterReload)
    );
    assert_eq!(metrics.render.first_after_reload.attempts, 1);
    assert_eq!(metrics.render.first_after_reload.successes, 1);
    assert_eq!(metrics.render.first_after_reload.failures, 0);
    assert!(metrics.render.first_after_reload.last_duration.is_some());

    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    let metrics = runtime.lifecycle_metrics();
    assert_eq!(
        metrics.render.last_kind,
        Some(RuntimeRenderKind::SteadyState)
    );
    assert_eq!(metrics.render.steady_state.attempts, 2);
    assert_eq!(metrics.render.steady_state.successes, 2);
    eprintln!(
        "phase0_runtime_baseline load_us={} reload_us={} first_after_load_us={} first_after_reload_us={} steady_us={}",
        metrics
            .load
            .last_duration
            .expect("load duration captured")
            .as_micros(),
        metrics
            .reload
            .last_duration
            .expect("reload duration captured")
            .as_micros(),
        metrics
            .render
            .first_after_load
            .last_duration
            .expect("first-after-load duration captured")
            .as_micros(),
        metrics
            .render
            .first_after_reload
            .last_duration
            .expect("first-after-reload duration captured")
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
fn lifecycle_metrics_count_failures() {
    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());

    assert!(runtime
        .load_root(
            ScriptSource::inline("broken_load.luau", "local module = {"),
            test_mount(),
        )
        .is_err());
    let metrics = runtime.lifecycle_metrics();
    assert_eq!(metrics.load.attempts, 1);
    assert_eq!(metrics.load.successes, 0);
    assert_eq!(metrics.load.failures, 1);
    assert!(metrics.load.last_duration.is_some());

    assert!(runtime.reload_now(test_mount()).is_err());
    let metrics = runtime.lifecycle_metrics();
    assert_eq!(metrics.reload.attempts, 1);
    assert_eq!(metrics.reload.successes, 0);
    assert_eq!(metrics.reload.failures, 1);
    assert!(metrics.reload.last_duration.is_some());

    runtime
        .load_root(
            ScriptSource::inline(
                "broken_render.luau",
                r#"
                    local module = {}

                    function module.render(state)
                        error("render boom")
                    end

                    return module
                "#,
            ),
            test_mount(),
        )
        .unwrap();
    assert!(runtime.render_frame(test_mount()).is_err());
    let metrics = runtime.lifecycle_metrics();
    assert_eq!(
        metrics.render.last_kind,
        Some(RuntimeRenderKind::FirstAfterLoad)
    );
    assert_eq!(metrics.render.first_after_load.attempts, 1);
    assert_eq!(metrics.render.first_after_load.successes, 0);
    assert_eq!(metrics.render.first_after_load.failures, 1);
    assert!(metrics.render.first_after_load.last_duration.is_some());
}

#[test]
fn reload_queue_only_marks_work_until_reload_boundary() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let leaf_path = temp_dir.path().join("leaf.luau");

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.message()
                return "leaf-v1"
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local leaf = require("./leaf")
            local module = {}

            function module.render(state)
                app.log("info", leaf.message())
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(host.logs.iter().any(|(_, message)| message == "leaf-v1"));

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.message()
                return "leaf-v2"
            end

            return module
        "#,
    )
    .unwrap();
    runtime.reload_queue().push(leaf_path.clone());
    runtime.reload_queue().push(leaf_path.clone());
    assert_eq!(runtime.reload_queue().len(), 1);

    host.logs.clear();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(host.logs.iter().any(|(_, message)| message == "leaf-v1"));
    assert_eq!(runtime.reload_queue().len(), 1);

    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    assert_eq!(runtime.reload_metrics().last_dirty_path_count, 1);
    host.logs.clear();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(host.logs.iter().any(|(_, message)| message == "leaf-v2"));
    assert_eq!(runtime.reload_queue().len(), 0);
}

#[test]
fn manual_reload_rebuilds_required_leaf_modules_when_queue_is_empty() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let leaf_path = temp_dir.path().join("leaf.luau");

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.message()
                return "leaf-v1"
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local leaf = require("./leaf")
            local module = {}

            function module.render(state)
                app.log("info", leaf.message())
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(ScriptSource::path(root_path), test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(host.logs.iter().any(|(_, message)| message == "leaf-v1"));

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.message()
                return "leaf-v2"
            end

            return module
        "#,
    )
    .unwrap();

    host.logs.clear();
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert!(host.logs.iter().any(|(_, message)| message == "leaf-v2"));
    let metrics = runtime.reload_metrics();
    assert_eq!(metrics.last_dirty_path_count, 0);
    assert_eq!(metrics.last_affected_path_count, 2);
    assert_eq!(metrics.last_rebuilt_module_count, 2);
}

#[test]
fn render_time_require_uses_committed_snapshot_across_frames_and_reload() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let leaf_path = temp_dir.path().join("leaf.luau");

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.message()
                return "leaf-v1"
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local initial_leaf = require("./leaf")
            local module = {}

            function module.render(state)
                local leaf = require("./leaf")
                app.log("info", leaf.message())
                app.log("debug", initial_leaf.message())
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    let leaf_v1_logs = host
        .logs
        .iter()
        .filter(|(_, message)| message == "leaf-v1")
        .count();
    assert_eq!(leaf_v1_logs, 4);

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.message()
                return "leaf-v2"
            end

            return module
        "#,
    )
    .unwrap();

    runtime.reload_queue().push(leaf_path);
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert!(host.logs.iter().any(|(_, message)| message == "leaf-v2"));
}

#[test]
fn shutdown_uses_old_committed_snapshot_after_swap() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let leaf_path = temp_dir.path().join("leaf.luau");
    let shared_path = temp_dir.path().join("shared.luau");

    fs::write(
        &shared_path,
        r#"
            local module = {}

            function module.value()
                return "old-shared"
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &leaf_path,
        r#"
            local shared = require("./shared")
            local module = {}

            function module.message()
                return "leaf-v1"
            end

            function module.shutdown(state)
                local shared = require("./shared")
                app.log("info", "shutdown:" .. shared.value())
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local initial_leaf = require("./leaf")
            local module = {}

            function module.render(state)
                local leaf = require("./leaf")
                app.log("info", leaf.message())
                app.log("debug", initial_leaf.message())
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    host.logs.clear();

    fs::write(
        &shared_path,
        r#"
            local module = {}

            function module.value()
                return "new-shared"
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &leaf_path,
        r#"
            local shared = require("./shared")
            local module = {}

            function module.message()
                return "leaf-v2"
            end

            function module.shutdown(state)
                local shared = require("./shared")
                app.log("info", "shutdown:" .. shared.value())
            end

            return module
        "#,
    )
    .unwrap();

    runtime.reload_queue().push(shared_path);
    runtime.reload_queue().push(leaf_path);
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert!(host
        .logs
        .iter()
        .any(|(_, message)| message == "shutdown:old-shared"));
    assert!(host.logs.iter().any(|(_, message)| message == "leaf-v2"));
    assert!(!host
        .logs
        .iter()
        .any(|(_, message)| message == "shutdown:new-shared"));
}

#[test]
fn shutdown_failure_after_commit_keeps_new_runtime_active() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.render(state)
                app.log("info", "stable-v1")
            end

            function module.shutdown(state)
                error("shutdown boom")
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.render(state)
                app.log("info", "candidate-v2")
            end

            return module
        "#,
    )
    .unwrap();

    host.logs.clear();
    runtime.reload_queue().push(root_path.clone());
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();

    let render = runtime.render_frame_with_host(test_mount(), &mut host);
    assert!(render.is_err());
    assert!(runtime.pending_candidate.is_none());
    let failure = runtime.last_failure().expect("shutdown failure recorded");
    let expected_context = root_path.display().to_string();
    assert_eq!(failure.class, RuntimeFailureClass::InitReload);
    assert_eq!(failure.stage, RuntimeFailureStage::FirstAfterReload);
    assert_eq!(
        failure.module_context.as_deref(),
        Some(expected_context.as_str())
    );
    assert_eq!(failure.hook_or_api.as_deref(), Some("shutdown"));
    assert!(!failure.rolled_back);
    assert!(runtime
        .last_runtime_error()
        .is_some_and(|message| message.contains("shutdown boom")));

    host.logs.clear();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(host
        .logs
        .iter()
        .any(|(_, message)| message == "candidate-v2"));
}

#[test]
fn unrelated_dirty_luau_file_does_not_reload_active_graph() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let leaf_path = temp_dir.path().join("leaf.luau");
    let scratch_path = temp_dir.path().join("scratch.luau");

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.reload(old_exports, state)
                app.log("info", "leaf reloaded")
            end

            function module.message()
                return "leaf stable"
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local leaf = require("./leaf")
            local module = {}

            function module.render(state)
                app.log("info", leaf.message())
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &scratch_path,
        r#"
            return {
                note = "scratch"
            }
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(ScriptSource::path(root_path), test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    host.logs.clear();

    runtime.reload_queue().push(scratch_path);
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert!(!host
        .logs
        .iter()
        .any(|(_, message)| message == "leaf reloaded"));
    assert!(host
        .logs
        .iter()
        .any(|(_, message)| message == "leaf stable"));

    let metrics = runtime.reload_metrics();
    assert_eq!(metrics.last_dirty_path_count, 1);
    assert_eq!(metrics.last_affected_path_count, 0);
    assert_eq!(metrics.last_rebuilt_module_count, 0);
    assert_eq!(metrics.last_active_module_count, 2);
}

#[test]
fn reload_stages_candidate_with_render_only_validation_until_first_render_commits() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.render(state)
                state.version = "v1"
                app.log("info", state.version)
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert_eq!(runtime.state_string("version"), Some("v1".to_owned()));

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.update(state, input)
                error("candidate update should be skipped")
            end

            function module.render(state)
                state.version = "v2"
                app.log("info", state.version)
            end

            return module
        "#,
    )
    .unwrap();

    host.logs.clear();
    runtime.reload_queue().push(root_path.clone());
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();

    assert!(runtime.pending_candidate.is_some());
    assert_eq!(runtime.state_string("version"), Some("v1".to_owned()));
    assert!(!host.logs.iter().any(|(_, message)| message == "v2"));

    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(runtime.pending_candidate.is_none());
    assert_eq!(runtime.state_string("version"), Some("v2".to_owned()));
    assert!(host.logs.iter().any(|(_, message)| message == "v2"));
}

#[test]
fn first_after_reload_render_failure_rolls_back_to_previous_graph() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.render(state)
                app.log("info", "stable-v1")
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.render(state)
                app.log("info", "candidate-v2")
                error("candidate render boom")
            end

            return module
        "#,
    )
    .unwrap();

    runtime.reload_queue().push(root_path.clone());
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();

    host.logs.clear();
    let render = runtime.render_frame_with_host(test_mount(), &mut host);
    assert!(render.is_err());
    assert!(runtime.pending_candidate.is_none());
    assert_eq!(
        runtime.last_failure(),
        Some(&RuntimeFailureRecord {
            class: RuntimeFailureClass::Render,
            stage: RuntimeFailureStage::FirstAfterReload,
            message: runtime
                .last_runtime_error()
                .expect("render failure recorded")
                .to_owned(),
            module_context: Some(root_path.display().to_string()),
            hook_or_api: None,
            rolled_back: true,
        })
    );

    host.logs.clear();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(host.logs.iter().any(|(_, message)| message == "stable-v1"));
}

#[test]
fn initial_first_render_failure_drops_candidate_without_rollback() {
    let source = ScriptSource::inline(
        "initial_render_failure.luau",
        r#"
            local module = {}

            function module.render(state)
                error("first frame boom")
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    assert!(runtime.has_render_target());

    let render = runtime.render_frame_with_host(test_mount(), &mut host);
    assert!(render.is_err());
    assert!(!runtime.has_render_target());
    let failure = runtime
        .last_failure()
        .expect("first-render failure recorded");
    assert_eq!(failure.class, RuntimeFailureClass::Render);
    assert_eq!(failure.stage, RuntimeFailureStage::FirstAfterLoad);
    assert!(!failure.rolled_back);
}

#[test]
fn init_and_reload_hook_failures_are_classified_with_module_context() {
    let source = ScriptSource::inline(
        "init_failure_demo.luau",
        r#"
            local module = {}

            function module.init(state)
                error("init boom")
            end

            function module.render(state)
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    let load = runtime.load_root_with_host(source, test_mount(), &mut host);
    assert!(load.is_err());
    let failure = runtime.last_failure().expect("init failure recorded");
    assert_eq!(failure.class, RuntimeFailureClass::InitReload);
    assert_eq!(failure.stage, RuntimeFailureStage::Load);
    assert_eq!(
        failure.module_context.as_deref(),
        Some("init_failure_demo.luau")
    );
    assert_eq!(failure.hook_or_api.as_deref(), Some("init"));

    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");
    let leaf_path = temp_dir.path().join("leaf.luau");
    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.message()
                return "leaf-v1"
            end

            return module
        "#,
    )
    .unwrap();
    fs::write(
        &root_path,
        r#"
            local leaf = require("./leaf")
            local module = {}

            function module.render(state)
                app.log("info", leaf.message())
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();
    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    fs::write(
        &leaf_path,
        r#"
            local module = {}

            function module.reload(old_exports, state)
                error("reload hook boom")
            end

            function module.message()
                return "leaf-v2"
            end

            return module
        "#,
    )
    .unwrap();

    runtime.reload_queue().push(leaf_path.clone());
    let reload = runtime.reload_now_with_host(test_mount(), &mut host);
    assert!(reload.is_err());
    let failure = runtime.last_failure().expect("reload failure recorded");
    assert_eq!(failure.class, RuntimeFailureClass::InitReload);
    assert_eq!(failure.stage, RuntimeFailureStage::Reload);
    assert_eq!(
        failure.module_context.as_deref(),
        Some(leaf_path.display().to_string().as_str())
    );
    assert_eq!(failure.hook_or_api.as_deref(), Some("reload"));
}

#[test]
fn update_and_steady_render_failures_are_classified() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.render(state)
                app.log("info", "stable-v1")
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();
    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.update(state, input)
                error("update boom")
            end

            function module.render(state)
                app.log("info", "candidate-v2")
            end

            return module
        "#,
    )
    .unwrap();

    runtime.reload_queue().push(root_path.clone());
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert!(host
        .logs
        .iter()
        .any(|(_, message)| message == "candidate-v2"));

    let steady_update = runtime.render_frame_with_host(test_mount(), &mut host);
    assert!(steady_update.is_err());
    let failure = runtime.last_failure().expect("update failure recorded");
    assert_eq!(failure.class, RuntimeFailureClass::Update);
    assert_eq!(failure.stage, RuntimeFailureStage::SteadyState);
    assert!(!failure.rolled_back);

    let source = ScriptSource::inline(
        "steady_failure_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                state.frames = (state.frames or 0) + 1
                if state.frames == 2 then
                    error("steady boom")
                end
                app.log("info", tostring(state.frames))
            end

            return module
        "#,
    );
    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();
    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    let steady_render = runtime.render_frame_with_host(test_mount(), &mut host);
    assert!(steady_render.is_err());
    let failure = runtime
        .last_failure()
        .expect("steady render failure recorded");
    assert_eq!(failure.class, RuntimeFailureClass::Render);
    assert_eq!(failure.stage, RuntimeFailureStage::SteadyState);
    assert!(!failure.rolled_back);

    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();
    assert_eq!(runtime.last_failure(), None);
}

#[test]
fn host_callback_panics_are_contained_and_classified() {
    let source = ScriptSource::inline(
        "host_panic_load_demo.luau",
        r#"
            local module = {}

            function module.init(state)
                app.log("info", "init")
            end

            function module.render(state)
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost {
        panic_log: true,
        ..RecordingHost::default()
    };
    let load = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.load_root_with_host(source, test_mount(), &mut host)
    }))
    .expect("host panic should be contained");
    assert!(load.is_err());
    let failure = runtime
        .last_failure()
        .expect("host callback panic recorded");
    assert_eq!(failure.class, RuntimeFailureClass::HostCallbackPanic);
    assert_eq!(failure.stage, RuntimeFailureStage::Load);
    assert_eq!(failure.hook_or_api.as_deref(), Some("app.log"));

    let source = ScriptSource::inline(
        "host_panic_render_demo.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.label("hello")
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();
    runtime
        .load_root_with_host(source, test_mount(), &mut host)
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    let mut panicking_host = RecordingHost {
        panic_label: true,
        ..RecordingHost::default()
    };
    let render = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.render_frame_with_host(test_mount(), &mut panicking_host)
    }))
    .expect("steady-state host panic should be contained");
    assert!(render.is_err());
    let failure = runtime
        .last_failure()
        .expect("steady host callback panic recorded");
    assert_eq!(failure.class, RuntimeFailureClass::HostCallbackPanic);
    assert_eq!(failure.stage, RuntimeFailureStage::SteadyState);
    assert_eq!(failure.hook_or_api.as_deref(), Some("ui.label"));

    runtime
        .render_frame_with_host(test_mount(), &mut RecordingHost::default())
        .unwrap();
    assert_eq!(runtime.last_failure(), None);
}

#[test]
fn command_flush_failure_rolls_back_candidate() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().join("demo.luau");

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.render(state)
                app.log("info", "stable-v1")
            end

            return module
        "#,
    )
    .unwrap();

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();
    runtime
        .load_root_with_host(
            ScriptSource::path(root_path.clone()),
            test_mount(),
            &mut host,
        )
        .unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    fs::write(
        &root_path,
        r#"
            local module = {}

            function module.render(state)
                app.request_reload()
                app.log("info", "candidate-v2")
            end

            return module
        "#,
    )
    .unwrap();

    runtime.reload_queue().push(root_path.clone());
    runtime
        .reload_now_with_host(test_mount(), &mut host)
        .unwrap();

    let mut failing_host = RecordingHost {
        panic_request_reload: true,
        ..RecordingHost::default()
    };
    let render = runtime.render_frame_with_host(test_mount(), &mut failing_host);
    assert!(render.is_err());
    let failure = runtime
        .last_failure()
        .expect("command flush failure recorded");
    assert_eq!(failure.class, RuntimeFailureClass::CommandFlush);
    assert_eq!(failure.stage, RuntimeFailureStage::FirstAfterReload);
    assert_eq!(failure.hook_or_api.as_deref(), Some("app.request_reload"));
    assert!(failure.rolled_back);

    let mut recovered_host = RecordingHost::default();
    runtime
        .render_frame_with_host(test_mount(), &mut recovered_host)
        .unwrap();
    assert!(recovered_host
        .logs
        .iter()
        .any(|(_, message)| message == "stable-v1"));
}

#[test]
fn parser_accepts_compatibility_enum_aliases() {
    let source = ScriptSource::inline(
        "compatibility_aliases.luau",
        r#"
            local module = {}

            function module.render(state)
                ui.label("Compatibility", {
                    tone = "Primary",
                    weight = "Bold",
                })
                ui.button("save", "Save", {
                    variant = "Primary",
                    size = "small",
                })
                ui.button("submit", "Submit", {
                    size = "medium",
                })
            end

            return module
        "#,
    );

    let mut runtime = ScriptRuntime::new(RuntimeConfig::default());
    let mut host = RecordingHost::default();

    runtime.load_root(source, test_mount()).unwrap();
    runtime
        .render_frame_with_host(test_mount(), &mut host)
        .unwrap();

    assert_eq!(host.labels.len(), 1);
    assert_eq!(host.labels[0].1.tone, UiLabelTone::Primary);
    assert_eq!(host.labels[0].1.weight, UiLabelWeight::Bold);
    assert_eq!(host.buttons.len(), 2);
    assert_eq!(host.buttons[0].2.variant, UiButtonVariant::Primary);
    assert_eq!(host.buttons[0].2.size, UiControlSize::Sm);
    assert_eq!(host.buttons[1].2.size, UiControlSize::Md);
}

#[test]
fn generated_runtime_api_typings_file_is_current() {
    let path = workspace_root()
        .join("examples")
        .join("runtime-luau")
        .join("ui")
        .join("core")
        .join("types.luau");
    let generated = runtime_api_luau_typings();
    let checked_in = fs::read_to_string(&path).unwrap();
    assert_eq!(
        checked_in, generated,
        "generated runtime API typings are stale; refresh with `cargo run -p luau-runtime-core --bin generate-runtime-api`"
    );
}

#[test]
fn generated_runtime_api_reference_file_is_current() {
    let path = workspace_root()
        .join("docs")
        .join("luau-runtime-api-reference.md");
    let generated = runtime_api_reference_markdown();
    let checked_in = fs::read_to_string(&path).unwrap();
    assert_eq!(
        checked_in, generated,
        "generated runtime API reference is stale; refresh with `cargo run -p luau-runtime-core --bin generate-runtime-api`"
    );
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("luau-runtime crate lives inside workspace root")
        .to_path_buf()
}

impl ScriptRuntime {
    fn state_bool(&self, key: &str) -> Option<bool> {
        let root = self.active_runtime.as_ref()?.graph.root_module()?;
        match root.state.raw_get::<Value>(key).ok()? {
            Value::Boolean(value) => Some(value),
            _ => None,
        }
    }

    fn state_i64(&self, key: &str) -> Option<i64> {
        let root = self.active_runtime.as_ref()?.graph.root_module()?;
        match root.state.raw_get::<Value>(key).ok()? {
            Value::Integer(value) => Some(value),
            Value::Number(value) => Some(value as i64),
            _ => None,
        }
    }

    fn state_string(&self, key: &str) -> Option<String> {
        let root = self.active_runtime.as_ref()?.graph.root_module()?;
        match root.state.raw_get::<Value>(key).ok()? {
            Value::String(value) => Some(value.to_str().ok()?.to_string()),
            _ => None,
        }
    }
}
