use crate::catalog::{self, ComponentDefinition, ComponentGroup, ComponentKind};
use crate::layout;
use crate::prelude::*;
use crate::theme::{self, BaseColor, ColorRole, RadiusRole, ThemeMode, ThemeSpec};
use egui::{
    vec2, Align2, CentralPanel, Color32, CursorIcon, Id, Layout, Rect, ScrollArea, Sense,
    SidePanel, Stroke, TopBottomPanel, Ui,
};

pub const WINDOW_TITLE: &str = "egui-component Showcase";
pub const WINDOW_INNER_SIZE: [f32; 2] = [1280.0, 900.0];

const SHOWCASE_IMAGE_BYTES: &[u8] = include_bytes!("../../assets/images/showcase-image.png");

const BUTTON_GROUP_OPTIONS: [&str; 3] = ["Move", "Rotate", "Scale"];
const TOOLBAR_ACTION_OPTIONS: [&str; 3] = ["Edit", "BG Remover", "Eraser"];
const TOOLBAR_SWATCHES: [Color32; 4] = [
    Color32::from_rgb(35, 45, 75),
    Color32::from_rgb(103, 132, 162),
    Color32::from_rgb(122, 24, 42),
    Color32::from_rgb(206, 164, 84),
];
const TOOLTIP_PLACEMENT_OPTIONS: [&str; 4] = ["Top", "Right", "Bottom", "Left"];
const SELECT_OPTIONS: [&str; 4] = ["Draft", "Review", "Approved", "Archived"];
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
const THEME_MODE_OPTIONS: [TabOption<'static>; 2] =
    [TabOption::new(0, "Light"), TabOption::new(1, "Dark")];
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
const IMAGE_TILE_META_ACCENT: Color32 = Color32::from_rgb(59, 130, 246);
const NUMBER_INPUT_GREEN: Color32 = Color32::from_rgb(34, 197, 94);
const NUMBER_INPUT_RED: Color32 = Color32::from_rgb(239, 68, 68);

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
        ThemeSpec::preset(BaseColor::Neutral),
        ThemeMode::Dark,
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
    canva_brand_category_index: usize,
    canva_edit_tool_index: usize,
    canva_edit_filter_index: usize,
    input_value: String,
    field_value: String,
    checkbox_value: bool,
    switch_value: bool,
    small_switch_value: bool,
    toolbar_color_index: usize,
    canva_position_tab_index: usize,
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
    segmented_tab_index: usize,
    stacked_tab_index: usize,
    rail_tab_index: usize,
    audio_playback_state: AudioPlaybackState,
    pagination_page: usize,
    collapsible_open: bool,
    dropdown_action: Option<usize>,
    combobox_query: String,
    combobox_index: usize,
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
            theme_mode: ThemeMode::Dark,
            canva_background_query: String::new(),
            canva_background_color_index: 2,
            canva_brand_query: String::new(),
            canva_brand_category_index: 0,
            canva_edit_tool_index: 0,
            canva_edit_filter_index: 0,
            input_value: "Player_Robot".to_owned(),
            field_value: "M_Robot_Body".to_owned(),
            checkbox_value: true,
            switch_value: true,
            small_switch_value: false,
            toolbar_color_index: 0,
            canva_position_tab_index: 0,
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
            segmented_tab_index: 0,
            stacked_tab_index: 0,
            rail_tab_index: 0,
            audio_playback_state: AudioPlaybackState::Paused,
            pagination_page: 2,
            collapsible_open: true,
            dropdown_action: None,
            combobox_query: "mat".to_owned(),
            combobox_index: 0,
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

pub fn prepare_frame(app: &mut ShowcaseApp, ctx: &egui::Context) {
    clamp_state(app);
    theme::set_theme(ctx, ThemeSpec::preset(BaseColor::Neutral));
    theme::set_mode(ctx, app.theme_mode);
}

pub fn configure_snapshot(app: &mut ShowcaseApp, component: ComponentKind, theme_mode: ThemeMode) {
    app.selected_component = component;
    app.theme_mode = theme_mode;

    match component {
        ComponentKind::CanvaBrandKit => {
            app.canva_brand_query.clear();
            app.canva_brand_category_index = 0;
        }
        ComponentKind::CanvaEditImage => {
            app.canva_edit_tool_index = 0;
            app.canva_edit_filter_index = 0;
        }
        ComponentKind::Command => {
            app.command_query = "view".to_owned();
        }
        ComponentKind::Dialogue => {
            app.dialogue_open = true;
        }
        ComponentKind::Popover => {
            app.popover_open = true;
        }
        ComponentKind::Progress => {
            app.progress_value = 0.68;
        }
        ComponentKind::Sidebar => {
            app.sidebar_preview_open = true;
        }
        ComponentKind::Spinner => {
            app.spinner_demo_until = Some(f64::MAX);
        }
        ComponentKind::Toast => {
            app.toast_stack.clear();
            app.toast_stack.push(
                Toast::new("Changes saved")
                    .description("Your layout tokens were published successfully.")
                    .intent(ToastIntent::Success),
            );
            app.toast_stack.push(
                Toast::new("Heads up")
                    .description("Neutral notifications stack cleanly without blocking the UI."),
            );
        }
        _ => {}
    }
}

pub fn update(app: &mut ShowcaseApp, ctx: &egui::Context) {
    prepare_frame(app, ctx);
    let runtime = theme::runtime_for_context(ctx);
    TopBottomPanel::top("component_showcase_topbar")
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(showcase_header_fill(runtime))
                .stroke(Stroke::NONE),
        )
        .show(ctx, |ui| app.render_topbar(ui));

    SidePanel::left("component_showcase_sidebar")
        .resizable(true)
        .default_width(SIDEBAR_WIDTH)
        .min_width(200.0)
        .max_width(320.0)
        .show(ctx, |ui| app.render_sidebar(ui));

    CentralPanel::default().show(ctx, |ui| app.render_center(ui));
}

impl ShowcaseApp {
    fn render_sidebar(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);

        let _ = layout::column().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new("Components")
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Primary),
            );
        });

        ui.add_space(8.0);
        let _ = ui.components().separator();
        ui.add_space(8.0);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for section in [
                    ShowcaseSection::PrimaryPrimitive,
                    ShowcaseSection::DerivedComposed,
                    ShowcaseSection::Examples,
                    ShowcaseSection::Canva,
                ] {
                    let _ = ui.components().label(
                        Label::new(section.title())
                            .tone(LabelTone::Muted)
                            .size(SMALL_TEXT)
                            .weight(LabelWeight::Semibold),
                    );
                    ui.add_space(4.0);

                    for definition in showcase_component_definitions_by_section(section) {
                        let selected = self.selected_component == definition.kind;
                        let button = Button::new(definition.label)
                            .variant(ButtonVariant::Ghost)
                            .selected(selected)
                            .label_weight(if selected {
                                ButtonLabelWeight::Medium
                            } else {
                                ButtonLabelWeight::Regular
                            })
                            .min_size(vec2(ui.available_width(), 30.0));
                        if ui.components().button(button).clicked() {
                            self.selected_component = definition.kind;
                        }
                    }

                    ui.add_space(8.0);
                }
            });
    }

    fn render_topbar(&mut self, ui: &mut Ui) {
        let _ = layout::inset().padding(16, 10).show(ui, |ui| {
            show_showcase_topbar_row(ui, &mut self.theme_mode);
        });
        let _ = ui.components().separator();
    }

    fn render_center(&mut self, ui: &mut Ui) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                    self.render_preview_surface(ui);
                });
            });
    }

    fn render_preview_surface(&mut self, ui: &mut Ui) {
        let definition = catalog_component_definition(self.selected_component);
        let width = preview_surface_width(self.selected_component, ui.available_width());

        let _ = ui.components().card(Card::new().padding(14, 14), |ui| {
            ui.set_width(width);

            let mut components = ui.components();
            let _ = components.label(
                Label::new(definition.label)
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Primary),
            );
            let _ = components.label(
                Label::new(showcase_description(self.selected_component))
                    .tone(LabelTone::Muted)
                    .size(SMALL_TEXT),
            );
            ui.add_space(8.0);
            let _ = ui.components().separator();
            ui.add_space(10.0);

            self.render_selected_preview(ui);
        });
    }

    fn render_selected_preview(&mut self, ui: &mut Ui) {
        match self.selected_component {
            ComponentKind::Label => self.render_label_preview(ui),
            ComponentKind::Color => self.render_color_preview(ui),
            ComponentKind::Image => self.render_image_preview(ui),
            ComponentKind::Icon => self.render_icon_preview(ui),
            ComponentKind::Kbd => self.render_kbd_preview(ui),
            ComponentKind::Input => self.render_input_preview(ui),
            ComponentKind::Field => self.render_field_preview(ui),
            ComponentKind::Button => self.render_button_preview(ui),
            ComponentKind::ButtonGroup => self.render_button_group_preview(ui),
            ComponentKind::CanvaBackgrounds => self.render_canva_backgrounds_preview(ui),
            ComponentKind::CanvaBrandKit => self.render_canva_brand_kit_preview(ui),
            ComponentKind::CanvaEditImage => self.render_canva_edit_image_preview(ui),
            ComponentKind::CanvaPosition => self.render_canva_position_preview(ui),
            ComponentKind::Checkbox => self.render_checkbox_preview(ui),
            ComponentKind::Switch => self.render_switch_preview(ui),
            ComponentKind::Slider => self.render_slider_preview(ui),
            ComponentKind::NumberInput => self.render_number_input_preview(ui),
            ComponentKind::Select => self.render_select_preview(ui),
            ComponentKind::Tabs => self.render_tabs_preview(ui),
            ComponentKind::Separator => self.render_separator_preview(ui),
            ComponentKind::Card => self.render_card_preview(ui),
            ComponentKind::Progress => self.render_progress_preview(ui),
            ComponentKind::Radio => self.render_radio_preview(ui),
            ComponentKind::RadioGroup => self.render_radio_group_preview(ui),
            ComponentKind::Popover => self.render_popover_preview(ui),
            ComponentKind::Tooltip => self.render_tooltip_preview(ui),
            ComponentKind::DropdownMenu => self.render_dropdown_menu_preview(ui),
            ComponentKind::Collapsible => self.render_collapsible_preview(ui),
            ComponentKind::AudioPlayback => self.render_audio_playback_preview(ui),
            ComponentKind::Combobox => self.render_combobox_preview(ui),
            ComponentKind::Command => self.render_command_preview(ui),
            ComponentKind::Dialogue => self.render_dialogue_preview(ui),
            ComponentKind::ImageTile => self.render_image_tile_preview(ui),
            ComponentKind::MenuBar => self.render_menu_bar_preview(ui),
            ComponentKind::Sidebar => self.render_sidebar_preview(ui),
            ComponentKind::Skeleton => self.render_skeleton_preview(ui),
            ComponentKind::Spinner => self.render_spinner_preview(ui),
            ComponentKind::Toast => self.render_toast_preview(ui),
            ComponentKind::Toolbar => self.render_toolbar_preview(ui),
            ComponentKind::Pagination => self.render_pagination_preview(ui),
        }
    }
}

pub fn render_snapshot_component(app: &mut ShowcaseApp, ui: &mut Ui) -> egui::Response {
    ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
        layout::sized_box()
            .width(preview_surface_width(
                app.selected_component,
                ui.available_width(),
            ))
            .show(ui, |ui| {
                app.render_selected_preview(ui);
            })
            .response
    })
    .inner
}

pub fn render_snapshot_surface(app: &mut ShowcaseApp, ui: &mut Ui) {
    let _ = render_snapshot_component(app, ui);
}

fn show_showcase_topbar_row(ui: &mut Ui, theme_mode: &mut ThemeMode) {
    let _ = layout::leading_trailing().gap(12.0).min_height(30.0).show(
        ui,
        |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new("egui-component Showcase")
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Primary),
            );
        },
        |ui| {
            let _ = layout::row().gap(12.0).show(ui, |ui| {
                let mut components = ui.components();
                let _ = components.label(
                    Label::new("Theme mode")
                        .tone(LabelTone::Muted)
                        .size(SMALL_TEXT),
                );

                let mut selected_mode = theme_mode_index(*theme_mode);
                components.segmented_tabs(
                    Id::new("component_showcase_theme_mode"),
                    &mut selected_mode,
                    &THEME_MODE_OPTIONS,
                );
                *theme_mode = theme_mode_from_index(selected_mode);
            });
        },
    );
}

fn show_canva_panel_header(ui: &mut Ui, title: &str) -> egui::Response {
    layout::leading_trailing().gap(8.0).min_height(28.0).show(
        ui,
        |ui| {
            let _ = ui.components().label(
                Label::new(title)
                    .tone(LabelTone::Primary)
                    .weight(LabelWeight::Semibold),
            );
        },
        |ui| {
            let _ = ui.components().button(
                Button::icon_only("x")
                    .variant(ButtonVariant::Ghost)
                    .icon_size(18.0),
            );
        },
    )
}

