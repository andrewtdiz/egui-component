use std::{
    cell::RefCell,
    collections::HashMap,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::NonNull,
    rc::Rc,
    sync::Arc,
};

use mlua::{Error as LuaError, Function, Lua, MultiValue, Table, Value};

use super::graph::RequireMode;
use super::{
    api::{function_spec, RuntimeApiFunctionId},
    parse::{
        ensure_no_extra_args, expect_string_arg, parse_begin_collapsible_call, parse_button_call,
        parse_button_group_call, parse_card_options, parse_checkbox_call, parse_container_options,
        parse_dropdown_menu_call, parse_label_call, parse_label_options, parse_number_input_call,
        parse_optional_table_arg, parse_progress_call, parse_radio_call, parse_select_call,
        parse_skeleton_call, parse_slider_call, parse_spinner_call, parse_switch_call,
        parse_tabs_call, parse_text_edit_call, parse_tooltip_call, parse_virtual_list_call,
    },
    runtime_error_from_lua_error, runtime_lua_error, unavailable_host_api, RuntimeAppHost,
    RuntimeError, RuntimeFailureStage, RuntimeUiHost, ScriptRuntime, SurfaceCapabilities,
    SurfaceId, SurfaceMount, UiBooleanOutput, UiButtonGroupOptions, UiButtonGroupOutput,
    UiButtonOptions, UiCardOptions, UiCheckboxOptions, UiCollapsibleOptions, UiCollapsibleOutput,
    UiContainerOptions, UiDropdownMenuEntry, UiDropdownMenuOptions, UiDropdownMenuOutput,
    UiLabelOptions, UiNumberInputOptions, UiNumberOutput, UiProgressOptions, UiRadioOptions,
    UiSelectOptions, UiSelectOutput, UiSkeletonOptions, UiSliderOptions, UiSpinnerOptions,
    UiSwitchOptions, UiTabOption, UiTabsOptions, UiTabsOutput, UiTextEditOptions, UiTextEditOutput,
    UiTooltipOptions, UiVirtualListOptions, UiVirtualListOutput,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HostPhase {
    Load,
    Reload,
    Update,
    Render,
}

impl HostPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Reload => "reload",
            Self::Update => "update",
            Self::Render => "render",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HostScopeToken(pub(super) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HostCapability {
    Log,
    Reload,
    Repaint,
}

impl HostCapability {
    fn as_str(self) -> &'static str {
        match self {
            Self::Log => "log",
            Self::Reload => "reload",
            Self::Repaint => "repaint",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HostCallKind {
    AppLog,
    AppRequestReload,
    AppRequestRepaint,
    UiLabel,
    UiSeparator,
    UiButton,
    UiTextEdit,
    UiCheckbox,
    UiSwitch,
    UiSlider,
    UiNumberInput,
    UiSelect,
    UiTabs,
    UiProgress,
    UiRadio,
    UiButtonGroup,
    UiBeginCollapsible,
    UiDropdownMenu,
    UiTooltip,
    UiSpinner,
    UiSkeleton,
    UiVirtualList,
    UiBeginRow,
    UiBeginColumn,
    UiBeginCard,
    UiEndScope,
    UiPushId,
    UiPopId,
}

impl HostCallKind {
    fn api_name(self) -> &'static str {
        match self {
            Self::AppLog => function_spec(RuntimeApiFunctionId::AppLog).full_name,
            Self::AppRequestReload => {
                function_spec(RuntimeApiFunctionId::AppRequestReload).full_name
            }
            Self::AppRequestRepaint => {
                function_spec(RuntimeApiFunctionId::AppRequestRepaint).full_name
            }
            Self::UiLabel => function_spec(RuntimeApiFunctionId::UiLabel).full_name,
            Self::UiSeparator => function_spec(RuntimeApiFunctionId::UiSeparator).full_name,
            Self::UiButton => function_spec(RuntimeApiFunctionId::UiButton).full_name,
            Self::UiTextEdit => function_spec(RuntimeApiFunctionId::UiTextEdit).full_name,
            Self::UiCheckbox => function_spec(RuntimeApiFunctionId::UiCheckbox).full_name,
            Self::UiSwitch => function_spec(RuntimeApiFunctionId::UiSwitch).full_name,
            Self::UiSlider => function_spec(RuntimeApiFunctionId::UiSlider).full_name,
            Self::UiNumberInput => function_spec(RuntimeApiFunctionId::UiNumberInput).full_name,
            Self::UiSelect => function_spec(RuntimeApiFunctionId::UiSelect).full_name,
            Self::UiTabs => function_spec(RuntimeApiFunctionId::UiTabs).full_name,
            Self::UiProgress => function_spec(RuntimeApiFunctionId::UiProgress).full_name,
            Self::UiRadio => function_spec(RuntimeApiFunctionId::UiRadio).full_name,
            Self::UiButtonGroup => function_spec(RuntimeApiFunctionId::UiButtonGroup).full_name,
            Self::UiBeginCollapsible => {
                function_spec(RuntimeApiFunctionId::UiBeginCollapsible).full_name
            }
            Self::UiDropdownMenu => function_spec(RuntimeApiFunctionId::UiDropdownMenu).full_name,
            Self::UiTooltip => function_spec(RuntimeApiFunctionId::UiTooltip).full_name,
            Self::UiSpinner => function_spec(RuntimeApiFunctionId::UiSpinner).full_name,
            Self::UiSkeleton => function_spec(RuntimeApiFunctionId::UiSkeleton).full_name,
            Self::UiVirtualList => function_spec(RuntimeApiFunctionId::UiVirtualList).full_name,
            Self::UiBeginRow => function_spec(RuntimeApiFunctionId::UiBeginRow).full_name,
            Self::UiBeginColumn => function_spec(RuntimeApiFunctionId::UiBeginColumn).full_name,
            Self::UiBeginCard => function_spec(RuntimeApiFunctionId::UiBeginCard).full_name,
            Self::UiEndScope => function_spec(RuntimeApiFunctionId::UiEndScope).full_name,
            Self::UiPushId => function_spec(RuntimeApiFunctionId::UiPushId).full_name,
            Self::UiPopId => function_spec(RuntimeApiFunctionId::UiPopId).full_name,
        }
    }

    fn required_capability(self) -> Option<HostCapability> {
        match self {
            Self::AppLog => Some(HostCapability::Log),
            Self::AppRequestReload => Some(HostCapability::Reload),
            Self::AppRequestRepaint => Some(HostCapability::Repaint),
            Self::UiLabel
            | Self::UiSeparator
            | Self::UiButton
            | Self::UiTextEdit
            | Self::UiCheckbox
            | Self::UiSwitch
            | Self::UiSlider
            | Self::UiNumberInput
            | Self::UiSelect
            | Self::UiTabs
            | Self::UiProgress
            | Self::UiRadio
            | Self::UiButtonGroup
            | Self::UiBeginCollapsible
            | Self::UiDropdownMenu
            | Self::UiTooltip
            | Self::UiSpinner
            | Self::UiSkeleton
            | Self::UiVirtualList
            | Self::UiBeginRow
            | Self::UiBeginColumn
            | Self::UiBeginCard
            | Self::UiEndScope
            | Self::UiPushId
            | Self::UiPopId => None,
        }
    }

    fn requires_frame_phase(self) -> bool {
        matches!(self, Self::AppRequestReload | Self::AppRequestRepaint)
    }

    fn requires_render_phase(self) -> bool {
        matches!(
            self,
            Self::UiLabel
                | Self::UiSeparator
                | Self::UiButton
                | Self::UiTextEdit
                | Self::UiCheckbox
                | Self::UiSwitch
                | Self::UiSlider
                | Self::UiNumberInput
                | Self::UiSelect
                | Self::UiTabs
                | Self::UiProgress
                | Self::UiRadio
                | Self::UiButtonGroup
                | Self::UiBeginCollapsible
                | Self::UiDropdownMenu
                | Self::UiTooltip
                | Self::UiSpinner
                | Self::UiSkeleton
                | Self::UiVirtualList
                | Self::UiBeginRow
                | Self::UiBeginColumn
                | Self::UiBeginCard
                | Self::UiEndScope
                | Self::UiPushId
                | Self::UiPopId
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AppProxyMember {
    Log,
    RequestReload,
    RequestRepaint,
}

impl AppProxyMember {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "log" => Some(Self::Log),
            "request_reload" => Some(Self::RequestReload),
            "request_repaint" => Some(Self::RequestRepaint),
            _ => None,
        }
    }

    fn function(self) -> &'static super::api::RuntimeApiFunctionSpec {
        match self {
            Self::Log => function_spec(RuntimeApiFunctionId::AppLog),
            Self::RequestReload => function_spec(RuntimeApiFunctionId::AppRequestReload),
            Self::RequestRepaint => function_spec(RuntimeApiFunctionId::AppRequestRepaint),
        }
    }

    fn kind(self) -> HostCallKind {
        match self {
            Self::Log => HostCallKind::AppLog,
            Self::RequestReload => HostCallKind::AppRequestReload,
            Self::RequestRepaint => HostCallKind::AppRequestRepaint,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum UiProxyMember {
    Label,
    Separator,
    Button,
    TextEdit,
    Checkbox,
    Switch,
    Slider,
    NumberInput,
    Select,
    Tabs,
    Progress,
    Radio,
    ButtonGroup,
    BeginCollapsible,
    DropdownMenu,
    Tooltip,
    Spinner,
    Skeleton,
    VirtualList,
    BeginRow,
    BeginColumn,
    BeginCard,
    EndScope,
    PushId,
    PopId,
}

impl UiProxyMember {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "label" => Some(Self::Label),
            "separator" => Some(Self::Separator),
            "button" => Some(Self::Button),
            "text_edit" => Some(Self::TextEdit),
            "checkbox" => Some(Self::Checkbox),
            "switch" => Some(Self::Switch),
            "slider" => Some(Self::Slider),
            "number_input" => Some(Self::NumberInput),
            "select" => Some(Self::Select),
            "tabs" => Some(Self::Tabs),
            "progress" => Some(Self::Progress),
            "radio" => Some(Self::Radio),
            "button_group" => Some(Self::ButtonGroup),
            "begin_collapsible" => Some(Self::BeginCollapsible),
            "dropdown_menu" => Some(Self::DropdownMenu),
            "tooltip" => Some(Self::Tooltip),
            "spinner" => Some(Self::Spinner),
            "skeleton" => Some(Self::Skeleton),
            "virtual_list" => Some(Self::VirtualList),
            "begin_row" => Some(Self::BeginRow),
            "begin_column" => Some(Self::BeginColumn),
            "begin_card" => Some(Self::BeginCard),
            "end_scope" | "end" => Some(Self::EndScope),
            "push_id" => Some(Self::PushId),
            "pop_id" => Some(Self::PopId),
            _ => None,
        }
    }

    fn function(self) -> &'static super::api::RuntimeApiFunctionSpec {
        match self {
            Self::Label => function_spec(RuntimeApiFunctionId::UiLabel),
            Self::Separator => function_spec(RuntimeApiFunctionId::UiSeparator),
            Self::Button => function_spec(RuntimeApiFunctionId::UiButton),
            Self::TextEdit => function_spec(RuntimeApiFunctionId::UiTextEdit),
            Self::Checkbox => function_spec(RuntimeApiFunctionId::UiCheckbox),
            Self::Switch => function_spec(RuntimeApiFunctionId::UiSwitch),
            Self::Slider => function_spec(RuntimeApiFunctionId::UiSlider),
            Self::NumberInput => function_spec(RuntimeApiFunctionId::UiNumberInput),
            Self::Select => function_spec(RuntimeApiFunctionId::UiSelect),
            Self::Tabs => function_spec(RuntimeApiFunctionId::UiTabs),
            Self::Progress => function_spec(RuntimeApiFunctionId::UiProgress),
            Self::Radio => function_spec(RuntimeApiFunctionId::UiRadio),
            Self::ButtonGroup => function_spec(RuntimeApiFunctionId::UiButtonGroup),
            Self::BeginCollapsible => function_spec(RuntimeApiFunctionId::UiBeginCollapsible),
            Self::DropdownMenu => function_spec(RuntimeApiFunctionId::UiDropdownMenu),
            Self::Tooltip => function_spec(RuntimeApiFunctionId::UiTooltip),
            Self::Spinner => function_spec(RuntimeApiFunctionId::UiSpinner),
            Self::Skeleton => function_spec(RuntimeApiFunctionId::UiSkeleton),
            Self::VirtualList => function_spec(RuntimeApiFunctionId::UiVirtualList),
            Self::BeginRow => function_spec(RuntimeApiFunctionId::UiBeginRow),
            Self::BeginColumn => function_spec(RuntimeApiFunctionId::UiBeginColumn),
            Self::BeginCard => function_spec(RuntimeApiFunctionId::UiBeginCard),
            Self::EndScope => function_spec(RuntimeApiFunctionId::UiEndScope),
            Self::PushId => function_spec(RuntimeApiFunctionId::UiPushId),
            Self::PopId => function_spec(RuntimeApiFunctionId::UiPopId),
        }
    }

    fn kind(self) -> HostCallKind {
        match self {
            Self::Label => HostCallKind::UiLabel,
            Self::Separator => HostCallKind::UiSeparator,
            Self::Button => HostCallKind::UiButton,
            Self::TextEdit => HostCallKind::UiTextEdit,
            Self::Checkbox => HostCallKind::UiCheckbox,
            Self::Switch => HostCallKind::UiSwitch,
            Self::Slider => HostCallKind::UiSlider,
            Self::NumberInput => HostCallKind::UiNumberInput,
            Self::Select => HostCallKind::UiSelect,
            Self::Tabs => HostCallKind::UiTabs,
            Self::Progress => HostCallKind::UiProgress,
            Self::Radio => HostCallKind::UiRadio,
            Self::ButtonGroup => HostCallKind::UiButtonGroup,
            Self::BeginCollapsible => HostCallKind::UiBeginCollapsible,
            Self::DropdownMenu => HostCallKind::UiDropdownMenu,
            Self::Tooltip => HostCallKind::UiTooltip,
            Self::Spinner => HostCallKind::UiSpinner,
            Self::Skeleton => HostCallKind::UiSkeleton,
            Self::VirtualList => HostCallKind::UiVirtualList,
            Self::BeginRow => HostCallKind::UiBeginRow,
            Self::BeginColumn => HostCallKind::UiBeginColumn,
            Self::BeginCard => HostCallKind::UiBeginCard,
            Self::EndScope => HostCallKind::UiEndScope,
            Self::PushId => HostCallKind::UiPushId,
            Self::PopId => HostCallKind::UiPopId,
        }
    }
}

#[derive(Clone)]
pub(super) struct HostBridgeCache {
    app: Table,
    ui: Table,
    session: Rc<RefCell<HostBridgeSession>>,
    virtual_list_cache: Rc<RefCell<VirtualListCache>>,
}

impl HostBridgeCache {
    fn new(lua: &Lua) -> mlua::Result<Self> {
        let session = Rc::new(RefCell::new(HostBridgeSession::default()));
        let virtual_list_cache = Rc::new(RefCell::new(VirtualListCache::default()));
        let app = build_app_proxy_table(lua, Rc::clone(&session))?;
        let ui = build_ui_proxy_table(lua, Rc::clone(&session))?;
        Ok(Self {
            app,
            ui,
            session,
            virtual_list_cache,
        })
    }

    fn prune_virtual_list_generation(&self, generation: u64) {
        self.virtual_list_cache
            .borrow_mut()
            .prune_to_generation(generation);
    }

    #[cfg(test)]
    fn virtual_list_cache_stats(&self) -> (usize, usize, usize) {
        self.virtual_list_cache.borrow().stats()
    }
}

#[derive(Clone)]
struct HostBridgeScope {
    access: Rc<RefCell<ActiveHostScope>>,
    token: HostScopeToken,
    surface_id: SurfaceId,
    app_host: Option<ScopedRuntimeAppHostBridge>,
    ui_host: Option<ScopedRuntimeUiHostBridge>,
    scoped_ui_state: Option<Rc<RefCell<ScopedUiState>>>,
    frame_commands: Option<Rc<RefCell<FrameCommandQueue>>>,
    virtual_list_cache: Rc<RefCell<VirtualListCache>>,
    virtual_list_generation: u64,
}

impl HostBridgeScope {
    fn with_app_host<R>(
        &self,
        f: impl FnOnce(&mut dyn RuntimeAppHost) -> Result<R, RuntimeError>,
    ) -> mlua::Result<R> {
        if let Some(ui_host) = self.ui_host {
            ui_host.with_host(|host| f(host))
        } else if let Some(app_host) = self.app_host {
            app_host.with_host(f)
        } else {
            Err(LuaError::external(unavailable_host_api("app")))
        }
    }

    fn with_ui_host<R>(
        &self,
        f: impl FnOnce(&mut dyn RuntimeUiHost) -> Result<R, RuntimeError>,
    ) -> mlua::Result<R> {
        self.ui_host
            .ok_or_else(|| LuaError::external(unavailable_host_api("ui")))
            .and_then(|ui_host| ui_host.with_host(f))
    }

    fn resolve_virtual_list_items(
        &self,
        table: &Table,
        fn_name: &str,
        index: usize,
    ) -> mlua::Result<Arc<[String]>> {
        self.virtual_list_cache.borrow_mut().resolve(
            table,
            fn_name,
            index,
            self.virtual_list_generation,
        )
    }
}

const VIRTUAL_LIST_CACHE_MAX_ENTRIES: usize = 16;
const VIRTUAL_LIST_CACHE_MAX_BYTES: usize = 16 * 1024 * 1024;
const VIRTUAL_LIST_ITEMS_ARG_INDEX: usize = 2;

#[derive(Debug)]
struct VirtualListCacheEntry {
    table: Table,
    items: Arc<[String]>,
    item_keys: Vec<usize>,
    total_string_bytes: usize,
    readonly: bool,
    generation: u64,
    last_used: u64,
}

#[derive(Default)]
struct VirtualListCache {
    entries: Vec<VirtualListCacheEntry>,
    total_string_bytes: usize,
    next_use_tick: u64,
    #[cfg(test)]
    hits: usize,
    #[cfg(test)]
    readonly_hits: usize,
    #[cfg(test)]
    rebuilds: usize,
}

impl VirtualListCache {
    fn resolve(
        &mut self,
        table: &Table,
        fn_name: &str,
        index: usize,
        generation: u64,
    ) -> mlua::Result<Arc<[String]>> {
        let table_pointer = table.to_pointer() as usize;
        if let Some(entry_index) = self
            .entries
            .iter()
            .position(|entry| entry.table.to_pointer() as usize == table_pointer)
        {
            if self.entries[entry_index].readonly {
                return Ok(self.record_hit(entry_index, generation, true));
            }

            let changed = super::parse::string_list_table_changed(
                table,
                fn_name,
                index,
                &self.entries[entry_index].item_keys,
            )?;
            if !changed {
                return Ok(self.record_hit(entry_index, generation, false));
            }

            let snapshot = super::parse::snapshot_string_list_table(table, fn_name, index)?;
            let items: Arc<[String]> = snapshot.items.into();
            let last_used = self.next_use_tick();
            let previous_bytes = self.entries[entry_index].total_string_bytes;
            self.total_string_bytes =
                self.total_string_bytes + snapshot.total_string_bytes - previous_bytes;
            {
                let entry = &mut self.entries[entry_index];
                entry.table = table.clone();
                entry.items = Arc::clone(&items);
                entry.item_keys = snapshot.item_keys;
                entry.total_string_bytes = snapshot.total_string_bytes;
                entry.readonly = table.is_readonly();
                entry.generation = generation;
                entry.last_used = last_used;
            }
            #[cfg(test)]
            {
                self.rebuilds += 1;
            }
            self.evict_if_needed(Some(table_pointer));
            return Ok(items);
        }

        let snapshot = super::parse::snapshot_string_list_table(table, fn_name, index)?;
        let items: Arc<[String]> = snapshot.items.into();
        let last_used = self.next_use_tick();
        self.total_string_bytes += snapshot.total_string_bytes;
        self.entries.push(VirtualListCacheEntry {
            table: table.clone(),
            items: Arc::clone(&items),
            item_keys: snapshot.item_keys,
            total_string_bytes: snapshot.total_string_bytes,
            readonly: table.is_readonly(),
            generation,
            last_used,
        });
        #[cfg(test)]
        {
            self.rebuilds += 1;
        }
        self.evict_if_needed(Some(table_pointer));
        Ok(items)
    }

    fn prune_to_generation(&mut self, generation: u64) {
        let mut retained = Vec::with_capacity(self.entries.len());
        let mut total_string_bytes = 0;
        for entry in self.entries.drain(..) {
            if entry.generation == generation {
                total_string_bytes += entry.total_string_bytes;
                retained.push(entry);
            }
        }
        self.entries = retained;
        self.total_string_bytes = total_string_bytes;
    }

    #[cfg(test)]
    fn stats(&self) -> (usize, usize, usize) {
        (self.hits, self.readonly_hits, self.rebuilds)
    }

    fn record_hit(
        &mut self,
        entry_index: usize,
        generation: u64,
        _readonly_hit: bool,
    ) -> Arc<[String]> {
        let last_used = self.next_use_tick();
        let entry = &mut self.entries[entry_index];
        entry.generation = generation;
        entry.last_used = last_used;
        #[cfg(test)]
        {
            self.hits += 1;
            if _readonly_hit {
                self.readonly_hits += 1;
            }
        }
        Arc::clone(&entry.items)
    }

    fn next_use_tick(&mut self) -> u64 {
        self.next_use_tick += 1;
        self.next_use_tick
    }

    fn evict_if_needed(&mut self, keep_table_pointer: Option<usize>) {
        while self.entries.len() > VIRTUAL_LIST_CACHE_MAX_ENTRIES
            || (self.total_string_bytes > VIRTUAL_LIST_CACHE_MAX_BYTES && self.entries.len() > 1)
        {
            let Some(victim_index) = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| Some(entry.table.to_pointer() as usize) != keep_table_pointer)
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(index, _)| index)
            else {
                break;
            };
            let victim = self.entries.swap_remove(victim_index);
            self.total_string_bytes -= victim.total_string_bytes;
        }
    }
}

#[derive(Default)]
struct HostBridgeSession {
    current_scope: Option<HostBridgeScope>,
    app_wrappers: HashMap<AppProxyMember, Function>,
    ui_wrappers: HashMap<UiProxyMember, Function>,
    #[cfg(test)]
    wrapper_creations: usize,
}

impl HostBridgeSession {
    fn begin_scope(&mut self, scope: HostBridgeScope) {
        self.current_scope = Some(scope);
        self.app_wrappers.clear();
        self.ui_wrappers.clear();
        #[cfg(test)]
        {
            self.wrapper_creations = 0;
        }
    }

    fn end_scope(&mut self) {
        self.current_scope = None;
        self.app_wrappers.clear();
        self.ui_wrappers.clear();
    }

    fn app_wrapper(&mut self, lua: &Lua, member: AppProxyMember) -> mlua::Result<Option<Function>> {
        let Some(scope) = self.current_scope.clone() else {
            return Ok(None);
        };
        if let Some(function) = self.app_wrappers.get(&member) {
            return Ok(Some(function.clone()));
        }
        let function = create_app_wrapper(lua, scope, member)?;
        #[cfg(test)]
        {
            self.wrapper_creations += 1;
        }
        self.app_wrappers.insert(member, function.clone());
        Ok(Some(function))
    }

    fn ui_wrapper(&mut self, lua: &Lua, member: UiProxyMember) -> mlua::Result<Option<Function>> {
        let Some(scope) = self.current_scope.clone() else {
            return Ok(None);
        };
        if let Some(function) = self.ui_wrappers.get(&member) {
            return Ok(Some(function.clone()));
        }
        let function = create_ui_wrapper(lua, scope, member)?;
        #[cfg(test)]
        {
            self.wrapper_creations += 1;
        }
        self.ui_wrappers.insert(member, function.clone());
        Ok(Some(function))
    }
}

struct ScopedBridgeSession {
    session: Rc<RefCell<HostBridgeSession>>,
}

impl ScopedBridgeSession {
    fn begin(session: Rc<RefCell<HostBridgeSession>>, scope: HostBridgeScope) -> Self {
        session.borrow_mut().begin_scope(scope);
        Self { session }
    }
}

impl Drop for ScopedBridgeSession {
    fn drop(&mut self) {
        self.session.borrow_mut().end_scope();
    }
}

#[derive(Debug, Clone)]
pub(super) struct ActiveHostScope {
    token: HostScopeToken,
    surface_id: SurfaceId,
    capabilities: SurfaceCapabilities,
    phase: HostPhase,
    active: bool,
}

impl ActiveHostScope {
    fn new(token: HostScopeToken, mount: &SurfaceMount, phase: HostPhase) -> Self {
        Self {
            token,
            surface_id: mount.surface_id.clone(),
            capabilities: mount.capabilities,
            phase,
            active: true,
        }
    }

    fn deactivate(&mut self) {
        self.active = false;
    }

    fn set_phase(&mut self, phase: HostPhase) {
        self.phase = phase;
    }

    fn validate(
        &self,
        expected_token: HostScopeToken,
        expected_surface_id: &SurfaceId,
        kind: HostCallKind,
    ) -> Result<(), RuntimeError> {
        if !self.active || self.token != expected_token {
            return Err(RuntimeError::backend(format!(
                "surface `{}` attempted to call `{}` after the frame closed",
                expected_surface_id,
                kind.api_name()
            )));
        }

        if self.surface_id != *expected_surface_id {
            return Err(RuntimeError::backend(format!(
                "surface `{}` attempted to call `{}` with stale surface bindings for `{}`",
                expected_surface_id,
                kind.api_name(),
                self.surface_id
            )));
        }

        if kind.requires_frame_phase()
            && !matches!(self.phase, HostPhase::Update | HostPhase::Render)
        {
            return Err(RuntimeError::backend(format!(
                "surface `{}` attempted to call frame-only API `{}` during {}",
                self.surface_id,
                kind.api_name(),
                self.phase.as_str()
            )));
        }

        if kind.requires_render_phase() && self.phase != HostPhase::Render {
            return Err(RuntimeError::backend(format!(
                "surface `{}` attempted to call render-only API `{}` during {}",
                self.surface_id,
                kind.api_name(),
                self.phase.as_str()
            )));
        }

        if let Some(capability) = kind.required_capability() {
            if !self.capabilities.allows(capability) {
                return Err(RuntimeError::backend(format!(
                    "surface `{}` is not allowed to call `{}` because capability `{}` is disabled",
                    self.surface_id,
                    kind.api_name(),
                    capability.as_str()
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameCommand {
    RequestReload,
    RequestRepaint,
}

impl FrameCommand {
    fn api_name(self) -> &'static str {
        match self {
            Self::RequestReload => function_spec(RuntimeApiFunctionId::AppRequestReload).full_name,
            Self::RequestRepaint => {
                function_spec(RuntimeApiFunctionId::AppRequestRepaint).full_name
            }
        }
    }
}

#[derive(Debug, Default)]
struct FrameCommandQueue {
    commands: Vec<FrameCommand>,
}

impl FrameCommandQueue {
    fn push(&mut self, command: FrameCommand) {
        self.commands.push(command);
    }

    fn take(&mut self) -> Vec<FrameCommand> {
        std::mem::take(&mut self.commands)
    }
}

impl SurfaceCapabilities {
    fn allows(self, capability: HostCapability) -> bool {
        match capability {
            HostCapability::Log => self.log,
            HostCapability::Reload => self.reload,
            HostCapability::Repaint => self.repaint,
        }
    }
}

struct ScopedActiveHostScope {
    scope: Rc<RefCell<ActiveHostScope>>,
}

impl ScopedActiveHostScope {
    fn new(scope: Rc<RefCell<ActiveHostScope>>) -> Self {
        Self { scope }
    }
}

impl Drop for ScopedActiveHostScope {
    fn drop(&mut self) {
        self.scope.borrow_mut().deactivate();
    }
}

#[derive(Clone, Copy)]
struct ScopedRuntimeAppHostBridge {
    host: NonNull<dyn RuntimeAppHost>,
}

impl ScopedRuntimeAppHostBridge {
    fn new(host: &mut dyn RuntimeAppHost) -> Self {
        let host_ptr = unsafe {
            std::mem::transmute::<&mut dyn RuntimeAppHost, *mut dyn RuntimeAppHost>(host)
        };
        Self {
            host: NonNull::new(host_ptr).expect("host pointer is never null"),
        }
    }

    fn with_host<R>(
        &self,
        f: impl FnOnce(&mut dyn RuntimeAppHost) -> Result<R, RuntimeError>,
    ) -> mlua::Result<R> {
        let mut host = self.host;
        unsafe { f(host.as_mut()).map_err(LuaError::external) }
    }
}

#[derive(Clone, Copy)]
struct ScopedRuntimeUiHostBridge {
    host: NonNull<dyn RuntimeUiHost>,
}

impl ScopedRuntimeUiHostBridge {
    fn new(host: &mut dyn RuntimeUiHost) -> Self {
        let host_ptr =
            unsafe { std::mem::transmute::<&mut dyn RuntimeUiHost, *mut dyn RuntimeUiHost>(host) };
        Self {
            host: NonNull::new(host_ptr).expect("host pointer is never null"),
        }
    }

    fn with_host<R>(
        &self,
        f: impl FnOnce(&mut dyn RuntimeUiHost) -> Result<R, RuntimeError>,
    ) -> mlua::Result<R> {
        let mut host = self.host;
        unsafe { f(host.as_mut()).map_err(LuaError::external) }
    }
}

struct ScopedHostGlobals {
    globals: Table,
    previous_app: Value,
    previous_ui: Value,
}

impl ScopedHostGlobals {
    fn install(globals: Table, app: Table, ui: Table) -> mlua::Result<Self> {
        let previous_app = globals.get::<Value>("app")?;
        let previous_ui = globals.get::<Value>("ui")?;
        globals.set("app", app)?;
        globals.set("ui", ui)?;
        Ok(Self {
            globals,
            previous_app,
            previous_ui,
        })
    }
}

impl Drop for ScopedHostGlobals {
    fn drop(&mut self) {
        let _ = self.globals.set("app", self.previous_app.clone());
        let _ = self.globals.set("ui", self.previous_ui.clone());
    }
}

#[derive(Debug, Default)]
struct ScopedUiState {
    open_containers: usize,
    open_ids: usize,
}

impl ScopedUiState {
    fn begin_container(&mut self) {
        self.open_containers += 1;
    }

    fn end_container(&mut self) -> mlua::Result<()> {
        if self.open_containers == 0 {
            return Err(runtime_lua_error(
                "ui.end_scope called without an open container scope",
            ));
        }
        self.open_containers -= 1;
        Ok(())
    }

    fn push_id(&mut self) {
        self.open_ids += 1;
    }

    fn pop_id(&mut self) -> mlua::Result<()> {
        if self.open_ids == 0 {
            return Err(runtime_lua_error(
                "ui.pop_id called without a pushed id scope",
            ));
        }
        self.open_ids -= 1;
        Ok(())
    }

    fn ensure_balanced(&self) -> Result<(), RuntimeError> {
        match (self.open_containers, self.open_ids) {
            (0, 0) => Ok(()),
            (containers, 0) => Err(RuntimeError::backend(format!(
                "render exited with {containers} unclosed container scope(s)"
            ))),
            (0, ids) => Err(RuntimeError::backend(format!(
                "render exited with {ids} unclosed id scope(s)"
            ))),
            (containers, ids) => Err(RuntimeError::backend(format!(
                "render exited with {containers} unclosed container scope(s) and {ids} unclosed id scope(s)"
            ))),
        }
    }
}

pub(super) struct NullRuntimeHost;

impl RuntimeAppHost for NullRuntimeHost {
    fn log(&mut self, _level: &str, _message: &str) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("app.log"))
    }

    fn request_reload(&mut self) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("app.request_reload"))
    }

    fn request_repaint(&mut self) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("app.request_repaint"))
    }
}

impl RuntimeUiHost for NullRuntimeHost {
    fn label(&mut self, _text: &str, _options: UiLabelOptions) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.label"))
    }

    fn separator(&mut self) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.separator"))
    }

    fn button(
        &mut self,
        _id: &str,
        _text: &str,
        _options: UiButtonOptions,
    ) -> Result<bool, RuntimeError> {
        Err(unavailable_host_api("ui.button"))
    }

    fn text_edit(
        &mut self,
        _id: &str,
        _value: &str,
        _options: UiTextEditOptions,
    ) -> Result<UiTextEditOutput, RuntimeError> {
        Err(unavailable_host_api("ui.text_edit"))
    }

    fn checkbox(
        &mut self,
        _id: &str,
        _checked: bool,
        _options: UiCheckboxOptions,
    ) -> Result<UiBooleanOutput, RuntimeError> {
        Err(unavailable_host_api("ui.checkbox"))
    }

    fn switch(
        &mut self,
        _id: &str,
        _checked: bool,
        _options: UiSwitchOptions,
    ) -> Result<UiBooleanOutput, RuntimeError> {
        Err(unavailable_host_api("ui.switch"))
    }

    fn slider(
        &mut self,
        _id: &str,
        _value: f32,
        _options: UiSliderOptions,
    ) -> Result<UiNumberOutput, RuntimeError> {
        Err(unavailable_host_api("ui.slider"))
    }

    fn number_input(
        &mut self,
        _id: &str,
        _value: f32,
        _options: UiNumberInputOptions,
    ) -> Result<UiNumberOutput, RuntimeError> {
        Err(unavailable_host_api("ui.number_input"))
    }

    fn select(
        &mut self,
        _id: &str,
        _selected_index: usize,
        _items: &[String],
        _options: UiSelectOptions,
    ) -> Result<UiSelectOutput, RuntimeError> {
        Err(unavailable_host_api("ui.select"))
    }

    fn tabs(
        &mut self,
        _id: &str,
        _selected_index: usize,
        _options: &[UiTabOption],
        _props: UiTabsOptions,
    ) -> Result<UiTabsOutput, RuntimeError> {
        Err(unavailable_host_api("ui.tabs"))
    }

    fn progress(&mut self, _value: f32, _options: UiProgressOptions) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.progress"))
    }

    fn radio(
        &mut self,
        _id: &str,
        _selected: bool,
        _options: UiRadioOptions,
    ) -> Result<UiBooleanOutput, RuntimeError> {
        Err(unavailable_host_api("ui.radio"))
    }

    fn button_group(
        &mut self,
        _id: &str,
        _options: &[String],
        _props: UiButtonGroupOptions,
    ) -> Result<UiButtonGroupOutput, RuntimeError> {
        Err(unavailable_host_api("ui.button_group"))
    }

    fn begin_collapsible(
        &mut self,
        _id: &str,
        _title: &str,
        _open: bool,
        _options: UiCollapsibleOptions,
    ) -> Result<UiCollapsibleOutput, RuntimeError> {
        Err(unavailable_host_api("ui.begin_collapsible"))
    }

    fn dropdown_menu(
        &mut self,
        _id: &str,
        _trigger_label: &str,
        _entries: &[UiDropdownMenuEntry],
        _options: UiDropdownMenuOptions,
    ) -> Result<UiDropdownMenuOutput, RuntimeError> {
        Err(unavailable_host_api("ui.dropdown_menu"))
    }

    fn tooltip(
        &mut self,
        _trigger_label: &str,
        _text: &str,
        _options: UiTooltipOptions,
    ) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.tooltip"))
    }

    fn spinner(&mut self, _options: UiSpinnerOptions) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.spinner"))
    }

    fn skeleton(&mut self, _options: UiSkeletonOptions) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.skeleton"))
    }

    fn virtual_list(
        &mut self,
        _id: &str,
        _items: &[String],
        _options: UiVirtualListOptions,
    ) -> Result<UiVirtualListOutput, RuntimeError> {
        Err(unavailable_host_api("ui.virtual_list"))
    }

    fn begin_row(&mut self, _options: UiContainerOptions) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.begin_row"))
    }

    fn begin_column(&mut self, _options: UiContainerOptions) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.begin_column"))
    }

    fn begin_card(&mut self, _options: UiCardOptions) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.begin_card"))
    }

    fn end(&mut self) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.end_scope"))
    }

    fn push_id(&mut self, _id: &str) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.push_id"))
    }

    fn pop_id(&mut self) -> Result<(), RuntimeError> {
        Err(unavailable_host_api("ui.pop_id"))
    }
}

