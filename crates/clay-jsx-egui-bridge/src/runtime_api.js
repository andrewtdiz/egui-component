import React from "react";
import Reconciler from "clay-internal:/react-reconciler";

import {
  createContractMetadataView,
  familyUsesTextContent,
  lowerCommittedRoot,
  lowerCommittedSubtree,
  normalizedFamilyForType,
  pathForCommittedChild,
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
    allowsChildren: false,
    children: [],
    contractProps: null,
    family: null,
    handlerRegistrations: [],
    hidden: false,
    kind: "host",
    materialized: false,
    motion: null,
    nodeId: null,
    parent: null,
    props: props ?? {},
    type: String(type),
  };
}

function createTextNode(text) {
  return {
    allowsChildren: false,
    children: [],
    contractProps: null,
    family: null,
    handlerRegistrations: [],
    hidden: false,
    kind: "text",
    materialized: false,
    motion: null,
    nodeId: null,
    parent: null,
    text: String(text),
  };
}

function isHostNode(value) {
  return value?.kind === "host";
}

function isTextNode(value) {
  return value?.kind === "text";
}

function childProducesDescriptor(node) {
  if (node == null || node.hidden === true) {
    return false;
  }
  if (isTextNode(node)) {
    return String(node.text).trim() !== "";
  }
  if (isHostNode(node)) {
    return true;
  }
  throw new Error(`Cannot lower committed node kind ${String(node?.kind)}.`);
}

function collectLowerableChildren(children) {
  const result = [];
  for (const child of children ?? []) {
    if (childProducesDescriptor(child)) {
      result.push(child);
    }
  }
  return result;
}

function loweredChildIndex(parent, child) {
  const siblings = parent.children ?? [];
  let index = 0;
  for (const sibling of siblings) {
    if (sibling === child) {
      return index;
    }
    if (childProducesDescriptor(sibling)) {
      index += 1;
    }
  }
  throw new Error("Committed child is not linked under its parent.");
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
  const previousParent = child.parent;
  if (previousParent != null) {
    detachLinkedChild(previousParent, child);
  }
  return previousParent;
}

function linkChild(parent, child, beforeChild = null) {
  const previousParent = detachFromCurrentParent(child);
  child.parent = parent;
  const children = parent.children ?? [];
  const index = beforeChild == null ? -1 : children.indexOf(beforeChild);
  if (index >= 0) {
    children.splice(index, 0, child);
  } else {
    children.push(child);
  }
  return previousParent;
}

function logLoweringWarnings(warnings) {
  for (const warning of warnings ?? []) {
    if (loggedWarnings.has(warning)) {
      continue;
    }
    loggedWarnings.add(warning);
    log("warn", warning);
  }
}

function withRuntimeErrorCapture(callback) {
  if (pendingRuntimeError != null) {
    return;
  }
  try {
    callback();
  } catch (error) {
    pendingRuntimeError = error;
  }
}

function throwPendingRuntimeError() {
  if (pendingRuntimeError == null) {
    return;
  }
  const error = pendingRuntimeError;
  pendingRuntimeError = null;
  throw error;
}

function removeHandlerRegistrations(container, instance) {
  for (const registration of instance.handlerRegistrations ?? []) {
    const handlers = container.handlers.get(registration.key);
    if (handlers == null) {
      continue;
    }
    const index = handlers.indexOf(registration.handler);
    if (index >= 0) {
      handlers.splice(index, 1);
    }
    if (handlers.length === 0) {
      container.handlers.delete(registration.key);
    }
  }
  instance.handlerRegistrations = [];
}

function clearMaterializedMetadata(container, instance) {
  removeHandlerRegistrations(container, instance);
  for (const child of instance.children ?? []) {
    clearMaterializedMetadata(container, child);
  }
  instance.allowsChildren = false;
  instance.contractProps = null;
  instance.family = null;
  instance.materialized = false;
  instance.motion = null;
  instance.nodeId = null;
}

function syncSubtreeFromDescriptor(container, instance, descriptor, handlerIndex) {
  removeHandlerRegistrations(container, instance);
  instance.allowsChildren = metadataView.familiesWithChildren.has(descriptor.family);
  instance.contractProps = descriptor.props;
  instance.family = descriptor.family;
  instance.materialized = true;
  instance.motion = descriptor.motion;
  instance.nodeId = descriptor.nodeId;

  const registrations = handlerIndex.get(descriptor.nodeId) ?? [];
  instance.handlerRegistrations = registrations;
  for (const registration of registrations) {
    const handlers = container.handlers.get(registration.key) ?? [];
    handlers.push(registration.handler);
    container.handlers.set(registration.key, handlers);
  }

  const descriptorChildren = descriptor.children ?? [];
  let descriptorIndex = 0;
  for (const child of instance.children ?? []) {
    if (!instance.allowsChildren || !childProducesDescriptor(child)) {
      clearMaterializedMetadata(container, child);
      continue;
    }
    const childDescriptor = descriptorChildren[descriptorIndex];
    if (childDescriptor == null) {
      clearMaterializedMetadata(container, child);
      continue;
    }
    syncSubtreeFromDescriptor(container, child, childDescriptor, handlerIndex);
    descriptorIndex += 1;
  }

  if (descriptorIndex !== descriptorChildren.length) {
    throw new Error(
      `Lowered descriptor for node "${descriptor.nodeId}" could not be mapped to committed children.`,
    );
  }
}