fn show_section_title_with_trailing_label(
    ui: &mut Ui,
    title: &str,
    trailing_label: &str,
) -> egui::Response {
    layout::leading_trailing().gap(8.0).show(
        ui,
        |ui| {
            let _ = ui.components().label(
                Label::new(title)
                    .tone(LabelTone::Primary)
                    .weight(LabelWeight::Semibold),
            );
        },
        |ui| {
            let _ = ui.components().label(
                Label::new(trailing_label)
                    .tone(LabelTone::Muted)
                    .weight(LabelWeight::Semibold),
            );
        },
    )
}

fn show_section_link_row(ui: &mut Ui, icon: &str, label: &str) -> egui::Response {
    let primary_tint = text_secondary(ui);
    let muted_tint = text_muted(ui);
    layout::leading_trailing().gap(10.0).show(
        ui,
        |ui| {
            let _ = layout::row().gap(10.0).show(ui, |ui| {
                let _ = ui
                    .components()
                    .icon(Icon::new(icon).size(18.0).tint(primary_tint));
                let _ = ui.components().label(
                    Label::new(label)
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold),
                );
            });
        },
        |ui| {
            let _ = ui
                .components()
                .icon(Icon::new("chevron-right").size(18.0).tint(muted_tint));
        },
    )
}

