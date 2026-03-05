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
        agent_chat, button, button_group, card, checkbox, collapsible, combobox, command,
        context_menu, dialogue, dialogue_body, dialogue_description, dialogue_footer,
        dialogue_header, dialogue_modal, dialogue_title, dropdown_menu, field, kbd, kbd_group,
        label, number_input, progress, resizable, scroll_area, select, separator, slider, switch,
        tabs, text_input, tooltip, AgentChatProps, AgentChatState, ButtonGroupProps, ButtonProps,
        ButtonVariant, CardProps, CheckboxProps, CollapsibleProps, ComboboxProps, CommandItem,
        CommandProps, ContextMenuAction, ContextMenuProps, ContextMenuState, DialogueHeaderProps,
        DialogueModalProps, DialogueProps, DialogueVariant, DropdownMenuAction, DropdownMenuEntry,
        DropdownMenuProps, DropdownMenuState, DropdownMenuSubmenu, FieldProps, KbdGroupProps,
        KbdProps, LabelProps, LabelTone, LabelWeight, NumberInputAxis, NumberInputProps,
        ProgressProps, ResizableProps, ScrollAreaProps, SelectProps, SliderProps, SwitchProps,
        SwitchSize, TabOption, TextInputProps, TooltipProps,
    };
}
