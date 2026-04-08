--!strict

local ui_types = require("../types")
local input = require("./input").api
local scope = require("../scope")
local text = require("../text")
local state_store = require("./state")
local tokens = require("../tokens")
local variant = require("../variant")

type LabelProps = ui_types.LabelProps
type TextEditProps = ui_types.TextEditProps

export type TextFieldConfig = {
    id: string,
    label: string,
    value: string,
    description: string?,
    helper_text: string?,
    placeholder: string?,
    width: number?,
    gap: number?,
    label_props: LabelProps?,
    input_props: TextEditProps?,
}

local api = {}

function api.text(config: TextFieldConfig): (string, boolean)
    local next_value = config.value
    local changed = false

    local resolved_input_props = config.input_props
    if config.width ~= nil or config.placeholder ~= nil then
        resolved_input_props = variant.merge(resolved_input_props, {
            width = config.width,
            placeholder = config.placeholder,
        })
    end
    if resolved_input_props == nil then
        resolved_input_props = tokens.input
    else
        resolved_input_props = variant.merge(tokens.input, resolved_input_props)
    end

    scope.with_id(config.id, function()
        scope.column({ gap = config.gap or tokens.spacing.xs }, function()
            text.label(config.label, variant.merge(tokens.field_label, config.label_props))
            next_value, changed = input.text("input", next_value, resolved_input_props)
            local helper_text = config.helper_text or config.description
            if helper_text ~= nil then
                text.label(helper_text, tokens.field_description)
            end
        end)
    end)

    return next_value, changed
end

local module = {
    api = api,
    id = "field",
    label = "Field",
    section = "content",
    summary = "Composable label, input, and helper text bundled into one clear recipe.",
}

function module.render_preview(state: any)
    local gallery = state.gallery
    local next_name, changed = api.text({
        id = "gallery_material",
        label = "Material",
        value = tostring(gallery.field_material),
        helper_text = "Assigned material for selected mesh",
        width = 280,
    })
    if changed then
        gallery.field_material = next_name
    end
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
