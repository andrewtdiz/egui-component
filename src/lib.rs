pub mod catalog;
pub mod components;
pub mod layout;
pub mod icons {
    pub use crate::ui::icons::{image, setup, svg_source};
}
pub mod primitives;
pub mod theme;
pub mod ui;

pub use catalog::{
    component_definitions, component_definitions_by_group, parse_component_kind,
    supported_component_ids_csv, ComponentDefinition, ComponentGroup, ComponentKind,
};
pub use theme::{
    BaseColor, ColorRole, OklchColor, RadiusRole, ShadowRole, ThemeMode, ThemePalette,
    ThemeShadows, ThemeSpec,
};

pub mod prelude {
    pub use crate::components::{
        AudioPlayback, AudioPlaybackResult, AudioPlaybackState, Button, ButtonGroup,
        ButtonLabelWeight, ButtonOverride, ButtonVariant, Card, CardOverride, Checkbox,
        Collapsible, Color, ColorInput, Combobox, Command, CommandItem, ComponentUiExt,
        ControlSize, Dialogue, DialogueHeader, DialogueIntent, DialogueModal, DropdownMenu,
        DropdownMenuAction, DropdownMenuEntry, DropdownMenuState, DropdownMenuSubmenu, Field, Icon,
        Image, ImageTile, ImageTilePlaybackState, ImageTileSize, ImageTileState, Kbd, KbdGroup,
        Label, LabelOverride, LabelTone, LabelWeight, MenuBar, MenuBarItem, MenuBarState,
        NumberInput, NumberInputAxis, Pagination, Popover, PopoverAlign, PopoverResponse,
        PopoverSide, Progress, Radio, RadioGroup, RadioOption, Select, Sidebar, SidebarSide,
        Skeleton, Slider, Spinner, Switch, TabOption, TabsVariant, TextInput, TextInputOverride,
        Toast, ToastIntent, ToastPlacement, ToastStack, ToastViewport, Toolbar, Tooltip,
        TooltipPlacement,
    };
    pub use crate::theme::{
        BaseColor, ColorRole, OklchColor, RadiusRole, ShadowRole, ThemeMode, ThemePalette,
        ThemeShadows, ThemeSpec,
    };
}
