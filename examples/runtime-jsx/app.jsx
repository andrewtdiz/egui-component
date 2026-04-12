import { eventValue, log, render, useState } from "egui";

const assetOptions = [
  { itemId: "material-glass", label: "Material Glass" },
  { itemId: "material-metal", label: "Material Metal" },
  { itemId: "sprite-atlas", label: "Sprite Atlas" },
  { itemId: "ui-text-style", label: "UI Text Style" },
];

const statusItems = [
  { itemId: "draft", label: "Draft" },
  { itemId: "review", label: "Review" },
  { itemId: "approved", label: "Approved" },
  { itemId: "archived", label: "Archived" },
];

const toolbarItems = [
  { itemId: "select", icon: "mouse-pointer-2", tooltip: "Select" },
  { itemId: "move", icon: "move", tooltip: "Move" },
  { itemId: "rotate", icon: "rotate-ccw", tooltip: "Rotate" },
  { itemId: "delete", icon: "trash", tooltip: "Delete", badgeFill: "#2896ff" },
];

const menuEntries = [
  { itemId: "profile", label: "Profile", leadingIcon: "user", shortcut: "Shift+Cmd+P" },
  { itemId: "settings", label: "Settings", leadingIcon: "settings", shortcut: "Cmd+S" },
  "-",
  {
    kind: "submenu",
    label: "Invite users",
    leadingIcon: "users",
    entries: [
      { itemId: "email", label: "Email", leadingIcon: "mail" },
      { itemId: "message", label: "Message", leadingIcon: "message-square" },
    ],
  },
  "-",
  { itemId: "delete", label: "Delete", leadingIcon: "trash" },
];

const commandItems = [
  { itemId: "open", group: "scene", label: "open scene search", shortcut: "Ctrl+P" },
  { itemId: "save", group: "scene", label: "save active scene", shortcut: "Ctrl+S" },
  { itemId: "camera", group: "gameobject", label: "add camera" },
  { itemId: "gizmos", group: "view", label: "toggle gizmos" },
  { itemId: "navmesh", group: "tools", label: "build nav mesh", shortcut: "Ctrl+B" },
];

function Section({ title, children }) {
  return (
    <card paddingX={14} paddingY={14}>
      <column gap={8}>
        <label text={title} tone="primary" weight="semibold" />
        {children}
      </column>
    </card>
  );
}

function Swatches() {
  return (
    <row gap={8} align="center">
      <color fill="#23304f" size={22} stroke={{ width: 1, color: "#94a3b8" }} cornerRadius={6} />
      <color fill="#6784a2" size={22} stroke={{ width: 1, color: "#0f172a" }} cornerRadius={6} />
      <color fill="#7a182a" size={22} stroke={{ width: 1, color: "#fecdd3" }} cornerRadius={6} />
      <color fill="#cea454" size={22} stroke={{ width: 1, color: "#fef3c7" }} cornerRadius={6} />
    </row>
  );
}

function Primitives() {
  return (
    <Section title="Primitives">
      <row gap={12} align="center">
        <label text="Primary label" tone="primary" weight="semibold" />
        <label text="Secondary label" tone="secondary" />
        <label text="Muted helper text" tone="muted" />
        <label text="ClassName label" className="text-destructive font-bold text-lg" />
      </row>
      <row gap={10} align="center">
        <icon name="bot" size={16} />
        <icon name="settings-2" size={16} />
        <icon name="sparkles" size={16} tint="#f1a84e" />
        <twemoji emoji="🔥" size={28} />
        <kbd text="Ctrl" minWidth={30} height={22} />
        <kbd text="B" minWidth={22} height={22} />
      </row>
      <Swatches />
      <image source="builtin:showcase-image" width={180} height={120} cornerRadius={8} />
      <skeleton width={240} height={14} cornerRadius={7} animated={true} />
      <row gap={12} align="center">
        <spinner size={18} />
        <progress value={0.72} width={260} height={10} />
      </row>
    </Section>
  );
}

