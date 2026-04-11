use super::*;
use crate::components::{
    AudioPlayback, AudioPlaybackState, Button, ButtonGroup, ButtonVariant, Card, Checkbox,
    CollabCursor, Collapsible, Color, Combobox, Command, CommandItem, ComponentUi, ComponentUiExt,
    ContextMenu, ControlSize, DialogueHeader, DialogueModal, DragBoard, DragBoardItem,
    DragBoardRegion, DropdownMenu, DropdownMenuAction, DropdownMenuEntry, EmojiSelector, Field,
    FileTree, FileTreeNode, Hierarchy as HierarchyWidget, HierarchyNode as HierarchyWidgetNode,
    Icon, IconToolbar, IconToolbarItem, Image, ImageTile, Kbd, Label, LabelTone, LabelWeight,
    MenuBar, MenuBarItem, NumberInput, OpenWith, Pagination, Popover, Progress, Radio, RadioGroup,
    RadioOption, Select, Sidebar, Skeleton, Slider, Spinner, Switch, TabOption, TextInput,
    ToastIntent, ToastPlacement, Toolbar, Tooltip, Twemoji,
};
use crate::layout::{
    column as layout_column, row as layout_row, Align as FlowAlign, Justify as FlowJustify,
};
use crate::primitives::{surface_frame, SurfaceFrame};
use crate::theme::ColorRole;
use crate::ui::tokens;
use egui::{Align, Align2, Color32, Id, Key, Layout, Order, Stroke, Vec2};
use std::collections::{BTreeMap, BTreeSet};

const SHOWCASE_IMAGE_BYTES: &[u8] = include_bytes!("../../assets/images/showcase-image.png");

pub fn render_tree(ui: &mut egui::Ui, tree: &ContractTree) -> Vec<ContractEvent> {
    let mut ui = ui.components();
    render_component_tree(&mut ui, tree)
}

pub fn render_component_tree(ui: &mut ComponentUi<'_>, tree: &ContractTree) -> Vec<ContractEvent> {
    let mut renderer = FrameRenderer { events: Vec::new() };
    renderer.render_node(ui, &tree.root);
    renderer.events
}

struct FrameRenderer {
    events: Vec<ContractEvent>,
}

#[derive(Debug)]
struct MenuActionBinding<'a> {
    item_id: &'a str,
    label: &'a str,
    action_id: Option<&'a ActionId>,
}

#[derive(Debug)]
struct HierarchyBinding<'a> {
    item_id: &'a str,
    label: &'a str,
    action_id: Option<&'a ActionId>,
    open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ToastLifecycleState {
    shown_at: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ToastViewportState {
    items: BTreeMap<String, ToastLifecycleState>,
    suppressed: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy)]
struct ToastPalette {
    fill: Color32,
    stroke: Stroke,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ContainerLayoutPlan {
    direction: ContractDirection,
    justify: ContractJustify,
    align: ContractAlign,
    gap: f32,
}

impl FrameRenderer {
    fn render_node(&mut self, ui: &mut ComponentUi<'_>, node: &ContractNode) {
        let common = node.common();
        if !common.visible {
            return;
        }

        // Common-field execution truth for Phase 9:
        // - visible, enabled, and the supported subset of layout are executed here
        // - class, class_list, slot_classes, and common.actions remain metadata-only
        ignore_metadata_only_common_fields(common);

        if common.enabled {
            with_layout_scope(ui, common.layout.as_ref(), |ui| {
                self.render_node_inner(ui, node);
            });
        } else {
            ui.ui_mut().add_enabled_ui(false, |ui| {
                let mut ui = ui.components();
                with_layout_scope(&mut ui, common.layout.as_ref(), |ui| {
                    self.render_node_inner(ui, node);
                });
            });
        }
    }

    fn render_node_inner(&mut self, ui: &mut ComponentUi<'_>, node: &ContractNode) {
        match node {
            ContractNode::Row(props) => self.render_row(ui, props),
            ContractNode::Column(props) => self.render_column(ui, props),
            ContractNode::Inset(props) => self.render_inset(ui, props),
            ContractNode::SizedBox(props) => self.render_sized_box(ui, props),
            ContractNode::Spacer(props) => self.render_spacer(ui, props),
            ContractNode::Card(props) => self.render_card(ui, props),
            ContractNode::Sidebar(props) => self.render_sidebar(ui, props),
            ContractNode::Toolbar(props) => self.render_toolbar(ui, props),
            ContractNode::MenuBar(props) => self.render_menu_bar(ui, props),
            ContractNode::Tabs(props) => self.render_tabs(ui, props),
            ContractNode::Label(props) => self.render_label(ui, props),
            ContractNode::Button(props) => self.render_button(ui, props),
            ContractNode::ButtonGroup(props) => self.render_button_group(ui, props),
            ContractNode::Input(props) => self.render_input(ui, props),
            ContractNode::NumberInput(props) => self.render_number_input(ui, props),
            ContractNode::Checkbox(props) => self.render_checkbox(ui, props),
            ContractNode::Switch(props) => self.render_switch(ui, props),
            ContractNode::Select(props) => self.render_select(ui, props),
            ContractNode::Field(props) => self.render_field(ui, props),
            ContractNode::Separator(_) => {
                let _ = ui.separator();
            }
            ContractNode::Collapsible(props) => self.render_collapsible(ui, props),
            ContractNode::DialogueModal(props) => self.render_dialogue_modal(ui, props),
            ContractNode::Hierarchy(props) => self.render_hierarchy(ui, props),
            ContractNode::Spinner(props) => {
                let _ = ui.spinner(Spinner::new().size(props.size));
            }
            ContractNode::Progress(props) => {
                let _ = ui.progress(
                    props.value,
                    Progress::new().width(props.width).height(props.height),
                );
            }
            ContractNode::ToastViewport(props) => self.render_toast_viewport(ui, props),
            ContractNode::Color(props) => self.render_color(ui, props),
            ContractNode::Icon(props) => self.render_icon(ui, props),
            ContractNode::Image(props) => self.render_image(ui, props),
            ContractNode::Twemoji(props) => self.render_twemoji(ui, props),
            ContractNode::Kbd(props) => self.render_kbd(ui, props),
            ContractNode::Skeleton(props) => self.render_skeleton(ui, props),
            ContractNode::Slider(props) => self.render_slider(ui, props),
            ContractNode::Radio(props) => self.render_radio(ui, props),
            ContractNode::RadioGroup(props) => self.render_radio_group(ui, props),
            ContractNode::Combobox(props) => self.render_combobox(ui, props),
            ContractNode::EmojiSelector(props) => self.render_emoji_selector(ui, props),
            ContractNode::Pagination(props) => self.render_pagination(ui, props),
            ContractNode::Tooltip(props) => self.render_tooltip(ui, props),
            ContractNode::Popover(props) => self.render_popover(ui, props),
            ContractNode::DropdownMenu(props) => self.render_dropdown_menu(ui, props),
            ContractNode::ContextMenu(props) => self.render_context_menu(ui, props),
            ContractNode::OpenWith(props) => self.render_open_with(ui, props),
            ContractNode::CollabCursor(props) => self.render_collab_cursor(ui, props),
            ContractNode::IconToolbar(props) => self.render_icon_toolbar(ui, props),
            ContractNode::FileTree(props) => self.render_file_tree(ui, props),
            ContractNode::DragBoard(props) => self.render_drag_board(ui, props),
            ContractNode::AudioPlayback(props) => self.render_audio_playback(ui, props),
            ContractNode::ImageTile(props) => self.render_image_tile(ui, props),
            ContractNode::Command(props) => self.render_command(ui, props),
        }
    }

    fn render_children(&mut self, ui: &mut egui::Ui, children: &[ContractNode]) {
        let mut ui = ui.components();
        for child in children {
            self.render_node(&mut ui, child);
        }
    }

    fn render_row(&mut self, ui: &mut ComponentUi<'_>, props: &ContractRow) {
        let _ = ui.ui_mut().scope(|ui| {
            render_container_children(
                ui,
                &props.common,
                ContractDirection::Row,
                props.gap,
                props.justify,
                props.align,
                |ui| self.render_children(ui, &props.children),
            );
        });
    }

    fn render_column(&mut self, ui: &mut ComponentUi<'_>, props: &ContractColumn) {
        let _ = ui.ui_mut().scope(|ui| {
            render_container_children(
                ui,
                &props.common,
                ContractDirection::Column,
                props.gap,
                props.justify,
                props.align,
                |ui| self.render_children(ui, &props.children),
            );
        });
    }

    fn render_inset(&mut self, ui: &mut ComponentUi<'_>, props: &ContractInset) {
        let _ = egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(
                props.padding_x as i8,
                props.padding_y as i8,
            ))
            .show(ui.ui_mut(), |ui| {
                render_container_children(
                    ui,
                    &props.common,
                    ContractDirection::Column,
                    0.0,
                    ContractJustify::Start,
                    ContractAlign::Start,
                    |ui| self.render_children(ui, &props.children),
                );
            });
    }

    fn render_sized_box(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSizedBox) {
        let _ = ui.ui_mut().scope(|ui| {
            if let Some(width) = props.width {
                ui.set_min_width(width);
                ui.set_max_width(width);
            }
            if let Some(height) = props.height {
                ui.set_min_height(height);
                ui.set_max_height(height);
            }
            self.render_children(ui, &props.children);
        });
    }

    fn render_spacer(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSpacer) {
        let ui = ui.ui_mut();
        let size = if props.flex {
            match ui.layout().main_dir() {
                egui::Direction::LeftToRight | egui::Direction::RightToLeft => {
                    egui::vec2(ui.available_width().max(0.0), props.height.unwrap_or(0.0))
                }
                egui::Direction::TopDown | egui::Direction::BottomUp => {
                    egui::vec2(props.width.unwrap_or(0.0), ui.available_height().max(0.0))
                }
            }
        } else {
            egui::vec2(props.width.unwrap_or(0.0), props.height.unwrap_or(0.0))
        };
        let _ = ui.allocate_exact_size(size, egui::Sense::hover());
    }

    fn render_card(&mut self, ui: &mut ComponentUi<'_>, props: &crate::contract::ContractCard) {
        let _ = ui.card(
            Card::new().padding(props.padding_x as i8, props.padding_y as i8),
            |ui| {
                render_container_children(
                    ui,
                    &props.common,
                    ContractDirection::Column,
                    0.0,
                    ContractJustify::Start,
                    ContractAlign::Start,
                    |ui| self.render_children(ui, &props.children),
                );
            },
        );
    }