fn show_preview_host(
    ui: &mut Ui,
    size: egui::Vec2,
    fill: Color32,
    stroke: Stroke,
    add: impl FnOnce(&mut Ui, Rect),
) {
    let (host_rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect(
        host_rect,
        egui::CornerRadius::ZERO,
        fill,
        stroke,
        egui::StrokeKind::Outside,
    );
    let _ = ui.scope_builder(egui::UiBuilder::new().max_rect(host_rect), |ui| {
        add(ui, host_rect);
    });
}

fn sidebar_preview_toggle_rect(
    host_rect: Rect,
    side_index: usize,
    preview_animation: f32,
    sidebar_width: f32,
) -> Rect {
    let x = if side_index == 0 {
        host_rect.left() + 8.0 + (preview_animation * (sidebar_width - 44.0))
    } else {
        host_rect.right() - 44.0 - (preview_animation * (sidebar_width - 44.0))
    };
    Rect::from_min_size(egui::pos2(x, host_rect.top() + 8.0), vec2(36.0, 36.0))
}

impl ShowcaseApp {
    fn render_label_preview(&mut self, ui: &mut Ui) {
        let mut components = ui.components();
        let _ = components.label(
            Label::new("Primary label")
                .tone(LabelTone::Primary)
                .weight(LabelWeight::Semibold),
        );
        let _ = components.label(Label::new("Secondary label").tone(LabelTone::Secondary));
        let _ = components.label(Label::new("Muted helper text").tone(LabelTone::Muted));
        let _ = components.label(
            Label::new("Destructive text")
                .tone(LabelTone::Destructive)
                .weight(LabelWeight::Semibold),
        );
    }

    fn render_color_preview(&mut self, ui: &mut Ui) {
        let border = Stroke::new(1.0, theme::color(ui, ColorRole::Border));

        let _ = ui.components().label(
            Label::new("Solid swatches")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(6.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            for fill in TOOLBAR_SWATCHES {
                let _ = components.color(Color::new(fill).size(20.0));
            }
        });

        ui.add_space(10.0);
        let _ = ui.components().label(
            Label::new("Bordered swatches")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(6.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            for fill in TOOLBAR_SWATCHES {
                let _ = components.color(Color::new(fill).size(20.0).stroke(border));
            }
        });
    }

    fn render_image_preview(&mut self, ui: &mut Ui) {
        let image = showcase_image("primary");
        let image_bg = input_background(ui);
        let md_radius = radius_md(ui);
        let sm_radius = radius_sm(ui);

        let _ = ui.components().label(
            Label::new("bytes:// PNG")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(6.0);
        let _ = ui.components().image(
            image
                .clone()
                .fit_to_exact_size(vec2(180.0, 180.0))
                .corner_radius(md_radius)
                .bg_fill(image_bg),
        );

        ui.add_space(10.0);
        let _ = ui.components().label(
            Label::new("Rotation")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(6.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.slider(&mut self.image_rotation_degrees, (-180.0..=180.0, 220.0));
            let rotation_label = format!("{:.0}deg", self.image_rotation_degrees.round());
            let _ = components.label(
                Label::new(rotation_label.as_str())
                    .tone(LabelTone::Secondary)
                    .weight(LabelWeight::Semibold),
            );
        });
        ui.add_space(6.0);
        let _ = ui.components().image(
            image
                .fit_to_exact_size(vec2(96.0, 96.0))
                .rotate(self.image_rotation_degrees.to_radians(), vec2(0.5, 0.5))
                .corner_radius(sm_radius)
                .bg_fill(image_bg),
        );
    }

    fn render_icon_preview(&mut self, ui: &mut Ui) {
        let _ = layout::row().gap(10.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.icon(Icon::new("bot").size(16.0));
            let _ = components.icon(Icon::new("settings-2").size(16.0));
            let _ = components.icon(Icon::new("sparkles").size(16.0));
            let _ = components.icon(Icon::new("gamepad-2").size(16.0));
            let _ = components.icon(Icon::new("wand-sparkles").size(16.0));
        });
    }

    fn render_kbd_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().kbd_group((), |ui| {
            let mut components = ui.components();
            let _ = components.kbd(Kbd::new("⌘"));
            let _ = components.kbd(Kbd::new("⇧"));
            let _ = components.kbd(Kbd::new("⌥"));
            let _ = components.kbd(Kbd::new("⌃"));
        });
        ui.add_space(6.0);
        let _ = ui.components().kbd_group(KbdGroup::new(), |ui| {
            let mut components = ui.components();
            let _ = components.kbd(Kbd::new("Ctrl"));
            let _ = components.label(Label::new("+").tone(LabelTone::Muted));
            let _ = components.kbd(Kbd::new("B"));
        });
    }

    fn render_input_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().text_input(
            &mut self.input_value,
            TextInput::new()
                .width(280.0)
                .placeholder("Type component name"),
        );
    }

    fn render_field_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().field(
            &mut self.field_value,
            Field::new("Material")
                .width(280.0)
                .helper_text("Assigned material for selected mesh"),
        );
    }

    fn render_button_preview(&mut self, ui: &mut Ui) {
        let selected_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Foreground));

        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.button(Button::new("Primary").variant(ButtonVariant::Primary));
            let _ = components.button(Button::new("Secondary").variant(ButtonVariant::Secondary));
            let _ = components.button(Button::new("Ghost").variant(ButtonVariant::Ghost));
            let _ = components.button(Button::new("Link").variant(ButtonVariant::Link));
        });

        ui.add_space(8.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.button(
                Button::icon_only("wand-sparkles")
                    .icon_size(15.0)
                    .variant(ButtonVariant::Primary),
            );
            let _ = components.button(
                Button::icon_only("wand-sparkles")
                    .icon_size(15.0)
                    .variant(ButtonVariant::Secondary),
            );
            let _ = components.button(
                Button::icon_only("wand-sparkles")
                    .icon_size(15.0)
                    .variant(ButtonVariant::Ghost),
            );
            let _ = components.button(
                Button::icon_only("wand-sparkles")
                    .icon_size(15.0)
                    .variant(ButtonVariant::Link),
            );
        });

        ui.add_space(8.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            for (index, fill) in TOOLBAR_SWATCHES.iter().copied().enumerate() {
                let stroke = if index == 1 {
                    selected_stroke
                } else {
                    Stroke::NONE
                };
                let _ = components.button(
                    Button::color_only(Color::new(fill).size(18.0).stroke(stroke))
                        .variant(ButtonVariant::Ghost),
                );
            }
        });
    }

    fn render_button_group_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().button_group(ButtonGroup::new(
            Id::new("component_showcase_button_group"),
            &BUTTON_GROUP_OPTIONS,
        ));
    }

    fn render_canva_backgrounds_preview(&mut self, ui: &mut Ui) {
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let panel_width = ui.available_width().min(352.0);
            let panel_fill = input_background(ui);
            let _ = ui
                .components()
                .card(Card::new().padding(16, 16).fill(panel_fill), |ui| {
                    ui.set_width(panel_width);
                    let full_width = ui.available_width();

                    let _ = ui.components().text_input(
                        &mut self.canva_background_query,
                        TextInput::new()
                            .width(full_width)
                            .placeholder("Search backgrounds"),
                    );

                    ui.add_space(12.0);
                    let _ = layout::leading_trailing().gap(12.0).min_height(36.0).show(
                        ui,
                        |ui| {
                            let _ = layout::row().gap(8.0).show(ui, |ui| {
                                let _ = ui.components().button(
                                    Button::icon_only("palette")
                                        .variant(ButtonVariant::Secondary)
                                        .icon_size(18.0)
                                        .min_size(vec2(40.0, 36.0)),
                                );

                                for (index, fill) in
                                    CANVA_BACKGROUND_SWATCHES.iter().copied().enumerate()
                                {
                                    let stroke = if self.canva_background_color_index == index {
                                        Stroke::new(1.0, theme::color(ui, ColorRole::Foreground))
                                    } else {
                                        Stroke::NONE
                                    };

                                    if ui
                                        .components()
                                        .button(
                                            Button::color_only(
                                                Color::new(fill).size(32.0).stroke(stroke),
                                            )
                                            .variant(ButtonVariant::Ghost),
                                        )
                                        .clicked()
                                    {
                                        self.canva_background_color_index = index;
                                    }
                                }
                            });
                        },
                        |ui| {
                            let _ = ui.components().button(
                                Button::icon_only("chevron-right")
                                    .variant(ButtonVariant::Primary)
                                    .icon_size(18.0)
                                    .min_size(vec2(36.0, 36.0)),
                            );
                        },
                    );

                    ui.add_space(12.0);
                    let _ = ui.components().button(
                        Button::new("Magic Background")
                            .variant(ButtonVariant::Secondary)
                            .leading_icon("sparkles")
                            .trailing_icon("crown")
                            .icon_size(16.0)
                            .label_weight(ButtonLabelWeight::Medium)
                            .min_size(vec2(full_width, 38.0)),
                    );

                    ui.add_space(18.0);
                    let _ = ui.components().label(
                        Label::new("All results")
                            .tone(LabelTone::Primary)
                            .weight(LabelWeight::Semibold),
                    );

                    ui.add_space(10.0);
                    draw_canva_background_placeholder_grid(ui);
                });
        });
    }

    fn render_canva_brand_kit_preview(&mut self, ui: &mut Ui) {
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let panel_width = ui.available_width().min(560.0);
            let panel_fill = input_background(ui);
            let _ = ui
                .components()
                .card(Card::new().padding(14, 14).fill(panel_fill), |ui| {
                    let _ = ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                        ui.set_width(panel_width);

                        let left_width = 170.0;
                        let gap = 14.0;

                        let _ = ui.scope(|ui| {
                            let _ = ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                                ui.set_width(left_width);

                                let _ = ui.components().text_input(
                                    &mut self.canva_brand_query,
                                    TextInput::new().width(left_width).placeholder("Search"),
                                );

                                ui.add_space(14.0);
                                let _ = ui.components().label(
                                    Label::new("All Brand Templates")
                                        .tone(LabelTone::Secondary)
                                        .weight(LabelWeight::Semibold),
                                );

                                ui.add_space(14.0);
                                let _ = ui.components().separator();
                                ui.add_space(14.0);
                                draw_canva_brand_dropdown(ui, left_width);

                                ui.add_space(8.0);
                                for (index, category) in CANVA_BRAND_CATEGORIES.iter().enumerate() {
                                    let selected = self.canva_brand_category_index == index;
                                    if draw_canva_brand_nav_item(ui, category, selected, left_width)
                                        .clicked()
                                    {
                                        self.canva_brand_category_index = index;
                                    }
                                    ui.add_space(4.0);
                                }
                            });
                        });

                        ui.add_space(gap);
                        draw_canva_brand_vertical_divider(ui, 520.0);
                        ui.add_space(gap);

                        let _ = ui.scope(|ui| {
                            let _ = ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                                ui.set_width(
                                    (panel_width - left_width - gap * 2.0 - 1.0).max(260.0),
                                );
                                render_canva_brand_detail(ui, self.canva_brand_category_index);
                            });
                        });
                    });
                });
        });
    }

    fn render_canva_edit_image_preview(&mut self, ui: &mut Ui) {
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let panel_width = ui.available_width().min(356.0);
            let panel_fill = input_background(ui);
            let _ = ui
                .components()
                .card(Card::new().padding(16, 16).fill(panel_fill), |ui| {
                    let _ = ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                        ui.set_width(panel_width);

                        let _ = show_canva_panel_header(ui, "Edit image");

                        ui.add_space(18.0);
                        let _ = ui.components().label(
                            Label::new("Select area")
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );
                        ui.add_space(10.0);
                        draw_canva_edit_tool_row(ui, &mut self.canva_edit_tool_index);

                        ui.add_space(12.0);
                        let _ = ui.components().separator();
                        ui.add_space(12.0);
                        draw_canva_edit_navigation_row(ui, "sliders-horizontal", "Adjust");

                        ui.add_space(12.0);
                        let _ = ui.components().separator();
                        ui.add_space(12.0);
                        let _ = ui.components().label(
                            Label::new("Magic Studio")
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );
                        ui.add_space(10.0);
                        draw_canva_edit_rail(
                            ui,
                            Id::new("component_showcase_canva_edit_magic"),
                            &CANVA_MAGIC_STUDIO_ITEMS,
                            None,
                            false,
                        );

                        ui.add_space(14.0);
                        let _ = ui.components().separator();
                        ui.add_space(12.0);
                        let _ = show_section_title_with_trailing_label(ui, "Filters", "See all");
                        ui.add_space(10.0);
                        draw_canva_edit_rail(
                            ui,
                            Id::new("component_showcase_canva_edit_filters"),
                            &CANVA_FILTER_ITEMS,
                            Some(&mut self.canva_edit_filter_index),
                            true,
                        );

                        ui.add_space(14.0);
                        let _ = ui.components().separator();
                        ui.add_space(12.0);
                        let _ = ui.components().label(
                            Label::new("Effects")
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );
                        ui.add_space(10.0);
                        draw_canva_edit_rail(
                            ui,
                            Id::new("component_showcase_canva_edit_effects"),
                            &CANVA_EFFECT_ITEMS,
                            None,
                            true,
                        );
                    });
                });
        });
    }

    fn render_canva_position_preview(&mut self, ui: &mut Ui) {
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let panel_width = ui.available_width().min(352.0);
            let panel_fill = input_background(ui);
            let _ = ui
                .components()
                .card(Card::new().padding(16, 16).fill(panel_fill), |ui| {
                    ui.set_width(panel_width);

                    let _ = show_canva_panel_header(ui, "Position");

                    ui.add_space(10.0);
                    ui.components().tabs(
                        Id::new("component_showcase_canva_position_tabs"),
                        &mut self.canva_position_tab_index,
                        &CANVA_POSITION_TAB_OPTIONS,
                    );

                    ui.add_space(14.0);
                    draw_canva_position_button_grid(ui, &CANVA_POSITION_ACTIONS);

                    if self.canva_position_tab_index == 0 {
                        ui.add_space(18.0);
                        let _ = ui.components().label(
                            Label::new("Align to page")
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );
                        ui.add_space(10.0);
                        draw_canva_position_button_grid(ui, &CANVA_POSITION_ALIGN_ACTIONS);
                    }

                    ui.add_space(18.0);
                    let _ = ui.components().label(
                        Label::new("Advanced")
                            .tone(LabelTone::Primary)
                            .weight(LabelWeight::Semibold),
                    );
                    ui.add_space(12.0);

                    let field_gap = 8.0;
                    let field_width = ((ui.available_width() - (field_gap * 2.0)) / 3.0)
                        .floor()
                        .max(96.0);

                    let _ = layout::row().gap(field_gap).show(ui, |ui| {
                        draw_canva_number_field(
                            ui,
                            "Width",
                            &mut self.canva_width_value,
                            NumberInput::new(Id::new("component_showcase_canva_width"))
                                .width(field_width)
                                .range(1.0..=4_000.0)
                                .speed(1.0)
                                .fine_speed(0.01)
                                .decimals(0)
                                .fine_decimals(2)
                                .suffix("px"),
                        );
                        draw_canva_number_field(
                            ui,
                            "Height",
                            &mut self.canva_height_value,
                            NumberInput::new(Id::new("component_showcase_canva_height"))
                                .width(field_width)
                                .range(1.0..=4_000.0)
                                .speed(1.0)
                                .fine_speed(0.01)
                                .decimals(0)
                                .fine_decimals(2)
                                .suffix("px"),
                        );
                        draw_canva_ratio_field(ui, field_width, &mut self.canva_ratio_locked);
                    });

                    ui.add_space(10.0);
                    let _ = layout::row().gap(field_gap).show(ui, |ui| {
                        draw_canva_number_field(
                            ui,
                            "X",
                            &mut self.canva_x_value,
                            NumberInput::new(Id::new("component_showcase_canva_x"))
                                .width(field_width)
                                .range(-4_000.0..=4_000.0)
                                .speed(1.0)
                                .fine_speed(0.01)
                                .decimals(0)
                                .fine_decimals(2)
                                .suffix("px"),
                        );
                        draw_canva_number_field(
                            ui,
                            "Y",
                            &mut self.canva_y_value,
                            NumberInput::new(Id::new("component_showcase_canva_y"))
                                .width(field_width)
                                .range(-4_000.0..=4_000.0)
                                .speed(1.0)
                                .fine_speed(0.01)
                                .decimals(0)
                                .fine_decimals(2)
                                .suffix("px"),
                        );
                        draw_canva_number_field(
                            ui,
                            "Rotate",
                            &mut self.canva_rotate_value,
                            NumberInput::new(Id::new("component_showcase_canva_rotate"))
                                .width(field_width)
                                .range(-360.0..=360.0)
                                .speed(1.0)
                                .fine_speed(0.01)
                                .decimals(0)
                                .fine_decimals(2)
                                .suffix("°"),
                        );
                    });
                });
        });
    }

    fn render_checkbox_preview(&mut self, ui: &mut Ui) {
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.checkbox(&mut self.checkbox_value, Checkbox::new());

            let label_color = if self.checkbox_value {
                text_secondary(ui)
            } else {
                text_muted(ui)
            };
            let response = ui
                .scope(|ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.add(
                        egui::Label::new(egui::RichText::new("Receive Shadows").color(label_color))
                            .selectable(false)
                            .sense(Sense::click()),
                    )
                })
                .inner
                .on_hover_cursor(CursorIcon::PointingHand);
            if response.clicked() {
                self.checkbox_value = !self.checkbox_value;
            }
        });
    }

    fn render_switch_preview(&mut self, ui: &mut Ui) {
        let _ = ui
            .components()
            .switch(&mut self.switch_value, "Enable Post FX");
        let _ = ui.components().switch(
            &mut self.small_switch_value,
            Switch::new()
                .label("Use Compact Handles")
                .size(ControlSize::Sm),
        );
    }

    fn render_slider_preview(&mut self, ui: &mut Ui) {
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.slider(&mut self.slider_value, (0.0..=100.0, 250.0));
            let value_label = format!("{:.0}", self.slider_value.round());
            let _ = components.label(
                Label::new(value_label.as_str())
                    .tone(LabelTone::Secondary)
                    .weight(LabelWeight::Semibold),
            );
        });
    }

    fn render_number_input_preview(&mut self, ui: &mut Ui) {
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.number_input(
                &mut self.number_x_value,
                NumberInput::new(Id::new("component_showcase_number_x"))
                    .width(110.0)
                    .range(0.0..=100.0)
                    .decimals(1)
                    .prefix("X")
                    .prefix_tint(NUMBER_INPUT_GREEN)
                    .prefix_align_left()
                    .axis(NumberInputAxis::Horizontal),
            );
            let _ = components.number_input(
                &mut self.number_y_value,
                NumberInput::new(Id::new("component_showcase_number_y"))
                    .width(110.0)
                    .range(0.0..=100.0)
                    .decimals(1)
                    .prefix("Y")
                    .prefix_tint(NUMBER_INPUT_RED)
                    .prefix_align_left()
                    .axis(NumberInputAxis::Vertical),
            );
        });
    }

    fn render_select_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().select(
            &mut self.select_index,
            Select::from_id(Id::new("component_showcase_select"), &SELECT_OPTIONS).width(280.0),
        );

        let selected_label = self
            .select_index
            .and_then(|index| SELECT_OPTIONS.get(index).copied())
            .unwrap_or("None");
        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new(selected_label)
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
    }

    fn render_tabs_preview(&mut self, ui: &mut Ui) {
        ui.components().tabs(
            Id::new("component_showcase_tabs"),
            &mut self.tab_index,
            &TAB_OPTIONS,
        );

        let selected_tab = TAB_OPTIONS
            .iter()
            .find(|option| option.value == self.tab_index)
            .map(|option| option.label)
            .unwrap_or(TAB_OPTIONS[0].label);
        ui.add_space(6.0);
        let _ = ui.components().label(
            Label::new(selected_tab)
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );

        ui.add_space(18.0);
        ui.components().segmented_tabs(
            Id::new("component_showcase_segmented_tabs"),
            &mut self.segmented_tab_index,
            &TAB_OPTIONS,
        );

        ui.add_space(18.0);
        ui.components().stacked_tabs(
            Id::new("component_showcase_stacked_tabs"),
            &mut self.stacked_tab_index,
            &STACKED_TAB_OPTIONS,
        );

        ui.add_space(18.0);
        ui.components().rail_tabs(
            Id::new("component_showcase_rail_tabs"),
            &mut self.rail_tab_index,
            &RAIL_TAB_OPTIONS,
        );
    }

    fn render_separator_preview(&mut self, ui: &mut Ui) {
        let mut components = ui.components();
        let _ = components.label(Label::new("Above separator").tone(LabelTone::Secondary));
        let _ = components.separator();
        let _ = components.label(Label::new("Below separator").tone(LabelTone::Secondary));
    }

    fn render_card_preview(&mut self, ui: &mut Ui) {
        let card_fill = input_background(ui);
        let card_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Border));
        let _ = layout::sized_box()
            .width(ui.available_width())
            .show(ui, |ui| {
                let _ =
                    ui.components()
                        .card(Card::new().fill(card_fill).stroke(card_stroke), |ui| {
                            ui.set_min_width(ui.available_width());

                            let _ = layout::column().gap(6.0).show(ui, |ui| {
                                let _ = ui.components().label(
                                    Label::new("Card Title")
                                        .tone(LabelTone::Primary)
                                        .weight(LabelWeight::Bold)
                                        .size(16.0),
                                );
                                let _ = ui.components().label(
                                    Label::new("Cards wrap related content in a bordered panel.")
                                        .tone(LabelTone::Muted),
                                );
                            });

                            ui.add_space(6.0);
                            let _ = ui.components().separator();
                            ui.add_space(6.0);

                            let footer_size =
                                egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                            let _ = ui.allocate_ui_with_layout(
                                footer_size,
                                Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let mut components = ui.components();
                                    let _ = components.button(
                                        Button::new("Save").variant(ButtonVariant::Primary),
                                    );
                                    let _ = components.button(
                                        Button::new("Cancel").variant(ButtonVariant::Secondary),
                                    );
                                },
                            );
                        });
            });
    }

    fn render_progress_preview(&mut self, ui: &mut Ui) {
        let _ = ui
            .components()
            .progress(self.progress_value, Progress::new().width(280.0));
        ui.add_space(10.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            if components
                .button(Button::new("Advance").variant(ButtonVariant::Primary))
                .clicked()
            {
                self.progress_value = (self.progress_value + 0.1).clamp(0.0, 1.0);
            }
            if components
                .button(Button::new("Reset").variant(ButtonVariant::Secondary))
                .clicked()
            {
                self.progress_value = 0.0;
            }
        });
    }

    fn render_spinner_preview(&mut self, ui: &mut Ui) {
        let spinner_tint = theme::color(ui, ColorRole::Foreground);
        let _ = layout::row().gap(16.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.spinner(Spinner::new().size(16.0));
            let _ = components.spinner(Spinner::new().size(22.0));
            let _ = components.spinner(
                Spinner::new()
                    .size(28.0)
                    .stroke_width(2.4)
                    .color(spinner_tint),
            );
        });

        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new("Indeterminate loading spinner with configurable size, stroke, and tint.")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );

        let now = ui.input(|input| input.time);
        let spinner_demo_active = self.spinner_demo_until.is_some_and(|until| until > now);
        if let Some(until) = self.spinner_demo_until {
            if until > now {
                ui.ctx().request_repaint_after_secs((until - now) as f32);
            } else {
                self.spinner_demo_until = None;
            }
        }

        ui.add_space(12.0);
        let _ = ui.components().label(
            Label::new("Loading button example")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(6.0);
        let trigger = ui
            .add_enabled_ui(!spinner_demo_active, |ui| {
                ui.components().button(
                    Button::new(if spinner_demo_active {
                        "  Publish"
                    } else {
                        "Publish"
                    })
                    .variant(ButtonVariant::Secondary)
                    .min_size(vec2(112.0, 34.0)),
                )
            })
            .inner;
        if trigger.clicked() {
            self.spinner_demo_until = Some(now + 3.0);
        }

        if spinner_demo_active {
            let spinner_color = theme::color(ui, ColorRole::Foreground);
            let spinner_rect = egui::Rect::from_center_size(
                egui::pos2(trigger.rect.left() + 18.0, trigger.rect.center().y),
                vec2(14.0, 14.0),
            );
            let _ = ui.scope_builder(egui::UiBuilder::new().max_rect(spinner_rect), |ui| {
                let _ = ui.components().spinner(
                    Spinner::new()
                        .size(14.0)
                        .stroke_width(2.0)
                        .speed(1.4)
                        .color(spinner_color),
                );
            });
        }
    }

    fn render_skeleton_preview(&mut self, ui: &mut Ui) {
        let toggle_label = if self.skeleton_loading {
            "Show Loaded State"
        } else {
            "Show Loading State"
        };
        if ui
            .components()
            .button(Button::new(toggle_label).variant(ButtonVariant::Secondary))
            .clicked()
        {
            self.skeleton_loading = !self.skeleton_loading;
        }

        ui.add_space(10.0);
        let card_fill = input_background(ui);
        let card_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Border));
        let _ = ui
            .components()
            .card(Card::new().fill(card_fill).stroke(card_stroke), |ui| {
                let primary_tint = theme::color(ui, ColorRole::Primary);
                let _ = layout::row().gap(12.0).show(ui, |ui| {
                    let mut components = ui.components();
                    if self.skeleton_loading {
                        let _ = components.skeleton(Skeleton::new().circle(44.0));
                    } else {
                        let _ = components.icon(
                            Icon::new("sparkles").size(20.0).tint(primary_tint),
                        );
                    }

                    let _ = layout::column().gap(8.0).show(ui, |ui| {
                        let mut components = ui.components();
                        if self.skeleton_loading {
                            let _ = components.skeleton((180.0, 16.0));
                            let _ = components.skeleton((240.0, 12.0));
                            let _ = components.skeleton((212.0, 12.0));
                            ui.add_space(2.0);
                            let _ = layout::row().gap(8.0).show(ui, |ui| {
                                let mut components = ui.components();
                                let _ = components.skeleton((72.0, 28.0));
                                let _ = components.skeleton((96.0, 28.0));
                            });
                        } else {
                            let _ = components.label(
                                Label::new("Loading state complete")
                                    .tone(LabelTone::Primary)
                                    .weight(LabelWeight::Semibold),
                            );
                            let _ = components.label(
                                Label::new(
                                    "Skeleton blocks can be mixed to mirror the final layout while data is in flight.",
                                )
                                .tone(LabelTone::Muted)
                                .size(SMALL_TEXT),
                            );
                            ui.add_space(2.0);
                            let _ = layout::row().gap(8.0).show(ui, |ui| {
                                let mut components = ui.components();
                                let _ = components.button(
                                    Button::new("Inspect").variant(ButtonVariant::Primary),
                                );
                                let _ = components.button(
                                    Button::new("Dismiss").variant(ButtonVariant::Secondary),
                                );
                            });
                        }
                    });
                });
            });
    }

    fn render_radio_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().radio(
            &mut self.radio_value,
            Radio::new()
                .label("Use publish channel")
                .description("Standalone radios stay selected until you explicitly reset them."),
        );

        ui.add_space(8.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            if components
                .button(Button::new("Reset").variant(ButtonVariant::Secondary))
                .clicked()
            {
                self.radio_value = false;
            }
            let state_label = if self.radio_value {
                "Selected"
            } else {
                "Unselected"
            };
            let _ = components.label(
                Label::new(state_label)
                    .tone(LabelTone::Muted)
                    .size(SMALL_TEXT),
            );
        });
    }

    fn render_radio_group_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().radio_group(
            &mut self.radio_group_value,
            RadioGroup::new(
                Id::new("component_showcase_radio_group"),
                &RADIO_GROUP_OPTIONS,
            ),
        );

        ui.add_space(8.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            if components
                .button(Button::new("Clear").variant(ButtonVariant::Secondary))
                .clicked()
            {
                self.radio_group_value = None;
            }
            let selected = self
                .radio_group_value
                .and_then(|value| {
                    RADIO_GROUP_OPTIONS
                        .iter()
                        .find(|option| option.value == value)
                        .map(|option| option.label)
                })
                .unwrap_or("No option selected");
            let _ = components.label(Label::new(selected).tone(LabelTone::Muted).size(SMALL_TEXT));
        });
    }

    fn render_tooltip_preview(&mut self, ui: &mut Ui) {
        let mut placement_index = tooltip_placement_index(self.tooltip_placement);
        let _ = ui.components().toggle_group(
            &mut placement_index,
            ToggleGroup::new(
                Id::new("component_showcase_tooltip_placement"),
                &TOOLTIP_PLACEMENT_OPTIONS,
            )
            .min_segment_width(72.0),
        );
        self.tooltip_placement = tooltip_placement_from_index(placement_index);
        ui.add_space(8.0);
        let _ = ui.components().tooltip(
            Tooltip::new("Hover this trigger", "Tooltip content example")
                .width(220.0)
                .placement(self.tooltip_placement),
        );
    }

    fn render_popover_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().popover(
            &mut self.popover_open,
            Popover::new(Id::new("component_showcase_popover"))
                .width(280.0)
                .side_offset(1.0),
            |ui| {
                ui.components()
                    .button(Button::new("Open Popover").variant(ButtonVariant::Secondary))
            },
            |ui, open| {
                let _ = layout::column().gap(10.0).show(ui, |ui| {
                    let mut components = ui.components();
                    let _ = components.label(
                        Label::new("Layout settings")
                            .weight(LabelWeight::Semibold)
                            .tone(LabelTone::Primary),
                    );
                    let _ = components.label(
                        Label::new("Interactive popovers work well for small settings panels.")
                            .tone(LabelTone::Muted)
                            .size(SMALL_TEXT),
                    );
                    let _ = components.switch(&mut self.popover_compact_mode, "Compact mode");
                    let _ = components.switch(&mut self.switch_value, "Snap to grid");

                    ui.add_space(4.0);
                    let _ = layout::row().gap(8.0).show(ui, |ui| {
                        let mut components = ui.components();
                        if components
                            .button(Button::new("Close").variant(ButtonVariant::Ghost))
                            .clicked()
                        {
                            *open = false;
                        }
                        let _ =
                            components.button(Button::new("Apply").variant(ButtonVariant::Primary));
                    });
                });
            },
        );
        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new(if self.popover_compact_mode {
                "Compact mode enabled"
            } else {
                "Compact mode disabled"
            })
            .tone(LabelTone::Muted)
            .size(SMALL_TEXT),
        );
    }

    fn render_dropdown_menu_preview(&mut self, ui: &mut Ui) {
        let (_, state) = ui.components().dropdown_menu(
            DropdownMenu::new("Open")
                .entries(&DROPDOWN_ENTRIES)
                .width(220.0),
        );
        if let Some(action) = state.action {
            self.dropdown_action = Some(action);
        }
        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new(dropdown_action_label(self.dropdown_action))
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
    }

    fn render_collapsible_preview(&mut self, ui: &mut Ui) {
        let collapsible_open = self.collapsible_open;
        let _ = ui.components().collapsible(
            &mut self.collapsible_open,
            Collapsible::new(Id::new("component_showcase_collapsible"), "Transform")
                .open(collapsible_open)
                .leading_icon("move-3d")
                .trailing_icon("ellipsis"),
            |ui| {
                let mut components = ui.components();
                let _ = components.label(Label::new("Position").tone(LabelTone::Secondary));
                let _ = components.label(Label::new("Rotation").tone(LabelTone::Secondary));
                let _ = components.label(Label::new("Scale").tone(LabelTone::Secondary));
            },
        );
    }

    fn render_audio_playback_preview(&mut self, ui: &mut Ui) {
        let (_, playback_result) = ui.components().audio_playback_with_actions(
            AudioPlayback::new(
                Id::new("component_showcase_audio_playback"),
                self.audio_playback_state,
            ),
            |ui| {
                let mut components = ui.components();
                let _ = components.button(
                    Button::icon_only("share-2")
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Sm),
                );
                let _ = components.button(
                    Button::icon_only("download")
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Sm),
                );
                let _ = components.button(
                    Button::icon_only("ellipsis")
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Sm),
                );
            },
        );

        if playback_result.play_pause_clicked {
            self.audio_playback_state = match self.audio_playback_state {
                AudioPlaybackState::Paused => AudioPlaybackState::Playing,
                AudioPlaybackState::Playing => AudioPlaybackState::Paused,
            };
        }

        ui.add_space(8.0);
        let status = match self.audio_playback_state {
            AudioPlaybackState::Paused => "Playback: Paused",
            AudioPlaybackState::Playing => "Playback: Playing",
        };
        let _ = ui
            .components()
            .label(Label::new(status).tone(LabelTone::Muted).size(SMALL_TEXT));
    }

    fn render_combobox_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().combobox(
            &mut self.combobox_query,
            &mut self.combobox_index,
            Combobox::new(Id::new("component_showcase_combobox"), &COMBOBOX_OPTIONS).width(280.0),
        );
        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new(COMBOBOX_OPTIONS[self.combobox_index])
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
    }

    fn render_command_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().command(
            &mut self.command_query,
            &COMMAND_ITEMS,
            Command::new(Id::new("component_showcase_command"))
                .width(380.0)
                .preview(true)
                .preview_height(220.0),
        );
    }

    fn render_dialogue_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().dialogue(
            &mut self.dialogue_open,
            Dialogue::new(Id::new("component_showcase_dialogue"), "Create Component")
                .description("Adds the selected component to the active object.")
                .trigger_label("Open Dialogue")
                .confirm_label("Create")
                .cancel_label("Cancel")
                .intent(DialogueIntent::Default)
                .width(380.0),
        );
    }

    fn render_image_tile_preview(&mut self, ui: &mut Ui) {
        let featured_image = showcase_image("featured");
        let secondary_image = showcase_image("secondary");

        let _ = ui.components().label(
            Label::new("Featured tile")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(8.0);

        let (_, featured_state) = ui.components().image_tile_with_body(
            ImageTile::new(featured_image.clone()).size(ImageTileSize::Lg),
            |ui| {
                let mut components = ui.components();
                let _ = components.label(
                    Label::new("Untitled Design")
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold)
                        .size(16.0),
                );
                draw_image_tile_metadata_row(ui, "Edited 2 days ago");
            },
        );
        if featured_state.tile_clicked {
            self.image_tile_selected = !self.image_tile_selected;
            self.image_tile_last_action = "Opened Untitled Design".to_owned();
        }

        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new("Audio preview")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(8.0);
        let (_, audio_state) = ui.components().image_tile_with_body(
            ImageTile::new(secondary_image)
                .size(ImageTileSize::Md)
                .selected(self.image_tile_selected)
                .playback_state(self.image_tile_playback_state),
            |ui| {
                let mut components = ui.components();
                let _ = components.label(
                    Label::new("Ambient Preview")
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold),
                );
                let playback_label = match self.image_tile_playback_state {
                    ImageTilePlaybackState::Paused => "Paused • Click play to preview",
                    ImageTilePlaybackState::Playing => "Playing • 0:27 loop",
                };
                draw_image_tile_metadata_row(ui, playback_label);
            },
        );
        if audio_state.play_pause_clicked {
            self.image_tile_playback_state = match self.image_tile_playback_state {
                ImageTilePlaybackState::Paused => ImageTilePlaybackState::Playing,
                ImageTilePlaybackState::Playing => ImageTilePlaybackState::Paused,
            };
            self.image_tile_last_action = match self.image_tile_playback_state {
                ImageTilePlaybackState::Paused => "Paused audio preview".to_owned(),
                ImageTilePlaybackState::Playing => "Started audio preview".to_owned(),
            };
        } else if audio_state.tile_clicked {
            self.image_tile_last_action = "Opened audio preview tile".to_owned();
        }

        ui.add_space(8.0);
        let status = format!("Last action: {}", self.image_tile_last_action);
        let _ = ui.components().label(
            Label::new(status.as_str())
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
    }

    fn render_menu_bar_preview(&mut self, ui: &mut Ui) {
        let card_fill = app_background(ui);
        let card_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Border));
        let menu_items = [
            MenuBarItem::new("File", &MENU_BAR_FILE_ENTRIES).width(220.0),
            MenuBarItem::new("Edit", &MENU_BAR_EDIT_ENTRIES).width(190.0),
            MenuBarItem::new("View", &MENU_BAR_VIEW_ENTRIES).width(196.0),
            MenuBarItem::new("Object", &MENU_BAR_OBJECT_ENTRIES).width(220.0),
        ];

        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let _ = ui
                .components()
                .card(Card::new().fill(card_fill).stroke(card_stroke), |ui| {
                    ui.set_width(MENU_BAR_PREVIEW_WIDTH.min(ui.available_width()));
                    let (_, state) = ui.components().menu_bar(MenuBar::new(
                        Id::new("component_showcase_menu_bar"),
                        &menu_items,
                    ));
                    if let Some(action) = state.action {
                        self.menu_bar_action = Some(action);
                    }
                });
        });

        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new(menu_bar_action_label(self.menu_bar_action))
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
    }

    fn render_sidebar_preview(&mut self, ui: &mut Ui) {
        if let Some(index) = ui.components().button_group(ButtonGroup::new(
            Id::new("component_showcase_sidebar_side"),
            &SIDEBAR_SIDE_OPTIONS,
        )) {
            self.sidebar_side_index = index;
        }

        ui.add_space(10.0);
        let canvas_fill = if self.theme_mode.is_dark() {
            app_background(ui)
        } else {
            TOOLBAR_CANVAS_LIGHT_FILL
        };
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let width = 700.0f32.min(ui.available_width());
            let host_size = vec2(width, 360.0);
            show_preview_host(
                ui,
                host_size,
                canvas_fill,
                Stroke::new(1.0, theme::color(ui, ColorRole::Border)),
                |ui, host_rect| {
                    let preview_animation = ui.ctx().animate_bool_responsive(
                        Id::new("component_showcase_sidebar_preview"),
                        self.sidebar_preview_open,
                    );
                    let sidebar_width = 240.0;
                    let toggle_rect = sidebar_preview_toggle_rect(
                        host_rect,
                        self.sidebar_side_index,
                        preview_animation,
                        sidebar_width,
                    );
                    let toggle_icon = sidebar_preview_toggle_icon(
                        self.sidebar_side_index,
                        self.sidebar_preview_open,
                    );
                    let toggle_response = ui
                        .scope_builder(egui::UiBuilder::new().max_rect(toggle_rect), |ui| {
                            ui.components().button(
                                Button::icon_only(toggle_icon)
                                    .variant(if self.sidebar_preview_open {
                                        ButtonVariant::Secondary
                                    } else {
                                        ButtonVariant::Ghost
                                    })
                                    .icon_size(16.0)
                                    .min_size(vec2(36.0, 36.0)),
                            )
                        })
                        .inner;
                    if toggle_response.clicked() {
                        self.sidebar_preview_open = !self.sidebar_preview_open;
                    }

                    ui.components().sidebar_in(
                        host_rect,
                        &mut self.sidebar_preview_open,
                        Sidebar::new(Id::new("component_showcase_overlay_sidebar"))
                            .title("Workspace")
                            .side(if self.sidebar_side_index == 0 {
                                SidebarSide::Left
                            } else {
                                SidebarSide::Right
                            })
                            .width(sidebar_width)
                            .backdrop(false),
                        |ui, open| {
                            let _ = layout::column().gap(8.0).show(ui, |ui| {
                                let _ = ui.components().button(
                                    Button::new("New Draft")
                                        .variant(ButtonVariant::Primary)
                                        .leading_icon("file-plus"),
                                );
                                let _ = ui.components().button(
                                    Button::new("Command Search")
                                        .variant(ButtonVariant::Ghost)
                                        .leading_icon("search"),
                                );
                                let _ = ui.components().button(
                                    Button::new("Theme Tokens")
                                        .variant(ButtonVariant::Ghost)
                                        .leading_icon("palette"),
                                );
                                let _ = ui.components().button(
                                    Button::new("Exports")
                                        .variant(ButtonVariant::Ghost)
                                        .leading_icon("rocket"),
                                );
                                ui.add_space(4.0);
                                if ui
                                    .components()
                                    .button(
                                        Button::new("Close Sidebar")
                                            .variant(ButtonVariant::Secondary),
                                    )
                                    .clicked()
                                {
                                    *open = false;
                                }
                            });
                        },
                    );
                },
            );
        });
    }

    fn render_toast_preview(&mut self, ui: &mut Ui) {
        let placement_label = self
            .toast_placement_index
            .and_then(|index| TOAST_PLACEMENT_OPTIONS.get(index).copied())
            .unwrap_or(TOAST_PLACEMENT_OPTIONS[8]);
        let _ = ui.components().select(
            &mut self.toast_placement_index,
            Select::from_id(
                Id::new("component_showcase_toast_position"),
                &TOAST_PLACEMENT_OPTIONS,
            )
            .width(220.0),
        );

        ui.add_space(8.0);
        let _ =
            layout::row().gap(8.0).show(ui, |ui| {
                let mut components = ui.components();
                if components
                    .button(Button::new("Neutral").variant(ButtonVariant::Secondary))
                    .clicked()
                {
                    self.toast_stack.push(Toast::new("Heads up").description(
                        "Neutral notifications stack cleanly without blocking the UI.",
                    ));
                }
                if components
                    .button(Button::new("Success").variant(ButtonVariant::Primary))
                    .clicked()
                {
                    self.toast_stack.push(
                        Toast::new("Changes saved")
                            .description("Your layout tokens were published successfully.")
                            .intent(ToastIntent::Success),
                    );
                }
                if components
                    .button(Button::new("Error").variant(ButtonVariant::Ghost))
                    .clicked()
                {
                    self.toast_stack.push(
                        Toast::new("Build failed")
                            .description("A component export is missing a required icon mapping.")
                            .intent(ToastIntent::Destructive),
                    );
                }
                if components
                    .button(Button::new("Clear").variant(ButtonVariant::Ghost))
                    .clicked()
                {
                    self.toast_stack.clear();
                }
            });

        ui.add_space(10.0);
        let canvas_fill = if self.theme_mode.is_dark() {
            app_background(ui)
        } else {
            TOOLBAR_CANVAS_LIGHT_FILL
        };
        let card_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Border));
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let width = 640.0f32.min(ui.available_width());
            let _ = ui
                .components()
                .card(Card::new().fill(canvas_fill).stroke(card_stroke), |ui| {
                    ui.set_width(width);
                    let (host_rect, _) =
                        ui.allocate_exact_size(vec2(width - 24.0, 300.0), Sense::hover());
                    let _ = ui.scope_builder(egui::UiBuilder::new().max_rect(host_rect), |ui| {
                        let mut components = ui.components();
                        let _ = components.label(
                            Label::new("Toast viewport host")
                                .tone(LabelTone::Muted)
                                .size(SMALL_TEXT),
                        );
                        let subtitle = format!("Current placement: {placement_label}");
                        let _ = components.label(
                            Label::new(subtitle.as_str())
                                .tone(LabelTone::Muted)
                                .size(SMALL_TEXT),
                        );

                        components.toast_viewport(
                            &mut self.toast_stack,
                            ToastViewport::new(Id::new("component_showcase_toast_viewport"))
                                .placement(toast_placement_from_index(
                                    self.toast_placement_index.unwrap_or(8),
                                ))
                                .width(280.0),
                        );
                    });
                });
        });
    }

    fn render_toolbar_preview(&mut self, ui: &mut Ui) {
        let canvas_fill = if self.theme_mode.is_dark() {
            app_background(ui)
        } else {
            TOOLBAR_CANVAS_LIGHT_FILL
        };
        let card_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Border));

        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let width = TOOLBAR_PREVIEW_WIDTH.min(ui.available_width());
            let _ = ui
                .components()
                .card(Card::new().fill(canvas_fill).stroke(card_stroke), |ui| {
                    ui.set_width(width);
                    let (host_rect, _) =
                        ui.allocate_exact_size(vec2(width - 24.0, 220.0), Sense::hover());
                    let _ = ui.scope_builder(egui::UiBuilder::new().max_rect(host_rect), |ui| {
                        let _ = ui.components().toolbar(
                            Toolbar::new(Id::new("component_showcase_toolbar"))
                                .anchor(Align2::CENTER_TOP)
                                .offset(vec2(0.0, 10.0)),
                            |ui| draw_toolbar_contents(ui, &mut self.toolbar_color_index),
                        );
                    });
                });
        });
    }

    fn render_pagination_preview(&mut self, ui: &mut Ui) {
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let _ = ui.vertical_centered(|ui| {
                let _ = ui.components().pagination(
                    &mut self.pagination_page,
                    Pagination::new(
                        Id::new("component_showcase_pagination"),
                        PAGINATION_PAGE_COUNT,
                    ),
                );

                ui.add_space(14.0);
                let summary = format!("Page {} of {}", self.pagination_page, PAGINATION_PAGE_COUNT);
                let _ = ui.components().label(
                    Label::new(summary.as_str())
                        .tone(LabelTone::Muted)
                        .size(SMALL_TEXT),
                );
            });
        });
    }
}