function materializeDescriptor(descriptor) {
  const node = { ...descriptor.props };
  if ((descriptor.children ?? []).length > 0) {
    node.children = descriptor.children.map((child) => materializeDescriptor(child));
  }
  return node;
}

function appendDescriptorMotionMutations(descriptor, mutations) {
  appendMotionMutation(descriptor.nodeId, descriptor.motion, mutations);
  for (const child of descriptor.children ?? []) {
    appendDescriptorMotionMutations(child, mutations);
  }
}

function appendMotionMutation(nodeId, motion, mutations) {
  if (motion == null) {
    mutations.push({ kind: "clear_motion", node_id: nodeId });
    return;
  }
  mutations.push({
    kind: "set_motion",
    node_id: nodeId,
    motion,
  });
}

function pathForInstance(instance) {
  const parent = instance.parent;
  if (parent == null) {
    throw new Error("Committed node is missing a parent.");
  }
  const index = loweredChildIndex(parent, instance);
  if (parent.kind === "container") {
    return pathForCommittedChild("root", instance, index);
  }
  if (parent.nodeId == null || parent.nodeId === "") {
    throw new Error("Committed node parent is missing a materialized node_id.");
  }
  return pathForCommittedChild(`${parent.nodeId}.child`, instance, index);
}

function lowerInstanceSubtree(instance) {
  const lowered = lowerCommittedSubtree(instance, pathForInstance(instance), metadataView);
  logLoweringWarnings(lowered.warnings);
  return lowered;
}

function queueMutation(container, mutation) {
  container.pendingMutations.push(mutation);
}

function queueDirtyParentOrder(container, parent) {
  if (parent == null) {
    return;
  }
  if (parent.kind === "container") {
    container.rootSyncRequired = true;
    return;
  }
  if (!parent.materialized || !parent.allowsChildren || parent.nodeId == null) {
    return;
  }
  container.dirtyParents.add(parent);
}

function removeMaterializedSubtree(container, parent, child) {
  if (!child.materialized || child.nodeId == null) {
    clearMaterializedMetadata(container, child);
    return;
  }
  if (parent?.kind === "container") {
    container.rootSyncRequired = true;
    clearMaterializedMetadata(container, child);
    return;
  }
  queueMutation(container, { kind: "remove_subtree", node_id: child.nodeId });
  clearMaterializedMetadata(container, child);
}

function insertMaterializedSubtree(container, parent, child) {
  if (parent == null) {
    return;
  }
  if (parent.kind === "container") {
    container.rootSyncRequired = true;
    return;
  }
  if (!parent.materialized || !parent.allowsChildren || parent.nodeId == null) {
    clearMaterializedMetadata(container, child);
    return;
  }
  if (!childProducesDescriptor(child)) {
    clearMaterializedMetadata(container, child);
    return;
  }

  const lowered = lowerInstanceSubtree(child);
  if (lowered.root == null) {
    clearMaterializedMetadata(container, child);
    return;
  }

  const index = loweredChildIndex(parent, child);
  queueMutation(container, {
    kind: "insert_subtree",
    index,
    parent_id: parent.nodeId,
    subtree: materializeDescriptor(lowered.root),
  });
  appendDescriptorMotionMutations(lowered.root, container.pendingMutations);
  syncSubtreeFromDescriptor(container, child, lowered.root, lowered.handler_index);
}

function replaceMaterializedSubtree(container, instance, nextDescriptor, nextHandlerIndex) {
  const parent = instance.parent;
  if (parent?.kind === "container" && !container.syntheticRoot && container.rootNodeId === instance.nodeId) {
    queueMutation(container, {
      kind: "replace_root",
      subtree: materializeDescriptor(nextDescriptor),
    });
    appendDescriptorMotionMutations(nextDescriptor, container.pendingMutations);
    syncSubtreeFromDescriptor(container, instance, nextDescriptor, nextHandlerIndex);
    container.rootNodeId = nextDescriptor.nodeId;
    return;
  }

  queueMutation(container, {
    kind: "replace_subtree",
    node_id: instance.nodeId,
    subtree: materializeDescriptor(nextDescriptor),
  });
  appendDescriptorMotionMutations(nextDescriptor, container.pendingMutations);
  syncSubtreeFromDescriptor(container, instance, nextDescriptor, nextHandlerIndex);
}

