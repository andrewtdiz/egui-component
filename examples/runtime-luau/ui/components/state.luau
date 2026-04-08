--!strict

local bridge = require("../bridge")

local module = {}

local DEFAULT_GALLERY_STATE = table.freeze({
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
    local preview_state = state.gallery
    if preview_state == nil then
        preview_state = {}
        state.gallery = preview_state
    end

    for key, value in pairs(DEFAULT_GALLERY_STATE) do
        if preview_state[key] == nil then
            preview_state[key] = value
        end
    end
end

function module.update(state: any)
    module.init(state)
    local preview_state = state.gallery
    if preview_state.spinner_demo_frames > 0 then
        preview_state.spinner_demo_frames = preview_state.spinner_demo_frames - 1
        bridge.current_app().request_repaint()
    end
end

return module
