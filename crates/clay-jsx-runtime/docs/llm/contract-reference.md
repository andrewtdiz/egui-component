# Contract Reference

Generated from `clay_jsx_runtime::contract::registry()` and `clay_jsx_runtime::contract::shared_types()`.

Export the schema and human-readable reference from Rust with:

```rust
clay_jsx_runtime::contract::schema_json_pretty()
clay_jsx_runtime::contract::reference_markdown()
```

## Position In The Runtime

`contract::*` is the active declarative render boundary for the JSX runtime and the remaining host-driven compatibility surfaces.

- The `clay-jsx-egui-bridge` crate in `crates/clay-jsx-egui-bridge` lowers authored JSX/TSX into Rust-owned host nodes, then materializes this contract tree before Rust renders it.
- `ContractTree` is useful when the runtime or a compatibility host wants a serializable declarative surface, schema tooling, or change-driven tree materialization.
- Renderer ownership stays in Rust; the contract tree is the data boundary, not a separate JS renderer.

## Shared Node Fields

| Field | Type | Support | Summary |
| --- | --- | --- | --- |
| `node_id` | `string` | `supported` | Stable host-owned node identifier. |
| `visible` | `boolean` | `supported` | Whether the node renders at all. Defaults to true. |
| `enabled` | `boolean` | `supported` | Whether interaction is enabled. Defaults to true. |
| `class` | `string` | `supported` | Optional primary class string. |
| `class_list` | `list<string>` | `supported` | Optional expanded class token list. |
| `slot_classes` | `map<string, string>` | `unsupported` | Optional slot-name to class-string overrides. Declared in the model and schema only. The current Rust renderer does not apply slot-specific class behavior. |
| `actions` | `object:actions` | `unsupported` | Optional common semantic action bindings. Declared in the model and schema only. The current renderer uses family-specific action fields instead. |
| `layout` | `object:layout` | `partial` | Optional shared layout hints. The renderer applies sizing, padding, and margin on every node and container direction/justify/align/gap/wrap overrides on flow containers only. |

## Layout Support

| Field | Type | Support | Summary |
| --- | --- | --- | --- |
| `display` | `enum:layout_display` | `partial` | Optional layout mode override. Executed by the taffy-backed flow-container renderer used by row, column, inset, and card. `overlay` remains unsupported. |
| `direction` | `enum:layout_direction` | `partial` | Optional row or column direction override. Only executed by the flow-container helpers used by row, column, inset, and card. |
| `grow` | `number` | `partial` | Optional flex grow factor. Executed as child-item layout when the node is placed inside the taffy-backed flow-container renderer used by row, column, inset, and card. |
| `shrink` | `number` | `partial` | Optional flex shrink factor. Executed as child-item layout when the node is placed inside the taffy-backed flow-container renderer used by row, column, inset, and card. |
| `basis` | `object:layout_length` | `partial` | Optional flex basis length. Executed as child-item layout when the node is placed inside the taffy-backed flow-container renderer used by row, column, inset, and card. |
| `width` | `object:layout_length` | `supported` | Optional width override. |
| `height` | `object:layout_length` | `supported` | Optional height override. |
| `min_width` | `object:layout_length` | `supported` | Optional minimum width override. |
| `min_height` | `object:layout_length` | `supported` | Optional minimum height override. |
| `max_width` | `object:layout_length` | `supported` | Optional maximum width override. |
| `max_height` | `object:layout_length` | `supported` | Optional maximum height override. |
| `gap_x` | `number` | `partial` | Optional horizontal gap override. Only executed by the flow-container helpers used by row, column, inset, and card. |
| `gap_y` | `number` | `partial` | Optional vertical gap override. Only executed by the flow-container helpers used by row, column, inset, and card. |
| `padding` | `object:layout_edges` | `partial` | Optional padding edges. Executed as an egui frame inner margin on every node. Percent class-derived padding is ignored because egui margins are pixel based. |
| `margin` | `object:layout_edges` | `supported` | Optional margin edges. |
| `align` | `enum:align` | `partial` | Optional cross-axis alignment override. Only executed by the flow-container helpers used by row, column, inset, and card. |
| `justify` | `enum:justify` | `partial` | Optional main-axis alignment override. Only executed by the flow-container helpers used by row, column, inset, and card. |
| `wrap` | `boolean` | `partial` | Optional wrap hint. Only executed by the flow-container helpers used by row, column, inset, and card. Wrap-reverse is not represented. |
| `columns` | `list<object:layout_track>` | `partial` | Optional grid column tracks. Executed when the taffy-backed flow-container renderer used by row, column, inset, and card is switched to grid display. |
| `rows` | `list<object:layout_track>` | `partial` | Optional grid row tracks. Executed when the taffy-backed flow-container renderer used by row, column, inset, and card is switched to grid display. |
| `col_span` | `number` | `partial` | Optional grid column span. Executed as child-item grid placement when the parent uses the taffy-backed flow-container renderer in grid mode. |
| `row_span` | `number` | `partial` | Optional grid row span. Executed as child-item grid placement when the parent uses the taffy-backed flow-container renderer in grid mode. |
| `overflow_x` | `enum:layout_overflow` | `partial` | Optional horizontal overflow mode. Executed by the taffy-backed flow-container renderer used by row, column, inset, and card for `visible`, `hidden`, and `scroll`. |
| `overflow_y` | `enum:layout_overflow` | `partial` | Optional vertical overflow mode. Executed by the taffy-backed flow-container renderer used by row, column, inset, and card for `visible`, `hidden`, and `scroll`. |

