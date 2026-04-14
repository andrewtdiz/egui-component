fn tooltip_placement_from_index(index: usize) -> TooltipPlacement {
    match index {
        1 => TooltipPlacement::Right,
        2 => TooltipPlacement::Bottom,
        3 => TooltipPlacement::Left,
        _ => TooltipPlacement::Top,
    }
}

fn toast_placement_from_index(index: usize) -> ToastPlacement {
    match index {
        0 => ToastPlacement::TopLeft,
        1 => ToastPlacement::TopCenter,
        2 => ToastPlacement::TopRight,
        3 => ToastPlacement::CenterLeft,
        4 => ToastPlacement::Center,
        5 => ToastPlacement::CenterRight,
        6 => ToastPlacement::BottomLeft,
        7 => ToastPlacement::BottomCenter,
        _ => ToastPlacement::BottomRight,
    }
}

fn dropdown_action_label(action: Option<usize>) -> &'static str {
    match action.and_then(|id| DROPDOWN_ACTION_LABELS.get(id).copied()) {
        Some(label) => label,
        None => "No action triggered",
    }
}

fn context_menu_action_label(action: Option<usize>) -> &'static str {
    match action.and_then(|id| CONTEXT_MENU_ACTION_LABELS.get(id).copied()) {
        Some(label) => label,
        None => "No context menu action triggered",
    }
}

fn menu_bar_action_label(action: Option<usize>) -> &'static str {
    match action.and_then(|id| MENU_BAR_ACTION_LABELS.get(id).copied()) {
        Some(label) => label,
        None => "No menu action triggered",
    }
}

fn showcase_image(name: &str) -> Image<'static> {
    Image::from_bytes(
        format!("bytes://examples/showcase/{name}.png"),
        SHOWCASE_IMAGE_BYTES,
    )
}

fn showcase_component_definitions_by_section(
    section: ShowcaseSection,
) -> impl Iterator<Item = &'static ComponentDefinition> {
    let mut definitions = component_definitions()
        .filter(move |definition| showcase_section(definition.kind) == section)
        .collect::<Vec<_>>();
    definitions.sort_unstable_by(|left, right| left.label.cmp(right.label));
    definitions.into_iter()
}

pub(crate) fn catalog_component_definition(kind: ComponentKind) -> &'static ComponentDefinition {
    component_definitions()
        .find(|definition| definition.kind == kind)
        .expect("missing catalog component definition")
}

fn showcase_section(kind: ComponentKind) -> ShowcaseSection {
    match kind {
        ComponentKind::CanvaBackgrounds => ShowcaseSection::Canva,
        ComponentKind::CanvaBrandKit => ShowcaseSection::Canva,
        ComponentKind::CanvaEditImage => ShowcaseSection::Canva,
        ComponentKind::CanvaPosition => ShowcaseSection::Canva,
        ComponentKind::CollabCursor
        | ComponentKind::MenuBar
        | ComponentKind::DragBoard
        | ComponentKind::FileTree
        | ComponentKind::Hierarchy
        | ComponentKind::Sidebar
        | ComponentKind::Toast => ShowcaseSection::Examples,
        _ => match catalog_component_definition(kind).group {
            ComponentGroup::Primitive => ShowcaseSection::PrimaryPrimitive,
            ComponentGroup::Composed => ShowcaseSection::DerivedComposed,
        },
    }
}

pub(crate) fn preview_surface_width(kind: ComponentKind, available_width: f32) -> f32 {
    match kind {
        ComponentKind::CanvaBackgrounds => available_width.min(460.0),
        ComponentKind::CanvaBrandKit => available_width.min(640.0),
        ComponentKind::CanvaEditImage => available_width.min(420.0),
        ComponentKind::CanvaPosition => available_width.min(440.0),
        ComponentKind::CollabCursor => available_width.min(560.0),
        ComponentKind::ContextMenu => available_width.min(520.0),
        ComponentKind::DragBoard => available_width.min(560.0),
        ComponentKind::FileTree => available_width.min(360.0),
        ComponentKind::Hierarchy => available_width.min(440.0),
        ComponentKind::MenuBar => available_width.min(560.0),
        ComponentKind::Sidebar => available_width.min(820.0),
        ComponentKind::Toast => available_width.min(760.0),
        _ => available_width.clamp(280.0, 480.0),
    }
}

