use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, bail};
use clay_jsx_runtime::contract::{ContractNode, ContractTree, NodeId};

use super::motion::{MotionFrame, MotionSpec, MotionTickResult, MotionValues};

#[derive(Debug, serde::Deserialize)]
pub struct HostMutationBatch {
    pub version: u32,
    #[serde(default)]
    pub mutations: Vec<HostMutation>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HostMutation {
    ReplaceRoot {
        subtree: ContractNode,
    },
    InsertSubtree {
        parent_id: NodeId,
        index: usize,
        subtree: ContractNode,
    },
    RemoveSubtree {
        node_id: NodeId,
    },
    ReplaceSubtree {
        node_id: NodeId,
        subtree: ContractNode,
    },
    UpdateNode {
        node: ContractNode,
    },
    SetChildren {
        node_id: NodeId,
        child_ids: Vec<NodeId>,
    },
    SetMotion {
        node_id: NodeId,
        motion: MotionSpec,
    },
    ClearMotion {
        node_id: NodeId,
    },
}

impl HostMutation {
    pub fn changes_contract_tree(&self) -> bool {
        !matches!(self, Self::SetMotion { .. } | Self::ClearMotion { .. })
    }
}

#[derive(Debug, Clone, Default)]
pub struct HostTree {
    root_id: Option<NodeId>,
    nodes: BTreeMap<NodeId, HostNode>,
    motion: BTreeMap<NodeId, MotionValues>,
}

#[derive(Debug, Clone)]
struct HostNode {
    parent_id: Option<NodeId>,
    child_ids: Vec<NodeId>,
    node: ContractNode,
}

impl HostTree {
    pub fn has_root(&self) -> bool {
        self.root_id.is_some()
    }

    pub fn apply_mutations(&mut self, mutations: Vec<HostMutation>) -> anyhow::Result<()> {
        if mutations.is_empty() {
            return Ok(());
        }

        let mut root_replacement = None;
        let mut removals = Vec::new();
        let mut replacements = Vec::new();
        let mut inserts = Vec::new();
        let mut updates = Vec::new();
        let mut child_orders = Vec::new();
        let mut motion_updates = Vec::new();
        let mut motion_clears = Vec::new();

        for mutation in mutations {
            match mutation {
                HostMutation::ReplaceRoot { subtree } => {
                    if root_replacement.replace(subtree).is_some() {
                        bail!("replace_root must appear at most once in a commit batch");
                    }
                }
                HostMutation::RemoveSubtree { node_id } => removals.push(node_id),
                HostMutation::ReplaceSubtree { node_id, subtree } => {
                    replacements.push((node_id, subtree));
                }
                HostMutation::InsertSubtree {
                    parent_id,
                    index,
                    subtree,
                } => inserts.push((parent_id, index, subtree)),
                HostMutation::UpdateNode { node } => updates.push(node),
                HostMutation::SetChildren { node_id, child_ids } => {
                    child_orders.push((node_id, child_ids));
                }
                HostMutation::SetMotion { node_id, motion } => {
                    motion_updates.push((node_id, motion));
                }
                HostMutation::ClearMotion { node_id } => {
                    motion_clears.push(node_id);
                }
            }
        }

        if let Some(root_replacement) = root_replacement {
            if !removals.is_empty()
                || !replacements.is_empty()
                || !inserts.is_empty()
                || !updates.is_empty()
                || !child_orders.is_empty()
            {
                bail!(
                    "replace_root cannot be combined with structural node mutations in one commit batch"
                );
            }
            self.replace_root(root_replacement)?;
        } else {
            for node_id in removals {
                self.remove_subtree(&node_id)?;
            }
            for (node_id, subtree) in replacements {
                self.replace_subtree(&node_id, subtree)?;
            }
            for (parent_id, index, subtree) in inserts {
                self.insert_subtree(&parent_id, index, subtree)?;
            }
            for node in updates {
                self.update_node(node)?;
            }
            for (node_id, child_ids) in child_orders {
                self.set_children(&node_id, child_ids)?;
            }
        }

        for node_id in motion_clears {
            self.clear_motion(&node_id)?;
        }
        for (node_id, motion) in motion_updates {
            self.set_motion(node_id, motion)?;
        }

        self.validate()
    }

