# Components Reference

This file describes the existing component surface exactly as it is implemented now.

Conventions used below:

- "Method" means the `ComponentUi` entry point.
- "State" means caller-owned mutable state.
- "Shorthands" lists the current `From<_>` forms accepted by `impl Into<...>`.
- "Defaults" are the builder defaults in source, not suggested redesigns.

## Wrapper And Theme

### `ComponentUi` and `ComponentUiExt`

- Method entry: `let mut ui = ui.components();`
- Source: `src/components/api.rs`
- Purpose: wraps `egui::Ui` and keeps scoped component overrides alive across nested layouts and scopes

Useful methods:

- `raw()` / `raw_mut()` for direct `egui::Ui` access
- `with_override(...)` for scoped type-based overrides
- `scope(...)`
- `scope_builder(...)`
- `push_id(...)`
- `horizontal(...)`
- `vertical(...)`
- `with_layout(...)`
- `allocate_ui_with_layout(...)`

Supported override types:

- `ButtonOverride`
- `CardOverride`
- `LabelOverride`
- `TextInputOverride`

`with_override` accepts tuples up to four items and merges by widget type.

### Theme Helpers

- `theme::setup(&Context)` installs style and icon loading
- `theme::apply_component_theme(ui)` applies component spacing/selection/radius rules inside one region

## Label

- Method: `ui.label(props) -> Response`
- Builder: `Label<'a>`
- State: none
- Shorthands:
  - `&str`
  - `(&str, LabelTone)`
  - `(&str, LabelTone, f32)`
  - `(&str, LabelTone, LabelWeight)`
- Defaults:
  - tone: `Primary`
  - weight: `Regular`
  - size: `12.0`
  - color override: `None`
- Notes:
  - tones are `Primary`, `Secondary`, `Muted`, `Destructive`
  - semibold labels use the named semibold font family from the theme

```rust
let _ = ui.label("Material");
let _ = ui.label(("Assigned mesh", LabelTone::Secondary));
let _ = ui.label(
    Label::new("Danger zone")
        .tone(LabelTone::Destructive)
        .weight(LabelWeight::Semibold),
);
```

## Kbd

- Method: `ui.kbd(props) -> Response`
- Builder: `Kbd<'a>`
- State: none
- Shorthands:
  - `&str`
  - `(&str, f32)`
- Defaults:
  - min width: `20.0`
  - height: `20.0`
  - padding: `4.0 x 2.0`
  - corner radius: `RADIUS_SM`
  - fill: `INPUT_BACKGROUND`
  - stroke: `INPUT_BORDER`
  - text color: `TEXT_MUTED`
  - text size: `10.0`
- Notes:
  - intended for small keycaps
  - supports symbol glyphs like `⌘`, `⇧`, `⌥`, `⌃`

### `KbdGroup`

- Method: `ui.kbd_group(props, |ui| ...) -> InnerResponse<R>`
- Builder: `KbdGroup`
- Shorthands:
  - `()`
  - `f32`
- Default gap: `4.0`

```rust
let _ = ui.kbd_group((), |ui| {
    let _ = ui.kbd("⌘");
    let _ = ui.kbd("K");
});
```

## Icon

- Method: `ui.icon(props) -> Response`
- Builder: `Icon<'a>`
- State: none
- Shorthands:
  - `&str`
  - `(&str, f32)`
  - `(&str, f32, Color32)`
- Defaults:
  - size: `14.0`
  - tint: `TEXT_SECONDARY`
- Notes:
  - names are normalized through `src/ui/icons.rs`
  - accepts common variants like `ChevronDown`, `chevron_down`, or `chevron-down.svg`
  - invalid or missing icons render as empty allocated space

```rust
let _ = ui.icon("search");
let _ = ui.icon(("chevron-down", 12.0));
```

## Text Input

- Method: `ui.text_input(&mut value, props) -> Response`
- Builder: `TextInput<'a>`
- State: `&mut String`
- Shorthands:
  - `()`
  - `&str`
  - `f32`
  - `(f32, &str)`
- Defaults:
  - width: `220.0`
  - hint text: `None`
- Notes:
  - uses `with_input_chrome`
  - input padding comes from `INPUT_PADDING_X/Y`
  - hover/focus colors come from `input_bg` and `input_stroke`

### `TextInputOverride`

- Scoped fields:
  - width

```rust
let _ = ui.text_input(&mut name, "Component name");
let _ = ui.text_input(&mut name, (280.0, "Name"));
```

## Field

