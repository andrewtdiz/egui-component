The host renderer pattern spans three files: `reconciler.tsx` (the HostConfig), `utils.tsx` (prop application), and `renderer.tsx` (root bootstrapping). Here is the full pipeline:

---

## 1. The Catalogue — JSX type → constructor mapping

A module-level `catalogue` object maps PascalCase names to Three.js constructors. `extend()` populates it: [1](#0-0) 

When you write `<mesh />`, the type string `"mesh"` is PascalCased to `"Mesh"` and looked up in `catalogue["Mesh"]` → `THREE.Mesh`.

---

## 2. `createInstance` — React element → bare `Instance` descriptor

When React encounters a host element, it calls `createInstance`. Crucially, **no Three.js object is constructed yet**. It only calls `prepare()` to create a lightweight `Instance` descriptor: [2](#0-1) 

`prepare()` in `utils.tsx` allocates the `Instance` struct (with `root`, `type`, `parent`, `children`, `props`, `object`, `handlers`, etc.) and sets `object.__r3f = instance` as a back-pointer: [3](#0-2) 

The `Instance` interface is the internal node type: [4](#0-3) 

---

## 3. `handleContainerEffects` — deferred object construction + scene attachment

The actual `new THREE.Mesh(...)` call is **deferred** until the instance is appended to a mounted parent. This avoids constructing objects that Suspense might discard. It happens inside `handleContainerEffects`, called by `appendChild`/`insertBefore`: [5](#0-4) 

Key steps inside:
- **Guard**: bails if the parent isn't rooted in the scene yet (line 222)
- **Construct**: `new target(...args)` or uses `props.object` for `<primitive>` (line 230)
- **Back-link**: `child.object.__r3f = child` (line 231)
- **Apply props**: `applyProps(child.object, child.props)` (line 235)
- **Attach**: either via `attach` prop (property assignment) or `parent.object.add(child.object)` for `Object3D` trees (lines 238–258)
- **Recurse**: walks the subtree to handle already-queued children (line 262)

---

## 4. `applyProps` — smart prop diffing and application

`applyProps` in `utils.tsx` handles the impedance mismatch between React props and Three.js's mutable API: [6](#0-5) 

It handles:
- **Event handlers** (`onPointerDown`, etc.) → stored in `instance.handlers`, registered in `rootState.internal.interaction`
- **`THREE.Color`** → `.set(value)` for color representations
- **Math objects** with `.copy()` (e.g. `Vector3`, `Quaternion`) → `.copy(value)` if constructors match
- **Objects with `.set()` + array** → `.fromArray(value)` or `.set(...value)`
- **Dash-piercing** (`material-color`) → resolved via `resolve()` to nested property paths
- **Auto-attach**: geometries get `attach="geometry"`, materials get `attach="material"` automatically (lines 495–498)

---

## 5. `commitUpdate` — prop updates and reconstruction

On re-render, the reconciler calls `commitUpdate`. If only props changed, `diffProps` + `applyProps` is used. If `args` changed (constructor arguments), the instance is queued for full reconstruction via `swapInstances()`: [7](#0-6) 

`swapInstances()` detaches the old object, disposes it on idle (via `scheduler`), constructs a new one, and re-attaches children: [8](#0-7) 

---

## 6. `HostConfig` wiring — the reconciler contract

The `HostConfig` type maps R3F's types to the reconciler's generic slots: [9](#0-8) 

The container is the `RootStore` (a zustand store). `appendChildToContainer` resolves the scene's `__r3f` instance and delegates to `appendChild`: [10](#0-9) 

`getPublicInstance` returns `instance.object` — so `ref` on any JSX element gives you the raw Three.js object directly: [11](#0-10) 

---

## 7. Root bootstrapping in `renderer.tsx`

`createRoot` creates the zustand store and calls `reconciler.createContainer(store, ConcurrentRoot, ...)`. The store (not a DOM node) is the container. `render()` calls `reconciler.updateContainer(...)` with a `<Provider>` wrapper that injects the store via React context: [12](#0-11) [13](#0-12) 

---

## Summary flow

```
JSX <mesh args={[...]} position={[0,1,0]} />
        │
        ▼
createInstance()          → allocates Instance{type, props, object:null}
        │
        ▼
appendChild()             → links parent/child in Instance tree
        │
        ▼
handleContainerEffects()  → new THREE.Mesh(...args)
                            object.__r3f = instance
                            applyProps(object, props)
                            parent.object.add(object)   ← enters THREE scene graph
        │
        ▼
commitUpdate() on re-render
  ├─ args changed?  → swapInstances() (dispose old, new THREE.Mesh(...newArgs))
  └─ props changed? → diffProps() + applyProps()
```

### Citations

**File:** packages/fiber/src/core/reconciler.tsx (L105-117)
```typescript
export interface Instance<O = any> {
  root: RootStore
  type: string
  parent: Instance | null
  children: Instance[]
  props: InstanceProps<O> & Record<string, unknown>
  object: O & { __r3f?: Instance<O> }
  eventCount: number
  handlers: Partial<EventHandlers>
  attach?: AttachType<O>
  previousAttach?: any
  isHidden: boolean
}
```

**File:** packages/fiber/src/core/reconciler.tsx (L119-134)
```typescript
interface HostConfig {
  type: string
  props: Instance['props']
  container: RootStore
  instance: Instance
  textInstance: void
  suspenseInstance: Instance
  hydratableInstance: never
  formInstance: never
  publicInstance: Instance['object']
  hostContext: {}
  childSet: never
  timeoutHandle: number | undefined
  noTimeout: -1
  TransitionStatus: null
}
```

**File:** packages/fiber/src/core/reconciler.tsx (L136-158)
```typescript
const catalogue: Catalogue = {}

const PREFIX_REGEX = /^three(?=[A-Z])/

const toPascalCase = (type: string): string => `${type[0].toUpperCase()}${type.slice(1)}`

let i = 0

const isConstructor = (object: unknown): object is ConstructorRepresentation => typeof object === 'function'

export function extend<T extends ConstructorRepresentation>(objects: T): React.ExoticComponent<ThreeElement<T>>
export function extend<T extends Catalogue>(objects: T): void
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

**File:** packages/fiber/src/core/reconciler.tsx (L178-188)
```typescript
function createInstance(type: string, props: HostConfig['props'], root: RootStore): HostConfig['instance'] {
  // Remove three* prefix from elements if native element not present
  type = toPascalCase(type) in catalogue ? type : type.replace(PREFIX_REGEX, '')

  validateInstance(type, props)

  // Regenerate the R3F instance for primitives to simulate a new object
  if (type === 'primitive' && props.object?.__r3f) delete props.object.__r3f

  return prepare(props.object, root, type, props)
}
```

**File:** packages/fiber/src/core/reconciler.tsx (L218-266)
```typescript
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

**File:** packages/fiber/src/core/reconciler.tsx (L376-444)
```typescript
function swapInstances(): void {
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

  // Update instance
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

      // Tree was updated, request a frame
      invalidateInstance(instance)
    }
  }

  reconstructed.length = 0
}
```

**File:** packages/fiber/src/core/reconciler.tsx (L483-500)
```typescript
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
```

**File:** packages/fiber/src/core/reconciler.tsx (L503-536)
```typescript
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
```

**File:** packages/fiber/src/core/reconciler.tsx (L539-539)
```typescript
  getPublicInstance: (instance) => instance?.object!,
```

**File:** packages/fiber/src/core/utils.tsx (L233-255)
```typescript
export function prepare<T = any>(target: T, root: RootStore, type: string, props: Instance<T>['props']): Instance<T> {
  const object = target as unknown as Instance['object']

  // Create instance descriptor
  let instance = object?.__r3f
  if (!instance) {
    instance = {
      root,
      type,
      parent: null,
      children: [],
      props: getInstanceProps(props),
      object,
      eventCount: 0,
      handlers: {},
      isHidden: false,
    }

    if (object) object.__r3f = instance
  }

  return instance
}
```

**File:** packages/fiber/src/core/utils.tsx (L389-504)
```typescript
export function applyProps<T = any>(object: Instance<T>['object'], props: Instance<T>['props']): Instance<T>['object'] {
  const instance = object.__r3f
  const rootState = instance && findInitialRoot(instance).getState()
  const prevHandlers = instance?.eventCount

  for (const prop in props) {
    let value = props[prop]

    // Don't mutate reserved keys
    if (RESERVED_PROPS.includes(prop)) continue

    // Deal with pointer events, including removing them if undefined
    if (instance && EVENT_REGEX.test(prop)) {
      if (typeof value === 'function') instance.handlers[prop as keyof EventHandlers] = value as any
      else delete instance.handlers[prop as keyof EventHandlers]
      instance.eventCount = Object.keys(instance.handlers).length
      continue
    }

    // Ignore setting undefined props
    // https://github.com/pmndrs/react-three-fiber/issues/274
    if (value === undefined) continue

    let { root, key, target } = resolve(object, prop)

    // Layers must be written to the mask property
    if (target instanceof THREE.Layers && value instanceof THREE.Layers) {
      target.mask = value.mask
    }
    // Set colors if valid color representation for automatic conversion (copy)
    else if (target instanceof THREE.Color && isColorRepresentation(value)) {
      target.set(value)
    }
    // Copy if properties match signatures and implement math interface (likely read-only)
    else if (
      target !== null &&
      typeof target === 'object' &&
      typeof target.set === 'function' &&
      typeof target.copy === 'function' &&
      (value as ClassConstructor | null)?.constructor &&
      (target as ClassConstructor).constructor === (value as ClassConstructor).constructor
    ) {
      target.copy(value)
    }
    // Set array types
    else if (
      target !== null &&
      typeof target === 'object' &&
      typeof target.set === 'function' &&
      Array.isArray(value)
    ) {
      if (typeof target.fromArray === 'function') target.fromArray(value)
      else target.set(...value)
    }
    // Set literal types
    else if (
      target !== null &&
      typeof target === 'object' &&
      typeof target.set === 'function' &&
      typeof value === 'number'
    ) {
      // Allow setting array scalars
      if (typeof target.setScalar === 'function') target.setScalar(value)
      // Otherwise just set single value
      else target.set(value)
    }
    // Else, just overwrite the value
    else {
      root[key] = value

      // Auto-convert sRGB texture parameters for built-in materials
      // https://github.com/pmndrs/react-three-fiber/issues/344
      // https://github.com/mrdoob/three.js/pull/25857
      if (
        rootState &&
        !rootState.linear &&
        colorMaps.includes(key) &&
        (root[key] as unknown as THREE.Texture | undefined)?.isTexture &&
        // sRGB textures must be RGBA8 since r137 https://github.com/mrdoob/three.js/pull/23129
        root[key].format === THREE.RGBAFormat &&
        root[key].type === THREE.UnsignedByteType
      ) {
        // NOTE: this cannot be set from the renderer (e.g. sRGB source textures rendered to P3)
        root[key].colorSpace = THREE.SRGBColorSpace
      }
    }
  }

  // Register event handlers
  if (
    instance?.parent &&
    rootState?.internal &&
    (instance.object as unknown as THREE.Object3D | undefined)?.isObject3D &&
    prevHandlers !== instance.eventCount
  ) {
    const object = instance.object as unknown as THREE.Object3D
    // Pre-emptively remove the instance from the interaction manager
    const index = rootState.internal.interaction.indexOf(object)
    if (index > -1) rootState.internal.interaction.splice(index, 1)
    // Add the instance to the interaction manager only when it has handlers
    if (instance.eventCount && object.raycast !== null) {
      rootState.internal.interaction.push(object)
    }
  }

  // Auto-attach geometries and materials
  if (instance && instance.props.attach === undefined) {
    if ((instance.object as unknown as THREE.BufferGeometry).isBufferGeometry) instance.props.attach = 'geometry'
    else if ((instance.object as unknown as THREE.Material).isMaterial) instance.props.attach = 'material'
  }

  // Instance was updated, request a frame
  if (instance) invalidateInstance(instance)

  return object
}
```

**File:** packages/fiber/src/core/renderer.tsx (L162-180)
```typescript
  // Create store
  const store = prevStore || createStore(invalidate, advance)
  // Create renderer
  const fiber =
    prevFiber ||
    (reconciler as any).createContainer(
      store, // container
      ConcurrentRoot, // tag
      null, // hydration callbacks
      false, // isStrictMode
      null, // concurrentUpdatesByDefaultOverride
      '', // identifierPrefix
      logRecoverableError, // onUncaughtError
      logRecoverableError, // onCaughtError
      logRecoverableError, // onRecoverableError
      null, // transitionCallbacks
    )
  // Map it
  if (!prevRoot) _roots.set(canvas, { fiber, store })
```

**File:** packages/fiber/src/core/renderer.tsx (L405-418)
```typescript
    render(children: React.ReactNode): RootStore {
      // The root has to be configured before it can be rendered
      if (!configured && !pending) this.configure()

      pending!.then(() => {
        reconciler.updateContainer(
          <Provider store={store} children={children} onCreated={onCreated} rootElement={canvas} />,
          fiber,
          null,
          () => undefined,
        )
      })

      return store
```
