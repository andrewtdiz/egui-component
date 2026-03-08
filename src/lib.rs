pub mod catalog;
pub mod components;
pub mod theme;
pub mod ui;

#[cfg(feature = "showcase")]
pub mod dev;

#[cfg(feature = "showcase")]
mod error;

pub use catalog::{
    component_definitions, component_definitions_by_group, parse_component_kind,
    supported_component_ids_csv, ComponentDefinition, ComponentGroup, ComponentKind,
};
#[cfg(feature = "showcase")]
pub use error::{ComponentLibraryError, Result};
pub use theme::ThemeMode;

pub mod prelude {
    pub use crate::components::{
        Button, ButtonGroup, ButtonOverride, ButtonStyle, Card, CardOverride, Checkbox,
        Collapsible, Color, Combobox, Command, CommandItem, ComponentUi, ComponentUiExt, Dialogue,
        DialogueHeader, DialogueModal, DialogueStyle, DropdownMenu, DropdownMenuAction,
        DropdownMenuEntry, DropdownMenuState, DropdownMenuSubmenu, Field, Icon, Image, Kbd,
        KbdGroup, Label, LabelOverride, LabelTone, LabelWeight, MenuBar, MenuBarItem, MenuBarState,
        NumberInput, NumberInputAxis, Progress, Select, Slider, Switch, SwitchSize, TabOption,
        TextInput, TextInputOverride, Toolbar, Tooltip, TooltipPlacement,
    };
    pub use crate::theme::ThemeMode;
}
