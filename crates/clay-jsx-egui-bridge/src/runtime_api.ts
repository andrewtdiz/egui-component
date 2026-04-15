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

// Ownership boundary:
// - JS owns only the ephemeral reconciler/shadow tree in this module.
// - JS must not create or mutate native egui renderer objects directly.
// - The only JS -> Rust retained-state mutation path is enqueueCommitBatch(),
//   which uses an explicit commit transport abstraction.
// - JSON is the only production-supported transport backend in v1.
// - A typed test/benchmark backend exists only as an explicit experiment so the
//   bridge can evaluate non-JSON transport without rewriting the reconciler
//   host config again.
// - All other host ops below are diagnostics or lifecycle telemetry only.

// Frozen v1 deferrals:
// - hydration
// - persistence mode
// - direct native host refs / public native instances
// - hook-state restoration across reload
// - production transport replacement away from JSON
// See docs/jsx-runtime-v1-deferred-features.md.

const hostOps = Object.freeze({
  commitMutationsJson(batchJson) {
    return Deno.core.ops.op_commit_mutations(batchJson);
  },
  commitMutationsTyped(batch) {
    return Deno.core.ops.op_commit_mutations_typed(batch);
  },
  log(level, message) {
    Deno.core.ops.op_host_log(String(level), String(message));
  },
  reportReconcilerError(category, message, componentStack, errorBoundary) {
    Deno.core.ops.op_host_report_reconciler_error(
      category,
      message,
      componentStack,
      errorBoundary,
    );
  },
  noteRender() {
    Deno.core.ops.op_host_note_render();
  },
  noteUnmount() {
    Deno.core.ops.op_host_note_unmount();
  },
  nowMs() {
    return Deno.core.ops.op_host_now_ms();
  },
});

const COMMIT_TRANSPORT_JSON = "json";
const COMMIT_TRANSPORT_TYPED = "typed";

function allowExperimentalTypedCommitTransport() {
  return globalThis.__eguiAllowExperimentalCommitTransportForTest === true;
}

function normalizeCommitTransportMode(mode) {
  if (mode == null || mode === "") {
    return COMMIT_TRANSPORT_JSON;
  }
  if (mode === COMMIT_TRANSPORT_JSON) {
    return mode;
  }
  if (mode === COMMIT_TRANSPORT_TYPED) {
    if (allowExperimentalTypedCommitTransport()) {
      return mode;
    }
    throw new Error(
      "Typed egui commit transport is deferred in v1. Production transport replacement requires explicit test/benchmark opt-in.",
    );
  }
  throw new Error(`Unknown egui commit transport mode ${String(mode)}.`);
}

function createJsonCommitTransport() {
  return Object.freeze({
    enqueueBatch(batch) {
      return hostOps.commitMutationsJson(JSON.stringify(batch));
    },
    mode: COMMIT_TRANSPORT_JSON,
  });
}

function createTypedCommitTransport() {
  return Object.freeze({
    enqueueBatch(batch) {
      return hostOps.commitMutationsTyped(batch);
    },
    mode: COMMIT_TRANSPORT_TYPED,
  });
}

function createCommitTransport(mode) {
  switch (normalizeCommitTransportMode(mode)) {
    case COMMIT_TRANSPORT_JSON:
      return createJsonCommitTransport();
    case COMMIT_TRANSPORT_TYPED:
      return createTypedCommitTransport();
    default:
      throw new Error(`Unknown egui commit transport mode ${String(mode)}.`);
  }
}

const contractMetadata = globalThis.__eguiContract ?? {};
const metadataView = createContractMetadataView(contractMetadata);
const loggedWarnings = new Set();

function createReadyCommitProtocolState(pendingCommitBatchIds = []) {
  return {
    kind: "ready",
    pendingCommitBatchIds: pendingCommitBatchIds.slice(),
  };
}

function createFailedCommitProtocolState(message) {
  return {
    kind: "failed",
    message: String(message),
  };
}

function createRecoveryState() {
  return {
    category: null,
    component_stack: "",
    disposition: "continue",
    error_boundary: null,
    message: null,
    rejected_commit_batch_id: null,
  };
}

