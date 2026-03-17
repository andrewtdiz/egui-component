use crate::catalog::{self, ComponentDefinition, ComponentGroup, ComponentKind};
use crate::components::{
    AudioPlayback, AudioPlaybackState, Button, ButtonOverride, ButtonVariant, Checkbox, Color,
    CommandItem, ComponentUi, ComponentUiExt, ControlSize, DialogueIntent, DropdownMenu,
    DropdownMenuEntry, Image, ImageTile, ImageTilePlaybackState, ImageTileSize, Kbd, KbdGroup,
    Label, LabelTone, LabelWeight, MenuBar, MenuBarItem, NumberInput, NumberInputAxis, Pagination,
    Select, TabOption, TabsVariant, Toolbar, Tooltip, TooltipPlacement,
};
use crate::layout;
use crate::ui::{tokens, typography};
use egui::{Align2, Color32, CornerRadius, CursorIcon, Id, Layout, Sense, Stroke, StrokeKind, Ui};

use egui::containers::scroll_area::ScrollSource;

const BUTTON_GROUP_OPTIONS: [&str; 3] = ["Move", "Rotate", "Scale"];
const TOOLBAR_ACTION_OPTIONS: [&str; 3] = ["Edit", "BG Remover", "Eraser"];
const TAB_OPTIONS: [TabOption<'static>; 3] = [
    TabOption::new(0, "Design"),
    TabOption::new(1, "Code"),
    TabOption::new(2, "History"),
];
const PAGINATION_PAGE_COUNT: usize = 12;
const STACKED_TAB_OPTIONS: [TabOption<'static>; 2] = [
    TabOption::with_icon(0, "Templates", "layout-template"),
    TabOption::with_icon(1, "Layouts", "layout-grid"),
];
const TOOLBAR_SWATCHES: [Color32; 4] = [
    Color32::from_rgb(35, 45, 75),
    Color32::from_rgb(103, 132, 162),
    Color32::from_rgb(122, 24, 42),
    Color32::from_rgb(206, 164, 84),
];
const TOOLBAR_CANVAS_LIGHT_FILL: Color32 = Color32::from_rgb(228, 228, 231);
const IMAGE_TILE_META_ACCENT: Color32 = Color32::from_rgb(59, 130, 246);
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
const COMMAND_OPTIONS: [CommandItem<'static>; 11] = [
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

#[derive(Debug, Clone, Copy)]
struct ShowcaseMetadata {
    kind: ComponentKind,
    description: &'static str,
    section_override: Option<ShowcaseSection>,
}

const SHOWCASE_METADATA: &[ShowcaseMetadata] = &[
    // xtask:showcase-metadata:start
    ShowcaseMetadata {
        kind: ComponentKind::Label,
        description: "Text styles and tones",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Color,
        description: "Circular solid color swatches",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Image,
        description: "PNG-backed raster image rendering",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Icon,
        description: "Lucide icon rendering",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Kbd,
        description: "Keyboard keycaps and shortcuts",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Input,
        description: "Single-line text input",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Field,
        description: "Label + input + helper text",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Button,
        description: "Text, icon, and link button variants",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::ButtonGroup,
        description: "Attached segmented action group",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Checkbox,
        description: "Boolean control with label",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Switch,
        description: "Toggle control",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Slider,
        description: "Range input",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::NumberInput,
        description: "Drag-based numeric input",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Select,
        description: "Popup option picker",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Tabs,
        description: "Segmented tab selector",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Separator,
        description: "Inline divider",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Card,
        description: "Container surface",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Progress,
        description: "Completion indicator",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Tooltip,
        description: "Hover helper text",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::DropdownMenu,
        description: "Triggered option list",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Collapsible,
        description: "Expandable content section",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::AudioPlayback,
        description: "Horizontal audio row with play/pause, waveform, and actions",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Combobox,
        description: "Filterable option list",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Command,
        description: "Searchable command palette",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::Dialogue,
        description: "Dialogue window trigger",
        section_override: None,
    },
    ShowcaseMetadata {
        kind: ComponentKind::MenuBar,
        description: "Top-level menu strip with hover switching",
        section_override: Some(ShowcaseSection::Examples),
    },
    ShowcaseMetadata {
        kind: ComponentKind::Toolbar,
        description: "Floating absolute-positioned editing bar",
        section_override: Some(ShowcaseSection::Examples),
    },
    ShowcaseMetadata {
        kind: ComponentKind::ImageTile,
        description: "Grid-friendly media tile with custom body and playback overlay",
        section_override: Some(ShowcaseSection::Examples),
    },
    ShowcaseMetadata {
        kind: ComponentKind::Pagination,
        description: "Previous/next pager with numbered pages and ellipsis",
        section_override: None,
    },
    // xtask:showcase-metadata:end
];

#[derive(Debug, Clone)]
pub struct ComponentShowcaseState {
    selected_component: ComponentKind,
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
    select_index: Option<usize>,
    tab_index: usize,
    segmented_tab_index: usize,
    stacked_tab_index: usize,
    audio_playback_state: AudioPlaybackState,
    pagination_page: usize,
    button_group_index: usize,
    collapsible_open: bool,
    dropdown_action: Option<usize>,
    combobox_query: String,
    combobox_index: usize,
    command_query: String,
    dialogue_open: bool,
    alert_dialogue_open: bool,
    menu_bar_action: Option<usize>,
    tooltip_placement: TooltipPlacement,
    image_tile_playback_state: ImageTilePlaybackState,
    image_tile_last_action: String,
}

impl Default for ComponentShowcaseState {
    fn default() -> Self {
        Self {
            selected_component: ComponentKind::Button,
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
            progress_value: 0.58,
            select_index: Some(1),
            tab_index: 0,
            segmented_tab_index: 0,
            stacked_tab_index: 0,
            audio_playback_state: AudioPlaybackState::Paused,
            pagination_page: 2,
            button_group_index: 0,
            collapsible_open: true,
            dropdown_action: None,
            combobox_query: "mat".to_owned(),
            combobox_index: 0,
            command_query: String::new(),
            dialogue_open: false,
            alert_dialogue_open: false,
            menu_bar_action: None,
            tooltip_placement: TooltipPlacement::Top,
            image_tile_playback_state: ImageTilePlaybackState::Paused,
            image_tile_last_action: "No image tile actions yet".to_owned(),
        }
    }
}

pub(super) fn render(ui: &mut Ui, state: &mut ComponentShowcaseState) {
    clamp_state(state);
    layout::set_debug_overlay(ui.ctx(), false);

    let _ = egui::SidePanel::left("component_showcase_sidebar")
        .default_width(238.0)
        .min_width(200.0)
        .max_width(320.0)
        .resizable(true)
        .frame(
            egui::Frame::new()
                .fill(tokens::app_background(crate::theme::runtime_for_context(ui.ctx())))
                .inner_margin(egui::Margin::same(8))
                .stroke(Stroke::new(1.0, tokens::separator(crate::theme::runtime_for_context(ui.ctx())))),
        )
        .show_inside(ui, |ui| {
            let mut ui = ui.components();
            draw_sidebar(&mut ui, state);
        });

    let _ = egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(tokens::app_background(crate::theme::runtime_for_context(ui.ctx())))
                .inner_margin(egui::Margin::same(12)),
        )
        .show_inside(ui, |ui| {
            let mut ui = ui.components();
            draw_center_preview(&mut ui, state);
        });
}

