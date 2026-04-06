use super::model::{ContractFamilyId, CONTRACT_MODEL_VERSION};
use crate::contract::EventKind;

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractChildPolicy {
    None,
    Children,
    Body,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractPropTypeKind {
    String,
    Boolean,
    Number,
    Enum,
    Object,
    ObjectList,
    Node,
    NodeList,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize)]
pub struct ContractPropSpec {
    pub name: &'static str,
    pub kind: ContractPropTypeKind,
    pub type_name: Option<&'static str>,
    pub required: bool,
    pub summary: &'static str,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize)]
pub struct ContractVariantSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub summary: &'static str,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize)]
pub struct ContractVariantRef {
    pub prop: &'static str,
    pub type_name: &'static str,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize)]
pub struct ContractEventSpec {
    pub kind: EventKind,
    pub summary: &'static str,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct ContractFamilySpec {
    pub id: ContractFamilyId,
    pub display_name: &'static str,
    pub summary: &'static str,
    pub includes_common_props: bool,
    pub child_policy: ContractChildPolicy,
    pub props: &'static [ContractPropSpec],
    pub variants: &'static [ContractVariantRef],
    pub events: &'static [ContractEventSpec],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractSharedTypeKind {
    Enum,
    Object,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct ContractSharedTypeSpec {
    pub name: &'static str,
    pub display_name: &'static str,
    pub kind: ContractSharedTypeKind,
    pub summary: &'static str,
    pub fields: &'static [ContractPropSpec],
    pub variants: &'static [ContractVariantSpec],
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct ContractSchema {
    pub version: u32,
    pub families: &'static [ContractFamilySpec],
    pub shared_types: &'static [ContractSharedTypeSpec],
}

const EMPTY_PROPS: [ContractPropSpec; 0] = [];
const EMPTY_VARIANTS: [ContractVariantSpec; 0] = [];
const EMPTY_VARIANT_REFS: [ContractVariantRef; 0] = [];
const EMPTY_EVENTS: [ContractEventSpec; 0] = [];

const NODE_COMMON_FIELDS: [ContractPropSpec; 3] = [
    ContractPropSpec {
        name: "node_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Stable host-owned node identifier.",
    },
    ContractPropSpec {
        name: "visible",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: false,
        summary: "Whether the node renders at all. Defaults to true.",
    },
    ContractPropSpec {
        name: "enabled",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: false,
        summary: "Whether interaction is enabled. Defaults to true.",
    },
];

const ACTION_ITEM_FIELDS: [ContractPropSpec; 3] = [
    ContractPropSpec {
        name: "item_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Stable item identifier.",
    },
    ContractPropSpec {
        name: "label",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Visible item label.",
    },
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional item-level action id.",
    },
];

const CHOICE_ITEM_FIELDS: [ContractPropSpec; 3] = ACTION_ITEM_FIELDS;

const TAB_ITEM_FIELDS: [ContractPropSpec; 5] = [
    ContractPropSpec {
        name: "item_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Stable tab identifier.",
    },
    ContractPropSpec {
        name: "label",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Visible tab label.",
    },
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional item-level action id.",
    },
    ContractPropSpec {
        name: "icon",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional icon name.",
    },
    ContractPropSpec {
        name: "icon_only",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: false,
        summary: "Whether the tab should render without text.",
    },
];

const MENU_ACTION_FIELDS: [ContractPropSpec; 5] = [
    ContractPropSpec {
        name: "item_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Stable menu action identifier.",
    },
    ContractPropSpec {
        name: "label",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Visible action label.",
    },
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional action id emitted back to the host.",
    },
    ContractPropSpec {
        name: "leading_icon",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional leading icon.",
    },
    ContractPropSpec {
        name: "shortcut",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional shortcut label.",
    },
];

const MENU_ENTRY_FIELDS: [ContractPropSpec; 2] = [
    ContractPropSpec {
        name: "kind",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("menu_entry_kind"),
        required: true,
        summary: "Whether the entry is an action or separator.",
    },
    ContractPropSpec {
        name: "action",
        kind: ContractPropTypeKind::Object,
        type_name: Some("menu_action"),
        required: false,
        summary: "Action payload when kind is action.",
    },
];

const MENU_FIELDS: [ContractPropSpec; 4] = [
    ContractPropSpec {
        name: "menu_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Stable top-level menu identifier.",
    },
    ContractPropSpec {
        name: "label",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Visible top-level menu label.",
    },
    ContractPropSpec {
        name: "width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Popup width for the menu surface.",
    },
    ContractPropSpec {
        name: "entries",
        kind: ContractPropTypeKind::ObjectList,
        type_name: Some("menu_entry"),
        required: true,
        summary: "Action and separator rows for the menu.",
    },
];

const HIERARCHY_ITEM_FIELDS: [ContractPropSpec; 7] = [
    ContractPropSpec {
        name: "item_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Stable hierarchy item identifier.",
    },
    ContractPropSpec {
        name: "label",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Visible hierarchy label.",
    },
    ContractPropSpec {
        name: "kind",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("hierarchy_item_kind"),
        required: true,
        summary: "Semantic item kind.",
    },
    ContractPropSpec {
        name: "open",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: false,
        summary: "Whether children are expanded.",
    },
    ContractPropSpec {
        name: "locked",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: false,
        summary: "Whether the row renders as locked.",
    },
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional item-level action id.",
    },
    ContractPropSpec {
        name: "children",
        kind: ContractPropTypeKind::ObjectList,
        type_name: Some("hierarchy_item"),
        required: false,
        summary: "Nested hierarchy children.",
    },
];

const TOAST_ITEM_FIELDS: [ContractPropSpec; 6] = [
    ContractPropSpec {
        name: "item_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Stable toast item identifier.",
    },
    ContractPropSpec {
        name: "title",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Visible toast title.",
    },
    ContractPropSpec {
        name: "description",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional supporting copy.",
    },
    ContractPropSpec {
        name: "intent",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("toast_intent"),
        required: false,
        summary: "Toast semantic intent.",
    },
    ContractPropSpec {
        name: "duration_secs",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Auto-dismiss duration in seconds. Zero keeps the toast open until dismissed.",
    },
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional toast lifecycle action id.",
    },
];