function Controls() {
  const [name, setName] = useState("ViewportHeader");
  const [owner, setOwner] = useState("Editor Team");
  const [snap, setSnap] = useState(true);
  const [enabled, setEnabled] = useState(true);
  const [radio, setRadio] = useState(false);
  const [plan, setPlan] = useState("team");
  const [rotation, setRotation] = useState(24);
  const [status, setStatus] = useState("review");
  const [emoji, setEmoji] = useState("🙂");
  const [page, setPage] = useState(2);
  const [comboQuery, setComboQuery] = useState("");
  const [selectedAssets, setSelectedAssets] = useState(["material-glass"]);

  return (
    <Section title="Controls">
      <row gap={10} align="center">
        <input
          nodeId="component-name"
          value={name}
          placeholder="Component name"
          width={240}
          leadingIcon="search"
          onChange={(event) => setName(eventValue(event) ?? "")}
        />
        <field
          nodeId="owner-field"
          label="Owner"
          value={owner}
          helperText="Host state updates through JSX hooks."
          width={240}
          onChange={(event) => setOwner(eventValue(event) ?? "")}
        />
      </row>
      <row gap={12} align="center">
        <checkbox
          nodeId="snap-checkbox"
          label="Snap to grid"
          value={snap}
          onToggle={(event) => setSnap(Boolean(eventValue(event)))}
        />
        <switch
          nodeId="enabled-switch"
          label="Use compact handles"
          size="sm"
          value={enabled}
          onToggle={(event) => setEnabled(Boolean(eventValue(event)))}
        />
      </row>
      <slider
        nodeId="rotation-slider"
        value={rotation}
        min={0}
        max={360}
        width={260}
        onChange={(event) => setRotation(eventValue(event) ?? rotation)}
      />
      <numberInput
        nodeId="rotation-number"
        value={rotation}
        min={0}
        max={360}
        width={96}
        decimals={1}
        suffix="deg"
        axis="horizontal"
        onChange={(event) => setRotation(eventValue(event) ?? rotation)}
      />
      <select
        nodeId="status-select"
        selectedItemId={status}
        width={240}
        leadingIcon="badge-check"
        items={statusItems}
        onSelect={(event) => setStatus(event.metadata?.item_id ?? status)}
      />
      <combobox
        nodeId="asset-combobox"
        query={comboQuery}
        selectedItemIds={selectedAssets}
        items={assetOptions}
        width={280}
        placeholder="Select assets"
        filterPlaceholder="Filter assets"
        onChange={(event) => setComboQuery(eventValue(event) ?? "")}
        onSelect={(event) => setSelectedAssets(eventValue(event) ?? [])}
      />
      <radio
        nodeId="publish-radio"
        value={radio}
        label="Use publish channel"
        description="Standalone radios stay selected until reset."
        onToggle={(event) => setRadio(Boolean(eventValue(event)))}
      />
      <radioGroup
        nodeId="plan-radio-group"
        selectedItemId={plan}
        items={[
          { itemId: "starter", label: "Starter", description: "Basic surfaces" },
          { itemId: "team", label: "Team", description: "Shared component previews" },
          { itemId: "enterprise", label: "Enterprise", description: "Extended controls" },
        ]}
        onSelect={(event) => setPlan(event.metadata?.item_id ?? plan)}
      />
      <row gap={12} align="center">
        <emojiSelector
          nodeId="emoji-selector"
          value={emoji}
          triggerVariant="secondary"
          onSelect={(event) => setEmoji(eventValue(event) ?? emoji)}
        />
        <pagination
          nodeId="pagination"
          currentPage={page}
          pageCount={8}
          onSelect={(event) => setPage(Math.round(eventValue(event) ?? page))}
        />
      </row>
    </Section>
  );
}

