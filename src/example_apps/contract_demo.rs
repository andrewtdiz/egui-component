use crate::components::{
    AudioPlaybackState, ButtonVariant, Card, ComponentUiExt, ControlSize, DialogueIntent,
    DragBoardRegion, FileTreeItemKind, HierarchyItemKind, ImageTilePlaybackState, ImageTileSize,
    Label, LabelTone, LabelWeight, PopoverAlign, PopoverSide, SelectVariant, ToastIntent,
    ToastPlacement, TooltipPlacement,
};
use crate::contract::*;
use crate::internal_taffy::{
    taffy,
    taffy::prelude::{auto, length, percent},
    tui, TuiBuilderLogic,
};
use crate::theme::{self, BaseColor, ThemeMode, ThemeSpec};
use egui::{CentralPanel, ScrollArea, Ui};
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
            event_log: vec!["Host bootstrapped the optional contract demo.".to_owned()],
        }
    }
}

pub fn install_context(ctx: &egui::Context) {
    theme::install(ctx, ThemeSpec::preset(BaseColor::Slate), ThemeMode::System);
}

pub fn update(app: &mut ContractDemoApp, ctx: &egui::Context) {
    let tree = app.build_tree();
    let mut events = Vec::new();

    #[allow(
        deprecated,
        reason = "eframe App::update still renders top-level panels from Context"
    )]
    {
        CentralPanel::default().show(ctx, |ui| {
            render_demo_shell(app, ui, &tree, &mut events);
        });
    }

    if !events.is_empty() {
        for event in &events {
            app.apply_event(event);
        }
        ctx.request_repaint();
    }
}

fn render_demo_shell(
    app: &ContractDemoApp,
    ui: &mut Ui,
    tree: &ContractTree,
    events: &mut Vec<ContractEvent>,
) {
    let shell_height = ui.available_height().max(1.0);
    let body_height = (shell_height - 132.0).max(420.0);

    tui(ui, ui.auto_id_with("contract_demo_shell"))
        .reserve_available_space()
        .style(taffy::Style {
            flex_direction: taffy::FlexDirection::Column,
            align_items: Some(taffy::AlignItems::Stretch),
            gap: length(16.0),
            size: taffy::Size {
                width: percent(1.0),
                height: length(shell_height),
            },
            ..Default::default()
        })
        .show(|tui| {
            tui.id("header")
                .style(taffy::Style {
                    size: taffy::Size {
                        width: percent(1.0),
                        height: auto(),
                    },
                    ..Default::default()
                })
                .ui(|ui| render_demo_header(app, ui));

            tui.id("body")
                .style(taffy::Style {
                    flex_direction: taffy::FlexDirection::Row,
                    align_items: Some(taffy::AlignItems::Stretch),
                    gap: length(16.0),
                    size: taffy::Size {
                        width: percent(1.0),
                        height: length(body_height),
                    },
                    ..Default::default()
                })
                .add(|tui| {
                    tui.id("main")
                        .style(taffy::Style {
                            flex_grow: 1.0,
                            flex_basis: length(0.0),
                            size: taffy::Size {
                                width: percent(1.0),
                                height: length(body_height),
                            },
                            min_size: taffy::Size {
                                width: length(0.0),
                                height: length(body_height),
                            },
                            ..Default::default()
                        })
                        .ui(|ui| render_demo_main(ui, tree, events));

                    tui.id("sidebar")
                        .style(taffy::Style {
                            min_size: taffy::Size {
                                width: length(280.0),
                                height: length(body_height),
                            },
                            size: taffy::Size {
                                width: length(320.0),
                                height: length(body_height),
                            },
                            ..Default::default()
                        })
                        .ui(|ui| render_demo_sidebar(app, ui));
                });
        });
}

