--!strict

local disclosure = require("../core/disclosure")
local scope = require("../core/layout")
local text = require("../core/text")
local tokens = require("../core/tokens")
local state_store = require("./catalog_state")

local api = {}

function api.render(
    id: string,
    title: string,
    open: boolean,
    props_or_render: any,
    maybe_render: (() -> ())?
): boolean
    return disclosure.collapsible(id, title, open, props_or_render, maybe_render)
end

local module = {
    api = api,
    id = "collapsible",
    label = "Collapsible",
    section = "content",
    summary = "Explicit-scope disclosure content with stable open state and inline metadata.",
}

function module.render_preview(state: any)
    local catalog = state.catalog
    catalog.collapsible_transform_open = api.render(
        "catalog_details",
        "Transform",
        catalog.collapsible_transform_open,
        {
            leading_icon = "move-3d",
            trailing_icon = "ellipsis",
        },
        function()
            scope.column({ gap = tokens.spacing.xs }, function()
                text.label("Position", {
                    tone = "secondary",
                })
                text.label("Rotation", {
                    tone = "secondary",
                })
                text.label("Scale", {
                    tone = "secondary",
                })
            end)
        end
    )
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