fn draw_toolbar_contents(ui: &mut Ui, toolbar_color_index: &mut usize) {
    let selected_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Foreground));

    let _ = layout::row().gap(6.0).show(ui, |ui| {
        for (index, label) in TOOLBAR_ACTION_OPTIONS.iter().copied().enumerate() {
            let _ = ui
                .components()
                .button(Button::new(label).variant(ButtonVariant::Ghost));
            if index == 1 {
                let _ = ui.components().button(
                    Button::icon_only("crown")
                        .variant(ButtonVariant::Ghost)
                        .icon_size(13.0)
                        .icon_tint(Color32::from_rgb(216, 168, 83)),
                );
            }
        }

        draw_toolbar_divider(ui);

        for (index, fill) in TOOLBAR_SWATCHES.iter().copied().enumerate() {
            let stroke = if *toolbar_color_index == index {
                selected_stroke
            } else {
                Stroke::NONE
            };
            if ui
                .components()
                .button(Button::color_only(
                    Color::new(fill).size(16.0).stroke(stroke),
                ))
                .clicked()
            {
                *toolbar_color_index = index;
            }
        }

        draw_toolbar_divider(ui);
        let _ = ui
            .components()
            .button(Button::icon_only("square-menu").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::icon_only("rotate-ccw").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::icon_only("crop").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::new("Flip").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::icon_only("grid-3x3").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::new("Animate").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::new("Position").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::icon_only("paint-roller").variant(ButtonVariant::Ghost));
    });
}

