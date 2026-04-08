--!strict

local ui_types = require("../core/types")
local bridge = require("../core/bridge")
local text = require("../core/text")
local state_store = require("./catalog_state")

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
    local catalog = state.catalog
    local next_select, changed = api.render(
        "review_state",
        catalog.select_review_state,
        SELECT_OPTIONS,
        {
            width = 280,
        }
    )
    if changed then
        catalog.select_review_state = next_select
    end
    local selected = SELECT_OPTIONS[catalog.select_review_state]
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
