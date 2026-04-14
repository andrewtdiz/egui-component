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
    StringList,
    StringMap,
    Boolean,
    Number,
    Enum,
    Object,
    ObjectList,
    Node,
    NodeList,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractSupportStatus {
    Supported,
    Partial,
    Unsupported,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize)]
pub struct ContractPropSpec {
    pub name: &'static str,
    pub kind: ContractPropTypeKind,
    pub type_name: Option<&'static str>,
    pub required: bool,
    pub support: ContractSupportStatus,
    pub support_summary: &'static str,
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
    pub support: ContractSupportStatus,
    pub support_summary: &'static str,
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

const fn supported_prop(
    name: &'static str,
    kind: ContractPropTypeKind,
    type_name: Option<&'static str>,
    required: bool,
    summary: &'static str,
) -> ContractPropSpec {
    ContractPropSpec {
        name,
        kind,
        type_name,
        required,
        support: ContractSupportStatus::Supported,
        support_summary: "",
        summary,
    }
}

const fn partial_prop(
    name: &'static str,
    kind: ContractPropTypeKind,
    type_name: Option<&'static str>,
    required: bool,
    summary: &'static str,
    support_summary: &'static str,
) -> ContractPropSpec {
    ContractPropSpec {
        name,
        kind,
        type_name,
        required,
        support: ContractSupportStatus::Partial,
        support_summary,
        summary,
    }
}

const fn unsupported_prop(
    name: &'static str,
    kind: ContractPropTypeKind,
    type_name: Option<&'static str>,
    required: bool,
    summary: &'static str,
    support_summary: &'static str,
) -> ContractPropSpec {
    ContractPropSpec {
        name,
        kind,
        type_name,
        required,
        support: ContractSupportStatus::Unsupported,
        support_summary,
        summary,
    }
}

const fn supported_shared_type(
    name: &'static str,
    display_name: &'static str,
    kind: ContractSharedTypeKind,
    summary: &'static str,
    fields: &'static [ContractPropSpec],
    variants: &'static [ContractVariantSpec],
) -> ContractSharedTypeSpec {
    ContractSharedTypeSpec {
        name,
        display_name,
        kind,
        support: ContractSupportStatus::Supported,
        support_summary: "",
        summary,
        fields,
        variants,
    }
}

const fn partial_shared_type(
    name: &'static str,
    display_name: &'static str,
    kind: ContractSharedTypeKind,
    summary: &'static str,
    support_summary: &'static str,
    fields: &'static [ContractPropSpec],
    variants: &'static [ContractVariantSpec],
) -> ContractSharedTypeSpec {
    ContractSharedTypeSpec {
        name,
        display_name,
        kind,
        support: ContractSupportStatus::Partial,
        support_summary,
        summary,
        fields,
        variants,
    }
}

const fn unsupported_shared_type(
    name: &'static str,
    display_name: &'static str,
    kind: ContractSharedTypeKind,
    summary: &'static str,
    support_summary: &'static str,
    fields: &'static [ContractPropSpec],
    variants: &'static [ContractVariantSpec],
) -> ContractSharedTypeSpec {
    ContractSharedTypeSpec {
        name,
        display_name,
        kind,
        support: ContractSupportStatus::Unsupported,
        support_summary,
        summary,
        fields,
        variants,
    }
}

const NODE_COMMON_FIELDS: [ContractPropSpec; 8] = [
    supported_prop(
        "node_id",
        ContractPropTypeKind::String,
        None,
        true,
        "Stable host-owned node identifier.",
    ),
    supported_prop(
        "visible",
        ContractPropTypeKind::Boolean,
        None,
        false,
        "Whether the node renders at all. Defaults to true.",
    ),
    supported_prop(
        "enabled",
        ContractPropTypeKind::Boolean,
        None,
        false,
        "Whether interaction is enabled. Defaults to true.",
    ),
    supported_prop(
        "class",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional primary class string.",
    ),
    supported_prop(
        "class_list",
        ContractPropTypeKind::StringList,
        None,
        false,
        "Optional expanded class token list.",
    ),
    unsupported_prop(
        "slot_classes",
        ContractPropTypeKind::StringMap,
        None,
        false,
        "Optional slot-name to class-string overrides.",
        "Declared in the model and schema only. The current Rust renderer does not apply slot-specific class behavior.",
    ),
    unsupported_prop(
        "actions",
        ContractPropTypeKind::Object,
        Some("actions"),
        false,
        "Optional common semantic action bindings.",
        "Declared in the model and schema only. The current renderer uses family-specific action fields instead.",
    ),
    partial_prop(
        "layout",
        ContractPropTypeKind::Object,
        Some("layout"),
        false,
        "Optional shared layout hints.",
        "The renderer applies sizing, padding, and margin on every node and container direction/justify/align/gap/wrap overrides on flow containers only.",
    ),
];

const ACTIONS_FIELDS: [ContractPropSpec; 8] = [
    unsupported_prop(
        "click",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional click action id.",
        "Common action bindings are declared but not executed by the current renderer.",
    ),
    unsupported_prop(
        "change",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional change action id.",
        "Common action bindings are declared but not executed by the current renderer.",
    ),
    unsupported_prop(
        "submit",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional submit action id.",
        "Common action bindings are declared but not executed by the current renderer.",
    ),
    unsupported_prop(
        "select",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional select action id.",
        "Common action bindings are declared but not executed by the current renderer.",
    ),
    unsupported_prop(
        "open",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional open action id.",
        "Common action bindings are declared but not executed by the current renderer.",
    ),
    unsupported_prop(
        "close",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional close action id.",
        "Common action bindings are declared but not executed by the current renderer.",
    ),
    unsupported_prop(
        "confirm",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional confirm action id.",
        "Common action bindings are declared but not executed by the current renderer.",
    ),
    unsupported_prop(
        "cancel",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional cancel action id.",
        "Common action bindings are declared but not executed by the current renderer.",
    ),
];

const LAYOUT_FIELDS: [ContractPropSpec; 24] = [
    partial_prop(
        "display",
        ContractPropTypeKind::Enum,
        Some("layout_display"),
        false,
        "Optional layout mode override.",
        "Executed by the taffy-backed flow-container renderer used by row, column, inset, and card. `overlay` remains unsupported.",
    ),
    partial_prop(
        "direction",
        ContractPropTypeKind::Enum,
        Some("layout_direction"),
        false,
        "Optional row or column direction override.",
        "Only executed by the flow-container helpers used by row, column, inset, and card.",
    ),
    partial_prop(
        "grow",
        ContractPropTypeKind::Number,
        None,
        false,
        "Optional flex grow factor.",
        "Executed as child-item layout when the node is placed inside the taffy-backed flow-container renderer used by row, column, inset, and card.",
    ),
    partial_prop(
        "shrink",
        ContractPropTypeKind::Number,
        None,
        false,
        "Optional flex shrink factor.",
        "Executed as child-item layout when the node is placed inside the taffy-backed flow-container renderer used by row, column, inset, and card.",
    ),
    partial_prop(
        "basis",
        ContractPropTypeKind::Object,
        Some("layout_length"),
        false,
        "Optional flex basis length.",
        "Executed as child-item layout when the node is placed inside the taffy-backed flow-container renderer used by row, column, inset, and card.",
    ),
    supported_prop(
        "width",
        ContractPropTypeKind::Object,
        Some("layout_length"),
        false,
        "Optional width override.",
    ),
    supported_prop(
        "height",
        ContractPropTypeKind::Object,
        Some("layout_length"),
        false,
        "Optional height override.",
    ),
    supported_prop(
        "min_width",
        ContractPropTypeKind::Object,
        Some("layout_length"),
        false,
        "Optional minimum width override.",
    ),
    supported_prop(
        "min_height",
        ContractPropTypeKind::Object,
        Some("layout_length"),
        false,
        "Optional minimum height override.",
    ),
    supported_prop(
        "max_width",
        ContractPropTypeKind::Object,
        Some("layout_length"),
        false,
        "Optional maximum width override.",
    ),
    supported_prop(
        "max_height",
        ContractPropTypeKind::Object,
        Some("layout_length"),
        false,
        "Optional maximum height override.",
    ),
    partial_prop(
        "gap_x",
        ContractPropTypeKind::Number,
        None,
        false,
        "Optional horizontal gap override.",
        "Only executed by the flow-container helpers used by row, column, inset, and card.",
    ),
    partial_prop(
        "gap_y",
        ContractPropTypeKind::Number,
        None,
        false,
        "Optional vertical gap override.",
        "Only executed by the flow-container helpers used by row, column, inset, and card.",
    ),
    partial_prop(
        "padding",
        ContractPropTypeKind::Object,
        Some("layout_edges"),
        false,
        "Optional padding edges.",
        "Executed as an egui frame inner margin on every node. Percent class-derived padding is ignored because egui margins are pixel based.",
    ),
    supported_prop(
        "margin",
        ContractPropTypeKind::Object,
        Some("layout_edges"),
        false,
        "Optional margin edges.",
    ),
    partial_prop(
        "align",
        ContractPropTypeKind::Enum,
        Some("align"),
        false,
        "Optional cross-axis alignment override.",
        "Only executed by the flow-container helpers used by row, column, inset, and card.",
    ),
    partial_prop(
        "justify",
        ContractPropTypeKind::Enum,
        Some("justify"),
        false,
        "Optional main-axis alignment override.",
        "Only executed by the flow-container helpers used by row, column, inset, and card.",
    ),
    partial_prop(
        "wrap",
        ContractPropTypeKind::Boolean,
        None,
        false,
        "Optional wrap hint.",
        "Only executed by the flow-container helpers used by row, column, inset, and card. Wrap-reverse is not represented.",
    ),
    partial_prop(
        "columns",
        ContractPropTypeKind::ObjectList,
        Some("layout_track"),
        false,
        "Optional grid column tracks.",
        "Executed when the taffy-backed flow-container renderer used by row, column, inset, and card is switched to grid display.",
    ),
    partial_prop(
        "rows",
        ContractPropTypeKind::ObjectList,
        Some("layout_track"),
        false,
        "Optional grid row tracks.",
        "Executed when the taffy-backed flow-container renderer used by row, column, inset, and card is switched to grid display.",
    ),
    partial_prop(
        "col_span",
        ContractPropTypeKind::Number,
        None,
        false,
        "Optional grid column span.",
        "Executed as child-item grid placement when the parent uses the taffy-backed flow-container renderer in grid mode.",
    ),
    partial_prop(
        "row_span",
        ContractPropTypeKind::Number,
        None,
        false,
        "Optional grid row span.",
        "Executed as child-item grid placement when the parent uses the taffy-backed flow-container renderer in grid mode.",
    ),
    partial_prop(
        "overflow_x",
        ContractPropTypeKind::Enum,
        Some("layout_overflow"),
        false,
        "Optional horizontal overflow mode.",
        "Executed by the taffy-backed flow-container renderer used by row, column, inset, and card for `visible`, `hidden`, and `scroll`.",
    ),
    partial_prop(
        "overflow_y",
        ContractPropTypeKind::Enum,
        Some("layout_overflow"),
        false,
        "Optional vertical overflow mode.",
        "Executed by the taffy-backed flow-container renderer used by row, column, inset, and card for `visible`, `hidden`, and `scroll`.",
    ),
];

const LAYOUT_EDGES_FIELDS: [ContractPropSpec; 4] = [
    partial_prop(
        "top",
        ContractPropTypeKind::Number,
        None,
        false,
        "Top edge value.",
        "Executed for shared layout padding and margin. Values are rounded and clamped to egui's margin range.",
    ),
    partial_prop(
        "right",
        ContractPropTypeKind::Number,
        None,
        false,
        "Right edge value.",
        "Executed for shared layout padding and margin. Values are rounded and clamped to egui's margin range.",
    ),
    partial_prop(
        "bottom",
        ContractPropTypeKind::Number,
        None,
        false,
        "Bottom edge value.",
        "Executed for shared layout padding and margin. Values are rounded and clamped to egui's margin range.",
    ),
    partial_prop(
        "left",
        ContractPropTypeKind::Number,
        None,
        false,
        "Left edge value.",
        "Executed for shared layout padding and margin. Values are rounded and clamped to egui's margin range.",
    ),
];

const LAYOUT_LENGTH_FIELDS: [ContractPropSpec; 2] = [
    partial_prop(
        "kind",
        ContractPropTypeKind::Enum,
        Some("layout_length_kind"),
        true,
        "Length representation kind.",
        "The value shape is supported when referenced by width, height, min, and max layout fields. Other consumers such as basis are still unsupported.",
    ),
    partial_prop(
        "value",
        ContractPropTypeKind::Number,
        None,
        false,
        "Numeric payload for px and percent lengths.",
        "The value shape is supported when referenced by width, height, min, and max layout fields. Other consumers such as basis are still unsupported.",
    ),
];

const LAYOUT_TRACK_FIELDS: [ContractPropSpec; 2] = [
    unsupported_prop(
        "kind",
        ContractPropTypeKind::Enum,
        Some("layout_track_kind"),
        true,
        "Track representation kind.",
        "Grid tracks are declared in the schema only and are not executed by the current renderer.",
    ),
    unsupported_prop(
        "value",
        ContractPropTypeKind::Number,
        None,
        false,
        "Numeric payload for fr, px, and percent tracks.",
        "Grid tracks are declared in the schema only and are not executed by the current renderer.",
    ),
];

const ACTION_ITEM_FIELDS: [ContractPropSpec; 3] = [
    supported_prop(
        "item_id",
        ContractPropTypeKind::String,
        None,
        true,
        "Stable item identifier.",
    ),
    supported_prop(
        "label",
        ContractPropTypeKind::String,
        None,
        true,
        "Visible item label.",
    ),
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional item-level action id.",
    ),
];

