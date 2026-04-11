const elementMarker = Symbol.for("egui-component.element");
export const Fragment = Symbol.for("egui-component.fragment");

const contractMetadata = globalThis.__eguiContract ?? {};
const knownFamilies = new Set(contractMetadata.families ?? []);
const familiesWithChildren = new Set(contractMetadata.familiesWithChildren ?? []);
const compactFamilyAliases = Object.fromEntries(
  [...knownFamilies].map((family) => [family.replace(/-/g, ""), family]),
);

const motionPropNames = new Set([
  "initial",
  "animate",
  "transition",
  "exit",
  "whileHover",
  "whileTap",
  "whileFocus",
  "whileInView",
]);

const unsupportedMotionPropNames = new Set([
  "exit",
  "whileHover",
  "whileTap",
  "whileFocus",
  "whileInView",
]);

const motionPropAliases = {
  scaleX: "scale_x",
  scaleY: "scale_y",
  paddingX: "padding_x",
  paddingY: "padding_y",
  cornerRadius: "corner_radius",
};

const supportedMotionValueProps = new Set([
  "opacity",
  "x",
  "y",
  "scale",
  "scale_x",
  "scale_y",
  "rotate",
  "width",
  "height",
  "gap",
  "padding_x",
  "padding_y",
  "corner_radius",
]);

let rootElement = null;
let rootInstance = null;
let isDispatching = false;
let currentHookKey = null;
let hookCursor = 0;

const hookState = new Map();
let eventHandlers = new Map();

const handlerEventKinds = {
  onClick: "clicked",
  onChange: "changed",
  onChanged: "changed",
  onSubmit: "submitted",
  onSubmitted: "submitted",
  onSelect: "selected",
  onSelected: "selected",
  onToggle: "toggled",
  onToggled: "toggled",
  onConfirm: "confirmed",
  onConfirmed: "confirmed",
  onCancel: "cancelled",
  onCancelled: "cancelled",
  onOpen: "opened",
  onOpened: "opened",
  onClose: "closed",
  onClosed: "closed",
  onCommand: "command_invoked",
  onCommandInvoked: "command_invoked",
  onEvent: "*",
};

const familyAliases = {
  box: "sized-box",
  sizedbox: "sized-box",
  dialog: "dialogue-modal",
  dialogue: "dialogue-modal",
  dialogueModal: "dialogue-modal",
  dropdown: "dropdown-menu",
  dropdownMenu: "dropdown-menu",
  filetree: "file-tree",
  imageTile: "image-tile",
  iconToolbar: "icon-toolbar",
  radioGroup: "radio-group",
  toast: "toast-viewport",
};

const propAliases = {
  className: "class",
  nodeId: "node_id",
  actionId: "action_id",
  selectedItemId: "selected_item_id",
  selectedItemIds: "selected_item_ids",
  itemId: "item_id",
  helperText: "helper_text",
  paddingX: "padding_x",
  paddingY: "padding_y",
  offsetX: "offset_x",
  offsetY: "offset_y",
  leadingIcon: "leading_icon",
  trailingIcon: "trailing_icon",
  trailingText: "trailing_text",
  iconOnly: "icon_only",
  confirmLabel: "confirm_label",
  cancelLabel: "cancel_label",
  confirmActionId: "confirm_action_id",
  cancelActionId: "cancel_action_id",
  selected: "selected",
  maxHeight: "max_height",
  filterPlaceholder: "filter_placeholder",
  popupWidth: "popup_width",
  popupMaxHeight: "popup_max_height",
  currentPage: "current_page",
  pageCount: "page_count",
  siblingCount: "sibling_count",
  triggerLabel: "trigger_label",
  triggerVariant: "trigger_variant",
  delayMs: "delay_ms",
  sideOffset: "side_offset",
  regionWidth: "region_width",
  regionHeight: "region_height",
  leftTitle: "left_title",
  rightTitle: "right_title",
  rowHeight: "row_height",
  indentWidth: "indent_width",
  badgeFill: "badge_fill",
  playbackState: "playback_state",
  durationSeconds: "duration_seconds",
  playPauseActionId: "play_pause_action_id",
  imageWidth: "image_width",
  imageHeight: "image_height",
  imageFrame: "image_frame",
  iconSize: "icon_size",
  maxVisible: "max_visible",
  durationSecs: "duration_secs",
  cornerRadius: "corner_radius",
};