impl ScriptRuntime {
    fn cached_host_bridge(&mut self) -> Result<HostBridgeCache, RuntimeError> {
        if self.bridge_cache.is_none() {
            self.bridge_cache =
                Some(HostBridgeCache::new(&self.lua).map_err(runtime_error_from_lua_error)?);
        }
        Ok(self
            .bridge_cache
            .as_ref()
            .expect("bridge cache initialized")
            .clone())
    }

    #[cfg(test)]
    pub(super) fn test_bridge_wrapper_creations(&self) -> usize {
        self.bridge_cache
            .as_ref()
            .map(|cache| cache.session.borrow().wrapper_creations)
            .unwrap_or(0)
    }

    #[cfg(test)]
    pub(super) fn test_virtual_list_cache_stats(&self) -> (usize, usize, usize) {
        self.bridge_cache
            .as_ref()
            .map(HostBridgeCache::virtual_list_cache_stats)
            .unwrap_or((0, 0, 0))
    }

    pub(super) fn promote_bridge_generation(&mut self) {
        self.bridge_generation += 1;
        if let Some(cache) = &self.bridge_cache {
            cache.prune_virtual_list_generation(self.bridge_generation);
        }
    }

    pub(super) fn discard_pending_bridge_generation(&mut self) {
        if let Some(cache) = &self.bridge_cache {
            cache.prune_virtual_list_generation(self.bridge_generation);
        }
    }

