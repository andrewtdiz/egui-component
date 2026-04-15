import { eventValue, useState } from "egui";
import { AudioPlayback } from "../audio-playback.tsx";
import { Button } from "../button.tsx";
import { ButtonGroup } from "../button-group.tsx";
import { CanvaAxisField, CanvaColorStop, CanvaInspectorCard, CanvaTimelineAction } from "./canva.tsx";
import { Card } from "../card.tsx";
import { Checkbox } from "../checkbox.tsx";
import { CollabCursor } from "../collab-cursor.tsx";
import { Collapsible } from "../collapsible.tsx";
import { Color } from "../primary/color.tsx";
import { ColorInput } from "../color-input.tsx";
import { ColorStrip } from "./color-strip.tsx";
import { Combobox } from "../combobox.tsx";
import { Command } from "../command.tsx";
import { ContextMenu } from "../context-menu.tsx";
import { DialogueModal } from "../dialogue.tsx";
import { DragBoard } from "../drag-board.tsx";
import { DropdownMenu } from "../dropdown-menu.tsx";
import { EmojiSelector } from "../emoji-selector.tsx";
import { Field } from "../field.tsx";
import { FileTree } from "../file-tree.tsx";
import { Hierarchy } from "../hierarchy.tsx";
import { Icon } from "../primary/icon.tsx";
import { IconToolbar } from "../icon-toolbar.tsx";
import { Image } from "../primary/image.tsx";
import { ImageTile } from "../image-tile.tsx";
import { Input } from "../input.tsx";
import { Kbd } from "../primary/kbd.tsx";
import { Label, LabelMuted } from "../primary/label.tsx";
import { MenuBar } from "../menu-bar.tsx";
import { OpenWith } from "../open-with.tsx";
import { Pagination } from "../pagination.tsx";
import { PaletteColorInput } from "../palette-color-input.tsx";
import { PalettePreview } from "./palette-preview.tsx";
import { Popover } from "../popover.tsx";
import { Progress } from "../primary/progress.tsx";
import { Radio, RadioGroup } from "../radio.tsx";
import { Select } from "../select.tsx";
import { Separator } from "../primary/separator.tsx";
import { Sidebar } from "../sidebar.tsx";
import { Skeleton } from "../primary/skeleton.tsx";
import { NumberInput, Slider } from "../slider.tsx";
import { Spinner } from "../primary/spinner.tsx";
import { Switch } from "../switch.tsx";
import { Tabs } from "../tabs.tsx";
import { ToastViewport } from "../toast.tsx";
import { ToggleGroup } from "../toggle-group.tsx";
import { Toolbar, ToolbarButton } from "../toolbar.tsx";
import { Tooltip } from "../tooltip.tsx";
import { Twemoji } from "../primary/twemoji.tsx";

const choiceItems = [{ itemId: "starter", label: "Starter" }, { itemId: "team", label: "Team" }, { itemId: "enterprise", label: "Enterprise" }];
const menuEntries = [{ itemId: "profile", label: "Profile", leadingIcon: "user", shortcut: "P" }, { itemId: "settings", label: "Settings", leadingIcon: "settings", shortcut: "S" }, "-", { label: "Create", leadingIcon: "plus", entries: [{ itemId: "scene", label: "Scene", leadingIcon: "box" }] }];
const tabs = [{ itemId: "overview", label: "Overview" }, { itemId: "activity", label: "Activity" }, { itemId: "settings", label: "Settings" }];
const toolbarItems = [{ itemId: "select", icon: "mouse-pointer-2", tooltip: "Select" }, { itemId: "move", icon: "move", tooltip: "Move" }, { itemId: "rotate", icon: "rotate-ccw", tooltip: "Rotate" }];
const fileTreeItems = [{ itemId: "workspace", label: "Workspace", kind: "project", open: true, children: [{ itemId: "components", label: "components", kind: "folder", open: true, children: [{ itemId: "button", label: "button.tsx", kind: "script" }] }] }];
const hierarchyItems = [{ itemId: "scene", label: "Scene", kind: "folder", open: true, children: [{ itemId: "player", label: "Player", kind: "player" }] }];
const boardItems = [{ itemId: "tokens", title: "Token sweep", description: "Class presets", region: "left" }, { itemId: "states", title: "State audit", description: "Explicit egui props", region: "right" }];
const commandItems = [{ itemId: "open", group: "workspace", label: "Open command menu", shortcut: "Ctrl+P" }, { itemId: "theme", group: "workspace", label: "Toggle theme" }];

