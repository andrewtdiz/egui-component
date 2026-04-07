local profile_card = require("./components/profile_card")

local module = {}

function module.init(state)
    state.name = state.name or "Runtime Showcase"
    state.count = state.count or 0
    state.last_event = state.last_event or "boot"
end

function module.render(state)
    ui.vertical({ gap = 16 }, function()
        ui.label("Showcase Runtime Shell", {
            weight = "Bold",
            size = 24,
        })
        ui.label(
            "This surface is scripted from Luau and rendered directly through the frame host.",
            {
                tone = "Muted",
            }
        )

        profile_card.view(state)
    end)
end

return module
