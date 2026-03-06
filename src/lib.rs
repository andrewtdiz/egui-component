pub mod catalog;
pub mod components;
pub mod dev;
pub mod theme;
pub mod ui;

mod error;

pub use catalog::{
    component_definitions, component_definitions_by_group, parse_component_kind,
    supported_component_ids_csv, ComponentDefinition, ComponentGroup, ComponentKind,
};
pub use error::{ComponentLibraryError, Result};

pub mod prelude {
    pub use crate::components::{
        Button, ButtonGroup, ButtonOverride, ButtonStyle, Card, CardOverride, Checkbox,
        Collapsible, Combobox, Command, CommandItem, ComponentUi, ComponentUiExt, Dialogue,
        DialogueHeader, DialogueModal, DialogueStyle, DropdownMenu, DropdownMenuAction,
        DropdownMenuEntry, DropdownMenuState, DropdownMenuSubmenu, Field, Icon, Kbd, KbdGroup,
        Label, LabelOverride, LabelTone, LabelWeight, NumberInput, NumberInputAxis, Progress,
        Select, Slider, Switch, SwitchSize, TabOption, TextInput, TextInputOverride, Tooltip,
    };
}