    fn render_sidebar(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSidebar) {
        let mut open = props.open;
        let mut sidebar = Sidebar::new(make_id(&props.common.node_id, "sidebar"))
            .side(props.side)
            .width(props.width);
        if let Some(title) = props.title.as_deref() {
            sidebar = sidebar.title(title);
        }
        ui.sidebar(&mut open, sidebar, |ui, _close_requested| {
            self.render_children(ui, &props.children);
        });
        if props.open && !open {
            self.emit_basic(&props.common.node_id, EventKind::Closed, None);
        }
    }

    fn render_toolbar(&mut self, ui: &mut ComponentUi<'_>, props: &ContractToolbar) {
        let toolbar = Toolbar::new(make_id(&props.common.node_id, "toolbar"))
            .anchor(map_anchor(props.anchor))
            .offset(egui::vec2(props.offset_x, props.offset_y));
        let _ = ui.toolbar(toolbar, |ui| {
            self.render_children(ui, &props.children);
        });
    }

    fn render_menu_bar(&mut self, ui: &mut ComponentUi<'_>, props: &ContractMenuBar) {
        let mut action_bindings = Vec::new();
        let runtime_entries = props
            .menus
            .iter()
            .map(|menu| {
                let mut entries = Vec::new();
                append_runtime_menu_entries(&menu.entries, &mut entries, &mut action_bindings);
                entries
            })
            .collect::<Vec<_>>();

        let runtime_menus = props
            .menus
            .iter()
            .zip(runtime_entries.iter())
            .map(|(menu, entries)| {
                MenuBarItem::new(menu.label.as_str(), entries.as_slice()).width(menu.width)
            })
            .collect::<Vec<_>>();

        let (_, state) = ui.menu_bar(MenuBar::new(
            make_id(&props.common.node_id, "menu_bar"),
            runtime_menus.as_slice(),
        ));

        if let Some(action_index) = state.action {
            if let Some(binding) = action_bindings.get(action_index) {
                self.emit_item(
                    &props.common.node_id,
                    EventKind::CommandInvoked,
                    binding.action_id,
                    binding.item_id,
                    binding.label,
                );
            }
        }
    }

    fn render_tabs(&mut self, ui: &mut ComponentUi<'_>, props: &ContractTabs) {
        if props.items.is_empty() {
            return;
        }

        let mut current = props
            .selected_item_id
            .as_deref()
            .and_then(|item_id| props.items.iter().position(|item| item.item_id == item_id))
            .unwrap_or(0);
        let previous = current;
        let runtime_options = props
            .items
            .iter()
            .enumerate()
            .map(
                |(index, item)| match (item.icon.as_deref(), item.icon_only) {
                    (Some(icon), true) => TabOption::icon_only(index, item.label.as_str(), icon)
                        .tooltip(item.label.as_str()),
                    (Some(icon), false) => TabOption::with_icon(index, item.label.as_str(), icon),
                    (None, _) => TabOption::new(index, item.label.as_str()),
                },
            )
            .collect::<Vec<_>>();
        let id = make_id(&props.common.node_id, "tabs");

        match props.style {
            ContractTabsStyle::Underline => ui.tabs(id, &mut current, runtime_options.as_slice()),
            ContractTabsStyle::Segmented => {
                ui.segmented_tabs(id, &mut current, runtime_options.as_slice())
            }
            ContractTabsStyle::BlenderTopbar => {
                ui.blender_topbar_tabs(id, &mut current, runtime_options.as_slice())
            }
            ContractTabsStyle::Stacked => {
                ui.stacked_tabs(id, &mut current, runtime_options.as_slice())
            }
            ContractTabsStyle::Rail => ui.rail_tabs(id, &mut current, runtime_options.as_slice()),
        }

        if current != previous {
            if let Some(item) = props.items.get(current) {
                self.emit_item(
                    &props.common.node_id,
                    EventKind::Selected,
                    item.action_id.as_ref(),
                    item.item_id.as_str(),
                    item.label.as_str(),
                );
            }
        }
    }

    fn render_label(&mut self, ui: &mut ComponentUi<'_>, props: &crate::contract::ContractLabel) {
        let mut label = Label::new(props.text.as_str());
        if let Some(tone) = props.tone {
            label = label.tone(tone);
        }
        if let Some(weight) = props.weight {
            label = label.weight(weight);
        }
        if let Some(size) = props.size {
            label = label.size(size);
        }
        if props.truncate {
            label = label.truncate();
        }
        let _ = ui.label(label);
    }

    fn render_button(&mut self, ui: &mut ComponentUi<'_>, props: &ContractButton) {
        let mut button = if props.icon_only {
            if let Some(icon) = props.leading_icon.as_deref() {
                Button::icon_only(icon)
            } else {
                Button::new(props.label.as_str())
            }
        } else {
            Button::new(props.label.as_str())
        };

        if let Some(variant) = props.variant {
            button = button.variant(variant);
        }
        if let Some(size) = props.size {
            button = button.size(size);
        }
        if let Some(icon) = props.leading_icon.as_deref() {
            if !props.icon_only {
                button = button.leading_icon(icon);
            }
        }
        if let Some(trailing_text) = props.trailing_text.as_deref() {
            button = button.trailing_text(trailing_text);
        }
        if let Some(trailing_icon) = props.trailing_icon.as_deref() {
            button = button.trailing_icon(trailing_icon);
        }
        if props.selected {
            button = button.selected(true);
        }

        if ui.button(button).clicked() {
            self.emit_basic(
                &props.common.node_id,
                EventKind::Clicked,
                props.action_id.as_ref(),
            );
        }
    }

    fn render_button_group(&mut self, ui: &mut ComponentUi<'_>, props: &ContractButtonGroup) {
        if props.items.is_empty() {
            return;
        }
        let labels = props
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        if let Some(index) = ui.button_group(ButtonGroup::new(
            make_id(&props.common.node_id, "button_group"),
            labels.as_slice(),
        )) {
            if let Some(item) = props.items.get(index) {
                self.emit_item(
                    &props.common.node_id,
                    EventKind::CommandInvoked,
                    item.action_id.as_ref(),
                    item.item_id.as_str(),
                    item.label.as_str(),
                );
            }
        }
    }

    fn render_input(&mut self, ui: &mut ComponentUi<'_>, props: &ContractInput) {
        let mut value = props.value.clone();
        let mut input = TextInput::new().width(props.width);
        if let Some(placeholder) = props.placeholder.as_deref() {
            input = input.placeholder(placeholder);
        }
        if let Some(icon) = props.leading_icon.as_deref() {
            input = input.leading_icon(icon);
        }

        let response = ui.text_input(&mut value, input);
        if value != props.value {
            self.emit_value(
                &props.common.node_id,
                EventKind::Changed,
                props.action_id.as_ref(),
                Some(EventValue::Text(value.clone())),
                None,
            );
        }
        if response.lost_focus() && ui.ui().input(|input| input.key_pressed(Key::Enter)) {
            self.emit_value(
                &props.common.node_id,
                EventKind::Submitted,
                props.action_id.as_ref(),
                Some(EventValue::Text(value)),
                None,
            );
        }
    }

    fn render_number_input(&mut self, ui: &mut ComponentUi<'_>, props: &ContractNumberInput) {
        let mut value = props.value;
        let mut number = NumberInput::new(make_id(&props.common.node_id, "number_input"))
            .width(props.width)
            .range(props.min..=props.max)
            .speed(props.speed)
            .decimals(props.decimals);
        if let Some(prefix) = props.prefix.as_deref() {
            number = number.prefix(prefix);
        }
        if let Some(suffix) = props.suffix.as_deref() {
            number = number.suffix(suffix);
        }
        if let Some(axis) = props.axis {
            number = number.axis(axis);
        }

        let _ = ui.number_input(&mut value, number);
        if (value - props.value).abs() > f32::EPSILON {
            self.emit_value(
                &props.common.node_id,
                EventKind::Changed,
                props.action_id.as_ref(),
                Some(EventValue::Number(value)),
                None,
            );
        }
    }

    fn render_checkbox(&mut self, ui: &mut ComponentUi<'_>, props: &ContractCheckbox) {
        let mut value = props.value;
        let checkbox = match props.label.as_deref() {
            Some(label) => Checkbox::new().label(label),
            None => Checkbox::new(),
        };
        let _ = ui.checkbox(&mut value, checkbox);
        if value != props.value {
            self.emit_value(
                &props.common.node_id,
                EventKind::Toggled,
                props.action_id.as_ref(),
                Some(EventValue::Boolean(value)),
                None,
            );
        }
    }

    fn render_switch(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSwitch) {
        let mut value = props.value;
        let mut switch = Switch::new();
        if let Some(label) = props.label.as_deref() {
            switch = switch.label(label);
        }
        if let Some(size) = props.size {
            switch = switch.size(size);
        }
        let _ = ui.switch(&mut value, switch);
        if value != props.value {
            self.emit_value(
                &props.common.node_id,
                EventKind::Toggled,
                props.action_id.as_ref(),
                Some(EventValue::Boolean(value)),
                None,
            );
        }
    }

    fn render_select(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSelect) {
        if props.items.is_empty() {
            return;
        }

        let labels = props
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        let mut selected_index = props
            .selected_item_id
            .as_deref()
            .and_then(|item_id| props.items.iter().position(|item| item.item_id == item_id));
        let previous = selected_index;
        let mut select =
            Select::from_id(make_id(&props.common.node_id, "select"), labels.as_slice());
        if let Some(width) = props.width {
            select = select.width(width);
        }
        if let Some(placeholder) = props.placeholder.as_deref() {
            select = select.placeholder(placeholder);
        }
        if let Some(variant) = props.variant {
            select = select.variant(variant);
        }
        if let Some(icon) = props.leading_icon.as_deref() {
            select = select.leading_icon(icon);
        }
        let _ = ui.select(&mut selected_index, select);

        if selected_index != previous {
            if let Some(index) = selected_index {
                if let Some(item) = props.items.get(index) {
                    self.emit_item(
                        &props.common.node_id,
                        EventKind::Selected,
                        item.action_id.as_ref().or(props.action_id.as_ref()),
                        item.item_id.as_str(),
                        item.label.as_str(),
                    );
                }
            }
        }
    }

