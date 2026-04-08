--!strict

local text = require("../core/text")
local scope = require("../core/layout")
local state_store = require("./catalog_state")

local api = {}

function api.render(content: string, props: any?)
    text.label(content, props)
end

function api.kicker(content: string, props: any?)
    text.kicker(content, props)
end

function api.title(content: string, props: any?)
    text.title(content, props)
end

function api.meta(content: string, props: any?)
    text.meta(content, props)
end

local module = {
    api = api,
    id = "label",
    label = "Label",
    section = "content",
    summary = "Primary, muted, and destructive text styles for compact UI copy.",
}

function module.render_preview(_state: any)
    scope.column({ gap = 8 }, function()
        api.render("Primary label", {
            tone = "primary",
            weight = "semibold",
        })
        api.render("Secondary label", {
            tone = "secondary",
        })
        api.render("Muted helper text", {
            tone = "muted",
        })
        api.render("Destructive text", {
            tone = "destructive",
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
