--!strict

local ui_types = require("../types")
local bridge = require("../bridge")
local scope = require("../scope")
local state_store = require("./state")
local button = require("./button").api

type ProgressProps = ui_types.ProgressProps

local api = {}

function api.render(value: number, props: ProgressProps?): ()
    bridge.current_ui().progress(value, props)
end

local module = {
    api = api,
    id = "progress",
    label = "Progress",
    section = "feedback",
    summary = "Determinate progress indicators with clear reset and advance controls.",
}

function module.render_preview(state: any)
    local gallery = state.gallery
    api.render(gallery.progress_value, {
        width = 280,
    })
    scope.row({ gap = 8 }, function()
        if button.render("progress_advance", "Advance", {
            variant = "primary",
        }) then
            gallery.progress_value = math.min(gallery.progress_value + 0.1, 1)
        end
        if button.render("progress_reset", "Reset", {
            variant = "secondary",
        }) then
            gallery.progress_value = 0
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