    fn render_field(&mut self, ui: &mut ComponentUi<'_>, props: &ContractField) {
        let mut value = props.value.clone();
        let mut field = Field::new(props.label.as_str()).width(props.width);
        if let Some(helper_text) = props.helper_text.as_deref() {
            field = field.helper_text(helper_text);
        }
        if let Some(placeholder) = props.placeholder.as_deref() {
            field = field.placeholder(placeholder);
        }
        let _ = ui.field(&mut value, field);
        if value != props.value {
            self.emit_value(
                &props.common.node_id,
                EventKind::Changed,
                props.action_id.as_ref(),
                Some(EventValue::Text(value)),
                None,
            );
        }
    }

    fn render_collapsible(&mut self, ui: &mut ComponentUi<'_>, props: &ContractCollapsible) {
        let mut open = props.open;
        let mut collapsible = Collapsible::new(
            make_id(&props.common.node_id, "collapsible"),
            props.title.as_str(),
        );
        if let Some(icon) = props.leading_icon.as_deref() {
            collapsible = collapsible.leading_icon(icon);
        }
        if let Some(icon) = props.trailing_icon.as_deref() {
            collapsible = collapsible.trailing_icon(icon);
        }
        let _ = ui.collapsible(&mut open, collapsible, |ui| {
            self.render_children(ui, &props.children);
        });

        if open != props.open {
            self.emit_value(
                &props.common.node_id,
                EventKind::Toggled,
                props.action_id.as_ref(),
                Some(EventValue::Boolean(open)),
                None,
            );
            self.emit_basic(
                &props.common.node_id,
                if open {
                    EventKind::Opened
                } else {
                    EventKind::Closed
                },
                props.action_id.as_ref(),
            );
        }
    }

    fn render_dialogue_modal(&mut self, ui: &mut ComponentUi<'_>, props: &ContractDialogueModal) {
        let mut open = props.open;
        let mut confirmed = false;
        let mut cancelled = false;
        ui.dialogue_modal(
            &mut open,
            DialogueModal::new(make_id(&props.common.node_id, "dialogue")).width(props.width),
            |ui, close_requested| {
                let mut ui = ui.components();
                ui.dialogue_header_with_close(
                    DialogueHeader::new(props.title.as_str())
                        .description(props.description.as_deref().unwrap_or_default())
                        .intent(
                            props
                                .intent
                                .unwrap_or(crate::components::DialogueIntent::Default),
                        ),
                    close_requested,
                );

                if !props.children.is_empty() {
                    ui.add_space(12.0);
                    self.render_children(ui.ui_mut(), &props.children);
                }

                ui.add_space(12.0);
                let footer_size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                let _ = ui.ui_mut().allocate_ui_with_layout(
                    footer_size,
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        let mut ui = ui.components();
                        let confirm_label = props.confirm_label.as_deref().unwrap_or("Confirm");
                        let cancel_label = props.cancel_label.as_deref().unwrap_or("Cancel");
                        if ui
                            .button(Button::new(confirm_label).variant(ButtonVariant::Primary))
                            .clicked()
                        {
                            *close_requested = true;
                            confirmed = true;
                        }
                        if ui
                            .button(Button::new(cancel_label).variant(ButtonVariant::Secondary))
                            .clicked()
                        {
                            *close_requested = true;
                            cancelled = true;
                        }
                    },
                );
            },
        );

