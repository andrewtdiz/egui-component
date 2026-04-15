import {
  createContext,
  eventValue,
  log,
  render,
  requestRepaint,
  startTransition,
  useContext,
  useDeferredValue,
  useEffect,
  useReducer,
  useRef,
  useState,
  useSyncExternalStore,
} from "egui";
import {
  AudioPlayback,
  Button,
  Card,
  Checkbox,
  CollabCursor,
  Collapsible,
  Color,
  Combobox,
  Command,
  ContextMenu,
  DialogueModal,
  DragBoard,
  DropdownMenu,
  EmojiSelector,
  Field,
  FileTree,
  Hierarchy,
  Icon,
  IconToolbar,
  Image,
  ImageTile,
  Input,
  Kbd,
  Label,
  MenuBar,
  NumberInput,
  OpenWith,
  Pagination,
  Popover,
  Progress,
  Radio,
  RadioGroup,
  Select,
  Separator,
  Sidebar,
  Skeleton,
  Slider,
  Spinner,
  Switch,
  Tabs,
  ToastViewport,
  Toolbar,
  Tooltip,
  Twemoji,
} from "./ui/components/index.tsx";

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

const RuntimeModeContext = createContext("idle");

function createRuntimePulseStore() {
  let snapshot = { phase: "idle", tick: 0 };
  let intervalHandle = null;
  const listeners = new Set();

  const emit = () => {
    for (const listener of listeners) {
      listener();
    }
    requestRepaint();
  };

  return {
    getSnapshot() {
      return snapshot;
    },
    subscribe(listener) {
      listeners.add(listener);
      if (intervalHandle == null) {
        snapshot = { ...snapshot, phase: "live" };
        emit();
        intervalHandle = setInterval(() => {
          snapshot = { phase: "live", tick: snapshot.tick + 1 };
          emit();
        }, 1000);
      }
      return () => {
        listeners.delete(listener);
        if (listeners.size === 0 && intervalHandle != null) {
          clearInterval(intervalHandle);
          intervalHandle = null;
          snapshot = { ...snapshot, phase: "idle" };
        }
      };
    },
  };
}

const runtimePulseStore = createRuntimePulseStore();

const runtimeAsyncInitialState = {
  requestId: 0,
  status: "idle",
  summary: "Waiting for an async effect to resolve.",
};

function runtimeAsyncReducer(state, action) {
  switch (action.type) {
    case "loading":
      return {
        requestId: action.requestId,
        status: "loading",
        summary: `Scheduling async work for "${action.query}".`,
      };
    case "resolved":
      if (action.requestId !== state.requestId) {
        return state;
      }
      return {
        requestId: action.requestId,
        status: "ready",
        summary: action.summary,
      };
    default:
      return state;
  }
}

function RuntimeModeBadge() {
  const mode = useContext(RuntimeModeContext);
  return <Label nodeId="runtime-effects-mode" text={`Context mode: ${mode}`} tone="muted" size={11} />;
}

function Section({ title, children }) {
  return (
    <Card paddingX={14} paddingY={14}>
      <div className="flex flex-col gap-[8px]">
        <Label text={title} tone="primary" weight="semibold" />
        {children}
      </div>
    </Card>
  );
}

function Swatches() {
  return (
    <div className="flex flex-row gap-[8px] items-center">
      <Color fill="#23304f" size={22} stroke={{ width: 1, color: "#94a3b8" }} cornerRadius={6} />
      <Color fill="#6784a2" size={22} stroke={{ width: 1, color: "#0f172a" }} cornerRadius={6} />
      <Color fill="#7a182a" size={22} stroke={{ width: 1, color: "#fecdd3" }} cornerRadius={6} />
      <Color fill="#cea454" size={22} stroke={{ width: 1, color: "#fef3c7" }} cornerRadius={6} />
    </div>
  );
}

