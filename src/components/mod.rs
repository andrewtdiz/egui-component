mod api;
// xtask:component-modules:start
mod button;
mod button_group;
mod card;
mod checkbox;
mod collapsible;
mod color;
mod combobox;
mod command;
mod common;
mod dialogue;
mod dropdown_menu;
mod field;
mod icon;
mod image;
mod input;
mod kbd;
mod label;
mod menu_bar;
mod progress;
mod select;
mod separator;
mod slider;
mod switch;
mod tabs;
mod toolbar;
mod tooltip;
// xtask:component-modules:end

pub use api::{ComponentUi, ComponentUiExt};
// xtask:component-exports:start
pub use button::{Button, ButtonOverride, ButtonVariant};
pub use button_group::ButtonGroup;
pub use card::{Card, CardOverride};
pub use checkbox::Checkbox;
pub use collapsible::Collapsible;
pub use color::Color;
pub use combobox::Combobox;
pub use command::{Command, CommandItem};
pub use common::ControlSize;
pub use dialogue::{Dialogue, DialogueHeader, DialogueIntent, DialogueModal};
pub use dropdown_menu::{
    DropdownMenu, DropdownMenuAction, DropdownMenuEntry, DropdownMenuState, DropdownMenuSubmenu,
};
pub use field::Field;
pub use icon::Icon;
pub use image::Image;
pub use input::{TextInput, TextInputOverride};
pub use kbd::{Kbd, KbdGroup};
pub use label::{Label, LabelOverride, LabelTone, LabelWeight};
pub use menu_bar::{MenuBar, MenuBarItem, MenuBarState};
pub use progress::Progress;
pub use select::Select;
pub use slider::{NumberInput, NumberInputAxis, Slider};
pub use switch::Switch;
pub use tabs::TabOption;
pub use toolbar::Toolbar;
pub use tooltip::{Tooltip, TooltipPlacement};
// xtask:component-exports:end