        self.emit_dialogue_events(props, open, confirmed, cancelled);
    }

    fn emit_dialogue_events(
        &mut self,
        props: &ContractDialogueModal,
        open: bool,
        confirmed: bool,
        cancelled: bool,
    ) {
        if confirmed {
            self.emit_basic(
                &props.common.node_id,
                EventKind::Confirmed,
                props.confirm_action_id.as_ref(),
            );
        }
        if cancelled {
            self.emit_basic(
                &props.common.node_id,
                EventKind::Cancelled,
                props.cancel_action_id.as_ref(),
            );
        }
        if should_emit_dialogue_closed(props.open, open, confirmed, cancelled) {
            self.emit_basic(&props.common.node_id, EventKind::Closed, None);
        }
    }

    fn render_toast_viewport(&mut self, ui: &mut ComponentUi<'_>, props: &ContractToastViewport) {
        let ctx = ui.ctx().clone();
        let now = ctx.input(|input| input.time);
        let mut state = load_toast_viewport_state(ui.ui(), &props.common.node_id);
        let current_ids = props
            .toasts
            .iter()
            .map(|toast| toast.item_id.clone())
            .collect::<BTreeSet<_>>();
        state
            .items
            .retain(|item_id, _| current_ids.contains(item_id));
        state
            .suppressed
            .retain(|item_id| current_ids.contains(item_id));

        let mut active_ids = BTreeSet::new();
        let mut next_expiration: Option<f64> = None;
        for toast in &props.toasts {
            if state.suppressed.contains(toast.item_id.as_str()) {
                continue;
            }

            let shown_at = state
                .items
                .entry(toast.item_id.clone())
                .or_insert_with(|| {
                    self.emit_item(
                        &props.common.node_id,
                        EventKind::Opened,
                        toast.action_id.as_ref(),
                        toast.item_id.as_str(),
                        toast.title.as_str(),
                    );
                    ToastLifecycleState { shown_at: now }
                })
                .shown_at;

            if toast.duration_secs > 0.0
                && now >= shown_at + f64::from(toast.duration_secs.max(0.0))
            {
                self.emit_item(
                    &props.common.node_id,
                    EventKind::Closed,
                    toast.action_id.as_ref(),
                    toast.item_id.as_str(),
                    toast.title.as_str(),
                );
                state.suppressed.insert(toast.item_id.clone());
                continue;
            }

            active_ids.insert(toast.item_id.clone());
            if toast.duration_secs > 0.0 {
                let expiry = shown_at + f64::from(toast.duration_secs.max(0.0));
                next_expiration = Some(match next_expiration {
                    Some(current) => current.min(expiry),
                    None => expiry,
                });
            }
        }

        if let Some(expiry) = next_expiration {
            let remaining = (expiry - now).max(0.0) as f32;
            ctx.request_repaint_after_secs(remaining.max(0.05));
        }

        let mut dismiss_ids = Vec::new();
        let visible_toasts = props
            .toasts
            .iter()
            .rev()
            .filter(|toast| active_ids.contains(toast.item_id.as_str()))
            .take(props.max_visible.max(1))
            .collect::<Vec<_>>();

        if !visible_toasts.is_empty() {
            let host_rect = ui.ui().max_rect();
            let width = props.width.max(1.0);
            let gap = props.gap.max(0.0);
            let overlap = props.overlap.max(0.0);
            let margin = egui::vec2(props.margin_x.max(0.0), props.margin_y.max(0.0));

            let _ = egui::Area::new(make_id(&props.common.node_id, "toast_viewport"))
                .order(Order::Foreground)
                .anchor(
                    toast_anchor(props.placement),
                    toast_anchor_offset(props.placement, margin),
                )
                .constrain_to(host_rect)
                .layout(toast_layout(props.placement))
                .show(&ctx, |ui| {
                    ui.set_min_width(width);
                    ui.set_max_width(width);
                    ui.spacing_mut().item_spacing.y = gap;

                    let _ = ui.with_layout(toast_layout(props.placement), |ui| {
                        for (depth, toast) in visible_toasts.iter().enumerate() {
                            let _ =
                                ui.push_id((&props.common.node_id, toast.item_id.as_str()), |ui| {
                                    if draw_contract_toast(ui, toast, width, depth) {
                                        dismiss_ids.push(toast.item_id.clone());
                                    }
                                });

                            if depth + 1 < visible_toasts.len() && overlap > 0.0 {
                                ui.add_space(-(gap + overlap));
                            }
                        }
                    });
                });
        }

        for dismiss_id in dismiss_ids {
            if let Some(toast) = props
                .toasts
                .iter()
                .find(|toast| toast.item_id == dismiss_id)
            {
                self.emit_item(
                    &props.common.node_id,
                    EventKind::Closed,
                    toast.action_id.as_ref(),
                    toast.item_id.as_str(),
                    toast.title.as_str(),
                );
                state.suppressed.insert(toast.item_id.clone());
            }
        }

        store_toast_viewport_state(ui.ui_mut(), &props.common.node_id, state);
    }

    fn render_hierarchy(&mut self, ui: &mut ComponentUi<'_>, props: &ContractHierarchy) {
        let mut bindings = Vec::new();
        let mut next_runtime_id = 0usize;
        let mut runtime_nodes =
            build_runtime_hierarchy(&props.items, &mut bindings, &mut next_runtime_id);
        let mut selected_runtime = props.selected_item_id.as_deref().and_then(|item_id| {
            bindings
                .iter()
                .position(|binding| binding.item_id == item_id)
        });
        let previous_selected = selected_runtime;
        let mut hierarchy = HierarchyWidget::new(
            make_id(&props.common.node_id, "hierarchy"),
            &mut runtime_nodes,
        )
        .width(props.width)
        .row_height(props.row_height)
        .indent_width(props.indent_width);
        if let Some(icon_style) = props.icon_style {
            hierarchy = hierarchy.icon_style(icon_style);
        }
        if let Some(style) = props.style {
            hierarchy = hierarchy.style(style);
        }

        let _ = ui.hierarchy(&mut selected_runtime, hierarchy);

        if selected_runtime != previous_selected {
            match selected_runtime.and_then(|index| bindings.get(index)) {
                Some(binding) => self.emit_item(
                    &props.common.node_id,
                    EventKind::Selected,
                    binding.action_id.or(props.action_id.as_ref()),
                    binding.item_id,
                    binding.label,
                ),
                None => self.emit_value(
                    &props.common.node_id,
                    EventKind::Selected,
                    props.action_id.as_ref(),
                    None,
                    None,
                ),
            }
        }

        self.emit_hierarchy_open_events(
            &props.common.node_id,
            props.action_id.as_ref(),
            runtime_nodes.as_slice(),
            bindings.as_slice(),
        );
    }

    fn render_color(&mut self, ui: &mut ComponentUi<'_>, props: &ContractColor) {
        let mut color = Color::new(parse_contract_color(&props.fill).unwrap_or(Color32::WHITE))
            .size(props.size);
        if let Some(stroke) = props.stroke.as_ref().and_then(parse_contract_stroke) {
            color = color.stroke(stroke);
        }
        if let Some(corner_radius) = props.corner_radius {
            color = color.rounded(corner_radius);
        }
        let _ = ui.color(color);
    }

    fn render_icon(&mut self, ui: &mut ComponentUi<'_>, props: &ContractIcon) {
        let mut icon = Icon::new(props.name.as_str()).size(props.size);
        if let Some(tint) = props.tint.as_ref().and_then(parse_contract_color) {
            icon = icon.tint(tint);
        }
        let _ = ui.icon(icon);
    }

    fn render_image(&mut self, ui: &mut ComponentUi<'_>, props: &ContractImage) {
        let mut image = contract_image(props.source.as_str())
            .fit_to_exact_size(egui::vec2(props.width.max(1.0), props.height.max(1.0)));
        if let Some(corner_radius) = props.corner_radius {
            image = image.corner_radius(egui::CornerRadius::same(corner_radius));
        }
        let _ = ui.image(image);
    }

    fn render_twemoji(&mut self, ui: &mut ComponentUi<'_>, props: &ContractTwemoji) {
        let _ = ui.twemoji(Twemoji::new(props.emoji.as_str()).size(props.size));
    }

    fn render_kbd(&mut self, ui: &mut ComponentUi<'_>, props: &ContractKbd) {
        let _ = ui.kbd(
            Kbd::new(props.text.as_str())
                .min_width(props.min_width)
                .height(props.height),
        );
    }

    fn render_skeleton(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSkeleton) {
        let mut skeleton = Skeleton::new()
            .width(props.width)
            .height(props.height)
            .animated(props.animated);
        if props.circle {
            skeleton = skeleton.circle(props.width.min(props.height).max(1.0));
        }
        if let Some(corner_radius) = props.corner_radius {
            skeleton = skeleton.corner_radius(corner_radius);
        }
        let _ = ui.skeleton(skeleton);
    }

    fn render_slider(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSlider) {
        let mut value = props.value;
        let _ = ui.slider(
            &mut value,
            Slider::new(props.min..=props.max).width(props.width),
        );
        if (value - props.value).abs() > f32::EPSILON {
            self.emit_value(
                &props.common.node_id,
                EventKind::Changed,
                props.action_id.as_ref(),
                Some(EventValue::Number(value)),
                None,
            );
        }
    }

    fn render_radio(&mut self, ui: &mut ComponentUi<'_>, props: &ContractRadio) {
        let mut value = props.value;
        let mut radio = Radio::new();
        if let Some(label) = props.label.as_deref() {
            radio = radio.label(label);
        }
        if let Some(description) = props.description.as_deref() {
            radio = radio.description(description);
        }
        let _ = ui.radio(&mut value, radio);
        if value != props.value {
            self.emit_value(
                &props.common.node_id,
                EventKind::Toggled,
                props.action_id.as_ref(),
                Some(EventValue::Boolean(value)),
                None,
            );
        }
    }

    fn render_radio_group(&mut self, ui: &mut ComponentUi<'_>, props: &ContractRadioGroup) {
        let options = props
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let mut option = RadioOption::new(index, item.label.as_str());
                if let Some(description) = item.description.as_deref() {
                    option = option.description(description);
                }
                option
            })
            .collect::<Vec<_>>();
        let mut current = props
            .selected_item_id
            .as_deref()
            .and_then(|item_id| props.items.iter().position(|item| item.item_id == item_id));
        let previous = current;
        let _ = ui.radio_group(
            &mut current,
            RadioGroup::new(
                make_id(&props.common.node_id, "radio_group"),
                options.as_slice(),
            )
            .gap(props.gap),
        );
        if current != previous {
            match current.and_then(|index| props.items.get(index)) {
                Some(item) => self.emit_item(
                    &props.common.node_id,
                    EventKind::Selected,
                    item.action_id.as_ref().or(props.action_id.as_ref()),
                    item.item_id.as_str(),
                    item.label.as_str(),
                ),
                None => self.emit_value(
                    &props.common.node_id,
                    EventKind::Selected,
                    props.action_id.as_ref(),
                    None,
                    None,
                ),
            }
        }
    }

    fn render_combobox(&mut self, ui: &mut ComponentUi<'_>, props: &ContractCombobox) {
        let options = props
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        let mut query = props.query.clone();
        let mut selected_indices = props
            .selected_item_ids
            .iter()
            .filter_map(|item_id| props.items.iter().position(|item| item.item_id == *item_id))
            .collect::<Vec<_>>();
        let previous_indices = selected_indices.clone();
        let previous_query = query.clone();
        let mut combobox = Combobox::new(
            make_id(&props.common.node_id, "combobox"),
            options.as_slice(),
        )
        .width(props.width)
        .max_height(props.max_height)
        .searchable(props.searchable);
        if let Some(placeholder) = props.placeholder.as_deref() {
            combobox = combobox.placeholder(placeholder);
        }
        if let Some(filter_placeholder) = props.filter_placeholder.as_deref() {
            combobox = combobox.filter_placeholder(filter_placeholder);
        }
        let _ = ui.combobox(&mut query, &mut selected_indices, combobox);
        if selected_indices != previous_indices {
            let selected_item_ids = selected_indices
                .iter()
                .filter_map(|index| props.items.get(*index))
                .map(|item| item.item_id.clone())
                .collect::<Vec<_>>();
            self.emit_value(
                &props.common.node_id,
                EventKind::Selected,
                props.action_id.as_ref(),
                Some(EventValue::ItemIds(selected_item_ids)),
                None,
            );
        } else if query != previous_query {
            self.emit_value(
                &props.common.node_id,
                EventKind::Changed,
                props.action_id.as_ref(),
                Some(EventValue::Text(query)),
                None,
            );
        }
    }

    fn render_emoji_selector(&mut self, ui: &mut ComponentUi<'_>, props: &ContractEmojiSelector) {
        let mut value = props.value.clone();
        let mut selector = EmojiSelector::new(make_id(&props.common.node_id, "emoji_selector"))
            .popup_width(props.popup_width)
            .popup_max_height(props.popup_max_height);
        if let Some(placeholder) = props.placeholder.as_deref() {
            selector = selector.placeholder(placeholder);
        }
        if let Some(trigger_variant) = props.trigger_variant {
            selector = selector.trigger_variant(trigger_variant);
        }
        let _ = ui.emoji_selector(&mut value, selector);
        if value != props.value {
            self.emit_value(
                &props.common.node_id,
                EventKind::Selected,
                props.action_id.as_ref(),
                Some(EventValue::Text(value)),
                None,
            );
        }
    }

    fn render_pagination(&mut self, ui: &mut ComponentUi<'_>, props: &ContractPagination) {
        let mut current_page = props.current_page;
        let _ = ui.pagination(
            &mut current_page,
            Pagination::new(
                make_id(&props.common.node_id, "pagination"),
                props.page_count,
            )
            .sibling_count(props.sibling_count),
        );
        if current_page != props.current_page {
            self.emit_value(
                &props.common.node_id,
                EventKind::Selected,
                props.action_id.as_ref(),
                Some(EventValue::Number(current_page as f32)),
                None,
            );
        }
    }

    fn render_tooltip(&mut self, ui: &mut ComponentUi<'_>, props: &ContractTooltip) {
        let _ = ui.tooltip(
            Tooltip::new(props.trigger_label.as_str(), props.text.as_str())
                .width(props.width)
                .delay_ms(props.delay_ms)
                .placement(props.placement),
        );
    }

    fn render_popover(&mut self, ui: &mut ComponentUi<'_>, props: &ContractPopover) {
        let mut open = props.open;
        let mut popover = Popover::new(make_id(&props.common.node_id, "popover"))
            .side(props.side)
            .align(props.align)
            .side_offset(props.side_offset)
            .padding(props.padding_x as i8, props.padding_y as i8);
        if let Some(width) = props.width {
            popover = popover.width(width);
        }
        let trigger_label = props.trigger_label.as_deref().unwrap_or("Open");
        let _ = ui.popover(
            &mut open,
            popover,
            |ui| {
                ui.components()
                    .button(Button::new(trigger_label).variant(ButtonVariant::Secondary))
            },
            |ui, _open| self.render_children(ui, &props.children),
        );
        if open != props.open {
            self.emit_value(
                &props.common.node_id,
                if open {
                    EventKind::Opened
                } else {
                    EventKind::Closed
                },
                props.action_id.as_ref(),
                Some(EventValue::Boolean(open)),
                None,
            );
        }
    }

    fn render_dropdown_menu(&mut self, ui: &mut ComponentUi<'_>, props: &ContractDropdownMenu) {
        let mut action_bindings = Vec::new();
        let mut runtime_entries = Vec::new();
        append_runtime_menu_entries(&props.entries, &mut runtime_entries, &mut action_bindings);
        let mut dropdown = DropdownMenu::new(props.trigger_label.as_str())
            .entries(runtime_entries.as_slice())
            .width(props.width);
        if let Some(variant) = props.trigger_variant {
            dropdown = dropdown.trigger_variant(variant);
        }
        let (_, state) = ui.dropdown_menu(dropdown);
        self.emit_menu_action(
            &props.common.node_id,
            props.action_id.as_ref(),
            state.action,
            &action_bindings,
        );
    }

    fn render_context_menu(&mut self, ui: &mut ComponentUi<'_>, props: &ContractContextMenu) {
        let mut action_bindings = Vec::new();
        let mut runtime_entries = Vec::new();
        append_runtime_menu_entries(&props.entries, &mut runtime_entries, &mut action_bindings);
        let (_, state) = ui.context_menu(
            ContextMenu::new(
                make_id(&props.common.node_id, "context_menu"),
                runtime_entries.as_slice(),
            )
            .width(props.width)
            .size(egui::vec2(props.region_width, props.region_height))
            .padding(props.padding_x as i8, props.padding_y as i8),
            |ui| self.render_children(ui, &props.children),
        );
        self.emit_menu_action(
            &props.common.node_id,
            props.action_id.as_ref(),
            state.action,
            &action_bindings,
        );
    }

    fn render_open_with(&mut self, ui: &mut ComponentUi<'_>, props: &ContractOpenWith) {
        let mut action_bindings = Vec::new();
        let mut runtime_entries = Vec::new();
        append_runtime_menu_entries(&props.entries, &mut runtime_entries, &mut action_bindings);
        let mut selected_action = props.selected_item_id.as_deref().and_then(|item_id| {
            action_bindings
                .iter()
                .position(|binding| binding.item_id == item_id)
        });
        let previous = selected_action;
        let mut open_with = OpenWith::new(
            make_id(&props.common.node_id, "open_with"),
            runtime_entries.as_slice(),
        )
        .width(props.width);
        if let Some(placeholder) = props.placeholder.as_deref() {
            open_with = open_with.placeholder(placeholder);
        }
        if let Some(size) = props.size {
            open_with = open_with.size(size);
        }
        if let Some(variant) = props.trigger_variant {
            open_with = open_with.trigger_variant(variant);
        }
        let _ = ui.open_with(&mut selected_action, open_with);
        if selected_action != previous {
            self.emit_menu_action(
                &props.common.node_id,
                props.action_id.as_ref(),
                selected_action,
                &action_bindings,
            );
        }
    }

    fn render_collab_cursor(&mut self, ui: &mut ComponentUi<'_>, props: &ContractCollabCursor) {
        let mut cursor = CollabCursor::new(
            make_id(&props.common.node_id, "collab_cursor"),
            props.name.as_str(),
            egui::pos2(props.x, props.y),
        )
        .size(props.size);
        if let Some(color) = props.color.as_ref().and_then(parse_contract_color) {
            cursor = cursor.color(color);
        }
        let _ = ui.collab_cursor(cursor);
    }

    fn render_icon_toolbar(&mut self, ui: &mut ComponentUi<'_>, props: &ContractIconToolbar) {
        let items = props
            .items
            .iter()
            .map(|item| {
                let mut runtime_item = IconToolbarItem::new(item.icon.as_str());
                if let Some(tooltip) = item.tooltip.as_deref() {
                    runtime_item = runtime_item.tooltip(tooltip);
                }
                if let Some(badge_fill) = item.badge_fill.as_ref().and_then(parse_contract_color) {
                    runtime_item = runtime_item.badge_fill(badge_fill);
                }
                runtime_item
            })
            .collect::<Vec<_>>();
        let mut current = props
            .selected_item_id
            .as_deref()
            .and_then(|item_id| props.items.iter().position(|item| item.item_id == item_id))
            .unwrap_or(0);
        let previous = current;
        let mut toolbar = IconToolbar::new(
            make_id(&props.common.node_id, "icon_toolbar"),
            items.as_slice(),
        )
        .gap(props.gap);
        if let Some(size) = props.size {
            toolbar = toolbar.size(size);
        }
        if let Some(icon_size) = props.icon_size {
            toolbar = toolbar.icon_size(icon_size);
        }
        let _ = ui.icon_toolbar(&mut current, toolbar);
        if current != previous {
            if let Some(item) = props.items.get(current) {
                self.emit_item(
                    &props.common.node_id,
                    EventKind::Selected,
                    item.action_id.as_ref().or(props.action_id.as_ref()),
                    item.item_id.as_str(),
                    item.tooltip.as_deref().unwrap_or(item.icon.as_str()),
                );
            }
        }
    }

    fn render_file_tree(&mut self, ui: &mut ComponentUi<'_>, props: &ContractFileTree) {
        let mut bindings = Vec::new();
        let mut runtime_nodes = build_runtime_file_tree(&props.items, &mut bindings);
        let mut selected_id = props.selected_item_id.as_deref().and_then(|item_id| {
            bindings
                .iter()
                .position(|binding| binding.item_id == item_id)
        });
        let previous_selected_id = selected_id;
        let _ = ui.file_tree(
            &mut selected_id,
            FileTree::new(
                make_id(&props.common.node_id, "file_tree"),
                &mut runtime_nodes,
            )
            .width(props.width)
            .row_height(props.row_height)
            .indent_width(props.indent_width),
        );
        if selected_id != previous_selected_id {
            if let Some(binding) = selected_id.and_then(|index| bindings.get(index)) {
                self.emit_item(
                    &props.common.node_id,
                    EventKind::Selected,
                    binding.action_id.or(props.action_id.as_ref()),
                    binding.item_id,
                    binding.label,
                );
            }
        }
        self.emit_file_tree_open_events(
            &props.common.node_id,
            props.action_id.as_ref(),
            runtime_nodes.as_slice(),
            bindings.as_slice(),
        );
    }

    fn render_drag_board(&mut self, ui: &mut ComponentUi<'_>, props: &ContractDragBoard) {
        let items = props
            .items
            .iter()
            .map(|item| {
                let mut runtime_item = DragBoardItem::new(item.title.as_str());
                if let Some(description) = item.description.as_deref() {
                    runtime_item = runtime_item.description(description);
                }
                runtime_item
            })
            .collect::<Vec<_>>();
        let mut regions = props
            .items
            .iter()
            .map(|item| item.region)
            .collect::<Vec<_>>();
        let previous_regions = regions.clone();
        let _ = ui.drag_board(
            regions.as_mut_slice(),
            DragBoard::new(
                make_id(&props.common.node_id, "drag_board"),
                props.left_title.as_str(),
                props.right_title.as_str(),
                items.as_slice(),
            )
            .height(props.height),
        );
        if let Some((index, next_region)) = regions
            .iter()
            .enumerate()
            .find(|(index, region)| previous_regions.get(*index) != Some(*region))
        {
            if let Some(item) = props.items.get(index) {
                self.emit_value(
                    &props.common.node_id,
                    EventKind::Changed,
                    item.action_id.as_ref().or(props.action_id.as_ref()),
                    Some(EventValue::ItemMove(ContractItemMove {
                        item_id: item.item_id.clone(),
                        from: drag_region_id(item.region).to_owned(),
                        to: drag_region_id(*next_region).to_owned(),
                    })),
                    Some(EventMetadata::item(
                        item.item_id.as_str(),
                        item.title.as_str(),
                    )),
                );
            }
        }
    }

    fn render_audio_playback(&mut self, ui: &mut ComponentUi<'_>, props: &ContractAudioPlayback) {
        let mut playback = AudioPlayback::new(
            make_id(&props.common.node_id, "audio_playback"),
            props.playback_state,
        );
        if let Some(duration_seconds) = props.duration_seconds {
            playback = playback.duration_seconds(duration_seconds);
        }
        let (_, result) = if props.children.is_empty() {
            ui.audio_playback(playback)
        } else {
            ui.audio_playback_with_actions(playback, |ui| {
                self.render_children(ui, &props.children);
            })
        };
        if result.play_pause_clicked {
            self.emit_value(
                &props.common.node_id,
                EventKind::Toggled,
                props.action_id.as_ref(),
                Some(EventValue::Boolean(matches!(
                    props.playback_state,
                    AudioPlaybackState::Paused
                ))),
                None,
            );
        }
    }

    fn render_image_tile(&mut self, ui: &mut ComponentUi<'_>, props: &ContractImageTile) {
        let mut tile = ImageTile::new(contract_image(props.source.as_str()))
            .image_frame(props.image_frame)
            .selected(props.selected);
        if let Some(size) = props.size {
            tile = tile.size(size);
        }
        if let (Some(width), Some(height)) = (props.image_width, props.image_height) {
            tile = tile.image_size(egui::vec2(width, height));
        }
        if let Some(playback_state) = props.playback_state {
            tile = tile.playback_state(playback_state);
        }
        let (_, state) = if props.children.is_empty() {
            ui.image_tile(tile)
        } else {
            ui.image_tile_with_body(tile, |ui| self.render_children(ui, &props.children))
        };
        if state.play_pause_clicked {
            self.emit_basic(
                &props.common.node_id,
                EventKind::Toggled,
                props
                    .play_pause_action_id
                    .as_ref()
                    .or(props.action_id.as_ref()),
            );
        } else if state.tile_clicked {
            self.emit_basic(
                &props.common.node_id,
                EventKind::Clicked,
                props.action_id.as_ref(),
            );
        }
    }

    fn render_command(&mut self, ui: &mut ComponentUi<'_>, props: &ContractCommand) {
        let items = props
            .items
            .iter()
            .map(|item| {
                let mut command_item = CommandItem::new(item.group.as_str(), item.label.as_str());
                if let Some(shortcut) = item.shortcut.as_deref() {
                    command_item = command_item.shortcut(shortcut);
                }
                command_item
            })
            .collect::<Vec<_>>();
        let mut query = props.query.clone();
        let previous_query = query.clone();
        let mut command = Command::new(make_id(&props.common.node_id, "command"))
            .width(props.width)
            .max_height(props.max_height)
            .preview(props.preview)
            .preview_height(props.preview_height);
        if let Some(placeholder) = props.placeholder.as_deref() {
            command = command.placeholder(placeholder);
        }
        let _ = ui.command(&mut query, items.as_slice(), command);
        if query != previous_query {
            self.emit_value(
                &props.common.node_id,
                EventKind::Changed,
                props.action_id.as_ref(),
                Some(EventValue::Text(query)),
                None,
            );
        }
    }

    fn emit_menu_action(
        &mut self,
        node_id: &NodeId,
        fallback_action: Option<&ActionId>,
        action_index: Option<usize>,
        bindings: &[MenuActionBinding<'_>],
    ) {
        if let Some(binding) = action_index.and_then(|index| bindings.get(index)) {
            self.emit_item(
                node_id,
                EventKind::CommandInvoked,
                binding.action_id.or(fallback_action),
                binding.item_id,
                binding.label,
            );
        }
    }

    fn emit_hierarchy_open_events(
        &mut self,
        node_id: &crate::contract::NodeId,
        hierarchy_action: Option<&ActionId>,
        nodes: &[HierarchyWidgetNode],
        bindings: &[HierarchyBinding<'_>],
    ) {
        for node in nodes {
            if let Some(binding) = bindings.get(node.id) {
                if node.expanded != binding.open {
                    self.emit_item(
                        node_id,
                        if node.expanded {
                            EventKind::Opened
                        } else {
                            EventKind::Closed
                        },
                        binding.action_id.or(hierarchy_action),
                        binding.item_id,
                        binding.label,
                    );
                }
                self.emit_hierarchy_open_events(
                    node_id,
                    hierarchy_action,
                    node.children.as_slice(),
                    bindings,
                );
            }
        }
    }

    fn emit_file_tree_open_events(
        &mut self,
        node_id: &NodeId,
        file_tree_action: Option<&ActionId>,
        nodes: &[FileTreeNode<'_>],
        bindings: &[HierarchyBinding<'_>],
    ) {
        for node in nodes {
            if let Some(binding) = bindings.get(node.id) {
                if node.expanded != binding.open {
                    self.emit_item(
                        node_id,
                        if node.expanded {
                            EventKind::Opened
                        } else {
                            EventKind::Closed
                        },
                        binding.action_id.or(file_tree_action),
                        binding.item_id,
                        binding.label,
                    );
                }
                self.emit_file_tree_open_events(
                    node_id,
                    file_tree_action,
                    node.children.as_slice(),
                    bindings,
                );
            }
        }
    }

    fn emit_basic(
        &mut self,
        node_id: &crate::contract::NodeId,
        kind: EventKind,
        action: Option<&ActionId>,
    ) {
        self.events.push(
            ContractEvent::new(node_id.clone(), kind)
                .action(action.cloned())
                .value(None)
                .metadata(None),
        );
    }

    fn emit_value(
        &mut self,
        node_id: &crate::contract::NodeId,
        kind: EventKind,
        action: Option<&ActionId>,
        value: Option<EventValue>,
        metadata: Option<EventMetadata>,
    ) {
        self.events.push(
            ContractEvent::new(node_id.clone(), kind)
                .action(action.cloned())
                .value(value)
                .metadata(metadata),
        );
    }

    fn emit_item(
        &mut self,
        node_id: &crate::contract::NodeId,
        kind: EventKind,
        action: Option<&ActionId>,
        item_id: &str,
        label: &str,
    ) {
        self.emit_value(
            node_id,
            kind,
            action,
            Some(EventValue::ItemId(item_id.to_owned())),
            Some(EventMetadata::item(item_id, label)),
        );
    }
}

fn with_layout_scope<R>(
    ui: &mut ComponentUi<'_>,
    layout: Option<&ContractLayout>,
    add: impl FnOnce(&mut ComponentUi<'_>) -> R,
) -> R {
    if layout.is_none() {
        return add(ui);
    }

    ui.ui_mut()
        .scope(|ui| {
            apply_layout_sizing(ui, layout);
            let mut components = ui.components();
            add(&mut components)
        })
        .inner
}

fn apply_layout_sizing(ui: &mut egui::Ui, layout: Option<&ContractLayout>) {
    let Some(layout) = layout else {
        return;
    };

    if let Some(width) = resolve_contract_length(layout.width.as_ref(), ui.available_width()) {
        ui.set_min_width(width);
        ui.set_max_width(width);
    }
    if let Some(height) = resolve_contract_length(layout.height.as_ref(), ui.available_height()) {
        ui.set_min_height(height);
        ui.set_max_height(height);
    }
    if let Some(min_width) =
        resolve_contract_length(layout.min_width.as_ref(), ui.available_width())
    {
        ui.set_min_width(min_width);
    }
    if let Some(max_width) =
        resolve_contract_length(layout.max_width.as_ref(), ui.available_width())
    {
        ui.set_max_width(max_width);
    }
    if let Some(min_height) =
        resolve_contract_length(layout.min_height.as_ref(), ui.available_height())
    {
        ui.set_min_height(min_height);
    }
    if let Some(max_height) =
        resolve_contract_length(layout.max_height.as_ref(), ui.available_height())
    {
        ui.set_max_height(max_height);
    }
}

fn render_container_children(
    ui: &mut egui::Ui,
    common: &ContractCommon,
    default_direction: ContractDirection,
    default_gap: f32,
    default_justify: ContractJustify,
    default_align: ContractAlign,
    add: impl FnOnce(&mut egui::Ui),
) {
    let plan = container_layout_plan(
        common,
        default_direction,
        default_gap,
        default_justify,
        default_align,
    );

    match plan.direction {
        ContractDirection::Row => {
            let _ = layout_row()
                .gap(plan.gap)
                .justify(map_flow_justify(plan.justify))
                .align(map_flow_align(plan.align))
                .show(ui, add);
        }
        ContractDirection::Column => {
            let _ = layout_column()
                .gap(plan.gap)
                .justify(map_flow_justify(plan.justify))
                .align(map_flow_align(plan.align))
                .show(ui, add);
        }
    }
}

fn ignore_metadata_only_common_fields(common: &ContractCommon) {
    let _ = (
        &common.class,
        &common.class_list,
        &common.slot_classes,
        &common.actions,
    );
}

fn container_layout_plan(
    common: &ContractCommon,
    default_direction: ContractDirection,
    default_gap: f32,
    default_justify: ContractJustify,
    default_align: ContractAlign,
) -> ContainerLayoutPlan {
    let layout = common.layout.as_ref();
    let direction = layout
        .and_then(|layout| layout.direction)
        .unwrap_or(default_direction);
    let justify = layout
        .and_then(|layout| layout.justify)
        .unwrap_or(default_justify);
    let align = layout
        .and_then(|layout| layout.align)
        .unwrap_or(default_align);
    let gap = match direction {
        ContractDirection::Row => layout
            .and_then(|layout| layout.gap_x.or(layout.gap_y))
            .unwrap_or(default_gap),
        ContractDirection::Column => layout
            .and_then(|layout| layout.gap_y.or(layout.gap_x))
            .unwrap_or(default_gap),
    };

    ContainerLayoutPlan {
        direction,
        justify,
        align,
        gap,
    }
}

fn resolve_contract_length(length: Option<&ContractLength>, available: f32) -> Option<f32> {
    match length? {
        ContractLength::Auto => None,
        ContractLength::Px { value } => Some(value.max(0.0)),
        ContractLength::Percent { value } => {
            let ratio = if *value > 1.0 { *value / 100.0 } else { *value };
            Some((available.max(0.0) * ratio.max(0.0)).max(0.0))
        }
    }
}

fn map_flow_justify(value: ContractJustify) -> FlowJustify {
    match value {
        ContractJustify::Start => FlowJustify::Start,
        ContractJustify::Center => FlowJustify::Center,
        ContractJustify::End => FlowJustify::End,
    }
}

fn map_flow_align(value: ContractAlign) -> FlowAlign {
    match value {
        ContractAlign::Start => FlowAlign::Start,
        ContractAlign::Center => FlowAlign::Center,
        ContractAlign::End => FlowAlign::End,
    }
}

fn map_anchor(value: ContractAnchor) -> Align2 {
    match value {
        ContractAnchor::TopLeft => Align2::LEFT_TOP,
        ContractAnchor::TopCenter => Align2::CENTER_TOP,
        ContractAnchor::TopRight => Align2::RIGHT_TOP,
        ContractAnchor::LeftCenter => Align2::LEFT_CENTER,
        ContractAnchor::Center => Align2::CENTER_CENTER,
        ContractAnchor::RightCenter => Align2::RIGHT_CENTER,
        ContractAnchor::BottomLeft => Align2::LEFT_BOTTOM,
        ContractAnchor::BottomCenter => Align2::CENTER_BOTTOM,
        ContractAnchor::BottomRight => Align2::RIGHT_BOTTOM,
    }
}

fn make_id(node_id: &crate::contract::NodeId, suffix: &str) -> Id {
    Id::new(("contract", node_id.as_str(), suffix))
}

fn build_runtime_hierarchy<'a>(
    items: &'a [ContractHierarchyItem],
    bindings: &mut Vec<HierarchyBinding<'a>>,
    next_runtime_id: &mut usize,
) -> Vec<HierarchyWidgetNode> {
    items
        .iter()
        .map(|item| {
            let runtime_id = *next_runtime_id;
            *next_runtime_id = next_runtime_id.saturating_add(1);
            bindings.push(HierarchyBinding {
                item_id: item.item_id.as_str(),
                label: item.label.as_str(),
                action_id: item.action_id.as_ref(),
                open: item.open,
            });
            HierarchyWidgetNode::new(runtime_id, item.label.as_str(), item.kind)
                .expanded(item.open)
                .locked(item.locked)
                .children(build_runtime_hierarchy(
                    item.children.as_slice(),
                    bindings,
                    next_runtime_id,
                ))
        })
        .collect()
}

fn build_runtime_file_tree<'a>(
    items: &'a [ContractFileTreeItem],
    bindings: &mut Vec<HierarchyBinding<'a>>,
) -> Vec<FileTreeNode<'a>> {
    items
        .iter()
        .map(|item| {
            let runtime_id = bindings.len();
            bindings.push(HierarchyBinding {
                item_id: item.item_id.as_str(),
                label: item.label.as_str(),
                action_id: item.action_id.as_ref(),
                open: item.open,
            });
            FileTreeNode::new(runtime_id, item.label.as_str(), item.kind)
                .expanded(item.open)
                .children(build_runtime_file_tree(item.children.as_slice(), bindings))
        })
        .collect()
}

fn append_runtime_menu_entries<'a>(
    entries: &'a [ContractMenuEntry],
    runtime_entries: &mut Vec<DropdownMenuEntry<'a>>,
    bindings: &mut Vec<MenuActionBinding<'a>>,
) {
    for entry in entries {
        match entry {
            ContractMenuEntry::Action(action) => {
                let action_index = bindings.len();
                bindings.push(MenuActionBinding {
                    item_id: action.item_id.as_str(),
                    label: action.label.as_str(),
                    action_id: action.action_id.as_ref(),
                });
                runtime_entries.push(runtime_menu_action_entry(action, action_index));
            }
            ContractMenuEntry::Separator => {
                runtime_entries.push(DropdownMenuEntry::separator());
            }
            ContractMenuEntry::Submenu(submenu) => {
                runtime_entries.push(DropdownMenuEntry::separator());
                let label_index = bindings.len();
                bindings.push(MenuActionBinding {
                    item_id: submenu.label.as_str(),
                    label: submenu.label.as_str(),
                    action_id: None,
                });
                let mut label_action =
                    DropdownMenuAction::new(label_index, submenu.label.as_str()).enabled(false);
                if let Some(icon) = submenu.leading_icon.as_deref() {
                    label_action = label_action.icon(icon);
                }
                runtime_entries.push(DropdownMenuEntry::Action(label_action));
                append_runtime_menu_entries(submenu.entries.as_slice(), runtime_entries, bindings);
            }
        }
    }
}

