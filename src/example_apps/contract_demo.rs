use crate::components::{
    ButtonVariant, DialogueIntent, HierarchyItemKind, LabelTone, LabelWeight, SelectVariant,
    ToastIntent, ToastPlacement,
};
use crate::contract::{
    registry, render_tree, ActionId, ContractAnchor, ContractButton, ContractButtonGroup,
    ContractCard, ContractCheckbox, ContractChoiceItem, ContractCollapsible, ContractColumn,
    ContractCommon, ContractDialogueModal, ContractEvent, ContractField, ContractHierarchy,
    ContractHierarchyItem, ContractInput, ContractInset, ContractLabel, ContractMenu,
    ContractMenuAction, ContractMenuBar, ContractMenuEntry, ContractNode, ContractNumberInput,
    ContractProgress, ContractRow, ContractSelect, ContractSidebar, ContractSizedBox,
    ContractSpacer, ContractSpinner, ContractSwitch, ContractTabItem, ContractTabs,
    ContractTabsStyle, ContractToastItem, ContractToastViewport, ContractToolbar, ContractTree,
    EventKind, EventValue,
};
use crate::theme::{self, BaseColor, ThemeMode, ThemeSpec};
use egui::{CentralPanel, ScrollArea};
use std::collections::BTreeSet;

pub const WINDOW_TITLE: &str = "egui-component Contract Demo";
pub const WINDOW_INNER_SIZE: [f32; 2] = [1320.0, 920.0];

const EVENT_LOG_LIMIT: usize = 10;

pub struct ContractDemoApp {
    project_name: String,
    owner: String,
    rotation: f32,
    snap_to_grid: bool,
    auto_save: bool,
    sidebar_open: bool,
    modal_open: bool,
    details_open: bool,
    active_tab: Option<String>,
    status: Option<String>,
    hierarchy_selected: Option<String>,
    open_hierarchy_items: BTreeSet<String>,
    next_toast_id: usize,
    toasts: Vec<ContractToastItem>,
    event_log: Vec<String>,
}

impl Default for ContractDemoApp {
    fn default() -> Self {
        Self {
            project_name: "Forest Village".to_owned(),
            owner: "Editor Team".to_owned(),
            rotation: 24.0,
            snap_to_grid: true,
            auto_save: true,
            sidebar_open: false,
            modal_open: false,
            details_open: true,
            active_tab: Some("overview".to_owned()),
            status: Some("review".to_owned()),
            hierarchy_selected: Some("scene-root".to_owned()),
            open_hierarchy_items: BTreeSet::from([
                "scene-root".to_owned(),
                "characters".to_owned(),
                "props".to_owned(),
            ]),
            next_toast_id: 1,
            toasts: vec![ContractToastItem {
                item_id: "toast.welcome".to_owned(),
                title: "Contract Demo Ready".to_owned(),
                description: Some(
                    "Toast lifecycle stays host-owned and comes back as semantic opened/closed events."
                        .to_owned(),
                ),
                intent: ToastIntent::Neutral,
                duration_secs: 0.0,
                action_id: None,
            }],
            event_log: vec!["Host bootstrapped the contract demo.".to_owned()],
        }
    }
}

pub fn install_context(ctx: &egui::Context) {
    theme::install(ctx, ThemeSpec::preset(BaseColor::Slate), ThemeMode::System);
}

pub fn update(app: &mut ContractDemoApp, ctx: &egui::Context) {
    let tree = app.build_tree();
    let mut events = Vec::new();

    CentralPanel::default().show(ctx, |ui| {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                events = render_tree(ui, &tree);
            });
    });

    if !events.is_empty() {
        for event in &events {
            app.apply_event(event);
        }
        ctx.request_repaint();
    }
}

