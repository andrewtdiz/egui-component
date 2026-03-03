use crate::editor::ui::chrome::{styled_checkbox, with_input_chrome};
use crate::editor::ui::icons;
use crate::editor::ui::tokens;
use egui::{Color32, CornerRadius, CursorIcon, Id, RichText, Sense, Stroke, StrokeKind, Ui};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ComponentKind {
    Accordion,
    Alert,
    AlertDialog,
    AspectRatio,
    Avatar,
    Badge,
    Breadcrumb,
    Button,
    ButtonGroup,
    Calendar,
    Card,
    Carousel,
    Chart,
    Checkbox,
    Collapsible,
    Combobox,
    Command,
    ContextMenu,
    DataTable,
    DatePicker,
    Dialog,
    Direction,
    Drawer,
    DropdownMenu,
    Empty,
    Field,
    HoverCard,
    Input,
    InputGroup,
    InputOtp,
    Item,
    Kbd,
    Label,
    Menubar,
    NativeSelect,
    NavigationMenu,
    Pagination,
    Popover,
    Progress,
    RadioGroup,
    Resizable,
    ScrollArea,
    Select,
    Separator,
    Sheet,
    Sidebar,
    Skeleton,
    Slider,
    Sonner,
    Spinner,
    Switch,
    Table,
    Tabs,
    Textarea,
    Toast,
    Toggle,
    ToggleGroup,
    Tooltip,
    Typography,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ComponentGroup {
    PrimaryPrimitive,
    Composed,
}

impl ComponentGroup {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::PrimaryPrimitive => "Primary / Primitive",
            Self::Composed => "Composed",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ComponentDefinition {
    pub(crate) kind: ComponentKind,
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) implemented: bool,
}

const COMPONENT_DEFINITIONS: [ComponentDefinition; 59] = [
    ComponentDefinition {
        kind: ComponentKind::Accordion,
        id: "accordion",
        label: "Accordion",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Alert,
        id: "alert",
        label: "Alert",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::AlertDialog,
        id: "alert-dialog",
        label: "Alert Dialog",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::AspectRatio,
        id: "aspect-ratio",
        label: "Aspect Ratio",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Avatar,
        id: "avatar",
        label: "Avatar",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Badge,
        id: "badge",
        label: "Badge",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Breadcrumb,
        id: "breadcrumb",
        label: "Breadcrumb",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Button,
        id: "button",
        label: "Button",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::ButtonGroup,
        id: "button-group",
        label: "Button Group",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Calendar,
        id: "calendar",
        label: "Calendar",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Card,
        id: "card",
        label: "Card",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Carousel,
        id: "carousel",
        label: "Carousel",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Chart,
        id: "chart",
        label: "Chart",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Checkbox,
        id: "checkbox",
        label: "Checkbox",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Collapsible,
        id: "collapsible",
        label: "Collapsible",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Combobox,
        id: "combobox",
        label: "Combobox",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Command,
        id: "command",
        label: "Command",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::ContextMenu,
        id: "context-menu",
        label: "Context Menu",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::DataTable,
        id: "data-table",
        label: "Data Table",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::DatePicker,
        id: "date-picker",
        label: "Date Picker",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Dialog,
        id: "dialog",
        label: "Dialog",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Direction,
        id: "direction",
        label: "Direction",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Drawer,
        id: "drawer",
        label: "Drawer",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::DropdownMenu,
        id: "dropdown-menu",
        label: "Dropdown Menu",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Empty,
        id: "empty",
        label: "Empty",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Field,
        id: "field",
        label: "Field",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::HoverCard,
        id: "hover-card",
        label: "Hover Card",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Input,
        id: "input",
        label: "Input",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::InputGroup,
        id: "input-group",
        label: "Input Group",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::InputOtp,
        id: "input-otp",
        label: "Input OTP",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Item,
        id: "item",
        label: "Item",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Kbd,
        id: "kbd",
        label: "Kbd",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Label,
        id: "label",
        label: "Label",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Menubar,
        id: "menubar",
        label: "Menubar",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::NativeSelect,
        id: "native-select",
        label: "Native Select",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::NavigationMenu,
        id: "navigation-menu",
        label: "Navigation Menu",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Pagination,
        id: "pagination",
        label: "Pagination",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Popover,
        id: "popover",
        label: "Popover",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Progress,
        id: "progress",
        label: "Progress",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::RadioGroup,
        id: "radio-group",
        label: "Radio Group",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Resizable,
        id: "resizable",
        label: "Resizable",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::ScrollArea,
        id: "scroll-area",
        label: "Scroll Area",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Select,
        id: "select",
        label: "Select",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Separator,
        id: "separator",
        label: "Separator",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Sheet,
        id: "sheet",
        label: "Sheet",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Sidebar,
        id: "sidebar",
        label: "Sidebar",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Skeleton,
        id: "skeleton",
        label: "Skeleton",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Slider,
        id: "slider",
        label: "Slider",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Sonner,
        id: "sonner",
        label: "Sonner",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Spinner,
        id: "spinner",
        label: "Spinner",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Switch,
        id: "switch",
        label: "Switch",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Table,
        id: "table",
        label: "Table",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Tabs,
        id: "tabs",
        label: "Tabs",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Textarea,
        id: "textarea",
        label: "Textarea",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Toast,
        id: "toast",
        label: "Toast",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Toggle,
        id: "toggle",
        label: "Toggle",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::ToggleGroup,
        id: "toggle-group",
        label: "Toggle Group",
        implemented: false,
    },
    ComponentDefinition {
        kind: ComponentKind::Tooltip,
        id: "tooltip",
        label: "Tooltip",
        implemented: true,
    },
    ComponentDefinition {
        kind: ComponentKind::Typography,
        id: "typography",
        label: "Typography",
        implemented: false,
    },
];

const BUTTON_GROUP_OPTIONS: &[(usize, &str)] = &[(0, "Primary"), (1, "Secondary"), (2, "Ghost")];
const TAB_OPTIONS: &[(usize, &str)] = &[(0, "Design"), (1, "Code"), (2, "History")];
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
const COMMAND_OPTIONS: [(&str, &str); 8] = [
    ("Scene", "Open Scene Search"),
    ("Scene", "Save Scene"),
    ("GameObject", "Create Empty"),
    ("GameObject", "Add Sprite Renderer"),
    ("View", "Toggle Grid"),
    ("View", "Toggle Gizmos"),
    ("Tools", "Snap to Pixels"),
    ("Tools", "Rebuild Lighting"),
];
const DEFAULT_TOOLTIP_DELAY_MS: u32 = 75;

#[derive(Debug, Clone, Copy)]
struct TooltipPreviewOptions {
    delay_ms: Option<u32>,
    top_center: bool,
}

impl TooltipPreviewOptions {
    fn resolved_delay_ms(self) -> u32 {
        self.delay_ms.unwrap_or(DEFAULT_TOOLTIP_DELAY_MS)
    }
}

impl ComponentKind {
    pub(crate) fn display_name(self) -> &'static str {
        component_definition(self).label
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ComponentStoryState {
    pub(crate) input_value: String,
    pub(crate) field_value: String,
    pub(crate) checkbox_value: bool,
    pub(crate) switch_value: bool,
    pub(crate) slider_value: f32,
    pub(crate) progress_value: f32,
    pub(crate) select_index: Option<usize>,
    pub(crate) tabs_index: usize,
    pub(crate) tooltip_top_center: bool,
    pub(crate) collapsible_open: bool,
    pub(crate) dropdown_index: usize,
    pub(crate) combobox_query: String,
    pub(crate) combobox_index: usize,
    pub(crate) command_query: String,
    pub(crate) dialog_open: bool,
    pub(crate) alert_dialog_open: bool,
    pub(crate) context_menu_toggle: bool,
}

impl Default for ComponentStoryState {
    fn default() -> Self {
        Self {
            input_value: "Sphere".to_owned(),
            field_value: "Material 1".to_owned(),
            checkbox_value: true,
            switch_value: true,
            slider_value: 60.0,
            progress_value: 42.0,
            select_index: None,
            tabs_index: 0,
            tooltip_top_center: true,
            collapsible_open: true,
            dropdown_index: 0,
            combobox_query: "mat".to_owned(),
            combobox_index: 0,
            command_query: String::new(),
            dialog_open: false,
            alert_dialog_open: false,
            context_menu_toggle: true,
        }
    }
}

pub(crate) fn supported_component_definitions() -> impl Iterator<Item = &'static ComponentDefinition>
{
    COMPONENT_DEFINITIONS
        .iter()
        .filter(|definition| definition.implemented)
}

pub(crate) fn supported_component_definitions_by_group(
    group: ComponentGroup,
) -> impl Iterator<Item = &'static ComponentDefinition> {
    supported_component_definitions()
        .filter(move |definition| component_group(definition.kind) == group)
}

pub(crate) fn parse_component_kind(value: &str) -> Option<ComponentKind> {
    let normalized = normalize_component_id(value);
    supported_component_definitions()
        .find(|definition| normalize_component_id(definition.id) == normalized)
        .map(|definition| definition.kind)
}

pub(crate) fn supported_component_ids_csv() -> String {
    supported_component_definitions()
        .map(|definition| definition.id)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn render_component_preview(
    ui: &mut Ui,
    component: ComponentKind,
    story: &mut ComponentStoryState,
) {
    match component {
        ComponentKind::Label => draw_label_preview(ui),
        ComponentKind::Input => draw_input_preview(ui, story),
        ComponentKind::Field => draw_field_preview(ui, story),
        ComponentKind::Button => draw_button_preview(ui),
        ComponentKind::ButtonGroup => draw_button_group_preview(ui),
        ComponentKind::Checkbox => draw_checkbox_preview(ui, story),
        ComponentKind::Switch => draw_switch_preview(ui, story),
        ComponentKind::Slider => draw_slider_preview(ui, story),
        ComponentKind::Progress => draw_progress_preview(ui, story),
        ComponentKind::Select => draw_select_preview(ui, story),
        ComponentKind::Tabs => draw_tabs_preview(ui, story),
        ComponentKind::Separator => draw_separator_preview(ui),
        ComponentKind::Card => draw_card_preview(ui),
        ComponentKind::ScrollArea => draw_scroll_area_preview(ui),
        ComponentKind::Resizable => draw_resizable_preview(ui),
        ComponentKind::Tooltip => draw_tooltip_preview(ui, story),
        ComponentKind::Collapsible => draw_collapsible_preview(ui, story),
        ComponentKind::ContextMenu => draw_context_menu_preview(ui, story),
        ComponentKind::DropdownMenu => draw_dropdown_menu_preview(ui, story),
        ComponentKind::Combobox => draw_combobox_preview(ui, story),
        ComponentKind::Command => draw_command_preview(ui, story),
        ComponentKind::Dialog => draw_dialog_preview(ui, story, false),
        ComponentKind::AlertDialog => draw_dialog_preview(ui, story, true),
        _ => draw_unimplemented_preview(ui, component),
    }
}

fn normalize_component_id(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn component_group(kind: ComponentKind) -> ComponentGroup {
    match kind {
        ComponentKind::Collapsible
        | ComponentKind::ContextMenu
        | ComponentKind::DropdownMenu
        | ComponentKind::Combobox
        | ComponentKind::Command
        | ComponentKind::Dialog
        | ComponentKind::AlertDialog => ComponentGroup::Composed,
        _ => ComponentGroup::PrimaryPrimitive,
    }
}

fn component_definition(kind: ComponentKind) -> &'static ComponentDefinition {
    COMPONENT_DEFINITIONS
        .iter()
        .find(|definition| definition.kind == kind)
        .unwrap_or(&COMPONENT_DEFINITIONS[0])
}

fn draw_preview_title(ui: &mut Ui, label: &str) {
    ui.label(
        RichText::new(label)
            .font(egui::FontId::new(
                12.0,
                egui::FontFamily::Name("clay-editor-segoe-semibold".into()),
            ))
            .color(tokens::TEXT_PRIMARY),
    );
}

fn draw_preview_subtitle(ui: &mut Ui, label: &str) {
    ui.label(
        RichText::new(label)
            .font(egui::FontId::new(
                11.0,
                egui::FontFamily::Name("clay-editor-segoe-semibold".into()),
            ))
            .color(tokens::TEXT_SECONDARY),
    );
}

fn draw_preview_muted(ui: &mut Ui, text: impl Into<String>) {
    ui.label(
        RichText::new(text.into())
            .size(11.0)
            .color(tokens::TEXT_MUTED),
    );
}

fn draw_preview_mock_frame(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(tokens::MUTED_SURFACE)
        .stroke(Stroke::new(1.0, tokens::CARD_BORDER))
        .corner_radius(CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(8, 8))
        .show(ui, add);
}

fn draw_unimplemented_preview(ui: &mut Ui, component: ComponentKind) {
    draw_preview_title(ui, component.display_name());
    ui.add_space(8.0);
    draw_preview_muted(ui, "Not implemented in this focused subset");
}

fn draw_label_preview(ui: &mut Ui) {
    draw_preview_title(ui, "Label");
    ui.add_space(8.0);
    ui.label(
        RichText::new("Inspector Label")
            .font(egui::FontId::new(
                11.0,
                egui::FontFamily::Name("clay-editor-segoe-semibold".into()),
            ))
            .color(tokens::TEXT_SECONDARY),
    );
    ui.label(
        RichText::new("Muted helper text")
            .size(11.0)
            .color(tokens::TEXT_MUTED),
    );
}

fn draw_input_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Input");
    ui.add_space(8.0);
    let _ = with_preview_input_chrome(ui, |ui| {
        ui.add_sized(
            [220.0, ui.spacing().interact_size.y],
            egui::TextEdit::singleline(&mut story.input_value)
                .horizontal_align(egui::Align::Min)
                .vertical_align(egui::Align::Center)
                .margin(egui::Margin::symmetric(8, 2)),
        )
    });
}

fn draw_field_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Field");
    ui.add_space(8.0);
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.label(
            RichText::new("Material")
                .font(egui::FontId::new(
                    11.0,
                    egui::FontFamily::Name("clay-editor-segoe-semibold".into()),
                ))
                .color(tokens::TEXT_SECONDARY),
        );
        let _ = with_preview_input_chrome(ui, |ui| {
            ui.add_sized(
                [220.0, ui.spacing().interact_size.y],
                egui::TextEdit::singleline(&mut story.field_value)
                    .horizontal_align(egui::Align::Min)
                    .vertical_align(egui::Align::Center)
                    .margin(egui::Margin::symmetric(8, 2)),
            )
        });
        ui.label(
            RichText::new("Assigned material slot")
                .size(11.0)
                .color(tokens::TEXT_MUTED),
        );
    });
}

