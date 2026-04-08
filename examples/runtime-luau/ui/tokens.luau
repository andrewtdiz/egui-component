--!strict

local ui_types = require("./types")

local tokens: {
    spacing: {
        xs: number,
        sm: number,
        md: number,
        lg: number,
    },
    width: {
        input: number,
        panel: number,
        sidebar: number,
        showcase: number,
    },
    card_panel: ui_types.CardProps,
    card_showcase: ui_types.CardProps,
    card_sidebar: ui_types.CardProps,
    kicker: ui_types.LabelProps,
    title: ui_types.LabelProps,
    meta: ui_types.LabelProps,
    field_label: ui_types.LabelProps,
    field_description: ui_types.LabelProps,
    primary_button: ui_types.ButtonProps,
    secondary_button: ui_types.ButtonProps,
    ghost_button: ui_types.ButtonProps,
    link_button: ui_types.ButtonProps,
    input: ui_types.TextEditProps,
    select_default: ui_types.SelectProps,
    select_secondary: ui_types.SelectProps,
    tabs_segmented: ui_types.TabsProps,
    progress_bar: ui_types.ProgressProps,
    spinner: ui_types.SpinnerProps,
    skeleton_line: ui_types.SkeletonProps,
    skeleton_avatar: ui_types.SkeletonProps,
} = {
    spacing = {
        xs = 8,
        sm = 12,
        md = 16,
        lg = 20,
    },
    width = {
        input = 280,
        panel = 360,
        sidebar = 320,
        showcase = 640,
    },
    card_panel = {
        width = 360,
        padding_x = 20,
        padding_y = 20,
    },
    card_showcase = {
        width = 640,
        padding_x = 20,
        padding_y = 20,
    },
    card_sidebar = {
        width = 320,
        padding_x = 18,
        padding_y = 18,
    },
    kicker = {
        tone = "muted",
        size = 12,
    },
    title = {
        weight = "bold",
        size = 22,
    },
    meta = {
        tone = "muted",
    },
    field_label = {
        tone = "secondary",
        weight = "semibold",
        size = 14,
    },
    field_description = {
        tone = "muted",
        size = 12,
    },
    primary_button = {
        variant = "primary",
        width = 120,
    },
    secondary_button = {
        variant = "secondary",
        width = 120,
    },
    ghost_button = {
        variant = "ghost",
        width = 120,
    },
    link_button = {
        variant = "link",
    },
    input = {
        width = 280,
        placeholder = "Enter text",
    },
    select_default = {
        width = 220,
    },
    select_secondary = {
        width = 220,
        variant = "secondary",
    },
    tabs_segmented = {
        variant = "segmented",
    },
    progress_bar = {
        width = 320,
        height = 10,
    },
    spinner = {
        size = 18,
    },
    skeleton_line = {
        width = 240,
        height = 12,
        shape = "rect",
    },
    skeleton_avatar = {
        width = 48,
        height = 48,
        shape = "circle",
    },
}

return tokens
