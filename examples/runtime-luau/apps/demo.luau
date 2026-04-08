--!strict

local ui_types = require("../ui/core/types")
local kit = require("../ui.luau")
local profile_panel = require("../ui/recipes/profile_panel")

type App = ui_types.App
type Ui = ui_types.Ui

local module = {}

function module.init(state: any)
    state.name = state.name or "Luau"
    state.count = state.count or 0
    state.variant = state.variant or "runtime-luau/apps/panel.luau"
    state.last_event = state.last_event or "boot"
    state.update_note = state.update_note or "idle"
    state.reloads = state.reloads or 0
end

function module.update(state: any, input: any)
    if next(input) == nil then
        state.update_note = "update(input={})"
    else
        state.update_note = "update(input=non-empty)"
    end
end

function module.reload(_old_exports: any, state: any)
    local app: App = app

    state.reloads = (state.reloads or 0) + 1
    app.log("info", "apps/demo.luau reloaded")
end

function module.render(state: any)
    local _ui: Ui = ui

    kit.stack.column({ gap = kit.tokens.spacing.md }, function()
        kit.text.title("Luau component library")
        kit.text.meta(
            "Edit examples/runtime-luau/ui/recipes/profile_panel.luau to live-update the main recipe surface."
        )
        kit.text.meta(
            "Keep examples/runtime-luau/apps/panel.luau as the raw direct-primitive sample for host debugging."
        )
        kit.text.meta("Frame hook: " .. tostring(state.update_note))

        profile_panel.render(state, {
            eyebrow = "Demo Surface",
            source_label = tostring(state.variant),
        })

        kit.stack.row({ gap = kit.tokens.spacing.md }, function()
            kit.text.label("Count: " .. tostring(state.count), {
                weight = "semibold",
            })
            kit.text.meta("Last event: " .. tostring(state.last_event))
        end)
    end)
end

return module
