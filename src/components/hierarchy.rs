use super::api::ComponentUi;
use crate::ui::{icons, tokens, twemoji, typography};
use egui::{
    vec2, Align2, Color32, CornerRadius, CursorIcon, Id, Rect, Response, Sense, Stroke, StrokeKind,
    Ui,
};

const HIERARCHY_DEFAULT_WIDTH: f32 = 320.0;
const HIERARCHY_ROW_HEIGHT: f32 = 32.0;
const HIERARCHY_INDENT_WIDTH: f32 = 18.0;
const HIERARCHY_PANEL_PADDING: i8 = 6;
const HIERARCHY_ROW_PADDING_X: f32 = 8.0;
const HIERARCHY_ICON_SIZE: f32 = 16.0;
const HIERARCHY_ACTION_ICON_SIZE: f32 = 15.0;
const HIERARCHY_DISCLOSURE_SIZE: f32 = 14.0;
const HIERARCHY_DROP_ZONE_HEIGHT: f32 = 6.0;
const HIERARCHY_SELECTION_LINE: Color32 = Color32::from_rgb(138, 182, 255);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HierarchyItemKind {
    Folder,
    Frame,
    Group,
    Player,
    Weapon,
    Clothing,
    Hitbox,
    Vector,
}

impl HierarchyItemKind {
    fn icon(self) -> HierarchyItemIcon {
        match self {
            Self::Folder => HierarchyItemIcon::Icon("bootstrap:folder-fill"),
            Self::Frame => HierarchyItemIcon::Twemoji("🧩"),
            Self::Group => HierarchyItemIcon::Twemoji("⚙️"),
            Self::Player => HierarchyItemIcon::Twemoji("🧍"),
            Self::Weapon => HierarchyItemIcon::Twemoji("⚔️"),
            Self::Clothing => HierarchyItemIcon::Twemoji("👕"),
            Self::Hitbox => HierarchyItemIcon::Twemoji("🎯"),
            Self::Vector => HierarchyItemIcon::Twemoji("🔷"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum HierarchyItemIcon {
    Icon(&'static str),
    Twemoji(&'static str),
}

#[derive(Debug, Clone)]
pub struct HierarchyNode<'a> {
    pub id: usize,
    pub label: &'a str,
    pub kind: HierarchyItemKind,
    pub expanded: bool,
    pub locked: bool,
    pub children: Vec<HierarchyNode<'a>>,
}

impl<'a> HierarchyNode<'a> {
    pub fn new(id: usize, label: &'a str, kind: HierarchyItemKind) -> Self {
        Self {
            id,
            label,
            kind,
            expanded: true,
            locked: false,
            children: Vec::new(),
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    pub fn children(mut self, children: Vec<HierarchyNode<'a>>) -> Self {
        self.children = children;
        self
    }
}

#[derive(Debug)]
pub struct Hierarchy<'a, 'nodes> {
    pub id: Id,
    pub nodes: &'nodes mut Vec<HierarchyNode<'a>>,
    pub width: f32,
    pub row_height: f32,
    pub indent_width: f32,
}

impl<'a, 'nodes> Hierarchy<'a, 'nodes> {
    pub fn new(id: Id, nodes: &'nodes mut Vec<HierarchyNode<'a>>) -> Self {
        Self {
            id,
            nodes,
            width: HIERARCHY_DEFAULT_WIDTH,
            row_height: HIERARCHY_ROW_HEIGHT,
            indent_width: HIERARCHY_INDENT_WIDTH,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(160.0);
        self
    }

    pub fn row_height(mut self, row_height: f32) -> Self {
        self.row_height = row_height.max(24.0);
        self
    }

    pub fn indent_width(mut self, indent_width: f32) -> Self {
        self.indent_width = indent_width.max(8.0);
        self
    }
}

impl ComponentUi<'_> {
    pub fn hierarchy<'a, 'nodes>(
        &mut self,
        selected_id: &mut Option<usize>,
        props: Hierarchy<'a, 'nodes>,
    ) -> Response {
        draw_hierarchy(self.ui_mut(), selected_id, props)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct HierarchyDragPayload {
    hierarchy_id: Id,
    parent_id: Option<usize>,
    node_id: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct PendingMove {
    parent_id: Option<usize>,
    node_id: usize,
    target_index: usize,
}

#[derive(Debug, Clone)]
struct RowOutcome {
    response: Response,
    has_children: bool,
    node_id: usize,
    expanded: bool,
}

#[derive(Debug, Clone)]
struct SelectionRange {
    ids: Vec<usize>,
    first_id: usize,
    last_id: usize,
}

fn draw_hierarchy(
    ui: &mut Ui,
    selected_id: &mut Option<usize>,
    props: Hierarchy<'_, '_>,
) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let mut pending_move = None;
    let mut changed = false;
    let mut pointer_over_interaction = false;
    let selection_range = selection_range(props.nodes, *selected_id);
    let frame = egui::Frame::new()
        .fill(tokens::muted_surface(runtime))
        .stroke(Stroke::new(1.0, tokens::separator(runtime)))
        .corner_radius(CornerRadius::same(tokens::radius_lg(runtime)))
        .inner_margin(egui::Margin::same(HIERARCHY_PANEL_PADDING));

    let mut response = frame
        .show(ui, |ui| {
            ui.set_width(props.width.min(ui.available_width().max(160.0)));
            let _ = ui.scope(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                draw_hierarchy_list(
                    ui,
                    props.id,
                    props.nodes,
                    None,
                    0,
                    selected_id,
                    selection_range.as_ref(),
                    runtime,
                    props.row_height,
                    props.indent_width,
                    &mut pending_move,
                    &mut changed,
                    &mut pointer_over_interaction,
                );
            });
        })
        .response;

    if let Some(move_request) = pending_move {
        if move_hierarchy_node_within_parent(
            props.nodes,
            move_request.parent_id,
            move_request.node_id,
            move_request.target_index,
        ) {
            *selected_id = Some(move_request.node_id);
            changed = true;
        }
    }

    let pointer_clicked_background = ui.input(|input| input.pointer.any_click());
    let pointer_pos = ui.ctx().pointer_interact_pos();
    if pointer_clicked_background
        && !pointer_over_interaction
        && pointer_pos.is_some_and(|pointer| response.rect.contains(pointer))
        && selected_id.take().is_some()
    {
        changed = true;
    }

    if changed {
        response.mark_changed();
    }
    response
}

#[allow(clippy::too_many_arguments)]
fn draw_hierarchy_list(
    ui: &mut Ui,
    hierarchy_id: Id,
    nodes: &mut Vec<HierarchyNode<'_>>,
    parent_id: Option<usize>,
    depth: usize,
    selected_id: &mut Option<usize>,
    selection_range: Option<&SelectionRange>,
    runtime: crate::theme::ThemeRuntime,
    row_height: f32,
    indent_width: f32,
    pending_move: &mut Option<PendingMove>,
    changed: &mut bool,
    pointer_over_interaction: &mut bool,
) {
    let sibling_count = nodes.len();
    for row_index in 0..sibling_count {
        let row_outcome = {
            let node = &mut nodes[row_index];
            draw_hierarchy_row(
                ui,
                hierarchy_id,
                parent_id,
                depth,
                node,
                selected_id,
                selection_range,
                runtime,
                row_height,
                indent_width,
                changed,
                pointer_over_interaction,
            )
        };

        if let Some(move_request) = hierarchy_pending_move(
            ui,
            &row_outcome.response,
            hierarchy_id,
            parent_id,
            row_outcome.node_id,
            row_index,
        ) {
            *pending_move = Some(move_request);
        }

        if row_outcome.has_children && row_outcome.expanded {
            let node = &mut nodes[row_index];
            draw_hierarchy_list(
                ui,
                hierarchy_id,
                &mut node.children,
                Some(node.id),
                depth + 1,
                selected_id,
                selection_range,
                runtime,
                row_height,
                indent_width,
                pending_move,
                changed,
                pointer_over_interaction,
            );
        }
    }

    let (drop_rect, drop_response) = ui.allocate_exact_size(
        vec2(ui.available_width(), HIERARCHY_DROP_ZONE_HEIGHT),
        Sense::hover(),
    );
    if let Some(payload) =
        hovered_hierarchy_payload(ui.ctx(), &drop_response, hierarchy_id).filter(|payload| {
            payload.parent_id == parent_id && !nodes.iter().any(|node| node.id == payload.node_id)
        })
    {
        ui.painter().line_segment(
            [
                egui::pos2(drop_rect.left() + 8.0, drop_rect.center().y),
                egui::pos2(drop_rect.right() - 8.0, drop_rect.center().y),
            ],
            Stroke::new(2.0, HIERARCHY_SELECTION_LINE),
        );
        if released_hierarchy_payload(ui.ctx(), &drop_response, hierarchy_id).is_some() {
            *pending_move = Some(PendingMove {
                parent_id,
                node_id: payload.node_id,
                target_index: nodes.len(),
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_hierarchy_row(
    ui: &mut Ui,
    hierarchy_id: Id,
    parent_id: Option<usize>,
    depth: usize,
    node: &mut HierarchyNode<'_>,
    selected_id: &mut Option<usize>,
    selection_range: Option<&SelectionRange>,
    runtime: crate::theme::ThemeRuntime,
    row_height: f32,
    indent_width: f32,
    changed: &mut bool,
    pointer_over_interaction: &mut bool,
) -> RowOutcome {
    let row_width = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(vec2(row_width, row_height), Sense::click_and_drag());
    let selected_exact = *selected_id == Some(node.id);
    let selected_subtree = selection_range
        .as_ref()
        .is_some_and(|range| range.ids.contains(&node.id));
    let round_top = selection_range
        .as_ref()
        .is_some_and(|range| range.first_id == node.id);
    let round_bottom = selection_range
        .as_ref()
        .is_some_and(|range| range.last_id == node.id);
    let hovered = response.hovered();
    let shows_child_lock = parent_id.is_some();
    let hover_fill = tokens::button_secondary_hover_bg(runtime).linear_multiply(0.72);
    let fill = if selected_exact {
        hierarchy_selection_strong_fill()
    } else if selected_subtree {
        hierarchy_selection_fill()
    } else if hovered {
        hover_fill
    } else {
        Color32::TRANSPARENT
    };
    let stroke = if selected_exact {
        Stroke::new(1.0, hierarchy_selection_stroke())
    } else {
        Stroke::NONE
    };

    paint_hierarchy_row(
        ui,
        rect,
        depth,
        node,
        runtime,
        fill,
        stroke,
        hovered,
        shows_child_lock,
        indent_width,
        selected_exact,
        round_top,
        round_bottom,
    );

    let payload = HierarchyDragPayload {
        hierarchy_id,
        parent_id,
        node_id: node.id,
    };
    response.dnd_set_drag_payload(payload);

    let left_offset = HIERARCHY_ROW_PADDING_X + depth as f32 * indent_width;
    let disclosure_rect = Rect::from_center_size(
        egui::pos2(
            rect.left() + left_offset + HIERARCHY_DISCLOSURE_SIZE * 0.5,
            rect.center().y,
        ),
        vec2(HIERARCHY_DISCLOSURE_SIZE, HIERARCHY_DISCLOSURE_SIZE),
    );
    let actions_right = rect.right() - HIERARCHY_ROW_PADDING_X;
    let lock_rect = Rect::from_center_size(
        egui::pos2(
            actions_right - HIERARCHY_ACTION_ICON_SIZE * 0.5,
            rect.center().y,
        ),
        vec2(HIERARCHY_ACTION_ICON_SIZE, HIERARCHY_ACTION_ICON_SIZE),
    );

    let disclosure_response = if node.children.is_empty() {
        None
    } else {
        Some(ui.interact(
            disclosure_rect,
            ui.id().with(("hierarchy_disclosure", node.id)),
            Sense::click(),
        ))
    };
    let lock_response = shows_child_lock.then(|| {
        ui.interact(
            lock_rect,
            ui.id().with(("hierarchy_lock", node.id)),
            Sense::click(),
        )
    });

    *pointer_over_interaction |= response.contains_pointer()
        || disclosure_response
            .as_ref()
            .is_some_and(Response::contains_pointer)
        || lock_response
            .as_ref()
            .is_some_and(Response::contains_pointer);

    if disclosure_response.as_ref().is_some_and(Response::clicked) {
        node.expanded = !node.expanded;
        *changed = true;
    } else if lock_response.as_ref().is_some_and(Response::clicked) {
        node.locked = !node.locked;
        *changed = true;
    } else if response.clicked() {
        if *selected_id != Some(node.id) {
            *selected_id = Some(node.id);
            *changed = true;
        }
    }

    RowOutcome {
        response: response.on_hover_cursor(CursorIcon::Grab),
        has_children: !node.children.is_empty(),
        node_id: node.id,
        expanded: node.expanded,
    }
}

fn paint_hierarchy_row(
    ui: &mut Ui,
    rect: Rect,
    depth: usize,
    node: &HierarchyNode<'_>,
    runtime: crate::theme::ThemeRuntime,
    fill: Color32,
    stroke: Stroke,
    hovered: bool,
    shows_child_lock: bool,
    indent_width: f32,
    _selected_exact: bool,
    round_top: bool,
    round_bottom: bool,
) {
    ui.painter().rect(
        rect,
        hierarchy_row_corner_radius(runtime, round_top, round_bottom),
        fill,
        stroke,
        StrokeKind::Inside,
    );

    let left_offset = HIERARCHY_ROW_PADDING_X + depth as f32 * indent_width;
    let disclosure_center_x = rect.left() + left_offset + HIERARCHY_DISCLOSURE_SIZE * 0.5;
    if !node.children.is_empty() {
        let disclosure_name = if node.expanded {
            "chevron-down"
        } else {
            "chevron-right"
        };
        if let Some(image) = icons::image(ui.ctx(), disclosure_name, 14.0) {
            let _ = image.tint(tokens::text_muted(runtime)).paint_at(
                ui,
                Rect::from_center_size(
                    egui::pos2(disclosure_center_x, rect.center().y),
                    vec2(14.0, 14.0),
                ),
            );
        }
    }

    let icon_left = rect.left() + left_offset + HIERARCHY_DISCLOSURE_SIZE + 6.0;
    paint_hierarchy_item_icon(ui, node.kind, runtime, icon_left, rect.center().y);

    let label_left = icon_left + HIERARCHY_ICON_SIZE + 10.0;
    let label_color =
        tokens::text_primary(runtime).lerp_to_gamma(tokens::text_secondary(runtime), 0.28);
    ui.painter().text(
        egui::pos2(label_left, rect.center().y),
        Align2::LEFT_CENTER,
        node.label,
        typography::label_font(),
        label_color,
    );

    let show_lock_action = shows_child_lock && (hovered || node.locked);
    if show_lock_action {
        let actions_right = rect.right() - HIERARCHY_ROW_PADDING_X;
        let lock_rect = Rect::from_center_size(
            egui::pos2(
                actions_right - HIERARCHY_ACTION_ICON_SIZE * 0.5,
                rect.center().y,
            ),
            vec2(HIERARCHY_ACTION_ICON_SIZE, HIERARCHY_ACTION_ICON_SIZE),
        );
        let lock_icon = if node.locked {
            "bootstrap:lock-fill"
        } else {
            "bootstrap:unlock-fill"
        };
        let lock_tint = if node.locked {
            tokens::text_secondary(runtime)
        } else {
            tokens::text_muted(runtime)
        };
        if let Some(image) = icons::image(ui.ctx(), lock_icon, HIERARCHY_ACTION_ICON_SIZE) {
            let _ = image.tint(lock_tint).paint_at(ui, lock_rect);
        }
    }
}

fn paint_hierarchy_item_icon(
    ui: &mut Ui,
    kind: HierarchyItemKind,
    runtime: crate::theme::ThemeRuntime,
    icon_left: f32,
    center_y: f32,
) {
    let icon_rect = Rect::from_center_size(
        egui::pos2(icon_left + HIERARCHY_ICON_SIZE * 0.5, center_y),
        vec2(HIERARCHY_ICON_SIZE, HIERARCHY_ICON_SIZE),
    );

    match kind.icon() {
        HierarchyItemIcon::Icon(name) => {
            if let Some(image) = icons::image(ui.ctx(), name, HIERARCHY_ICON_SIZE) {
                let _ = image
                    .tint(tokens::text_secondary(runtime))
                    .paint_at(ui, icon_rect);
            }
        }
        HierarchyItemIcon::Twemoji(emoji) => {
            if let Some(image) = twemoji::image(emoji, HIERARCHY_ICON_SIZE) {
                let _ = image.paint_at(ui, icon_rect);
            }
        }
    }
}

fn hierarchy_pending_move(
    ui: &Ui,
    response: &Response,
    hierarchy_id: Id,
    parent_id: Option<usize>,
    node_id: usize,
    row_index: usize,
) -> Option<PendingMove> {
    hovered_hierarchy_payload(ui.ctx(), response, hierarchy_id)
        .filter(|payload| payload.parent_id == parent_id && payload.node_id != node_id)
        .and_then(|payload| {
            ui.ctx().pointer_interact_pos().map(|pointer| {
                let insert_after = pointer.y > response.rect.center().y;
                let y = if insert_after {
                    response.rect.bottom()
                } else {
                    response.rect.top()
                };
                ui.painter().line_segment(
                    [
                        egui::pos2(response.rect.left() + 8.0, y),
                        egui::pos2(response.rect.right() - 8.0, y),
                    ],
                    Stroke::new(2.0, HIERARCHY_SELECTION_LINE),
                );
                released_hierarchy_payload(ui.ctx(), response, hierarchy_id).map(|_| PendingMove {
                    parent_id,
                    node_id: payload.node_id,
                    target_index: row_index + usize::from(insert_after),
                })
            })?
        })
}

fn hovered_hierarchy_payload(
    _context: &egui::Context,
    response: &Response,
    hierarchy_id: Id,
) -> Option<HierarchyDragPayload> {
    response
        .dnd_hover_payload::<HierarchyDragPayload>()
        .filter(|payload| payload.hierarchy_id == hierarchy_id)
        .map(|payload| *payload)
}

fn released_hierarchy_payload(
    _context: &egui::Context,
    response: &Response,
    hierarchy_id: Id,
) -> Option<HierarchyDragPayload> {
    response
        .dnd_release_payload::<HierarchyDragPayload>()
        .filter(|payload| payload.hierarchy_id == hierarchy_id)
        .map(|payload| *payload)
}

fn move_hierarchy_node_within_parent<'a>(
    nodes: &mut Vec<HierarchyNode<'a>>,
    parent_id: Option<usize>,
    node_id: usize,
    target_index: usize,
) -> bool {
    let siblings = match find_hierarchy_siblings_mut(nodes, parent_id) {
        Some(siblings) => siblings,
        None => return false,
    };

    let current_index = match siblings.iter().position(|node| node.id == node_id) {
        Some(index) => index,
        None => return false,
    };

    let node = siblings.remove(current_index);
    let adjusted_index = if target_index > current_index {
        target_index.saturating_sub(1)
    } else {
        target_index
    }
    .min(siblings.len());
    siblings.insert(adjusted_index, node);
    true
}

fn find_hierarchy_siblings_mut<'nodes, 'data>(
    nodes: &'nodes mut Vec<HierarchyNode<'data>>,
    parent_id: Option<usize>,
) -> Option<&'nodes mut Vec<HierarchyNode<'data>>> {
    if parent_id.is_none() {
        return Some(nodes);
    }

    for node in nodes.iter_mut() {
        if Some(node.id) == parent_id {
            return Some(&mut node.children);
        }
        if let Some(found) = find_hierarchy_siblings_mut(&mut node.children, parent_id) {
            return Some(found);
        }
    }

    None
}

fn hierarchy_selection_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(118, 162, 255, 48)
}

fn hierarchy_selection_strong_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(118, 162, 255, 68)
}

fn hierarchy_selection_stroke() -> Color32 {
    Color32::from_rgba_unmultiplied(138, 182, 255, 132)
}

fn hierarchy_row_corner_radius(
    runtime: crate::theme::ThemeRuntime,
    round_top: bool,
    round_bottom: bool,
) -> CornerRadius {
    if !(round_top || round_bottom) {
        return CornerRadius::ZERO;
    }

    let radius = tokens::radius_sm(runtime);
    CornerRadius {
        nw: if round_top { radius } else { 0 },
        ne: if round_top { radius } else { 0 },
        sw: if round_bottom { radius } else { 0 },
        se: if round_bottom { radius } else { 0 },
    }
}

fn selection_range(
    nodes: &[HierarchyNode<'_>],
    selected_id: Option<usize>,
) -> Option<SelectionRange> {
    let selected_id = selected_id?;
    let mut ids = Vec::new();
    if !collect_selection_range(nodes, selected_id, &mut ids) {
        return None;
    }
    let first_id = *ids.first()?;
    let last_id = *ids.last()?;
    Some(SelectionRange {
        ids,
        first_id,
        last_id,
    })
}

fn collect_selection_range(
    nodes: &[HierarchyNode<'_>],
    selected_id: usize,
    ids: &mut Vec<usize>,
) -> bool {
    for node in nodes {
        if node.id == selected_id {
            collect_selection_subtree_ids(node, ids);
            return true;
        }
        if node.expanded && collect_selection_range(&node.children, selected_id, ids) {
            return true;
        }
    }
    false
}

fn collect_selection_subtree_ids(node: &HierarchyNode<'_>, ids: &mut Vec<usize>) {
    ids.push(node.id);
    if node.expanded {
        for child in &node.children {
            collect_selection_subtree_ids(child, ids);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        draw_hierarchy, move_hierarchy_node_within_parent, Hierarchy, HierarchyItemKind,
        HierarchyNode,
    };
    use egui::{CentralPanel, Context, Id, RawInput};

    #[test]
    fn move_hierarchy_node_within_same_parent_reorders_siblings() {
        let mut nodes = vec![
            HierarchyNode::new(1, "A", HierarchyItemKind::Frame),
            HierarchyNode::new(2, "B", HierarchyItemKind::Frame),
            HierarchyNode::new(3, "C", HierarchyItemKind::Frame),
        ];

        assert!(move_hierarchy_node_within_parent(&mut nodes, None, 1, 3));
        assert_eq!(
            nodes.iter().map(|node| node.id).collect::<Vec<_>>(),
            vec![2, 3, 1]
        );
    }

    #[test]
    fn move_hierarchy_node_ignores_missing_parent() {
        let mut nodes = vec![HierarchyNode::new(1, "A", HierarchyItemKind::Frame)];
        assert!(!move_hierarchy_node_within_parent(
            &mut nodes,
            Some(99),
            1,
            0
        ));
    }

    #[test]
    fn renders_hierarchy_tree() {
        let context = Context::default();
        let mut selected_id = Some(1usize);
        let mut nodes = vec![
            HierarchyNode::new(1, "Frame 87", HierarchyItemKind::Frame).children(vec![
                HierarchyNode::new(2, "Group 76", HierarchyItemKind::Group).children(vec![
                    HierarchyNode::new(3, "Vector", HierarchyItemKind::Vector),
                ]),
            ]),
        ];

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = draw_hierarchy(
                    ui,
                    &mut selected_id,
                    Hierarchy::new(Id::new("hierarchy_test"), &mut nodes),
                );
            });
        });
    }
}
