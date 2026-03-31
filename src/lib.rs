pub mod catalog;
pub mod components;
pub mod contract;
#[doc(hidden)]
pub mod example_apps;
mod internal_taffy;
pub mod layout;
pub mod icons {
    pub use crate::ui::icons::{image, setup, svg_source};
}
pub mod primitives;
pub mod theme;
pub mod ui;

pub(crate) use internal_taffy::{
    setup_tui_visuals, tid, AsTuiBuilder, TaffyContainerUi, Tui, TuiBuilder, TuiBuilderLogic,
    TuiBuilderParamsAccess, TuiContainerResponse, TuiId, TuiInnerResponse, TuiWidget,
};

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
<<<<<<< HEAD
        CollabCursor, Collapsible, Color, ColorInput, Combobox, Command, CommandItem,
        ComponentUiExt, ContextMenu, ContextMenuState, ControlSize, Dialogue, DialogueHeader,
        DialogueIntent, DialogueModal, DragBoard, DragBoardItem, DragBoardRegion, DropdownMenu,
        DropdownMenuAction, DropdownMenuEntry, DropdownMenuState, DropdownMenuSubmenu,
        EmojiSelector, Field, FileTree, FileTreeItemKind, FileTreeNode, Hierarchy,
        HierarchyIconStyle, HierarchyItemKind, HierarchyNode, HierarchyStyle, Icon, IconToolbar,
        IconToolbarItem, Image, ImageTile, ImageTilePlaybackState, ImageTileSize, ImageTileState,
        Kbd, KbdGroup, Label, LabelOverride, LabelTone, LabelWeight, MenuBar, MenuBarItem,
        MenuBarState, NumberInput, NumberInputAxis, Pagination, Popover, PopoverAlign,
        PopoverResponse, PopoverSide, Progress, Radio, RadioGroup, RadioOption, Select,
        SelectVariant, Sidebar, SidebarSide, Skeleton, Slider, Spinner, Switch, TabOption,
        TabsVariant, TextInput, TextInputOverride, Toast, ToastIntent, ToastPlacement, ToastStack,
        ToastViewport, ToggleGroup, Toolbar, Tooltip, TooltipPlacement, Twemoji,
=======
        CollabCursor, Collapsible, Color, ColorInput, ColorStrip, ColorStripKind, Combobox,
        Command, CommandItem, ComponentUi, ComponentUiExt, ContextMenu, ContextMenuState,
        ControlSize, Dialogue, DialogueHeader, DialogueIntent, DialogueModal, DragBoard,
        DragBoardItem, DragBoardRegion, DropdownMenu, DropdownMenuAction, DropdownMenuEntry,
        DropdownMenuState, DropdownMenuSubmenu, EmojiSelector, Field, Hierarchy,
        HierarchyDropPlacement, HierarchyIconStyle, HierarchyItemKind, HierarchyMoveRequest,
        HierarchyNode, HierarchyResponse, HierarchyRowState, HierarchyStyle, Icon, Image,
        ImageTile, ImageTilePlaybackState, ImageTileSize, ImageTileState, Kbd, KbdGroup, Label,
        LabelOverride, LabelTone, LabelWeight, MenuBar, MenuBarItem, MenuBarState, NumberInput,
        NumberInputAxis, Pagination, PalettePreview, Popover, PopoverAlign, PopoverResponse,
        PopoverSide, Progress, Radio, RadioGroup, RadioOption, Select, SelectVariant, Sidebar,
        SidebarSide, Skeleton, Slider, Spinner, Switch, TabOption, TabsVariant, TextInput,
        TextInputOverride, Toast, ToastIntent, ToastPlacement, ToastStack, ToastViewport,
        ToggleGroup, Toolbar, Tooltip, TooltipPlacement, Twemoji,
>>>>>>> 70230a4 (updates)
    };
    pub use crate::theme::{
        BaseColor, ColorRole, OklchColor, RadiusRole, ShadowRole, ThemeMode, ThemePalette,
        ThemeShadows, ThemeSpec,
    };
}