    pub(super) fn with_app_host_scope<T>(
        &mut self,
        host: &mut dyn RuntimeAppHost,
        mount: &SurfaceMount,
        phase: HostPhase,
        f: impl FnOnce(&mut Self) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        let lua = self.lua.clone();
        let bridge_cache = self.cached_host_bridge()?;
        let access = Rc::new(RefCell::new(ActiveHostScope::new(
            self.next_scope_token(),
            mount,
            phase,
        )));
        let bridge = ScopedRuntimeAppHostBridge::new(host);
        let globals = lua.globals();
        let token = access.borrow().token;
        let scope = HostBridgeScope {
            access: Rc::clone(&access),
            token,
            surface_id: mount.surface_id.clone(),
            app_host: Some(bridge),
            ui_host: None,
            scoped_ui_state: None,
            frame_commands: None,
            virtual_list_cache: Rc::clone(&bridge_cache.virtual_list_cache),
            virtual_list_generation: self.bridge_generation,
        };
        let _bridge_session = ScopedBridgeSession::begin(Rc::clone(&bridge_cache.session), scope);
        let _globals =
            ScopedHostGlobals::install(globals, bridge_cache.app.clone(), bridge_cache.ui.clone())
                .map_err(runtime_error_from_lua_error)?;
        let _host_scope = ScopedActiveHostScope::new(Rc::clone(&access));
        f(self)
    }

