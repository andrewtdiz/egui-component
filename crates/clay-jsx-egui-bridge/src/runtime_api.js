import React from "react";
import Reconciler from "clay-internal:/react-reconciler";

import {
  createContractMetadataView,
  familyUsesTextContent,
  lowerCommittedRoot,
  normalizedFamilyForType,
} from "clay-internal:/egui-lowering";

const contractMetadata = globalThis.__eguiContract ?? {};
const metadataView = createContractMetadataView(contractMetadata);
const loggedWarnings = new Set();

let currentUpdatePriority = 0;
let pendingRuntimeError = null;
let rootRendered = false;
let allowEmptyCommit = false;

delete globalThis.__eguiHotReloadState;
globalThis.__eguiCaptureHotReloadState = () => ({ hook_state: {} });

export const createContext = React.createContext;
export const startTransition = React.startTransition;
export const useContext = React.useContext;
export const useDeferredValue = React.useDeferredValue;
export const useEffect = React.useEffect;
export const useReducer = React.useReducer;
export const useRef = React.useRef;
export const useState = React.useState;
export const useSyncExternalStore = React.useSyncExternalStore;

export function log(level, message) {
  Deno.core.ops.op_host_log(String(level), String(message));
}

export function eventValue(event) {
  const value = event?.value;
  if (value == null || typeof value !== "object") {
    return undefined;
  }
  return value.value;
}

export function requestRepaint() {
  if (typeof globalThis.requestRepaint === "function") {
    globalThis.requestRepaint();
  }
}

function createHostNode(type, props) {
  return {
    children: [],
    hidden: false,
    kind: "host",
    parent: null,
    props: props ?? {},
    type: String(type),
  };
}

function createTextNode(text) {
  return {
    hidden: false,
    kind: "text",
    parent: null,
    text: String(text),
  };
}

function detachLinkedChild(parent, child) {
  const children = parent.children ?? [];
  const index = children.indexOf(child);
  if (index >= 0) {
    children.splice(index, 1);
  }
  if (child.parent === parent) {
    child.parent = null;
  }
}

function detachFromCurrentParent(child) {
  if (child.parent != null) {
    detachLinkedChild(child.parent, child);
  }
}

function linkChild(parent, child, beforeChild = null) {
  detachFromCurrentParent(child);
  child.parent = parent;
  const children = parent.children ?? [];
  const index = beforeChild == null ? -1 : children.indexOf(beforeChild);
  if (index >= 0) {
    children.splice(index, 0, child);
  } else {
    children.push(child);
  }
}

function commitContainer(container) {
  try {
    const lowered = lowerCommittedRoot(container.children, metadataView);
    for (const warning of lowered.warnings) {
      if (loggedWarnings.has(warning)) {
        continue;
      }
      loggedWarnings.add(warning);
      log("warn", warning);
    }

    if (lowered.root == null) {
      if (rootRendered && !allowEmptyCommit) {
        throw new Error("render() expected a JSX element.");
      }
      container.rootInstance = null;
      container.handlers = new Map();
      return;
    }

    const mutations = [];
    const previousIndex = indexInstanceTree(container.rootInstance);
    container.rootInstance = reconcileRoot(
      container.rootInstance,
      lowered.root,
      mutations,
      previousIndex,
    );
    container.handlers = lowered.handlers;

    if (mutations.length > 0) {
      Deno.core.ops.op_commit_mutations(
        JSON.stringify({ version: metadataView.version, mutations }),
      );
    }
  } catch (error) {
    pendingRuntimeError = error;
  }
}

function reconcileRoot(previous, descriptor, mutations, previousIndex) {
  if (
    previous == null ||
    previous.nodeId !== descriptor.nodeId ||
    previous.family !== descriptor.family
  ) {
    // Root identity is the normalized family + nodeId pair. Either side changing
    // remounts the subtree so renderer-local egui state cannot drift across identities.
    const next = mountInstance(descriptor, null);
    mutations.push({ kind: "replace_root", subtree: materializeInstance(next) });
    appendMountedMotionMutations(next, mutations);
    return next;
  }

  reconcileInstance(previous, descriptor, mutations, previousIndex);
  return previous;
}

function reconcileInstance(instance, descriptor, mutations, previousIndex) {
  if (!sameValue(instance.props, descriptor.props)) {
    instance.props = descriptor.props;
    mutations.push({ kind: "update_node", node: { ...descriptor.props } });
  }

  if (!sameValue(instance.motion, descriptor.motion)) {
    instance.motion = descriptor.motion;
    appendMotionMutation(instance, mutations);
  }

  reconcileChildren(instance, descriptor.children, mutations, previousIndex);
}

