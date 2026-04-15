import React from "react";
import Reconciler from "clay-internal:/react-reconciler";

type ReconcilerErrorCategory = "uncaught" | "caught" | "recoverable";

type RuntimeAction = {
  code: string | null;
  ok: boolean;
};

type CommitBatch = {
  commands: RuntimeCommand[];
  version: number;
};

type RuntimeCommand =
  | {
      id: number;
      op: "remove";
    }
  | {
      beforeId: number | null;
      id: number;
      op: "createText";
      parentId: number;
      text: string;
    }
  | {
      beforeId: number | null;
      node: RuntimeNodeSpec;
      op: "create";
      parentId: number;
    }
  | {
      beforeId: number | null;
      id: number;
      op: "insert";
      parentId: number;
    }
  | {
      node: RuntimeNodeSpec;
      op: "update";
    }
  | {
      id: number;
      op: "setText";
      text: string;
    }
  | {
      op: "reset";
    };

type RuntimeNodeSpec = {
  class?: string;
  control?: RuntimeControl;
  element: string;
  listen?: string[];
  runtimeId: number;
  [key: string]: unknown;
};

type RuntimeControl = {
  disabled?: boolean;
  [key: string]: unknown;
};

type RuntimeDispatchEvent = {
  id: number;
  kind: number;
  payload: unknown;
  target: { id: number };
};

type RuntimeDispatchHandler = (event: RuntimeDispatchEvent) => void;

type RuntimeTextNode = {
  children?: RuntimeNode[];
  hidden: boolean;
  id: number;
  kind: "text";
  mounted: boolean;
  parent: RuntimeParent | null;
  siblingIndex: number;
  text: string;
};

type RuntimeHostNode = {
  children: RuntimeNode[];
  element: string;
  hidden: boolean;
  id: number;
  kind: "host";
  mounted: boolean;
  parent: RuntimeParent | null;
  props: PrimitiveProps;
  siblingIndex: number;
  type: string;
};

type RuntimeNode = RuntimeHostNode | RuntimeTextNode;

type RuntimeParent = RuntimeContainer | RuntimeHostNode;

type RuntimePendingReconcilerError = {
  category: ReconcilerErrorCategory;
  message: string;
};

type RuntimeRecoveryCategory =
  | "recoverable"
  | "boundary_contained"
  | "fatal"
  | "protocol_failed";

type RuntimeRecoveryDisposition =
  | "continue"
  | "bounded_failure"
  | "reload_required"
  | "teardown";

type RuntimeRecoveryState = {
  category: RuntimeRecoveryCategory | null;
  component_stack: string;
  disposition: RuntimeRecoveryDisposition;
  error_boundary: string | null;
  message: string | null;
  rejected_commit_batch_id: number | null;
};

type RuntimeContainer = {
  children: RuntimeNode[];
  commitPatch: (batch: CommitBatch) => unknown;
  eventProps: Map<string, [string, number]>;
  handlers: Map<number, Map<number, RuntimeDispatchHandler>>;
  kind: "container";
  lastAction: RuntimeAction;
  pending: RuntimeCommand[];
  pendingReconcilerError: RuntimePendingReconcilerError | null;
  recoveryState: RuntimeRecoveryState;
};

type ReconcilerErrorBoundary =
  | string
  | {
      constructor?: {
        displayName?: string | null;
        name?: string | null;
      } | null;
      displayName?: string | null;
      name?: string | null;
    };

type ReconcilerErrorInfo = {
  componentStack?: unknown;
  errorBoundary?: ReconcilerErrorBoundary | null;
};

type SignalLike<T = unknown> = {
  subscribe: (notify: () => void) => () => void;
  value: T;
};

type PrimitiveProps = {
  children?: React.ReactNode;
} & Record<string, unknown>;

type ClayJsxRuntimeOptions = {
  commitPatch: (batch: CommitBatch) => unknown;
  eventProps: Map<string, [string, number]>;
  hostTags: Map<string, string>;
};

function toErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

function toComponentStack(errorInfo: ReconcilerErrorInfo): string {
  return typeof errorInfo.componentStack === "string" ? errorInfo.componentStack : "";
}