fn draw_button_preview(ui: &mut Ui) {
    draw_preview_title(ui, "Button");
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let _ = button_variant(ui, "Primary", ButtonVariant::Primary);
        let _ = button_variant(ui, "Secondary", ButtonVariant::Secondary);
        let _ = button_variant(ui, "Ghost", ButtonVariant::Ghost);
        ui.scope(|ui| {
            ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
            ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
            ui.visuals_mut().widgets.hovered.bg_fill = Color32::TRANSPARENT;
            ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::NONE;
            ui.visuals_mut().widgets.active.bg_fill = Color32::TRANSPARENT;
            ui.visuals_mut().widgets.active.bg_stroke = Stroke::NONE;
            let _ = ui.link("Link").on_hover_cursor(CursorIcon::PointingHand);
        });
    });
}

fn draw_button_group_preview(ui: &mut Ui) {
    draw_preview_title(ui, "Button Group");
    ui.add_space(8.0);
    ui.push_id(Id::new("component_preview_button_group"), |ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.horizontal(|ui| {
            for (_, label) in BUTTON_GROUP_OPTIONS {
                let _ = button_variant(ui, label, ButtonVariant::Secondary);
            }
        });
    });
}

fn draw_checkbox_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Checkbox");
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let _ = styled_checkbox(ui, &mut story.checkbox_value);
        let label = ui
            .scope(|ui| {
                ui.style_mut().interaction.selectable_labels = false;
                ui.add(
                    egui::Label::new(RichText::new("Enable shadows").color(tokens::TEXT_PRIMARY))
                        .selectable(false)
                        .sense(Sense::click()),
                )
            })
            .inner
            .on_hover_cursor(CursorIcon::PointingHand);
        if label.clicked() {
            story.checkbox_value = !story.checkbox_value;
        }
    });
}

