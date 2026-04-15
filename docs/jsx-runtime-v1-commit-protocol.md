# JSX Runtime v1 Commit Protocol

This document freezes the session-level commit protocol shared by:

- `crates/clay-jsx-runtime/src/runtime.rs`
- `crates/clay-jsx-egui-bridge/src/runtime_api.ts`
- `crates/clay-jsx-egui-bridge/src/runtime.rs`
- `crates/clay-jsx-egui-bridge/src/host_tree.rs`

It defines the minimum supported v1 commit lifecycle for the retained renderer.

The separate wake / timer / invalidation contract is frozen in
`docs/jsx-runtime-v1-scheduling-model.md`.

## Why this exists

The runtime already behaves like a protocol: JS enqueues declarative commit
batches, Rust drains them, decodes them, validates them, applies them to a
staged retained tree, acknowledges successful batch ids, and enters controlled
session failure if anything breaks.

v1 treats that as one explicit session-level contract.

## Core invariants

1. JS owns only shadow reconciler state.
2. Rust owns the retained `HostTree`.
3. Commit batches are the only JS -> Rust retained-state mutation boundary.
4. A drained runtime update is **session-atomic**:
   - Rust stages all queued batches against a cloned `HostTree`.
   - Rust acknowledges batch ids only after the staged tree decodes, validates,
     applies, materializes, and passes post-apply validation.
   - Rust swaps the live `HostTree` and applies retained-tree metrics only
     after acknowledgement succeeds.
5. Any protocol violation leaves the last committed tree intact and moves the
   session into controlled failure until reload.

## Protocol states

### JS queue state (`runtime_api.ts`)

- `ready`
  - JS may enqueue commit batches.
  - JS tracks `pendingCommitBatchIds` in FIFO order.
- `failed`
  - JS must reject further render/dispatch/commit work.
  - Recovery requires session reload / remount.

### Rust session state (`runtime.rs`)

- `ready`
- `decoding { commit_batch_id }`
- `validating_envelope { commit_batch_id }`
- `applying { commit_batch_id }`
- `validating_committed_tree { last_applied_commit_batch_id }`
- `acknowledging { commit_batch_ids }`
- `failed { rejected_commit_batch_id, reason }`

These are explicit protocol phases, not renderer-visible tree states.

## Legal transitions

### Healthy path

1. **enqueue**
   - JS lowers committed shadow mutations into a `HostMutationBatch`.
   - `enqueueCommitBatch()` submits the batch through the active transport.
   - JS appends the returned batch id to `pendingCommitBatchIds`.
2. **decode**
   - Rust drains queued `RuntimeCommitBatch` values.
   - The session enters `decoding { commit_batch_id }`.
3. **validate envelope**
   - Rust decodes `HostMutationBatch`.
   - Rust verifies model version and schema fingerprint.
   - The session enters `validating_envelope { commit_batch_id }`.
4. **apply to staged tree**
   - Rust clones the committed `HostTree`.
   - Rust applies each batch to that staged clone.
   - `HostTree::apply_mutations_in_place()` validates retained-tree topology.
   - The session enters `applying { commit_batch_id }`.
5. **post-apply validation**
   - Rust materializes a `ContractTree` when contract-visible structure changed.
   - Rust validates the materialized tree against the checked-in contract.
   - The session enters
     `validating_committed_tree { last_applied_commit_batch_id }`.
6. **acknowledge**
   - Rust acknowledges the exact FIFO list of applied batch ids back into JS.
   - JS removes those ids from `pendingCommitBatchIds`.
   - The session enters `acknowledging { commit_batch_ids }`.
7. **commit**
   - Only after acknowledgement succeeds does Rust replace the live `HostTree`
     and apply retained-tree metrics deltas.
   - The session returns to `ready`.

### Failure path

Any failure during decode, version/schema validation, retained-tree mutation,
post-apply validation, or acknowledgement breaks the protocol.

- **reject**
  - Used when Rust can attribute the failure to a specific batch id.
  - JS clears pending ids and enters `failed`.
  - Rust enters `failed { rejected_commit_batch_id, reason }`.
- **fail**
  - Used when the protocol breaks without a specific rejected batch id, such as
    acknowledgement failure.
  - JS clears pending ids and enters `failed`.
  - Rust enters `failed { rejected_commit_batch_id: null, reason }`.
  - Rust aborts host runtime execution for the failed session without advancing
    the exposed `RuntimeDebugMetrics.shutdown_count` metric.

Once failed, the session must not resume normal work. Reload is the only
recovery boundary in v1.

## Atomicity guarantees

For every drained runtime update in v1:

- the previously committed `HostTree` remains byte-for-byte intact if any batch
  in the drained set fails
- retained-tree mutation metrics do not advance for rejected work
- only commit-protocol failure counters advance on rejected work
- exposed nested runtime shutdown metrics remain unchanged on rejected work
- JS never observes acknowledged ids for work Rust did not fully validate

## Checked-in test coverage

The bridge runtime freezes this contract with targeted tests covering:

- decode failure
- version mismatch
- schema mismatch
- invalid retained-tree mutations
- post-apply validation failure
- rollback / unchanged committed tree after rejection
- unchanged retained metrics except commit-protocol failure counters

Those tests live in `crates/clay-jsx-egui-bridge/src/runtime.rs`.
