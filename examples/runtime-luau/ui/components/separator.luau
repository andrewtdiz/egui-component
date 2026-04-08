--!strict

local bridge = require("../bridge")
local text = require("../text")
local state_store = require("./state")

local api = {}

function api.render()
    bridge.current_ui().separator()
end

local module = {
    api = api,
    id = "separator",
    label = "Separator",
    section = "content",
    summary = "A simple structural divider for splitting related content inside a surface.",
}

function module.render_preview(_state: any)
    text.label("Above separator", {
        tone = "secondary",
    })
    api.render()
    text.label("Below separator", {
        tone = "secondary",
    })
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