fn draw_switch_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Switch");
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Use Gravity").color(tokens::TEXT_PRIMARY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let _ = draw_toggle_switch(ui, &mut story.switch_value);
        });
    });
}

fn draw_slider_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Slider");
    ui.add_space(8.0);
    story.slider_value = story.slider_value.clamp(0.0, 100.0);
    ui.horizontal(|ui| {
        let _ = with_preview_slider_chrome(ui, |ui| {
            ui.scope(|ui| {
                ui.spacing_mut().interact_size.y = 16.0;
                ui.add_sized(
                    [156.0, ui.spacing().interact_size.y],
                    egui::Slider::new(&mut story.slider_value, 0.0..=100.0).show_value(false),
                )
            })
            .inner
        });
        ui.add_space(6.0);
        let _ = draw_preview_number_input(
            ui,
            Id::new("component_preview_slider_value"),
            &mut story.slider_value,
            58.0,
            1,
        );
        ui.label(RichText::new("%").color(tokens::TEXT_SECONDARY));
    });
    story.slider_value = story.slider_value.clamp(0.0, 100.0);
}

fn draw_select_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Select");
    ui.add_space(8.0);
    story.select_index = story
        .select_index
        .filter(|index| *index < SELECT_OPTIONS.len());
    let popup_id = Id::new("component_preview_select_popup");
    let trigger = draw_select_trigger(
        ui,
        Id::new("component_preview_select_trigger"),
        popup_id,
        story
            .select_index
            .and_then(|index| SELECT_OPTIONS.get(index).copied()),
        172.0,
    );
    let mut next_selection = None;
    let _ = egui::Popup::menu(&trigger)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .gap(2.0)
        .show(|ui| {
            ui.set_min_width(172.0);
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.add_space(2.0);
            for (index, value) in SELECT_OPTIONS.iter().enumerate() {
                let option_response =
                    draw_select_option_row(ui, *value, story.select_index == Some(index));
                if option_response.clicked() {
                    next_selection = Some(index);
                    ui.close();
                }
            }
            ui.add_space(2.0);
        });
    if let Some(next_index) = next_selection {
        story.select_index = Some(next_index);
    }
}

