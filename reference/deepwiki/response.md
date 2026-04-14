# DeepWiki Response

Repository: `facebook/react`
Question: In react-refresh, for a custom renderer/runtime that owns its own module loading and currently recreates the whole JS session on file change, what are the core architectural boundaries between the react-refresh transform/runtime layer and the module-graph/HMR host? Specifically: how are refresh boundaries identified, when is local state preserved versus forced remount or full reload, what dependency invalidation/tracking is required, and what minimum hooks/APIs would a custom non-DOM host need to implement to match React Fast Refresh semantics?
Search: https://deepwiki.com/search/in-reactrefresh-for-a-custom-r_0e090f18-8e2a-4351-b6f7-429d24ec7625

**Question**: In react-refresh, for a custom renderer/runtime that owns its own module loading and currently recreates the whole JS session on file change, what are the core architectural boundaries between the react-refresh transform/runtime layer and the module-graph/HMR host? Specifically: how are refresh boundaries identified, when is local state preserved versus forced remount or full reload, what dependency invalidation/tracking is required, and what minimum hooks/APIs would a custom non-DOM host need to implement to match React Fast Refresh semantics?

The user is asking about the architectural boundaries and required APIs for integrating React Fast Refresh into a custom renderer/runtime that manages its own module loading and HMR. This involves understanding how refresh boundaries are identified, when state is preserved or reset, and the necessary dependency tracking.

