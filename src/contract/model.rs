use crate::components::{
    ButtonVariant, ControlSize, DialogueIntent, HierarchyIconStyle, HierarchyItemKind,
    HierarchyStyle, LabelTone, LabelWeight, NumberInputAxis, SelectVariant, SidebarSide,
    ToastIntent, ToastPlacement,
};
use std::{collections::BTreeMap, fmt};

pub const CONTRACT_MODEL_VERSION: u32 = 1;

#[derive(
    Debug,
    Clone,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(transparent)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for NodeId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for NodeId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for NodeId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(
    Debug,
    Clone,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(transparent)]
pub struct ActionId(pub String);

impl ActionId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for ActionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for ActionId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for ActionId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for ActionId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContractFamilyId {
    Row,
    Column,
    Inset,
    SizedBox,
    Spacer,
    Card,
    Sidebar,
    Toolbar,
    MenuBar,
    Tabs,
    Label,
    Button,
    ButtonGroup,
    Input,
    NumberInput,
    Checkbox,
    Switch,
    Select,
    Field,
    Separator,
    Collapsible,
    DialogueModal,
    Hierarchy,
    Spinner,
    Progress,
    ToastViewport,
}

impl ContractFamilyId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Column => "column",
            Self::Inset => "inset",
            Self::SizedBox => "sized-box",
            Self::Spacer => "spacer",
            Self::Card => "card",
            Self::Sidebar => "sidebar",
            Self::Toolbar => "toolbar",
            Self::MenuBar => "menu-bar",
            Self::Tabs => "tabs",
            Self::Label => "label",
            Self::Button => "button",
            Self::ButtonGroup => "button-group",
            Self::Input => "input",
            Self::NumberInput => "number-input",
            Self::Checkbox => "checkbox",
            Self::Switch => "switch",
            Self::Select => "select",
            Self::Field => "field",
            Self::Separator => "separator",
            Self::Collapsible => "collapsible",
            Self::DialogueModal => "dialogue-modal",
            Self::Hierarchy => "hierarchy",
            Self::Spinner => "spinner",
            Self::Progress => "progress",
            Self::ToastViewport => "toast-viewport",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractJustify {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractAnchor {
    TopLeft,
    #[default]
    TopCenter,
    TopRight,
    LeftCenter,
    Center,
    RightCenter,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractTabsStyle {
    #[default]
    Underline,
    Segmented,
    BlenderTopbar,
    Stacked,
    Rail,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractActions {
    #[serde(default)]
    pub click: Option<ActionId>,
    #[serde(default)]
    pub change: Option<ActionId>,
    #[serde(default)]
    pub submit: Option<ActionId>,
    #[serde(default)]
    pub select: Option<ActionId>,
    #[serde(default)]
    pub open: Option<ActionId>,
    #[serde(default)]
    pub close: Option<ActionId>,
    #[serde(default)]
    pub confirm: Option<ActionId>,
    #[serde(default)]
    pub cancel: Option<ActionId>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContractDisplay {
    Flow,
    Flex,
    Grid,
    Overlay,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContractDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContractOverflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractEdges {
    #[serde(default)]
    pub top: f32,
    #[serde(default)]
    pub right: f32,
    #[serde(default)]
    pub bottom: f32,
    #[serde(default)]
    pub left: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ContractLength {
    Auto,
    Px { value: f32 },
    Percent { value: f32 },
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ContractTrack {
    Auto,
    Fr { value: f32 },
    Px { value: f32 },
    Percent { value: f32 },
}

#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractLayout {
    #[serde(default)]
    pub display: Option<ContractDisplay>,
    #[serde(default)]
    pub direction: Option<ContractDirection>,
    #[serde(default)]
    pub grow: Option<f32>,
    #[serde(default)]
    pub shrink: Option<f32>,
    #[serde(default)]
    pub basis: Option<ContractLength>,
    #[serde(default)]
    pub width: Option<ContractLength>,
    #[serde(default)]
    pub height: Option<ContractLength>,
    #[serde(default)]
    pub min_width: Option<ContractLength>,
    #[serde(default)]
    pub min_height: Option<ContractLength>,
    #[serde(default)]
    pub max_width: Option<ContractLength>,
    #[serde(default)]
    pub max_height: Option<ContractLength>,
    #[serde(default)]
    pub gap_x: Option<f32>,
    #[serde(default)]
    pub gap_y: Option<f32>,
    #[serde(default)]
    pub padding: Option<ContractEdges>,
    #[serde(default)]
    pub margin: Option<ContractEdges>,
    #[serde(default)]
    pub align: Option<ContractAlign>,
    #[serde(default)]
    pub justify: Option<ContractJustify>,
    #[serde(default)]
    pub wrap: Option<bool>,
    #[serde(default)]
    pub columns: Vec<ContractTrack>,
    #[serde(default)]
    pub rows: Vec<ContractTrack>,
    #[serde(default)]
    pub col_span: Option<u16>,
    #[serde(default)]
    pub row_span: Option<u16>,
    #[serde(default)]
    pub overflow_x: Option<ContractOverflow>,
    #[serde(default)]
    pub overflow_y: Option<ContractOverflow>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractTree {
    #[serde(default = "contract_model_version")]
    pub version: u32,
    pub root: ContractNode,
}

impl ContractTree {
    pub fn new(root: ContractNode) -> Self {
        Self {
            version: CONTRACT_MODEL_VERSION,
            root,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractCommon {
    pub node_id: NodeId,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub class: Option<String>,
    #[serde(default)]
    pub class_list: Vec<String>,
    #[serde(default)]
    pub slot_classes: BTreeMap<String, String>,
    #[serde(default)]
    pub actions: ContractActions,
    #[serde(default)]
    pub layout: Option<ContractLayout>,
}

impl ContractCommon {
    pub fn new(node_id: impl Into<NodeId>) -> Self {
        Self {
            node_id: node_id.into(),
            visible: true,
            enabled: true,
            class: None,
            class_list: Vec::new(),
            slot_classes: BTreeMap::new(),
            actions: ContractActions::default(),
            layout: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "family", rename_all = "kebab-case")]
pub enum ContractNode {
    Row(ContractRow),
    Column(ContractColumn),
    Inset(ContractInset),
    SizedBox(ContractSizedBox),
    Spacer(ContractSpacer),
    Card(ContractCard),
    Sidebar(ContractSidebar),
    Toolbar(ContractToolbar),
    MenuBar(ContractMenuBar),
    Tabs(ContractTabs),
    Label(ContractLabel),
    Button(ContractButton),
    ButtonGroup(ContractButtonGroup),
    Input(ContractInput),
    NumberInput(ContractNumberInput),
    Checkbox(ContractCheckbox),
    Switch(ContractSwitch),
    Select(ContractSelect),
    Field(ContractField),
    Separator(ContractSeparator),
    Collapsible(ContractCollapsible),
    DialogueModal(ContractDialogueModal),
    Hierarchy(ContractHierarchy),
    Spinner(ContractSpinner),
    Progress(ContractProgress),
    ToastViewport(ContractToastViewport),
}

impl ContractNode {
    pub fn family_id(&self) -> ContractFamilyId {
        match self {
            Self::Row(_) => ContractFamilyId::Row,
            Self::Column(_) => ContractFamilyId::Column,
            Self::Inset(_) => ContractFamilyId::Inset,
            Self::SizedBox(_) => ContractFamilyId::SizedBox,
            Self::Spacer(_) => ContractFamilyId::Spacer,
            Self::Card(_) => ContractFamilyId::Card,
            Self::Sidebar(_) => ContractFamilyId::Sidebar,
            Self::Toolbar(_) => ContractFamilyId::Toolbar,
            Self::MenuBar(_) => ContractFamilyId::MenuBar,
            Self::Tabs(_) => ContractFamilyId::Tabs,
            Self::Label(_) => ContractFamilyId::Label,
            Self::Button(_) => ContractFamilyId::Button,
            Self::ButtonGroup(_) => ContractFamilyId::ButtonGroup,
            Self::Input(_) => ContractFamilyId::Input,
            Self::NumberInput(_) => ContractFamilyId::NumberInput,
            Self::Checkbox(_) => ContractFamilyId::Checkbox,
            Self::Switch(_) => ContractFamilyId::Switch,
            Self::Select(_) => ContractFamilyId::Select,
            Self::Field(_) => ContractFamilyId::Field,
            Self::Separator(_) => ContractFamilyId::Separator,
            Self::Collapsible(_) => ContractFamilyId::Collapsible,
            Self::DialogueModal(_) => ContractFamilyId::DialogueModal,
            Self::Hierarchy(_) => ContractFamilyId::Hierarchy,
            Self::Spinner(_) => ContractFamilyId::Spinner,
            Self::Progress(_) => ContractFamilyId::Progress,
            Self::ToastViewport(_) => ContractFamilyId::ToastViewport,
        }
    }

    pub fn common(&self) -> &ContractCommon {
        match self {
            Self::Row(node) => &node.common,
            Self::Column(node) => &node.common,
            Self::Inset(node) => &node.common,
            Self::SizedBox(node) => &node.common,
            Self::Spacer(node) => &node.common,
            Self::Card(node) => &node.common,
            Self::Sidebar(node) => &node.common,
            Self::Toolbar(node) => &node.common,
            Self::MenuBar(node) => &node.common,
            Self::Tabs(node) => &node.common,
            Self::Label(node) => &node.common,
            Self::Button(node) => &node.common,
            Self::ButtonGroup(node) => &node.common,
            Self::Input(node) => &node.common,
            Self::NumberInput(node) => &node.common,
            Self::Checkbox(node) => &node.common,
            Self::Switch(node) => &node.common,
            Self::Select(node) => &node.common,
            Self::Field(node) => &node.common,
            Self::Separator(node) => &node.common,
            Self::Collapsible(node) => &node.common,
            Self::DialogueModal(node) => &node.common,
            Self::Hierarchy(node) => &node.common,
            Self::Spinner(node) => &node.common,
            Self::Progress(node) => &node.common,
            Self::ToastViewport(node) => &node.common,
        }
    }

    pub fn node_id(&self) -> &NodeId {
        &self.common().node_id
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractRow {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default = "default_gap")]
    pub gap: f32,
    #[serde(default)]
    pub justify: ContractJustify,
    #[serde(default = "default_row_align")]
    pub align: ContractAlign,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractColumn {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default = "default_gap")]
    pub gap: f32,
    #[serde(default)]
    pub justify: ContractJustify,
    #[serde(default)]
    pub align: ContractAlign,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractInset {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default = "default_padding_x")]
    pub padding_x: u8,
    #[serde(default = "default_padding_y")]
    pub padding_y: u8,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractSizedBox {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractSpacer {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default = "default_true")]
    pub flex: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractCard {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default = "default_card_padding_x")]
    pub padding_x: u8,
    #[serde(default = "default_card_padding_y")]
    pub padding_y: u8,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractSidebar {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub side: SidebarSide,
    #[serde(default = "default_sidebar_width")]
    pub width: f32,
    #[serde(default)]
    pub open: bool,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractToolbar {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub anchor: ContractAnchor,
    #[serde(default)]
    pub offset_x: f32,
    #[serde(default = "default_toolbar_offset_y")]
    pub offset_y: f32,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractMenuBar {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub menus: Vec<ContractMenu>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractTabs {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub style: ContractTabsStyle,
    #[serde(default)]
    pub selected_item_id: Option<String>,
    #[serde(default)]
    pub items: Vec<ContractTabItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractLabel {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub text: String,
    #[serde(default)]
    pub tone: Option<LabelTone>,
    #[serde(default)]
    pub weight: Option<LabelWeight>,
    #[serde(default)]
    pub size: Option<f32>,
    #[serde(default)]
    pub truncate: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractButton {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub label: String,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub variant: Option<ButtonVariant>,
    #[serde(default)]
    pub size: Option<ControlSize>,
    #[serde(default)]
    pub leading_icon: Option<String>,
    #[serde(default)]
    pub trailing_text: Option<String>,
    #[serde(default)]
    pub trailing_icon: Option<String>,
    #[serde(default)]
    pub icon_only: bool,
    #[serde(default)]
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractButtonGroup {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub items: Vec<ContractActionItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractInput {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub value: String,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub leading_icon: Option<String>,
    #[serde(default = "default_input_width")]
    pub width: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractNumberInput {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub value: f32,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default = "default_number_input_width")]
    pub width: f32,
    #[serde(default = "default_number_min")]
    pub min: f32,
    #[serde(default = "default_number_max")]
    pub max: f32,
    #[serde(default = "default_number_speed")]
    pub speed: f64,
    #[serde(default = "default_number_decimals")]
    pub decimals: usize,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub suffix: Option<String>,
    #[serde(default)]
    pub axis: Option<NumberInputAxis>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractCheckbox {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub value: bool,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractSwitch {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub value: bool,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub size: Option<ControlSize>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractSelect {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub selected_item_id: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub variant: Option<SelectVariant>,
    #[serde(default)]
    pub leading_icon: Option<String>,
    #[serde(default)]
    pub items: Vec<ContractChoiceItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractField {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub helper_text: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default = "default_input_width")]
    pub width: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractSeparator {
    #[serde(flatten)]
    pub common: ContractCommon,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractCollapsible {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub title: String,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default = "default_true")]
    pub open: bool,
    #[serde(default)]
    pub leading_icon: Option<String>,
    #[serde(default)]
    pub trailing_icon: Option<String>,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractDialogueModal {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub open: bool,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub confirm_label: Option<String>,
    #[serde(default)]
    pub cancel_label: Option<String>,
    #[serde(default)]
    pub intent: Option<DialogueIntent>,
    #[serde(default = "default_dialogue_width")]
    pub width: f32,
    #[serde(default)]
    pub confirm_action_id: Option<ActionId>,
    #[serde(default)]
    pub cancel_action_id: Option<ActionId>,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractHierarchy {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub selected_item_id: Option<String>,
    #[serde(default = "default_hierarchy_width")]
    pub width: f32,
    #[serde(default = "default_hierarchy_row_height")]
    pub row_height: f32,
    #[serde(default = "default_hierarchy_indent_width")]
    pub indent_width: f32,
    #[serde(default)]
    pub icon_style: Option<HierarchyIconStyle>,
    #[serde(default)]
    pub style: Option<HierarchyStyle>,
    #[serde(default)]
    pub items: Vec<ContractHierarchyItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractSpinner {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default = "default_spinner_size")]
    pub size: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractProgress {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub value: f32,
    #[serde(default = "default_progress_width")]
    pub width: f32,
    #[serde(default = "default_progress_height")]
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractToastViewport {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub placement: ToastPlacement,
    #[serde(default = "default_toast_width")]
    pub width: f32,
    #[serde(default = "default_toast_margin_x")]
    pub margin_x: f32,
    #[serde(default = "default_toast_margin_y")]
    pub margin_y: f32,
    #[serde(default = "default_toast_gap")]
    pub gap: f32,
    #[serde(default)]
    pub overlap: f32,
    #[serde(default = "default_toast_max_visible")]
    pub max_visible: usize,
    #[serde(default)]
    pub toasts: Vec<ContractToastItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractActionItem {
    pub item_id: String,
    pub label: String,
    #[serde(default)]
    pub action_id: Option<ActionId>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractChoiceItem {
    pub item_id: String,
    pub label: String,
    #[serde(default)]
    pub action_id: Option<ActionId>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractTabItem {
    pub item_id: String,
    pub label: String,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub icon_only: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractMenu {
    pub menu_id: String,
    pub label: String,
    #[serde(default = "default_menu_width")]
    pub width: f32,
    #[serde(default)]
    pub entries: Vec<ContractMenuEntry>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContractMenuEntry {
    Action(ContractMenuAction),
    Separator,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractMenuAction {
    pub item_id: String,
    pub label: String,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub leading_icon: Option<String>,
    #[serde(default)]
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractHierarchyItem {
    pub item_id: String,
    pub label: String,
    pub kind: HierarchyItemKind,
    #[serde(default = "default_true")]
    pub open: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub children: Vec<ContractHierarchyItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractToastItem {
    pub item_id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub intent: ToastIntent,
    #[serde(default = "default_toast_duration_secs")]
    pub duration_secs: f32,
    #[serde(default)]
    pub action_id: Option<ActionId>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractEvent {
    pub node_id: NodeId,
    pub kind: EventKind,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub value: Option<EventValue>,
    #[serde(default)]
    pub metadata: Option<EventMetadata>,
}

impl ContractEvent {
    pub fn new(node_id: impl Into<NodeId>, kind: EventKind) -> Self {
        Self {
            node_id: node_id.into(),
            kind,
            action_id: None,
            value: None,
            metadata: None,
        }
    }

    pub fn action(mut self, action_id: Option<ActionId>) -> Self {
        self.action_id = action_id;
        self
    }

    pub fn value(mut self, value: Option<EventValue>) -> Self {
        self.value = value;
        self
    }

    pub fn metadata(mut self, metadata: Option<EventMetadata>) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Clicked,
    Changed,
    Submitted,
    Selected,
    Toggled,
    Confirmed,
    Cancelled,
    Opened,
    Closed,
    CommandInvoked,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum EventValue {
    Boolean(bool),
    Number(f32),
    Text(String),
    ItemId(String),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EventMetadata {
    #[serde(default)]
    pub item_id: Option<String>,
    #[serde(default)]
    pub item_label: Option<String>,
}

impl EventMetadata {
    pub fn item(item_id: impl Into<String>, item_label: impl Into<String>) -> Self {
        Self {
            item_id: Some(item_id.into()),
            item_label: Some(item_label.into()),
        }
    }
}

fn contract_model_version() -> u32 {
    CONTRACT_MODEL_VERSION
}

fn default_true() -> bool {
    true
}

fn default_gap() -> f32 {
    8.0
}

fn default_row_align() -> ContractAlign {
    ContractAlign::Center
}

fn default_padding_x() -> u8 {
    12
}

fn default_padding_y() -> u8 {
    10
}

fn default_card_padding_x() -> u8 {
    12
}

fn default_card_padding_y() -> u8 {
    12
}

fn default_sidebar_width() -> f32 {
    280.0
}

fn default_toolbar_offset_y() -> f32 {
    8.0
}

fn default_input_width() -> f32 {
    220.0
}

fn default_number_input_width() -> f32 {
    58.0
}

fn default_number_min() -> f32 {
    0.0
}

fn default_number_max() -> f32 {
    100.0
}

fn default_number_speed() -> f64 {
    0.2
}

fn default_number_decimals() -> usize {
    1
}

fn default_dialogue_width() -> f32 {
    360.0
}

fn default_hierarchy_width() -> f32 {
    320.0
}

fn default_hierarchy_row_height() -> f32 {
    32.0
}

fn default_hierarchy_indent_width() -> f32 {
    18.0
}

fn default_spinner_size() -> f32 {
    16.0
}

fn default_progress_width() -> f32 {
    188.0
}

fn default_progress_height() -> f32 {
    10.0
}

fn default_menu_width() -> f32 {
    208.0
}

fn default_toast_width() -> f32 {
    320.0
}

fn default_toast_margin_x() -> f32 {
    16.0
}

fn default_toast_margin_y() -> f32 {
    16.0
}

fn default_toast_gap() -> f32 {
    8.0
}

fn default_toast_max_visible() -> usize {
    4
}

fn default_toast_duration_secs() -> f32 {
    4.0
}

#[cfg(test)]
mod tests {
    use super::{
        ContractButton, ContractCommon, ContractDirection, ContractDisplay, ContractEvent,
        ContractLength, ContractNode, ContractTree, EventKind, EventValue,
    };

    #[test]
    fn contract_tree_round_trips_through_json() {
        let tree = ContractTree::new(ContractNode::Button(ContractButton {
            common: ContractCommon::new("contract.button"),
            label: "Save".to_owned(),
            action_id: Some("button.save".into()),
            variant: Some(crate::components::ButtonVariant::Primary),
            size: Some(crate::components::ControlSize::Md),
            leading_icon: Some("save".to_owned()),
            trailing_text: None,
            trailing_icon: None,
            icon_only: false,
            selected: false,
        }));

        let json = serde_json::to_string(&tree).expect("serialize contract tree");
        let decoded: ContractTree = serde_json::from_str(&json).expect("deserialize contract tree");

        assert_eq!(decoded, tree);
    }

    #[test]
    fn contract_event_serializes_typed_value_payloads() {
        let event = ContractEvent::new("contract.button", EventKind::Clicked)
            .action(Some("button.save".into()))
            .value(Some(EventValue::Text("Save".to_owned())))
            .metadata(None);

        let json = serde_json::to_string(&event).expect("serialize event");
        assert!(json.contains("\"clicked\""));
        assert!(json.contains("\"button.save\""));
        assert!(json.contains("\"Save\""));
    }

    #[test]
    fn contract_common_scaffolding_round_trips_class_actions_and_layout() {
        let json = r#"
        {
            "version": 1,
            "root": {
                "family": "label",
                "node_id": "contract.label",
                "text": "Hello",
                "class": "text-lg font-semibold",
                "class_list": ["text-lg", "font-semibold"],
                "slot_classes": {
                    "icon": "text-muted"
                },
                "actions": {
                    "click": "label.clicked"
                },
                "layout": {
                    "display": "flex",
                    "direction": "column",
                    "gap_y": 12.0,
                    "width": {
                        "kind": "px",
                        "value": 320.0
                    }
                }
            }
        }
        "#;

        let decoded: ContractTree = serde_json::from_str(json).expect("deserialize contract tree");

        match decoded.root {
            ContractNode::Label(label) => {
                assert_eq!(label.common.class.as_deref(), Some("text-lg font-semibold"));
                assert_eq!(
                    label.common.class_list,
                    vec!["text-lg".to_owned(), "font-semibold".to_owned()]
                );
                assert_eq!(
                    label.common.slot_classes.get("icon").map(String::as_str),
                    Some("text-muted")
                );
                assert_eq!(
                    label
                        .common
                        .actions
                        .click
                        .as_ref()
                        .map(|action| action.as_str()),
                    Some("label.clicked")
                );
                let layout = label.common.layout.expect("layout scaffolding");
                assert_eq!(layout.display, Some(ContractDisplay::Flex));
                assert_eq!(layout.direction, Some(ContractDirection::Column));
                assert_eq!(layout.gap_y, Some(12.0));
                assert_eq!(layout.width, Some(ContractLength::Px { value: 320.0 }));
            }
            node => panic!("expected label node, got {node:?}"),
        }
    }
}
