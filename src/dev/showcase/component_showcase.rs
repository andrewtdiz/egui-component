use crate::components::{
    button, button_group, card, checkbox, collapsible, combobox, command, context_menu, dialog,
    dropdown_menu, field, icon, label, number_input, progress, resizable, scroll_area, select,
    separator, slider, switch, tabs, text_input, tooltip, ButtonGroupProps, ButtonProps,
    ButtonVariant, CardProps, CheckboxProps, CollapsibleProps, ComboboxProps, CommandItem,
    CommandProps, ContextMenuAction, ContextMenuProps, DialogProps, DialogVariant,
    DropdownMenuProps, FieldProps, IconProps, LabelProps, LabelTone, LabelWeight, NumberInputAxis,
    NumberInputProps, ProgressProps, ResizableProps, ScrollAreaProps, SelectProps, SliderProps,
    SwitchProps, SwitchSize, TabOption, TextInputProps, TooltipProps,
};
use crate::ui::tokens;
use egui::{Align2, Color32, CornerRadius, CursorIcon, Id, Layout, Stroke, StrokeKind, Ui};

const BUTTON_GROUP_OPTIONS: [&str; 3] = ["Move", "Rotate", "Scale"];
const TAB_OPTIONS: [TabOption<'static>; 3] = [
    TabOption::new(0, "Design"),
    TabOption::new(1, "Code"),
    TabOption::new(2, "History"),
];
const SELECT_OPTIONS: [&str; 4] = ["Draft", "Review", "Approved", "Archived"];
const DROPDOWN_OPTIONS: [&str; 4] = ["Create Material", "Create Script", "Duplicate", "Delete"];
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
    ScrollArea,
    Resizable,
    Tooltip,
    Collapsible,
    ContextMenu,
    DropdownMenu,
    Combobox,
    Command,
    Dialog,
    Icon,
}

#[derive(Debug, Clone, Copy)]
struct ShowcaseComponentDefinition {
    kind: ShowcaseComponentKind,
    label: &'static str,
    description: &'static str,
    group: ShowcaseGroup,
}

const COMPONENT_DEFINITIONS: [ShowcaseComponentDefinition; 24] = [
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Label,
        label: "Label",
        description: "Text styles and tones",
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
        kind: ShowcaseComponentKind::ScrollArea,
        label: "Scroll Area",
        description: "Vertical scrolling container",
        group: ShowcaseGroup::PrimaryPrimitive,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::Resizable,
        label: "Resizable",
        description: "Draggable resize handle container",
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
        kind: ShowcaseComponentKind::ContextMenu,
        label: "Context Menu",
        description: "Right-click menu actions",
        group: ShowcaseGroup::DerivedComposed,
    },
    ShowcaseComponentDefinition {
        kind: ShowcaseComponentKind::DropdownMenu,
        label: "Dropdown Menu",
        description: "Triggered option list",
        group: ShowcaseGroup::DerivedComposed,
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
        kind: ShowcaseComponentKind::Dialog,
        label: "Dialog",
        description: "Modal window trigger",
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
    context_menu_toggle: bool,
    context_menu_action: Option<ContextMenuAction>,
    dropdown_index: usize,
    combobox_query: String,
    combobox_index: usize,
    command_query: String,
    dialog_open: bool,
    alert_dialog_open: bool,
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
            context_menu_toggle: true,
            context_menu_action: None,
            dropdown_index: 0,
            combobox_query: "mat".to_owned(),
            combobox_index: 0,
            command_query: String::new(),
            dialog_open: false,
            alert_dialog_open: false,
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
            draw_sidebar(ui, state);
        });

    let _ = egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(tokens::APP_BACKGROUND)
                .inner_margin(egui::Margin::same(12)),
        )
        .show_inside(ui, |ui| {
            draw_center_preview(ui, state);
        });
}