function Primitives() {
  return (
    <Section title="Primitives">
      <div className="flex flex-row gap-[12px] items-center">
        <Label text="Primary label" tone="primary" weight="semibold" />
        <Label text="Secondary label" tone="secondary" />
        <Label text="Muted helper text" tone="muted" />
        <Label text="ClassName label" className="text-destructive font-bold text-lg" />
      </div>
      <div className="flex flex-row gap-[10px] items-center">
        <Icon name="bot" size={16} />
        <Icon name="settings-2" size={16} />
        <Icon name="sparkles" size={16} tint="#f1a84e" />
        <Twemoji emoji="🔥" size={28} />
        <Kbd text="Ctrl" minWidth={30} height={22} />
        <Kbd text="B" minWidth={22} height={22} />
      </div>
      <Swatches />
      <Image source="builtin:showcase-image" width={180} height={120} cornerRadius={8} />
      <Skeleton width={240} height={14} cornerRadius={7} animated={true} />
      <div className="flex flex-row gap-[12px] items-center">
        <Spinner size={18} />
        <Progress nodeId="primitives-progress" value={0.72} width={260} height={10} />
      </div>
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
      <div className="flex flex-row gap-[10px] items-center">
        <Input
          nodeId="component-name"
          value={name}
          placeholder="Component name"
          width={240}
          leadingIcon="search"
          onChange={(event) => setName(eventValue(event) ?? "")}
        />
        <Field
          nodeId="owner-field"
          label="Owner"
          value={owner}
          helperText="Host state updates through JSX hooks."
          width={240}
          onChange={(event) => setOwner(eventValue(event) ?? "")}
        />
      </div>
      <div className="flex flex-row gap-[12px] items-center">
        <Checkbox
          nodeId="snap-checkbox"
          label="Snap to grid"
          value={snap}
          onToggle={(event) => setSnap(Boolean(eventValue(event)))}
        />
        <Switch
          nodeId="enabled-switch"
          label="Use compact handles"
          size="sm"
          value={enabled}
          onToggle={(event) => setEnabled(Boolean(eventValue(event)))}
        />
      </div>
      <Slider
        nodeId="rotation-slider"
        value={rotation}
        min={0}
        max={360}
        width={260}
        onChange={(event) => setRotation(eventValue(event) ?? rotation)}
      />
      <NumberInput
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
      <Select
        nodeId="status-select"
        selectedItemId={status}
        width={240}
        leadingIcon="badge-check"
        items={statusItems}
        onSelect={(event) => setStatus(event.metadata?.item_id ?? status)}
      />
      <Combobox
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
      <Radio
        nodeId="publish-radio"
        value={radio}
        label="Use publish channel"
        description="Standalone radios stay selected until reset."
        onToggle={(event) => setRadio(Boolean(eventValue(event)))}
      />
      <RadioGroup
        nodeId="plan-radio-group"
        selectedItemId={plan}
        items={[
          { itemId: "starter", label: "Starter", description: "Basic surfaces" },
          { itemId: "team", label: "Team", description: "Shared component previews" },
          { itemId: "enterprise", label: "Enterprise", description: "Extended controls" },
        ]}
        onSelect={(event) => setPlan(event.metadata?.item_id ?? plan)}
      />
      <div className="flex flex-row gap-[12px] items-center">
        <EmojiSelector
          nodeId="emoji-selector"
          value={emoji}
          triggerVariant="secondary"
          onSelect={(event) => setEmoji(eventValue(event) ?? emoji)}
        />
        <Pagination
          nodeId="pagination"
          currentPage={page}
          pageCount={8}
          onSelect={(event) => setPage(Math.round(eventValue(event) ?? page))}
        />
      </div>
    </Section>
  );
}