fn render_demo_header(app: &ContractDemoApp, ui: &mut Ui) {
    ui.set_width(ui.available_width().max(1.0));
    ui.components().card(Card::new().padding(18, 16), |ui| {
        ui.vertical(|ui| {
            let _ = ui.components().label(
                Label::new("Optional Contract Layer")
                    .weight(LabelWeight::Bold)
                    .size(20.0),
            );
            ui.add_space(4.0);
            let _ = ui.components().label(
                Label::new(
                    "A host-authored `ContractTree` rendered by Rust. The authored JSX runtime path lives in `runtime-jsx-host`.",
                )
                    .tone(LabelTone::Muted),
            );

            ui.add_space(14.0);
            ui.horizontal(|ui| {
                render_state_row(ui, "Project", &app.project_name);
                ui.add_space(18.0);
                render_state_row(ui, "Owner", &app.owner);
                ui.add_space(18.0);
                render_state_row(
                    ui,
                    "Sidebar",
                    if app.sidebar_open { "open" } else { "closed" },
                );
            });
        });
    });
}

fn render_demo_main(ui: &mut Ui, tree: &ContractTree, events: &mut Vec<ContractEvent>) {
    ui.set_width(ui.available_width().max(1.0));
    ui.components().card(Card::new().padding(16, 16), |ui| {
        ui.vertical(|ui| {
            let _ = ui.components().label(
                Label::new("Optional Contract Surface")
                    .weight(LabelWeight::Semibold)
                    .size(14.0),
            );
            let _ = ui.components().label(
                Label::new(
                    "This alternate contract-mode renderer stays host-owned and returns semantic events only.",
                )
                    .tone(LabelTone::Muted),
            );

            ui.add_space(12.0);
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    *events = render_tree(ui, tree);
                });
        });
    });
}

fn render_demo_sidebar(app: &ContractDemoApp, ui: &mut Ui) {
    ui.set_width(ui.available_width().max(1.0));
    ui.components().card(Card::new().padding(16, 16), |ui| {
        ui.vertical(|ui| {
            let _ = ui.components().label(
                Label::new("Contract Mode Summary")
                    .weight(LabelWeight::Semibold)
                    .size(14.0),
            );
            let _ = ui.components().label(
                Label::new(
                    "Live host state and recent events for the optional contract-layer example.",
                )
                .tone(LabelTone::Muted),
            );

            ui.add_space(12.0);
            render_state_row(ui, "Project", &app.project_name);
            render_state_row(ui, "Owner", &app.owner);
            render_state_row(
                ui,
                "Active tab",
                app.active_tab.as_deref().unwrap_or("overview"),
            );
            render_state_row(
                ui,
                "Selected item",
                app.hierarchy_selected.as_deref().unwrap_or("none"),
            );
            render_state_row(ui, "Toasts", app.toasts.len().to_string());
            render_state_row(ui, "Families", registry().len().to_string());

            ui.add_space(12.0);
            let _ = ui.components().separator();
            ui.add_space(12.0);
            let _ = ui.components().label(
                Label::new("Recent events")
                    .weight(LabelWeight::Semibold)
                    .size(13.0),
            );

            ui.add_space(6.0);
            if app.event_log.is_empty() {
                let _ = ui.components().label(
                    Label::new("No events yet.")
                        .tone(LabelTone::Muted)
                        .size(12.0),
                );
            } else {
                for entry in app.event_log.iter().take(6) {
                    let _ = ui.components().label(
                        Label::new(entry)
                            .tone(LabelTone::Secondary)
                            .size(12.0)
                            .truncate(),
                    );
                }
            }
        });
    });
}