fn draw_sidebar(ui: &mut ComponentUi<'_>, state: &mut ComponentShowcaseState) {
    let _ = ui.label(
        Label::new("Components")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    let _ = ui.label(
        Label::new("Select a component to preview")
            .tone(LabelTone::Muted)
            .size(typography::SMALL_SIZE),
    );
    ui.add_space(8.0);
    let _ = ui.separator();
    ui.add_space(8.0);

    let _ = egui::ScrollArea::vertical()
        .id_salt("component_showcase_sidebar_scroll")
        .scroll_source(ScrollSource {
            drag: false,
            ..ScrollSource::default()
        })
        .auto_shrink([false, false])
        .show(ui.ui_mut(), |ui| {
            let mut ui = ui.components();
            ui.spacing_mut().item_spacing.y = 1.0;
            for section in [
                ShowcaseSection::PrimaryPrimitive,
                ShowcaseSection::DerivedComposed,
                ShowcaseSection::Examples,
            ] {
                let _ = ui.label(
                    Label::new(section.title())
                        .tone(LabelTone::Muted)
                        .size(typography::SMALL_SIZE)
                        .weight(LabelWeight::Semibold),
                );
                ui.add_space(4.0);

                for definition in showcase_component_definitions_by_section(section) {
                    let selected = state.selected_component == definition.kind;
                    if draw_sidebar_component_row(ui.ui_mut(), definition.label, selected)
                    .clicked()
                    {
                        state.selected_component = definition.kind;
                    }
                }

                ui.add_space(1.0);
            }
        });
}

fn draw_sidebar_component_row(
    ui: &mut Ui,
    label_text: &str,
    selected: bool,
) -> egui::Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let desired_size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let fill = if response.is_pointer_button_down_on() {
        tokens::row_active_bg(runtime)
    } else if selected || response.hovered() {
        tokens::row_selected_bg(runtime)
    } else {
        tokens::TRANSPARENT
    };

    ui.painter().rect(
        rect,
        CornerRadius::same(tokens::radius_sm(runtime)),
        fill,
        Stroke::NONE,
        StrokeKind::Outside,
    );

    ui.painter().text(
        egui::pos2(rect.left() + 9.0, rect.center().y),
        Align2::LEFT_CENTER,
        label_text,
        typography::label_font(),
        if selected || response.hovered() {
            tokens::row_selected_text(runtime)
        } else {
            tokens::text_secondary(runtime)
        },
    );

    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn draw_center_preview(ui: &mut ComponentUi<'_>, state: &mut ComponentShowcaseState) {
    let definition = catalog_component_definition(state.selected_component);
    let metadata = showcase_metadata(state.selected_component);
    let overrides = ui.overrides();

    let _ = egui::ScrollArea::vertical()
        .id_salt("component_showcase_preview_scroll")
        .scroll_source(ScrollSource {
            drag: false,
            ..ScrollSource::default()
        })
        .auto_shrink([false, false])
        .show(ui.ui_mut(), |ui| {
            let mut ui = ComponentUi::with_overrides(ui, overrides);
            let overrides = ui.overrides();
            ui.ui_mut()
                .with_layout(Layout::top_down(egui::Align::Center), |ui| {
                    let mut ui = ComponentUi::with_overrides(ui, overrides);
                    let width = if matches!(
                        state.selected_component,
                        ComponentKind::Toolbar | ComponentKind::MenuBar
                    ) {
                        ui.available_width().min(980.0)
                    } else {
                        ui.available_width().clamp(260.0, 460.0)
                    };

                    let _ = ui.card((), |ui| {
                        let mut ui = ui.components();
                        ui.ui_mut().set_width(width);

                        let _ = ui.label(
                            Label::new(definition.label)
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );
                        let _ = ui.label(
                            Label::new(metadata.description)
                                .tone(LabelTone::Muted)
                                .size(typography::SMALL_SIZE),
                        );
                        ui.add_space(8.0);
                        let _ = ui.separator();
                        ui.add_space(10.0);

                        render_selected_preview(&mut ui, state);
                    });
                });
        });
}

fn render_selected_preview(ui: &mut ComponentUi<'_>, state: &mut ComponentShowcaseState) {
    let overrides = ui.overrides();
    match state.selected_component {
        // xtask:showcase-render-arms:start
        ComponentKind::Label => {
            let _ = ui.label(
                Label::new("Primary label")
                    .tone(LabelTone::Primary)
                    .weight(LabelWeight::Semibold),
            );
            let _ = ui.label(Label::new("Secondary label").tone(LabelTone::Secondary));
            let _ = ui.label(Label::new("Muted helper text").tone(LabelTone::Muted));
            let _ = ui.label(
                Label::new("Destructive text")
                    .tone(LabelTone::Destructive)
                    .weight(LabelWeight::Semibold),
            );
        }
        ComponentKind::Color => {
            let runtime = crate::theme::runtime_for_ui(ui);
            let border = Stroke::new(1.0, tokens::input_border(runtime));

            let _ = ui.label(
                Label::new("Solid swatches")
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
            ui.add_space(6.0);
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                for fill in TOOLBAR_SWATCHES {
                    let _ = ui.color(Color::new(fill).size(20.0));
                }
            });
            ui.add_space(10.0);
            let _ = ui.label(
                Label::new("Bordered swatches")
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
            ui.add_space(6.0);
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                for fill in TOOLBAR_SWATCHES {
                    let _ = ui.color(Color::new(fill).size(20.0).stroke(border));
                }
            });
            ui.add_space(10.0);
            let _ = ui.label(
                Label::new("Sizes")
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
            ui.add_space(6.0);
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let _ = ui.color(Color::new(TOOLBAR_SWATCHES[0]).size(12.0));
                let _ = ui.color(Color::new(TOOLBAR_SWATCHES[1]).size(16.0));
                let _ = ui.color(Color::new(TOOLBAR_SWATCHES[2]).size(20.0));
                let _ = ui.color(Color::new(TOOLBAR_SWATCHES[3]).size(28.0).stroke(border));
            });
        }
        ComponentKind::Image => {
            let runtime = crate::theme::runtime_for_ui(ui);
            let sample_png = egui::include_image!("../../../assets/images/clay_logo_large.png");

            let _ = ui.label(
                Label::new("Embedded PNG")
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
            ui.add_space(6.0);
            let _ = ui.image(
                Image::new(sample_png.clone())
                    .fit_to_exact_size(egui::vec2(180.0, 180.0))
                    .corner_radius(tokens::radius_md(runtime))
                    .bg_fill(tokens::input_background(runtime)),
            );

            ui.add_space(10.0);
            let _ = ui.label(
                Label::new("bytes:// source")
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
            ui.add_space(6.0);
            let _ = ui.image(
                Image::from_bytes(
                    "bytes://component-showcase/clay-logo-large.png",
                    include_bytes!("../../../assets/images/clay_logo_large.png"),
                )
                .fit_to_exact_size(egui::vec2(96.0, 96.0))
                .corner_radius(tokens::radius_sm(runtime)),
            );

            ui.add_space(10.0);
            let _ = ui.label(
                Label::new("Rotation")
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
            ui.add_space(6.0);
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let _ = ui.slider(&mut state.image_rotation_degrees, (-180.0..=180.0, 220.0));
                let rotation_label = format!("{:.0}deg", state.image_rotation_degrees.round());
                let _ = ui.label(
                    Label::new(rotation_label.as_str())
                        .tone(LabelTone::Secondary)
                        .weight(LabelWeight::Semibold),
                );
            });
            ui.add_space(6.0);
            let _ = ui.image(
                Image::new(sample_png)
                    .fit_to_exact_size(egui::vec2(96.0, 96.0))
                    .rotate(
                        state.image_rotation_degrees.to_radians(),
                        egui::vec2(0.5, 0.5),
                    )
                    .bg_fill(tokens::input_background(runtime)),
            );

            ui.add_space(8.0);
            let _ = ui.label(
                Label::new("Supports include_image!, bytes://, and file:// PNG sources.")
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
        }
        ComponentKind::Kbd => {
            let _ = ui.kbd_group(KbdGroup::new(), |ui| {
                let mut ui = ui.components();
                let _ = ui.kbd(Kbd::new("⌘"));
                let _ = ui.kbd(Kbd::new("⇧"));
                let _ = ui.kbd(Kbd::new("⌥"));
                let _ = ui.kbd(Kbd::new("⌃"));
            });
            ui.add_space(6.0);
            let _ = ui.kbd_group(KbdGroup::new(), |ui| {
                let mut ui = ui.components();
                let _ = ui.kbd(Kbd::new("Ctrl"));
                let _ = ui.label(
                    Label::new("+")
                        .tone(LabelTone::Muted)
                        .weight(LabelWeight::Semibold),
                );
                let _ = ui.kbd(Kbd::new("B"));
            });
        }
        ComponentKind::Input => {
            let _ = ui.text_input(
                &mut state.input_value,
                crate::components::TextInput::new()
                    .width(280.0)
                    .placeholder("Type component name"),
            );
        }
        ComponentKind::Field => {
            let _ = ui.field(
                &mut state.field_value,
                crate::components::Field::new("Material")
                    .width(280.0)
                    .helper_text("Assigned material for selected mesh"),
            );
        }
        ComponentKind::Button => {
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let _ = ui.button(Button::new("Primary").variant(ButtonVariant::Primary));
                let _ = ui.button(Button::new("Secondary").variant(ButtonVariant::Secondary));
                let _ = ui.button(Button::new("Ghost").variant(ButtonVariant::Ghost));
                let _ = ui.button(Button::new("Link").variant(ButtonVariant::Link));
            });
            ui.add_space(8.0);
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let _ = ui.button(
                    Button::icon_only("wand-sparkles")
                        .icon_size(15.0)
                        .variant(ButtonVariant::Primary),
                );
                let _ = ui.button(
                    Button::icon_only("wand-sparkles")
                        .icon_size(15.0)
                        .variant(ButtonVariant::Secondary),
                );
                let _ = ui.button(
                    Button::icon_only("wand-sparkles")
                        .icon_size(15.0)
                        .variant(ButtonVariant::Ghost),
                );
                let _ = ui.button(
                    Button::icon_only("wand-sparkles")
                        .icon_size(15.0)
                        .variant(ButtonVariant::Link),
                );
            });
            ui.add_space(8.0);
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let _ = ui.button(
                    Button::new("Create")
                        .leading_icon("plus")
                        .variant(ButtonVariant::Primary),
                );
                let _ = ui.button(
                    Button::new("Create")
                        .leading_icon("plus")
                        .variant(ButtonVariant::Secondary),
                );
                let _ = ui.button(
                    Button::new("Create")
                        .leading_icon("plus")
                        .variant(ButtonVariant::Ghost),
                );
                let _ = ui.button(
                    Button::new("Create")
                        .leading_icon("plus")
                        .variant(ButtonVariant::Link),
                );
            });
            ui.add_space(8.0);
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let runtime = crate::theme::runtime_for_ui(&ui);
                for (index, fill) in TOOLBAR_SWATCHES.iter().copied().enumerate() {
                    let stroke = if index == 1 {
                        Stroke::new(1.0, tokens::text_primary(runtime))
                    } else {
                        Stroke::NONE
                    };
                    let _ = ui.button(
                        Button::color_only(Color::new(fill).size(18.0).stroke(stroke))
                            .variant(ButtonVariant::Ghost),
                    );
                }
            });
        }
        ComponentKind::ButtonGroup => {
            ui.button_group(
                &mut state.button_group_index,
                crate::components::ButtonGroup::new(
                    Id::new("component_showcase_button_group"),
                    &BUTTON_GROUP_OPTIONS[..],
                ),
            );
        }
        ComponentKind::Checkbox => {
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let mut checkbox_response = ui.checkbox(&mut state.checkbox_value, Checkbox::new());
                let runtime = crate::theme::runtime_for_ui(&ui);
                let base_label_color = if state.checkbox_value {
                    tokens::text_secondary(runtime)
                } else {
                    tokens::text_muted(runtime)
                };
                let label_response = ui
                    .ui_mut()
                    .scope(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new("Receive Shadows").color(base_label_color),
                            )
                            .selectable(false)
                            .sense(egui::Sense::click()),
                        )
                    })
                    .inner
                    .on_hover_cursor(CursorIcon::PointingHand);
                if label_response.clicked() {
                    state.checkbox_value = !state.checkbox_value;
                    checkbox_response.mark_changed();
                }
                let _ = checkbox_response.on_hover_cursor(CursorIcon::PointingHand);
            });
        }
        ComponentKind::Switch => {
            let _ = ui.switch(&mut state.switch_value, "Enable Post FX");
            let _ = ui.switch(
                &mut state.small_switch_value,
                crate::components::Switch::new()
                    .label("Use Compact Handles")
                    .size(ControlSize::Sm),
            );
        }
        ComponentKind::Slider => {
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let _ = ui.slider(&mut state.slider_value, (0.0..=100.0, 250.0));
                let value_label = format!("{:.0}", state.slider_value.round());
                let _ = ui.label(
                    Label::new(value_label.as_str())
                        .tone(LabelTone::Secondary)
                        .weight(LabelWeight::Semibold),
                );
            });
        }
        ComponentKind::NumberInput => {
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let _ = ui.number_input(
                    &mut state.number_x_value,
                    NumberInput::new(Id::new("component_showcase_number_x"))
                        .width(110.0)
                        .range(0.0..=100.0)
                        .decimals(1)
                        .prefix("X")
                        .prefix_tint(tokens::GAME_ENGINE_GREEN)
                        .prefix_align_left()
                        .axis(NumberInputAxis::Horizontal),
                );
                let _ = ui.number_input(
                    &mut state.number_y_value,
                    NumberInput::new(Id::new("component_showcase_number_y"))
                        .width(110.0)
                        .range(0.0..=100.0)
                        .decimals(1)
                        .prefix("Y")
                        .prefix_tint(tokens::GAME_ENGINE_RED)
                        .prefix_align_left()
                        .axis(NumberInputAxis::Vertical),
                );
            });
        }
        ComponentKind::Select => {
            let _ = ui.select(
                &mut state.select_index,
                Select::from_id(Id::new("component_showcase_select"), &SELECT_OPTIONS[..])
                    .width(280.0),
            );

            let selected_label = state
                .select_index
                .and_then(|index| SELECT_OPTIONS.get(index).copied())
                .unwrap_or("None");
            let _ = ui.label(
                Label::new(selected_label)
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
        }
        ComponentKind::Tabs => {
            ui.tabs(
                Id::new("component_showcase_tabs"),
                &mut state.tab_index,
                &TAB_OPTIONS,
            );

            let selected_tab = TAB_OPTIONS
                .iter()
                .find(|option| option.value == state.tab_index)
                .map(|option| option.label)
                .unwrap_or(TAB_OPTIONS[0].label);
            let _ = ui.label(
                Label::new(selected_tab)
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );

            ui.add_space(18.0);
            ui.tabs_variant(
                Id::new("component_showcase_segmented_tabs"),
                &mut state.segmented_tab_index,
                &TAB_OPTIONS,
                TabsVariant::Segmented,
            );

            let selected_segmented_tab = TAB_OPTIONS
                .iter()
                .find(|option| option.value == state.segmented_tab_index)
                .map(|option| option.label)
                .unwrap_or(TAB_OPTIONS[0].label);
            let _ = ui.label(
                Label::new(selected_segmented_tab)
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );

            ui.add_space(18.0);
            ui.stacked_tabs(
                Id::new("component_showcase_stacked_tabs"),
                &mut state.stacked_tab_index,
                &STACKED_TAB_OPTIONS,
            );

            let selected_stacked_tab = STACKED_TAB_OPTIONS
                .iter()
                .find(|option| option.value == state.stacked_tab_index)
                .map(|option| option.label)
                .unwrap_or(STACKED_TAB_OPTIONS[0].label);
            let _ = ui.label(
                Label::new(selected_stacked_tab)
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
        }
        ComponentKind::Separator => {
            let _ = ui.label(Label::new("Above separator").tone(LabelTone::Secondary));
            let _ = ui.separator();
            let _ = ui.label(Label::new("Below separator").tone(LabelTone::Secondary));
        }
        ComponentKind::Card => {
            let runtime = crate::theme::runtime_for_ui(ui);
            let _ = ui.card(
                (
                    tokens::input_background(runtime),
                    Stroke::new(1.0, tokens::input_border(runtime)),
                ),
                |ui| {
                    let mut ui = ui.components();
                    let overrides = ui.overrides();
                    ui.ui_mut()
                        .with_layout(Layout::top_down(egui::Align::Min), |ui| {
                            let mut ui = ComponentUi::with_overrides(ui, overrides);
                            let _ = ui.label(
                                Label::new("Card Title")
                                    .tone(LabelTone::Primary)
                                    .weight(LabelWeight::Bold)
                                    .size(16.0),
                            );
                            let _ = ui.label(
                                Label::new("Cards wrap related content in a bordered panel.")
                                    .tone(LabelTone::Muted),
                            );
                            ui.add_space(6.0);
                            let _ = ui.separator();
                            ui.add_space(6.0);
                            let footer_size =
                                egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                            let overrides = ui.overrides();
                            let _ = ui.ui_mut().allocate_ui_with_layout(
                                footer_size,
                                Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let mut ui = ComponentUi::with_overrides(ui, overrides);
                                    let _ = ui.button(
                                        Button::new("Save").variant(ButtonVariant::Primary),
                                    );
                                    let _ = ui.button(
                                        Button::new("Cancel").variant(ButtonVariant::Secondary),
                                    );
                                },
                            );
                        });
                },
            );
        }
        ComponentKind::Progress => {
            let _ = ui.progress(state.progress_value, (280.0, 10.0));
        }
        ComponentKind::Tooltip => {
            let mut placement_index = tooltip_placement_index(state.tooltip_placement);
            ui.button_group(
                &mut placement_index,
                crate::components::ButtonGroup::new(
                    Id::new("component_showcase_tooltip_placement"),
                    &TOOLTIP_PLACEMENT_OPTIONS[..],
                ),
            );
            state.tooltip_placement = tooltip_placement_from_index(placement_index);
            ui.add_space(6.0);
            let _ = ui.tooltip(
                Tooltip::new("Hover this trigger", "Tooltip content example")
                    .width(220.0)
                    .placement(state.tooltip_placement),
            );
        }
        ComponentKind::Collapsible => {
            let collapsible_open = state.collapsible_open;
            let _ = ui.collapsible(
                &mut state.collapsible_open,
                crate::components::Collapsible::new(
                    Id::new("component_showcase_collapsible"),
                    "Transform",
                )
                .open(collapsible_open)
                .leading_icon("move-3d")
                .trailing_icon("ellipsis_vertical"),
                |ui| {
                    let mut ui = ui.components();
                    let _ = ui.label(Label::new("Position").tone(LabelTone::Secondary));
                    let _ = ui.label(Label::new("Rotation").tone(LabelTone::Secondary));
                    let _ = ui.label(Label::new("Scale").tone(LabelTone::Secondary));
                },
            );
        }
        ComponentKind::DropdownMenu => {
            let (_response, menu_state) = ui.dropdown_menu(
                DropdownMenu::new("Open")
                    .entries(&DROPDOWN_ENTRIES[..])
                    .width(220.0),
            );
            if let Some(action) = menu_state.action {
                state.dropdown_action = Some(action);
            }
            let _ = ui.label(
                Label::new(dropdown_action_label(state.dropdown_action))
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
            ui.add_space(8.0);
            let _ = ui.label(
                Label::new("Shortcut keycaps")
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
            ui.add_space(4.0);
            draw_dropdown_shortcut_row(ui, "New Team", &["⌘", "T"]);
            draw_dropdown_shortcut_row(ui, "Log out", &["⇧", "⌘", "Q"]);
        }
        ComponentKind::Combobox => {
            let _ = ui.combobox(
                &mut state.combobox_query,
                &mut state.combobox_index,
                crate::components::Combobox::new(
                    Id::new("component_showcase_combobox"),
                    &COMBOBOX_OPTIONS[..],
                )
                .width(280.0),
            );
            let _ = ui.label(
                Label::new(COMBOBOX_OPTIONS[state.combobox_index])
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
        }
        ComponentKind::Command => {
            let _ = ui.command(
                &mut state.command_query,
                &COMMAND_OPTIONS,
                crate::components::Command::new(Id::new("component_showcase_command"))
                    .width(380.0)
                    .preview(true),
            );
        }
        ComponentKind::Dialogue => {
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                if ui
                    .button(Button::new("Open Dialogue").variant(ButtonVariant::Secondary))
                    .clicked()
                {
                    state.dialogue_open = true;
                }

                if ui
                    .button(Button::new("Open Alert Dialogue").variant(ButtonVariant::Secondary))
                    .clicked()
                {
                    state.alert_dialogue_open = true;
                }
            });

            ui.dialogue_modal(
                &mut state.dialogue_open,
                crate::components::DialogueModal::new(Id::new("component_showcase_dialogue"))
                    .width(380.0),
                |ui, close_requested| {
                    let mut ui = ui.components();
                    ui.dialogue_header_with_close(
                        crate::components::DialogueHeader::new("Create Component")
                            .description("Adds the selected component to the active object."),
                        close_requested,
                    );
                    ui.add_space(12.0);
                    let overrides = ui.overrides();
                    let _ = ui.ui_mut().vertical(|ui| {
                        let mut ui = ComponentUi::with_overrides(ui, overrides);
                        let _ = ui.label(
                            Label::new(
                                "Pick a component from the sidebar and confirm to add it to the object.",
                            )
                            .tone(LabelTone::Muted)
                            .size(12.0),
                        );
                    });
                    ui.add_space(12.0);
                    let footer_size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                    let overrides = ui.overrides();
                    let _ = ui.ui_mut().allocate_ui_with_layout(
                        footer_size,
                        Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let mut ui = ComponentUi::with_overrides(ui, overrides);
                            if ui
                                .button(Button::new("Create").variant(ButtonVariant::Primary))
                                .clicked()
                            {
                                *close_requested = true;
                            }
                            if ui
                                .button(Button::new("Cancel").variant(ButtonVariant::Secondary))
                                .clicked()
                            {
                                *close_requested = true;
                            }
                        },
                    );
                },
            );

            ui.dialogue_modal(
                &mut state.alert_dialogue_open,
                crate::components::DialogueModal::new(
                    Id::new("component_showcase_alert_dialogue"),
                )
                .width(380.0),
                |ui, close_requested| {
                    let mut ui = ui.components();
                    ui.dialogue_header_with_close(
                        crate::components::DialogueHeader::new("Delete Object")
                            .description("This action cannot be undone.")
                            .intent(DialogueIntent::Alert),
                        close_requested,
                    );
                    ui.add_space(12.0);
                    let overrides = ui.overrides();
                    let _ = ui.ui_mut().vertical(|ui| {
                        let mut ui = ComponentUi::with_overrides(ui, overrides);
                        let _ = ui.label(
                            Label::new(
                                "Deleting removes the object and every child object in this hierarchy.",
                            )
                            .tone(LabelTone::Muted)
                            .size(12.0),
                        );
                    });
                    ui.add_space(12.0);
                    let footer_size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                    let overrides = ui.overrides();
                    let _ = ui.ui_mut().allocate_ui_with_layout(
                        footer_size,
                        Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let mut ui = ComponentUi::with_overrides(ui, overrides);
                            if ui
                                .button(Button::new("Delete").variant(ButtonVariant::Primary))
                                .clicked()
                            {
                                *close_requested = true;
                            }
                            if ui
                                .button(Button::new("Cancel").variant(ButtonVariant::Secondary))
                                .clicked()
                            {
                                *close_requested = true;
                            }
                        },
                    );
                },
            );
        }
        ComponentKind::Icon => {
            ui.ui_mut().horizontal(|ui| {
                let mut ui = ComponentUi::with_overrides(ui, overrides);
                let _ = ui.icon(crate::components::Icon::new("bot").size(16.0));
                let _ = ui.icon(crate::components::Icon::new("settings-2").size(16.0));
                let _ = ui.icon(crate::components::Icon::new("sparkles").size(16.0));
                let _ = ui.icon(crate::components::Icon::new("gamepad-2").size(16.0));
                let _ = ui.icon(crate::components::Icon::new("wand-sparkles").size(16.0));
            });
        }
        ComponentKind::MenuBar => {
            draw_menu_bar_preview(ui, state);
        }
        ComponentKind::Toolbar => {
            draw_toolbar_preview(ui, state);
        }
        ComponentKind::ImageTile => {
            draw_image_tile_preview(ui, state);
        }
        ComponentKind::Pagination => {
            let _ = ui.pagination(
                &mut state.pagination_page,
                Pagination::new(
                    Id::new("component_showcase_pagination"),
                    PAGINATION_PAGE_COUNT,
                ),
            );

            let page_summary = format!(
                "Page {} of {}",
                state.pagination_page, PAGINATION_PAGE_COUNT
            );
            let _ = ui.label(
                Label::new(page_summary.as_str())
                    .tone(LabelTone::Muted)
                    .size(typography::SMALL_SIZE),
            );
        }
        ComponentKind::AudioPlayback => {
            let available_rect = ui.ui().available_rect_before_wrap();
            let lane_width = 520.0_f32.min(available_rect.width().max(0.0));
            let top = ui.ui().cursor().top().max(available_rect.top());
            let bottom = available_rect.bottom().max(top);
            let lane_rect = egui::Rect::from_min_max(
                egui::pos2(available_rect.center().x - lane_width * 0.5, top),
                egui::pos2(available_rect.center().x + lane_width * 0.5, bottom),
            );
            let mut used_rect = egui::Rect::from_min_size(lane_rect.min, egui::Vec2::ZERO);
            let _ = ui.ui_mut().scope_builder(
                egui::UiBuilder::new()
                    .max_rect(lane_rect)
                    .layout(Layout::top_down(egui::Align::Min)),
                |ui| {
                    let mut ui = ComponentUi::with_overrides(ui, overrides);
                    let _ = ui.label(
                        Label::new("Generation 1")
                            .tone(LabelTone::Secondary)
                            .size(typography::SMALL_SIZE),
                    );
                    ui.add_space(8.0);

                    let (_, playback_result) = ui.audio_playback_with_actions(
                        AudioPlayback::new(
                            Id::new("component_showcase_audio_playback"),
                            state.audio_playback_state,
                        ),
                        |ui| {
                            let mut ui = ui.components();
                            let _ = ui.button(
                                Button::icon_only("share-2")
                                    .variant(ButtonVariant::Ghost)
                                    .size(ControlSize::Sm),
                            );
                            let _ = ui.button(
                                Button::icon_only("download")
                                    .variant(ButtonVariant::Ghost)
                                    .size(ControlSize::Sm),
                            );
                            let _ = ui.button(
                                Button::icon_only("ellipsis")
                                    .variant(ButtonVariant::Ghost)
                                    .size(ControlSize::Sm),
                            );
                        },
                    );

                    if playback_result.play_pause_clicked {
                        state.audio_playback_state = match state.audio_playback_state {
                            AudioPlaybackState::Paused => AudioPlaybackState::Playing,
                            AudioPlaybackState::Playing => AudioPlaybackState::Paused,
                        };
                    }

                    let status = match state.audio_playback_state {
                        AudioPlaybackState::Paused => "Playback: Paused",
                        AudioPlaybackState::Playing => "Playback: Playing",
                    };
                    let _ = ui.label(
                        Label::new(status)
                            .tone(LabelTone::Muted)
                            .size(typography::SMALL_SIZE),
                    );
                    used_rect = ui.ui().min_rect();
                },
            );
            ui.ui_mut().advance_cursor_after_rect(used_rect);
        } // xtask:showcase-render-arms:end
    }
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

