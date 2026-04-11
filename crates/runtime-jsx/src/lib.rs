pub mod diagnostics;
pub mod host_tree;
pub mod motion;
mod runtime;

pub use diagnostics::{
    extend_logs, push_log, RuntimeLogBuffer, LOG_HISTORY_LIMIT, LOG_MESSAGE_LIMIT_BYTES,
};
pub use host_tree::{HostMutation, HostMutationBatch, HostTree};
pub use motion::{
    MotionEase, MotionFrame, MotionProperty, MotionSpec, MotionTickResult, MotionTransition,
    MotionValues,
};
pub use runtime::{JsxRuntimeSession, RenderedJsx};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use egui_component::contract::{ContractEvent, ContractNode, EventKind, EventValue, NodeId};
    use tempfile::tempdir;

    use super::{JsxRuntimeSession, MotionFrame, MotionProperty};

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
    <column id="root">
      <checkbox
        id="toggle"
        value={checked}
        onToggle={(event) => setChecked(Boolean(eventValue(event)))}
      />
    </column>
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
    <column id="root">
      <button id="toggle" label="Toggle" onClick={() => setOpen(true)} />
      <motion.label
        id="status"
        text="Status"
        animate={{ opacity: open ? 1 : 0, x: open ? 0 : -10 }}
        transition={{ duration: 1, ease: "linear" }}
      />
    </column>
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
    fn repository_jsx_example_still_loads_through_the_runtime_crate() {
        let (_session, rendered) = JsxRuntimeSession::load(&repository_example_path())
            .expect("repository JSX file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(tree.root.family_id().as_str(), "column");
    }

    fn repository_example_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join("runtime-jsx")
            .join("app.jsx")
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
