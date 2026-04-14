export type MigrationStrategy =
  | "contract-family"
  | "compose"
  | "family-alias"
  | "support-types";

export type MigrationStatus = "implemented" | "pending";

export type RustApiStatus =
  | "runtime-primitive"
  | "runtime-behavior"
  | "tsx-owned";

export type MigrationEntry = {
  id: string;
  label: string;
  source: string;
  target: string;
  strategy: MigrationStrategy;
  status: MigrationStatus;
  contractFamilies?: string[];
  rustApiStatus?: RustApiStatus;
  notes?: string;
};

const rootComponentIds = new Set([
  "audio-playback",
  "button",
  "button-group",
  "card",
  "checkbox",
  "collab-cursor",
  "collapsible",
  "color-input",
  "combobox",
  "command",
  "context-menu",
  "dialogue",
  "drag-board",
  "dropdown-menu",
  "emoji-selector",
  "field",
  "file-tree",
  "hierarchy",
  "icon-toolbar",
  "image-tile",
  "input",
  "menu-bar",
  "open-with",
  "pagination",
  "palette-color-input",
  "popover",
  "radio",
  "select",
  "sidebar",
  "slider",
  "switch",
  "tabs",
  "toast",
  "toggle-group",
  "toolbar",
  "tooltip",
]);

const primaryComponentIds = new Set([
  "color",
  "common",
  "icon",
  "image",
  "kbd",
  "label",
  "progress",
  "separator",
  "skeleton",
  "spinner",
  "twemoji",
]);

