# Taffy Layout In `egui-component`

Use `egui_component::layout::*` when native `egui` stacks are not enough and you need real flex or grid layout.

For simple rows and columns, keep using `ui.horizontal`, `ui.vertical`, or the higher-level helpers in `layout.rs`. Reach for taffy when you need:

- flex growth or shrink behavior
- grid columns or rows
- explicit gaps, min sizes, and stretch behavior
- stable layout for repeated or dynamic children

## Imports

```rust
use egui::Id;
use egui_component::layout::{taffy, tid, tui, TuiBuilderLogic};
use egui_component::layout::taffy::prelude::{auto, fr, length, percent};
```

## Mental Model

There are three layers:

1. `tui(ui, root_id)` creates one taffy layout tree inside an `egui::Ui`.
2. `.style(taffy::Style { ... })` defines the root container layout.
3. Inside `.show(|tui| { ... })`, each child becomes either:
   - a container node with `.add(...)`
   - a measured leaf with `.ui(...)`
   - a custom-measured leaf with `.ui_manual(...)`

## Basic Pattern

```rust
tui(ui, Id::new("actions"))
    .reserve_available_width()
    .style(taffy::Style {
        flex_direction: taffy::FlexDirection::Row,
        gap: length(8.0),
        size: taffy::Size {
            width: percent(1.0),
            height: auto(),
        },
        ..Default::default()
    })
    .show(|tui| {
        tui.id(tid("save")).ui(|ui| {
            let _ = ui.components().button(Button::new("Save"));
        });

        tui.id(tid("cancel")).ui(|ui| {
            let _ = ui.components().button(Button::new("Cancel"));
        });
    });
```

## Root Sizing

The root node needs space from egui. The common options are:

- `.reserve_available_width()` for full-width rows and grids
- `.reserve_available_height()` for full-height layouts
- `.reserve_available_space()` for both
- `.reserve_width(...)`, `.reserve_height(...)`, or `.reserve_space(...)` for fixed space
- `.with_allocated_rect(rect)` if the parent already allocated the region

If you do not reserve enough space, the layout can compute against the wrong available size.

## IDs

Use a stable root `Id` for the layout tree and stable child ids with `tid(...)`.

```rust
tui(ui, Id::new("inventory_grid"))
    .show(|tui| {
        for item in items {
            tui.id(tid(("item", item.id))).ui(|ui| {
                ui.label(&item.name);
            });
        }
    });
```

Use `tid(...)` for repeated or dynamic children. Stable ids let the internal taffy tree match nodes across frames.

## The Methods You Usually Need

- `.style(style)` sets the taffy style for the next node
- `.id(tid(...))` gives the next node a stable identity
- `.add(|tui| { ... })` creates a container node with children
- `.ui(|ui| { ... })` creates a normal measured leaf node
- `.ui_infinite(|ui| { ... })` creates a leaf that can grow without a hard max in one or both dimensions
- `.ui_manual(|ui, container| { ... })` is for custom measurement or custom widgets

Most usage is just `.add(...)` for containers and `.ui(...)` for leaves.

## Flex Example

```rust
tui(ui, Id::new("toolbar"))
    .reserve_available_width()
    .style(taffy::Style {
        flex_direction: taffy::FlexDirection::Row,
        align_items: Some(taffy::AlignItems::Center),
        gap: length(10.0),
        size: taffy::Size {
            width: percent(1.0),
            height: auto(),
        },
        ..Default::default()
    })
    .show(|tui| {
        tui.id(tid("title"))
            .style(taffy::Style {
                flex_grow: 1.0,
                ..Default::default()
            })
            .ui(|ui| {
                ui.label("Project");
            });

        tui.id(tid("save")).ui(|ui| {
            let _ = ui.components().button(Button::new("Save"));
        });

        tui.id(tid("publish")).ui(|ui| {
            let _ = ui.components().button(Button::new("Publish"));
        });
    });
```

## Grid Example

```rust
tui(ui, Id::new("asset_grid"))
    .reserve_available_width()
    .style(taffy::Style {
        display: taffy::Display::Grid,
        grid_template_columns: vec![fr(1.0); 3],
        gap: length(10.0),
        size: percent(1.0),
        align_items: Some(taffy::AlignItems::Stretch),
        justify_items: Some(taffy::AlignItems::Stretch),
        ..Default::default()
    })
    .show(|tui| {
        for index in 0..9 {
            tui.id(tid(("tile", index)))
                .style(taffy::Style {
                    size: taffy::Size {
                        width: auto(),
                        height: length(92.0),
                    },
                    ..Default::default()
                })
                .ui(|ui| {
                    let _ = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 92.0),
                        egui::Sense::hover(),
                    );
                });
        }
    });
```

## When To Use `.add(...)` Vs `.ui(...)`

Use `.add(...)` when the node itself contains more taffy children.

Use `.ui(...)` when the node is just ordinary egui content and should be measured as a leaf.

```rust
tui.id(tid("panel"))
    .style(panel_style)
    .add(|tui| {
        tui.id(tid("heading")).ui(|ui| {
            ui.heading("Panel");
        });

        tui.id(tid("body")).ui(|ui| {
            ui.label("Leaf content");
        });
    });
```

## Practical Rules

- Keep one clear root `Id` per layout tree.
- Give repeated children stable `tid(...)` values.
- Reserve root space before `.show(...)`.
- Use `percent(1.0)` on container width when you want full available width.
- Put explicit heights on grid tiles or any leaf that should not collapse.
- Use plain egui layout for trivial cases. Taffy is for layout problems, not every row.

## If You Need More

The public surface is re-exported from `egui_component::layout::*`.

Start with:

- `tui(...)`
- `tid(...)`
- `taffy::Style`
- `taffy::prelude::{auto, fr, length, percent}`
- `TuiBuilderLogic`

If a layout still feels hard, inspect these working examples before reading the full internal implementation:

- `README.md`
- `src/example_apps/showcase/brand_kit_layout.rs`
- `src/example_apps/showcase.rs`