type ComponentPreview = { id: string; label: string; section: string; summary: string; render: () => unknown };
function frame(id: string, children: unknown) {
  return (
    <div id={`jsx-${id}-preview`} className="flex flex-col items-stretch gap-2 py-[4px]">
      {children}
    </div>
  );
}
function preview(id: string, label: string, section: string, render: () => unknown): ComponentPreview { return { id, label, section, summary: `${label} shadcn-style JSX component.`, render }; }

export const componentPreviews: ComponentPreview[] = [
  preview("audio-playback", "Audio Playback", "media", () => frame("audio-playback", <AudioPlayback id="jsx-audio-playback" playbackState="playing"><Button id="jsx-audio-action" variant="ghost" size="sm" leadingIcon="download" iconOnly /></AudioPlayback>)),
  preview("button", "Button", "controls", () => frame("button", <div className="flex flex-row items-center gap-2"><Button id="jsx-button-primary" variant="primary">Primary</Button><Button id="jsx-button-selected" selected>Selected</Button><Button id="jsx-button-disabled" disabled>Disabled</Button></div>)),
  preview("button-group", "Button Group", "controls", ButtonGroupPreviewDemo),
  preview("canva", "Canva Helpers", "recipes", CanvaPreviewDemo),
  preview("card", "Card", "surfaces", () => frame("card", <Card id="jsx-card" className="bg-card border-border rounded-lg"><Label text="Card surface" /><LabelMuted text="Class names flow into the egui card renderer." /></Card>)),
  preview("checkbox", "Checkbox", "controls", CheckboxPreviewDemo),
  preview("collab-cursor", "Collab Cursor", "presence", () => frame("collab-cursor", <CollabCursor id="jsx-collab-cursor" name="Mira" x={80} y={24} color="#2896ff" />)),
  preview("collapsible", "Collapsible", "surfaces", () => frame("collapsible", <Collapsible id="jsx-collapsible" title="Advanced"><LabelMuted text="Nested content stays authored as JSX children." /></Collapsible>)),
  preview("color", "Color", "primitives", () => frame("color", <Color id="jsx-color" fill="#2896ff" size={28} cornerRadius={8} />)),
  preview("color-input", "Color Input", "recipes", ColorInputPreviewDemo),
  preview("color-strip", "Color Strip", "recipes", () => frame("color-strip", <ColorStrip id="jsx-color-strip" />)),
  preview("combobox", "Combobox", "controls", ComboboxPreviewDemo),
  preview("command", "Command", "menus", CommandPreviewDemo),
  preview("context-menu", "Context Menu", "menus", () => frame("context-menu", <ContextMenu id="jsx-context-menu" entries={menuEntries} regionWidth={320} regionHeight={96}><LabelMuted text="Right-click region" /></ContextMenu>)),
  preview("dialogue", "Dialogue", "surfaces", () => frame("dialogue", <DialogueModal id="jsx-dialogue" title="Create component" description="Authored in TSX" open={false}><LabelMuted text="Modal body" /></DialogueModal>)),
  preview("drag-board", "Drag Board", "data", () => frame("drag-board", <DragBoard id="jsx-drag-board" height={180} items={boardItems} />)),
  preview("dropdown-menu", "Dropdown Menu", "menus", DropdownMenuPreviewDemo),
  preview("emoji-selector", "Emoji Selector", "controls", EmojiSelectorPreviewDemo),
  preview("field", "Field", "controls", FieldPreviewDemo),
  preview("file-tree", "File Tree", "data", FileTreePreviewDemo),
  preview("hierarchy", "Hierarchy", "data", HierarchyPreviewDemo),
  preview("icon", "Icon", "primitives", () => frame("icon", <div className="flex flex-row items-center gap-2"><Icon id="jsx-icon-sparkles" name="sparkles" /><Icon id="jsx-icon-settings" name="settings-2" /></div>)),
  preview("icon-toolbar", "Icon Toolbar", "controls", IconToolbarPreviewDemo),
  preview("image", "Image", "media", () => frame("image", <Image id="jsx-image" width={180} height={108} cornerRadius={8} />)),
  preview("image-tile", "Image Tile", "media", () => frame("image-tile", <ImageTile id="jsx-image-tile" size="sm" selected playbackState="paused"><Label text="Preview tile" /><LabelMuted text="Playback paused" /></ImageTile>)),
  preview("input", "Input", "controls", InputPreviewDemo),
  preview("kbd", "Kbd", "primitives", () => frame("kbd", <div className="flex flex-row items-center gap-2"><Kbd id="jsx-kbd-command" text="Ctrl" /><Kbd id="jsx-kbd-key" text="K" /></div>)),
  preview("label", "Label", "primitives", () => frame("label", <div className="flex flex-col items-stretch gap-1"><Label id="jsx-label" text="Foreground label" className="text-lg font-semibold" /><LabelMuted id="jsx-muted-label" text="Muted label" /></div>)),
  preview("menu-bar", "Menu Bar", "menus", () => frame("menu-bar", <MenuBar id="jsx-menu-bar" menus={[{ menuId: "file", label: "File", width: 220, entries: menuEntries }]} />)),
  preview("open-with", "Open With", "menus", OpenWithPreviewDemo),
  preview("pagination", "Pagination", "controls", PaginationPreviewDemo),
  preview("palette-color-input", "Palette Color Input", "recipes", PaletteColorInputPreviewDemo),
  preview("palette-preview", "Palette Preview", "recipes", () => frame("palette-preview", <PalettePreview id="jsx-palette-preview" />)),
  preview("popover", "Popover", "surfaces", () => frame("popover", <Popover id="jsx-popover" triggerLabel="Open popover" width={260}><LabelMuted text="Popover body" /></Popover>)),
  preview("progress", "Progress", "feedback", () => frame("progress", <Progress id="jsx-progress" value={0.66} width={260} height={10} />)),
  preview("radio", "Radio", "controls", RadioPreviewDemo),
  preview("select", "Select", "controls", SelectPreviewDemo),
  preview("separator", "Separator", "primitives", () => frame("separator", <div className="flex flex-col items-stretch gap-1.5"><Label text="Before" /><Separator id="jsx-separator" /><LabelMuted text="After" /></div>)),
  preview("sidebar", "Sidebar", "surfaces", () => frame("sidebar", <Sidebar id="jsx-sidebar" title="Workspace" side="right" open={false}><Button variant="secondary">Close</Button></Sidebar>)),
  preview("skeleton", "Skeleton", "feedback", () => frame("skeleton", <Skeleton id="jsx-skeleton" width={260} height={16} cornerRadius={8} />)),
  preview("slider", "Slider", "controls", SliderPreviewDemo),
  preview("spinner", "Spinner", "feedback", () => frame("spinner", <Spinner id="jsx-spinner" size={18} />)),
  preview("switch", "Switch", "controls", SwitchPreviewDemo),
  preview("tabs", "Tabs", "navigation", TabsPreviewDemo),
  preview("toast", "Toast", "feedback", () => frame("toast", <ToastViewport id="jsx-toast-viewport" toasts={[{ itemId: "ready", title: "Ready", description: "Rendered from JSX.", intent: "success", durationSecs: 0 }]} />)),
  preview("toggle-group", "Toggle Group", "recipes", ToggleGroupPreviewDemo),
  preview("toolbar", "Toolbar", "surfaces", () => frame("toolbar", <Toolbar id="jsx-toolbar" anchor="top_center"><ToolbarButton iconOnly leadingIcon="move" /><ToolbarButton iconOnly leadingIcon="rotate-ccw" /></Toolbar>)),
  preview("tooltip", "Tooltip", "overlays", () => frame("tooltip", <Tooltip id="jsx-tooltip" triggerLabel="Hover" text="Tailwind-styled JSX trigger." />)),
  preview("twemoji", "Twemoji", "primitives", () => frame("twemoji", <Twemoji id="jsx-twemoji" emoji="🙂" size={28} />)),
];

