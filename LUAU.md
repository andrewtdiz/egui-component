# Prompt For `egui-component` Maintainer

You are working in the `egui-component` codebase inside Clay Engine. Your job is to implement the `egui-component` side of Clay's editor rearchitecture so the editor can move volatile product UX into Luau without exposing raw immediate-mode widget calls across the scripting boundary.

This is not a request for a generic plugin system and not a request to mirror all of `egui-component` into scripts. The goal is a narrow, stable, declarative contract that the Clay editor host can render through `egui-component`.

## Context

Clay Engine is moving toward this split:

- Rust editor core owns authoritative state, transactions, undo or redo, rendering internals, preview machinery, and invariants.
- Luau owns fast-moving editor experience such as workflows, inspectors, onboarding, empty states, commands, copy, and product iteration.
- `egui-component` remains the typed UI substrate, but it needs to expose a host-owned declarative API surface that can be driven by Luau.

The key constraint is:

- do not expose raw `egui` or raw `egui-component` immediate-mode widget APIs directly to Luau

Instead, implement a declarative contract that:

- is serializable
- is introspectable
- is narrow and curated
- maps onto `egui-component`
- exposes semantic events
- is suitable for generated Luau typings and builders

## Outcome

At the end of this work, `egui-component` must provide all of the following:

1. A stable declarative component contract for editor surfaces.
2. A registry or schema export describing supported component families, props, variants, and events.
3. A host renderer that can take a declarative tree and render it through `egui-component`.
4. A semantic event model produced by rendered contract nodes.
5. Generated metadata suitable for Clay to turn into Luau typings and helper builders.
6. Documentation and showcase coverage for the supported contract surface.

## Non-Goals

Do not do these:

- Do not expose one Rust function per widget call for scripts to invoke every frame.
- Do not expose the full internal `egui-component` API surface.
- Do not build a third-party extension marketplace.
- Do not own authoritative editor state in `egui-component`.
- Do not force Clay editor UX to understand internal `egui` concepts.

## Architecture Direction

Implement a new declarative contract layer inside `egui-component`.

The shape should be:

- contract model types
- contract registry and schema export
- contract renderer
- contract event model
- generated metadata output

This contract layer should sit above the existing typed component layer, not replace it.

Existing component builders remain the source of truth for drawing behavior. The new layer should translate declarative nodes into those typed builders.

## Initial Supported Contract Surface

Do not start with every component. Start with the curated set needed for Clay editor product UX:

- layout containers
- card and surface containers
- sidebar
- toolbar
- menu bar
- tabs
- label and text presentation
- button
- button group
- input
- number input
- checkbox
- switch
- select or combobox
- field
- separator
- collapsible
- dialogue or modal
- hierarchy tree
- spinner
- progress
- toast or banner if the current codebase supports it cleanly

If a component is too specialized, geometry-heavy, or not yet stable enough for a declarative scripting contract, exclude it from the first contract release.

## Required Deliverables

### 1. Contract Model

Add a new contract module in `egui-component` that defines:

- a node tree model
- component family ids
- typed property payloads
- variant enums where relevant
- stable node ids
- layout and sizing primitives
- optional visibility and enabled state
- semantic event handler ids or action ids

The contract must be serializable with `serde`.

Strong recommendation:

- use tagged enums and explicit structs rather than untyped property maps

The goal is introspectable stability, not runtime flexibility at any cost.

### 2. Contract Registry And Schema Export

Implement a registry describing every supported declarative component family.

For each family, publish:

- family id
- display name
- supported props
- prop types
- supported variants
- supported semantic events
- documentation summary

Expose this as:

- Rust registry APIs
- a machine-readable export format such as JSON

The schema export should be generated from Rust source of truth, not hand-maintained in a separate file.

### 3. Host Renderer

Implement a renderer entry point that:

- accepts a declarative tree
- renders it through `egui-component`
- emits semantic events
- preserves stable node ids in responses

Keep the renderer host-friendly:

- one tree in
- one frame of semantic events out

Do not require the host to know internal component builder details.

