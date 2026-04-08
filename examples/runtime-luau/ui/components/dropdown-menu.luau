--!strict

local ui_types = require("../types")
local bridge = require("../bridge")
local text = require("../text")
local state_store = require("./state")

type DropdownMenuEntry = ui_types.DropdownMenuEntry
type DropdownMenuProps = ui_types.DropdownMenuProps

local api = {}

local function action_entry(
    id: number,
    label: string,
    shortcut: string?
): DropdownMenuEntry
    return {
        kind = "action",
        id = id,
        label = label,
        shortcut = shortcut,
    }
end

local function submenu_entry(label: string, entries: { DropdownMenuEntry }): DropdownMenuEntry
    return {
        kind = "submenu",
        label = label,
        entries = entries,
    }
end

local function separator_entry(): DropdownMenuEntry
    return {
        kind = "separator",
    }
end

local INVITE_USERS_ENTRIES: { DropdownMenuEntry } = table.freeze({
    action_entry(5, "Email"),
    action_entry(6, "Message"),
    separator_entry(),
    action_entry(7, "More..."),
})

local MENU_ENTRIES: { DropdownMenuEntry } = table.freeze({
    action_entry(1, "My Account"),
    action_entry(2, "Profile", "Shift+Cmd+P"),
    action_entry(3, "Billing", "Cmd+B"),
    action_entry(4, "Settings", "Cmd+S"),
    separator_entry(),
    submenu_entry("Invite users", INVITE_USERS_ENTRIES),
    action_entry(8, "New Team", "Cmd+T"),
    separator_entry(),
    action_entry(9, "GitHub"),
    action_entry(10, "Support"),
    action_entry(11, "API"),
    separator_entry(),
    action_entry(12, "Log out", "Shift+Cmd+Q"),
    action_entry(13, "Delete"),
})

local MENU_ACTION_LABELS = table.freeze({
    [1] = "My Account",
    [2] = "Profile",
    [3] = "Billing",
    [4] = "Settings",
    [5] = "Email",
    [6] = "Message",
    [7] = "More...",
    [8] = "New Team",
    [9] = "GitHub",
    [10] = "Support",
    [11] = "API",
    [12] = "Log out",
    [13] = "Delete",
})

function api.render(
    id: string,
    trigger_label: string,
    entries: { DropdownMenuEntry },
    props: DropdownMenuProps?
): (number, boolean)
    return bridge.current_ui().dropdown_menu(id, trigger_label, entries, props)
end

local module = {
    api = api,
    id = "dropdown-menu",
    label = "Dropdown Menu",
    section = "overlays",
    summary = "Typed action lists, separators, shortcuts, and nested submenus through one API.",
}

function module.render_preview(state: any)
    local gallery = state.gallery
    local action_id, changed = api.render("gallery_menu", "Open", MENU_ENTRIES, {
        width = 220,
    })
    if changed then
        gallery.dropdown_menu_action = action_id
        gallery.dropdown_menu_label = MENU_ACTION_LABELS[action_id] or "No action triggered"
    end
    text.meta(tostring(gallery.dropdown_menu_label))
end

function module.init(state: any)
    state_store.init(state)
end

function module.update(state: any, _input: any)
    state_store.update(state)
end

function module.render(state: any)
    state_store.init(state)
    module.render_preview(state)
end

return module