function nextText(event: unknown, fallback: string) {
  const value = eventValue(event);
  return typeof value === "string" ? value : fallback;
}

function nextNumber(event: unknown, fallback: number) {
  const value = eventValue(event);
  return typeof value === "number" ? value : fallback;
}

function nextBoolean(event: unknown, fallback: boolean) {
  const value = eventValue(event);
  return typeof value === "boolean" ? value : fallback;
}

function resolveItemId(value: unknown, fallback: string) {
  return typeof value === "string" ? value : fallback;
}

function resolveItemIds(value: unknown, fallback: string[]) {
  return Array.isArray(value) ? value.map((item) => String(item)) : fallback;
}

function resolveNumber(value: unknown, fallback: number) {
  return typeof value === "number" ? value : fallback;
}

function ButtonGroupPreviewDemo() {
  const [selectedPlan, setSelectedPlan] = useState("team");

  return frame("button-group", <ButtonGroup id="jsx-button-group" items={choiceItems} selectedItemId={selectedPlan} onSelect={(_, value) => setSelectedPlan(resolveItemId(value, selectedPlan))} />);
}

function CanvaPreviewDemo() {
  const [positionX, setPositionX] = useState(120);
  const [positionY, setPositionY] = useState(80);
  const [accentColor, setAccentColor] = useState("#2896ff");

  return frame("canva", <CanvaInspectorCard id="jsx-canva-card" title="Position" description="Canvas inspector recipe"><CanvaAxisField id="jsx-canva-axis-x" axis="X" value={positionX} onChange={(event) => setPositionX(nextNumber(event, positionX))} /><CanvaAxisField id="jsx-canva-axis-y" axis="Y" value={positionY} onChange={(event) => setPositionY(nextNumber(event, positionY))} /><CanvaColorStop id="jsx-canva-color-stop" label="Accent" color={accentColor} onChange={(event) => setAccentColor(nextText(event, accentColor))} /><CanvaTimelineAction id="jsx-canva-timeline" /></CanvaInspectorCard>);
}

