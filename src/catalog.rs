#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ComponentKind {
    // xtask:component-kinds:start
    Button,
    ButtonGroup,
    Card,
    Checkbox,
    Color,
    Collapsible,
    Combobox,
    Command,
    Dialogue,
    DropdownMenu,
    Field,
    Icon,
    Image,
    Input,
    Kbd,
    Label,
    MenuBar,
    NumberInput,
    Progress,
    Select,
    Separator,
    Slider,
    Switch,
    Tabs,
    Toolbar,
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
        kind: ComponentKind::Kbd,
        id: "kbd",
        label: "Kbd",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Input,
        id: "input",
        label: "Input",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Field,
        id: "field",
        label: "Field",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Button,
        id: "button",
        label: "Button",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::ButtonGroup,
        id: "button-group",
        label: "Button Group",
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
        kind: ComponentKind::Separator,
        id: "separator",
        label: "Separator",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Card,
        id: "card",
        label: "Card",
        group: ComponentGroup::Primitive,
    },
    ComponentDefinition {
        kind: ComponentKind::Progress,
        id: "progress",
        label: "Progress",
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
        kind: ComponentKind::Collapsible,
        id: "collapsible",
        label: "Collapsible",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::Combobox,
        id: "combobox",
        label: "Combobox",
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
        kind: ComponentKind::MenuBar,
        id: "menu-bar",
        label: "Menu Bar",
        group: ComponentGroup::Composed,
    },
    ComponentDefinition {
        kind: ComponentKind::Toolbar,
        id: "toolbar",
        label: "Toolbar",
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
        assert!(ids.len() >= 26);
        assert!(ids.contains(&"button"));
        assert!(ids.contains(&"color"));
        assert!(ids.contains(&"icon"));
        assert!(ids.contains(&"image"));
        assert!(ids.contains(&"kbd"));
        assert!(ids.contains(&"number-input"));
        assert!(ids.contains(&"dropdown-menu"));
        assert!(ids.contains(&"menu-bar"));
        assert!(ids.contains(&"toolbar"));
        assert!(!ids.contains(&"alert-dialogue"));
        assert!(!ids.contains(&"accordion"));
    }

    #[test]
    fn parser_accepts_alias_variants() {
        assert_eq!(
            parse_component_kind("button_group"),
            Some(ComponentKind::ButtonGroup)
        );
        assert_eq!(
            parse_component_kind("dialogue"),
            Some(ComponentKind::Dialogue)
        );
        assert_eq!(
            parse_component_kind("number_input"),
            Some(ComponentKind::NumberInput)
        );
        assert_eq!(parse_component_kind("icon"), Some(ComponentKind::Icon));
        assert_eq!(
            parse_component_kind("toolbar"),
            Some(ComponentKind::Toolbar)
        );
        assert_eq!(parse_component_kind("image"), Some(ComponentKind::Image));
        assert_eq!(
            parse_component_kind("menu_bar"),
            Some(ComponentKind::MenuBar)
        );
        assert_eq!(parse_component_kind("alertdialogue"), None);
        assert_eq!(parse_component_kind("contextmenu"), None);
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
        assert!(primitive >= 20);
        assert!(composed >= 6);
    }
}