fn draw_toolbar_divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(1.0, 16.0), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(1.0, theme::color(ui, ColorRole::Border)),
    );
}

fn draw_canva_brand_dropdown(ui: &mut Ui, width: f32) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let border = crate::ui::tokens::button_secondary_border(runtime);
    let fill = crate::ui::tokens::button_secondary_bg(runtime);
    let foreground = theme::color(ui, ColorRole::Foreground);
    let muted = text_muted(ui);
    let (rect, _) = ui.allocate_exact_size(vec2(width, 38.0), Sense::hover());

    ui.painter().rect(
        rect,
        egui::CornerRadius::same(radius_md(ui)),
        fill,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );

    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 20.0, rect.center().y),
        vec2(16.0, 16.0),
    );
    if let Some(image) = crate::icons::image(ui.ctx(), "badge-cent", 16.0) {
        let _ = image.tint(muted).paint_at(ui, icon_rect);
    }

    ui.painter().text(
        egui::pos2(rect.left() + 38.0, rect.center().y),
        Align2::LEFT_CENTER,
        "Brand Kit",
        crate::ui::typography::semibold_font(crate::ui::typography::BODY_SIZE),
        foreground,
    );

    if let Some(image) = crate::icons::image(ui.ctx(), "chevron-down", 16.0) {
        let _ = image.tint(muted).paint_at(
            ui,
            egui::Rect::from_center_size(
                egui::pos2(rect.right() - 18.0, rect.center().y),
                vec2(16.0, 16.0),
            ),
        );
    }
}

fn draw_canva_brand_nav_item(
    ui: &mut Ui,
    label: &str,
    selected: bool,
    width: f32,
) -> egui::Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let normal_fill = Color32::TRANSPARENT;
    let selected_fill = CANVA_BRAND_ACCENT.linear_multiply(0.28);
    let hover_fill = crate::ui::tokens::button_secondary_hover_bg(runtime).linear_multiply(0.7);
    let foreground = if selected {
        theme::color(ui, ColorRole::Foreground)
    } else {
        text_secondary(ui)
    };
    let (rect, response) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());

    let fill = if selected {
        selected_fill
    } else if response.hovered() {
        hover_fill
    } else {
        normal_fill
    };

    if selected || response.hovered() {
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(radius_md(ui)), fill);
    }

    ui.painter().text(
        egui::pos2(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        if selected {
            crate::ui::typography::semibold_font(crate::ui::typography::BODY_SIZE)
        } else {
            crate::ui::typography::body_font()
        },
        foreground,
    );

    response
}

