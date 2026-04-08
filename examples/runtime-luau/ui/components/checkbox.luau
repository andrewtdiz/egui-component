--!strict

local ui_types = require("../types")
local bridge = require("../bridge")
local state_store = require("./state")

type CheckboxProps = ui_types.CheckboxProps

local api = {}

function api.render(id: string, checked: boolean, props: CheckboxProps?): (boolean, boolean)
    return bridge.current_ui().checkbox(id, checked, props)
end

local module = {
    api = api,
    id = "checkbox",
    label = "Checkbox",
    section = "inputs",
    summary = "Boolean selection with a typed label prop and stable widget identity.",
}

function module.render_preview(state: any)
    local gallery = state.gallery
    local next_value, changed = api.render("receive_shadows", gallery.checkbox_receive_shadows, {
        label = "Receive Shadows",
    })
    if changed then
        gallery.checkbox_receive_shadows = next_value
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
