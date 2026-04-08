--!strict

local button = require("./button").api
local scope = require("../core/layout")
local text = require("../core/text")
local state_store = require("./catalog_state")

type PaginationItem = number | "ellipsis"

local api = {}

local function pagination_items(
    current_page: number,
    page_count: number,
    sibling_count: number
): { PaginationItem }
    if page_count <= 0 then
        return {}
    end

    local current = math.clamp(current_page, 1, page_count)
    local visible_slots = sibling_count * 2 + 5
    if page_count <= visible_slots then
        local items: { PaginationItem } = {}
        for page = 1, page_count do
            table.insert(items, page)
        end
        return items
    end

    local left_sibling = math.max(current - sibling_count, 1)
    local right_sibling = math.min(current + sibling_count, page_count)
    local show_left_ellipsis = left_sibling > 2
    local show_right_ellipsis = right_sibling < (page_count - 1)

    if not show_left_ellipsis and not show_right_ellipsis then
        local items: { PaginationItem } = {}
        for page = 1, page_count do
            table.insert(items, page)
        end
        return items
    end

    if not show_left_ellipsis and show_right_ellipsis then
        local last_left_page = math.min(sibling_count * 2 + 2, page_count - 1)
        local items: { PaginationItem } = {}
        for page = 1, last_left_page do
            table.insert(items, page)
        end
        table.insert(items, "ellipsis" :: "ellipsis")
        table.insert(items, page_count)
        return items
    end

    if show_left_ellipsis and not show_right_ellipsis then
        local start_page = math.max(page_count - (sibling_count * 2 + 1), 2)
        local items: { PaginationItem } = { 1, "ellipsis" }
        for page = start_page, page_count do
            table.insert(items, page)
        end
        return items
    end

    local items: { PaginationItem } = { 1, "ellipsis" }
    for page = left_sibling, right_sibling do
        table.insert(items, page)
    end
    table.insert(items, "ellipsis")
    table.insert(items, page_count)
    return items
end

function api.render(id: string, current_page: number, total_pages: number): (number, boolean)
    if total_pages <= 0 then
        return 1, false
    end

    local next_page = math.clamp(current_page, 1, total_pages)
    local changed = false
    local items = pagination_items(next_page, total_pages, 1)

    scope.with_id(id, function()
        scope.row({ gap = 4 }, function()
            if next_page > 1 and button.render("previous", "Previous", {
                variant = "ghost",
                size = "sm",
                leading_icon = "chevron-left",
                icon_size = 12,
            }) then
                next_page -= 1
                changed = true
            end

            for item_index, item in ipairs(items) do
                if item == "ellipsis" then
                    text.meta("...", {
                        size = 14,
                    })
                else
                    local page = item :: number
                    if button.render("page_" .. tostring(item_index), tostring(page), {
                        variant = "ghost",
                        size = "sm",
                        width = 32,
                        selected = page == next_page,
                    }) then
                        if page ~= next_page then
                            next_page = page
                            changed = true
                        end
                    end
                end
            end

            if next_page < total_pages and button.render("next", "Next", {
                variant = "ghost",
                size = "sm",
                trailing_icon = "chevron-right",
                icon_size = 12,
            }) then
                next_page += 1
                changed = true
            end
        end)
    end)

    return next_page, changed
end

local module = {
    api = api,
    id = "pagination",
    label = "Pagination",
    section = "actions",
    summary = "Composable previous and next paging built on the Luau button-group style API.",
}

function module.render_preview(state: any)
    local catalog = state.catalog
    local next_page, changed = api.render("catalog_pages", catalog.pagination_page, 12)
    if changed then
        catalog.pagination_page = next_page
    end
    text.meta("Page " .. tostring(catalog.pagination_page) .. " of 12")
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
