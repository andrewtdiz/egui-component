pub mod catalog;
pub mod components;
pub mod dev;
pub mod ui;

mod error;

pub use catalog::{
    component_definitions, component_definitions_by_group, parse_component_kind,
    supported_component_ids_csv, ComponentDefinition, ComponentGroup, ComponentKind,
};
pub use error::{ComponentLibraryError, Result};

pub mod prelude {
    pub use crate::components::{
        button, button_group, card, checkbox, collapsible, combobox, command, context_menu, dialog,
        dropdown_menu, field, label, number_input, progress, resizable, scroll_area, select,
        separator, slider, switch, tabs, text_input, tooltip, ButtonGroupProps, ButtonProps,
        ButtonVariant, CardProps, CheckboxProps, CollapsibleProps, ComboboxProps, CommandItem,
        CommandProps, ContextMenuAction, ContextMenuProps, ContextMenuState, DialogProps,
        DialogVariant, DropdownMenuProps, FieldProps, LabelProps, LabelTone, LabelWeight,
        NumberInputProps, ProgressProps, ResizableProps, ScrollAreaProps, SelectProps, SliderProps,
        SwitchProps, TabOption, TextInputProps, TooltipProps,
    };
}