function recoveryDispositionPriority(disposition) {
  switch (disposition) {
    case "continue":
      return 0;
    case "bounded_failure":
      return 1;
    case "teardown":
      return 2;
    case "reload_required":
      return 3;
    default:
      throw new Error(`Unknown egui recovery disposition ${String(disposition)}.`);
  }
}

function normalizeRecoveryState(nextState) {
  return {
    category: nextState?.category == null ? null : String(nextState.category),
    component_stack: typeof nextState?.component_stack === "string" ? nextState.component_stack : "",
    disposition: String(nextState?.disposition ?? "continue"),
    error_boundary: nextState?.error_boundary == null || String(nextState.error_boundary).length === 0
      ? null
      : String(nextState.error_boundary),
    message: nextState?.message == null ? null : String(nextState.message),
    rejected_commit_batch_id: nextState?.rejected_commit_batch_id == null
      ? null
      : Number(nextState.rejected_commit_batch_id),
  };
}

function observeRecoveryState(nextState) {
  const normalized = normalizeRecoveryState(nextState);
  if (
    recoveryDispositionPriority(normalized.disposition) <
      recoveryDispositionPriority(recoveryState.disposition)
  ) {
    return recoveryState;
  }
  recoveryState = normalized;
  return recoveryState;
}

function setFatalRecoveryState(message, componentStack = "", errorBoundary = null) {
  return observeRecoveryState({
    category: "fatal",
    component_stack: componentStack,
    disposition: "teardown",
    error_boundary: errorBoundary,
    message,
    rejected_commit_batch_id: null,
  });
}

function setProtocolFailedRecoveryState(message, rejectedCommitBatchId = null) {
  return observeRecoveryState({
    category: "protocol_failed",
    component_stack: "",
    disposition: "reload_required",
    error_boundary: null,
    message,
    rejected_commit_batch_id: rejectedCommitBatchId,
  });
}

let currentUpdatePriority = 0;
let pendingRuntimeError = null;
let rootRendered = false;
let allowEmptyCommit = false;
let commitProtocolState = createReadyCommitProtocolState();
let recoveryState = createRecoveryState();
let indexedParentChildMappingEnabled = true;
let commitTransport = createCommitTransport(globalThis.__eguiCommitTransportModeForTest);

// V1 hot-reload policy:
// - reload validation happens outside this JS runtime
// - a successful reload installs a brand-new runtime and remounts cold
// - hook-state capture/restore is API-compatibility only in v1 and is
//   intentionally not applied to the next runtime instance
// - callers must not assume partial state preservation across reloads
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
  hostOps.log(level, message);
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

function resetCommitProtocolState() {
  commitProtocolState = createReadyCommitProtocolState();
}

function setCommitProtocolFailure(message) {
  if (commitProtocolState.kind === "failed") {
    return;
  }
  commitProtocolState = createFailedCommitProtocolState(message);
  setProtocolFailedRecoveryState(message);
}

function throwIfCommitProtocolFailed() {
  if (commitProtocolState.kind !== "failed") {
    return;
  }
  throw new Error(commitProtocolState.message);
}

function throwIfRecoveryStateBlocksWork() {
  const disposition = recoveryState.disposition;
  if (disposition !== "teardown" && disposition !== "reload_required") {
    return;
  }
  throw new Error(
    recoveryState.message ??
      (disposition === "reload_required"
        ? "egui session requires reload before it can continue."
        : "egui session requires teardown before it can continue."),
  );
}

function setCommitTransportMode(mode) {
  throwIfCommitProtocolFailed();
  if (
    commitProtocolState.kind === "ready" &&
    commitProtocolState.pendingCommitBatchIds.length > 0
  ) {
    throw new Error(
      "Cannot switch egui commit transport while commit acknowledgements are pending.",
    );
  }
  if (mode === COMMIT_TRANSPORT_TYPED && !allowExperimentalTypedCommitTransport()) {
    throw new Error(
      "Typed egui commit transport is deferred in v1. Production transport replacement requires explicit test/benchmark opt-in.",
    );
  }
  commitTransport = createCommitTransport(mode);
  globalThis.__eguiCommitTransportModeForTest = commitTransport.mode;
  return commitTransport.mode;
}

