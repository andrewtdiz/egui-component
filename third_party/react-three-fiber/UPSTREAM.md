# react-three-fiber Reference Snapshot

This directory is a sparse, reference-only snapshot of
https://github.com/pmndrs/react-three-fiber.

- Upstream commit: `582f7871bd5f2107bd62bd83ef5a70f8364d7979`
- Upstream default branch at snapshot time: `master`
- License: MIT, see `LICENSE`

Copied files:

- `LICENSE`
- `package.json`
- `packages/fiber/src/core/reconciler.tsx`
- `packages/fiber/src/core/utils.tsx`
- `packages/fiber/src/core/renderer.tsx`

The egui JSX runtime uses this as an architecture reference for retained host
instances, mutation commits, prop diffing, root bootstrapping, and lifecycle
cleanup. The files are not build inputs for the Rust crate.
