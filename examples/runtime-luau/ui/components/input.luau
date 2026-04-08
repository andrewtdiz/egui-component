--!strict

local ui_types = require("../types")
local bridge = require("../bridge")
local scope = require("../scope")
local state_store = require("./state")
local tokens = require("../tokens")
local variant = require("../variant")

type TextEditProps = ui_types.TextEditProps

local api = {}

local function resolve_props(base: TextEditProps, overrides: TextEditProps?): TextEditProps
    if overrides == nil then
        return base
    end
    return variant.merge(base, overrides)
end

function api.text(id: string, value: string, props: TextEditProps?): (string, boolean)
    return bridge.current_ui().text_edit(id, value, props)
end

function api.password(id: string, value: string, overrides: TextEditProps?): (string, boolean)
    return api.text(id, value, resolve_props(tokens.input, variant.merge(overrides, {
        password = true,
    })))
end

local module = {
    api = api,
    id = "input",
    label = "Input",
    section = "content",
    summary = "Direct text-edit primitives with stable ids, placeholder copy, and icons.",
}

function module.render_preview(state: any)
    local gallery = state.gallery

    scope.column({ gap = 10 }, function()
        local next_input, input_changed = api.text(
            "gallery_input_primary",
            tostring(gallery.input_value),
            {
                width = 280,
                placeholder = "Type component name",
            }
        )
        if input_changed then
            gallery.input_value = next_input
        end

        local next_search, search_changed = api.text(
            "gallery_input_search",
            tostring(gallery.search_query),
            {
                width = 280,
                placeholder = "Search",
                leading_icon = "search",
            }
        )
        if search_changed then
            gallery.search_query = next_search
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