    pub fn materialize(&self) -> anyhow::Result<ContractTree> {
        debug_assert!(self.validate().is_ok(), "invalid JSX host tree");
        let root_id = self
            .root_id
            .as_ref()
            .ok_or_else(|| anyhow!("JSX runtime has not committed a root tree"))?;
        Ok(ContractTree::new(self.materialize_node(root_id)?))
    }

    pub fn motion_frame(&self) -> MotionFrame {
        MotionFrame {
            values: self.motion.clone(),
            active: false,
        }
    }

    pub fn tick_motion(&mut self, _now_secs: f64) -> MotionTickResult {
        MotionTickResult {
            changed: false,
            frame: self.motion_frame(),
        }
    }

    fn replace_root(&mut self, subtree: ContractNode) -> anyhow::Result<()> {
        let (root_id, nodes) = collect_subtree(subtree, None)?;
        self.root_id = Some(root_id);
        self.nodes = nodes;
        self.motion.clear();
        Ok(())
    }

    fn insert_subtree(
        &mut self,
        parent_id: &NodeId,
        index: usize,
        subtree: ContractNode,
    ) -> anyhow::Result<()> {
        if !self.nodes.contains_key(parent_id) {
            bail!("cannot insert subtree under missing parent {parent_id}");
        }

        let (root_id, new_nodes) = collect_subtree(subtree, Some(parent_id.clone()))?;
        for node_id in new_nodes.keys() {
            if self.nodes.contains_key(node_id) {
                bail!("cannot insert duplicate node id {node_id}");
            }
        }

        let parent = self
            .nodes
            .get_mut(parent_id)
            .ok_or_else(|| anyhow!("cannot insert subtree under missing parent {parent_id}"))?;
        if index > parent.child_ids.len() {
            bail!(
                "cannot insert subtree at index {index} under parent {parent_id}; parent has {} children",
                parent.child_ids.len()
            );
        }

        parent.child_ids.insert(index, root_id);
        self.nodes.extend(new_nodes);
        Ok(())
    }

    fn remove_subtree(&mut self, node_id: &NodeId) -> anyhow::Result<()> {
        if self.root_id.as_ref() == Some(node_id) {
            bail!("cannot remove root node {node_id}; replace_root is required");
        }

        let subtree_ids = self.subtree_ids(node_id)?;
        let parent_id = self
            .nodes
            .get(node_id)
            .and_then(|node| node.parent_id.clone())
            .ok_or_else(|| anyhow!("cannot remove missing node {node_id}"))?;

        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.child_ids.retain(|child_id| child_id != node_id);
        }
        for subtree_id in subtree_ids {
            self.motion.remove(&subtree_id);
            self.nodes.remove(&subtree_id);
        }
        Ok(())
    }

    fn replace_subtree(&mut self, node_id: &NodeId, subtree: ContractNode) -> anyhow::Result<()> {
        if self.root_id.as_ref() == Some(node_id) {
            self.replace_root(subtree)?;
            return Ok(());
        }

        let old_subtree_ids = self.subtree_ids(node_id)?;
        let parent_id = self
            .nodes
            .get(node_id)
            .and_then(|node| node.parent_id.clone())
            .ok_or_else(|| anyhow!("cannot replace missing node {node_id}"))?;
        let index = self
            .nodes
            .get(&parent_id)
            .and_then(|parent| {
                parent
                    .child_ids
                    .iter()
                    .position(|child_id| child_id == node_id)
            })
            .ok_or_else(|| anyhow!("parent {parent_id} does not reference child {node_id}"))?;

        let (new_root_id, new_nodes) = collect_subtree(subtree, Some(parent_id.clone()))?;
        for new_node_id in new_nodes.keys() {
            if self.nodes.contains_key(new_node_id) && !old_subtree_ids.contains(new_node_id) {
                bail!("cannot replace subtree with duplicate node id {new_node_id}");
            }
        }

        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.child_ids.retain(|child_id| child_id != node_id);
        }
        for old_node_id in old_subtree_ids {
            self.motion.remove(&old_node_id);
            self.nodes.remove(&old_node_id);
        }