fn draw_canva_brand_vertical_divider(ui: &mut Ui, height: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(1.0, height), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(1.0, theme::color(ui, ColorRole::Border)),
    );
}

fn render_canva_brand_detail(ui: &mut Ui, selected_index: usize) {
    match selected_index {
        0 => render_canva_brand_overview(ui),
        1 => render_canva_brand_guidelines(ui),
        2 => render_canva_brand_logo_grid(ui),
        3 => render_canva_brand_color_swatches(ui),
        4 => render_canva_brand_font_list(ui),
        5 => render_canva_brand_voice_cards(ui),
        6 | 7 => render_canva_brand_media_grid(ui, selected_index == 6),
        8 => render_canva_brand_icon_grid(ui),
        _ => render_canva_brand_chart_cards(ui),
    }
}

fn render_canva_brand_overview(ui: &mut Ui) {
    let hero_height = 192.0;
    let full_width = ui.available_width();
    let hero_rect = ui
        .allocate_exact_size(vec2(full_width, hero_height), Sense::hover())
        .0;
    paint_canva_brand_hero(ui, hero_rect);

    ui.add_space(12.0);
    let _ = ui.components().label(
        Label::new(
            "Apply your brand colors, fonts, logo, and much more effortlessly to every design",
        )
        .tone(LabelTone::Secondary)
        .weight(LabelWeight::Semibold),
    );
    ui.add_space(12.0);
    let _ = ui.components().button(
        Button::new("Try Business for 30 days")
            .variant(ButtonVariant::Primary)
            .leading_icon("crown")
            .icon_size(15.0)
            .min_size(vec2(full_width, 36.0))
            .label_weight(ButtonLabelWeight::Medium),
    );
    ui.add_space(10.0);
    let _ = ui.components().button(
        Button::new("Start with 3 free colors")
            .variant(ButtonVariant::Secondary)
            .leading_icon("plus")
            .icon_size(16.0)
            .min_size(vec2(full_width, 36.0))
            .label_weight(ButtonLabelWeight::Medium),
    );
}

fn paint_canva_brand_hero(ui: &Ui, rect: egui::Rect) {
    let painter = ui.painter();
    let radius = egui::CornerRadius::same(radius_md(ui));
    painter.rect_filled(rect, radius, Color32::from_rgb(92, 180, 206));

    painter.text(
        egui::pos2(rect.left() + 18.0, rect.center().y + 6.0),
        Align2::LEFT_CENTER,
        "Aa",
        crate::ui::typography::bold_font(74.0),
        Color32::from_rgb(226, 246, 247),
    );

    let swatch_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 18.0, rect.bottom() - 34.0),
        vec2(140.0, 18.0),
    );
    let swatch_width = swatch_rect.width() / 3.0;
    for (index, fill) in [
        Color32::from_rgb(25, 73, 208),
        Color32::from_rgb(189, 86, 230),
        Color32::from_rgb(246, 198, 78),
    ]
    .iter()
    .enumerate()
    {
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(
                    swatch_rect.left() + swatch_width * index as f32,
                    swatch_rect.top(),
                ),
                vec2(swatch_width, swatch_rect.height()),
            ),
            egui::CornerRadius::same(6),
            *fill,
        );
    }

    let portrait_center = egui::pos2(rect.right() - 64.0, rect.center().y - 2.0);
    painter.circle_filled(portrait_center, 42.0, CANVA_BRAND_ACCENT);
    painter.circle_filled(
        egui::pos2(portrait_center.x + 8.0, portrait_center.y + 2.0),
        28.0,
        Color32::from_rgb(166, 116, 63),
    );
    painter.rect_filled(
        egui::Rect::from_min_size(
            egui::pos2(rect.right() - 104.0, rect.bottom() - 62.0),
            vec2(68.0, 36.0),
        ),
        egui::CornerRadius::same(18),
        Color32::from_rgb(234, 173, 124),
    );

    let badge_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 112.0, rect.top() + 48.0),
        vec2(40.0, 22.0),
    );
    painter.rect_filled(
        badge_rect,
        egui::CornerRadius::same(10),
        CANVA_BRAND_ACCENT.linear_multiply(0.94),
    );
    painter.text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        "Andy",
        crate::ui::typography::semibold_font(10.0),
        Color32::WHITE,
    );
}

fn render_canva_brand_guidelines(ui: &mut Ui) {
    let _ = ui.components().label(
        Label::new("Guidelines")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    for (title, body) in [
        (
            "Logo spacing",
            "Keep a minimum clear-space area equal to the icon height.",
        ),
        (
            "Tone of voice",
            "Friendly, plainspoken, and direct across marketing and support copy.",
        ),
        (
            "Primary usage",
            "Use the soft orange accent only for primary actions and highlights.",
        ),
    ] {
        draw_canva_brand_info_card(ui, title, body);
        ui.add_space(10.0);
    }
}

fn draw_canva_brand_info_card(ui: &mut Ui, title: &str, body: &str) {
    let fill = theme::color(ui, ColorRole::Card);
    let _ = ui
        .components()
        .card(Card::new().padding(12, 12).fill(fill), |ui| {
            let _ = ui.components().label(
                Label::new(title)
                    .tone(LabelTone::Primary)
                    .weight(LabelWeight::Semibold),
            );
            ui.add_space(6.0);
            let _ = ui
                .components()
                .label(Label::new(body).tone(LabelTone::Secondary));
        });
}

fn render_canva_brand_logo_grid(ui: &mut Ui) {
    let _ = ui.components().label(
        Label::new("Logos")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    draw_canva_brand_asset_grid(ui, 2, 6, false, "logo");
}

fn render_canva_brand_color_swatches(ui: &mut Ui) {
    let border = Stroke::new(1.0, theme::color(ui, ColorRole::Border));
    let _ = ui.components().label(
        Label::new("Colors")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(12.0);
    for (name, hex, fill) in [
        ("Primary Purple", "#7C3AED", Color32::from_rgb(124, 58, 237)),
        ("Ocean Blue", "#3B82F6", Color32::from_rgb(59, 130, 246)),
        ("Warm Gold", "#F5C04A", Color32::from_rgb(245, 192, 74)),
        ("Ink", "#111827", Color32::from_rgb(17, 24, 39)),
    ] {
        let _ = layout::row().gap(10.0).show(ui, |ui| {
            let _ = ui
                .components()
                .color(Color::new(fill).size(28.0).stroke(border));
            let _ = layout::column().gap(2.0).show(ui, |ui| {
                let _ = ui.components().label(
                    Label::new(name)
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold),
                );
                let _ = ui
                    .components()
                    .label(Label::new(hex).tone(LabelTone::Muted));
            });
        });
        ui.add_space(10.0);
    }
}

fn render_canva_brand_font_list(ui: &mut Ui) {
    let _ = ui.components().label(
        Label::new("Fonts")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    for (title, sample) in [
        ("Display Serif", "The quick brown fox"),
        ("UI Sans", "Design systems that scale"),
        ("Editorial Italic", "Bold ideas, clean layouts"),
    ] {
        draw_canva_brand_info_card(ui, title, sample);
        ui.add_space(10.0);
    }
}

fn render_canva_brand_voice_cards(ui: &mut Ui) {
    let _ = ui.components().label(
        Label::new("Brand voice")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    for (title, body) in [
        (
            "Confident",
            "Clear calls to action and high-clarity product language.",
        ),
        ("Warm", "Conversational phrasing without overexplaining."),
        (
            "Modern",
            "Short sentences, active verbs, and clean visual hierarchy.",
        ),
    ] {
        draw_canva_brand_info_card(ui, title, body);
        ui.add_space(10.0);
    }
}

fn render_canva_brand_media_grid(ui: &mut Ui, photo_mode: bool) {
    let title = if photo_mode { "Photos" } else { "Graphics" };
    let prefix = if photo_mode { "photo" } else { "graphic" };
    let _ = ui.components().label(
        Label::new(title)
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    draw_canva_brand_asset_grid(ui, 2, 6, true, prefix);
}

fn render_canva_brand_icon_grid(ui: &mut Ui) {
    let _ = ui.components().label(
        Label::new("Icons")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    draw_canva_brand_asset_grid(ui, 3, 9, false, "icon");
}

fn render_canva_brand_chart_cards(ui: &mut Ui) {
    let _ = ui.components().label(
        Label::new("Charts")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    draw_canva_brand_asset_grid(ui, 2, 4, false, "chart");
}

fn draw_canva_brand_asset_grid(ui: &mut Ui, columns: usize, count: usize, tall: bool, kind: &str) {
    let tile_height = if tall { 120.0 } else { 92.0 };
    let _ = layout::tile_grid()
        .columns(columns)
        .gap(10.0)
        .tile_height(tile_height)
        .show(ui, count, |ui, index, rect| {
            paint_canva_brand_asset_tile(ui, rect, kind, index);
        });
}

fn paint_canva_brand_asset_tile(ui: &Ui, rect: egui::Rect, kind: &str, index: usize) {
    let painter = ui.painter();
    let radius = egui::CornerRadius::same(radius_md(ui));
    let border = Stroke::new(1.0, theme::color(ui, ColorRole::Border));

    let (fill, accent) = match (kind, index % 5) {
        ("logo", 0) => (
            Color32::from_rgb(38, 49, 79),
            Color32::from_rgb(124, 173, 255),
        ),
        ("logo", 1) => (
            Color32::from_rgb(243, 244, 246),
            Color32::from_rgb(79, 70, 229),
        ),
        ("photo", 0) => (
            Color32::from_rgb(200, 209, 223),
            Color32::from_rgb(102, 126, 173),
        ),
        ("photo", 1) => (
            Color32::from_rgb(184, 159, 122),
            Color32::from_rgb(237, 218, 185),
        ),
        ("graphic", 0) => (
            Color32::from_rgb(114, 84, 196),
            Color32::from_rgb(255, 202, 58),
        ),
        ("graphic", 1) => (
            Color32::from_rgb(61, 166, 177),
            Color32::from_rgb(255, 255, 255),
        ),
        ("icon", 0) => (
            Color32::from_rgb(32, 38, 56),
            Color32::from_rgb(182, 197, 255),
        ),
        ("icon", 1) => (
            Color32::from_rgb(49, 58, 91),
            Color32::from_rgb(255, 205, 86),
        ),
        ("chart", 0) => (
            Color32::from_rgb(35, 46, 66),
            Color32::from_rgb(114, 152, 255),
        ),
        ("chart", 1) => (
            Color32::from_rgb(70, 54, 105),
            Color32::from_rgb(222, 133, 255),
        ),
        _ => (
            Color32::from_rgb(74, 80, 96),
            Color32::from_rgb(212, 219, 233),
        ),
    };

    painter.rect_filled(rect, radius, fill);
    match kind {
        "logo" => {
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Aa",
                crate::ui::typography::bold_font(36.0),
                accent,
            );
        }
        "icon" => {
            painter.circle_filled(rect.center(), rect.width() * 0.22, accent);
            painter.line_segment(
                [
                    egui::pos2(rect.center().x, rect.top() + rect.height() * 0.18),
                    egui::pos2(rect.center().x, rect.bottom() - rect.height() * 0.18),
                ],
                Stroke::new(2.0, fill.linear_multiply(0.6)),
            );
        }
        "chart" => {
            for step in 0..4 {
                let bar_width = rect.width() * 0.12;
                let x = rect.left() + rect.width() * (0.18 + step as f32 * 0.18);
                let height = rect.height() * (0.2 + step as f32 * 0.12);
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(x, rect.bottom() - height - 12.0),
                        vec2(bar_width, height),
                    ),
                    egui::CornerRadius::same(4),
                    accent,
                );
            }
        }
        _ => {
            painter.rect_filled(
                rect.shrink2(vec2(rect.width() * 0.18, rect.height() * 0.18)),
                egui::CornerRadius::same(8),
                accent.linear_multiply(0.85),
            );
            painter.circle_filled(
                egui::pos2(
                    rect.right() - rect.width() * 0.14,
                    rect.top() + rect.height() * 0.18,
                ),
                rect.width() * 0.08,
                Color32::from_rgba_unmultiplied(255, 255, 255, 110),
            );
        }
    }
    painter.rect_stroke(rect, radius, border, egui::StrokeKind::Inside);
}

fn draw_canva_edit_tool_row(ui: &mut Ui, selected_index: &mut usize) {
    ScrollArea::horizontal()
        .id_salt("component_showcase_canva_edit_tool_row")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let _ = layout::row().gap(8.0).show(ui, |ui| {
                for (index, option) in CANVA_EDIT_SELECTION_OPTIONS.iter().enumerate() {
                    let selected = *selected_index == index;
                    if draw_canva_edit_tool_chip(ui, option, selected).clicked() {
                        *selected_index = index;
                    }
                }
            });
        });
}

fn draw_canva_edit_tool_chip(
    ui: &mut Ui,
    option: &CanvaEditChip<'_>,
    selected: bool,
) -> egui::Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let muted_foreground = crate::ui::tokens::text_muted(runtime);
    let border = if selected {
        CANVA_EDIT_ACCENT
    } else {
        crate::ui::tokens::button_secondary_border(runtime)
    };
    let fill = if selected {
        CANVA_EDIT_ACCENT.linear_multiply(0.22)
    } else {
        crate::ui::tokens::button_secondary_bg(runtime)
    };
    let foreground = if selected {
        theme::color(ui, ColorRole::Foreground)
    } else {
        muted_foreground.lerp_to_gamma(theme::color(ui, ColorRole::Foreground), 0.55)
    };
    let width = (option.label.len() as f32 * 7.4 + 44.0).max(76.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, 40.0), Sense::click());

    let fill = if response.is_pointer_button_down_on() {
        fill.linear_multiply(0.92)
    } else if response.hovered() && !selected {
        fill.linear_multiply(1.08)
    } else {
        fill
    };

    ui.painter().rect(
        rect,
        egui::CornerRadius::same(radius_md(ui)),
        fill,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );

    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 18.0, rect.center().y),
        vec2(18.0, 18.0),
    );
    if let Some(image) = crate::icons::image(ui.ctx(), option.icon, 18.0) {
        let _ = image.tint(foreground).paint_at(ui, icon_rect);
    }

    ui.painter().text(
        egui::pos2(rect.left() + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        option.label,
        crate::ui::typography::semibold_font(crate::ui::typography::BODY_SIZE),
        foreground,
    );

    response
}

