--!strict

local ui_types = require("../types")
local bridge = require("../bridge")
local scope = require("../scope")
local text = require("../text")
local state_store = require("./state")

type SliderProps = ui_types.SliderProps

local api = {}

function api.render(id: string, value: number, props: SliderProps): (number, boolean)
    return bridge.current_ui().slider(id, value, props)
end

local module = {
    api = api,
    id = "slider",
    label = "Slider",
    section = "inputs",
    summary = "Range inputs with typed min and max bounds plus live value copy.",
}

function module.render_preview(state: any)
    local gallery = state.gallery
    scope.row({ gap = 8 }, function()
        local next_slider, changed = api.render("slider_value", gallery.slider_value, {
            min = 0,
            max = 100,
            width = 250,
        })
        if changed then
            gallery.slider_value = next_slider
        end
        text.label(tostring(math.floor(gallery.slider_value + 0.5)), {
            tone = "secondary",
            weight = "semibold",
        })
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