fn render_state_row(ui: &mut Ui, label: &str, value: impl AsRef<str>) {
    let value = value.as_ref();
    ui.horizontal(|ui| {
        let _ = ui
            .components()
            .label(Label::new(label).tone(LabelTone::Muted).size(12.0));
        ui.add_space(8.0);
        let _ = ui
            .components()
            .label(Label::new(value).weight(LabelWeight::Semibold).size(12.0));
    });
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
                        heading_node("contract-demo.heading", "Optional Contract Layer"),
                        muted_node(
                            "contract-demo.subtitle",
                            format!(
                                "Host-owned state, one contract tree in, semantic events out. JSX runtime path: runtime-jsx-host. Active tab: {}",
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
                        card_node(
                            "contract-demo.layout-primitives",
                            self.layout_primitives_nodes(),
                        ),
                        card_node(
                            "contract-demo.component-parity",
                            self.component_parity_nodes(),
                        ),
                        card_node(
                            "contract-demo.registry-card",
                            self.supported_family_nodes(),
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
                    "{} families in the shared contract registry.",
                    registry().len()
                ),
            ),
            separator_node("contract-demo.registry.sep"),
        ];

        for family in registry() {
            children.push(ContractNode::Label(ContractLabel {
                common: common(format!("contract-demo.registry.{}", family.id.as_str())),
                text: format!("{} | {}", family.id.as_str(), family.summary),
                tone: Some(LabelTone::Muted),
                weight: None,
                size: Some(11.0),
                truncate: false,
            }));
        }

        children
    }

    fn component_parity_nodes(&self) -> Vec<ContractNode> {
        vec![
            subheading_node("contract-demo.parity.title", "Component Library Parity"),
            muted_node(
                "contract-demo.parity.subtitle",
                "Reusable showcase widgets are available through contract nodes so JSX can author the same surfaces."
                    .to_owned(),
            ),
            separator_node("contract-demo.parity.sep"),
            row_node(
                "contract-demo.parity.media-row",
                12.0,
                vec![
                    ContractNode::Color(ContractColor {
                        common: common("contract-demo.parity.color"),
                        fill: "#22334d".into(),
                        size: 22.0,
                        stroke: Some(ContractStroke {
                            width: 1.0,
                            color: "#94a3b8".into(),
                        }),
                        corner_radius: Some(6),
                    }),
                    ContractNode::Icon(ContractIcon {
                        common: common("contract-demo.parity.icon"),
                        name: "sparkles".to_owned(),
                        size: 18.0,
                        tint: Some("#f1a84e".into()),
                    }),
                    ContractNode::Twemoji(ContractTwemoji {
                        common: common("contract-demo.parity.twemoji"),
                        emoji: "🔥".to_owned(),
                        size: 26.0,
                    }),
                    ContractNode::Kbd(ContractKbd {
                        common: common("contract-demo.parity.kbd"),
                        text: "Ctrl".to_owned(),
                        min_width: 30.0,
                        height: 22.0,
                    }),
                ],
            ),
            ContractNode::Image(ContractImage {
                common: common("contract-demo.parity.image"),
                source: "builtin:showcase-image".to_owned(),
                width: 144.0,
                height: 96.0,
                corner_radius: Some(8),
            }),
            ContractNode::Skeleton(ContractSkeleton {
                common: common("contract-demo.parity.skeleton"),
                width: 220.0,
                height: 14.0,
                circle: false,
                corner_radius: Some(7),
                animated: true,
            }),
            ContractNode::Slider(ContractSlider {
                common: common("contract-demo.parity.slider"),
                value: self.rotation,
                action_id: Some(action("field.rotation")),
                width: 220.0,
                min: 0.0,
                max: 360.0,
            }),
            ContractNode::Radio(ContractRadio {
                common: common("contract-demo.parity.radio"),
                value: self.snap_to_grid,
                action_id: Some(action("toggle.snap")),
                label: Some("Use publish channel".to_owned()),
                description: Some("Standalone radio authoring through JSX.".to_owned()),
            }),
            ContractNode::RadioGroup(ContractRadioGroup {
                common: common("contract-demo.parity.radio-group"),
                action_id: Some(action("parity.radio_group")),
                selected_item_id: Some("team".to_owned()),
                gap: 6.0,
                items: vec![
                    radio_item("starter", "Starter", "Basic surfaces"),
                    radio_item("team", "Team", "Shared component previews"),
                    radio_item("enterprise", "Enterprise", "Extended controls"),
                ],
            }),
            ContractNode::Combobox(ContractCombobox {
                common: common("contract-demo.parity.combobox"),
                action_id: Some(action("parity.combobox")),
                query: String::new(),
                selected_item_ids: vec!["material-glass".to_owned()],
                width: 280.0,
                max_height: 140.0,
                placeholder: Some("Select assets".to_owned()),
                filter_placeholder: Some("Filter assets".to_owned()),
                searchable: true,
                items: vec![
                    choice_item("material-glass", "Material Glass"),
                    choice_item("material-metal", "Material Metal"),
                    choice_item("sprite-atlas", "Sprite Atlas"),
                ],
            }),
            ContractNode::EmojiSelector(ContractEmojiSelector {
                common: common("contract-demo.parity.emoji-selector"),
                value: "🙂".to_owned(),
                action_id: Some(action("parity.emoji")),
                popup_width: 320.0,
                popup_max_height: 360.0,
                placeholder: Some("Pick emoji".to_owned()),
                trigger_variant: Some(ButtonVariant::Secondary),
            }),
            ContractNode::Pagination(ContractPagination {
                common: common("contract-demo.parity.pagination"),
                action_id: Some(action("parity.pagination")),
                current_page: 2,
                page_count: 8,
                sibling_count: 1,
            }),
            ContractNode::Tooltip(ContractTooltip {
                common: common("contract-demo.parity.tooltip"),
                trigger_label: "Hover this trigger".to_owned(),
                text: "Tooltip content example".to_owned(),
                width: 220.0,
                delay_ms: 0,
                placement: TooltipPlacement::Top,
            }),
            ContractNode::Popover(ContractPopover {
                common: common("contract-demo.parity.popover"),
                action_id: Some(action("parity.popover")),
                open: false,
                trigger_label: Some("Open Popover".to_owned()),
                side: PopoverSide::Bottom,
                align: PopoverAlign::Center,
                side_offset: 4.0,
                width: Some(280.0),
                padding_x: 12,
                padding_y: 12,
                children: vec![muted_node(
                    "contract-demo.parity.popover.copy",
                    "Interactive popovers can host contract children.".to_owned(),
                )],
            }),
            row_node(
                "contract-demo.parity.menu-row",
                8.0,
                vec![
                    ContractNode::DropdownMenu(ContractDropdownMenu {
                        common: common("contract-demo.parity.dropdown"),
                        action_id: Some(action("parity.dropdown")),
                        trigger_label: "Open".to_owned(),
                        width: 220.0,
                        trigger_variant: Some(ButtonVariant::Secondary),
                        entries: parity_menu_entries(),
                    }),
                    ContractNode::OpenWith(ContractOpenWith {
                        common: common("contract-demo.parity.open-with"),
                        action_id: Some(action("parity.open_with")),
                        width: 240.0,
                        placeholder: Some("Open With".to_owned()),
                        size: Some(ControlSize::Md),
                        trigger_variant: Some(ButtonVariant::Secondary),
                        entries: vec![ContractMenuEntry::Action(ContractMenuAction {
                            item_id: "codex".to_owned(),
                            label: "Codex".to_owned(),
                            action_id: Some(action("parity.open_with.codex")),
                            leading_icon: Some("codex".to_owned()),
                            shortcut: None,
                        })],
                        selected_item_id: Some("codex".to_owned()),
                    }),
                ],
            ),
            ContractNode::ContextMenu(ContractContextMenu {
                common: common("contract-demo.parity.context-menu"),
                action_id: Some(action("parity.context_menu")),
                width: 220.0,
                region_width: 360.0,
                region_height: 128.0,
                padding_x: 14,
                padding_y: 14,
                entries: parity_menu_entries(),
                children: vec![muted_node(
                    "contract-demo.parity.context-menu.copy",
                    "Right-click this region for scene actions.".to_owned(),
                )],
            }),
            ContractNode::CollabCursor(ContractCollabCursor {
                common: common("contract-demo.parity.collab-cursor"),
                name: "Lisa Chen".to_owned(),
                x: 96.0,
                y: 34.0,
                color: Some("#39bdf8".into()),
                size: 32.0,
            }),
            ContractNode::IconToolbar(ContractIconToolbar {
                common: common("contract-demo.parity.icon-toolbar"),
                action_id: Some(action("parity.icon_toolbar")),
                selected_item_id: Some("move".to_owned()),
                size: Some(ControlSize::Md),
                gap: 4.0,
                icon_size: Some(16.0),
                items: vec![
                    icon_toolbar_item("select", "mouse-pointer-2", "Select"),
                    icon_toolbar_item("move", "move", "Move"),
                    icon_toolbar_item("delete", "trash", "Delete"),
                ],
            }),
            ContractNode::FileTree(ContractFileTree {
                common: common("contract-demo.parity.file-tree"),
                action_id: Some(action("parity.file_tree")),
                selected_item_id: Some("main-script".to_owned()),
                width: 260.0,
                row_height: 20.0,
                indent_width: 14.0,
                items: parity_file_tree_items(),
            }),
            ContractNode::DragBoard(ContractDragBoard {
                common: common("contract-demo.parity.drag-board"),
                action_id: Some(action("parity.drag_board")),
                left_title: "Backlog".to_owned(),
                right_title: "Done".to_owned(),
                height: 220.0,
                items: vec![
                    drag_board_item("spacing", "Polish header spacing", DragBoardRegion::Left),
                    drag_board_item("sidebar", "Tune sidebar spacing", DragBoardRegion::Left),
                    drag_board_item("runtime", "Ship JSX runtime", DragBoardRegion::Right),
                ],
            }),
            ContractNode::AudioPlayback(ContractAudioPlayback {
                common: common("contract-demo.parity.audio"),
                action_id: Some(action("parity.audio")),
                playback_state: AudioPlaybackState::Paused,
                duration_seconds: Some(8.0),
                children: vec![icon_button(
                    "contract-demo.parity.audio.download",
                    "download",
                    "parity.audio.download",
                )],
            }),
            ContractNode::ImageTile(ContractImageTile {
                common: common("contract-demo.parity.image-tile"),
                source: "builtin:showcase-image".to_owned(),
                action_id: Some(action("parity.image_tile")),
                play_pause_action_id: Some(action("parity.image_tile.play")),
                size: Some(ImageTileSize::Md),
                image_width: None,
                image_height: None,
                image_frame: true,
                selected: true,
                playback_state: Some(ImageTilePlaybackState::Paused),
                children: vec![muted_node(
                    "contract-demo.parity.image-tile.label",
                    "Ambient Preview".to_owned(),
                )],
            }),
            ContractNode::Command(ContractCommand {
                common: common("contract-demo.parity.command"),
                action_id: Some(action("parity.command")),
                query: String::new(),
                width: 360.0,
                max_height: 180.0,
                placeholder: Some("Execute a command...".to_owned()),
                preview: true,
                preview_height: 220.0,
                items: vec![
                    command_item("scene", "open scene search", "Ctrl+P"),
                    command_item("scene", "save active scene", "Ctrl+S"),
                    command_item("tools", "build nav mesh", "Ctrl+B"),
                ],
            }),
        ]
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

fn radio_item(item_id: &str, label: &str, description: &str) -> ContractRadioItem {
    ContractRadioItem {
        item_id: item_id.to_owned(),
        label: label.to_owned(),
        description: Some(description.to_owned()),
        action_id: None,
    }
}

fn parity_menu_entries() -> Vec<ContractMenuEntry> {
    vec![
        ContractMenuEntry::Action(ContractMenuAction {
            item_id: "profile".to_owned(),
            label: "Profile".to_owned(),
            action_id: Some(action("parity.menu.profile")),
            leading_icon: Some("user".to_owned()),
            shortcut: Some("Shift+Cmd+P".to_owned()),
        }),
        ContractMenuEntry::Action(ContractMenuAction {
            item_id: "settings".to_owned(),
            label: "Settings".to_owned(),
            action_id: Some(action("parity.menu.settings")),
            leading_icon: Some("settings".to_owned()),
            shortcut: Some("Cmd+S".to_owned()),
        }),
        ContractMenuEntry::Separator,
        ContractMenuEntry::Submenu(ContractMenuSubmenu {
            label: "Invite users".to_owned(),
            leading_icon: Some("users".to_owned()),
            entries: vec![
                ContractMenuEntry::Action(ContractMenuAction {
                    item_id: "email".to_owned(),
                    label: "Email".to_owned(),
                    action_id: Some(action("parity.menu.email")),
                    leading_icon: Some("mail".to_owned()),
                    shortcut: None,
                }),
                ContractMenuEntry::Action(ContractMenuAction {
                    item_id: "message".to_owned(),
                    label: "Message".to_owned(),
                    action_id: Some(action("parity.menu.message")),
                    leading_icon: Some("message-square".to_owned()),
                    shortcut: None,
                }),
            ],
        }),
    ]
}

fn icon_toolbar_item(item_id: &str, icon: &str, tooltip: &str) -> ContractIconToolbarItem {
    ContractIconToolbarItem {
        item_id: item_id.to_owned(),
        icon: icon.to_owned(),
        tooltip: Some(tooltip.to_owned()),
        badge_fill: None,
        action_id: None,
    }
}

fn parity_file_tree_items() -> Vec<ContractFileTreeItem> {
    vec![ContractFileTreeItem {
        item_id: "workspace".to_owned(),
        label: "Workspace".to_owned(),
        kind: FileTreeItemKind::Project,
        open: true,
        action_id: None,
        children: vec![
            ContractFileTreeItem {
                item_id: "scripts".to_owned(),
                label: "scripts".to_owned(),
                kind: FileTreeItemKind::Folder,
                open: true,
                action_id: None,
                children: vec![ContractFileTreeItem {
                    item_id: "main-script".to_owned(),
                    label: "main.jsx".to_owned(),
                    kind: FileTreeItemKind::Script,
                    open: true,
                    action_id: None,
                    children: Vec::new(),
                }],
            },
            ContractFileTreeItem {
                item_id: "readme".to_owned(),
                label: "README.md".to_owned(),
                kind: FileTreeItemKind::Markdown,
                open: true,
                action_id: None,
                children: Vec::new(),
            },
        ],
    }]
}

fn drag_board_item(item_id: &str, title: &str, region: DragBoardRegion) -> ContractDragBoardItem {
    ContractDragBoardItem {
        item_id: item_id.to_owned(),
        title: title.to_owned(),
        description: Some("Contract-authored board card".to_owned()),
        region,
        action_id: None,
    }
}

fn command_item(group: &str, label: &str, shortcut: &str) -> ContractCommandItem {
    ContractCommandItem {
        item_id: format!("{group}.{label}"),
        group: group.to_owned(),
        label: label.to_owned(),
        shortcut: Some(shortcut.to_owned()),
        action_id: None,
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
        Some(EventValue::ItemIds(values)) => values.join(","),
        Some(EventValue::ItemMove(value)) => {
            format!("{}:{}->{}", value.item_id, value.from, value.to)
        }
        None => "-".to_owned(),
    };

    format!(
        "{:?} | node={} | action={} | item={} | value={}",
        event.kind, event.node_id, action, item, value
    )
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
            ContractNode::Popover(props) => collect_children(&props.children, families),
            ContractNode::ContextMenu(props) => collect_children(&props.children, families),
            ContractNode::AudioPlayback(props) => collect_children(&props.children, families),
            ContractNode::ImageTile(props) => collect_children(&props.children, families),
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
            | ContractNode::ToastViewport(_)
            | ContractNode::Color(_)
            | ContractNode::Icon(_)
            | ContractNode::Image(_)
            | ContractNode::Twemoji(_)
            | ContractNode::Kbd(_)
            | ContractNode::Skeleton(_)
            | ContractNode::Slider(_)
            | ContractNode::Radio(_)
            | ContractNode::RadioGroup(_)
            | ContractNode::Combobox(_)
            | ContractNode::EmojiSelector(_)
            | ContractNode::Pagination(_)
            | ContractNode::Tooltip(_)
            | ContractNode::DropdownMenu(_)
            | ContractNode::OpenWith(_)
            | ContractNode::CollabCursor(_)
            | ContractNode::IconToolbar(_)
            | ContractNode::FileTree(_)
            | ContractNode::DragBoard(_)
            | ContractNode::Command(_) => {}
        }
    }

    fn collect_children(children: &[ContractNode], families: &mut BTreeSet<String>) {
        for child in children {
            collect_families(child, families);
        }
    }
}
