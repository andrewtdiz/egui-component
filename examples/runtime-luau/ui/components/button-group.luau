--!strict

local ui_types = require("../types")
local bridge = require("../bridge")
local state_store = require("./state")

type ButtonGroupOption = ui_types.ButtonGroupOption
type ButtonGroupProps = ui_types.ButtonGroupProps

local ACTION_GROUP_OPTIONS = table.freeze({
    { label = "Move" },
    { label = "Rotate" },
    { label = "Scale" },
})

local api = {}

function api.render(
    id: string,
    options: { ButtonGroupOption },
    props: ButtonGroupProps?
): (number, boolean)
    return bridge.current_ui().button_group(id, options, props)
end

local module = {
    api = api,
    id = "button-group",
    label = "Button Group",
    section = "actions",
    summary = "Attached action strips that return a 1-based selection index when clicked.",
}

function module.render_preview(state: any)
    local gallery = state.gallery
    local clicked_group, changed = api.render("gallery_actions", ACTION_GROUP_OPTIONS)
    if changed then
        gallery.button_group_transform_mode = clicked_group
    end
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