fn draw_select_trigger(
    ui: &mut Ui,
    id: Id,
    popup_id: Id,
    selected_text: Option<&str>,
    width: f32,
) -> egui::Response {
    ui.push_id(id, |ui| {
        let desired_size = egui::vec2(width, ui.spacing().interact_size.y);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        let focused = response.has_focus() || egui::Popup::is_id_open(ui.ctx(), popup_id);
        let fill = if focused {
            tokens::INPUT_FOCUS_BACKGROUND
        } else if response.hovered() {
            tokens::INPUT_HOVER_BACKGROUND
        } else {
            tokens::INPUT_BACKGROUND
        };
        let stroke = if focused {
            tokens::input_focus_stroke()
        } else if response.hovered() {
            Stroke::NONE
        } else {
            Stroke::NONE
        };
        ui.painter().rect(
            rect,
            CornerRadius::same(5),
            fill,
            stroke,
            StrokeKind::Outside,
        );
        ui.painter().text(
            egui::pos2(rect.left() + 8.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            selected_text.unwrap_or("Select an option."),
            egui::FontId::new(11.0, egui::FontFamily::Proportional),
            if selected_text.is_some() {
                tokens::TEXT_PRIMARY
            } else {
                tokens::TEXT_MUTED
            },
        );
        if let Some(image) = icons::image(ui.ctx(), "chevron-down", 12.0) {
            let icon_size = 12.0;
            let icon_rect = egui::Rect::from_center_size(
                egui::pos2(rect.right() - 10.0, rect.center().y),
                egui::vec2(icon_size, icon_size),
            );
            let _ = ui.put(icon_rect, image.tint(tokens::TEXT_SECONDARY));
        }
        response.on_hover_cursor(CursorIcon::PointingHand)
    })
    .inner
}

fn draw_tabs_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Tabs");
    ui.add_space(8.0);
    draw_tabs_control(
        ui,
        Id::new("component_preview_tabs"),
        &mut story.tabs_index,
        TAB_OPTIONS,
    );
    ui.add_space(6.0);
    egui::Frame::new()
        .fill(tokens::MUTED_SURFACE)
        .stroke(Stroke::new(1.0, tokens::CARD_BORDER))
        .corner_radius(CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(8, 8))
        .show(ui, |ui| {
            let text = match story.tabs_index {
                0 => "Design tab content",
                1 => "Code tab content",
                _ => "History tab content",
            };
            ui.label(RichText::new(text).color(tokens::TEXT_SECONDARY));
        });
}

fn draw_separator_preview(ui: &mut Ui) {
    draw_preview_title(ui, "Separator");
    ui.add_space(8.0);
    ui.label(RichText::new("Before").color(tokens::TEXT_SECONDARY));
    ui.separator();
    ui.label(RichText::new("After").color(tokens::TEXT_SECONDARY));
}

fn draw_card_preview(ui: &mut Ui) {
    draw_preview_title(ui, "Card");
    ui.add_space(8.0);
    egui::Frame::new()
        .fill(tokens::MUTED_SURFACE)
        .stroke(Stroke::new(1.0, tokens::CARD_BORDER))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(10, 10))
        .show(ui, |ui| {
            ui.set_min_height(80.0);
            ui.label(
                RichText::new("Preview Card")
                    .font(egui::FontId::new(
                        12.0,
                        egui::FontFamily::Name("clay-editor-segoe-semibold".into()),
                    ))
                    .color(tokens::TEXT_PRIMARY),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new("Compact content for inspector workflows")
                    .color(tokens::TEXT_SECONDARY),
            );
            ui.add_space((ui.available_height() - ui.spacing().interact_size.y).max(0.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let _ = button_variant(ui, "Save", ButtonVariant::Primary);
                ui.add_space(6.0);
                let _ = button_variant(ui, "Cancel", ButtonVariant::Secondary);
            });
        });
}