const CHOICE_ITEM_FIELDS: [ContractPropSpec; 3] = ACTION_ITEM_FIELDS;

const TAB_ITEM_FIELDS: [ContractPropSpec; 5] = [
    supported_prop(
        "item_id",
        ContractPropTypeKind::String,
        None,
        true,
        "Stable tab identifier.",
    ),
    supported_prop(
        "label",
        ContractPropTypeKind::String,
        None,
        true,
        "Visible tab label.",
    ),
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional item-level action id.",
    ),
    supported_prop(
        "icon",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional icon name.",
    ),
    supported_prop(
        "icon_only",
        ContractPropTypeKind::Boolean,
        None,
        false,
        "Whether the tab should render without text.",
    ),
];

const MENU_ACTION_FIELDS: [ContractPropSpec; 5] = [
    supported_prop(
        "item_id",
        ContractPropTypeKind::String,
        None,
        true,
        "Stable menu action identifier.",
    ),
    supported_prop(
        "label",
        ContractPropTypeKind::String,
        None,
        true,
        "Visible action label.",
    ),
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional action id emitted back to the host.",
    ),
    supported_prop(
        "leading_icon",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional leading icon.",
    ),
    supported_prop(
        "shortcut",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional shortcut label.",
    ),
];

const MENU_ENTRY_FIELDS: [ContractPropSpec; 2] = [
    supported_prop(
        "kind",
        ContractPropTypeKind::Enum,
        Some("menu_entry_kind"),
        true,
        "Whether the entry is an action or separator.",
    ),
    supported_prop(
        "action",
        ContractPropTypeKind::Object,
        Some("menu_action"),
        false,
        "Action payload when kind is action.",
    ),
];