export function jsx(type, props, key) {
  const nextProps = props == null ? {} : props;
  if (key === undefined || key === null) {
    return { $$typeof: elementMarker, type, props: nextProps };
  }
  return {
    $$typeof: elementMarker,
    type,
    props: { ...nextProps, key: String(key) },
  };
}

export const jsxs = jsx;
export const jsxDEV = jsx;

export function log(level, message) {
  Deno.core.ops.op_host_log(String(level), String(message));
}

export function useState(initialValue) {
  if (currentHookKey == null) {
    throw new Error("useState() can only be called inside a JSX function component.");
  }

  let hooks = hookState.get(currentHookKey);
  if (hooks == null) {
    hooks = [];
    hookState.set(currentHookKey, hooks);
  }

  const hookKey = currentHookKey;
  const index = hookCursor;
  hookCursor += 1;

  if (index >= hooks.length) {
    hooks.push(typeof initialValue === "function" ? initialValue() : initialValue);
  }

  const setState = (nextValue) => {
    const targetHooks = hookState.get(hookKey);
    const previous = targetHooks[index];
    targetHooks[index] =
      typeof nextValue === "function" ? nextValue(previous) : nextValue;
    if (!isDispatching) {
      rerenderRoot();
    }
  };

  return [hooks[index], setState];
}

export function eventValue(event) {
  const value = event?.value;
  if (value == null || typeof value !== "object") {
    return undefined;
  }
  return value.value;
}

export function render(element) {
  rootElement = element;
  rerenderRoot();
}

globalThis.__eguiDispatchEvents = function dispatchEguiEvents(events) {
  if (!Array.isArray(events)) {
    return;
  }

  isDispatching = true;
  try {
    for (const event of events) {
      dispatchEvent(event);
    }
  } finally {
    isDispatching = false;
  }

  rerenderRoot();
};

function rerenderRoot() {
  if (rootElement == null) {
    throw new Error("render(<... />) must be called before dispatching events.");
  }

  eventHandlers = new Map();
  const context = { seenNodeIds: new Set() };
  const rootDescriptor = describeNode(rootElement, "root", context);
  if (rootDescriptor == null) {
    throw new Error("render() expected a JSX element.");
  }

  const mutations = [];
  rootInstance = reconcileRoot(rootInstance, rootDescriptor, mutations);
  if (mutations.length > 0) {
    Deno.core.ops.op_commit_mutations(
      JSON.stringify({ version: contractMetadata.version ?? 1, mutations }),
    );
  }
}

function reconcileRoot(previous, descriptor, mutations) {
  if (
    previous == null ||
    previous.nodeId !== descriptor.nodeId ||
    previous.family !== descriptor.family
  ) {
    const next = mountInstance(descriptor, null);
    mutations.push({ kind: "replace_root", subtree: materializeInstance(next) });
    appendMountedMotionMutations(next, mutations);
    return next;
  }

  reconcileInstance(previous, descriptor, mutations);
  return previous;
}

function reconcileInstance(instance, descriptor, mutations) {
  if (!sameValue(instance.props, descriptor.props)) {
    instance.props = descriptor.props;
    mutations.push({ kind: "update_node", node: { ...descriptor.props } });
  }

  if (!sameValue(instance.motion, descriptor.motion)) {
    instance.motion = descriptor.motion;
    appendMotionMutation(instance, mutations);
  }

  reconcileChildren(instance, descriptor.children, mutations);
}

function reconcileChildren(parent, descriptors, mutations) {
  const previousChildren = parent.children;
  const previousOrder = previousChildren.map((child) => child.nodeId);
  const previousById = new Map();
  for (const child of previousChildren) {
    previousById.set(child.nodeId, child);
  }

  const nextChildren = [];
  const handledPrevious = new Set();
  descriptors.forEach((descriptor, index) => {
    const existing = previousById.get(descriptor.nodeId);
    if (existing != null && existing.family === descriptor.family) {
      existing.parent = parent;
      reconcileInstance(existing, descriptor, mutations);
      nextChildren.push(existing);
      handledPrevious.add(existing);
      return;
    }

    const next = mountInstance(descriptor, parent);
    nextChildren.push(next);
    if (existing != null) {
      mutations.push({
        kind: "replace_subtree",
        node_id: existing.nodeId,
        subtree: materializeInstance(next),
      });
      appendMountedMotionMutations(next, mutations);
      handledPrevious.add(existing);
    } else {
      mutations.push({
        kind: "insert_subtree",
        parent_id: parent.nodeId,
        index,
        subtree: materializeInstance(next),
      });
      appendMountedMotionMutations(next, mutations);
    }
  });

  for (const child of previousChildren) {
    if (!handledPrevious.has(child)) {
      mutations.push({ kind: "remove_subtree", node_id: child.nodeId });
    }
  }

  parent.children = nextChildren;
  const nextOrder = nextChildren.map((child) => child.nodeId);
  if (!sameStringArray(previousOrder, nextOrder)) {
    mutations.push({
      kind: "set_children",
      node_id: parent.nodeId,
      child_ids: nextOrder,
    });
  }
}