fn draw_progress_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Progress");
    ui.add_space(8.0);
    draw_preview_subtitle(ui, "Import Progress");
    story.progress_value = story.progress_value.clamp(0.0, 100.0);
    ui.horizontal(|ui| {
        let _ = with_preview_slider_chrome(ui, |ui| {
            ui.scope(|ui| {
                ui.spacing_mut().interact_size.y = 16.0;
                ui.add_sized(
                    [156.0, ui.spacing().interact_size.y],
                    egui::Slider::new(&mut story.progress_value, 0.0..=100.0).show_value(false),
                )
            })
            .inner
        });
        ui.add_space(6.0);
        let _ = draw_preview_number_input(
            ui,
            Id::new("component_preview_progress_value"),
            &mut story.progress_value,
            58.0,
            1,
        );
    });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_sized(
            [188.0, 16.0],
            egui::ProgressBar::new((story.progress_value / 100.0).clamp(0.0, 1.0))
                .text("")
                .fill(tokens::blue(500)),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!("{:.0}%", story.progress_value))
                .size(11.0)
                .color(tokens::TEXT_SECONDARY),
        );
    });
}

fn draw_scroll_area_preview(ui: &mut Ui) {
    draw_preview_title(ui, "Scroll Area");
    ui.add_space(8.0);
    draw_preview_mock_frame(ui, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("component_preview_scroll_area")
            .max_height(130.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for index in 0..14 {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Layer {:02}", index + 1))
                                .color(Color32::from_gray(205)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new("Visible")
                                    .size(11.0)
                                    .color(Color32::from_gray(135)),
                            );
                        });
                    });
                    if index + 1 < 14 {
                        ui.separator();
                    }
                }
            });
    });
}

fn draw_resizable_preview(ui: &mut Ui) {
    draw_preview_title(ui, "Resizable");
    ui.add_space(8.0);
    egui::Resize::default()
        .id_salt("component_preview_resizable")
        .default_size(egui::vec2(228.0, 108.0))
        .min_size(egui::vec2(168.0, 78.0))
        .max_size(egui::vec2(320.0, 180.0))
        .show(ui, |ui| {
            draw_preview_mock_frame(ui, |ui| {
                draw_preview_subtitle(ui, "Inspector Pane");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(RichText::new("W").color(Color32::from_gray(168)));
                    ui.label(RichText::new("84").color(Color32::from_gray(214)));
                    ui.add_space(10.0);
                    ui.label(RichText::new("H").color(Color32::from_gray(168)));
                    ui.label(RichText::new("84").color(Color32::from_gray(214)));
                });
                draw_preview_muted(ui, "Drag bottom-right handle to resize");
            });
        });
}

fn draw_tooltip_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Tooltip");
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let _ = styled_checkbox(ui, &mut story.tooltip_top_center);
        let label = ui
            .add(
                egui::Label::new(
                    RichText::new("Show above and centered").color(tokens::TEXT_PRIMARY),
                )
                .sense(Sense::click()),
            )
            .on_hover_cursor(CursorIcon::PointingHand);
        if label.clicked() {
            story.tooltip_top_center = !story.tooltip_top_center;
        }
    });
    ui.add_space(6.0);
    let options = TooltipPreviewOptions {
        delay_ms: None,
        top_center: story.tooltip_top_center,
    };
    with_tooltip_delay(ui, options, |ui| {
        let response = ui
            .add_sized(
                [220.0, ui.spacing().interact_size.y],
                egui::Button::new("Hover for tooltip"),
            )
            .on_hover_cursor(CursorIcon::PointingHand);
        if options.top_center {
            let mut tooltip = egui::Tooltip::for_enabled(&response).gap(6.0);
            tooltip.popup = tooltip
                .popup
                .align(egui::RectAlign::TOP)
                .align_alternatives(&[egui::RectAlign::TOP]);
            let _ = tooltip.show(|ui| {
                ui.label(RichText::new("Snap to pixel grid").color(tokens::TEXT_PRIMARY));
            });
        } else {
            response.on_hover_text("Snap to pixel grid");
        }
    });
    ui.add_space(8.0);
    draw_preview_subtitle(ui, "Always-visible mock");
    draw_preview_mock_frame(ui, |ui| {
        ui.label(RichText::new("Snap to pixel grid").color(tokens::TEXT_PRIMARY));
    });
}

fn draw_collapsible_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Collapsible");
    ui.add_space(8.0);
    let collapsing = egui::CollapsingHeader::new("Transform")
        .id_salt("component_preview_collapsible")
        .open(Some(story.collapsible_open))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Position").color(Color32::from_gray(188)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("X 74  Y 74").color(Color32::from_gray(214)));
                });
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Rotation").color(Color32::from_gray(188)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("0°").color(Color32::from_gray(214)));
                });
            });
        });
    if collapsing.header_response.clicked() {
        story.collapsible_open = !story.collapsible_open;
    }
}

fn draw_context_menu_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Context Menu");
    ui.add_space(8.0);
    let target = ui.add_sized(
        [220.0, ui.spacing().interact_size.y],
        egui::Button::new("Right-click target row"),
    );
    egui::Popup::context_menu(&target).show(|ui| {
        if ui
            .button("Rename")
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            ui.close();
        }
        if ui
            .button("Duplicate")
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            ui.close();
        }
        if ui
            .checkbox(&mut story.context_menu_toggle, "Visible")
            .clicked()
        {
            ui.close();
        }
        if ui
            .button("Delete")
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            ui.close();
        }
    });
    ui.add_space(8.0);
    draw_preview_subtitle(ui, "Open-state mock");
    draw_preview_mock_frame(ui, |ui| {
        draw_menu_row(ui, "Rename", false, false);
        draw_menu_row(ui, "Duplicate", false, false);
        draw_menu_row(ui, "Visible", true, story.context_menu_toggle);
        draw_menu_row(ui, "Delete", false, false);
    });
}