    pub(super) fn with_ui_host_scope<T>(
        &mut self,
        host: &mut dyn RuntimeUiHost,
        mount: &SurfaceMount,
        frame_stage: RuntimeFailureStage,
        rolled_back_on_failure: bool,
        commit_candidate_on_success: bool,
        f: impl FnOnce(&mut Self, &Rc<RefCell<ActiveHostScope>>) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        let lua = self.lua.clone();
        let bridge_cache = self.cached_host_bridge()?;
        let scoped_ui_state = Rc::new(RefCell::new(ScopedUiState::default()));
        let frame_commands = Rc::new(RefCell::new(FrameCommandQueue::default()));
        let access = Rc::new(RefCell::new(ActiveHostScope::new(
            self.next_scope_token(),
            mount,
            HostPhase::Update,
        )));
        let virtual_list_generation = if commit_candidate_on_success {
            self.bridge_generation + 1
        } else {
            self.bridge_generation
        };
        let bridge = ScopedRuntimeUiHostBridge::new(host);
        let globals = lua.globals();
        let token = access.borrow().token;
        let scope = HostBridgeScope {
            access: Rc::clone(&access),
            token,
            surface_id: mount.surface_id.clone(),
            app_host: None,
            ui_host: Some(bridge),
            scoped_ui_state: Some(Rc::clone(&scoped_ui_state)),
            frame_commands: Some(Rc::clone(&frame_commands)),
            virtual_list_cache: Rc::clone(&bridge_cache.virtual_list_cache),
            virtual_list_generation,
        };
        let _bridge_session =
            ScopedBridgeSession::begin(Rc::clone(&bridge_cache.session), scope.clone());
        let _globals =
            ScopedHostGlobals::install(globals, bridge_cache.app.clone(), bridge_cache.ui.clone())
                .map_err(runtime_error_from_lua_error)?;
        let _host_scope = ScopedActiveHostScope::new(Rc::clone(&access));
        let value = f(self, &access)?;
        scoped_ui_state.borrow().ensure_balanced().map_err(|err| {
            self.record_frame_runtime_error(
                frame_stage,
                HostPhase::Render,
                self.current_root_module_context(),
                err,
                rolled_back_on_failure,
            )
        })?;
        let commands = frame_commands.borrow_mut().take();
        bridge
            .with_host(|host| {
                apply_frame_commands(host, &mount.surface_id, commands).map_err(|err| {
                    self.record_command_flush_failure(
                        frame_stage,
                        Some(err.api_name),
                        err.error,
                        rolled_back_on_failure,
                    )
                })
            })
            .map_err(runtime_error_from_lua_error)?;
        if commit_candidate_on_success {
            self.commit_staged_candidate(frame_stage)?;
        }
        Ok(value)
    }

