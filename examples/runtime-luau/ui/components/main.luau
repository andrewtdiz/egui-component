--!strict

local bridge = require("../bridge")
local kit = require("../../ui.luau")
local catalog = require("./catalog")
local component_state = require("./state")

type GalleryPreview = catalog.GalleryPreview
type GallerySection = catalog.GallerySection

local module = {}

local function render_preview(state: any, item: GalleryPreview)
    item.render_preview(state)
end

function module.init(state: any)
    component_state.init(state)
end

function module.update(state: any, _input: any)
    component_state.update(state)
end

function module.render(state: any)
    component_state.init(state)
    local gallery = state.gallery
    local current_ui = bridge.current_ui()
    local selected_id = tostring(gallery.selected or catalog.default_preview.id)
    local section, item = catalog.find(selected_id)

    kit.stack.column({ gap = kit.tokens.spacing.md }, function()
        kit.text.title("Luau Primitive Component Gallery")
        kit.text.meta(
            "Browse the shadcn-like direct ui.* primitives through the same showcase pattern as the Rust catalog."
        )

        kit.stack.row({ gap = kit.tokens.spacing.lg }, function()
            kit.card.surface({
                width = 248,
                padding_x = 16,
                padding_y = 18,
            }, function()
                kit.stack.column({ gap = kit.tokens.spacing.xs }, function()
                    kit.text.kicker("Components")
                    for _, sidebar_section in ipairs(catalog.sections) do
                        kit.text.meta(sidebar_section.title)
                        for _, sidebar_item in ipairs(sidebar_section.items) do
                            local clicked = if sidebar_item.id == item.id
                                then kit.button.primary("nav_" .. sidebar_item.id, sidebar_item.label, {
                                    width = 216,
                                })
                                else kit.button.ghost("nav_" .. sidebar_item.id, sidebar_item.label, {
                                    width = 216,
                                })
                            if clicked then
                                gallery.selected = sidebar_item.id
                            end
                        end
                        current_ui.separator()
                    end
                end)
            end)

            kit.card.surface({
                width = 860,
                padding_x = 22,
                padding_y = 22,
            }, function()
                kit.stack.column({ gap = kit.tokens.spacing.sm }, function()
                    kit.text.kicker(section.title)
                    kit.text.title(item.label)
                    kit.text.meta(item.summary)
                    current_ui.separator()
                    render_preview(state, item)
                end)
            end)
        end)
    end)
end

return module