fn draw_canva_edit_navigation_row(ui: &mut Ui, icon: &str, label: &str) {
    let _ = show_section_link_row(ui, icon, label);
}

fn draw_canva_edit_rail(
    ui: &mut Ui,
    id: Id,
    items: &[CanvaEditRailItem<'_>],
    mut selected_index: Option<&mut usize>,
    text_below: bool,
) {
    ScrollArea::horizontal()
        .id_salt(id)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let _ = layout::row().gap(12.0).show(ui, |ui| {
                for (index, item) in items.iter().enumerate() {
                    let selected = selected_index
                        .as_ref()
                        .is_some_and(|current| **current == index);
                    let response = draw_canva_edit_rail_item(ui, item, selected, text_below, index);
                    if response.clicked() {
                        if let Some(current) = selected_index.as_deref_mut() {
                            *current = index;
                        }
                    }
                }
            });
        });
}

fn draw_canva_edit_rail_item(
    ui: &mut Ui,
    item: &CanvaEditRailItem<'_>,
    selected: bool,
    text_below: bool,
    pattern_index: usize,
) -> egui::Response {
    let tile_size = if text_below { 92.0 } else { 88.0 };
    let total_height = if text_below { 116.0 } else { 112.0 };
    let sense = if selected || text_below {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(tile_size, total_height), sense);
    let tile_rect = egui::Rect::from_min_size(rect.min, vec2(tile_size, tile_size));

    paint_canva_edit_tile(ui, tile_rect, item, pattern_index, selected);

    let label_color = if selected {
        theme::color(ui, ColorRole::Foreground)
    } else {
        text_secondary(ui)
    };
    ui.painter().text(
        egui::pos2(tile_rect.center().x, tile_rect.bottom() + 16.0),
        Align2::CENTER_CENTER,
        item.label,
        crate::ui::typography::semibold_font(SMALL_TEXT),
        label_color,
    );

    response
}

fn paint_canva_edit_tile(
    ui: &Ui,
    rect: egui::Rect,
    item: &CanvaEditRailItem<'_>,
    pattern_index: usize,
    selected: bool,
) {
    let radius = egui::CornerRadius::same(radius_md(ui));
    let border = if selected {
        Stroke::new(2.0, CANVA_EDIT_ACCENT)
    } else {
        Stroke::new(1.0, theme::color(ui, ColorRole::Border))
    };
    let painter = ui.painter();

    painter.rect_filled(rect, radius, item.fill);

    match pattern_index % 5 {
        0 => {
            painter.circle_filled(
                egui::pos2(rect.center().x, rect.center().y + rect.height() * 0.08),
                rect.width() * 0.28,
                item.accent,
            );
            painter.line_segment(
                [
                    egui::pos2(rect.center().x, rect.top() + rect.height() * 0.12),
                    egui::pos2(rect.center().x, rect.bottom() - rect.height() * 0.14),
                ],
                Stroke::new(2.0, Color32::from_rgba_unmultiplied(20, 20, 20, 64)),
            );
        }
        1 => {
            painter.rect_filled(
                rect.shrink2(vec2(rect.width() * 0.17, rect.height() * 0.17)),
                radius,
                item.accent,
            );
            painter.circle_filled(
                egui::pos2(
                    rect.right() - rect.width() * 0.2,
                    rect.top() + rect.height() * 0.22,
                ),
                rect.width() * 0.1,
                Color32::from_rgba_unmultiplied(255, 255, 255, 128),
            );
        }
        2 => {
            painter.circle_filled(
                egui::pos2(rect.center().x, rect.center().y),
                rect.width() * 0.34,
                item.accent,
            );
            painter.circle_filled(
                egui::pos2(rect.center().x, rect.center().y),
                rect.width() * 0.15,
                Color32::from_rgba_unmultiplied(255, 255, 255, 120),
            );
        }
        3 => {
            for step in 0..5 {
                let x = rect.left() + rect.width() * step as f32 / 4.0;
                painter.line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    Stroke::new(8.0, item.accent.linear_multiply(0.7)),
                );
            }
        }
        _ => {
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.left(), rect.bottom() - rect.height() * 0.38),
                    vec2(rect.width(), rect.height() * 0.38),
                ),
                radius,
                item.accent,
            );
            painter.circle_filled(
                egui::pos2(rect.center().x, rect.top() + rect.height() * 0.28),
                rect.width() * 0.18,
                Color32::from_rgba_unmultiplied(255, 255, 255, 110),
            );
        }
    }

    if let Some(badge) = item.badge {
        let badge_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 8.0, rect.bottom() - 22.0),
            vec2(34.0, 16.0),
        );
        painter.rect_filled(badge_rect, egui::CornerRadius::same(8), CANVA_EDIT_ACCENT);
        painter.text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            badge,
            crate::ui::typography::semibold_font(10.0),
            Color32::WHITE,
        );
    }

    painter.rect_stroke(rect, radius, border, egui::StrokeKind::Inside);
}

fn draw_canva_position_button_grid(ui: &mut Ui, actions: &[(&str, &str, bool)]) {
    let _ = layout::tile_grid()
        .columns(2)
        .gap(8.0)
        .tile_height(40.0)
        .show(ui, actions.len(), |ui, index, rect| {
            let (label, icon, enabled) = actions[index];
            let _ = draw_canva_position_action_button(ui, rect, index, label, icon, enabled);
        });
}

fn draw_canva_position_action_button(
    ui: &mut Ui,
    rect: egui::Rect,
    index: usize,
    label: &str,
    icon: &str,
    enabled: bool,
) -> egui::Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let response = ui.interact(
        rect,
        ui.id().with(("canva_position_action_button", index, label)),
        sense,
    );

    let is_pressed = enabled && response.is_pointer_button_down_on();
    let is_hovered = enabled && response.hovered();
    let fill = if is_pressed {
        crate::ui::tokens::button_secondary_active_bg(runtime)
    } else if is_hovered {
        crate::ui::tokens::button_secondary_hover_bg(runtime)
    } else {
        crate::ui::tokens::button_secondary_bg(runtime)
    };
    let stroke_color = if is_pressed {
        crate::ui::tokens::button_secondary_active_border(runtime)
    } else if is_hovered {
        crate::ui::tokens::button_secondary_hover_border(runtime)
    } else {
        crate::ui::tokens::button_secondary_border(runtime)
    };
    let foreground = if enabled {
        crate::ui::tokens::text_primary(runtime)
    } else {
        crate::ui::tokens::text_muted(runtime)
    };

    ui.painter().rect(
        rect,
        egui::CornerRadius::same(radius_md(ui)),
        fill,
        Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );

    let content_left = rect.left() + 12.0;
    let icon_size = 18.0;
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(content_left + (icon_size * 0.5), rect.center().y),
        vec2(icon_size, icon_size),
    );
    if let Some(image) = crate::icons::image(ui.ctx(), icon, icon_size) {
        let _ = image.tint(foreground).paint_at(ui, icon_rect);
    }

    ui.painter().text(
        egui::pos2(content_left + icon_size + 8.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        crate::ui::typography::semibold_font(crate::ui::typography::LABEL_SIZE),
        foreground,
    );

    response
}

fn draw_canva_background_placeholder_grid(ui: &mut Ui) {
    let _ = layout::tile_grid()
        .min_tile_width(84.0)
        .gap(8.0)
        .show(ui, 15, |ui, index, rect| {
            paint_canva_background_placeholder(ui, rect, index);
        });
}

fn paint_canva_background_placeholder(ui: &Ui, rect: egui::Rect, index: usize) {
    let radius = egui::CornerRadius::ZERO;
    let painter = ui.painter();
    let border = Stroke::new(1.0, theme::color(ui, ColorRole::Border));

    match index {
        0 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(241, 240, 236));
            painter.rect_filled(
                egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.center().y)),
                radius,
                Color32::from_rgb(252, 252, 250),
            );
        }
        1 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(228, 222, 208));
            for step in 1..6 {
                let y = rect.top() + (rect.height() * step as f32 / 6.0);
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)),
                );
            }
        }
        2 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(118, 90, 109));
            painter.rect_filled(
                egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.center().y)),
                radius,
                Color32::from_rgb(222, 206, 202),
            );
        }
        3 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(73, 159, 183));
            for step in 0..5 {
                let x = rect.left() + rect.width() * step as f32 / 4.0;
                painter.circle_filled(
                    egui::pos2(x, rect.top() + rect.height() * 0.4),
                    rect.width() * 0.18,
                    Color32::from_rgba_unmultiplied(173, 230, 237, 48),
                );
            }
        }
        4 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(210, 187, 163));
            painter.circle_stroke(
                egui::pos2(
                    rect.center().x + rect.width() * 0.16,
                    rect.center().y + rect.height() * 0.1,
                ),
                rect.width() * 0.34,
                Stroke::new(3.0, Color32::from_rgb(188, 105, 52)),
            );
            painter.line_segment(
                [
                    egui::pos2(rect.center().x - rect.width() * 0.04, rect.top()),
                    egui::pos2(rect.center().x + rect.width() * 0.2, rect.bottom()),
                ],
                Stroke::new(2.0, Color32::from_rgb(76, 45, 26)),
            );
        }
        5 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(224, 198, 129));
            for center in [
                egui::pos2(
                    rect.left() + rect.width() * 0.25,
                    rect.top() + rect.height() * 0.34,
                ),
                egui::pos2(
                    rect.left() + rect.width() * 0.62,
                    rect.top() + rect.height() * 0.52,
                ),
                egui::pos2(
                    rect.left() + rect.width() * 0.46,
                    rect.top() + rect.height() * 0.78,
                ),
            ] {
                painter.circle_filled(
                    center,
                    rect.width() * 0.18,
                    Color32::from_rgba_unmultiplied(255, 243, 195, 72),
                );
            }
        }
        6 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(209, 101, 154));
            for center in [
                egui::pos2(
                    rect.left() + rect.width() * 0.18,
                    rect.top() + rect.height() * 0.2,
                ),
                egui::pos2(
                    rect.left() + rect.width() * 0.62,
                    rect.top() + rect.height() * 0.38,
                ),
                egui::pos2(
                    rect.left() + rect.width() * 0.78,
                    rect.top() + rect.height() * 0.68,
                ),
            ] {
                painter.circle_filled(
                    center,
                    rect.width() * 0.16,
                    Color32::from_rgba_unmultiplied(255, 210, 230, 64),
                );
            }
        }
        7 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(130, 171, 86));
            painter.rect_stroke(
                rect.shrink(rect.width() * 0.18),
                radius,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(248, 247, 220, 120)),
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [rect.center_top(), rect.center_bottom()],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(248, 247, 220, 100)),
            );
        }
        8 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(191, 203, 219));
            for step in 0..4 {
                let y = rect.top() + rect.height() * (0.58 + (step as f32 * 0.08));
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 88)),
                );
            }
        }
        9 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(239, 238, 232));
            for step in 0..8 {
                let y = rect.top() + rect.height() * step as f32 / 8.0;
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 32)),
                );
            }
        }
        10 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(13, 29, 92));
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.top() + rect.height() * 0.68),
                    rect.max,
                ),
                radius,
                Color32::from_rgb(16, 20, 40),
            );
            painter.circle_filled(
                egui::pos2(rect.center().x, rect.top() + rect.height() * 0.36),
                rect.width() * 0.14,
                Color32::from_rgb(245, 213, 136),
            );
        }
        11 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(198, 177, 151));
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.top() + rect.height() * 0.72),
                    rect.max,
                ),
                radius,
                Color32::from_rgb(143, 96, 71),
            );
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(
                        rect.left() + rect.width() * 0.12,
                        rect.top() + rect.height() * 0.62,
                    ),
                    vec2(rect.width() * 0.36, rect.height() * 0.12),
                ),
                radius,
                Color32::from_rgb(112, 73, 52),
            );
        }
        12 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(166, 153, 135));
            for step in 1..6 {
                let x = rect.left() + (rect.width() * step as f32 / 6.0);
                painter.line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    Stroke::new(2.0, Color32::from_rgba_unmultiplied(84, 69, 56, 72)),
                );
            }
        }
        13 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(222, 210, 195));
            for step in 0..6 {
                let y = rect.top() + rect.height() * (0.16 + step as f32 * 0.12);
                painter.line_segment(
                    [
                        egui::pos2(rect.left() + rect.width() * 0.1, y),
                        egui::pos2(rect.right() - rect.width() * 0.1, y - rect.height() * 0.05),
                    ],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(120, 104, 96, 80)),
                );
            }
        }
        _ => {
            painter.rect_filled(rect, radius, Color32::from_rgb(203, 158, 126));
            for step in 0..5 {
                let x = rect.left() + rect.width() * step as f32 / 5.0;
                painter.line_segment(
                    [
                        egui::pos2(x, rect.top() + rect.height() * 0.08),
                        egui::pos2(
                            x + rect.width() * 0.08,
                            rect.bottom() - rect.height() * 0.08,
                        ),
                    ],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(247, 222, 205, 72)),
                );
            }
        }
    }

    painter.rect_stroke(rect, radius, border, egui::StrokeKind::Inside);
}