function enqueueCommitBatch(batch) {
  throwIfCommitProtocolFailed();
  const rawCommitBatchId = commitTransport.enqueueBatch(batch);
  const commitBatchId = Number(rawCommitBatchId);
  if (!Number.isSafeInteger(commitBatchId) || commitBatchId <= 0) {
    throw new Error(
      `op_commit_mutations returned invalid commit batch id ${String(rawCommitBatchId)}.`,
    );
  }
  if (commitProtocolState.kind !== "ready") {
    throw new Error("Cannot enqueue egui commit batches after protocol failure.");
  }
  commitProtocolState.pendingCommitBatchIds.push(commitBatchId);
  return commitBatchId;
}

globalThis.__eguiSetCommitTransportModeForTest = function setEguiCommitTransportModeForTest(mode) {
  return setCommitTransportMode(mode);
};

globalThis.__eguiGetCommitTransportModeForTest = function getEguiCommitTransportModeForTest() {
  return commitTransport.mode;
};

globalThis.__eguiDescribeCommitProtocolForTest = function describeEguiCommitProtocolForTest() {
  if (commitProtocolState.kind === "ready") {
    return {
      kind: commitProtocolState.kind,
      pendingCommitBatchIds: commitProtocolState.pendingCommitBatchIds.slice(),
    };
  }

  return {
    kind: commitProtocolState.kind,
    message: commitProtocolState.message,
  };
};

globalThis.__eguiDescribeRecoveryStateForTest = function describeEguiRecoveryStateForTest() {
  return {
    ...recoveryState,
  };
};

globalThis.__eguiAcknowledgeCommits = function acknowledgeEguiCommits(commitBatchIds) {
  if (!Array.isArray(commitBatchIds) || commitBatchIds.length === 0) {
    return;
  }

  if (commitProtocolState.kind !== "ready") {
    return;
  }

  for (const rawCommitBatchId of commitBatchIds) {
    const commitBatchId = Number(rawCommitBatchId);
    if (!Number.isSafeInteger(commitBatchId) || commitBatchId <= 0) {
      resetCommitProtocolState();
      setCommitProtocolFailure(
        `Host acknowledged invalid commit batch id ${String(rawCommitBatchId)}.`,
      );
      return;
    }

    const expectedCommitBatchId = commitProtocolState.pendingCommitBatchIds.shift();
    if (expectedCommitBatchId == null || expectedCommitBatchId !== commitBatchId) {
      resetCommitProtocolState();
      setCommitProtocolFailure(
        `Host acknowledged unexpected commit batch ${commitBatchId} (expected ${String(
          expectedCommitBatchId,
        )}).`,
      );
      return;
    }
  }
};

globalThis.__eguiRejectCommit = function rejectEguiCommit(commitBatchId, reason) {
  const message = `egui session entered controlled session failure after commit batch ${String(commitBatchId)} was rejected: ${String(
    reason ?? "unknown reason",
  )}. Reload the session to recover.`;
  setCommitProtocolFailure(message);
  setProtocolFailedRecoveryState(message, commitBatchId);
};

globalThis.__eguiFailCommitProtocol = function failEguiCommitProtocol(reason) {
  const message = `egui session entered controlled session failure after a commit protocol error: ${String(reason ?? "unknown reason")}. Reload the session to recover.`;
  setCommitProtocolFailure(message);
  setProtocolFailedRecoveryState(message);
};

function createHostNode(type, props) {
  // This is a JS shadow node for reconciliation/lowering only. Rust owns the
  // retained HostTree and any native renderer state derived from commit batches.
  return {
    allowsChildren: false,
    children: [],
    contractProps: null,
    family: null,
    handlerRegistrations: [],
    hidden: false,
    kind: "host",
    loweredChildIndices: null,
    materialized: false,
    motion: null,
    nodeId: null,
    parent: null,
    props: props ?? {},
    siblingIndex: -1,
    type: String(type),
  };
}