const MENU_FIELDS: [ContractPropSpec; 4] = [
    supported_prop(
        "menu_id",
        ContractPropTypeKind::String,
        None,
        true,
        "Stable top-level menu identifier.",
    ),
    supported_prop(
        "label",
        ContractPropTypeKind::String,
        None,
        true,
        "Visible top-level menu label.",
    ),
    supported_prop(
        "width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Popup width for the menu surface.",
    ),
    supported_prop(
        "entries",
        ContractPropTypeKind::ObjectList,
        Some("menu_entry"),
        true,
        "Action and separator rows for the menu.",
    ),
];

const HIERARCHY_ITEM_FIELDS: [ContractPropSpec; 7] = [
    supported_prop(
        "item_id",
        ContractPropTypeKind::String,
        None,
        true,
        "Stable hierarchy item identifier.",
    ),
    supported_prop(
        "label",
        ContractPropTypeKind::String,
        None,
        true,
        "Visible hierarchy label.",
    ),
    supported_prop(
        "kind",
        ContractPropTypeKind::Enum,
        Some("hierarchy_item_kind"),
        true,
        "Semantic item kind.",
    ),
    supported_prop(
        "open",
        ContractPropTypeKind::Boolean,
        None,
        false,
        "Whether children are expanded.",
    ),
    supported_prop(
        "locked",
        ContractPropTypeKind::Boolean,
        None,
        false,
        "Whether the row renders as locked.",
    ),
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional item-level action id.",
    ),
    supported_prop(
        "children",
        ContractPropTypeKind::ObjectList,
        Some("hierarchy_item"),
        false,
        "Nested hierarchy children.",
    ),
];

const TOAST_ITEM_FIELDS: [ContractPropSpec; 6] = [
    supported_prop(
        "item_id",
        ContractPropTypeKind::String,
        None,
        true,
        "Stable toast item identifier.",
    ),
    supported_prop(
        "title",
        ContractPropTypeKind::String,
        None,
        true,
        "Visible toast title.",
    ),
    supported_prop(
        "description",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional supporting copy.",
    ),
    supported_prop(
        "intent",
        ContractPropTypeKind::Enum,
        Some("toast_intent"),
        false,
        "Toast semantic intent.",
    ),
    supported_prop(
        "duration_secs",
        ContractPropTypeKind::Number,
        None,
        false,
        "Auto-dismiss duration in seconds. Zero keeps the toast open until dismissed.",
    ),
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional toast lifecycle action id.",
    ),
];

const LAYOUT_LENGTH_KIND_VARIANTS: [ContractVariantSpec; 3] = [
    ContractVariantSpec {
        id: "auto",
        label: "Auto",
        summary: "Automatic size chosen by the renderer.",
    },
    ContractVariantSpec {
        id: "px",
        label: "Px",
        summary: "Absolute pixel size.",
    },
    ContractVariantSpec {
        id: "percent",
        label: "Percent",
        summary: "Percentage of the available size.",
    },
];

const LAYOUT_TRACK_KIND_VARIANTS: [ContractVariantSpec; 4] = [
    ContractVariantSpec {
        id: "auto",
        label: "Auto",
        summary: "Automatic track size.",
    },
    ContractVariantSpec {
        id: "fr",
        label: "Fr",
        summary: "Fractional grid track size.",
    },
    ContractVariantSpec {
        id: "px",
        label: "Px",
        summary: "Absolute pixel track size.",
    },
    ContractVariantSpec {
        id: "percent",
        label: "Percent",
        summary: "Percentage-based track size.",
    },
];

const LAYOUT_DISPLAY_VARIANTS: [ContractVariantSpec; 4] = [
    ContractVariantSpec {
        id: "flow",
        label: "Flow",
        summary: "Default flow layout.",
    },
    ContractVariantSpec {
        id: "flex",
        label: "Flex",
        summary: "Flex-style layout hint.",
    },
    ContractVariantSpec {
        id: "grid",
        label: "Grid",
        summary: "Grid-style layout hint.",
    },
    ContractVariantSpec {
        id: "overlay",
        label: "Overlay",
        summary: "Overlay-style layout hint.",
    },
];

const LAYOUT_DIRECTION_VARIANTS: [ContractVariantSpec; 2] = [
    ContractVariantSpec {
        id: "row",
        label: "Row",
        summary: "Lay out children horizontally.",
    },
    ContractVariantSpec {
        id: "column",
        label: "Column",
        summary: "Lay out children vertically.",
    },
];

