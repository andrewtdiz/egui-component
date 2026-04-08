--!strict

local text = require("../core/text")
local scope = require("../core/layout")
local loading = require("../core/loading")
local state_store = require("./catalog_state")
local button = require("./button").api
local card = require("./card").api

local api = {}

function api.render(props: any?)
    loading.skeleton(props)
end

local module = {
    api = api,
    id = "skeleton",
    label = "Skeleton",
    section = "feedback",
    summary = "Loading placeholders that mirror the final layout with rect and circle shapes.",
}

function module.render_preview(state: any)
    local catalog = state.catalog
    local toggle_label = if catalog.skeleton_loading then "Show Loaded State" else "Show Loading State"
    if button.render("skeleton_toggle", toggle_label, {
        variant = "secondary",
    }) then
        catalog.skeleton_loading = not catalog.skeleton_loading
    end

    card.surface(function()
        scope.row({ gap = 12 }, function()
            if catalog.skeleton_loading then
                api.render({
                    width = 44,
                    height = 44,
                    shape = "circle",
                })
            else
                button.render("skeleton_loaded_icon", "", {
                    variant = "ghost",
                    leading_icon = "sparkles",
                    icon_size = 20,
                    icon_only = true,
                })
            end

            scope.column({ gap = 8 }, function()
                if catalog.skeleton_loading then
                    api.render({
                        width = 180,
                        height = 16,
                        shape = "rect",
                    })
                    api.render({
                        width = 240,
                        height = 12,
                        shape = "rect",
                    })
                    api.render({
                        width = 212,
                        height = 12,
                        shape = "rect",
                    })
                    scope.row({ gap = 8 }, function()
                        api.render({
                            width = 72,
                            height = 28,
                            shape = "rect",
                        })
                        api.render({
                            width = 96,
                            height = 28,
                            shape = "rect",
                        })
                    end)
                else
                    text.label("Loading state complete", {
                        tone = "primary",
                        weight = "semibold",
                    })
                    text.label(
                        "Skeleton blocks can be mixed to mirror the final layout while data is in flight.",
                        {
                            tone = "muted",
                            size = 12,
                        }
                    )
                    scope.row({ gap = 8 }, function()
                        button.render("skeleton_inspect", "Inspect", {
                            variant = "primary",
                        })
                        button.render("skeleton_dismiss", "Dismiss", {
                            variant = "secondary",
                        })
                    end)
                end
            end)
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
