use std::fmt::Write as _;

use super::types::{
    UiButtonVariant, UiControlSize, UiLabelTone, UiLabelWeight, UiNumberInputAxis, UiSelectVariant,
    UiSkeletonShape, UiTabsVariant, UiTooltipPlacement,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RuntimeApiFunctionId {
    AppLog,
    AppRequestReload,
    AppRequestRepaint,
    UiLabel,
    UiSeparator,
    UiButton,
    UiTextEdit,
    UiCheckbox,
    UiSwitch,
    UiSlider,
    UiNumberInput,
    UiSelect,
    UiTabs,
    UiProgress,
    UiRadio,
    UiButtonGroup,
    UiBeginCollapsible,
    UiDropdownMenu,
    UiTooltip,
    UiSpinner,
    UiSkeleton,
    UiVirtualList,
    UiBeginRow,
    UiBeginColumn,
    UiBeginCard,
    UiEndScope,
    UiPushId,
    UiPopId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RuntimeApiPropSchemaId {
    LabelProps,
    ButtonProps,
    TextEditProps,
    CheckboxProps,
    SwitchProps,
    SliderProps,
    NumberInputProps,
    SelectProps,
    TabsProps,
    ProgressProps,
    RadioProps,
    ButtonGroupProps,
    CollapsibleProps,
    DropdownMenuProps,
    TooltipProps,
    SpinnerProps,
    SkeletonProps,
    VirtualListProps,
    ContainerProps,
    CardProps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RuntimeApiEnumId {
    LabelTone,
    LabelWeight,
    ButtonVariant,
    ControlSize,
    NumberInputAxis,
    SelectVariant,
    TabsVariant,
    TooltipPlacement,
    SkeletonShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RuntimeApiNamedTypeId {
    SelectOption,
    TabOption,
    ButtonGroupOption,
    DropdownMenuEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RuntimeApiCapability {
    Log,
    Reload,
    Repaint,
}

impl RuntimeApiCapability {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Log => "log",
            Self::Reload => "reload",
            Self::Repaint => "repaint",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RuntimeApiPhaseRule {
    Any,
    FrameOnly,
    RenderOnly,
}

impl RuntimeApiPhaseRule {
    fn description(self) -> &'static str {
        match self {
            Self::Any => "Available during `load`, `reload`, `update`, and `render`.",
            Self::FrameOnly => "Available during frame phases only: `update` and `render`.",
            Self::RenderOnly => "Available during `render` only.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RuntimeApiTypeRef {
    String,
    StringArray,
    Number,
    Boolean,
    PropSchema(RuntimeApiPropSchemaId),
    Enum(RuntimeApiEnumId),
    Named(RuntimeApiNamedTypeId),
    NamedArray(RuntimeApiNamedTypeId),
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeApiEnumValueSpec {
    pub canonical: &'static str,
    pub aliases: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeApiEnumSpec {
    pub name: &'static str,
    pub values: &'static [RuntimeApiEnumValueSpec],
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeApiFieldSpec {
    pub name: &'static str,
    pub ty: RuntimeApiTypeRef,
    pub optional: bool,
    pub default: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeApiPropSchemaSpec {
    pub name: &'static str,
    pub fields: &'static [RuntimeApiFieldSpec],
}

#[derive(Debug, Clone, Copy)]
pub(super) enum RuntimeApiNamedTypeSpec {
    Object {
        name: &'static str,
        fields: &'static [RuntimeApiFieldSpec],
    },
    Alias {
        name: &'static str,
        luau: &'static str,
        markdown: &'static str,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeApiArgumentSpec {
    pub name: &'static str,
    pub ty: RuntimeApiTypeRef,
    pub optional: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeApiReturnSpec {
    pub name: Option<&'static str>,
    pub ty: RuntimeApiTypeRef,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeApiFunctionSpec {
    pub namespace: &'static str,
    pub member: &'static str,
    pub full_name: &'static str,
    pub summary: &'static str,
    pub args: &'static [RuntimeApiArgumentSpec],
    pub returns: &'static [RuntimeApiReturnSpec],
    pub phase_rule: RuntimeApiPhaseRule,
    pub capability: Option<RuntimeApiCapability>,
    pub compatibility_aliases: &'static [&'static str],
}

const APP_LOG_ARGS: [RuntimeApiArgumentSpec; 2] = [
    RuntimeApiArgumentSpec {
        name: "level",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "message",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
];

const UI_LABEL_ARGS: [RuntimeApiArgumentSpec; 2] = [
    RuntimeApiArgumentSpec {
        name: "text",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::LabelProps),
        optional: true,
    },
];

const UI_BUTTON_ARGS: [RuntimeApiArgumentSpec; 3] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "text",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::ButtonProps),
        optional: true,
    },
];

const UI_TEXT_EDIT_ARGS: [RuntimeApiArgumentSpec; 3] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "value",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::TextEditProps),
        optional: true,
    },
];

const UI_CHECKBOX_ARGS: [RuntimeApiArgumentSpec; 3] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "checked",
        ty: RuntimeApiTypeRef::Boolean,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::CheckboxProps),
        optional: true,
    },
];

const UI_SWITCH_ARGS: [RuntimeApiArgumentSpec; 3] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "checked",
        ty: RuntimeApiTypeRef::Boolean,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::SwitchProps),
        optional: true,
    },
];

const UI_SLIDER_ARGS: [RuntimeApiArgumentSpec; 3] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "value",
        ty: RuntimeApiTypeRef::Number,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::SliderProps),
        optional: false,
    },
];

const UI_NUMBER_INPUT_ARGS: [RuntimeApiArgumentSpec; 3] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "value",
        ty: RuntimeApiTypeRef::Number,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::NumberInputProps),
        optional: true,
    },
];

const UI_SELECT_ARGS: [RuntimeApiArgumentSpec; 4] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "selected_index",
        ty: RuntimeApiTypeRef::Number,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "options",
        ty: RuntimeApiTypeRef::NamedArray(RuntimeApiNamedTypeId::SelectOption),
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::SelectProps),
        optional: true,
    },
];

const UI_TABS_ARGS: [RuntimeApiArgumentSpec; 4] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "selected_index",
        ty: RuntimeApiTypeRef::Number,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "options",
        ty: RuntimeApiTypeRef::NamedArray(RuntimeApiNamedTypeId::TabOption),
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::TabsProps),
        optional: true,
    },
];

const UI_PROGRESS_ARGS: [RuntimeApiArgumentSpec; 2] = [
    RuntimeApiArgumentSpec {
        name: "value",
        ty: RuntimeApiTypeRef::Number,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::ProgressProps),
        optional: true,
    },
];

const UI_RADIO_ARGS: [RuntimeApiArgumentSpec; 3] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "selected",
        ty: RuntimeApiTypeRef::Boolean,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::RadioProps),
        optional: true,
    },
];

const UI_BUTTON_GROUP_ARGS: [RuntimeApiArgumentSpec; 3] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "options",
        ty: RuntimeApiTypeRef::NamedArray(RuntimeApiNamedTypeId::ButtonGroupOption),
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::ButtonGroupProps),
        optional: true,
    },
];

const UI_BEGIN_COLLAPSIBLE_ARGS: [RuntimeApiArgumentSpec; 4] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "title",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "open",
        ty: RuntimeApiTypeRef::Boolean,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::CollapsibleProps),
        optional: true,
    },
];

