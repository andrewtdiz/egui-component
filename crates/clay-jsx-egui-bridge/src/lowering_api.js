const sharedLayoutPropNames = new Set([
  "display",
  "direction",
  "grow",
  "shrink",
  "basis",
  "width",
  "height",
  "min_width",
  "min_height",
  "max_width",
  "max_height",
  "padding",
  "margin",
  "align",
  "justify",
  "wrap",
  "columns",
  "rows",
  "col_span",
  "row_span",
  "overflow_x",
  "overflow_y",
]);

const contractLengthPropNames = new Set([
  "basis",
  "width",
  "height",
  "min_width",
  "min_height",
  "max_width",
  "max_height",
]);

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

const handlerEventKinds = {
  onClick: "clicked",
  onInput: "changed",
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

const htmlHostElements = new Set([
  "a",
  "article",
  "aside",
  "button",
  "code",
  "dialog",
  "div",
  "em",
  "footer",
  "form",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "header",
  "hr",
  "img",
  "input",
  "kbd",
  "label",
  "li",
  "main",
  "nav",
  "ol",
  "p",
  "pre",
  "progress",
  "section",
  "small",
  "span",
  "strong",
  "textarea",
  "ul",
]);

const htmlContainerElements = new Set([
  "article",
  "aside",
  "div",
  "footer",
  "form",
  "header",
  "li",
  "main",
  "nav",
  "ol",
  "section",
  "ul",
]);

const htmlTextElements = new Set([
  "a",
  "code",
  "em",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "p",
  "pre",
  "small",
  "span",
  "strong",
]);

const htmlElementFamilies = {
  button: "button",
  dialog: "dialogue-modal",
  hr: "separator",
  img: "image",
  kbd: "kbd",
  label: "label",
  progress: "progress",
  textarea: "input",
};

const familyAliases = {
  box: "sized-box",
  sizedbox: "sized-box",
  img: "image",
  hr: "separator",
  section: "column",
  main: "column",
  header: "column",
  footer: "column",
  nav: "column",
  article: "column",
  aside: "column",
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
  classNames: "class",
  classList: "class_list",
  checked: "value",
  defaultChecked: "value",
  htmlFor: "for",
  src: "source",
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

const textBearingFamilies = new Set(["button", "kbd", "label", "tooltip"]);

export function createContractMetadataView(contractMetadata = {}) {
  const knownFamilies = new Set(contractMetadata.families ?? []);
  return {
    version: Number(contractMetadata.version ?? 1),
    knownFamilies,
    familiesWithChildren: new Set(contractMetadata.familiesWithChildren ?? []),
    identitySensitiveFamilies: new Set(contractMetadata.identitySensitiveFamilies ?? []),
    fallbackNodeIdFamilies: new Set(contractMetadata.fallbackNodeIdFamilies ?? []),
    familyPropNames: new Map(
      Object.entries(contractMetadata.familyProps ?? {}).map(([family, props]) => [
        family,
        new Set(Array.isArray(props) ? props : []),
      ]),
    ),
    compactFamilyAliases: Object.fromEntries(
      [...knownFamilies].map((family) => [family.replace(/-/g, ""), family]),
    ),
  };
}

export function familyUsesTextContent(family) {
  return textBearingFamilies.has(family);
}

export function familyIsIdentitySensitive(family, metadataView) {
  return metadataView.identitySensitiveFamilies.has(family);
}

export function familyAllowsFallbackNodeId(family, metadataView) {
  return metadataView.fallbackNodeIdFamilies.has(family);
}

export function normalizedFamilyForType(type, props, metadataView) {
  return normalizeFamilyName(type, props ?? {}, metadataView, []);
}

export function lowerCommittedRoot(children, metadataView) {
  const state = {
    handlers: new Map(),
    metadataView,
    seenNodeIds: new Set(),
    warnings: [],
  };
  const loweredChildren = lowerChildren(children, "root", state);
  if (loweredChildren.length === 0) {
    return { root: null, handlers: state.handlers, warnings: state.warnings };
  }
  if (loweredChildren.length === 1) {
    return {
      root: loweredChildren[0],
      handlers: state.handlers,
      warnings: state.warnings,
    };
  }
  return {
    root: hostDescriptor(
      "column",
      "root",
      { family: "column", node_id: "root", gap: 8 },
      loweredChildren,
      null,
      state,
    ),
    handlers: state.handlers,
    warnings: state.warnings,
  };
}

function lowerChildren(children, path, state) {
  const result = [];
  let index = 0;
  for (const child of children ?? []) {
    const node = lowerChild(child, childPathFor(path, child, index), state);
    if (node != null) {
      result.push(node);
      index += 1;
    }
  }
  return result;
}

function lowerChild(node, path, state) {
  if (node == null || node.hidden === true) {
    return null;
  }

  if (isTextNode(node)) {
    const text = String(node.text).trim();
    if (text === "") {
      return null;
    }
    return hostDescriptor(
      "label",
      path,
      { family: "label", node_id: path, text },
      [],
      null,
      state,
    );
  }

  if (!isHostNode(node)) {
    throw new Error(`Cannot lower committed node kind ${String(node?.kind)}.`);
  }

  const props = node.props ?? {};
  const family = normalizeFamilyName(node.type, props, state.metadataView, state.warnings);
  const motion = props.__motion_host === true ? normalizeMotionSpec(props) : null;
  const explicitNodeId = explicitNodeIdFromProps(props);
  const normalizedNode = {
    family,
    node_id: resolveNodeId(family, explicitNodeId, path, state.metadataView),
  };
  const handlerEntries = [];

  for (const [rawName, rawValue] of Object.entries(props)) {
    if (
      rawName === "children" ||
      rawName === "key" ||
      rawName === "id" ||
      rawName === "type" ||
      rawName === "role" ||
      rawName === "data-slot" ||
      rawName === "dataSlot" ||
      rawName === "aria-label" ||
      rawName === "ariaLabel" ||
      rawName === "aria-hidden" ||
      rawName === "ariaHidden" ||
      rawName === "aria-busy" ||
      rawName === "ariaBusy" ||
      rawName === "__motion_host" ||
      rawName === "__eguiKey"
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
      handlerEntries.push([rawName, rawValue]);
      continue;
    }

    const propName = normalizePropName(rawName);
    if (propName === "node_id") {
      continue;
    }
    if (propName === "layout") {
      normalizedNode.layout = mergeLayoutProps(
        normalizedNode.layout,
        normalizeLayoutObject(rawValue),
      );
      continue;
    }
    if (shouldHoistPropToLayout(family, propName, state.metadataView)) {
      normalizedNode.layout = mergeLayoutProps(normalizedNode.layout, {
        [propName]: normalizeLayoutPropValue(propName, rawValue),
      });
      continue;
    }
    normalizedNode[propName] = normalizePropValue(family, propName, rawValue);
  }

  const childNodes = state.metadataView.familiesWithChildren.has(family)
    ? lowerChildren(node.children, `${normalizedNode.node_id}.child`, state)
    : [];

  applyFamilyDefaults(normalizedNode, props.children);
  registerHandlers(
    normalizedNode.node_id,
    normalizedNode.action_id == null ? null : String(normalizedNode.action_id),
    handlerEntries,
    state.handlers,
  );

  return hostDescriptor(
    family,
    normalizedNode.node_id,
    normalizedNode,
    childNodes,
    motion,
    state,
  );
}

function hostDescriptor(family, nodeId, props, children, motion, state) {
  if (nodeId === "") {
    throw new Error("Contract nodes must have a stable node_id.");
  }
  if (state.seenNodeIds.has(nodeId)) {
    throw new Error(`Duplicate contract node_id "${nodeId}".`);
  }
  state.seenNodeIds.add(nodeId);
  return { family, nodeId, props, children, motion };
}

function explicitNodeIdFromProps(props) {
  for (const name of ["nodeId", "node_id", "id"]) {
    if (!Object.prototype.hasOwnProperty.call(props ?? {}, name)) {
      continue;
    }
    const value = props?.[name];
    if (value == null) {
      continue;
    }
    const nodeId = String(value).trim();
    if (nodeId === "") {
      throw new Error("Contract nodes must not set an empty node_id.");
    }
    return nodeId;
  }
  return null;
}

function resolveNodeId(family, explicitNodeId, path, metadataView) {
  if (explicitNodeId != null) {
    return explicitNodeId;
  }
  if (familyAllowsFallbackNodeId(family, metadataView)) {
    return path;
  }
  if (familyIsIdentitySensitive(family, metadataView)) {
    // Focus, popup state, selection, drag state, and similar egui-local memory
    // are only safe when the authored tree supplies a stable identity explicitly.
    throw new Error(
      `Contract family "${family}" requires an explicit stable node_id. Fallback ids are only allowed for stateless layout/text sugar.`,
    );
  }
  throw new Error(
    `Contract family "${family}" requires an explicit node_id. Fallback ids are only allowed for stateless layout/text sugar.`,
  );
}

function registerHandlers(nodeId, actionId, handlers, table) {
  for (const [propName, handler] of handlers) {
    const kind = handlerEventKinds[propName];
    if (kind == null) {
      throw new Error(`Unsupported function prop "${propName}".`);
    }
    pushHandler(table, `${nodeId}:${kind}`, handler);
    if (actionId != null && actionId !== "") {
      pushHandler(table, `action:${actionId}:${kind}`, handler);
    }
  }
}

function pushHandler(table, key, handler) {
  const handlers = table.get(key) ?? [];
  handlers.push(handler);
  table.set(key, handlers);
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
  if (!isPlainObject(values) || isReactElement(values)) {
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
  if (!isPlainObject(transition) || isReactElement(transition)) {
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

function normalizeFamilyName(type, props = {}, metadataView, warnings) {
  if (typeof type !== "string") {
    throw new Error(`Unsupported JSX element type ${String(type)}.`);
  }
  const typeName = String(type);
  const nativeFamily = nativeFamilyForElement(typeName, props, metadataView);
  if (nativeFamily != null) {
    return nativeFamily;
  }

  const kebabName = camelToKebab(typeName);
  const compactName = kebabName.replace(/-/g, "");
  const family = metadataView.knownFamilies.has(typeName)
    ? typeName
    : familyAliases[typeName] ??
      familyAliases[kebabName] ??
      metadataView.compactFamilyAliases[typeName.toLowerCase()] ??
      metadataView.compactFamilyAliases[compactName] ??
      kebabName;
  if (metadataView.knownFamilies.size > 0 && !metadataView.knownFamilies.has(family)) {
    throw new Error(`Unknown contract family "${family}" from JSX element "${typeName}".`);
  }
  warnDeprecatedHostElement(typeName, family, warnings);
  return family;
}

function nativeFamilyForElement(typeName, props, metadataView) {
  if (!htmlHostElements.has(typeName)) {
    return null;
  }

  const slotFamily = nativeSlotFamily(props, metadataView);
  if (slotFamily != null) {
    return slotFamily;
  }

  if (typeName === "input") {
    return nativeInputFamily(props);
  }

  if (htmlContainerElements.has(typeName)) {
    return nativeContainerFamily(props);
  }

  if (htmlTextElements.has(typeName)) {
    return "label";
  }

  return htmlElementFamilies[typeName] ?? null;
}

function nativeInputFamily(props) {
  if (String(props?.role ?? "").toLowerCase() === "switch") {
    return "switch";
  }

  const inputType = String(props?.type ?? "text").toLowerCase();
  if (inputType === "checkbox") {
    return "checkbox";
  }
  if (inputType === "radio") {
    return "radio";
  }
  if (inputType === "range") {
    return "slider";
  }
  if (inputType === "number") {
    return "number-input";
  }
  return "input";
}

function warnDeprecatedHostElement(typeName, family, warnings) {
  if (htmlHostElements.has(typeName)) {
    return;
  }
  warnings.push(
    `Deprecated JSX host element <${typeName}> lowered to contract family "${family}". Use an HTML tag with data-slot="${family}" instead.`,
  );
}

function nativeContainerFamily(props) {
  const tokens = classTokens(props);
  if (tokens.has("flex-col")) {
    return "column";
  }
  if (tokens.has("flex") || tokens.has("flex-row")) {
    return "row";
  }
  return "column";
}

function classTokens(props) {
  const parts = [];
  for (const name of ["class", "className", "classNames"]) {
    const value = props?.[name];
    if (typeof value === "string") {
      parts.push(value);
    }
  }
  if (Array.isArray(props?.classList)) {
    for (const value of props.classList) {
      if (typeof value === "string") {
        parts.push(value);
      }
    }
  }
  return new Set(parts.flatMap((value) => value.split(/\s+/)).filter(Boolean));
}

function nativeSlotFamily(props, metadataView) {
  const rawSlotName = props?.["data-slot"] ?? props?.dataSlot;
  if (rawSlotName == null) {
    return null;
  }

  const slotName = String(rawSlotName).trim();
  if (slotName === "") {
    return null;
  }

  const family = normalizeSlotFamilyName(slotName, metadataView);
  if (family == null) {
    throw new Error(`Unknown contract family "${slotName}" from data-slot.`);
  }
  return family;
}

function normalizeSlotFamilyName(slotName, metadataView) {
  const kebabName = camelToKebab(slotName);
  const compactName = kebabName.replace(/-/g, "");
  const family = metadataView.knownFamilies.has(slotName)
    ? slotName
    : familyAliases[slotName] ??
      familyAliases[kebabName] ??
      metadataView.compactFamilyAliases[slotName.toLowerCase()] ??
      metadataView.compactFamilyAliases[compactName] ??
      kebabName;
  if (metadataView.knownFamilies.size > 0 && !metadataView.knownFamilies.has(family)) {
    return null;
  }
  return family;
}

function normalizePropName(name) {
  return propAliases[name] ?? camelToSnake(name);
}

function shouldHoistPropToLayout(family, propName, metadataView) {
  return sharedLayoutPropNames.has(propName) && !familyOwnsProp(family, propName, metadataView);
}

function familyOwnsProp(family, propName, metadataView) {
  return metadataView.familyPropNames.get(family)?.has(propName) === true;
}

function mergeLayoutProps(previousLayout, nextLayout) {
  if (!isPlainObject(nextLayout) || isReactElement(nextLayout)) {
    return previousLayout;
  }
  return { ...(isPlainObject(previousLayout) ? previousLayout : {}), ...nextLayout };
}

function normalizeLayoutObject(value) {
  if (!isPlainObject(value) || isReactElement(value)) {
    throw new Error('Shared "layout" props must be plain objects.');
  }
  const out = {};
  for (const [rawName, rawValue] of Object.entries(value)) {
    if (rawValue === undefined) {
      continue;
    }
    const propName = normalizePropName(rawName);
    out[propName] = normalizeLayoutPropValue(propName, rawValue);
  }
  return out;
}

function normalizeLayoutPropValue(propName, value) {
  const normalized = normalizePropValue("layout", propName, value);
  if (contractLengthPropNames.has(propName)) {
    return normalizeContractLength(propName, normalized);
  }
  if (
    propName === "display" ||
    propName === "direction" ||
    propName === "overflow_x" ||
    propName === "overflow_y"
  ) {
    return camelToKebab(normalized);
  }
  return normalized;
}

function normalizeContractLength(propName, value) {
  if (value == null) {
    return value;
  }
  if (typeof value === "number") {
    if (!Number.isFinite(value)) {
      throw new Error(`Layout prop "${propName}" must be a finite number.`);
    }
    return { kind: "px", value };
  }
  if (typeof value === "string") {
    const text = value.trim();
    if (text === "auto") {
      return { kind: "auto" };
    }
    if (/^-?\d+(?:\.\d+)?%$/.test(text)) {
      return { kind: "percent", value: Number(text.slice(0, -1)) / 100 };
    }
    if (/^-?\d+(?:\.\d+)?$/.test(text)) {
      return { kind: "px", value: Number(text) };
    }
    throw new Error(`Layout prop "${propName}" must be a number, "auto", or a percentage string.`);
  }
  if (isPlainObject(value) && typeof value.kind === "string") {
    return value;
  }
  throw new Error(`Layout prop "${propName}" must be a contract length value.`);
}

function normalizePropValue(family, propName, value) {
  if (propName === "entries" && Array.isArray(value)) {
    return value.map(normalizeMenuEntry);
  }

  if (Array.isArray(value)) {
    return value.map((item) => normalizePropValue(family, propName, item));
  }

  if (isPlainObject(value) && !isReactElement(value)) {
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
  if (!isPlainObject(entry) || isReactElement(entry)) {
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
  if ((family === "row" || family === "column") && (propName === "justify" || propName === "align")) {
    return "snake";
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

function childPathFor(path, node, index) {
  const key = isHostNode(node) ? node.props?.__eguiKey : null;
  return key == null ? `${path}.${index}` : `${path}.${sanitizePathSegment(key)}`;
}

function sanitizePathSegment(value) {
  return String(value).replace(/[^A-Za-z0-9_-]/g, "_");
}

function textFromChildren(children) {
  return flattenAuthorChildren(children)
    .filter((child) => typeof child === "string" || typeof child === "number")
    .map((child) => String(child).trim())
    .filter(Boolean)
    .join(" ");
}

function flattenAuthorChildren(value) {
  if (value == null || value === false || value === true) {
    return [];
  }
  if (!Array.isArray(value)) {
    return [value];
  }
  const result = [];
  for (const child of value) {
    result.push(...flattenAuthorChildren(child));
  }
  return result;
}

function isHostNode(value) {
  return value?.kind === "host";
}

function isTextNode(value) {
  return value?.kind === "text";
}

function isPlainObject(value) {
  return typeof value === "object" && value != null && !Array.isArray(value);
}

function isReactElement(value) {
  return isPlainObject(value) && Object.prototype.hasOwnProperty.call(value, "$$typeof");
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