const LAYOUT_OVERFLOW_VARIANTS: [ContractVariantSpec; 3] = [
    ContractVariantSpec {
        id: "visible",
        label: "Visible",
        summary: "Allow content to remain visible outside the box.",
    },
    ContractVariantSpec {
        id: "hidden",
        label: "Hidden",
        summary: "Clip overflowing content.",
    },
    ContractVariantSpec {
        id: "scroll",
        label: "Scroll",
        summary: "Show scrollable overflow behavior.",
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

const ALIGN_VARIANTS: [ContractVariantSpec; 4] = [
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
    ContractVariantSpec {
        id: "stretch",
        label: "Stretch",
        summary: "Stretch content along the cross axis when the egui container can justify it.",
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
    supported_prop(
        "gap",
        ContractPropTypeKind::Number,
        None,
        false,
        "Horizontal gap between child nodes.",
    ),
    supported_prop(
        "justify",
        ContractPropTypeKind::Enum,
        Some("justify"),
        false,
        "Main-axis alignment.",
    ),
    supported_prop(
        "align",
        ContractPropTypeKind::Enum,
        Some("align"),
        false,
        "Cross-axis alignment.",
    ),
    supported_prop(
        "children",
        ContractPropTypeKind::NodeList,
        None,
        true,
        "Nested contract child nodes.",
    ),
];

const COLUMN_PROPS: [ContractPropSpec; 4] = [
    supported_prop(
        "gap",
        ContractPropTypeKind::Number,
        None,
        false,
        "Vertical gap between child nodes.",
    ),
    supported_prop(
        "justify",
        ContractPropTypeKind::Enum,
        Some("justify"),
        false,
        "Main-axis alignment.",
    ),
    supported_prop(
        "align",
        ContractPropTypeKind::Enum,
        Some("align"),
        false,
        "Cross-axis alignment.",
    ),
    supported_prop(
        "children",
        ContractPropTypeKind::NodeList,
        None,
        true,
        "Nested contract child nodes.",
    ),
];

const INSET_PROPS: [ContractPropSpec; 3] = [
    supported_prop(
        "padding_x",
        ContractPropTypeKind::Number,
        None,
        false,
        "Horizontal inset padding.",
    ),
    supported_prop(
        "padding_y",
        ContractPropTypeKind::Number,
        None,
        false,
        "Vertical inset padding.",
    ),
    supported_prop(
        "children",
        ContractPropTypeKind::NodeList,
        None,
        true,
        "Nested contract child nodes.",
    ),
];

const SIZED_BOX_PROPS: [ContractPropSpec; 3] = [
    supported_prop(
        "width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Optional explicit width.",
    ),
    supported_prop(
        "height",
        ContractPropTypeKind::Number,
        None,
        false,
        "Optional explicit height.",
    ),
    supported_prop(
        "children",
        ContractPropTypeKind::NodeList,
        None,
        true,
        "Nested contract child nodes.",
    ),
];

const SPACER_PROPS: [ContractPropSpec; 3] = [
    supported_prop(
        "width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Optional fixed spacer width.",
    ),
    supported_prop(
        "height",
        ContractPropTypeKind::Number,
        None,
        false,
        "Optional fixed spacer height.",
    ),
    supported_prop(
        "flex",
        ContractPropTypeKind::Boolean,
        None,
        false,
        "Whether the spacer should fill remaining space.",
    ),
];

const CARD_PROPS: [ContractPropSpec; 3] = [
    supported_prop(
        "padding_x",
        ContractPropTypeKind::Number,
        None,
        false,
        "Horizontal card padding.",
    ),
    supported_prop(
        "padding_y",
        ContractPropTypeKind::Number,
        None,
        false,
        "Vertical card padding.",
    ),
    supported_prop(
        "children",
        ContractPropTypeKind::NodeList,
        None,
        true,
        "Nested contract child nodes.",
    ),
];

const SIDEBAR_PROPS: [ContractPropSpec; 5] = [
    supported_prop(
        "title",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional sidebar title.",
    ),
    supported_prop(
        "side",
        ContractPropTypeKind::Enum,
        Some("sidebar_side"),
        false,
        "Docking side.",
    ),
    supported_prop(
        "width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Sidebar width.",
    ),
    supported_prop(
        "open",
        ContractPropTypeKind::Boolean,
        None,
        false,
        "Whether the sidebar is currently open.",
    ),
    supported_prop(
        "children",
        ContractPropTypeKind::NodeList,
        None,
        true,
        "Sidebar body children.",
    ),
];

const TOOLBAR_PROPS: [ContractPropSpec; 4] = [
    supported_prop(
        "anchor",
        ContractPropTypeKind::Enum,
        Some("toolbar_anchor"),
        false,
        "Toolbar anchor point.",
    ),
    supported_prop(
        "offset_x",
        ContractPropTypeKind::Number,
        None,
        false,
        "Horizontal anchor offset.",
    ),
    supported_prop(
        "offset_y",
        ContractPropTypeKind::Number,
        None,
        false,
        "Vertical anchor offset.",
    ),
    supported_prop(
        "children",
        ContractPropTypeKind::NodeList,
        None,
        true,
        "Toolbar child nodes.",
    ),
];

const MENU_BAR_PROPS: [ContractPropSpec; 1] = [supported_prop(
    "menus",
    ContractPropTypeKind::ObjectList,
    Some("menu"),
    true,
    "Top-level menus and entries.",
)];

const TABS_PROPS: [ContractPropSpec; 3] = [
    supported_prop(
        "style",
        ContractPropTypeKind::Enum,
        Some("tabs_style"),
        false,
        "Visual tab presentation.",
    ),
    supported_prop(
        "selected_item_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Currently selected tab id.",
    ),
    supported_prop(
        "items",
        ContractPropTypeKind::ObjectList,
        Some("tab_item"),
        true,
        "Tab items.",
    ),
];

const LABEL_PROPS: [ContractPropSpec; 4] = [
    supported_prop(
        "text",
        ContractPropTypeKind::String,
        None,
        true,
        "Visible label text.",
    ),
    supported_prop(
        "tone",
        ContractPropTypeKind::Enum,
        Some("label_tone"),
        false,
        "Semantic text tone.",
    ),
    supported_prop(
        "weight",
        ContractPropTypeKind::Enum,
        Some("label_weight"),
        false,
        "Semantic text weight.",
    ),
    supported_prop(
        "size",
        ContractPropTypeKind::Number,
        None,
        false,
        "Optional text size override.",
    ),
];

const BUTTON_PROPS: [ContractPropSpec; 9] = [
    supported_prop(
        "label",
        ContractPropTypeKind::String,
        None,
        true,
        "Visible button label.",
    ),
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional action id emitted on click.",
    ),
    supported_prop(
        "variant",
        ContractPropTypeKind::Enum,
        Some("button_variant"),
        false,
        "Button visual variant.",
    ),
    supported_prop(
        "size",
        ContractPropTypeKind::Enum,
        Some("control_size"),
        false,
        "Button size token.",
    ),
    supported_prop(
        "leading_icon",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional leading icon.",
    ),
    supported_prop(
        "trailing_text",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional trailing helper text.",
    ),
    supported_prop(
        "trailing_icon",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional trailing icon.",
    ),
    supported_prop(
        "icon_only",
        ContractPropTypeKind::Boolean,
        None,
        false,
        "Whether the button should render as icon-only.",
    ),
    supported_prop(
        "selected",
        ContractPropTypeKind::Boolean,
        None,
        false,
        "Whether the button should render in a selected state.",
    ),
];

const BUTTON_GROUP_PROPS: [ContractPropSpec; 1] = [supported_prop(
    "items",
    ContractPropTypeKind::ObjectList,
    Some("action_item"),
    true,
    "Action items for the group.",
)];

const INPUT_PROPS: [ContractPropSpec; 5] = [
    supported_prop(
        "value",
        ContractPropTypeKind::String,
        None,
        true,
        "Current text value.",
    ),
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional input action id.",
    ),
    supported_prop(
        "placeholder",
        ContractPropTypeKind::String,
        None,
        false,
        "Placeholder copy.",
    ),
    supported_prop(
        "leading_icon",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional leading icon.",
    ),
    supported_prop(
        "width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Requested control width.",
    ),
];

const NUMBER_INPUT_PROPS: [ContractPropSpec; 10] = [
    supported_prop(
        "value",
        ContractPropTypeKind::Number,
        None,
        true,
        "Current numeric value.",
    ),
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional action id.",
    ),
    supported_prop(
        "width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Requested control width.",
    ),
    supported_prop(
        "min",
        ContractPropTypeKind::Number,
        None,
        false,
        "Inclusive numeric minimum.",
    ),
    supported_prop(
        "max",
        ContractPropTypeKind::Number,
        None,
        false,
        "Inclusive numeric maximum.",
    ),
    supported_prop(
        "speed",
        ContractPropTypeKind::Number,
        None,
        false,
        "Drag speed.",
    ),
    supported_prop(
        "decimals",
        ContractPropTypeKind::Number,
        None,
        false,
        "Displayed decimal precision.",
    ),
    supported_prop(
        "prefix",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional leading unit text.",
    ),
    supported_prop(
        "suffix",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional trailing unit text.",
    ),
    supported_prop(
        "axis",
        ContractPropTypeKind::Enum,
        Some("number_input_axis"),
        false,
        "Primary drag axis.",
    ),
];

