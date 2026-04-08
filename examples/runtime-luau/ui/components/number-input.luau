--!strict

local ui_types = require("../types")
local bridge = require("../bridge")
local scope = require("../scope")
local state_store = require("./state")

type NumberInputProps = ui_types.NumberInputProps

local api = {}

function api.render(id: string, value: number, props: NumberInputProps?): (number, boolean)
    return bridge.current_ui().number_input(id, value, props)
end

local module = {
    api = api,
    id = "number-input",
    label = "Number Input",
    section = "inputs",
    summary = "Numeric controls with bounds, prefix styling, and axis-aware drag affordances.",
}

function module.render_preview(state: any)
    local gallery = state.gallery

    scope.row({ gap = 8 }, function()
        local next_number_x, changed_x = api.render("number_x", gallery.number_input_x, {
            min = 0,
            max = 100,
            width = 110,
            decimals = 1,
            prefix = "X",
            prefix_tint = "#7BAE6E",
            prefix_align_left = true,
            axis = "horizontal",
        })
        if changed_x then
            gallery.number_input_x = next_number_x
        end

        local next_number_y, changed_y = api.render("number_y", gallery.number_input_y, {
            min = 0,
            max = 100,
            width = 110,
            decimals = 1,
            prefix = "Y",
            prefix_tint = "#D65A6A",
            prefix_align_left = true,
            axis = "vertical",
        })
        if changed_y then
            gallery.number_input_y = next_number_y
        end
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