## Supported Families

| Family | Child Policy | Primary Events | Summary |
| --- | --- | --- | --- |
| `row` | children | none | Horizontal layout container. |
| `column` | children | none | Vertical layout container. |
| `inset` | children | none | Padding wrapper around child nodes. |
| `sized-box` | children | none | Explicit width and height wrapper. |
| `spacer` | none | none | Fixed or flex spacer. |
| `card` | children | none | General framed surface container. |
| `sidebar` | children | `closed` | Overlay sidebar container. |
| `toolbar` | children | none | Anchored floating toolbar surface. |
| `menu-bar` | none | `command_invoked` | Desktop-style menu bar with action entries and separators. |
| `tabs` | none | `selected` | Single-selection tab navigation. |
| `label` | none | none | Semantic text presentation. |
| `button` | none | `clicked` | Clickable action button. |
| `button-group` | none | `command_invoked` | Attached action group. |
| `input` | none | `changed`, `submitted` | Single-line text input. |
| `number-input` | none | `changed` | Numeric entry control. |
| `checkbox` | none | `toggled` | Boolean checkbox control. |
| `switch` | none | `toggled` | Boolean switch control. |
| `select` | none | `selected` | Single-choice select menu. |
| `field` | none | `changed` | Label plus text input plus helper text. |
| `separator` | none | none | Visual content divider. |
| `collapsible` | children | `toggled`, `opened`, `closed` | Expandable section with body children. |
| `dialogue-modal` | body | `confirmed`, `cancelled`, `closed` | Host-owned modal confirmation surface. |
| `hierarchy` | none | `selected`, `opened`, `closed` | Hierarchical selection tree with expand and collapse state. |
| `spinner` | none | none | Indeterminate loading spinner. |
| `progress` | none | none | Determinate progress indicator. |
| `toast-viewport` | none | `opened`, `closed` | Overlay toast stack with lifecycle events. |
| `color` | none | none | Color swatch primitive. |
| `icon` | none | none | Icon glyph primitive. |
| `image` | none | none | Raster image primitive. |
| `twemoji` | none | none | Twemoji image primitive. |
| `kbd` | none | none | Keyboard keycap primitive. |
| `skeleton` | none | none | Animated placeholder primitive. |
| `slider` | none | `changed` | Numeric range slider. |
| `radio` | none | `toggled` | Single boolean radio option. |
| `radio-group` | none | `selected` | Mutually exclusive radio options. |
| `combobox` | none | `selected` | Filterable multi-select picker. |
| `emoji-selector` | none | `selected` | Button-triggered emoji picker. |
| `pagination` | none | `selected` | Page-number navigation. |
| `tooltip` | none | none | Hover-triggered helper content. |
| `popover` | children | `toggled`, `opened`, `closed` | Click-triggered popup surface. |
| `dropdown-menu` | none | `command_invoked` | Button-triggered action menu. |
| `context-menu` | children | `command_invoked` | Right-click action menu surface. |
| `open-with` | none | `command_invoked` | Split current-editor picker. |
| `collab-cursor` | none | none | Presence cursor and name badge. |
| `icon-toolbar` | none | `selected` | Icon-first selection toolbar. |
| `file-tree` | none | `selected`, `opened`, `closed` | Compact file explorer tree. |
| `drag-board` | none | `changed` | Two-region drag-and-drop board. |
| `audio-playback` | children | `toggled` | Playback row with optional trailing actions. |
| `image-tile` | children | `clicked` | Media tile with optional body and playback state. |
| `command` | none | `changed` | Searchable command list. |