    pub(super) fn render_frame_inner(
        &mut self,
        access: &RefCell<ActiveHostScope>,
        frame_stage: RuntimeFailureStage,
        rolled_back_on_failure: bool,
    ) -> Result<(), RuntimeError> {
        let frame_runtime = self
            .pending_candidate
            .as_ref()
            .map(|candidate| candidate.runtime.clone())
            .or_else(|| self.active_runtime.clone())
            .ok_or_else(|| RuntimeError::config("no root script has been loaded"))?;
        let validating_candidate = self.pending_candidate.is_some();
        let update = frame_runtime.root_update.clone();
        let render = frame_runtime.root_render.clone();
        let state = frame_runtime.root_state.clone();
        let module_context = Some(frame_runtime.graph.root_context().to_owned());
        let previous = {
            let mut mode = self.require_mode.lock();
            std::mem::replace(
                &mut *mode,
                RequireMode::Active(Arc::clone(&frame_runtime.require_snapshot)),
            )
        };
        let frame_result = (|| -> Result<(), RuntimeError> {
            // Staged candidates validate with render only so update hooks cannot mutate
            // candidate state before the runtime commits the new graph.
            if !validating_candidate {
                if let Some(update) = update {
                    access.borrow_mut().set_phase(HostPhase::Update);
                    let input = self.lua.create_table().map_err(|err| {
                        self.record_frame_lua_failure(
                            frame_stage,
                            HostPhase::Update,
                            module_context.clone(),
                            err,
                            rolled_back_on_failure,
                        )
                    })?;
                    update.call::<()>((state.clone(), input)).map_err(|err| {
                        self.record_frame_lua_failure(
                            frame_stage,
                            HostPhase::Update,
                            module_context.clone(),
                            err,
                            rolled_back_on_failure,
                        )
                    })?;
                }
            }

            access.borrow_mut().set_phase(HostPhase::Render);
            render.call::<()>(state).map_err(|err| {
                self.record_frame_lua_failure(
                    frame_stage,
                    HostPhase::Render,
                    module_context,
                    err,
                    rolled_back_on_failure,
                )
            })?;
            Ok(())
        })();
        let current = {
            let mut mode = self.require_mode.lock();
            std::mem::replace(&mut *mode, previous)
        };
        match current {
            RequireMode::Active(_) => {}
            _ => unreachable!("require mode changed unexpectedly during render"),
        }

        frame_result
    }
}

