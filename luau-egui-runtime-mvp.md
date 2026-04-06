
# MVP architecture: Luau + egui UI runtime with hot reload

## Goal

Build the smallest architecture that is still structurally correct for an in-process **Luau-driven egui UI runtime** with **fast edit / save / reload iteration**.

The design below optimizes for:

- fast implementation velocity
- good runtime performance for editor and debug UI
- low integration complexity into a new engine runtime
- clean upgrade paths later

It intentionally does **not** optimize for:

- multi-language scripting
- generic reflection over the whole engine
- background script execution
- multi-window egui viewport management
- rich object graphs or retained UI trees

---

## The recommended MVP shape

Use a **single main-thread UI runtime**:

- one `egui::Context` owned by the engine UI layer
- one shared Luau VM owned by the same thread
- one custom `require` implementation with module cache + dependency graph
- one hot-reload queue fed by a filesystem watcher
- one root Luau module that renders the UI each frame
- one persistent `state` table per hot-reloadable module

This gives the best initial tradeoff.

It avoids:

- cross-thread UI synchronization
- command marshalling across a client API boundary
- generic `Variant`-style value transport for every call
- complex script instance registries
- difficult lifetime bugs from spreading UI work across threads

### Why this is the right MVP

For egui, the natural control flow is:

1. gather input
2. run `ctx.run(raw_input, |ctx| { ... })`
3. build widgets immediately
4. process `FullOutput`
5. tessellate and paint

For Luau, the lowest-friction host integration is:

1. compile source to bytecode
2. load bytecode into the VM
3. keep one sandboxed state alive
4. expose a very small host API
5. reload code only at frame boundaries

Put together, the simplest architecture is: **Luau executes directly inside the egui frame callback on the main UI thread**.

---

## Top-level runtime diagram

```mermaid
flowchart LR
    OS[OS/window/input] --> Input[RawInput builder]
    Watcher[filesystem watcher] --> Dirty[dirty module queue]

    subgraph Engine Main Thread
        Input --> Frame[UiRuntime::frame]
        Dirty --> Reload[apply_pending_reload]

        Reload --> VM[Luau VM + module cache]
        Frame --> EguiRun[egui_ctx.run]
        EguiRun --> Script[call root.render(state)]
        Script --> Bindings[host ui/app bindings]
        Bindings --> EguiWidgets[egui widgets]
        EguiWidgets --> Output[FullOutput]
        Output --> Tess[tessellate]
        Tess --> Paint[renderer]
    end
```

---

## Core runtime pieces

### `UiRuntime`

Owns the frame loop and the hot-reload boundary.

```rust
pub struct UiRuntime {
    pub egui_ctx: egui::Context,
    pub vm: ScriptVm,
    pub reloader: HotReloadManager,
    pub frame_scope: FrameScope,
    pub root: ModuleId,
    pub error_overlay: Option<String>,
}
```

Responsibilities:

- collect `egui::RawInput`
- apply queued reloads before a frame starts
- call the Luau root `render` function during `egui_ctx.run`
- collect `FullOutput`
- tessellate shapes
- expose input-capture results back to the engine
- render a fallback error panel if compile or runtime errors occur

### `ScriptVm`

Owns the Luau state, module cache, registry refs, and binding registration.

```rust
pub struct ScriptVm {
    pub state: *mut lua_State,
    pub modules: HashMap<ModuleId, ModuleEntry>,
    pub path_index: HashMap<PathBuf, ModuleId>,
    pub root_globals_ref: LuauRef,
}
```

### `ModuleEntry`

Tracks the minimum metadata required for correct reload.

```rust
pub struct ModuleEntry {
    pub id: ModuleId,
    pub name: String,
    pub path: PathBuf,
    pub exports_ref: LuauRef,
    pub state_ref: LuauRef,
    pub env_ref: LuauRef,
    pub dependencies: Vec<ModuleId>,
    pub reverse_deps: Vec<ModuleId>,
    pub source_hash: u64,
    pub version: u64,
    pub last_error: Option<String>,
}
```