fn runtime_menu_action_entry<'a>(
    action: &'a ContractMenuAction,
    action_index: usize,
) -> DropdownMenuEntry<'a> {
    match (action.leading_icon.as_deref(), action.shortcut.as_deref()) {
        (Some(icon), Some(shortcut)) => DropdownMenuEntry::action_with_icon_and_shortcut(
            action_index,
            action.label.as_str(),
            icon,
            shortcut,
        ),
        (Some(icon), None) => {
            DropdownMenuEntry::action_with_icon(action_index, action.label.as_str(), icon)
        }
        (None, Some(shortcut)) => {
            DropdownMenuEntry::action_with_shortcut(action_index, action.label.as_str(), shortcut)
        }
        (None, None) => DropdownMenuEntry::action(action_index, action.label.as_str()),
    }
}

fn parse_contract_color(value: &ContractColorValue) -> Option<Color32> {
    parse_color(value.as_str())
}

fn parse_color(value: &str) -> Option<Color32> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("transparent") {
        return Some(Color32::TRANSPARENT);
    }
    if value.eq_ignore_ascii_case("black") {
        return Some(Color32::BLACK);
    }
    if value.eq_ignore_ascii_case("white") {
        return Some(Color32::WHITE);
    }

    let hex = value.strip_prefix('#')?;
    match hex.len() {
        6 => {
            let rgb = u32::from_str_radix(hex, 16).ok()?;
            Some(Color32::from_rgb(
                ((rgb >> 16) & 0xff) as u8,
                ((rgb >> 8) & 0xff) as u8,
                (rgb & 0xff) as u8,
            ))
        }
        8 => {
            let rgba = u32::from_str_radix(hex, 16).ok()?;
            Some(Color32::from_rgba_unmultiplied(
                ((rgba >> 24) & 0xff) as u8,
                ((rgba >> 16) & 0xff) as u8,
                ((rgba >> 8) & 0xff) as u8,
                (rgba & 0xff) as u8,
            ))
        }
        _ => None,
    }
}

