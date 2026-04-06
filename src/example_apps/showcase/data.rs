pub const WINDOW_TITLE: &str = "egui-component Showcase";
pub const WINDOW_INNER_SIZE: [f32; 2] = [1280.0, 900.0];

const SHOWCASE_IMAGE_BYTES: &[u8] = include_bytes!("../../../assets/images/showcase-image.png");

const BUTTON_GROUP_OPTIONS: [&str; 3] = ["Move", "Rotate", "Scale"];
const TOOLBAR_ACTION_OPTIONS: [&str; 3] = ["Edit", "BG Remover", "Eraser"];
const ICON_TOOLBAR_ITEMS: [IconToolbarItem<'static>; 8] = [
    IconToolbarItem::new("mouse-pointer-2").tooltip("Select"),
    IconToolbarItem::new("move").tooltip("Move"),
    IconToolbarItem::new("rotate-ccw").tooltip("Rotate"),
    IconToolbarItem::new("arrow-up-right").tooltip("Expand"),
    IconToolbarItem::new("crosshair").tooltip("Center"),
    IconToolbarItem::new("package-2")
        .tooltip("Group")
        .badge_fill(Color32::from_rgb(40, 150, 255)),
    IconToolbarItem::new("copy").tooltip("Duplicate"),
    IconToolbarItem::new("trash").tooltip("Delete"),
];
const TOOLBAR_SWATCHES: [Color32; 4] = [
    Color32::from_rgb(35, 45, 75),
    Color32::from_rgb(103, 132, 162),
    Color32::from_rgb(122, 24, 42),
    Color32::from_rgb(206, 164, 84),
];
const TOOLTIP_PLACEMENT_OPTIONS: [&str; 4] = ["Top", "Right", "Bottom", "Left"];
const TWEMOJI_SEQUENCE_SAMPLES: [(&str, &str); 5] = [
    ("🙂", "Simple"),
    ("👩‍💻", "ZWJ"),
    ("🧑🏽‍🚀", "Skin tone"),
    ("❤️", "Variant"),
    ("🇺🇸", "Flag"),
];
const SELECT_OPTIONS: [&str; 4] = ["Draft", "Review", "Approved", "Archived"];
const OPEN_WITH_ENTRIES: [DropdownMenuEntry<'static>; 1] =
    [DropdownMenuEntry::action_with_icon(0, "Codex", "codex")];
const COMBOBOX_OPTIONS: [&str; 6] = [
    "Material 1",
    "Material Glass",
    "Material Metal",
    "Sprite Atlas",
    "Sprite Mask",
    "UI Text Style",
];
const TAB_OPTIONS: [TabOption<'static>; 3] = [
    TabOption::new(0, "Design"),
    TabOption::new(1, "Code"),
    TabOption::new(2, "History"),
];
const BLENDER_TAB_OPTIONS: [TabOption<'static>; 5] = [
    TabOption::new(0, "Layout"),
    TabOption::new(1, "Modeling"),
    TabOption::new(2, "Sculpting"),
    TabOption::new(3, "UV Editing"),
    TabOption::new(4, "Texture Paint"),
];
const STACKED_TAB_OPTIONS: [TabOption<'static>; 2] = [
    TabOption::with_icon(0, "Templates", "layout-template"),
    TabOption::with_icon(1, "Layouts", "layout-grid"),
];
const CANVA_BACKGROUND_SWATCHES: [Color32; 5] = [
    Color32::from_rgb(154, 181, 208),
    Color32::from_rgb(247, 246, 243),
    Color32::from_rgb(8, 8, 8),
    Color32::from_rgb(245, 103, 97),
    Color32::from_rgb(214, 90, 181),
];
const CANVA_BRAND_CATEGORIES: [&str; 10] = [
    "All assets",
    "Guidelines",
    "Logos",
    "Colors",
    "Fonts",
    "Brand voice",
    "Photos",
    "Graphics",
    "Icons",
    "Charts",
];
const CANVA_BRAND_SELECT_OPTIONS: [&str; 1] = ["Brand Kit"];
const CANVA_BRAND_ACCENT: Color32 = Color32::from_rgb(241, 168, 78);
const CANVA_EDIT_ACCENT: Color32 = Color32::from_rgb(241, 168, 78);
const CANVA_EDIT_SELECTION_OPTIONS: [CanvaEditChip<'static>; 4] = [
    CanvaEditChip::new("All", "image"),
    CanvaEditChip::new("Click", "mouse-pointer-click"),
    CanvaEditChip::new("Brush", "paintbrush"),
    CanvaEditChip::new("Magic", "wand-sparkles"),
];
const CANVA_MAGIC_STUDIO_ITEMS: [CanvaEditRailItem<'static>; 5] = [
    CanvaEditRailItem::new(
        "Magic Layers",
        Color32::from_rgb(159, 77, 255),
        Color32::from_rgb(123, 92, 255),
    )
    .badge("New"),
    CanvaEditRailItem::new(
        "BG Remover",
        Color32::from_rgb(119, 194, 250),
        Color32::from_rgb(255, 201, 72),
    ),
    CanvaEditRailItem::new(
        "BG Generator",
        Color32::from_rgb(143, 176, 59),
        Color32::from_rgb(234, 190, 197),
    ),
    CanvaEditRailItem::new(
        "Magic Expand",
        Color32::from_rgb(188, 143, 173),
        Color32::from_rgb(73, 58, 86),
    ),
    CanvaEditRailItem::new(
        "Magic Grab",
        Color32::from_rgb(56, 125, 176),
        Color32::from_rgb(14, 31, 54),
    ),
];
const CANVA_FILTER_ITEMS: [CanvaEditRailItem<'static>; 5] = [
    CanvaEditRailItem::new(
        "None",
        Color32::from_rgb(52, 63, 94),
        Color32::from_rgb(118, 167, 238),
    ),
    CanvaEditRailItem::new(
        "Fresco",
        Color32::from_rgb(67, 77, 110),
        Color32::from_rgb(191, 46, 46),
    ),
    CanvaEditRailItem::new(
        "Belvedere",
        Color32::from_rgb(88, 68, 121),
        Color32::from_rgb(214, 74, 96),
    ),
    CanvaEditRailItem::new(
        "Prism",
        Color32::from_rgb(90, 87, 96),
        Color32::from_rgb(226, 173, 66),
    ),
    CanvaEditRailItem::new(
        "Noir",
        Color32::from_rgb(47, 49, 57),
        Color32::from_rgb(170, 177, 191),
    ),
];
const CANVA_EFFECT_ITEMS: [CanvaEditRailItem<'static>; 5] = [
    CanvaEditRailItem::new(
        "Shadows",
        Color32::from_rgb(100, 132, 255),
        Color32::from_rgb(239, 244, 255),
    ),
    CanvaEditRailItem::new(
        "Duotone",
        Color32::from_rgb(238, 206, 55),
        Color32::from_rgb(72, 57, 170),
    ),
    CanvaEditRailItem::new(
        "Blur",
        Color32::from_rgb(98, 205, 214),
        Color32::from_rgb(255, 196, 61),
    ),
    CanvaEditRailItem::new(
        "Auto focus",
        Color32::from_rgb(118, 86, 61),
        Color32::from_rgb(226, 208, 186),
    ),
    CanvaEditRailItem::new(
        "Liquify",
        Color32::from_rgb(82, 98, 132),
        Color32::from_rgb(213, 236, 250),
    ),
];
const CANVA_POSITION_TAB_OPTIONS: [TabOption<'static>; 2] =
    [TabOption::new(0, "Arrange"), TabOption::new(1, "Layers")];
const CANVA_LAYER_FILTER_OPTIONS: [TabOption<'static>; 2] =
    [TabOption::new(0, "All"), TabOption::new(1, "Overlapping")];
const HIERARCHY_STYLE_OPTIONS: [TabOption<'static>; 2] =
    [TabOption::new(0, "Normal"), TabOption::new(1, "Component")];
const HIERARCHY_ICON_STYLE_OPTIONS: [TabOption<'static>; 2] =
    [TabOption::new(0, "Emoji"), TabOption::new(1, "Icons")];
const SHOWCASE_BASE_COLOR: BaseColor = BaseColor::Slate;
const SIDEBAR_SIDE_OPTIONS: [&str; 2] = ["Left", "Right"];
const RAIL_TAB_OPTIONS: [TabOption<'static>; 3] = [
    TabOption::with_icon(0, "Home", "house"),
    TabOption::with_icon(1, "Assets", "image"),
    TabOption::with_icon(2, "Export", "rocket"),
];
const TOAST_PLACEMENT_OPTIONS: [&str; 9] = [
    "Top Left",
    "Top Center",
    "Top Right",
    "Center Left",
    "Center",
    "Center Right",
    "Bottom Left",
    "Bottom Center",
    "Bottom Right",
];
const DRAG_BOARD_ITEMS: [DragBoardItem<'static>; 3] = [
    DragBoardItem::new("Polish header spacing").description("Shared toolbar chrome"),
    DragBoardItem::new("Tune sidebar spacing").description("Examples rail"),
    DragBoardItem::new("Ship drag board").description("Trello-style preview"),
];
const DRAG_BOARD_DEFAULT_REGIONS: [DragBoardRegion; 3] = [
    DragBoardRegion::Left,
    DragBoardRegion::Left,
    DragBoardRegion::Right,
];
const FILE_TREE_DEFAULT_SELECTED_ID: usize = 5;
const HIERARCHY_DEFAULT_SELECTED_ID: usize = 1;
const CANVA_LAYER_DEFAULT_ORDER: [usize; 3] = [0, 1, 2];
const RADIO_GROUP_OPTIONS: [RadioOption<'static>; 3] = [
    RadioOption::new(0, "Starter").description("Basic surfaces and controls for smaller tools."),
    RadioOption::new(1, "Team").description("Shared tokens, overrides, and example screens."),
    RadioOption::new(2, "Enterprise")
        .description("Extended theming, audit trails, and environment presets."),
];
const COMMAND_ITEMS: [CommandItem<'static>; 11] = [
    CommandItem::new("", "scene: open scene search").shortcut("Ctrl+P"),
    CommandItem::new("", "scene: save active scene").shortcut("Ctrl+S"),
    CommandItem::new("", "gameobject: create empty").shortcut("Ctrl+Shift+N"),
    CommandItem::new("", "gameobject: add camera"),
    CommandItem::new("", "assets: reimport selected").shortcut("Ctrl+R"),
    CommandItem::new("", "view: toggle gizmos"),
    CommandItem::new("", "view: focus selection").shortcut("F"),
    CommandItem::new("", "window: animation"),
    CommandItem::new("", "window: inspector").shortcut("Ctrl+I"),
    CommandItem::new("", "tools: bake lighting"),
    CommandItem::new("", "tools: build nav mesh").shortcut("Ctrl+B"),
];
const DROPDOWN_INVITE_ENTRIES: [DropdownMenuEntry<'static>; 4] = [
    DropdownMenuEntry::action(4, "Email"),
    DropdownMenuEntry::action(5, "Message"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action(6, "More..."),
];
const DROPDOWN_ENTRIES: [DropdownMenuEntry<'static>; 14] = [
    DropdownMenuEntry::action(0, "My Account"),
    DropdownMenuEntry::action_with_shortcut(1, "Profile", "Shift+Cmd+P"),
    DropdownMenuEntry::action_with_shortcut(2, "Billing", "Cmd+B"),
    DropdownMenuEntry::action_with_shortcut(3, "Settings", "Cmd+S"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::submenu("Invite users", &DROPDOWN_INVITE_ENTRIES),
    DropdownMenuEntry::action_with_shortcut(7, "New Team", "Cmd+T"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action(8, "GitHub"),
    DropdownMenuEntry::action(9, "Support"),
    DropdownMenuEntry::action(10, "API"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action_with_shortcut(11, "Log out", "Shift+Cmd+Q"),
    DropdownMenuEntry::action(12, "Delete"),
];
const DROPDOWN_ACTION_LABELS: [&str; 13] = [
    "My Account",
    "Profile",
    "Billing",
    "Settings",
    "Email",
    "Message",
    "More...",
    "New Team",
    "GitHub",
    "Support",
    "API",
    "Log out",
    "Delete",
];
const CONTEXT_MENU_ENTRIES: [DropdownMenuEntry<'static>; 6] = [
    DropdownMenuEntry::action_with_icon(0, "Rename", "pen-line"),
    DropdownMenuEntry::action_with_icon(1, "Duplicate", "copy"),
    DropdownMenuEntry::action_with_icon(2, "Create Prefab", "package-plus"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action_with_icon(3, "Focus Selection", "focus"),
    DropdownMenuEntry::action_with_icon(4, "Delete", "trash-2"),
];
const CONTEXT_MENU_ACTION_LABELS: [&str; 5] = [
    "Rename",
    "Duplicate",
    "Create Prefab",
    "Focus Selection",
    "Delete",
];
const MENU_BAR_RECENT_ENTRIES: [DropdownMenuEntry<'static>; 3] = [
    DropdownMenuEntry::action(3, "Design Tokens.fig"),
    DropdownMenuEntry::action(4, "Toolbar Draft.rs"),
    DropdownMenuEntry::action(5, "App Shell.md"),
];
const CANVA_POSITION_ACTIONS: [(&str, &str, bool); 4] = [
    ("Forward", "arrow-up", false),
    ("Backward", "arrow-down", true),
    ("To front", "bring-to-front", false),
    ("To back", "send-to-back", true),
];
const CANVA_POSITION_ALIGN_ACTIONS: [(&str, &str, bool); 6] = [
    ("Top", "align-start-horizontal", true),
    ("Left", "align-start-vertical", true),
    ("Middle", "align-center-horizontal", false),
    ("Center", "align-center-vertical", false),
    ("Bottom", "align-end-horizontal", true),
    ("Right", "align-end-vertical", true),
];
const MENU_BAR_FILE_ENTRIES: [DropdownMenuEntry<'static>; 7] = [
    DropdownMenuEntry::action_with_icon(0, "New File", "file-plus"),
    DropdownMenuEntry::action_with_icon(1, "Open...", "folder-open"),
    DropdownMenuEntry::submenu_with_icon("Open Recent", "history", &MENU_BAR_RECENT_ENTRIES),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action_with_icon(6, "Save", "save"),
    DropdownMenuEntry::action_with_icon(7, "Save As...", "file-pen"),
    DropdownMenuEntry::action_with_icon(8, "Export", "download"),
];
const MENU_BAR_EDIT_ENTRIES: [DropdownMenuEntry<'static>; 5] = [
    DropdownMenuEntry::action_with_icon(9, "Undo", "undo-2"),
    DropdownMenuEntry::action_with_icon(10, "Redo", "redo-2"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action_with_icon(11, "Cut", "scissors"),
    DropdownMenuEntry::action_with_icon(12, "Paste", "clipboard"),
];
const MENU_BAR_VIEW_ENTRIES: [DropdownMenuEntry<'static>; 4] = [
    DropdownMenuEntry::action_with_icon(13, "Zoom In", "zoom-in"),
    DropdownMenuEntry::action_with_icon(14, "Zoom Out", "zoom-out"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action_with_icon(15, "Toggle Guides", "layout-grid"),
];
const MENU_BAR_OBJECT_ENTRIES: [DropdownMenuEntry<'static>; 4] = [
    DropdownMenuEntry::action_with_icon(16, "Group", "group"),
    DropdownMenuEntry::action_with_icon(17, "Ungroup", "ungroup"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action_with_icon(18, "Bring to Front", "bring-to-front"),
];
const MENU_BAR_ACTION_LABELS: [&str; 19] = [
    "New File",
    "Open...",
    "Open Recent",
    "Design Tokens.fig",
    "Toolbar Draft.rs",
    "App Shell.md",
    "Save",
    "Save As...",
    "Export",
    "Undo",
    "Redo",
    "Cut",
    "Paste",
    "Zoom In",
    "Zoom Out",
    "Toggle Guides",
    "Group",
    "Ungroup",
    "Bring to Front",
];
const PAGINATION_PAGE_COUNT: usize = 12;
const SMALL_TEXT: f32 = 12.0;
const SIDEBAR_WIDTH: f32 = 238.0;
const TOOLBAR_PREVIEW_WIDTH: f32 = 820.0;
const MENU_BAR_PREVIEW_WIDTH: f32 = 420.0;
const TOOLBAR_CANVAS_LIGHT_FILL: Color32 = Color32::from_rgb(228, 228, 231);
const COLLAB_CURSOR_DEFAULT_COLOR: Color32 = Color32::from_rgb(255, 122, 36);
const COLLAB_CURSOR_DEFAULT_PREVIEW_POSITION: egui::Vec2 = egui::vec2(0.5, 0.5);
const IMAGE_TILE_META_ACCENT: Color32 = Color32::from_rgb(59, 130, 246);
const NUMBER_INPUT_GREEN: Color32 = Color32::from_rgb(34, 197, 94);
const NUMBER_INPUT_RED: Color32 = Color32::from_rgb(239, 68, 68);

fn default_hierarchy_nodes() -> Vec<HierarchyNode> {
    vec![
        HierarchyNode::new(1, "Gameplay_Systems", HierarchyItemKind::Folder)
            .locked(true)
            .children(vec![
                HierarchyNode::new(2, "Character_Rig_A", HierarchyItemKind::Group).children(vec![
                    HierarchyNode::new(3, "Player_Controller", HierarchyItemKind::Player).children(
                        vec![
                            HierarchyNode::new(4, "Iron_Sword_01", HierarchyItemKind::Weapon),
                            HierarchyNode::new(5, "Red_Shirt_1", HierarchyItemKind::Clothing),
                            HierarchyNode::new(6, "Hitbox_Main", HierarchyItemKind::Hitbox),
                        ],
                    ),
                ]),
            ]),
    ]
}

fn default_file_tree_nodes() -> Vec<FileTreeNode<'static>> {
    vec![
        FileTreeNode::new(1, "SpaceShooter", FileTreeItemKind::Folder).children(vec![
            FileTreeNode::new(2, "builtins", FileTreeItemKind::Folder)
                .expanded(false)
                .children(vec![FileTreeNode::new(
                    9,
                    "render.pipeline",
                    FileTreeItemKind::File,
                )]),
            FileTreeNode::new(3, "assets", FileTreeItemKind::Folder)
                .expanded(false)
                .children(vec![FileTreeNode::new(
                    10,
                    "ship.sprite",
                    FileTreeItemKind::File,
                )]),
            FileTreeNode::new(4, "input", FileTreeItemKind::Folder)
                .expanded(false)
                .children(vec![FileTreeNode::new(
                    11,
                    "bindings.inputmap",
                    FileTreeItemKind::File,
                )]),
            FileTreeNode::new(5, "main.collection", FileTreeItemKind::Collection),
            FileTreeNode::new(6, "player.script", FileTreeItemKind::Script),
            FileTreeNode::new(7, "game.project", FileTreeItemKind::Project),
            FileTreeNode::new(8, "README.md", FileTreeItemKind::Markdown),
        ]),
    ]
}

#[derive(Clone, Copy)]
struct CanvaEditChip<'a> {
    label: &'a str,
    icon: &'a str,
}

impl<'a> CanvaEditChip<'a> {
    const fn new(label: &'a str, icon: &'a str) -> Self {
        Self { label, icon }
    }
}

#[derive(Clone, Copy)]
struct CanvaEditRailItem<'a> {
    label: &'a str,
    fill: Color32,
    accent: Color32,
    badge: Option<&'a str>,
}

#[derive(Clone, Copy)]
enum CanvaLayerVisual {
    Sprite,
    Frame,
    Text,
}

#[derive(Clone, Copy)]
struct CanvaLayerItem {
    id: usize,
    visual: CanvaLayerVisual,
}

const CANVA_LAYER_ITEMS: [CanvaLayerItem; 3] = [
    CanvaLayerItem {
        id: 0,
        visual: CanvaLayerVisual::Sprite,
    },
    CanvaLayerItem {
        id: 1,
        visual: CanvaLayerVisual::Frame,
    },
    CanvaLayerItem {
        id: 2,
        visual: CanvaLayerVisual::Text,
    },
];

impl<'a> CanvaEditRailItem<'a> {
    const fn new(label: &'a str, fill: Color32, accent: Color32) -> Self {
        Self {
            label,
            fill,
            accent,
            badge: None,
        }
    }

    const fn badge(self, badge: &'a str) -> Self {
        Self {
            badge: Some(badge),
            ..self
        }
    }
}

pub fn install_context(context: &egui::Context) {
    theme::install(
        context,
        ThemeSpec::preset(SHOWCASE_BASE_COLOR),
        ThemeMode::System,
    );
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ShowcaseSection {
    PrimaryPrimitive,
    DerivedComposed,
    Examples,
    Canva,
}

impl ShowcaseSection {
    fn title(self) -> &'static str {
        match self {
            Self::PrimaryPrimitive => "Primary / Primitive",
            Self::DerivedComposed => "Derived / Composed",
            Self::Examples => "Examples",
            Self::Canva => "Canva",
        }
    }
}

pub struct ShowcaseApp {
    selected_component: ComponentKind,
    theme_mode: ThemeMode,
    canva_background_query: String,
    canva_background_color_index: usize,
    canva_brand_query: String,
    canva_brand_select_index: Option<usize>,
    canva_brand_category_index: usize,
    canva_edit_tool_index: usize,
    canva_edit_filter_index: usize,
    drag_board_regions: [DragBoardRegion; 3],
    file_tree_nodes: Vec<FileTreeNode<'static>>,
    file_tree_selected_id: Option<usize>,
    hierarchy_nodes: Vec<HierarchyNode>,
    hierarchy_selected_id: Option<usize>,
    hierarchy_style_index: usize,
    hierarchy_icon_style_index: usize,
    input_value: String,
    search_input_value: String,
    field_value: String,
    emoji_selector_value: String,
    checkbox_value: bool,
    collab_cursor_name: String,
    collab_cursor_color: Color32,
    collab_cursor_preview_position: egui::Vec2,
    switch_value: bool,
    small_switch_value: bool,
    icon_toolbar_selected_index: usize,
    toolbar_color_index: usize,
    canva_position_tab_index: usize,
    canva_layer_filter_index: usize,
    canva_selected_layer_id: usize,
    canva_layer_order: [usize; 3],
    image_rotation_degrees: f32,
    slider_value: f32,
    number_x_value: f32,
    number_y_value: f32,
    canva_width_value: f32,
    canva_height_value: f32,
    canva_ratio_locked: bool,
    canva_x_value: f32,
    canva_y_value: f32,
    canva_rotate_value: f32,
    progress_value: f32,
    radio_value: bool,
    radio_group_value: Option<usize>,
    select_index: Option<usize>,
    skeleton_loading: bool,
    sidebar_preview_open: bool,
    sidebar_side_index: usize,
    tab_index: usize,
    blender_tab_index: usize,
    segmented_tab_index: usize,
    stacked_tab_index: usize,
    rail_tab_index: usize,
    audio_playback_state: AudioPlaybackState,
    pagination_page: usize,
    collapsible_open: bool,
    dropdown_action: Option<usize>,
    open_with_action: Option<usize>,
    context_menu_action: Option<usize>,
    combobox_query: String,
    combobox_indices: Vec<usize>,
    command_query: String,
    dialogue_open: bool,
    popover_open: bool,
    popover_compact_mode: bool,
    toast_stack: ToastStack,
    toast_placement_index: Option<usize>,
    menu_bar_action: Option<usize>,
    tooltip_placement: TooltipPlacement,
    image_tile_selected: bool,
    image_tile_playback_state: ImageTilePlaybackState,
    image_tile_last_action: String,
    spinner_demo_until: Option<f64>,
}

impl Default for ShowcaseApp {
    fn default() -> Self {
        Self {
            selected_component: ComponentKind::Button,
            theme_mode: ThemeMode::System,
            canva_background_query: String::new(),
            canva_background_color_index: 2,
            canva_brand_query: String::new(),
            canva_brand_select_index: Some(0),
            canva_brand_category_index: 0,
            canva_edit_tool_index: 0,
            canva_edit_filter_index: 0,
            drag_board_regions: DRAG_BOARD_DEFAULT_REGIONS,
            file_tree_nodes: default_file_tree_nodes(),
            file_tree_selected_id: Some(FILE_TREE_DEFAULT_SELECTED_ID),
            hierarchy_nodes: default_hierarchy_nodes(),
            hierarchy_selected_id: Some(HIERARCHY_DEFAULT_SELECTED_ID),
            hierarchy_style_index: 0,
            hierarchy_icon_style_index: 0,
            input_value: "Player_Robot".to_owned(),
            search_input_value: "Robot".to_owned(),
            field_value: "M_Robot_Body".to_owned(),
            emoji_selector_value: "🙂".to_owned(),
            checkbox_value: true,
            collab_cursor_name: "Lisa Chen".to_owned(),
            collab_cursor_color: COLLAB_CURSOR_DEFAULT_COLOR,
            collab_cursor_preview_position: COLLAB_CURSOR_DEFAULT_PREVIEW_POSITION,
            switch_value: true,
            small_switch_value: false,
            icon_toolbar_selected_index: 0,
            toolbar_color_index: 0,
            canva_position_tab_index: 0,
            canva_layer_filter_index: 0,
            canva_selected_layer_id: 1,
            canva_layer_order: CANVA_LAYER_DEFAULT_ORDER,
            image_rotation_degrees: 18.0,
            slider_value: 62.0,
            number_x_value: 42.0,
            number_y_value: 16.0,
            canva_width_value: 326.0,
            canva_height_value: 326.0,
            canva_ratio_locked: true,
            canva_x_value: 87.0,
            canva_y_value: 87.0,
            canva_rotate_value: 0.0,
            progress_value: 0.0,
            radio_value: true,
            radio_group_value: Some(1),
            select_index: Some(1),
            skeleton_loading: true,
            sidebar_preview_open: false,
            sidebar_side_index: 0,
            tab_index: 0,
            blender_tab_index: 2,
            segmented_tab_index: 0,
            stacked_tab_index: 0,
            rail_tab_index: 0,
            audio_playback_state: AudioPlaybackState::Paused,
            pagination_page: 2,
            collapsible_open: true,
            dropdown_action: None,
            open_with_action: Some(0),
            context_menu_action: None,
            combobox_query: "mat".to_owned(),
            combobox_indices: vec![0, 2],
            command_query: String::new(),
            dialogue_open: false,
            popover_open: false,
            popover_compact_mode: true,
            toast_stack: ToastStack::default(),
            toast_placement_index: Some(8),
            menu_bar_action: None,
            tooltip_placement: TooltipPlacement::Top,
            image_tile_selected: false,
            image_tile_playback_state: ImageTilePlaybackState::Paused,
            image_tile_last_action: "No image tile actions yet".to_owned(),
            spinner_demo_until: None,
        }
    }
}
