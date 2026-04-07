local presets = require("../styles/presets")

local module = {}

function module.view(state)
    ui.card(presets.card, function()
        ui.vertical({ gap = 14 }, function()
            ui.label("Immediate Runtime Surface", presets.kicker)
            ui.label(state.name, presets.title)
            ui.label(
                "Count: " .. tostring(state.count) .. " | Last event: " .. tostring(state.last_event),
                presets.meta
            )

            ui.horizontal({ gap = 12 }, function()
                if ui.button("showcase.profile.increment", "Increment", presets.primary_button) then
                    state.count = (state.count or 0) + 1
                    state.last_event = "increment"
                end

                if ui.button("showcase.profile.save", "Save", presets.secondary_button) then
                    state.last_event = "save"
                end
            end)

            local value, changed = ui.text_edit("showcase.profile.name", state.name, presets.input)
            if changed then
                state.name = value
                state.last_event = "name"
            end
        end)
    end)
end

return module