fn validate_host_call(
    access: &RefCell<ActiveHostScope>,
    expected_token: HostScopeToken,
    expected_surface_id: &SurfaceId,
    kind: HostCallKind,
) -> mlua::Result<()> {
    access
        .borrow()
        .validate(expected_token, expected_surface_id, kind)
        .map_err(LuaError::external)
}

#[derive(Debug)]
struct FrameCommandError {
    api_name: String,
    error: RuntimeError,
}

fn apply_frame_commands(
    host: &mut dyn RuntimeUiHost,
    surface_id: &SurfaceId,
    commands: Vec<FrameCommand>,
) -> Result<(), FrameCommandError> {
    for command in commands {
        let api_name = command.api_name().to_owned();
        let result = catch_unwind(AssertUnwindSafe(|| match command {
            FrameCommand::RequestReload => host.request_reload(),
            FrameCommand::RequestRepaint => host.request_repaint(),
        }))
        .map_err(|payload| FrameCommandError {
            api_name: api_name.clone(),
            error: RuntimeError::backend(format!(
                "surface `{surface_id}` frame command panic in `{}`: {}",
                api_name,
                panic_payload_message(payload),
            )),
        })?;
        result.map_err(|err| FrameCommandError {
            api_name: api_name.clone(),
            error: RuntimeError::backend(format!(
                "surface `{surface_id}` frame command `{}` failed: {}",
                api_name, err
            )),
        })?;
    }
    Ok(())
}