fn draw_sidebar(ui: &mut Ui, state: &mut ComponentShowcaseState) {
    let _ = label(
        ui,
        LabelProps::new("Components")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    let _ = label(
        ui,
        LabelProps::new("Select a component to preview")
            .tone(LabelTone::Muted)
            .size(11.0),
    );
    ui.add_space(8.0);
    let _ = separator(ui);
    ui.add_space(8.0);

    let dark_mode = ui.visuals().dark_mode;

    let _ = egui::ScrollArea::vertical()
        .id_salt("component_showcase_sidebar_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for group in [
                ShowcaseGroup::PrimaryPrimitive,
                ShowcaseGroup::DerivedComposed,
            ] {
                let _ = label(
                    ui,
                    LabelProps::new(group.title())
                        .tone(LabelTone::Muted)
                        .size(11.0)
                        .weight(LabelWeight::Semibold),
                );
                ui.add_space(4.0);

                for definition in component_definitions_by_group(group) {
                    let selected = state.selected_component == definition.kind;
                    if draw_sidebar_component_row(ui, definition.label, selected, dark_mode)
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

    let fill = if selected {
        tokens::row_selected_bg(dark_mode)
    } else if response.hovered() {
        tokens::ROW_HOVER_BG
    } else {
        Color32::TRANSPARENT
    };

    ui.painter().rect(
        rect,
        CornerRadius::same(tokens::RADIUS_SM),
        fill,
        Stroke::NONE,
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

fn draw_center_preview(ui: &mut Ui, state: &mut ComponentShowcaseState) {
    let definition = component_definition(state.selected_component);

    let _ = egui::ScrollArea::vertical()
        .id_salt("component_showcase_preview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                let width = ui.available_width().min(460.0).max(260.0);

                let _ = card(
                    ui,
                    CardProps::new()
                        .fill(tokens::MUTED_SURFACE)
                        .stroke(Stroke::new(1.0, tokens::SEPARATOR)),
                    |ui| {
                        ui.set_width(width);

                        let _ = label(
                            ui,
                            LabelProps::new(definition.label)
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );
                        let _ = label(
                            ui,
                            LabelProps::new(definition.description)
                                .tone(LabelTone::Muted)
                                .size(11.0),
                        );
                        ui.add_space(8.0);
                        let _ = separator(ui);
                        ui.add_space(10.0);

                        render_selected_preview(ui, state);
                    },
                );
            });
        });
}

fn render_selected_preview(ui: &mut Ui, state: &mut ComponentShowcaseState) {
    match state.selected_component {
        ShowcaseComponentKind::Label => {
            let _ = label(
                ui,
                LabelProps::new("Primary label")
                    .tone(LabelTone::Primary)
                    .weight(LabelWeight::Semibold),
            );
            let _ = label(
                ui,
                LabelProps::new("Secondary label").tone(LabelTone::Secondary),
            );
            let _ = label(
                ui,
                LabelProps::new("Muted helper text").tone(LabelTone::Muted),
            );
            let _ = label(
                ui,
                LabelProps::new("Destructive text")
                    .tone(LabelTone::Destructive)
                    .weight(LabelWeight::Semibold),
            );
        }
        ShowcaseComponentKind::Input => {
            let _ = text_input(
                ui,
                &mut state.input_value,
                TextInputProps::new()
                    .width(280.0)
                    .hint_text("Type component name"),
            );
        }
        ShowcaseComponentKind::Field => {
            let _ = field(
                ui,
                &mut state.field_value,
                FieldProps::new("Material")
                    .width(280.0)
                    .helper_text("Assigned material for selected mesh"),
            );
        }
        ShowcaseComponentKind::Button => {
            ui.horizontal(|ui| {
                let _ = button(ui, ButtonProps::new("Primary"));
                let _ = button(
                    ui,
                    ButtonProps::new("Secondary").variant(ButtonVariant::Secondary),
                );
                let _ = button(ui, ButtonProps::new("Ghost").variant(ButtonVariant::Ghost));
                let _ = button(ui, ButtonProps::new("Link").variant(ButtonVariant::Link));
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let _ = button(ui, ButtonProps::icon_only("wand-sparkles").icon_size(15.0));
                let _ = button(
                    ui,
                    ButtonProps::icon_only("wand-sparkles")
                        .icon_size(15.0)
                        .variant(ButtonVariant::Secondary),
                );
                let _ = button(
                    ui,
                    ButtonProps::icon_only("wand-sparkles")
                        .icon_size(15.0)
                        .variant(ButtonVariant::Ghost),
                );
                let _ = button(
                    ui,
                    ButtonProps::icon_only("wand-sparkles")
                        .icon_size(15.0)
                        .variant(ButtonVariant::Link),
                );
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let _ = button(
                    ui,
                    ButtonProps::new("Create")
                        .icon("plus")
                        .variant(ButtonVariant::Primary),
                );
                let _ = button(
                    ui,
                    ButtonProps::new("Create")
                        .icon("plus")
                        .variant(ButtonVariant::Secondary),
                );
                let _ = button(
                    ui,
                    ButtonProps::new("Create")
                        .icon("plus")
                        .variant(ButtonVariant::Ghost),
                );
                let _ = button(
                    ui,
                    ButtonProps::new("Create")
                        .icon("plus")
                        .variant(ButtonVariant::Link),
                );
            });
        }
        ShowcaseComponentKind::ButtonGroup => {
            button_group(
                ui,
                &mut state.button_group_index,
                ButtonGroupProps::new(
                    Id::new("component_showcase_button_group"),
                    &BUTTON_GROUP_OPTIONS,
                ),
            );
        }
        ShowcaseComponentKind::Checkbox => {
            ui.horizontal(|ui| {
                let mut checkbox_response =
                    checkbox(ui, &mut state.checkbox_value, CheckboxProps::new());
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
            let _ = switch(
                ui,
                &mut state.switch_value,
                SwitchProps::new().label("Enable Post FX"),
            );
            let _ = switch(
                ui,
                &mut state.small_switch_value,
                SwitchProps::new()
                    .label("Use Compact Handles")
                    .size(SwitchSize::Small),
            );
        }
        ShowcaseComponentKind::Slider => {
            ui.horizontal(|ui| {
                let _ = slider(
                    ui,
                    &mut state.slider_value,
                    SliderProps::new(0.0..=100.0).width(250.0),
                );
                let value_label = format!("{:.0}", state.slider_value.round());
                let _ = label(
                    ui,
                    LabelProps::new(value_label.as_str())
                        .tone(LabelTone::Secondary)
                        .weight(LabelWeight::Semibold),
                );
            });
        }
        ShowcaseComponentKind::NumberInput => {
            ui.horizontal(|ui| {
                let _ = number_input(
                    ui,
                    &mut state.number_x_value,
                    NumberInputProps::new(Id::new("component_showcase_number_x"))
                        .width(110.0)
                        .range(0.0..=100.0)
                        .decimals(1)
                        .prefix("X")
                        .prefix_tint(tokens::GAME_ENGINE_GREEN)
                        .prefix_align_left()
                        .axis(NumberInputAxis::Horizontal),
                );
                let _ = number_input(
                    ui,
                    &mut state.number_y_value,
                    NumberInputProps::new(Id::new("component_showcase_number_y"))
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
            let _ = select(
                ui,
                &mut state.select_index,
                SelectProps::new(
                    Id::new("component_showcase_select_trigger"),
                    Id::new("component_showcase_select_popup"),
                    &SELECT_OPTIONS,
                )
                .width(280.0),
            );

            let selected_label = state
                .select_index
                .and_then(|index| SELECT_OPTIONS.get(index).copied())
                .unwrap_or("None");
            let _ = label(
                ui,
                LabelProps::new(selected_label)
                    .tone(LabelTone::Muted)
                    .size(11.0),
            );
        }
        ShowcaseComponentKind::Tabs => {
            tabs(
                ui,
                Id::new("component_showcase_tabs"),
                &mut state.tab_index,
                &TAB_OPTIONS,
            );

            let selected_tab = TAB_OPTIONS
                .iter()
                .find(|option| option.value == state.tab_index)
                .map(|option| option.label)
                .unwrap_or(TAB_OPTIONS[0].label);
            let _ = label(
                ui,
                LabelProps::new(selected_tab)
                    .tone(LabelTone::Muted)
                    .size(11.0),
            );
        }
        ShowcaseComponentKind::Separator => {
            let _ = label(
                ui,
                LabelProps::new("Above separator").tone(LabelTone::Secondary),
            );
            let _ = separator(ui);
            let _ = label(
                ui,
                LabelProps::new("Below separator").tone(LabelTone::Secondary),
            );
        }
        ShowcaseComponentKind::Card => {
            let _ = card(
                ui,
                CardProps::new()
                    .fill(tokens::INPUT_BACKGROUND)
                    .stroke(Stroke::new(1.0, tokens::INPUT_BORDER)),
                |ui| {
                    ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                        let _ = label(
                            ui,
                            LabelProps::new("Card Title")
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold)
                                .size(15.0),
                        );
                        let _ = label(
                            ui,
                            LabelProps::new("Cards wrap related content in a bordered panel.")
                                .tone(LabelTone::Muted),
                        );
                        ui.add_space(8.0);
                        let _ = separator(ui);
                        ui.add_space(8.0);
                        let footer_size =
                            egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                        let _ = ui.allocate_ui_with_layout(
                            footer_size,
                            Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let _ = button(ui, ButtonProps::new("Save"));
                                let _ = button(
                                    ui,
                                    ButtonProps::new("Cancel").variant(ButtonVariant::Secondary),
                                );
                            },
                        );
                    });
                },
            );
        }
        ShowcaseComponentKind::Progress => {
            let _ = progress(
                ui,
                state.progress_value,
                ProgressProps::new().width(280.0).height(10.0),
            );
        }
        ShowcaseComponentKind::ScrollArea => {
            let _ = card(
                ui,
                CardProps::new()
                    .fill(tokens::INPUT_BACKGROUND)
                    .stroke(Stroke::new(1.0, tokens::INPUT_BORDER)),
                |ui| {
                    scroll_area(
                        ui,
                        ScrollAreaProps::new(Id::new("component_showcase_scroll_area"))
                            .max_height(110.0),
                        |ui| {
                            for index in 0..12 {
                                let row = format!("Scrollable row {}", index + 1);
                                let _ = label(ui, LabelProps::new(row.as_str()));
                            }
                        },
                    );
                },
            );
        }
        ShowcaseComponentKind::Resizable => {
            let _ = card(
                ui,
                CardProps::new()
                    .fill(tokens::INPUT_BACKGROUND)
                    .stroke(Stroke::new(1.0, tokens::INPUT_BORDER)),
                |ui| {
                    resizable(
                        ui,
                        ResizableProps::new(Id::new("component_showcase_resizable")),
                        |ui| {
                            let _ = label(
                                ui,
                                LabelProps::new("Drag the bottom-right handle")
                                    .tone(LabelTone::Secondary),
                            );
                            let _ = label(
                                ui,
                                LabelProps::new("Min/Max size is constrained")
                                    .tone(LabelTone::Muted),
                            );
                        },
                    );
                },
            );
        }
        ShowcaseComponentKind::Tooltip => {
            let _ = switch(
                ui,
                &mut state.tooltip_top_center,
                SwitchProps::new().label("Top-center placement"),
            );
            ui.add_space(6.0);
            let _ = tooltip(
                ui,
                TooltipProps::new("Hover this trigger", "Tooltip content example")
                    .width(220.0)
                    .top_center(state.tooltip_top_center),
            );
        }
        ShowcaseComponentKind::Collapsible => {
            let collapsible_open = state.collapsible_open;
            let _ = collapsible(
                ui,
                &mut state.collapsible_open,
                CollapsibleProps::new(Id::new("component_showcase_collapsible"), "Transform")
                    .open(collapsible_open)
                    .leading_icon("move-3d")
                    .trailing_icon("ellipsis_vertical"),
                |ui| {
                    let _ = label(ui, LabelProps::new("Position").tone(LabelTone::Secondary));
                    let _ = label(ui, LabelProps::new("Rotation").tone(LabelTone::Secondary));
                    let _ = label(ui, LabelProps::new("Scale").tone(LabelTone::Secondary));
                },
            );
        }
        ShowcaseComponentKind::ContextMenu => {
            let (_response, menu_state) = context_menu(
                ui,
                "Visible",
                &mut state.context_menu_toggle,
                ContextMenuProps::new("Right click for menu").width(220.0),
            );
            if let Some(action) = menu_state.action {
                state.context_menu_action = Some(action);
            }

            let _ = label(
                ui,
                LabelProps::new(context_action_label(state.context_menu_action))
                    .tone(LabelTone::Muted)
                    .size(11.0),
            );
        }
        ShowcaseComponentKind::DropdownMenu => {
            let _ = dropdown_menu(
                ui,
                &mut state.dropdown_index,
                DropdownMenuProps::new("Actions", &DROPDOWN_OPTIONS),
            );
            let _ = label(
                ui,
                LabelProps::new(DROPDOWN_OPTIONS[state.dropdown_index])
                    .tone(LabelTone::Muted)
                    .size(11.0),
            );
        }
        ShowcaseComponentKind::Combobox => {
            let _ = combobox(
                ui,
                &mut state.combobox_query,
                &mut state.combobox_index,
                ComboboxProps::new(Id::new("component_showcase_combobox"), &COMBOBOX_OPTIONS)
                    .width(280.0),
            );
            let _ = label(
                ui,
                LabelProps::new(COMBOBOX_OPTIONS[state.combobox_index])
                    .tone(LabelTone::Muted)
                    .size(11.0),
            );
        }
        ShowcaseComponentKind::Command => {
            let _ = command(
                ui,
                &mut state.command_query,
                &COMMAND_OPTIONS,
                CommandProps::new(Id::new("component_showcase_command")).width(280.0),
            );
        }
        ShowcaseComponentKind::Dialog => {
            ui.horizontal(|ui| {
                let _ = dialog(
                    ui,
                    &mut state.dialog_open,
                    DialogProps::new(Id::new("component_showcase_dialog"), "Create Component")
                        .description("Adds the selected component to the active object.")
                        .trigger_label("Open Dialog")
                        .confirm_label("Create"),
                );

                let _ = dialog(
                    ui,
                    &mut state.alert_dialog_open,
                    DialogProps::new(Id::new("component_showcase_alert_dialog"), "Delete Object")
                        .description("This action cannot be undone.")
                        .trigger_label("Open Alert")
                        .confirm_label("Delete")
                        .variant(DialogVariant::Alert),
                );
            });
        }
        ShowcaseComponentKind::Icon => {
            ui.horizontal(|ui| {
                let _ = icon(ui, IconProps::new("bot").size(16.0));
                let _ = icon(ui, IconProps::new("settings-2").size(16.0));
                let _ = icon(ui, IconProps::new("sparkles").size(16.0));
                let _ = icon(ui, IconProps::new("gamepad-2").size(16.0));
                let _ = icon(ui, IconProps::new("wand-sparkles").size(16.0));
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

fn context_action_label(action: Option<ContextMenuAction>) -> &'static str {
    match action {
        Some(ContextMenuAction::First) => "Last action: Rename",
        Some(ContextMenuAction::Second) => "Last action: Duplicate",
        Some(ContextMenuAction::Third) => "Last action: Delete",
        None => "Last action: None",
    }
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

    state.dropdown_index = state
        .dropdown_index
        .min(DROPDOWN_OPTIONS.len().saturating_sub(1));
    state.combobox_index = state
        .combobox_index
        .min(COMBOBOX_OPTIONS.len().saturating_sub(1));
}