const CHECKBOX_PROPS: [ContractPropSpec; 3] = [
    supported_prop(
        "value",
        ContractPropTypeKind::Boolean,
        None,
        true,
        "Current checked value.",
    ),
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional action id.",
    ),
    supported_prop(
        "label",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional checkbox label.",
    ),
];

const SWITCH_PROPS: [ContractPropSpec; 4] = [
    supported_prop(
        "value",
        ContractPropTypeKind::Boolean,
        None,
        true,
        "Current switch value.",
    ),
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional action id.",
    ),
    supported_prop(
        "label",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional switch label.",
    ),
    supported_prop(
        "size",
        ContractPropTypeKind::Enum,
        Some("control_size"),
        false,
        "Switch size token.",
    ),
];

const SELECT_PROPS: [ContractPropSpec; 7] = [
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional select action id.",
    ),
    supported_prop(
        "selected_item_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Currently selected item id.",
    ),
    supported_prop(
        "placeholder",
        ContractPropTypeKind::String,
        None,
        false,
        "Placeholder copy.",
    ),
    supported_prop(
        "width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Requested trigger width.",
    ),
    supported_prop(
        "variant",
        ContractPropTypeKind::Enum,
        Some("select_variant"),
        false,
        "Trigger styling variant.",
    ),
    supported_prop(
        "leading_icon",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional leading icon.",
    ),
    supported_prop(
        "items",
        ContractPropTypeKind::ObjectList,
        Some("choice_item"),
        true,
        "Selectable items.",
    ),
];

const FIELD_PROPS: [ContractPropSpec; 6] = [
    supported_prop(
        "label",
        ContractPropTypeKind::String,
        None,
        true,
        "Field label.",
    ),
    supported_prop(
        "value",
        ContractPropTypeKind::String,
        None,
        true,
        "Current text value.",
    ),
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional field action id.",
    ),
    supported_prop(
        "helper_text",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional helper copy.",
    ),
    supported_prop(
        "placeholder",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional placeholder copy.",
    ),
    supported_prop(
        "width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Requested input width.",
    ),
];

const COLLAPSIBLE_PROPS: [ContractPropSpec; 5] = [
    supported_prop(
        "title",
        ContractPropTypeKind::String,
        None,
        true,
        "Section title.",
    ),
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional section action id.",
    ),
    supported_prop(
        "open",
        ContractPropTypeKind::Boolean,
        None,
        false,
        "Whether the body is currently expanded.",
    ),
    supported_prop(
        "leading_icon",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional leading icon.",
    ),
    supported_prop(
        "children",
        ContractPropTypeKind::NodeList,
        None,
        true,
        "Expandable body children.",
    ),
];

const DIALOGUE_PROPS: [ContractPropSpec; 10] = [
    supported_prop(
        "open",
        ContractPropTypeKind::Boolean,
        None,
        false,
        "Whether the modal is currently open.",
    ),
    supported_prop(
        "title",
        ContractPropTypeKind::String,
        None,
        true,
        "Modal title.",
    ),
    supported_prop(
        "description",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional modal description.",
    ),
    supported_prop(
        "confirm_label",
        ContractPropTypeKind::String,
        None,
        false,
        "Confirm button label.",
    ),
    supported_prop(
        "cancel_label",
        ContractPropTypeKind::String,
        None,
        false,
        "Cancel button label.",
    ),
    supported_prop(
        "intent",
        ContractPropTypeKind::Enum,
        Some("dialogue_intent"),
        false,
        "Modal semantic intent.",
    ),
    supported_prop(
        "width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Modal width.",
    ),
    supported_prop(
        "confirm_action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Confirm action id.",
    ),
    supported_prop(
        "cancel_action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Cancel action id.",
    ),
    supported_prop(
        "children",
        ContractPropTypeKind::NodeList,
        None,
        true,
        "Modal body children.",
    ),
];

const HIERARCHY_PROPS: [ContractPropSpec; 8] = [
    supported_prop(
        "action_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Optional hierarchy-level action id.",
    ),
    supported_prop(
        "selected_item_id",
        ContractPropTypeKind::String,
        None,
        false,
        "Currently selected item id.",
    ),
    supported_prop(
        "width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Requested hierarchy width.",
    ),
    supported_prop(
        "row_height",
        ContractPropTypeKind::Number,
        None,
        false,
        "Row height override.",
    ),
    supported_prop(
        "indent_width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Indent width override.",
    ),
    supported_prop(
        "icon_style",
        ContractPropTypeKind::Enum,
        Some("hierarchy_icon_style"),
        false,
        "Icon rendering style.",
    ),
    supported_prop(
        "style",
        ContractPropTypeKind::Enum,
        Some("hierarchy_style"),
        false,
        "Hierarchy surface style.",
    ),
    supported_prop(
        "items",
        ContractPropTypeKind::ObjectList,
        Some("hierarchy_item"),
        true,
        "Hierarchy tree items.",
    ),
];

const SPINNER_PROPS: [ContractPropSpec; 1] = [supported_prop(
    "size",
    ContractPropTypeKind::Number,
    None,
    false,
    "Spinner diameter.",
)];

const PROGRESS_PROPS: [ContractPropSpec; 3] = [
    supported_prop(
        "value",
        ContractPropTypeKind::Number,
        None,
        true,
        "Determinate progress value in the 0..=1 range.",
    ),
    supported_prop(
        "width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Progress bar width.",
    ),
    supported_prop(
        "height",
        ContractPropTypeKind::Number,
        None,
        false,
        "Progress bar height.",
    ),
];