const JUSTIFY_VARIANTS: [ContractVariantSpec; 3] = [
    ContractVariantSpec {
        id: "start",
        label: "Start",
        summary: "Align content to the leading edge.",
    },
    ContractVariantSpec {
        id: "center",
        label: "Center",
        summary: "Center content on the main axis.",
    },
    ContractVariantSpec {
        id: "end",
        label: "End",
        summary: "Align content to the trailing edge.",
    },
];

const ALIGN_VARIANTS: [ContractVariantSpec; 3] = [
    ContractVariantSpec {
        id: "start",
        label: "Start",
        summary: "Align content to the leading cross-axis edge.",
    },
    ContractVariantSpec {
        id: "center",
        label: "Center",
        summary: "Center content on the cross axis.",
    },
    ContractVariantSpec {
        id: "end",
        label: "End",
        summary: "Align content to the trailing cross-axis edge.",
    },
];

const TOOLBAR_ANCHOR_VARIANTS: [ContractVariantSpec; 9] = [
    ContractVariantSpec {
        id: "top_left",
        label: "Top Left",
        summary: "Anchor the toolbar to the top-left corner.",
    },
    ContractVariantSpec {
        id: "top_center",
        label: "Top Center",
        summary: "Anchor the toolbar to the top center.",
    },
    ContractVariantSpec {
        id: "top_right",
        label: "Top Right",
        summary: "Anchor the toolbar to the top-right corner.",
    },
    ContractVariantSpec {
        id: "left_center",
        label: "Left Center",
        summary: "Anchor the toolbar to the left center.",
    },
    ContractVariantSpec {
        id: "center",
        label: "Center",
        summary: "Anchor the toolbar to the center.",
    },
    ContractVariantSpec {
        id: "right_center",
        label: "Right Center",
        summary: "Anchor the toolbar to the right center.",
    },
    ContractVariantSpec {
        id: "bottom_left",
        label: "Bottom Left",
        summary: "Anchor the toolbar to the bottom-left corner.",
    },
    ContractVariantSpec {
        id: "bottom_center",
        label: "Bottom Center",
        summary: "Anchor the toolbar to the bottom center.",
    },
    ContractVariantSpec {
        id: "bottom_right",
        label: "Bottom Right",
        summary: "Anchor the toolbar to the bottom-right corner.",
    },
];

const TABS_STYLE_VARIANTS: [ContractVariantSpec; 5] = [
    ContractVariantSpec {
        id: "underline",
        label: "Underline",
        summary: "Inline underline tabs.",
    },
    ContractVariantSpec {
        id: "segmented",
        label: "Segmented",
        summary: "Attached segmented tabs.",
    },
    ContractVariantSpec {
        id: "blender_topbar",
        label: "Blender Topbar",
        summary: "Dense topbar-style tabs.",
    },
    ContractVariantSpec {
        id: "stacked",
        label: "Stacked",
        summary: "Large stacked icon tabs.",
    },
    ContractVariantSpec {
        id: "rail",
        label: "Rail",
        summary: "Compact vertical rail tabs.",
    },
];

const BUTTON_VARIANTS: [ContractVariantSpec; 4] = [
    ContractVariantSpec {
        id: "primary",
        label: "Primary",
        summary: "Primary emphasis action.",
    },
    ContractVariantSpec {
        id: "secondary",
        label: "Secondary",
        summary: "Secondary neutral action.",
    },
    ContractVariantSpec {
        id: "ghost",
        label: "Ghost",
        summary: "Minimal low-chrome action.",
    },
    ContractVariantSpec {
        id: "link",
        label: "Link",
        summary: "Text link action styling.",
    },
];

const CONTROL_SIZE_VARIANTS: [ContractVariantSpec; 2] = [
    ContractVariantSpec {
        id: "sm",
        label: "Small",
        summary: "Compact control sizing.",
    },
    ContractVariantSpec {
        id: "md",
        label: "Medium",
        summary: "Default control sizing.",
    },
];

const LABEL_TONE_VARIANTS: [ContractVariantSpec; 4] = [
    ContractVariantSpec {
        id: "primary",
        label: "Primary",
        summary: "Primary text tone.",
    },
    ContractVariantSpec {
        id: "secondary",
        label: "Secondary",
        summary: "Secondary text tone.",
    },
    ContractVariantSpec {
        id: "muted",
        label: "Muted",
        summary: "Muted helper text tone.",
    },
    ContractVariantSpec {
        id: "destructive",
        label: "Destructive",
        summary: "Error or destructive text tone.",
    },
];

const LABEL_WEIGHT_VARIANTS: [ContractVariantSpec; 3] = [
    ContractVariantSpec {
        id: "regular",
        label: "Regular",
        summary: "Regular label weight.",
    },
    ContractVariantSpec {
        id: "semibold",
        label: "Semibold",
        summary: "Semibold label weight.",
    },
    ContractVariantSpec {
        id: "bold",
        label: "Bold",
        summary: "Bold label weight.",
    },
];

const SELECT_VARIANTS: [ContractVariantSpec; 2] = [
    ContractVariantSpec {
        id: "default",
        label: "Default",
        summary: "Input-styled select trigger.",
    },
    ContractVariantSpec {
        id: "secondary",
        label: "Secondary",
        summary: "Secondary button-styled select trigger.",
    },
];

const SIDEBAR_SIDE_VARIANTS: [ContractVariantSpec; 2] = [
    ContractVariantSpec {
        id: "left",
        label: "Left",
        summary: "Dock the sidebar to the left edge.",
    },
    ContractVariantSpec {
        id: "right",
        label: "Right",
        summary: "Dock the sidebar to the right edge.",
    },
];

const DIALOGUE_INTENT_VARIANTS: [ContractVariantSpec; 2] = [
    ContractVariantSpec {
        id: "default",
        label: "Default",
        summary: "Neutral modal intent.",
    },
    ContractVariantSpec {
        id: "alert",
        label: "Alert",
        summary: "Alert or destructive modal intent.",
    },
];

const TOAST_INTENT_VARIANTS: [ContractVariantSpec; 3] = [
    ContractVariantSpec {
        id: "neutral",
        label: "Neutral",
        summary: "Neutral informational toast.",
    },
    ContractVariantSpec {
        id: "success",
        label: "Success",
        summary: "Success toast.",
    },
    ContractVariantSpec {
        id: "destructive",
        label: "Destructive",
        summary: "Destructive or failure toast.",
    },
];