fn showcase_metadata(kind: ComponentKind) -> &'static ShowcaseMetadata {
    SHOWCASE_METADATA
        .iter()
        .find(|metadata| metadata.kind == kind)
        .expect("missing showcase metadata")
}

fn showcase_section(kind: ComponentKind) -> ShowcaseSection {
    showcase_metadata(kind).section_override.unwrap_or(
        match catalog_component_definition(kind).group {
            ComponentGroup::Primitive => ShowcaseSection::PrimaryPrimitive,
            ComponentGroup::Composed => ShowcaseSection::DerivedComposed,
        },
    )
}

fn dropdown_action_label(action: Option<usize>) -> &'static str {
    match action.and_then(|id| DROPDOWN_ACTION_LABELS.get(id).copied()) {
        Some(label) => label,
        None => "No action triggered",
    }
}

fn tooltip_placement_from_index(index: usize) -> TooltipPlacement {
    match index {
        1 => TooltipPlacement::Right,
        2 => TooltipPlacement::Bottom,
        3 => TooltipPlacement::Left,
        _ => TooltipPlacement::Top,
    }
}

fn tooltip_placement_index(placement: TooltipPlacement) -> usize {
    match placement {
        TooltipPlacement::Top => 0,
        TooltipPlacement::Right => 1,
        TooltipPlacement::Bottom => 2,
        TooltipPlacement::Left => 3,
        TooltipPlacement::Auto => 0,
    }
}