function createTextNode(text) {
  // Text instances are also JS-only shadow nodes until lowered into a commit
  // batch and validated by Rust.
  return {
    allowsChildren: false,
    children: [],
    contractProps: null,
    family: null,
    handlerRegistrations: [],
    hidden: false,
    kind: "text",
    loweredChildIndices: null,
    materialized: false,
    motion: null,
    nodeId: null,
    parent: null,
    siblingIndex: -1,
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

function indexedChildArrayIndex(parent, child) {
  if (child?.parent !== parent) {
    return -1;
  }

  const children = parent.children ?? [];
  const hintedIndex = Number.isInteger(child.siblingIndex)
    ? child.siblingIndex
    : -1;
  if (
    hintedIndex >= 0 &&
    hintedIndex < children.length &&
    children[hintedIndex] === child
  ) {
    return hintedIndex;
  }

  const scannedIndex = children.indexOf(child);
  if (scannedIndex >= 0) {
    child.siblingIndex = scannedIndex;
    return scannedIndex;
  }
  return -1;
}

function invalidateLoweredChildIndices(parent) {
  if (parent == null || typeof parent !== "object") {
    return;
  }
  parent.loweredChildIndices = null;
}

function resequenceSiblingIndices(parent, startIndex = 0) {
  if (!indexedParentChildMappingEnabled) {
    return;
  }

  const children = parent.children ?? [];
  const normalizedStart = Math.max(0, Math.min(startIndex, children.length));
  for (let index = normalizedStart; index < children.length; index += 1) {
    children[index].siblingIndex = index;
  }
  invalidateLoweredChildIndices(parent);
}

function loweredChildIndexBySiblingScan(parent, child) {
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

function loweredChildIndex(parent, child) {
  if (!indexedParentChildMappingEnabled) {
    return loweredChildIndexBySiblingScan(parent, child);
  }

  const linkedChildIndex = indexedChildArrayIndex(parent, child);
  if (linkedChildIndex < 0) {
    throw new Error("Committed child is not linked under its parent.");
  }

  if (parent.loweredChildIndices == null) {
    const loweredByChild = new Map();
    let index = 0;
    for (const sibling of parent.children ?? []) {
      loweredByChild.set(sibling, index);
      if (childProducesDescriptor(sibling)) {
        index += 1;
      }
    }
    parent.loweredChildIndices = loweredByChild;
  }

  const loweredIndex = parent.loweredChildIndices.get(child);
  if (loweredIndex == null) {
    throw new Error("Committed child is not linked under its parent.");
  }
  return loweredIndex;
}

function moveChildWithinParent(parent, fromIndex, toIndex) {
  const children = parent.children ?? [];
  if (
    fromIndex < 0 ||
    fromIndex >= children.length ||
    toIndex < 0 ||
    toIndex >= children.length ||
    fromIndex === toIndex
  ) {
    return;
  }

  const movedChild = children[fromIndex];
  if (fromIndex < toIndex) {
    for (let index = fromIndex; index < toIndex; index += 1) {
      const shifted = children[index + 1];
      children[index] = shifted;
      shifted.siblingIndex = index;
    }
  } else {
    for (let index = fromIndex; index > toIndex; index -= 1) {
      const shifted = children[index - 1];
      children[index] = shifted;
      shifted.siblingIndex = index;
    }
  }

  children[toIndex] = movedChild;
  movedChild.siblingIndex = toIndex;
  invalidateLoweredChildIndices(parent);
}

function detachLinkedChild(parent, child) {
  const children = parent.children ?? [];
  const index = indexedParentChildMappingEnabled
    ? indexedChildArrayIndex(parent, child)
    : children.indexOf(child);
  if (index >= 0) {
    children.splice(index, 1);
    if (indexedParentChildMappingEnabled) {
      resequenceSiblingIndices(parent, index);
    } else {
      invalidateLoweredChildIndices(parent);
    }
  }
  if (child.parent === parent) {
    child.parent = null;
  }
  child.siblingIndex = -1;
}

function detachFromCurrentParent(child) {
  const previousParent = child.parent;
  if (previousParent != null) {
    detachLinkedChild(previousParent, child);
  }
  return previousParent;
}

function linkChild(parent, child, beforeChild = null) {
  if (indexedParentChildMappingEnabled && child.parent === parent) {
    const fromIndex = indexedChildArrayIndex(parent, child);
    if (fromIndex >= 0) {
      const children = parent.children ?? [];
      let targetIndex = beforeChild == null
        ? children.length - 1
        : indexedChildArrayIndex(parent, beforeChild);
      if (targetIndex < 0) {
        targetIndex = children.length - 1;
      }
      if (beforeChild != null && targetIndex > fromIndex) {
        targetIndex -= 1;
      }
      moveChildWithinParent(parent, fromIndex, targetIndex);
      return parent;
    }
  }

  const previousParent = detachFromCurrentParent(child);
  child.parent = parent;
  const children = parent.children ?? [];
  const index = beforeChild == null
    ? children.length
    : indexedParentChildMappingEnabled
    ? indexedChildArrayIndex(parent, beforeChild)
    : children.indexOf(beforeChild);
  if (index >= 0 && index <= children.length) {
    children.splice(index, 0, child);
    child.siblingIndex = index;
    if (indexedParentChildMappingEnabled) {
      resequenceSiblingIndices(parent, index + 1);
    }
  } else {
    children.push(child);
    child.siblingIndex = children.length - 1;
    invalidateLoweredChildIndices(parent);
  }

  if (!indexedParentChildMappingEnabled) {
    invalidateLoweredChildIndices(parent);
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

function captureRuntimeError(callback, fallbackValue) {
  if (pendingRuntimeError != null) {
    return fallbackValue;
  }
  try {
    return callback();
  } catch (error) {
    pendingRuntimeError = error;
    setFatalRecoveryState(
      error instanceof Error ? error.message : String(error),
    );
    return fallbackValue;
  }
}

function withRuntimeErrorCapture(callback) {
  captureRuntimeError(callback, undefined);
}

function throwPendingRuntimeError() {
  if (pendingRuntimeError == null) {
    return;
  }
  const error = pendingRuntimeError;
  pendingRuntimeError = null;
  throw error;
}

function normalizeReconcilerErrorCategory(category) {
  const normalizedCategory = String(category);
  switch (normalizedCategory) {
    case "uncaught":
    case "caught":
    case "recoverable":
      return normalizedCategory;
    default:
      throw new Error(`Unknown reconciler error category ${normalizedCategory}.`);
  }
}

function reconcilerErrorMessage(error) {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

function reconcilerComponentStack(errorInfo) {
  return typeof errorInfo?.componentStack === "string" ? errorInfo.componentStack : "";
}

function reconcilerErrorBoundaryLabel(errorInfo) {
  const boundary = errorInfo?.errorBoundary;
  if (boundary == null) {
    return "";
  }
  if (typeof boundary === "string") {
    return boundary;
  }
  if (typeof boundary?.displayName === "string" && boundary.displayName.length > 0) {
    return boundary.displayName;
  }
  if (typeof boundary?.name === "string" && boundary.name.length > 0) {
    return boundary.name;
  }
  const ctor = boundary?.constructor;
  if (ctor != null) {
    if (typeof ctor.displayName === "string" && ctor.displayName.length > 0) {
      return ctor.displayName;
    }
    if (typeof ctor.name === "string" && ctor.name.length > 0) {
      return ctor.name;
    }
  }
  return String(boundary);
}

function createReconcilerHostError(category, message, componentStack, errorBoundary) {
  const details = [];
  if (errorBoundary.length > 0) {
    details.push(`boundary=${errorBoundary}`);
  }
  if (componentStack.length > 0) {
    details.push(`component_stack=${componentStack}`);
  }
  const suffix = details.length > 0 ? ` (${details.join(", ")})` : "";
  return new Error(`React reconciler ${category} error: ${message}${suffix}`);
}

function reconcileRuntimeErrorContainmentPolicy(category, errorBoundary) {
  if (category === "recoverable") {
    return {
      disposition: "non_fatal",
      reason: "recoverable",
    };
  }

  if (category === "uncaught") {
    return {
      disposition: "fatal",
      reason: "uncaught",
    };
  }

  if (errorBoundary.length > 0) {
    return {
      disposition: "non_fatal",
      reason: "boundary_contained",
    };
  }

  return {
    disposition: "fatal",
    reason: "caught_without_boundary",
  };
}

function reportReconcilerError(category, error, errorInfo) {
  const normalizedCategory = normalizeReconcilerErrorCategory(category);
  const message = reconcilerErrorMessage(error);
  const componentStack = reconcilerComponentStack(errorInfo);
  const errorBoundary = normalizedCategory === "caught" ? reconcilerErrorBoundaryLabel(errorInfo) : "";
  const containmentPolicy = reconcileRuntimeErrorContainmentPolicy(
    normalizedCategory,
    errorBoundary,
  );

  hostOps.reportReconcilerError(
    normalizedCategory,
    message,
    componentStack,
    errorBoundary,
  );

  if (normalizedCategory === "recoverable") {
    observeRecoveryState({
      category: "recoverable",
      component_stack: componentStack,
      disposition: "continue",
      error_boundary: errorBoundary,
      message,
      rejected_commit_batch_id: null,
    });
  } else if (containmentPolicy.reason === "boundary_contained") {
    observeRecoveryState({
      category: "boundary_contained",
      component_stack: componentStack,
      disposition: "bounded_failure",
      error_boundary: errorBoundary,
      message,
      rejected_commit_batch_id: null,
    });
  }

  if (containmentPolicy.disposition === "non_fatal") {
    return;
  }

  if (pendingRuntimeError == null) {
    const hostError = createReconcilerHostError(
      normalizedCategory,
      message,
      componentStack,
      errorBoundary,
    );
    if (containmentPolicy.reason === "caught_without_boundary") {
      hostError.message += " (missing error boundary containment)";
    }
    pendingRuntimeError = hostError;
    setFatalRecoveryState(hostError.message, componentStack, errorBoundary);
  }
}

globalThis.__eguiReportReconcilerErrorForTest = function reportEguiReconcilerErrorForTest(
  category,
  message,
  componentStack = "",
  errorBoundary = "",
) {
  const normalizedCategory = normalizeReconcilerErrorCategory(category);
  const info = {
    componentStack: String(componentStack),
    errorBoundary: errorBoundary == null || String(errorBoundary).length === 0
      ? null
      : { displayName: String(errorBoundary) },
  };
  reportReconcilerError(normalizedCategory, new Error(String(message)), info);
};

globalThis.__eguiSetIndexedParentChildMappingEnabledForTest =
  function setIndexedParentChildMappingEnabledForTest(enabled) {
    indexedParentChildMappingEnabled = Boolean(enabled);
    container.loweredChildIndices = null;
    for (const child of container.children ?? []) {
      clearChildIndexStateRecursive(child);
    }
  };

function clearChildIndexStateRecursive(node) {
  if (node == null || typeof node !== "object") {
    return;
  }
  node.loweredChildIndices = null;
  node.siblingIndex = -1;
  for (const child of node.children ?? []) {
    clearChildIndexStateRecursive(child);
  }
}

function removeHandlerRegistrations(container, instance) {
  for (const registration of instance.handlerRegistrations ?? []) {
    const handlers = container.handlers.get(registration.key);
    if (handlers == null) {
      continue;
    }
    const index = handlers.indexOf(registration.registration);
    if (index >= 0) {
      handlers.splice(index, 1);
    }
    if (handlers.length === 0) {
      container.handlers.delete(registration.key);
    }
  }
  instance.handlerRegistrations = [];
}

function eventRouteKeys(nodeId, kind, actionId) {
  return [
    `${nodeId}:${kind}`,
    `${nodeId}:*`,
    actionId == null ? null : `action:${actionId}:${kind}`,
    actionId == null ? null : `action:${actionId}:*`,
  ].filter(Boolean);
}

function describeEventHandlerRegistry(container) {
  const keys = [...container.handlers.keys()].sort();
  const keyCounts = {};
  let registrationSlotCount = 0;
  const uniqueRegistrations = new Set();

  for (const key of keys) {
    const handlers = container.handlers.get(key) ?? [];
    keyCounts[key] = handlers.length;
    registrationSlotCount += handlers.length;
    for (const registration of handlers) {
      if (registration != null) {
        uniqueRegistrations.add(registration);
      }
    }
  }

  return {
    keyCounts,
    keys,
    registrationSlotCount,
    uniqueRegistrationCount: uniqueRegistrations.size,
  };
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
    handlers.push(registration.registration);
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
  if (descriptor.motion != null) {
    mutations.push({
      kind: "set_motion",
      node_id: descriptor.nodeId,
      motion: descriptor.motion,
    });
  }
  for (const child of descriptor.children ?? []) {
    appendDescriptorMotionMutations(child, mutations);
  }
}

function appendMotionMutation(nodeId, previousMotion, nextMotion, mutations) {
  if (nextMotion == null) {
    if (previousMotion == null) {
      return;
    }
    mutations.push({ kind: "clear_motion", node_id: nodeId });
    return;
  }

  mutations.push({
    kind: "set_motion",
    node_id: nodeId,
    motion: nextMotion,
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
    appendMotionMutation(
      lowered.root.nodeId,
      previousMotion,
      lowered.root.motion,
      container.pendingMutations,
    );
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

  enqueueCommitBatch(
    {
      version: metadataView.version,
      schema_fingerprint: metadataView.schemaFingerprint,
      mutations,
    },
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
  enqueueCommitBatch(
    {
      version: metadataView.version,
      schema_fingerprint: metadataView.schemaFingerprint,
      mutations: container.pendingMutations,
    },
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
  // The retained-tree event boundary is keyed by authored node identity plus
  // optional semantic action identity. A single logical handler registration
  // can be reachable through multiple keys, so dispatch must de-duplicate by
  // registration object identity rather than by route key.
  const candidates = eventRouteKeys(nodeId, kind, actionId);

  const dispatchedRegistrations = new Set();
  const value = eventValue(event);

  for (const key of candidates) {
    for (const registration of container.handlers.get(key) ?? []) {
      if (registration == null || typeof registration.handler !== "function") {
        continue;
      }
      if (dispatchedRegistrations.has(registration)) {
        continue;
      }
      dispatchedRegistrations.add(registration);
      withRuntimeErrorCapture(() => {
        registration.handler(event, value);
      });
      if (pendingRuntimeError != null) {
        return;
      }
    }
  }
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
      parentContainer.loweredChildIndices = null;
      parentContainer.rootSyncRequired = true;
    });
  },
  commitTextUpdate(textInstance, _oldText, newText) {
    withRuntimeErrorCapture(() => {
      textInstance.text = String(newText);
      invalidateLoweredChildIndices(textInstance.parent);
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
    // V1 intentionally does not surface direct native host refs. JS public
    // instances remain shadow objects only.
    return null;
  },
  getInstanceFromScope() {
    // V1 intentionally does not surface direct native host refs. JS public
    // instances remain shadow objects only.
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
      invalidateLoweredChildIndices(instance.parent);
      removeMaterializedSubtree(container, instance.parent, instance);
    });
  },
  hideTextInstance(textInstance) {
    withRuntimeErrorCapture(() => {
      textInstance.hidden = true;
      invalidateLoweredChildIndices(textInstance.parent);
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
    return captureRuntimeError(() => {
      const family = normalizedFamilyForType(type, props ?? {}, metadataView);
      return familyUsesTextContent(family);
    }, false);
  },
  supportsHydration: false,
  supportsMicrotasks: true,
  supportsMutation: true,
  supportsPersistence: false,
  trackSchedulerEvent() {},
  unhideInstance(instance) {
    withRuntimeErrorCapture(() => {
      instance.hidden = false;
      invalidateLoweredChildIndices(instance.parent);
      reconcileMaterializedNode(container, instance);
    });
  },
  unhideTextInstance(textInstance) {
    withRuntimeErrorCapture(() => {
      textInstance.hidden = false;
      invalidateLoweredChildIndices(textInstance.parent);
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
  loweredChildIndices: null,
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
  (error, errorInfo) => {
    reportReconcilerError("uncaught", error, errorInfo);
  },
  (error, errorInfo) => {
    reportReconcilerError("caught", error, errorInfo);
  },
  (error, errorInfo) => {
    reportReconcilerError("recoverable", error, errorInfo);
  },
  () => {},
);

function resetHostConfigStateForTest() {
  for (const child of container.children ?? []) {
    clearMaterializedMetadata(container, child);
  }
  container.children = [];
  container.dirtyParents = new Set();
  container.handlers = new Map();
  container.loweredChildIndices = null;
  container.pendingMutations = [];
  container.rootNodeId = null;
  container.rootSyncRequired = false;
  container.syntheticRoot = false;

  resetCommitProtocolState();
  recoveryState = createRecoveryState();
  pendingRuntimeError = null;
  rootRendered = false;
  allowEmptyCommit = false;
}

globalThis.__eguiHostConfigForTest = hostConfig;
globalThis.__eguiContainerForTest = container;
globalThis.__eguiResetHostConfigStateForTest = resetHostConfigStateForTest;
globalThis.__eguiDispatchEventForTest = function dispatchEguiEventForTest(event) {
  dispatchEvent(container, event ?? {});
};
globalThis.__eguiDescribeEventHandlerRegistryForTest =
  function describeEguiEventHandlerRegistryForTest() {
    return describeEventHandlerRegistry(container);
  };
globalThis.__eguiDescribeHostConfigForTest = function describeEguiHostConfigForTest() {
  const hostInstance = hostConfig.createInstance("div", {
    id: "shadow-root",
    "data-slot": "column",
  });
  const textInstance = hostConfig.createTextInstance("shadow text");
  const publicHostInstance = hostConfig.getPublicInstance(hostInstance);
  const publicTextInstance = hostConfig.getPublicInstance(textInstance);
  return {
    publicHostInstanceKeys: Object.keys(publicHostInstance).sort(),
    publicHostInstanceKind: publicHostInstance?.kind ?? null,
    publicHostInstanceSameObject: publicHostInstance === hostInstance,
    publicTextInstanceKeys: Object.keys(publicTextInstance).sort(),
    publicTextInstanceKind: publicTextInstance?.kind ?? null,
    publicTextInstanceSameObject: publicTextInstance === textInstance,
    supportsHydration: hostConfig.supportsHydration,
    supportsMicrotasks: hostConfig.supportsMicrotasks,
    supportsMutation: hostConfig.supportsMutation,
    supportsPersistence: hostConfig.supportsPersistence,
    getInstanceFromNodeResult: hostConfig.getInstanceFromNode({}),
    getInstanceFromScopeResult: hostConfig.getInstanceFromScope({}),
  };
};

export function render(element) {
  throwIfRecoveryStateBlocksWork();
  throwIfCommitProtocolFailed();
  pendingRuntimeError = null;
  hostOps.noteRender();
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
    hostOps.noteUnmount();
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
    container.loweredChildIndices = null;
    resetCommitProtocolState();
    container.rootNodeId = null;
    container.rootSyncRequired = false;
    container.syntheticRoot = false;
    recoveryState = createRecoveryState();
    rootRendered = false;
    throwPendingRuntimeError();
  } finally {
    allowEmptyCommit = false;
  }
};

globalThis.__eguiDispatchEvents = function dispatchEguiEvents(events) {
  throwIfRecoveryStateBlocksWork();
  throwIfCommitProtocolFailed();
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

globalThis.__eguiBenchmarkCommitOnlyForTest = function benchmarkEguiCommitOnlyForTest(
  iterations = 1,
) {
  throwIfRecoveryStateBlocksWork();
  throwIfCommitProtocolFailed();
  if (!rootRendered) {
    throw new Error("render(<... />) must be called before benchmarking commits.");
  }

  const commitIterations = Number(iterations);
  if (!Number.isSafeInteger(commitIterations) || commitIterations <= 0) {
    throw new Error(
      `commit benchmark iterations must be a positive safe integer, got ${String(iterations)}.`,
    );
  }

  const renderStepForBenchmark = globalThis.__eguiCommitBenchmarkRenderStepForTest;
  const mutateForBenchmark = globalThis.__eguiCommitBenchmarkMutateForTest;
  if (
    typeof renderStepForBenchmark !== "function" &&
    typeof mutateForBenchmark !== "function"
  ) {
    throw new Error(
      "globalThis.__eguiCommitBenchmarkRenderStepForTest or globalThis.__eguiCommitBenchmarkMutateForTest must be a function before benchmarking commits.",
    );
  }

  const nowMs = () => Number(hostOps.nowMs());

  const commitDurationsMs = [];
  for (let index = 0; index < commitIterations; index += 1) {
    pendingRuntimeError = null;
    const start = nowMs();
    reconciler.flushSyncFromReconciler(() => {
      if (typeof renderStepForBenchmark === "function") {
        const nextElement = renderStepForBenchmark();
        reconciler.updateContainerSync(nextElement, root, null, null);
        return;
      }
      mutateForBenchmark();
    });
    const elapsedMs = nowMs() - start;
    throwPendingRuntimeError();
    commitDurationsMs.push(elapsedMs);
  }

  return commitDurationsMs;
};
