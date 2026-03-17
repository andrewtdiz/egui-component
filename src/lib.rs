pub mod catalog;
pub mod components;
pub mod layout;
pub mod icons {
    pub use crate::ui::icons::{image, setup, svg_source};
}
pub mod primitives;
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
pub use theme::{BaseColor, ColorRole, OklchColor, RadiusRole, ShadowRole, ThemeMode, ThemePalette, ThemeShadows, ThemeSpec};

pub mod prelude {
    pub use crate::components::{
        AudioPlayback,
        AudioPlaybackResult,
        AudioPlaybackState,
        Button,
        ButtonGroup,
        ButtonLabelWeight,
        ButtonOverride,
        ButtonVariant,
        Card,
        CardOverride,
        Checkbox,
        Collapsible,
        Color,
        ColorInput,
        Combobox,
        Command,
        CommandItem,
        ComponentUiExt,
        ControlSize,
        Dialogue,
        DialogueHeader,
        DialogueIntent,
        DialogueModal,
        DropdownMenu,
        DropdownMenuAction,
        DropdownMenuEntry,
        DropdownMenuState,
        DropdownMenuSubmenu,
        Field,
        Icon,
        Image,
        ImageTile,
        ImageTilePlaybackState,
        ImageTileSize,
        ImageTileState,
        Kbd,
        KbdGroup,
        Label,
        LabelOverride,
        LabelTone,
        LabelWeight,
        MenuBar,
        MenuBarItem,
        MenuBarState,
        NumberInput,
        NumberInputAxis,
        Pagination,
        Progress,
        Select,
        Slider,
        Switch,
        TabOption,
        TabsVariant,
        TextInput,
        TextInputOverride,
        Toolbar,
        Tooltip,
        TooltipPlacement,
        // xtask:prelude-exports:start
        // xtask:prelude-exports:end
    };
    pub use crate::theme::{BaseColor, ColorRole, OklchColor, RadiusRole, ShadowRole, ThemeMode, ThemePalette, ThemeShadows, ThemeSpec};
}