function toErrorBoundaryLabel(errorInfo: ReconcilerErrorInfo): string {
  const boundary = errorInfo.errorBoundary;
  if (boundary == null) {
    return "";
  }
  if (typeof boundary === "string") {
    return boundary;
  }
  if (typeof boundary.displayName === "string" && boundary.displayName.length > 0) {
    return boundary.displayName;
  }
  if (typeof boundary.name === "string" && boundary.name.length > 0) {
    return boundary.name;
  }
  const ctor = boundary.constructor;
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

function isReconcilerErrorCategory(category: unknown): category is ReconcilerErrorCategory {
  return category === "uncaught" || category === "caught" || category === "recoverable";
}

function normalizeReconcilerErrorCategory(category: unknown): ReconcilerErrorCategory {
  if (isReconcilerErrorCategory(category)) {
    return category;
  }
  throw new Error(`Unknown reconciler error category ${String(category)}.`);
}

function createRecoveryState(): RuntimeRecoveryState {
  return {
    category: null,
    component_stack: "",
    disposition: "continue",
    error_boundary: null,
    message: null,
    rejected_commit_batch_id: null,
  };
}

function recoveryDispositionPriority(disposition: RuntimeRecoveryDisposition): number {
  switch (disposition) {
    case "continue":
      return 0;
    case "bounded_failure":
      return 1;
    case "teardown":
      return 2;
    case "reload_required":
      return 3;
  }
}

function observeRecoveryState(
  container: RuntimeContainer,
  nextState: Partial<RuntimeRecoveryState> & Pick<RuntimeRecoveryState, "disposition">,
) {
  const candidate: RuntimeRecoveryState = {
    category: nextState.category ?? null,
    component_stack: typeof nextState.component_stack === "string" ? nextState.component_stack : "",
    disposition: nextState.disposition,
    error_boundary: nextState.error_boundary == null || String(nextState.error_boundary).length === 0
      ? null
      : String(nextState.error_boundary),
    message: nextState.message == null ? null : String(nextState.message),
    rejected_commit_batch_id: nextState.rejected_commit_batch_id == null
      ? null
      : Number(nextState.rejected_commit_batch_id),
  };

  if (
    recoveryDispositionPriority(candidate.disposition) <
      recoveryDispositionPriority(container.recoveryState.disposition)
  ) {
    return;
  }

  container.recoveryState = candidate;
}

function isRuntimeAction(value: unknown): value is RuntimeAction {
  const action = value as { code?: unknown; ok?: unknown };
  return typeof value === "object"
    && value !== null
    && !Array.isArray(value)
    && typeof action.ok === "boolean"
    && (
      action.code === null ||
      typeof action.code === "string"
    );
}

function backendRejectedAction(error: unknown = null, fallbackCode = "backendRejected"): RuntimeAction {
  if (error == null) {
    return { ok: false, code: fallbackCode };
  }
  return {
    ok: false,
    code: `${fallbackCode}:${toErrorMessage(error)}`,
  };
}

export function action(code: string | null = null): RuntimeAction {
  return { ok: code === null, code };
}

export function nativeAction(encoded: string, fallbackCode = "backendRejected"): RuntimeAction {
  try {
    const parsed: unknown = JSON.parse(encoded);
    if (isRuntimeAction(parsed)) {
      return parsed;
    }
  } catch (error) {
    return backendRejectedAction(error, fallbackCode);
  }
  return backendRejectedAction(null, fallbackCode);
}

function reportReconcilerError(
  category: ReconcilerErrorCategory,
  error: unknown,
  errorInfo: ReconcilerErrorInfo,
  container: RuntimeContainer,
) {
  const message = toErrorMessage(error);
  const componentStack = toComponentStack(errorInfo);
  const errorBoundary = category === "caught" ? toErrorBoundaryLabel(errorInfo) : "";

  const reportOp = globalThis?.Deno?.core?.ops?.op_host_report_reconciler_error;
  if (typeof reportOp === "function") {
    reportOp(category, message, componentStack, errorBoundary);
  }

  if (category === "recoverable") {
    observeRecoveryState(container, {
      category: "recoverable",
      component_stack: componentStack,
      disposition: "continue",
      error_boundary: errorBoundary,
      message,
    });
    return;
  }

  observeRecoveryState(container, {
    category: category === "caught" && errorBoundary.length > 0 ? "boundary_contained" : "fatal",
    component_stack: componentStack,
    disposition: category === "caught" && errorBoundary.length > 0 ? "bounded_failure" : "teardown",
    error_boundary: errorBoundary,
    message,
  });

  if (container.pendingReconcilerError == null) {
    container.pendingReconcilerError = { category, message };
  }
}

function consumePendingReconcilerFailure(container: RuntimeContainer): RuntimeAction | null {
  const pendingError = container.pendingReconcilerError;
  if (pendingError == null) {
    return null;
  }
  container.pendingReconcilerError = null;
  container.pending = [];
  const result = action(`reconciler:${pendingError.category}:${pendingError.message}`);
  container.lastAction = result;
  return result;
}

function hasSignalShape(value: unknown): value is SignalLike {
  return value !== null
    && typeof value === "object"
    && "value" in value
    && "subscribe" in value
    && typeof (value as { subscribe?: unknown }).subscribe === "function";
}

function SignalValue({ signal }: { signal: SignalLike }) {
  const value = React.useSyncExternalStore(
    (notify) => signal.subscribe(notify),
    () => signal.value,
    () => signal.value,
  );
  if (value === null || value === undefined || value === false) {
    return null;
  }
  return String(value);
}

function resolveSignalValue(value: unknown): unknown {
  if (hasSignalShape(value)) {
    return value.value;
  }
  return value;
}

function wrapSignalChild(child: unknown): React.ReactNode {
  if (hasSignalShape(child)) {
    return React.createElement(SignalValue, { signal: child });
  }
  if (Array.isArray(child)) {
    return child.map(wrapSignalChild);
  }
  return child as React.ReactNode;
}

export function primitive(tag: React.ElementType, props?: PrimitiveProps | null) {
  const nextProps: PrimitiveProps = props ?? {};
  return React.createElement(tag, nextProps, wrapSignalChild(nextProps.children));
}

function isTextNode(node) {
  return node?.kind === "text";
}

function isContainer(value) {
  return value?.kind === "container";
}

function rootFor(value) {
  let current = value;
  while (current && !isContainer(current)) {
    current = current.parent;
  }
  return current ?? null;
}

function parentId(parent) {
  return isContainer(parent) ? 0 : parent.id;
}

function isParentMounted(parent) {
  return isContainer(parent) || parent.mounted === true;
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
  if (child.parent) {
    detachLinkedChild(child.parent, child);
  }
}

function linkChild(parent, child, beforeChild = null) {
  detachFromCurrentParent(child);
  child.parent = parent;
  const children = parent.children ?? [];
  const index = beforeChild ? children.indexOf(beforeChild) : -1;
  if (index >= 0) {
    children.splice(index, 0, child);
  } else {
    children.push(child);
  }
}

function mountedBeforeId(parent, child) {
  const children = parent.children ?? [];
  const index = children.indexOf(child);
  if (index < 0) {
    return null;
  }
  for (let i = index + 1; i < children.length; i += 1) {
    if (children[i].mounted === true) {
      return children[i].id;
    }
  }
  return null;
}

function deleteHandlers(container, node) {
  container.handlers.delete(node.id);
  if (node.children) {
    for (const child of node.children) {
      deleteHandlers(container, child);
    }
  }
}

function markSubtreeUnmounted(container, node) {
  node.mounted = false;
  deleteHandlers(container, node);
  if (node.children) {
    for (const child of node.children) {
      markSubtreeUnmounted(container, child);
    }
  }
}

function queue(container, command) {
  container.pending.push(command);
}

function removeMaterializedSubtree(container, node) {
  if (node.mounted === true) {
    queue(container, { op: "remove", id: node.id });
  }
  markSubtreeUnmounted(container, node);
}

function materializeSubtree(container, node, nextParentId, beforeId = null) {
  if (node.hidden === true) {
    markSubtreeUnmounted(container, node);
    return;
  }
  if (isTextNode(node)) {
    queue(container, {
      op: "createText",
      id: node.id,
      parentId: nextParentId,
      beforeId,
      text: node.text,
    });
    node.mounted = true;
    return;
  }
  queue(container, {
    op: "create",
    parentId: nextParentId,
    beforeId,
    node: nodeSpec(node, container),
  });
  node.mounted = true;
  for (const child of node.children) {
    materializeSubtree(container, child, node.id, null);
  }
}

function reconcileLinkedChild(parent, child, beforeId = null) {
  const container = rootFor(parent);
  if (!container) {
    return;
  }
  if (!isParentMounted(parent) || child.hidden === true) {
    if (child.mounted === true) {
      removeMaterializedSubtree(container, child);
    }
    return;
  }
  const nextParentId = parentId(parent);
  if (child.mounted === true) {
    queue(container, { op: "insert", id: child.id, parentId: nextParentId, beforeId });
  } else {
    materializeSubtree(container, child, nextParentId, beforeId);
  }
}

function appendLinkedChild(parent, child) {
  const previousContainer = rootFor(child);
  linkChild(parent, child, null);
  const container = rootFor(parent);
  if (!container) {
    if (previousContainer && child.mounted === true) {
      removeMaterializedSubtree(previousContainer, child);
    }
    return;
  }
  reconcileLinkedChild(parent, child, null);
}

function insertLinkedChild(parent, child, beforeChild) {
  const previousContainer = rootFor(child);
  linkChild(parent, child, beforeChild);
  const container = rootFor(parent);
  if (!container) {
    if (previousContainer && child.mounted === true) {
      removeMaterializedSubtree(previousContainer, child);
    }
    return;
  }
  reconcileLinkedChild(parent, child, beforeChild?.id ?? null);
}

function removeLinkedChild(parent, child) {
  const container = rootFor(parent) ?? rootFor(child);
  detachLinkedChild(parent, child);
  if (container) {
    removeMaterializedSubtree(container, child);
  }
}

function copyProp(spec, props, source, target = source) {
  const value = resolveSignalValue(props[source]);
  if (value !== undefined && typeof value !== "function") {
    spec[target] = value;
  }
}

function collectEvents(id, props, container) {
  const listen = [];
  const handlers = new Map();
  for (const [prop, [name, kind]] of container.eventProps) {
    const handler = props[prop];
    if (typeof handler !== "function") {
      continue;
    }
    listen.push(name);
    handlers.set(kind, handler);
  }
  if (handlers.size > 0) {
    container.handlers.set(id, handlers);
  } else {
    container.handlers.delete(id);
  }
  return listen;
}

function nodeSpec(node, container) {
  const props = node.props ?? {};
  const spec = {
    element: node.element,
    runtimeId: node.id,
  };
  const className = resolveSignalValue(props.className ?? props.class);
  if (className !== undefined && className !== null && className !== false) {
    spec.class = String(className);
  }
  for (const [source, target] of [
    ["key", "key"],
    ["hidden", "hidden"],
    ["testId", "testId"],
    ["automationId", "automationId"],
    ["src", "src"],
    ["blend", "blend"],
    ["image", "image"],
    ["sprite", "sprite"],
    ["surface", "surface"],
    ["value", "value"],
    ["placeholder", "placeholder"],
    ["control", "control"],
    ["text", "text"],
    ["bg", "bg"],
    ["layout", "layout"],
    ["scale", "scale"],
    ["translate", "translate"],
    ["motion", "motion"],
    ["flex", "flex"],
    ["grid", "grid"],
    ["margin", "margin"],
    ["pad", "pad"],
    ["border", "border"],
    ["rounded", "rounded"],
    ["opacity", "opacity"],
    ["z", "z"],
    ["clip", "clip"],
    ["overflow", "overflow"],
    ["clipStrategy", "clipStrategy"],
    ["classExtra", "classExtra"],
  ]) {
    copyProp(spec, props, source, target);
  }
  if (props.disabled === true) {
    spec.control = { ...(spec.control && typeof spec.control === "object" ? spec.control : {}), disabled: true };
  }
  const listen = collectEvents(node.id, props, container);
  if (listen.length > 0) {
    spec.listen = listen;
  }
  return spec;
}

function flushPending(container: RuntimeContainer): RuntimeAction {
  const reconcilerFailure = consumePendingReconcilerFailure(container);
  if (reconcilerFailure != null) {
    return reconcilerFailure;
  }
  if (container.pending.length === 0) {
    return action();
  }
  const commands = container.pending;
  container.pending = [];
  try {
    const result = container.commitPatch({ version: 1, commands });
    if (isRuntimeAction(result)) {
      return result;
    }
  } catch (error) {
    return backendRejectedAction(error);
  }
  return backendRejectedAction(null);
}

export function createClayJsxRuntime(options: ClayJsxRuntimeOptions) {
  let nextNodeId = 2;
  let currentUpdatePriority = 0;

  function hostElementFor(type) {
    const key = String(type).toLowerCase();
    const element = options.hostTags.get(key);
    if (!element) {
      throw new Error("Unsupported Clay UI host element: " + String(type));
    }
    return element;
  }

  function makeNode(type, props) {
    return {
      kind: "host",
      id: nextNodeId++,
      type: String(type),
      element: hostElementFor(type),
      props: props ?? {},
      children: [],
      parent: null,
      mounted: false,
      hidden: false,
    };
  }

  function makeTextNode(text) {
    return {
      kind: "text",
      id: nextNodeId++,
      text: String(text),
      parent: null,
      mounted: false,
      hidden: false,
    };
  }

  const hostConfig = {
    supportsMutation: true,
    supportsPersistence: false,
    supportsHydration: false,
    isPrimaryRenderer: true,
    noTimeout: -1,
    scheduleTimeout(fn, delay) {
      if (typeof setTimeout === "function") {
        return setTimeout(fn, delay);
      }
      queueMicrotask(fn);
      return -1;
    },
    cancelTimeout(id) {
      if (typeof clearTimeout === "function" && id !== -1) {
        clearTimeout(id);
      }
    },
    supportsMicrotasks: true,
    scheduleMicrotask(fn) {
      queueMicrotask(fn);
    },
    getRootHostContext() {
      return {};
    },
    getChildHostContext(parentHostContext) {
      return parentHostContext;
    },
    getPublicInstance(instance) {
      return instance;
    },
    prepareForCommit() {
      return null;
    },
    resetAfterCommit(container) {
      container.lastAction = flushPending(container);
    },
    preparePortalMount() {},
    createInstance(type, props) {
      return makeNode(type, props);
    },
    createTextInstance(text) {
      return makeTextNode(text);
    },
    appendInitialChild(parent, child) {
      linkChild(parent, child, null);
    },
    finalizeInitialChildren() {
      return false;
    },
    shouldSetTextContent() {
      return false;
    },
    appendChild(parent, child) {
      appendLinkedChild(parent, child);
    },
    appendChildToContainer(container, child) {
      appendLinkedChild(container, child);
    },
    insertBefore(parent, child, beforeChild) {
      insertLinkedChild(parent, child, beforeChild);
    },
    insertInContainerBefore(container, child, beforeChild) {
      insertLinkedChild(container, child, beforeChild);
    },
    removeChild(parent, child) {
      removeLinkedChild(parent, child);
    },
    removeChildFromContainer(container, child) {
      removeLinkedChild(container, child);
    },
    clearContainer(container) {
      for (const child of [...container.children]) {
        removeLinkedChild(container, child);
      }
      container.children = [];
    },
    commitUpdate(instance, _type, _prevProps, nextProps) {
      instance.props = nextProps ?? {};
      const container = rootFor(instance);
      if (container && instance.mounted === true && instance.hidden !== true) {
        queue(container, { op: "update", node: nodeSpec(instance, container) });
      }
    },
    commitTextUpdate(textInstance, _oldText, newText) {
      textInstance.text = String(newText);
      const container = rootFor(textInstance);
      if (container && textInstance.mounted === true && textInstance.hidden !== true) {
        queue(container, { op: "setText", id: textInstance.id, text: textInstance.text });
      }
    },
    resetTextContent(instance) {
      const container = rootFor(instance);
      if (container) {
        for (const child of [...instance.children]) {
          removeLinkedChild(instance, child);
        }
      }
      instance.children = [];
    },
    hideInstance(instance) {
      if (instance.hidden === true) {
        return;
      }
      instance.hidden = true;
      const container = rootFor(instance);
      if (container) {
        removeMaterializedSubtree(container, instance);
      }
    },
    unhideInstance(instance) {
      if (instance.hidden !== true) {
        return;
      }
      instance.hidden = false;
      const parent = instance.parent;
      const container = rootFor(instance);
      if (container && parent && isParentMounted(parent)) {
        materializeSubtree(container, instance, parentId(parent), mountedBeforeId(parent, instance));
      }
    },
    hideTextInstance(textInstance) {
      if (textInstance.hidden === true) {
        return;
      }
      textInstance.hidden = true;
      const container = rootFor(textInstance);
      if (container) {
        removeMaterializedSubtree(container, textInstance);
      }
    },
    unhideTextInstance(textInstance) {
      if (textInstance.hidden !== true) {
        return;
      }
      textInstance.hidden = false;
      const parent = textInstance.parent;
      const container = rootFor(textInstance);
      if (container && parent && isParentMounted(parent)) {
        materializeSubtree(container, textInstance, parentId(parent), mountedBeforeId(parent, textInstance));
      }
    },
    getInstanceFromNode() {
      return null;
    },
    beforeActiveInstanceBlur() {},
    afterActiveInstanceBlur() {},
    prepareScopeUpdate() {},
    getInstanceFromScope() {
      return null;
    },
    detachDeletedInstance() {},
    NotPendingTransition: null,
    HostTransitionContext: React.createContext(null),
    setCurrentUpdatePriority(priority) {
      currentUpdatePriority = priority;
    },
    getCurrentUpdatePriority() {
      return currentUpdatePriority;
    },
    resolveUpdatePriority() {
      return 16;
    },
    resetFormInstance() {},
    requestPostPaintCallback(callback) {
      queueMicrotask(() => callback(performance.now()));
    },
    shouldAttemptEagerTransition() {
      return false;
    },
    trackSchedulerEvent() {},
    resolveEventType() {
      return null;
    },
    resolveEventTimeStamp() {
      return performance.now();
    },
  };

  const reconciler = Reconciler(hostConfig);
  const container: RuntimeContainer = {
    kind: "container",
    children: [],
    handlers: new Map(),
    pending: [],
    pendingReconcilerError: null,
    recoveryState: createRecoveryState(),
    lastAction: action(),
    eventProps: options.eventProps,
    commitPatch: options.commitPatch,
  };
  const onUncaughtError = (error, errorInfo) => {
    reportReconcilerError("uncaught", error, errorInfo, container);
  };
  const onCaughtError = (error, errorInfo) => {
    reportReconcilerError("caught", error, errorInfo, container);
  };
  const onRecoverableError = (error, errorInfo) => {
    reportReconcilerError("recoverable", error, errorInfo, container);
  };
  const root = reconciler.createContainer(
    container,
    1,
    null,
    false,
    null,
    "",
    onUncaughtError,
    onCaughtError,
    onRecoverableError,
    () => {},
  );

  function render(element) {
    container.pendingReconcilerError = null;
    container.recoveryState = createRecoveryState();
    container.lastAction = action();
    reconciler.updateContainerSync(element, root, null, null);
    reconciler.flushSyncWork();
    const reconcilerFailure = consumePendingReconcilerFailure(container);
    if (reconcilerFailure != null) {
      return reconcilerFailure;
    }
    return container.lastAction ?? action();
  }

  function unmount() {
    container.pendingReconcilerError = null;
    container.recoveryState = createRecoveryState();
    container.lastAction = action();
    reconciler.updateContainerSync(null, root, null, null);
    reconciler.flushSyncWork();
    container.children = [];
    container.handlers = new Map();
    container.pending.push({ op: "reset" });
    container.lastAction = flushPending(container);
    const reconcilerFailure = consumePendingReconcilerFailure(container);
    if (reconcilerFailure != null) {
      return reconcilerFailure;
    }
    return container.lastAction;
  }

  function dispatch(event) {
    if (!event || event.type !== "ui_dispatch") {
      return false;
    }
    container.recoveryState = createRecoveryState();
    const nodeHandlers = container.handlers.get(Number(event.id));
    const handler = nodeHandlers?.get(Number(event.kind));
    if (typeof handler !== "function") {
      return false;
    }
    handler({
      kind: Number(event.kind),
      id: Number(event.id),
      payload: event.payload ?? null,
      target: { id: Number(event.id) },
    });
    const reconcilerFailure = consumePendingReconcilerFailure(container);
    if (reconcilerFailure != null) {
      return false;
    }
    return true;
  }

  function reportReconcilerErrorForTest(category, message, componentStack = "", errorBoundary = "") {
    const normalizedCategory = normalizeReconcilerErrorCategory(category);
    reportReconcilerError(
      normalizedCategory,
      new Error(String(message)),
      {
        componentStack: String(componentStack),
        errorBoundary: errorBoundary == null || String(errorBoundary).length === 0
          ? null
          : { displayName: String(errorBoundary) },
      },
      container,
    );
  }

  return {
    __describeRecoveryStateForTest() {
      return { ...container.recoveryState };
    },
    __reportReconcilerErrorForTest: reportReconcilerErrorForTest,
    dispatch,
    render,
    unmount,
  };
}