function reconcileMaterializedNode(container, instance) {
  const parent = instance.parent;
  if (parent == null) {
    return;
  }

  if (parent.kind !== "container" && (!parent.materialized || !parent.allowsChildren)) {
    clearMaterializedMetadata(container, instance);
    return;
  }

  const lowerable = childProducesDescriptor(instance);
  if (!lowerable) {
    removeMaterializedSubtree(container, parent, instance);
    return;
  }

  const lowered = lowerInstanceSubtree(instance);
  if (lowered.root == null) {
    removeMaterializedSubtree(container, parent, instance);
    return;
  }

  if (!instance.materialized || instance.nodeId == null || instance.family == null) {
    insertMaterializedSubtree(container, parent, instance);
    return;
  }

  if (instance.nodeId !== lowered.root.nodeId || instance.family !== lowered.root.family) {
    if (parent.kind === "container" && container.syntheticRoot) {
      container.rootSyncRequired = true;
      return;
    }
    replaceMaterializedSubtree(container, instance, lowered.root, lowered.handler_index);
    return;
  }

  const previousProps = instance.contractProps;
  const previousMotion = instance.motion;
  syncSubtreeFromDescriptor(container, instance, lowered.root, lowered.handler_index);

  if (!sameValue(previousProps, lowered.root.props)) {
    queueMutation(container, {
      kind: "update_node",
      node: { ...lowered.root.props },
    });
  }

  if (!sameValue(previousMotion, lowered.root.motion)) {
    appendMotionMutation(lowered.root.nodeId, lowered.root.motion, container.pendingMutations);
  }
}

function flushRootSync(container) {
  const lowered = lowerCommittedRoot(container.children, metadataView);
  logLoweringWarnings(lowered.warnings);

  for (const child of container.children) {
    clearMaterializedMetadata(container, child);
  }
  container.handlers = new Map();

  if (lowered.root == null) {
    if (rootRendered && !allowEmptyCommit) {
      throw new Error("render() expected a JSX element.");
    }
    container.rootNodeId = null;
    container.syntheticRoot = false;
    return;
  }

  const topLevel = collectLowerableChildren(container.children);
  if (topLevel.length === 1) {
    syncSubtreeFromDescriptor(container, topLevel[0], lowered.root, lowered.handler_index);
  } else {
    if (topLevel.length !== lowered.root.children.length) {
      throw new Error("Root lowering could not be mapped back to committed top-level nodes.");
    }
    for (let index = 0; index < topLevel.length; index += 1) {
      syncSubtreeFromDescriptor(
        container,
        topLevel[index],
        lowered.root.children[index],
        lowered.handler_index,
      );
    }
  }

  const mutations = [
    {
      kind: "replace_root",
      subtree: materializeDescriptor(lowered.root),
    },
  ];
  appendDescriptorMotionMutations(lowered.root, mutations);

  Deno.core.ops.op_commit_mutations(
    JSON.stringify({ version: metadataView.version, mutations }),
  );

  container.rootNodeId = lowered.root.nodeId;
  container.syntheticRoot = topLevel.length > 1;
}

function flushDirtyParentOrders(container) {
  for (const parent of container.dirtyParents) {
    if (!parent.materialized || !parent.allowsChildren || parent.nodeId == null) {
      continue;
    }
    const childIds = [];
    for (const child of parent.children ?? []) {
      if (!child.materialized || child.nodeId == null) {
        continue;
      }
      childIds.push(child.nodeId);
    }
    queueMutation(container, {
      kind: "set_children",
      node_id: parent.nodeId,
      child_ids: childIds,
    });
  }
}

function beginCommit(container) {
  container.pendingMutations = [];
  container.dirtyParents = new Set();
  container.rootSyncRequired = false;
}

function flushCommit(container) {
  if (pendingRuntimeError != null) {
    return;
  }
  if (container.rootSyncRequired) {
    flushRootSync(container);
    return;
  }
  flushDirtyParentOrders(container);
  if (container.pendingMutations.length === 0) {
    return;
  }
  Deno.core.ops.op_commit_mutations(
    JSON.stringify({ version: metadataView.version, mutations: container.pendingMutations }),
  );
}

function linkChildWithMutations(container, parent, child, beforeChild = null) {
  const previousParent = linkChild(parent, child, beforeChild);

  if (previousParent === parent) {
    if (child.materialized) {
      queueDirtyParentOrder(container, parent);
    }
    return;
  }

  if (child.materialized && previousParent != null) {
    if (previousParent.kind === "container" || parent.kind === "container") {
      container.rootSyncRequired = true;
      return;
    }
    queueMutation(container, { kind: "remove_subtree", node_id: child.nodeId });
    clearMaterializedMetadata(container, child);
    insertMaterializedSubtree(container, parent, child);
    return;
  }

  insertMaterializedSubtree(container, parent, child);
}