fn build_app_proxy_table(
    lua: &Lua,
    session: Rc<RefCell<HostBridgeSession>>,
) -> mlua::Result<Table> {
    let app = lua.create_table()?;
    let metatable = lua.create_table()?;
    metatable.set(
        "__index",
        lua.create_function(move |lua, (_table, key): (Table, Value)| {
            let Value::String(key) = key else {
                return Ok(Value::Nil);
            };
            let Some(member) = AppProxyMember::from_name(key.to_str()?.as_ref()) else {
                return Ok(Value::Nil);
            };
            let function = session.borrow_mut().app_wrapper(lua, member)?;
            Ok(function.map(Value::Function).unwrap_or(Value::Nil))
        })?,
    )?;
    app.set_metatable(Some(metatable))?;
    Ok(app)
}

fn build_ui_proxy_table(lua: &Lua, session: Rc<RefCell<HostBridgeSession>>) -> mlua::Result<Table> {
    let ui = lua.create_table()?;
    let metatable = lua.create_table()?;
    metatable.set(
        "__index",
        lua.create_function(move |lua, (_table, key): (Table, Value)| {
            let Value::String(key) = key else {
                return Ok(Value::Nil);
            };
            let Some(member) = UiProxyMember::from_name(key.to_str()?.as_ref()) else {
                return Ok(Value::Nil);
            };
            let function = session.borrow_mut().ui_wrapper(lua, member)?;
            Ok(function.map(Value::Function).unwrap_or(Value::Nil))
        })?,
    )?;
    ui.set_metatable(Some(metatable))?;
    Ok(ui)
}

fn create_app_wrapper(
    lua: &Lua,
    scope: HostBridgeScope,
    member: AppProxyMember,
) -> mlua::Result<Function> {
    let access = Rc::clone(&scope.access);
    let token = scope.token;
    let surface_id = scope.surface_id.clone();
    let dispatch_scope = scope.clone();
    let function = member.function();
    let kind = member.kind();

    match member {
        AppProxyMember::Log => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let mut args = args.into_vec().into_iter();
                let level = expect_string_arg(args.next(), function.full_name, 1)?;
                let message = expect_string_arg(args.next(), function.full_name, 2)?;
                ensure_no_extra_args(function.full_name, args)?;
                dispatch_scope.with_app_host(|host| host.log(&level, &message))
            })
        }),
        AppProxyMember::RequestReload => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                ensure_no_extra_args(function.full_name, args.into_vec().into_iter())?;
                if let Some(frame_commands) = dispatch_scope.frame_commands.as_ref() {
                    frame_commands
                        .borrow_mut()
                        .push(FrameCommand::RequestReload);
                    Ok(())
                } else {
                    dispatch_scope.with_app_host(|host| host.request_reload())
                }
            })
        }),
        AppProxyMember::RequestRepaint => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                ensure_no_extra_args(function.full_name, args.into_vec().into_iter())?;
                if let Some(frame_commands) = dispatch_scope.frame_commands.as_ref() {
                    frame_commands
                        .borrow_mut()
                        .push(FrameCommand::RequestRepaint);
                    Ok(())
                } else {
                    dispatch_scope.with_app_host(|host| host.request_repaint())
                }
            })
        }),
    }
}