        let parent = self
            .nodes
            .get_mut(&parent_id)
            .ok_or_else(|| anyhow!("cannot replace subtree under missing parent {parent_id}"))?;
        parent.child_ids.insert(index, new_root_id);
        self.nodes.extend(new_nodes);
        Ok(())
    }

    fn update_node(&mut self, node: ContractNode) -> anyhow::Result<()> {
        let (node, _ignored_children) = strip_contract_children(node);
        let node_id = node.node_id().clone();
        let existing = self
            .nodes
            .get_mut(&node_id)
            .ok_or_else(|| anyhow!("cannot update missing node {node_id}"))?;

        if existing.node.family_id() != node.family_id() {
            bail!(
                "cannot update node {node_id} from family {} to {}; replace_subtree is required",
                existing.node.family_id().as_str(),
                node.family_id().as_str()
            );
        }

        existing.node = node;
        Ok(())
    }

    fn set_children(&mut self, node_id: &NodeId, child_ids: Vec<NodeId>) -> anyhow::Result<()> {
        let existing_children = self
            .nodes
            .get(node_id)
            .ok_or_else(|| anyhow!("cannot order children for missing node {node_id}"))?
            .child_ids
            .clone();

        let expected = id_set(&existing_children)?;
        let requested = id_set(&child_ids)?;
        if expected != requested {
            bail!("set_children for {node_id} must reference the same children after inserts/removals");
        }

        for child_id in &child_ids {
            let child = self.nodes.get(child_id).ok_or_else(|| {
                anyhow!("set_children for {node_id} references missing child {child_id}")
            })?;
            if child.parent_id.as_ref() != Some(node_id) {
                bail!("set_children for {node_id} references non-child node {child_id}");
            }
        }

        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| anyhow!("cannot order children for missing node {node_id}"))?;
        node.child_ids = child_ids;
        Ok(())
    }

    fn set_motion(&mut self, node_id: NodeId, motion: MotionSpec) -> anyhow::Result<()> {
        if !self.nodes.contains_key(&node_id) {
            bail!("cannot set motion for missing node {node_id}");
        }
        self.motion.insert(node_id, motion.animate);
        Ok(())
    }

    fn clear_motion(&mut self, node_id: &NodeId) -> anyhow::Result<()> {
        if !self.nodes.contains_key(node_id) {
            bail!("cannot clear motion for missing node {node_id}");
        }
        self.motion.remove(node_id);
        Ok(())
    }

    fn subtree_ids(&self, node_id: &NodeId) -> anyhow::Result<BTreeSet<NodeId>> {
        if !self.nodes.contains_key(node_id) {
            bail!("missing subtree root {node_id}");
        }
        let mut ids = BTreeSet::new();
        self.collect_subtree_ids(node_id, &mut ids)?;
        Ok(ids)
    }

    fn collect_subtree_ids(
        &self,
        node_id: &NodeId,
        ids: &mut BTreeSet<NodeId>,
    ) -> anyhow::Result<()> {
        if !ids.insert(node_id.clone()) {
            bail!("cycle detected while collecting subtree {node_id}");
        }
        let node = self
            .nodes
            .get(node_id)
            .ok_or_else(|| anyhow!("missing node {node_id}"))?;
        for child_id in &node.child_ids {
            self.collect_subtree_ids(child_id, ids)?;
        }
        Ok(())
    }

    fn materialize_node(&self, node_id: &NodeId) -> anyhow::Result<ContractNode> {
        let host_node = self
            .nodes
            .get(node_id)
            .ok_or_else(|| anyhow!("missing node {node_id}"))?;
        let mut node = host_node.node.clone();
        let children = host_node
            .child_ids
            .iter()
            .map(|child_id| self.materialize_node(child_id))
            .collect::<anyhow::Result<Vec<_>>>()?;
        set_contract_children(&mut node, children)?;
        Ok(node)
    }

    fn validate(&self) -> anyhow::Result<()> {
        let Some(root_id) = &self.root_id else {
            if self.nodes.is_empty() {
                return Ok(());
            }
            bail!("host tree contains nodes without a root");
        };

        let root = self
            .nodes
            .get(root_id)
            .ok_or_else(|| anyhow!("host tree root {root_id} is missing"))?;
        if root.parent_id.is_some() {
            bail!("host tree root {root_id} must not have a parent");
        }

        for (node_id, node) in &self.nodes {
            if node_id != root_id && node.parent_id.is_none() {
                bail!("non-root host node {node_id} is missing a parent");
            }
            if !node.child_ids.is_empty() && !supports_contract_children(&node.node) {
                bail!(
                    "host node {node_id} of family {} cannot contain contract children",
                    node.node.family_id().as_str()
                );
            }
            let mut seen_children = BTreeSet::new();
            for child_id in &node.child_ids {
                if !seen_children.insert(child_id) {
                    bail!("host node {node_id} references child {child_id} more than once");
                }
                let child = self.nodes.get(child_id).ok_or_else(|| {
                    anyhow!("host node {node_id} references missing child {child_id}")
                })?;
                if child.parent_id.as_ref() != Some(node_id) {
                    bail!("host node {node_id} references non-child node {child_id}");
                }
            }
        }

        for node_id in self.motion.keys() {
            if !self.nodes.contains_key(node_id) {
                bail!("host motion references missing node {node_id}");
            }
        }

        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        self.validate_reachable(root_id, &mut visiting, &mut visited)?;
        if visited.len() != self.nodes.len() {
            bail!(
                "host tree has {} unreachable nodes",
                self.nodes.len() - visited.len()
            );
        }
        Ok(())
    }

    fn validate_reachable(
        &self,
        node_id: &NodeId,
        visiting: &mut BTreeSet<NodeId>,
        visited: &mut BTreeSet<NodeId>,
    ) -> anyhow::Result<()> {
        if visited.contains(node_id) {
            return Ok(());
        }
        if !visiting.insert(node_id.clone()) {
            bail!("host tree contains a cycle at node {node_id}");
        }
        let node = self
            .nodes
            .get(node_id)
            .ok_or_else(|| anyhow!("host tree references missing node {node_id}"))?;
        for child_id in &node.child_ids {
            self.validate_reachable(child_id, visiting, visited)?;
        }
        visiting.remove(node_id);
        visited.insert(node_id.clone());
        Ok(())
    }
}

