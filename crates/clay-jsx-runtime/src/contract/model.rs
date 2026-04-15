use crate::runtime_components::{
    AudioPlaybackState, ButtonVariant, ControlSize, DialogueIntent, DragBoardRegion,
    FileTreeItemKind, HierarchyIconStyle, HierarchyItemKind, HierarchyStyle, LabelTone,
    LabelWeight, NumberInputAxis, PopoverAlign, PopoverSide, SelectVariant, SidebarSide,
    ToastIntent, ToastPlacement, TooltipPlacement,
};
use std::{collections::BTreeMap, fmt};

pub const CONTRACT_MODEL_VERSION: u32 = 2;

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
    Color,
    Icon,
    Image,
    Twemoji,
    Kbd,
    Skeleton,
    Slider,
    Radio,
    RadioGroup,
    Combobox,
    EmojiSelector,
    Pagination,
    Tooltip,
    Popover,
    DropdownMenu,
    ContextMenu,
    OpenWith,
    CollabCursor,
    IconToolbar,
    FileTree,
    DragBoard,
    AudioPlayback,
    ImageTile,
    Command,
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
            Self::Color => "color",
            Self::Icon => "icon",
            Self::Image => "image",
            Self::Twemoji => "twemoji",
            Self::Kbd => "kbd",
            Self::Skeleton => "skeleton",
            Self::Slider => "slider",
            Self::Radio => "radio",
            Self::RadioGroup => "radio-group",
            Self::Combobox => "combobox",
            Self::EmojiSelector => "emoji-selector",
            Self::Pagination => "pagination",
            Self::Tooltip => "tooltip",
            Self::Popover => "popover",
            Self::DropdownMenu => "dropdown-menu",
            Self::ContextMenu => "context-menu",
            Self::OpenWith => "open-with",
            Self::CollabCursor => "collab-cursor",
            Self::IconToolbar => "icon-toolbar",
            Self::FileTree => "file-tree",
            Self::DragBoard => "drag-board",
            Self::AudioPlayback => "audio-playback",
            Self::ImageTile => "image-tile",
            Self::Command => "command",
        }
    }

    pub fn is_identity_sensitive(self) -> bool {
        matches!(
            self,
            Self::Sidebar
                | Self::MenuBar
                | Self::Tabs
                | Self::Input
                | Self::NumberInput
                | Self::Checkbox
                | Self::Switch
                | Self::Select
                | Self::Field
                | Self::Collapsible
                | Self::DialogueModal
                | Self::Hierarchy
                | Self::ToastViewport
                | Self::Slider
                | Self::Radio
                | Self::RadioGroup
                | Self::Combobox
                | Self::EmojiSelector
                | Self::Popover
                | Self::DropdownMenu
                | Self::ContextMenu
                | Self::OpenWith
                | Self::FileTree
                | Self::DragBoard
                | Self::Command
        )
    }

    pub fn allows_fallback_node_id(self) -> bool {
        !self.is_identity_sensitive()
    }

    pub fn requires_explicit_node_id(self) -> bool {
        !self.allows_fallback_node_id()
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
    Stretch,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ImageTileSize {
    Sm,
    Md,
    Lg,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ImageTilePlaybackState {
    Paused,
    Playing,
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
    Color(ContractColor),
    Icon(ContractIcon),
    Image(ContractImage),
    Twemoji(ContractTwemoji),
    Kbd(ContractKbd),
    Skeleton(ContractSkeleton),
    Slider(ContractSlider),
    Radio(ContractRadio),
    RadioGroup(ContractRadioGroup),
    Combobox(ContractCombobox),
    EmojiSelector(ContractEmojiSelector),
    Pagination(ContractPagination),
    Tooltip(ContractTooltip),
    Popover(ContractPopover),
    DropdownMenu(ContractDropdownMenu),
    ContextMenu(ContractContextMenu),
    OpenWith(ContractOpenWith),
    CollabCursor(ContractCollabCursor),
    IconToolbar(ContractIconToolbar),
    FileTree(ContractFileTree),
    DragBoard(ContractDragBoard),
    AudioPlayback(ContractAudioPlayback),
    ImageTile(ContractImageTile),
    Command(ContractCommand),
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
            Self::Color(_) => ContractFamilyId::Color,
            Self::Icon(_) => ContractFamilyId::Icon,
            Self::Image(_) => ContractFamilyId::Image,
            Self::Twemoji(_) => ContractFamilyId::Twemoji,
            Self::Kbd(_) => ContractFamilyId::Kbd,
            Self::Skeleton(_) => ContractFamilyId::Skeleton,
            Self::Slider(_) => ContractFamilyId::Slider,
            Self::Radio(_) => ContractFamilyId::Radio,
            Self::RadioGroup(_) => ContractFamilyId::RadioGroup,
            Self::Combobox(_) => ContractFamilyId::Combobox,
            Self::EmojiSelector(_) => ContractFamilyId::EmojiSelector,
            Self::Pagination(_) => ContractFamilyId::Pagination,
            Self::Tooltip(_) => ContractFamilyId::Tooltip,
            Self::Popover(_) => ContractFamilyId::Popover,
            Self::DropdownMenu(_) => ContractFamilyId::DropdownMenu,
            Self::ContextMenu(_) => ContractFamilyId::ContextMenu,
            Self::OpenWith(_) => ContractFamilyId::OpenWith,
            Self::CollabCursor(_) => ContractFamilyId::CollabCursor,
            Self::IconToolbar(_) => ContractFamilyId::IconToolbar,
            Self::FileTree(_) => ContractFamilyId::FileTree,
            Self::DragBoard(_) => ContractFamilyId::DragBoard,
            Self::AudioPlayback(_) => ContractFamilyId::AudioPlayback,
            Self::ImageTile(_) => ContractFamilyId::ImageTile,
            Self::Command(_) => ContractFamilyId::Command,
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
            Self::Color(node) => &node.common,
            Self::Icon(node) => &node.common,
            Self::Image(node) => &node.common,
            Self::Twemoji(node) => &node.common,
            Self::Kbd(node) => &node.common,
            Self::Skeleton(node) => &node.common,
            Self::Slider(node) => &node.common,
            Self::Radio(node) => &node.common,
            Self::RadioGroup(node) => &node.common,
            Self::Combobox(node) => &node.common,
            Self::EmojiSelector(node) => &node.common,
            Self::Pagination(node) => &node.common,
            Self::Tooltip(node) => &node.common,
            Self::Popover(node) => &node.common,
            Self::DropdownMenu(node) => &node.common,
            Self::ContextMenu(node) => &node.common,
            Self::OpenWith(node) => &node.common,
            Self::CollabCursor(node) => &node.common,
            Self::IconToolbar(node) => &node.common,
            Self::FileTree(node) => &node.common,
            Self::DragBoard(node) => &node.common,
            Self::AudioPlayback(node) => &node.common,
            Self::ImageTile(node) => &node.common,
            Self::Command(node) => &node.common,
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
pub struct ContractColor {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub fill: ContractColorValue,
    #[serde(default = "default_color_size")]
    pub size: f32,
    #[serde(default)]
    pub stroke: Option<ContractStroke>,
    #[serde(default)]
    pub corner_radius: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractIcon {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub name: String,
    #[serde(default = "default_icon_size")]
    pub size: f32,
    #[serde(default)]
    pub tint: Option<ContractColorValue>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractImage {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub source: String,
    #[serde(default = "default_image_width")]
    pub width: f32,
    #[serde(default = "default_image_height")]
    pub height: f32,
    #[serde(default)]
    pub corner_radius: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractTwemoji {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub emoji: String,
    #[serde(default = "default_twemoji_size")]
    pub size: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractKbd {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub text: String,
    #[serde(default = "default_kbd_min_width")]
    pub min_width: f32,
    #[serde(default = "default_kbd_height")]
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractSkeleton {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default = "default_skeleton_width")]
    pub width: f32,
    #[serde(default = "default_skeleton_height")]
    pub height: f32,
    #[serde(default)]
    pub circle: bool,
    #[serde(default)]
    pub corner_radius: Option<u8>,
    #[serde(default = "default_true")]
    pub animated: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractSlider {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub value: f32,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default = "default_slider_width")]
    pub width: f32,
    #[serde(default = "default_number_min")]
    pub min: f32,
    #[serde(default = "default_number_max")]
    pub max: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractRadio {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub value: bool,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractRadioGroup {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub selected_item_id: Option<String>,
    #[serde(default = "default_gap")]
    pub gap: f32,
    #[serde(default)]
    pub items: Vec<ContractRadioItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractCombobox {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub selected_item_ids: Vec<String>,
    #[serde(default = "default_input_width")]
    pub width: f32,
    #[serde(default = "default_combobox_max_height")]
    pub max_height: f32,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub filter_placeholder: Option<String>,
    #[serde(default = "default_true")]
    pub searchable: bool,
    #[serde(default)]
    pub items: Vec<ContractChoiceItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractEmojiSelector {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub value: String,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default = "default_emoji_popup_width")]
    pub popup_width: f32,
    #[serde(default = "default_emoji_popup_max_height")]
    pub popup_max_height: f32,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub trigger_variant: Option<ButtonVariant>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractPagination {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default = "default_pagination_current_page")]
    pub current_page: usize,
    #[serde(default = "default_pagination_page_count")]
    pub page_count: usize,
    #[serde(default = "default_pagination_sibling_count")]
    pub sibling_count: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractTooltip {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub trigger_label: String,
    pub text: String,
    #[serde(default = "default_tooltip_width")]
    pub width: f32,
    #[serde(default)]
    pub delay_ms: u32,
    #[serde(default = "default_tooltip_placement")]
    pub placement: TooltipPlacement,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractPopover {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub open: bool,
    #[serde(default)]
    pub trigger_label: Option<String>,
    #[serde(default = "default_popover_side")]
    pub side: PopoverSide,
    #[serde(default = "default_popover_align")]
    pub align: PopoverAlign,
    #[serde(default = "default_popover_side_offset")]
    pub side_offset: f32,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default = "default_popover_padding_x")]
    pub padding_x: u8,
    #[serde(default = "default_popover_padding_y")]
    pub padding_y: u8,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractDropdownMenu {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    pub trigger_label: String,
    #[serde(default = "default_dropdown_width")]
    pub width: f32,
    #[serde(default)]
    pub trigger_variant: Option<ButtonVariant>,
    #[serde(default)]
    pub entries: Vec<ContractMenuEntry>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractContextMenu {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default = "default_context_menu_width")]
    pub width: f32,
    #[serde(default = "default_context_menu_region_width")]
    pub region_width: f32,
    #[serde(default = "default_context_menu_region_height")]
    pub region_height: f32,
    #[serde(default = "default_context_menu_padding_x")]
    pub padding_x: u8,
    #[serde(default = "default_context_menu_padding_y")]
    pub padding_y: u8,
    #[serde(default)]
    pub entries: Vec<ContractMenuEntry>,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractOpenWith {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default = "default_dropdown_width")]
    pub width: f32,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub size: Option<ControlSize>,
    #[serde(default)]
    pub trigger_variant: Option<ButtonVariant>,
    #[serde(default)]
    pub entries: Vec<ContractMenuEntry>,
    #[serde(default)]
    pub selected_item_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractCollabCursor {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub name: String,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub color: Option<ContractColorValue>,
    #[serde(default = "default_collab_cursor_size")]
    pub size: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractIconToolbar {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub selected_item_id: Option<String>,
    #[serde(default)]
    pub size: Option<ControlSize>,
    #[serde(default = "default_icon_toolbar_gap")]
    pub gap: f32,
    #[serde(default)]
    pub icon_size: Option<f32>,
    #[serde(default)]
    pub items: Vec<ContractIconToolbarItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractFileTree {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub selected_item_id: Option<String>,
    #[serde(default = "default_file_tree_width")]
    pub width: f32,
    #[serde(default = "default_file_tree_row_height")]
    pub row_height: f32,
    #[serde(default = "default_file_tree_indent_width")]
    pub indent_width: f32,
    #[serde(default)]
    pub items: Vec<ContractFileTreeItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractDragBoard {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default = "default_drag_board_left_title")]
    pub left_title: String,
    #[serde(default = "default_drag_board_right_title")]
    pub right_title: String,
    #[serde(default = "default_drag_board_height")]
    pub height: f32,
    #[serde(default)]
    pub items: Vec<ContractDragBoardItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractAudioPlayback {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default = "default_audio_playback_state")]
    pub playback_state: AudioPlaybackState,
    #[serde(default)]
    pub duration_seconds: Option<f64>,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractImageTile {
    #[serde(flatten)]
    pub common: ContractCommon,
    pub source: String,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub play_pause_action_id: Option<ActionId>,
    #[serde(default)]
    pub size: Option<ImageTileSize>,
    #[serde(default)]
    pub image_width: Option<f32>,
    #[serde(default)]
    pub image_height: Option<f32>,
    #[serde(default = "default_true")]
    pub image_frame: bool,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub playback_state: Option<ImageTilePlaybackState>,
    #[serde(default)]
    pub children: Vec<ContractNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractCommand {
    #[serde(flatten)]
    pub common: ContractCommon,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub query: String,
    #[serde(default = "default_command_width")]
    pub width: f32,
    #[serde(default = "default_command_max_height")]
    pub max_height: f32,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub preview: bool,
    #[serde(default = "default_command_preview_height")]
    pub preview_height: f32,
    #[serde(default)]
    pub items: Vec<ContractCommandItem>,
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
#[serde(transparent)]
pub struct ContractColorValue(pub String);

impl ContractColorValue {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for ContractColorValue {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for ContractColorValue {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractStroke {
    #[serde(default = "default_stroke_width")]
    pub width: f32,
    pub color: ContractColorValue,
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
    Submenu(ContractMenuSubmenu),
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
pub struct ContractMenuSubmenu {
    pub label: String,
    #[serde(default)]
    pub leading_icon: Option<String>,
    #[serde(default)]
    pub entries: Vec<ContractMenuEntry>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractRadioItem {
    pub item_id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub action_id: Option<ActionId>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractIconToolbarItem {
    pub item_id: String,
    pub icon: String,
    #[serde(default)]
    pub tooltip: Option<String>,
    #[serde(default)]
    pub badge_fill: Option<ContractColorValue>,
    #[serde(default)]
    pub action_id: Option<ActionId>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractFileTreeItem {
    pub item_id: String,
    pub label: String,
    pub kind: FileTreeItemKind,
    #[serde(default = "default_true")]
    pub open: bool,
    #[serde(default)]
    pub action_id: Option<ActionId>,
    #[serde(default)]
    pub children: Vec<ContractFileTreeItem>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractDragBoardItem {
    pub item_id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub region: DragBoardRegion,
    #[serde(default)]
    pub action_id: Option<ActionId>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractCommandItem {
    pub item_id: String,
    #[serde(default)]
    pub group: String,
    pub label: String,
    #[serde(default)]
    pub shortcut: Option<String>,
    #[serde(default)]
    pub action_id: Option<ActionId>,
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
    ItemIds(Vec<String>),
    ItemMove(ContractItemMove),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ContractItemMove {
    pub item_id: String,
    pub from: String,
    pub to: String,
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

fn default_color_size() -> f32 {
    20.0
}

fn default_icon_size() -> f32 {
    16.0
}

fn default_image_width() -> f32 {
    160.0
}

fn default_image_height() -> f32 {
    104.0
}

fn default_twemoji_size() -> f32 {
    22.0
}

fn default_kbd_min_width() -> f32 {
    20.0
}

fn default_kbd_height() -> f32 {
    20.0
}

fn default_skeleton_width() -> f32 {
    120.0
}

fn default_skeleton_height() -> f32 {
    16.0
}

fn default_slider_width() -> f32 {
    156.0
}

fn default_combobox_max_height() -> f32 {
    104.0
}

fn default_emoji_popup_width() -> f32 {
    320.0
}

fn default_emoji_popup_max_height() -> f32 {
    360.0
}

fn default_pagination_current_page() -> usize {
    1
}

fn default_pagination_page_count() -> usize {
    1
}

fn default_pagination_sibling_count() -> usize {
    1
}

fn default_tooltip_width() -> f32 {
    220.0
}

fn default_tooltip_placement() -> TooltipPlacement {
    TooltipPlacement::Top
}

fn default_popover_side() -> PopoverSide {
    PopoverSide::Bottom
}

fn default_popover_align() -> PopoverAlign {
    PopoverAlign::Center
}

fn default_popover_side_offset() -> f32 {
    8.0
}

fn default_popover_padding_x() -> u8 {
    12
}

fn default_popover_padding_y() -> u8 {
    12
}

fn default_dropdown_width() -> f32 {
    220.0
}

fn default_context_menu_width() -> f32 {
    220.0
}

fn default_context_menu_region_width() -> f32 {
    360.0
}

fn default_context_menu_region_height() -> f32 {
    176.0
}

fn default_context_menu_padding_x() -> u8 {
    16
}

fn default_context_menu_padding_y() -> u8 {
    14
}

fn default_icon_toolbar_gap() -> f32 {
    4.0
}

fn default_collab_cursor_size() -> f32 {
    30.0
}

fn default_file_tree_width() -> f32 {
    240.0
}

fn default_file_tree_row_height() -> f32 {
    20.0
}

fn default_file_tree_indent_width() -> f32 {
    14.0
}

fn default_drag_board_left_title() -> String {
    "Backlog".to_owned()
}

fn default_drag_board_right_title() -> String {
    "Done".to_owned()
}

fn default_drag_board_height() -> f32 {
    280.0
}

fn default_audio_playback_state() -> AudioPlaybackState {
    AudioPlaybackState::Paused
}

fn default_command_width() -> f32 {
    360.0
}

fn default_command_max_height() -> f32 {
    216.0
}

fn default_command_preview_height() -> f32 {
    244.0
}

fn default_stroke_width() -> f32 {
    1.0
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
            variant: Some(crate::runtime_components::ButtonVariant::Primary),
            size: Some(crate::runtime_components::ControlSize::Md),
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
            "version": 2,
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
