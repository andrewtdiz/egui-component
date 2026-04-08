use std::fs;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use luau_runtime_core::{
    RuntimeAppHost, RuntimeConfig, RuntimeError, RuntimeUiHost, ScriptRuntime, ScriptSource,
    SurfaceCapabilities, SurfaceId, SurfaceMount, UiBooleanOutput, UiButtonGroupOptions,
    UiButtonGroupOutput, UiButtonOptions, UiCardOptions, UiCheckboxOptions, UiCollapsibleOptions,
    UiCollapsibleOutput, UiContainerOptions, UiDropdownMenuEntry, UiDropdownMenuOptions,
    UiDropdownMenuOutput, UiLabelOptions, UiNumberInputOptions, UiNumberOutput, UiProgressOptions,
    UiRadioOptions, UiSelectOptions, UiSelectOutput, UiSkeletonOptions, UiSliderOptions,
    UiSpinnerOptions, UiSwitchOptions, UiTabOption, UiTabsOptions, UiTabsOutput, UiTextEditOptions,
    UiTextEditOutput, UiTooltipOptions, UiVirtualListOptions, UiVirtualListOutput,
};
use tempfile::TempDir;

const SURFACE_SIZES: &[usize] = &[100, 1_000, 5_000, 10_000, 25_000];
const SURFACE_ID: &str = "bench.runtime";

#[derive(Default)]
struct BenchHost;

