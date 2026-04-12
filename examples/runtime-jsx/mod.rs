mod app;

use std::path::PathBuf;

use egui::{Context, ViewportBuilder};
use egui_component::theme::{self, BaseColor, ThemeMode, ThemeSpec};

pub use app::RuntimeJsxApp;

pub const WINDOW_TITLE: &str = "egui-component JSX Runtime";
pub const WINDOW_INNER_SIZE: [f32; 2] = [1360.0, 940.0];

pub fn install_context(ctx: &Context) {
    theme::install(
        ctx,
        ThemeSpec::preset(BaseColor::Neutral),
        ThemeMode::System,
    );
}

pub fn default_entry_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("runtime-jsx")
        .join("app.jsx")
}

pub fn entry_path_from_args() -> PathBuf {
    std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_entry_path)
}

pub fn run_from_args() -> eframe::Result {
    run_native(entry_path_from_args())
}

pub fn run_native(entry_path: PathBuf) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size(WINDOW_INNER_SIZE),
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_TITLE,
        options,
        Box::new(move |creation_context| {
            install_context(&creation_context.egui_ctx);
            Ok(Box::new(RuntimeJsxApp::new(entry_path)))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::default_entry_path;
    use clay_jsx_egui_bridge::{JsxRuntimeSession, MotionFrame, MotionProperty};
    use egui_component::contract::{ContractEvent, ContractNode, EventKind, EventValue, NodeId};
    use tempfile::tempdir;

    #[test]
    fn default_jsx_file_renders_contract_tree() {
        let (_session, rendered) =
            JsxRuntimeSession::load(&default_entry_path()).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(tree.root.family_id().as_str(), "column");
    }

    #[test]
    fn dispatches_egui_events_back_into_jsx_hooks() {
        let (mut session, rendered) =
            JsxRuntimeSession::load(&default_entry_path()).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "snap-checkbox"), Some(true));

        let event = ContractEvent::new("snap-checkbox", EventKind::Toggled)
            .value(Some(EventValue::Boolean(false)));
        let rendered = session
            .dispatch_events(&[event])
            .expect("event dispatch should rerender");
        let tree = rendered.tree.expect("changed render should return a tree");

        assert_eq!(checkbox_value(&tree.root, "snap-checkbox"), Some(false));
    }

    #[test]
    fn no_op_rerender_skips_contract_tree_materialization() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("noop.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [checked, setChecked] = useState(false);
  return (
    <column id="root">
      <checkbox id="toggle" value={checked} onToggle={(event, value) => setChecked(value)} />
    </column>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(false));

        let event = ContractEvent::new("toggle", EventKind::Toggled)
            .value(Some(EventValue::Boolean(false)));
        let rendered = session
            .dispatch_events(&[event])
            .expect("event dispatch should return current tree");

        assert!(rendered.tree.is_none());
    }

    #[test]
    fn dynamic_jsx_lists_apply_incremental_insert_remove_and_reorder() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("dynamic-list.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [items, setItems] = useState(["a", "c"]);
  return (
    <column id="root">
      <button id="change" label="Change" onClick={() => setItems(["c", "b"])} />
      {items.map((item) => <label key={item} id={item} text={item} />)}
    </column>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            contract_children(&tree.root)
                .iter()
                .map(|child| child.node_id().as_str())
                .collect::<Vec<_>>(),
            vec!["change", "a", "c"]
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("change", EventKind::Clicked)])
            .expect("event dispatch should rerender dynamic list");
        let tree = rendered.tree.expect("changed render should return a tree");

        assert_eq!(
            contract_children(&tree.root)
                .iter()
                .map(|child| child.node_id().as_str())
                .collect::<Vec<_>>(),
            vec!["change", "c", "b"]
        );
    }

    #[test]
    fn stable_jsx_node_ids_can_move_between_parents() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("move-between-parents.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [right, setRight] = useState(false);
  return (
    <row id="root">
      <column id="left">{!right && <label id="moving" text="Moving" />}</column>
      <column id="right">{right && <label id="moving" text="Moving" />}</column>
      <button id="move" label="Move" onClick={() => setRight(true)} />
    </row>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            contract_children(find_node(&tree.root, "left").expect("left should exist"))
                .iter()
                .map(|child| child.node_id().as_str())
                .collect::<Vec<_>>(),
            vec!["moving"]
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("move", EventKind::Clicked)])
            .expect("event dispatch should move child");
        let tree = rendered.tree.expect("changed render should return a tree");

        assert!(
            contract_children(find_node(&tree.root, "left").expect("left should exist")).is_empty()
        );
        assert_eq!(
            contract_children(find_node(&tree.root, "right").expect("right should exist"))
                .iter()
                .map(|child| child.node_id().as_str())
                .collect::<Vec<_>>(),
            vec!["moving"]
        );
    }

    #[test]
    fn prop_updates_change_one_leaf_without_remounting_siblings() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("prop-update.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [text, setText] = useState("old");
  return (
    <column id="root">
      <button id="change" label="Change" onClick={() => setText("new")} />
      <label id="target" text={text} />
      <label id="sibling" text="stable" />
    </column>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "target").unwrap()),
            Some("old")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "sibling").unwrap()),
            Some("stable")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("change", EventKind::Clicked)])
            .expect("event dispatch should update label");
        let tree = rendered.tree.expect("changed render should return a tree");

        assert_eq!(
            label_text(find_node(&tree.root, "target").unwrap()),
            Some("new")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "sibling").unwrap()),
            Some("stable")
        );
    }

    #[test]
    fn removed_node_handlers_are_not_dispatched() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("handler-cleanup.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [visible, setVisible] = useState(true);
  const [count, setCount] = useState(0);
  return (
    <column id="root">
      <button id="remove" label="Remove" onClick={() => setVisible(false)} />
      {visible && <button id="target" label="Target" onClick={() => setCount(count + 1)} />}
      <label id="count" text={String(count)} />
    </column>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert!(find_node(&tree.root, "target").is_some());
        assert_eq!(
            label_text(find_node(&tree.root, "count").unwrap()),
            Some("0")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("remove", EventKind::Clicked)])
            .expect("remove event should rerender");
        let tree = rendered.tree.expect("changed render should return a tree");
        assert!(find_node(&tree.root, "target").is_none());
        assert_eq!(
            label_text(find_node(&tree.root, "count").unwrap()),
            Some("0")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("target", EventKind::Clicked)])
            .expect("stale target event should be ignored");
        assert!(rendered.tree.is_none());
    }

    #[test]
    fn motion_react_import_commits_retained_motion_values() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("motion.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";
import { motion } from "motion/react";

function App() {
  return (
    <motion.card
      id="panel"
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 1, ease: "linear" }}
    >
      <label id="copy" text="Animated" />
    </motion.card>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        assert_eq!(
            rendered
                .tree
                .expect("initial render should return a tree")
                .root
                .family_id()
                .as_str(),
            "card"
        );
        assert!(rendered.motion.active);
        assert_motion_close(&rendered.motion, "panel", MotionProperty::Opacity, 0.0);
        assert_motion_close(&rendered.motion, "panel", MotionProperty::X, -10.0);

        let rendered = session
            .tick_motion(0.0)
            .expect("motion tick should succeed");
        assert!(rendered.tree.is_none());
        assert_motion_close(&rendered.motion, "panel", MotionProperty::Opacity, 0.0);

        let rendered = session
            .tick_motion(0.5)
            .expect("motion tick should advance");
        assert!(rendered.tree.is_none());
        assert!(rendered.motion.active);
        assert_motion_close(&rendered.motion, "panel", MotionProperty::Opacity, 0.5);
        assert_motion_close(&rendered.motion, "panel", MotionProperty::X, -5.0);

        let rendered = session.tick_motion(1.0).expect("motion tick should finish");
        assert!(!rendered.motion.active);
        assert_motion_close(&rendered.motion, "panel", MotionProperty::Opacity, 1.0);
        assert_motion_close(&rendered.motion, "panel", MotionProperty::X, 0.0);
    }

    #[test]
    fn react_motion_alias_updates_motion_without_contract_tree_materialization() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("motion-update.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";
import { motion } from "react/motion";

function App() {
  const [open, setOpen] = useState(false);
  return (
    <column id="root">
      <button id="toggle" label="Toggle" onClick={() => setOpen(true)} />
      <motion.label
        id="status"
        text="Status"
        animate={{ opacity: open ? 1 : 0 }}
        transition={{ duration: 1, ease: "linear" }}
      />
    </column>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        assert!(!rendered.motion.active);
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 0.0);

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle", EventKind::Clicked)])
            .expect("event dispatch should retarget motion");
        assert!(rendered.tree.is_none());
        assert!(rendered.motion.active);
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 0.0);

        let rendered = session.tick_motion(0.0).expect("motion tick should start");
        assert!(rendered.tree.is_none());
        let rendered = session
            .tick_motion(0.25)
            .expect("motion tick should advance");
        assert!(rendered.tree.is_none());
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 0.25);
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

    fn label_text(node: &ContractNode) -> Option<&str> {
        match node {
            ContractNode::Label(props) => Some(props.text.as_str()),
            _ => None,
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