function mountInstance(descriptor, parent) {
  const instance = {
    nodeId: descriptor.nodeId,
    family: descriptor.family,
    props: descriptor.props,
    motion: descriptor.motion,
    parent,
    children: [],
  };
  instance.children = descriptor.children.map((child) => mountInstance(child, instance));
  return instance;
}

function materializeInstance(instance) {
  const node = { ...instance.props };
  if (familiesWithChildren.has(instance.family) && instance.children.length > 0) {
    node.children = instance.children.map(materializeInstance);
  }
  return node;
}

function appendMountedMotionMutations(instance, mutations) {
  if (instance.motion != null) {
    appendMotionMutation(instance, mutations);
  }
  for (const child of instance.children) {
    appendMountedMotionMutations(child, mutations);
  }
}

function appendMotionMutation(instance, mutations) {
  if (instance.motion == null) {
    mutations.push({ kind: "clear_motion", node_id: instance.nodeId });
    return;
  }
  mutations.push({
    kind: "set_motion",
    node_id: instance.nodeId,
    motion: instance.motion,
  });
}

function sameStringArray(a, b) {
  if (a.length !== b.length) {
    return false;
  }
  return a.every((item, index) => item === b[index]);
}

function sameValue(a, b) {
  if (Object.is(a, b)) {
    return true;
  }
  if (Array.isArray(a) || Array.isArray(b)) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) {
      return false;
    }
    return a.every((item, index) => sameValue(item, b[index]));
  }
  if (isPlainObject(a) || isPlainObject(b)) {
    if (!isPlainObject(a) || !isPlainObject(b)) {
      return false;
    }
    const aKeys = Object.keys(a);
    const bKeys = Object.keys(b);
    if (aKeys.length !== bKeys.length) {
      return false;
    }
    return aKeys.every(
      (key) => Object.prototype.hasOwnProperty.call(b, key) && sameValue(a[key], b[key]),
    );
  }
  return false;
}

function dispatchEvent(event) {
  const nodeId = String(event?.node_id ?? "");
  const kind = String(event?.kind ?? "");
  const candidates = [
    `${nodeId}:${kind}`,
    `${nodeId}:*`,
    event?.action_id == null ? null : `action:${event.action_id}:${kind}`,
    event?.action_id == null ? null : `action:${event.action_id}:*`,
  ].filter(Boolean);

  for (const key of candidates) {
    for (const handler of eventHandlers.get(key) ?? []) {
      handler(event, eventValue(event));
    }
  }
}

function registerEventHandler(nodeId, propName, handler) {
  const kind = handlerEventKinds[propName];
  if (kind == null) {
    return false;
  }
  pushHandler(`${nodeId}:${kind}`, handler);
  return true;
}

function pushHandler(key, handler) {
  const handlers = eventHandlers.get(key) ?? [];
  handlers.push(handler);
  eventHandlers.set(key, handlers);
}