- Method: `ui.field(&mut value, props) -> Response`
- Builder: `Field<'a>`
- State: `&mut String`
- Shorthands:
  - `&str`
  - `(&str, &str)`
  - `(&str, f32)`
  - `(&str, f32, &str)`
- Defaults:
  - helper text: `None`
  - width: `220.0`
  - placeholder: `None`
- Notes:
  - composed from `label` + `text_input` + optional helper text
  - label uses secondary semibold styling
  - helper text uses muted `11.0` text

```rust
let _ = ui.field(&mut material, ("Material", 280.0, "Assigned asset"));
```

## Button

- Method: `ui.button(props) -> Response`
- Builder: `Button<'a>`
- State: none
- Styles:
  - `Primary`
  - `Secondary`
  - `Ghost`
  - `Link`
- Shorthands:
  - `&str`
  - `(&str, ButtonStyle)`
  - `(&str, &str)`
  - `(&str, &str, ButtonStyle)`
  - `(&str, ButtonStyle, &str)`
- Defaults:
  - style: `Primary`
  - icon: `None`
  - icon size: `14.0`
  - icon tint: `None`
  - icon only: `false`
  - right text: `None`
  - selected: `false`
  - min size: `None`
- Notes:
  - primary buttons invert foreground/background through `primary_*` tokens
  - icon-only buttons become square using `interact_size.y`
  - `right_text_weak` maps to `shortcut_text`
  - text-only hovered link buttons draw an underline manually

### `ButtonOverride`

- Scoped fields:
  - style
  - icon size
  - icon tint
  - icon only
  - selected
  - min size

```rust
let _ = ui.button("Save");
let _ = ui.button(("Cancel", ButtonStyle::Secondary));
let _ = ui.button(Button::new("Search").icon("search"));
let _ = ui.button(Button::icon_only("ellipsis").style(ButtonStyle::Ghost));
```

## Button Group

- Method: `ui.button_group(&mut selected_index, props)`
- Builder: `ButtonGroup<'a>`
- State: `&mut usize`
- Shorthands:
  - `(Id, &[&str])`
- Notes:
  - segmented control
  - clamps `selected_index` into range
  - if options are empty, it resets the index to `0`
  - selection uses row-selected tokens

```rust
let options = ["Move", "Rotate", "Scale"];
ui.button_group(&mut selected, (Id::new("tool-mode"), &options[..]));
```

## Checkbox

- Method: `ui.checkbox(&mut value, props) -> Response`
- Builder: `Checkbox<'a>`
- State: `&mut bool`
- Shorthands:
  - `()`
  - `&str`
- Defaults:
  - label: `None`
- Notes:
  - text label is clickable and toggles the control
  - checked state uses the primary action token set

```rust
let _ = ui.checkbox(&mut enabled, "Receive shadows");
```

## Switch

- Method: `ui.switch(&mut value, props) -> Response`
- Builder: `Switch<'a>`
- State: `&mut bool`
- Sizes:
  - `Default`
  - `Small`
- Shorthands:
  - `()`
  - `&str`
  - `SwitchSize`
  - `(&str, SwitchSize)`
- Defaults:
  - label: `None`
  - size: `Default`
- Notes:
  - uses animated knob movement through `animate_bool`
  - focus uses input focus stroke
  - label, when present, is placed to the left

```rust
let _ = ui.switch(&mut visible, "Visible");
let _ = ui.switch(&mut compact, SwitchSize::Small);
```

## Slider

- Method: `ui.slider(&mut value, props) -> Response`
- Builder: `Slider`
- State: `&mut f32`
- Shorthands:
  - `RangeInclusive<f32>`
  - `(RangeInclusive<f32>, f32)`
- Defaults:
  - width: `156.0`
- Notes:
  - uses `with_slider_chrome`
  - suppresses the built-in numeric value display

```rust
let _ = ui.slider(&mut opacity, (0.0..=100.0, 180.0));
```

## Number Input

- Method: `ui.number_input(&mut value, props) -> Response`
- Builder: `NumberInput`
- State: `&mut f32`
- Shorthands:
  - `Id`
  - `(Id, RangeInclusive<f32>)`
  - `(Id, f32)`
- Defaults:
  - width: `58.0`
  - range: `0.0..=100.0`
  - speed: `0.2`
  - decimals: `1`
  - prefix: `None`
  - prefix tint: `TEXT_SECONDARY`
  - prefix align left: `false`
  - axis: `Horizontal`
- Notes:
  - built on `egui::DragValue`
  - uses input chrome
  - axis controls the resize cursor icon
  - left-aligned prefixes are painted manually