fn draw_dropdown_menu_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Dropdown Menu");
    ui.add_space(8.0);
    ui.menu_button("Actions", |ui| {
        for (index, label) in DROPDOWN_OPTIONS.iter().enumerate() {
            if ui
                .button(*label)
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                story.dropdown_index = index;
                ui.close();
            }
        }
    });
    draw_preview_muted(
        ui,
        format!(
            "Selected: {}",
            DROPDOWN_OPTIONS[story.dropdown_index.min(DROPDOWN_OPTIONS.len() - 1)]
        ),
    );
    ui.add_space(8.0);
    draw_preview_subtitle(ui, "Open-state mock");
    draw_preview_mock_frame(ui, |ui| {
        for (index, label) in DROPDOWN_OPTIONS.iter().enumerate() {
            draw_menu_row(ui, label, false, false);
            if index + 1 < DROPDOWN_OPTIONS.len() {
                ui.separator();
            }
        }
    });
}

fn draw_combobox_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Combobox");
    ui.add_space(8.0);
    let _ = with_input_chrome(ui, |ui| {
        ui.add_sized(
            [220.0, ui.spacing().interact_size.y],
            egui::TextEdit::singleline(&mut story.combobox_query).hint_text("Filter assets"),
        )
    });
    ui.add_space(6.0);
    draw_preview_mock_frame(ui, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("component_preview_combobox")
            .max_height(104.0)
            .show(ui, |ui| {
                let query = story.combobox_query.to_ascii_lowercase();
                let mut shown = 0usize;
                for (index, label) in COMBOBOX_OPTIONS.iter().enumerate() {
                    if !query.is_empty() && !label.to_ascii_lowercase().contains(query.as_str()) {
                        continue;
                    }
                    shown += 1;
                    let selected = story.combobox_index == index;
                    if ui.selectable_label(selected, *label).clicked() {
                        story.combobox_index = index;
                    }
                }
                if shown == 0 {
                    draw_preview_muted(ui, "No matches");
                }
            });
    });
    draw_preview_muted(
        ui,
        format!(
            "Selected: {}",
            COMBOBOX_OPTIONS[story.combobox_index.min(COMBOBOX_OPTIONS.len() - 1)]
        ),
    );
}

fn draw_command_preview(ui: &mut Ui, story: &mut ComponentStoryState) {
    draw_preview_title(ui, "Command");
    ui.add_space(8.0);
    let _ = with_input_chrome(ui, |ui| {
        ui.add_sized(
            [220.0, ui.spacing().interact_size.y],
            egui::TextEdit::singleline(&mut story.command_query).hint_text("Type a command"),
        )
    });
    ui.add_space(6.0);
    draw_preview_mock_frame(ui, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("component_preview_command")
            .max_height(132.0)
            .show(ui, |ui| {
                let query = story.command_query.to_ascii_lowercase();
                let mut shown = 0usize;
                for (group, label) in COMMAND_OPTIONS {
                    let searchable = format!("{group} {label}").to_ascii_lowercase();
                    if !query.is_empty() && !searchable.contains(query.as_str()) {
                        continue;
                    }
                    shown += 1;
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(group).size(10.0).color(tokens::TEXT_MUTED));
                        ui.add_space(8.0);
                        ui.label(RichText::new(label).color(tokens::TEXT_PRIMARY));
                    });
                }
                if shown == 0 {
                    draw_preview_muted(ui, "No commands");
                }
            });
    });
}

fn draw_dialog_preview(ui: &mut Ui, story: &mut ComponentStoryState, alert_variant: bool) {
    let title = if alert_variant {
        "Alert Dialog"
    } else {
        "Dialog"
    };
    draw_preview_title(ui, title);
    ui.add_space(8.0);
    let open = if alert_variant {
        &mut story.alert_dialog_open
    } else {
        &mut story.dialog_open
    };
    let open_label = if alert_variant {
        "Open Alert"
    } else {
        "Open Dialog"
    };
    if ui
        .button(open_label)
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
    {
        *open = true;
    }
    if *open {
        egui::Window::new(title)
            .id(egui::Id::new("component_preview_dialog_window").with(title))
            .collapsible(false)
            .resizable(false)
            .open(open)
            .default_width(300.0)
            .show(ui.ctx(), |ui| {
                ui.label(
                    RichText::new("This uses native egui window behavior.")
                        .color(tokens::TEXT_PRIMARY),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui
                        .button("Cancel")
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        ui.close();
                    }
                    if alert_variant {
                        let _ = button_variant(ui, "Delete", ButtonVariant::Primary);
                    } else {
                        let _ = button_variant(ui, "Confirm", ButtonVariant::Primary);
                    }
                });
            });
    }
    ui.add_space(8.0);
    draw_preview_subtitle(ui, "Open-state mock");
    draw_preview_mock_frame(ui, |ui| {
        ui.label(
            RichText::new(if alert_variant {
                "Delete selected entity?"
            } else {
                "Create a new component?"
            })
            .font(egui::FontId::new(
                11.0,
                egui::FontFamily::Name("clay-editor-segoe-semibold".into()),
            ))
            .color(if alert_variant {
                Color32::from_rgb(238, 136, 136)
            } else {
                tokens::TEXT_PRIMARY
            }),
        );
        ui.add_space(6.0);
        draw_preview_muted(
            ui,
            if alert_variant {
                "This action cannot be undone."
            } else {
                "Adds the selected component to the active object."
            },
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let _ = button_variant(ui, "Cancel", ButtonVariant::Secondary);
            if alert_variant {
                let _ = button_variant(ui, "Delete", ButtonVariant::Primary);
            } else {
                let _ = button_variant(ui, "Create", ButtonVariant::Primary);
            }
        });
    });
}

