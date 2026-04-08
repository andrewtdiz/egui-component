--!strict

local bridge = require("../core/bridge")
local kit = require("../../ui.luau")

local OPEN_FILES = table.freeze({
    "src/lib.rs",
    "src/example_apps/runtime_egui_host.rs",
    "examples/runtime-luau/apps/demo/main.luau",
    "examples/runtime-luau/ui/recipes/editor_sidebar.luau",
    "luau-runtime/src/runtime/api.rs",
})

local DEFAULT_EDITOR = {
    query = "type:button variant:primary",
    note = "Use Luau helpers for presets and recipes.",
    open_files = OPEN_FILES,
}

local module = {}

function module.render(state: any)
    local app = bridge.current_app()
    local editor = state.editor
    if editor == nil then
        editor = {
            query = DEFAULT_EDITOR.query,
            note = DEFAULT_EDITOR.note,
            selected_file_index = 1,
            selected_file = DEFAULT_EDITOR.open_files[1],
        }
        state.editor = editor
    end

    kit.card.surface(kit.tokens.card_sidebar, function()
        kit.stack.column({ gap = kit.tokens.spacing.sm }, function()
            kit.text.kicker("Editor Recipe")
            kit.text.title("Editor Sidebar")
            kit.text.meta("Recipe modules stay in Luau; the bridge stays narrow and typed.")

            local query, query_changed = kit.field.text({
                id = "query",
                label = "Search Query",
                value = tostring(editor.query),
                description = "Represents transient editor filtering state.",
                width = kit.tokens.width.sidebar,
            })
            if query_changed then
                editor.query = query
                state.last_event = "editor:query"
            end

            local note, note_changed = kit.field.text({
                id = "note",
                label = "Scratch Note",
                value = tostring(editor.note),
                description = "Recipe-local presentation state can stay in Luau.",
                width = kit.tokens.width.sidebar,
            })
            if note_changed then
                editor.note = note
                state.last_event = "editor:note"
            end

            kit.text.meta("Buffered query: " .. tostring(editor.query))
            kit.text.meta("Virtual list keeps long file sets native to the host.")

            local selected_index, selection_changed = kit.list.render(
                "open_files",
                DEFAULT_EDITOR.open_files,
                {
                    width = kit.tokens.width.sidebar,
                    height = 132,
                    row_height = 24,
                    selected_index = tonumber(editor.selected_file_index) or 0,
                }
            )
            if selection_changed then
                editor.selected_file_index = selected_index
                editor.selected_file = DEFAULT_EDITOR.open_files[selected_index]
                state.last_event = "editor:file:" .. tostring(editor.selected_file)
            end

            kit.text.meta("Focused file: " .. tostring(editor.selected_file))

            kit.stack.row({ gap = kit.tokens.spacing.xs }, function()
                if kit.button.secondary("apply", "Apply", {
                    width = 96,
                }) then
                    state.last_event = "editor:apply"
                end
                if kit.button.ghost("refresh", "Refresh Preview", {
                    width = 140,
                }) then
                    state.last_event = "editor:refresh"
                    app.request_repaint()
                end
            end)
        end)
    end)
end

return module
