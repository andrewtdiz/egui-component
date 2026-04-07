local panel = require("./panel")

local module = {}

function module.init(state)
    state.name = state.name or "Luau"
    state.count = state.count or 0
    state.variant = state.variant or "runtime-luau/panel.luau"
    state.last_event = state.last_event or "boot"
end

function module.render(state)
    ui.vertical({ gap = 16 }, function()
        ui.label("Luau -> egui immediate bridge", {
            weight = "Bold",
            size = 24,
        })

        ui.label(
            "Edit examples/runtime-luau/panel.luau and save to live-update this surface.",
            {
                tone = "Muted",
            }
        )

        panel.view(state)

        ui.horizontal({ gap = 16 }, function()
            ui.label("Count: " .. tostring(state.count), {
                weight = "Semibold",
            })
            ui.label("Last event: " .. tostring(state.last_event), {
                tone = "Muted",
            })
        end)
    end)
end

return module
