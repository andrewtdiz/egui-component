--!strict

local label = require("./label")
local input = require("./input")
local field = require("./field")
local separator = require("./separator")
local card = require("./card")
local collapsible = require("./collapsible")
local checkbox = require("./checkbox")
local switch = require("./switch")
local slider = require("./slider")
local number_input = require("./number-input")
local select = require("./select")
local radio = require("./radio")
local radio_group = require("./radio-group")
local button = require("./button")
local button_group = require("./button-group")
local tabs = require("./tabs")
local pagination = require("./pagination")
local progress = require("./progress")
local spinner = require("./spinner")
local skeleton = require("./skeleton")
local dropdown_menu = require("./dropdown-menu")
local tooltip = require("./tooltip")

export type GalleryPreview = {
    id: string,
    label: string,
    section: string,
    summary: string,
    render_preview: (state: any) -> (),
    init: (state: any) -> (),
    update: (state: any, input: any) -> (),
    render: (state: any) -> (),
    api: any,
}

export type GallerySection = {
    title: string,
    items: { GalleryPreview },
}

local SECTIONS: { GallerySection } = table.freeze({
    {
        title = "Content",
        items = {
            label :: GalleryPreview,
            input :: GalleryPreview,
            field :: GalleryPreview,
            separator :: GalleryPreview,
            card :: GalleryPreview,
            collapsible :: GalleryPreview,
        },
    },
    {
        title = "Inputs",
        items = {
            checkbox :: GalleryPreview,
            switch :: GalleryPreview,
            slider :: GalleryPreview,
            number_input :: GalleryPreview,
            select :: GalleryPreview,
            radio :: GalleryPreview,
            radio_group :: GalleryPreview,
        },
    },
    {
        title = "Actions",
        items = {
            button :: GalleryPreview,
            button_group :: GalleryPreview,
            tabs :: GalleryPreview,
            pagination :: GalleryPreview,
        },
    },
    {
        title = "Feedback",
        items = {
            progress :: GalleryPreview,
            spinner :: GalleryPreview,
            skeleton :: GalleryPreview,
        },
    },
    {
        title = "Overlays",
        items = {
            dropdown_menu :: GalleryPreview,
            tooltip :: GalleryPreview,
        },
    },
})

local DEFAULT_PREVIEW = SECTIONS[1].items[1]

local module = {
    sections = SECTIONS,
    default_preview = DEFAULT_PREVIEW,
}

function module.find(selected_id: string): (GallerySection, GalleryPreview)
    for _, section in ipairs(SECTIONS) do
        for _, item in ipairs(section.items) do
            if item.id == selected_id then
                return section, item
            end
        end
    end

    return SECTIONS[1], DEFAULT_PREVIEW
end

return module
