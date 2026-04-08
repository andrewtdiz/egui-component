--!strict

local ui_types = require("../types")
local bridge = require("../bridge")
local scope = require("../scope")
local text = require("../text")
local state_store = require("./state")
local button = require("./button").api

type RadioProps = ui_types.RadioProps

export type RadioGroupOption = {
    label: string,
    description: string?,
}

local api = {}

function api.render(id: string, selected: boolean, props: RadioProps?): (boolean, boolean)
    return bridge.current_ui().radio(id, selected, props)
end

function api.group(
    id: string,
    current_index: number?,
    options: { RadioGroupOption },
    gap: number?
): (number, boolean)
    local next_index = current_index or 0
    local changed = false

    scope.with_id(id, function()
        scope.column({ gap = gap or 8 }, function()
            for index, option in ipairs(options) do
                local selected, selected_changed = api.render(
                    "option_" .. tostring(index),
                    next_index == index,
                    {
                        label = option.label,
                        description = option.description,
                    }
                )
                if selected_changed and selected then
                    next_index = index
                    changed = true
                end
            end
        end)
    end)

    return next_index, changed
end

local module = {
    api = api,
    id = "radio",
    label = "Radio",
    section = "inputs",
    summary = "Standalone radio controls with explicit reset behavior and descriptive copy.",
}

function module.render_preview(state: any)
    local gallery = state.gallery
    local next_radio, changed = api.render("publish_channel", gallery.radio_publish_channel, {
        label = "Use publish channel",
        description = "Standalone radios stay selected until you explicitly reset them.",
    })
    if changed then
        gallery.radio_publish_channel = next_radio
    end
    scope.row({ gap = 8 }, function()
        if button.render("radio_reset", "Reset", {
            variant = "secondary",
        }) then
            gallery.radio_publish_channel = false
        end
        text.meta(if gallery.radio_publish_channel then "Selected" else "Unselected")
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
