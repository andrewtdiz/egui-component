#![doc = r#"
egui-component contract bridge for `clay-jsx-runtime` TS/TSX hosts.

## Ownership contract

This crate makes the retained-renderer boundary explicit:

- JS in `runtime_api.ts` owns only an ephemeral reconciler shadow tree,
  handler registrations, and lowering state.
- JS does **not** create, hold, or mutate native egui renderer objects.
- JS may propose retained-tree changes only by committing `HostMutationBatch`
  payloads through the bridge commit transport abstraction.
- The default backend remains JSON, while typed transport evaluation stays
  behind the same `enqueueCommitBatch()` protocol surface.
- Rust owns the retained `HostTree`, validates every committed batch, applies it
  transactionally, materializes `ContractTree` snapshots, and exposes retained
  motion state.
- Commit batches are acknowledged only after Rust validation succeeds. A failed
  batch enters controlled session failure instead of partially mutating the
  retained tree.

Treat the JS side as a declarative mutation producer and the Rust side as the
sole owner of retained UI state.

The frozen minimal v1 host-config contract is checked in at
`docs/jsx-runtime-v1-host-config-contract.md`.

The frozen minimal v1 commit protocol is checked in at
`docs/jsx-runtime-v1-commit-protocol.md`.

The frozen minimal v1 scheduling model is checked in at
`docs/jsx-runtime-v1-scheduling-model.md`.

The frozen v1 deferred-features list is checked in at
`docs/jsx-runtime-v1-deferred-features.md`.

The frozen v1 host-config surface is intentionally narrow: mutation mode only,
microtasks enabled, hydration/persistence disabled, JS public instances limited
to shadow nodes, no direct native host refs, and host-config verbs limited to
create/append/insert/remove/reorder/text/hide/unhide plus scheduling hooks.

The checked-in commit protocol freezes the session-atomic states and legal
transitions for enqueue, decode, validate, acknowledge, reject, and fail.

The checked-in scheduling model freezes how `requestRepaint`, timers, RAF, and
effect-driven async work converge through `pending_host_wake`, plus the rule
that one bridge drain is a bounded scheduling step rather than an unbounded
flush.

The checked-in deferred-features list freezes the non-v1 surfaces that must not
silently expand the architecture contract: hydration, persistence, direct
native host refs/public native instances, hook-state restoration, and
production transport replacement away from JSON.

## Motion / contract-tree separation

Retained motion is part of the v1 bridge contract, but it is not treated as a
structural contract-tree update:

- motion-only commit batches may update retained `MotionFrame` state without
  forcing `ContractTree` rematerialization
- `tick_motion()` returns `tree: None` for motion-only frame advancement
- structural commits must preserve retained motion values for unaffected nodes
  until those nodes are replaced or removed

## Hot reload / state restoration policy

V1 reload is a validated cold remount, not stateful HMR:

- replacement entrypoints validate in an isolated runtime before the live
  session is touched
- failed replacement loads leave the last good session live and interactive
- successful reload tears down and unmounts the previous session
  deterministically, then installs a fresh cold remount
- `HotReloadState` exists for API compatibility only; hook-state restoration is
  explicitly deferred work and not a v1 guarantee

## Error containment / host-visible recovery

The bridge normalizes session-affecting failures into one host-visible recovery
contract:

- recoverable reconciler issues remain `continue`
- error-boundary-contained failures surface `bounded_failure`
- fatal runtime/reconciler failures require `teardown`
- commit-protocol failures require `reload_required`

## Architecture integration / benchmark gates

The bridge keeps one checked-in end-to-end architecture gate for the v1 session
flow: load, initial render, sync dispatch, async drain, motion tick, reload,
and teardown. Representative-tree benchmark artifacts in
`reference/benchmarks/` freeze the hot-path proof surface for commit
throughput, event-dispatch latency, and callback-drain latency.

## Event routing boundary

The JS bridge also owns the stable event-registration boundary for the retained
tree:

- materialized descriptors install handler registrations into keyed routing
  tables owned by `runtime_api.ts`
- one incoming event may route through `node_id` and optional `action_id`
  candidates, including wildcard handlers
- duplicate route hits for the same logical registration are de-duplicated by
  registration identity so one authored handler runs at most once per event
- subtree clear, replace, hide, unmount, and reload paths must remove stale
  registrations before new retained descriptors become observable

Treat handler registration and event dispatch routing as a first-class retained
boundary alongside commit transport and retained-tree mutation validation.
"#]

pub mod host_tree;
pub mod motion;
mod runtime;

pub const EGUI_MODULE_SOURCE: &str =
    include_str!(concat!(env!("OUT_DIR"), "/virtual_modules/mod.js"));
pub const EGUI_RUNTIME_SOURCE: &str =
    include_str!(concat!(env!("OUT_DIR"), "/virtual_modules/runtime_api.js"));
pub const EGUI_JSX_RUNTIME_SOURCE: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/virtual_modules/jsx_runtime_api.js"
));
pub const EGUI_LOWERING_SOURCE: &str =
    include_str!(concat!(env!("OUT_DIR"), "/virtual_modules/lowering_api.js"));
pub const EGUI_MOTION_REACT_SOURCE: &str =
    include_str!(concat!(env!("OUT_DIR"), "/virtual_modules/motion_api.js"));

pub use clay_jsx_runtime::{
    extend_logs, push_log, RuntimeLogBuffer, RuntimeRecoveryCategory, RuntimeRecoveryDisposition,
    RuntimeRecoveryState, LOG_HISTORY_LIMIT, LOG_MESSAGE_LIMIT_BYTES,
};
pub use host_tree::{HostMutation, HostMutationBatch, HostTree};
pub use motion::{
    MotionEase, MotionFrame, MotionProperty, MotionSpec, MotionTickResult, MotionTransition,
    MotionValues,
};
pub use runtime::{
    CommitProtocolState, HotReloadState, JsxRuntimeDebugMetrics, JsxRuntimeHostWakeCallback,
    JsxRuntimeLoadFailure, JsxRuntimeLoadOutcome, JsxRuntimeSession, JsxRuntimeSessionWorker,
    JsxRuntimeWorkerDebugSnapshot, JsxRuntimeWorkerEvent, JsxRuntimeWorkerReloadOutcome,
    RenderedJsx,
};

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::{Path, PathBuf},
        time::{Duration, Instant},
    };

    use clay_jsx_runtime::contract::{
        ContractEvent, ContractLength, ContractNode, EventKind, EventValue, NodeId,
    };
    use clay_jsx_runtime::HOST_CALLBACK_DRAIN_LIMIT;
    use tempfile::tempdir;

    use super::{
        HotReloadState, JsxRuntimeLoadOutcome, JsxRuntimeSession, JsxRuntimeSessionWorker,
        JsxRuntimeWorkerDebugSnapshot, JsxRuntimeWorkerEvent, JsxRuntimeWorkerReloadOutcome,
        MotionFrame, MotionProperty, RenderedJsx, RuntimeLogBuffer, RuntimeRecoveryCategory,
        RuntimeRecoveryDisposition,
    };

    const LIFECYCLE_ENDURANCE_CYCLES: usize = 128;

    #[test]
    fn tsx_entrypoint_transpiles_and_dispatches_hooks() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { eventValue, render, useState } from "egui";

type AppProps = { initial: boolean };

function App(props: AppProps) {
  const [checked, setChecked] = useState(props.initial);
  return (
    <div id="root" data-slot="column">
      <input
        type="checkbox"
        id="toggle"
        checked={checked}
        onToggle={(event) => setChecked(Boolean(eventValue(event)))}
      />
    </div>
  );
}