impl ContractDemoApp {
    fn build_tree(&self) -> ContractTree {
        ContractTree::new(column_node(
            "contract-demo.root",
            12.0,
            vec![
                ContractNode::MenuBar(ContractMenuBar {
                    common: common("contract-demo.menu-bar"),
                    menus: vec![
                        ContractMenu {
                            menu_id: "file".to_owned(),
                            label: "File".to_owned(),
                            width: 220.0,
                            entries: vec![
                                ContractMenuEntry::Action(ContractMenuAction {
                                    item_id: "save".to_owned(),
                                    label: "Save Scene".to_owned(),
                                    action_id: Some(action("menu.file.save")),
                                    leading_icon: Some("save".to_owned()),
                                    shortcut: Some("Cmd+S".to_owned()),
                                }),
                                ContractMenuEntry::Action(ContractMenuAction {
                                    item_id: "modal".to_owned(),
                                    label: "Publish".to_owned(),
                                    action_id: Some(action("menu.file.publish")),
                                    leading_icon: Some("rocket".to_owned()),
                                    shortcut: Some("Shift+Cmd+P".to_owned()),
                                }),
                                ContractMenuEntry::Separator,
                                ContractMenuEntry::Action(ContractMenuAction {
                                    item_id: "autosave".to_owned(),
                                    label: if self.auto_save {
                                        "Pause Auto Save".to_owned()
                                    } else {
                                        "Resume Auto Save".to_owned()
                                    },
                                    action_id: Some(action("menu.file.autosave")),
                                    leading_icon: Some("clock-3".to_owned()),
                                    shortcut: None,
                                }),
                            ],
                        },
                        ContractMenu {
                            menu_id: "view".to_owned(),
                            label: "View".to_owned(),
                            width: 208.0,
                            entries: vec![
                                ContractMenuEntry::Action(ContractMenuAction {
                                    item_id: "sidebar".to_owned(),
                                    label: "Open Sidebar".to_owned(),
                                    action_id: Some(action("menu.view.sidebar")),
                                    leading_icon: Some("panel-right-open".to_owned()),
                                    shortcut: None,
                                }),
                                ContractMenuEntry::Action(ContractMenuAction {
                                    item_id: "details".to_owned(),
                                    label: if self.details_open {
                                        "Collapse Details".to_owned()
                                    } else {
                                        "Expand Details".to_owned()
                                    },
                                    action_id: Some(action("menu.view.details")),
                                    leading_icon: Some("chevrons-down".to_owned()),
                                    shortcut: None,
                                }),
                            ],
                        },
                    ],
                }),
                ContractNode::Toolbar(ContractToolbar {
                    common: common("contract-demo.toolbar"),
                    anchor: ContractAnchor::TopCenter,
                    offset_x: 0.0,
                    offset_y: 10.0,
                    children: vec![
                        icon_button("contract-demo.toolbar.move", "move", "toolbar.move"),
                        icon_button("contract-demo.toolbar.rotate", "rotate-ccw", "toolbar.rotate"),
                        icon_button("contract-demo.toolbar.scale", "maximize-2", "toolbar.scale"),
                    ],
                }),
                card_node(
                    "contract-demo.canvas",
                    vec![
                        heading_node("contract-demo.heading", "Luau Contract Layer"),
                        muted_node(
                            "contract-demo.subtitle",
                            format!(
                                "Host-owned state, one declarative tree in, semantic events out. Active tab: {}",
                                self.active_tab.as_deref().unwrap_or("overview")
                            ),
                        ),
                        separator_node("contract-demo.canvas.sep"),
                        ContractNode::Tabs(ContractTabs {
                            common: common("contract-demo.tabs"),
                            style: ContractTabsStyle::Segmented,
                            selected_item_id: self.active_tab.clone(),
                            items: vec![
                                tab_item("overview", "Overview", "layout-panel-top"),
                                tab_item("inspect", "Inspect", "scan-search"),
                                tab_item("history", "History", "history"),
                            ],
                        }),
                        row_node(
                            "contract-demo.primary-row",
                            12.0,
                            vec![
                                card_node(
                                    "contract-demo.form-card",
                                    vec![
                                        subheading_node(
                                            "contract-demo.form-card.title",
                                            "Inspector Controls",
                                        ),
                                        ContractNode::Input(ContractInput {
                                            common: common("contract-demo.project-name"),
                                            value: self.project_name.clone(),
                                            action_id: Some(action("field.project_name")),
                                            placeholder: Some("Project name".to_owned()),
                                            leading_icon: Some("folder-kanban".to_owned()),
                                            width: 280.0,
                                        }),
                                        ContractNode::Field(ContractField {
                                            common: common("contract-demo.owner"),
                                            label: "Owner".to_owned(),
                                            value: self.owner.clone(),
                                            action_id: Some(action("field.owner")),
                                            helper_text: Some(
                                                "This stays authoritative in the host.".to_owned(),
                                            ),
                                            placeholder: Some("Assignee".to_owned()),
                                            width: 280.0,
                                        }),
                                        ContractNode::Select(ContractSelect {
                                            common: common("contract-demo.status"),
                                            action_id: Some(action("field.status")),
                                            selected_item_id: self.status.clone(),
                                            placeholder: Some("Select status".to_owned()),
                                            width: Some(240.0),
                                            variant: Some(SelectVariant::Default),
                                            leading_icon: Some("badge-check".to_owned()),
                                            items: vec![
                                                choice_item("draft", "Draft"),
                                                choice_item("review", "Review"),
                                                choice_item("approved", "Approved"),
                                                choice_item("archived", "Archived"),
                                            ],
                                        }),
                                        ContractNode::NumberInput(ContractNumberInput {
                                            common: common("contract-demo.rotation"),
                                            value: self.rotation,
                                            action_id: Some(action("field.rotation")),
                                            width: 92.0,
                                            min: 0.0,
                                            max: 360.0,
                                            speed: 0.35,
                                            decimals: 1,
                                            prefix: None,
                                            suffix: Some("deg".to_owned()),
                                            axis: None,
                                        }),
                                        row_node(
                                            "contract-demo.bool-row",
                                            12.0,
                                            vec![
                                                ContractNode::Checkbox(ContractCheckbox {
                                                    common: common("contract-demo.snap"),
                                                    value: self.snap_to_grid,
                                                    action_id: Some(action("toggle.snap")),
                                                    label: Some("Snap To Grid".to_owned()),
                                                }),
                                                ContractNode::Switch(ContractSwitch {
                                                    common: common("contract-demo.autosave"),
                                                    value: self.auto_save,
                                                    action_id: Some(action("toggle.autosave")),
                                                    label: Some("Auto Save".to_owned()),
                                                    size: None,
                                                }),
                                            ],
                                        ),
                                        ContractNode::ButtonGroup(ContractButtonGroup {
                                            common: common("contract-demo.align-actions"),
                                            items: vec![
                                                action_item("left", "Align Left", "command.align_left"),
                                                action_item("center", "Align Center", "command.align_center"),
                                                action_item("right", "Align Right", "command.align_right"),
                                            ],
                                        }),
                                        row_node(
                                            "contract-demo.action-row",
                                            8.0,
                                            vec![
                                                button_node(
                                                    "contract-demo.open-sidebar",
                                                    "Open Sidebar",
                                                    "button.open_sidebar",
                                                    ButtonVariant::Secondary,
                                                    Some("panel-right-open"),
                                                ),
                                                button_node(
                                                    "contract-demo.open-modal",
                                                    "Publish",
                                                    "button.open_modal",
                                                    ButtonVariant::Primary,
                                                    Some("rocket"),
                                                ),
                                                button_node(
                                                    "contract-demo.save",
                                                    "Save",
                                                    "button.save",
                                                    ButtonVariant::Ghost,
                                                    Some("save"),
                                                ),
                                            ],
                                        ),
                                    ],
                                ),
                                card_node(
                                    "contract-demo.structure-card",
                                    vec![
                                        subheading_node(
                                            "contract-demo.structure-card.title",
                                            "Hierarchy And Feedback",
                                        ),
                                        ContractNode::Hierarchy(ContractHierarchy {
                                            common: common("contract-demo.hierarchy"),
                                            action_id: Some(action("hierarchy.selection")),
                                            selected_item_id: self.hierarchy_selected.clone(),
                                            width: 340.0,
                                            row_height: 32.0,
                                            indent_width: 18.0,
                                            icon_style: None,
                                            style: None,
                                            items: demo_hierarchy(self),
                                        }),
                                        muted_node(
                                            "contract-demo.progress.label",
                                            "Build Progress".to_owned(),
                                        ),
                                        ContractNode::Progress(ContractProgress {
                                            common: common("contract-demo.progress"),
                                            value: (self.rotation / 360.0).clamp(0.0, 1.0),
                                            width: 240.0,
                                            height: 10.0,
                                        }),
                                        muted_node(
                                            "contract-demo.status-copy",
                                            "Schema export and semantic events are generated from Rust source of truth."
                                                .to_owned(),
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        ContractNode::Collapsible(ContractCollapsible {
                            common: common("contract-demo.details"),
                            title: "Advanced Notes".to_owned(),
                            action_id: Some(action("section.details")),
                            open: self.details_open,
                            leading_icon: Some("flask-conical".to_owned()),
                            trailing_icon: Some("sparkles".to_owned()),
                            children: vec![
                                muted_node(
                                    "contract-demo.details.copy",
                                    "This section is rendered through the contract layer and the host updates its open state from semantic events."
                                        .to_owned(),
                                ),
                                ContractNode::Label(ContractLabel {
                                    common: common("contract-demo.details.state"),
                                    text: format!(
                                        "Current status: {}",
                                        self.status.as_deref().unwrap_or("review")
                                    ),
                                    tone: Some(LabelTone::Secondary),
                                    weight: Some(LabelWeight::Semibold),
                                    size: Some(12.0),
                                    truncate: false,
                                }),
                            ],
                        }),
                        row_node(
                            "contract-demo.foundation-row",
                            12.0,
                            vec![
                                card_node(
                                    "contract-demo.layout-primitives",
                                    self.layout_primitives_nodes(),
                                ),
                                card_node(
                                    "contract-demo.registry-card",
                                    self.supported_family_nodes(),
                                ),
                            ],
                        ),
                    ],
                ),
                ContractNode::Sidebar(ContractSidebar {
                    common: common("contract-demo.sidebar"),
                    title: Some("Workspace Summary".to_owned()),
                    side: crate::components::SidebarSide::Right,
                    width: 320.0,
                    open: self.sidebar_open,
                    children: vec![
                        muted_node(
                            "contract-demo.sidebar.copy",
                            "The sidebar open state stays in the host. Backdrop clicks and Escape emit `closed`."
                                .to_owned(),
                        ),
                        ContractNode::Label(ContractLabel {
                            common: common("contract-demo.sidebar.selection"),
                            text: format!(
                                "Selected hierarchy item: {}",
                                self.hierarchy_selected.as_deref().unwrap_or("none")
                            ),
                            tone: Some(LabelTone::Secondary),
                            weight: Some(LabelWeight::Semibold),
                            size: Some(12.0),
                            truncate: false,
                        }),
                        button_node(
                            "contract-demo.sidebar.close",
                            "Close Sidebar",
                            "button.close_sidebar",
                            ButtonVariant::Secondary,
                            Some("x"),
                        ),
                    ],
                }),
                ContractNode::DialogueModal(ContractDialogueModal {
                    common: common("contract-demo.dialogue"),
                    open: self.modal_open,
                    title: "Publish Changes".to_owned(),
                    description: Some(
                        "The dialogue footer buttons map to semantic `confirmed` and `cancelled` events."
                            .to_owned(),
                    ),
                    confirm_label: Some("Ship It".to_owned()),
                    cancel_label: Some("Not Yet".to_owned()),
                    intent: Some(DialogueIntent::Default),
                    width: 420.0,
                    confirm_action_id: Some(action("modal.confirm")),
                    cancel_action_id: Some(action("modal.cancel")),
                    children: vec![
                        row_node(
                            "contract-demo.dialogue.status-row",
                            8.0,
                            vec![
                                ContractNode::Spinner(ContractSpinner {
                                    common: common("contract-demo.spinner"),
                                    size: 18.0,
                                }),
                                muted_node(
                                    "contract-demo.dialogue.status-copy",
                                    "Reviewing the contract payload before dispatch."
                                        .to_owned(),
                                ),
                            ],
                        ),
                        muted_node(
                            "contract-demo.dialogue.body",
                            "Clay can rebuild this tree from Luau every frame while Rust keeps the authoritative state and event handling."
                                .to_owned(),
                        ),
                    ],
                }),
                ContractNode::ToastViewport(ContractToastViewport {
                    common: common("contract-demo.toasts"),
                    placement: ToastPlacement::BottomRight,
                    width: 320.0,
                    margin_x: 20.0,
                    margin_y: 20.0,
                    gap: 8.0,
                    overlap: 0.0,
                    max_visible: 4,
                    toasts: self.toasts.clone(),
                }),
                card_node("contract-demo.events", self.event_nodes()),
            ],
        ))
    }

    fn layout_primitives_nodes(&self) -> Vec<ContractNode> {
        vec![
            subheading_node("contract-demo.layout.title", "Layout Primitives"),
            muted_node(
                "contract-demo.layout.subtitle",
                "Inset, sized-box, and spacer stay semantic contract nodes instead of exposing raw layout calls."
                    .to_owned(),
            ),
            separator_node("contract-demo.layout.sep"),
            ContractNode::Inset(ContractInset {
                common: common("contract-demo.layout.inset"),
                padding_x: 12,
                padding_y: 10,
                children: vec![muted_node(
                    "contract-demo.layout.inset.copy",
                    "Inset adds host-readable padding around its children.".to_owned(),
                )],
            }),
            ContractNode::SizedBox(ContractSizedBox {
                common: common("contract-demo.layout.sized-box"),
                width: Some(220.0),
                height: None,
                children: vec![button_node(
                    "contract-demo.layout.toast-button",
                    "Trigger Toast",
                    "button.toast",
                    ButtonVariant::Secondary,
                    Some("bell"),
                )],
            }),
            row_node(
                "contract-demo.layout.spacer-row",
                8.0,
                vec![
                    muted_node(
                        "contract-demo.layout.spacer.left",
                        "Leading content".to_owned(),
                    ),
                    ContractNode::Spacer(ContractSpacer {
                        common: common("contract-demo.layout.spacer"),
                        width: None,
                        height: None,
                        flex: true,
                    }),
                    muted_node(
                        "contract-demo.layout.spacer.right",
                        "Trailing content".to_owned(),
                    ),
                ],
            ),
        ]
    }

    fn supported_family_nodes(&self) -> Vec<ContractNode> {
        let mut children = vec![
            subheading_node("contract-demo.registry.title", "Supported Families"),
            muted_node(
                "contract-demo.registry.subtitle",
                format!(
                    "{} families rendered from the same registry that powers schema export and docs.",
                    registry().len()
                ),
            ),
            separator_node("contract-demo.registry.sep"),
        ];

        for family in registry() {
            let events = if family.events.is_empty() {
                "none".to_owned()
            } else {
                family
                    .events
                    .iter()
                    .map(|event| event_kind_name(event.kind).to_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            children.push(ContractNode::Label(ContractLabel {
                common: common(format!("contract-demo.registry.{}", family.id.as_str())),
                text: format!(
                    "{} | {} | events: {}",
                    family.id.as_str(),
                    family.summary,
                    events
                ),
                tone: Some(LabelTone::Muted),
                weight: None,
                size: Some(11.0),
                truncate: false,
            }));
        }

        children
    }

    fn event_nodes(&self) -> Vec<ContractNode> {
        let mut children = vec![
            subheading_node("contract-demo.events.title", "Semantic Event Log"),
            muted_node(
                "contract-demo.events.subtitle",
                "Recent events returned by the renderer this frame boundary.".to_owned(),
            ),
            separator_node("contract-demo.events.sep"),
        ];

        if self.event_log.is_empty() {
            children.push(muted_node(
                "contract-demo.events.empty",
                "Interact with the contract surface to populate the log.".to_owned(),
            ));
        } else {
            for (index, entry) in self.event_log.iter().enumerate() {
                children.push(ContractNode::Label(ContractLabel {
                    common: common(format!("contract-demo.events.item.{index}")),
                    text: entry.clone(),
                    tone: Some(LabelTone::Secondary),
                    weight: None,
                    size: Some(12.0),
                    truncate: false,
                }));
            }
        }

        children
    }

    fn apply_event(&mut self, event: &ContractEvent) {
        self.push_event_log(format_event(event));

        match event.action_id.as_ref().map(ActionId::as_str) {
            Some("field.project_name") => {
                if let Some(EventValue::Text(value)) = &event.value {
                    self.project_name = value.clone();
                }
            }
            Some("field.owner") => {
                if let Some(EventValue::Text(value)) = &event.value {
                    self.owner = value.clone();
                }
            }
            Some("field.rotation") => {
                if let Some(EventValue::Number(value)) = event.value.as_ref() {
                    self.rotation = *value;
                }
            }
            Some("toggle.snap") => {
                if let Some(EventValue::Boolean(value)) = event.value.as_ref() {
                    self.snap_to_grid = *value;
                }
            }
            Some("toggle.autosave") => {
                if let Some(EventValue::Boolean(value)) = event.value.as_ref() {
                    self.auto_save = *value;
                }
            }
            Some("field.status") => {
                self.status = event
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.item_id.clone());
            }
            Some("section.details") => match event.kind {
                EventKind::Opened => self.details_open = true,
                EventKind::Closed => self.details_open = false,
                EventKind::Toggled => {
                    if let Some(EventValue::Boolean(value)) = event.value.as_ref() {
                        self.details_open = *value;
                    }
                }
                _ => {}
            },
            Some("hierarchy.selection") => match event.kind {
                EventKind::Selected => {
                    self.hierarchy_selected = event
                        .metadata
                        .as_ref()
                        .and_then(|metadata| metadata.item_id.clone());
                }
                EventKind::Opened => {
                    if let Some(item_id) = event
                        .metadata
                        .as_ref()
                        .and_then(|metadata| metadata.item_id.clone())
                    {
                        self.open_hierarchy_items.insert(item_id);
                    }
                }
                EventKind::Closed => {
                    if let Some(item_id) = event
                        .metadata
                        .as_ref()
                        .and_then(|metadata| metadata.item_id.clone())
                    {
                        self.open_hierarchy_items.remove(&item_id);
                    }
                }
                _ => {}
            },
            Some("button.open_sidebar") | Some("menu.view.sidebar") => {
                self.sidebar_open = true;
            }
            Some("button.close_sidebar") => {
                self.sidebar_open = false;
            }
            Some("button.open_modal") | Some("menu.file.publish") => {
                self.modal_open = true;
            }
            Some("button.toast") => {
                self.push_toast(
                    "Contract Toast",
                    Some(
                        "Toast items are host-owned contract data and close semantically."
                            .to_owned(),
                    ),
                    ToastIntent::Neutral,
                    3.5,
                );
            }
            Some("modal.confirm") | Some("modal.cancel") => {
                self.modal_open = false;
                if matches!(event.kind, EventKind::Confirmed) {
                    self.push_toast(
                        "Publish Started",
                        Some("The host received a semantic confirm event from the declarative modal."
                            .to_owned()),
                        ToastIntent::Success,
                        4.0,
                    );
                }
            }
            Some("button.save") | Some("menu.file.save") => {
                self.push_toast(
                    "Scene Saved",
                    Some("A success toast was queued from a semantic action id.".to_owned()),
                    ToastIntent::Success,
                    3.5,
                );
            }
            Some("menu.file.autosave") => {
                self.auto_save = !self.auto_save;
            }
            Some("menu.view.details") => {
                self.details_open = !self.details_open;
            }
            Some("command.align_left")
            | Some("command.align_center")
            | Some("command.align_right") => {}
            Some("toolbar.move") | Some("toolbar.rotate") | Some("toolbar.scale") => {}
            Some("tab.overview") => self.active_tab = Some("overview".to_owned()),
            Some("tab.inspect") => self.active_tab = Some("inspect".to_owned()),
            Some("tab.history") => self.active_tab = Some("history".to_owned()),
            _ => match event.kind {
                EventKind::Closed if event.node_id.as_str() == "contract-demo.toasts" => {
                    if let Some(item_id) = event
                        .metadata
                        .as_ref()
                        .and_then(|metadata| metadata.item_id.as_deref())
                    {
                        self.dismiss_toast(item_id);
                    }
                }
                EventKind::Closed if event.node_id.as_str() == "contract-demo.sidebar" => {
                    self.sidebar_open = false;
                }
                EventKind::Closed if event.node_id.as_str() == "contract-demo.dialogue" => {
                    self.modal_open = false;
                }
                _ => {}
            },
        }
    }

    fn push_event_log(&mut self, entry: String) {
        self.event_log.insert(0, entry);
        self.event_log.truncate(EVENT_LOG_LIMIT);
    }

    fn push_toast(
        &mut self,
        title: &str,
        description: Option<String>,
        intent: ToastIntent,
        duration_secs: f32,
    ) {
        let item_id = format!("toast.generated.{}", self.next_toast_id);
        self.next_toast_id = self.next_toast_id.saturating_add(1);
        self.toasts.push(ContractToastItem {
            item_id,
            title: title.to_owned(),
            description,
            intent,
            duration_secs,
            action_id: None,
        });
    }

    fn dismiss_toast(&mut self, item_id: &str) {
        self.toasts.retain(|toast| toast.item_id != item_id);
    }
}

fn common(id: impl Into<String>) -> ContractCommon {
    ContractCommon::new(id.into())
}

fn action(value: &str) -> ActionId {
    value.into()
}

fn column_node(id: &str, gap: f32, children: Vec<ContractNode>) -> ContractNode {
    ContractNode::Column(ContractColumn {
        common: common(id),
        gap,
        justify: Default::default(),
        align: Default::default(),
        children,
    })
}

fn row_node(id: &str, gap: f32, children: Vec<ContractNode>) -> ContractNode {
    ContractNode::Row(ContractRow {
        common: common(id),
        gap,
        justify: Default::default(),
        align: crate::contract::ContractAlign::Center,
        children,
    })
}

fn card_node(id: &str, children: Vec<ContractNode>) -> ContractNode {
    ContractNode::Card(ContractCard {
        common: common(id),
        padding_x: 14,
        padding_y: 14,
        children,
    })
}

fn heading_node(id: &str, text: impl Into<String>) -> ContractNode {
    ContractNode::Label(ContractLabel {
        common: common(id),
        text: text.into(),
        tone: Some(LabelTone::Primary),
        weight: Some(LabelWeight::Bold),
        size: Some(16.0),
        truncate: false,
    })
}

fn subheading_node(id: &str, text: impl Into<String>) -> ContractNode {
    ContractNode::Label(ContractLabel {
        common: common(id),
        text: text.into(),
        tone: Some(LabelTone::Primary),
        weight: Some(LabelWeight::Semibold),
        size: Some(13.0),
        truncate: false,
    })
}

fn muted_node(id: &str, text: impl Into<String>) -> ContractNode {
    ContractNode::Label(ContractLabel {
        common: common(id),
        text: text.into(),
        tone: Some(LabelTone::Muted),
        weight: None,
        size: Some(12.0),
        truncate: false,
    })
}

fn separator_node(id: &str) -> ContractNode {
    ContractNode::Separator(crate::contract::ContractSeparator { common: common(id) })
}

fn button_node(
    id: &str,
    label: &str,
    action_id: &str,
    variant: ButtonVariant,
    icon: Option<&str>,
) -> ContractNode {
    ContractNode::Button(ContractButton {
        common: common(id),
        label: label.to_owned(),
        action_id: Some(action(action_id)),
        variant: Some(variant),
        size: None,
        leading_icon: icon.map(str::to_owned),
        trailing_text: None,
        trailing_icon: None,
        icon_only: false,
        selected: false,
    })
}

fn icon_button(id: &str, icon: &str, action_id: &str) -> ContractNode {
    ContractNode::Button(ContractButton {
        common: common(id),
        label: String::new(),
        action_id: Some(action(action_id)),
        variant: Some(ButtonVariant::Ghost),
        size: None,
        leading_icon: Some(icon.to_owned()),
        trailing_text: None,
        trailing_icon: None,
        icon_only: true,
        selected: false,
    })
}

fn action_item(item_id: &str, label: &str, action_id: &str) -> crate::contract::ContractActionItem {
    crate::contract::ContractActionItem {
        item_id: item_id.to_owned(),
        label: label.to_owned(),
        action_id: Some(action(action_id)),
    }
}

fn choice_item(item_id: &str, label: &str) -> ContractChoiceItem {
    ContractChoiceItem {
        item_id: item_id.to_owned(),
        label: label.to_owned(),
        action_id: None,
    }
}

fn tab_item(item_id: &str, label: &str, icon: &str) -> ContractTabItem {
    ContractTabItem {
        item_id: item_id.to_owned(),
        label: label.to_owned(),
        action_id: Some(action(match item_id {
            "overview" => "tab.overview",
            "inspect" => "tab.inspect",
            _ => "tab.history",
        })),
        icon: Some(icon.to_owned()),
        icon_only: false,
    }
}

fn demo_hierarchy(app: &ContractDemoApp) -> Vec<ContractHierarchyItem> {
    fn item(
        app: &ContractDemoApp,
        item_id: &str,
        label: &str,
        kind: HierarchyItemKind,
        children: Vec<ContractHierarchyItem>,
    ) -> ContractHierarchyItem {
        ContractHierarchyItem {
            item_id: item_id.to_owned(),
            label: label.to_owned(),
            kind,
            open: app.open_hierarchy_items.contains(item_id),
            locked: false,
            action_id: None,
            children,
        }
    }

    vec![item(
        app,
        "scene-root",
        "Scene Root",
        HierarchyItemKind::Folder,
        vec![
            item(
                app,
                "characters",
                "Characters",
                HierarchyItemKind::Group,
                vec![
                    item(app, "hero", "Hero", HierarchyItemKind::Player, vec![]),
                    item(app, "rival", "Rival", HierarchyItemKind::Player, vec![]),
                ],
            ),
            item(
                app,
                "props",
                "Props",
                HierarchyItemKind::Group,
                vec![
                    item(app, "bridge", "Bridge", HierarchyItemKind::Frame, vec![]),
                    item(app, "banner", "Banner", HierarchyItemKind::Clothing, vec![]),
                ],
            ),
        ],
    )]
}

fn format_event(event: &ContractEvent) -> String {
    let action = event
        .action_id
        .as_ref()
        .map(ActionId::as_str)
        .unwrap_or("no-action");
    let item = event
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.item_id.as_deref())
        .unwrap_or("-");
    let value = match event.value.as_ref() {
        Some(EventValue::Boolean(value)) => value.to_string(),
        Some(EventValue::Number(value)) => format!("{value:.1}"),
        Some(EventValue::Text(value)) => value.clone(),
        Some(EventValue::ItemId(value)) => value.clone(),
        None => "-".to_owned(),
    };

    format!(
        "{:?} | node={} | action={} | item={} | value={}",
        event.kind, event.node_id, action, item, value
    )
}

fn event_kind_name(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Clicked => "clicked",
        EventKind::Changed => "changed",
        EventKind::Submitted => "submitted",
        EventKind::Selected => "selected",
        EventKind::Toggled => "toggled",
        EventKind::Confirmed => "confirmed",
        EventKind::Cancelled => "cancelled",
        EventKind::Opened => "opened",
        EventKind::Closed => "closed",
        EventKind::CommandInvoked => "command_invoked",
    }
}

#[cfg(test)]
mod tests {
    use super::ContractDemoApp;
    use crate::contract::{registry, ContractNode};
    use std::collections::BTreeSet;

    #[test]
    fn demo_tree_covers_every_registered_family() {
        let app = ContractDemoApp::default();
        let tree = app.build_tree();
        let mut families = BTreeSet::new();
        collect_families(&tree.root, &mut families);

        let expected = registry()
            .iter()
            .map(|family| family.id.as_str().to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(families, expected);
    }

    fn collect_families(node: &ContractNode, families: &mut BTreeSet<String>) {
        families.insert(node.family_id().as_str().to_owned());

        match node {
            ContractNode::Row(props) => collect_children(&props.children, families),
            ContractNode::Column(props) => collect_children(&props.children, families),
            ContractNode::Inset(props) => collect_children(&props.children, families),
            ContractNode::SizedBox(props) => collect_children(&props.children, families),
            ContractNode::Card(props) => collect_children(&props.children, families),
            ContractNode::Sidebar(props) => collect_children(&props.children, families),
            ContractNode::Toolbar(props) => collect_children(&props.children, families),
            ContractNode::Collapsible(props) => collect_children(&props.children, families),
            ContractNode::DialogueModal(props) => collect_children(&props.children, families),
            ContractNode::Spacer(_)
            | ContractNode::MenuBar(_)
            | ContractNode::Tabs(_)
            | ContractNode::Label(_)
            | ContractNode::Button(_)
            | ContractNode::ButtonGroup(_)
            | ContractNode::Input(_)
            | ContractNode::NumberInput(_)
            | ContractNode::Checkbox(_)
            | ContractNode::Switch(_)
            | ContractNode::Select(_)
            | ContractNode::Field(_)
            | ContractNode::Separator(_)
            | ContractNode::Hierarchy(_)
            | ContractNode::Spinner(_)
            | ContractNode::Progress(_)
            | ContractNode::ToastViewport(_) => {}
        }
    }

    fn collect_children(children: &[ContractNode], families: &mut BTreeSet<String>) {
        for child in children {
            collect_families(child, families);
        }
    }
}