fn draw_dropdown_shortcut_row(ui: &mut ComponentUi<'_>, action_label: &str, keys: &[&str]) {
    let overrides = ui.overrides();
    ui.ui_mut().horizontal(|ui| {
        let mut ui = ComponentUi::with_overrides(ui, overrides);
        let _ = ui.label(
            Label::new(action_label)
                .tone(LabelTone::Secondary)
                .size(typography::SMALL_SIZE),
        );
        ui.add_space(8.0);
        let _ = ui.kbd_group(KbdGroup::new().gap(3.0), |ui| {
            let mut ui = ui.components();
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    let _ = ui.label(
                        Label::new("+")
                            .tone(LabelTone::Muted)
                            .size(typography::SMALL_SIZE),
                    );
                }
                let _ = ui.kbd(Kbd::new(key).height(18.0).text_size(9.5));
            }
        });
    });
}

fn draw_toolbar_preview(ui: &mut ComponentUi<'_>, state: &mut ComponentShowcaseState) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let dark_mode = runtime.mode.is_dark();
    let overrides = ui.overrides();
    let canvas_fill = if dark_mode {
        tokens::app_background(runtime)
    } else {
        TOOLBAR_CANVAS_LIGHT_FILL
    };

    let _ = egui::Frame::new()
        .fill(canvas_fill)
        .stroke(Stroke::new(1.0, tokens::separator(runtime)))
        .corner_radius(CornerRadius::same(tokens::radius_md(runtime)))
        .inner_margin(egui::Margin::same(12))
        .show(ui.ui_mut(), |ui| {
            let mut ui = ComponentUi::with_overrides(ui, overrides);
            let canvas_size = egui::vec2(ui.available_width(), 220.0);
            let (canvas_rect, _) = ui.allocate_exact_size(canvas_size, Sense::hover());
            let overrides = ui.overrides();
            let _ = ui
                .ui_mut()
                .scope_builder(egui::UiBuilder::new().max_rect(canvas_rect), |ui| {
                    let mut ui = ComponentUi::with_overrides(ui, overrides);
                    let _ = ui.toolbar(
                        Toolbar::new(Id::new("component_showcase_toolbar"))
                            .anchor(Align2::CENTER_TOP)
                            .offset(egui::vec2(0.0, 10.0)),
                        |ui| {
                            let mut ui = ui.components();
                            ui.with_override(
                                ButtonOverride::new()
                                    .min_size(egui::vec2(28.0, 28.0))
                                    .icon_size(12.0),
                                |ui| {
                                    let mut ui = ui.components();
                                    draw_toolbar_contents(&mut ui, state);
                                },
                            )
                        },
                    );
                });
        });
}