function MenusAndSurfaces() {
  const [tab, setTab] = useState("design");
  const [detailsOpen, setDetailsOpen] = useState(true);
  const [popoverOpen, setPopoverOpen] = useState(false);
  const [dialogueOpen, setDialogueOpen] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [lastAction, setLastAction] = useState("No action triggered");
  const [toasts, setToasts] = useState([
    {
      itemId: "toast-ready",
      title: "JSX runtime ready",
      description: "Events can flow back into hooks.",
      intent: "neutral",
      durationSecs: 0,
    },
  ]);

  const pushToast = (title, intent = "neutral") => {
    setToasts((items) => [
      ...items,
      {
        itemId: `toast-${Date.now()}`,
        title,
        description: "Generated by a JSX event handler.",
        intent,
        durationSecs: 4,
      },
    ]);
  };

  return (
    <Section title="Menus and surfaces">
      <menuBar
        nodeId="main-menu"
        menus={[
          { menuId: "file", label: "File", width: 220, entries: menuEntries },
          {
            menuId: "view",
            label: "View",
            width: 196,
            entries: [
              { itemId: "sidebar", label: "Open Sidebar", leadingIcon: "panel-right-open" },
              { itemId: "dialogue", label: "Open Dialogue", leadingIcon: "message-square" },
            ],
          },
        ]}
        onCommand={(event) => setLastAction(event.metadata?.item_label ?? "Menu action")}
      />
      <tabs
        nodeId="main-tabs"
        style="segmented"
        selectedItemId={tab}
        items={[
          { itemId: "design", label: "Design" },
          { itemId: "code", label: "Code" },
          { itemId: "history", label: "History" },
        ]}
        onSelect={(event) => setTab(event.metadata?.item_id ?? tab)}
      />
      <row gap={8} align="center">
        <button variant="primary" actionId="dialogue.open" onClick={() => setDialogueOpen(true)}>
          Open Dialogue
        </button>
        <button variant="secondary" actionId="sidebar.open" onClick={() => setSidebarOpen(true)}>
          Open Sidebar
        </button>
        <button variant="ghost" actionId="toast.add" onClick={() => pushToast("Changes saved", "success")}>
          Add Toast
        </button>
      </row>
      <row gap={8} align="center">
        <dropdownMenu
          nodeId="dropdown-menu"
          triggerLabel="Open"
          entries={menuEntries}
          onCommand={(event) => setLastAction(event.metadata?.item_label ?? "Dropdown action")}
        />
        <openWith
          nodeId="open-with"
          selectedItemId="codex"
          entries={[{ itemId: "codex", label: "Codex", leadingIcon: "codex" }]}
          onCommand={(event) => setLastAction(event.metadata?.item_label ?? "Open with")}
        />
        <tooltip triggerLabel="Hover this trigger" text="Tooltip content example" placement="top" />
      </row>
      <popover
        nodeId="settings-popover"
        open={popoverOpen}
        triggerLabel="Open Popover"
        width={280}
        side="bottom"
        align="center"
        onOpen={() => setPopoverOpen(true)}
        onClose={() => setPopoverOpen(false)}
      >
        <label text="Interactive popovers can host contract children." tone="muted" />
      </popover>
      <contextMenu
        nodeId="context-menu"
        width={220}
        regionWidth={420}
        regionHeight={132}
        entries={menuEntries}
        onCommand={(event) => setLastAction(event.metadata?.item_label ?? "Context menu action")}
      >
        <column gap={6}>
          <label text="Scene View" tone="muted" size={11} />
          <label text="Right-click this region" tone="primary" weight="semibold" size={18} />
        </column>
      </contextMenu>
      <collapsible
        nodeId="details"
        title="Transform"
        open={detailsOpen}
        leadingIcon="move-3d"
        trailingIcon="ellipsis"
        onToggle={(event) => setDetailsOpen(Boolean(eventValue(event)))}
      >
        <label text="Position" tone="secondary" />
        <label text="Rotation" tone="secondary" />
        <label text="Scale" tone="secondary" />
      </collapsible>
      <dialogueModal
        nodeId="dialogue"
        open={dialogueOpen}
        title="Create Component"
        description="Adds the selected component to the active object."
        confirmLabel="Create"
        cancelLabel="Cancel"
        intent="default"
        width={380}
        onConfirm={() => {
          setDialogueOpen(false);
          pushToast("Component created", "success");
        }}
        onCancel={() => setDialogueOpen(false)}
        onClose={() => setDialogueOpen(false)}
      >
        <label text="The modal body is authored from JSX children." tone="muted" />
      </dialogueModal>
      <sidebar
        nodeId="sidebar"
        title="Workspace"
        side="right"
        width={280}
        open={sidebarOpen}
        onClose={() => setSidebarOpen(false)}
      >
        <button variant="primary" leadingIcon="file-plus">New Draft</button>
        <button variant="ghost" leadingIcon="search">Command Search</button>
        <button variant="secondary" onClick={() => setSidebarOpen(false)}>Close Sidebar</button>
      </sidebar>
      <toastViewport
        nodeId="toasts"
        placement="bottomRight"
        width={300}
        toasts={toasts}
        onClose={(event) => {
          const itemId = event.metadata?.item_id;
          if (itemId) {
            setToasts((items) => items.filter((item) => item.itemId !== itemId));
          }
        }}
      />
      <label text={`Last action: ${lastAction}`} tone="muted" size={11} />
    </Section>
  );
}