function detachChildWithMutations(container, parent, child) {
  detachLinkedChild(parent, child);
  removeMaterializedSubtree(container, parent, child);
}

function resetTextContentWithMutations(container, instance) {
  for (const child of [...instance.children]) {
    detachLinkedChild(instance, child);
    removeMaterializedSubtree(container, instance, child);
  }
  instance.children = [];
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

function isPlainObject(value) {
  return typeof value === "object" && value != null && !Array.isArray(value);
}

const hostConfig = {
  HostTransitionContext: React.createContext(null),
  NotPendingTransition: null,
  afterActiveInstanceBlur() {},
  appendChild(parent, child) {
    withRuntimeErrorCapture(() => {
      linkChildWithMutations(container, parent, child, null);
    });
  },
  appendChildToContainer(parentContainer, child) {
    withRuntimeErrorCapture(() => {
      linkChild(parentContainer, child, null);
      parentContainer.rootSyncRequired = true;
    });
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
  clearContainer(parentContainer) {
    withRuntimeErrorCapture(() => {
      for (const child of [...parentContainer.children]) {
        detachLinkedChild(parentContainer, child);
        clearMaterializedMetadata(parentContainer, child);
      }
      parentContainer.children = [];
      parentContainer.rootSyncRequired = true;
    });
  },
  commitTextUpdate(textInstance, _oldText, newText) {
    withRuntimeErrorCapture(() => {
      textInstance.text = String(newText);
      reconcileMaterializedNode(container, textInstance);
    });
  },
  commitUpdate(instance, _type, _prevProps, nextProps) {
    withRuntimeErrorCapture(() => {
      instance.props = nextProps ?? {};
      reconcileMaterializedNode(container, instance);
    });
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
    withRuntimeErrorCapture(() => {
      instance.hidden = true;
      removeMaterializedSubtree(container, instance.parent, instance);
    });
  },
  hideTextInstance(textInstance) {
    withRuntimeErrorCapture(() => {
      textInstance.hidden = true;
      removeMaterializedSubtree(container, textInstance.parent, textInstance);
    });
  },
  insertBefore(parent, child, beforeChild) {
    withRuntimeErrorCapture(() => {
      linkChildWithMutations(container, parent, child, beforeChild);
    });
  },
  insertInContainerBefore(parentContainer, child, beforeChild) {
    withRuntimeErrorCapture(() => {
      linkChild(parentContainer, child, beforeChild);
      parentContainer.rootSyncRequired = true;
    });
  },
  isPrimaryRenderer: true,
  noTimeout: -1,
  prepareForCommit() {
    beginCommit(container);
    return null;
  },
  preparePortalMount() {},
  prepareScopeUpdate() {},
  removeChild(parent, child) {
    withRuntimeErrorCapture(() => {
      detachChildWithMutations(container, parent, child);
    });
  },
  removeChildFromContainer(parentContainer, child) {
    withRuntimeErrorCapture(() => {
      detachLinkedChild(parentContainer, child);
      clearMaterializedMetadata(parentContainer, child);
      parentContainer.rootSyncRequired = true;
    });
  },
  requestPostPaintCallback(callback) {
    queueMicrotask(() => callback(performance.now()));
  },
  resetAfterCommit(parentContainer) {
    withRuntimeErrorCapture(() => {
      flushCommit(parentContainer);
    });
  },
  resetFormInstance() {},
  resetTextContent(instance) {
    withRuntimeErrorCapture(() => {
      resetTextContentWithMutations(container, instance);
    });
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
    withRuntimeErrorCapture(() => {
      instance.hidden = false;
      reconcileMaterializedNode(container, instance);
    });
  },
  unhideTextInstance(textInstance) {
    withRuntimeErrorCapture(() => {
      textInstance.hidden = false;
      reconcileMaterializedNode(container, textInstance);
    });
  },
};

const reconciler = Reconciler(hostConfig);
const container = {
  children: [],
  dirtyParents: new Set(),
  handlers: new Map(),
  kind: "container",
  pendingMutations: [],
  rootNodeId: null,
  rootSyncRequired: false,
  syntheticRoot: false,
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
  Deno.core.ops.op_host_note_render();
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
    Deno.core.ops.op_host_note_unmount();
    reconciler.flushSyncFromReconciler(() => {
      reconciler.updateContainerSync(null, root, null, null);
    });
    for (const child of container.children) {
      clearMaterializedMetadata(container, child);
    }
    container.children = [];
    container.dirtyParents = new Set();
    container.handlers = new Map();
    container.pendingMutations = [];
    container.rootNodeId = null;
    container.rootSyncRequired = false;
    container.syntheticRoot = false;
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
