# Runtime Architecture TASK List

Scope:
- First structurally solid and reliable retained-renderer architecture for the embedded React + `deno_core` + Rust host stack.
- JS owns the ephemeral reconciler tree and emits declarative mutation batches.
- Rust owns the retained `HostTree`, validates committed batches, and drives rendering.

Reference architecture notes captured from DeepWiki:
- `reference/deepwiki/react-three-fiber-runtime-architecture.md`
- `reference/deepwiki/react-three-fiber-solid-v1-runtime-architecture.md`