const UI_DROPDOWN_MENU_ARGS: [RuntimeApiArgumentSpec; 4] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "trigger_label",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "entries",
        ty: RuntimeApiTypeRef::NamedArray(RuntimeApiNamedTypeId::DropdownMenuEntry),
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::DropdownMenuProps),
        optional: true,
    },
];

const UI_TOOLTIP_ARGS: [RuntimeApiArgumentSpec; 3] = [
    RuntimeApiArgumentSpec {
        name: "trigger_label",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "text",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::TooltipProps),
        optional: true,
    },
];

const UI_SPINNER_ARGS: [RuntimeApiArgumentSpec; 1] = [RuntimeApiArgumentSpec {
    name: "props",
    ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::SpinnerProps),
    optional: true,
}];

const UI_SKELETON_ARGS: [RuntimeApiArgumentSpec; 1] = [RuntimeApiArgumentSpec {
    name: "props",
    ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::SkeletonProps),
    optional: true,
}];

const UI_VIRTUAL_LIST_ARGS: [RuntimeApiArgumentSpec; 3] = [
    RuntimeApiArgumentSpec {
        name: "id",
        ty: RuntimeApiTypeRef::String,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "items",
        ty: RuntimeApiTypeRef::StringArray,
        optional: false,
    },
    RuntimeApiArgumentSpec {
        name: "props",
        ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::VirtualListProps),
        optional: true,
    },
];

const UI_CONTAINER_ARGS: [RuntimeApiArgumentSpec; 1] = [RuntimeApiArgumentSpec {
    name: "props",
    ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::ContainerProps),
    optional: true,
}];

const UI_CARD_ARGS: [RuntimeApiArgumentSpec; 1] = [RuntimeApiArgumentSpec {
    name: "props",
    ty: RuntimeApiTypeRef::PropSchema(RuntimeApiPropSchemaId::CardProps),
    optional: true,
}];

const UI_PUSH_ID_ARGS: [RuntimeApiArgumentSpec; 1] = [RuntimeApiArgumentSpec {
    name: "id",
    ty: RuntimeApiTypeRef::String,
    optional: false,
}];

const NO_ARGS: [RuntimeApiArgumentSpec; 0] = [];
const NO_RETURNS: [RuntimeApiReturnSpec; 0] = [];
const BUTTON_RETURNS: [RuntimeApiReturnSpec; 1] = [RuntimeApiReturnSpec {
    name: Some("clicked"),
    ty: RuntimeApiTypeRef::Boolean,
}];
const BOOLEAN_RETURNS: [RuntimeApiReturnSpec; 2] = [
    RuntimeApiReturnSpec {
        name: Some("value"),
        ty: RuntimeApiTypeRef::Boolean,
    },
    RuntimeApiReturnSpec {
        name: Some("changed"),
        ty: RuntimeApiTypeRef::Boolean,
    },
];
const TEXT_EDIT_RETURNS: [RuntimeApiReturnSpec; 2] = [
    RuntimeApiReturnSpec {
        name: Some("value"),
        ty: RuntimeApiTypeRef::String,
    },
    RuntimeApiReturnSpec {
        name: Some("changed"),
        ty: RuntimeApiTypeRef::Boolean,
    },
];

const NUMBER_RETURNS: [RuntimeApiReturnSpec; 2] = [
    RuntimeApiReturnSpec {
        name: Some("value"),
        ty: RuntimeApiTypeRef::Number,
    },
    RuntimeApiReturnSpec {
        name: Some("changed"),
        ty: RuntimeApiTypeRef::Boolean,
    },
];

const SELECT_RETURNS: [RuntimeApiReturnSpec; 2] = [
    RuntimeApiReturnSpec {
        name: Some("selected_index"),
        ty: RuntimeApiTypeRef::Number,
    },
    RuntimeApiReturnSpec {
        name: Some("changed"),
        ty: RuntimeApiTypeRef::Boolean,
    },
];

const BUTTON_GROUP_RETURNS: [RuntimeApiReturnSpec; 2] = [
    RuntimeApiReturnSpec {
        name: Some("clicked_index"),
        ty: RuntimeApiTypeRef::Number,
    },
    RuntimeApiReturnSpec {
        name: Some("changed"),
        ty: RuntimeApiTypeRef::Boolean,
    },
];

const COLLAPSIBLE_RETURNS: [RuntimeApiReturnSpec; 2] = [
    RuntimeApiReturnSpec {
        name: Some("open"),
        ty: RuntimeApiTypeRef::Boolean,
    },
    RuntimeApiReturnSpec {
        name: Some("visible"),
        ty: RuntimeApiTypeRef::Boolean,
    },
];

const DROPDOWN_MENU_RETURNS: [RuntimeApiReturnSpec; 2] = [
    RuntimeApiReturnSpec {
        name: Some("action_id"),
        ty: RuntimeApiTypeRef::Number,
    },
    RuntimeApiReturnSpec {
        name: Some("changed"),
        ty: RuntimeApiTypeRef::Boolean,
    },
];

const VIRTUAL_LIST_RETURNS: [RuntimeApiReturnSpec; 2] = [
    RuntimeApiReturnSpec {
        name: Some("selected_index"),
        ty: RuntimeApiTypeRef::Number,
    },
    RuntimeApiReturnSpec {
        name: Some("changed"),
        ty: RuntimeApiTypeRef::Boolean,
    },
];

const LABEL_TONE_VALUES: [RuntimeApiEnumValueSpec; 4] = [
    RuntimeApiEnumValueSpec {
        canonical: "primary",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "secondary",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "muted",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "destructive",
        aliases: &[],
    },
];

const LABEL_WEIGHT_VALUES: [RuntimeApiEnumValueSpec; 3] = [
    RuntimeApiEnumValueSpec {
        canonical: "regular",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "semibold",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "bold",
        aliases: &[],
    },
];

const BUTTON_VARIANT_VALUES: [RuntimeApiEnumValueSpec; 4] = [
    RuntimeApiEnumValueSpec {
        canonical: "primary",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "secondary",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "ghost",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "link",
        aliases: &[],
    },
];

const CONTROL_SIZE_VALUES: [RuntimeApiEnumValueSpec; 2] = [
    RuntimeApiEnumValueSpec {
        canonical: "sm",
        aliases: &["small"],
    },
    RuntimeApiEnumValueSpec {
        canonical: "md",
        aliases: &["medium"],
    },
];

const NUMBER_INPUT_AXIS_VALUES: [RuntimeApiEnumValueSpec; 2] = [
    RuntimeApiEnumValueSpec {
        canonical: "horizontal",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "vertical",
        aliases: &[],
    },
];

const SELECT_VARIANT_VALUES: [RuntimeApiEnumValueSpec; 2] = [
    RuntimeApiEnumValueSpec {
        canonical: "default",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "secondary",
        aliases: &[],
    },
];

const TABS_VARIANT_VALUES: [RuntimeApiEnumValueSpec; 5] = [
    RuntimeApiEnumValueSpec {
        canonical: "underline",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "segmented",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "stacked",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "rail",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "blender_topbar",
        aliases: &[],
    },
];