const TOAST_PLACEMENT_VARIANTS: [ContractVariantSpec; 9] = [
    ContractVariantSpec {
        id: "top_left",
        label: "Top Left",
        summary: "Anchor the toast viewport to the top-left corner.",
    },
    ContractVariantSpec {
        id: "top_center",
        label: "Top Center",
        summary: "Anchor the toast viewport to the top center.",
    },
    ContractVariantSpec {
        id: "top_right",
        label: "Top Right",
        summary: "Anchor the toast viewport to the top-right corner.",
    },
    ContractVariantSpec {
        id: "center_left",
        label: "Center Left",
        summary: "Anchor the toast viewport to the center-left edge.",
    },
    ContractVariantSpec {
        id: "center",
        label: "Center",
        summary: "Anchor the toast viewport to the center.",
    },
    ContractVariantSpec {
        id: "center_right",
        label: "Center Right",
        summary: "Anchor the toast viewport to the center-right edge.",
    },
    ContractVariantSpec {
        id: "bottom_left",
        label: "Bottom Left",
        summary: "Anchor the toast viewport to the bottom-left corner.",
    },
    ContractVariantSpec {
        id: "bottom_center",
        label: "Bottom Center",
        summary: "Anchor the toast viewport to the bottom center.",
    },
    ContractVariantSpec {
        id: "bottom_right",
        label: "Bottom Right",
        summary: "Anchor the toast viewport to the bottom-right corner.",
    },
];

const NUMBER_INPUT_AXIS_VARIANTS: [ContractVariantSpec; 2] = [
    ContractVariantSpec {
        id: "horizontal",
        label: "Horizontal",
        summary: "Horizontal drag axis.",
    },
    ContractVariantSpec {
        id: "vertical",
        label: "Vertical",
        summary: "Vertical drag axis.",
    },
];

const HIERARCHY_KIND_VARIANTS: [ContractVariantSpec; 8] = [
    ContractVariantSpec {
        id: "folder",
        label: "Folder",
        summary: "Folder hierarchy item.",
    },
    ContractVariantSpec {
        id: "frame",
        label: "Frame",
        summary: "Frame hierarchy item.",
    },
    ContractVariantSpec {
        id: "group",
        label: "Group",
        summary: "Group hierarchy item.",
    },
    ContractVariantSpec {
        id: "player",
        label: "Player",
        summary: "Player hierarchy item.",
    },
    ContractVariantSpec {
        id: "weapon",
        label: "Weapon",
        summary: "Weapon hierarchy item.",
    },
    ContractVariantSpec {
        id: "clothing",
        label: "Clothing",
        summary: "Clothing hierarchy item.",
    },
    ContractVariantSpec {
        id: "hitbox",
        label: "Hitbox",
        summary: "Hitbox hierarchy item.",
    },
    ContractVariantSpec {
        id: "vector",
        label: "Vector",
        summary: "Vector hierarchy item.",
    },
];

const HIERARCHY_ICON_STYLE_VARIANTS: [ContractVariantSpec; 2] = [
    ContractVariantSpec {
        id: "emoji",
        label: "Emoji",
        summary: "Use Twemoji-style icons.",
    },
    ContractVariantSpec {
        id: "icons",
        label: "Icons",
        summary: "Use standard icon glyphs.",
    },
];

const HIERARCHY_STYLE_VARIANTS: [ContractVariantSpec; 2] = [
    ContractVariantSpec {
        id: "normal",
        label: "Normal",
        summary: "Normal hierarchy rendering.",
    },
    ContractVariantSpec {
        id: "component",
        label: "Component",
        summary: "Component-themed hierarchy rendering.",
    },
];

const MENU_ENTRY_KIND_VARIANTS: [ContractVariantSpec; 2] = [
    ContractVariantSpec {
        id: "action",
        label: "Action",
        summary: "Clickable menu action.",
    },
    ContractVariantSpec {
        id: "separator",
        label: "Separator",
        summary: "Visual menu divider.",
    },
];

const EVENT_CLICKED: [ContractEventSpec; 1] = [ContractEventSpec {
    kind: EventKind::Clicked,
    summary: "Emitted when the control is clicked.",
}];
const EVENT_SELECTED: [ContractEventSpec; 1] = [ContractEventSpec {
    kind: EventKind::Selected,
    summary: "Emitted when a new item is selected.",
}];
const EVENT_TOGGLED: [ContractEventSpec; 1] = [ContractEventSpec {
    kind: EventKind::Toggled,
    summary: "Emitted when a boolean state toggles.",
}];
const EVENT_CHANGED: [ContractEventSpec; 1] = [ContractEventSpec {
    kind: EventKind::Changed,
    summary: "Emitted when the value changes.",
}];
const EVENT_COMMAND: [ContractEventSpec; 1] = [ContractEventSpec {
    kind: EventKind::CommandInvoked,
    summary: "Emitted when an item action is invoked.",
}];
const EVENT_CLOSED: [ContractEventSpec; 1] = [ContractEventSpec {
    kind: EventKind::Closed,
    summary: "Emitted when the surface closes.",
}];
const INPUT_EVENTS: [ContractEventSpec; 2] = [
    ContractEventSpec {
        kind: EventKind::Changed,
        summary: "Emitted when the input text changes.",
    },
    ContractEventSpec {
        kind: EventKind::Submitted,
        summary: "Emitted when the input is submitted with Enter.",
    },
];
const COLLAPSIBLE_EVENTS: [ContractEventSpec; 3] = [
    ContractEventSpec {
        kind: EventKind::Toggled,
        summary: "Emitted when the open state toggles.",
    },
    ContractEventSpec {
        kind: EventKind::Opened,
        summary: "Emitted when the section opens.",
    },
    ContractEventSpec {
        kind: EventKind::Closed,
        summary: "Emitted when the section closes.",
    },
];
const DIALOGUE_EVENTS: [ContractEventSpec; 3] = [
    ContractEventSpec {
        kind: EventKind::Confirmed,
        summary: "Emitted when the confirm action is chosen.",
    },
    ContractEventSpec {
        kind: EventKind::Cancelled,
        summary: "Emitted when the cancel action is chosen.",
    },
    ContractEventSpec {
        kind: EventKind::Closed,
        summary: "Emitted when the modal closes without a semantic confirm/cancel action.",
    },
];
const HIERARCHY_EVENTS: [ContractEventSpec; 3] = [
    ContractEventSpec {
        kind: EventKind::Selected,
        summary: "Emitted when the selected row changes.",
    },
    ContractEventSpec {
        kind: EventKind::Opened,
        summary: "Emitted when a branch expands.",
    },
    ContractEventSpec {
        kind: EventKind::Closed,
        summary: "Emitted when a branch collapses.",
    },
];
const TOAST_EVENTS: [ContractEventSpec; 2] = [
    ContractEventSpec {
        kind: EventKind::Opened,
        summary: "Emitted when a toast item first appears in the viewport.",
    },
    ContractEventSpec {
        kind: EventKind::Closed,
        summary: "Emitted when a toast item expires or is dismissed.",
    },
];

