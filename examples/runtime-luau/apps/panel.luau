--!strict

local ui_types = require("../ui/core/types")

type App = ui_types.App
type Ui = ui_types.Ui

local module = {}
local module_state: any = nil

function module.init(state: any)
    module_state = state
    state.reloads = state.reloads or 0
end

function module.reload(_old_exports: any, state: any)
    local app: App = app

    module_state = state
    state.reloads = (state.reloads or 0) + 1
    app.log("info", "apps/panel.luau reloaded")
end

function module.view(state: any)
    local ui: Ui = ui
    local app: App = app

    ui.begin_card({
        width = 360,
        padding_x = 20,
        padding_y = 20,
    })
    ui.begin_column({ gap = 12 })
    ui.label("Immediate Mode Surface", {
        tone = "muted",
        size = 12,
    })

    ui.label(
        "Hello, " .. state.name .. " from " .. (state.variant or "immediate panel"),
        {
            weight = "bold",
            size = 22,
        }
    )

    ui.label(
        "Leaf reloads: " .. tostring(module_state.reloads),
        {
            tone = "muted",
        }
    )

    ui.push_id("profile")
    ui.begin_row({ gap = 10 })
    if ui.button("increment", "Increment", {
        variant = "primary",
        width = 120,
    }) then
        state.count = (state.count or 0) + 1
        state.last_event = "increment"
    end

    if ui.button("reload", "Reload Script", {
        variant = "secondary",
        width = 140,
    }) then
        state.last_event = "reload"
        app.request_reload()
    end
    ui.end_scope()

    local value, changed = ui.text_edit("name", state.name, {
        placeholder = "Enter a name",
        width = 280,
    })
    if changed then
        state.name = value
        state.last_event = "name:" .. value
    end
    ui.pop_id()
    ui.end_scope()
    ui.end_scope()
end

function module.render(state: any)
    module.view(state)
end

return module
