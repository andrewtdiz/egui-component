--!strict

local ui_types = require("../core/types")
local bridge = require("../core/bridge")
local scope = require("../core/layout")
local state_store = require("./catalog_state")
local tokens = require("../core/tokens")
local variant = require("../core/variant")

type ButtonProps = ui_types.ButtonProps

local api = {}

local function resolve_props(base: ButtonProps, overrides: ButtonProps?): ButtonProps
    if overrides == nil then
        return base
    end
    return variant.merge(base, overrides)
end

function api.render(id: string, text: string, props: ButtonProps?): boolean
    return bridge.current_ui().button(id, text, props)
end

function api.primary(id: string, text: string, overrides: ButtonProps?): boolean
    return api.render(id, text, resolve_props(tokens.primary_button, overrides))
end

function api.secondary(id: string, text: string, overrides: ButtonProps?): boolean
    return api.render(id, text, resolve_props(tokens.secondary_button, overrides))
end

function api.ghost(id: string, text: string, overrides: ButtonProps?): boolean
    return api.render(id, text, resolve_props(tokens.ghost_button, overrides))
end

function api.link(id: string, text: string, overrides: ButtonProps?): boolean
    return api.render(id, text, resolve_props(tokens.link_button, overrides))
end

local module = {
    api = api,
    id = "button",
    label = "Button",
    section = "actions",
    summary = "Primary, secondary, ghost, and link actions built directly on the typed bridge.",
}

function module.render_preview(_state: any)
    scope.column({ gap = 8 }, function()
        scope.row({ gap = 8 }, function()
            api.render("button_primary", "Primary", { variant = "primary" })
            api.render("button_secondary", "Secondary", { variant = "secondary" })
            api.render("button_ghost", "Ghost", { variant = "ghost" })
            api.render("button_link", "Link", { variant = "link" })
        end)

        scope.row({ gap = 8 }, function()
            api.render("button_new_draft", "New Draft", {
                variant = "primary",
                leading_icon = "file-plus",
                icon_size = 15,
            })
            api.render("button_search", "Search", {
                variant = "secondary",
                leading_icon = "search",
                icon_size = 15,
            })
            api.render("button_share", "Share", {
                variant = "ghost",
                leading_icon = "share-2",
                icon_size = 15,
            })
            api.render("button_export", "Export", {
                variant = "link",
                leading_icon = "download",
                icon_size = 15,
            })
        end)

        scope.row({ gap = 8 }, function()
            api.render("button_icon_primary", "", {
                variant = "primary",
                leading_icon = "wand-sparkles",
                icon_size = 15,
                icon_only = true,
            })
            api.render("button_icon_secondary", "", {
                variant = "secondary",
                leading_icon = "wand-sparkles",
                icon_size = 15,
                icon_only = true,
            })
            api.render("button_icon_ghost", "", {
                variant = "ghost",
                leading_icon = "wand-sparkles",
                icon_size = 15,
                icon_only = true,
            })
            api.render("button_icon_link", "", {
                variant = "link",
                leading_icon = "wand-sparkles",
                icon_size = 15,
                icon_only = true,
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