const ROW_PROPS: [ContractPropSpec; 4] = [
    ContractPropSpec {
        name: "gap",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Horizontal gap between child nodes.",
    },
    ContractPropSpec {
        name: "justify",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("justify"),
        required: false,
        summary: "Main-axis alignment.",
    },
    ContractPropSpec {
        name: "align",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("align"),
        required: false,
        summary: "Cross-axis alignment.",
    },
    ContractPropSpec {
        name: "children",
        kind: ContractPropTypeKind::NodeList,
        type_name: None,
        required: true,
        summary: "Nested contract child nodes.",
    },
];

const COLUMN_PROPS: [ContractPropSpec; 4] = [
    ContractPropSpec {
        name: "gap",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Vertical gap between child nodes.",
    },
    ContractPropSpec {
        name: "justify",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("justify"),
        required: false,
        summary: "Main-axis alignment.",
    },
    ContractPropSpec {
        name: "align",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("align"),
        required: false,
        summary: "Cross-axis alignment.",
    },
    ContractPropSpec {
        name: "children",
        kind: ContractPropTypeKind::NodeList,
        type_name: None,
        required: true,
        summary: "Nested contract child nodes.",
    },
];

const INSET_PROPS: [ContractPropSpec; 3] = [
    ContractPropSpec {
        name: "padding_x",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Horizontal inset padding.",
    },
    ContractPropSpec {
        name: "padding_y",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Vertical inset padding.",
    },
    ContractPropSpec {
        name: "children",
        kind: ContractPropTypeKind::NodeList,
        type_name: None,
        required: true,
        summary: "Nested contract child nodes.",
    },
];

const SIZED_BOX_PROPS: [ContractPropSpec; 3] = [
    ContractPropSpec {
        name: "width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Optional explicit width.",
    },
    ContractPropSpec {
        name: "height",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Optional explicit height.",
    },
    ContractPropSpec {
        name: "children",
        kind: ContractPropTypeKind::NodeList,
        type_name: None,
        required: true,
        summary: "Nested contract child nodes.",
    },
];

const SPACER_PROPS: [ContractPropSpec; 3] = [
    ContractPropSpec {
        name: "width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Optional fixed spacer width.",
    },
    ContractPropSpec {
        name: "height",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Optional fixed spacer height.",
    },
    ContractPropSpec {
        name: "flex",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: false,
        summary: "Whether the spacer should fill remaining space.",
    },
];

const CARD_PROPS: [ContractPropSpec; 3] = [
    ContractPropSpec {
        name: "padding_x",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Horizontal card padding.",
    },
    ContractPropSpec {
        name: "padding_y",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Vertical card padding.",
    },
    ContractPropSpec {
        name: "children",
        kind: ContractPropTypeKind::NodeList,
        type_name: None,
        required: true,
        summary: "Nested contract child nodes.",
    },
];

const SIDEBAR_PROPS: [ContractPropSpec; 5] = [
    ContractPropSpec {
        name: "title",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional sidebar title.",
    },
    ContractPropSpec {
        name: "side",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("sidebar_side"),
        required: false,
        summary: "Docking side.",
    },
    ContractPropSpec {
        name: "width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Sidebar width.",
    },
    ContractPropSpec {
        name: "open",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: false,
        summary: "Whether the sidebar is currently open.",
    },
    ContractPropSpec {
        name: "children",
        kind: ContractPropTypeKind::NodeList,
        type_name: None,
        required: true,
        summary: "Sidebar body children.",
    },
];

const TOOLBAR_PROPS: [ContractPropSpec; 4] = [
    ContractPropSpec {
        name: "anchor",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("toolbar_anchor"),
        required: false,
        summary: "Toolbar anchor point.",
    },
    ContractPropSpec {
        name: "offset_x",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Horizontal anchor offset.",
    },
    ContractPropSpec {
        name: "offset_y",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Vertical anchor offset.",
    },
    ContractPropSpec {
        name: "children",
        kind: ContractPropTypeKind::NodeList,
        type_name: None,
        required: true,
        summary: "Toolbar child nodes.",
    },
];

const MENU_BAR_PROPS: [ContractPropSpec; 1] = [ContractPropSpec {
    name: "menus",
    kind: ContractPropTypeKind::ObjectList,
    type_name: Some("menu"),
    required: true,
    summary: "Top-level menus and entries.",
}];

const TABS_PROPS: [ContractPropSpec; 3] = [
    ContractPropSpec {
        name: "style",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("tabs_style"),
        required: false,
        summary: "Visual tab presentation.",
    },
    ContractPropSpec {
        name: "selected_item_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Currently selected tab id.",
    },
    ContractPropSpec {
        name: "items",
        kind: ContractPropTypeKind::ObjectList,
        type_name: Some("tab_item"),
        required: true,
        summary: "Tab items.",
    },
];

const LABEL_PROPS: [ContractPropSpec; 4] = [
    ContractPropSpec {
        name: "text",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Visible label text.",
    },
    ContractPropSpec {
        name: "tone",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("label_tone"),
        required: false,
        summary: "Semantic text tone.",
    },
    ContractPropSpec {
        name: "weight",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("label_weight"),
        required: false,
        summary: "Semantic text weight.",
    },
    ContractPropSpec {
        name: "size",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Optional text size override.",
    },
];

const BUTTON_PROPS: [ContractPropSpec; 9] = [
    ContractPropSpec {
        name: "label",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Visible button label.",
    },
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional action id emitted on click.",
    },
    ContractPropSpec {
        name: "variant",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("button_variant"),
        required: false,
        summary: "Button visual variant.",
    },
    ContractPropSpec {
        name: "size",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("control_size"),
        required: false,
        summary: "Button size token.",
    },
    ContractPropSpec {
        name: "leading_icon",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional leading icon.",
    },
    ContractPropSpec {
        name: "trailing_text",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional trailing helper text.",
    },
    ContractPropSpec {
        name: "trailing_icon",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional trailing icon.",
    },
    ContractPropSpec {
        name: "icon_only",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: false,
        summary: "Whether the button should render as icon-only.",
    },
    ContractPropSpec {
        name: "selected",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: false,
        summary: "Whether the button should render in a selected state.",
    },
];