function describeNode(value, path, context) {
  if (value == null || value === false || value === true) {
    return null;
  }

  if (Array.isArray(value)) {
    const children = describeChildren(value, path, context);
    return children.length === 0
      ? null
      : hostDescriptor(
          "column",
          path,
          { family: "column", node_id: path, gap: 8 },
          children,
          null,
          context,
        );
  }

  if (typeof value === "string" || typeof value === "number") {
    const text = String(value).trim();
    return text === ""
      ? null
      : hostDescriptor(
          "label",
          path,
          { family: "label", node_id: path, text },
          [],
          null,
          context,
        );
  }

  if (!isElement(value)) {
    throw new Error(`Cannot render ${Object.prototype.toString.call(value)} as JSX.`);
  }

  if (typeof value.type === "function") {
    return describeFunctionComponent(value, path, context);
  }

  if (value.type === Fragment) {
    const children = describeChildren(value.props?.children, path, context);
    return children.length === 0
      ? null
      : hostDescriptor(
          "column",
          path,
          { family: "column", node_id: path, gap: 8 },
          children,
          null,
          context,
        );
  }

  const family = normalizeFamilyName(value.type);
  const props = value.props ?? {};
  const motion = props.__motion_host === true ? normalizeMotionSpec(props) : null;
  const node = {
    family,
    node_id: String(props.nodeId ?? props.node_id ?? props.id ?? path),
  };

  for (const [rawName, rawValue] of Object.entries(props)) {
    if (
      rawName === "children" ||
      rawName === "key" ||
      rawName === "id" ||
      rawName === "__motion_host"
    ) {
      continue;
    }
    if (props.__motion_host === true && motionPropNames.has(rawName)) {
      continue;
    }
    if (rawValue === undefined) {
      continue;
    }
    if (typeof rawValue === "function") {
      if (!registerEventHandler(node.node_id, rawName, rawValue)) {
        throw new Error(`Unsupported function prop "${rawName}" on ${family}.`);
      }
      continue;
    }

    const propName = normalizePropName(rawName);
    if (propName === "node_id") {
      continue;
    }
    node[propName] = normalizePropValue(family, propName, rawValue);
  }

  const childNodes = familiesWithChildren.has(family)
    ? describeChildren(props.children, `${node.node_id}.child`, context)
    : [];

  applyFamilyDefaults(node, props.children);
  return hostDescriptor(family, node.node_id, node, childNodes, motion, context);
}

function hostDescriptor(family, nodeId, props, children, motion, context) {
  if (nodeId === "") {
    throw new Error("Contract nodes must have a stable node_id.");
  }
  if (context.seenNodeIds.has(nodeId)) {
    throw new Error(`Duplicate contract node_id "${nodeId}".`);
  }
  context.seenNodeIds.add(nodeId);
  return { family, nodeId, props, children, motion };
}

function describeFunctionComponent(value, path, context) {
  const componentName = value.type.displayName || value.type.name || "Component";
  const hookKey = `${path}:${componentName}`;
  const previousHookKey = currentHookKey;
  const previousHookCursor = hookCursor;
  currentHookKey = hookKey;
  hookCursor = 0;
  try {
    return describeNode(value.type({ ...(value.props ?? {}) }), path, context);
  } finally {
    currentHookKey = previousHookKey;
    hookCursor = previousHookCursor;
  }
}

function describeChildren(value, path, context) {
  const result = [];
  let index = 0;
  for (const child of flattenChildren(value)) {
    if (isBlankText(child)) {
      continue;
    }

    if (isElement(child) && child.type === Fragment) {
      const childPath = childPathFor(path, child, index);
      result.push(...describeChildren(child.props?.children, childPath, context));
      index += 1;
      continue;
    }

    const node = describeNode(child, childPathFor(path, child, index), context);
    if (node != null) {
      result.push(node);
      index += 1;
    }
  }
  return result;
}

function childPathFor(path, child, index) {
  const key = isElement(child) ? child.props?.key : null;
  return key == null ? `${path}.${index}` : `${path}.${sanitizePathSegment(key)}`;
}

function sanitizePathSegment(value) {
  return String(value).replace(/[^A-Za-z0-9_-]/g, "_");
}

function flattenChildren(value) {
  if (value == null || value === false || value === true) {
    return [];
  }
  if (!Array.isArray(value)) {
    return [value];
  }
  const result = [];
  for (const child of value) {
    result.push(...flattenChildren(child));
  }
  return result;
}