render(<App initial={true} />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(true));

        let event = ContractEvent::new("toggle", EventKind::Toggled)
            .value(Some(EventValue::Boolean(false)));
        let rendered = session
            .dispatch_events(&[event])
            .expect("event dispatch should update hook state");
        let tree = rendered.tree.expect("changed render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(false));
    }

    #[test]
    fn event_with_node_and_action_id_invokes_click_handler_once() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [clickCount, setClickCount] = useState(0);
  return (
    <div id="root" data-slot="column">
      <button
        id="trigger"
        actionId="action.trigger"
        label="Trigger"
        onClick={() => setClickCount((count) => count + 1)}
      />
      <label id="click-count" text={String(clickCount)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "click-count").expect("counter label should exist")),
            Some("0")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("trigger", EventKind::Clicked)
                .action(Some("action.trigger".into()))])
            .expect("event dispatch should update state exactly once");
        let tree = rendered.tree.expect("event render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "click-count").expect("counter label should exist")),
            Some("1")
        );
    }

    #[test]
    fn event_with_matching_node_id_routes_without_action_id() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [clickCount, setClickCount] = useState(0);
  return (
    <div id="root" data-slot="column">
      <button
        id="node-route"
        label="Node Route"
        onClick={() => setClickCount((count) => count + 1)}
      />
      <label id="click-count" text={String(clickCount)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "click-count").expect("counter label should exist")),
            Some("0")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("node-route", EventKind::Clicked)])
            .expect("node-id event dispatch should update state exactly once");
        let tree = rendered.tree.expect("event render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "click-count").expect("counter label should exist")),
            Some("1")
        );
    }

    #[test]
    fn event_with_matching_action_id_routes_without_matching_node_id() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [clickCount, setClickCount] = useState(0);
  return (
    <div id="root" data-slot="column">
      <button
        id="physical-node"
        actionId="action.route"
        label="Action Route"
        onClick={() => setClickCount((count) => count + 1)}
      />
      <label id="click-count" text={String(clickCount)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "click-count").expect("counter label should exist")),
            Some("0")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("unmatched-node", EventKind::Clicked)
                .action(Some("action.route".into()))])
            .expect("action-id event dispatch should update state exactly once");
        let tree = rendered.tree.expect("event render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "click-count").expect("counter label should exist")),
            Some("1")
        );
    }

    #[test]
    fn event_with_node_and_action_id_invokes_wildcard_handler_once() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [eventCount, setEventCount] = useState(0);
  return (
    <div id="root" data-slot="column">
      <button
        id="trigger"
        actionId="action.trigger"
        label="Trigger"
        onEvent={() => setEventCount((count) => count + 1)}
      />
      <label id="event-count" text={String(eventCount)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "event-count").expect("counter label should exist")),
            Some("0")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("trigger", EventKind::Clicked)
                .action(Some("action.trigger".into()))])
            .expect("event dispatch should update state exactly once");
        let tree = rendered.tree.expect("event render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "event-count").expect("counter label should exist")),
            Some("1")
        );
    }

    #[test]
    fn wildcard_event_handler_routes_without_action_id() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [eventCount, setEventCount] = useState(0);
  return (
    <div id="root" data-slot="column">
      <button
        id="wildcard-route"
        label="Wildcard Route"
        onEvent={() => setEventCount((count) => count + 1)}
      />
      <label id="event-count" text={String(eventCount)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "event-count").expect("counter label should exist")),
            Some("0")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("wildcard-route", EventKind::Toggled)])
            .expect("wildcard event dispatch should update state exactly once");
        let tree = rendered.tree.expect("event render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "event-count").expect("counter label should exist")),
            Some("1")
        );
    }

    #[test]
    fn reordered_children_preserve_handler_behavior_without_double_invocation() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [reversed, setReversed] = useState(false);
  const [alphaCount, setAlphaCount] = useState(0);
  const [betaCount, setBetaCount] = useState(0);
  const alpha = (
    <button
      key="alpha"
      id="alpha"
      actionId="action.alpha"
      label="Alpha"
      onClick={() => setAlphaCount((count) => count + 1)}
    />
  );
  const beta = (
    <button
      key="beta"
      id="beta"
      actionId="action.beta"
      label="Beta"
      onClick={() => setBetaCount((count) => count + 1)}
    />
  );
  const orderedChildren = reversed ? [beta, alpha] : [alpha, beta];

  return (
    <div id="root" data-slot="column">
      <button
        id="reorder"
        label="Reorder"
        onClick={() => setReversed((value) => !value)}
      />
      {orderedChildren}
      <label id="alpha-count" text={String(alphaCount)} />
      <label id="beta-count" text={String(betaCount)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");

        let rendered = session
            .dispatch_events(&[ContractEvent::new("reorder", EventKind::Clicked)])
            .expect("reorder dispatch should rerender the tree");
        let tree = rendered.tree.expect("reorder should materialize a tree");
        let root_children = contract_children(&tree.root);
        assert_eq!(root_children[1].node_id().as_str(), "beta");
        assert_eq!(root_children[2].node_id().as_str(), "alpha");

        let rendered = session
            .dispatch_events(&[
                ContractEvent::new("alpha", EventKind::Clicked).action(Some("action.alpha".into())),
                ContractEvent::new("beta", EventKind::Clicked).action(Some("action.beta".into())),
            ])
            .expect("reordered handlers should still dispatch exactly once each");
        let tree = rendered.tree.expect("event render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "alpha-count").expect("alpha counter should exist")),
            Some("1")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "beta-count").expect("beta counter should exist")),
            Some("1")
        );
    }

    #[test]
    fn remounted_handlers_do_not_double_invoke_after_visibility_toggle() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [visible, setVisible] = useState(true);
  const [count, setCount] = useState(0);
  return (
    <div id="root" data-slot="column">
      <button
        id="toggle-visibility"
        label="Toggle Visibility"
        onClick={() => setVisible((value) => !value)}
      />
      {visible ? (
        <button
          id="remount-target"
          actionId="action.remount"
          label="Remount Target"
          onClick={() => setCount((value) => value + 1)}
        />
      ) : null}
      <label id="count" text={String(count)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle-visibility", EventKind::Clicked)])
            .expect("hide dispatch should succeed");
        let tree = rendered.tree.expect("hide should materialize a tree");
        assert!(
            find_node(&tree.root, "remount-target").is_none(),
            "remount target should be absent while hidden"
        );

        session
            .dispatch_events(&[ContractEvent::new("remount-target", EventKind::Clicked)
                .action(Some("action.remount".into()))])
            .expect("dispatching against a hidden route should be a no-op");

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle-visibility", EventKind::Clicked)])
            .expect("show dispatch should succeed");
        let tree = rendered.tree.expect("show should materialize a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "count").expect("count label should exist")),
            Some("0"),
            "hidden dispatches must not keep stale handler registrations alive"
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("remount-target", EventKind::Clicked)
                .action(Some("action.remount".into()))])
            .expect("remounted handler should dispatch exactly once");
        let tree = rendered.tree.expect("event render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "count").expect("count label should exist")),
            Some("1")
        );
    }

    #[test]
    fn reload_replaces_handler_routes_without_stale_dispatch() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [count, setCount] = useState(0);
  return (
    <div id="root" data-slot="column">
      <button
        id="reload-target"
        actionId="action.reload"
        label="Reload Target"
        onClick={() => setCount((value) => value + 1)}
      />
      <label id="count" text={String(count)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("initial reload fixture should be written");

        let mut worker =
            JsxRuntimeSessionWorker::spawn(None).expect("worker should spawn successfully");
        worker
            .request_reload(&entry_path)
            .expect("initial worker reload should queue");
        let initial_rendered = wait_for_loaded_reload(&worker);
        let tree = initial_rendered
            .tree
            .expect("initial reload should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "count").expect("count label should exist")),
            Some("0")
        );

        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [count, setCount] = useState(0);
  return (
    <div id="root" data-slot="column">
      <button
        id="reload-target"
        actionId="action.reload"
        label="Reload Target"
        onClick={() => setCount((value) => value + 10)}
      />
      <label id="count" text={String(count)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("updated reload fixture should be written");

        worker
            .request_reload(&entry_path)
            .expect("second worker reload should queue");
        let reloaded = wait_for_loaded_reload(&worker);
        let tree = reloaded.tree.expect("reloaded tree should exist");
        assert_eq!(
            label_text(find_node(&tree.root, "count").expect("count label should exist")),
            Some("0")
        );

        worker
            .request_dispatch_events(vec![ContractEvent::new(
                "reload-target",
                EventKind::Clicked,
            )
            .action(Some("action.reload".into()))])
            .expect("reload dispatch should queue");
        let rendered = wait_for_dispatch_result(&worker);
        let tree = rendered
            .tree
            .expect("dispatch after reload should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "count").expect("count label should exist")),
            Some("10"),
            "reloaded handler should replace the old route instead of double-invoking"
        );

        worker.shutdown().expect("worker should shut down cleanly");
    }

    #[test]
    fn reload_runs_effect_cleanup_before_replacing_the_previous_session() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useEffect } from "egui";

function App() {
  useEffect(() => {
    const handle = setInterval(() => {}, 100);
    return () => clearInterval(handle);
  }, []);

  return (
    <div id="root" data-slot="column">
      <label id="version" text="old" />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("initial reload fixture should be written");

        let mut worker =
            JsxRuntimeSessionWorker::spawn(None).expect("worker should spawn successfully");
        worker
            .request_reload(&entry_path)
            .expect("initial worker reload should queue");
        let initial_rendered = wait_for_loaded_reload(&worker);
        let tree = initial_rendered
            .tree
            .expect("initial reload should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "version").expect("version label should exist")),
            Some("old")
        );

        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return (
    <div id="root" data-slot="column">
      <label id="version" text="new" />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("updated reload fixture should be written");

        worker
            .request_reload(&entry_path)
            .expect("replacement reload should queue");
        let (outcome, snapshot) = wait_for_reload_completion(&worker);
        let reloaded = match outcome {
            JsxRuntimeWorkerReloadOutcome::Loaded { rendered, .. } => rendered,
            JsxRuntimeWorkerReloadOutcome::Failed(failure) => {
                panic!("worker reload should succeed: {:#}", failure.error)
            }
        };
        let tree = reloaded.tree.expect("reloaded tree should exist");
        assert_eq!(
            label_text(find_node(&tree.root, "version").expect("version label should exist")),
            Some("new")
        );

        let torn_down = snapshot
            .last_torn_down_session
            .expect("successful reload should tear down the previous session");
        assert_eq!(torn_down.teardown_count, 1);
        assert_eq!(torn_down.unmount_count, 1);
        assert_eq!(
            torn_down.runtime.active_timer_count, 0,
            "effect cleanup should clear timers before the old session is dropped"
        );
        assert!(
            !torn_down.runtime.pending_host_wake,
            "effect cleanup should leave no pending wake on the torn-down session"
        );

        let live = snapshot
            .live_session
            .expect("successful reload should install a live replacement session");
        assert_eq!(live.render_call_count, 1);
        assert_eq!(live.teardown_count, 0);
        assert_eq!(live.unmount_count, 0);

        worker.shutdown().expect("worker should shut down cleanly");
    }

    #[test]
    fn failed_replacement_load_keeps_the_last_good_session_live_until_validation_succeeds() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        let missing_path = dir.path().join("missing.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [count, setCount] = useState(0);
  return (
    <div id="root" data-slot="column">
      <button id="increment" label="Increment" onClick={() => setCount((value) => value + 1)} />
      <label id="count" text={String(count)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("initial reload fixture should be written");

        let mut worker =
            JsxRuntimeSessionWorker::spawn(None).expect("worker should spawn successfully");
        worker
            .request_reload(&entry_path)
            .expect("initial worker reload should queue");
        let initial_rendered = wait_for_loaded_reload(&worker);
        let tree = initial_rendered
            .tree
            .expect("initial reload should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "count").expect("count label should exist")),
            Some("0")
        );

        worker
            .request_dispatch_events(vec![ContractEvent::new("increment", EventKind::Clicked)])
            .expect("initial dispatch should queue");
        let rendered = wait_for_dispatch_result(&worker);
        let tree = rendered.tree.expect("dispatch should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "count").expect("count label should exist")),
            Some("1")
        );

        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";
import { Missing } from "./missing.tsx";

function App() {
  return <Missing />;
}

render(<App />);
"#,
        )
        .expect("broken reload fixture should be written");

        worker
            .request_reload(&entry_path)
            .expect("failed replacement reload should queue");
        let (outcome, snapshot) = wait_for_reload_completion(&worker);
        match outcome {
            JsxRuntimeWorkerReloadOutcome::Loaded { .. } => {
                panic!("broken replacement load should fail validation")
            }
            JsxRuntimeWorkerReloadOutcome::Failed(failure) => {
                assert!(
                    failure
                        .dependency_paths
                        .iter()
                        .any(|path| path == &missing_path),
                    "failed replacement load should report the missing dependency"
                );
            }
        }

        assert!(
            snapshot.last_torn_down_session.is_none(),
            "failed validation must not tear down the last good session"
        );
        let live = snapshot
            .live_session
            .expect("failed validation should keep the last good session live");
        assert_eq!(live.dispatch_event_batch_count, 1);
        assert_eq!(live.teardown_count, 0);

        worker
            .request_dispatch_events(vec![ContractEvent::new("increment", EventKind::Clicked)])
            .expect(
                "dispatch after failed reload should still queue against the last good session",
            );
        let rendered = wait_for_dispatch_result(&worker);
        let tree = rendered
            .tree
            .expect("last good session should still produce renders after failed reload");
        assert_eq!(
            label_text(find_node(&tree.root, "count").expect("count label should exist")),
            Some("2")
        );

        worker.shutdown().expect("worker should shut down cleanly");
    }

    #[test]
    fn architecture_v1_integration_matrix_covers_load_dispatch_async_motion_reload_and_teardown() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("architecture-matrix.tsx");
        write_architecture_matrix_fixture(&entry_path, "v1");

        let mut worker = JsxRuntimeSessionWorker::spawn(None).expect("worker should spawn");

        worker
            .request_reload(&entry_path)
            .expect("initial worker reload should queue");
        let (initial_outcome, initial_snapshot) = wait_for_reload_completion(&worker);
        let initial_rendered = match initial_outcome {
            JsxRuntimeWorkerReloadOutcome::Loaded { rendered, .. } => rendered,
            JsxRuntimeWorkerReloadOutcome::Failed(failure) => {
                panic!(
                    "initial architecture reload should succeed: {:#}",
                    failure.error
                )
            }
        };
        let initial_tree = initial_rendered
            .tree
            .expect("initial architecture load should return a tree");
        assert_eq!(
            label_text(
                find_node(&initial_tree.root, "version").expect("version label should exist")
            ),
            Some("v1")
        );
        assert_eq!(
            label_text(find_node(&initial_tree.root, "count").expect("count label should exist")),
            Some("0")
        );
        assert_eq!(
            label_text(
                find_node(&initial_tree.root, "async-status")
                    .expect("async status label should exist"),
            ),
            Some("idle:v1")
        );
        assert_motion_close(
            &initial_rendered.motion,
            "motion-status",
            MotionProperty::Opacity,
            0.0,
        );
        assert_motion_close(
            &initial_rendered.motion,
            "motion-status",
            MotionProperty::X,
            -12.0,
        );
        let initial_live = initial_snapshot
            .live_session
            .expect("initial reload should install a live session");
        assert_eq!(initial_live.render_call_count, 1);
        assert_eq!(initial_live.dispatch_event_batch_count, 0);
        assert!(initial_snapshot.last_torn_down_session.is_none());

        worker
            .request_dispatch_events(vec![ContractEvent::new(
                "sync-increment",
                EventKind::Clicked,
            )])
            .expect("sync dispatch should queue");
        let sync_rendered = wait_for_dispatch_result(&worker);
        let sync_tree = sync_rendered
            .tree
            .expect("sync dispatch should return a tree");
        assert_eq!(
            label_text(find_node(&sync_tree.root, "count").expect("count label should exist")),
            Some("1")
        );

        worker
            .request_dispatch_events(vec![ContractEvent::new(
                "schedule-async",
                EventKind::Clicked,
            )])
            .expect("async scheduling dispatch should queue");
        let scheduled_rendered = wait_for_dispatch_result(&worker);
        assert!(
            scheduled_rendered.tree.is_none(),
            "timer scheduling should not require a synchronous tree update"
        );

        let deadline = Instant::now() + Duration::from_secs(2);
        let (async_rendered, async_snapshot) = loop {
            worker
                .request_drain_pending_runtime_updates()
                .expect("async drain should queue");
            let (result, snapshot) = wait_for_drain_completion(&worker);
            match result {
                Some(rendered) if rendered.tree.is_some() => break (rendered, snapshot),
                Some(_) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(2))
                }
                None if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(2)),
                _ => panic!("timed out waiting for architecture async drain"),
            }
        };
        let async_tree = async_rendered
            .tree
            .expect("async drain should eventually materialize a tree");
        assert_eq!(
            label_text(
                find_node(&async_tree.root, "async-status")
                    .expect("async status label should exist"),
            ),
            Some("done:v1")
        );
        let async_live = async_snapshot
            .live_session
            .expect("async drain should keep the live session installed");
        assert!(
            async_live.runtime.timer_fire_count >= 1,
            "async drain should reflect at least one fired timer"
        );
        assert!(!async_live.runtime.pending_host_wake);

        worker
            .request_dispatch_events(vec![ContractEvent::new(
                "toggle-motion",
                EventKind::Clicked,
            )])
            .expect("motion dispatch should queue");
        let motion_commit = wait_for_dispatch_result(&worker);
        assert!(
            motion_commit.tree.is_none(),
            "motion retargets should remain separate from contract-tree rematerialization"
        );
        assert_motion_close(
            &motion_commit.motion,
            "motion-status",
            MotionProperty::Opacity,
            1.0,
        );
        assert_motion_close(
            &motion_commit.motion,
            "motion-status",
            MotionProperty::X,
            0.0,
        );

        worker
            .request_tick_motion(0.5)
            .expect("motion tick should queue");
        let (motion_tick, tick_snapshot) = wait_for_tick_completion(&worker);
        assert!(
            motion_tick.tree.is_none(),
            "motion ticks should not rematerialize the contract tree"
        );
        assert_motion_close(
            &motion_tick.motion,
            "motion-status",
            MotionProperty::Opacity,
            1.0,
        );
        assert_motion_close(&motion_tick.motion, "motion-status", MotionProperty::X, 0.0);
        let tick_live = tick_snapshot
            .live_session
            .expect("motion tick should keep a live session");
        assert_eq!(tick_live.motion_commit_update_count, 1);
        assert_eq!(tick_live.motion_only_update_count, 1);

        write_architecture_matrix_fixture(&entry_path, "v2");
        worker
            .request_reload(&entry_path)
            .expect("replacement reload should queue");
        let (reload_outcome, reload_snapshot) = wait_for_reload_completion(&worker);
        let reloaded = match reload_outcome {
            JsxRuntimeWorkerReloadOutcome::Loaded { rendered, .. } => rendered,
            JsxRuntimeWorkerReloadOutcome::Failed(failure) => {
                panic!(
                    "replacement architecture reload should succeed: {:#}",
                    failure.error
                )
            }
        };
        let reloaded_tree = reloaded
            .tree
            .expect("replacement reload should return a tree");
        assert_eq!(
            label_text(
                find_node(&reloaded_tree.root, "version").expect("version label should exist")
            ),
            Some("v2")
        );
        assert_eq!(
            label_text(find_node(&reloaded_tree.root, "count").expect("count label should exist")),
            Some("0")
        );
        let torn_down = reload_snapshot
            .last_torn_down_session
            .expect("successful reload should tear down the previous session");
        assert_eq!(torn_down.teardown_count, 1);
        assert_eq!(torn_down.unmount_count, 1);
        assert_eq!(torn_down.runtime.active_timer_count, 0);
        assert_eq!(
            reload_snapshot
                .last_torn_down_recovery_state
                .expect("reload should snapshot the previous recovery state")
                .disposition,
            RuntimeRecoveryDisposition::Continue
        );

        worker.request_teardown().expect("teardown should queue");
        let (teardown_logs, teardown_snapshot) = wait_for_teardown_completion(&worker);
        assert!(
            teardown_logs
                .iter()
                .any(|entry| entry.contains("cleanup:v2")),
            "teardown should run cleanup for the live replacement session"
        );
        assert!(teardown_snapshot.live_session.is_none());
        let final_torn_down = teardown_snapshot
            .last_torn_down_session
            .expect("teardown should snapshot the torn-down live session");
        assert_eq!(final_torn_down.teardown_count, 1);
        assert_eq!(final_torn_down.runtime.active_timer_count, 0);

        worker.shutdown().expect("worker should shut down cleanly");
    }

    #[test]
    fn motion_react_import_commits_retained_motion_values() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("motion.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";
import { motion } from "motion/react";

function App() {
  const [open, setOpen] = useState(false);
  return (
    <div id="root" data-slot="column">
      <button id="toggle" label="Toggle" onClick={() => setOpen(true)} />
      <motion.label
        id="status"
        text="Status"
        animate={{ opacity: open ? 1 : 0, x: open ? 0 : -10 }}
        transition={{ duration: 1, ease: "linear" }}
      />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        assert!(!rendered.motion.active);
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 0.0);
        assert_motion_close(&rendered.motion, "status", MotionProperty::X, -10.0);

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle", EventKind::Clicked)])
            .expect("event dispatch should retarget motion");
        assert!(rendered.tree.is_none());
        assert!(!rendered.motion.active);
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 1.0);
        assert_motion_close(&rendered.motion, "status", MotionProperty::X, 0.0);

        let rendered = session.tick_motion(0.0).expect("motion tick should start");
        assert!(rendered.tree.is_none());
        assert!(!rendered.motion.active);
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 1.0);
        assert_motion_close(&rendered.motion, "status", MotionProperty::X, 0.0);

        let rendered = session
            .tick_motion(0.5)
            .expect("motion tick should advance");
        assert!(rendered.tree.is_none());
        assert!(!rendered.motion.active);
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 1.0);
        assert_motion_close(&rendered.motion, "status", MotionProperty::X, 0.0);
    }

    #[test]
    fn motion_only_commit_updates_do_not_rematerialize_the_contract_tree() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("motion-only.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";
import { motion } from "motion/react";

function App() {
  const [open, setOpen] = useState(false);
  return (
    <div id="root" data-slot="column">
      <button id="toggle" label="Toggle" onClick={() => setOpen((value) => !value)} />
      <motion.label
        id="status"
        text="Status"
        animate={{ opacity: open ? 1 : 0, x: open ? 0 : -10 }}
        transition={{ duration: 1, ease: "linear" }}
      />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let initial_tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&initial_tree.root, "status").expect("status label should exist")),
            Some("Status")
        );
        let initial_metrics = session.debug_metrics();
        assert_eq!(initial_metrics.contract_tree_materialization_count, 1);
        assert_eq!(initial_metrics.motion_commit_update_count, 0);

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle", EventKind::Clicked)])
            .expect("motion-only dispatch should succeed");
        assert!(
            rendered.tree.is_none(),
            "motion-only commits must not rematerialize the contract tree"
        );
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 1.0);
        assert_motion_close(&rendered.motion, "status", MotionProperty::X, 0.0);

        let metrics = session.debug_metrics();
        assert_eq!(metrics.contract_tree_materialization_count, 1);
        assert_eq!(metrics.motion_commit_update_count, 1);
        assert_eq!(metrics.contract_tree_noop_update_count, 0);
    }

    #[test]
    fn structural_updates_preserve_retained_motion_state_for_unaffected_nodes() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("motion-structural.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";
import { motion } from "motion/react";

function App() {
  const [open, setOpen] = useState(false);
  const [message, setMessage] = useState("before");

  return (
    <div id="root" data-slot="column">
      <button id="toggle-motion" label="Toggle Motion" onClick={() => setOpen(true)} />
      <button id="toggle-text" label="Toggle Text" onClick={() => setMessage("after")} />
      <motion.label
        id="status"
        text="Status"
        animate={{ opacity: open ? 1 : 0, x: open ? 0 : -10 }}
        transition={{ duration: 1, ease: "linear" }}
      />
      <label id="message" text={message} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle-motion", EventKind::Clicked)])
            .expect("motion retarget should succeed");
        assert!(rendered.tree.is_none());
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 1.0);
        assert_motion_close(&rendered.motion, "status", MotionProperty::X, 0.0);

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle-text", EventKind::Clicked)])
            .expect("structural update should succeed");
        let tree = rendered
            .tree
            .expect("structural update should rematerialize the tree");
        assert_eq!(
            label_text(find_node(&tree.root, "message").expect("message label should exist")),
            Some("after")
        );
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 1.0);
        assert_motion_close(&rendered.motion, "status", MotionProperty::X, 0.0);

        let metrics = session.debug_metrics();
        assert_eq!(metrics.contract_tree_materialization_count, 2);
        assert_eq!(metrics.motion_commit_update_count, 1);
    }

    #[test]
    fn reload_with_hot_state_api_reloads_with_cold_state_across_import_updates() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        let child_path = dir.path().join("copy.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";
import { Copy } from "./copy.tsx";

function App() {
  const [checked, setChecked] = useState(false);
  return (
    <div id="root" data-slot="column">
      <input
        id="toggle"
        type="checkbox"
        checked={checked}
        onToggle={(event, value) => setChecked(Boolean(value))}
      />
      <label id="status" text={checked ? "On" : "Off"} />
      <Copy />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("entry file should be written");
        std::fs::write(
            &child_path,
            r#"
export function Copy() {
  return <label id="copy" text="Old copy" />;
}
"#,
        )
        .expect("child file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(false));
        assert_eq!(
            label_text(find_node(&tree.root, "copy").expect("copy label should exist")),
            Some("Old copy")
        );
        assert!(
            session
                .dependency_paths()
                .iter()
                .any(|path| path == &child_path),
            "dependency paths should include imported child module",
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle", EventKind::Toggled)
                .value(Some(EventValue::Boolean(true)))])
            .expect("event dispatch should update hook state");
        let tree = rendered.tree.expect("changed render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(true));
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("On")
        );

        std::fs::write(
            &child_path,
            r#"
export function Copy() {
  return <label id="copy" text="New copy" />;
}
"#,
        )
        .expect("updated child file should be written");

        let hot_reload_state = session
            .capture_hot_reload_state()
            .expect("hot reload state should be captured");
        assert!(
            hot_reload_state.hook_state.is_empty(),
            "capture is API compatibility only in the React cutover bundle",
        );
        let (_reloaded_session, rendered) =
            JsxRuntimeSession::load_with_hot_reload_state(&entry_path, Some(&hot_reload_state))
                .expect("tsx should reload with a cold remount");
        let tree = rendered.tree.expect("reloaded render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(false));
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("Off")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "copy").expect("copy label should exist")),
            Some("New copy")
        );
    }

    #[test]
    fn reload_with_changed_hook_count_resets_component_state() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [checked, setChecked] = useState(false);
  return (
    <div id="root" data-slot="column">
      <input
        id="toggle"
        type="checkbox"
        checked={checked}
        onToggle={(event, value) => setChecked(Boolean(value))}
      />
      <label id="status" text={checked ? "On" : "Off"} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("entry file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(false));

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle", EventKind::Toggled)
                .value(Some(EventValue::Boolean(true)))])
            .expect("event dispatch should update hook state");
        let tree = rendered.tree.expect("changed render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(true));

        let hot_reload_state = session
            .capture_hot_reload_state()
            .expect("hot reload state should be captured");
        assert!(
            hot_reload_state.hook_state.is_empty(),
            "capture is API compatibility only in the React cutover bundle",
        );
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [phase] = useState("new");
  const [checked, setChecked] = useState(false);
  return (
    <div id="root" data-slot="column">
      <label id="phase" text={phase} />
      <input
        id="toggle"
        type="checkbox"
        checked={checked}
        onToggle={(event, value) => setChecked(Boolean(value))}
      />
      <label id="status" text={checked ? "On" : "Off"} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("updated entry file should be written");

        let (_reloaded_session, rendered) =
            JsxRuntimeSession::load_with_hot_reload_state(&entry_path, Some(&hot_reload_state))
                .expect("tsx should reload with reset state");
        let tree = rendered.tree.expect("reloaded render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(false));
        assert_eq!(
            label_text(find_node(&tree.root, "phase").expect("phase label should exist")),
            Some("new")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("Off")
        );
    }

    #[test]
    fn hot_reload_capture_returns_the_default_empty_snapshot() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function Child() {
  const [count] = useState(1);
  return <label id="child" text={String(count)} />;
}

function App() {
  const [visible, setVisible] = useState(true);
  return (
    <div id="root" data-slot="column">
      <button id="toggle" label="Toggle" onClick={() => setVisible(false)} />
      {visible && <Child />}
      <label id="status" text={visible ? "Visible" : "Hidden"} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("entry file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert!(find_node(&tree.root, "child").is_some());

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle", EventKind::Clicked)])
            .expect("toggle should rerender");
        let tree = rendered.tree.expect("changed render should return a tree");
        assert!(find_node(&tree.root, "child").is_none());
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("Hidden")
        );

        let hot_reload_state = session
            .capture_hot_reload_state()
            .expect("hot reload state should be captured");
        assert!(
            hot_reload_state.hook_state.is_empty(),
            "capture returns the default empty snapshot during the React cutover bundle",
        );
    }

    #[test]
    fn hot_reload_restore_input_is_ignored() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [handler] = useState(() => () => "handler");
  const [checked, setChecked] = useState(false);
  return (
    <div id="root" data-slot="column">
      <label id="handler-type" text={typeof handler} />
      <input
        id="toggle"
        type="checkbox"
        checked={checked}
        onToggle={(event, value) => setChecked(Boolean(value))}
      />
      <label id="status" text={checked ? "On" : "Off"} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("entry file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(false));

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle", EventKind::Toggled)
                .value(Some(EventValue::Boolean(true)))])
            .expect("event dispatch should update hook state");
        let tree = rendered.tree.expect("changed render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(true));

        let hot_reload_state = HotReloadState {
            hook_state: std::collections::BTreeMap::from([(
                "root:App".to_owned(),
                vec![serde_json::json!(true)],
            )]),
        };

        let (_reloaded_session, rendered) =
            JsxRuntimeSession::load_with_hot_reload_state(&entry_path, Some(&hot_reload_state))
                .expect("tsx should reload with ignored restore input");
        let tree = rendered.tree.expect("reloaded render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(false));
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("Off")
        );
    }

    #[test]
    fn load_outcome_reports_attempted_dependency_paths_on_failure() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        let child_path = dir.path().join("child.tsx");
        let missing_path = dir.path().join("missing.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";
import { Child } from "./child.tsx";

render(<Child />);
"#,
        )
        .expect("entry file should be written");
        std::fs::write(
            &child_path,
            r#"
import { Missing } from "./missing.tsx";

export function Child() {
  return <Missing />;
}
"#,
        )
        .expect("child file should be written");

        match JsxRuntimeSession::load_with_hot_reload_state_outcome(&entry_path, None) {
            JsxRuntimeLoadOutcome::Loaded { .. } => {
                panic!("missing dependency should fail to load")
            }
            JsxRuntimeLoadOutcome::Failed(failure) => {
                assert!(
                    !failure.error.to_string().trim().is_empty(),
                    "expected load failure to produce an error"
                );
                assert!(
                    failure.dependency_paths.contains(&missing_path),
                    "expected attempted dependency set to include the missing import, got: {:?}",
                    failure.error
                );
                assert_eq!(
                    failure.dependency_paths,
                    vec![entry_path, child_path, missing_path]
                );
            }
        }
    }

    #[test]
    fn native_html_style_tags_lower_to_contract_families() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("native.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return (
    <div id="root" data-slot="column">
      <input id="check" type="checkbox" checked={true} />
      <input id="switch" type="checkbox" role="switch" checked={true} />
      <input id="radio" type="radio" checked={false} />
      <input id="slider" type="range" value={0.75} />
      <input id="text" data-slot="input" value="Text" />
      <input id="number" type="number" value={7} />
      <button id="button">Button</button>
      <label id="label" text="Label" />
      <progress id="progress" value={0.5} />
      <kbd id="kbd">Ctrl</kbd>
      <img id="image" src="builtin:showcase-image" />
      <hr id="rule" />
      <div id="row" data-slot="row" />
      <div id="inset" data-slot="inset" />
      <div id="box" data-slot="sized-box" />
      <div id="spacer" data-slot="spacer" />
      <div id="card" data-slot="card" />
      <div id="context" data-slot="context-menu" entries={[]} />
      <div id="spinner" data-slot="spinner" role="status" aria-busy="true" />
      <div id="skeleton" data-slot="skeleton" aria-hidden="true" />
      <span id="color" data-slot="color" />
      <span id="icon" data-slot="icon" aria-hidden="true" />
      <span id="twemoji" data-slot="twemoji" />
      <span id="tooltip" data-slot="tooltip" triggerLabel="Hover" text="Tip" />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (_session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        let ContractNode::Column(root) = tree.root else {
            panic!("expected column root");
        };
        let families = root
            .children
            .iter()
            .map(|node| node.family_id().as_str().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            families,
            vec![
                "checkbox".to_owned(),
                "switch".to_owned(),
                "radio".to_owned(),
                "slider".to_owned(),
                "input".to_owned(),
                "number-input".to_owned(),
                "button".to_owned(),
                "label".to_owned(),
                "progress".to_owned(),
                "kbd".to_owned(),
                "image".to_owned(),
                "separator".to_owned(),
                "row".to_owned(),
                "inset".to_owned(),
                "sized-box".to_owned(),
                "spacer".to_owned(),
                "card".to_owned(),
                "context-menu".to_owned(),
                "spinner".to_owned(),
                "skeleton".to_owned(),
                "color".to_owned(),
                "icon".to_owned(),
                "twemoji".to_owned(),
                "tooltip".to_owned(),
            ]
        );
    }

    #[test]
    fn legacy_contract_family_host_tags_emit_deprecation_log() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("legacy-host-tag.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return (
    <column id="root">
      <button id="button">Button</button>
    </column>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (_session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("legacy host tags still render");
        assert!(
            rendered
                .logs
                .iter()
                .any(|entry| entry.contains("Deprecated JSX host element <column>")),
            "expected deprecated host element warning, got {:?}",
            rendered.logs
        );
    }

    #[test]
    fn data_slot_requires_an_html_style_host_tag() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("non-html-slot.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return <not-html id="root" data-slot="column" />;
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let error = JsxRuntimeSession::load(&entry_path)
            .expect_err("non-html host tag should not be accepted through data-slot");
        assert!(
            error.to_string().contains("Unknown contract family"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn invalid_html_data_slot_is_rejected() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("invalid-slot.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return <div id="root" data-slot="not-a-family" />;
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let error = JsxRuntimeSession::load(&entry_path)
            .expect_err("unknown data-slot should not fall back to div lowering");
        assert!(
            error.to_string().contains("Unknown contract family"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn identity_sensitive_families_require_explicit_stable_node_ids() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("missing-input-id.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return (
    <div id="root" data-slot="column">
      <input data-slot="input" value="Name" />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let error =
            JsxRuntimeSession::load(&entry_path).expect_err("identity-sensitive input should fail");
        assert!(
            error
                .to_string()
                .contains("requires an explicit stable node_id"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn duplicate_contract_node_ids_fail_clearly() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("duplicate-node-id.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return (
    <div id="root" data-slot="column">
      <label id="dup" text="One" />
      <label id="dup" text="Two" />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let error =
            JsxRuntimeSession::load(&entry_path).expect_err("duplicate node ids should fail");
        assert!(
            error
                .to_string()
                .contains(r#"Duplicate contract node_id "dup""#),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn stateless_layout_and_text_sugar_can_still_use_fallback_node_ids() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("fallback-layout.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return (
    <div data-slot="column">
      <div data-slot="row">
        <label text="Fallback label" />
      </div>
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (_session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("stateless fallback ids should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        let ContractNode::Column(root) = tree.root else {
            panic!("expected column root");
        };
        assert!(!root.common.node_id.as_str().is_empty());
        assert_eq!(root.children.len(), 1);
        let ContractNode::Row(row) = &root.children[0] else {
            panic!("expected row child");
        };
        assert!(!row.common.node_id.as_str().is_empty());
        assert_eq!(row.children.len(), 1);
        let ContractNode::Label(label) = &row.children[0] else {
            panic!("expected label grandchild");
        };
        assert!(!label.common.node_id.as_str().is_empty());
        assert_eq!(label.text, "Fallback label");
    }

    #[test]
    fn native_div_lowers_to_flow_container_from_flex_classes() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("native-container.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return (
    <div id="root" className="flex flex-col gap-3">
      <button id="child">Save</button>
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (_session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        let ContractNode::Column(root) = tree.root else {
            panic!("expected flex-col div root to lower to a column");
        };
        assert_eq!(root.common.class.as_deref(), Some("flex flex-col gap-3"));
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].family_id().as_str(), "button");
    }
    #[test]
    fn repository_jsx_example_still_loads_through_the_bridge_crate() {
        let example_path = repository_example_path();
        if !example_path.is_file() {
            return;
        }

        let (_session, rendered) =
            JsxRuntimeSession::load(&example_path).expect("repository JSX file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(tree.root.family_id().as_str(), "column");
        assert!(
            rendered
                .logs
                .iter()
                .all(|entry| !entry.contains("warn: Tailwind")),
            "repository example should not emit Tailwind diagnostics, got {:?}",
            rendered.logs
        );
    }

    #[test]
    fn class_name_and_class_list_materialize_into_contract_common_fields() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("classes.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function StyledLabel() {
  return (
    <label
      id="styled"
      text="Styled"
      className="text-lg font-bold text-destructive"
      classList={["bg-card", "border-border"]}
    />
  );
}

render(<StyledLabel />);
"#,
        )
        .expect("tsx file should be written");

        let (_session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        let ContractNode::Label(label) = tree.root else {
            panic!("expected label root");
        };
        assert_eq!(
            label.common.class.as_deref(),
            Some("text-lg font-bold text-destructive")
        );
        assert_eq!(
            label.common.class_list,
            vec!["bg-card".to_owned(), "border-border".to_owned()]
        );
    }

    #[test]
    fn generic_layout_sizing_props_hoist_into_contract_common_layout() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("layout-props.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

render(
  <div id="root" data-slot="column">
    <label
      id="nav-label"
      text="Component authoring"
      width={216}
      layout={{ minWidth: 180 }}
    />
    <div id="lane" data-slot="column" width={320}>
      <label id="copy" text="Lane copy" />
    </div>
  </div>
);
"#,
        )
        .expect("tsx file should be written");

        let (_session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");

        let nav_label = find_node(&tree.root, "nav-label").expect("nav label should exist");
        let ContractNode::Label(label) = nav_label else {
            panic!("expected nav label node");
        };
        let layout = label
            .common
            .layout
            .as_ref()
            .expect("nav label layout should exist");
        assert_eq!(layout.width, Some(ContractLength::Px { value: 216.0 }));
        assert_eq!(layout.min_width, Some(ContractLength::Px { value: 180.0 }));

        let lane = find_node(&tree.root, "lane").expect("lane column should exist");
        let ContractNode::Column(column) = lane else {
            panic!("expected lane column node");
        };
        let layout = column
            .common
            .layout
            .as_ref()
            .expect("lane layout should exist");
        assert_eq!(layout.width, Some(ContractLength::Px { value: 320.0 }));
    }

    #[test]
    fn unknown_tailwind_tokens_emit_runtime_warnings() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("unknown-tailwind.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

render(<button id="bad-button" className="totally-unknown-class">Bad</button>);
"#,
        )
        .expect("tsx file should be written");

        let (_session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        assert!(
            rendered.logs.iter().any(|entry| {
                entry.contains("warn: Tailwind unknown-token")
                    && entry.contains("bad-button")
                    && entry.contains("(button)")
                    && entry.contains("totally-unknown-class")
            }),
            "expected unknown-token warning, got {:?}",
            rendered.logs
        );
    }

    #[test]
    fn unsupported_tailwind_tokens_emit_runtime_warnings() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("unsupported-tailwind.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

render(<input id="search" data-slot="input" className="bg-card" value="" />);
"#,
        )
        .expect("tsx file should be written");

        let (_session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        assert!(
            rendered.logs.iter().any(|entry| {
                entry.contains("warn: Tailwind unsupported-family")
                    && entry.contains("search")
                    && entry.contains("(input)")
                    && entry.contains("bg-card")
            }),
            "expected unsupported-family warning, got {:?}",
            rendered.logs
        );
    }

    #[test]
    fn core_taffy_tailwind_surface_renders_without_runtime_warnings() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("core-taffy-tailwind.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

render(
  <div id="root" data-slot="column" className="gap-2">
    <div
      id="grid-card"
      data-slot="card"
      className="grid grid-cols-2 gap-2 overflow-hidden rounded-lg rounded-l-none rounded-tr-xl"
    >
      <button id="grid-a" className="min-w-44">A</button>
      <button id="grid-b">B</button>
    </div>
    <div id="flex-row" data-slot="row" className="flex gap-2">
      <button id="grower" className="grow basis-[50%]">Grow</button>
      <button id="peer">Peer</button>
    </div>
  </div>
);
"#,
        )
        .expect("tsx file should be written");

        let (_session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        assert!(
            rendered
                .logs
                .iter()
                .all(|entry| !entry.contains("warn: Tailwind")),
            "expected no Tailwind warnings, got {:?}",
            rendered.logs
        );
    }

    #[test]
    fn supported_react_hooks_drive_async_and_subscription_updates() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("hooks.tsx");
        std::fs::write(
            &entry_path,
            r#"
import {
  createContext,
  render,
  startTransition,
  useContext,
  useDeferredValue,
  useEffect,
  useReducer,
  useRef,
  useState,
  useSyncExternalStore,
} from "egui";

const PhaseContext = createContext("boot");

function createStore() {
  let snapshot = 0;
  const listeners = new Set();
  return {
    getSnapshot() {
      return snapshot;
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    emit(nextValue) {
      snapshot = nextValue;
      for (const listener of listeners) {
        listener();
      }
    },
  };
}

const store = createStore();

function reducer(state, action) {
  switch (action.type) {
    case "loading":
      return { requestId: action.requestId, status: "loading", summary: "loading" };
    case "ready":
      if (action.requestId !== state.requestId) {
        return state;
      }
      return {
        requestId: action.requestId,
        status: "ready",
        summary: `ready:${action.query}`,
      };
    default:
      return state;
  }
}

function PhaseLabel() {
  const phase = useContext(PhaseContext);
  return <label id="phase" text={phase} />;
}

function App() {
  const [query] = useState("runtime");
  const deferred = useDeferredValue(query);
  const [state, dispatch] = useReducer(reducer, {
    requestId: 0,
    status: "idle",
    summary: "waiting",
  });
  const requestIdRef = useRef(0);
  const tick = useSyncExternalStore(
    store.subscribe,
    store.getSnapshot,
    store.getSnapshot,
  );

  useEffect(() => {
    const requestId = requestIdRef.current + 1;
    requestIdRef.current = requestId;
    dispatch({ type: "loading", requestId });
    const timeoutHandle = setTimeout(() => {
      store.emit(7);
      startTransition(() => {
        dispatch({ type: "ready", requestId, query: deferred });
      });
    }, 15);
    return () => clearTimeout(timeoutHandle);
  }, [deferred]);

  return (
    <PhaseContext.Provider value={state.status}>
      <div id="root" data-slot="column">
        <PhaseLabel />
        <label id="deferred" text={deferred} />
        <label id="summary" text={state.summary} />
        <label id="tick" text={String(tick)} />
      </div>
    </PhaseContext.Provider>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "summary").expect("summary label should exist")),
            Some("waiting")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "tick").expect("tick label should exist")),
            Some("0")
        );

        let rendered = drain_async_until(&mut session, Duration::from_millis(100), |rendered| {
            let Some(tree) = rendered.tree.as_ref() else {
                return false;
            };
            label_text(find_node(&tree.root, "phase").expect("phase label should exist"))
                == Some("ready")
        })
        .expect("timer-driven async update should arrive");
        let tree = rendered.tree.expect("async update should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "phase").expect("phase label should exist")),
            Some("ready")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "deferred").expect("deferred label should exist")),
            Some("runtime")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "summary").expect("summary label should exist")),
            Some("ready:runtime")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "tick").expect("tick label should exist")),
            Some("7")
        );
    }

    #[test]
    fn noop_rerender_does_not_materialize_a_new_contract_tree() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("noop.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [label, setLabel] = useState("steady");
  return (
    <div id="root" data-slot="column">
      <button id="noop" label="No-op" onClick={() => setLabel((current) => current)} />
      <label id="status" text={label} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        assert!(rendered.tree.is_some());
        let initial_metrics = session.debug_metrics();
        assert_eq!(initial_metrics.contract_tree_materialization_count, 1);
        assert_eq!(initial_metrics.render_call_count, 1);

        let rendered = session
            .dispatch_events(&[ContractEvent::new("noop", EventKind::Clicked)])
            .expect("noop dispatch should succeed");
        assert!(rendered.tree.is_none());

        let metrics = session.debug_metrics();
        assert_eq!(metrics.contract_tree_materialization_count, 1);
        assert_eq!(metrics.contract_tree_noop_update_count, 1);
        assert_eq!(metrics.dispatch_event_batch_count, 1);
        assert_eq!(
            metrics.mutation_batch_count,
            initial_metrics.mutation_batch_count
        );
    }

    #[test]
    fn motion_only_tick_keeps_the_contract_tree_materialization_count_flat() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("motion-metrics.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";
import { motion } from "motion/react";

function App() {
  const [open, setOpen] = useState(false);
  return (
    <div id="root" data-slot="column">
      <button id="toggle" label="Toggle" onClick={() => setOpen(true)} />
      <motion.label
        id="status"
        text="Status"
        animate={{ opacity: open ? 1 : 0, x: open ? 0 : -10 }}
        transition={{ duration: 1, ease: "linear" }}
      />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        assert!(rendered.tree.is_some());
        assert_eq!(
            session.debug_metrics().contract_tree_materialization_count,
            1
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle", EventKind::Clicked)])
            .expect("motion retarget should succeed");
        assert!(rendered.tree.is_none());

        let rendered = session
            .tick_motion(0.5)
            .expect("motion-only tick should succeed");
        assert!(rendered.tree.is_none());

        let metrics = session.debug_metrics();
        assert_eq!(metrics.contract_tree_materialization_count, 1);
        assert_eq!(metrics.motion_commit_update_count, 1);
        assert_eq!(metrics.motion_only_update_count, 1);
    }

    #[test]
    fn timer_driven_async_update_wakes_the_host_and_commits_once() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("async-metrics.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, requestRepaint, useEffect, useState } from "egui";

function App() {
  const [status, setStatus] = useState("idle");

  useEffect(() => {
    const handle = setTimeout(() => {
      setStatus("ready");
    }, 0);
    return () => clearTimeout(handle);
  }, []);

  return <label id="status" text={status} />;
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("idle")
        );
        let initial_metrics = session.debug_metrics();

        let rendered = drain_async_until(&mut session, Duration::from_millis(100), |rendered| {
            let Some(tree) = rendered.tree.as_ref() else {
                return false;
            };
            label_text(find_node(&tree.root, "status").expect("status label should exist"))
                == Some("ready")
        })
        .expect("timer-driven rerender should arrive");
        let tree = rendered.tree.expect("async update should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("ready")
        );

        let metrics = session.debug_metrics();
        assert_eq!(
            metrics.mutation_batch_count,
            initial_metrics.mutation_batch_count + 1
        );
        assert_eq!(
            metrics.contract_tree_materialization_count,
            initial_metrics.contract_tree_materialization_count + 1
        );
        assert!(metrics.runtime.host_wake_count >= 1);
        assert!(
            metrics.runtime.host_callback_drain_cycles
                >= initial_metrics.runtime.host_callback_drain_cycles + 1
        );
        assert!(
            metrics.runtime.host_callbacks_invoked
                >= initial_metrics.runtime.host_callbacks_invoked + 1
        );
    }

    #[test]
    fn effect_driven_async_update_wakes_the_host_and_commits_once() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("effect-async-metrics.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, requestRepaint, useEffect, useState } from "egui";

function App() {
  const [status, setStatus] = useState("idle");

  useEffect(() => {
    requestRepaint();
    setStatus("effect-ready");
  }, []);

  return <label id="status" text={status} />;
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("idle")
        );
        let initial_metrics = session.debug_metrics();

        let rendered = drain_async_until(&mut session, Duration::from_millis(100), |rendered| {
            let Some(tree) = rendered.tree.as_ref() else {
                return false;
            };
            label_text(find_node(&tree.root, "status").expect("status label should exist"))
                == Some("effect-ready")
        })
        .expect("effect-driven rerender should arrive");
        let tree = rendered.tree.expect("async update should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("effect-ready")
        );

        let metrics = session.debug_metrics();
        assert_eq!(
            metrics.mutation_batch_count,
            initial_metrics.mutation_batch_count + 1
        );
        assert_eq!(
            metrics.contract_tree_materialization_count,
            initial_metrics.contract_tree_materialization_count + 1
        );
        assert!(
            initial_metrics.runtime.host_wake_count >= 1,
            "the effect should request a host wake before the async drain runs"
        );
        assert!(initial_metrics.runtime.pending_host_wake);
        assert!(
            metrics.runtime.host_callback_drain_cycles
                >= initial_metrics.runtime.host_callback_drain_cycles + 1
        );
    }

    #[test]
    fn raf_callback_driven_update_wakes_the_host_and_commits_once() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("raf-async-metrics.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useEffect, useState } from "egui";

function App() {
  const [status, setStatus] = useState("idle");

  useEffect(() => {
    const handle = requestAnimationFrame(() => {
      setStatus("raf-ready");
    });
    return () => cancelAnimationFrame(handle);
  }, []);

  return <label id="status" text={status} />;
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("idle")
        );
        let initial_metrics = session.debug_metrics();

        let rendered = drain_async_until(&mut session, Duration::from_millis(250), |rendered| {
            let Some(tree) = rendered.tree.as_ref() else {
                return false;
            };
            label_text(find_node(&tree.root, "status").expect("status label should exist"))
                == Some("raf-ready")
        })
        .expect("raf-driven rerender should arrive");
        let tree = rendered.tree.expect("async update should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("raf-ready")
        );

        let metrics = session.debug_metrics();
        assert_eq!(
            metrics.mutation_batch_count,
            initial_metrics.mutation_batch_count + 1
        );
        assert!(
            metrics.runtime.timer_schedule_count
                >= initial_metrics.runtime.timer_schedule_count + 1
        );
        assert!(metrics.runtime.timer_fire_count >= initial_metrics.runtime.timer_fire_count + 1);
        assert!(
            metrics.runtime.host_callback_drain_cycles
                >= initial_metrics.runtime.host_callback_drain_cycles + 1
        );
    }

    #[test]
    fn bounded_multi_drain_converges_after_a_worst_case_callback_burst() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("burst-async-metrics.tsx");
        let burst_callbacks = HOST_CALLBACK_DRAIN_LIMIT * 2 + 17;
        std::fs::write(
            &entry_path,
            format!(
                r#"
import {{ render, useState }} from "egui";

function App() {{
  const [count, setCount] = useState(0);

  const scheduleBurst = () => {{
    for (let i = 0; i < {burst_callbacks}; i += 1) {{
      setTimeout(() => {{
        setCount((value) => value + 1);
      }}, 0);
    }}
  }};

  return (
    <div id="root" data-slot="column">
      <button id="start" label="Start" onClick={{scheduleBurst}} />
      <label id="count" text={{String(count)}} />
    </div>
  );
}}

render(<App />);
"#
            ),
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "count").expect("count label should exist")),
            Some("0")
        );

        let initial_runtime_metrics = session.debug_metrics().runtime;
        let first_render = session
            .dispatch_events(&[ContractEvent::new("start", EventKind::Clicked)])
            .expect("burst scheduling event should succeed");
        let first_count = first_render
            .tree
            .as_ref()
            .and_then(|tree| find_node(&tree.root, "count"))
            .and_then(label_text)
            .and_then(|text| text.parse::<usize>().ok())
            .unwrap_or(0);
        assert!(
            session.debug_metrics().runtime.host_callback_drain_cycles
                >= initial_runtime_metrics.host_callback_drain_cycles + 1,
            "burst dispatch should consume at least one bounded host drain"
        );

        let initial_runtime_metrics = session.debug_metrics().runtime;
        let mut observed_counts = vec![first_count];
        let deadline = Instant::now() + Duration::from_millis(500);
        while Instant::now() < deadline {
            match session
                .drain_pending_runtime_updates()
                .expect("bounded drain should succeed")
            {
                Some(rendered) => {
                    let Some(tree) = rendered.tree else {
                        continue;
                    };
                    let count = label_text(
                        find_node(&tree.root, "count").expect("count label should exist"),
                    )
                    .expect("count label should be text")
                    .parse::<usize>()
                    .expect("count label should parse as usize");
                    observed_counts.push(count);
                    if count == burst_callbacks {
                        break;
                    }
                }
                None => std::thread::sleep(Duration::from_millis(5)),
            }
        }

        assert!(
            !observed_counts.is_empty(),
            "expected at least one bounded async drain result"
        );
        assert_eq!(
            *observed_counts
                .last()
                .expect("at least one drain result should exist"),
            burst_callbacks,
            "bounded multi-drain convergence should eventually process the full callback burst"
        );
        let metrics = session.debug_metrics();
        assert!(
            metrics.runtime.host_callback_drain_cycles
                >= initial_runtime_metrics.host_callback_drain_cycles + 2,
            "expected additional bounded host drain cycles after the initial dispatch for a burst larger than the per-drain cap: renders={observed_counts:?}"
        );
        assert!(
            metrics.runtime.host_callbacks_invoked >= burst_callbacks as u64,
            "the bounded drains should invoke at least one host callback per scheduled timeout"
        );
        assert!(
            metrics.runtime.host_callbacks_invoked <= burst_callbacks as u64 + 2,
            "worst-case burst convergence should not introduce an unbounded number of extra host callbacks"
        );
        assert!(!metrics.runtime.pending_host_wake);
    }

    #[test]
    fn effect_cleanup_runs_on_dependency_change() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("cleanup.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { log, render, useEffect, useState } from "egui";

function App() {
  const [phase, setPhase] = useState("alpha");

  useEffect(() => {
    log("info", `mount:${phase}`);
    return () => {
      log("info", `cleanup:${phase}`);
    };
  }, [phase]);

  return (
    <div id="root" data-slot="column">
      <button id="advance" label="Advance" onClick={() => setPhase("beta")} />
      <label id="phase" text={phase} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        assert!(
            rendered
                .logs
                .iter()
                .any(|entry| entry.contains("mount:alpha")),
            "expected initial mount log, got {:?}",
            rendered.logs
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("advance", EventKind::Clicked)])
            .expect("event dispatch should rerender");
        let tree = rendered.tree.expect("changed render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "phase").expect("phase label should exist")),
            Some("beta")
        );
        assert!(
            rendered
                .logs
                .iter()
                .any(|entry| entry.contains("cleanup:alpha")),
            "expected cleanup log for alpha, got {:?}",
            rendered.logs
        );
        assert!(
            rendered
                .logs
                .iter()
                .any(|entry| entry.contains("mount:beta")),
            "expected mount log for beta, got {:?}",
            rendered.logs
        );
    }

    #[test]
    fn teardown_runs_effect_cleanup_for_the_live_session() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("teardown.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { log, render, useEffect } from "egui";

function App() {
  useEffect(() => {
    const intervalHandle = setInterval(() => log("info", "tick"), 100);
    return () => {
      clearInterval(intervalHandle);
      log("info", "cleanup:teardown");
    };
  }, []);

  return (
    <div id="root" data-slot="column">
      <label id="status" text="mounted" />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let logs = session.teardown().expect("teardown should succeed");
        assert!(
            logs.iter().any(|entry| entry.contains("cleanup:teardown")),
            "expected teardown cleanup log, got {:?}",
            logs
        );
        let metrics = session.debug_metrics();
        assert_eq!(metrics.teardown_count, 1);
        assert_eq!(metrics.unmount_count, 1);
        assert_eq!(metrics.runtime.active_timer_count, 0);
        assert!(!metrics.runtime.pending_host_wake);
    }

    #[test]
    fn repeated_mount_unmount_cycles_do_not_accumulate_timers_or_pending_wakes() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("mount-unmount.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useEffect } from "egui";

function App() {
  useEffect(() => {
    const handle = setInterval(() => {}, 100);
    return () => clearInterval(handle);
  }, []);

  return <label id="status" text="mounted" />;
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let mut teardown_counts = Vec::with_capacity(LIFECYCLE_ENDURANCE_CYCLES);
        let mut unmount_counts = Vec::with_capacity(LIFECYCLE_ENDURANCE_CYCLES);

        for cycle in 0..LIFECYCLE_ENDURANCE_CYCLES {
            let (mut session, _rendered) =
                JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
            let _ = session.teardown().expect("teardown should succeed");
            let metrics = session.debug_metrics();
            assert_eq!(
                metrics.runtime.active_timer_count, 0,
                "cycle {cycle} should leave no active timers"
            );
            assert!(
                !metrics.runtime.pending_host_wake,
                "cycle {cycle} should leave no pending host wake"
            );
            assert_eq!(
                metrics.teardown_count, 1,
                "cycle {cycle} should increment teardown count exactly once"
            );
            assert_eq!(
                metrics.unmount_count, 1,
                "cycle {cycle} should increment unmount count exactly once"
            );
            teardown_counts.push(metrics.teardown_count);
            unmount_counts.push(metrics.unmount_count);
        }

        assert!(
            teardown_counts.iter().all(|count| *count == 1),
            "teardown counters should be consistent for every endurance cycle"
        );
        assert!(
            unmount_counts.iter().all(|count| *count == 1),
            "unmount counters should be consistent for every endurance cycle"
        );
    }

    #[test]
    fn failed_reload_recovery_does_not_leave_timers_running_in_the_dead_session() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("app.tsx");
        let child_path = dir.path().join("copy.tsx");
        let missing_path = dir.path().join("missing.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";
import { Copy } from "./copy.tsx";

function App() {
  return (
    <div id="root" data-slot="column">
      <Copy />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("entry file should be written");
        std::fs::write(
            &child_path,
            r#"
import { useEffect } from "egui";

export function Copy() {
  useEffect(() => {
    const handle = setInterval(() => {}, 100);
    return () => clearInterval(handle);
  }, []);
  return <label id="copy" text="Old copy" />;
}
"#,
        )
        .expect("child file should be written");

        let (mut old_session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");

        std::fs::write(
            &child_path,
            r#"
import { Missing } from "./missing.tsx";

export function Copy() {
  return <Missing />;
}
"#,
        )
        .expect("updated child file should be written");

        let _ = old_session
            .teardown()
            .expect("old session should tear down");
        let old_metrics = old_session.debug_metrics();
        assert_eq!(old_metrics.teardown_count, 1);
        assert_eq!(old_metrics.runtime.active_timer_count, 0);
        assert!(!old_metrics.runtime.pending_host_wake);

        let failure = match JsxRuntimeSession::load_with_hot_reload_state_outcome(&entry_path, None)
        {
            JsxRuntimeLoadOutcome::Failed(failure) => failure,
            JsxRuntimeLoadOutcome::Loaded { .. } => {
                panic!("reload should fail while the dependency is missing")
            }
        };
        assert!(
            failure
                .dependency_paths
                .iter()
                .any(|path| path == &missing_path),
            "missing dependency should be tracked for recovery"
        );

        std::fs::write(
            &missing_path,
            r#"
export function Missing() {
  return <label id="copy" text="Recovered copy" />;
}
"#,
        )
        .expect("missing dependency should be written");

        let (recovered_session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("reload should recover");
        let tree = rendered
            .tree
            .expect("recovered render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "copy").expect("copy label should exist")),
            Some("Recovered copy")
        );
        let metrics = recovered_session.debug_metrics();
        assert_eq!(metrics.render_call_count, 1);
        assert_eq!(metrics.contract_tree_materialization_count, 1);
    }

    #[test]
    fn uncaught_reconciler_errors_fail_dispatch_deterministically() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("reconciler-uncaught.tsx");
        write_reconciler_error_fixture(&entry_path);

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");

        let error = session
            .dispatch_events(&[ContractEvent::new("uncaught-trigger", EventKind::Clicked)])
            .expect_err("uncaught reconciler errors should fail dispatch");
        assert!(
            error
                .to_string()
                .contains("React reconciler uncaught error: uncaught boom"),
            "unexpected uncaught reconciler error: {error:#}"
        );

        let recovery = session.debug_recovery_state();
        assert_eq!(recovery.category, Some(RuntimeRecoveryCategory::Fatal));
        assert_eq!(recovery.disposition, RuntimeRecoveryDisposition::Teardown);
        assert!(
            recovery
                .message
                .as_deref()
                .is_some_and(|message| message.contains("uncaught boom")),
            "fatal recovery state should retain the uncaught error message"
        );

        let follow_up = session
            .dispatch_events(&[ContractEvent::new("increment", EventKind::Clicked)])
            .expect_err("fatal sessions should require teardown before continuing");
        assert!(
            follow_up.to_string().contains("uncaught boom"),
            "unexpected follow-up fatal error: {follow_up:#}"
        );
    }

    #[test]
    fn render_throw_is_contained_by_error_boundary_and_runtime_remains_interactive() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("reconciler-render-throw.tsx");
        write_error_boundary_render_throw_fixture(&entry_path);

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "outside-count").expect("outside count should exist")),
            Some("0")
        );
        assert!(
            find_node(&tree.root, "render-fallback-message").is_none(),
            "error boundary should not be active before the throw"
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "trigger-render-throw",
                EventKind::Clicked,
            )])
            .expect("render throws should be contained by an error boundary");
        assert!(
            rendered
                .logs
                .iter()
                .any(|entry| entry.contains("reconciler:caught: render boom")),
            "caught render throws should be reported through structured reconciler logs"
        );
        let tree = rendered
            .tree
            .expect("boundary fallback should materialize after a contained render throw");
        assert_eq!(
            label_text(find_node(&tree.root, "outside-count").expect("outside count should exist")),
            Some("0")
        );
        assert_eq!(
            label_text(
                find_node(&tree.root, "render-fallback-message")
                    .expect("boundary fallback message should exist"),
            ),
            Some("render boom")
        );

        let recovery = session.debug_recovery_state();
        assert_eq!(
            recovery.category,
            Some(RuntimeRecoveryCategory::BoundaryContained)
        );
        assert_eq!(
            recovery.disposition,
            RuntimeRecoveryDisposition::BoundedFailure
        );
        assert_eq!(recovery.message.as_deref(), Some("render boom"));

        let rendered = session
            .dispatch_events(&[ContractEvent::new("increment", EventKind::Clicked)])
            .expect("runtime should remain interactive after a contained render throw");
        let tree = rendered
            .tree
            .expect("incrementing outside the boundary should still update the tree");
        assert_eq!(
            label_text(find_node(&tree.root, "outside-count").expect("outside count should exist")),
            Some("1")
        );
        assert_eq!(
            label_text(
                find_node(&tree.root, "render-fallback-message")
                    .expect("boundary fallback message should still exist"),
            ),
            Some("render boom")
        );
    }

    #[test]
    fn effect_throw_is_contained_by_error_boundary_and_runtime_remains_interactive() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("reconciler-effect-throw.tsx");
        write_error_boundary_effect_throw_fixture(&entry_path);

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "outside-count").expect("outside count should exist")),
            Some("0")
        );
        assert!(
            find_node(&tree.root, "effect-fallback-message").is_none(),
            "error boundary should not be active before the throw"
        );

        let mut rendered = session
            .dispatch_events(&[ContractEvent::new(
                "trigger-effect-throw",
                EventKind::Clicked,
            )])
            .expect("effect throws should be contained by an error boundary");
        let mut saw_caught_log = rendered
            .logs
            .iter()
            .any(|entry| entry.contains("reconciler:caught: effect boom"));

        if rendered
            .tree
            .as_ref()
            .is_none_or(|tree| find_node(&tree.root, "effect-fallback-message").is_none())
        {
            let drained =
                drain_async_until(&mut session, Duration::from_millis(250), |candidate| {
                    candidate.tree.as_ref().is_some_and(|tree| {
                        find_node(&tree.root, "effect-fallback-message").is_some()
                    })
                })
                .expect(
                    "contained effect throw should eventually publish a boundary fallback tree",
                );
            saw_caught_log |= drained
                .logs
                .iter()
                .any(|entry| entry.contains("reconciler:caught: effect boom"));
            rendered = drained;
        }

        assert!(
            saw_caught_log,
            "caught effect throws should be reported through structured reconciler logs"
        );
        let tree = rendered
            .tree
            .expect("boundary fallback should materialize after a contained effect throw");
        assert_eq!(
            label_text(find_node(&tree.root, "outside-count").expect("outside count should exist")),
            Some("0")
        );
        assert_eq!(
            label_text(
                find_node(&tree.root, "effect-fallback-message")
                    .expect("boundary fallback message should exist"),
            ),
            Some("effect boom")
        );

        let recovery = session.debug_recovery_state();
        assert_eq!(
            recovery.category,
            Some(RuntimeRecoveryCategory::BoundaryContained)
        );
        assert_eq!(
            recovery.disposition,
            RuntimeRecoveryDisposition::BoundedFailure
        );
        assert_eq!(recovery.message.as_deref(), Some("effect boom"));

        let rendered = session
            .dispatch_events(&[ContractEvent::new("increment", EventKind::Clicked)])
            .expect("runtime should remain interactive after a contained effect throw");
        let tree = rendered
            .tree
            .expect("incrementing outside the boundary should still update the tree");
        assert_eq!(
            label_text(find_node(&tree.root, "outside-count").expect("outside count should exist")),
            Some("1")
        );
        assert_eq!(
            label_text(
                find_node(&tree.root, "effect-fallback-message")
                    .expect("boundary fallback message should still exist"),
            ),
            Some("effect boom")
        );
    }

    #[test]
    fn recoverable_reconciler_errors_are_non_fatal_and_logged() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("reconciler-recoverable.tsx");
        write_reconciler_error_fixture(&entry_path);

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");

        let recoverable = session
            .dispatch_events(&[ContractEvent::new(
                "recoverable-trigger",
                EventKind::Clicked,
            )])
            .expect("recoverable reconciler errors should not fail dispatch");
        assert!(
            recoverable
                .logs
                .iter()
                .any(|entry| entry.contains("reconciler:recoverable: recoverable wobble")),
            "recoverable reconciler errors should be surfaced to Rust logs"
        );

        let recovery = session.debug_recovery_state();
        assert_eq!(
            recovery.category,
            Some(RuntimeRecoveryCategory::Recoverable)
        );
        assert_eq!(recovery.disposition, RuntimeRecoveryDisposition::Continue);
        assert_eq!(recovery.message.as_deref(), Some("recoverable wobble"));

        let rendered = session
            .dispatch_events(&[ContractEvent::new("increment", EventKind::Clicked)])
            .expect("runtime should remain interactive after recoverable reconciler errors");
        let tree = rendered
            .tree
            .expect("increment event should still update and materialize a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "count").expect("count label should exist")),
            Some("1")
        );
    }

    #[test]
    fn rejected_commit_enters_controlled_session_failure_mode() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("commit-rejection.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return (
    <div id="root" data-slot="column">
      <button
        id="trigger"
        label="Trigger"
        onClick={() => {
          const version = globalThis.__eguiContract?.version ?? 1;
          const schemaFingerprint = globalThis.__eguiContract?.schema_fingerprint ?? "";
          const invalidBatch = {
            version,
            schema_fingerprint: schemaFingerprint,
            mutations: [{ kind: "remove_subtree", node_id: "missing" }],
          };
          // @ts-ignore
          Deno.core.ops.op_commit_mutations(JSON.stringify(invalidBatch));
        }}
      />
      <label id="status" text="ready" />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");

        let rejection = session
            .dispatch_events(&[ContractEvent::new("trigger", EventKind::Clicked)])
            .expect_err("invalid commit should be rejected");
        assert!(
            rejection
                .to_string()
                .contains("missing subtree root missing"),
            "unexpected rejection error: {rejection:#}"
        );

        let recovery = session.debug_recovery_state();
        assert_eq!(
            recovery.category,
            Some(RuntimeRecoveryCategory::ProtocolFailed)
        );
        assert_eq!(
            recovery.disposition,
            RuntimeRecoveryDisposition::ReloadRequired
        );
        assert_eq!(recovery.rejected_commit_batch_id, Some(2));

        let follow_up = session
            .dispatch_events(&[ContractEvent::new("trigger", EventKind::Clicked)])
            .expect_err("session should fail deterministically after commit rejection");
        assert!(
            follow_up.to_string().contains("controlled session failure"),
            "unexpected follow-up error: {follow_up:#}"
        );
        assert!(
            follow_up
                .to_string()
                .contains("Reload the session to recover"),
            "unexpected follow-up error: {follow_up:#}"
        );
    }

    #[test]
    fn event_handler_throw_requires_teardown_before_further_host_work() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("event-handler-throw.tsx");
        write_event_handler_throw_fixture(&entry_path);

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");

        let failure = session
            .dispatch_events(&[ContractEvent::new(
                "trigger-event-throw",
                EventKind::Clicked,
            )])
            .expect_err("event-handler throws should fail dispatch");
        assert!(
            failure.to_string().contains("event handler boom"),
            "unexpected event-handler failure: {failure:#}"
        );

        let recovery = session.debug_recovery_state();
        assert_eq!(recovery.category, Some(RuntimeRecoveryCategory::Fatal));
        assert_eq!(recovery.disposition, RuntimeRecoveryDisposition::Teardown);
        assert_eq!(recovery.message.as_deref(), Some("event handler boom"));

        let follow_up = session
            .dispatch_events(&[ContractEvent::new("increment", EventKind::Clicked)])
            .expect_err("fatal sessions should refuse further dispatch work");
        assert!(
            follow_up.to_string().contains("event handler boom"),
            "unexpected fatal follow-up error: {follow_up:#}"
        );

        let logs = session
            .teardown()
            .expect("fatal sessions should still support deterministic teardown");
        assert_eq!(session.debug_metrics().teardown_count, 1);
        assert!(
            logs.iter().all(|entry| !entry.is_empty()),
            "teardown log buffer should remain well-formed"
        );
    }

    #[test]
    fn rejected_commit_blocks_async_drains_with_controlled_session_failure() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("commit-rejection-async.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return (
    <div id="root" data-slot="column">
      <button
        id="trigger"
        label="Trigger"
        onClick={() => {
          const version = globalThis.__eguiContract?.version ?? 1;
          const schemaFingerprint = globalThis.__eguiContract?.schema_fingerprint ?? "";
          const invalidBatch = {
            version,
            schema_fingerprint: schemaFingerprint,
            mutations: [{ kind: "remove_subtree", node_id: "missing" }],
          };
          // @ts-ignore
          Deno.core.ops.op_commit_mutations(JSON.stringify(invalidBatch));
        }}
      />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");

        let _ = session
            .dispatch_events(&[ContractEvent::new("trigger", EventKind::Clicked)])
            .expect_err("invalid commit should be rejected");

        let drain = session
            .drain_pending_runtime_updates()
            .expect_err("draining should fail after commit rejection");
        assert!(
            drain.to_string().contains("controlled session failure"),
            "unexpected drain error: {drain:#}"
        );

        let motion_tick = session
            .tick_motion(0.0)
            .expect_err("motion ticks should fail after commit rejection");
        assert!(
            motion_tick
                .to_string()
                .contains("controlled session failure"),
            "unexpected motion tick error: {motion_tick:#}"
        );
    }

    #[test]
    fn schema_fingerprint_mismatch_is_rejected() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("schema-fingerprint-mismatch.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";

function App() {
  return (
    <div id="root" data-slot="column">
      <button
        id="trigger"
        label="Trigger"
        onClick={() => {
          const version = globalThis.__eguiContract?.version ?? 1;
          const invalidBatch = {
            version,
            schema_fingerprint: "schema-mismatch",
            mutations: [],
          };
          // @ts-ignore
          Deno.core.ops.op_commit_mutations(JSON.stringify(invalidBatch));
        }}
      />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx file should be written");

        let (mut session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");

        let rejection = session
            .dispatch_events(&[ContractEvent::new("trigger", EventKind::Clicked)])
            .expect_err("schema fingerprint mismatch should be rejected");
        assert!(
            rejection.to_string().contains("schema fingerprint"),
            "unexpected rejection error: {rejection:#}"
        );
    }

    #[test]
    fn runtime_api_limits_js_to_the_documented_host_ops_boundary() {
        let source = include_str!("runtime_api.ts");
        let marker = "Deno.core.ops.";
        let mut counts = BTreeMap::<String, usize>::new();
        let mut remainder = source;

        while let Some(index) = remainder.find(marker) {
            let start = index + marker.len();
            let suffix = &remainder[start..];
            let op_name = suffix
                .chars()
                .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
                .collect::<String>();
            assert!(
                !op_name.is_empty(),
                "every Deno.core.ops reference should name a host op"
            );
            *counts.entry(op_name.clone()).or_default() += 1;
            remainder = &suffix[op_name.len()..];
        }

        let actual = counts.keys().cloned().collect::<BTreeSet<_>>();
        let expected = BTreeSet::from([
            "op_commit_mutations".to_owned(),
            "op_commit_mutations_typed".to_owned(),
            "op_host_log".to_owned(),
            "op_host_note_render".to_owned(),
            "op_host_note_unmount".to_owned(),
            "op_host_now_ms".to_owned(),
            "op_host_report_reconciler_error".to_owned(),
        ]);

        assert_eq!(
            actual, expected,
            "runtime_api.ts should only use the documented JS -> Rust host ops boundary"
        );
        assert_eq!(
            counts.get("op_commit_mutations"),
            Some(&1),
            "JSON commit transport should only cross into Rust through enqueueCommitBatch()"
        );
        assert_eq!(
            counts.get("op_commit_mutations_typed"),
            Some(&1),
            "typed commit transport should only cross into Rust through enqueueCommitBatch()"
        );
    }

    fn write_error_boundary_render_throw_fixture(entry_path: &Path) {
        std::fs::write(
            entry_path,
            r#"
import React from "react";
import { render, useState } from "egui";

class RuntimeErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error) {
    return { error };
  }

  render() {
    if (this.state.error != null) {
      const message = this.state.error instanceof Error
        ? this.state.error.message
        : String(this.state.error);
      return (
        <div id="render-fallback" data-slot="column">
          <label id="render-fallback-message" text={message} />
        </div>
      );
    }

    return this.props.children;
  }
}

function ThrowOnRender({ shouldThrow }) {
  if (shouldThrow) {
    throw new Error("render boom");
  }
  return <label id="render-leaf" text="render-ok" />;
}

function App() {
  const [count, setCount] = useState(0);
  const [shouldThrow, setShouldThrow] = useState(false);

  return (
    <div id="root" data-slot="column">
      <button
        id="increment"
        label="Increment"
        onClick={() => setCount((value) => value + 1)}
      />
      <button
        id="trigger-render-throw"
        label="Trigger render throw"
        onClick={() => setShouldThrow(true)}
      />
      <label id="outside-count" text={String(count)} />
      <RuntimeErrorBoundary>
        <ThrowOnRender shouldThrow={shouldThrow} />
      </RuntimeErrorBoundary>
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx fixture should be written");
    }

    fn write_error_boundary_effect_throw_fixture(entry_path: &Path) {
        std::fs::write(
            entry_path,
            r#"
import React from "react";
import { render, useEffect, useState } from "egui";

class RuntimeErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error) {
    return { error };
  }

  render() {
    if (this.state.error != null) {
      const message = this.state.error instanceof Error
        ? this.state.error.message
        : String(this.state.error);
      return (
        <div id="effect-fallback" data-slot="column">
          <label id="effect-fallback-message" text={message} />
        </div>
      );
    }

    return this.props.children;
  }
}

function ThrowOnEffect({ shouldThrow }) {
  useEffect(() => {
    if (shouldThrow) {
      throw new Error("effect boom");
    }
  }, [shouldThrow]);

  return <label id="effect-leaf" text={shouldThrow ? "effect-armed" : "effect-ok"} />;
}

function App() {
  const [count, setCount] = useState(0);
  const [shouldThrow, setShouldThrow] = useState(false);

  return (
    <div id="root" data-slot="column">
      <button
        id="increment"
        label="Increment"
        onClick={() => setCount((value) => value + 1)}
      />
      <button
        id="trigger-effect-throw"
        label="Trigger effect throw"
        onClick={() => setShouldThrow(true)}
      />
      <label id="outside-count" text={String(count)} />
      <RuntimeErrorBoundary>
        <ThrowOnEffect shouldThrow={shouldThrow} />
      </RuntimeErrorBoundary>
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx fixture should be written");
    }

    fn write_event_handler_throw_fixture(entry_path: &Path) {
        std::fs::write(
            entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [count, setCount] = useState(0);

  return (
    <div id="root" data-slot="column">
      <button
        id="increment"
        label="Increment"
        onClick={() => setCount((value) => value + 1)}
      />
      <button
        id="trigger-event-throw"
        label="Trigger event throw"
        onClick={() => {
          throw new Error("event handler boom");
        }}
      />
      <label id="count" text={String(count)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx fixture should be written");
    }

    fn write_architecture_matrix_fixture(entry_path: &Path, version: &str) {
        std::fs::write(
            entry_path,
            format!(
                r#"
import {{ log, render, useEffect, useState }} from "egui";
import {{ motion }} from "motion/react";

function App() {{
  const [count, setCount] = useState(0);
  const [open, setOpen] = useState(false);
  const [asyncStatus, setAsyncStatus] = useState("idle:{version}");

  useEffect(() => {{
    return () => log("info", "cleanup:{version}");
  }}, []);

  return (
    <div id="root" data-slot="column">
      <label id="version" text="{version}" />
      <button
        id="sync-increment"
        label="Increment"
        onClick={{() => setCount((value) => value + 1)}}
      />
      <button
        id="schedule-async"
        label="Schedule async"
        onClick={{() => setTimeout(() => setAsyncStatus("done:{version}"), 1)}}
      />
      <button
        id="toggle-motion"
        label="Toggle motion"
        onClick={{() => setOpen((value) => !value)}}
      />
      <label id="count" text={{String(count)}} />
      <label id="async-status" text={{asyncStatus}} />
      <motion.label
        id="motion-status"
        text="motion-status"
        animate={{{{ opacity: open ? 1 : 0, x: open ? 0 : -12 }}}}
        transition={{{{ duration: 1, ease: "linear" }}}}
      />
    </div>
  );
}}

render(<App />);
"#
            ),
        )
        .expect("tsx fixture should be written");
    }

    fn write_reconciler_error_fixture(entry_path: &Path) {
        std::fs::write(
            entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [count, setCount] = useState(0);
  return (
    <div id="root" data-slot="column">
      <button
        id="increment"
        label="Increment"
        onClick={() => setCount((value) => value + 1)}
      />
      <button
        id="uncaught-trigger"
        label="Uncaught"
        onClick={() => {
          // @ts-ignore
          globalThis.__eguiReportReconcilerErrorForTest("uncaught", "uncaught boom", "<App />", "BoundaryShell");
        }}
      />
      <button
        id="caught-trigger"
        label="Caught"
        onClick={() => {
          // @ts-ignore
          globalThis.__eguiReportReconcilerErrorForTest("caught", "caught boom", "<App />", "BoundaryShell");
        }}
      />
      <button
        id="recoverable-trigger"
        label="Recoverable"
        onClick={() => {
          // @ts-ignore
          globalThis.__eguiReportReconcilerErrorForTest("recoverable", "recoverable wobble", "<App />", "BoundaryShell");
        }}
      />
      <label id="count" text={String(count)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("tsx fixture should be written");
    }

    fn wait_for_reload_completion(
        worker: &JsxRuntimeSessionWorker,
    ) -> (JsxRuntimeWorkerReloadOutcome, JsxRuntimeWorkerDebugSnapshot) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            match worker
                .try_recv_event()
                .expect("worker reload polling should succeed")
            {
                Some(JsxRuntimeWorkerEvent::ReloadCompleted { outcome, snapshot }) => {
                    return (outcome, snapshot);
                }
                Some(_) => continue,
                None => std::thread::sleep(Duration::from_millis(1)),
            }
        }

        panic!("timed out waiting for worker reload");
    }

    fn wait_for_loaded_reload(worker: &JsxRuntimeSessionWorker) -> RenderedJsx {
        let (outcome, _snapshot) = wait_for_reload_completion(worker);
        match outcome {
            JsxRuntimeWorkerReloadOutcome::Loaded { rendered, .. } => rendered,
            JsxRuntimeWorkerReloadOutcome::Failed(failure) => {
                panic!("worker reload should succeed: {:#}", failure.error)
            }
        }
    }

    fn wait_for_dispatch_result(worker: &JsxRuntimeSessionWorker) -> RenderedJsx {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            match worker
                .try_recv_event()
                .expect("worker dispatch polling should succeed")
            {
                Some(JsxRuntimeWorkerEvent::DispatchCompleted { result, .. }) => {
                    return result.expect("worker dispatch should succeed");
                }
                Some(_) => continue,
                None => std::thread::sleep(Duration::from_millis(1)),
            }
        }

        panic!("timed out waiting for worker dispatch");
    }

    fn wait_for_drain_completion(
        worker: &JsxRuntimeSessionWorker,
    ) -> (Option<RenderedJsx>, JsxRuntimeWorkerDebugSnapshot) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            match worker
                .try_recv_event()
                .expect("worker drain polling should succeed")
            {
                Some(JsxRuntimeWorkerEvent::DrainCompleted { result, snapshot }) => {
                    return (result.expect("worker drain should succeed"), snapshot);
                }
                Some(_) => continue,
                None => std::thread::sleep(Duration::from_millis(1)),
            }
        }

        panic!("timed out waiting for worker drain");
    }

    fn wait_for_tick_completion(
        worker: &JsxRuntimeSessionWorker,
    ) -> (RenderedJsx, JsxRuntimeWorkerDebugSnapshot) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            match worker
                .try_recv_event()
                .expect("worker tick polling should succeed")
            {
                Some(JsxRuntimeWorkerEvent::TickCompleted { result, snapshot }) => {
                    return (result.expect("worker tick should succeed"), snapshot);
                }
                Some(_) => continue,
                None => std::thread::sleep(Duration::from_millis(1)),
            }
        }

        panic!("timed out waiting for worker motion tick");
    }

    fn wait_for_teardown_completion(
        worker: &JsxRuntimeSessionWorker,
    ) -> (RuntimeLogBuffer, JsxRuntimeWorkerDebugSnapshot) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            match worker
                .try_recv_event()
                .expect("worker teardown polling should succeed")
            {
                Some(JsxRuntimeWorkerEvent::TeardownCompleted { result, snapshot }) => {
                    return (result.expect("worker teardown should succeed"), snapshot);
                }
                Some(_) => continue,
                None => std::thread::sleep(Duration::from_millis(1)),
            }
        }

        panic!("timed out waiting for worker teardown");
    }

    fn repository_example_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join("runtime-jsx")
            .join("app.jsx")
    }

    fn find_node<'a>(node: &'a ContractNode, node_id: &str) -> Option<&'a ContractNode> {
        if node.node_id().as_str() == node_id {
            return Some(node);
        }

        for child in contract_children(node) {
            if let Some(found) = find_node(child, node_id) {
                return Some(found);
            }
        }

        None
    }

    fn checkbox_value(node: &ContractNode, node_id: &str) -> Option<bool> {
        if node.node_id().as_str() == node_id {
            if let ContractNode::Checkbox(checkbox) = node {
                return Some(checkbox.value);
            }
        }

        for child in contract_children(node) {
            if let Some(value) = checkbox_value(child, node_id) {
                return Some(value);
            }
        }

        None
    }

    fn label_text(node: &ContractNode) -> Option<&str> {
        match node {
            ContractNode::Label(props) => Some(props.text.as_str()),
            _ => None,
        }
    }

    fn contract_children(node: &ContractNode) -> &[ContractNode] {
        match node {
            ContractNode::Row(props) => &props.children,
            ContractNode::Column(props) => &props.children,
            ContractNode::Inset(props) => &props.children,
            ContractNode::SizedBox(props) => &props.children,
            ContractNode::Card(props) => &props.children,
            ContractNode::Sidebar(props) => &props.children,
            ContractNode::Toolbar(props) => &props.children,
            ContractNode::Collapsible(props) => &props.children,
            ContractNode::DialogueModal(props) => &props.children,
            ContractNode::Popover(props) => &props.children,
            ContractNode::ContextMenu(props) => &props.children,
            ContractNode::AudioPlayback(props) => &props.children,
            ContractNode::ImageTile(props) => &props.children,
            _ => &[],
        }
    }

    fn drain_async_until(
        session: &mut JsxRuntimeSession,
        timeout: Duration,
        predicate: impl Fn(&RenderedJsx) -> bool,
    ) -> Option<RenderedJsx> {
        let deadline = Instant::now() + timeout;
        loop {
            match session
                .drain_pending_runtime_updates()
                .expect("draining runtime updates should succeed")
            {
                Some(rendered) if predicate(&rendered) => return Some(rendered),
                Some(_) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                Some(_) => return None,
                None if Instant::now() >= deadline => return None,
                None => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    }

    fn assert_motion_close(
        frame: &MotionFrame,
        node_id: &str,
        property: MotionProperty,
        expected: f32,
    ) {
        let actual = frame
            .values
            .get(&NodeId::from(node_id))
            .and_then(|values| values.get(property))
            .unwrap_or_else(|| panic!("missing motion value {property:?} on node {node_id}"));
        assert!(
            (actual - expected).abs() <= 0.001,
            "expected {property:?} on {node_id} to be close to {expected}, got {actual}"
        );
    }
}
