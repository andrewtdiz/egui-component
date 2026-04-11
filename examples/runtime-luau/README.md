# Retired Luau Runtime Sources

The compiled Luau runtime host examples were retired with the Luau runtime crate. This directory remains as historical source reference only.

Use `cargo run --example runtime-jsx-host` for the active authored-runtime example.

## Historical Hierarchy

This tree is organized around one rule: runtime roots live in `apps/`, and the reusable library lives in `ui/`.

- `examples/runtime-luau/apps/demo.luau`
  Canonical library-backed demo root.
- `examples/runtime-luau/apps/panel.luau`
  Raw direct `ui.*` debug surface kept for bridge validation.
- `examples/runtime-luau/apps/showcase.luau`
  Recipe showcase root.
- `examples/runtime-luau/apps/gallery.luau`
  Component catalog root.
- `examples/runtime-luau/ui.luau`
  Public Luau library entrypoint.
- `examples/runtime-luau/ui/core/*.luau`
  Runtime-facing internals: typed bridge access, layout scopes, tokens, merge helpers, generated types.
- `examples/runtime-luau/ui/components/*.luau`
  The only component directory. Each component is a single self-contained file with its API and preview surface.
- `examples/runtime-luau/ui/recipes/*.luau`
  Higher-level composed surfaces built from `ui/components`.

The rendering flow is:

1. Rust mounts one runtime root from `apps/*.luau`.
2. That root imports `ui.luau` plus any `ui/recipes/...` modules it needs.
3. Recipes compose `ui/components/...`.
4. Components bottom out in `ui/core/...`, which is the only layer that talks to the frame-local `app` and `ui` globals.

There is only one component directory in this example tree: `ui/components/`.

## Historical Runtime Contract

The root script must return a table. These hooks are supported:

- `init(state)`
  Called once on first successful load.
- `update(state, input)`
  Called before `render` on committed steady-state frames. `input` is currently a reserved empty table.
- `render(state)`
  Called every frame. This is where scripts call `ui.*` directly.
- `reload(old_exports, state)`
  Called on successful replacement of an existing module.
- `shutdown(state)`
  Called on the old module after a successful swap.

`state` is persistent across frames and successful reloads. On failed reload, the previous graph keeps running.

Reload contract:

- rollback guarantees which runtime graph stays active, not full VM-state isolation
- active and staged graphs share one Lua heap
- unaffected modules may be reused by handle during reload
- reloadable `state` must be plain-data tables only
- supported reloadable values are tables, booleans, integers, numbers, and strings
- functions, userdata, threads, and similar values are not reloadable state

Module discipline:

- put mutable reload-persistent data in `state`
- treat module exports as immutable after load/reload
- prefer stateless shared helpers; module-local caches and mutated exports live in shared heap memory
- `reload` can produce side effects outside `state` if it mutates shared modules or exports

Shutdown contract:

- after a staged candidate's first render succeeds, the runtime commits the new graph before running old-module `shutdown`
- `shutdown` runs against the old require snapshot
- shutdown failure is surfaced as a runtime error, but it does not restore the previous graph

The shipped demo and showcase use Luau `state` for demo-local presentation state. For real product surfaces, keep durable document state, transactions, and undoable mutations in the Rust host. See the migration guide:

- [`docs/luau-runtime-migration-guide.md`](/home/andy/Documents/egui-component/docs/luau-runtime-migration-guide.md)

## Host API

The public runtime surface is intentionally small:

- Generated Luau editor typings:
  [`examples/runtime-luau/ui/core/types.luau`](/home/andy/Documents/egui-component/examples/runtime-luau/ui/core/types.luau)
  Kept inside `ui/core/` because it describes the host bridge rather than product components.
- Generated markdown API reference:
  [`docs/luau-runtime-api-reference.md`](/home/andy/Documents/egui-component/docs/luau-runtime-api-reference.md)

The direct `ui.*` bridge is immediate-mode and frame-local. Scripted components should be composed as normal Luau functions that call these primitives, and scope/id stacks must be balanced before `render` returns.

The shipped examples opt into the generated typings with `--!strict`, `require("../ui/core/types")` from app roots, and typed local aliases for the mounted `app` and `ui` globals.

## Recommended Luau Composition

The repo-local component library entrypoint is:

- [`examples/runtime-luau/ui.luau`](/home/andy/Documents/egui-component/examples/runtime-luau/ui.luau)

Use it from product-facing surfaces:

```luau
--!strict

local kit = require("../ui.luau")
local profile_panel = require("../ui/recipes/profile_panel")

function module.render(state)
    kit.stack.column({ gap = kit.tokens.spacing.md }, function()
        kit.text.title("Library-backed surface")
        profile_panel.render(state)
    end)
end
```

This library is intentionally Luau-only:

- presets and variants expand in Luau before crossing into Rust
- helper modules call the direct typed `ui.*` bridge
- no retained IR or contract tree is built on the frame path

Use raw `ui.*` directly in `apps/panel.luau` when you are debugging the host bridge or validating a primitive. Use `ui/recipes/*.luau` for reusable product-facing surfaces, and use `ui/components/*.luau` when you want the primitive-by-primitive catalog surface.

Use the explicit file path `require("../ui.luau")` from app roots so both the embedded runtime and `luau-lsp` resolve the same entrypoint. The public library module is `ui.luau`; host-facing internals stay under `ui/core/`, components stay under `ui/components/`, and composed surfaces stay under `ui/recipes/`.

Scoped helpers such as `kit.stack.column(...)`, `kit.card.surface(...)`, and `kit.stack.with_id(...)` use Luau-only callbacks and always unwind their scopes before rethrowing callback errors.

Validate the example tree with:

```bash
luau-lsp analyze --platform=standard examples/runtime-luau
```

`ui.end_scope()` is the callable close operation because `end` is a reserved Luau keyword.

Stateful widget identity is explicit:

- full widget identity is `surface_id + push_id stack + local_id`
- only `ui.push_id(...)` scopes affect identity
- layout scopes such as row, column, and card do not affect identity

Current `props` tables are closed schemas. Unknown prop keys are runtime errors instead of being ignored.

Phase behavior is explicit:

- staged first-frame validation after load or reload runs `render(state)` only
- committed steady-state frames run `update(state, input)` before `render(state)`
- `ui.*` is render-only and will fail if called from `update`
- `app.request_reload()` and `app.request_repaint()` are deferred frame commands and only apply after a successful frame
- `app.log(...)` remains an immediate diagnostic sink

## Boundary

Keep composition, styling presets, and reusable view helpers in Luau.

Keep rendering, frame ownership, widget execution, file watching, runtime-selection rollback, dependency tracking, and host correctness in Rust.
Keep durable document state, transactions, and undoable mutations in Rust as well; the Luau example surfaces keep that state local only because they are demos.
