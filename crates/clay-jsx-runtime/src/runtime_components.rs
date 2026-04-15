#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AudioPlaybackState {
    Paused,
    Playing,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ControlSize {
    Sm,
    Md,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum DialogueIntent {
    Default,
    Alert,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub enum DragBoardRegion {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum FileTreeItemKind {
    Folder,
    Collection,
    Script,
    Project,
    Markdown,
    File,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum HierarchyItemKind {
    Folder,
    GameObject,
    Frame,
    Group,
    Player,
    Weapon,
    Clothing,
    Hitbox,
    Vector,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum HierarchyIconStyle {
    #[default]
    Emoji,
    Icons,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum HierarchyStyle {
    #[default]
    Normal,
    Component,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum LabelTone {
    Primary,
    Secondary,
    Muted,
    Destructive,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum LabelWeight {
    Regular,
    Semibold,
    Bold,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum NumberInputAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PopoverSide {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PopoverAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum SelectVariant {
    Default,
    Secondary,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToastIntent {
    #[default]
    Neutral,
    Success,
    Destructive,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToastPlacement {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    #[default]
    BottomRight,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum TooltipPlacement {
    Auto,
    Top,
    Right,
    Bottom,
    Left,
}