fn create_ui_wrapper(
    lua: &Lua,
    scope: HostBridgeScope,
    member: UiProxyMember,
) -> mlua::Result<Function> {
    let access = Rc::clone(&scope.access);
    let token = scope.token;
    let surface_id = scope.surface_id.clone();
    let dispatch_scope = scope.clone();
    let function = member.function();
    let kind = member.kind();

    match member {
        UiProxyMember::Label => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (text, props) = parse_label_call(args)?;
                let props = parse_label_options(props)?;
                dispatch_scope.with_ui_host(|host| host.label(&text, props))
            })
        }),
        UiProxyMember::Separator => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                ensure_no_extra_args(function.full_name, args.into_vec().into_iter())?;
                dispatch_scope.with_ui_host(|host| host.separator())
            })
        }),
        UiProxyMember::Button => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, text, props) = parse_button_call(args)?;
                dispatch_scope.with_ui_host(|host| host.button(&id, &text, props))
            })
        }),
        UiProxyMember::TextEdit => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, value, props) = parse_text_edit_call(args)?;
                let output =
                    dispatch_scope.with_ui_host(|host| host.text_edit(&id, &value, props))?;
                Ok((output.value, output.changed))
            })
        }),
        UiProxyMember::Checkbox => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, checked, props) = parse_checkbox_call(args)?;
                let output =
                    dispatch_scope.with_ui_host(|host| host.checkbox(&id, checked, props))?;
                Ok((output.value, output.changed))
            })
        }),
        UiProxyMember::Switch => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, checked, props) = parse_switch_call(args)?;
                let output =
                    dispatch_scope.with_ui_host(|host| host.switch(&id, checked, props))?;
                Ok((output.value, output.changed))
            })
        }),
        UiProxyMember::Slider => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, value, props) = parse_slider_call(args)?;
                let output = dispatch_scope.with_ui_host(|host| host.slider(&id, value, props))?;
                Ok((output.value, output.changed))
            })
        }),
        UiProxyMember::NumberInput => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, value, props) = parse_number_input_call(args)?;
                let output =
                    dispatch_scope.with_ui_host(|host| host.number_input(&id, value, props))?;
                Ok((output.value, output.changed))
            })
        }),
        UiProxyMember::Select => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, selected_index, items, props) = parse_select_call(args)?;
                let output = dispatch_scope
                    .with_ui_host(|host| host.select(&id, selected_index, &items, props))?;
                Ok((output.selected_index, output.changed))
            })
        }),
        UiProxyMember::Tabs => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, selected_index, options, props) = parse_tabs_call(args)?;
                let output = dispatch_scope
                    .with_ui_host(|host| host.tabs(&id, selected_index, &options, props))?;
                Ok((output.selected_index, output.changed))
            })
        }),
        UiProxyMember::Progress => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (value, props) = parse_progress_call(args)?;
                dispatch_scope.with_ui_host(|host| host.progress(value, props))
            })
        }),
        UiProxyMember::Radio => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, selected, props) = parse_radio_call(args)?;
                let output =
                    dispatch_scope.with_ui_host(|host| host.radio(&id, selected, props))?;
                Ok((output.value, output.changed))
            })
        }),
        UiProxyMember::ButtonGroup => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, options, props) = parse_button_group_call(args)?;
                let output =
                    dispatch_scope.with_ui_host(|host| host.button_group(&id, &options, props))?;
                Ok((output.clicked_index, output.changed))
            })
        }),
        UiProxyMember::BeginCollapsible => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, title, open, props) = parse_begin_collapsible_call(args)?;
                let output = dispatch_scope
                    .with_ui_host(|host| host.begin_collapsible(&id, &title, open, props))?;
                if output.visible {
                    dispatch_scope
                        .scoped_ui_state
                        .as_ref()
                        .expect("ui scope state available")
                        .borrow_mut()
                        .begin_container();
                }
                Ok((output.open, output.visible))
            })
        }),
        UiProxyMember::DropdownMenu => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, trigger_label, entries, props) = parse_dropdown_menu_call(args)?;
                let output = dispatch_scope.with_ui_host(|host| {
                    host.dropdown_menu(&id, &trigger_label, &entries, props)
                })?;
                Ok((output.action_id, output.changed))
            })
        }),
        UiProxyMember::Tooltip => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (trigger_label, text, props) = parse_tooltip_call(args)?;
                dispatch_scope.with_ui_host(|host| host.tooltip(&trigger_label, &text, props))
            })
        }),
        UiProxyMember::Spinner => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let props = parse_spinner_call(args)?;
                dispatch_scope.with_ui_host(|host| host.spinner(props))
            })
        }),
        UiProxyMember::Skeleton => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let props = parse_skeleton_call(args)?;
                dispatch_scope.with_ui_host(|host| host.skeleton(props))
            })
        }),
        UiProxyMember::VirtualList => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let (id, items, props) = parse_virtual_list_call(args)?;
                let items = dispatch_scope.resolve_virtual_list_items(
                    &items,
                    function.full_name,
                    VIRTUAL_LIST_ITEMS_ARG_INDEX,
                )?;
                let output = dispatch_scope
                    .with_ui_host(|host| host.virtual_list(&id, items.as_ref(), props))?;
                Ok((output.selected_index, output.changed))
            })
        }),
        UiProxyMember::BeginRow => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let props = parse_container_options(
                    parse_optional_table_arg(args, function.full_name)?,
                    function.full_name,
                )?;
                dispatch_scope.with_ui_host(|host| host.begin_row(props))?;
                dispatch_scope
                    .scoped_ui_state
                    .as_ref()
                    .expect("ui scope state available")
                    .borrow_mut()
                    .begin_container();
                Ok(())
            })
        }),
        UiProxyMember::BeginColumn => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let props = parse_container_options(
                    parse_optional_table_arg(args, function.full_name)?,
                    function.full_name,
                )?;
                dispatch_scope.with_ui_host(|host| host.begin_column(props))?;
                dispatch_scope
                    .scoped_ui_state
                    .as_ref()
                    .expect("ui scope state available")
                    .borrow_mut()
                    .begin_container();
                Ok(())
            })
        }),
        UiProxyMember::BeginCard => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let props = parse_card_options(
                    parse_optional_table_arg(args, function.full_name)?,
                    function.full_name,
                )?;
                dispatch_scope.with_ui_host(|host| host.begin_card(props))?;
                dispatch_scope
                    .scoped_ui_state
                    .as_ref()
                    .expect("ui scope state available")
                    .borrow_mut()
                    .begin_container();
                Ok(())
            })
        }),
        UiProxyMember::EndScope => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                ensure_no_extra_args(function.full_name, args.into_vec().into_iter())?;
                dispatch_scope.with_ui_host(|host| host.end())?;
                dispatch_scope
                    .scoped_ui_state
                    .as_ref()
                    .expect("ui scope state available")
                    .borrow_mut()
                    .end_container()?;
                Ok(())
            })
        }),
        UiProxyMember::PushId => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                let mut args = args.into_vec().into_iter();
                let id = expect_string_arg(args.next(), function.full_name, 1)?;
                ensure_no_extra_args(function.full_name, args)?;
                dispatch_scope.with_ui_host(|host| host.push_id(&id))?;
                dispatch_scope
                    .scoped_ui_state
                    .as_ref()
                    .expect("ui scope state available")
                    .borrow_mut()
                    .push_id();
                Ok(())
            })
        }),
        UiProxyMember::PopId => lua.create_function(move |_, args: MultiValue| {
            run_caught_lua_callback(&surface_id, kind, || {
                validate_host_call(access.as_ref(), token, &surface_id, kind)?;
                ensure_no_extra_args(function.full_name, args.into_vec().into_iter())?;
                dispatch_scope.with_ui_host(|host| host.pop_id())?;
                dispatch_scope
                    .scoped_ui_state
                    .as_ref()
                    .expect("ui scope state available")
                    .borrow_mut()
                    .pop_id()?;
                Ok(())
            })
        }),
    }
}

fn run_caught_lua_callback<R>(
    surface_id: &SurfaceId,
    kind: HostCallKind,
    f: impl FnOnce() -> mlua::Result<R>,
) -> mlua::Result<R> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(payload) => Err(LuaError::external(RuntimeError::backend(format!(
            "surface `{surface_id}` host callback panic in `{}`: {}",
            kind.api_name(),
            panic_payload_message(payload),
        )))),
    }
}

fn panic_payload_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_owned()
    }
}