pub(crate) fn showcase_description(kind: ComponentKind) -> &'static str {
    match kind {
        ComponentKind::Label => "Text styles and tones.",
        ComponentKind::Color => "Circular solid color swatches.",
        ComponentKind::Image => "PNG-backed raster image rendering.",
        ComponentKind::Icon => "Lucide icon rendering.",
        ComponentKind::Input => "Single-line text input.",
        ComponentKind::Button => "Text, icon, and link button variants.",
        ComponentKind::CanvaBackgrounds => {
            "Canva-style background browser with search, swatches, and a tiled result grid."
        }
        ComponentKind::CanvaBrandKit => {
            "Canva-style brand kit browser with an internal category rail and placeholder asset views."
        }
        ComponentKind::CanvaEditImage => {
            "Canva-style image editing side panel with selection tools and effect rails."
        }
        ComponentKind::CanvaPosition => {
            "Canva-style position inspector with arrange, align, and transform controls."
        }
        ComponentKind::Checkbox => "Boolean control with label.",
        ComponentKind::CollabCursor => {
            "Presence cursor with a saturated SVG pointer and attached collaborator name badge."
        }
        ComponentKind::Switch => "Toggle control.",
        ComponentKind::Slider => "Range input.",
        ComponentKind::NumberInput => "Numeric entry with drag axis support.",
        ComponentKind::Select => "Single-choice selection menu.",
        ComponentKind::Tabs => "Inline, segmented, stacked, and rail tabs.",
        ComponentKind::Card => "Framed content surface.",
        ComponentKind::Radio => "Single-choice control for mutually exclusive selections.",
        ComponentKind::RadioGroup => "Vertical radio list with optional descriptions.",
        ComponentKind::Popover => "Click-triggered interactive popup surface.",
        ComponentKind::Tooltip => "Hover-triggered helper content.",
        ComponentKind::DropdownMenu => "Actions, shortcuts, separators, and nested menus.",
        ComponentKind::ContextMenu => {
            "Right-click menu surface built on egui's built-in context popup behavior."
        }
        ComponentKind::AudioPlayback => "Playback row with optional trailing actions.",
        ComponentKind::Combobox => "Filterable multi-select picker with checkbox menu rows.",
        ComponentKind::Command => "Searchable command list with preview mode.",
        ComponentKind::Dialogue => "Modal confirmation flow.",
        ComponentKind::DragBoard => {
            "Single-card drag and drop between two board regions using egui's built-in DnD."
        }
        ComponentKind::FileTree => {
            "Compact file explorer tree with condensed rows, bootstrap caret disclosure icons, and edge-to-edge selection fills."
        }
        ComponentKind::Hierarchy => {
            "Game-style hierarchy tree with selection, subtree highlighting, and cross-parent drag reparenting."
        }
        ComponentKind::MenuBar => "Desktop-style menu bar surface.",
        ComponentKind::Sidebar => "Overlay sidebar previewed inside a host surface.",
        ComponentKind::Toast => "Stacked toast notifications with configurable placement.",
    }
}

fn app_background(ui: &Ui) -> Color32 {
    theme::color(ui, ColorRole::Background)
}

pub(crate) fn showcase_header_fill(runtime: theme::ThemeRuntime) -> Color32 {
    theme::resolved_color(runtime, ColorRole::Background).lerp_to_gamma(
        theme::resolved_color(runtime, ColorRole::Card),
        if runtime.mode.is_dark() { 0.84 } else { 0.92 },
    )
}

fn input_background(ui: &Ui) -> Color32 {
    theme::color(ui, ColorRole::Background).lerp_to_gamma(
        theme::color(ui, ColorRole::Card),
        if ui.visuals().dark_mode { 0.82 } else { 0.72 },
    )
}

fn text_secondary(ui: &Ui) -> Color32 {
    theme::color(ui, ColorRole::Foreground)
        .lerp_to_gamma(theme::color(ui, ColorRole::MutedForeground), 0.55)
}

fn text_muted(ui: &Ui) -> Color32 {
    theme::color(ui, ColorRole::MutedForeground)
}

fn radius_sm(ui: &Ui) -> u8 {
    theme::radius(ui, RadiusRole::Sm)
}

fn radius_md(ui: &Ui) -> u8 {
    theme::radius(ui, RadiusRole::Md)
}

fn tooltip_placement_index(placement: TooltipPlacement) -> usize {
    match placement {
        TooltipPlacement::Top | TooltipPlacement::Auto => 0,
        TooltipPlacement::Right => 1,
        TooltipPlacement::Bottom => 2,
        TooltipPlacement::Left => 3,
    }
}

fn sidebar_preview_toggle_icon(side_index: usize, open: bool) -> &'static str {
    match (side_index, open) {
        (0, true) => "panel-left-close",
        (0, false) => "panel-left-open",
        (_, true) => "panel-right-close",
        (_, false) => "panel-right-open",
    }
}

