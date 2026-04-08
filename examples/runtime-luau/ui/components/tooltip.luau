--!strict

local ui_types = require("../core/types")
local bridge = require("../core/bridge")
local state_store = require("./catalog_state")
local tabs = require("./tabs").api

type TooltipProps = ui_types.TooltipProps

local TOOLTIP_PLACEMENT_TABS = table.freeze({
    { label = "Top" },
    { label = "Right" },
    { label = "Bottom" },
    { label = "Left" },
})

local TOOLTIP_PLACEMENTS = table.freeze({
    "top",
    "right",
    "bottom",
    "left",
})

local api = {}

function api.render(trigger_label: string, text: string, props: TooltipProps?): ()
    bridge.current_ui().tooltip(trigger_label, text, props)
end

local module = {
    api = api,
    id = "tooltip",
    label = "Tooltip",
    section = "overlays",
    summary = "Hover-triggered helper content with typed placement and width props.",
}

function module.render_preview(state: any)
    local catalog = state.catalog
    local placement_index = 1
    for index, placement in ipairs(TOOLTIP_PLACEMENTS) do
        if placement == catalog.tooltip_placement then
            placement_index = index
            break
        end
    end
    local next_index, changed = tabs.render(
        "tooltip_placement",
        placement_index,
        TOOLTIP_PLACEMENT_TABS,
        {
            variant = "segmented",
        }
    )
    if changed then
        catalog.tooltip_placement = TOOLTIP_PLACEMENTS[next_index] or "top"
    end

    api.render("Hover this trigger", "Tooltip content example", {
        width = 220,
        placement = catalog.tooltip_placement,
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