fn parse_contract_stroke(stroke: &ContractStroke) -> Option<Stroke> {
    Some(Stroke::new(
        stroke.width.max(0.0),
        parse_contract_color(&stroke.color)?,
    ))
}

fn contract_image(source: &str) -> Image<'static> {
    match source {
        "showcase" | "showcase-image" | "builtin:showcase-image" => {
            Image::from_bytes("bytes://contract/showcase-image.png", SHOWCASE_IMAGE_BYTES)
        }
        source => Image::from_uri(source.to_owned()),
    }
}

fn drag_region_id(region: DragBoardRegion) -> &'static str {
    match region {
        DragBoardRegion::Left => "left",
        DragBoardRegion::Right => "right",
    }
}

fn should_emit_dialogue_closed(
    was_open: bool,
    open: bool,
    confirmed: bool,
    cancelled: bool,
) -> bool {
    was_open && !open && !confirmed && !cancelled
}

fn load_toast_viewport_state(
    ui: &egui::Ui,
    node_id: &crate::contract::NodeId,
) -> ToastViewportState {
    ui.data(|data| {
        data.get_temp::<ToastViewportState>(make_id(node_id, "toast_state"))
            .unwrap_or_default()
    })
}

fn store_toast_viewport_state(
    ui: &mut egui::Ui,
    node_id: &crate::contract::NodeId,
    state: ToastViewportState,
) {
    ui.data_mut(|data| {
        let state_id = make_id(node_id, "toast_state");
        if state == ToastViewportState::default() {
            data.remove::<ToastViewportState>(state_id);
        } else {
            data.insert_temp(state_id, state);
        }
    });
}