### `HotReloadManager`

The hot-reload queue should never mutate the VM directly from the watcher thread.

```rust
pub struct HotReloadManager {
    pub dirty_paths: crossbeam_queue::SegQueue<PathBuf>,
}
```

Watcher thread behavior:

- observe file changes
- push changed paths into `dirty_paths`
- wake the main loop

Main thread behavior:

- drain `dirty_paths`
- rebuild affected subgraph
- atomically swap successful modules
- keep last good version alive on failure

### `FrameScope`

This is the frame-local bridge used by host bindings.

```rust
pub struct FrameScope {
    pub active_ctx: Option<*const egui::Context>,
    pub ui_stack: Vec<*mut egui::Ui>,
    pub input_capture_pointer: bool,
    pub input_capture_keyboard: bool,
}
```

`FrameScope` is **only valid during `render()`**. Nothing stored in it may escape the frame.

---

## The actual frame pipeline

This is the core host loop.

```rust
pub struct UiFrameOutput {
    pub platform_output: egui::PlatformOutput,
    pub textures_delta: egui::TexturesDelta,
    pub primitives: Vec<egui::ClippedPrimitive>,
    pub viewport_output: egui::viewport::ViewportIdMap<egui::ViewportOutput>,
    pub wants_pointer: bool,
    pub wants_keyboard: bool,
}

impl UiRuntime {
    pub fn frame(&mut self, raw_input: egui::RawInput) -> UiFrameOutput {
        self.apply_pending_reload();

        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            self.frame_scope.begin(ctx);

            let render_result = self.vm.call_root_render(&mut self.frame_scope);

            if let Err(err) = render_result {
                self.error_overlay = Some(err);
            }

            if let Some(err) = &self.error_overlay {
                egui::Window::new("Luau UI Error")
                    .default_open(true)
                    .show(ctx, |ui| {
                        ui.colored_label(egui::Color32::RED, "Script error");
                        ui.separator();
                        ui.code(err);
                    });
            }

            self.frame_scope.end();
        });

        let primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        UiFrameOutput {
            platform_output: full_output.platform_output,
            textures_delta: full_output.textures_delta,
            primitives,
            viewport_output: full_output.viewport_output,
            wants_pointer: self.egui_ctx.egui_wants_pointer_input(),
            wants_keyboard: self.egui_ctx.egui_wants_keyboard_input(),
        }
    }
}
```

### Important notes

- `egui::RawInput` coordinates are in **points**, not physical pixels.
- `FullOutput.textures_delta.set` must be applied **before painting**.
- `FullOutput.textures_delta.free` must be released **after painting**.
- `viewport_output` supports multiple viewports, but the MVP should support **root viewport only**.

If non-root viewports appear in the MVP, log them and ignore them instead of building half-working multi-window support.

---

## The Luau embedding boundary

Use a **thin C/C++ shim** around the official Luau compiler + VM APIs, then call that shim from Rust.

Do **not** build the whole runtime design around a third-party abstraction until the MVP is working.

### Minimal compile/load shim

```cpp
// luau_host.cpp
#include "lua.h"
#include "lualib.h"
#include "Luau/Compiler.h"
#include "Luau/BytecodeBuilder.h"

extern "C" int luau_compile_and_load(
    lua_State* L,
    const char* chunk_name,
    const char* source,
    size_t source_len,
    char* errbuf,
    size_t errcap)
{
    size_t bytecode_size = 0;
    char* bytecode = luau_compile(source, source_len, nullptr, &bytecode_size);

    int status = luau_load(L, chunk_name, bytecode, bytecode_size, 0);
    free(bytecode);

    if (status != 0) {
        const char* err = lua_tostring(L, -1);
        snprintf(errbuf, errcap, "%s", err ? err : "unknown Luau load error");
        return status;
    }

    return 0;
}
```