function CheckboxPreviewDemo() {
  const [checked, setChecked] = useState(true);

  return frame("checkbox", <Checkbox id="jsx-checkbox" value={checked} label="Sync layout tokens" onToggle={(event) => setChecked(nextBoolean(event, checked))} />);
}

function ColorInputPreviewDemo() {
  const [color, setColor] = useState("#10b981");

  return frame("color-input", <ColorInput id="jsx-color-input" value={color} onChange={(event) => setColor(nextText(event, color))} />);
}

function ComboboxPreviewDemo() {
  const [query, setQuery] = useState("");
  const [selectedItemIds, setSelectedItemIds] = useState(["team"]);

  return frame("combobox", <Combobox id="jsx-combobox" query={query} selectedItemIds={selectedItemIds} items={choiceItems} width={260} onChange={(event) => setQuery(nextText(event, query))} onSelect={(_, value) => setSelectedItemIds(resolveItemIds(value, selectedItemIds))} />);
}

function CommandPreviewDemo() {
  const [query, setQuery] = useState("");
  const [lastCommand, setLastCommand] = useState("Open command menu");

  return frame("command", <div className="flex flex-col items-stretch gap-2"><Command id="jsx-command" query={query} width={360} maxHeight={160} items={commandItems} onChange={(event) => setQuery(nextText(event, query))} onCommand={(_, value) => setLastCommand(resolveItemId(value, lastCommand))} /><LabelMuted id="jsx-command-selection" text={`Last command: ${lastCommand}`} /></div>);
}

function DropdownMenuPreviewDemo() {
  const [lastAction, setLastAction] = useState("profile");

  return frame("dropdown-menu", <div className="flex flex-col items-stretch gap-2"><DropdownMenu id="jsx-dropdown-menu" triggerLabel="Actions" entries={menuEntries} onCommand={(_, value) => setLastAction(resolveItemId(value, lastAction))} /><LabelMuted id="jsx-dropdown-menu-selection" text={`Last action: ${lastAction}`} /></div>);
}

function EmojiSelectorPreviewDemo() {
  const [emoji, setEmoji] = useState("🙂");

  return frame("emoji-selector", <EmojiSelector id="jsx-emoji-selector" value={emoji} triggerVariant="secondary" onSelect={(_, value) => setEmoji(resolveItemId(value, emoji))} />);
}

function FieldPreviewDemo() {
  const [value, setValue] = useState("Runtime JSX");

  return frame("field", <Field id="jsx-field" label="Name" value={value} helperText="Controlled from JS state." width={260} onChange={(event) => setValue(nextText(event, value))} />);
}

function FileTreePreviewDemo() {
  const [selectedItemId, setSelectedItemId] = useState("button");

  return frame("file-tree", <div className="flex flex-col items-stretch gap-2"><FileTree id="jsx-file-tree" selectedItemId={selectedItemId} width={280} items={fileTreeItems} onSelect={(_, value) => setSelectedItemId(resolveItemId(value, selectedItemId))} /><LabelMuted id="jsx-file-tree-selection" text={`Selected item: ${selectedItemId}`} /></div>);
}