The renderer should be coarse-grained and should not require per-widget imperative scripting calls.

### 4. Semantic Event Model

Define a normalized event model for the declarative contract.

Support events like:

- clicked
- changed
- submitted
- selected
- toggled
- confirmed
- cancelled
- opened
- closed
- command_invoked

Each event payload should include:

- node id
- event kind
- optional value payload
- optional metadata that is safe and stable

Do not leak raw `egui` event internals unless there is a strong reason.

### 5. Luau Generation Support

Provide generated metadata that Clay can use to generate Luau bindings.

Minimum acceptable output:

- machine-readable schema export covering nodes, props, variants, and events

Preferred output:

- a small codegen helper or stable schema export step that makes Luau type generation deterministic

The maintainer does not need to author all Clay-side Luau files, but `egui-component` must provide enough structured metadata for that generation to be safe.

### 6. Docs And Showcase

Update the docs and showcase so maintainers can inspect and verify the declarative contract.

Add:

- a contract architecture doc
- supported family documentation
- one showcase or demo app driven through the declarative contract layer
- guidance on when to add a component to the contract and when not to

The showcase must prove that the declarative layer is not theoretical.

## File And Module Expectations

You should introduce a clear public surface for the contract layer. The exact file structure is up to you, but the implementation will likely need changes in areas like:

- `src/lib.rs`
- `src/catalog.rs`
- `src/components/api.rs`
- `src/layout.rs`
- `src/theme.rs`
- new `src/contract/*.rs` modules
- `docs/llm/architecture-overview.md`
- `docs/llm/components-reference.md`
- `src/example_apps/showcase.rs` or equivalent showcase surface

If the current catalog shape is not sufficient for schema export, refactor the catalog so component metadata is authored once and reused for:

- live showcase
- docs metadata
- declarative contract schema export

## Implementation Rules

- Keep the contract curated and narrow.
- Favor semantic props over low-level painting knobs.
- Favor additive evolution over breaking churn.
- Avoid introducing engine-specific product terminology into `egui-component`.
- Keep rendering logic in existing typed component builders where possible.
- Keep the contract source of truth in Rust and generate outward-facing metadata from it.

## Acceptance Criteria

This work is complete only when all of the following are true:

1. There is a public declarative contract API in `egui-component`.
2. That contract is serializable and documented.
3. A host can render a non-trivial surface entirely from the contract layer.
4. The host receives semantic events from that rendered surface.
5. There is a machine-readable schema export suitable for Luau type generation.
6. The supported component families are documented and visible in a showcase or demo.
7. The implementation does not expose raw immediate-mode widget calls as the scripting boundary.

## Suggested Execution Plan

### Phase 1

Build the contract types, registry, and schema export shape.

Exit condition:

- a serialized tree can be validated and introspected

### Phase 2

Build the contract renderer and semantic event collector for the initial curated family set.

Exit condition:

- a declarative demo renders and emits semantic events

### Phase 3

Connect registry metadata to docs and showcase so the contract becomes maintainable.

Exit condition:

- supported families are documented from the same source of truth

### Phase 4

Harden naming, ergonomics, serialization, and compatibility expectations for external host use.

Exit condition:

- the API is stable enough for Clay editor integration to begin

## Quality Bar

This is infrastructure for a scripting boundary. Optimize for:

- stability
- clarity
- introspection
- narrowness
- maintainability

Do not optimize for:

- maximum widget parity
- dynamic flexibility at the cost of typing
- direct scripting convenience if it leaks immediate-mode internals

## Deliverable Format

When you finish, provide:

1. A short architecture summary.
2. The list of files changed.
3. The supported declarative component families.
4. The public APIs added.
5. The schema export format.
6. The showcase or demo entry point.
7. Any follow-up work Clay must implement on the host side.

## Important Constraint

If the current `egui-component` architecture makes this hard, do the smallest refactor needed to establish a maintainable contract source of truth. Do not hide the problem by shipping a hand-wired one-off bridge. The output must be a reusable, typed contract surface that can carry Clay's editor UX for a long time.