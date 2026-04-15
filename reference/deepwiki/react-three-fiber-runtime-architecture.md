# DeepWiki Response

Repository: `pmndrs/react-three-fiber`
Question: For a retained React host runtime, what architecture patterns does react-three-fiber use for (1) atomic/transactional host tree updates, (2) stable event-handler routing without duplicate invocation, (3) frame scheduling/timer coordination, and (4) recoverable error boundaries? Provide concrete implementation references that can be adapted to a Rust-hosted JSX->native UI bridge.
Search: https://deepwiki.com/search/for-a-retained-react-host-runt_56dbc754-ae95-4243-92c0-bd5df1c9720f

**Question**: For a retained React host runtime, what architecture patterns does react-three-fiber use for (1) atomic/transactional host tree updates, (2) stable event-handler routing without duplicate invocation, (3) frame scheduling/timer coordination, and (4) recoverable error boundaries? Provide concrete implementation references that can be adapted to a Rust-hosted JSX->native UI bridge.

React Three Fiber (R3F) leverages React's reconciler to manage a retained host tree for Three.js, providing patterns for atomic updates, stable event handling, frame scheduling, and error recovery. These patterns are implemented through a custom React reconciler, a global frame loop, and React's error boundary mechanism. [1](#ref-q1-1)  [2](#ref-q1-2)  [3](#ref-q1-3) 

## Architecture Patterns

### 1. Atomic/Transactional Host Tree Updates

R3F uses a custom React reconciler to manage the Three.js scene graph as its host tree [1](#ref-q1-1) . Updates to the host tree are handled transactionally by React's reconciliation process.

*   **Reconciler Configuration**: The `reconciler` is configured with `supportsMutation: true` [4](#ref-q1-4) , indicating that it directly modifies the host instances.
*   **Instance Creation and Updates**: When a component renders, `createInstance` is called to create a Three.js object [5](#ref-q1-5) . Subsequent updates are processed by `commitUpdate`, which calculates prop differences using `diffProps` and applies them to the Three.js object via `applyProps` [6](#ref-q1-6) .
*   **Tree Manipulation**: `appendChild`, `insertBefore`, and `removeChild` methods manage the hierarchy of Three.js objects [7](#ref-q1-7)  [8](#ref-q1-8) . The `handleContainerEffects` function ensures that objects are properly linked and attached to their parents in the Three.js scene [9](#ref-q1-9) .
*   **Reconstruction**: If fundamental properties like `object` or `args` change for a primitive, the reconciler reconstructs the instance by detaching the old object, disposing of it, and creating a new one [10](#ref-q1-10)  [11](#ref-q1-11)  [12](#ref-q1-12) . This ensures atomicity by replacing the entire object rather than attempting partial updates that might be inconsistent.

### 2. Stable Event-Handler Routing Without Duplicate Invocation

R3F integrates with the browser's DOM events (or React Native's gesture system) and translates them into Three.js-specific events without duplicate invocations.

*   **Event Manager**: The `EventManager` interface defines how events are connected, computed, and filtered [13](#ref-q1-13) .
*   **Event Connection**: The `onCreated` callback in `Canvas.tsx` connects the event manager to the appropriate DOM element or `GLView` [14](#ref-q1-14)  [15](#ref-q1-15) .
*   **Compute Function**: The `compute` function within the event manager is responsible for setting up the raycaster and pointer coordinates based on the native event [16](#ref-q1-16) . This ensures that raycasting is performed correctly for each event.
*   **Reactive Event Props**: Event handlers are updated reactively. When event props change on a `mesh`, the `internal.interaction` array in the `RootState` is updated, ensuring that only the current handlers are active [17](#ref-q1-17) .

### 3. Frame Scheduling/Timer Coordination

R3F manages the rendering loop and coordinates frame updates through a centralized `loop` function and a subscription system.

*   **Frame Loop**: The `loop` function is the core of the rendering pipeline, called via `requestAnimationFrame` [2](#ref-q1-2) . It orchestrates the execution of global effects and renders all active roots [18](#ref-q1-18) .
*   **Frameloop Modes**: The `frameloop` property in `RootState` controls rendering behavior:
    *   `always`: Renders continuously every `requestAnimationFrame` [19](#ref-q1-19) .
    *   `demand`: Renders only when `invalidate()` is called, which increments `internal.frames` [20](#ref-q1-20)  [21](#ref-q1-21) .
    *   `never`: Requires manual calls to `advance()` [22](#ref-q1-22) .
*   **`useFrame` Subscriptions**: Callbacks registered with `useFrame` are stored in `state.internal.subscribers` and executed during the `update` phase of the loop [23](#ref-q1-23) . These subscriptions can have priorities, influencing their execution order [24](#ref-q1-24) .
*   **Global Effects**: `addEffect`, `addAfterEffect`, and `addTail` allow registering global callbacks that run before rendering, after rendering, or when rendering stops, respectively [25](#ref-q1-25)  [26](#ref-q1-26) .

### 4. Recoverable Error Boundaries

R3F utilizes React's error boundary mechanism to catch and recover from errors within the Three.js rendering tree.

*   **`ErrorBoundary` Component**: The `ErrorBoundary` component is a standard React class component that implements `static getDerivedStateFromError` and `componentDidCatch` [3](#ref-q1-3) .
*   **Integration**: The `Canvas` component wraps its children with this `ErrorBoundary` [27](#ref-q1-27)  [28](#ref-q1-28) . If an error occurs during rendering within the R3F tree, it is caught by this boundary, and the `set` prop is called to update the error state [29](#ref-q1-29) .
*   **Recoverable Errors**: The `createRoot` function passes `logRecoverableError` to the React reconciler, allowing React to report errors that it can potentially recover from [30](#ref-q1-30)  [31](#ref-q1-31) . This is crucial for maintaining application stability even when parts of the UI encounter issues.

## Implementation References for Rust-hosted JSX->native UI bridge

To adapt these patterns to a Rust-hosted JSX->native UI bridge, you would need to:

1.  **Custom Reconciler**: Implement a custom React reconciler in Rust that interfaces with your native UI library. This reconciler would need to provide methods equivalent to R3F's `createInstance`, `commitUpdate`, `appendChild`, `insertBefore`, and `removeChild` [32](#ref-q1-32) .
2.  **Host Instances**: Define Rust structs that represent your native UI elements, similar to how R3F uses `Instance` to represent Three.js objects [9](#ref-q1-9) .
3.  **Prop Application**: Develop a mechanism to apply properties from React's virtual DOM to your native UI elements, mirroring R3F's `applyProps` [33](#ref-q1-33) .
4.  **Event System**: Create an event dispatching system that captures native UI events and routes them to the appropriate React event handlers, similar to R3F's `EventManager` and `compute` functions [13](#ref-q1-13) .
5.  **Frame Scheduling**: Implement a frame loop in Rust that can be triggered by a timer or an equivalent of `requestAnimationFrame`. This loop would need to execute subscribed callbacks and manage rendering updates, analogous to R3F's `loop` and `useFrame` [2](#ref-q1-2)  [23](#ref-q1-23) .
6.  **Error Boundaries**: Integrate Rust's error handling mechanisms with React's error boundaries. This would involve catching panics or errors in your native UI code and propagating them back to the React `ErrorBoundary` component [3](#ref-q1-3) . The `logRecoverableError` function passed to the reconciler is a key part of this [31](#ref-q1-31) .

## Notes

The `packages/fiber/src/core/reconciler.tsx` file is central to understanding how React Three Fiber manages the host tree and its updates. [1](#ref-q1-1)  The `packages/fiber/src/core/loop.ts` file details the frame scheduling and global effects system. [2](#ref-q1-2)  Event handling is primarily configured in `packages/fiber/src/web/Canvas.tsx` and `packages/fiber/src/native/Canvas.tsx` through the `EventManager` interface defined in `packages/fiber/src/core/events.ts`. [14](#ref-q1-14)  [15](#ref-q1-15)  [13](#ref-q1-13)  Error boundaries are implemented using a standard React `ErrorBoundary` component in `packages/fiber/src/core/utils.tsx`. [3](#ref-q1-3) 

Wiki pages you might want to explore:
- [Rendering Pipeline (pmndrs/react-three-fiber)](/wiki/pmndrs/react-three-fiber#2.2)

View this search on DeepWiki: https://deepwiki.com/search/for-a-retained-react-host-runt_56dbc754-ae95-4243-92c0-bd5df1c9720f

## References

<a id="ref-q1-1"></a>
### [1] `packages/fiber/src/core/reconciler.tsx:457-472`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L457-L472)

```tsx
export const reconciler = /* @__PURE__ */ createReconciler<
  HostConfig['type'],
  HostConfig['props'],
  HostConfig['container'],
  HostConfig['instance'],
  HostConfig['textInstance'],
  HostConfig['suspenseInstance'],
  HostConfig['hydratableInstance'],
  HostConfig['formInstance'],
  HostConfig['publicInstance'],
  HostConfig['hostContext'],
  HostConfig['childSet'],
  HostConfig['timeoutHandle'],
  HostConfig['noTimeout'],
  HostConfig['TransitionStatus']
>({
```

<a id="ref-q1-2"></a>
### [2] `packages/fiber/src/core/loop.ts:93`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/loop.ts#L93)

```typescript
  frame = requestAnimationFrame(loop)
```

<a id="ref-q1-3"></a>
### [3] `packages/fiber/src/core/utils.tsx:104-117`
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

<a id="ref-q1-4"></a>
### [4] `packages/fiber/src/core/reconciler.tsx:475`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L475)

```tsx
  supportsMutation: true,
```

<a id="ref-q1-5"></a>
### [5] `packages/fiber/src/core/reconciler.tsx:227-230`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L227-L230)

```tsx
    const target = catalogue[toPascalCase(child.type)]

    // Create object
    child.object = child.props.object ?? new target(...(child.props.args ?? []))
```

<a id="ref-q1-6"></a>
### [6] `packages/fiber/src/core/reconciler.tsx:526-529`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L526-L529)

```tsx
      const changedProps = diffProps(instance, newProps)
      if (Object.keys(changedProps).length) {
        Object.assign(instance.props, changedProps)
        applyProps(instance.object, changedProps)
```

<a id="ref-q1-7"></a>
### [7] `packages/fiber/src/core/reconciler.tsx:268-277`
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

<a id="ref-q1-8"></a>
### [8] `packages/fiber/src/core/reconciler.tsx:279-294`
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

<a id="ref-q1-9"></a>
### [9] `packages/fiber/src/core/reconciler.tsx:218-266`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L218-L266)

```tsx
function handleContainerEffects(parent: Instance, child: Instance, beforeChild?: Instance) {
  // Bail if tree isn't mounted or parent is not a container.
  // This ensures that the tree is finalized and React won't discard results to Suspense
  const state = child.root.getState()
  if (!parent.parent && parent.object !== state.scene) return

  // Create & link object on first run
  if (!child.object) {
    // Get target from catalogue
    const target = catalogue[toPascalCase(child.type)]

    // Create object
    child.object = child.props.object ?? new target(...(child.props.args ?? []))
    child.object.__r3f = child
  }

  // Set initial props
  applyProps(child.object, child.props)

  // Append instance
  if (child.props.attach) {
    attach(parent, child)
  } else if (isObject3D(child.object) && isObject3D(parent.object)) {
    const childIndex = parent.object.children.indexOf(beforeChild?.object)
    if (beforeChild && childIndex !== -1) {
      // If the child is already in the parent's children array, move it to the new position
      // Otherwise, just insert it at the target position
      const existingIndex = parent.object.children.indexOf(child.object)
      if (existingIndex !== -1) {
        parent.object.children.splice(existingIndex, 1)
        const adjustedIndex = existingIndex < childIndex ? childIndex - 1 : childIndex
        parent.object.children.splice(adjustedIndex, 0, child.object)
      } else {
        child.object.parent = parent.object
        parent.object.children.splice(childIndex, 0, child.object)
        child.object.dispatchEvent({ type: 'added' })
        parent.object.dispatchEvent({ type: 'childadded', child: child.object })
      }
    } else {
      parent.object.add(child.object)
    }
  }

  // Link subtree
  for (const childInstance of child.children) handleContainerEffects(child, childInstance)

  // Tree was updated, request a frame
  invalidateInstance(child)
}
```

<a id="ref-q1-10"></a>
### [10] `packages/fiber/src/core/reconciler.tsx:514-523`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L514-L523)

```tsx
    // Reconstruct primitives if object prop changes
    if (instance.type === 'primitive' && oldProps.object !== newProps.object) reconstruct = true
    // Reconstruct instance if args were added or removed
    else if (newProps.args?.length !== oldProps.args?.length) reconstruct = true
    // Reconstruct instance if args were changed
    else if (newProps.args?.some((value, index) => value !== oldProps.args?.[index])) reconstruct = true

    // Reconstruct when args or <primitive object={...} have changes
    if (reconstruct) {
      reconstructed.push([instance, { ...newProps }, fiber])
```

<a id="ref-q1-11"></a>
### [11] `packages/fiber/src/core/reconciler.tsx:377-405`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L377-L405)

```tsx
  // Detach instance
  for (const [instance] of reconstructed) {
    const parent = instance.parent
    if (parent) {
      if (instance.props.attach) {
        detach(parent, instance)
      } else if (isObject3D(instance.object) && isObject3D(parent.object)) {
        parent.object.remove(instance.object)
      }

      for (const child of instance.children) {
        if (child.props.attach) {
          detach(instance, child)
        } else if (isObject3D(child.object) && isObject3D(instance.object)) {
          instance.object.remove(child.object)
        }
      }
    }

    // If the old instance is hidden, we need to unhide it.
    // React assumes it can discard instances since they're pure for DOM.
    // This isn't true for us since our lifetimes are impure and longliving.
    // So, we manually check if an instance was hidden and unhide it.
    if (instance.isHidden) unhideInstance(instance)

    // Dispose of old object if able
    if (instance.object.__r3f) delete instance.object.__r3f
    if (instance.type !== 'primitive') disposeOnIdle(instance.object)
  }
```

<a id="ref-q1-12"></a>
### [12] `packages/fiber/src/core/reconciler.tsx:408-436`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L408-L436)

```tsx
  for (const [instance, props, fiber] of reconstructed) {
    instance.props = props

    const parent = instance.parent
    if (parent) {
      // Get target from catalogue
      const target = catalogue[toPascalCase(instance.type)]

      // Create object
      instance.object = instance.props.object ?? new target(...(instance.props.args ?? []))
      instance.object.__r3f = instance
      setFiberRef(fiber, instance.object)

      // Set initial props
      applyProps(instance.object, instance.props)

      if (instance.props.attach) {
        attach(parent, instance)
      } else if (isObject3D(instance.object) && isObject3D(parent.object)) {
        parent.object.add(instance.object)
      }

      for (const child of instance.children) {
        if (child.props.attach) {
          attach(instance, child)
        } else if (isObject3D(child.object) && isObject3D(instance.object)) {
          instance.object.add(child.object)
        }
      }
```

<a id="ref-q1-13"></a>
### [13] `packages/fiber/src/core/events.ts:71-92`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/events.ts#L71-L92)

```typescript
export interface EventManager<TTarget> {
  /** Determines if the event layer is active */
  enabled: boolean
  /** Event layer priority, higher prioritized layers come first and may stop(-propagate) lower layer  */
  priority: number
  /** The compute function needs to set up the raycaster and an xy- pointer  */
  compute?: ComputeFunction
  /** The filter can re-order or re-structure the intersections  */
  filter?: FilterFunction
  /** The target node the event layer is tied to */
  connected?: TTarget
  /** All the pointer event handlers through which the host forwards native events */
  handlers?: Events
  /** Allows re-connecting to another target */
  connect?: (target: TTarget) => void
  /** Removes all existing events handlers from the target */
  disconnect?: () => void
  /** Triggers a onPointerMove with the last known event. This can be useful to enable raycasting without
   *  explicit user interaction, for instance when the camera moves a hoverable object underneath the cursor.
   */
  update?: () => void
}
```

<a id="ref-q1-14"></a>
### [14] `packages/fiber/src/web/Canvas.tsx:107-111`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/web/Canvas.tsx#L107-L111)

```tsx
          onCreated: (state) => {
            // Connect to event source
            state.events.connect?.(
              eventSource ? (isRef(eventSource) ? eventSource.current : eventSource) : divRef.current,
            )
```

<a id="ref-q1-15"></a>
### [15] `packages/fiber/src/native/Canvas.tsx:197-207`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/native/Canvas.tsx#L197-L207)

```tsx
          onCreated: (state: RootState) => {
            // Bind render to RN bridge
            const context = state.gl.getContext() as ExpoWebGLRenderingContext
            const renderFrame = state.gl.render.bind(state.gl)
            state.gl.render = (scene: THREE.Scene, camera: THREE.Camera) => {
              renderFrame(scene, camera)
              context.endFrameEXP()
            }

            return onCreated?.(state)
          },
```

<a id="ref-q1-16"></a>
### [16] `packages/fiber/src/web/Canvas.tsx:113-120`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/web/Canvas.tsx#L113-L120)

```tsx
            if (eventPrefix) {
              state.setEvents({
                compute: (event, state) => {
                  const x = event[(eventPrefix + 'X') as keyof DomEvent] as number
                  const y = event[(eventPrefix + 'Y') as keyof DomEvent] as number
                  state.pointer.set((x / state.size.width) * 2 - 1, -(y / state.size.height) * 2 + 1)
                  state.raycaster.setFromCamera(state.pointer, state.camera)
                },
```

<a id="ref-q1-17"></a>
### [17] `packages/fiber/tests/renderer.test.tsx:281-295`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/tests/renderer.test.tsx#L281-L295)

```tsx

    // Set
    await act(async () => root.render(<mesh onClick={() => void 0} />))
    expect(internal.interaction.length).toBe(1)
    expect(internal.interaction).toStrictEqual([mesh])

    // Update
    await act(async () => root.render(<mesh onPointerOver={() => void 0} />))
    expect(internal.interaction.length).toBe(1)
    expect(internal.interaction).toStrictEqual([mesh])

    // Unset
    await act(async () => root.render(<mesh />))
    expect(internal.interaction.length).toBe(0)
  })
```

<a id="ref-q1-18"></a>
### [18] `packages/fiber/src/core/loop.ts:98-113`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/loop.ts#L98-L113)

```typescript
  flushGlobalEffects('before', timestamp)

  // Render all roots
  useFrameInProgress = true
  for (const root of _roots.values()) {
    state = root.store.getState()

    // If the frameloop is invalidated, do not run another frame
    if (
      state.internal.active &&
      (state.frameloop === 'always' || state.internal.frames > 0) &&
      !state.gl.xr?.isPresenting
    ) {
      repeat += update(timestamp, state)
    }
  }
```

<a id="ref-q1-19"></a>
### [19] `packages/fiber/src/core/loop.ts:108`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/loop.ts#L108)

```typescript
      (state.frameloop === 'always' || state.internal.frames > 0) &&
```

<a id="ref-q1-20"></a>
### [20] `docs/advanced/scaling-performance.mdx:26-27`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/docs/advanced/scaling-performance.mdx#L26-L27)

```

One major caveat is that if anything in the tree _mutates_ props, then React cannot be aware of it and the display would be stale. For instance, camera controls just grab into the camera and mutate its values. Here you can use React Three Fiber's `invalidate` function to trigger frames manually.
```

<a id="ref-q1-21"></a>
### [21] `packages/fiber/src/core/loop.ts:83`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/loop.ts#L83)

```typescript
  return state.frameloop === 'always' ? 1 : state.internal.frames
```

<a id="ref-q1-22"></a>
### [22] `packages/fiber/src/core/loop.ts:65-69`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/loop.ts#L65-L69)

```typescript
  if (state.frameloop === 'never' && typeof timestamp === 'number') {
    delta = timestamp - state.clock.elapsedTime
    state.clock.oldTime = state.clock.elapsedTime
    state.clock.elapsedTime = timestamp
  }
```

<a id="ref-q1-23"></a>
### [23] `packages/fiber/src/core/loop.ts:71-76`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/loop.ts#L71-L76)

```typescript
  // Call subscribers (useFrame)
  subscribers = state.internal.subscribers
  for (let i = 0; i < subscribers.length; i++) {
    subscription = subscribers[i]
    subscription.ref.current(subscription.store.getState(), delta, frame)
  }
```

<a id="ref-q1-24"></a>
### [24] `docs/API/hooks.mdx:149-151`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/docs/API/hooks.mdx#L149-L151)

```
> [!NOTE]
> Callbacks will be executed in order of ascending priority values (lowest first, highest last.), similar to the DOM's z-order.
```

<a id="ref-q1-25"></a>
### [25] `packages/fiber/src/core/loop.ts:23-35`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/loop.ts#L23-L35)

```typescript
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

<a id="ref-q1-26"></a>
### [26] `packages/fiber/src/core/loop.ts:46-55`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/loop.ts#L46-L55)

```typescript
export function flushGlobalEffects(type: GlobalEffectType, timestamp: number): void {
  switch (type) {
    case 'before':
      return run(globalEffects, timestamp)
    case 'after':
      return run(globalAfterEffects, timestamp)
    case 'tail':
      return run(globalTailEffects, timestamp)
  }
}
```

<a id="ref-q1-27"></a>
### [27] `packages/fiber/src/web/Canvas.tsx:129-131`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/web/Canvas.tsx#L129-L131)

```tsx
            <ErrorBoundary set={setError}>
              <React.Suspense fallback={<Block set={setBlock} />}>{children ?? null}</React.Suspense>
            </ErrorBoundary>
```

<a id="ref-q1-28"></a>
### [28] `packages/fiber/src/native/Canvas.tsx:211-213`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/native/Canvas.tsx#L211-L213)

```tsx
            <ErrorBoundary set={setError}>
              <React.Suspense fallback={<Block set={setBlock} />}>{children ?? null}</React.Suspense>
            </ErrorBoundary>
```

<a id="ref-q1-29"></a>
### [29] `packages/fiber/src/core/utils.tsx:111-112`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/utils.tsx#L111-L112)

```tsx
    componentDidCatch(err: Error) {
      this.props.set(err)
```

<a id="ref-q1-30"></a>
### [30] `packages/fiber/src/core/renderer.tsx:154-160`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/renderer.tsx#L154-L160)

```tsx
  const logRecoverableError =
    typeof reportError === 'function'
      ? // In modern browsers, reportError will dispatch an error event,
        // emulating an uncaught JavaScript error.
        reportError
      : // In older browsers and test environments, fallback to console.error.
        console.error
```

<a id="ref-q1-31"></a>
### [31] `packages/fiber/src/core/renderer.tsx:173-176`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/renderer.tsx#L173-L176)

```tsx
      '', // identifierPrefix
      logRecoverableError, // onUncaughtError
      logRecoverableError, // onCaughtError
      logRecoverableError, // onRecoverableError
```

<a id="ref-q1-32"></a>
### [32] `packages/fiber/src/core/reconciler.tsx:478-482`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L478-L482)

```tsx
  createInstance,
  removeChild,
  appendChild,
  appendInitialChild: appendChild,
  insertBefore,
```

<a id="ref-q1-33"></a>
### [33] `packages/fiber/src/core/reconciler.tsx:235`
Source: [pmndrs/react-three-fiber @ e53d667a](https://github.com/pmndrs/react-three-fiber/blob/e53d667a/packages/fiber/src/core/reconciler.tsx#L235)

```tsx
  applyProps(child.object, child.props)
```
