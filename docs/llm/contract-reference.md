# Contract Reference

Generated from `egui_component::contract::registry()` and `egui_component::contract::shared_types()`.

Export the schema and human-readable reference from Rust with:

```rust
egui_component::contract::schema_json_pretty()
egui_component::contract::reference_markdown()
```

## Position In The Runtime

`contract::*` is an optional host-driven declarative layer.

- The direct embedded Luau runtime path is the typed frame-local `app.*` / `ui.*` bridge shown by `runtime-egui-host`.
- `ContractTree` is useful when a host wants a serializable declarative surface, schema tooling, or change-driven host-authored trees.
- Renderer ownership stays in Rust. `contract::*` is not the default render boundary for the Luau runtime hot path.

## Shared Node Fields

| Field | Type | Support | Summary |
| --- | --- | --- | --- |
| `node_id` | `string` | `supported` | Stable host-owned node identifier. |
| `visible` | `boolean` | `supported` | Whether the node renders at all. Defaults to true. |
| `enabled` | `boolean` | `supported` | Whether interaction is enabled. Defaults to true. |
| `class` | `string` | `unsupported` | Optional primary class string. Declared in the model and schema only. The current Rust renderer ignores class strings. |
| `class_list` | `list<string>` | `unsupported` | Optional expanded class token list. Declared in the model and schema only. The current Rust renderer ignores class lists. |
| `slot_classes` | `map<string, string>` | `unsupported` | Optional slot-name to class-string overrides. Declared in the model and schema only. The current Rust renderer does not apply slot-specific class behavior. |
| `actions` | `object:actions` | `unsupported` | Optional common semantic action bindings. Declared in the model and schema only. The current renderer uses family-specific action fields instead. |
| `layout` | `object:layout` | `partial` | Optional shared layout hints. The renderer applies sizing on every node and container direction/justify/align/gap overrides on flow containers only. |

## Layout Support

| Field | Type | Support | Summary |
| --- | --- | --- | --- |
| `display` | `enum:layout_display` | `unsupported` | Optional layout mode override. Declared in the schema only. The current renderer does not execute display-mode switching. |
| `direction` | `enum:layout_direction` | `partial` | Optional row or column direction override. Only executed by the flow-container helpers used by row, column, inset, and card. |
| `grow` | `number` | `unsupported` | Optional flex grow factor. Declared in the schema only. Flex growth is not executed by the current renderer. |
| `shrink` | `number` | `unsupported` | Optional flex shrink factor. Declared in the schema only. Flex shrink is not executed by the current renderer. |
| `basis` | `object:layout_length` | `unsupported` | Optional flex basis length. Declared in the schema only. Flex basis is not executed by the current renderer. |
| `width` | `object:layout_length` | `supported` | Optional width override. |
| `height` | `object:layout_length` | `supported` | Optional height override. |
| `min_width` | `object:layout_length` | `supported` | Optional minimum width override. |
| `min_height` | `object:layout_length` | `supported` | Optional minimum height override. |
| `max_width` | `object:layout_length` | `supported` | Optional maximum width override. |
| `max_height` | `object:layout_length` | `supported` | Optional maximum height override. |
| `gap_x` | `number` | `partial` | Optional horizontal gap override. Only executed by the flow-container helpers used by row, column, inset, and card. |
| `gap_y` | `number` | `partial` | Optional vertical gap override. Only executed by the flow-container helpers used by row, column, inset, and card. |
| `padding` | `object:layout_edges` | `unsupported` | Optional padding edges. Declared in the schema only. Shared layout padding is not executed by the current renderer. |
| `margin` | `object:layout_edges` | `unsupported` | Optional margin edges. Declared in the schema only. Shared layout margins are not executed by the current renderer. |
| `align` | `enum:align` | `partial` | Optional cross-axis alignment override. Only executed by the flow-container helpers used by row, column, inset, and card. |
| `justify` | `enum:justify` | `partial` | Optional main-axis alignment override. Only executed by the flow-container helpers used by row, column, inset, and card. |
| `wrap` | `boolean` | `unsupported` | Optional wrap hint. Declared in the schema only. Wrapping is not executed by the current renderer. |
| `columns` | `list<object:layout_track>` | `unsupported` | Optional grid column tracks. Declared in the schema only. Grid tracks are not executed by the current renderer. |
| `rows` | `list<object:layout_track>` | `unsupported` | Optional grid row tracks. Declared in the schema only. Grid tracks are not executed by the current renderer. |
| `col_span` | `number` | `unsupported` | Optional grid column span. Declared in the schema only. Grid spans are not executed by the current renderer. |
| `row_span` | `number` | `unsupported` | Optional grid row span. Declared in the schema only. Grid spans are not executed by the current renderer. |
| `overflow_x` | `enum:layout_overflow` | `unsupported` | Optional horizontal overflow mode. Declared in the schema only. Overflow handling is not executed by the current renderer. |
| `overflow_y` | `enum:layout_overflow` | `unsupported` | Optional vertical overflow mode. Declared in the schema only. Overflow handling is not executed by the current renderer. |

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

## Shared Types

| Type | Kind | Support | Summary |
| --- | --- | --- | --- |
| `node_common` | `object` | `partial` | Common fields flattened into every node. Visible, enabled, and part of layout are executed today. Class, slot, and common action fields are metadata-only. |
| `actions` | `object` | `unsupported` | Optional common semantic action bindings. Declared in the schema only. The current renderer uses family-specific action fields instead. |
| `layout` | `object` | `partial` | Shared layout hints available on every node. Sizing is executed on every node. Direction, gap, justify, and align are only executed by the current flow-container helpers. |
| `layout_edges` | `object` | `unsupported` | Top, right, bottom, and left edge values for shared layout padding and margin. Edge-based shared padding and margin are not executed by the current renderer. |
| `layout_length` | `object` | `partial` | Tagged length value used by shared layout sizing fields. Supported when referenced from width, height, min, and max layout fields. Other consumers such as basis remain unsupported. |
| `layout_length_kind` | `enum` | `supported` | Supported layout length kinds. |
| `layout_track` | `object` | `unsupported` | Tagged grid track value for declared column and row tracks. Grid tracks are declared in the schema only and are not executed by the current renderer. |
| `layout_track_kind` | `enum` | `supported` | Declared layout track kinds. |
| `layout_display` | `enum` | `unsupported` | Declared layout display modes. Display-mode switching is declared in the schema only and is not executed by the current renderer. |
| `layout_direction` | `enum` | `partial` | Row and column direction values for shared layout hints. Only executed by the current flow-container helpers used by row, column, inset, and card. |
| `layout_overflow` | `enum` | `unsupported` | Declared overflow modes for shared layout hints. Overflow handling is declared in the schema only and is not executed by the current renderer. |
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