function HierarchyPreviewDemo() {
  const [selectedItemId, setSelectedItemId] = useState("player");

  return frame("hierarchy", <div className="flex flex-col items-stretch gap-2"><Hierarchy id="jsx-hierarchy" selectedItemId={selectedItemId} width={320} iconStyle="icons" style="component" items={hierarchyItems} onSelect={(_, value) => setSelectedItemId(resolveItemId(value, selectedItemId))} /><LabelMuted id="jsx-hierarchy-selection" text={`Selected item: ${selectedItemId}`} /></div>);
}

function IconToolbarPreviewDemo() {
  const [selectedItemId, setSelectedItemId] = useState("move");

  return frame("icon-toolbar", <IconToolbar id="jsx-icon-toolbar" selectedItemId={selectedItemId} items={toolbarItems} onSelect={(_, value) => setSelectedItemId(resolveItemId(value, selectedItemId))} />);
}

function InputPreviewDemo() {
  const [value, setValue] = useState("className support");

  return frame("input", <Input id="jsx-input" value={value} leadingIcon="search" width={260} onChange={(event) => setValue(nextText(event, value))} />);
}

function OpenWithPreviewDemo() {
  const [selectedItemId, setSelectedItemId] = useState("profile");

  return frame("open-with", <OpenWith id="jsx-open-with" selectedItemId={selectedItemId} entries={menuEntries} onCommand={(_, value) => setSelectedItemId(resolveItemId(value, selectedItemId))} />);
}

function PaletteColorInputPreviewDemo() {
  const [color, setColor] = useState("#f43f5e");

  return frame("palette-color-input", <PaletteColorInput id="jsx-palette-color-input" value={color} onChange={(event) => setColor(nextText(event, color))} />);
}

function PaginationPreviewDemo() {
  const [page, setPage] = useState(2);

  return frame("pagination", <Pagination id="jsx-pagination" currentPage={page} pageCount={8} onSelect={(_, value) => setPage(resolveNumber(value, page))} />);
}

function RadioPreviewDemo() {
  const [checked, setChecked] = useState(false);
  const [selectedItemId, setSelectedItemId] = useState("team");

  return frame("radio", <div className="flex flex-col items-stretch gap-2"><Radio id="jsx-radio" checked={checked} label="Standalone radio" description="Explicit checked state" onToggle={(event) => setChecked(nextBoolean(event, checked))} /><RadioGroup id="jsx-radio-group" selectedItemId={selectedItemId} items={choiceItems} onSelect={(_, value) => setSelectedItemId(resolveItemId(value, selectedItemId))} /></div>);
}

function SelectPreviewDemo() {
  const [selectedItemId, setSelectedItemId] = useState("team");

  return frame("select", <Select id="jsx-select" selectedItemId={selectedItemId} items={choiceItems} width={240} onSelect={(_, value) => setSelectedItemId(resolveItemId(value, selectedItemId))} />);
}

function SliderPreviewDemo() {
  const [value, setValue] = useState(42);

  return frame("slider", <div className="flex flex-col items-stretch gap-2"><Slider id="jsx-slider" value={value} min={0} max={100} width={260} onChange={(event) => setValue(nextNumber(event, value))} /><NumberInput id="jsx-number-input" value={value} width={90} suffix="px" onChange={(event) => setValue(nextNumber(event, value))} /></div>);
}

function SwitchPreviewDemo() {
  const [checked, setChecked] = useState(true);

  return frame("switch", <Switch id="jsx-switch" checked={checked} label="Publish preview" size="sm" onToggle={(event) => setChecked(nextBoolean(event, checked))} />);
}

function TabsPreviewDemo() {
  const [selectedItemId, setSelectedItemId] = useState("activity");

  return frame("tabs", <Tabs id="jsx-tabs" selectedItemId={selectedItemId} style="segmented" items={tabs} onSelect={(_, value) => setSelectedItemId(resolveItemId(value, selectedItemId))} />);
}

function ToggleGroupPreviewDemo() {
  const [selectedItemId, setSelectedItemId] = useState("team");

  return frame("toggle-group", <ToggleGroup id="jsx-toggle-group" selectedItemId={selectedItemId} items={choiceItems} onSelect={(_, value) => setSelectedItemId(resolveItemId(value, selectedItemId))} />);
}
