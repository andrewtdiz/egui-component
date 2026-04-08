--!strict

local ui_types = require("../core/types")
local bridge = require("../core/bridge")
local kit = require("../../ui.luau")

type CardProps = ui_types.CardProps

export type ProfilePanelConfig = {
    eyebrow: string?,
    source_label: string?,
    card_props: CardProps?,
    placeholder: string?,
}

local module = {}

local function resolve_card_props(config: ProfilePanelConfig?): CardProps
    if config == nil or config.card_props == nil then
        return kit.tokens.card_panel
    end
    return kit.variant.merge(kit.tokens.card_panel, config.card_props)
end

function module.render(state: any, config: ProfilePanelConfig?)
    local app = bridge.current_app()
    local eyebrow = if config ~= nil and config.eyebrow ~= nil
        then config.eyebrow
        else "Library-backed Surface"
    local source_label = if config ~= nil and config.source_label ~= nil
        then config.source_label
        else tostring(state.variant or "examples/runtime-luau/apps/demo.luau")
    local placeholder = if config ~= nil and config.placeholder ~= nil
        then config.placeholder
        else "Enter a display name"

    kit.card.surface(resolve_card_props(config), function()
        kit.stack.column({ gap = kit.tokens.spacing.sm }, function()
            kit.text.kicker(eyebrow)
            kit.text.title("Hello, " .. tostring(state.name) .. " from " .. source_label)
            kit.text.meta(
                "Count: "
                    .. tostring(state.count)
                    .. " | Last event: "
                    .. tostring(state.last_event)
            )
            kit.text.meta("Reloads: " .. tostring(state.reloads or 0))

            kit.stack.with_id("profile", function()
                kit.stack.row({ gap = kit.tokens.spacing.sm }, function()
                    if kit.button.primary("increment", "Increment") then
                        state.count = (state.count or 0) + 1
                        state.last_event = "increment"
                    end

                    if kit.button.secondary("reload", "Reload Script", {
                        width = 140,
                    }) then
                        state.last_event = "reload"
                        app.request_reload()
                    end
                end)

                local next_name, changed = kit.field.text({
                    id = "name_field",
                    label = "Display name",
                    value = tostring(state.name),
                    description = "State stays stable across rerender and successful reload.",
                    placeholder = placeholder,
                    width = kit.tokens.width.input,
                })
                if changed then
                    state.name = next_name
                    state.last_event = "name:" .. next_name
                end
            end)
        end)
    end)
end

return module
