# Luau Runtime API Reference

Generated from `luau_runtime_core::runtime_api_reference_markdown()` and `luau_runtime_core::runtime_api_luau_typings()`.

Refresh with:

```bash
cargo run -p luau-runtime-core --bin generate-runtime-api
```

Generated artifacts:

- [`examples/runtime-luau/ui/types.luau`](../examples/runtime-luau/ui/types.luau)
- [`docs/luau-runtime-api-reference.md`](./luau-runtime-api-reference.md)

The host mounts frame-local `app` and `ui` globals for Luau scripts. This document covers that generated bridge surface only; lifecycle hooks such as `init`, `update`, `render`, `reload`, and `shutdown` remain documented in `examples/runtime-luau/README.md`.

## Functions

### `app`

#### `app.log(level: string, message: string) -> ()`

Emit a host diagnostic log message immediately.

- Phase rule: Available during `load`, `reload`, `update`, and `render`.
- Capability: `log`

#### `app.request_reload() -> ()`

Queue a script reload after the current successful frame.

- Phase rule: Available during frame phases only: `update` and `render`.
- Capability: `reload`

#### `app.request_repaint() -> ()`

Queue a host repaint after the current successful frame.

- Phase rule: Available during frame phases only: `update` and `render`.
- Capability: `repaint`

### `ui`

#### `ui.label(text: string, props: LabelProps?) -> ()`

Render non-interactive text in the active container.

- Phase rule: Available during `render` only.

#### `ui.separator() -> ()`

Render a visual separator in the active container.

- Phase rule: Available during `render` only.

#### `ui.button(id: string, text: string, props: ButtonProps?) -> boolean`

Render a clickable button with an explicit local id.

- Phase rule: Available during `render` only.
- Returns: `clicked: boolean`

#### `ui.text_edit(id: string, value: string, props: TextEditProps?) -> (string, boolean)`

Render a single-line text input with an explicit local id.

- Phase rule: Available during `render` only.
- Returns: `value: string`, `changed: boolean`

#### `ui.checkbox(id: string, checked: boolean, props: CheckboxProps?) -> (boolean, boolean)`

Render a checkbox control with an explicit local id.

- Phase rule: Available during `render` only.
- Returns: `value: boolean`, `changed: boolean`

#### `ui.switch(id: string, checked: boolean, props: SwitchProps?) -> (boolean, boolean)`

Render a switch control with an explicit local id.

- Phase rule: Available during `render` only.
- Returns: `value: boolean`, `changed: boolean`

#### `ui.slider(id: string, value: number, props: SliderProps) -> (number, boolean)`

Render a slider control with an explicit local id.

- Phase rule: Available during `render` only.
- Returns: `value: number`, `changed: boolean`

#### `ui.number_input(id: string, value: number, props: NumberInputProps?) -> (number, boolean)`

Render a numeric input control with an explicit local id.

- Phase rule: Available during `render` only.
- Returns: `value: number`, `changed: boolean`

#### `ui.select(id: string, selected_index: number, options: {SelectOption}, props: SelectProps?) -> (number, boolean)`

Render a select trigger and return the next 1-based selection index (`0` means no selection).

- Phase rule: Available during `render` only.
- Returns: `selected_index: number`, `changed: boolean`

#### `ui.tabs(id: string, selected_index: number, options: {TabOption}, props: TabsProps?) -> (number, boolean)`

Render tabs and return the next 1-based selection index (`0` means no selection).

- Phase rule: Available during `render` only.
- Returns: `selected_index: number`, `changed: boolean`

#### `ui.progress(value: number, props: ProgressProps?) -> ()`

Render a determinate progress indicator.

- Phase rule: Available during `render` only.

#### `ui.radio(id: string, selected: boolean, props: RadioProps?) -> (boolean, boolean)`

Render a radio control with an explicit local id.

- Phase rule: Available during `render` only.
- Returns: `value: boolean`, `changed: boolean`

#### `ui.button_group(id: string, options: {ButtonGroupOption}, props: ButtonGroupProps?) -> (number, boolean)`

Render an attached button group and return the 1-based clicked index (`0` means no click).

- Phase rule: Available during `render` only.
- Returns: `clicked_index: number`, `changed: boolean`

#### `ui.begin_collapsible(id: string, title: string, open: boolean, props: CollapsibleProps?) -> (boolean, boolean)`

