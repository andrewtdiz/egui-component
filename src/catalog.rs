#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ComponentKind {
    // xtask:component-kinds:start
    AudioPlayback,
    CanvaBackgrounds,
    CanvaBrandKit,
    CanvaEditImage,
    Button,
    CanvaPosition,
    Card,
    Checkbox,
    CollabCursor,
    Color,
    Combobox,
    Command,
    ContextMenu,
    Dialogue,
    DragBoard,
    DropdownMenu,
    FileTree,
    Hierarchy,
    Icon,
    Image,
    Input,
    Label,
    MenuBar,
    NumberInput,
    Popover,
    Radio,
    RadioGroup,
    Select,
    Sidebar,
    Slider,
    Switch,
    Tabs,
    Toast,
    Tooltip,
    // xtask:component-kinds:end
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ComponentGroup {
    Primitive,
    Composed,
}

impl ComponentGroup {
    pub fn title(self) -> &'static str {
        match self {
            Self::Primitive => "Primitive",
            Self::Composed => "Composed",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentDefinition {
    pub kind: ComponentKind,
    pub id: &'static str,
    pub label: &'static str,
    pub group: ComponentGroup,
}

const COMPONENT_DEFINITIONS: &[ComponentDefinition] = &[
    // xtask:component-definitions:start
    ComponentDefinition {
        kind: ComponentKind::Label,
        id: "label",
        label: "Label",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Color,
        id: "color",
        label: "Color",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Image,
        id: "image",
        label: "Image",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Icon,
        id: "icon",
        label: "Icon",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Input,
        id: "input",
        label: "Input",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Button,
        id: "button",
        label: "Button",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Checkbox,
        id: "checkbox",
        label: "Checkbox",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Switch,
        id: "switch",
        label: "Switch",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Slider,
        id: "slider",
        label: "Slider",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::NumberInput,
        id: "number-input",
        label: "Number Input",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Select,
        id: "select",
        label: "Select",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Tabs,
        id: "tabs",
        label: "Tabs",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Card,
        id: "card",
        label: "Card",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Radio,
        id: "radio",
        label: "Radio",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Popover,
        id: "popover",
        label: "Popover",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Tooltip,
        id: "tooltip",
        label: "Tooltip",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::DropdownMenu,
        id: "dropdown-menu",
        label: "Dropdown Menu",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::CanvaBackgrounds,
        id: "canva-backgrounds",
        label: "Backgrounds",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::CanvaBrandKit,
        id: "canva-brand-kit",
        label: "Brand Kit",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::CanvaEditImage,
        id: "canva-edit-image",
        label: "Edit Image",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::CanvaPosition,
        id: "canva-position",
        label: "Position",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::CollabCursor,
        id: "collab-cursor",
        label: "Collab Cursor",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::Hierarchy,
        id: "hierarchy",
        label: "Hierarchy",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::FileTree,
        id: "file-tree",
        label: "FileTree",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::AudioPlayback,
        id: "audio-playback",
        label: "Audio Playback",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::Combobox,
        id: "combobox",
        label: "Combobox",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::ContextMenu,
        id: "context-menu",
        label: "Context Menu",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::RadioGroup,
        id: "radio-group",
        label: "Radio Group",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::Command,
        id: "command",
        label: "Command",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::Dialogue,
        id: "dialogue",
        label: "Dialogue",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::DragBoard,
        id: "drag-board",
        label: "Drag Board",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::MenuBar,
        id: "menu-bar",
        label: "Menu Bar",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::Sidebar,
        id: "sidebar",
        label: "Sidebar",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::Toast,
        id: "toast",
        label: "Toast",
        group: ComponentGroup::Composed,
    },
    // xtask:component-definitions:end
];

impl ComponentKind {
    pub fn display_name(self) -> &'static str {
        component_definition(self).label
    }
}

pub fn component_definitions() -> impl Iterator<Item = &'static ComponentDefinition> {
    COMPONENT_DEFINITIONS.iter()
}

pub fn component_definitions_by_group(
    group: ComponentGroup,
) -> impl Iterator<Item = &'static ComponentDefinition> {
    component_definitions().filter(move |definition| definition.group == group)
}

pub fn parse_component_kind(value: &str) -> Option<ComponentKind> {
    let normalized = normalize_component_id(value);
    component_definitions()
        .find(|definition| normalize_component_id(definition.id) == normalized)
        .map(|definition| definition.kind)
}

pub fn supported_component_ids_csv() -> String {
    component_definitions()
        .map(|definition| definition.id)
        .collect::<Vec<_>>()
        .join(", ")
}

fn component_definition(kind: ComponentKind) -> &'static ComponentDefinition {
    COMPONENT_DEFINITIONS
        .iter()
        .find(|definition| definition.kind == kind)
        .expect("missing component definition")
}

fn normalize_component_id(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{component_definitions, parse_component_kind, ComponentGroup, ComponentKind};

    #[test]
    fn catalog_contains_only_implemented_components() {
        let ids = component_definitions()
            .map(|definition| definition.id)
            .collect::<Vec<_>>();
        assert!(ids.len() >= 24);
        assert!(ids.contains(&"button"));
        assert!(ids.contains(&"collab-cursor"));
        assert!(ids.contains(&"color"));
        assert!(ids.contains(&"icon"));
        assert!(ids.contains(&"image"));
        assert!(ids.contains(&"number-input"));
        assert!(ids.contains(&"popover"));
        assert!(ids.contains(&"dropdown-menu"));
        assert!(ids.contains(&"context-menu"));
        assert!(ids.contains(&"menu-bar"));
        assert!(ids.contains(&"file-tree"));
        assert!(ids.contains(&"radio"));
        assert!(ids.contains(&"radio-group"));
        assert!(ids.contains(&"sidebar"));
        assert!(ids.contains(&"toast"));
        assert!(!ids.contains(&"alert-dialogue"));
        assert!(!ids.contains(&"accordion"));
    }

    #[test]
    fn parser_accepts_alias_variants() {
        assert_eq!(
            parse_component_kind("collab_cursor"),
            Some(ComponentKind::CollabCursor)
        );
        assert_eq!(
            parse_component_kind("dialogue"),
            Some(ComponentKind::Dialogue)
        );
        assert_eq!(
            parse_component_kind("file_tree"),
            Some(ComponentKind::FileTree)
        );
        assert_eq!(
            parse_component_kind("number_input"),
            Some(ComponentKind::NumberInput)
        );
        assert_eq!(parse_component_kind("icon"), Some(ComponentKind::Icon));
        assert_eq!(
            parse_component_kind("popover"),
            Some(ComponentKind::Popover)
        );
        assert_eq!(
            parse_component_kind("contextmenu"),
            Some(ComponentKind::ContextMenu)
        );
        assert_eq!(parse_component_kind("image"), Some(ComponentKind::Image));
        assert_eq!(
            parse_component_kind("menu_bar"),
            Some(ComponentKind::MenuBar)
        );
        assert_eq!(parse_component_kind("alertdialogue"), None);
        assert_eq!(parse_component_kind("agentchat"), None);
    }

    #[test]
    fn groups_match_expected_split() {
        let primitive = component_definitions()
            .filter(|definition| definition.group == ComponentGroup::Primitive)
            .count();
        let composed = component_definitions()
            .filter(|definition| definition.group == ComponentGroup::Composed)
            .count();
        assert_eq!(primitive + composed, component_definitions().count());
        assert!(primitive >= 12);
        assert!(composed >= 8);
    }
}
