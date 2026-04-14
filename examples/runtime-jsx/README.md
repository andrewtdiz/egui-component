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
2. `crates/clay-jsx-runtime` owns the host-neutral `deno_core::JsRuntime` session, `.jsx`/`.tsx` transpilation through `deno_ast`, and the shared React/reconciler/scheduler virtual modules.
3. `crates/clay-jsx-egui-bridge/src/mod.js`, `jsx_runtime_api.js`, `runtime_api.js`, `lowering_api.js`, and `motion_api.js` provide the virtual `egui`, `clay`, `motion/react`, and `react/motion` modules plus the egui-specific lowering and event runtime.
4. `render(<App />)` updates one persistent React root for the session.
5. The stable `egui` / `clay` surface now exposes the supported React hooks and helpers: `useState`, `useEffect`, `useReducer`, `useRef`, `useContext`, `useSyncExternalStore`, `startTransition`, `useDeferredValue`, `createContext`, `render`, `eventValue`, `log`, and `requestRepaint`.
6. The host-neutral runtime installs `setTimeout`, `setInterval`, `requestAnimationFrame`, and explicit wake plumbing so timer-driven or effect-driven state updates can request the next egui frame without relying on incidental input.
7. After each React commit, the bridge lowers the committed host tree into normalized egui contract descriptors and diffs those normalized descriptors into semantic host mutation batches.
8. Rust applies those mutations to a retained host tree, materializes a `ContractTree`, checks the contract model version and registered families, and renders it with `egui_component::contract::render_tree`.
9. Motion props such as `initial`, `animate`, and `transition` are stored beside the host tree and ticked by Rust as retained numeric values. The current egui renderer does not consume those values yet.
10. egui events are sent back into the retained V8 session so React handlers and state can update the next host-tree commit without reloading the file. Async and timer callbacks follow the same retained-tree path once the host drains pending runtime work on wake.

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

The authored entrypoint is `examples/runtime-jsx/app.jsx`. Edit it or any imported `.js`, `.jsx`, `.ts`, `.tsx`, or `.json` module on disk and the rendered egui surface reloads automatically. The default app now includes effect-driven demos for timers, async reducer transitions, deferred values, context, refs, and `useSyncExternalStore` subscriptions. The host uses a watcher-driven wake-up path instead of per-frame file polling, and it also drains runtime-driven wake-ups from timers/effects before each frame render. Reloads rebuild a fresh JS session and remount cold in this cutover bundle; hook-state restoration is deferred until a later pass. Failed rebuilds still keep watching the attempted dependency set so fixing the broken file or newly introduced import wakes the next reload automatically.

The focused motion entrypoint is `examples/runtime-jsx/motion-sync.tsx`. It authors opacity, translation, scale, and rotation values in TSX, then `runtime-jsx-motion` ticks those retained values from egui time and draws them with the egui painter.