const TOAST_VIEWPORT_PROPS: [ContractPropSpec; 8] = [
    supported_prop(
        "placement",
        ContractPropTypeKind::Enum,
        Some("toast_placement"),
        false,
        "Viewport anchor placement.",
    ),
    supported_prop(
        "width",
        ContractPropTypeKind::Number,
        None,
        false,
        "Requested toast width.",
    ),
    supported_prop(
        "margin_x",
        ContractPropTypeKind::Number,
        None,
        false,
        "Horizontal viewport margin.",
    ),
    supported_prop(
        "margin_y",
        ContractPropTypeKind::Number,
        None,
        false,
        "Vertical viewport margin.",
    ),
    supported_prop(
        "gap",
        ContractPropTypeKind::Number,
        None,
        false,
        "Gap between toast rows.",
    ),
    supported_prop(
        "overlap",
        ContractPropTypeKind::Number,
        None,
        false,
        "Optional stacked overlap between visible toasts.",
    ),
    supported_prop(
        "max_visible",
        ContractPropTypeKind::Number,
        None,
        false,
        "Maximum number of visible toasts at once.",
    ),
    supported_prop(
        "toasts",
        ContractPropTypeKind::ObjectList,
        Some("toast_item"),
        true,
        "Host-owned toast items.",
    ),
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

const FAMILIES: &[ContractFamilySpec] = &[
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
    ContractFamilySpec {
        id: ContractFamilyId::Color,
        display_name: "Color",
        summary: "Color swatch primitive.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Icon,
        display_name: "Icon",
        summary: "Icon glyph primitive.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Image,
        display_name: "Image",
        summary: "Raster image primitive.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Twemoji,
        display_name: "Twemoji",
        summary: "Twemoji image primitive.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Kbd,
        display_name: "Kbd",
        summary: "Keyboard keycap primitive.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Skeleton,
        display_name: "Skeleton",
        summary: "Animated placeholder primitive.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Slider,
        display_name: "Slider",
        summary: "Numeric range slider.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_CHANGED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Radio,
        display_name: "Radio",
        summary: "Single boolean radio option.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_TOGGLED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::RadioGroup,
        display_name: "Radio Group",
        summary: "Mutually exclusive radio options.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_SELECTED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Combobox,
        display_name: "Combobox",
        summary: "Filterable multi-select picker.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_SELECTED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::EmojiSelector,
        display_name: "Emoji Selector",
        summary: "Button-triggered emoji picker.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_SELECTED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Pagination,
        display_name: "Pagination",
        summary: "Page-number navigation.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_SELECTED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Tooltip,
        display_name: "Tooltip",
        summary: "Hover-triggered helper content.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Popover,
        display_name: "Popover",
        summary: "Click-triggered popup surface.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Children,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &COLLAPSIBLE_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::DropdownMenu,
        display_name: "Dropdown Menu",
        summary: "Button-triggered action menu.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_COMMAND,
    },
    ContractFamilySpec {
        id: ContractFamilyId::ContextMenu,
        display_name: "Context Menu",
        summary: "Right-click action menu surface.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Children,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_COMMAND,
    },
    ContractFamilySpec {
        id: ContractFamilyId::OpenWith,
        display_name: "Open With",
        summary: "Split current-editor picker.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_COMMAND,
    },
    ContractFamilySpec {
        id: ContractFamilyId::CollabCursor,
        display_name: "Collab Cursor",
        summary: "Presence cursor and name badge.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EMPTY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::IconToolbar,
        display_name: "Icon Toolbar",
        summary: "Icon-first selection toolbar.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_SELECTED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::FileTree,
        display_name: "File Tree",
        summary: "Compact file explorer tree.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &HIERARCHY_EVENTS,
    },
    ContractFamilySpec {
        id: ContractFamilyId::DragBoard,
        display_name: "Drag Board",
        summary: "Two-region drag-and-drop board.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_CHANGED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::AudioPlayback,
        display_name: "Audio Playback",
        summary: "Playback row with optional trailing actions.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Children,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_TOGGLED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::ImageTile,
        display_name: "Image Tile",
        summary: "Media tile with optional body and playback state.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::Children,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_CLICKED,
    },
    ContractFamilySpec {
        id: ContractFamilyId::Command,
        display_name: "Command",
        summary: "Searchable command list.",
        includes_common_props: true,
        child_policy: ContractChildPolicy::None,
        props: &EMPTY_PROPS,
        variants: &EMPTY_VARIANT_REFS,
        events: &EVENT_CHANGED,
    },
];

const SHARED_TYPES: [ContractSharedTypeSpec; 37] = [
    partial_shared_type(
        "node_common",
        "Node Common",
        ContractSharedTypeKind::Object,
        "Common fields flattened into every node.",
        "Visible, enabled, class, class_list, and part of layout are executed today. Slot and common action fields are metadata-only.",
        &NODE_COMMON_FIELDS,
        &EMPTY_VARIANTS,
    ),
    unsupported_shared_type(
        "actions",
        "Actions",
        ContractSharedTypeKind::Object,
        "Optional common semantic action bindings.",
        "Declared in the schema only. The current renderer uses family-specific action fields instead.",
        &ACTIONS_FIELDS,
        &EMPTY_VARIANTS,
    ),
    partial_shared_type(
        "layout",
        "Layout",
        ContractSharedTypeKind::Object,
        "Shared layout hints available on every node.",
        "Sizing, padding, and margin are executed on every node. Display, direction, gap, justify, align, wrap, grid tracks, and overflow are executed by the taffy-backed flow-container renderer used by row, column, inset, and card.",
        &LAYOUT_FIELDS,
        &EMPTY_VARIANTS,
    ),
    partial_shared_type(
        "layout_edges",
        "Layout Edges",
        ContractSharedTypeKind::Object,
        "Top, right, bottom, and left edge values for shared layout padding and margin.",
        "Executed for padding and margin with egui margin rounding and clamping.",
        &LAYOUT_EDGES_FIELDS,
        &EMPTY_VARIANTS,
    ),
    partial_shared_type(
        "layout_length",
        "Layout Length",
        ContractSharedTypeKind::Object,
        "Tagged length value used by shared layout sizing fields.",
        "Supported when referenced from width, height, min, max, and taffy-backed flex basis layout fields.",
        &LAYOUT_LENGTH_FIELDS,
        &EMPTY_VARIANTS,
    ),
    supported_shared_type(
        "layout_length_kind",
        "Layout Length Kind",
        ContractSharedTypeKind::Enum,
        "Supported layout length kinds.",
        &EMPTY_PROPS,
        &LAYOUT_LENGTH_KIND_VARIANTS,
    ),
    partial_shared_type(
        "layout_track",
        "Layout Track",
        ContractSharedTypeKind::Object,
        "Tagged grid track value for declared column and row tracks.",
        "Executed when the taffy-backed flow-container renderer used by row, column, inset, and card is switched to grid display.",
        &LAYOUT_TRACK_FIELDS,
        &EMPTY_VARIANTS,
    ),
    supported_shared_type(
        "layout_track_kind",
        "Layout Track Kind",
        ContractSharedTypeKind::Enum,
        "Declared layout track kinds.",
        &EMPTY_PROPS,
        &LAYOUT_TRACK_KIND_VARIANTS,
    ),
    partial_shared_type(
        "layout_display",
        "Layout Display",
        ContractSharedTypeKind::Enum,
        "Declared layout display modes.",
        "Executed by the taffy-backed flow-container renderer used by row, column, inset, and card. `overlay` remains unsupported.",
        &EMPTY_PROPS,
        &LAYOUT_DISPLAY_VARIANTS,
    ),
    partial_shared_type(
        "layout_direction",
        "Layout Direction",
        ContractSharedTypeKind::Enum,
        "Row and column direction values for shared layout hints.",
        "Only executed by the current flow-container helpers used by row, column, inset, and card.",
        &EMPTY_PROPS,
        &LAYOUT_DIRECTION_VARIANTS,
    ),
    partial_shared_type(
        "layout_overflow",
        "Layout Overflow",
        ContractSharedTypeKind::Enum,
        "Declared overflow modes for shared layout hints.",
        "Executed by the taffy-backed flow-container renderer used by row, column, inset, and card for `visible`, `hidden`, and `scroll`.",
        &EMPTY_PROPS,
        &LAYOUT_OVERFLOW_VARIANTS,
    ),
    supported_shared_type("justify", "Justify", ContractSharedTypeKind::Enum, "Main-axis alignment values.", &EMPTY_PROPS, &JUSTIFY_VARIANTS),
    supported_shared_type("align", "Align", ContractSharedTypeKind::Enum, "Cross-axis alignment values.", &EMPTY_PROPS, &ALIGN_VARIANTS),
    supported_shared_type("toolbar_anchor", "Toolbar Anchor", ContractSharedTypeKind::Enum, "Supported toolbar anchor points.", &EMPTY_PROPS, &TOOLBAR_ANCHOR_VARIANTS),
    supported_shared_type("tabs_style", "Tabs Style", ContractSharedTypeKind::Enum, "Supported tab presentations.", &EMPTY_PROPS, &TABS_STYLE_VARIANTS),
    supported_shared_type("button_variant", "Button Variant", ContractSharedTypeKind::Enum, "Supported button variants.", &EMPTY_PROPS, &BUTTON_VARIANTS),
    supported_shared_type("control_size", "Control Size", ContractSharedTypeKind::Enum, "Supported control sizes.", &EMPTY_PROPS, &CONTROL_SIZE_VARIANTS),
    supported_shared_type("label_tone", "Label Tone", ContractSharedTypeKind::Enum, "Supported text tones.", &EMPTY_PROPS, &LABEL_TONE_VARIANTS),
    supported_shared_type("label_weight", "Label Weight", ContractSharedTypeKind::Enum, "Supported text weights.", &EMPTY_PROPS, &LABEL_WEIGHT_VARIANTS),
    supported_shared_type("select_variant", "Select Variant", ContractSharedTypeKind::Enum, "Supported select trigger variants.", &EMPTY_PROPS, &SELECT_VARIANTS),
    supported_shared_type("sidebar_side", "Sidebar Side", ContractSharedTypeKind::Enum, "Supported sidebar docking sides.", &EMPTY_PROPS, &SIDEBAR_SIDE_VARIANTS),
    supported_shared_type("dialogue_intent", "Dialogue Intent", ContractSharedTypeKind::Enum, "Supported dialogue intents.", &EMPTY_PROPS, &DIALOGUE_INTENT_VARIANTS),
    supported_shared_type("toast_intent", "Toast Intent", ContractSharedTypeKind::Enum, "Supported toast intents.", &EMPTY_PROPS, &TOAST_INTENT_VARIANTS),
    supported_shared_type("toast_placement", "Toast Placement", ContractSharedTypeKind::Enum, "Supported toast viewport placements.", &EMPTY_PROPS, &TOAST_PLACEMENT_VARIANTS),
    supported_shared_type("number_input_axis", "Number Input Axis", ContractSharedTypeKind::Enum, "Supported drag axes for number input.", &EMPTY_PROPS, &NUMBER_INPUT_AXIS_VARIANTS),
    supported_shared_type("hierarchy_item_kind", "Hierarchy Item Kind", ContractSharedTypeKind::Enum, "Supported hierarchy semantic item kinds.", &EMPTY_PROPS, &HIERARCHY_KIND_VARIANTS),
    supported_shared_type("menu_entry_kind", "Menu Entry Kind", ContractSharedTypeKind::Enum, "Supported menu entry kinds.", &EMPTY_PROPS, &MENU_ENTRY_KIND_VARIANTS),
    supported_shared_type("hierarchy_icon_style", "Hierarchy Icon Style", ContractSharedTypeKind::Enum, "Supported hierarchy icon styles.", &EMPTY_PROPS, &HIERARCHY_ICON_STYLE_VARIANTS),
    supported_shared_type("hierarchy_style", "Hierarchy Style", ContractSharedTypeKind::Enum, "Supported hierarchy surface styles.", &EMPTY_PROPS, &HIERARCHY_STYLE_VARIANTS),
    supported_shared_type("action_item", "Action Item", ContractSharedTypeKind::Object, "Button-group style action item.", &ACTION_ITEM_FIELDS, &EMPTY_VARIANTS),
    supported_shared_type("choice_item", "Choice Item", ContractSharedTypeKind::Object, "Select choice item.", &CHOICE_ITEM_FIELDS, &EMPTY_VARIANTS),
    supported_shared_type("tab_item", "Tab Item", ContractSharedTypeKind::Object, "Tab navigation item.", &TAB_ITEM_FIELDS, &EMPTY_VARIANTS),
    supported_shared_type("menu_action", "Menu Action", ContractSharedTypeKind::Object, "Clickable menu action row.", &MENU_ACTION_FIELDS, &EMPTY_VARIANTS),
    supported_shared_type("menu_entry", "Menu Entry", ContractSharedTypeKind::Object, "Action-or-separator menu entry union.", &MENU_ENTRY_FIELDS, &EMPTY_VARIANTS),
    supported_shared_type("menu", "Menu", ContractSharedTypeKind::Object, "Top-level menu bar menu.", &MENU_FIELDS, &EMPTY_VARIANTS),
    supported_shared_type("hierarchy_item", "Hierarchy Item", ContractSharedTypeKind::Object, "Recursive hierarchy tree item.", &HIERARCHY_ITEM_FIELDS, &EMPTY_VARIANTS),
    supported_shared_type("toast_item", "Toast Item", ContractSharedTypeKind::Object, "Toast lifecycle item.", &TOAST_ITEM_FIELDS, &EMPTY_VARIANTS),
];

pub fn registry() -> &'static [ContractFamilySpec] {
    FAMILIES
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
    let node_common = shared_type("node_common").expect("node_common shared type");
    let layout = shared_type("layout").expect("layout shared type");

    let mut markdown = String::from(
        "# Contract Reference\n\nGenerated from `egui_component::contract::registry()` and `egui_component::contract::shared_types()`.\n\nExport the schema and human-readable reference from Rust with:\n\n```rust\negui_component::contract::schema_json_pretty()\negui_component::contract::reference_markdown()\n```\n\n## Position In The Runtime\n\n`contract::*` is the active declarative render boundary for the JSX runtime and the remaining host-driven compatibility surfaces.\n\n- The `clay-jsx-egui-bridge` crate in `crates/clay-jsx-egui-bridge` lowers authored JSX/TSX into Rust-owned host nodes, then materializes this contract tree before Rust renders it.\n- `ContractTree` is useful when the runtime or a compatibility host wants a serializable declarative surface, schema tooling, or change-driven tree materialization.\n- Renderer ownership stays in Rust; the contract tree is the data boundary, not a separate JS renderer.\n\n## Shared Node Fields\n\n| Field | Type | Support | Summary |\n| --- | --- | --- | --- |\n",
    );

    for field in node_common.fields {
        markdown.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} |\n",
            field.name,
            prop_type_label(field),
            support_status_id(field.support),
            doc_summary(field.summary, field.support_summary),
        ));
    }

    markdown.push_str(
        "\n## Layout Support\n\n| Field | Type | Support | Summary |\n| --- | --- | --- | --- |\n",
    );
    for field in layout.fields {
        markdown.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} |\n",
            field.name,
            prop_type_label(field),
            support_status_id(field.support),
            doc_summary(field.summary, field.support_summary),
        ));
    }

    markdown.push_str("\n## Supported Families\n\n| Family | Child Policy | Primary Events | Summary |\n| --- | --- | --- | --- |\n");
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

    markdown.push_str(
        "\n## Shared Types\n\n| Type | Kind | Support | Summary |\n| --- | --- | --- | --- |\n",
    );
    for shared_type in shared_types() {
        markdown.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} |\n",
            shared_type.name,
            shared_type_kind_id(shared_type.kind),
            support_status_id(shared_type.support),
            doc_summary(shared_type.summary, shared_type.support_summary),
        ));
    }

    markdown.push_str(
        "\n## Contract Mode Integration Pattern\n\n1. Build a `ContractTree` from authoritative host state.\n2. Render it with `render_tree` or `render_component_tree`.\n3. Apply the returned `ContractEvent`s to host state.\n4. Rebuild the next frame's tree from the updated authoritative host state.\n",
    );

    markdown
}

