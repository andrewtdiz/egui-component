# DeepWiki Response

Repository: `pmndrs/react-three-fiber`
Question: For a Rust-hosted JSX runtime building a first structurally solid and reliable retained-renderer architecture, what is the minimum viable core architecture to copy from react-three-fiber? Focus on the smallest trustworthy v1 for: (1) reconciler/host-config responsibilities, (2) atomic commit phases and rollback or recovery boundaries, (3) stable event-handler registration and dispatch routing without duplicate invocation, (4) frame invalidation and effect/timer scheduling, (5) error boundaries and recoverable failure handling, and (6) which advanced features should be explicitly deferred from v1 to keep the architecture clean. Please cite concrete implementation references and translate them into a recommended v1 architecture shape for a Rust-owned host tree and host-driven renderer.
Search: https://deepwiki.com/search/for-a-rusthosted-jsx-runtime-b_3a4fbd6f-c2c6-45e1-b098-0e2ec3c8dab6

**Question**: For a Rust-hosted JSX runtime building a first structurally solid and reliable retained-renderer architecture, what is the minimum viable core architecture to copy from react-three-fiber? Focus on the smallest trustworthy v1 for: (1) reconciler/host-config responsibilities, (2) atomic commit phases and rollback or recovery boundaries, (3) stable event-handler registration and dispatch routing without duplicate invocation, (4) frame invalidation and effect/timer scheduling, (5) error boundaries and recoverable failure handling, and (6) which advanced features should be explicitly deferred from v1 to keep the architecture clean. Please cite concrete implementation references and translate them into a recommended v1 architecture shape for a Rust-owned host tree and host-driven renderer.

The user is asking for a minimum viable core architecture for a Rust-hosted JSX runtime, drawing inspiration from `react-three-fiber`. The focus is on specific aspects: reconciler/host-config, atomic commit phases, event handling, frame invalidation, error boundaries, and deferred features.

`react-three-fiber` (R3F) is a React renderer for Three.js, meaning it uses React's reconciliation process to manage Three.js objects declaratively. The core of R3F's architecture for this request lies within its React Reconciler implementation.

Here's a breakdown of the recommended v1 architecture based on `react-three-fiber`:

## Minimum Viable Core Architecture

### 1. Reconciler/Host-Config Responsibilities

The core of the architecture will be a custom React Reconciler. This reconciler acts as the bridge between React's virtual DOM and your Rust-owned host tree. [1](#ref-q1-1) 