```rust
let _ = ui.number_input(
    &mut x,
    NumberInput::new(Id::new("x"))
        .range(-100.0..=100.0)
        .prefix("X")
        .prefix_align_left(),
);
```

## Select

- Method: `ui.select(&mut selected_index, props) -> Response`
- Builder: `Select<'a>`
- State: `&mut Option<usize>`
- Shorthands:
  - `(Id, Id, &[&str])`
  - `(Id, &[&str])`
  - `(Id, &[&str], f32)`
- Defaults:
  - width: `220.0`
  - placeholder: `"Select an option"`
- Notes:
  - one `Id` can generate trigger and popup ids through `Select::from_id`
  - out-of-range selection resets to `None`
  - menu rows use selected-row tokens

```rust
let _ = ui.select(
    &mut selected,
    (Id::new("status"), &["Draft", "Review", "Approved"][..], 220.0),
);
```

## Tabs

- Method: `ui.tabs(id, &mut current, options)`
- Builder type: none, uses `TabOption<'a>`
- State: `&mut usize`
- Notes:
  - `TabOption` is `{ value, label }`
  - selected tabs render a bottom underline
  - tab buttons are text-first with transparent chrome

```rust
let options = [
    TabOption::new(0, "Design"),
    TabOption::new(1, "Code"),
];
ui.tabs(Id::new("editor-tabs"), &mut current_tab, &options);
```

## Separator

- Method: `ui.separator() -> Response`
- Builder: none
- State: none
- Notes:
  - draws a full-width `1.0` horizontal line using the `SEPARATOR` token

## Card

- Method: `ui.card(props, |ui| ...) -> InnerResponse<R>`
- Builder: `Card`
- State: none
- Shorthands:
  - `()`
  - `Color32`
  - `(Color32, Stroke)`
- Defaults:
  - fill: `MUTED_SURFACE`
  - stroke: `Stroke::new(1.0, SEPARATOR)`
  - corner radius: `RADIUS_LG`
  - padding: `12 x 12`
- Notes:
  - implemented through `egui::Frame`
  - propagates current component overrides into nested content

### `CardOverride`

- Scoped fields:
  - fill
  - stroke
  - corner radius
  - padding x/y

```rust
let _ = ui.card((), |ui| {
    let _ = ui.label("Inspector");
});
```

## Progress

- Method: `ui.progress(value, props) -> Response`
- Builder: `Progress`
- State: value passed by copy
- Shorthands:
  - `()`
  - `f32`
  - `(f32, f32)`
- Defaults:
  - width: `188.0`
  - height: `10.0`
- Notes:
  - value is clamped into `0.0..=1.0`
  - fill uses the primary action background
  - text is intentionally hidden

```rust
let _ = ui.progress(0.58, ());
```

## Tooltip

- Method: `ui.tooltip(props) -> Response`
- Builder: `Tooltip<'a>`
- State: none from the caller
- Shorthands:
  - `(&str, &str)`
  - `(&str, &str, f32)`
  - `(&str, &str, f32, bool)`
- Defaults:
  - width: `220.0`
  - delay: `0 ms`
  - top center: `true`
- Notes:
  - renders a secondary button as the trigger
  - width controls trigger min width
  - hover delay is implemented with temporary `Ui` data
  - tooltip content is currently a single secondary label

```rust
let _ = ui.tooltip(("Hover for help", "Tooltip body", 220.0, true));
```

## Collapsible

- Method: `ui.collapsible(&mut open, props, |ui| ...) -> Response`
- Builder: `Collapsible<'a>`
- State: `&mut bool`
- Shorthands:
  - `(Id, &str)`
  - `(Id, &str, bool)`
  - `(Id, &str, bool, &str, &str)`
- Defaults:
  - open: `true`
  - leading icon: `None`
  - trailing icon: `None`
  - leading icon tint: `TEXT_SECONDARY`
  - trailing icon tint: `TEXT_MUTED`
- Notes:
  - clicking the header toggles open state
  - if `props.open` differs from caller state, caller state is overwritten to match the props value
  - header uses chevron icons plus optional leading/trailing icons

```rust
let _ = ui.collapsible(
    &mut open,
    (Id::new("materials"), "Materials", true, "package", "plus"),
    |ui| {
        let _ = ui.label(("Body", LabelTone::Secondary));
    },
);
```

## Dropdown Menu

- Method: `ui.dropdown_menu(props) -> (Response, DropdownMenuState)`
- Builder: `DropdownMenu<'a>`
- State: none from the caller, action result returned in `DropdownMenuState`
- Shorthands:
  - `&str`
  - `(&str, &[&str])`
  - `(&str, &[DropdownMenuEntry])`
  - `(&str, &[&str], f32)`
  - `(&str, &[DropdownMenuEntry], f32)`