fn collect_subtree(
    node: ContractNode,
    parent_id: Option<NodeId>,
) -> anyhow::Result<(NodeId, BTreeMap<NodeId, HostNode>)> {
    let mut nodes = BTreeMap::new();
    let root_id = collect_subtree_node(node, parent_id, &mut nodes)?;
    Ok((root_id, nodes))
}

fn collect_subtree_node(
    node: ContractNode,
    parent_id: Option<NodeId>,
    nodes: &mut BTreeMap<NodeId, HostNode>,
) -> anyhow::Result<NodeId> {
    let (node, children) = strip_contract_children(node);
    let node_id = node.node_id().clone();
    if nodes.contains_key(&node_id) {
        bail!("duplicate node id {node_id} in committed subtree");
    }

    let mut child_ids = Vec::with_capacity(children.len());
    for child in children {
        child_ids.push(collect_subtree_node(child, Some(node_id.clone()), nodes)?);
    }

    nodes.insert(
        node_id.clone(),
        HostNode {
            parent_id,
            child_ids,
            node,
        },
    );
    Ok(node_id)
}

fn id_set(ids: &[NodeId]) -> anyhow::Result<BTreeSet<NodeId>> {
    let mut out = BTreeSet::new();
    for id in ids {
        if !out.insert(id.clone()) {
            bail!("duplicate child id {id}");
        }
    }
    Ok(out)
}

