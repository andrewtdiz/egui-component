# Contract Reference

Generated from `egui_component::contract::registry()` and `egui_component::contract::shared_types()`.

Export the schema and human-readable reference from Rust with:

```rust
egui_component::contract::schema_json_pretty()
egui_component::contract::reference_markdown()
```

## Shared Node Fields

Every contract node includes:

- `node_id`
- `visible`
- `enabled`

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

- `node_common` (`object`): Common fields flattened into every node.
- `justify` (`enum`): Main-axis alignment values.
- `align` (`enum`): Cross-axis alignment values.
- `toolbar_anchor` (`enum`): Supported toolbar anchor points.
- `tabs_style` (`enum`): Supported tab presentations.
- `button_variant` (`enum`): Supported button variants.
- `control_size` (`enum`): Supported control sizes.
- `label_tone` (`enum`): Supported text tones.
- `label_weight` (`enum`): Supported text weights.
- `select_variant` (`enum`): Supported select trigger variants.
- `sidebar_side` (`enum`): Supported sidebar docking sides.
- `dialogue_intent` (`enum`): Supported dialogue intents.
- `toast_intent` (`enum`): Supported toast intents.
- `toast_placement` (`enum`): Supported toast viewport placements.
- `number_input_axis` (`enum`): Supported drag axes for number input.
- `hierarchy_item_kind` (`enum`): Supported hierarchy semantic item kinds.
- `menu_entry_kind` (`enum`): Supported menu entry kinds.
- `hierarchy_icon_style` (`enum`): Supported hierarchy icon styles.
- `hierarchy_style` (`enum`): Supported hierarchy surface styles.
- `action_item` (`object`): Button-group style action item.
- `choice_item` (`object`): Select choice item.
- `tab_item` (`object`): Tab navigation item.
- `menu_action` (`object`): Clickable menu action row.
- `menu_entry` (`object`): Action-or-separator menu entry union.
- `menu` (`object`): Top-level menu bar menu.
- `hierarchy_item` (`object`): Recursive hierarchy tree item.
- `toast_item` (`object`): Toast lifecycle item.

## Host Integration Pattern

1. Build a `ContractTree` from host state.
2. Render it with `render_tree` or `render_component_tree`.
3. Apply the returned `ContractEvent`s to host state.
4. Rebuild the next frame's tree from the updated authoritative host state.