### VM initialization rules

At startup:

1. create the Luau state
2. open standard libraries you want
3. sandbox the default globals
4. register host libraries (`app`, `ui`, `asset`, etc.)
5. install the custom `require`
6. install an interrupt/watchdog hook

Recommended MVP rules:

- one VM
- one main execution thread
- sandboxed globals
- custom module environments layered over readonly builtins
- no file system access from scripts
- no dynamic native module loading
- no direct engine object reflection

### What to avoid in the MVP

Do not start with:

- host object userdata for everything
- generic table-to-host reflection
- arbitrary callback storage into host objects
- per-widget host userdata
- script execution on worker threads

For the MVP, prefer:

- primitive returns (`bool`, `number`, `string`)
- explicit ids
- opaque integer handles if absolutely needed
- separate `state` tables for persistent data

---

## The script contract

Every hot-reloadable UI module should return a plain table with a narrow lifecycle.

### Recommended module contract

```lua
export type Module = {
    init: ((state: {[string]: any}) -> ())?,
    render: (state: {[string]: any}) -> (),
    reload: ((old_exports: any, state: {[string]: any}) -> ())?,
    shutdown: ((state: {[string]: any}) -> ())?,
}
```

### Example root script

```lua
local M = {}

function M.init(state)
    state.counter = state.counter or 0
    state.name = state.name or "world"
end

function M.render(state)
    ui.window({ id = "demo.main", title = "Demo" }, function()
        ui.label("Hello, " .. state.name)

        if ui.button("Increment") then
            state.counter += 1
        end

        ui.label("Counter: " .. tostring(state.counter))

        local changed, new_name = ui.text_edit("name", state.name)
        if changed then
            state.name = new_name
        end
    end)
end

function M.reload(old, state)
    -- optional migration hook
    state.counter = state.counter or 0
end

return M
```

### Why this contract works

- `state` survives reloads
- code is disposable
- module exports are replaceable
- reload migration is explicit
- render stays immediate-mode and frame-local

---

## The host API surface

Keep the first public API very small.

### `app` namespace

```lua
app.log(level: string, message: string)
app.request_reload()
app.request_repaint()
app.now_seconds() -> number
```

### `ui` namespace

```lua
ui.window(opts, callback)
ui.horizontal(callback)
ui.vertical(callback)
ui.collapsing(label, callback) -> bool

ui.label(text)
ui.separator()
ui.spacing()

ui.button(text) -> bool
ui.checkbox(label, value: boolean) -> (boolean, boolean)
ui.slider_float(id, label, value: number, min: number, max: number) -> (boolean, number)
ui.text_edit(id, value: string) -> (boolean, string)

ui.push_id(id, callback)
```

That is enough to build:

- debug panels
- simple inspector UIs
- toolbars
- settings screens
- consoles
- property editors

Do not add tables of generic widget options yet unless the need is real.

---

## The binding model

The binding model should be **direct and typed**, not generic.

### Good pattern

```lua
if ui.button("Reload") then
    app.request_reload()
end

local changed, speed = ui.slider_float("speed", "Speed", state.speed, 0.0, 10.0)
if changed then
    state.speed = speed
end
```

### Avoid this pattern

```lua
local response = ui.widget({
    kind = "slider",
    id = "speed",
    label = "Speed",
    min = 0,
    max = 10,
    value = state.speed,
})
```

The typed pattern is better for the MVP because it:

- keeps host bindings small
- avoids a generic marshalling layer
- produces fewer allocations
- makes hot paths obvious
- is much easier to debug

---

## Critical touch point: nested egui scopes from Luau callbacks

egui container widgets use closures. Luau also needs callbacks for nested UI. The bridge must therefore manage an **active UI stack**.

### Host-side pattern