fn clamp_state(state: &mut ShowcaseApp) {
    state.canva_background_color_index = state
        .canva_background_color_index
        .min(CANVA_BACKGROUND_SWATCHES.len().saturating_sub(1));
    state.canva_brand_category_index = state
        .canva_brand_category_index
        .min(CANVA_BRAND_CATEGORIES.len().saturating_sub(1));
    state.canva_edit_tool_index = state
        .canva_edit_tool_index
        .min(CANVA_EDIT_SELECTION_OPTIONS.len().saturating_sub(1));
    state.canva_edit_filter_index = state
        .canva_edit_filter_index
        .min(CANVA_FILTER_ITEMS.len().saturating_sub(1));
    state.toolbar_color_index = state
        .toolbar_color_index
        .min(TOOLBAR_SWATCHES.len().saturating_sub(1));
    state.collab_cursor_preview_position.x = state.collab_cursor_preview_position.x.clamp(0.0, 1.0);
    state.collab_cursor_preview_position.y = state.collab_cursor_preview_position.y.clamp(0.0, 1.0);
    state.canva_position_tab_index = state
        .canva_position_tab_index
        .min(CANVA_POSITION_TAB_OPTIONS.len().saturating_sub(1));
    state.canva_layer_filter_index = state
        .canva_layer_filter_index
        .min(CANVA_LAYER_FILTER_OPTIONS.len().saturating_sub(1));
    state.hierarchy_style_index = state
        .hierarchy_style_index
        .min(HIERARCHY_STYLE_OPTIONS.len().saturating_sub(1));
    state.hierarchy_icon_style_index = state
        .hierarchy_icon_style_index
        .min(HIERARCHY_ICON_STYLE_OPTIONS.len().saturating_sub(1));
    state.image_rotation_degrees = state.image_rotation_degrees.clamp(-180.0, 180.0);
    state.tab_index = state.tab_index.min(TAB_OPTIONS.len().saturating_sub(1));
    state.blender_tab_index = state
        .blender_tab_index
        .min(BLENDER_TAB_OPTIONS.len().saturating_sub(1));
    state.segmented_tab_index = state
        .segmented_tab_index
        .min(TAB_OPTIONS.len().saturating_sub(1));
    state.stacked_tab_index = state
        .stacked_tab_index
        .min(STACKED_TAB_OPTIONS.len().saturating_sub(1));
    state.rail_tab_index = state
        .rail_tab_index
        .min(RAIL_TAB_OPTIONS.len().saturating_sub(1));
    state.slider_value = state.slider_value.clamp(0.0, 100.0);
    state.number_x_value = state.number_x_value.clamp(0.0, 100.0);
    state.number_y_value = state.number_y_value.clamp(0.0, 100.0);
    state.canva_width_value = state.canva_width_value.clamp(1.0, 4_000.0);
    state.canva_height_value = state.canva_height_value.clamp(1.0, 4_000.0);
    state.canva_x_value = state.canva_x_value.clamp(-4_000.0, 4_000.0);
    state.canva_y_value = state.canva_y_value.clamp(-4_000.0, 4_000.0);
    state.canva_rotate_value = state.canva_rotate_value.clamp(-360.0, 360.0);
    state.sidebar_side_index = state
        .sidebar_side_index
        .min(SIDEBAR_SIDE_OPTIONS.len().saturating_sub(1));

    if state.radio_group_value.is_some_and(|value| {
        !RADIO_GROUP_OPTIONS
            .iter()
            .any(|option| option.value == value)
    }) {
        state.radio_group_value = None;
    }

    if state
        .select_index
        .is_some_and(|index| index >= SELECT_OPTIONS.len())
    {
        state.select_index = None;
    }
    if state
        .canva_brand_select_index
        .is_some_and(|index| index >= CANVA_BRAND_SELECT_OPTIONS.len())
    {
        state.canva_brand_select_index = Some(CANVA_BRAND_SELECT_OPTIONS.len().saturating_sub(1));
    }
    if state
        .toast_placement_index
        .is_some_and(|index| index >= TOAST_PLACEMENT_OPTIONS.len())
    {
        state.toast_placement_index = Some(TOAST_PLACEMENT_OPTIONS.len().saturating_sub(1));
    }
    if state
        .dropdown_action
        .is_some_and(|id| id >= DROPDOWN_ACTION_LABELS.len())
    {
        state.dropdown_action = None;
    }
    if state
        .context_menu_action
        .is_some_and(|id| id >= CONTEXT_MENU_ACTION_LABELS.len())
    {
        state.context_menu_action = None;
    }
    if state
        .menu_bar_action
        .is_some_and(|id| id >= MENU_BAR_ACTION_LABELS.len())
    {
        state.menu_bar_action = None;
    }
    state
        .combobox_indices
        .retain(|index| *index < COMBOBOX_OPTIONS.len());
    state.combobox_indices.sort_unstable();
    state.combobox_indices.dedup();
}