fn draw_menu_bar_preview(ui: &mut ComponentUi<'_>, state: &mut ComponentShowcaseState) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let menu_items = [
        MenuBarItem::new("File", &MENU_BAR_FILE_ENTRIES).width(220.0),
        MenuBarItem::new("Edit", &MENU_BAR_EDIT_ENTRIES).width(190.0),
        MenuBarItem::new("View", &MENU_BAR_VIEW_ENTRIES).width(196.0),
        MenuBarItem::new("Object", &MENU_BAR_OBJECT_ENTRIES).width(220.0),
    ];

    let menu_state = egui::Frame::new()
        .fill(tokens::app_background(runtime))
        .stroke(Stroke::new(1.0, tokens::separator(runtime)))
        .corner_radius(CornerRadius::same(tokens::radius_md(runtime)))
        .inner_margin(egui::Margin::same(12))
        .show(ui.ui_mut(), |ui| {
            let mut ui = ui.components();
            ui.menu_bar(MenuBar::new(
                Id::new("component_showcase_menu_bar"),
                &menu_items,
            ))
            .1
        })
        .inner;

    if let Some(action) = menu_state.action {
        state.menu_bar_action = Some(action);
    }
    ui.add_space(8.0);
    let _ = ui.label(
        Label::new(menu_bar_action_label(state.menu_bar_action))
            .tone(LabelTone::Muted)
            .size(typography::SMALL_SIZE),
    );
}