- Defaults:
  - entries: `[]`
  - options: `[]`
  - width: `220.0`
  - trigger style: `Secondary`
- Notes:
  - if both `options` and `entries` are empty, it behaves like a plain button and returns `action: None`
  - simple string options are converted to action ids by index
  - nested menus use `DropdownMenuEntry::Submenu`
  - shortcuts render through `Kbd` keycaps

Supporting types:

- `DropdownMenuAction { id, label, shortcut }`
- `DropdownMenuEntry::Action`
- `DropdownMenuEntry::Separator`
- `DropdownMenuEntry::Submenu`
- `DropdownMenuState { action: Option<usize> }`

```rust
let entries = [
    DropdownMenuEntry::action_with_shortcut(1, "Profile", "Shift+Cmd+P"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action(2, "Log out"),
];
let (_response, state) = ui.dropdown_menu(("Open menu", &entries[..], 220.0));
```

## Combobox

- Method: `ui.combobox(&mut query, &mut selected_index, props) -> Response`
- Builder: `Combobox<'a>`
- State:
  - `&mut String` query
  - `&mut usize` selected index
- Shorthands:
  - `(Id, &[&str])`
  - `(Id, &[&str], f32)`
- Defaults:
  - width: `220.0`
  - max height: `104.0`
  - placeholder: `"Filter"`
- Notes:
  - composed from `text_input` and a `card` containing filterable rows
  - filtering is case-insensitive substring matching
  - empty options clamp selection back to `0`
  - "No matches" is rendered as muted text

```rust
let _ = ui.combobox(
    &mut query,
    &mut selected_index,
    (Id::new("materials"), &options[..], 220.0),
);
```

## Command

- Method: `ui.command(&mut query, items, props) -> Response`
- Builder: `Command<'a>`
- State:
  - `&mut String` query
- Item type: `CommandItem<'a> { group, label }`
- Shorthands:
  - `Id`
  - `(Id, f32)`
  - `(Id, f32, f32)`
- Defaults:
  - width: `220.0`
  - max height: `132.0`
  - placeholder: `"Type a command"`
- Notes:
  - composed from `text_input` and a `card`
  - filtering matches against `group + label`
  - current implementation displays matching rows but does not expose selection state or click actions
  - "No commands" is rendered as muted text

```rust
let items = [
    CommandItem::new("Scene", "Open Scene Search"),
    CommandItem::new("View", "Toggle Grid"),
];
let _ = ui.command(&mut query, &items, (Id::new("command"), 240.0, 132.0));
```

## Dialogue

- High-level method: `ui.dialogue(&mut open, props) -> Response`
- Lower-level methods:
  - `ui.dialogue_modal(&mut open, props, |ui, close_requested| ...)`
  - `ui.dialogue_header(props)`
  - `ui.dialogue_title(title, style)`
  - `ui.dialogue_description(description)`
- Builders:
  - `Dialogue<'a>`
  - `DialogueModal`
  - `DialogueHeader<'a>`
- State:
  - `&mut bool` open flag

### `DialogueStyle`

- `Default`
- `Alert`

`Alert` is the existing "alert dialogue" implementation. The title becomes destructive-toned, but the rest of the structure stays the same.

### `Dialogue`

- Shorthands:
  - `(Id, &str)`
  - `(Id, &str, &str)`
  - `(Id, &str, f32)`
  - `(Id, &str, &str, DialogueStyle)`
- Defaults:
  - description: `""`
  - trigger label: `"Open"`
  - cancel label: `"Cancel"`
  - confirm label: `"Confirm"`
  - width: `360.0`
  - style: `Default`
- Notes:
  - `ui.dialogue(...)` renders the trigger button and modal together
  - clicking trigger sets `open = true`
  - cancel and confirm both currently request close

### `DialogueModal`

- Shorthands:
  - `Id`
  - `(Id, f32)`
- Default width: `360.0`

### `DialogueHeader`

- Shorthands:
  - `&str`
  - `(&str, &str)`
  - `(&str, &str, DialogueStyle)`
- Defaults:
  - description: `""`
  - style: `Default`

```rust
let _ = ui.dialogue(
    &mut open,
    Dialogue::new(Id::new("delete-dialogue"), "Delete file")
        .description("This action cannot be undone.")
        .style(DialogueStyle::Alert)
        .trigger_label("Delete"),
);
```