```rust
impl FrameScope {
    pub fn with_ctx<T>(&mut self, f: impl FnOnce(&egui::Context, &mut FrameScope) -> T) -> T {
        let ctx = unsafe { &*self.active_ctx.expect("no active egui context") };
        f(ctx, self)
    }

    pub fn with_ui<T>(&mut self, f: impl FnOnce(&mut egui::Ui) -> T) -> T {
        let ui_ptr = *self.ui_stack.last().expect("no active egui ui");
        let ui = unsafe { &mut *ui_ptr };
        f(ui)
    }

    pub fn push_ui(&mut self, ui: &mut egui::Ui) {
        self.ui_stack.push(ui as *mut _);
    }

    pub fn pop_ui(&mut self) {
        self.ui_stack.pop().expect("ui stack underflow");
    }
}
```

### Example `ui.window`

```rust
fn bind_window(frame: &mut FrameScope, opts: WindowOpts, cb: LuauCallback) -> LuauResult<()> {
    frame.with_ctx(|ctx, frame| {
        egui::Window::new(opts.title)
            .id(egui::Id::new(opts.id))
            .show(ctx, |ui| {
                frame.push_ui(ui);
                let result = cb.call0();
                frame.pop_ui();

                if let Err(err) = result {
                    // convert to engine error reporting
                }
            });
    });

    Ok(())
}
```

### Example `ui.button`

```rust
fn bind_button(frame: &mut FrameScope, text: String) -> LuauResult<bool> {
    Ok(frame.with_ui(|ui| ui.button(text).clicked()))
}
```

### Example `ui.text_edit`

```rust
fn bind_text_edit(frame: &mut FrameScope, id: String, value: String) -> LuauResult<(bool, String)> {
    frame.with_ui(|ui| {
        let mut buf = value;
        let response = ui.push_id(id, |ui| ui.text_edit_singleline(&mut buf)).inner;
        Ok((response.changed(), buf))
    })
}
```

### Important rule

Never let a `Ui`, `Context`, `Response`, or raw pointer escape the frame callback.

Frame-local objects are not stable script objects.

---

## Critical touch point: stable widget ids across hot reload

This matters more than it looks.

egui stores state such as:

- text cursor position
- open/closed collapsing headers
- window state
- selection state
- keyboard focus

That state is keyed by `Id`.

If you rely on implicit ids derived from callsites or dynamic label text, hot reload can destabilize that memory.

### Rule

For every stateful widget or container, expose an explicit script-side id.

Good:

```lua
ui.window({ id = "inspector.main", title = "Inspector" }, function()
    local changed, value = ui.text_edit("search_box", state.query)
end)
```

Bad:

```lua
ui.window({ title = "Inspector" }, function()
    local changed, value = ui.text_edit(state.query)
end)
```

### MVP recommendation

Require explicit ids for:

- windows
- text edits
- sliders
- collapsing containers
- tree-like controls
- any widget that should keep state across reloads

---

## Critical touch point: custom `require` and module cache

Do not use the file system directly from script code.

The host should own all module loading.

### Required behavior

- resolve module path in the host
- track dependency edges
- compile + load module
- cache exports
- preserve per-module `state`
- invalidate reverse dependencies on change
- swap modules only at a safe point

### Host-side loading sketch

```rust
impl ScriptVm {
    pub fn require_module(&mut self, requester: ModuleId, spec: &str) -> Result<LuauRef, String> {
        let target = self.resolve(requester, spec)?;
        self.link_dependency(requester, target);

        if let Some(entry) = self.modules.get(&target) {
            return Ok(entry.exports_ref);
        }

        self.load_module_fresh(target)
    }

    fn load_module_fresh(&mut self, id: ModuleId) -> Result<LuauRef, String> {
        let source = std::fs::read_to_string(self.path_for(id)).map_err(|e| e.to_string())?;
        let state_ref = self.make_or_reuse_state(id)?;
        let env_ref = self.make_module_env(id, state_ref)?;
        let exports_ref = self.exec_module_chunk(id, &source, env_ref, state_ref)?;

        self.modules.insert(id, ModuleEntry {
            id,
            name: self.name_for(id).to_owned(),
            path: self.path_for(id).to_owned(),
            exports_ref,
            state_ref,
            env_ref,
            dependencies: Vec::new(),
            reverse_deps: Vec::new(),
            source_hash: fxhash::hash64(source.as_bytes()),
            version: 1,
            last_error: None,
        });

        Ok(exports_ref)
    }
}
```

