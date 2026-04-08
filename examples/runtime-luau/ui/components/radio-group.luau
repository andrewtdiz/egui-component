--!strict

local scope = require("../scope")
local text = require("../text")
local state_store = require("./state")
local button = require("./button").api
local radio = require("./radio").api

type RadioGroupOption = {
    label: string,
    description: string?,
}

local RADIO_OPTIONS = table.freeze({
    {
        label = "Starter",
        description = "Basic surfaces and controls for smaller tools.",
    },
    {
        label = "Team",
        description = "Shared tokens, overrides, and example screens.",
    },
    {
        label = "Enterprise",
        description = "Extended theming, audit trails, and environment presets.",
    },
})

local api = {}

function api.render(
    id: string,
    current_index: number?,
    options: { RadioGroupOption },
    gap: number?
): (number, boolean)
    return radio.group(id, current_index, options, gap)
end

local module = {
    api = api,
    id = "radio-group",
    label = "Radio Group",
    section = "inputs",
    summary = "Mutually exclusive option lists with per-row descriptions and clear selection copy.",
}

function module.render_preview(state: any)
    local gallery = state.gallery
    local next_index, changed = api.render("account_plan", gallery.radio_group_plan, RADIO_OPTIONS, 8)
    if changed then
        gallery.radio_group_plan = next_index
    end
    scope.row({ gap = 8 }, function()
        if button.render("radio_group_clear", "Clear", {
            variant = "secondary",
        }) then
            gallery.radio_group_plan = 0
        end
        local selected = RADIO_OPTIONS[gallery.radio_group_plan]
        text.meta(if selected ~= nil then selected.label else "No option selected")
    end)
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
