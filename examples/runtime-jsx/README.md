# JSX Runtime Example

This example keeps the window, input, and egui frame loop in Rust while the UI surface is authored in JSX.

Run it from the repo root:

```bash
cargo run --example runtime-jsx-host
```

Run the focused motion sync demo:

```bash
cargo run --example runtime-jsx-motion
```

Pass a different JSX entry file as the first argument:

```bash
cargo run --example runtime-jsx-host -- ./examples/runtime-jsx/app.jsx
```

The runtime path is:

1. `examples/runtime-jsx/mod.rs` wires the eframe shell and `examples/runtime-jsx/app.rs` owns the editor/preview UI.
2. `crates/clay-jsx-runtime` owns the host-neutral `deno_core::JsRuntime` session and `.jsx`, `.tsx`, and `.ts` transpilation through `deno_ast`.
3. `crates/clay-jsx-egui-bridge/src/mod.js`, `crates/clay-jsx-egui-bridge/src/runtime_api.js`, and `crates/clay-jsx-egui-bridge/src/motion_api.js` provide the virtual `egui`, `clay`, `motion/react`, and `react/motion` modules, `render`, `log`, `useState`, event dispatch, and the JSX runtime functions.
4. `render(<column ... />)` lowers JSX into host nodes and commits incremental mutation batches through a Deno op.
5. Rust applies those mutations to a retained host tree, materializes a `ContractTree`, checks the contract model version and registered families, and renders it with `egui_component::contract::render_tree`.
6. Motion props such as `initial`, `animate`, and `transition` are stored beside the host tree and ticked by Rust as retained numeric values. The current egui renderer does not consume those values yet.
7. egui events are sent back into the retained V8 session so JSX handlers and hooks can update the next host-tree commit without reloading the file.

The host tree is inspired by the retained instance pattern in `THREEJS_FIBER.md`: JS authors declarative nodes, while Rust owns the authoritative tree, render submission, and validation.

Motion authoring starts with a small numeric subset:

```jsx
import { motion } from "motion/react";

<motion.card
  id="panel"
  initial={{ opacity: 0, y: -8 }}
  animate={{ opacity: 1, y: 0 }}
  transition={{ duration: 0.2, ease: "easeOut" }}
/>
```

Supported values are `opacity`, `x`, `y`, `scale`, `scaleX`, `scaleY`, `rotate`, `width`, `height`, `gap`, `paddingX`, `paddingY`, and `cornerRadius`.

The authored entrypoint is `examples/runtime-jsx/app.jsx`. Edit it on disk and the rendered egui surface reloads automatically.

The focused motion entrypoint is `examples/runtime-jsx/motion-sync.tsx`. It authors opacity, translation, scale, and rotation values in TSX, then `runtime-jsx-motion` ticks those retained values from egui time and draws them with the egui painter.