fn draw_select_option_row(ui: &mut Ui, label: &str, selected: bool) -> egui::Response {
    let desired_size = egui::vec2(ui.available_width().max(96.0), ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let fill = if selected {
        tokens::neutral(700)
    } else if response.hovered() {
        tokens::neutral(800)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(4),
        fill,
        Stroke::NONE,
        StrokeKind::Outside,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::new(11.0, egui::FontFamily::Proportional),
        if selected {
            tokens::TEXT_PRIMARY
        } else {
            tokens::TEXT_SECONDARY
        },
    );
    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn draw_tabs_control(ui: &mut Ui, id: Id, current: &mut usize, options: &[(usize, &str)]) {
    if options.is_empty() {
        return;
    }
    ui.push_id(id, |ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.horizontal(|ui| {
            for (candidate, label) in options.iter().copied() {
                let selected = *current == candidate;
                let response = ui
                    .add(
                        egui::Button::new(RichText::new(label).color(if selected {
                            tokens::TEXT_PRIMARY
                        } else {
                            tokens::TEXT_SECONDARY
                        }))
                        .fill(if selected {
                            tokens::neutral(800)
                        } else {
                            Color32::TRANSPARENT
                        })
                        .stroke(if selected {
                            Stroke::new(1.0, tokens::neutral(500))
                        } else {
                            Stroke::new(1.0, tokens::neutral(700))
                        })
                        .corner_radius(CornerRadius::ZERO)
                        .min_size(egui::vec2(0.0, ui.spacing().interact_size.y)),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand);
                if response.clicked() && !selected {
                    *current = candidate;
                }
            }
        });
    });
}

fn with_preview_input_chrome<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        let style = ui.style_mut();
        style.visuals.text_edit_bg_color = Some(tokens::INPUT_BACKGROUND);
        style.visuals.code_bg_color = tokens::INPUT_BACKGROUND;
        style.visuals.selection.stroke = Stroke::NONE;
        let visuals = &mut style.visuals.widgets;
        visuals.inactive.bg_fill = tokens::INPUT_BACKGROUND;
        visuals.inactive.weak_bg_fill = tokens::INPUT_BACKGROUND;
        visuals.hovered.bg_fill = tokens::INPUT_HOVER_BACKGROUND;
        visuals.hovered.weak_bg_fill = tokens::INPUT_HOVER_BACKGROUND;
        visuals.active.bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.active.weak_bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.open.bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.open.weak_bg_fill = tokens::INPUT_FOCUS_BACKGROUND;
        visuals.inactive.bg_stroke = Stroke::NONE;
        visuals.hovered.bg_stroke = Stroke::NONE;
        visuals.active.bg_stroke = tokens::input_focus_stroke();
        visuals.open.bg_stroke = tokens::input_focus_stroke();
        add(ui)
    })
    .inner
}

fn with_preview_slider_chrome<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        let style = ui.style_mut();
        style.spacing.slider_rail_height = style.spacing.slider_rail_height.max(4.0);
        style.visuals.selection.bg_fill = tokens::neutral(500);
        style.visuals.selection.stroke = Stroke::new(1.0, tokens::neutral(300));
        let visuals = &mut style.visuals.widgets;
        visuals.inactive.bg_fill = tokens::neutral(600);
        visuals.inactive.weak_bg_fill = tokens::neutral(600);
        visuals.hovered.bg_fill = tokens::neutral(500);
        visuals.hovered.weak_bg_fill = tokens::neutral(500);
        visuals.active.bg_fill = tokens::neutral(500);
        visuals.active.weak_bg_fill = tokens::neutral(500);
        visuals.open.bg_fill = tokens::neutral(500);
        visuals.open.weak_bg_fill = tokens::neutral(500);
        visuals.inactive.bg_stroke = Stroke::new(1.0, tokens::neutral(500));
        visuals.hovered.bg_stroke = Stroke::new(1.0, tokens::neutral(400));
        visuals.active.bg_stroke = Stroke::new(1.0, tokens::neutral(300));
        visuals.open.bg_stroke = Stroke::new(1.0, tokens::neutral(300));
        visuals.inactive.fg_stroke = Stroke::new(1.0, tokens::neutral(500));
        visuals.hovered.fg_stroke = Stroke::new(1.0, tokens::neutral(400));
        visuals.active.fg_stroke = Stroke::new(1.0, tokens::neutral(300));
        visuals.open.fg_stroke = Stroke::new(1.0, tokens::neutral(300));
        add(ui)
    })
    .inner
}

fn draw_preview_number_input(
    ui: &mut Ui,
    id: Id,
    value: &mut f32,
    width: f32,
    decimals: usize,
) -> egui::Response {
    with_preview_input_chrome(ui, |ui| {
        ui.push_id(id, |ui| {
            ui.add_sized(
                [width, ui.spacing().interact_size.y],
                egui::DragValue::new(value)
                    .range(0.0..=100.0)
                    .speed(0.2)
                    .fixed_decimals(decimals),
            )
            .on_hover_cursor(CursorIcon::ResizeHorizontal)
        })
        .inner
    })
}

fn with_tooltip_delay<R>(
    ui: &mut Ui,
    options: TooltipPreviewOptions,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.scope(|ui| {
        ui.style_mut().interaction.tooltip_delay = options.resolved_delay_ms() as f32 / 1000.0;
        ui.style_mut().interaction.show_tooltips_only_when_still = false;
        add(ui)
    })
    .inner
}

fn draw_menu_row(ui: &mut Ui, label: &str, show_check: bool, checked: bool) {
    ui.horizontal(|ui| {
        if show_check {
            let icon = if checked { "check" } else { "dot" };
            if let Some(image) = icons::image(ui.ctx(), icon, 11.0) {
                ui.add(image.tint(tokens::TEXT_SECONDARY));
            }
        }
        ui.label(RichText::new(label).color(tokens::TEXT_PRIMARY));
    });
}