const BUTTON_GROUP_PROPS: [ContractPropSpec; 1] = [ContractPropSpec {
    name: "items",
    kind: ContractPropTypeKind::ObjectList,
    type_name: Some("action_item"),
    required: true,
    summary: "Action items for the group.",
}];

const INPUT_PROPS: [ContractPropSpec; 5] = [
    ContractPropSpec {
        name: "value",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Current text value.",
    },
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional input action id.",
    },
    ContractPropSpec {
        name: "placeholder",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Placeholder copy.",
    },
    ContractPropSpec {
        name: "leading_icon",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional leading icon.",
    },
    ContractPropSpec {
        name: "width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Requested control width.",
    },
];

const NUMBER_INPUT_PROPS: [ContractPropSpec; 10] = [
    ContractPropSpec {
        name: "value",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: true,
        summary: "Current numeric value.",
    },
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional action id.",
    },
    ContractPropSpec {
        name: "width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Requested control width.",
    },
    ContractPropSpec {
        name: "min",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Inclusive numeric minimum.",
    },
    ContractPropSpec {
        name: "max",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Inclusive numeric maximum.",
    },
    ContractPropSpec {
        name: "speed",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Drag speed.",
    },
    ContractPropSpec {
        name: "decimals",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Displayed decimal precision.",
    },
    ContractPropSpec {
        name: "prefix",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional leading unit text.",
    },
    ContractPropSpec {
        name: "suffix",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional trailing unit text.",
    },
    ContractPropSpec {
        name: "axis",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("number_input_axis"),
        required: false,
        summary: "Primary drag axis.",
    },
];

const CHECKBOX_PROPS: [ContractPropSpec; 3] = [
    ContractPropSpec {
        name: "value",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: true,
        summary: "Current checked value.",
    },
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional action id.",
    },
    ContractPropSpec {
        name: "label",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional checkbox label.",
    },
];

const SWITCH_PROPS: [ContractPropSpec; 4] = [
    ContractPropSpec {
        name: "value",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: true,
        summary: "Current switch value.",
    },
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional action id.",
    },
    ContractPropSpec {
        name: "label",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional switch label.",
    },
    ContractPropSpec {
        name: "size",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("control_size"),
        required: false,
        summary: "Switch size token.",
    },
];

const SELECT_PROPS: [ContractPropSpec; 7] = [
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional select action id.",
    },
    ContractPropSpec {
        name: "selected_item_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Currently selected item id.",
    },
    ContractPropSpec {
        name: "placeholder",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Placeholder copy.",
    },
    ContractPropSpec {
        name: "width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Requested trigger width.",
    },
    ContractPropSpec {
        name: "variant",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("select_variant"),
        required: false,
        summary: "Trigger styling variant.",
    },
    ContractPropSpec {
        name: "leading_icon",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional leading icon.",
    },
    ContractPropSpec {
        name: "items",
        kind: ContractPropTypeKind::ObjectList,
        type_name: Some("choice_item"),
        required: true,
        summary: "Selectable items.",
    },
];

const FIELD_PROPS: [ContractPropSpec; 6] = [
    ContractPropSpec {
        name: "label",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Field label.",
    },
    ContractPropSpec {
        name: "value",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Current text value.",
    },
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional field action id.",
    },
    ContractPropSpec {
        name: "helper_text",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional helper copy.",
    },
    ContractPropSpec {
        name: "placeholder",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional placeholder copy.",
    },
    ContractPropSpec {
        name: "width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Requested input width.",
    },
];

const COLLAPSIBLE_PROPS: [ContractPropSpec; 5] = [
    ContractPropSpec {
        name: "title",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Section title.",
    },
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional section action id.",
    },
    ContractPropSpec {
        name: "open",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: false,
        summary: "Whether the body is currently expanded.",
    },
    ContractPropSpec {
        name: "leading_icon",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional leading icon.",
    },
    ContractPropSpec {
        name: "children",
        kind: ContractPropTypeKind::NodeList,
        type_name: None,
        required: true,
        summary: "Expandable body children.",
    },
];

const DIALOGUE_PROPS: [ContractPropSpec; 10] = [
    ContractPropSpec {
        name: "open",
        kind: ContractPropTypeKind::Boolean,
        type_name: None,
        required: false,
        summary: "Whether the modal is currently open.",
    },
    ContractPropSpec {
        name: "title",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: true,
        summary: "Modal title.",
    },
    ContractPropSpec {
        name: "description",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional modal description.",
    },
    ContractPropSpec {
        name: "confirm_label",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Confirm button label.",
    },
    ContractPropSpec {
        name: "cancel_label",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Cancel button label.",
    },
    ContractPropSpec {
        name: "intent",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("dialogue_intent"),
        required: false,
        summary: "Modal semantic intent.",
    },
    ContractPropSpec {
        name: "width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Modal width.",
    },
    ContractPropSpec {
        name: "confirm_action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Confirm action id.",
    },
    ContractPropSpec {
        name: "cancel_action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Cancel action id.",
    },
    ContractPropSpec {
        name: "children",
        kind: ContractPropTypeKind::NodeList,
        type_name: None,
        required: true,
        summary: "Modal body children.",
    },
];

const HIERARCHY_PROPS: [ContractPropSpec; 8] = [
    ContractPropSpec {
        name: "action_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Optional hierarchy-level action id.",
    },
    ContractPropSpec {
        name: "selected_item_id",
        kind: ContractPropTypeKind::String,
        type_name: None,
        required: false,
        summary: "Currently selected item id.",
    },
    ContractPropSpec {
        name: "width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Requested hierarchy width.",
    },
    ContractPropSpec {
        name: "row_height",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Row height override.",
    },
    ContractPropSpec {
        name: "indent_width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Indent width override.",
    },
    ContractPropSpec {
        name: "icon_style",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("hierarchy_icon_style"),
        required: false,
        summary: "Icon rendering style.",
    },
    ContractPropSpec {
        name: "style",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("hierarchy_style"),
        required: false,
        summary: "Hierarchy surface style.",
    },
    ContractPropSpec {
        name: "items",
        kind: ContractPropTypeKind::ObjectList,
        type_name: Some("hierarchy_item"),
        required: true,
        summary: "Hierarchy tree items.",
    },
];