fn strip_contract_children(mut node: ContractNode) -> (ContractNode, Vec<ContractNode>) {
    let children = match &mut node {
        ContractNode::Row(props) => std::mem::take(&mut props.children),
        ContractNode::Column(props) => std::mem::take(&mut props.children),
        ContractNode::Inset(props) => std::mem::take(&mut props.children),
        ContractNode::SizedBox(props) => std::mem::take(&mut props.children),
        ContractNode::Card(props) => std::mem::take(&mut props.children),
        ContractNode::Sidebar(props) => std::mem::take(&mut props.children),
        ContractNode::Toolbar(props) => std::mem::take(&mut props.children),
        ContractNode::Collapsible(props) => std::mem::take(&mut props.children),
        ContractNode::DialogueModal(props) => std::mem::take(&mut props.children),
        ContractNode::Popover(props) => std::mem::take(&mut props.children),
        ContractNode::ContextMenu(props) => std::mem::take(&mut props.children),
        ContractNode::AudioPlayback(props) => std::mem::take(&mut props.children),
        ContractNode::ImageTile(props) => std::mem::take(&mut props.children),
        ContractNode::Spacer(_)
        | ContractNode::MenuBar(_)
        | ContractNode::Tabs(_)
        | ContractNode::Label(_)
        | ContractNode::Button(_)
        | ContractNode::ButtonGroup(_)
        | ContractNode::Input(_)
        | ContractNode::NumberInput(_)
        | ContractNode::Checkbox(_)
        | ContractNode::Switch(_)
        | ContractNode::Select(_)
        | ContractNode::Field(_)
        | ContractNode::Separator(_)
        | ContractNode::Hierarchy(_)
        | ContractNode::Spinner(_)
        | ContractNode::Progress(_)
        | ContractNode::ToastViewport(_)
        | ContractNode::Color(_)
        | ContractNode::Icon(_)
        | ContractNode::Image(_)
        | ContractNode::Twemoji(_)
        | ContractNode::Kbd(_)
        | ContractNode::Skeleton(_)
        | ContractNode::Slider(_)
        | ContractNode::Radio(_)
        | ContractNode::RadioGroup(_)
        | ContractNode::Combobox(_)
        | ContractNode::EmojiSelector(_)
        | ContractNode::Pagination(_)
        | ContractNode::Tooltip(_)
        | ContractNode::DropdownMenu(_)
        | ContractNode::OpenWith(_)
        | ContractNode::CollabCursor(_)
        | ContractNode::IconToolbar(_)
        | ContractNode::FileTree(_)
        | ContractNode::DragBoard(_)
        | ContractNode::Command(_) => Vec::new(),
    };
    (node, children)
}

fn set_contract_children(
    node: &mut ContractNode,
    children: Vec<ContractNode>,
) -> anyhow::Result<()> {
    if children.is_empty() {
        return Ok(());
    }

    match node {
        ContractNode::Row(props) => props.children = children,
        ContractNode::Column(props) => props.children = children,
        ContractNode::Inset(props) => props.children = children,
        ContractNode::SizedBox(props) => props.children = children,
        ContractNode::Card(props) => props.children = children,
        ContractNode::Sidebar(props) => props.children = children,
        ContractNode::Toolbar(props) => props.children = children,
        ContractNode::Collapsible(props) => props.children = children,
        ContractNode::DialogueModal(props) => props.children = children,
        ContractNode::Popover(props) => props.children = children,
        ContractNode::ContextMenu(props) => props.children = children,
        ContractNode::AudioPlayback(props) => props.children = children,
        ContractNode::ImageTile(props) => props.children = children,
        node => {
            bail!(
                "family {} cannot contain contract children",
                node.family_id().as_str()
            );
        }
    }
    Ok(())
}

fn supports_contract_children(node: &ContractNode) -> bool {
    matches!(
        node,
        ContractNode::Row(_)
            | ContractNode::Column(_)
            | ContractNode::Inset(_)
            | ContractNode::SizedBox(_)
            | ContractNode::Card(_)
            | ContractNode::Sidebar(_)
            | ContractNode::Toolbar(_)
            | ContractNode::Collapsible(_)
            | ContractNode::DialogueModal(_)
            | ContractNode::Popover(_)
            | ContractNode::ContextMenu(_)
            | ContractNode::AudioPlayback(_)
            | ContractNode::ImageTile(_)
    )
}

#[cfg(test)]
mod tests {
    use super::{HostMutation, HostTree};
    use clay_jsx_runtime::contract::{ContractNode, NodeId};
    use serde_json::json;

    #[test]
    fn replace_root_materializes_tree() {
        let mut tree = HostTree::default();

        tree.apply_mutations(vec![HostMutation::ReplaceRoot {
            subtree: column("root", vec![label("a", "A"), label("b", "B")]),
        }])
        .expect("replace root should apply");

        let root = tree.materialize().expect("tree should materialize").root;
        assert_eq!(root.node_id().as_str(), "root");
        assert_eq!(child_ids(&root), vec!["a", "b"]);
    }