Render a collapsible header and optionally open a body scope. Returns the next open state and whether the body scope is visible.

- Phase rule: Available during `render` only.
- Returns: `open: boolean`, `visible: boolean`

#### `ui.dropdown_menu(id: string, trigger_label: string, entries: {DropdownMenuEntry}, props: DropdownMenuProps?) -> (number, boolean)`

Render a dropdown-menu trigger and return the selected action id (`0` means no action).

- Phase rule: Available during `render` only.
- Returns: `action_id: number`, `changed: boolean`

#### `ui.tooltip(trigger_label: string, text: string, props: TooltipProps?) -> ()`

Render a secondary trigger button that reveals tooltip text on hover.

- Phase rule: Available during `render` only.

#### `ui.spinner(props: SpinnerProps?) -> ()`

Render an indeterminate loading spinner.

- Phase rule: Available during `render` only.

#### `ui.skeleton(props: SkeletonProps?) -> ()`

Render an animated skeleton placeholder.

- Phase rule: Available during `render` only.

#### `ui.virtual_list(id: string, items: {string}, props: VirtualListProps?) -> (number, boolean)`

Render a native scroll-virtualized string list and return the next 1-based selection index (`0` means no selection).

- Phase rule: Available during `render` only.
- Returns: `selected_index: number`, `changed: boolean`

#### `ui.begin_row(props: ContainerProps?) -> ()`

Open a horizontal layout scope.

- Phase rule: Available during `render` only.

#### `ui.begin_column(props: ContainerProps?) -> ()`

Open a vertical layout scope.

- Phase rule: Available during `render` only.

#### `ui.begin_card(props: CardProps?) -> ()`

Open a framed card scope.

- Phase rule: Available during `render` only.

#### `ui.end_scope() -> ()`

Close the most recently opened row, column, or card scope.

- Phase rule: Available during `render` only.
- Compatibility aliases accepted at runtime: `end`

#### `ui.push_id(id: string) -> ()`

Push an explicit identity scope for stateful widgets.

- Phase rule: Available during `render` only.

#### `ui.pop_id() -> ()`

Pop the most recently pushed identity scope.

- Phase rule: Available during `render` only.

## Prop Schemas

### `LabelProps`

| Field | Type | Default |
| --- | --- | --- |
| `tone` | `LabelTone?` | `primary` |
| `weight` | `LabelWeight?` | `regular` |
| `size` | `number?` | `none` |

### `ButtonProps`

| Field | Type | Default |
| --- | --- | --- |
| `variant` | `ButtonVariant?` | `primary` |
| `size` | `ControlSize?` | `md` |
| `width` | `number?` | `none` |
| `leading_icon` | `string?` | `none` |
| `trailing_icon` | `string?` | `none` |
| `icon_size` | `number?` | `none` |
| `icon_only` | `boolean?` | `false` |
| `selected` | `boolean?` | `false` |

### `TextEditProps`

| Field | Type | Default |
| --- | --- | --- |
| `width` | `number?` | `none` |
| `placeholder` | `string?` | `none` |
| `leading_icon` | `string?` | `none` |
| `password` | `boolean?` | `false` |

### `CheckboxProps`

| Field | Type | Default |
| --- | --- | --- |
| `label` | `string?` | `none` |

### `SwitchProps`

| Field | Type | Default |
| --- | --- | --- |
| `label` | `string?` | `none` |
| `size` | `ControlSize?` | `md` |

### `SliderProps`

| Field | Type | Default |
| --- | --- | --- |
| `min` | `number` | `none` |
| `max` | `number` | `none` |
| `width` | `number?` | `156` |

### `NumberInputProps`

| Field | Type | Default |
| --- | --- | --- |
| `min` | `number?` | `none` |
| `max` | `number?` | `none` |
| `width` | `number?` | `58` |
| `speed` | `number?` | `none` |
| `fine_speed` | `number?` | `none` |
| `decimals` | `number?` | `none` |
| `fine_decimals` | `number?` | `none` |
| `prefix` | `string?` | `none` |
| `suffix` | `string?` | `none` |
| `prefix_tint` | `string?` | `none` |
| `prefix_align_left` | `boolean?` | `false` |
| `axis` | `NumberInputAxis?` | `horizontal` |

### `SelectProps`

| Field | Type | Default |
| --- | --- | --- |
| `width` | `number?` | `220` |
| `placeholder` | `string?` | `Select an option` |
| `variant` | `SelectVariant?` | `default` |

