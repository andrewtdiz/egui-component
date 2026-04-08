--!strict

local ui_types = require("../core/types")
local bridge = require("../core/bridge")
local scope = require("../core/layout")
local text = require("../core/text")
local state_store = require("./catalog_state")

type TabOption = ui_types.TabOption
type TabsProps = ui_types.TabsProps

local INLINE_TABS = table.freeze({
    { label = "Design" },
    { label = "Code" },
    { label = "History" },
})

local BLENDER_TABS = table.freeze({
    { label = "Layout" },
    { label = "Modeling" },
    { label = "Sculpting" },
    { label = "UV Editing" },
    { label = "Texture Paint" },
})

local STACKED_TABS = table.freeze({
    { label = "Templates", icon = "layout-template" },
    { label = "Layouts", icon = "layout-grid" },
})

local RAIL_TABS = table.freeze({
    { label = "Home", icon = "house" },
    { label = "Assets", icon = "image" },
    { label = "Export", icon = "rocket" },
})

local api = {}

function api.render(
    id: string,
    selected_index: number?,
    options: { TabOption },
    props: TabsProps?
): (number, boolean)
    return bridge.current_ui().tabs(id, selected_index or 0, options, props)
end

local module = {
    api = api,
    id = "tabs",
    label = "Tabs",
    section = "actions",
    summary = "Underline, segmented, stacked, rail, and topbar variants over one native tab primitive.",
}

function module.render_preview(state: any)
    local catalog = state.catalog

    scope.column({ gap = 18 }, function()
        local next_tab, changed = api.render("preview_tabs_inline", catalog.tabs_inline_index, INLINE_TABS, {
            variant = "underline",
        })
        if changed then
            catalog.tabs_inline_index = next_tab
        end
        scope.column({ gap = 6 }, function()
            local selected = INLINE_TABS[catalog.tabs_inline_index]
            text.meta(if selected ~= nil then selected.label else INLINE_TABS[1].label)
        end)

        local next_blender_tab, blender_changed = api.render(
            "preview_tabs_blender",
            catalog.tabs_blender_index,
            BLENDER_TABS,
            {
                variant = "blender_topbar",
            }
        )
        if blender_changed then
            catalog.tabs_blender_index = next_blender_tab
        end

        local next_segmented_tab, segmented_changed = api.render(
            "preview_tabs_segmented",
            catalog.tabs_segmented_index,
            INLINE_TABS,
            {
                variant = "segmented",
            }
        )
        if segmented_changed then
            catalog.tabs_segmented_index = next_segmented_tab
        end

        local next_stacked_tab, stacked_changed = api.render(
            "preview_tabs_stacked",
            catalog.tabs_stacked_index,
            STACKED_TABS,
            {
                variant = "stacked",
            }
        )
        if stacked_changed then
            catalog.tabs_stacked_index = next_stacked_tab
        end

        local next_rail_tab, rail_changed = api.render(
            "preview_tabs_rail",
            catalog.tabs_rail_index,
            RAIL_TABS,
            {
                variant = "rail",
            }
        )
        if rail_changed then
            catalog.tabs_rail_index = next_rail_tab
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