**Responsibilities:**
*   **`createInstance`**: This function is responsible for creating actual host objects (your Rust-owned objects) based on the JSX element type and props. In R3F, this involves looking up a constructor in a `catalogue` and instantiating a Three.js object. For a Rust-hosted system, this would involve calling into your Rust FFI to create the corresponding Rust object. [2](#ref-q1-2) 
*   **`appendChild`, `insertBefore`, `removeChild`**: These functions manage the hierarchical structure of your host tree. They dictate how host objects are added, reordered, and removed from their parents. R3F handles attaching and detaching Three.js objects from their parent's children array or using a custom `attach` mechanism. [3](#ref-q1-3)  [4](#ref-q1-4)  [5](#ref-q1-5) 
*   **`commitUpdate`**: This is where property updates are applied to existing host objects. R3F uses `diffProps` to determine changes and then `applyProps` to update the Three.js object. Your Rust system would need a similar mechanism to update properties on your Rust objects. [6](#ref-q1-6) 
*   **`getPublicInstance`**: Returns the actual host object that React components can interact with (e.g., via `ref`). [7](#ref-q1-7) 
*   **`extend` API**: A mechanism to register custom host object types with the reconciler. This allows users to define their own Rust-backed components and have them recognized by the JSX runtime. [8](#ref-q1-8) 

### 2. Atomic Commit Phases and Rollback/Recovery Boundaries

React's reconciler inherently provides atomic commit phases. When `reconciler.updateContainer` is called, React performs its reconciliation algorithm, and all updates are applied in a single, atomic commit. [9](#ref-q1-9) 

*   **Commit Phase**: The `commitUpdate`, `appendChild`, `insertBefore`, and `removeChild` methods within the host config are part of the commit phase. These operations should be designed to be as efficient as possible.
*   **Rollback/Recovery**: React's error boundaries provide a mechanism for recovering from errors during rendering. If an error occurs during reconciliation or in a component's lifecycle, React can catch it and render a fallback UI. In R3F, an `ErrorBoundary` component is used to catch errors and report them. [10](#ref-q1-10)  The `createRoot` function also registers `logRecoverableError` callbacks with the reconciler. [11](#ref-q1-11) 

### 3. Stable Event-Handler Registration and Dispatch Routing without Duplicate Invocation

Event handling in R3F is managed by an `EventManager`. [12](#ref-q1-12) 

*   **Registration**: Event handlers are typically registered on the host objects themselves. When a component mounts, its event handlers are attached to the corresponding host object.
*   **Dispatch Routing**: R3F's event system likely involves a single event listener on the canvas element, which then dispatches events to the appropriate Three.js objects based on raycasting. This prevents duplicate invocations by having a centralized dispatch mechanism. The `removeInteractivity` function is called when a child is removed, ensuring event handlers are cleaned up. [13](#ref-q1-13) 

### 4. Frame Invalidation and Effect/Timer Scheduling

*   **Frame Invalidation**: R3F uses an `invalidate` function to request a new frame to be rendered. This is crucial for "on-demand rendering" where frames are only rendered when necessary, saving resources. [14](#ref-q1-14)  The `invalidateInstance` function is called whenever an instance is updated or its visibility changes, signaling that a re-render is needed. [15](#ref-q1-15) 
*   **Effect Scheduling**: The `useFrame` hook allows components to execute code on every frame of the render loop. This is managed by a subscription system where callbacks are registered and executed during the update cycle. [16](#ref-q1-16)  Global effects can also be added using `addEffect`, `addAfterEffect`, and `addTail`. [17](#ref-q1-17) 
*   **Timer Scheduling**: React Reconciler provides `scheduleTimeout` and `cancelTimeout` for managing timers, which R3F utilizes. [18](#ref-q1-18) 

### 5. Error Boundaries and Recoverable Failure Handling

As mentioned in section 2, error boundaries are a key part of React's recovery mechanism.

*   **`ErrorBoundary` Component**: A dedicated React component that catches JavaScript errors anywhere in its child component tree, logs those errors, and displays a fallback UI. [10](#ref-q1-10) 
*   **`logRecoverableError`**: The `createRoot` function passes a `logRecoverableError` callback to the React Reconciler, which is invoked when React encounters an uncaught or recoverable error during reconciliation. [11](#ref-q1-11) 

### 6. Advanced Features to Explicitly Deferred from v1

To keep the v1 architecture clean and focused, the following advanced features from R3F could be deferred:

*   **Concurrency (React 18 features)**: While R3F supports React 18's concurrent scheduling, implementing this in a v1 Rust-hosted renderer might add significant complexity. Deferring `startTransition` and `useTransition` would simplify the initial implementation. [19](#ref-q1-19) 
*   **Performance Scaling (`regress` API)**: R3F has an adaptive performance system that can dynamically adjust rendering quality based on performance metrics. This is a valuable feature but can be added in a later iteration. [20](#ref-q1-20) 
*   **Hydration**: R3F's reconciler explicitly states `supportsHydration: false`. This means it doesn't support server-side rendering and then "hydrating" the client-side React tree. For a v1, this is a reasonable deferral. [21](#ref-q1-21) 
*   **Suspense for Data Fetching**: While React supports Suspense for data fetching, integrating this deeply with a custom Rust backend might be complex for a v1. R3F does have `Block` and `UnblockProps` which are related to Suspense, but the full data fetching integration can be deferred. [22](#ref-q1-22) 

## Recommended v1 Architecture Shape

```mermaid
graph TD
    A[React JSX] --> B(React Reconciler)
    B --> C{Host Config (Rust FFI)}
    C --> D[Rust Host Tree]
    D --> E[Rust-driven Renderer]

    subgraph "React Reconciler"
        B -- "createInstance" --> C
        B -- "appendChild" --> C
        B -- "insertBefore" --> C
        B -- "removeChild" --> C
        B -- "commitUpdate" --> C
        B -- "getPublicInstance" --> C
        B -- "logRecoverableError" --> F[Error Boundary / Logging]
    end

    subgraph "Host Config (Rust FFI)"
        C -- "Create Rust Object" --> D
        C -- "Manipulate Rust Object" --> D
        C -- "Event Registration" --> G[Event Manager]
        C -- "Invalidate Frame" --> H[Frame Scheduler]
    end

    subgraph "Rust Host Tree"
        D -- "Render Commands" --> E
    end

    subgraph "Event System"
        G -- "Event Dispatch" --> D
    end

    subgraph "Rendering Loop"
        H -- "Request Frame" --> E
        E -- "Render Frame" --> I[Display]
        H -- "Effect Scheduling" --> D
    end

    subgraph "Error Handling"
        F -- "Catch Errors" --> J[Fallback UI / Logging]
    end
```

This architecture outlines a clear separation of concerns: React handles the declarative UI description and reconciliation, the Host Config (via Rust FFI) translates these operations into commands for your Rust host tree, and the Rust host tree manages the actual rendering and event handling.

## Notes

The provided codebase snippets primarily focus on the JavaScript/TypeScript side of `react-three-fiber`, specifically its React Reconciler implementation and how it interacts with Three.js. The translation to a "Rust-owned host tree and host-driven renderer" would involve implementing the host config methods in Rust, exposed via a Foreign Function Interface (FFI) to the JavaScript reconciler. The `packages/fiber/src/core/reconciler.tsx` file is the most relevant as it defines the host config for R3F. [23](#ref-q1-23) 

Wiki pages you might want to explore:
- [Your First Scene (pmndrs/react-three-fiber)](/wiki/pmndrs/react-three-fiber#1.3)

View this search on DeepWiki: https://deepwiki.com/search/for-a-rusthosted-jsx-runtime-b_3a4fbd6f-c2c6-45e1-b098-0e2ec3c8dab6

## References

<a id="ref-q1-1"></a>
### [1] `packages/fiber/src/core/reconciler.tsx:56-65`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L56-L65)

```tsx
  const reconciler = Reconciler(config as any)

  reconciler.injectIntoDevTools({
    bundleType: typeof process !== 'undefined' && process.env.NODE_ENV !== 'production' ? 1 : 0,
    rendererPackageName: '@react-three/fiber',
    version: React.version,
  })

  return reconciler as any
}
```

<a id="ref-q1-2"></a>
### [2] `packages/fiber/src/core/reconciler.tsx:178-188`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L178-L188)

```tsx
function createInstance(type: string, props: HostConfig['props'], root: RootStore): HostConfig['instance'] {
  // Remove three* prefix from elements if native element not present
  type = toPascalCase(type) in catalogue ? type : type.replace(PREFIX_REGEX, '')

  validateInstance(type, props)

  // Regenerate the R3F instance for primitives to simulate a new object
  if (type === 'primitive' && props.object?.__r3f) delete props.object.__r3f

  return prepare(props.object, root, type, props)
}
```

<a id="ref-q1-3"></a>
### [3] `packages/fiber/src/core/reconciler.tsx:268-277`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L268-L277)

```tsx
function appendChild(parent: HostConfig['instance'], child: HostConfig['instance'] | HostConfig['textInstance']) {
  if (!child) return

  // Link instances
  child.parent = parent
  parent.children.push(child)

  // Attach tree once complete
  handleContainerEffects(parent, child)
}
```

<a id="ref-q1-4"></a>
### [4] `packages/fiber/src/core/reconciler.tsx:279-294`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L279-L294)

```tsx
function insertBefore(
  parent: HostConfig['instance'],
  child: HostConfig['instance'] | HostConfig['textInstance'],
  beforeChild: HostConfig['instance'] | HostConfig['textInstance'],
) {
  if (!child || !beforeChild) return

  // Link instances
  child.parent = parent
  const childIndex = parent.children.indexOf(beforeChild)
  if (childIndex !== -1) parent.children.splice(childIndex, 0, child)
  else parent.children.push(child)

  // Attach tree once complete
  handleContainerEffects(parent, child, beforeChild)
}
```

<a id="ref-q1-5"></a>
### [5] `packages/fiber/src/core/reconciler.tsx:313-358`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L313-L358)

```tsx
function removeChild(
  parent: HostConfig['instance'],
  child: HostConfig['instance'] | HostConfig['textInstance'],
  dispose?: boolean,
) {
  if (!child) return

  // Unlink instances
  child.parent = null
  const childIndex = parent.children.indexOf(child)
  if (childIndex !== -1) parent.children.splice(childIndex, 1)

  // Eagerly tear down tree
  if (child.props.attach) {
    detach(parent, child)
  } else if (isObject3D(child.object) && isObject3D(parent.object)) {
    parent.object.remove(child.object)
    removeInteractivity(findInitialRoot(child), child.object)
  }

  // Allow objects to bail out of unmount disposal with dispose={null}
  const shouldDispose = child.props.dispose !== null && dispose !== false

  // Recursively remove instance children
  for (let i = child.children.length - 1; i >= 0; i--) {
    const node = child.children[i]
    removeChild(child, node, shouldDispose)
  }
  child.children.length = 0

  // Unlink instance object
  delete child.object.__r3f

  // Dispose object whenever the reconciler feels like it.
  // Never dispose of primitives because their state may be kept outside of React!
  // In order for an object to be able to dispose it
  //   - has a dispose method
  //   - cannot be a <primitive object={...} />
  //   - cannot be a THREE.Scene, because three has broken its own API
  if (shouldDispose && child.type !== 'primitive' && child.object.type !== 'Scene') {
    disposeOnIdle(child.object)
  }

  // Tree was updated, request a frame for top-level instance
  if (dispose === undefined) invalidateInstance(child)
}
```

<a id="ref-q1-6"></a>
### [6] `packages/fiber/src/core/reconciler.tsx:504-536`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L504-L536)

```tsx
    instance: HostConfig['instance'],
    type: HostConfig['type'],
    oldProps: HostConfig['props'],
    newProps: HostConfig['props'],
    fiber: Fiber,
  ) {
    validateInstance(type, newProps)

    let reconstruct = false

    // Reconstruct primitives if object prop changes
    if (instance.type === 'primitive' && oldProps.object !== newProps.object) reconstruct = true
    // Reconstruct instance if args were added or removed
    else if (newProps.args?.length !== oldProps.args?.length) reconstruct = true
    // Reconstruct instance if args were changed
    else if (newProps.args?.some((value, index) => value !== oldProps.args?.[index])) reconstruct = true

    // Reconstruct when args or <primitive object={...} have changes
    if (reconstruct) {
      reconstructed.push([instance, { ...newProps }, fiber])
    } else {
      // Create a diff-set, flag if there are any changes
      const changedProps = diffProps(instance, newProps)
      if (Object.keys(changedProps).length) {
        Object.assign(instance.props, changedProps)
        applyProps(instance.object, changedProps)
      }
    }

    // Flush reconstructed siblings when we hit the last updated child in a sequence
    const isTailSibling = fiber.sibling === null || (fiber.flags & Update) === NoFlags
    if (isTailSibling) swapInstances()
  },
```

<a id="ref-q1-7"></a>
### [7] `packages/fiber/src/core/reconciler.tsx:539`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L539)

```tsx
  getPublicInstance: (instance) => instance?.object!,
```

<a id="ref-q1-8"></a>
### [8] `packages/fiber/src/core/reconciler.tsx:148-158`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L148-L158)

```tsx
export function extend<T extends Catalogue | ConstructorRepresentation>(
  objects: T,
): React.ExoticComponent<ThreeElement<any>> | void {
  if (isConstructor(objects)) {
    const Component = `${i++}`
    catalogue[Component] = objects
    return Component as any
  } else {
    Object.assign(catalogue, objects)
  }
}
```

<a id="ref-q1-9"></a>
### [9] `packages/fiber/src/core/renderer.tsx:409-415`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/renderer.tsx#L409-L415)

```tsx
      pending!.then(() => {
        reconciler.updateContainer(
          <Provider store={store} children={children} onCreated={onCreated} rootElement={canvas} />,
          fiber,
          null,
          () => undefined,
        )
```

<a id="ref-q1-10"></a>
### [10] `packages/fiber/src/core/utils.tsx:104-117`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/utils.tsx#L104-L117)

```tsx
export const ErrorBoundary = /* @__PURE__ */ (() =>
  class ErrorBoundary extends React.Component<
    { set: React.Dispatch<Error | undefined>; children: React.ReactNode },
    { error: boolean }
  > {
    state = { error: false }
    static getDerivedStateFromError = () => ({ error: true })
    componentDidCatch(err: Error) {
      this.props.set(err)
    }
    render() {
      return this.state.error ? null : this.props.children
    }
  })()
```

<a id="ref-q1-11"></a>
### [11] `packages/fiber/src/core/renderer.tsx:174-176`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/renderer.tsx#L174-L176)

```tsx
      logRecoverableError, // onUncaughtError
      logRecoverableError, // onCaughtError
      logRecoverableError, // onRecoverableError
```

<a id="ref-q1-12"></a>
### [12] `packages/fiber/src/core/renderer.tsx:108`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/renderer.tsx#L108)

```tsx
  events?: (store: RootStore) => EventManager<HTMLElement>
```

<a id="ref-q1-13"></a>
### [13] `packages/fiber/src/core/reconciler.tsx:330`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L330)

```tsx
    removeInteractivity(findInitialRoot(child), child.object)
```

<a id="ref-q1-14"></a>
### [14] `docs/advanced/scaling-performance.mdx:26-28`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/docs/advanced/scaling-performance.mdx#L26-L28)

```

One major caveat is that if anything in the tree _mutates_ props, then React cannot be aware of it and the display would be stale. For instance, camera controls just grab into the camera and mutate its values. Here you can use React Three Fiber's `invalidate` function to trigger frames manually.
```

<a id="ref-q1-15"></a>
### [15] `packages/fiber/src/core/reconciler.tsx:199`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L199)

```tsx
    invalidateInstance(instance)
```

<a id="ref-q1-16"></a>
### [16] `docs/tutorials/basic-animations.mdx:11-16`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/docs/tutorials/basic-animations.mdx#L11-L16)

```
## `useFrame`

`useFrame` is a Fiber hook that lets you execute code on every frame of Fiber's render loop. This can have a lot of uses, but we will focus on building an animation with it.

It's important to remember that **Fiber hooks can only be called inside a `<Canvas />` parent**!
```

<a id="ref-q1-17"></a>
### [17] `packages/fiber/src/core/loop.ts:22-35`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/loop.ts#L22-L35)

```typescript
 */
export const addEffect = (callback: GlobalRenderCallback) => createSubs(callback, globalEffects)

/**
 * Adds a global after-render callback which is called each frame.
 * @see https://docs.pmnd.rs/react-three-fiber/api/additional-exports#addAfterEffect
 */
export const addAfterEffect = (callback: GlobalRenderCallback) => createSubs(callback, globalAfterEffects)

/**
 * Adds a global callback which is called when rendering stops.
 * @see https://docs.pmnd.rs/react-three-fiber/api/additional-exports#addTail
 */
export const addTail = (callback: GlobalRenderCallback) => createSubs(callback, globalTailEffects)
```

<a id="ref-q1-18"></a>
### [18] `packages/fiber/src/core/reconciler.tsx:550-551`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L550-L551)

```tsx
  scheduleTimeout: (typeof setTimeout === 'function' ? setTimeout : undefined) as any,
  cancelTimeout: (typeof clearTimeout === 'function' ? clearTimeout : undefined) as any,
```

<a id="ref-q1-19"></a>
### [19] `docs/advanced/scaling-performance.mdx:328-332`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/docs/advanced/scaling-performance.mdx#L328-L332)

```
## Enable concurrency

React 18 introduces concurrent scheduling, specifically time slicing via `startTransition` and `useTransition`. This will virtualize the component graph, which then allows you to prioritise components and actions. Think of how a virtual list avoids scaling issues because it only renders as many items as the screen can take, it is not affected by the amount of items it has to render, be it 10 or 100.000.000.

React 18 functions very similar to this, it can potentially defer load and heavy tasks in ways that would be hard or impossible to achieve in a vanilla application. It thereby holds on to a stable framerate even in the most demanding situations.
```

<a id="ref-q1-20"></a>
### [20] `docs/advanced/scaling-performance.mdx:95-98`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/docs/advanced/scaling-performance.mdx#L95-L98)

```
```

### Caching with `useLoader`
```

<a id="ref-q1-21"></a>
### [21] `packages/fiber/src/core/reconciler.tsx:477`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L477)

```tsx
  supportsHydration: false,
```

<a id="ref-q1-22"></a>
### [22] `packages/fiber/src/core/utils.tsx:92-101`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/utils.tsx#L92-L101)

```tsx
export type SetBlock = false | Promise<null> | null
export type UnblockProps = { set: React.Dispatch<React.SetStateAction<SetBlock>>; children: React.ReactNode }

export function Block({ set }: Omit<UnblockProps, 'children'>) {
  useIsomorphicLayoutEffect(() => {
    set(new Promise(() => null))
    return () => set(false)
  }, [set])
  return null
}
```

<a id="ref-q1-23"></a>
### [23] `packages/fiber/src/core/reconciler.tsx:473-604`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L473-L604)

```tsx
  isPrimaryRenderer: false,
  warnsIfNotActing: false,
  supportsMutation: true,
  supportsPersistence: false,
  supportsHydration: false,
  createInstance,
  removeChild,
  appendChild,
  appendInitialChild: appendChild,
  insertBefore,
  appendChildToContainer(container, child) {
    const scene = (container.getState().scene as unknown as Instance<THREE.Scene>['object']).__r3f
    if (!child || !scene) return

    appendChild(scene, child)
  },
  removeChildFromContainer(container, child) {
    const scene = (container.getState().scene as unknown as Instance<THREE.Scene>['object']).__r3f
    if (!child || !scene) return

    removeChild(scene, child)
  },
  insertInContainerBefore(container, child, beforeChild) {
    const scene = (container.getState().scene as unknown as Instance<THREE.Scene>['object']).__r3f
    if (!child || !beforeChild || !scene) return

    insertBefore(scene, child, beforeChild)
  },
  getRootHostContext: () => NO_CONTEXT,
  getChildHostContext: () => NO_CONTEXT,
  commitUpdate(
    instance: HostConfig['instance'],
    type: HostConfig['type'],
    oldProps: HostConfig['props'],
    newProps: HostConfig['props'],
    fiber: Fiber,
  ) {
    validateInstance(type, newProps)

    let reconstruct = false

    // Reconstruct primitives if object prop changes
    if (instance.type === 'primitive' && oldProps.object !== newProps.object) reconstruct = true
    // Reconstruct instance if args were added or removed
    else if (newProps.args?.length !== oldProps.args?.length) reconstruct = true
    // Reconstruct instance if args were changed
    else if (newProps.args?.some((value, index) => value !== oldProps.args?.[index])) reconstruct = true

    // Reconstruct when args or <primitive object={...} have changes
    if (reconstruct) {
      reconstructed.push([instance, { ...newProps }, fiber])
    } else {
      // Create a diff-set, flag if there are any changes
      const changedProps = diffProps(instance, newProps)
      if (Object.keys(changedProps).length) {
        Object.assign(instance.props, changedProps)
        applyProps(instance.object, changedProps)
      }
    }

    // Flush reconstructed siblings when we hit the last updated child in a sequence
    const isTailSibling = fiber.sibling === null || (fiber.flags & Update) === NoFlags
    if (isTailSibling) swapInstances()
  },
  finalizeInitialChildren: () => false,
  commitMount() {},
  getPublicInstance: (instance) => instance?.object!,
  prepareForCommit: () => null,
  preparePortalMount: (container) => prepare(container.getState().scene, container, '', {}),
  resetAfterCommit: () => {},
  shouldSetTextContent: () => false,
  clearContainer: () => false,
  hideInstance,
  unhideInstance,
  createTextInstance: handleTextInstance,
  hideTextInstance: handleTextInstance,
  unhideTextInstance: handleTextInstance,
  scheduleTimeout: (typeof setTimeout === 'function' ? setTimeout : undefined) as any,
  cancelTimeout: (typeof clearTimeout === 'function' ? clearTimeout : undefined) as any,
  noTimeout: -1,
  getInstanceFromNode: () => null,
  beforeActiveInstanceBlur() {},
  afterActiveInstanceBlur() {},
  detachDeletedInstance() {},
  prepareScopeUpdate() {},
  getInstanceFromScope: () => null,
  shouldAttemptEagerTransition: () => false,
  trackSchedulerEvent: () => {},
  resolveEventType: () => null,
  resolveEventTimeStamp: () => -1.1,
  requestPostPaintCallback() {},
  maySuspendCommit: () => false,
  preloadInstance: () => true, // true indicates already loaded
  startSuspendingCommit() {},
  suspendInstance() {},
  waitForCommitToBeReady: () => null,
  NotPendingTransition: null,
  // The reconciler types use the internal ReactContext with all the hidden properties
  // so we have to cast from the public React.Context type
  HostTransitionContext: /* @__PURE__ */ React.createContext<HostConfig['TransitionStatus']>(
    null,
  ) as unknown as Reconciler.ReactContext<HostConfig['TransitionStatus']>,
  setCurrentUpdatePriority(newPriority: number) {
    currentUpdatePriority = newPriority
  },
  getCurrentUpdatePriority() {
    return currentUpdatePriority
  },
  resolveUpdatePriority() {
    if (currentUpdatePriority !== NoEventPriority) return currentUpdatePriority

    switch (typeof window !== 'undefined' && window.event?.type) {
      case 'click':
      case 'contextmenu':
      case 'dblclick':
      case 'pointercancel':
      case 'pointerdown':
      case 'pointerup':
        return DiscreteEventPriority
      case 'pointermove':
      case 'pointerout':
      case 'pointerover':
      case 'pointerenter':
      case 'pointerleave':
      case 'wheel':
        return ContinuousEventPriority
      default:
        return DefaultEventPriority
    }
  },
  resetFormInstance() {},
})
```
