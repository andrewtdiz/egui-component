use crate::components::{
    Button, ButtonStyle, Checkbox, CommandItem, ComponentUi, ComponentUiExt, DialogueStyle,
    DropdownMenuEntry, Kbd, KbdGroup, Label, LabelTone, LabelWeight, NumberInput, NumberInputAxis,
    SwitchSize, TabOption,
};
use crate::ui::tokens;
use egui::{Align2, CornerRadius, CursorIcon, Id, Layout, Stroke, StrokeKind, Ui};

use egui::containers::scroll_area::ScrollSource;

const BUTTON_GROUP_OPTIONS: [&str; 3] = ["Move", "Rotate", "Scale"];
const TAB_OPTIONS: [TabOption<'static>; 3] = [
    TabOption::new(0, "Design"),
    TabOption::new(1, "Code"),
    TabOption::new(2, "History"),
];
const SELECT_OPTIONS: [&str; 4] = ["Draft", "Review", "Approved", "Archived"];
const COMBOBOX_OPTIONS: [&str; 6] = [
    "Material 1",
    "Material Glass",
    "Material Metal",
    "Sprite Atlas",
    "Sprite Mask",
    "UI Text Style",
];
const COMMAND_OPTIONS: [CommandItem<'static>; 8] = [
    CommandItem::new("Scene", "Open Scene Search"),
    CommandItem::new("Scene", "Save Scene"),
    CommandItem::new("GameObject", "Create Empty"),
    CommandItem::new("GameObject", "Add Sprite Renderer"),
    CommandItem::new("View", "Toggle Grid"),
    CommandItem::new("View", "Toggle Gizmos"),
    CommandItem::new("Tools", "Snap to Pixels"),
    CommandItem::new("Tools", "Rebuild Lighting"),
];
const DROPDOWN_INVITE_ENTRIES: [DropdownMenuEntry<'static>; 4] = [
    DropdownMenuEntry::action(4, "Email"),
    DropdownMenuEntry::action(5, "Message"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action(6, "More..."),
];
const DROPDOWN_ENTRIES: [DropdownMenuEntry<'static>; 15] = [
    DropdownMenuEntry::action(0, "My Account"),
    DropdownMenuEntry::action_with_shortcut(1, "Profile", "Shift+Cmd+P"),
    DropdownMenuEntry::action_with_shortcut(2, "Billing", "Cmd+B"),
    DropdownMenuEntry::action_with_shortcut(3, "Settings", "Cmd+S"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::submenu("Invite users", &DROPDOWN_INVITE_ENTRIES),
    DropdownMenuEntry::separator(),
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ShowcaseGroup {
    PrimaryPrimitive,
    DerivedComposed,
}

impl ShowcaseGroup {
    fn title(self) -> &'static str {
        match self {
            Self::PrimaryPrimitive => "Primary / Primitive",
            Self::DerivedComposed => "Derived / Composed",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ShowcaseComponentKind {
    Label,
    Kbd,
    Input,
    Field,
    Button,
    ButtonGroup,
    Checkbox,
    Switch,
    Slider,
    NumberInput,
    Select,
    Tabs,
    Separator,
    Card,
    Progress,
    Tooltip,
    Collapsible,
    DropdownMenu,
    Combobox,
    Command,
    Dialogue,
    Icon,
}

#[derive(Debug, Clone, Copy)]
struct ShowcaseComponentDefinition {
    kind: ShowcaseComponentKind,
    label: &'static str,
    description: &'static str,
    group: ShowcaseGroup,
}

const COMPONENT_DEFINITIONS: [ShowcaseComponentDefinition; 22] = [
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Label,
        label: "Label",
        description: "Text styles and tones",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Kbd,
        label: "Kbd",
        description: "Keyboard keycaps and shortcuts",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Input,
        label: "Input",
        description: "Single-line text input",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Field,
        label: "Field",
        description: "Label + input + helper text",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Button,
        label: "Button",
        description: "Text, icon, and link button variants",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::ButtonGroup,
        label: "Button Group",
        description: "Attached segmented action group",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Checkbox,
        label: "Checkbox",
        description: "Boolean control with label",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Switch,
        label: "Switch",
        description: "Toggle control",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Slider,
        label: "Slider",
        description: "Range input",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::NumberInput,
        label: "Number Input",
        description: "Drag-based numeric input",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Select,
        label: "Select",
        description: "Popup option picker",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Tabs,
        label: "Tabs",
        description: "Segmented tab selector",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Separator,
        label: "Separator",
        description: "Inline divider",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Card,
        label: "Card",
        description: "Container surface",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Progress,
        label: "Progress",
        description: "Completion indicator",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Tooltip,
        label: "Tooltip",
        description: "Hover helper text",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Collapsible,
        label: "Collapsible",
        description: "Expandable content section",
        group: ShowcaseGroup::DerivedComposed,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::DropdownMenu,
        label: "Dropdown Menu",
        description: "Triggered option list",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Combobox,
        label: "Combobox",
        description: "Filterable option list",
        group: ShowcaseGroup::DerivedComposed,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Command,
        label: "Command",
        description: "Searchable command palette",
        group: ShowcaseGroup::DerivedComposed,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Dialogue,
        label: "Dialogue",
        description: "Dialogue window trigger",
        group: ShowcaseGroup::DerivedComposed,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Icon,
        label: "Icon",
        description: "Lucide icon rendering",
        group: ShowcaseGroup::DerivedComposed,
    },
];

#[derive(Debug, Clone)]
pub struct ComponentShowcaseState {
    selected_component: ShowcaseComponentKind,
    input_value: String,
    field_value: String,
    checkbox_value: bool,
    switch_value: bool,
    small_switch_value: bool,
    slider_value: f32,
    number_x_value: f32,
    number_y_value: f32,
    progress_value: f32,
    select_index: Option<usize>,
    tab_index: usize,
    button_group_index: usize,
    collapsible_open: bool,
    dropdown_action: Option<usize>,
    combobox_query: String,
    combobox_index: usize,
    command_query: String,
    dialogue_open: bool,
    alert_dialogue_open: bool,
    tooltip_top_center: bool,
}

impl Default for ComponentShowcaseState {
    fn default() -> Self {
        Self {
            selected_component: ShowcaseComponentKind::Button,
            input_value: "Player_Robot".to_owned(),
            field_value: "M_Robot_Body".to_owned(),
            checkbox_value: true,
            switch_value: true,
            small_switch_value: false,
            slider_value: 62.0,
            number_x_value: 42.0,
            number_y_value: 16.0,
            progress_value: 0.58,
            select_index: Some(1),
            tab_index: 0,
            button_group_index: 0,
            collapsible_open: true,
            dropdown_action: None,
            combobox_query: "mat".to_owned(),
            combobox_index: 0,
            command_query: String::new(),
            dialogue_open: false,
            alert_dialogue_open: false,
            tooltip_top_center: true,
        }
    }
}

pub(super) fn render(ui: &mut Ui, state: &mut ComponentShowcaseState) {
    clamp_state(state);

    let _ = egui::SidePanel::left("component_showcase_sidebar")
        .default_width(238.0)
        .min_width(200.0)
        .max_width(320.0)
        .resizable(true)
        .frame(
            egui::Frame::new()
                .fill(tokens::APP_BACKGROUND)
                .inner_margin(egui::Margin::same(8))
                .stroke(Stroke::new(1.0, tokens::SEPARATOR)),
        )
        .show_inside(ui, |ui| {
            let mut ui = ui.components();
            draw_sidebar(&mut ui, state);
        });

    let _ = egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(tokens::APP_BACKGROUND)
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
            .size(11.0),
    );
    ui.add_space(8.0);
    let _ = ui.separator();
    ui.add_space(8.0);

    let dark_mode = ui.visuals().dark_mode;

    let _ = egui::ScrollArea::vertical()
        .id_salt("component_showcase_sidebar_scroll")
        .scroll_source(ScrollSource {
            drag: false,
            ..ScrollSource::default()
        })
        .auto_shrink([false, false])
        .show(ui.raw_mut(), |ui| {
            let mut ui = ui.components();
            for group in [
                ShowcaseGroup::PrimaryPrimitive,
                ShowcaseGroup::DerivedComposed,
            ] {
                let _ = ui.label(
                    Label::new(group.title())
                        .tone(LabelTone::Muted)
                        .size(11.0)
                        .weight(LabelWeight::Semibold),
                );
                ui.add_space(4.0);

                for definition in component_definitions_by_group(group) {
                    let selected = state.selected_component == definition.kind;
                    if draw_sidebar_component_row(
                        ui.raw_mut(),
                        definition.label,
                        selected,
                        dark_mode,
                    )
                    .clicked()
                    {
                        state.selected_component = definition.kind;
                    }
                }

                ui.add_space(10.0);
            }
        });
}

fn draw_sidebar_component_row(
    ui: &mut Ui,
    label_text: &str,
    selected: bool,
    dark_mode: bool,
) -> egui::Response {
    let desired_size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let fill = tokens::row_bg(
        selected,
        response.is_pointer_button_down_on(),
        response.hovered(),
        dark_mode,
    );
    let stroke = tokens::row_stroke(selected, dark_mode);

    ui.painter().rect(
        rect,
        CornerRadius::same(tokens::RADIUS_SM),
        fill,
        stroke,
        StrokeKind::Outside,
    );

    ui.painter().text(
        egui::pos2(rect.left() + 9.0, rect.center().y),
        Align2::LEFT_CENTER,
        label_text,
        egui::FontId::new(12.0, egui::FontFamily::Proportional),
        if selected {
            tokens::row_selected_text(dark_mode)
        } else {
            tokens::TEXT_SECONDARY
        },
    );

    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn draw_center_preview(ui: &mut ComponentUi<'_>, state: &mut ComponentShowcaseState) {
    let definition = component_definition(state.selected_component);

    let _ = egui::ScrollArea::vertical()
        .id_salt("component_showcase_preview_scroll")
        .scroll_source(ScrollSource {
            drag: false,
            ..ScrollSource::default()
        })
        .auto_shrink([false, false])
        .show(ui.raw_mut(), |ui| {
            let mut ui = ui.components();
            ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                let width = ui.available_width().clamp(260.0, 460.0);

                let _ = ui.card((), |ui| {
                    ui.set_width(width);

                    let _ = ui.label(
                        Label::new(definition.label)
                            .tone(LabelTone::Primary)
                            .weight(LabelWeight::Semibold),
                    );
                    let _ = ui.label(
                        Label::new(definition.description)
                            .tone(LabelTone::Muted)
                            .size(11.0),
                    );
                    ui.add_space(8.0);
                    let _ = ui.separator();
                    ui.add_space(10.0);

                    render_selected_preview(ui, state);
                });
            });
        });
}

fn render_selected_preview(ui: &mut ComponentUi<'_>, state: &mut ComponentShowcaseState) {
    match state.selected_component {
        ShowcaseComponentKind::Label => {
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
        ShowcaseComponentKind::Kbd => {
            let _ = ui.kbd_group(KbdGroup::new(), |ui| {
                let _ = ui.kbd(Kbd::new("⌘"));
                let _ = ui.kbd(Kbd::new("⇧"));
                let _ = ui.kbd(Kbd::new("⌥"));
                let _ = ui.kbd(Kbd::new("⌃"));
            });
            ui.add_space(6.0);
            let _ = ui.kbd_group(KbdGroup::new(), |ui| {
                let _ = ui.kbd(Kbd::new("Ctrl"));
                let _ = ui.label(
                    Label::new("+")
                        .tone(LabelTone::Muted)
                        .weight(LabelWeight::Semibold),
                );
                let _ = ui.kbd(Kbd::new("B"));
            });
        }
        ShowcaseComponentKind::Input => {
            let _ = ui.text_input(&mut state.input_value, (280.0, "Type component name"));
        }
        ShowcaseComponentKind::Field => {
            let _ = ui.field(
                &mut state.field_value,
                ("Material", 280.0, "Assigned material for selected mesh"),
            );
        }
        ShowcaseComponentKind::Button => {
            ui.horizontal(|ui| {
                let _ = ui.button(("Primary", ButtonStyle::Primary));
                let _ = ui.button(("Secondary", ButtonStyle::Secondary));
                let _ = ui.button(("Ghost", ButtonStyle::Ghost));
                let _ = ui.button(("Link", ButtonStyle::Link));
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let _ = ui.button(
                    Button::icon_only("wand-sparkles")
                        .icon_size(15.0)
                        .style(ButtonStyle::Primary),
                );
                let _ = ui.button(
                    Button::icon_only("wand-sparkles")
                        .icon_size(15.0)
                        .style(ButtonStyle::Secondary),
                );
                let _ = ui.button(
                    Button::icon_only("wand-sparkles")
                        .icon_size(15.0)
                        .style(ButtonStyle::Ghost),
                );
                let _ = ui.button(
                    Button::icon_only("wand-sparkles")
                        .icon_size(15.0)
                        .style(ButtonStyle::Link),
                );
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let _ = ui.button(("Create", "plus", ButtonStyle::Primary));
                let _ = ui.button(("Create", "plus", ButtonStyle::Secondary));
                let _ = ui.button(("Create", "plus", ButtonStyle::Ghost));
                let _ = ui.button(("Create", "plus", ButtonStyle::Link));
            });
        }
        ShowcaseComponentKind::ButtonGroup => {
            ui.button_group(
                &mut state.button_group_index,
                (
                    Id::new("component_showcase_button_group"),
                    &BUTTON_GROUP_OPTIONS[..],
                ),
            );
        }
        ShowcaseComponentKind::Checkbox => {
            ui.horizontal(|ui| {
                let mut checkbox_response = ui.checkbox(&mut state.checkbox_value, Checkbox::new());
                let base_label_color = if state.checkbox_value {
                    tokens::TEXT_SECONDARY
                } else {
                    tokens::TEXT_MUTED
                };
                let label_response = ui
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
        ShowcaseComponentKind::Switch => {
            let _ = ui.switch(&mut state.switch_value, "Enable Post FX");
            let _ = ui.switch(
                &mut state.small_switch_value,
                ("Use Compact Handles", SwitchSize::Small),
            );
        }
        ShowcaseComponentKind::Slider => {
            ui.horizontal(|ui| {
                let _ = ui.slider(&mut state.slider_value, (0.0..=100.0, 250.0));
                let value_label = format!("{:.0}", state.slider_value.round());
                let _ = ui.label(
                    Label::new(value_label.as_str())
                        .tone(LabelTone::Secondary)
                        .weight(LabelWeight::Semibold),
                );
            });
        }
        ShowcaseComponentKind::NumberInput => {
            ui.horizontal(|ui| {
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
        ShowcaseComponentKind::Select => {
            let _ = ui.select(
                &mut state.select_index,
                (
                    Id::new("component_showcase_select"),
                    &SELECT_OPTIONS[..],
                    280.0,
                ),
            );

            let selected_label = state
                .select_index
                .and_then(|index| SELECT_OPTIONS.get(index).copied())
                .unwrap_or("None");
            let _ = ui.label(Label::new(selected_label).tone(LabelTone::Muted).size(11.0));
        }
        ShowcaseComponentKind::Tabs => {
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
            let _ = ui.label(Label::new(selected_tab).tone(LabelTone::Muted).size(11.0));
        }
        ShowcaseComponentKind::Separator => {
            let _ = ui.label(Label::new("Above separator").tone(LabelTone::Secondary));
            let _ = ui.separator();
            let _ = ui.label(Label::new("Below separator").tone(LabelTone::Secondary));
        }
        ShowcaseComponentKind::Card => {
            let _ = ui.card(
                (
                    tokens::INPUT_BACKGROUND,
                    Stroke::new(1.0, tokens::INPUT_BORDER),
                ),
                |ui| {
                    ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                        let _ = ui.label(
                            Label::new("Card Title")
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold)
                                .size(15.0),
                        );
                        let _ = ui.label(
                            Label::new("Cards wrap related content in a bordered panel.")
                                .tone(LabelTone::Muted),
                        );
                        ui.add_space(8.0);
                        let _ = ui.separator();
                        ui.add_space(8.0);
                        let footer_size =
                            egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                        let _ = ui.allocate_ui_with_layout(
                            footer_size,
                            Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let _ = ui.button(("Save", ButtonStyle::Primary));
                                let _ = ui.button(("Cancel", ButtonStyle::Secondary));
                            },
                        );
                    });
                },
            );
        }
        ShowcaseComponentKind::Progress => {
            let _ = ui.progress(state.progress_value, (280.0, 10.0));
        }
        ShowcaseComponentKind::Tooltip => {
            let _ = ui.switch(&mut state.tooltip_top_center, "Top-center placement");
            ui.add_space(6.0);
            let _ = ui.tooltip((
                "Hover this trigger",
                "Tooltip content example",
                220.0,
                state.tooltip_top_center,
            ));
        }
        ShowcaseComponentKind::Collapsible => {
            let collapsible_open = state.collapsible_open;
            let _ = ui.collapsible(
                &mut state.collapsible_open,
                (
                    Id::new("component_showcase_collapsible"),
                    "Transform",
                    collapsible_open,
                    "move-3d",
                    "ellipsis_vertical",
                ),
                |ui| {
                    let _ = ui.label(("Position", LabelTone::Secondary));
                    let _ = ui.label(("Rotation", LabelTone::Secondary));
                    let _ = ui.label(("Scale", LabelTone::Secondary));
                },
            );
        }
        ShowcaseComponentKind::DropdownMenu => {
            let (_response, menu_state) = ui.dropdown_menu(("Open", &DROPDOWN_ENTRIES[..], 220.0));
            if let Some(action) = menu_state.action {
                state.dropdown_action = Some(action);
            }
            let _ = ui.label(
                Label::new(dropdown_action_label(state.dropdown_action))
                    .tone(LabelTone::Muted)
                    .size(11.0),
            );
            ui.add_space(8.0);
            let _ = ui.label(
                Label::new("Shortcut keycaps")
                    .tone(LabelTone::Muted)
                    .size(11.0),
            );
            ui.add_space(4.0);
            draw_dropdown_shortcut_row(ui, "New Team", &["⌘", "T"]);
            draw_dropdown_shortcut_row(ui, "Log out", &["⇧", "⌘", "Q"]);
        }
        ShowcaseComponentKind::Combobox => {
            let _ = ui.combobox(
                &mut state.combobox_query,
                &mut state.combobox_index,
                (
                    Id::new("component_showcase_combobox"),
                    &COMBOBOX_OPTIONS[..],
                    280.0,
                ),
            );
            let _ = ui.label(
                Label::new(COMBOBOX_OPTIONS[state.combobox_index])
                    .tone(LabelTone::Muted)
                    .size(11.0),
            );
        }
        ShowcaseComponentKind::Command => {
            let _ = ui.command(
                &mut state.command_query,
                &COMMAND_OPTIONS,
                (Id::new("component_showcase_command"), 280.0),
            );
        }
        ShowcaseComponentKind::Dialogue => {
            ui.horizontal(|ui| {
                if ui
                    .button(("Open Dialogue", ButtonStyle::Secondary))
                    .clicked()
                {
                    state.dialogue_open = true;
                }

                if ui
                    .button(("Open Alert Dialogue", ButtonStyle::Secondary))
                    .clicked()
                {
                    state.alert_dialogue_open = true;
                }
            });

            ui.dialogue_modal(
                &mut state.dialogue_open,
                (Id::new("component_showcase_dialogue"), 380.0),
                |ui, close_requested| {
                    ui.dialogue_header((
                        "Create Component",
                        "Adds the selected component to the active object.",
                    ));
                    ui.add_space(10.0);
                    let _ = ui.vertical(|ui| {
                        let _ = ui.label(
                            Label::new(
                                "Pick a component from the sidebar and confirm to add it to the object.",
                            )
                            .tone(LabelTone::Muted)
                            .size(11.0),
                        );
                    });
                    ui.add_space(10.0);
                    let _ = ui.horizontal(|ui| {
                        if ui.button(("Cancel", ButtonStyle::Secondary)).clicked() {
                            *close_requested = true;
                        }
                        if ui.button(("Create", ButtonStyle::Primary)).clicked() {
                            *close_requested = true;
                        }
                    });
                },
            );

            ui.dialogue_modal(
                &mut state.alert_dialogue_open,
                (Id::new("component_showcase_alert_dialogue"), 380.0),
                |ui, close_requested| {
                    ui.dialogue_header((
                        "Delete Object",
                        "This action cannot be undone.",
                        DialogueStyle::Alert,
                    ));
                    ui.add_space(10.0);
                    let _ = ui.vertical(|ui| {
                        let _ = ui.label(
                            Label::new(
                                "Deleting removes the object and every child object in this hierarchy.",
                            )
                            .tone(LabelTone::Secondary)
                            .size(11.0),
                        );
                    });
                    ui.add_space(10.0);
                    let _ = ui.horizontal(|ui| {
                        if ui.button(("Cancel", ButtonStyle::Secondary)).clicked() {
                            *close_requested = true;
                        }
                        if ui.button(("Delete", ButtonStyle::Primary)).clicked() {
                            *close_requested = true;
                        }
                    });
                },
            );
        }
        ShowcaseComponentKind::Icon => {
            ui.horizontal(|ui| {
                let _ = ui.icon(("bot", 16.0));
                let _ = ui.icon(("settings-2", 16.0));
                let _ = ui.icon(("sparkles", 16.0));
                let _ = ui.icon(("gamepad-2", 16.0));
                let _ = ui.icon(("wand-sparkles", 16.0));
            });
        }
    }
}

fn component_definitions() -> impl Iterator<Item = &'static ShowcaseComponentDefinition> {
    COMPONENT_DEFINITIONS.iter()
}

fn component_definitions_by_group(
    group: ShowcaseGroup,
) -> impl Iterator<Item = &'static ShowcaseComponentDefinition> {
    component_definitions().filter(move |definition| definition.group == group)
}

fn component_definition(kind: ShowcaseComponentKind) -> &'static ShowcaseComponentDefinition {
    COMPONENT_DEFINITIONS
        .iter()
        .find(|definition| definition.kind == kind)
        .expect("missing showcase component definition")
}

fn dropdown_action_label(action: Option<usize>) -> &'static str {
    match action.and_then(|id| DROPDOWN_ACTION_LABELS.get(id).copied()) {
        Some(label) => label,
        None => "No action triggered",
    }
}

fn draw_dropdown_shortcut_row(ui: &mut ComponentUi<'_>, action_label: &str, keys: &[&str]) {
    ui.horizontal(|ui| {
        let _ = ui.label(
            Label::new(action_label)
                .tone(LabelTone::Secondary)
                .size(11.0),
        );
        ui.add_space(8.0);
        let _ = ui.kbd_group(KbdGroup::new().gap(3.0), |ui| {
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    let _ = ui.label(Label::new("+").tone(LabelTone::Muted).size(10.0));
                }
                let _ = ui.kbd(Kbd::new(key).height(18.0).text_size(9.5));
            }
        });
    });
}

fn clamp_state(state: &mut ComponentShowcaseState) {
    state.button_group_index = state
        .button_group_index
        .min(BUTTON_GROUP_OPTIONS.len().saturating_sub(1));
    state.tab_index = state.tab_index.min(TAB_OPTIONS.len().saturating_sub(1));
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
    state.combobox_index = state
        .combobox_index
        .min(COMBOBOX_OPTIONS.len().saturating_sub(1));
}
