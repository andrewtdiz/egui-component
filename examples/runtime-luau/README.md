# Luau Runtime Immediate Example

Run the canonical host from the repo root:

```bash
cargo run --example runtime-egui-host
```

Edit these files while it is running:

- `examples/runtime-luau/demo.luau`
  Owns app-level state and the main frame layout.
- `examples/runtime-luau/panel.luau`
  Owns a reusable immediate-mode card surface and is the main hot-edit target.
- `examples/runtime-luau/showcase/main.luau`
  Alternate direct-ui example entry point.

## Runtime Contract

The root script must return a table. These hooks are supported:

- `init(state)`
  Called once on first successful load.
- `render(state)`
  Called every frame. This is where scripts call `ui.*` directly.
- `reload(old_exports, state)`
  Called on successful replacement of an existing module.
- `shutdown(state)`
  Called on the old module after a successful swap.

`state` is persistent across frames and successful reloads. On failed reload, the previous graph keeps running.

## Host API

The public runtime surface is intentionally small:

- `app.log(level, message)`
- `app.request_reload()`
- `app.request_repaint()`
- `ui.label(text, props?)`
- `ui.separator()`
- `ui.button(id, text, props?) -> clicked`
- `ui.text_edit(id, value, props?) -> value, changed`
- `ui.horizontal(props?, fn)`
- `ui.vertical(props?, fn)`
- `ui.card(props?, fn)`
- `ui.window(id, title, props?, fn)`

The direct `ui.*` bridge is immediate-mode and frame-local. Scripted components should be composed as normal Luau functions that call these primitives.

## Boundary

Keep composition, styling presets, and reusable view helpers in Luau.

Keep rendering, frame ownership, widget execution, file watching, reload rollback, dependency tracking, and host correctness in Rust.
