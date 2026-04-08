--!strict

local ui_types = require("../types")
local bridge = require("../bridge")
local text = require("../text")
local state_store = require("./state")

type SelectOption = ui_types.SelectOption
type SelectProps = ui_types.SelectProps

local SELECT_OPTIONS = table.freeze({
    { label = "Draft" },
    { label = "Review" },
    { label = "Approved" },
    { label = "Archived" },
})

local api = {}

function api.render(
    id: string,
    selected_index: number?,
    options: { SelectOption },
    props: SelectProps?
): (number, boolean)
    return bridge.current_ui().select(id, selected_index or 0, options, props)
end

local module = {
    api = api,
    id = "select",
    label = "Select",
    section = "inputs",
    summary = "Typed single-select menus with 1-based indices and explicit option tables.",
}

function module.render_preview(state: any)
    local gallery = state.gallery
    local next_select, changed = api.render(
        "review_state",
        gallery.select_review_state,
        SELECT_OPTIONS,
        {
            width = 280,
        }
    )
    if changed then
        gallery.select_review_state = next_select
    end
    local selected = SELECT_OPTIONS[gallery.select_review_state]
    text.meta(if selected ~= nil then selected.label else "None")
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
