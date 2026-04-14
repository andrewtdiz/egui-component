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

Run the JSX component catalog:

```bash
cargo run --example runtime-jsx-host -- ./examples/runtime-jsx/catalog.tsx
```

Shadcn-style component modules live under `examples/runtime-jsx/ui/components`. The full component barrel remains `examples/runtime-jsx/ui/components/index.tsx`. Standalone components with whole behavior and state live directly in `examples/runtime-jsx/ui/components`, such as `examples/runtime-jsx/ui/components/button.tsx`, `examples/runtime-jsx/ui/components/select.tsx`, and `examples/runtime-jsx/ui/components/dropdown-menu.tsx`. Primary element modules live under `examples/runtime-jsx/ui/components/primary` for visual or single-tag building blocks such as `examples/runtime-jsx/ui/components/primary/color.tsx`. Secondary modules live under `examples/runtime-jsx/ui/components/secondary` for composed recipes and catalog helpers such as `examples/runtime-jsx/ui/components/secondary/canva.tsx` and `examples/runtime-jsx/ui/components/secondary/palette-preview.tsx`. Shared behavior such as `className`, `classNames`, `classList`, `slotClasses`, `disabled`, generated ids, and child text extraction lives in `ui/component-support.ts`; shadcn-style variant composition lives in `ui/lib/variants.ts` and `ui/lib/styles.ts`. The default `app.jsx`, `motion-sync.tsx`, and `catalog.tsx` entrypoints import the component barrel instead of authoring raw contract tags directly.

```tsx
import { Button, Card, Checkbox } from "./ui/components/index.tsx";

<Card id="settings-panel" className="bg-card border border-border rounded-lg">
  <Button id="save" variant="primary" className="font-semibold">
    Save
  </Button>
  <Checkbox id="snap" checked label="Snap to grid" />
</Card>
```

`className`, `classNames`, and `classList` are forwarded into the egui contract class fields, and `slotClasses` is forwarded as contract metadata. Component state stays explicit in egui props such as `variant`, `selected`, `checked`, `open`, and `disabled`; the TSX components also derive state classes from those props before forwarding to egui.

Tailwind layout classes map onto the minimal egui flow layer for row, column, card, and inset containers: `flex`, `flex-row`, `flex-col`, `gap`, `gap-x`, `gap-y`, `items-start`, `items-center`, `items-end`, `items-stretch`, `justify-start`, `justify-center`, `justify-end`, `flex-wrap`, and `flex-nowrap`. Plain `<div>` also lowers to a row or column when authored with flex classes. When `items-*` is omitted on a Tailwind-authored flex container, the bridge now defaults the cross axis to stretch so text and fields occupy the available lane width the way authors expect from CSS flexbox. Use `<Spacer flex />` for `justify-between` or `justify-around` layouts because those do not have an exact egui flow-container equivalent.

Shared sizing props such as `width`, `height`, `minWidth`, `minHeight`, `maxWidth`, `maxHeight`, and `basis` are hoisted into `common.layout` for families that do not own those props directly, so labels and plain flow containers can be sized from TSX without dropping down to a raw `layout={{ ... }}` object.

Rust remains the internal JSX runtime layer for egui-owned primitives and behavior families: layout, native controls, popups, menus, trees, drag/drop, media, and retained event dispatch. Higher-level recipes should be authored in the hot-reloading TSX/Tailwind layer. The Rust APIs for Canva helpers, `ButtonGroup`, `Collapsible`, `EmojiSelector`, `Field`, `Pagination`, `PalettePreview`, `PaletteColorInput`, and `ToggleGroup` are deprecated compatibility surfaces. `ColorInput` and `ColorStrip` stay active internal Rust runtime controls because they wrap native egui color editing and continuous strip painting.

The runtime path is:

1. `examples/runtime-jsx/mod.rs` wires the eframe shell and `examples/runtime-jsx/app.rs` owns the editor/preview UI.
2. `crates/clay-jsx-runtime` owns the host-neutral `deno_core::JsRuntime` session and `.jsx`, `.tsx`, and `.ts` transpilation through `deno_ast`.
3. `crates/clay-jsx-egui-bridge/src/mod.js`, `crates/clay-jsx-egui-bridge/src/runtime_api.js`, and `crates/clay-jsx-egui-bridge/src/motion_api.js` provide the virtual `egui`, `clay`, `motion/react`, and `react/motion` modules, `render`, `log`, `useState`, event dispatch, and the JSX runtime functions.
4. `render(<App />)` lowers component-authored JSX into host contract nodes and commits incremental mutation batches through a Deno op.
5. Rust applies those mutations to a retained host tree, materializes a `ContractTree`, checks the contract model version and registered families, and renders it with `egui_component::contract::render_tree`.
6. Motion props such as `initial`, `animate`, and `transition` are stored beside the host tree and ticked by Rust as retained numeric values. The current egui renderer does not consume those values yet.
7. egui events are sent back into the retained V8 session so JSX handlers and hooks can update the next host-tree commit without reloading the file.

The host tree is inspired by the retained instance pattern in `THREEJS_FIBER.md`: JS authors declarative nodes, while Rust owns the authoritative tree, render submission, and validation.

Motion authoring starts with a small numeric subset:

```jsx
import { motion } from "motion/react";

<motion.div
  data-slot="card"
  id="panel"
  initial={{ opacity: 0, y: -8 }}
  animate={{ opacity: 1, y: 0 }}
  transition={{ duration: 0.2, ease: "easeOut" }}
/>
```

Supported values are `opacity`, `x`, `y`, `scale`, `scaleX`, `scaleY`, `rotate`, `width`, `height`, `gap`, `paddingX`, `paddingY`, and `cornerRadius`.

The authored entrypoint is `examples/runtime-jsx/app.jsx`. Edit it or any imported `.js`, `.jsx`, `.ts`, `.tsx`, or `.json` module on disk and the rendered egui surface reloads automatically. Reloads always rebuild a fresh JS session, then best-effort restore serializable `useState` for components that were still rendered in the last successful frame and whose component path/name plus hook count still match. Components with changed hook counts, changed identity, or non-serializable hook state remount cold. Failed rebuilds keep the last good hot-reload snapshot and continue watching the attempted dependency set so fixing the broken file or newly introduced import can restore prior serializable state on the next successful reload. Same-count hook reorders are still out of scope.

The focused motion entrypoint is `examples/runtime-jsx/motion-sync.tsx`. It authors opacity, translation, scale, and rotation values in TSX, then `runtime-jsx-motion` ticks those retained values from egui time and draws them with the egui painter.