fn toast_anchor(placement: ToastPlacement) -> Align2 {
    match placement {
        ToastPlacement::TopLeft => Align2::LEFT_TOP,
        ToastPlacement::TopCenter => Align2::CENTER_TOP,
        ToastPlacement::TopRight => Align2::RIGHT_TOP,
        ToastPlacement::CenterLeft => Align2::LEFT_CENTER,
        ToastPlacement::Center => Align2::CENTER_CENTER,
        ToastPlacement::CenterRight => Align2::RIGHT_CENTER,
        ToastPlacement::BottomLeft => Align2::LEFT_BOTTOM,
        ToastPlacement::BottomCenter => Align2::CENTER_BOTTOM,
        ToastPlacement::BottomRight => Align2::RIGHT_BOTTOM,
    }
}

fn toast_layout(placement: ToastPlacement) -> Layout {
    match placement {
        ToastPlacement::TopLeft => Layout::top_down(Align::Min),
        ToastPlacement::TopCenter => Layout::top_down(Align::Center),
        ToastPlacement::TopRight => Layout::top_down(Align::Max),
        ToastPlacement::CenterLeft => Layout::top_down(Align::Min),
        ToastPlacement::Center => Layout::top_down(Align::Center),
        ToastPlacement::CenterRight => Layout::top_down(Align::Max),
        ToastPlacement::BottomLeft => Layout::bottom_up(Align::Min),
        ToastPlacement::BottomCenter => Layout::bottom_up(Align::Center),
        ToastPlacement::BottomRight => Layout::bottom_up(Align::Max),
    }
}

fn toast_anchor_offset(placement: ToastPlacement, margin: Vec2) -> Vec2 {
    match placement {
        ToastPlacement::TopLeft => margin,
        ToastPlacement::TopCenter => egui::vec2(0.0, margin.y),
        ToastPlacement::TopRight => egui::vec2(-margin.x, margin.y),
        ToastPlacement::CenterLeft => egui::vec2(margin.x, 0.0),
        ToastPlacement::Center => Vec2::ZERO,
        ToastPlacement::CenterRight => egui::vec2(-margin.x, 0.0),
        ToastPlacement::BottomLeft => egui::vec2(margin.x, -margin.y),
        ToastPlacement::BottomCenter => egui::vec2(0.0, -margin.y),
        ToastPlacement::BottomRight => egui::vec2(-margin.x, -margin.y),
    }
}

fn draw_contract_toast(
    ui: &mut egui::Ui,
    toast: &ContractToastItem,
    width: f32,
    depth: usize,
) -> bool {
    const TOAST_FRAME_PADDING_X: i8 = 10;
    const TOAST_FRAME_PADDING_Y: i8 = 10;
    const TOAST_CLOSE_BUTTON_SIZE: f32 = 22.0;

    let runtime = crate::theme::runtime_for_ui(ui);
    let palette = contract_toast_palette(runtime, toast.intent, depth);
    let shadow = contract_toast_shadow(runtime, depth);
    let inner_width = (width - f32::from(TOAST_FRAME_PADDING_X * 2)).max(1.0);
    let mut dismissed = false;

    let _ = ui.with_layout(Layout::top_down(Align::Min), |ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);

        let _ = surface_frame(
            ui,
            SurfaceFrame::new(palette.fill, palette.stroke)
                .corner_radius(tokens::radius_lg(runtime))
                .padding(TOAST_FRAME_PADDING_X, TOAST_FRAME_PADDING_Y)
                .shadow(shadow),
            |ui| {
                ui.set_min_width(inner_width);
                ui.set_max_width(inner_width);

                let header_width = ui.available_width();
                let _ = ui.allocate_ui_with_layout(
                    egui::vec2(header_width, TOAST_CLOSE_BUTTON_SIZE),
                    Layout::left_to_right(Align::Center),
                    |ui: &mut egui::Ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let mut components = ui.components();
                        let _ = components.label(
                            Label::new(toast.title.as_str())
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );

                        let trailing_width = ui.available_width().max(0.0);
                        let _ = ui.allocate_ui_with_layout(
                            egui::vec2(trailing_width, TOAST_CLOSE_BUTTON_SIZE),
                            Layout::right_to_left(Align::Center),
                            |ui: &mut egui::Ui| {
                                let mut components = ui.components();
                                if components
                                    .button(
                                        Button::icon_only("x")
                                            .variant(ButtonVariant::Ghost)
                                            .size(ControlSize::Sm)
                                            .icon_size(12.0)
                                            .icon_tint(tokens::text_muted(runtime))
                                            .min_size(egui::vec2(
                                                TOAST_CLOSE_BUTTON_SIZE,
                                                TOAST_CLOSE_BUTTON_SIZE,
                                            )),
                                    )
                                    .clicked()
                                {
                                    dismissed = true;
                                }
                            },
                        );
                    },
                );

                if let Some(description) = toast.description.as_deref() {
                    ui.add_space(4.0);
                    let mut components = ui.components();
                    let _ =
                        components.label(Label::new(description).tone(LabelTone::Muted).size(12.0));
                }
            },
        );
    });

    dismissed
}

fn contract_toast_palette(
    runtime: crate::theme::ThemeRuntime,
    intent: ToastIntent,
    depth: usize,
) -> ToastPalette {
    let card_fill = tokens::card_background(runtime);
    let border = tokens::separator(runtime);
    let mut palette = match intent {
        ToastIntent::Neutral => ToastPalette {
            fill: card_fill,
            stroke: Stroke::new(1.0, border),
        },
        ToastIntent::Success => {
            let accent = Color32::from_rgb(34, 197, 94);
            ToastPalette {
                fill: card_fill
                    .lerp_to_gamma(accent, if runtime.mode.is_dark() { 0.18 } else { 0.1 }),
                stroke: Stroke::new(1.0, border.lerp_to_gamma(accent, 0.55)),
            }
        }
        ToastIntent::Destructive => {
            let accent = crate::theme::resolved_color(runtime, ColorRole::Destructive);
            ToastPalette {
                fill: card_fill
                    .lerp_to_gamma(accent, if runtime.mode.is_dark() { 0.2 } else { 0.12 }),
                stroke: Stroke::new(1.0, border.lerp_to_gamma(accent, 0.6)),
            }
        }
    };

    let shade = (depth as f32 * 0.075).clamp(0.0, 0.28);
    let stroke_shade = (depth as f32 * 0.05).clamp(0.0, 0.18);
    palette.fill = palette
        .fill
        .lerp_to_gamma(tokens::muted_surface(runtime), shade * 0.8)
        .gamma_multiply(1.0 - (shade * 0.22));
    palette.stroke.color = palette
        .stroke
        .color
        .lerp_to_gamma(tokens::input_border(runtime), stroke_shade);
    palette
}

