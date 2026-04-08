--!strict

local ui_types = require("../core/types")
local bridge = require("../core/bridge")
local state_store = require("./catalog_state")

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
    local catalog = state.catalog
    local next_value, changed = api.render("receive_shadows", catalog.checkbox_receive_shadows, {
        label = "Receive Shadows",
    })
    if changed then
        catalog.checkbox_receive_shadows = next_value
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
