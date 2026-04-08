#![doc = "Portable Luau runtime core for Rust host applications."]

mod runtime;
mod watch;

pub use runtime::{
    runtime_api_luau_typings, runtime_api_reference_markdown, ReloadKind, ReloadMetrics,
    ReloadQueue, RuntimeAppHost, RuntimeConfig, RuntimeError, RuntimeFailureClass,
    RuntimeFailureMetrics, RuntimeFailureRecord, RuntimeFailureStage, RuntimeFrameMemoryMetrics,
    RuntimeGcMetrics, RuntimeInstrumentationConfig, RuntimeLeakMetrics, RuntimeLeakSample,
    RuntimeLifecycleMetrics, RuntimeMemoryMetrics, RuntimeMemoryOperationMetrics,
    RuntimeReloadMemoryMetrics, RuntimeRenderKind, RuntimeStageMetrics, RuntimeUiHost,
    ScriptRuntime, ScriptSource, SurfaceCapabilities, SurfaceId, SurfaceMount, UiBooleanOutput,
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
pub use watch::RootScriptWatcher;