fn draw_image_tile_preview(ui: &mut ComponentUi<'_>, state: &mut ComponentShowcaseState) {
    let showcase_image = egui::include_image!("../../../assets/images/showcase-image.png");
    let logo_image = egui::include_image!("../../../assets/images/clay_logo_large.png");
    let overrides = ui.overrides();

    let _ = ui.label(
        Label::new("Shared spacing, stacked examples")
            .tone(LabelTone::Muted)
            .size(typography::SMALL_SIZE),
    );
    ui.add_space(tokens::SPACING_ITEM_Y);

    let _ = ui.ui_mut().vertical(|ui| {
        let mut ui = ComponentUi::with_overrides(ui, overrides);
        ui.spacing_mut().item_spacing.y = tokens::SPACING_ITEM_Y;

        let _ = ui.label(
            Label::new("Featured tile")
                .tone(LabelTone::Muted)
                .size(typography::SMALL_SIZE),
        );
        let (_, featured_state) = ui.image_tile_with_body(
            ImageTile::new(Image::new(showcase_image.clone())).size(ImageTileSize::Lg),
            |ui| {
                let mut ui = ui.components();
                let _ = ui.label(
                    Label::new("Untitled Design")
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold)
                        .size(16.0),
                );
                draw_image_tile_metadata_row(&mut ui, "Edited 2 days ago");
            },
        );
        if featured_state.tile_clicked {
            state.image_tile_last_action = "Opened Untitled Design".to_owned();
        }

        ui.add_space(tokens::SPACING_ITEM_Y);
        let _ = ui.label(
            Label::new("Custom body")
                .tone(LabelTone::Muted)
                .size(typography::SMALL_SIZE),
        );
        let (_, custom_state) = ui.image_tile_with_body(
            ImageTile::new(Image::new(logo_image.clone())).image_size(egui::vec2(180.0, 120.0)),
            |ui| {
                let mut ui = ui.components();
                let _ = ui.label(
                    Label::new("Custom body")
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold),
                );
                let _ = ui.label(
                    Label::new("Compose badges, helper text, or any other tile footer.")
                        .tone(LabelTone::Muted)
                        .size(typography::SMALL_SIZE),
                );
            },
        );
        if custom_state.tile_clicked {
            state.image_tile_last_action = "Opened custom body tile".to_owned();
        }

        ui.add_space(tokens::SPACING_ITEM_Y);
        let _ = ui.label(
            Label::new("Bodyless grid cell")
                .tone(LabelTone::Muted)
                .size(typography::SMALL_SIZE),
        );
        let (_, bodyless_state) = ui
            .image_tile(ImageTile::new(Image::new(showcase_image.clone())).size(ImageTileSize::Sm));
        if bodyless_state.tile_clicked {
            state.image_tile_last_action = "Opened bodyless tile".to_owned();
        }

        ui.add_space(tokens::SPACING_ITEM_Y);
        let _ = ui.label(
            Label::new("Audio preview")
                .tone(LabelTone::Muted)
                .size(typography::SMALL_SIZE),
        );
        let (_, audio_state) = ui.image_tile_with_body(
            ImageTile::new(Image::new(showcase_image))
                .size(ImageTileSize::Md)
                .playback_state(state.image_tile_playback_state),
            |ui| {
                let mut ui = ui.components();
                let _ = ui.label(
                    Label::new("Ambient Preview")
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold),
                );
                let playback_label = match state.image_tile_playback_state {
                    ImageTilePlaybackState::Paused => "Paused • Click play to preview",
                    ImageTilePlaybackState::Playing => "Playing • 0:27 loop",
                };
                draw_image_tile_metadata_row(&mut ui, playback_label);
            },
        );
        if audio_state.play_pause_clicked {
            state.image_tile_playback_state = match state.image_tile_playback_state {
                ImageTilePlaybackState::Paused => ImageTilePlaybackState::Playing,
                ImageTilePlaybackState::Playing => ImageTilePlaybackState::Paused,
            };
            state.image_tile_last_action = match state.image_tile_playback_state {
                ImageTilePlaybackState::Paused => "Paused audio preview".to_owned(),
                ImageTilePlaybackState::Playing => "Started audio preview".to_owned(),
            };
        } else if audio_state.tile_clicked {
            state.image_tile_last_action = "Opened audio preview tile".to_owned();
        }
    });

    ui.add_space(tokens::SPACING_ITEM_Y);
    let playback_status = match state.image_tile_playback_state {
        ImageTilePlaybackState::Paused => "Paused",
        ImageTilePlaybackState::Playing => "Playing",
    };
    let status_text = format!(
        "Playback: {playback_status} | Last action: {}",
        state.image_tile_last_action
    );
    let _ = ui.label(
        Label::new(status_text.as_str())
            .tone(LabelTone::Muted)
            .size(typography::SMALL_SIZE),
    );
}