function RuntimeEffects() {
  const [query, setQuery] = useState("shader compiler");
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const deferredQuery = useDeferredValue(query);
  const pulse = useSyncExternalStore(
    runtimePulseStore.subscribe,
    runtimePulseStore.getSnapshot,
    runtimePulseStore.getSnapshot,
  );
  const [asyncState, dispatchAsync] = useReducer(
    runtimeAsyncReducer,
    runtimeAsyncInitialState,
  );
  const requestIdRef = useRef(0);

  useEffect(() => {
    log("info", "RuntimeEffects mounted local interval");
    const intervalHandle = setInterval(() => {
      setElapsedSeconds((value) => value + 1);
    }, 1000);
    return () => {
      clearInterval(intervalHandle);
      log("info", "RuntimeEffects cleaned local interval");
    };
  }, []);

  useEffect(() => {
    const requestId = requestIdRef.current + 1;
    requestIdRef.current = requestId;
    dispatchAsync({ type: "loading", query, requestId });
    const timeoutHandle = setTimeout(() => {
      startTransition(() => {
        dispatchAsync({
          type: "resolved",
          requestId,
          summary: `Resolved "${query}" after ${String(query.length)} token checks.`,
        });
      });
    }, 800);
    return () => {
      clearTimeout(timeoutHandle);
      log("info", `RuntimeEffects cleaned async request ${String(requestId)}`);
    };
  }, [query]);

  return (
    <Section title="Effects and async">
      <RuntimeModeContext.Provider value={asyncState.status}>
        <div className="flex flex-col gap-[10px]">
          <Label
            text="Timers, subscriptions, transitions, and effect cleanup now run as authored React behavior."
            tone="muted"
            size={11}
          />
          <Input
            nodeId="runtime-effects-query"
            value={query}
            width={280}
            leadingIcon="search"
            placeholder="Search runtime work"
            onChange={(event) => setQuery(eventValue(event) ?? "")}
          />
          <div className="flex flex-row gap-[12px] items-center">
            <Label
              nodeId="runtime-effects-elapsed"
              text={`Elapsed interval: ${elapsedSeconds}s`}
              tone="primary"
              weight="semibold"
            />
            <Label
              nodeId="runtime-effects-store-tick"
              text={`Store tick: ${pulse.tick}`}
              tone="secondary"
            />
            <RuntimeModeBadge />
          </div>
          <div className="flex flex-col gap-[4px]">
            <Label
              nodeId="runtime-effects-deferred"
              text={`Deferred query: ${deferredQuery}`}
              tone="secondary"
            />
            <Label
              nodeId="runtime-effects-async"
              text={`Async reducer: ${asyncState.summary}`}
              tone="muted"
              size={11}
            />
            <Label
              nodeId="runtime-effects-store-phase"
              text={`External store phase: ${pulse.phase}`}
              tone="muted"
              size={11}
            />
          </div>
        </div>
      </RuntimeModeContext.Provider>
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
      <MenuBar
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
      <Tabs
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
      <div className="flex flex-row gap-[8px] items-center">
        <Button variant="primary" actionId="dialogue.open" onClick={() => setDialogueOpen(true)}>
          Open Dialogue
        </Button>
        <Button variant="secondary" actionId="sidebar.open" onClick={() => setSidebarOpen(true)}>
          Open Sidebar
        </Button>
        <Button variant="ghost" actionId="toast.add" onClick={() => pushToast("Changes saved", "success")}>
          Add Toast
        </Button>
      </div>
      <div className="flex flex-row gap-[8px] items-center">
        <DropdownMenu
          nodeId="dropdown-menu"
          triggerLabel="Open"
          entries={menuEntries}
          onCommand={(event) => setLastAction(event.metadata?.item_label ?? "Dropdown action")}
        />
        <OpenWith
          nodeId="open-with"
          selectedItemId="codex"
          entries={[{ itemId: "codex", label: "Codex", leadingIcon: "codex" }]}
          onCommand={(event) => setLastAction(event.metadata?.item_label ?? "Open with")}
        />
        <Tooltip triggerLabel="Hover this trigger" text="Tooltip content example" placement="top" />
      </div>
      <Popover
        nodeId="settings-popover"
        open={popoverOpen}
        triggerLabel="Open Popover"
        width={280}
        side="bottom"
        align="center"
        onOpen={() => setPopoverOpen(true)}
        onClose={() => setPopoverOpen(false)}
      >
        <Label text="Interactive popovers can host contract children." tone="muted" />
      </Popover>
      <ContextMenu
        nodeId="context-menu"
        width={220}
        regionWidth={420}
        regionHeight={132}
        entries={menuEntries}
        onCommand={(event) => setLastAction(event.metadata?.item_label ?? "Context menu action")}
      >
        <div className="flex flex-col gap-[6px]">
          <Label text="Scene View" tone="muted" size={11} />
          <Label text="Right-click this region" tone="primary" weight="semibold" size={18} />
        </div>
      </ContextMenu>
      <Collapsible
        nodeId="details"
        title="Transform"
        open={detailsOpen}
        leadingIcon="move-3d"
        trailingIcon="ellipsis"
        onToggle={(event) => setDetailsOpen(Boolean(eventValue(event)))}
      >
        <Label text="Position" tone="secondary" />
        <Label text="Rotation" tone="secondary" />
        <Label text="Scale" tone="secondary" />
      </Collapsible>
      <DialogueModal
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
        <Label text="The modal body is authored from JSX children." tone="muted" />
      </DialogueModal>
      <Sidebar
        nodeId="sidebar"
        title="Workspace"
        side="right"
        width={280}
        open={sidebarOpen}
        onClose={() => setSidebarOpen(false)}
      >
        <Button variant="primary" leadingIcon="file-plus">New Draft</Button>
        <Button variant="ghost" leadingIcon="search">Command Search</Button>
        <Button variant="secondary" onClick={() => setSidebarOpen(false)}>Close Sidebar</Button>
      </Sidebar>
      <ToastViewport
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
      <Label text={`Last action: ${lastAction}`} tone="muted" size={11} />
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
      <Toolbar anchor="top_center" offsetY={10}>
        <Button iconOnly={true} leadingIcon="move" variant="ghost" actionId="tool.move" />
        <Button iconOnly={true} leadingIcon="rotate-ccw" variant="ghost" actionId="tool.rotate" />
        <Button iconOnly={true} leadingIcon="maximize-2" variant="ghost" actionId="tool.scale" />
      </Toolbar>
      <IconToolbar
        nodeId="icon-toolbar"
        selectedItemId={selectedTool}
        items={toolbarItems}
        onSelect={(event) => setSelectedTool(event.metadata?.item_id ?? selectedTool)}
      />
      <CollabCursor name="Lisa Chen" x={96} y={34} color="#39bdf8" size={32} />
      <FileTree
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
      <Hierarchy
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
      <DragBoard
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
      <AudioPlayback
        nodeId="audio-playback"
        playbackState={audioState}
        onToggle={() => setAudioState((state) => (state === "Paused" ? "Playing" : "Paused"))}
      >
        <Button iconOnly={true} leadingIcon="download" variant="ghost" size="sm" />
        <Button iconOnly={true} leadingIcon="ellipsis" variant="ghost" size="sm" />
      </AudioPlayback>
      <ImageTile
        nodeId="image-tile"
        source="builtin:showcase-image"
        size="md"
        selected={tileSelected}
        playbackState={tilePlayback}
        onClick={() => setTileSelected((selected) => !selected)}
        onToggle={() => setTilePlayback((state) => (state === "Paused" ? "Playing" : "Paused"))}
      >
        <Label text="Ambient Preview" tone="primary" weight="semibold" />
        <Label text={`Playback: ${tilePlayback}`} tone="muted" size={11} />
      </ImageTile>
      <Command
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
      <Card paddingX={16} paddingY={16}>
        <div className="flex flex-col gap-[10px]">
          <Label text="Backgrounds" tone="primary" weight="semibold" />
          <Input nodeId="canva-background-search" value="" placeholder="Search backgrounds" width={320} leadingIcon="search" />
          <div className="flex flex-row gap-[8px] items-center">
            <Button iconOnly={true} leadingIcon="palette" variant="secondary" />
            <Color fill="#9ab5d0" size={32} cornerRadius={8} />
            <Color fill="#f7f6f3" size={32} cornerRadius={8} />
            <Color fill="#080808" size={32} cornerRadius={8} />
            <Button iconOnly={true} leadingIcon="chevron-right" variant="primary" />
          </div>
          <Button variant="secondary" leadingIcon="sparkles" trailingIcon="crown">Magic Background</Button>
        </div>
      </Card>
      <Card paddingX={16} paddingY={16}>
        <div className="flex flex-col gap-[10px]">
          <Label text="Brand Kit" tone="primary" weight="semibold" />
          <Tabs
            nodeId="canva-brand-kit-tabs"
            style="segmented"
            selectedItemId="logos"
            items={[
              { itemId: "logos", label: "Logos" },
              { itemId: "colors", label: "Colors" },
              { itemId: "fonts", label: "Fonts" },
            ]}
          />
          <Swatches />
          <Button variant="secondary" leadingIcon="upload">Upload brand asset</Button>
        </div>
      </Card>
      <Card paddingX={16} paddingY={16}>
        <div className="flex flex-col gap-[10px]">
          <Label text="Edit image" tone="primary" weight="semibold" />
          <div className="flex flex-row gap-[8px] items-center">
            <Button variant="secondary" leadingIcon="image">All</Button>
            <Button variant="ghost" leadingIcon="mouse-pointer-click">Click</Button>
            <Button variant="ghost" leadingIcon="wand-sparkles">Magic</Button>
          </div>
          <Button variant="secondary" leadingIcon="sliders-horizontal">Adjust</Button>
          <Button variant="secondary" leadingIcon="sparkles">BG Remover</Button>
        </div>
      </Card>
      <Card paddingX={16} paddingY={16}>
        <div className="flex flex-col gap-[10px]">
          <Label text="Position" tone="primary" weight="semibold" />
          <Tabs
            nodeId="canva-position-tabs"
            style="segmented"
            selectedItemId="arrange"
            items={[
              { itemId: "arrange", label: "Arrange" },
              { itemId: "layers", label: "Layers" },
            ]}
          />
          <div className="flex flex-row gap-[8px] items-center">
            <NumberInput nodeId="canva-position-width" value={1920} width={90} suffix="px" />
            <NumberInput nodeId="canva-position-height" value={1080} width={90} suffix="px" />
            <NumberInput nodeId="canva-position-rotation" value={0} width={90} suffix="deg" />
          </div>
        </div>
      </Card>
    </Section>
  );
}

function App() {
  return (
    <div id="jsx-runtime-root" className="flex flex-col gap-[12px]">
      <div className="flex flex-row gap-[8px] items-center">
        <Label text="JSX authored egui component library" tone="primary" weight="semibold" size={22} />
        <Button nodeId="header-reload" actionId="reload" variant="secondary" size="sm">
          Runtime session
        </Button>
      </div>
      <Label
        text="Edit this file in the left panel or on disk. Events are dispatched back into V8 and hooks rebuild the contract tree."
        tone="muted"
        size={12}
      />
      <Separator nodeId="intro-separator" />
      <Primitives />
      <Controls />
      <RuntimeEffects />
      <MenusAndSurfaces />
      <ShowcaseExamples />
      <CanvaRecipes />
    </div>
  );
}

log("info", "src/showcase/app.jsx rendered with component parity catalog");
render(<App />);