React Fast Refresh's core architectural boundaries lie between the `react-refresh/babel` transform, the `ReactFreshRuntime` in `packages/react-refresh/src/ReactFreshRuntime.js`, and the React Reconciler's hot reloading mechanisms in `packages/react-reconciler/src/ReactFiberHotReloading.js` [1](#ref-q1-1)  [2](#ref-q1-2) . The module-graph/HMR host is responsible for detecting file changes, re-executing modules, and triggering the refresh process through `ReactFreshRuntime.performReactRefresh()` [3](#ref-q1-3) .

### Refresh Boundary Identification

Refresh boundaries are primarily identified by the `react-refresh/babel` transform, which injects calls to `ReactFreshRuntime.register()` and `ReactFreshRuntime.setSignature()` [4](#ref-q1-4)  [5](#ref-q1-5) .

*   **`$RefreshReg$`**: This function, mapped to `ReactFreshRuntime.register(type, id)` [6](#ref-q1-6) , associates a component `type` with a unique `id` [4](#ref-q1-4) . This creates a "family" for the component, allowing the runtime to track its different versions across updates [7](#ref-q1-7) .
*   **`$RefreshSig$`**: This function, mapped to `ReactFreshRuntime.createSignatureFunctionForTransform()` [8](#ref-q1-8) , is used to generate a "signature" for function components, especially those using Hooks [9](#ref-q1-9) . The signature includes an `ownKey` and a `fullKey` (computed lazily) that incorporates the keys of nested Hooks [10](#ref-q1-10)  [11](#ref-q1-11) . This signature is crucial for determining if the Hook order has changed, which would necessitate a remount [12](#ref-q1-12) .

### State Preservation vs. Forced Remount/Full Reload

React Fast Refresh attempts to preserve component state whenever possible [13](#ref-q1-13) .

*   **State Preservation**: State is preserved if `canPreserveStateBetween(prevType, nextType)` returns `true` [14](#ref-q1-14) . This typically happens when the component's signature (including its Hooks) remains the same between edits [15](#ref-q1-15) . The `updatedFamilies` set in the `RefreshUpdate` object contains components that will re-render while preserving state [16](#ref-q1-16) .
*   **Forced Remount**: A component is remounted (its state is reset) if:
    *   It's a class component [17](#ref-q1-17)  [18](#ref-q1-18) .
    *   Its signature changes, indicating a change in Hook order or usage [19](#ref-q1-19)  [20](#ref-q1-20) .
    *   The `forceReset` flag is explicitly set in its signature [21](#ref-q1-21)  [22](#ref-q1-22) .
    *   A special `/* @refresh reset */` annotation is present in the source code [23](#ref-q1-23) .
    The `staleFamilies` set in the `RefreshUpdate` object contains components that will be remounted [24](#ref-q1-24) .
*   **Full Reload**: If a module's exports change in a way that cannot be handled by Fast Refresh (e.g., converting a function component to a class component), the HMR host might need to propagate the update upwards and re-execute modules that imported it, potentially leading to a full reload of parts of the application [25](#ref-q1-25) .

### Dependency Invalidation/Tracking

The module-graph/HMR host is responsible for:

1.  **Detecting File Changes**: The host needs to monitor file system changes for JavaScript/TypeScript files.
2.  **Re-executing Modules**: When a file changes, the host should re-execute the modified module. This re-execution will call `ReactFreshRuntime.register()` and `ReactFreshRuntime.setSignature()` again for the components within that module, updating their "families" and "signatures" [26](#ref-q1-26) .
3.  **Triggering Refresh**: After re-executing the module(s), the host must call `ReactFreshRuntime.performReactRefresh()` [27](#ref-q1-27) . This function then determines which components need to be updated or remounted based on the collected families and signatures [28](#ref-q1-28) .

### Minimum Hooks/APIs for a Custom Non-DOM Host

A custom non-DOM host would need to implement the following APIs, which are exposed via the DevTools global hook (`__REACT_DEVTOOLS_GLOBAL_HOOK__`) [29](#ref-q1-29) :

*   **`inject(renderer)`**: The renderer calls this to register itself with the global hook. The `ReactFreshRuntime` then checks if the injected `renderer` object provides `scheduleRefresh` and `setRefreshHandler` [30](#ref-q1-30) .
*   **`scheduleRefresh(root: FiberRoot, update: RefreshUpdate)`**: This function, provided by the renderer, is called by `ReactFreshRuntime.performReactRefresh()` to schedule a refresh for a specific React root [31](#ref-q1-31) . The `update` object contains `updatedFamilies` (for state preservation) and `staleFamilies` (for remounting) [32](#ref-q1-32) .
*   **`setRefreshHandler(handler: RefreshHandler | null)`**: This function, also provided by the renderer, is used by `ReactFreshRuntime` to set a handler that can resolve a component `type` to its "family" [33](#ref-q1-33) . This allows the React reconciler to use the latest known implementation of a component during rendering [34](#ref-q1-34) .
*   **`scheduleRoot(root: FiberRoot, element: ReactNodeList)`**: This is used to retry rendering a root that previously failed to mount, typically after an error [35](#ref-q1-35) .
*   **`onCommitFiberRoot`**: The renderer should call this hook when a React root commits [36](#ref-q1-36) . This allows `ReactFreshRuntime` to track mounted roots and associate them with the correct renderer helpers [37](#ref-q1-37) .
*   **`onScheduleFiberRoot`**: The renderer should call this hook when a React root is scheduled [38](#ref-q1-38) . This helps `ReactFreshRuntime` track the elements associated with roots, especially for retrying failed roots [39](#ref-q1-39) .

The custom host would also need to provide a mechanism for the `react-refresh/babel` plugin to inject the `$RefreshReg$` and `$RefreshSig$` calls into the transformed code, and then execute that transformed code in a way that updates the `ReactFreshRuntime`'s internal maps [40](#ref-q1-40) .

## Notes

The provided context primarily focuses on the `react-refresh` runtime and its integration with React DevTools and the React Reconciler. The examples in the test files `packages/react-refresh/src/__tests__/ReactFreshIntegration-test.js` [41](#ref-q1-41)  and `packages/react-devtools-shared/src/__tests__/FastRefreshDevToolsIntegration-test.js` [42](#ref-q1-42)  demonstrate how a module system would interact with `ReactFreshRuntime` by calling `register`, `createSignatureFunctionForTransform`, and `performReactRefresh`. The `ReactFiberHotReloading.js` file [2](#ref-q1-2)  shows the reconciler's side of the integration, specifically how it uses the `resolveFamily` handler to get the latest component implementation.

Wiki pages you might want to explore:
- [DevTools Frontend and Distribution (facebook/react)](/wiki/facebook/react#7.2)

View this search on DeepWiki: https://deepwiki.com/search/in-reactrefresh-for-a-custom-r_0e090f18-8e2a-4351-b6f7-429d24ec7625

## References

<a id="ref-q1-1"></a>
### [1] `packages/react-refresh/src/ReactFreshRuntime.js:1-547`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L1-L547)

```javascript
/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 *
 * @flow
 */

import type {FiberRoot} from 'react-reconciler/src/ReactInternalTypes';
import type {
  Family,
  RefreshUpdate,
  ScheduleRefresh,
  ScheduleRoot,
  SetRefreshHandler,
} from 'react-reconciler/src/ReactFiberHotReloading';
import type {ReactNodeList} from 'shared/ReactTypes';

import {REACT_MEMO_TYPE, REACT_FORWARD_REF_TYPE} from 'shared/ReactSymbols';

type Signature = {
  ownKey: string,
  forceReset: boolean,
  fullKey: string | null, // Contains keys of nested Hooks. Computed lazily.
  getCustomHooks: () => Array<Function>,
};

type RendererHelpers = {
  scheduleRefresh: ScheduleRefresh,
  scheduleRoot: ScheduleRoot,
  setRefreshHandler: SetRefreshHandler,
};

if (!__DEV__) {
  throw new Error(
    'React Refresh runtime should not be included in the production bundle.',
  );
}

// In old environments, we'll leak previous types after every edit.
const PossiblyWeakMap = typeof WeakMap === 'function' ? WeakMap : Map;

// We never remove these associations.
// It's OK to reference families, but use WeakMap/Set for types.
const allFamiliesByID: Map<string, Family> = new Map();
const allFamiliesByType: WeakMap<any, Family> | Map<any, Family> =
  new PossiblyWeakMap();
const allSignaturesByType: WeakMap<any, Signature> | Map<any, Signature> =
  new PossiblyWeakMap();
// This WeakMap is read by React, so we only put families
// that have actually been edited here. This keeps checks fast.
const updatedFamiliesByType: WeakMap<any, Family> | Map<any, Family> =
  new PossiblyWeakMap();

// This is cleared on every performReactRefresh() call.
// It is an array of [Family, NextType] tuples.
let pendingUpdates: Array<[Family, any]> = [];

// This is injected by the renderer via DevTools global hook.
const helpersByRendererID: Map<number, RendererHelpers> = new Map();

const helpersByRoot: Map<FiberRoot, RendererHelpers> = new Map();

// We keep track of mounted roots so we can schedule updates.
const mountedRoots: Set<FiberRoot> = new Set();
// If a root captures an error, we remember it so we can retry on edit.
const failedRoots: Set<FiberRoot> = new Set();

// In environments that support WeakMap, we also remember the last element for every root.
// It needs to be weak because we do this even for roots that failed to mount.
// If there is no WeakMap, we won't attempt to do retrying.
const rootElements: WeakMap<any, ReactNodeList> | null =
  typeof WeakMap === 'function' ? new WeakMap() : null;

let isPerformingRefresh = false;

function computeFullKey(signature: Signature): string {
  if (signature.fullKey !== null) {
    return signature.fullKey;
  }

  let fullKey: string = signature.ownKey;
  let hooks;
  try {
    hooks = signature.getCustomHooks();
  } catch (err) {
    // This can happen in an edge case, e.g. if expression like Foo.useSomething
    // depends on Foo which is lazily initialized during rendering.
    // In that case just assume we'll have to remount.
    signature.forceReset = true;
    signature.fullKey = fullKey;
    return fullKey;
  }

  for (let i = 0; i < hooks.length; i++) {
    const hook = hooks[i];
    if (typeof hook !== 'function') {
      // Something's wrong. Assume we need to remount.
      signature.forceReset = true;
      signature.fullKey = fullKey;
      return fullKey;
    }
    const nestedHookSignature = allSignaturesByType.get(hook);
    if (nestedHookSignature === undefined) {
      // No signature means Hook wasn't in the source code, e.g. in a library.
      // We'll skip it because we can assume it won't change during this session.
      continue;
    }
    const nestedHookKey = computeFullKey(nestedHookSignature);
    if (nestedHookSignature.forceReset) {
      signature.forceReset = true;
    }
    fullKey += '\n---\n' + nestedHookKey;
  }

  signature.fullKey = fullKey;
  return fullKey;
}

function haveEqualSignatures(prevType: any, nextType: any) {
  const prevSignature = allSignaturesByType.get(prevType);
  const nextSignature = allSignaturesByType.get(nextType);

  if (prevSignature === undefined && nextSignature === undefined) {
    return true;
  }
  if (prevSignature === undefined || nextSignature === undefined) {
    return false;
  }
  if (computeFullKey(prevSignature) !== computeFullKey(nextSignature)) {
    return false;
  }
  if (nextSignature.forceReset) {
    return false;
  }

  return true;
}

function isReactClass(type: any) {
  return type.prototype && type.prototype.isReactComponent;
}

function canPreserveStateBetween(prevType: any, nextType: any) {
  if (isReactClass(prevType) || isReactClass(nextType)) {
    return false;
  }
  if (haveEqualSignatures(prevType, nextType)) {
    return true;
  }
  return false;
}

function resolveFamily(type: any) {
  // Only check updated types to keep lookups fast.
  return updatedFamiliesByType.get(type);
}

// If we didn't care about IE11, we could use new Map/Set(iterable).
function cloneMap<K, V>(map: Map<K, V>): Map<K, V> {
  const clone = new Map<K, V>();
  map.forEach((value, key) => {
    clone.set(key, value);
  });
  return clone;
}
function cloneSet<T>(set: Set<T>): Set<T> {
  const clone = new Set<T>();
  set.forEach(value => {
    clone.add(value);
  });
  return clone;
}

// This is a safety mechanism to protect against rogue getters and Proxies.
function getProperty(object: any, property: string) {
  try {
    return object[property];
  } catch (err) {
    // Intentionally ignore.
    return undefined;
  }
}

export function performReactRefresh(): RefreshUpdate | null {
  if (!__DEV__) {
    throw new Error(
      'Unexpected call to React Refresh in a production environment.',
    );
  }
  if (pendingUpdates.length === 0) {
    return null;
  }
  if (isPerformingRefresh) {
    return null;
  }

  isPerformingRefresh = true;
  try {
    const staleFamilies = new Set<Family>();
    const updatedFamilies = new Set<Family>();

    const updates = pendingUpdates;
    pendingUpdates = [];
    updates.forEach(([family, nextType]) => {
      // Now that we got a real edit, we can create associations
      // that will be read by the React reconciler.
      const prevType = family.current;
      updatedFamiliesByType.set(prevType, family);
      updatedFamiliesByType.set(nextType, family);
      family.current = nextType;

      // Determine whether this should be a re-render or a re-mount.
      if (canPreserveStateBetween(prevType, nextType)) {
        updatedFamilies.add(family);
      } else {
        staleFamilies.add(family);
      }
    });

    // TODO: rename these fields to something more meaningful.
    const update: RefreshUpdate = {
      updatedFamilies, // Families that will re-render preserving state
      staleFamilies, // Families that will be remounted
    };

    helpersByRendererID.forEach(helpers => {
      // Even if there are no roots, set the handler on first update.
      // This ensures that if *new* roots are mounted, they'll use the resolve handler.
      helpers.setRefreshHandler(resolveFamily);
    });

    let didError = false;
    let firstError = null;

    // We snapshot maps and sets that are mutated during commits.
    // If we don't do this, there is a risk they will be mutated while
    // we iterate over them. For example, trying to recover a failed root
    // may cause another root to be added to the failed list -- an infinite loop.
    const failedRootsSnapshot = cloneSet(failedRoots);
    const mountedRootsSnapshot = cloneSet(mountedRoots);
    const helpersByRootSnapshot = cloneMap(helpersByRoot);

    failedRootsSnapshot.forEach(root => {
      const helpers = helpersByRootSnapshot.get(root);
      if (helpers === undefined) {
        throw new Error(
          'Could not find helpers for a root. This is a bug in React Refresh.',
        );
      }
      if (!failedRoots.has(root)) {
        // No longer failed.
      }
      if (rootElements === null) {
        return;
      }
      if (!rootElements.has(root)) {
        return;
      }
      const element = rootElements.get(root);
      try {
        helpers.scheduleRoot(root, element);
      } catch (err) {
        if (!didError) {
          didError = true;
          firstError = err;
        }
        // Keep trying other roots.
      }
    });
    mountedRootsSnapshot.forEach(root => {
      const helpers = helpersByRootSnapshot.get(root);
      if (helpers === undefined) {
        throw new Error(
          'Could not find helpers for a root. This is a bug in React Refresh.',
        );
      }
      if (!mountedRoots.has(root)) {
        // No longer mounted.
      }
      try {
        helpers.scheduleRefresh(root, update);
      } catch (err) {
        if (!didError) {
          didError = true;
          firstError = err;
        }
        // Keep trying other roots.
      }
    });
    if (didError) {
      throw firstError;
    }
    return update;
  } finally {
    isPerformingRefresh = false;
  }
}

export function register(type: any, id: string): void {
  if (__DEV__) {
    if (type === null) {
      return;
    }
    if (typeof type !== 'function' && typeof type !== 'object') {
      return;
    }

    // This can happen in an edge case, e.g. if we register
    // return value of a HOC but it returns a cached component.
    // Ignore anything but the first registration for each type.
    if (allFamiliesByType.has(type)) {
      return;
    }
    // Create family or remember to update it.
    // None of this bookkeeping affects reconciliation
    // until the first performReactRefresh() call above.
    let family = allFamiliesByID.get(id);
    if (family === undefined) {
      family = {current: type};
      allFamiliesByID.set(id, family);
    } else {
      pendingUpdates.push([family, type]);
    }
    allFamiliesByType.set(type, family);

    // Visit inner types because we might not have registered them.
    if (typeof type === 'object' && type !== null) {
      switch (getProperty(type, '$$typeof')) {
        case REACT_FORWARD_REF_TYPE:
          register(type.render, id + '$render');
          break;
        case REACT_MEMO_TYPE:
          register(type.type, id + '$type');
          break;
      }
    }
  } else {
    throw new Error(
      'Unexpected call to React Refresh in a production environment.',
    );
  }
}

export function setSignature(
  type: any,
  key: string,
  forceReset?: boolean = false,
  getCustomHooks?: () => Array<Function>,
): void {
  if (__DEV__) {
    if (!allSignaturesByType.has(type)) {
      allSignaturesByType.set(type, {
        forceReset,
        ownKey: key,
        fullKey: null,
        getCustomHooks: getCustomHooks || (() => []),
      });
    }
    // Visit inner types because we might not have signed them.
    if (typeof type === 'object' && type !== null) {
      switch (getProperty(type, '$$typeof')) {
        case REACT_FORWARD_REF_TYPE:
          setSignature(type.render, key, forceReset, getCustomHooks);
          break;
        case REACT_MEMO_TYPE:
          setSignature(type.type, key, forceReset, getCustomHooks);
          break;
      }
    }
  } else {
    throw new Error(
      'Unexpected call to React Refresh in a production environment.',
    );
  }
}

// This is lazily called during first render for a type.
// It captures Hook list at that time so inline requires don't break comparisons.
export function collectCustomHooksForSignature(type: any) {
  if (__DEV__) {
    const signature = allSignaturesByType.get(type);
    if (signature !== undefined) {
      computeFullKey(signature);
    }
  } else {
    throw new Error(
      'Unexpected call to React Refresh in a production environment.',
    );
  }
}

export function getFamilyByID(id: string): Family | void {
  if (__DEV__) {
    return allFamiliesByID.get(id);
  } else {
    throw new Error(
      'Unexpected call to React Refresh in a production environment.',
    );
  }
}

export function getFamilyByType(type: any): Family | void {
  if (__DEV__) {
    return allFamiliesByType.get(type);
  } else {
    throw new Error(
      'Unexpected call to React Refresh in a production environment.',
    );
  }
}

export function injectIntoGlobalHook(globalObject: any): void {
  if (__DEV__) {
    // For React Native, the global hook will be set up by require('react-devtools-core').
    // That code will run before us. So we need to monkeypatch functions on existing hook.

    // For React Web, the global hook will be set up by the extension.
    // This will also run before us.
    let hook = globalObject.__REACT_DEVTOOLS_GLOBAL_HOOK__;
    if (hook === undefined) {
      // However, if there is no DevTools extension, we'll need to set up the global hook ourselves.
      // Note that in this case it's important that renderer code runs *after* this method call.
      // Otherwise, the renderer will think that there is no global hook, and won't do the injection.
      let nextID = 0;
      globalObject.__REACT_DEVTOOLS_GLOBAL_HOOK__ = hook = {
        renderers: new Map(),
        supportsFiber: true,
        inject: injected => nextID++,
        onScheduleFiberRoot: (
          id: number,
          root: FiberRoot,
          children: ReactNodeList,
        ) => {},
        onCommitFiberRoot: (
          id: number,
          root: FiberRoot,
          maybePriorityLevel: mixed,
          didError: boolean,
        ) => {},
        onCommitFiberUnmount() {},
      };
    }

    if (hook.isDisabled) {
      // This isn't a real property on the hook, but it can be set to opt out
      // of DevTools integration and associated warnings and logs.
      // Using console['warn'] to evade Babel and ESLint
      console['warn'](
        'Something has shimmed the React DevTools global hook (__REACT_DEVTOOLS_GLOBAL_HOOK__). ' +
          'Fast Refresh is not compatible with this shim and will be disabled.',
      );
      return;
    }

    // Here, we just want to get a reference to scheduleRefresh.
    const oldInject = hook.inject;
    hook.inject = function (this: mixed, injected) {
      const id = oldInject.apply(this, arguments);
      if (
        typeof injected.scheduleRefresh === 'function' &&
        typeof injected.setRefreshHandler === 'function'
      ) {
        // This version supports React Refresh.
        helpersByRendererID.set(id, ((injected: any): RendererHelpers));
      }
      return id;
    };

    // Do the same for any already injected roots.
    // This is useful if ReactDOM has already been initialized.
    // https://github.com/facebook/react/issues/17626
    hook.renderers.forEach((injected, id) => {
      if (
        typeof injected.scheduleRefresh === 'function' &&
        typeof injected.setRefreshHandler === 'function'
      ) {
        // This version supports React Refresh.
        helpersByRendererID.set(id, ((injected: any): RendererHelpers));
      }
    });

    // We also want to track currently mounted roots.
    const oldOnCommitFiberRoot = hook.onCommitFiberRoot;
    const oldOnScheduleFiberRoot = hook.onScheduleFiberRoot || (() => {});
    hook.onScheduleFiberRoot = function (
      this: mixed,
      id: number,
      root: FiberRoot,
      children: ReactNodeList,
    ) {
      if (!isPerformingRefresh) {
        // If it was intentionally scheduled, don't attempt to restore.
        // This includes intentionally scheduled unmounts.
        failedRoots.delete(root);
        if (rootElements !== null) {
          rootElements.set(root, children);
        }
      }
      return oldOnScheduleFiberRoot.apply(this, arguments);
    };
    hook.onCommitFiberRoot = function (
      this: mixed,
      id: number,
      root: FiberRoot,
      maybePriorityLevel: mixed,
      didError: boolean,
    ) {
      const helpers = helpersByRendererID.get(id);
      if (helpers !== undefined) {
        helpersByRoot.set(root, helpers);

        const current = root.current;
        const alternate = current.alternate;

        // We need to determine whether this root has just (un)mounted.
        // This logic is copy-pasted from similar logic in the DevTools backend.
        // If this breaks with some refactoring, you'll want to update DevTools too.

        if (alternate !== null) {
          const wasMounted =
            alternate.memoizedState != null &&
            alternate.memoizedState.element != null &&
            mountedRoots.has(root);

          const isMounted =
            current.memoizedState != null &&
            current.memoizedState.element != null;

          if (!wasMounted && isMounted) {
            // Mount a new root.
            mountedRoots.add(root);
            failedRoots.delete(root);
          } else if (wasMounted && isMounted) {
            // Update an existing root.
            // This doesn't affect our mounted root Set.
          } else if (wasMounted && !isMounted) {
            // Unmount an existing root.
            mountedRoots.delete(root);
            if (didError) {
              // We'll remount it on future edits.
              failedRoots.add(root);
            } else {
              helpersByRoot.delete(root);
            }
          } else if (!wasMounted && !isMounted) {
```

<a id="ref-q1-2"></a>
### [2] `packages/react-reconciler/src/ReactFiberHotReloading.js:38-123`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-reconciler/src/ReactFiberHotReloading.js#L38-L123)

```javascript
export type Family = {
  current: any,
};

export type RefreshUpdate = {
  staleFamilies: Set<Family>,
  updatedFamilies: Set<Family>,
};

// Resolves type to a family.
type RefreshHandler = any => Family | void;

// Used by React Refresh runtime through DevTools Global Hook.
export type SetRefreshHandler = (handler: RefreshHandler | null) => void;
export type ScheduleRefresh = (root: FiberRoot, update: RefreshUpdate) => void;
export type ScheduleRoot = (root: FiberRoot, element: ReactNodeList) => void;

let resolveFamily: RefreshHandler | null = null;
let failedBoundaries: WeakSet<Fiber> | null = null;

export const setRefreshHandler = (handler: RefreshHandler | null): void => {
  if (__DEV__) {
    resolveFamily = handler;
  }
};

export function resolveFunctionForHotReloading(type: any): any {
  if (__DEV__) {
    if (resolveFamily === null) {
      // Hot reloading is disabled.
      return type;
    }
    const family = resolveFamily(type);
    if (family === undefined) {
      return type;
    }
    // Use the latest known implementation.
    return family.current;
  } else {
    return type;
  }
}

export function resolveClassForHotReloading(type: any): any {
  // No implementation differences.
  return resolveFunctionForHotReloading(type);
}

export function resolveForwardRefForHotReloading(type: any): any {
  if (__DEV__) {
    if (resolveFamily === null) {
      // Hot reloading is disabled.
      return type;
    }
    const family = resolveFamily(type);
    if (family === undefined) {
      // Check if we're dealing with a real forwardRef. Don't want to crash early.
      if (
        type !== null &&
        type !== undefined &&
        typeof type.render === 'function'
      ) {
        // ForwardRef is special because its resolved .type is an object,
        // but it's possible that we only have its inner render function in the map.
        // If that inner render function is different, we'll build a new forwardRef type.
        const currentRender = resolveFunctionForHotReloading(type.render);
        if (type.render !== currentRender) {
          const syntheticType = {
            $$typeof: REACT_FORWARD_REF_TYPE,
            render: currentRender,
          };
          if (type.displayName !== undefined) {
            (syntheticType: any).displayName = type.displayName;
          }
          return syntheticType;
        }
      }
      return type;
    }
    // Use the latest known implementation.
    return family.current;
  } else {
    return type;
  }
}
```

<a id="ref-q1-3"></a>
### [3] `packages/react-refresh/src/ReactFreshRuntime.js:186-299`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L186-L299)

```javascript
export function performReactRefresh(): RefreshUpdate | null {
  if (!__DEV__) {
    throw new Error(
      'Unexpected call to React Refresh in a production environment.',
    );
  }
  if (pendingUpdates.length === 0) {
    return null;
  }
  if (isPerformingRefresh) {
    return null;
  }

  isPerformingRefresh = true;
  try {
    const staleFamilies = new Set<Family>();
    const updatedFamilies = new Set<Family>();

    const updates = pendingUpdates;
    pendingUpdates = [];
    updates.forEach(([family, nextType]) => {
      // Now that we got a real edit, we can create associations
      // that will be read by the React reconciler.
      const prevType = family.current;
      updatedFamiliesByType.set(prevType, family);
      updatedFamiliesByType.set(nextType, family);
      family.current = nextType;

      // Determine whether this should be a re-render or a re-mount.
      if (canPreserveStateBetween(prevType, nextType)) {
        updatedFamilies.add(family);
      } else {
        staleFamilies.add(family);
      }
    });

    // TODO: rename these fields to something more meaningful.
    const update: RefreshUpdate = {
      updatedFamilies, // Families that will re-render preserving state
      staleFamilies, // Families that will be remounted
    };

    helpersByRendererID.forEach(helpers => {
      // Even if there are no roots, set the handler on first update.
      // This ensures that if *new* roots are mounted, they'll use the resolve handler.
      helpers.setRefreshHandler(resolveFamily);
    });

    let didError = false;
    let firstError = null;

    // We snapshot maps and sets that are mutated during commits.
    // If we don't do this, there is a risk they will be mutated while
    // we iterate over them. For example, trying to recover a failed root
    // may cause another root to be added to the failed list -- an infinite loop.
    const failedRootsSnapshot = cloneSet(failedRoots);
    const mountedRootsSnapshot = cloneSet(mountedRoots);
    const helpersByRootSnapshot = cloneMap(helpersByRoot);

    failedRootsSnapshot.forEach(root => {
      const helpers = helpersByRootSnapshot.get(root);
      if (helpers === undefined) {
        throw new Error(
          'Could not find helpers for a root. This is a bug in React Refresh.',
        );
      }
      if (!failedRoots.has(root)) {
        // No longer failed.
      }
      if (rootElements === null) {
        return;
      }
      if (!rootElements.has(root)) {
        return;
      }
      const element = rootElements.get(root);
      try {
        helpers.scheduleRoot(root, element);
      } catch (err) {
        if (!didError) {
          didError = true;
          firstError = err;
        }
        // Keep trying other roots.
      }
    });
    mountedRootsSnapshot.forEach(root => {
      const helpers = helpersByRootSnapshot.get(root);
      if (helpers === undefined) {
        throw new Error(
          'Could not find helpers for a root. This is a bug in React Refresh.',
        );
      }
      if (!mountedRoots.has(root)) {
        // No longer mounted.
      }
      try {
        helpers.scheduleRefresh(root, update);
      } catch (err) {
        if (!didError) {
          didError = true;
          firstError = err;
        }
        // Keep trying other roots.
      }
    });
    if (didError) {
      throw firstError;
    }
    return update;
  } finally {
    isPerformingRefresh = false;
  }
}
```

<a id="ref-q1-4"></a>
### [4] `packages/react-refresh/src/ReactFreshRuntime.js:301-344`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L301-L344)

```javascript
export function register(type: any, id: string): void {
  if (__DEV__) {
    if (type === null) {
      return;
    }
    if (typeof type !== 'function' && typeof type !== 'object') {
      return;
    }

    // This can happen in an edge case, e.g. if we register
    // return value of a HOC but it returns a cached component.
    // Ignore anything but the first registration for each type.
    if (allFamiliesByType.has(type)) {
      return;
    }
    // Create family or remember to update it.
    // None of this bookkeeping affects reconciliation
    // until the first performReactRefresh() call above.
    let family = allFamiliesByID.get(id);
    if (family === undefined) {
      family = {current: type};
      allFamiliesByID.set(id, family);
    } else {
      pendingUpdates.push([family, type]);
    }
    allFamiliesByType.set(type, family);

    // Visit inner types because we might not have registered them.
    if (typeof type === 'object' && type !== null) {
      switch (getProperty(type, '$$typeof')) {
        case REACT_FORWARD_REF_TYPE:
          register(type.render, id + '$render');
          break;
        case REACT_MEMO_TYPE:
          register(type.type, id + '$type');
          break;
      }
    }
  } else {
    throw new Error(
      'Unexpected call to React Refresh in a production environment.',
    );
  }
}
```

<a id="ref-q1-5"></a>
### [5] `packages/react-refresh/src/ReactFreshRuntime.js:346-377`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L346-L377)

```javascript
export function setSignature(
  type: any,
  key: string,
  forceReset?: boolean = false,
  getCustomHooks?: () => Array<Function>,
): void {
  if (__DEV__) {
    if (!allSignaturesByType.has(type)) {
      allSignaturesByType.set(type, {
        forceReset,
        ownKey: key,
        fullKey: null,
        getCustomHooks: getCustomHooks || (() => []),
      });
    }
    // Visit inner types because we might not have signed them.
    if (typeof type === 'object' && type !== null) {
      switch (getProperty(type, '$$typeof')) {
        case REACT_FORWARD_REF_TYPE:
          setSignature(type.render, key, forceReset, getCustomHooks);
          break;
        case REACT_MEMO_TYPE:
          setSignature(type.type, key, forceReset, getCustomHooks);
          break;
      }
    }
  } else {
    throw new Error(
      'Unexpected call to React Refresh in a production environment.',
    );
  }
}
```

<a id="ref-q1-6"></a>
### [6] `packages/react-refresh/src/__tests__/ReactFreshIntegration-test.js:110-112`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/__tests__/ReactFreshIntegration-test.js#L110-L112)

```javascript
  function $RefreshReg$(type, id) {
    ReactFreshRuntime.register(type, id);
  }
```

<a id="ref-q1-7"></a>
### [7] `packages/react-refresh/src/ReactFreshRuntime.js:46-47`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L46-L47)

```javascript
const allFamiliesByID: Map<string, Family> = new Map();
const allFamiliesByType: WeakMap<any, Family> | Map<any, Family> =
```

<a id="ref-q1-8"></a>
### [8] `packages/react-refresh/src/__tests__/ReactFreshIntegration-test.js:114-116`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/__tests__/ReactFreshIntegration-test.js#L114-L116)

```javascript
  function $RefreshSig$() {
    return ReactFreshRuntime.createSignatureFunctionForTransform();
  }
```

<a id="ref-q1-9"></a>
### [9] `packages/react-refresh/src/ReactFreshRuntime.js:586-612`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L586-L612)

```javascript
// Signatures let us decide whether the Hook order has changed on refresh.
//
// This function is intended to be used as a transform target, e.g.:
// var _s = createSignatureFunctionForTransform()
//
// function Hello() {
//   const [foo, setFoo] = useState(0);
//   const value = useCustomHook();
//   _s(); /* Call without arguments triggers collecting the custom Hook list.
//          * This doesn't happen during the module evaluation because we
//          * don't want to change the module order with inline requires.
//          * Next calls are noops. */
//   return <h1>Hi</h1>;
// }
//
// /* Call with arguments attaches the signature to the type: */
// _s(
//   Hello,
//   'useState{[foo, setFoo]}(0)',
//   () => [useCustomHook], /* Lazy to avoid triggering inline requires */
// );
export function createSignatureFunctionForTransform(): <T>(
  type: T,
  key: string,
  forceReset?: boolean,
  getCustomHooks?: () => Array<Function>,
) => T | void {
```

<a id="ref-q1-10"></a>
### [10] `packages/react-refresh/src/ReactFreshRuntime.js:27`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L27)

```javascript
};
```

<a id="ref-q1-11"></a>
### [11] `packages/react-refresh/src/ReactFreshRuntime.js:78-119`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L78-L119)

```javascript
function computeFullKey(signature: Signature): string {
  if (signature.fullKey !== null) {
    return signature.fullKey;
  }

  let fullKey: string = signature.ownKey;
  let hooks;
  try {
    hooks = signature.getCustomHooks();
  } catch (err) {
    // This can happen in an edge case, e.g. if expression like Foo.useSomething
    // depends on Foo which is lazily initialized during rendering.
    // In that case just assume we'll have to remount.
    signature.forceReset = true;
    signature.fullKey = fullKey;
    return fullKey;
  }

  for (let i = 0; i < hooks.length; i++) {
    const hook = hooks[i];
    if (typeof hook !== 'function') {
      // Something's wrong. Assume we need to remount.
      signature.forceReset = true;
      signature.fullKey = fullKey;
      return fullKey;
    }
    const nestedHookSignature = allSignaturesByType.get(hook);
    if (nestedHookSignature === undefined) {
      // No signature means Hook wasn't in the source code, e.g. in a library.
      // We'll skip it because we can assume it won't change during this session.
      continue;
    }
    const nestedHookKey = computeFullKey(nestedHookSignature);
    if (nestedHookSignature.forceReset) {
      signature.forceReset = true;
    }
    fullKey += '\n---\n' + nestedHookKey;
  }

  signature.fullKey = fullKey;
  return fullKey;
}
```

<a id="ref-q1-12"></a>
### [12] `packages/react-refresh/src/ReactFreshRuntime.js:121-139`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L121-L139)

```javascript
function haveEqualSignatures(prevType: any, nextType: any) {
  const prevSignature = allSignaturesByType.get(prevType);
  const nextSignature = allSignaturesByType.get(nextType);

  if (prevSignature === undefined && nextSignature === undefined) {
    return true;
  }
  if (prevSignature === undefined || nextSignature === undefined) {
    return false;
  }
  if (computeFullKey(prevSignature) !== computeFullKey(nextSignature)) {
    return false;
  }
  if (nextSignature.forceReset) {
    return false;
  }

  return true;
}
```

<a id="ref-q1-13"></a>
### [13] `packages/react-refresh/src/ReactFreshRuntime.js:215-217`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L215-L217)

```javascript
      if (canPreserveStateBetween(prevType, nextType)) {
        updatedFamilies.add(family);
      } else {
```

<a id="ref-q1-14"></a>
### [14] `packages/react-refresh/src/ReactFreshRuntime.js:145-153`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L145-L153)

```javascript
function canPreserveStateBetween(prevType: any, nextType: any) {
  if (isReactClass(prevType) || isReactClass(nextType)) {
    return false;
  }
  if (haveEqualSignatures(prevType, nextType)) {
    return true;
  }
  return false;
}
```

<a id="ref-q1-15"></a>
### [15] `packages/react-refresh/src/ReactFreshRuntime.js:149-150`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L149-L150)

```javascript
  if (haveEqualSignatures(prevType, nextType)) {
    return true;
```

<a id="ref-q1-16"></a>
### [16] `packages/react-refresh/src/ReactFreshRuntime.js:224`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L224)

```javascript
      updatedFamilies, // Families that will re-render preserving state
```

<a id="ref-q1-17"></a>
### [17] `packages/react-refresh/src/ReactFreshRuntime.js:141-143`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L141-L143)

```javascript
function isReactClass(type: any) {
  return type.prototype && type.prototype.isReactComponent;
}
```

<a id="ref-q1-18"></a>
### [18] `packages/react-refresh/src/__tests__/ReactFresh-test.js:3111-3112`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/__tests__/ReactFresh-test.js#L3111-L3112)

```javascript
  it('remounts classes on every edit', async () => {
    if (__DEV__) {
```

<a id="ref-q1-19"></a>
### [19] `packages/react-refresh/src/ReactFreshRuntime.js:131-134`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L131-L134)

```javascript
  if (computeFullKey(prevSignature) !== computeFullKey(nextSignature)) {
    return false;
  }
  if (nextSignature.forceReset) {
```

<a id="ref-q1-20"></a>
### [20] `packages/react-refresh/src/__tests__/ReactFresh-test.js:1663-1664`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/__tests__/ReactFresh-test.js#L1663-L1664)

```javascript
  it('can force remount by changing signature', async () => {
    if (__DEV__) {
```

<a id="ref-q1-21"></a>
### [21] `packages/react-refresh/src/ReactFreshRuntime.js:90-91`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L90-L91)

```javascript
    // In that case just assume we'll have to remount.
    signature.forceReset = true;
```

<a id="ref-q1-22"></a>
### [22] `packages/react-refresh/src/ReactFreshRuntime.js:112-113`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L112-L113)

```javascript
      signature.forceReset = true;
    }
```

<a id="ref-q1-23"></a>
### [23] `packages/react-refresh/src/__tests__/ReactFreshIntegration-test.js:1411`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/__tests__/ReactFreshIntegration-test.js#L1411)

```javascript
          /* @refresh reset */
```

<a id="ref-q1-24"></a>
### [24] `packages/react-refresh/src/ReactFreshRuntime.js:225`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L225)

```javascript
      staleFamilies, // Families that will be remounted
```

<a id="ref-q1-25"></a>
### [25] `packages/react-devtools-shared/src/__tests__/FastRefreshDevToolsIntegration-test.js:87-92`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-devtools-shared/src/__tests__/FastRefreshDevToolsIntegration-test.js#L87-L92)

```javascript
    // (In a real module system we'd do this for *all* exports.)
    // For example, this can happen if you convert a class to a function.
    // Or if you wrap something in a HOC.
    const didExportsChange =
      ReactFreshRuntime.getFamilyByType(prevExports.default) !==
      ReactFreshRuntime.getFamilyByType(nextExports.default);
```

<a id="ref-q1-26"></a>
### [26] `packages/react-refresh/src/ReactFreshRuntime.js:301-377`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L301-L377)

```javascript
export function register(type: any, id: string): void {
  if (__DEV__) {
    if (type === null) {
      return;
    }
    if (typeof type !== 'function' && typeof type !== 'object') {
      return;
    }

    // This can happen in an edge case, e.g. if we register
    // return value of a HOC but it returns a cached component.
    // Ignore anything but the first registration for each type.
    if (allFamiliesByType.has(type)) {
      return;
    }
    // Create family or remember to update it.
    // None of this bookkeeping affects reconciliation
    // until the first performReactRefresh() call above.
    let family = allFamiliesByID.get(id);
    if (family === undefined) {
      family = {current: type};
      allFamiliesByID.set(id, family);
    } else {
      pendingUpdates.push([family, type]);
    }
    allFamiliesByType.set(type, family);

    // Visit inner types because we might not have registered them.
    if (typeof type === 'object' && type !== null) {
      switch (getProperty(type, '$$typeof')) {
        case REACT_FORWARD_REF_TYPE:
          register(type.render, id + '$render');
          break;
        case REACT_MEMO_TYPE:
          register(type.type, id + '$type');
          break;
      }
    }
  } else {
    throw new Error(
      'Unexpected call to React Refresh in a production environment.',
    );
  }
}

export function setSignature(
  type: any,
  key: string,
  forceReset?: boolean = false,
  getCustomHooks?: () => Array<Function>,
): void {
  if (__DEV__) {
    if (!allSignaturesByType.has(type)) {
      allSignaturesByType.set(type, {
        forceReset,
        ownKey: key,
        fullKey: null,
        getCustomHooks: getCustomHooks || (() => []),
      });
    }
    // Visit inner types because we might not have signed them.
    if (typeof type === 'object' && type !== null) {
      switch (getProperty(type, '$$typeof')) {
        case REACT_FORWARD_REF_TYPE:
          setSignature(type.render, key, forceReset, getCustomHooks);
          break;
        case REACT_MEMO_TYPE:
          setSignature(type.type, key, forceReset, getCustomHooks);
          break;
      }
    }
  } else {
    throw new Error(
      'Unexpected call to React Refresh in a production environment.',
    );
  }
}
```

<a id="ref-q1-27"></a>
### [27] `packages/react-devtools-shared/src/__tests__/FastRefreshDevToolsIntegration-test.js:103`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-devtools-shared/src/__tests__/FastRefreshDevToolsIntegration-test.js#L103)

```javascript
    act(() => {
```

<a id="ref-q1-28"></a>
### [28] `packages/react-refresh/src/ReactFreshRuntime.js:201-226`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L201-L226)

```javascript
    const staleFamilies = new Set<Family>();
    const updatedFamilies = new Set<Family>();

    const updates = pendingUpdates;
    pendingUpdates = [];
    updates.forEach(([family, nextType]) => {
      // Now that we got a real edit, we can create associations
      // that will be read by the React reconciler.
      const prevType = family.current;
      updatedFamiliesByType.set(prevType, family);
      updatedFamiliesByType.set(nextType, family);
      family.current = nextType;

      // Determine whether this should be a re-render or a re-mount.
      if (canPreserveStateBetween(prevType, nextType)) {
        updatedFamilies.add(family);
      } else {
        staleFamilies.add(family);
      }
    });

    // TODO: rename these fields to something more meaningful.
    const update: RefreshUpdate = {
      updatedFamilies, // Families that will re-render preserving state
      staleFamilies, // Families that will be remounted
    };
```

<a id="ref-q1-29"></a>
### [29] `packages/react-refresh/src/ReactFreshRuntime.js:414`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L414)

```javascript
export function injectIntoGlobalHook(globalObject: any): void {
```

<a id="ref-q1-30"></a>
### [30] `packages/react-refresh/src/ReactFreshRuntime.js:458-467`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L458-L467)

```javascript
    const oldInject = hook.inject;
    hook.inject = function (this: mixed, injected) {
      const id = oldInject.apply(this, arguments);
      if (
        typeof injected.scheduleRefresh === 'function' &&
        typeof injected.setRefreshHandler === 'function'
      ) {
        // This version supports React Refresh.
        helpersByRendererID.set(id, ((injected: any): RendererHelpers));
      }
```

<a id="ref-q1-31"></a>
### [31] `packages/react-refresh/src/ReactFreshRuntime.js:283`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L283)

```javascript
        helpers.scheduleRefresh(root, update);
```

<a id="ref-q1-32"></a>
### [32] `packages/react-refresh/src/ReactFreshRuntime.js:224-225`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L224-L225)

```javascript
      updatedFamilies, // Families that will re-render preserving state
      staleFamilies, // Families that will be remounted
```

<a id="ref-q1-33"></a>
### [33] `packages/react-reconciler/src/ReactFiberHotReloading.js:58-62`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-reconciler/src/ReactFiberHotReloading.js#L58-L62)

```javascript
export const setRefreshHandler = (handler: RefreshHandler | null): void => {
  if (__DEV__) {
    resolveFamily = handler;
  }
};
```

<a id="ref-q1-34"></a>
### [34] `packages/react-reconciler/src/ReactFiberHotReloading.js:74-75`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-reconciler/src/ReactFiberHotReloading.js#L74-L75)

```javascript
    // Use the latest known implementation.
    return family.current;
```

<a id="ref-q1-35"></a>
### [35] `packages/react-refresh/src/ReactFreshRuntime.js:263`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L263)

```javascript
        helpers.scheduleRoot(root, element);
```

<a id="ref-q1-36"></a>
### [36] `packages/react-refresh/src/ReactFreshRuntime.js:503-509`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L503-L509)

```javascript
    hook.onCommitFiberRoot = function (
      this: mixed,
      id: number,
      root: FiberRoot,
      maybePriorityLevel: mixed,
      didError: boolean,
    ) {
```

<a id="ref-q1-37"></a>
### [37] `packages/react-refresh/src/ReactFreshRuntime.js:511-512`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L511-L512)

```javascript
      if (helpers !== undefined) {
        helpersByRoot.set(root, helpers);
```

<a id="ref-q1-38"></a>
### [38] `packages/react-refresh/src/ReactFreshRuntime.js:487-492`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L487-L492)

```javascript
    hook.onScheduleFiberRoot = function (
      this: mixed,
      id: number,
      root: FiberRoot,
      children: ReactNodeList,
    ) {
```

<a id="ref-q1-39"></a>
### [39] `packages/react-refresh/src/ReactFreshRuntime.js:497-498`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/ReactFreshRuntime.js#L497-L498)

```javascript
        if (rootElements !== null) {
          rootElements.set(root, children);
```

<a id="ref-q1-40"></a>
### [40] `packages/react-devtools-shared/src/__tests__/FastRefreshDevToolsIntegration-test.js:61-64`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-devtools-shared/src/__tests__/FastRefreshDevToolsIntegration-test.js#L61-L64)

```javascript
      '$RefreshReg$',
      '$RefreshSig$',
      compiled,
    )(global, React, exportsObj, $RefreshReg$, $RefreshSig$);
```

<a id="ref-q1-41"></a>
### [41] `packages/react-refresh/src/__tests__/ReactFreshIntegration-test.js:1-178`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-refresh/src/__tests__/ReactFreshIntegration-test.js#L1-L178)

```javascript
/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 *
 * @emails react-core
 */

'use strict';

let React;
let ReactDOMClient;
let ReactFreshRuntime;
let Scheduler;
let act;
let assertLog;

const babel = require('@babel/core');
const freshPlugin = require('react-refresh/babel');
const ts = require('typescript');

describe('ReactFreshIntegration', () => {
  let container;
  let root;
  let exportsObj;

  beforeEach(() => {
    if (__DEV__) {
      jest.resetModules();
      React = require('react');
      ReactFreshRuntime = require('react-refresh/runtime');
      ReactFreshRuntime.injectIntoGlobalHook(global);
      ReactDOMClient = require('react-dom/client');
      Scheduler = require('scheduler/unstable_mock');
      ({act, assertLog} = require('internal-test-utils'));
      container = document.createElement('div');
      document.body.appendChild(container);
      root = ReactDOMClient.createRoot(container);
      exportsObj = undefined;
    }
  });

  afterEach(() => {
    if (__DEV__) {
      root.unmount();
      // Ensure we don't leak memory by holding onto dead roots.
      expect(ReactFreshRuntime._getMountedRootCount()).toBe(0);
      document.body.removeChild(container);
    }
  });

  function executeJavaScript(source, compileDestructuring) {
    const compiled = babel.transform(source, {
      babelrc: false,
      presets: ['@babel/react'],
      plugins: [
        [freshPlugin, {skipEnvCheck: true}],
        '@babel/plugin-transform-modules-commonjs',
        compileDestructuring && '@babel/plugin-transform-destructuring',
      ].filter(Boolean),
    }).code;
    return executeCompiled(compiled);
  }

  function executeTypescript(source) {
    const typescriptSource = babel.transform(source, {
      babelrc: false,
      configFile: false,
      presets: ['@babel/react'],
      plugins: [
        [freshPlugin, {skipEnvCheck: true}],
        ['@babel/plugin-syntax-typescript', {isTSX: true}],
      ],
    }).code;
    const compiled = ts.transpileModule(typescriptSource, {
      module: ts.ModuleKind.CommonJS,
    }).outputText;
    return executeCompiled(compiled);
  }

  function executeCompiled(compiled) {
    exportsObj = {};
    // eslint-disable-next-line no-new-func
    new Function(
      'global',
      'require',
      'React',
      'Scheduler',
      'exports',
      '$RefreshReg$',
      '$RefreshSig$',
      compiled,
    )(
      global,
      require,
      React,
      Scheduler,
      exportsObj,
      $RefreshReg$,
      $RefreshSig$,
    );
    // Module systems will register exports as a fallback.
    // This is useful for cases when e.g. a class is exported,
    // and we don't want to propagate the update beyond this module.
    $RefreshReg$(exportsObj.default, 'exports.default');
    return exportsObj.default;
  }

  function $RefreshReg$(type, id) {
    ReactFreshRuntime.register(type, id);
  }

  function $RefreshSig$() {
    return ReactFreshRuntime.createSignatureFunctionForTransform();
  }

  describe.each([
    [
      'JavaScript syntax with destructuring enabled',
      source => executeJavaScript(source, true),
      testJavaScript,
    ],
    [
      'JavaScript syntax with destructuring disabled',
      source => executeJavaScript(source, false),
      testJavaScript,
    ],
    ['TypeScript syntax', executeTypescript, testTypeScript],
  ])('%s', (language, execute, runTest) => {
    async function render(source) {
      const Component = execute(source);
      await act(() => {
        root.render(<Component />);
      });
      // Module initialization shouldn't be counted as a hot update.
      expect(ReactFreshRuntime.performReactRefresh()).toBe(null);
    }

    async function patch(source) {
      const prevExports = exportsObj;
      execute(source);
      const nextExports = exportsObj;

      // Check if exported families have changed.
      // (In a real module system we'd do this for *all* exports.)
      // For example, this can happen if you convert a class to a function.
      // Or if you wrap something in a HOC.
      const didExportsChange =
        ReactFreshRuntime.getFamilyByType(prevExports.default) !==
        ReactFreshRuntime.getFamilyByType(nextExports.default);
      if (didExportsChange) {
        // In a real module system, we would propagate such updates upwards,
        // and re-execute modules that imported this one. (Just like if we edited them.)
        // This makes adding/removing/renaming exports re-render references to them.
        // Here, we'll just force a re-render using the newer type to emulate this.
        const NextComponent = nextExports.default;
        await act(() => {
          root.render(<NextComponent />);
        });
      }
      await act(() => {
        const result = ReactFreshRuntime.performReactRefresh();
        if (!didExportsChange) {
          // Normally we expect that some components got updated in our tests.
          expect(result).not.toBe(null);
        } else {
          // However, we have tests where we convert functions to classes,
          // and in those cases it's expected nothing would get updated.
          // (Instead, the export change branch above would take care of it.)
        }
      });
      expect(ReactFreshRuntime._getMountedRootCount()).toBe(1);
    }

    runTest(render, patch);
  });
```

<a id="ref-q1-42"></a>
### [42] `packages/react-devtools-shared/src/__tests__/FastRefreshDevToolsIntegration-test.js:1-116`
Source: [facebook/react @ 3cb2c420](https://github.com/facebook/react/blob/3cb2c420/packages/react-devtools-shared/src/__tests__/FastRefreshDevToolsIntegration-test.js#L1-L116)

```javascript
/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 *
 * @flow
 */

import {getVersionedRenderImplementation} from './utils';

describe('Fast Refresh', () => {
  let React;
  let ReactFreshRuntime;
  let act;
  let babel;
  let exportsObj;
  let freshPlugin;
  let store;
  let withErrorsOrWarningsIgnored;

  beforeEach(() => {
    global.IS_REACT_ACT_ENVIRONMENT = true;

    exportsObj = undefined;

    babel = require('@babel/core');
    freshPlugin = require('react-refresh/babel');

    store = global.store;

    React = require('react');

    ReactFreshRuntime = require('react-refresh/runtime');
    ReactFreshRuntime.injectIntoGlobalHook(global);

    const utils = require('./utils');
    act = utils.act;
    withErrorsOrWarningsIgnored = utils.withErrorsOrWarningsIgnored;
  });

  const {render: renderImplementation, getContainer} =
    getVersionedRenderImplementation();

  function execute(source) {
    const compiled = babel.transform(source, {
      babelrc: false,
      presets: ['@babel/react'],
      plugins: [
        [freshPlugin, {skipEnvCheck: true}],
        '@babel/plugin-transform-modules-commonjs',
        '@babel/plugin-transform-destructuring',
      ].filter(Boolean),
    }).code;
    exportsObj = {};
    // eslint-disable-next-line no-new-func
    new Function(
      'global',
      'React',
      'exports',
      '$RefreshReg$',
      '$RefreshSig$',
      compiled,
    )(global, React, exportsObj, $RefreshReg$, $RefreshSig$);
    // Module systems will register exports as a fallback.
    // This is useful for cases when e.g. a class is exported,
    // and we don't want to propagate the update beyond this module.
    $RefreshReg$(exportsObj.default, 'exports.default');
    return exportsObj.default;
  }

  function render(source) {
    const Component = execute(source);
    act(() => {
      renderImplementation(<Component />);
    });
    // Module initialization shouldn't be counted as a hot update.
    expect(ReactFreshRuntime.performReactRefresh()).toBe(null);
  }

  function patch(source) {
    const prevExports = exportsObj;
    execute(source);
    const nextExports = exportsObj;

    // Check if exported families have changed.
    // (In a real module system we'd do this for *all* exports.)
    // For example, this can happen if you convert a class to a function.
    // Or if you wrap something in a HOC.
    const didExportsChange =
      ReactFreshRuntime.getFamilyByType(prevExports.default) !==
      ReactFreshRuntime.getFamilyByType(nextExports.default);
    if (didExportsChange) {
      // In a real module system, we would propagate such updates upwards,
      // and re-execute modules that imported this one. (Just like if we edited them.)
      // This makes adding/removing/renaming exports re-render references to them.
      // Here, we'll just force a re-render using the newer type to emulate this.
      const NextComponent = nextExports.default;
      act(() => {
        renderImplementation(<NextComponent />);
      });
    }
    act(() => {
      const result = ReactFreshRuntime.performReactRefresh();
      if (!didExportsChange) {
        // Normally we expect that some components got updated in our tests.
        expect(result).not.toBe(null);
      } else {
        // However, we have tests where we convert functions to classes,
        // and in those cases it's expected nothing would get updated.
        // (Instead, the export change branch above would take care of it.)
      }
    });
    expect(ReactFreshRuntime._getMountedRootCount()).toBe(1);
  }
```
