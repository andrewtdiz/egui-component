--!strict

local kit = require("../ui.luau")

local DEFAULT_SETTINGS = {
    workspace = "egui-component",
    profile = "runtime-dev",
    command = "cargo run --example runtime-egui-host",
}

local module = {}

function module.render(state: any)
    local settings = state.settings
    if settings == nil then
        settings = {
            workspace = DEFAULT_SETTINGS.workspace,
            profile = DEFAULT_SETTINGS.profile,
            command = DEFAULT_SETTINGS.command,
        }
        state.settings = settings
    end

    kit.card.surface(kit.tokens.card_sidebar, function()
        kit.stack.column({ gap = kit.tokens.spacing.sm }, function()
            kit.text.kicker("Settings Recipe")
            kit.text.title("Workspace Settings")
            kit.text.meta("Thin Luau composition over labels, text inputs, and buttons.")

            local workspace, workspace_changed = kit.field.text({
                id = "workspace",
                label = "Workspace",
                value = tostring(settings.workspace),
                description = "Project name or surface group.",
                width = kit.tokens.width.sidebar,
            })
            if workspace_changed then
                settings.workspace = workspace
                state.last_event = "settings:workspace"
            end

            local profile, profile_changed = kit.field.text({
                id = "profile",
                label = "Profile",
                value = tostring(settings.profile),
                description = "Semantic preset or environment name.",
                width = kit.tokens.width.sidebar,
            })
            if profile_changed then
                settings.profile = profile
                state.last_event = "settings:profile"
            end

            local command, command_changed = kit.field.text({
                id = "command",
                label = "Launch Command",
                value = tostring(settings.command),
                description = "Stored as script state, not host state.",
                width = kit.tokens.width.sidebar,
            })
            if command_changed then
                settings.command = command
                state.last_event = "settings:command"
            end

            kit.stack.row({ gap = kit.tokens.spacing.xs }, function()
                if kit.button.primary("save", "Save", {
                    width = 96,
                }) then
                    state.last_event = "settings:save"
                end
                if kit.button.ghost("reset", "Reset", {
                    width = 96,
                }) then
                    settings.workspace = DEFAULT_SETTINGS.workspace
                    settings.profile = DEFAULT_SETTINGS.profile
                    settings.command = DEFAULT_SETTINGS.command
                    state.last_event = "settings:reset"
                end
            end)
        end)
    end)
end

return module
