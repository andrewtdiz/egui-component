--!strict

local text = require("../core/text")
local scope = require("../core/layout")
local loading = require("../core/loading")
local state_store = require("./catalog_state")
local button = require("./button").api

local api = {}

function api.render(props: any?)
    loading.spinner(props)
end

local module = {
    api = api,
    id = "spinner",
    label = "Spinner",
    section = "feedback",
    summary = "Indeterminate loading indicators with size, speed, and stroke tuning.",
}

function module.render_preview(state: any)
    local catalog = state.catalog
    local spinner_demo_active = catalog.spinner_demo_frames > 0

    scope.row({ gap = 16 }, function()
        api.render({
            size = 16,
        })
        api.render({
            size = 22,
        })
        api.render({
            size = 28,
            stroke_width = 2.4,
            color = "foreground",
        })
    end)

    scope.column({ gap = 4 }, function()
        text.meta("Indeterminate loading spinner with configurable size, stroke, and tint.")
        text.meta("Loading button example")
    end)
    scope.row({ gap = 8 }, function()
        if spinner_demo_active then
            api.render({
                size = 14,
                stroke_width = 2,
                speed = 1.4,
                color = "foreground",
            })
        end
        if button.render(
            "spinner_publish",
            if spinner_demo_active then "  Publish" else "Publish",
            {
                variant = "secondary",
                width = 112,
            }
        ) then
            catalog.spinner_demo_frames = 180
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