fn shared_type(name: &str) -> Option<&'static ContractSharedTypeSpec> {
    shared_types()
        .iter()
        .find(|shared_type| shared_type.name == name)
}

fn child_policy_id(policy: ContractChildPolicy) -> &'static str {
    match policy {
        ContractChildPolicy::None => "none",
        ContractChildPolicy::Children => "children",
        ContractChildPolicy::Body => "body",
    }
}

fn prop_type_label(prop: &ContractPropSpec) -> String {
    match prop.kind {
        ContractPropTypeKind::String => String::from("string"),
        ContractPropTypeKind::StringList => String::from("list<string>"),
        ContractPropTypeKind::StringMap => String::from("map<string, string>"),
        ContractPropTypeKind::Boolean => String::from("boolean"),
        ContractPropTypeKind::Number => String::from("number"),
        ContractPropTypeKind::Enum => format!("enum:{}", prop.type_name.unwrap_or("unknown")),
        ContractPropTypeKind::Object => format!("object:{}", prop.type_name.unwrap_or("unknown")),
        ContractPropTypeKind::ObjectList => {
            format!("list<object:{}>", prop.type_name.unwrap_or("unknown"))
        }
        ContractPropTypeKind::Node => String::from("node"),
        ContractPropTypeKind::NodeList => String::from("list<node>"),
    }
}

