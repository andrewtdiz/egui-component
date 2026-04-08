--!strict

local ui_types = require("./types")
local bridge = require("./bridge")

type CollapsibleProps = ui_types.CollapsibleProps

local module = {}

local function rethrow(err: any): never
    error(err, 0)
end

function module.collapsible(
    id: string,
    title: string,
    open: boolean,
    props_or_render: CollapsibleProps | (() -> ())?,
    maybe_render: (() -> ())?
): boolean
    local props = props_or_render
    local render = maybe_render
    if type(props_or_render) == "function" then
        props = nil
        render = props_or_render
    end

    local current_ui = bridge.current_ui()
    local typed_props: CollapsibleProps? = if type(props) == "table" then props else nil
    local next_open, visible = current_ui.begin_collapsible(id, title, open, typed_props)
    if not visible then
        return next_open
    end

    local callback = render or function() end
    local results = table.pack(xpcall(callback, function(err)
        return err
    end))
    local close_results = table.pack(pcall(function()
        current_ui.end_scope()
    end))
    if not close_results[1] then
        rethrow(close_results[2])
    end
    if not results[1] then
        rethrow(results[2])
    end

    return next_open
end

return module