    #[test]
    fn update_node_changes_leaf_props_without_replacing_siblings() {
        let mut tree = HostTree::default();
        tree.apply_mutations(vec![HostMutation::ReplaceRoot {
            subtree: column("root", vec![label("a", "old"), label("b", "B")]),
        }])
        .expect("replace root should apply");

        tree.apply_mutations(vec![HostMutation::UpdateNode {
            node: label("a", "new"),
        }])
        .expect("update should apply");

        let root = tree.materialize().expect("tree should materialize").root;
        let children = children(&root);
        assert_eq!(label_text(&children[0]), "new");
        assert_eq!(label_text(&children[1]), "B");
    }

    #[test]
    fn insert_remove_and_reorder_children_preserve_topology() {
        let mut tree = HostTree::default();
        tree.apply_mutations(vec![HostMutation::ReplaceRoot {
            subtree: column("root", vec![label("a", "A"), label("c", "C")]),
        }])
        .expect("replace root should apply");

        tree.apply_mutations(vec![
            HostMutation::InsertSubtree {
                parent_id: NodeId::from("root"),
                index: 1,
                subtree: label("b", "B"),
            },
            HostMutation::SetChildren {
                node_id: NodeId::from("root"),
                child_ids: vec![NodeId::from("a"), NodeId::from("b"), NodeId::from("c")],
            },
        ])
        .expect("insert should apply");
        assert_eq!(
            child_ids(&tree.materialize().expect("tree should materialize").root),
            vec!["a", "b", "c"]
        );

        tree.apply_mutations(vec![
            HostMutation::RemoveSubtree {
                node_id: NodeId::from("a"),
            },
            HostMutation::SetChildren {
                node_id: NodeId::from("root"),
                child_ids: vec![NodeId::from("c"), NodeId::from("b")],
            },
        ])
        .expect("remove and reorder should apply");
        assert_eq!(
            child_ids(&tree.materialize().expect("tree should materialize").root),
            vec!["c", "b"]
        );
    }

    #[test]
    fn invalid_duplicate_ids_are_rejected() {
        let mut tree = HostTree::default();

        let result = tree.apply_mutations(vec![HostMutation::ReplaceRoot {
            subtree: column("root", vec![label("a", "A"), label("a", "duplicate")]),
        }]);

        assert!(result.is_err());
    }

    #[test]
    fn invalid_missing_parent_and_child_orders_are_rejected() {
        let mut tree = HostTree::default();
        tree.apply_mutations(vec![HostMutation::ReplaceRoot {
            subtree: column("root", vec![label("a", "A")]),
        }])
        .expect("replace root should apply");

        assert!(tree
            .apply_mutations(vec![HostMutation::InsertSubtree {
                parent_id: NodeId::from("missing"),
                index: 0,
                subtree: label("b", "B"),
            }])
            .is_err());
        assert!(tree
            .apply_mutations(vec![HostMutation::SetChildren {
                node_id: NodeId::from("root"),
                child_ids: vec![NodeId::from("root")],
            }])
            .is_err());
    }

    fn column(node_id: &str, children: Vec<ContractNode>) -> ContractNode {
        serde_json::from_value(json!({
            "family": "column",
            "node_id": node_id,
            "children": children,
        }))
        .expect("column should deserialize")
    }

    fn label(node_id: &str, text: &str) -> ContractNode {
        serde_json::from_value(json!({
            "family": "label",
            "node_id": node_id,
            "text": text,
        }))
        .expect("label should deserialize")
    }

    fn children(node: &ContractNode) -> &[ContractNode] {
        match node {
            ContractNode::Column(props) => &props.children,
            _ => panic!("expected column node"),
        }
    }

    fn child_ids(node: &ContractNode) -> Vec<&str> {
        children(node)
            .iter()
            .map(|child| child.node_id().as_str())
            .collect()
    }

    fn label_text(node: &ContractNode) -> &str {
        match node {
            ContractNode::Label(props) => props.text.as_str(),
            _ => panic!("expected label node"),
        }
    }
}
