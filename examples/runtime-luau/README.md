# Luau Runtime Immediate Example

Run the canonical host from the repo root:

```bash
cargo run --example runtime-egui-host
```

Run the library-backed showcase wrapper with:

```bash
cargo run --example showcase-runtime
```

Run the primitive component gallery with:

```bash
cargo run --example component-gallery-runtime
```

Edit these files while it is running:

- `examples/runtime-luau/demo.luau`
  Library-first root surface for the main demo.
- `examples/runtime-luau/panel.luau`
  Low-level direct `ui.*` sample kept for debugging and host API validation.
- `examples/runtime-luau/ui.luau`
  Public entrypoint for the repo-local Luau component library.
- `examples/runtime-luau/ui/components/*.luau`
  Single-source component modules. Each file owns the reusable API and its gallery/demo surface.
- `examples/runtime-luau/components/*.luau`
  Reusable product-style recipes built on the direct bridge.
- `examples/runtime-luau/showcase/main.luau`
  Library-backed showcase entry point.
- `examples/runtime-luau/ui/components/main.luau`
  Luau-authored component catalog shell mounted by the Rust-owned gallery app.
- `examples/runtime-luau/ui/components/catalog.luau`
  Central component registry for the catalog shell.

## Runtime Contract

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
  [`examples/runtime-luau/ui/types.luau`](/home/andy/Documents/egui-component/examples/runtime-luau/ui/types.luau)
  Kept inside the `ui/` package so strict Luau surfaces import types and helpers from the same namespace.
- Generated markdown API reference:
  [`docs/luau-runtime-api-reference.md`](/home/andy/Documents/egui-component/docs/luau-runtime-api-reference.md)

The direct `ui.*` bridge is immediate-mode and frame-local. Scripted components should be composed as normal Luau functions that call these primitives, and scope/id stacks must be balanced before `render` returns.

The shipped examples opt into the generated typings with `--!strict`, `require("./ui/types")`, and typed local aliases for the mounted `app` and `ui` globals.

## Recommended Luau Composition

The repo-local component library entrypoint is:

- [`examples/runtime-luau/ui.luau`](/home/andy/Documents/egui-component/examples/runtime-luau/ui.luau)

Use it from product-facing surfaces:

```luau
--!strict

local kit = require("./ui.luau")
local profile_panel = require("./components/profile_panel")

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

Use raw `ui.*` directly in `panel.luau` when you are debugging the host bridge or validating a primitive. Use the component library for reusable product-facing recipes, and use `ui/components/*.luau` when you want the primitive-by-primitive catalog surface.

Use the explicit file path `require("./ui.luau")` so both the embedded runtime and `luau-lsp` resolve the same entrypoint. The public library module is `ui.luau`; shared helpers stay under `ui/`, and catalogable components now live under `ui/components/`.

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
