--!strict

local bridge = require("../bridge")
local scope = require("../scope")
local text = require("../text")
local state_store = require("./state")
local button = require("./button").api

local api = {}

function api.surface(props_or_render: any, maybe_render: (() -> ())?): any
    return scope.card(props_or_render, maybe_render)
end

local module = {
    api = api,
    id = "card",
    label = "Card",
    section = "content",
    summary = "Framed content surfaces with composable spacing, separators, and actions.",
}

function module.render_preview(_state: any)
    local current_ui = bridge.current_ui()

    api.surface({
        width = 460,
        padding_x = 12,
        padding_y = 12,
    }, function()
        scope.column({ gap = 6 }, function()
            text.label("Card Title", {
                tone = "primary",
                weight = "bold",
                size = 16,
            })
            text.label("Cards wrap related content in a bordered panel.", {
                tone = "muted",
            })
        end)
        current_ui.separator()
        scope.row({ gap = 8 }, function()
            button.render("card_cancel", "Cancel", {
                variant = "secondary",
            })
            button.render("card_save", "Save", {
                variant = "primary",
            })
        end)
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
