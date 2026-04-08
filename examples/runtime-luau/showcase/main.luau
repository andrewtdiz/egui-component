--!strict

local ui_types = require("../ui/types")
local kit = require("../ui.luau")
local editor_sidebar = require("../components/editor_sidebar")
local profile_panel = require("../components/profile_panel")
local settings_form = require("../components/settings_form")

type App = ui_types.App
type Ui = ui_types.Ui

local module = {}

function module.init(state: any)
    state.name = state.name or "Runtime Showcase"
    state.count = state.count or 0
    state.last_event = state.last_event or "boot"
    state.reloads = state.reloads or 0
end

function module.reload(_old_exports: any, state: any)
    local app: App = app

    state.reloads = (state.reloads or 0) + 1
    app.log("info", "showcase/main.luau reloaded")
end

function module.render(state: any)
    local _ui: Ui = ui

    kit.stack.column({ gap = kit.tokens.spacing.md }, function()
        kit.text.title("Showcase Runtime Shell")
        kit.text.meta(
            "This surface is authored through the Luau component library and rendered through the direct frame host."
        )

        kit.stack.row({ gap = kit.tokens.spacing.lg }, function()
            profile_panel.render(state, {
                eyebrow = "Profile Recipe",
                source_label = "showcase/profile_panel",
                card_props = kit.tokens.card_showcase,
                placeholder = "Rename the showcase profile",
            })

            kit.stack.column({ gap = kit.tokens.spacing.lg }, function()
                settings_form.render(state)
                editor_sidebar.render(state)
            end)
        end)
    end)
end

return module
