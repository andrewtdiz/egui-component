use egui::{
    vec2, Align2, CentralPanel, Color32, Context, CursorIcon, Id, Layout, ScrollArea, Sense,
    SidePanel, Stroke, TopBottomPanel, Ui, ViewportBuilder,
};
use egui_component::catalog::{self, ComponentDefinition, ComponentGroup, ComponentKind};
use egui_component::layout;
use egui_component::prelude::*;
use egui_component::theme::{self, BaseColor, ColorRole, RadiusRole, ThemeMode, ThemeSpec};

const SHOWCASE_IMAGE_BYTES: &[u8] = include_bytes!("../assets/images/showcase-image.png");

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

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("egui-component Showcase")
            .with_inner_size([1280.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui-component Showcase",
        options,
        Box::new(|creation_context| {
            theme::install(
                &creation_context.egui_ctx,
                ThemeSpec::preset(BaseColor::Neutral),
                ThemeMode::Dark,
            );
            Ok(Box::<ShowcaseApp>::default())
        }),
    )
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ShowcaseSection {
    PrimaryPrimitive,
    DerivedComposed,
    Examples,
}

impl ShowcaseSection {
    fn title(self) -> &'static str {
        match self {
            Self::PrimaryPrimitive => "Primary / Primitive",
            Self::DerivedComposed => "Derived / Composed",
            Self::Examples => "Examples",
        }
    }
}

struct ShowcaseApp {
    selected_component: ComponentKind,
    theme_mode: ThemeMode,
    input_value: String,
    field_value: String,
    checkbox_value: bool,
    switch_value: bool,
    small_switch_value: bool,
    toolbar_color_index: usize,
    image_rotation_degrees: f32,
    slider_value: f32,
    number_x_value: f32,
    number_y_value: f32,
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
            input_value: "Player_Robot".to_owned(),
            field_value: "M_Robot_Body".to_owned(),
            checkbox_value: true,
            switch_value: true,
            small_switch_value: false,
            toolbar_color_index: 0,
            image_rotation_degrees: 18.0,
            slider_value: 62.0,
            number_x_value: 42.0,
            number_y_value: 16.0,
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

impl eframe::App for ShowcaseApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        clamp_state(self);
        theme::set_theme(ctx, ThemeSpec::preset(BaseColor::Neutral));
        theme::set_mode(ctx, self.theme_mode);

        TopBottomPanel::top("component_showcase_topbar")
            .resizable(false)
            .show(ctx, |ui| self.render_topbar(ui));

        SidePanel::left("component_showcase_sidebar")
            .resizable(true)
            .default_width(SIDEBAR_WIDTH)
            .min_width(200.0)
            .max_width(320.0)
            .show(ctx, |ui| self.render_sidebar(ui));