fn draw_toggle_switch(ui: &mut Ui, value: &mut bool) -> egui::Response {
    let desired_size = egui::vec2(34.0, 18.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    let t = ui.ctx().animate_bool(response.id, *value);
    let fill = tokens::neutral(600).lerp_to_gamma(tokens::green(600), t);
    let stroke = Stroke::new(1.0, tokens::neutral(500));
    ui.painter().rect(
        rect,
        CornerRadius::same(9),
        fill,
        stroke,
        StrokeKind::Outside,
    );
    let knob_radius = 6.0;
    let knob_x = egui::lerp((rect.left() + 9.0)..=(rect.right() - 9.0), t);
    let knob_color = tokens::neutral(200).lerp_to_gamma(Color32::WHITE, t);
    ui.painter()
        .circle_filled(egui::pos2(knob_x, rect.center().y), knob_radius, knob_color);
    response
}

#[derive(Debug, Clone, Copy)]
enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
}

fn button_variant(ui: &mut Ui, label: &str, variant: ButtonVariant) -> egui::Response {
    ui.scope(|ui| {
        let visuals = &mut ui.style_mut().visuals.widgets;
        match variant {
            ButtonVariant::Primary => {
                visuals.inactive.bg_fill = tokens::blue(500);
                visuals.inactive.weak_bg_fill = tokens::blue(500);
                visuals.hovered.bg_fill = tokens::blue(400);
                visuals.hovered.weak_bg_fill = tokens::blue(400);
                visuals.active.bg_fill = tokens::blue(600);
                visuals.active.weak_bg_fill = tokens::blue(600);
                visuals.inactive.bg_stroke = Stroke::NONE;
                visuals.hovered.bg_stroke = Stroke::NONE;
                visuals.active.bg_stroke = Stroke::NONE;
            }
            ButtonVariant::Secondary => {
                visuals.inactive.bg_fill = tokens::neutral(800);
                visuals.inactive.weak_bg_fill = tokens::neutral(800);
                visuals.hovered.bg_fill = tokens::neutral(700);
                visuals.hovered.weak_bg_fill = tokens::neutral(700);
                visuals.active.bg_fill = tokens::neutral(700);
                visuals.active.weak_bg_fill = tokens::neutral(700);
                visuals.inactive.bg_stroke = Stroke::new(1.0, tokens::neutral(700));
                visuals.hovered.bg_stroke = Stroke::new(1.0, tokens::neutral(600));
                visuals.active.bg_stroke = Stroke::new(1.0, tokens::neutral(600));
            }
            ButtonVariant::Ghost => {
                visuals.inactive.bg_fill = Color32::TRANSPARENT;
                visuals.inactive.weak_bg_fill = Color32::TRANSPARENT;
                visuals.hovered.bg_fill = tokens::neutral(700);
                visuals.hovered.weak_bg_fill = tokens::neutral(700);
                visuals.active.bg_fill = tokens::neutral(600);
                visuals.active.weak_bg_fill = tokens::neutral(600);
                visuals.inactive.bg_stroke = Stroke::NONE;
                visuals.hovered.bg_stroke = Stroke::NONE;
                visuals.active.bg_stroke = Stroke::NONE;
            }
        }
        ui.button(label).on_hover_cursor(CursorIcon::PointingHand)
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::{
        component_group, parse_component_kind, supported_component_definitions, ComponentGroup,
        ComponentKind, ComponentStoryState, TooltipPreviewOptions, DEFAULT_TOOLTIP_DELAY_MS,
    };

    #[test]
    fn next_core_components_are_implemented() {
        let ids = supported_component_definitions()
            .map(|definition| definition.id)
            .collect::<Vec<_>>();
        for expected in [
            "scroll-area",
            "resizable",
            "tooltip",
            "progress",
            "collapsible",
            "context-menu",
            "dropdown-menu",
            "combobox",
            "command",
            "dialog",
            "alert-dialog",
        ] {
            assert!(
                ids.contains(&expected),
                "missing implemented component: {expected}"
            );
        }
    }

    #[test]
    fn next_core_components_have_expected_grouping() {
        for kind in [
            ComponentKind::ScrollArea,
            ComponentKind::Resizable,
            ComponentKind::Tooltip,
            ComponentKind::Progress,
        ] {
            assert_eq!(component_group(kind), ComponentGroup::PrimaryPrimitive);
        }
        for kind in [
            ComponentKind::Collapsible,
            ComponentKind::ContextMenu,
            ComponentKind::DropdownMenu,
            ComponentKind::Combobox,
            ComponentKind::Command,
            ComponentKind::Dialog,
            ComponentKind::AlertDialog,
        ] {
            assert_eq!(component_group(kind), ComponentGroup::Composed);
        }
    }

    #[test]
    fn parser_accepts_new_component_ids_and_aliases() {
        assert_eq!(
            parse_component_kind("scroll-area"),
            Some(ComponentKind::ScrollArea)
        );
        assert_eq!(
            parse_component_kind("scroll_area"),
            Some(ComponentKind::ScrollArea)
        );
        assert_eq!(
            parse_component_kind("contextmenu"),
            Some(ComponentKind::ContextMenu)
        );
        assert_eq!(
            parse_component_kind("dropdown_menu"),
            Some(ComponentKind::DropdownMenu)
        );
        assert_eq!(
            parse_component_kind("alertdialog"),
            Some(ComponentKind::AlertDialog)
        );
    }

    #[test]
    fn story_defaults_to_select_placeholder_state() {
        let story = ComponentStoryState::default();
        assert_eq!(story.select_index, None);
    }

    #[test]
    fn tooltip_preview_uses_expected_default_delay() {
        let options = TooltipPreviewOptions {
            delay_ms: None,
            top_center: true,
        };
        assert_eq!(options.resolved_delay_ms(), DEFAULT_TOOLTIP_DELAY_MS);
        let custom = TooltipPreviewOptions {
            delay_ms: Some(100),
            top_center: true,
        };
        assert_eq!(custom.resolved_delay_ms(), 100);
    }
}