fn support_status_id(status: ContractSupportStatus) -> &'static str {
    match status {
        ContractSupportStatus::Supported => "supported",
        ContractSupportStatus::Partial => "partial",
        ContractSupportStatus::Unsupported => "unsupported",
    }
}

fn doc_summary(summary: &str, support_summary: &str) -> String {
    if support_summary.is_empty() {
        summary.to_owned()
    } else {
        format!("{summary} {support_summary}")
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
    use super::{
        reference_markdown, registry, schema_json_pretty, shared_types, ContractSupportStatus,
    };

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
        assert!(json.contains("\"actions\""));
        assert!(json.contains("\"layout\""));
        assert!(json.contains("\"layout_overflow\""));
        assert!(json.contains("\"menu\""));
        assert!(json.contains("\"hierarchy_item\""));
        assert!(json.contains("\"toast_item\""));
        assert!(json.contains("\"support\""));
        assert!(json.contains("\"support_summary\""));
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
        assert!(markdown.contains("Position In The Runtime"));
        assert!(markdown.contains("Layout Support"));
        assert!(markdown.contains("`unsupported`"));
    }

    #[test]
    fn node_common_includes_contract_scaffolding_fields() {
        let node_common = shared_types()
            .iter()
            .find(|shared_type| shared_type.name == "node_common")
            .expect("node_common shared type");
        let fields = node_common
            .fields
            .iter()
            .map(|field| field.name)
            .collect::<Vec<_>>();

        assert_eq!(
            fields,
            vec![
                "node_id",
                "visible",
                "enabled",
                "class",
                "class_list",
                "slot_classes",
                "actions",
                "layout",
            ]
        );
        assert_eq!(node_common.support, ContractSupportStatus::Partial);

        let class = node_common
            .fields
            .iter()
            .find(|field| field.name == "class")
            .expect("class field");
        assert_eq!(class.support, ContractSupportStatus::Supported);
    }

    #[test]
    fn layout_support_matrix_matches_current_renderer_truth() {
        let layout = shared_types()
            .iter()
            .find(|shared_type| shared_type.name == "layout")
            .expect("layout shared type");

        let width = layout
            .fields
            .iter()
            .find(|field| field.name == "width")
            .expect("width field");
        assert_eq!(width.support, ContractSupportStatus::Supported);

        let direction = layout
            .fields
            .iter()
            .find(|field| field.name == "direction")
            .expect("direction field");
        assert_eq!(direction.support, ContractSupportStatus::Partial);

        let display = layout
            .fields
            .iter()
            .find(|field| field.name == "display")
            .expect("display field");
        assert_eq!(display.support, ContractSupportStatus::Partial);

        let padding = layout
            .fields
            .iter()
            .find(|field| field.name == "padding")
            .expect("padding field");
        assert_eq!(padding.support, ContractSupportStatus::Partial);

        let margin = layout
            .fields
            .iter()
            .find(|field| field.name == "margin")
            .expect("margin field");
        assert_eq!(margin.support, ContractSupportStatus::Supported);

        let columns = layout
            .fields
            .iter()
            .find(|field| field.name == "columns")
            .expect("columns field");
        assert_eq!(columns.support, ContractSupportStatus::Partial);

        let wrap = layout
            .fields
            .iter()
            .find(|field| field.name == "wrap")
            .expect("wrap field");
        assert_eq!(wrap.support, ContractSupportStatus::Partial);

        let overflow = layout
            .fields
            .iter()
            .find(|field| field.name == "overflow_y")
            .expect("overflow_y field");
        assert_eq!(overflow.support, ContractSupportStatus::Partial);
    }

    #[test]
    fn checked_in_contract_reference_is_current() {
        assert_eq!(
            reference_markdown(),
            include_str!("../../docs/llm/contract-reference.md")
        );
    }
}