### Script-side behavior

A module should behave like a normal Luau module:

```lua
local M = {}

function M.render(state)
    ui.label("tools")
end

return M
```

No filesystem, no package loader, no implicit globals shared across modules.

---

## Critical touch point: hot reload strategy

### The safe MVP algorithm

1. filesystem watcher marks path dirty
2. main thread drains dirty paths at next frame boundary
3. find changed module ids
4. compute reverse dependency closure
5. topologically sort affected modules
6. for each module:
   - keep old `state_ref`
   - build a fresh env
   - compile + load new chunk
   - run module chunk
   - call optional `reload(old_exports, state)`
7. if all affected modules succeed, swap them in
8. if any module fails:
   - keep old modules live
   - store the error
   - show error overlay

### Why this works

It guarantees:

- no mid-frame code swaps
- last-known-good behavior on compile failure
- stable persistent data
- deterministic reload order

### Host-side sketch

```rust
impl UiRuntime {
    fn apply_pending_reload(&mut self) {
        let dirty = self.reloader.drain_dirty_paths();
        if dirty.is_empty() {
            return;
        }

        match self.vm.reload_subgraph(&dirty) {
            Ok(()) => {
                self.error_overlay = None;
                self.egui_ctx.request_repaint();
            }
            Err(err) => {
                self.error_overlay = Some(err);
                self.egui_ctx.request_repaint();
            }
        }
    }
}
```

---

## Critical touch point: state preservation

Hot reload works only if code and state are separate.

### Rule

Persist only plain data in `state`:

- numbers
- booleans
- strings
- arrays / tables of plain data
- opaque handles if necessary

Do not persist:

- `Ui` objects
- widget responses
- host closures
- engine object references with unstable lifetimes
- environment tables
- module export tables from previous versions

### State migration pattern

```lua
function M.reload(old, state)
    if state.old_name ~= nil and state.name == nil then
        state.name = state.old_name
        state.old_name = nil
    end
end
```

---

## Critical touch point: error handling

You need **two** error classes:

1. **reload-time errors**
   - compile error
   - load error
   - exception while executing module body
   - exception during `reload`

2. **frame-time render errors**
   - exception during `render`
   - invalid nested container usage
   - bad widget arguments

### Recommended behavior

- reload-time error:
  - keep old version alive
  - show overlay with compile/runtime trace
- render-time error:
  - abort only the current script call
  - keep the runtime alive
  - show overlay
  - do not tear down the whole UI system

### Minimal fallback overlay

Always keep a host-native error panel that does not depend on Luau bindings being healthy.

---

## Critical touch point: script time budget / runaway protection

Even for trusted local UI scripts, install an interrupt budget.

The UI runtime should not freeze the whole engine because of:

- accidental infinite loops
- large table construction every frame
- bad recursion
- pathological reload hooks

### MVP recommendation

- install a Luau interrupt/watchdog
- budget UI render scripts tightly
- abort and report if exceeded

A practical starting point:

- 2-5 ms per script frame for tool UI in a game frame
- optionally higher in editor-only builds

---

## Critical touch point: repaint scheduling

There are two repaint sources:

1. egui-driven repaint requests from widget behavior and animation
2. external host-driven repaint requests from file changes or engine events

### The runtime should support both

#### egui-driven

Register a wakeup callback once:

```rust
let wake = engine_wakeup.clone();
egui_ctx.set_request_repaint_callback(move |_info| {
    wake.wake();
});
```