function applyFamilyDefaults(node, children) {
  if (node.family === "label" && node.text == null) {
    node.text = textFromChildren(children);
  }
  if (node.family === "button" && node.label == null) {
    node.label = textFromChildren(children) || "Button";
  }
  if (node.family === "input" && node.value == null) {
    node.value = "";
  }
  if (node.family === "field") {
    node.label ??= "Field";
    node.value ??= "";
  }
  if (node.family === "number-input" && node.value == null) {
    node.value = 0;
  }
  if ((node.family === "checkbox" || node.family === "switch") && node.value == null) {
    node.value = false;
  }
  if (node.family === "progress" && node.value == null) {
    node.value = 0;
  }
  if (node.family === "collapsible") {
    node.title ??= "Section";
  }
  if (node.family === "dialogue-modal") {
    node.title ??= "Dialog";
  }
  if (node.family === "color") {
    node.fill ??= "#ffffff";
  }
  if (node.family === "icon") {
    node.name ??= "sparkles";
  }
  if (node.family === "image" || node.family === "image-tile") {
    node.source ??= "builtin:showcase-image";
  }
  if (node.family === "twemoji") {
    node.emoji ??= "🙂";
  }
  if (node.family === "kbd") {
    node.text ??= textFromChildren(children) || "Ctrl";
  }
  if (node.family === "slider" && node.value == null) {
    node.value = 0;
  }
  if (node.family === "radio" && node.value == null) {
    node.value = false;
  }
  if (node.family === "emoji-selector") {
    node.value ??= "🙂";
  }
  if (node.family === "tooltip") {
    node.trigger_label ??= "Hover this trigger";
    node.text ??= textFromChildren(children) || "Tooltip content";
  }
  if (node.family === "popover") {
    node.open ??= false;
  }
  if (node.family === "dropdown-menu") {
    node.trigger_label ??= "Open";
    node.entries ??= [];
  }
  if (node.family === "context-menu" || node.family === "open-with") {
    node.entries ??= [];
  }
  if (node.family === "collab-cursor") {
    node.name ??= "Collaborator";
  }
  if (node.family === "audio-playback") {
    node.playback_state ??= "Paused";
  }
  if (node.family === "command") {
    node.query ??= "";
    node.items ??= [];
  }
}

function normalizeMotionSpec(props) {
  for (const name of unsupportedMotionPropNames) {
    if (props[name] !== undefined && props[name] !== null) {
      throw new Error(
        `Motion prop "${name}" is not supported by the retained JSX host yet.`,
      );
    }
  }

  const initial =
    props.initial === undefined || props.initial === null || props.initial === false
      ? null
      : normalizeMotionValues("initial", props.initial);
  const animate =
    props.animate === undefined || props.animate === null || props.animate === false
      ? null
      : normalizeMotionValues("animate", props.animate);

  if (initial == null && animate == null) {
    return null;
  }

  return {
    initial,
    animate: animate ?? initial ?? {},
    transition: normalizeMotionTransition(props.transition),
  };
}

function normalizeMotionValues(propName, values) {
  if (!isPlainObject(values) || isElement(values)) {
    throw new Error(`Motion prop "${propName}" must be an object of numeric values.`);
  }

  const out = {};
  for (const [rawName, rawValue] of Object.entries(values)) {
    if (rawValue === undefined) {
      continue;
    }
    if (typeof rawValue !== "number" || !Number.isFinite(rawValue)) {
      throw new Error(`Motion value "${rawName}" must be a finite number.`);
    }

    const name = normalizeMotionValueName(rawName);
    if (!supportedMotionValueProps.has(name)) {
      throw new Error(
        `Unsupported motion value "${rawName}". Supported values are ${[
          ...supportedMotionValueProps,
        ].join(", ")}.`,
      );
    }
    out[name] = rawValue;
  }

  return Object.keys(out).length === 0 ? null : out;
}

function normalizeMotionTransition(transition) {
  if (transition === undefined || transition === null) {
    return {};
  }
  if (!isPlainObject(transition) || isElement(transition)) {
    throw new Error('Motion prop "transition" must be an object.');
  }
  if (
    transition.type !== undefined &&
    transition.type !== null &&
    transition.type !== "tween"
  ) {
    throw new Error('Only tween motion transitions are supported by the retained JSX host.');
  }

  const out = {};
  if (transition.duration !== undefined) {
    out.duration = normalizeMotionTransitionNumber("duration", transition.duration);
  }
  if (transition.delay !== undefined) {
    out.delay = normalizeMotionTransitionNumber("delay", transition.delay);
  }
  if (transition.ease !== undefined && transition.ease !== null) {
    out.ease = normalizeMotionEase(transition.ease);
  }
  return out;
}

function normalizeMotionTransitionNumber(name, value) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    throw new Error(`Motion transition "${name}" must be a non-negative finite number.`);
  }
  return value;
}

function normalizeMotionEase(value) {
  if (typeof value !== "string") {
    throw new Error('Motion transition "ease" must be a string.');
  }
  const ease = camelToSnake(value);
  if (
    ease !== "linear" &&
    ease !== "ease_in" &&
    ease !== "ease_out" &&
    ease !== "ease_in_out"
  ) {
    throw new Error(
      'Motion transition "ease" must be one of linear, easeIn, easeOut, or easeInOut.',
    );
  }
  return ease;
}