fn draw_image_tile_metadata_row(ui: &mut ComponentUi<'_>, text: &str) {
    let overrides = ui.overrides();
    ui.ui_mut().horizontal(|ui| {
        let mut ui = ComponentUi::with_overrides(ui, overrides);
        ui.spacing_mut().item_spacing.x = 6.0;
        let _ = ui.icon(
            crate::components::Icon::new("globe")
                .size(12.0)
                .tint(IMAGE_TILE_META_ACCENT),
        );
        let _ = ui.label(Label::new("•").tone(LabelTone::Muted));
        let _ = ui.label(
            Label::new(text)
                .tone(LabelTone::Muted)
                .size(typography::SMALL_SIZE),
        );
    });
}

fn draw_toolbar_contents(ui: &mut ComponentUi<'_>, state: &mut ComponentShowcaseState) {
    for (index, label) in TOOLBAR_ACTION_OPTIONS.iter().copied().enumerate() {
        let _ = ui.button(Button::new(label).variant(ButtonVariant::Ghost));
        if index == 1 {
            let _ = ui.button(
                Button::icon_only("crown")
                    .variant(ButtonVariant::Ghost)
                    .icon_size(13.0)
                    .icon_tint(Color32::from_rgb(216, 168, 83)),
            );
        }
        if index + 1 < TOOLBAR_ACTION_OPTIONS.len() {
            draw_toolbar_divider(ui);
        }
    }

    draw_toolbar_divider(ui);

    for (index, fill) in TOOLBAR_SWATCHES.iter().copied().enumerate() {
        let runtime = crate::theme::runtime_for_ui(ui);
        let stroke = if state.toolbar_color_index == index {
            Stroke::new(1.0, tokens::text_primary(runtime))
        } else {
            Stroke::NONE
        };
        if ui
            .button(Button::color_only(
                Color::new(fill).size(16.0).stroke(stroke),
            ))
            .clicked()
        {
            state.toolbar_color_index = index;
        }
    }

    draw_toolbar_divider(ui);

    let _ = ui.button(Button::icon_only("square-menu").variant(ButtonVariant::Ghost));
    let _ = ui.button(Button::icon_only("rotate-ccw").variant(ButtonVariant::Ghost));
    draw_toolbar_divider(ui);
    let _ = ui.button(Button::icon_only("crop").variant(ButtonVariant::Ghost));
    let _ = ui.button(Button::new("Flip").variant(ButtonVariant::Ghost));
    draw_toolbar_divider(ui);
    let _ = ui.button(Button::icon_only("grid-3x3").variant(ButtonVariant::Ghost));
    let _ = ui.button(Button::new("Animate").variant(ButtonVariant::Ghost));
    let _ = ui.button(Button::new("Position").variant(ButtonVariant::Ghost));
    draw_toolbar_divider(ui);
    let _ = ui.button(Button::icon_only("paint-roller").variant(ButtonVariant::Ghost));
}

