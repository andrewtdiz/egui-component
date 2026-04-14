#![doc = "egui-component contract bridge for clay-jsx-runtime TS/TSX hosts."]

pub mod host_tree;
pub mod motion;
mod runtime;

pub const EGUI_MODULE_SOURCE: &str = include_str!("mod.js");
pub const EGUI_JSX_RUNTIME_SOURCE: &str = include_str!("runtime_api.js");
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
    HotReloadState, JsxRuntimeLoadFailure, JsxRuntimeLoadOutcome, JsxRuntimeSession, RenderedJsx,
};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use egui_component::contract::{
        ContractEvent, ContractLength, ContractNode, EventKind, EventValue, NodeId,
    };
    use tempfile::tempdir;

    use super::{JsxRuntimeLoadOutcome, JsxRuntimeSession, MotionFrame, MotionProperty};

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
    fn reload_with_hot_state_preserves_serializable_use_state_across_import_updates() {
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
        let (_reloaded_session, rendered) =
            JsxRuntimeSession::load_with_hot_reload_state(&entry_path, Some(&hot_reload_state))
                .expect("tsx should reload with restored state");
        let tree = rendered.tree.expect("reloaded render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(true));
        assert_eq!(
            label_text(find_node(&tree.root, "status").expect("status label should exist")),
            Some("On")
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
    fn hot_reload_snapshot_only_includes_currently_rendered_components() {
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
        let hook_keys = hot_reload_state
            .hook_state
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(hook_keys, vec!["root:App".to_owned()]);
    }

    #[test]
    fn hot_reload_snapshot_drops_components_with_non_serializable_use_state() {
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

        let hot_reload_state = session
            .capture_hot_reload_state()
            .expect("hot reload state should be captured");
        assert!(
            hot_reload_state.hook_state.is_empty(),
            "non-serializable components should be omitted from hot reload snapshots"
        );

        let (_reloaded_session, rendered) =
            JsxRuntimeSession::load_with_hot_reload_state(&entry_path, Some(&hot_reload_state))
                .expect("tsx should reload with dropped non-serializable state");
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