const SPINNER_PROPS: [ContractPropSpec; 1] = [ContractPropSpec {
    name: "size",
    kind: ContractPropTypeKind::Number,
    type_name: None,
    required: false,
    summary: "Spinner diameter.",
}];

const PROGRESS_PROPS: [ContractPropSpec; 3] = [
    ContractPropSpec {
        name: "value",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: true,
        summary: "Determinate progress value in the 0..=1 range.",
    },
    ContractPropSpec {
        name: "width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Progress bar width.",
    },
    ContractPropSpec {
        name: "height",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Progress bar height.",
    },
];

const TOAST_VIEWPORT_PROPS: [ContractPropSpec; 8] = [
    ContractPropSpec {
        name: "placement",
        kind: ContractPropTypeKind::Enum,
        type_name: Some("toast_placement"),
        required: false,
        summary: "Viewport anchor placement.",
    },
    ContractPropSpec {
        name: "width",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Requested toast width.",
    },
    ContractPropSpec {
        name: "margin_x",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Horizontal viewport margin.",
    },
    ContractPropSpec {
        name: "margin_y",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Vertical viewport margin.",
    },
    ContractPropSpec {
        name: "gap",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Gap between toast rows.",
    },
    ContractPropSpec {
        name: "overlap",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Optional stacked overlap between visible toasts.",
    },
    ContractPropSpec {
        name: "max_visible",
        kind: ContractPropTypeKind::Number,
        type_name: None,
        required: false,
        summary: "Maximum number of visible toasts at once.",
    },
    ContractPropSpec {
        name: "toasts",
        kind: ContractPropTypeKind::ObjectList,
        type_name: Some("toast_item"),
        required: true,
        summary: "Host-owned toast items.",
    },
];

const FAMILY_VARIANT_LAYOUT: [ContractVariantRef; 2] = [
    ContractVariantRef {
        prop: "justify",
        type_name: "justify",
    },
    ContractVariantRef {
        prop: "align",
        type_name: "align",
    },
];

const FAMILY_VARIANT_BUTTON: [ContractVariantRef; 2] = [
    ContractVariantRef {
        prop: "variant",
        type_name: "button_variant",
    },
    ContractVariantRef {
        prop: "size",
        type_name: "control_size",
    },
];

const FAMILY_VARIANT_SWITCH: [ContractVariantRef; 1] = [ContractVariantRef {
    prop: "size",
    type_name: "control_size",
}];

const FAMILY_VARIANT_LABEL: [ContractVariantRef; 2] = [
    ContractVariantRef {
        prop: "tone",
        type_name: "label_tone",
    },
    ContractVariantRef {
        prop: "weight",
        type_name: "label_weight",
    },
];

const FAMILY_VARIANT_TABS: [ContractVariantRef; 1] = [ContractVariantRef {
    prop: "style",
    type_name: "tabs_style",
}];

const FAMILY_VARIANT_SIDEBAR: [ContractVariantRef; 1] = [ContractVariantRef {
    prop: "side",
    type_name: "sidebar_side",
}];

const FAMILY_VARIANT_TOOLBAR: [ContractVariantRef; 1] = [ContractVariantRef {
    prop: "anchor",
    type_name: "toolbar_anchor",
}];

const FAMILY_VARIANT_SELECT: [ContractVariantRef; 1] = [ContractVariantRef {
    prop: "variant",
    type_name: "select_variant",
}];

const FAMILY_VARIANT_NUMBER_INPUT: [ContractVariantRef; 1] = [ContractVariantRef {
    prop: "axis",
    type_name: "number_input_axis",
}];

const FAMILY_VARIANT_DIALOGUE: [ContractVariantRef; 1] = [ContractVariantRef {
    prop: "intent",
    type_name: "dialogue_intent",
}];

const FAMILY_VARIANT_HIERARCHY: [ContractVariantRef; 2] = [
    ContractVariantRef {
        prop: "icon_style",
        type_name: "hierarchy_icon_style",
    },
    ContractVariantRef {
        prop: "style",
        type_name: "hierarchy_style",
    },
];

const FAMILY_VARIANT_TOAST_VIEWPORT: [ContractVariantRef; 1] = [ContractVariantRef {
    prop: "placement",
    type_name: "toast_placement",
}];

