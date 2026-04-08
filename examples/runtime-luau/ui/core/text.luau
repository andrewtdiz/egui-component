--!strict

local ui_types = require("./types")
local bridge = require("./bridge")
local tokens = require("./tokens")
local variant = require("./variant")

type LabelProps = ui_types.LabelProps

local module = {}

local function resolve_props(base: LabelProps, overrides: LabelProps?): LabelProps
    if overrides == nil then
        return base
    end
    return variant.merge(base, overrides)
end

function module.label(text: string, props: LabelProps?)
    bridge.current_ui().label(text, props)
end

function module.kicker(text: string, overrides: LabelProps?)
    module.label(text, resolve_props(tokens.kicker, overrides))
end

function module.title(text: string, overrides: LabelProps?)
    module.label(text, resolve_props(tokens.title, overrides))
end

function module.meta(text: string, overrides: LabelProps?)
    module.label(text, resolve_props(tokens.meta, overrides))
end

return module