function ShowcaseExamples() {
  const [selectedTool, setSelectedTool] = useState("move");
  const [audioState, setAudioState] = useState("Paused");
  const [tilePlayback, setTilePlayback] = useState("Paused");
  const [tileSelected, setTileSelected] = useState(true);
  const [commandQuery, setCommandQuery] = useState("");
  const [selectedFile, setSelectedFile] = useState("main-script");
  const [selectedHierarchy, setSelectedHierarchy] = useState("scene-root");
  const [boardItems, setBoardItems] = useState([
    { itemId: "spacing", title: "Polish header spacing", description: "Shared toolbar chrome", region: "Left" },
    { itemId: "sidebar", title: "Tune sidebar spacing", description: "Examples rail", region: "Left" },
    { itemId: "runtime", title: "Ship JSX runtime", description: "Feature parity", region: "Right" },
  ]);

  return (
    <Section title="Examples">
      <toolbar anchor="top_center" offsetY={10}>
        <button iconOnly={true} leadingIcon="move" variant="ghost" actionId="tool.move" />
        <button iconOnly={true} leadingIcon="rotate-ccw" variant="ghost" actionId="tool.rotate" />
        <button iconOnly={true} leadingIcon="maximize-2" variant="ghost" actionId="tool.scale" />
      </toolbar>
      <iconToolbar
        nodeId="icon-toolbar"
        selectedItemId={selectedTool}
        items={toolbarItems}
        onSelect={(event) => setSelectedTool(event.metadata?.item_id ?? selectedTool)}
      />
      <collabCursor name="Lisa Chen" x={96} y={34} color="#39bdf8" size={32} />
      <fileTree
        nodeId="file-tree"
        selectedItemId={selectedFile}
        width={260}
        items={[
          {
            itemId: "workspace",
            label: "Workspace",
            kind: "Project",
            open: true,
            children: [
              {
                itemId: "scripts",
                label: "scripts",
                kind: "Folder",
                open: true,
                children: [{ itemId: "main-script", label: "main.jsx", kind: "Script" }],
              },
              { itemId: "readme", label: "README.md", kind: "Markdown" },
            ],
          },
        ]}
        onSelect={(event) => setSelectedFile(event.metadata?.item_id ?? selectedFile)}
      />
      <hierarchy
        nodeId="hierarchy"
        selectedItemId={selectedHierarchy}
        width={340}
        rowHeight={32}
        iconStyle="icons"
        style="component"
        items={[
          {
            itemId: "scene-root",
            label: "Scene Root",
            kind: "Folder",
            open: true,
            children: [
              { itemId: "hero", label: "Hero", kind: "Player" },
              { itemId: "weapon", label: "Weapon", kind: "Weapon" },
            ],
          },
        ]}
        onSelect={(event) => setSelectedHierarchy(event.metadata?.item_id ?? selectedHierarchy)}
      />
      <dragBoard
        nodeId="drag-board"
        leftTitle="Backlog"
        rightTitle="Done"
        items={boardItems}
        height={220}
        onChange={(event) => {
          const move = eventValue(event);
          if (move) {
            setBoardItems((items) =>
              items.map((item) =>
                item.itemId === move.item_id ? { ...item, region: move.to === "right" ? "Right" : "Left" } : item,
              ),
            );
          }
        }}
      />
      <audioPlayback
        nodeId="audio-playback"
        playbackState={audioState}
        onToggle={() => setAudioState((state) => (state === "Paused" ? "Playing" : "Paused"))}
      >
        <button iconOnly={true} leadingIcon="download" variant="ghost" size="sm" />
        <button iconOnly={true} leadingIcon="ellipsis" variant="ghost" size="sm" />
      </audioPlayback>
      <imageTile
        nodeId="image-tile"
        source="builtin:showcase-image"
        size="md"
        selected={tileSelected}
        playbackState={tilePlayback}
        onClick={() => setTileSelected((selected) => !selected)}
        onToggle={() => setTilePlayback((state) => (state === "Paused" ? "Playing" : "Paused"))}
      >
        <label text="Ambient Preview" tone="primary" weight="semibold" />
        <label text={`Playback: ${tilePlayback}`} tone="muted" size={11} />
      </imageTile>
      <command
        nodeId="command"
        query={commandQuery}
        width={380}
        maxHeight={180}
        preview={true}
        previewHeight={220}
        items={commandItems}
        onChange={(event) => setCommandQuery(eventValue(event) ?? "")}
      />
    </Section>
  );
}