### `TabsProps`

| Field | Type | Default |
| --- | --- | --- |
| `variant` | `TabsVariant?` | `underline` |

### `ProgressProps`

| Field | Type | Default |
| --- | --- | --- |
| `width` | `number?` | `188` |
| `height` | `number?` | `10` |

### `RadioProps`

| Field | Type | Default |
| --- | --- | --- |
| `label` | `string?` | `none` |
| `description` | `string?` | `none` |

### `ButtonGroupProps`

| Field | Type | Default |
| --- | --- | --- |

### `CollapsibleProps`

| Field | Type | Default |
| --- | --- | --- |
| `open` | `boolean?` | `none` |
| `leading_icon` | `string?` | `none` |
| `trailing_icon` | `string?` | `none` |

### `DropdownMenuProps`

| Field | Type | Default |
| --- | --- | --- |
| `width` | `number?` | `220` |
| `trigger_variant` | `ButtonVariant?` | `secondary` |

### `TooltipProps`

| Field | Type | Default |
| --- | --- | --- |
| `width` | `number?` | `220` |
| `delay_ms` | `number?` | `0` |
| `placement` | `TooltipPlacement?` | `auto` |

### `SpinnerProps`

| Field | Type | Default |
| --- | --- | --- |
| `size` | `number?` | `16` |
| `stroke_width` | `number?` | `none` |
| `speed` | `number?` | `none` |
| `color` | `string?` | `none` |

### `SkeletonProps`

| Field | Type | Default |
| --- | --- | --- |
| `width` | `number?` | `120` |
| `height` | `number?` | `16` |
| `shape` | `SkeletonShape?` | `rect` |

### `VirtualListProps`

| Field | Type | Default |
| --- | --- | --- |
| `width` | `number?` | `none` |
| `height` | `number?` | `220` |
| `row_height` | `number?` | `24` |
| `selected_index` | `number?` | `0` |

### `ContainerProps`

| Field | Type | Default |
| --- | --- | --- |
| `gap` | `number?` | `none` |

### `CardProps`

| Field | Type | Default |
| --- | --- | --- |
| `width` | `number?` | `none` |
| `padding_x` | `number?` | `12` |
| `padding_y` | `number?` | `12` |

## Structured Types

### `SelectOption`

| Field | Type | Default |
| --- | --- | --- |
| `label` | `string` | `none` |

### `TabOption`

| Field | Type | Default |
| --- | --- | --- |
| `label` | `string` | `none` |
| `icon` | `string?` | `none` |
| `icon_only` | `boolean?` | `false` |
| `tooltip` | `string?` | `none` |

### `ButtonGroupOption`

| Field | Type | Default |
| --- | --- | --- |
| `label` | `string` | `none` |

### `DropdownMenuEntry`

`{ kind = "action", id: number, label: string, icon?: string, shortcut?: string, enabled?: boolean, selected?: boolean }`

`{ kind = "separator" }`

`{ kind = "submenu", label: string, icon?: string, entries: {DropdownMenuEntry} }`

## Enum Tokens

### `type LabelTone = "primary" | "secondary" | "muted" | "destructive"`

Generated typings use canonical values only. The runtime parser is case-insensitive.

### `type LabelWeight = "regular" | "semibold" | "bold"`

Generated typings use canonical values only. The runtime parser is case-insensitive.

### `type ButtonVariant = "primary" | "secondary" | "ghost" | "link"`

Generated typings use canonical values only. The runtime parser is case-insensitive.

### `type ControlSize = "sm" | "md"`

Generated typings use canonical values only. The runtime parser is case-insensitive and also accepts: `small` -> `sm`, `medium` -> `md`.

### `type NumberInputAxis = "horizontal" | "vertical"`

Generated typings use canonical values only. The runtime parser is case-insensitive.

### `type SelectVariant = "default" | "secondary"`

Generated typings use canonical values only. The runtime parser is case-insensitive.

### `type TabsVariant = "underline" | "segmented" | "stacked" | "rail" | "blender_topbar"`

Generated typings use canonical values only. The runtime parser is case-insensitive.

### `type TooltipPlacement = "auto" | "top" | "right" | "bottom" | "left"`

Generated typings use canonical values only. The runtime parser is case-insensitive.

### `type SkeletonShape = "rect" | "circle"`

Generated typings use canonical values only. The runtime parser is case-insensitive.

