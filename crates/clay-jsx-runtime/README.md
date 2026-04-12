# clay-jsx-runtime

Configurable `deno_core`/V8 runtime primitives for Clay JSX/TSX host bridges.

This crate owns the host-neutral pieces:

1. `RuntimeSession` creates and retains a `deno_core::JsRuntime`.
2. `deno_ast` transpiles `.jsx`, `.tsx`, and `.ts` modules with a caller-provided automatic JSX import source.
3. Callers register virtual modules such as `clay`, `clay/jsx-runtime`, or renderer-specific aliases.
4. JS can commit JSON batches through `Deno.core.ops.op_commit_mutations(...)` and write diagnostics through `op_host_log(...)`.
5. Renderer-specific crates interpret the committed JSON and own their contract model, validation, retained tree, and event dispatch semantics.

The egui-component bridge lives in `crates/clay-jsx-egui-bridge`.
