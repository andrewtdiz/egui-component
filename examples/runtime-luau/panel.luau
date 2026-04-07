local module = {}
local module_state = nil

function module.init(state)
    module_state = state
    state.reloads = state.reloads or 0
end

function module.reload(old_exports, state)
    module_state = state
    state.reloads = (state.reloads or 0) + 1
    app.log("info", "panel.luau reloaded")
end

function module.view(state)
    ui.card({
        width = 360,
        padding_x = 20,
        padding_y = 20,
    }, function()
        ui.vertical({ gap = 12 }, function()
            ui.label("Immediate Mode Surface", {
                tone = "Muted",
                size = 12,
            })

            ui.label(
                "Hello, " .. state.name .. " from " .. (state.variant or "immediate panel"),
                {
                    weight = "Bold",
                    size = 22,
                }
            )

            ui.label(
                "Leaf reloads: " .. tostring(module_state.reloads),
                {
                    tone = "Muted",
                }
            )

            ui.horizontal({ gap = 10 }, function()
                if ui.button("profile.increment", "Increment", {
                    variant = "Primary",
                    width = 120,
                }) then
                    state.count = (state.count or 0) + 1
                    state.last_event = "increment"
                end

                if ui.button("profile.reload", "Reload Script", {
                    variant = "Secondary",
                    width = 140,
                }) then
                    state.last_event = "reload"
                    app.request_reload()
                end
            end)

            local value, changed = ui.text_edit("profile.name", state.name, {
                placeholder = "Enter a name",
                width = 280,
            })
            if changed then
                state.name = value
                state.last_event = "name:" .. value
            end
        end)
    end)
end

return module