function normalizeMotionValueName(name) {
  return motionPropAliases[name] ?? camelToSnake(name);
}

function normalizeFamilyName(type) {
  if (typeof type !== "string") {
    throw new Error(`Unsupported JSX element type ${String(type)}.`);
  }
  const typeName = String(type);
  const kebabName = camelToKebab(typeName);
  const compactName = kebabName.replace(/-/g, "");
  const family = knownFamilies.has(typeName)
    ? typeName
    : familyAliases[typeName] ??
      familyAliases[kebabName] ??
      compactFamilyAliases[typeName.toLowerCase()] ??
      compactFamilyAliases[compactName] ??
      kebabName;
  if (knownFamilies.size > 0 && !knownFamilies.has(family)) {
    throw new Error(`Unknown contract family "${family}" from JSX element "${typeName}".`);
  }
  return family;
}

function normalizePropName(name) {
  return propAliases[name] ?? camelToSnake(name);
}

function normalizePropValue(family, propName, value) {
  if (propName === "entries" && Array.isArray(value)) {
    return value.map(normalizeMenuEntry);
  }

  if (Array.isArray(value)) {
    return value.map((item) => normalizePropValue(family, propName, item));
  }

  if (isPlainObject(value) && !isElement(value)) {
    const out = {};
    for (const [name, childValue] of Object.entries(value)) {
      if (childValue === undefined) {
        continue;
      }
      const normalizedName = normalizePropName(name);
      out[normalizedName] = normalizePropValue(family, normalizedName, childValue);
    }
    return out;
  }

  if (typeof value === "string") {
    const enumStyle = enumStyleForProp(family, propName);
    if (enumStyle === "pascal") {
      return toPascalEnum(value);
    }
    if (enumStyle === "snake") {
      return camelToSnake(value);
    }
  }

  return value;
}

function normalizeMenuEntry(entry) {
  if (entry === "-" || entry === "separator") {
    return { kind: "separator" };
  }
  if (!isPlainObject(entry)) {
    throw new Error("Menu entries must be objects, '-' or 'separator'.");
  }

  const out = {};
  for (const [name, value] of Object.entries(entry)) {
    const normalizedName = normalizePropName(name);
    out[normalizedName] = normalizePropValue("menu-entry", normalizedName, value);
  }
  out.kind ??= Array.isArray(out.entries) ? "submenu" : "action";
  out.kind = camelToSnake(out.kind);
  return out;
}

function enumStyleForProp(family, propName) {
  if (family === "row" || family === "column") {
    if (propName === "justify" || propName === "align") {
      return "snake";
    }
  }
  if (family === "tabs" && propName === "style") {
    return "snake";
  }
  if (family === "toast-viewport" && (propName === "placement" || propName === "intent")) {
    return "snake";
  }
  if (family === "menu-entry" && propName === "kind") {
    return "snake";
  }

  if (
    propName === "variant" ||
    propName === "size" ||
    propName === "tone" ||
    propName === "weight" ||
    propName === "axis" ||
    propName === "side" ||
    propName === "icon_style" ||
    propName === "kind" ||
    propName === "playback_state" ||
    propName === "trigger_variant" ||
    propName === "placement" ||
    propName === "intent" ||
    propName === "style" ||
    propName === "align" ||
    propName === "region"
  ) {
    return "pascal";
  }

  return null;
}

function textFromChildren(children) {
  return flattenChildren(children)
    .filter((child) => typeof child === "string" || typeof child === "number")
    .map((child) => String(child).trim())
    .filter(Boolean)
    .join(" ");
}

function isElement(value) {
  return isPlainObject(value) && value.$$typeof === elementMarker;
}

function isPlainObject(value) {
  return typeof value === "object" && value != null && !Array.isArray(value);
}

function isBlankText(value) {
  return typeof value === "string" && value.trim() === "";
}

function toPascalEnum(value) {
  const text = String(value);
  if (text === "sm") {
    return "Sm";
  }
  if (text === "md") {
    return "Md";
  }
  return text
    .split(/[-_\s]+/g)
    .filter(Boolean)
    .map((part) => part.slice(0, 1).toUpperCase() + part.slice(1))
    .join("");
}

function camelToKebab(value) {
  return String(value)
    .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
    .replace(/_/g, "-")
    .toLowerCase();
}

function camelToSnake(value) {
  return String(value)
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .replace(/-/g, "_")
    .toLowerCase();
}
