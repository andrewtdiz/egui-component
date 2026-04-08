--!strict

local ui_types = require("./types")
local bridge = require("./bridge")

type ContainerProps = ui_types.ContainerProps
type CardProps = ui_types.CardProps

local module = {}

local function rethrow(err: any): never
    error(err, 0)
end

local function run_scoped(
    begin_scope: (any?) -> (),
    end_scope: () -> (),
    props_or_render: any,
    maybe_render: (() -> ())?
): any
    local props = props_or_render
    local render = maybe_render
    if type(props_or_render) == "function" then
        props = nil
        render = props_or_render
    end

    local callback = render or function() end
    begin_scope(props)
    local results = table.pack(xpcall(callback, function(err)
        return err
    end))
    local close_results = table.pack(pcall(end_scope))
    local close_ok = close_results[1]
    local close_err = close_results[2]
    if not close_ok then
        rethrow(close_err)
    end
    if not results[1] then
        rethrow(results[2])
    end
    return table.unpack(results, 2, results.n)
end

function module.column(props_or_render: ContainerProps | (() -> ())?, maybe_render: (() -> ())?): any
    local current_ui = bridge.current_ui()
    return run_scoped(function(props)
        current_ui.begin_column(props)
    end, function()
        current_ui.end_scope()
    end, props_or_render, maybe_render)
end

function module.row(props_or_render: ContainerProps | (() -> ())?, maybe_render: (() -> ())?): any
    local current_ui = bridge.current_ui()
    return run_scoped(function(props)
        current_ui.begin_row(props)
    end, function()
        current_ui.end_scope()
    end, props_or_render, maybe_render)
end

function module.card(props_or_render: CardProps | (() -> ())?, maybe_render: (() -> ())?): any
    local current_ui = bridge.current_ui()
    return run_scoped(function(props)
        current_ui.begin_card(props)
    end, function()
        current_ui.end_scope()
    end, props_or_render, maybe_render)
end

function module.with_id(id: string, render: () -> ()): any
    local current_ui = bridge.current_ui()
    current_ui.push_id(id)
    local results = table.pack(xpcall(render, function(err)
        return err
    end))
    local close_results = table.pack(pcall(function()
        current_ui.pop_id()
    end))
    local close_ok = close_results[1]
    local close_err = close_results[2]
    if not close_ok then
        rethrow(close_err)
    end
    if not results[1] then
        rethrow(results[2])
    end
    return table.unpack(results, 2, results.n)
end

return module