## Shared Types

| Type | Kind | Support | Summary |
| --- | --- | --- | --- |
| `node_common` | `object` | `partial` | Common fields flattened into every node. Visible, enabled, class, class_list, and part of layout are executed today. Slot and common action fields are metadata-only. |
| `actions` | `object` | `unsupported` | Optional common semantic action bindings. Declared in the schema only. The current renderer uses family-specific action fields instead. |
| `layout` | `object` | `partial` | Shared layout hints available on every node. Sizing, padding, and margin are executed on every node. Display, direction, gap, justify, align, wrap, grid tracks, and overflow are executed by the taffy-backed flow-container renderer used by row, column, inset, and card. |
| `layout_edges` | `object` | `partial` | Top, right, bottom, and left edge values for shared layout padding and margin. Executed for padding and margin with egui margin rounding and clamping. |
| `layout_length` | `object` | `partial` | Tagged length value used by shared layout sizing fields. Supported when referenced from width, height, min, max, and taffy-backed flex basis layout fields. |
| `layout_length_kind` | `enum` | `supported` | Supported layout length kinds. |
| `layout_track` | `object` | `partial` | Tagged grid track value for declared column and row tracks. Executed when the taffy-backed flow-container renderer used by row, column, inset, and card is switched to grid display. |
| `layout_track_kind` | `enum` | `supported` | Declared layout track kinds. |
| `layout_display` | `enum` | `partial` | Declared layout display modes. Executed by the taffy-backed flow-container renderer used by row, column, inset, and card. `overlay` remains unsupported. |
| `layout_direction` | `enum` | `partial` | Row and column direction values for shared layout hints. Only executed by the current flow-container helpers used by row, column, inset, and card. |
| `layout_overflow` | `enum` | `partial` | Declared overflow modes for shared layout hints. Executed by the taffy-backed flow-container renderer used by row, column, inset, and card for `visible`, `hidden`, and `scroll`. |
| `justify` | `enum` | `supported` | Main-axis alignment values. |
| `align` | `enum` | `supported` | Cross-axis alignment values. |
| `toolbar_anchor` | `enum` | `supported` | Supported toolbar anchor points. |
| `tabs_style` | `enum` | `supported` | Supported tab presentations. |
| `button_variant` | `enum` | `supported` | Supported button variants. |
| `control_size` | `enum` | `supported` | Supported control sizes. |
| `label_tone` | `enum` | `supported` | Supported text tones. |
| `label_weight` | `enum` | `supported` | Supported text weights. |
| `select_variant` | `enum` | `supported` | Supported select trigger variants. |
| `sidebar_side` | `enum` | `supported` | Supported sidebar docking sides. |
| `dialogue_intent` | `enum` | `supported` | Supported dialogue intents. |
| `toast_intent` | `enum` | `supported` | Supported toast intents. |
| `toast_placement` | `enum` | `supported` | Supported toast viewport placements. |
| `number_input_axis` | `enum` | `supported` | Supported drag axes for number input. |
| `hierarchy_item_kind` | `enum` | `supported` | Supported hierarchy semantic item kinds. |
| `menu_entry_kind` | `enum` | `supported` | Supported menu entry kinds. |
| `hierarchy_icon_style` | `enum` | `supported` | Supported hierarchy icon styles. |
| `hierarchy_style` | `enum` | `supported` | Supported hierarchy surface styles. |
| `action_item` | `object` | `supported` | Button-group style action item. |
| `choice_item` | `object` | `supported` | Select choice item. |
| `tab_item` | `object` | `supported` | Tab navigation item. |
| `menu_action` | `object` | `supported` | Clickable menu action row. |
| `menu_entry` | `object` | `supported` | Action-or-separator menu entry union. |
| `menu` | `object` | `supported` | Top-level menu bar menu. |
| `hierarchy_item` | `object` | `supported` | Recursive hierarchy tree item. |
| `toast_item` | `object` | `supported` | Toast lifecycle item. |

## Contract Mode Integration Pattern

1. Build a `ContractTree` from authoritative host state.
2. Render it with `render_tree` or `render_component_tree`.
3. Apply the returned `ContractEvent`s to host state.
4. Rebuild the next frame's tree from the updated authoritative host state.