export const migrationManifest: MigrationEntry[] = [
  support("api", "Component API support", "api", "examples/runtime-jsx/ui/component-support.ts", "Shared JSX NodeProps, events, className/classList, and slot class helpers mirror the Rust component API support layer."),
  behavior("audio-playback", "Audio Playback", "audio_playback", ["audio-playback"], "Retains internal Rust runtime behavior for waveform/progress painting, playback state, and trailing action children."),
  entry("button", "Button", "button", ["button"]),
  tsxOwned("button-group", "Button Group", "button_group", ["button-group"], "TSX owns the button group recipe composed from row and button primitives. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  tsxOwned("canva", "Canva helpers", "canva", [], "TSX now owns the Canva helper recipes composed from card, row, column, button, color, tabs, input, and number-input. The Rust component module was deleted from src/components."),
  tsxOwned("card", "Card", "card", ["card"], "TSX owns the card surface primitive. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  entry("checkbox", "Checkbox", "checkbox", ["checkbox"]),
  behavior("collab-cursor", "Collab Cursor", "collab_cursor", ["collab-cursor"], "Retains internal Rust runtime behavior for painter-backed presence cursor positioning."),
  tsxOwned("collapsible", "Collapsible", "collapsible", ["collapsible"], "TSX owns the collapsible recipe composed from card, button, label, and layout primitives. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  tsxOwned("color", "Color", "color", ["color"], "TSX owns the color swatch primitive. The Rust component module was deleted from src/components; the contract renderer now uses a lower-level swatch primitive."),
  primitive("color-input", "Color Input", "color_input", ["color-input"], "TSX owns the composed color input recipe. The Rust ColorInput API remains an active internal egui color editor runtime control."),
  primitive("color-strip", "Color Strip", "color_strip", ["color-strip"], "TSX owns the palette-row recipe composed from repeated color primitives. The Rust ColorStrip API remains an active continuous strip runtime control."),
  tsxOwned("combobox", "Combobox", "combobox", ["combobox"], "TSX owns the combobox recipe. The Rust component module was deleted from src/components; the contract renderer keeps a small compatibility renderer."),
  tsxOwned("command", "Command", "command", ["command"], "TSX owns the command palette recipe. The Rust component module was deleted from src/components; the contract renderer keeps a small compatibility renderer."),
  support("common", "Common support types", "common"),
  behavior("context-menu", "Context Menu", "context_menu", ["context-menu"], "Retains internal Rust runtime behavior for egui context menu opening and command dispatch."),
  behaviorAlias("dialogue", "Dialogue", "dialogue", ["dialogue-modal"], "Retains internal Rust runtime behavior for modal events while JSX authors compose dialogue surfaces in TSX."),
  behavior("drag-board", "Drag Board", "drag_board", ["drag-board"], "Retains internal Rust runtime behavior for drag/drop mutation semantics."),
  behavior("dropdown-menu", "Dropdown Menu", "dropdown_menu", ["dropdown-menu"], "Retains internal Rust runtime behavior for popup menu opening and command dispatch."),
  tsxOwned("emoji-selector", "Emoji Selector", "emoji_selector", ["emoji-selector"], "TSX owns the emoji selector recipe composed from button, card, row, column, and twemoji primitives. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  support("emoji-selector-data", "Emoji Selector Data", "emoji_selector_data", "examples/runtime-jsx/ui/components/emoji-selector.tsx", "The JSX emoji selector reuses the egui contract family; data remains owned by the Rust renderer."),
  tsxOwned("field", "Field", "field", ["field"], "TSX owns the field recipe composed from label, input, and helper text primitives. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  behavior("file-tree", "File Tree", "file_tree", ["file-tree"], "Retains internal Rust runtime behavior for tree expansion and selection events."),
  behavior("hierarchy", "Hierarchy", "hierarchy", ["hierarchy"], "Retains internal Rust runtime behavior for hierarchy selection, expansion, and drag/drop movement."),
  entry("icon", "Icon", "icon", ["icon"]),
  tsxOwned("icon-toolbar", "Icon Toolbar", "icon_toolbar", ["icon-toolbar"], "TSX owns the icon toolbar component. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  entry("image", "Image", "image", ["image"]),
  tsxOwned("image-tile", "Image Tile", "image_tile", ["image-tile"], "TSX owns the image tile component. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  entry("input", "Input", "input", ["input"]),
  tsxOwned("kbd", "Kbd", "kbd", ["kbd"], "TSX owns the keyboard keycap primitive composed from card and label primitives. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  entry("label", "Label", "label", ["label"]),
  tsxOwned("menu-bar", "Menu Bar", "menu_bar", ["menu-bar"], "TSX owns the menu bar recipe. The Rust component module was deleted from src/components; the contract renderer keeps a small compatibility renderer."),
  tsxOwned("open-with", "Open With", "open_with", ["open-with"], "TSX owns the split-button recipe. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  tsxOwned("pagination", "Pagination", "pagination", ["pagination"], "TSX owns the pagination recipe composed from buttons, labels, and row layout primitives. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  tsxOwned("palette-color-input", "Palette Color Input", "palette_color_input", [], "TSX now owns the palette color input recipe composed from color, input, and palette preview patterns. The Rust component module was deleted from src/components."),
  tsxOwned("palette-preview", "Palette Preview", "palette_preview", [], "TSX now owns the palette preview recipe composed from color primitives. The Rust component module was deleted from src/components."),
  behavior("popover", "Popover", "popover", ["popover"], "Retains internal Rust runtime behavior for popup lifecycle and positioning."),
  tsxOwned("progress", "Progress", "progress", ["progress"], "TSX owns the progress primitive composed from card primitives. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  entry("radio", "Radio", "radio", ["radio", "radio-group"]),
  behavior("select", "Select", "select", ["select"], "Retains internal Rust runtime behavior for select popup state and event dispatch while JSX authors compose the hot-reloading TSX recipe."),
  tsxOwned("separator", "Separator", "separator", ["separator"], "TSX owns the separator primitive composed from card primitives. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  behavior("sidebar", "Sidebar", "sidebar", ["sidebar"], "Retains internal Rust runtime behavior for sidebar close events while JSX owns the surface recipe."),
  tsxOwned("skeleton", "Skeleton", "skeleton", ["skeleton"], "TSX owns the skeleton primitive composed from card primitives. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  entry("slider", "Slider", "slider", ["slider", "number-input"]),
  tsxOwned("spinner", "Spinner", "spinner", ["spinner"], "TSX owns the spinner primitive composed from icon primitives. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  entry("switch", "Switch", "switch", ["switch"]),
  tsxOwned("tabs", "Tabs", "tabs", ["tabs"], "TSX owns the tabs recipe. The Rust component module was deleted from src/components; the contract renderer keeps a small compatibility renderer."),
  behaviorAlias("toast", "Toast", "toast", ["toast-viewport"], "Retains internal Rust runtime behavior for toast viewport lifecycle and expiry; JSX can compose individual toast rows."),
  tsxOwned("toggle-group", "Toggle Group", "toggle_group", ["toggle-group"], "TSX now owns the toggle group recipe composed from buttons. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  tsxOwned("toolbar", "Toolbar", "toolbar", ["toolbar"], "TSX owns the toolbar surface recipe. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
  behavior("tooltip", "Tooltip", "tooltip", ["tooltip"], "Retains internal Rust runtime behavior for hover-triggered tooltip display."),
  tsxOwned("twemoji", "Twemoji", "twemoji", ["twemoji"], "TSX owns the twemoji primitive composed from image primitives. The Rust component module was deleted from src/components; the contract renderer keeps a compatibility family."),
];

export const pendingMigrationCount = migrationManifest.filter(
  (item) => item.status === "pending",
).length;

export const tsxOwnedCount = migrationManifest.filter(
  (item) => item.rustApiStatus === "tsx-owned",
).length;

export const runtimePrimitiveCount = migrationManifest.filter(
  (item) => item.rustApiStatus === "runtime-primitive",
).length;

export const runtimeBehaviorCount = migrationManifest.filter(
  (item) => item.rustApiStatus === "runtime-behavior",
).length;

function componentTargetPath(id: string) {
  if (rootComponentIds.has(id)) {
    return `examples/runtime-jsx/ui/components/${id}.tsx`;
  }

  const layer = primaryComponentIds.has(id) ? "primary" : "secondary";
  return `examples/runtime-jsx/ui/components/${layer}/${id}.tsx`;
}

function entry(
  id: string,
  label: string,
  sourceModule: string,
  contractFamilies: string[],
  rustApiStatus: RustApiStatus = "runtime-primitive",
  notes?: string,
): MigrationEntry {
  return {
    id,
    label,
    source: `src/components/${sourceModule}.rs`,
    target: componentTargetPath(id),
    strategy: "contract-family",
    status: "implemented",
    contractFamilies,
    rustApiStatus,
    notes,
  };
}

function alias(
  id: string,
  label: string,
  sourceModule: string,
  contractFamilies: string[],
  rustApiStatus: RustApiStatus = "runtime-primitive",
  notes?: string,
): MigrationEntry {
  return {
    ...entry(id, label, sourceModule, contractFamilies, rustApiStatus, notes),
    strategy: "family-alias",
  };
}

function primitive(
  id: string,
  label: string,
  sourceModule: string,
  contractFamilies: string[],
  notes?: string,
): MigrationEntry {
  return {
    ...compose(id, label, sourceModule, notes ?? "Retains an active internal Rust runtime primitive for the JSX contract renderer."),
    contractFamilies,
    rustApiStatus: "runtime-primitive",
  };
}

function behavior(
  id: string,
  label: string,
  sourceModule: string,
  contractFamilies: string[],
  notes?: string,
): MigrationEntry {
  return entry(id, label, sourceModule, contractFamilies, "runtime-behavior", notes);
}

function behaviorAlias(
  id: string,
  label: string,
  sourceModule: string,
  contractFamilies: string[],
  notes?: string,
): MigrationEntry {
  return alias(id, label, sourceModule, contractFamilies, "runtime-behavior", notes);
}

function tsxOwned(
  id: string,
  label: string,
  sourceModule: string,
  contractFamilies: string[],
  notes: string,
): MigrationEntry {
  return {
    id,
    label,
    source: `deleted src/components/${sourceModule}.rs`,
    target: componentTargetPath(id),
    strategy: "compose",
    status: "implemented",
    contractFamilies: contractFamilies.length > 0 ? contractFamilies : undefined,
    rustApiStatus: "tsx-owned",
    notes,
  };
}

function compose(id: string, label: string, sourceModule: string, notes: string): MigrationEntry {
  return {
    id,
    label,
    source: `src/components/${sourceModule}.rs`,
    target: componentTargetPath(id),
    strategy: "compose",
    status: "implemented",
    notes,
  };
}


function support(
  id: string,
  label: string,
  sourceModule: string,
  target = "examples/runtime-jsx/ui/components/primary/common.tsx",
  notes = "Shared Rust support types inform JSX layout wrappers and style presets rather than a standalone preview.",
): MigrationEntry {
  return {
    id,
    label,
    source: `src/components/${sourceModule}.rs`,
    target,
    strategy: "support-types",
    status: "implemented",
    notes,
  };
}