function CanvaRecipes() {
  return (
    <Section title="Canva recipes">
      <card paddingX={16} paddingY={16}>
        <column gap={10}>
          <label text="Backgrounds" tone="primary" weight="semibold" />
          <input value="" placeholder="Search backgrounds" width={320} leadingIcon="search" />
          <row gap={8} align="center">
            <button iconOnly={true} leadingIcon="palette" variant="secondary" />
            <color fill="#9ab5d0" size={32} cornerRadius={8} />
            <color fill="#f7f6f3" size={32} cornerRadius={8} />
            <color fill="#080808" size={32} cornerRadius={8} />
            <button iconOnly={true} leadingIcon="chevron-right" variant="primary" />
          </row>
          <button variant="secondary" leadingIcon="sparkles" trailingIcon="crown">Magic Background</button>
        </column>
      </card>
      <card paddingX={16} paddingY={16}>
        <column gap={10}>
          <label text="Brand Kit" tone="primary" weight="semibold" />
          <tabs
            style="segmented"
            selectedItemId="logos"
            items={[
              { itemId: "logos", label: "Logos" },
              { itemId: "colors", label: "Colors" },
              { itemId: "fonts", label: "Fonts" },
            ]}
          />
          <Swatches />
          <button variant="secondary" leadingIcon="upload">Upload brand asset</button>
        </column>
      </card>
      <card paddingX={16} paddingY={16}>
        <column gap={10}>
          <label text="Edit image" tone="primary" weight="semibold" />
          <row gap={8} align="center">
            <button variant="secondary" leadingIcon="image">All</button>
            <button variant="ghost" leadingIcon="mouse-pointer-click">Click</button>
            <button variant="ghost" leadingIcon="wand-sparkles">Magic</button>
          </row>
          <button variant="secondary" leadingIcon="sliders-horizontal">Adjust</button>
          <button variant="secondary" leadingIcon="sparkles">BG Remover</button>
        </column>
      </card>
      <card paddingX={16} paddingY={16}>
        <column gap={10}>
          <label text="Position" tone="primary" weight="semibold" />
          <tabs
            style="segmented"
            selectedItemId="arrange"
            items={[
              { itemId: "arrange", label: "Arrange" },
              { itemId: "layers", label: "Layers" },
            ]}
          />
          <row gap={8} align="center">
            <numberInput value={1920} width={90} suffix="px" />
            <numberInput value={1080} width={90} suffix="px" />
            <numberInput value={0} width={90} suffix="deg" />
          </row>
        </column>
      </card>
    </Section>
  );
}

function App() {
  return (
    <column nodeId="jsx-runtime-root" gap={12}>
      <row gap={8} align="center">
        <label text="JSX authored egui component library" tone="primary" weight="semibold" size={22} />
        <button nodeId="header-reload" actionId="reload" variant="secondary" size="sm">
          Runtime session
        </button>
      </row>
      <label
        text="Edit this file in the left panel or on disk. Events are dispatched back into V8 and hooks rebuild the contract tree."
        tone="muted"
        size={12}
      />
      <separator nodeId="intro-separator" />
      <Primitives />
      <Controls />
      <MenusAndSurfaces />
      <ShowcaseExamples />
      <CanvaRecipes />
    </column>
  );
}

log("info", "examples/runtime-jsx/app.jsx rendered with component parity catalog");
render(<App />);