impl RuntimeAppHost for BenchHost {
    fn log(&mut self, _level: &str, _message: &str) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn request_reload(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn request_repaint(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
}

impl RuntimeUiHost for BenchHost {
    fn label(&mut self, _text: &str, _options: UiLabelOptions) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn separator(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn button(
        &mut self,
        _id: &str,
        _text: &str,
        _options: UiButtonOptions,
    ) -> Result<bool, RuntimeError> {
        Ok(false)
    }

    fn text_edit(
        &mut self,
        _id: &str,
        value: &str,
        _options: UiTextEditOptions,
    ) -> Result<UiTextEditOutput, RuntimeError> {
        Ok(UiTextEditOutput {
            value: value.to_owned(),
            changed: false,
        })
    }

    fn checkbox(
        &mut self,
        _id: &str,
        checked: bool,
        _options: UiCheckboxOptions,
    ) -> Result<UiBooleanOutput, RuntimeError> {
        Ok(UiBooleanOutput {
            value: checked,
            changed: false,
        })
    }

    fn switch(
        &mut self,
        _id: &str,
        checked: bool,
        _options: UiSwitchOptions,
    ) -> Result<UiBooleanOutput, RuntimeError> {
        Ok(UiBooleanOutput {
            value: checked,
            changed: false,
        })
    }

    fn slider(
        &mut self,
        _id: &str,
        value: f32,
        _options: UiSliderOptions,
    ) -> Result<UiNumberOutput, RuntimeError> {
        Ok(UiNumberOutput {
            value,
            changed: false,
        })
    }

    fn number_input(
        &mut self,
        _id: &str,
        value: f32,
        _options: UiNumberInputOptions,
    ) -> Result<UiNumberOutput, RuntimeError> {
        Ok(UiNumberOutput {
            value,
            changed: false,
        })
    }

    fn select(
        &mut self,
        _id: &str,
        selected_index: usize,
        _items: &[String],
        _options: UiSelectOptions,
    ) -> Result<UiSelectOutput, RuntimeError> {
        Ok(UiSelectOutput {
            selected_index,
            changed: false,
        })
    }

    fn tabs(
        &mut self,
        _id: &str,
        selected_index: usize,
        _options: &[UiTabOption],
        _props: UiTabsOptions,
    ) -> Result<UiTabsOutput, RuntimeError> {
        Ok(UiTabsOutput {
            selected_index,
            changed: false,
        })
    }

    fn progress(&mut self, _value: f32, _options: UiProgressOptions) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn radio(
        &mut self,
        _id: &str,
        selected: bool,
        _options: UiRadioOptions,
    ) -> Result<UiBooleanOutput, RuntimeError> {
        Ok(UiBooleanOutput {
            value: selected,
            changed: false,
        })
    }

    fn button_group(
        &mut self,
        _id: &str,
        _options: &[String],
        _props: UiButtonGroupOptions,
    ) -> Result<UiButtonGroupOutput, RuntimeError> {
        Ok(UiButtonGroupOutput {
            clicked_index: 0,
            changed: false,
        })
    }

    fn begin_collapsible(
        &mut self,
        _id: &str,
        _title: &str,
        open: bool,
        _options: UiCollapsibleOptions,
    ) -> Result<UiCollapsibleOutput, RuntimeError> {
        Ok(UiCollapsibleOutput {
            open,
            visible: open,
        })
    }

    fn dropdown_menu(
        &mut self,
        _id: &str,
        _trigger_label: &str,
        _entries: &[UiDropdownMenuEntry],
        _options: UiDropdownMenuOptions,
    ) -> Result<UiDropdownMenuOutput, RuntimeError> {
        Ok(UiDropdownMenuOutput {
            action_id: 0,
            changed: false,
        })
    }

    fn tooltip(
        &mut self,
        _trigger_label: &str,
        _text: &str,
        _options: UiTooltipOptions,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn spinner(&mut self, _options: UiSpinnerOptions) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn skeleton(&mut self, _options: UiSkeletonOptions) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn virtual_list(
        &mut self,
        _id: &str,
        _items: &[String],
        options: UiVirtualListOptions,
    ) -> Result<UiVirtualListOutput, RuntimeError> {
        Ok(UiVirtualListOutput {
            selected_index: options.selected_index,
            changed: false,
        })
    }

    fn begin_row(&mut self, _options: UiContainerOptions) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn begin_column(&mut self, _options: UiContainerOptions) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn begin_card(&mut self, _options: UiCardOptions) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn end(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn push_id(&mut self, _id: &str) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn pop_id(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
}

fn benchmark_mount() -> SurfaceMount {
    SurfaceMount::new(
        SurfaceId::new(SURFACE_ID),
        SurfaceCapabilities {
            log: true,
            reload: true,
            repaint: true,
            ..SurfaceCapabilities::none()
        },
    )
}

fn benchmark_config() -> RuntimeConfig {
    let mut config = RuntimeConfig::default();
    config.instrumentation.leak_sample_reload_interval = 0;
    config
}

fn label_flood_script(count: usize) -> String {
    format!(
        r#"
            local module = {{}}

            function module.init(state)
                state.labels = {{}}
                for i = 1, {count} do
                    state.labels[i] = "Row " .. tostring(i)
                end
            end

            function module.render(state)
                for i = 1, {count} do
                    ui.label(state.labels[i])
                end
            end

            return module
        "#
    )
}

fn mixed_surface_script(count: usize) -> String {
    format!(
        r#"
            local module = {{}}

            function module.init(state)
                state.labels = {{}}
                state.button_ids = {{}}
                state.input_ids = {{}}
                state.values = {{}}

                for i = 1, {count} do
                    local suffix = tostring(i)
                    state.labels[i] = "Row " .. suffix
                    state.button_ids[i] = "button_" .. suffix
                    state.input_ids[i] = "input_" .. suffix
                    state.values[i] = "value"
                end
            end

            function module.render(state)
                for i = 1, {count} do
                    ui.label(state.labels[i])
                    ui.button(state.button_ids[i], "Click")
                    local next_value, _changed = ui.text_edit(state.input_ids[i], state.values[i], {{
                        width = 160,
                    }})
                    state.values[i] = next_value
                end
            end

            return module
        "#
    )
}

fn virtual_list_script(count: usize) -> String {
    format!(
        r#"
            local module = {{}}

            function module.init(state)
                state.items = {{}}
                state.selected_index = 0
                for i = 1, {count} do
                    state.items[i] = "Row " .. tostring(i)
                end
            end

            function module.render(state)
                local selected_index, changed = ui.virtual_list("bench.virtual_list", state.items, {{
                    height = 320,
                    row_height = 22,
                    selected_index = state.selected_index,
                }})
                if changed then
                    state.selected_index = selected_index
                end
            end

            return module
        "#
    )
}

fn versioned_script(script: &str, version: usize) -> String {
    format!("{script}\n-- benchmark_version:{version}\n")
}

struct ReloadHarness {
    _temp_dir: TempDir,
    root_path: std::path::PathBuf,
    runtime: ScriptRuntime,
    host: BenchHost,
    next_version: usize,
    script_factory: fn(usize) -> String,
    size: usize,
}

impl ReloadHarness {
    fn new(script_factory: fn(usize) -> String, size: usize) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("runtime_scale.luau");
        let initial_script = versioned_script(&script_factory(size), 0);
        fs::write(&root_path, initial_script).unwrap();

        let mut runtime = ScriptRuntime::new(benchmark_config());
        let mut host = BenchHost;
        runtime
            .load_root_with_host(
                ScriptSource::path(root_path.clone()),
                benchmark_mount(),
                &mut host,
            )
            .unwrap();
        runtime
            .render_frame_with_host(benchmark_mount(), &mut host)
            .unwrap();

        Self {
            _temp_dir: temp_dir,
            root_path,
            runtime,
            host,
            next_version: 1,
            script_factory,
            size,
        }
    }

    fn stage_reload(&mut self) {
        let script = versioned_script(&(self.script_factory)(self.size), self.next_version);
        self.next_version += 1;
        fs::write(&self.root_path, script).unwrap();
        self.runtime.reload_queue().push(self.root_path.clone());
        self.runtime
            .reload_now_with_host(benchmark_mount(), &mut self.host)
            .unwrap();
    }

    fn render_first_after_reload(&mut self) {
        self.runtime
            .render_frame_with_host(benchmark_mount(), &mut self.host)
            .unwrap();
    }
}

fn bench_steady_state_group(
    criterion: &mut Criterion,
    name: &str,
    script_factory: fn(usize) -> String,
) {
    let mut group = criterion.benchmark_group(format!("steady_state/{name}"));
    group.sample_size(10);

    for &size in SURFACE_SIZES {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |bench, &size| {
            bench.iter_batched(
                || {
                    let mut runtime = ScriptRuntime::new(benchmark_config());
                    let mut host = BenchHost;
                    runtime
                        .load_root_with_host(
                            ScriptSource::inline(
                                format!("{name}_{size}.luau"),
                                script_factory(size),
                            ),
                            benchmark_mount(),
                            &mut host,
                        )
                        .unwrap();
                    runtime
                        .render_frame_with_host(benchmark_mount(), &mut host)
                        .unwrap();
                    (runtime, host)
                },
                |(mut runtime, mut host)| {
                    runtime
                        .render_frame_with_host(benchmark_mount(), &mut host)
                        .unwrap();
                },
                BatchSize::PerIteration,
            );
        });
    }

    group.finish();
}

fn bench_first_after_reload_group(
    criterion: &mut Criterion,
    name: &str,
    script_factory: fn(usize) -> String,
) {
    let mut group = criterion.benchmark_group(format!("first_after_reload/{name}"));
    group.sample_size(10);

    for &size in SURFACE_SIZES {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |bench, &size| {
            bench.iter_batched(
                || {
                    let mut harness = ReloadHarness::new(script_factory, size);
                    harness.stage_reload();
                    harness
                },
                |mut harness| {
                    harness.render_first_after_reload();
                },
                BatchSize::PerIteration,
            );
        });
    }

    group.finish();
}

fn runtime_scale_benchmarks(criterion: &mut Criterion) {
    bench_steady_state_group(criterion, "label_flood", label_flood_script);
    bench_steady_state_group(criterion, "mixed_surface", mixed_surface_script);
    bench_steady_state_group(criterion, "virtual_list", virtual_list_script);
    bench_first_after_reload_group(criterion, "label_flood", label_flood_script);
    bench_first_after_reload_group(criterion, "mixed_surface", mixed_surface_script);
    bench_first_after_reload_group(criterion, "virtual_list", virtual_list_script);
}

criterion_group!(runtime_scale, runtime_scale_benchmarks);
criterion_main!(runtime_scale);