fn contract_toast_shadow(runtime: crate::theme::ThemeRuntime, depth: usize) -> egui::Shadow {
    let mut shadow = tokens::tailwind_shadow_md(runtime);
    let fade = (1.0 - (depth as f32 * 0.18)).clamp(0.3, 1.0);
    shadow.color = shadow.color.gamma_multiply(fade);
    shadow.blur = (shadow.blur as f32 * (1.0 - (depth as f32 * 0.12)).clamp(0.45, 1.0)) as u8;
    shadow
}

#[cfg(test)]
mod tests {
    use super::{
        container_layout_plan, contract_toast_shadow, render_tree, should_emit_dialogue_closed,
    };
    use crate::components::ToastIntent;
    use crate::contract::{
        ContractActions, ContractAlign, ContractButton, ContractCommon, ContractDirection,
        ContractDisplay, ContractEdges, ContractJustify, ContractLayout, ContractLength,
        ContractNode, ContractToastItem, ContractToastViewport, ContractTrack, ContractTree,
        EventKind,
    };
    use crate::theme::{self, ThemeMode, ThemeSpec};
    use crate::ui::tokens;
    use egui::{pos2, CentralPanel, Context, Event, Modifiers, PointerButton, RawInput};
    use std::collections::BTreeMap;

    fn run_frame(
        context: &Context,
        input: RawInput,
        tree: &ContractTree,
    ) -> Vec<crate::contract::ContractEvent> {
        let mut events = Vec::new();
        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                let new_events = render_tree(ui, tree);
                if !new_events.is_empty() {
                    events = new_events;
                }
            });
        });
        events
    }

    #[test]
    fn button_click_emits_clicked_event() {
        let context = Context::default();
        let tree = ContractTree::new(ContractNode::Button(ContractButton {
            common: ContractCommon::new("save"),
            label: "Save".to_owned(),
            action_id: Some("save.clicked".into()),
            variant: None,
            size: None,
            leading_icon: None,
            trailing_text: None,
            trailing_icon: None,
            icon_only: false,
            selected: false,
        }));

        let mut events = run_frame(&context, RawInput::default(), &tree);
        assert!(events.is_empty());

        let click_input = RawInput {
            events: vec![
                Event::PointerMoved(pos2(20.0, 20.0)),
                Event::PointerButton {
                    pos: pos2(20.0, 20.0),
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..RawInput::default()
        };
        let _ = run_frame(&context, click_input, &tree);

        let release_input = RawInput {
            events: vec![
                Event::PointerMoved(pos2(20.0, 20.0)),
                Event::PointerButton {
                    pos: pos2(20.0, 20.0),
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..RawInput::default()
        };
        events = run_frame(&context, release_input, &tree);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Clicked);
    }

    #[test]
    fn dialogue_closed_only_emits_for_non_confirm_non_cancel_paths() {
        assert!(!should_emit_dialogue_closed(true, false, true, false));
        assert!(!should_emit_dialogue_closed(true, false, false, true));
        assert!(should_emit_dialogue_closed(true, false, false, false));
        assert!(!should_emit_dialogue_closed(false, false, false, false));
    }

    #[test]
    fn contract_toast_shadow_keeps_md_offset_and_fades_with_depth() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);

        let runtime = theme::runtime_for_context(&context);
        let base = tokens::tailwind_shadow_md(runtime);

        assert_eq!(contract_toast_shadow(runtime, 0), base);

        let stacked = contract_toast_shadow(runtime, 2);
        assert_eq!(stacked.offset, base.offset);
        assert!(stacked.blur < base.blur);
        assert!(stacked.color.a() < base.color.a());
    }

    #[test]
    fn toast_viewport_emits_opened_then_closed_once_on_expiry() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);

        let toast_tree = ContractTree::new(ContractNode::ToastViewport(ContractToastViewport {
            common: ContractCommon::new("toast.viewport"),
            placement: crate::components::ToastPlacement::BottomRight,
            width: 320.0,
            margin_x: 16.0,
            margin_y: 16.0,
            gap: 8.0,
            overlap: 0.0,
            max_visible: 4,
            toasts: vec![ContractToastItem {
                item_id: "toast.saved".to_owned(),
                title: "Saved".to_owned(),
                description: Some("The contract tree round-tripped successfully.".to_owned()),
                intent: ToastIntent::Success,
                duration_secs: 0.05,
                action_id: Some("toast.saved".into()),
            }],
        }));

        let events = run_frame(
            &context,
            RawInput {
                time: Some(0.0),
                ..RawInput::default()
            },
            &toast_tree,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Opened);

        let events = run_frame(
            &context,
            RawInput {
                time: Some(0.1),
                ..RawInput::default()
            },
            &toast_tree,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Closed);

        let events = run_frame(
            &context,
            RawInput {
                time: Some(0.2),
                ..RawInput::default()
            },
            &toast_tree,
        );
        assert!(events.is_empty());

        let cleared_tree = ContractTree::new(ContractNode::ToastViewport(ContractToastViewport {
            common: ContractCommon::new("toast.viewport"),
            placement: crate::components::ToastPlacement::BottomRight,
            width: 320.0,
            margin_x: 16.0,
            margin_y: 16.0,
            gap: 8.0,
            overlap: 0.0,
            max_visible: 4,
            toasts: Vec::new(),
        }));
        let events = run_frame(
            &context,
            RawInput {
                time: Some(0.3),
                ..RawInput::default()
            },
            &cleared_tree,
        );
        assert!(events.is_empty());

        let events = run_frame(
            &context,
            RawInput {
                time: Some(0.4),
                ..RawInput::default()
            },
            &toast_tree,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Opened);
    }

    #[test]
    fn common_actions_and_classes_remain_metadata_only() {
        let context = Context::default();
        let mut slot_classes = BTreeMap::new();
        slot_classes.insert(String::from("icon"), String::from("text-lg"));

        let tree = ContractTree::new(ContractNode::Button(ContractButton {
            common: ContractCommon {
                node_id: "save".into(),
                visible: true,
                enabled: true,
                class: Some(String::from("font-semibold")),
                class_list: vec![String::from("font-semibold"), String::from("text-lg")],
                slot_classes,
                actions: ContractActions {
                    click: Some("common.click".into()),
                    ..ContractActions::default()
                },
                layout: None,
            },
            label: "Save".to_owned(),
            action_id: Some("button.click".into()),
            variant: None,
            size: None,
            leading_icon: None,
            trailing_text: None,
            trailing_icon: None,
            icon_only: false,
            selected: false,
        }));

        let _ = run_frame(&context, RawInput::default(), &tree);
        let _ = run_frame(
            &context,
            RawInput {
                events: vec![
                    Event::PointerMoved(pos2(20.0, 20.0)),
                    Event::PointerButton {
                        pos: pos2(20.0, 20.0),
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    },
                ],
                ..RawInput::default()
            },
            &tree,
        );
        let events = run_frame(
            &context,
            RawInput {
                events: vec![
                    Event::PointerMoved(pos2(20.0, 20.0)),
                    Event::PointerButton {
                        pos: pos2(20.0, 20.0),
                        button: PointerButton::Primary,
                        pressed: false,
                        modifiers: Modifiers::NONE,
                    },
                ],
                ..RawInput::default()
            },
            &tree,
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Clicked);
        assert_eq!(
            events[0].action_id.as_ref().map(|id| id.as_str()),
            Some("button.click")
        );
    }

    #[test]
    fn partial_layout_plan_applies_flow_container_overrides() {
        let mut common = ContractCommon::new("layout.node");
        common.layout = Some(ContractLayout {
            direction: Some(ContractDirection::Column),
            gap_y: Some(14.0),
            align: Some(ContractAlign::End),
            justify: Some(ContractJustify::Center),
            ..ContractLayout::default()
        });

        let plan = container_layout_plan(
            &common,
            ContractDirection::Row,
            8.0,
            ContractJustify::Start,
            ContractAlign::Start,
        );

        assert_eq!(plan.direction, ContractDirection::Column);
        assert_eq!(plan.gap, 14.0);
        assert_eq!(plan.align, ContractAlign::End);
        assert_eq!(plan.justify, ContractJustify::Center);
    }

    #[test]
    fn unsupported_layout_fields_are_ignored_without_breaking_interaction() {
        let context = Context::default();
        let tree = ContractTree::new(ContractNode::Button(ContractButton {
            common: ContractCommon {
                node_id: "gridy".into(),
                visible: true,
                enabled: true,
                class: None,
                class_list: Vec::new(),
                slot_classes: BTreeMap::new(),
                actions: ContractActions::default(),
                layout: Some(ContractLayout {
                    display: Some(ContractDisplay::Grid),
                    grow: Some(1.0),
                    shrink: Some(1.0),
                    basis: Some(ContractLength::Px { value: 120.0 }),
                    padding: Some(ContractEdges {
                        top: 4.0,
                        right: 8.0,
                        bottom: 4.0,
                        left: 8.0,
                    }),
                    margin: Some(ContractEdges {
                        top: 3.0,
                        right: 3.0,
                        bottom: 3.0,
                        left: 3.0,
                    }),
                    wrap: Some(true),
                    columns: vec![ContractTrack::Fr { value: 1.0 }],
                    rows: vec![ContractTrack::Auto],
                    col_span: Some(2),
                    row_span: Some(2),
                    overflow_x: Some(crate::contract::ContractOverflow::Scroll),
                    overflow_y: Some(crate::contract::ContractOverflow::Hidden),
                    ..ContractLayout::default()
                }),
            },
            label: "Save".to_owned(),
            action_id: Some("button.click".into()),
            variant: None,
            size: None,
            leading_icon: None,
            trailing_text: None,
            trailing_icon: None,
            icon_only: false,
            selected: false,
        }));

        let _ = run_frame(&context, RawInput::default(), &tree);
        let _ = run_frame(
            &context,
            RawInput {
                events: vec![
                    Event::PointerMoved(pos2(20.0, 20.0)),
                    Event::PointerButton {
                        pos: pos2(20.0, 20.0),
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    },
                ],
                ..RawInput::default()
            },
            &tree,
        );
        let events = run_frame(
            &context,
            RawInput {
                events: vec![
                    Event::PointerMoved(pos2(20.0, 20.0)),
                    Event::PointerButton {
                        pos: pos2(20.0, 20.0),
                        button: PointerButton::Primary,
                        pressed: false,
                        modifiers: Modifiers::NONE,
                    },
                ],
                ..RawInput::default()
            },
            &tree,
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Clicked);
    }
}