const FAMILIES: [ContractFamilySpec; 26] = [
    ContractFamilySpec {
        id: ContractFamilyId::Row,
        display_name: "Row",
        summary: "Horizontal layout container.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Children,
        props: &ROW_PROPS,
        variants: &FAMILY_VARIANT_LAYOUT,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Column,
        display_name: "Column",
        summary: "Vertical layout container.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Children,
        props: &COLUMN_PROPS,
        variants: &FAMILY_VARIANT_LAYOUT,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Inset,
        display_name: "Inset",
        summary: "Padding wrapper around child nodes.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Children,
        props: &INSET_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::SizedBox,
        display_name: "Sized Box",
        summary: "Explicit width and height wrapper.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Children,
        props: &SIZED_BOX_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Spacer,
        display_name: "Spacer",
        summary: "Fixed or flex spacer.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &SPACER_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Card,
        display_name: "Card",
        summary: "General framed surface container.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Children,
        props: &CARD_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Sidebar,
        display_name: "Sidebar",
        summary: "Overlay sidebar container.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Children,
        props: &SIDEBAR_PROPS,
        variants: &FAMILY_VARIANT_SIDEBAR,
        events: &EVENT_CLOSED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Toolbar,
        display_name: "Toolbar",
        summary: "Anchored floating toolbar surface.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Children,
        props: &TOOLBAR_PROPS,
        variants: &FAMILY_VARIANT_TOOLBAR,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::MenuBar,
        display_name: "Menu Bar",
        summary: "Desktop-style menu bar with action entries and separators.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &MENU_BAR_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_COMMAND,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Tabs,
        display_name: "Tabs",
        summary: "Single-selection tab navigation.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &TABS_PROPS,
        variants: &FAMILY_VARIANT_TABS,
        events: &EVENT_SELECTED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Label,
        display_name: "Label",
        summary: "Semantic text presentation.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &LABEL_PROPS,
        variants: &FAMILY_VARIANT_LABEL,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Button,
        display_name: "Button",
        summary: "Clickable action button.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &BUTTON_PROPS,
        variants: &FAMILY_VARIANT_BUTTON,
        events: &EVENT_CLICKED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::ButtonGroup,
        display_name: "Button Group",
        summary: "Attached action group.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &BUTTON_GROUP_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_COMMAND,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Input,
        display_name: "Input",
        summary: "Single-line text input.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &INPUT_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &INPUT_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::NumberInput,
        display_name: "Number Input",
        summary: "Numeric entry control.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &NUMBER_INPUT_PROPS,
        variants: &FAMILY_VARIANT_NUMBER_INPUT,
        events: &EVENT_CHANGED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Checkbox,
        display_name: "Checkbox",
        summary: "Boolean checkbox control.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &CHECKBOX_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_TOGGLED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Switch,
        display_name: "Switch",
        summary: "Boolean switch control.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &SWITCH_PROPS,
        variants: &FAMILY_VARIANT_SWITCH,
        events: &EVENT_TOGGLED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Select,
        display_name: "Select",
        summary: "Single-choice select menu.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &SELECT_PROPS,
        variants: &FAMILY_VARIANT_SELECT,
        events: &EVENT_SELECTED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Field,
        display_name: "Field",
        summary: "Label plus text input plus helper text.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &FIELD_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_CHANGED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Separator,
        display_name: "Separator",
        summary: "Visual content divider.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Collapsible,
        display_name: "Collapsible",
        summary: "Expandable section with body children.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Children,
        props: &COLLAPSIBLE_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &COLLAPSIBLE_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::DialogueModal,
        display_name: "Dialogue Modal",
        summary: "Host-owned modal confirmation surface.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Body,
        props: &DIALOGUE_PROPS,
        variants: &FAMILY_VARIANT_DIALOGUE,
        events: &DIALOGUE_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Hierarchy,
        display_name: "Hierarchy",
        summary: "Hierarchical selection tree with expand and collapse state.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &HIERARCHY_PROPS,
        variants: &FAMILY_VARIANT_HIERARCHY,
        events: &HIERARCHY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Spinner,
        display_name: "Spinner",
        summary: "Indeterminate loading spinner.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &SPINNER_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Progress,
        display_name: "Progress",
        summary: "Determinate progress indicator.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &PROGRESS_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::ToastViewport,
        display_name: "Toast Viewport",
        summary: "Overlay toast stack with lifecycle events.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &TOAST_VIEWPORT_PROPS,
        variants: &FAMILY_VARIANT_TOAST_VIEWPORT,
        events: &TOAST_EVENTS,
    },
];

const SHARED_TYPES: [ContractSharedTypeSpec; 27] = [
    ContractSharedTypeSpec {
        name: "node_common",
        display_name: "Node Common",
        kind: ContractSharedTypeKind::Object,
        summary: "Common fields flattened into every node.",
        fields: &NODE_COMMON_FIELDS,
        variants: &EMPTY_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "justify",
        display_name: "Justify",
        kind: ContractSharedTypeKind::Enum,
        summary: "Main-axis alignment values.",
        fields: &EMPTY_PROPS,
        variants: &JUSTIFY_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "align",
        display_name: "Align",
        kind: ContractSharedTypeKind::Enum,
        summary: "Cross-axis alignment values.",
        fields: &EMPTY_PROPS,
        variants: &ALIGN_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "toolbar_anchor",
        display_name: "Toolbar Anchor",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported toolbar anchor points.",
        fields: &EMPTY_PROPS,
        variants: &TOOLBAR_ANCHOR_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "tabs_style",
        display_name: "Tabs Style",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported tab presentations.",
        fields: &EMPTY_PROPS,
        variants: &TABS_STYLE_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "button_variant",
        display_name: "Button Variant",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported button variants.",
        fields: &EMPTY_PROPS,
        variants: &BUTTON_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "control_size",
        display_name: "Control Size",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported control sizes.",
        fields: &EMPTY_PROPS,
        variants: &CONTROL_SIZE_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "label_tone",
        display_name: "Label Tone",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported text tones.",
        fields: &EMPTY_PROPS,
        variants: &LABEL_TONE_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "label_weight",
        display_name: "Label Weight",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported text weights.",
        fields: &EMPTY_PROPS,
        variants: &LABEL_WEIGHT_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "select_variant",
        display_name: "Select Variant",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported select trigger variants.",
        fields: &EMPTY_PROPS,
        variants: &SELECT_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "sidebar_side",
        display_name: "Sidebar Side",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported sidebar docking sides.",
        fields: &EMPTY_PROPS,
        variants: &SIDEBAR_SIDE_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "dialogue_intent",
        display_name: "Dialogue Intent",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported dialogue intents.",
        fields: &EMPTY_PROPS,
        variants: &DIALOGUE_INTENT_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "toast_intent",
        display_name: "Toast Intent",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported toast intents.",
        fields: &EMPTY_PROPS,
        variants: &TOAST_INTENT_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "toast_placement",
        display_name: "Toast Placement",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported toast viewport placements.",
        fields: &EMPTY_PROPS,
        variants: &TOAST_PLACEMENT_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "number_input_axis",
        display_name: "Number Input Axis",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported drag axes for number input.",
        fields: &EMPTY_PROPS,
        variants: &NUMBER_INPUT_AXIS_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "hierarchy_item_kind",
        display_name: "Hierarchy Item Kind",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported hierarchy semantic item kinds.",
        fields: &EMPTY_PROPS,
        variants: &HIERARCHY_KIND_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "menu_entry_kind",
        display_name: "Menu Entry Kind",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported menu entry kinds.",
        fields: &EMPTY_PROPS,
        variants: &MENU_ENTRY_KIND_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "hierarchy_icon_style",
        display_name: "Hierarchy Icon Style",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported hierarchy icon styles.",
        fields: &EMPTY_PROPS,
        variants: &HIERARCHY_ICON_STYLE_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "hierarchy_style",
        display_name: "Hierarchy Style",
        kind: ContractSharedTypeKind::Enum,
        summary: "Supported hierarchy surface styles.",
        fields: &EMPTY_PROPS,
        variants: &HIERARCHY_STYLE_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "action_item",
        display_name: "Action Item",
        kind: ContractSharedTypeKind::Object,
        summary: "Button-group style action item.",
        fields: &ACTION_ITEM_FIELDS,
        variants: &EMPTY_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "choice_item",
        display_name: "Choice Item",
        kind: ContractSharedTypeKind::Object,
        summary: "Select choice item.",
        fields: &CHOICE_ITEM_FIELDS,
        variants: &EMPTY_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "tab_item",
        display_name: "Tab Item",
        kind: ContractSharedTypeKind::Object,
        summary: "Tab navigation item.",
        fields: &TAB_ITEM_FIELDS,
        variants: &EMPTY_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "menu_action",
        display_name: "Menu Action",
        kind: ContractSharedTypeKind::Object,
        summary: "Clickable menu action row.",
        fields: &MENU_ACTION_FIELDS,
        variants: &EMPTY_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "menu_entry",
        display_name: "Menu Entry",
        kind: ContractSharedTypeKind::Object,
        summary: "Action-or-separator menu entry union.",
        fields: &MENU_ENTRY_FIELDS,
        variants: &EMPTY_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "menu",
        display_name: "Menu",
        kind: ContractSharedTypeKind::Object,
        summary: "Top-level menu bar menu.",
        fields: &MENU_FIELDS,
        variants: &EMPTY_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "hierarchy_item",
        display_name: "Hierarchy Item",
        kind: ContractSharedTypeKind::Object,
        summary: "Recursive hierarchy tree item.",
        fields: &HIERARCHY_ITEM_FIELDS,
        variants: &EMPTY_VARIANTS,
    },
    ContractSharedTypeSpec {
        name: "toast_item",
        display_name: "Toast Item",
        kind: ContractSharedTypeKind::Object,
        summary: "Toast lifecycle item.",
        fields: &TOAST_ITEM_FIELDS,
        variants: &EMPTY_VARIANTS,
    },
];