const TOOLTIP_PLACEMENT_VALUES: [RuntimeApiEnumValueSpec; 5] = [
    RuntimeApiEnumValueSpec {
        canonical: "auto",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "top",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "right",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "bottom",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "left",
        aliases: &[],
    },
];

const SKELETON_SHAPE_VALUES: [RuntimeApiEnumValueSpec; 2] = [
    RuntimeApiEnumValueSpec {
        canonical: "rect",
        aliases: &[],
    },
    RuntimeApiEnumValueSpec {
        canonical: "circle",
        aliases: &[],
    },
];

const LABEL_PROPS_FIELDS: [RuntimeApiFieldSpec; 3] = [
    RuntimeApiFieldSpec {
        name: "tone",
        ty: RuntimeApiTypeRef::Enum(RuntimeApiEnumId::LabelTone),
        optional: true,
        default: Some("primary"),
    },
    RuntimeApiFieldSpec {
        name: "weight",
        ty: RuntimeApiTypeRef::Enum(RuntimeApiEnumId::LabelWeight),
        optional: true,
        default: Some("regular"),
    },
    RuntimeApiFieldSpec {
        name: "size",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
];

const BUTTON_PROPS_FIELDS: [RuntimeApiFieldSpec; 8] = [
    RuntimeApiFieldSpec {
        name: "variant",
        ty: RuntimeApiTypeRef::Enum(RuntimeApiEnumId::ButtonVariant),
        optional: true,
        default: Some("primary"),
    },
    RuntimeApiFieldSpec {
        name: "size",
        ty: RuntimeApiTypeRef::Enum(RuntimeApiEnumId::ControlSize),
        optional: true,
        default: Some("md"),
    },
    RuntimeApiFieldSpec {
        name: "width",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "leading_icon",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "trailing_icon",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "icon_size",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "icon_only",
        ty: RuntimeApiTypeRef::Boolean,
        optional: true,
        default: Some("false"),
    },
    RuntimeApiFieldSpec {
        name: "selected",
        ty: RuntimeApiTypeRef::Boolean,
        optional: true,
        default: Some("false"),
    },
];

const TEXT_EDIT_PROPS_FIELDS: [RuntimeApiFieldSpec; 4] = [
    RuntimeApiFieldSpec {
        name: "width",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "placeholder",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "leading_icon",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "password",
        ty: RuntimeApiTypeRef::Boolean,
        optional: true,
        default: Some("false"),
    },
];

const CHECKBOX_PROPS_FIELDS: [RuntimeApiFieldSpec; 1] = [RuntimeApiFieldSpec {
    name: "label",
    ty: RuntimeApiTypeRef::String,
    optional: true,
    default: None,
}];

const SWITCH_PROPS_FIELDS: [RuntimeApiFieldSpec; 2] = [
    RuntimeApiFieldSpec {
        name: "label",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "size",
        ty: RuntimeApiTypeRef::Enum(RuntimeApiEnumId::ControlSize),
        optional: true,
        default: Some("md"),
    },
];

const SLIDER_PROPS_FIELDS: [RuntimeApiFieldSpec; 3] = [
    RuntimeApiFieldSpec {
        name: "min",
        ty: RuntimeApiTypeRef::Number,
        optional: false,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "max",
        ty: RuntimeApiTypeRef::Number,
        optional: false,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "width",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("156"),
    },
];

const NUMBER_INPUT_PROPS_FIELDS: [RuntimeApiFieldSpec; 12] = [
    RuntimeApiFieldSpec {
        name: "min",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "max",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "width",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("58"),
    },
    RuntimeApiFieldSpec {
        name: "speed",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "fine_speed",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "decimals",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "fine_decimals",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "prefix",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "suffix",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "prefix_tint",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "prefix_align_left",
        ty: RuntimeApiTypeRef::Boolean,
        optional: true,
        default: Some("false"),
    },
    RuntimeApiFieldSpec {
        name: "axis",
        ty: RuntimeApiTypeRef::Enum(RuntimeApiEnumId::NumberInputAxis),
        optional: true,
        default: Some("horizontal"),
    },
];

const SELECT_PROPS_FIELDS: [RuntimeApiFieldSpec; 3] = [
    RuntimeApiFieldSpec {
        name: "width",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("220"),
    },
    RuntimeApiFieldSpec {
        name: "placeholder",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: Some("Select an option"),
    },
    RuntimeApiFieldSpec {
        name: "variant",
        ty: RuntimeApiTypeRef::Enum(RuntimeApiEnumId::SelectVariant),
        optional: true,
        default: Some("default"),
    },
];

const TABS_PROPS_FIELDS: [RuntimeApiFieldSpec; 1] = [RuntimeApiFieldSpec {
    name: "variant",
    ty: RuntimeApiTypeRef::Enum(RuntimeApiEnumId::TabsVariant),
    optional: true,
    default: Some("underline"),
}];

const PROGRESS_PROPS_FIELDS: [RuntimeApiFieldSpec; 2] = [
    RuntimeApiFieldSpec {
        name: "width",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("188"),
    },
    RuntimeApiFieldSpec {
        name: "height",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("10"),
    },
];

const RADIO_PROPS_FIELDS: [RuntimeApiFieldSpec; 2] = [
    RuntimeApiFieldSpec {
        name: "label",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "description",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
];

const BUTTON_GROUP_PROPS_FIELDS: [RuntimeApiFieldSpec; 0] = [];

const COLLAPSIBLE_PROPS_FIELDS: [RuntimeApiFieldSpec; 3] = [
    RuntimeApiFieldSpec {
        name: "open",
        ty: RuntimeApiTypeRef::Boolean,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "leading_icon",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "trailing_icon",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
];

const DROPDOWN_MENU_PROPS_FIELDS: [RuntimeApiFieldSpec; 2] = [
    RuntimeApiFieldSpec {
        name: "width",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("220"),
    },
    RuntimeApiFieldSpec {
        name: "trigger_variant",
        ty: RuntimeApiTypeRef::Enum(RuntimeApiEnumId::ButtonVariant),
        optional: true,
        default: Some("secondary"),
    },
];

const TOOLTIP_PROPS_FIELDS: [RuntimeApiFieldSpec; 3] = [
    RuntimeApiFieldSpec {
        name: "width",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("220"),
    },
    RuntimeApiFieldSpec {
        name: "delay_ms",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("0"),
    },
    RuntimeApiFieldSpec {
        name: "placement",
        ty: RuntimeApiTypeRef::Enum(RuntimeApiEnumId::TooltipPlacement),
        optional: true,
        default: Some("auto"),
    },
];

const SPINNER_PROPS_FIELDS: [RuntimeApiFieldSpec; 4] = [
    RuntimeApiFieldSpec {
        name: "size",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("16"),
    },
    RuntimeApiFieldSpec {
        name: "stroke_width",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "speed",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "color",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
];

const SKELETON_PROPS_FIELDS: [RuntimeApiFieldSpec; 3] = [
    RuntimeApiFieldSpec {
        name: "width",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("120"),
    },
    RuntimeApiFieldSpec {
        name: "height",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("16"),
    },
    RuntimeApiFieldSpec {
        name: "shape",
        ty: RuntimeApiTypeRef::Enum(RuntimeApiEnumId::SkeletonShape),
        optional: true,
        default: Some("rect"),
    },
];

const VIRTUAL_LIST_PROPS_FIELDS: [RuntimeApiFieldSpec; 4] = [
    RuntimeApiFieldSpec {
        name: "width",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "height",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("220"),
    },
    RuntimeApiFieldSpec {
        name: "row_height",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("24"),
    },
    RuntimeApiFieldSpec {
        name: "selected_index",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("0"),
    },
];

const CONTAINER_PROPS_FIELDS: [RuntimeApiFieldSpec; 1] = [RuntimeApiFieldSpec {
    name: "gap",
    ty: RuntimeApiTypeRef::Number,
    optional: true,
    default: None,
}];

const CARD_PROPS_FIELDS: [RuntimeApiFieldSpec; 3] = [
    RuntimeApiFieldSpec {
        name: "width",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "padding_x",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("12"),
    },
    RuntimeApiFieldSpec {
        name: "padding_y",
        ty: RuntimeApiTypeRef::Number,
        optional: true,
        default: Some("12"),
    },
];

const SELECT_OPTION_FIELDS: [RuntimeApiFieldSpec; 1] = [RuntimeApiFieldSpec {
    name: "label",
    ty: RuntimeApiTypeRef::String,
    optional: false,
    default: None,
}];

const TAB_OPTION_FIELDS: [RuntimeApiFieldSpec; 4] = [
    RuntimeApiFieldSpec {
        name: "label",
        ty: RuntimeApiTypeRef::String,
        optional: false,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "icon",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
    RuntimeApiFieldSpec {
        name: "icon_only",
        ty: RuntimeApiTypeRef::Boolean,
        optional: true,
        default: Some("false"),
    },
    RuntimeApiFieldSpec {
        name: "tooltip",
        ty: RuntimeApiTypeRef::String,
        optional: true,
        default: None,
    },
];

const BUTTON_GROUP_OPTION_FIELDS: [RuntimeApiFieldSpec; 1] = [RuntimeApiFieldSpec {
    name: "label",
    ty: RuntimeApiTypeRef::String,
    optional: false,
    default: None,
}];

const APP_LOG_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "app",
    member: "log",
    full_name: "app.log",
    summary: "Emit a host diagnostic log message immediately.",
    args: &APP_LOG_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::Any,
    capability: Some(RuntimeApiCapability::Log),
    compatibility_aliases: &[],
};

const APP_REQUEST_RELOAD_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "app",
    member: "request_reload",
    full_name: "app.request_reload",
    summary: "Queue a script reload after the current successful frame.",
    args: &NO_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::FrameOnly,
    capability: Some(RuntimeApiCapability::Reload),
    compatibility_aliases: &[],
};

const APP_REQUEST_REPAINT_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "app",
    member: "request_repaint",
    full_name: "app.request_repaint",
    summary: "Queue a host repaint after the current successful frame.",
    args: &NO_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::FrameOnly,
    capability: Some(RuntimeApiCapability::Repaint),
    compatibility_aliases: &[],
};

const UI_LABEL_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "label",
    full_name: "ui.label",
    summary: "Render non-interactive text in the active container.",
    args: &UI_LABEL_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_SEPARATOR_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "separator",
    full_name: "ui.separator",
    summary: "Render a visual separator in the active container.",
    args: &NO_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_BUTTON_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "button",
    full_name: "ui.button",
    summary: "Render a clickable button with an explicit local id.",
    args: &UI_BUTTON_ARGS,
    returns: &BUTTON_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_TEXT_EDIT_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "text_edit",
    full_name: "ui.text_edit",
    summary: "Render a single-line text input with an explicit local id.",
    args: &UI_TEXT_EDIT_ARGS,
    returns: &TEXT_EDIT_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_CHECKBOX_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "checkbox",
    full_name: "ui.checkbox",
    summary: "Render a checkbox control with an explicit local id.",
    args: &UI_CHECKBOX_ARGS,
    returns: &BOOLEAN_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_SWITCH_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "switch",
    full_name: "ui.switch",
    summary: "Render a switch control with an explicit local id.",
    args: &UI_SWITCH_ARGS,
    returns: &BOOLEAN_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_SLIDER_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "slider",
    full_name: "ui.slider",
    summary: "Render a slider control with an explicit local id.",
    args: &UI_SLIDER_ARGS,
    returns: &NUMBER_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_NUMBER_INPUT_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "number_input",
    full_name: "ui.number_input",
    summary: "Render a numeric input control with an explicit local id.",
    args: &UI_NUMBER_INPUT_ARGS,
    returns: &NUMBER_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_SELECT_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "select",
    full_name: "ui.select",
    summary: "Render a select trigger and return the next 1-based selection index (`0` means no selection).",
    args: &UI_SELECT_ARGS,
    returns: &SELECT_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_TABS_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "tabs",
    full_name: "ui.tabs",
    summary: "Render tabs and return the next 1-based selection index (`0` means no selection).",
    args: &UI_TABS_ARGS,
    returns: &SELECT_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_PROGRESS_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "progress",
    full_name: "ui.progress",
    summary: "Render a determinate progress indicator.",
    args: &UI_PROGRESS_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_RADIO_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "radio",
    full_name: "ui.radio",
    summary: "Render a radio control with an explicit local id.",
    args: &UI_RADIO_ARGS,
    returns: &BOOLEAN_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_BUTTON_GROUP_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "button_group",
    full_name: "ui.button_group",
    summary:
        "Render an attached button group and return the 1-based clicked index (`0` means no click).",
    args: &UI_BUTTON_GROUP_ARGS,
    returns: &BUTTON_GROUP_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_BEGIN_COLLAPSIBLE_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "begin_collapsible",
    full_name: "ui.begin_collapsible",
    summary: "Render a collapsible header and optionally open a body scope. Returns the next open state and whether the body scope is visible.",
    args: &UI_BEGIN_COLLAPSIBLE_ARGS,
    returns: &COLLAPSIBLE_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_DROPDOWN_MENU_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "dropdown_menu",
    full_name: "ui.dropdown_menu",
    summary:
        "Render a dropdown-menu trigger and return the selected action id (`0` means no action).",
    args: &UI_DROPDOWN_MENU_ARGS,
    returns: &DROPDOWN_MENU_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_TOOLTIP_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "tooltip",
    full_name: "ui.tooltip",
    summary: "Render a secondary trigger button that reveals tooltip text on hover.",
    args: &UI_TOOLTIP_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_SPINNER_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "spinner",
    full_name: "ui.spinner",
    summary: "Render an indeterminate loading spinner.",
    args: &UI_SPINNER_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_SKELETON_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "skeleton",
    full_name: "ui.skeleton",
    summary: "Render an animated skeleton placeholder.",
    args: &UI_SKELETON_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_VIRTUAL_LIST_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "virtual_list",
    full_name: "ui.virtual_list",
    summary: "Render a native scroll-virtualized string list and return the next 1-based selection index (`0` means no selection).",
    args: &UI_VIRTUAL_LIST_ARGS,
    returns: &VIRTUAL_LIST_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_BEGIN_ROW_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "begin_row",
    full_name: "ui.begin_row",
    summary: "Open a horizontal layout scope.",
    args: &UI_CONTAINER_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_BEGIN_COLUMN_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "begin_column",
    full_name: "ui.begin_column",
    summary: "Open a vertical layout scope.",
    args: &UI_CONTAINER_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_BEGIN_CARD_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "begin_card",
    full_name: "ui.begin_card",
    summary: "Open a framed card scope.",
    args: &UI_CARD_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_END_SCOPE_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "end_scope",
    full_name: "ui.end_scope",
    summary: "Close the most recently opened row, column, or card scope.",
    args: &NO_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &["end"],
};

const UI_PUSH_ID_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "push_id",
    full_name: "ui.push_id",
    summary: "Push an explicit identity scope for stateful widgets.",
    args: &UI_PUSH_ID_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const UI_POP_ID_FUNCTION: RuntimeApiFunctionSpec = RuntimeApiFunctionSpec {
    namespace: "ui",
    member: "pop_id",
    full_name: "ui.pop_id",
    summary: "Pop the most recently pushed identity scope.",
    args: &NO_ARGS,
    returns: &NO_RETURNS,
    phase_rule: RuntimeApiPhaseRule::RenderOnly,
    capability: None,
    compatibility_aliases: &[],
};

const ALL_FUNCTIONS: [RuntimeApiFunctionSpec; 28] = [
    APP_LOG_FUNCTION,
    APP_REQUEST_RELOAD_FUNCTION,
    APP_REQUEST_REPAINT_FUNCTION,
    UI_LABEL_FUNCTION,
    UI_SEPARATOR_FUNCTION,
    UI_BUTTON_FUNCTION,
    UI_TEXT_EDIT_FUNCTION,
    UI_CHECKBOX_FUNCTION,
    UI_SWITCH_FUNCTION,
    UI_SLIDER_FUNCTION,
    UI_NUMBER_INPUT_FUNCTION,
    UI_SELECT_FUNCTION,
    UI_TABS_FUNCTION,
    UI_PROGRESS_FUNCTION,
    UI_RADIO_FUNCTION,
    UI_BUTTON_GROUP_FUNCTION,
    UI_BEGIN_COLLAPSIBLE_FUNCTION,
    UI_DROPDOWN_MENU_FUNCTION,
    UI_TOOLTIP_FUNCTION,
    UI_SPINNER_FUNCTION,
    UI_SKELETON_FUNCTION,
    UI_VIRTUAL_LIST_FUNCTION,
    UI_BEGIN_ROW_FUNCTION,
    UI_BEGIN_COLUMN_FUNCTION,
    UI_BEGIN_CARD_FUNCTION,
    UI_END_SCOPE_FUNCTION,
    UI_PUSH_ID_FUNCTION,
    UI_POP_ID_FUNCTION,
];

pub(super) fn function_spec(id: RuntimeApiFunctionId) -> &'static RuntimeApiFunctionSpec {
    match id {
        RuntimeApiFunctionId::AppLog => &APP_LOG_FUNCTION,
        RuntimeApiFunctionId::AppRequestReload => &APP_REQUEST_RELOAD_FUNCTION,
        RuntimeApiFunctionId::AppRequestRepaint => &APP_REQUEST_REPAINT_FUNCTION,
        RuntimeApiFunctionId::UiLabel => &UI_LABEL_FUNCTION,
        RuntimeApiFunctionId::UiSeparator => &UI_SEPARATOR_FUNCTION,
        RuntimeApiFunctionId::UiButton => &UI_BUTTON_FUNCTION,
        RuntimeApiFunctionId::UiTextEdit => &UI_TEXT_EDIT_FUNCTION,
        RuntimeApiFunctionId::UiCheckbox => &UI_CHECKBOX_FUNCTION,
        RuntimeApiFunctionId::UiSwitch => &UI_SWITCH_FUNCTION,
        RuntimeApiFunctionId::UiSlider => &UI_SLIDER_FUNCTION,
        RuntimeApiFunctionId::UiNumberInput => &UI_NUMBER_INPUT_FUNCTION,
        RuntimeApiFunctionId::UiSelect => &UI_SELECT_FUNCTION,
        RuntimeApiFunctionId::UiTabs => &UI_TABS_FUNCTION,
        RuntimeApiFunctionId::UiProgress => &UI_PROGRESS_FUNCTION,
        RuntimeApiFunctionId::UiRadio => &UI_RADIO_FUNCTION,
        RuntimeApiFunctionId::UiButtonGroup => &UI_BUTTON_GROUP_FUNCTION,
        RuntimeApiFunctionId::UiBeginCollapsible => &UI_BEGIN_COLLAPSIBLE_FUNCTION,
        RuntimeApiFunctionId::UiDropdownMenu => &UI_DROPDOWN_MENU_FUNCTION,
        RuntimeApiFunctionId::UiTooltip => &UI_TOOLTIP_FUNCTION,
        RuntimeApiFunctionId::UiSpinner => &UI_SPINNER_FUNCTION,
        RuntimeApiFunctionId::UiSkeleton => &UI_SKELETON_FUNCTION,
        RuntimeApiFunctionId::UiVirtualList => &UI_VIRTUAL_LIST_FUNCTION,
        RuntimeApiFunctionId::UiBeginRow => &UI_BEGIN_ROW_FUNCTION,
        RuntimeApiFunctionId::UiBeginColumn => &UI_BEGIN_COLUMN_FUNCTION,
        RuntimeApiFunctionId::UiBeginCard => &UI_BEGIN_CARD_FUNCTION,
        RuntimeApiFunctionId::UiEndScope => &UI_END_SCOPE_FUNCTION,
        RuntimeApiFunctionId::UiPushId => &UI_PUSH_ID_FUNCTION,
        RuntimeApiFunctionId::UiPopId => &UI_POP_ID_FUNCTION,
    }
}

