# egui-component

Standalone component showcase runtime extracted from Clay Engine.

## Commands

Run all components in a window:

```bash
cargo components
```

Run one component in a window:

```bash
cargo component button
```

Render one component headlessly to PNG:

```bash
cargo component button --headless --verify-png screenshots/component-button.png --max-frames 2
```

## Isolation

- This crate lives at `deps/egui-component/`.
- It has its own Cargo aliases in `deps/egui-component/.cargo/config.toml`.
- It uses its own build output in `deps/egui-component/target/`.
- It does not depend on the root Clay Engine crate.
