#![doc = "egui-component contract bridge for clay-jsx-runtime TS/TSX hosts."]

pub mod host_tree;
#[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
pub mod motion;
mod runtime;

pub const EGUI_MODULE_SOURCE: &str = include_str!("mod.js");
pub const EGUI_RUNTIME_SOURCE: &str = include_str!("runtime_api.js");
pub const EGUI_JSX_RUNTIME_SOURCE: &str = include_str!("jsx_runtime_api.js");
pub const EGUI_LOWERING_SOURCE: &str = include_str!("lowering_api.js");
pub const EGUI_MOTION_REACT_SOURCE: &str = include_str!("motion_api.js");

pub use clay_jsx_runtime::{
    extend_logs, push_log, RuntimeLogBuffer, LOG_HISTORY_LIMIT, LOG_MESSAGE_LIMIT_BYTES,
};
pub use host_tree::{HostMutation, HostMutationBatch, HostTree};
pub use motion::{
    MotionEase, MotionFrame, MotionProperty, MotionSpec, MotionTickResult, MotionTransition,
    MotionValues,
};
pub use runtime::{
    HotReloadState, JsxRuntimeDebugMetrics, JsxRuntimeLoadFailure, JsxRuntimeLoadOutcome,
    JsxRuntimeSession, RenderedJsx,
};

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        time::{Duration, Instant},
    };

    use clay_jsx_runtime::contract::{
        ContractEvent, ContractLength, ContractNode, EventKind, EventValue, NodeId,
    };
    use tempfile::tempdir;

    use super::{
        HotReloadState, JsxRuntimeLoadOutcome, JsxRuntimeSession, MotionFrame, MotionProperty,
        RenderedJsx,
    };

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
        assert!(rendered.motion.active);
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 0.0);

        let rendered = session.tick_motion(0.0).expect("motion tick should start");
        assert!(rendered.tree.is_none());
        let rendered = session
            .tick_motion(0.5)
            .expect("motion tick should advance");
        assert!(rendered.tree.is_none());
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 0.5);
        assert_motion_close(&rendered.motion, "status", MotionProperty::X, -5.0);
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
        let (_session, rendered) = JsxRuntimeSession::load(&repository_example_path())
            .expect("repository JSX file should render");
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
        assert_eq!(metrics.motion_only_update_count, 1);
    }

    #[test]
    fn timer_driven_async_update_wakes_the_host_and_commits_once() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("async-metrics.tsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useEffect, useState } from "egui";

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

        for _ in 0..10 {
            let (mut session, _rendered) =
                JsxRuntimeSession::load(&entry_path).expect("tsx should transpile and render");
            let _ = session.teardown().expect("teardown should succeed");
            let metrics = session.debug_metrics();
            assert_eq!(metrics.runtime.active_timer_count, 0);
            assert!(!metrics.runtime.pending_host_wake);
        }
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