pub(super) fn all_function_specs() -> &'static [RuntimeApiFunctionSpec] {
    &ALL_FUNCTIONS
}

pub(super) fn prop_schema_spec(id: RuntimeApiPropSchemaId) -> RuntimeApiPropSchemaSpec {
    match id {
        RuntimeApiPropSchemaId::LabelProps => RuntimeApiPropSchemaSpec {
            name: "LabelProps",
            fields: &LABEL_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::ButtonProps => RuntimeApiPropSchemaSpec {
            name: "ButtonProps",
            fields: &BUTTON_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::TextEditProps => RuntimeApiPropSchemaSpec {
            name: "TextEditProps",
            fields: &TEXT_EDIT_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::CheckboxProps => RuntimeApiPropSchemaSpec {
            name: "CheckboxProps",
            fields: &CHECKBOX_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::SwitchProps => RuntimeApiPropSchemaSpec {
            name: "SwitchProps",
            fields: &SWITCH_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::SliderProps => RuntimeApiPropSchemaSpec {
            name: "SliderProps",
            fields: &SLIDER_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::NumberInputProps => RuntimeApiPropSchemaSpec {
            name: "NumberInputProps",
            fields: &NUMBER_INPUT_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::SelectProps => RuntimeApiPropSchemaSpec {
            name: "SelectProps",
            fields: &SELECT_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::TabsProps => RuntimeApiPropSchemaSpec {
            name: "TabsProps",
            fields: &TABS_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::ProgressProps => RuntimeApiPropSchemaSpec {
            name: "ProgressProps",
            fields: &PROGRESS_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::RadioProps => RuntimeApiPropSchemaSpec {
            name: "RadioProps",
            fields: &RADIO_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::ButtonGroupProps => RuntimeApiPropSchemaSpec {
            name: "ButtonGroupProps",
            fields: &BUTTON_GROUP_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::CollapsibleProps => RuntimeApiPropSchemaSpec {
            name: "CollapsibleProps",
            fields: &COLLAPSIBLE_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::DropdownMenuProps => RuntimeApiPropSchemaSpec {
            name: "DropdownMenuProps",
            fields: &DROPDOWN_MENU_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::TooltipProps => RuntimeApiPropSchemaSpec {
            name: "TooltipProps",
            fields: &TOOLTIP_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::SpinnerProps => RuntimeApiPropSchemaSpec {
            name: "SpinnerProps",
            fields: &SPINNER_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::SkeletonProps => RuntimeApiPropSchemaSpec {
            name: "SkeletonProps",
            fields: &SKELETON_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::VirtualListProps => RuntimeApiPropSchemaSpec {
            name: "VirtualListProps",
            fields: &VIRTUAL_LIST_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::ContainerProps => RuntimeApiPropSchemaSpec {
            name: "ContainerProps",
            fields: &CONTAINER_PROPS_FIELDS,
        },
        RuntimeApiPropSchemaId::CardProps => RuntimeApiPropSchemaSpec {
            name: "CardProps",
            fields: &CARD_PROPS_FIELDS,
        },
    }
}

pub(super) fn enum_spec(id: RuntimeApiEnumId) -> RuntimeApiEnumSpec {
    match id {
        RuntimeApiEnumId::LabelTone => RuntimeApiEnumSpec {
            name: "LabelTone",
            values: &LABEL_TONE_VALUES,
        },
        RuntimeApiEnumId::LabelWeight => RuntimeApiEnumSpec {
            name: "LabelWeight",
            values: &LABEL_WEIGHT_VALUES,
        },
        RuntimeApiEnumId::ButtonVariant => RuntimeApiEnumSpec {
            name: "ButtonVariant",
            values: &BUTTON_VARIANT_VALUES,
        },
        RuntimeApiEnumId::ControlSize => RuntimeApiEnumSpec {
            name: "ControlSize",
            values: &CONTROL_SIZE_VALUES,
        },
        RuntimeApiEnumId::NumberInputAxis => RuntimeApiEnumSpec {
            name: "NumberInputAxis",
            values: &NUMBER_INPUT_AXIS_VALUES,
        },
        RuntimeApiEnumId::SelectVariant => RuntimeApiEnumSpec {
            name: "SelectVariant",
            values: &SELECT_VARIANT_VALUES,
        },
        RuntimeApiEnumId::TabsVariant => RuntimeApiEnumSpec {
            name: "TabsVariant",
            values: &TABS_VARIANT_VALUES,
        },
        RuntimeApiEnumId::TooltipPlacement => RuntimeApiEnumSpec {
            name: "TooltipPlacement",
            values: &TOOLTIP_PLACEMENT_VALUES,
        },
        RuntimeApiEnumId::SkeletonShape => RuntimeApiEnumSpec {
            name: "SkeletonShape",
            values: &SKELETON_SHAPE_VALUES,
        },
    }
}

pub(super) fn named_type_spec(id: RuntimeApiNamedTypeId) -> RuntimeApiNamedTypeSpec {
    match id {
        RuntimeApiNamedTypeId::SelectOption => RuntimeApiNamedTypeSpec::Object {
            name: "SelectOption",
            fields: &SELECT_OPTION_FIELDS,
        },
        RuntimeApiNamedTypeId::TabOption => RuntimeApiNamedTypeSpec::Object {
            name: "TabOption",
            fields: &TAB_OPTION_FIELDS,
        },
        RuntimeApiNamedTypeId::ButtonGroupOption => RuntimeApiNamedTypeSpec::Object {
            name: "ButtonGroupOption",
            fields: &BUTTON_GROUP_OPTION_FIELDS,
        },
        RuntimeApiNamedTypeId::DropdownMenuEntry => RuntimeApiNamedTypeSpec::Alias {
            name: "DropdownMenuEntry",
            luau: "{ kind: \"action\", id: number, label: string, icon: string?, shortcut: string?, enabled: boolean?, selected: boolean? } | { kind: \"separator\" } | { kind: \"submenu\", label: string, icon: string?, entries: {DropdownMenuEntry} }",
            markdown: "`{ kind = \"action\", id: number, label: string, icon?: string, shortcut?: string, enabled?: boolean, selected?: boolean }`\n\n`{ kind = \"separator\" }`\n\n`{ kind = \"submenu\", label: string, icon?: string, entries: {DropdownMenuEntry} }`",
        },
    }
}

pub(super) fn resolve_enum_canonical(id: RuntimeApiEnumId, value: &str) -> Option<&'static str> {
    let normalized = value.to_ascii_lowercase();
    enum_spec(id).values.iter().find_map(|candidate| {
        if candidate.canonical == normalized
            || candidate.aliases.iter().any(|alias| *alias == normalized)
        {
            Some(candidate.canonical)
        } else {
            None
        }
    })
}

pub(super) fn parse_label_tone(value: &str) -> Option<UiLabelTone> {
    match resolve_enum_canonical(RuntimeApiEnumId::LabelTone, value)? {
        "primary" => Some(UiLabelTone::Primary),
        "secondary" => Some(UiLabelTone::Secondary),
        "muted" => Some(UiLabelTone::Muted),
        "destructive" => Some(UiLabelTone::Destructive),
        _ => None,
    }
}

pub(super) fn parse_label_weight(value: &str) -> Option<UiLabelWeight> {
    match resolve_enum_canonical(RuntimeApiEnumId::LabelWeight, value)? {
        "regular" => Some(UiLabelWeight::Regular),
        "semibold" => Some(UiLabelWeight::Semibold),
        "bold" => Some(UiLabelWeight::Bold),
        _ => None,
    }
}

pub(super) fn parse_button_variant(value: &str) -> Option<UiButtonVariant> {
    match resolve_enum_canonical(RuntimeApiEnumId::ButtonVariant, value)? {
        "primary" => Some(UiButtonVariant::Primary),
        "secondary" => Some(UiButtonVariant::Secondary),
        "ghost" => Some(UiButtonVariant::Ghost),
        "link" => Some(UiButtonVariant::Link),
        _ => None,
    }
}

pub(super) fn parse_control_size(value: &str) -> Option<UiControlSize> {
    match resolve_enum_canonical(RuntimeApiEnumId::ControlSize, value)? {
        "sm" => Some(UiControlSize::Sm),
        "md" => Some(UiControlSize::Md),
        _ => None,
    }
}

pub(super) fn parse_number_input_axis(value: &str) -> Option<UiNumberInputAxis> {
    match resolve_enum_canonical(RuntimeApiEnumId::NumberInputAxis, value)? {
        "horizontal" => Some(UiNumberInputAxis::Horizontal),
        "vertical" => Some(UiNumberInputAxis::Vertical),
        _ => None,
    }
}

pub(super) fn parse_select_variant(value: &str) -> Option<UiSelectVariant> {
    match resolve_enum_canonical(RuntimeApiEnumId::SelectVariant, value)? {
        "default" => Some(UiSelectVariant::Default),
        "secondary" => Some(UiSelectVariant::Secondary),
        _ => None,
    }
}

pub(super) fn parse_tabs_variant(value: &str) -> Option<UiTabsVariant> {
    match resolve_enum_canonical(RuntimeApiEnumId::TabsVariant, value)? {
        "underline" => Some(UiTabsVariant::Underline),
        "segmented" => Some(UiTabsVariant::Segmented),
        "stacked" => Some(UiTabsVariant::Stacked),
        "rail" => Some(UiTabsVariant::Rail),
        "blender_topbar" => Some(UiTabsVariant::BlenderTopbar),
        _ => None,
    }
}

pub(super) fn parse_tooltip_placement(value: &str) -> Option<UiTooltipPlacement> {
    match resolve_enum_canonical(RuntimeApiEnumId::TooltipPlacement, value)? {
        "auto" => Some(UiTooltipPlacement::Auto),
        "top" => Some(UiTooltipPlacement::Top),
        "right" => Some(UiTooltipPlacement::Right),
        "bottom" => Some(UiTooltipPlacement::Bottom),
        "left" => Some(UiTooltipPlacement::Left),
        _ => None,
    }
}

pub(super) fn parse_skeleton_shape(value: &str) -> Option<UiSkeletonShape> {
    match resolve_enum_canonical(RuntimeApiEnumId::SkeletonShape, value)? {
        "rect" => Some(UiSkeletonShape::Rect),
        "circle" => Some(UiSkeletonShape::Circle),
        _ => None,
    }
}

pub fn runtime_api_reference_markdown() -> String {
    let mut markdown = String::from(
        "# Luau Runtime API Reference\n\nGenerated from `luau_runtime_core::runtime_api_reference_markdown()` and `luau_runtime_core::runtime_api_luau_typings()`.\n\nRefresh with:\n\n```bash\ncargo run -p luau-runtime-core --bin generate-runtime-api\n```\n\nGenerated artifacts:\n\n- [`examples/runtime-luau/ui/core/types.luau`](../examples/runtime-luau/ui/core/types.luau)\n- [`docs/luau-runtime-api-reference.md`](./luau-runtime-api-reference.md)\n\nThe host mounts frame-local `app` and `ui` globals for Luau scripts. This document covers that generated bridge surface only; lifecycle hooks such as `init`, `update`, `render`, `reload`, and `shutdown` remain documented in `examples/runtime-luau/README.md`.\n\n## Functions\n\n",
    );

    for namespace in ["app", "ui"] {
        let _ = writeln!(markdown, "### `{namespace}`\n");
        for function in all_function_specs()
            .iter()
            .filter(|function| function.namespace == namespace)
        {
            let _ = writeln!(markdown, "#### `{}`\n", render_markdown_signature(function));
            let _ = writeln!(markdown, "{}\n", function.summary);
            let _ = writeln!(
                markdown,
                "- Phase rule: {}",
                function.phase_rule.description()
            );
            if let Some(capability) = function.capability {
                let _ = writeln!(markdown, "- Capability: `{}`", capability.as_str());
            }
            if !function.returns.is_empty() {
                let _ = writeln!(
                    markdown,
                    "- Returns: {}",
                    render_named_returns(function.returns)
                );
            }
            if !function.compatibility_aliases.is_empty() {
                let _ = writeln!(
                    markdown,
                    "- Compatibility aliases accepted at runtime: {}",
                    render_string_list(function.compatibility_aliases)
                );
            }
            markdown.push('\n');
        }
    }

    markdown.push_str("## Prop Schemas\n\n");
    for schema_id in [
        RuntimeApiPropSchemaId::LabelProps,
        RuntimeApiPropSchemaId::ButtonProps,
        RuntimeApiPropSchemaId::TextEditProps,
        RuntimeApiPropSchemaId::CheckboxProps,
        RuntimeApiPropSchemaId::SwitchProps,
        RuntimeApiPropSchemaId::SliderProps,
        RuntimeApiPropSchemaId::NumberInputProps,
        RuntimeApiPropSchemaId::SelectProps,
        RuntimeApiPropSchemaId::TabsProps,
        RuntimeApiPropSchemaId::ProgressProps,
        RuntimeApiPropSchemaId::RadioProps,
        RuntimeApiPropSchemaId::ButtonGroupProps,
        RuntimeApiPropSchemaId::CollapsibleProps,
        RuntimeApiPropSchemaId::DropdownMenuProps,
        RuntimeApiPropSchemaId::TooltipProps,
        RuntimeApiPropSchemaId::SpinnerProps,
        RuntimeApiPropSchemaId::SkeletonProps,
        RuntimeApiPropSchemaId::VirtualListProps,
        RuntimeApiPropSchemaId::ContainerProps,
        RuntimeApiPropSchemaId::CardProps,
    ] {
        let schema = prop_schema_spec(schema_id);
        let _ = writeln!(markdown, "### `{}`\n", schema.name);
        markdown.push_str("| Field | Type | Default |\n| --- | --- | --- |\n");
        for field in schema.fields {
            let default = field.default.unwrap_or("none");
            let _ = writeln!(
                markdown,
                "| `{}` | `{}` | `{}` |",
                field.name,
                render_type_ref(field.ty, field.optional),
                default
            );
        }
        markdown.push('\n');
    }

    markdown.push_str("## Structured Types\n\n");
    for named_id in [
        RuntimeApiNamedTypeId::SelectOption,
        RuntimeApiNamedTypeId::TabOption,
        RuntimeApiNamedTypeId::ButtonGroupOption,
        RuntimeApiNamedTypeId::DropdownMenuEntry,
    ] {
        match named_type_spec(named_id) {
            RuntimeApiNamedTypeSpec::Object { name, fields } => {
                let _ = writeln!(markdown, "### `{name}`\n");
                markdown.push_str("| Field | Type | Default |\n| --- | --- | --- |\n");
                for field in fields {
                    let default = field.default.unwrap_or("none");
                    let _ = writeln!(
                        markdown,
                        "| `{}` | `{}` | `{}` |",
                        field.name,
                        render_type_ref(field.ty, field.optional),
                        default
                    );
                }
                markdown.push('\n');
            }
            RuntimeApiNamedTypeSpec::Alias {
                name,
                markdown: body,
                ..
            } => {
                let _ = writeln!(markdown, "### `{name}`\n");
                let _ = writeln!(markdown, "{body}\n");
            }
        }
    }

    markdown.push_str("## Enum Tokens\n\n");
    for enum_id in [
        RuntimeApiEnumId::LabelTone,
        RuntimeApiEnumId::LabelWeight,
        RuntimeApiEnumId::ButtonVariant,
        RuntimeApiEnumId::ControlSize,
        RuntimeApiEnumId::NumberInputAxis,
        RuntimeApiEnumId::SelectVariant,
        RuntimeApiEnumId::TabsVariant,
        RuntimeApiEnumId::TooltipPlacement,
        RuntimeApiEnumId::SkeletonShape,
    ] {
        let enum_spec = enum_spec(enum_id);
        let _ = writeln!(
            markdown,
            "### `type {} = {}`\n",
            enum_spec.name,
            render_enum_union(&enum_spec)
        );
        let compatibility_aliases = render_enum_aliases(&enum_spec);
        if compatibility_aliases.is_empty() {
            markdown.push_str("Generated typings use canonical values only. The runtime parser is case-insensitive.\n\n");
        } else {
            let _ = writeln!(
                markdown,
                "Generated typings use canonical values only. The runtime parser is case-insensitive and also accepts: {}.\n",
                compatibility_aliases
            );
        }
    }

    markdown
}

pub fn runtime_api_luau_typings() -> String {
    let mut typings = String::from(
        "--!strict\n-- AUTO-GENERATED by `cargo run -p luau-runtime-core --bin generate-runtime-api`.\n-- DO NOT EDIT BY HAND.\n\n",
    );

    for enum_id in [
        RuntimeApiEnumId::LabelTone,
        RuntimeApiEnumId::LabelWeight,
        RuntimeApiEnumId::ButtonVariant,
        RuntimeApiEnumId::ControlSize,
        RuntimeApiEnumId::NumberInputAxis,
        RuntimeApiEnumId::SelectVariant,
        RuntimeApiEnumId::TabsVariant,
        RuntimeApiEnumId::TooltipPlacement,
        RuntimeApiEnumId::SkeletonShape,
    ] {
        let enum_spec = enum_spec(enum_id);
        let _ = writeln!(
            typings,
            "export type {} = {}\n",
            enum_spec.name,
            render_enum_union(&enum_spec)
        );
    }

    for schema_id in [
        RuntimeApiPropSchemaId::LabelProps,
        RuntimeApiPropSchemaId::ButtonProps,
        RuntimeApiPropSchemaId::TextEditProps,
        RuntimeApiPropSchemaId::CheckboxProps,
        RuntimeApiPropSchemaId::SwitchProps,
        RuntimeApiPropSchemaId::SliderProps,
        RuntimeApiPropSchemaId::NumberInputProps,
        RuntimeApiPropSchemaId::SelectProps,
        RuntimeApiPropSchemaId::TabsProps,
        RuntimeApiPropSchemaId::ProgressProps,
        RuntimeApiPropSchemaId::RadioProps,
        RuntimeApiPropSchemaId::ButtonGroupProps,
        RuntimeApiPropSchemaId::CollapsibleProps,
        RuntimeApiPropSchemaId::DropdownMenuProps,
        RuntimeApiPropSchemaId::TooltipProps,
        RuntimeApiPropSchemaId::SpinnerProps,
        RuntimeApiPropSchemaId::SkeletonProps,
        RuntimeApiPropSchemaId::VirtualListProps,
        RuntimeApiPropSchemaId::ContainerProps,
        RuntimeApiPropSchemaId::CardProps,
    ] {
        let schema = prop_schema_spec(schema_id);
        let _ = writeln!(typings, "export type {} = {{", schema.name);
        for field in schema.fields {
            let _ = writeln!(
                typings,
                "    {}: {},",
                field.name,
                render_type_ref(field.ty, field.optional)
            );
        }
        typings.push_str("}\n\n");
    }

    for named_id in [
        RuntimeApiNamedTypeId::SelectOption,
        RuntimeApiNamedTypeId::TabOption,
        RuntimeApiNamedTypeId::ButtonGroupOption,
        RuntimeApiNamedTypeId::DropdownMenuEntry,
    ] {
        match named_type_spec(named_id) {
            RuntimeApiNamedTypeSpec::Object { name, fields } => {
                let _ = writeln!(typings, "export type {name} = {{");
                for field in fields {
                    let _ = writeln!(
                        typings,
                        "    {}: {},",
                        field.name,
                        render_type_ref(field.ty, field.optional)
                    );
                }
                typings.push_str("}\n\n");
            }
            RuntimeApiNamedTypeSpec::Alias { name, luau, .. } => {
                let _ = writeln!(typings, "export type {name} = {luau}\n");
            }
        }
    }

    for namespace in ["App", "Ui"] {
        let table_name = namespace.to_ascii_lowercase();
        let _ = writeln!(typings, "export type {namespace} = {{");
        for function in all_function_specs()
            .iter()
            .filter(|function| function.namespace == table_name)
        {
            let _ = writeln!(
                typings,
                "    {}: {},",
                function.member,
                render_luau_function_type(function)
            );
        }
        typings.push_str("}\n\n");
    }

    typings.push_str("return {}\n");
    typings
}

fn render_markdown_signature(function: &RuntimeApiFunctionSpec) -> String {
    format!(
        "{}({}) -> {}",
        function.full_name,
        function
            .args
            .iter()
            .map(|arg| format!("{}: {}", arg.name, render_type_ref(arg.ty, arg.optional)))
            .collect::<Vec<_>>()
            .join(", "),
        render_return_types(function.returns)
    )
}

fn render_luau_function_type(function: &RuntimeApiFunctionSpec) -> String {
    format!(
        "({}) -> {}",
        function
            .args
            .iter()
            .map(|arg| format!("{}: {}", arg.name, render_type_ref(arg.ty, arg.optional)))
            .collect::<Vec<_>>()
            .join(", "),
        render_return_types(function.returns)
    )
}

fn render_return_types(returns: &[RuntimeApiReturnSpec]) -> String {
    match returns {
        [] => "()".to_owned(),
        [single] => render_type_ref(single.ty, false),
        many => format!(
            "({})",
            many.iter()
                .map(|item| render_type_ref(item.ty, false))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn render_named_returns(returns: &[RuntimeApiReturnSpec]) -> String {
    returns
        .iter()
        .map(|item| match item.name {
            Some(name) => format!("`{name}: {}`", render_type_ref(item.ty, false)),
            None => format!("`{}`", render_type_ref(item.ty, false)),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_type_ref(ty: RuntimeApiTypeRef, optional: bool) -> String {
    let mut rendered = match ty {
        RuntimeApiTypeRef::String => "string".to_owned(),
        RuntimeApiTypeRef::StringArray => "{string}".to_owned(),
        RuntimeApiTypeRef::Number => "number".to_owned(),
        RuntimeApiTypeRef::Boolean => "boolean".to_owned(),
        RuntimeApiTypeRef::PropSchema(schema) => prop_schema_spec(schema).name.to_owned(),
        RuntimeApiTypeRef::Enum(enum_id) => enum_spec(enum_id).name.to_owned(),
        RuntimeApiTypeRef::Named(id) => match named_type_spec(id) {
            RuntimeApiNamedTypeSpec::Object { name, .. } => name.to_owned(),
            RuntimeApiNamedTypeSpec::Alias { name, .. } => name.to_owned(),
        },
        RuntimeApiTypeRef::NamedArray(id) => {
            format!(
                "{{{}}}",
                render_type_ref(RuntimeApiTypeRef::Named(id), false)
            )
        }
    };
    if optional {
        rendered.push('?');
    }
    rendered
}

fn render_enum_union(enum_spec: &RuntimeApiEnumSpec) -> String {
    enum_spec
        .values
        .iter()
        .map(|value| format!("\"{}\"", value.canonical))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn render_enum_aliases(enum_spec: &RuntimeApiEnumSpec) -> String {
    enum_spec
        .values
        .iter()
        .filter(|value| !value.aliases.is_empty())
        .map(|value| {
            format!(
                "{} -> `{}`",
                render_string_list(value.aliases),
                value.canonical
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_string_list(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}