#### host-driven

When a file watcher detects a change:

- enqueue dirty path
- wake main loop
- apply reload on next frame
- call `request_repaint`

---

## Recommended file layout

```text
engine/
  ui/
    ui_runtime.rs
    frame_scope.rs
    egui_renderer.rs
    input_bridge.rs

  script/
    script_vm.rs
    module_cache.rs
    hot_reload.rs
    require.rs
    error.rs

  ffi/
    luau_host.h
    luau_host.cpp
    build.rs

scripts/
  ui/
    main.luau
    inspector.luau
    console.luau
```

---

## Step-by-step implementation order

### Phase 1: hardcoded egui

- initialize `egui::Context`
- gather `RawInput`
- handle `FullOutput`
- tessellate and paint
- expose input capture flags

### Phase 2: boot Luau

- create VM
- compile + load one root script
- call a `render(state)` export each frame
- add error overlay

### Phase 3: add minimal bindings

Implement only:

- `ui.window`
- `ui.label`
- `ui.button`
- `ui.text_edit`
- `ui.separator`
- `ui.horizontal`
- `app.log`
- `app.request_reload`
- `app.request_repaint`

### Phase 4: custom require

- add module resolution
- add module cache
- add dependency graph
- add per-module state tables

### Phase 5: hot reload

- add filesystem watcher
- reload only at frame boundaries
- keep last-good version alive
- show errors in overlay

### Phase 6: stateful widgets and ids

- require explicit ids
- stabilize text edits / windows / collapsers across reloads

### Phase 7: time budget and polish

- add interrupt budget
- collect reload metrics
- add tracebacks
- add better host logging

---

## What not to do yet

Avoid these until the MVP is stable:

- exposing egui `Response` objects to Luau
- full engine reflection
- automatic userdata wrappers for all engine types
- multi-threaded script execution
- live patching code in the middle of a frame
- diffing UI trees between frames
- caching widget trees
- secondary egui viewports
- generalized property editors

---

## The one-sentence design rule

For the MVP, **keep Luau and egui on the same UI thread, expose a tiny typed widget API, preserve only plain state across reloads, and swap code only at frame boundaries**.

---

## Reference snippets

### Minimal host entry point

```rust
fn tick_ui(runtime: &mut UiRuntime, engine: &mut Engine) {
    let raw_input = engine.build_egui_input(); // convert to points
    let output = runtime.frame(raw_input);

    engine.apply_platform_output(output.platform_output);
    engine.upload_textures(output.textures_delta.set);
    engine.paint(output.primitives);
    engine.free_textures(output.textures_delta.free);

    engine.set_ui_input_capture(output.wants_pointer, output.wants_keyboard);
}
```

### Minimal Luau root

```lua
local M = {}

function M.init(state)
    state.value = state.value or 0.5
end

function M.render(state)
    ui.window({ id = "audio", title = "Audio" }, function()
        local changed, value = ui.slider_float("gain", "Gain", state.value, 0.0, 1.0)
        if changed then
            state.value = value
        end

        if ui.button("Reload UI") then
            app.request_reload()
        end
    end)
end

return M
```

---

## External references

These are the official references this design assumes:

- Luau repository README:
  - compile source separately, then `luau_load`
  - `safeenv` / sandbox guidance
- Luau sandbox documentation:
  - readonly builtins, per-script globals, interrupt model
- egui `Context`, `RawInput`, and `FullOutput` docs:
  - `ctx.run(raw_input, ...)`
  - `textures_delta`
  - `platform_output`
  - `tessellate`
  - repaint callback
  - input capture queries

Links:

- https://github.com/luau-lang/luau
- https://luau.org/sandbox/
- https://docs.rs/egui/latest/egui/struct.Context.html
- https://docs.rs/egui/latest/egui/struct.RawInput.html
- https://docs.rs/egui/latest/egui/struct.FullOutput.html