pub fn registry() -> &'static [ContractFamilySpec] {
    &FAMILIES
}

pub fn shared_types() -> &'static [ContractSharedTypeSpec] {
    &SHARED_TYPES
}

pub fn schema() -> ContractSchema {
    ContractSchema {
        version: CONTRACT_MODEL_VERSION,
        families: registry(),
        shared_types: shared_types(),
    }
}

pub fn schema_json() -> serde_json::Result<String> {
    serde_json::to_string(&schema())
}

pub fn schema_json_pretty() -> serde_json::Result<String> {
    serde_json::to_string_pretty(&schema())
}

pub fn reference_markdown() -> String {
    let mut markdown = String::from(
        "# Contract Reference\n\nGenerated from `egui_component::contract::registry()` and `egui_component::contract::shared_types()`.\n\nExport the schema and human-readable reference from Rust with:\n\n```rust\negui_component::contract::schema_json_pretty()\negui_component::contract::reference_markdown()\n```\n\n## Shared Node Fields\n\nEvery contract node includes:\n\n- `node_id`\n- `visible`\n- `enabled`\n\n## Supported Families\n\n| Family | Child Policy | Primary Events | Summary |\n| --- | --- | --- | --- |\n",
    );

    for family in registry() {
        let events = if family.events.is_empty() {
            String::from("none")
        } else {
            family
                .events
                .iter()
                .map(|event| format!("`{}`", event_kind_id(event.kind)))
                .collect::<Vec<_>>()
                .join(", ")
        };
        markdown.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            family.id.as_str(),
            child_policy_id(family.child_policy),
            events,
            family.summary
        ));
    }

    markdown.push_str("\n## Shared Types\n\n");
    for shared_type in shared_types() {
        markdown.push_str(&format!(
            "- `{}` (`{}`): {}\n",
            shared_type.name,
            shared_type_kind_id(shared_type.kind),
            shared_type.summary
        ));
    }

    markdown.push_str(
        "\n## Host Integration Pattern\n\n1. Build a `ContractTree` from host state.\n2. Render it with `render_tree` or `render_component_tree`.\n3. Apply the returned `ContractEvent`s to host state.\n4. Rebuild the next frame's tree from the updated authoritative host state.\n",
    );

    markdown
}

fn child_policy_id(policy: ContractChildPolicy) -> &'static str {
    match policy {
        ContractChildPolicy::None => "none",
        ContractChildPolicy::Children => "children",
        ContractChildPolicy::Body => "body",
    }
}

fn shared_type_kind_id(kind: ContractSharedTypeKind) -> &'static str {
    match kind {
        ContractSharedTypeKind::Enum => "enum",
        ContractSharedTypeKind::Object => "object",
    }
}

fn event_kind_id(kind: EventKind) -> &'static str {
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
    use super::{reference_markdown, registry, schema_json_pretty, shared_types};

    #[test]
    fn registry_has_unique_family_ids() {
        let ids = registry()
            .iter()
            .map(|family| family.id.as_str())
            .collect::<Vec<_>>();
        let mut unique = ids.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn schema_json_mentions_core_families_and_shared_types() {
        let json = schema_json_pretty().expect("schema json");
        assert!(json.contains("\"button\""));
        assert!(json.contains("\"dialogue-modal\""));
        assert!(json.contains("\"toast-viewport\""));
        assert!(json.contains("\"node_common\""));
        assert!(json.contains("\"menu\""));
        assert!(json.contains("\"hierarchy_item\""));
        assert!(json.contains("\"toast_item\""));
        assert!(!shared_types().is_empty());
    }

    #[test]
    fn registry_includes_expected_prop_coverage_for_curated_families() {
        let button = registry()
            .iter()
            .find(|family| family.id.as_str() == "button")
            .expect("button family");
        assert!(button.props.iter().any(|prop| prop.name == "selected"));

        let number_input = registry()
            .iter()
            .find(|family| family.id.as_str() == "number-input")
            .expect("number-input family");
        assert!(number_input.props.iter().any(|prop| prop.name == "suffix"));

        let dialogue = registry()
            .iter()
            .find(|family| family.id.as_str() == "dialogue-modal")
            .expect("dialogue-modal family");
        assert!(dialogue
            .props
            .iter()
            .any(|prop| prop.name == "cancel_action_id"));

        let hierarchy = registry()
            .iter()
            .find(|family| family.id.as_str() == "hierarchy")
            .expect("hierarchy family");
        assert!(hierarchy.props.iter().any(|prop| prop.name == "style"));

        let toast = registry()
            .iter()
            .find(|family| family.id.as_str() == "toast-viewport")
            .expect("toast-viewport family");
        assert!(toast.props.iter().any(|prop| prop.name == "toasts"));
    }

    #[test]
    fn reference_markdown_mentions_every_family() {
        let markdown = reference_markdown();
        for family in registry() {
            assert!(markdown.contains(&format!("`{}`", family.id.as_str())));
        }
        assert!(markdown.contains("toast_item"));
    }
}
