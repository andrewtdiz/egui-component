--!strict

local ui_types = require("../core/types")
local bridge = require("../core/bridge")
local state_store = require("./catalog_state")

type SwitchProps = ui_types.SwitchProps

local api = {}

function api.render(id: string, checked: boolean, props: SwitchProps?): (boolean, boolean)
    return bridge.current_ui().switch(id, checked, props)
end

local module = {
    api = api,
    id = "switch",
    label = "Switch",
    section = "inputs",
    summary = "Secondary boolean controls with size variants and descriptive labels.",
}

function module.render_preview(state: any)
    local catalog = state.catalog

    local next_switch, changed = api.render("post_fx_toggle", catalog.switch_enable_post_fx, {
        label = "Enable Post FX",
        size = "md",
    })
    if changed then
        catalog.switch_enable_post_fx = next_switch
    end

    local next_small_switch, small_changed = api.render(
        "compact_handles_toggle",
        catalog.switch_compact_handles,
        {
            label = "Use Compact Handles",
            size = "sm",
        }
    )
    if small_changed then
        catalog.switch_compact_handles = next_small_switch
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