fn draw_toolbar_divider(ui: &mut ComponentUi<'_>) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 16.0), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(1.0, tokens::separator(runtime)),
    );
}

fn clamp_state(state: &mut ComponentShowcaseState) {
    state.button_group_index = state
        .button_group_index
        .min(BUTTON_GROUP_OPTIONS.len().saturating_sub(1));
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
    state.pagination_page = state.pagination_page.clamp(1, PAGINATION_PAGE_COUNT);
    state.slider_value = state.slider_value.clamp(0.0, 100.0);
    state.number_x_value = state.number_x_value.clamp(0.0, 100.0);
    state.number_y_value = state.number_y_value.clamp(0.0, 100.0);
    state.progress_value = state.progress_value.clamp(0.0, 1.0);

    if state
        .select_index
        .is_some_and(|index| index >= SELECT_OPTIONS.len())
    {
        state.select_index = None;
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

fn menu_bar_action_label(action: Option<usize>) -> &'static str {
    match action.and_then(|id| MENU_BAR_ACTION_LABELS.get(id).copied()) {
        Some(label) => label,
        None => "No menu action triggered",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn showcase_metadata_covers_each_catalog_component_once() {
        assert_eq!(
            SHOWCASE_METADATA.len(),
            catalog::component_definitions().count()
        );

        for definition in catalog::component_definitions() {
            assert_eq!(
                SHOWCASE_METADATA
                    .iter()
                    .filter(|metadata| metadata.kind == definition.kind)
                    .count(),
                1,
                "missing or duplicate showcase metadata for {:?}",
                definition.kind
            );
        }
    }

    #[test]
    fn showcase_sections_follow_catalog_groups_and_example_overrides() {
        assert_eq!(
            showcase_section(ComponentKind::Icon),
            ShowcaseSection::PrimaryPrimitive
        );
        assert_eq!(
            showcase_section(ComponentKind::NumberInput),
            ShowcaseSection::PrimaryPrimitive
        );
        assert_eq!(
            showcase_section(ComponentKind::Dialogue),
            ShowcaseSection::DerivedComposed
        );
        assert_eq!(
            showcase_section(ComponentKind::MenuBar),
            ShowcaseSection::Examples
        );
        assert_eq!(
            showcase_section(ComponentKind::Toolbar),
            ShowcaseSection::Examples
        );
        assert_eq!(
            showcase_section(ComponentKind::ImageTile),
            ShowcaseSection::Examples
        );
    }

    #[test]
    fn showcase_component_lists_are_alphabetical_within_sections() {
        for section in [
            ShowcaseSection::PrimaryPrimitive,
            ShowcaseSection::DerivedComposed,
            ShowcaseSection::Examples,
        ] {
            let labels = showcase_component_definitions_by_section(section)
                .map(|definition| definition.label)
                .collect::<Vec<_>>();
            let mut sorted_labels = labels.clone();
            sorted_labels.sort_unstable();
            assert_eq!(
                labels, sorted_labels,
                "{section:?} showcase entries are not alphabetical"
            );
        }
    }
}
