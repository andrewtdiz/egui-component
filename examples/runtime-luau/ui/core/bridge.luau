--!strict

local ui_types = require("./types")

type App = ui_types.App
type Ui = ui_types.Ui

local module = {}

function module.current_app(): App
    local current: App = app
    return current
end

function module.current_ui(): Ui
    local current: Ui = ui
    return current
end

return module