fn draw_canva_number_field(ui: &mut Ui, label: &str, value: &mut f32, input: NumberInput) {
    let _ = layout::column().gap(6.0).show(ui, |ui| {
        let _ = ui.components().label(
            Label::new(label)
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT)
                .weight(LabelWeight::Semibold),
        );
        let _ = ui.components().number_input(value, input);
    });
}

fn draw_canva_ratio_field(ui: &mut Ui, width: f32, locked: &mut bool) {
    let _ = layout::column().gap(6.0).show(ui, |ui| {
        let _ = ui.components().label(
            Label::new("Ratio")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT)
                .weight(LabelWeight::Semibold),
        );
        if ui
            .components()
            .button(
                Button::icon_only(if *locked { "lock" } else { "lock-open" })
                    .variant(ButtonVariant::Secondary)
                    .icon_size(18.0)
                    .min_size(vec2(width, 30.0)),
            )
            .clicked()
        {
            *locked = !*locked;
        }
    });
}

fn draw_image_tile_metadata_row(ui: &mut Ui, text: &str) {
    let _ = layout::row().gap(6.0).show(ui, |ui| {
        let mut components = ui.components();
        let _ = components.icon(Icon::new("globe").size(12.0).tint(IMAGE_TILE_META_ACCENT));
        let _ = components.label(Label::new("•").tone(LabelTone::Muted));
        let _ = components.label(Label::new(text).tone(LabelTone::Muted).size(SMALL_TEXT));
    });
}

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
    let mut definitions = catalog::component_definitions()
        .filter(move |definition| showcase_section(definition.kind) == section)
        .collect::<Vec<_>>();
    definitions.sort_unstable_by(|left, right| left.label.cmp(right.label));
    definitions.into_iter()
}

fn catalog_component_definition(kind: ComponentKind) -> &'static ComponentDefinition {
    catalog::component_definitions()
        .find(|definition| definition.kind == kind)
        .expect("missing catalog component definition")
}

fn showcase_section(kind: ComponentKind) -> ShowcaseSection {
    match kind {
        ComponentKind::CanvaBackgrounds => ShowcaseSection::Canva,
        ComponentKind::CanvaBrandKit => ShowcaseSection::Canva,
        ComponentKind::CanvaEditImage => ShowcaseSection::Canva,
        ComponentKind::CanvaPosition => ShowcaseSection::Canva,
        ComponentKind::MenuBar
        | ComponentKind::Toolbar
        | ComponentKind::ImageTile
        | ComponentKind::Sidebar
        | ComponentKind::Toast => ShowcaseSection::Examples,
        _ => match catalog_component_definition(kind).group {
            ComponentGroup::Primitive => ShowcaseSection::PrimaryPrimitive,
            ComponentGroup::Composed => ShowcaseSection::DerivedComposed,
        },
    }
}

fn preview_surface_width(kind: ComponentKind, available_width: f32) -> f32 {
    match kind {
        ComponentKind::CanvaBackgrounds => available_width.min(460.0),
        ComponentKind::CanvaBrandKit => available_width.min(640.0),
        ComponentKind::CanvaEditImage => available_width.min(420.0),
        ComponentKind::CanvaPosition => available_width.min(440.0),
        ComponentKind::Toolbar => available_width.min(920.0),
        ComponentKind::MenuBar => available_width.min(560.0),
        ComponentKind::Sidebar => available_width.min(820.0),
        ComponentKind::Toast => available_width.min(760.0),
        _ => available_width.clamp(280.0, 480.0),
    }
}

fn showcase_description(kind: ComponentKind) -> &'static str {
    match kind {
        ComponentKind::Label => "Text styles and tones.",
        ComponentKind::Color => "Circular solid color swatches.",
        ComponentKind::Image => "PNG-backed raster image rendering.",
        ComponentKind::Icon => "Lucide icon rendering.",
        ComponentKind::Kbd => "Keyboard keycaps and shortcuts.",
        ComponentKind::Input => "Single-line text input.",
        ComponentKind::Field => "Label + input + helper text.",
        ComponentKind::Button => "Text, icon, and link button variants.",
        ComponentKind::ButtonGroup => "Attached action button group.",
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
        ComponentKind::Switch => "Toggle control.",
        ComponentKind::Slider => "Range input.",
        ComponentKind::NumberInput => "Numeric entry with drag axis support.",
        ComponentKind::Select => "Single-choice selection menu.",
        ComponentKind::Tabs => "Inline, segmented, stacked, and rail tabs.",
        ComponentKind::Separator => "Lightweight content divider.",
        ComponentKind::Card => "Framed content surface.",
        ComponentKind::Progress => "Determinate progress indicator.",
        ComponentKind::Radio => "Single-choice control for mutually exclusive selections.",
        ComponentKind::RadioGroup => "Vertical radio list with optional descriptions.",
        ComponentKind::Popover => "Click-triggered interactive popup surface.",
        ComponentKind::Tooltip => "Hover-triggered helper content.",
        ComponentKind::DropdownMenu => "Actions, shortcuts, separators, and nested menus.",
        ComponentKind::Collapsible => "Expandable content section.",
        ComponentKind::AudioPlayback => "Playback row with optional trailing actions.",
        ComponentKind::Combobox => "Filterable text-backed option picker.",
        ComponentKind::Command => "Searchable command list with preview mode.",
        ComponentKind::Dialogue => "Modal confirmation flow.",
        ComponentKind::ImageTile => "Media tile with body and playback states.",
        ComponentKind::MenuBar => "Desktop-style menu bar surface.",
        ComponentKind::Sidebar => "Overlay sidebar previewed inside a host surface.",
        ComponentKind::Skeleton => "Animated placeholder blocks for loading layouts.",
        ComponentKind::Spinner => "Indeterminate loading spinner.",
        ComponentKind::Toast => "Stacked toast notifications with configurable placement.",
        ComponentKind::Toolbar => "Floating tool cluster anchored in a canvas.",
        ComponentKind::Pagination => "Previous/next pager with page numbers.",
    }
}

fn app_background(ui: &Ui) -> Color32 {
    theme::color(ui, ColorRole::Background)
}

fn showcase_header_fill(runtime: theme::ThemeRuntime) -> Color32 {
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

fn theme_mode_index(mode: ThemeMode) -> usize {
    match mode {
        ThemeMode::Light => 0,
        ThemeMode::Dark => 1,
    }
}

fn theme_mode_from_index(index: usize) -> ThemeMode {
    match index {
        0 => ThemeMode::Light,
        _ => ThemeMode::Dark,
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
    state.canva_position_tab_index = state
        .canva_position_tab_index
        .min(CANVA_POSITION_TAB_OPTIONS.len().saturating_sub(1));
    state.image_rotation_degrees = state.image_rotation_degrees.clamp(-180.0, 180.0);
    state.tab_index = state.tab_index.min(TAB_OPTIONS.len().saturating_sub(1));
    state.segmented_tab_index = state
        .segmented_tab_index
        .min(TAB_OPTIONS.len().saturating_sub(1));
    state.stacked_tab_index = state
        .stacked_tab_index
        .min(STACKED_TAB_OPTIONS.len().saturating_sub(1));
    state.rail_tab_index = state
        .rail_tab_index
        .min(RAIL_TAB_OPTIONS.len().saturating_sub(1));
    state.pagination_page = state.pagination_page.clamp(1, PAGINATION_PAGE_COUNT);
    state.slider_value = state.slider_value.clamp(0.0, 100.0);
    state.number_x_value = state.number_x_value.clamp(0.0, 100.0);
    state.number_y_value = state.number_y_value.clamp(0.0, 100.0);
    state.canva_width_value = state.canva_width_value.clamp(1.0, 4_000.0);
    state.canva_height_value = state.canva_height_value.clamp(1.0, 4_000.0);
    state.canva_x_value = state.canva_x_value.clamp(-4_000.0, 4_000.0);
    state.canva_y_value = state.canva_y_value.clamp(-4_000.0, 4_000.0);
    state.canva_rotate_value = state.canva_rotate_value.clamp(-360.0, 360.0);
    state.progress_value = state.progress_value.clamp(0.0, 1.0);
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
        .menu_bar_action
        .is_some_and(|id| id >= MENU_BAR_ACTION_LABELS.len())
    {
        state.menu_bar_action = None;
    }
    state.combobox_index = state
        .combobox_index
        .min(COMBOBOX_OPTIONS.len().saturating_sub(1));
}

#[cfg(test)]
mod tests {
    use super::{
        configure_snapshot, install_context, render_snapshot_surface, sidebar_preview_toggle_rect,
        update, ShowcaseApp,
    };
    use crate::{ComponentKind, ThemeMode};
    use egui::{pos2, vec2, CentralPanel, Context, RawInput, Rect};

    #[test]
    fn snapshot_surface_renders_representative_components_without_panic() {
        let context = Context::default();
        install_context(&context);

        for component in [
            ComponentKind::CanvaBackgrounds,
            ComponentKind::CanvaBrandKit,
            ComponentKind::CanvaEditImage,
            ComponentKind::CanvaPosition,
            ComponentKind::Sidebar,
            ComponentKind::Toast,
        ] {
            let mut app = ShowcaseApp::default();
            configure_snapshot(&mut app, component, ThemeMode::Dark);

            let _ = context.run(RawInput::default(), |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    render_snapshot_surface(&mut app, ui);
                });
            });
        }
    }

    #[test]
    fn showcase_update_renders_narrow_layout_without_panic() {
        let context = Context::default();
        install_context(&context);
        let mut app = ShowcaseApp::default();
        app.selected_component = ComponentKind::Sidebar;

        let _ = context.run(
            RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(720.0, 540.0))),
                ..Default::default()
            },
            |ctx| update(&mut app, ctx),
        );
    }

    #[test]
    fn sidebar_preview_toggle_rect_tracks_host_edges() {
        let host_rect = Rect::from_min_size(pos2(40.0, 24.0), vec2(720.0, 360.0));
        let sidebar_width = 240.0;

        let left_closed = sidebar_preview_toggle_rect(host_rect, 0, 0.0, sidebar_width);
        let left_open = sidebar_preview_toggle_rect(host_rect, 0, 1.0, sidebar_width);
        let right_closed = sidebar_preview_toggle_rect(host_rect, 1, 0.0, sidebar_width);
        let right_open = sidebar_preview_toggle_rect(host_rect, 1, 1.0, sidebar_width);

        assert_eq!(left_closed.left(), host_rect.left() + 8.0);
        assert_eq!(left_open.left(), host_rect.left() + sidebar_width - 36.0);
        assert_eq!(right_closed.right(), host_rect.right() - 8.0);
        assert_eq!(right_open.right(), host_rect.right() - sidebar_width + 36.0);
        assert_eq!(left_closed.top(), host_rect.top() + 8.0);
        assert_eq!(right_closed.top(), host_rect.top() + 8.0);
    }
}