function reconcileChildren(parent, descriptors, mutations, previousIndex) {
  const previousChildren = parent.children;
  const previousOrder = previousChildren.map((child) => child.nodeId);
  const previousById = new Map(previousChildren.map((child) => [child.nodeId, child]));

  const nextChildren = [];
  const handledPrevious = new Set();

  descriptors.forEach((descriptor, index) => {
    const existing = previousById.get(descriptor.nodeId);
    if (existing != null && existing.family === descriptor.family) {
      existing.parent = parent;
      reconcileInstance(existing, descriptor, mutations, previousIndex);
      nextChildren.push(existing);
      handledPrevious.add(existing);
      return;
    }

    const globalExisting = previousIndex.get(descriptor.nodeId);
    const positionalChild = previousChildren[index];
    if (
      positionalChild != null &&
      !handledPrevious.has(positionalChild) &&
      positionalChild.family === descriptor.family &&
      positionalChild.nodeId !== descriptor.nodeId &&
      globalExisting == null
    ) {
      // Same-family, same-slot replacements with a fresh nodeId are deliberate remounts.
      // This keeps identity-sensitive widgets from inheriting egui-local state when the
      // authored identity changes in place.
      const next = mountInstance(descriptor, parent);
      nextChildren.push(next);
      mutations.push({
        kind: "replace_subtree",
        node_id: positionalChild.nodeId,
        subtree: materializeInstance(next),
      });
      appendMountedMotionMutations(next, mutations);
      handledPrevious.add(positionalChild);
      return;
    }

    const next = mountInstance(descriptor, parent);
    nextChildren.push(next);
    if (existing != null) {
      // A reused nodeId with a different normalized family is also a remount boundary.
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
        index,
        parent_id: parent.nodeId,
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

function indexInstanceTree(root) {
  const index = new Map();
  visitInstance(root, index);
  return index;
}

function visitInstance(instance, index) {
  if (instance == null) {
    return;
  }
  index.set(instance.nodeId, instance);
  for (const child of instance.children) {
    visitInstance(child, index);
  }
}

function mountInstance(descriptor, parent) {
  const instance = {
    children: [],
    family: descriptor.family,
    motion: descriptor.motion,
    nodeId: descriptor.nodeId,
    parent,
    props: descriptor.props,
  };
  instance.children = descriptor.children.map((child) => mountInstance(child, instance));
  return instance;
}

function materializeInstance(instance) {
  const node = { ...instance.props };
  if (instance.children.length > 0) {
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

function dispatchEvent(container, event) {
  const nodeId = String(event?.node_id ?? "");
  const kind = String(event?.kind ?? "");
  const actionId = event?.action_id == null ? null : String(event.action_id);
  const candidates = [
    `${nodeId}:${kind}`,
    `${nodeId}:*`,
    actionId == null ? null : `action:${actionId}:${kind}`,
    actionId == null ? null : `action:${actionId}:*`,
  ].filter(Boolean);

  for (const key of candidates) {
    for (const handler of container.handlers.get(key) ?? []) {
      handler(event, eventValue(event));
    }
  }
}

function isPlainObject(value) {
  return typeof value === "object" && value != null && !Array.isArray(value);
}

function throwPendingRuntimeError() {
  if (pendingRuntimeError == null) {
    return;
  }
  const error = pendingRuntimeError;
  pendingRuntimeError = null;
  throw error;
}

const hostConfig = {
  HostTransitionContext: React.createContext(null),
  NotPendingTransition: null,
  afterActiveInstanceBlur() {},
  appendChild(parent, child) {
    linkChild(parent, child, null);
  },
  appendChildToContainer(container, child) {
    linkChild(container, child, null);
  },
  appendInitialChild(parent, child) {
    linkChild(parent, child, null);
  },
  beforeActiveInstanceBlur() {},
  cancelTimeout(id) {
    if (typeof clearTimeout === "function" && id !== -1) {
      clearTimeout(id);
    }
  },
  clearContainer(container) {
    for (const child of [...container.children]) {
      detachLinkedChild(container, child);
    }
    container.children = [];
  },
  commitTextUpdate(textInstance, _oldText, newText) {
    textInstance.text = String(newText);
  },
  commitUpdate(instance, _type, _prevProps, nextProps) {
    instance.props = nextProps ?? {};
  },
  createInstance(type, props) {
    return createHostNode(type, props);
  },
  createTextInstance(text) {
    return createTextNode(text);
  },
  detachDeletedInstance() {},
  finalizeInitialChildren() {
    return false;
  },
  getChildHostContext(parentHostContext) {
    return parentHostContext;
  },
  getCurrentUpdatePriority() {
    return currentUpdatePriority;
  },
  getInstanceFromNode() {
    return null;
  },
  getInstanceFromScope() {
    return null;
  },
  getPublicInstance(instance) {
    return instance;
  },
  getRootHostContext() {
    return {};
  },
  hideInstance(instance) {
    instance.hidden = true;
  },
  hideTextInstance(textInstance) {
    textInstance.hidden = true;
  },
  insertBefore(parent, child, beforeChild) {
    linkChild(parent, child, beforeChild);
  },
  insertInContainerBefore(container, child, beforeChild) {
    linkChild(container, child, beforeChild);
  },
  isPrimaryRenderer: true,
  noTimeout: -1,
  prepareForCommit() {
    return null;
  },
  preparePortalMount() {},
  prepareScopeUpdate() {},
  removeChild(parent, child) {
    detachLinkedChild(parent, child);
  },
  removeChildFromContainer(container, child) {
    detachLinkedChild(container, child);
  },
  requestPostPaintCallback(callback) {
    queueMicrotask(() => callback(performance.now()));
  },
  resetAfterCommit(container) {
    commitContainer(container);
  },
  resetFormInstance() {},
  resetTextContent(instance) {
    for (const child of [...instance.children]) {
      detachLinkedChild(instance, child);
    }
    instance.children = [];
  },
  resolveEventTimeStamp() {
    return performance.now();
  },
  resolveEventType() {
    return null;
  },
  resolveUpdatePriority() {
    return 16;
  },
  scheduleMicrotask(fn) {
    queueMicrotask(fn);
  },
  scheduleTimeout(fn, delay) {
    if (typeof setTimeout === "function") {
      return setTimeout(fn, delay);
    }
    queueMicrotask(fn);
    return -1;
  },
  setCurrentUpdatePriority(priority) {
    currentUpdatePriority = priority;
  },
  shouldAttemptEagerTransition() {
    return false;
  },
  shouldSetTextContent(type, props) {
    try {
      const family = normalizedFamilyForType(type, props ?? {}, metadataView);
      return familyUsesTextContent(family);
    } catch (error) {
      pendingRuntimeError = error;
      return false;
    }
  },
  supportsHydration: false,
  supportsMicrotasks: true,
  supportsMutation: true,
  supportsPersistence: false,
  trackSchedulerEvent() {},
  unhideInstance(instance) {
    instance.hidden = false;
  },
  unhideTextInstance(textInstance) {
    textInstance.hidden = false;
  },
};

const reconciler = Reconciler(hostConfig);
const container = {
  children: [],
  handlers: new Map(),
  kind: "container",
  rootInstance: null,
};
const root = reconciler.createContainer(
  container,
  1,
  null,
  false,
  null,
  "",
  console.log,
  console.log,
  console.log,
  () => {},
);

export function render(element) {
  pendingRuntimeError = null;
  reconciler.flushSyncFromReconciler(() => {
    reconciler.updateContainerSync(element, root, null, null);
  });
  throwPendingRuntimeError();
  rootRendered = true;
}

globalThis.__eguiUnmountRuntime = function unmountEguiRuntime() {
  pendingRuntimeError = null;
  allowEmptyCommit = true;
  try {
    reconciler.flushSyncFromReconciler(() => {
      reconciler.updateContainerSync(null, root, null, null);
    });
    container.children = [];
    container.handlers = new Map();
    container.rootInstance = null;
    rootRendered = false;
    throwPendingRuntimeError();
  } finally {
    allowEmptyCommit = false;
  }
};

globalThis.__eguiDispatchEvents = function dispatchEguiEvents(events) {
  if (!rootRendered) {
    throw new Error("render(<... />) must be called before dispatching events.");
  }
  if (!Array.isArray(events) || events.length === 0) {
    return;
  }
  pendingRuntimeError = null;
  reconciler.flushSyncFromReconciler(() => {
    reconciler.discreteUpdates(() => {
      for (const event of events) {
        dispatchEvent(container, event);
      }
    });
  });
  throwPendingRuntimeError();
};
