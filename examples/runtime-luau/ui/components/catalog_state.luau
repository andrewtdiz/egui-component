--!strict

local bridge = require("../core/bridge")

local module = {}

local DEFAULT_CATALOG_STATE = table.freeze({
    selected = "field",
    field_material = "M_Robot_Body",
    input_value = "Player_Robot",
    search_query = "Robot",
    checkbox_receive_shadows = true,
    switch_enable_post_fx = true,
    switch_compact_handles = false,
    slider_value = 62,
    number_input_x = 42,
    number_input_y = 16,
    select_review_state = 2,
    radio_publish_channel = true,
    radio_group_plan = 2,
    button_group_transform_mode = 0,
    pagination_page = 2,
    tabs_inline_index = 1,
    tabs_blender_index = 3,
    tabs_segmented_index = 1,
    tabs_stacked_index = 1,
    tabs_rail_index = 1,
    progress_value = 0,
    dropdown_menu_action = 0,
    dropdown_menu_label = "No action triggered",
    collapsible_transform_open = true,
    tooltip_placement = "top",
    skeleton_loading = true,
    spinner_demo_frames = 0,
})

function module.init(state: any)
    local catalog_state = state.catalog
    if catalog_state == nil then
        catalog_state = {}
        state.catalog = catalog_state
    end

    for key, value in pairs(DEFAULT_CATALOG_STATE) do
        if catalog_state[key] == nil then
            catalog_state[key] = value
        end
    end
end

function module.update(state: any)
    module.init(state)
    local catalog_state = state.catalog
    if catalog_state.spinner_demo_frames > 0 then
        catalog_state.spinner_demo_frames = catalog_state.spinner_demo_frames - 1
        bridge.current_app().request_repaint()
    end
end

return module