        CentralPanel::default().show(ctx, |ui| self.render_center(ui));
    }
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
            let _ = components.label(
                Label::new("Old showcase layout, updated for the current component set.")
                    .tone(LabelTone::Muted)
                    .size(SMALL_TEXT),
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
            let _ = layout::row().gap(12.0).show(ui, |ui| {
                {
                    let mut components = ui.components();
                    let _ = components.label(
                        Label::new("egui-component Showcase")
                            .weight(LabelWeight::Semibold)
                            .tone(LabelTone::Primary),
                    );
                }
                let _ = layout::spacer().show(ui);
                {
                    let mut components = ui.components();
                    let _ = components.label(
                        Label::new("Theme mode")
                            .tone(LabelTone::Muted)
                            .size(SMALL_TEXT),
                    );

                    let mut selected_mode = theme_mode_index(self.theme_mode);
                    components.segmented_tabs(
                        Id::new("component_showcase_theme_mode"),
                        &mut selected_mode,
                        &THEME_MODE_OPTIONS,
                    );
                    self.theme_mode = theme_mode_from_index(selected_mode);
                }
            });
        });
        let _ = ui.components().separator();
    }

    fn render_center(&mut self, ui: &mut Ui) {
        let definition = catalog_component_definition(self.selected_component);
        let width = match self.selected_component {
            ComponentKind::Toolbar => ui.available_width().min(920.0),
            ComponentKind::MenuBar => ui.available_width().min(560.0),
            ComponentKind::Sidebar => ui.available_width().min(820.0),
            ComponentKind::Toast => ui.available_width().min(760.0),
            _ => ui.available_width().clamp(280.0, 480.0),
        };

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
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
                });
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
                        "  Publishing build"
                    } else {
                        "Publish build"
                    })
                    .variant(ButtonVariant::Primary)
                    .min_size(vec2(152.0, 34.0)),
                )
            })
            .inner;
        if trigger.clicked() {
            self.spinner_demo_until = Some(now + 1.0);
        }

        if spinner_demo_active {
            let spinner_color = theme::color(ui, ColorRole::PrimaryForeground);
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
            ),
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
        let card_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Border));
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let width = 700.0f32.min(ui.available_width());
            let _ = ui
                .components()
                .card(Card::new().fill(canvas_fill).stroke(card_stroke), |ui| {
                    ui.set_width(width);
                    let (host_rect, _) =
                        ui.allocate_exact_size(vec2(width - 24.0, 360.0), Sense::hover());
                    let _ = ui.scope_builder(egui::UiBuilder::new().max_rect(host_rect), |ui| {
                        let preview_rect = host_rect.shrink(1.0);
                        let preview_radius = radius_md(ui);
                        let preview_animation = ui.ctx().animate_bool_responsive(
                            Id::new("component_showcase_sidebar_preview"),
                            self.sidebar_preview_open,
                        );
                        let preview_fill = if self.theme_mode.is_dark() {
                            app_background(ui).lerp_to_gamma(theme::color(ui, ColorRole::Card), 0.18)
                        } else {
                            theme::color(ui, ColorRole::Card)
                        };
                        let preview_header_rect = egui::Rect::from_min_max(
                            preview_rect.min,
                            egui::pos2(preview_rect.right(), preview_rect.top() + 46.0),
                        );
                        let preview_rail_rect = if self.sidebar_side_index == 0 {
                            egui::Rect::from_min_max(
                                egui::pos2(preview_rect.left(), preview_header_rect.bottom()),
                                egui::pos2(preview_rect.left() + 54.0, preview_rect.bottom()),
                            )
                        } else {
                            egui::Rect::from_min_max(
                                egui::pos2(preview_rect.right() - 54.0, preview_header_rect.bottom()),
                                preview_rect.right_bottom(),
                            )
                        };

                        ui.painter().rect(
                            preview_rect,
                            egui::CornerRadius::same(preview_radius),
                            preview_fill,
                            Stroke::new(1.0, theme::color(ui, ColorRole::Border)),
                            egui::StrokeKind::Outside,
                        );
                        ui.painter().rect(
                            preview_header_rect,
                            egui::CornerRadius {
                                nw: preview_radius,
                                ne: preview_radius,
                                sw: 0,
                                se: 0,
                            },
                            theme::color(ui, ColorRole::Background),
                            Stroke::NONE,
                            egui::StrokeKind::Outside,
                        );
                        ui.painter().line_segment(
                            [
                                egui::pos2(preview_rect.left(), preview_header_rect.bottom()),
                                egui::pos2(preview_rect.right(), preview_header_rect.bottom()),
                            ],
                            Stroke::new(1.0, theme::color(ui, ColorRole::Border)),
                        );
                        ui.painter().rect(
                            preview_rail_rect,
                            sidebar_preview_rail_radius(self.sidebar_side_index, preview_radius),
                            theme::color(ui, ColorRole::Muted),
                            Stroke::NONE,
                            egui::StrokeKind::Outside,
                        );

                        let _ = ui.scope_builder(
                            egui::UiBuilder::new()
                                .max_rect(preview_header_rect.shrink2(vec2(14.0, 10.0))),
                            |ui| {
                                let _ = layout::row().gap(10.0).show(ui, |ui| {
                                    {
                                        let mut components = ui.components();
                                        let _ = components.label(
                                            Label::new("Sidebar Preview")
                                                .tone(LabelTone::Primary)
                                                .weight(LabelWeight::Semibold),
                                        );
                                        let _ = components.label(
                                            Label::new(if self.sidebar_preview_open {
                                                "Open"
                                            } else {
                                                "Closed"
                                            })
                                            .tone(LabelTone::Muted)
                                            .size(SMALL_TEXT),
                                        );
                                    }
                                    let _ = layout::spacer().show(ui);
                                    {
                                        let mut components = ui.components();
                                        let _ = components.label(
                                            Label::new(if self.sidebar_side_index == 0 {
                                                "Left dock"
                                            } else {
                                                "Right dock"
                                            })
                                            .tone(LabelTone::Muted)
                                            .size(SMALL_TEXT),
                                        );
                                    }
                                });
                            },
                        );

                        let toggle_rect = if self.sidebar_side_index == 0 {
                            egui::Rect::from_min_size(
                                egui::pos2(
                                    preview_rect.left() + 9.0 + (preview_animation * 4.0),
                                    preview_header_rect.bottom() + 12.0,
                                ),
                                vec2(36.0, 36.0),
                            )
                        } else {
                            egui::Rect::from_min_size(
                                egui::pos2(
                                    preview_rect.right() - 45.0 - (preview_animation * 4.0),
                                    preview_header_rect.bottom() + 12.0,
                                ),
                                vec2(36.0, 36.0),
                            )
                        };
                        let toggle_icon =
                            sidebar_preview_toggle_icon(self.sidebar_side_index, self.sidebar_preview_open);
                        let toggle_response =
                            ui.scope_builder(egui::UiBuilder::new().max_rect(toggle_rect), |ui| {
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

                        let content_rect = if self.sidebar_side_index == 0 {
                            egui::Rect::from_min_max(
                                egui::pos2(
                                    preview_rail_rect.right() + 20.0,
                                    preview_header_rect.bottom() + 16.0,
                                ),
                                egui::pos2(preview_rect.right() - 18.0, preview_rect.bottom() - 18.0),
                            )
                        } else {
                            egui::Rect::from_min_max(
                                egui::pos2(preview_rect.left() + 18.0, preview_header_rect.bottom() + 16.0),
                                egui::pos2(preview_rail_rect.left() - 20.0, preview_rect.bottom() - 18.0),
                            )
                        };
                        let _ = ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                            let _ = layout::column().gap(10.0).show(ui, |ui| {
                                let mut components = ui.components();
                                let _ = components.label(
                                    Label::new("Workspace canvas")
                                        .tone(LabelTone::Primary)
                                        .weight(LabelWeight::Semibold),
                                );
                                let _ = components.label(
                                    Label::new("Use the edge control to preview the animated open and close states.")
                                        .tone(LabelTone::Muted)
                                        .size(SMALL_TEXT),
                                );

                                let card_fill = input_background(ui);
                                let card_stroke =
                                    Stroke::new(1.0, theme::color(ui, ColorRole::Border));
                                let _ = ui.components().card(
                                    Card::new().fill(card_fill).stroke(card_stroke),
                                    |ui| {
                                        ui.set_width(ui.available_width());
                                        let _ = ui.components().label(
                                            Label::new("Inspector")
                                                .tone(LabelTone::Primary)
                                                .weight(LabelWeight::Semibold),
                                        );
                                        let _ = ui.components().label(
                                            Label::new("Layer, transforms, and appearance metadata stay visible while the sidebar animates.")
                                                .tone(LabelTone::Muted)
                                                .size(SMALL_TEXT),
                                        );
                                    },
                                );
                            });
                        });

                        ui.components().sidebar(
                            &mut self.sidebar_preview_open,
                            Sidebar::new(Id::new("component_showcase_overlay_sidebar"))
                                .title("Workspace")
                                .side(if self.sidebar_side_index == 0 {
                                    SidebarSide::Left
                                } else {
                                    SidebarSide::Right
                                })
                                .width(240.0),
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
                    });
                });
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
                                .width(280.0)
                                .overlap(42.0),
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
        let _ = ui.components().pagination(
            &mut self.pagination_page,
            Pagination::new(
                Id::new("component_showcase_pagination"),
                PAGINATION_PAGE_COUNT,
            ),
        );

        ui.add_space(8.0);
        let summary = format!("Page {} of {}", self.pagination_page, PAGINATION_PAGE_COUNT);
        let _ = ui.components().label(
            Label::new(summary.as_str())
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
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
        ComponentKind::Sidebar => {
            "Button-triggered overlay sidebar rendered inside a host surface."
        }
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

fn sidebar_preview_rail_radius(side_index: usize, radius: u8) -> egui::CornerRadius {
    if side_index == 0 {
        egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: radius,
            se: 0,
        }
    } else {
        egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: 0,
            se: radius,
        }
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
    state.toolbar_color_index = state
        .toolbar_color_index
        .min(TOOLBAR_SWATCHES.len().saturating_sub(1));
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
