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
const HIERARCHY_ICON_SIZE: f32 = 13.0;
const HIERARCHY_ACTION_ICON_SIZE: f32 = HIERARCHY_ICON_SIZE;
const HIERARCHY_LOCK_GLYPH_SIZE: f32 = HIERARCHY_ACTION_ICON_SIZE;
const HIERARCHY_DISCLOSURE_BUTTON_SIZE: f32 = 20.0;
const HIERARCHY_DISCLOSURE_GLYPH_SIZE: f32 = 12.0;
const HIERARCHY_DROP_ZONE_HEIGHT: f32 = 6.0;
const HIERARCHY_DROP_EDGE_HEIGHT: f32 = 8.0;
const HIERARCHY_SELECTION_LINE: Color32 = Color32::from_rgb(138, 182, 255);

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum HierarchyIconStyle {
    #[default]
    Emoji,
    Icons,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum HierarchyStyle {
    #[default]
    Normal,
    Component,
}

impl HierarchyItemKind {
    fn icon(self, icon_style: HierarchyIconStyle) -> HierarchyItemIcon {
        match icon_style {
            HierarchyIconStyle::Emoji => match self {
                Self::Folder => HierarchyItemIcon::Icon("bootstrap:folder-fill"),
                Self::Frame => HierarchyItemIcon::Twemoji("🧩"),
                Self::Group => HierarchyItemIcon::Twemoji("⚙️"),
                Self::Player => HierarchyItemIcon::Twemoji("🧍"),
                Self::Weapon => HierarchyItemIcon::Twemoji("⚔️"),
                Self::Clothing => HierarchyItemIcon::Twemoji("👕"),
                Self::Hitbox => HierarchyItemIcon::Twemoji("🎯"),
                Self::Vector => HierarchyItemIcon::Twemoji("🔷"),
            },
            HierarchyIconStyle::Icons => match self {
                Self::Folder => HierarchyItemIcon::Icon("bootstrap:folder-fill"),
                Self::Frame => HierarchyItemIcon::Icon("frame"),
                Self::Group => HierarchyItemIcon::Icon("group"),
                Self::Player => HierarchyItemIcon::Icon("person-standing"),
                Self::Weapon => HierarchyItemIcon::Icon("sword"),
                Self::Clothing => HierarchyItemIcon::Icon("shirt"),
                Self::Hitbox => HierarchyItemIcon::Icon("crosshair"),
                Self::Vector => HierarchyItemIcon::Icon("diamond"),
            },
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
    pub icon_style: HierarchyIconStyle,
    pub style: HierarchyStyle,
}

impl<'a, 'nodes> Hierarchy<'a, 'nodes> {
    pub fn new(id: Id, nodes: &'nodes mut Vec<HierarchyNode<'a>>) -> Self {
        Self {
            id,
            nodes,
            width: HIERARCHY_DEFAULT_WIDTH,
            row_height: HIERARCHY_ROW_HEIGHT,
            indent_width: HIERARCHY_INDENT_WIDTH,
            icon_style: HierarchyIconStyle::default(),
            style: HierarchyStyle::default(),
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

    pub fn icon_style(mut self, icon_style: HierarchyIconStyle) -> Self {
        self.icon_style = icon_style;
        self
    }

    pub fn style(mut self, style: HierarchyStyle) -> Self {
        self.style = style;
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
    node_id: usize,
    target_parent_id: Option<usize>,
    target_index: usize,
    expand_target_id: Option<usize>,
}

#[derive(Debug, Clone)]
struct RowOutcome {
    response: Response,
    has_children: bool,
    child_count: usize,
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
                    props.icon_style,
                    props.style,
                    &mut pending_move,
                    &mut changed,
                    &mut pointer_over_interaction,
                );
            });
        })
        .response;

    if let Some(move_request) = pending_move {
        if move_hierarchy_node(
            props.nodes,
            move_request.node_id,
            move_request.target_parent_id,
            move_request.target_index,
            move_request.expand_target_id,
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
    icon_style: HierarchyIconStyle,
    style: HierarchyStyle,
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
                icon_style,
                style,
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
            row_outcome.child_count,
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
                icon_style,
                style,
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
    if let Some(payload) = hovered_hierarchy_payload(ui.ctx(), &drop_response, hierarchy_id) {
        ui.painter().line_segment(
            [
                egui::pos2(drop_rect.left() + 8.0, drop_rect.center().y),
                egui::pos2(drop_rect.right() - 8.0, drop_rect.center().y),
            ],
            Stroke::new(2.0, HIERARCHY_SELECTION_LINE),
        );
        if released_hierarchy_payload(ui.ctx(), &drop_response, hierarchy_id).is_some() {
            *pending_move = Some(PendingMove {
                node_id: payload.node_id,
                target_parent_id: parent_id,
                target_index: nodes.len(),
                expand_target_id: None,
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
    icon_style: HierarchyIconStyle,
    style: HierarchyStyle,
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
    let stroke = Stroke::NONE;

    let payload = HierarchyDragPayload {
        hierarchy_id,
        parent_id,
        node_id: node.id,
    };
    response.dnd_set_drag_payload(payload);

    let left_offset = HIERARCHY_ROW_PADDING_X + depth as f32 * indent_width;
    let disclosure_rect = Rect::from_center_size(
        egui::pos2(
            rect.left() + left_offset + HIERARCHY_DISCLOSURE_BUTTON_SIZE * 0.5,
            rect.center().y,
        ),
        vec2(
            HIERARCHY_DISCLOSURE_BUTTON_SIZE,
            HIERARCHY_DISCLOSURE_BUTTON_SIZE,
        ),
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

    paint_hierarchy_row(
        ui,
        rect,
        depth,
        node,
        runtime,
        fill,
        stroke,
        hovered,
        disclosure_response.as_ref(),
        shows_child_lock,
        indent_width,
        icon_style,
        style,
        selected_exact,
        round_top,
        round_bottom,
    );

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
        child_count: node.children.len(),
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
    disclosure_response: Option<&Response>,
    shows_child_lock: bool,
    indent_width: f32,
    icon_style: HierarchyIconStyle,
    style: HierarchyStyle,
    _selected_exact: bool,
    round_top: bool,
    round_bottom: bool,
) {
    ui.painter().rect(
        rect,
        hierarchy_row_corner_radius(
            runtime,
            hovered && !round_top && !round_bottom,
            round_top,
            round_bottom,
        ),
        fill,
        stroke,
        StrokeKind::Inside,
    );

    let left_offset = HIERARCHY_ROW_PADDING_X + depth as f32 * indent_width;
    let disclosure_rect = Rect::from_center_size(
        egui::pos2(
            rect.left() + left_offset + HIERARCHY_DISCLOSURE_BUTTON_SIZE * 0.5,
            rect.center().y,
        ),
        vec2(
            HIERARCHY_DISCLOSURE_BUTTON_SIZE,
            HIERARCHY_DISCLOSURE_BUTTON_SIZE,
        ),
    );
    if !node.children.is_empty() {
        let (disclosure_fill, disclosure_stroke) =
            if disclosure_response.is_some_and(Response::is_pointer_button_down_on) {
                (
                    tokens::button_secondary_active_bg(runtime),
                    Stroke::new(1.0, tokens::button_secondary_hover_border(runtime)),
                )
            } else if disclosure_response.is_some_and(Response::hovered) {
                (
                    tokens::button_secondary_hover_bg(runtime),
                    Stroke::new(1.0, tokens::button_secondary_hover_border(runtime)),
                )
            } else {
                (Color32::TRANSPARENT, Stroke::NONE)
            };
        if disclosure_fill != Color32::TRANSPARENT || disclosure_stroke != Stroke::NONE {
            ui.painter().rect(
                disclosure_rect,
                CornerRadius::same(6),
                disclosure_fill,
                disclosure_stroke,
                StrokeKind::Inside,
            );
        }

        let disclosure_name = if node.expanded {
            "chevron-down"
        } else {
            "chevron-right"
        };
        if let Some(image) =
            icons::image(ui.ctx(), disclosure_name, HIERARCHY_DISCLOSURE_GLYPH_SIZE)
        {
            let chevron_tint = if disclosure_response.is_some_and(Response::hovered) {
                tokens::text_secondary(runtime)
            } else {
                tokens::text_muted(runtime)
            };
            let _ = image.tint(chevron_tint).paint_at(
                ui,
                Rect::from_center_size(
                    disclosure_rect.center(),
                    vec2(
                        HIERARCHY_DISCLOSURE_GLYPH_SIZE,
                        HIERARCHY_DISCLOSURE_GLYPH_SIZE,
                    ),
                ),
            );
        }
    }

    let icon_left = rect.left() + left_offset + HIERARCHY_DISCLOSURE_BUTTON_SIZE + 6.0;
    paint_hierarchy_item_icon(
        ui,
        node.kind,
        icon_style,
        style,
        runtime,
        icon_left,
        rect.center().y,
    );

    let label_left = icon_left + HIERARCHY_ICON_SIZE + 10.0;
    let label_color = hierarchy_label_color(style, runtime);
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
        if let Some(image) = icons::image(ui.ctx(), lock_icon, HIERARCHY_LOCK_GLYPH_SIZE) {
            let _ = image.tint(lock_tint).paint_at(ui, lock_rect);
        }
    }
}

fn paint_hierarchy_item_icon(
    ui: &mut Ui,
    kind: HierarchyItemKind,
    icon_style: HierarchyIconStyle,
    style: HierarchyStyle,
    runtime: crate::theme::ThemeRuntime,
    icon_left: f32,
    center_y: f32,
) {
    let icon_rect = Rect::from_center_size(
        egui::pos2(icon_left + HIERARCHY_ICON_SIZE * 0.5, center_y),
        vec2(HIERARCHY_ICON_SIZE, HIERARCHY_ICON_SIZE),
    );

    match kind.icon(icon_style) {
        HierarchyItemIcon::Icon(name) => {
            if let Some(image) = icons::image(ui.ctx(), name, HIERARCHY_ICON_SIZE) {
                let _ = image
                    .tint(hierarchy_icon_tint(style, runtime))
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

fn hierarchy_label_color(style: HierarchyStyle, runtime: crate::theme::ThemeRuntime) -> Color32 {
    match style {
        HierarchyStyle::Normal => {
            tokens::text_primary(runtime).lerp_to_gamma(tokens::text_secondary(runtime), 0.28)
        }
        HierarchyStyle::Component => Color32::from_rgb(193, 153, 255),
    }
}

fn hierarchy_icon_tint(style: HierarchyStyle, runtime: crate::theme::ThemeRuntime) -> Color32 {
    match style {
        HierarchyStyle::Normal => tokens::text_secondary(runtime),
        HierarchyStyle::Component => hierarchy_label_color(style, runtime),
    }
}

fn hierarchy_pending_move(
    ui: &Ui,
    response: &Response,
    hierarchy_id: Id,
    parent_id: Option<usize>,
    node_id: usize,
    row_index: usize,
    child_count: usize,
) -> Option<PendingMove> {
    hovered_hierarchy_payload(ui.ctx(), response, hierarchy_id)
        .filter(|payload| payload.node_id != node_id)
        .and_then(|payload| {
            ui.ctx().pointer_interact_pos().map(|pointer| {
                let placement = if pointer.y <= response.rect.top() + HIERARCHY_DROP_EDGE_HEIGHT {
                    HierarchyDropPlacement::Before
                } else if pointer.y >= response.rect.bottom() - HIERARCHY_DROP_EDGE_HEIGHT {
                    HierarchyDropPlacement::After
                } else {
                    HierarchyDropPlacement::Into
                };

                let pending_move = match placement {
                    HierarchyDropPlacement::Before => {
                        ui.painter().line_segment(
                            [
                                egui::pos2(response.rect.left() + 8.0, response.rect.top()),
                                egui::pos2(response.rect.right() - 8.0, response.rect.top()),
                            ],
                            Stroke::new(2.0, HIERARCHY_SELECTION_LINE),
                        );
                        PendingMove {
                            node_id: payload.node_id,
                            target_parent_id: parent_id,
                            target_index: row_index,
                            expand_target_id: None,
                        }
                    }
                    HierarchyDropPlacement::After => {
                        ui.painter().line_segment(
                            [
                                egui::pos2(response.rect.left() + 8.0, response.rect.bottom()),
                                egui::pos2(response.rect.right() - 8.0, response.rect.bottom()),
                            ],
                            Stroke::new(2.0, HIERARCHY_SELECTION_LINE),
                        );
                        PendingMove {
                            node_id: payload.node_id,
                            target_parent_id: parent_id,
                            target_index: row_index + 1,
                            expand_target_id: None,
                        }
                    }
                    HierarchyDropPlacement::Into => {
                        ui.painter().rect_stroke(
                            response.rect.shrink2(vec2(8.0, 4.0)),
                            CornerRadius::same(6),
                            Stroke::new(1.5, HIERARCHY_SELECTION_LINE),
                            StrokeKind::Inside,
                        );
                        PendingMove {
                            node_id: payload.node_id,
                            target_parent_id: Some(node_id),
                            target_index: child_count,
                            expand_target_id: Some(node_id),
                        }
                    }
                };

                released_hierarchy_payload(ui.ctx(), response, hierarchy_id).map(|_| pending_move)
            })?
        })
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum HierarchyDropPlacement {
    Before,
    Into,
    After,
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

fn move_hierarchy_node<'a>(
    nodes: &mut Vec<HierarchyNode<'a>>,
    node_id: usize,
    target_parent_id: Option<usize>,
    target_index: usize,
    expand_target_id: Option<usize>,
) -> bool {
    if target_parent_id == Some(node_id) {
        return false;
    }

    if target_parent_id
        .is_some_and(|target_parent_id| hierarchy_node_contains(nodes, node_id, target_parent_id))
    {
        return false;
    }

    if target_parent_id.is_some_and(|target_parent_id| {
        find_hierarchy_node(nodes.as_slice(), target_parent_id).is_none()
    }) {
        return false;
    }

    let (node, source_parent_id, source_index) = match take_hierarchy_node(nodes, None, node_id) {
        Some(result) => result,
        None => return false,
    };

    let siblings = match find_hierarchy_siblings_mut(nodes, target_parent_id) {
        Some(siblings) => siblings,
        None => return false,
    };

    let adjusted_index = if source_parent_id == target_parent_id && source_index < target_index {
        target_index.saturating_sub(1)
    } else {
        target_index
    }
    .min(siblings.len());
    siblings.insert(adjusted_index, node);

    if let Some(expand_target_id) = expand_target_id {
        if let Some(target_node) = find_hierarchy_node_mut(nodes, expand_target_id) {
            target_node.expanded = true;
        }
    }

    true
}

fn take_hierarchy_node<'nodes, 'data>(
    nodes: &'nodes mut Vec<HierarchyNode<'data>>,
    parent_id: Option<usize>,
    node_id: usize,
) -> Option<(HierarchyNode<'data>, Option<usize>, usize)> {
    if let Some(index) = nodes.iter().position(|node| node.id == node_id) {
        return Some((nodes.remove(index), parent_id, index));
    }

    for node in nodes.iter_mut() {
        if let Some(result) = take_hierarchy_node(&mut node.children, Some(node.id), node_id) {
            return Some(result);
        }
    }

    None
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

fn find_hierarchy_node<'nodes, 'data>(
    nodes: &'nodes [HierarchyNode<'data>],
    node_id: usize,
) -> Option<&'nodes HierarchyNode<'data>> {
    for node in nodes {
        if node.id == node_id {
            return Some(node);
        }
        if let Some(found) = find_hierarchy_node(node.children.as_slice(), node_id) {
            return Some(found);
        }
    }

    None
}

fn find_hierarchy_node_mut<'nodes, 'data>(
    nodes: &'nodes mut Vec<HierarchyNode<'data>>,
    node_id: usize,
) -> Option<&'nodes mut HierarchyNode<'data>> {
    for node in nodes.iter_mut() {
        if node.id == node_id {
            return Some(node);
        }
        if let Some(found) = find_hierarchy_node_mut(&mut node.children, node_id) {
            return Some(found);
        }
    }

    None
}

fn hierarchy_node_contains(
    nodes: &[HierarchyNode<'_>],
    ancestor_id: usize,
    candidate_id: usize,
) -> bool {
    find_hierarchy_node(nodes, ancestor_id)
        .is_some_and(|ancestor| hierarchy_subtree_contains(ancestor, candidate_id))
}

fn hierarchy_subtree_contains(node: &HierarchyNode<'_>, candidate_id: usize) -> bool {
    node.id == candidate_id
        || node
            .children
            .iter()
            .any(|child| hierarchy_subtree_contains(child, candidate_id))
}

fn hierarchy_selection_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(118, 162, 255, 48)
}

fn hierarchy_selection_strong_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(118, 162, 255, 68)
}

fn hierarchy_row_corner_radius(
    runtime: crate::theme::ThemeRuntime,
    hovered: bool,
    round_top: bool,
    round_bottom: bool,
) -> CornerRadius {
    if hovered {
        return CornerRadius::same(tokens::radius_sm(runtime));
    }

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
        draw_hierarchy, move_hierarchy_node, Hierarchy, HierarchyIconStyle, HierarchyItemKind,
        HierarchyNode, HierarchyStyle,
    };
    use crate::ui::tokens;
    use egui::{CentralPanel, Context, Id, RawInput};

    #[test]
    fn hovered_hierarchy_rows_use_uniform_corner_radius() {
        let runtime = crate::theme::ThemeRuntime::default();
        let radius = super::hierarchy_row_corner_radius(runtime, true, false, false);
        let expected = tokens::radius_sm(runtime);

        assert_eq!(radius.nw, expected);
        assert_eq!(radius.ne, expected);
        assert_eq!(radius.sw, expected);
        assert_eq!(radius.se, expected);
    }

    #[test]
    fn selected_hierarchy_ranges_only_round_exposed_edges() {
        let runtime = crate::theme::ThemeRuntime::default();
        let radius = super::hierarchy_row_corner_radius(runtime, false, true, false);
        let expected = tokens::radius_sm(runtime);

        assert_eq!(radius.nw, expected);
        assert_eq!(radius.ne, expected);
        assert_eq!(radius.sw, 0);
        assert_eq!(radius.se, 0);
    }

    #[test]
    fn move_hierarchy_node_within_same_parent_reorders_siblings() {
        let mut nodes = vec![
            HierarchyNode::new(1, "A", HierarchyItemKind::Frame),
            HierarchyNode::new(2, "B", HierarchyItemKind::Frame),
            HierarchyNode::new(3, "C", HierarchyItemKind::Frame),
        ];

        assert!(move_hierarchy_node(&mut nodes, 1, None, 3, None));
        assert_eq!(
            nodes.iter().map(|node| node.id).collect::<Vec<_>>(),
            vec![2, 3, 1]
        );
    }

    #[test]
    fn move_hierarchy_node_ignores_missing_parent() {
        let mut nodes = vec![HierarchyNode::new(1, "A", HierarchyItemKind::Frame)];
        assert!(!move_hierarchy_node(&mut nodes, 1, Some(99), 0, Some(99)));
    }

    #[test]
    fn move_hierarchy_node_into_other_parent_appends_as_last_child() {
        let mut nodes = vec![
            HierarchyNode::new(1, "FolderA", HierarchyItemKind::Folder).children(vec![
                HierarchyNode::new(2, "Child", HierarchyItemKind::Player),
            ]),
            HierarchyNode::new(3, "FolderB", HierarchyItemKind::Folder),
        ];

        assert!(move_hierarchy_node(&mut nodes, 2, Some(3), 0, Some(3)));
        assert!(nodes[0].children.is_empty());
        assert_eq!(
            nodes[1]
                .children
                .iter()
                .map(|node| node.id)
                .collect::<Vec<_>>(),
            vec![2]
        );
        assert!(nodes[1].expanded);
    }

    #[test]
    fn move_hierarchy_node_to_root_from_child_parent() {
        let mut nodes = vec![
            HierarchyNode::new(1, "Folder", HierarchyItemKind::Folder).children(vec![
                HierarchyNode::new(2, "Child", HierarchyItemKind::Player),
            ]),
        ];

        assert!(move_hierarchy_node(&mut nodes, 2, None, 1, None));
        assert_eq!(
            nodes.iter().map(|node| node.id).collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert!(nodes[0].children.is_empty());
    }

    #[test]
    fn move_hierarchy_node_rejects_descendant_target_parent() {
        let mut nodes = vec![
            HierarchyNode::new(1, "Root", HierarchyItemKind::Folder).children(vec![
                HierarchyNode::new(2, "Child", HierarchyItemKind::Group).children(vec![
                    HierarchyNode::new(3, "Leaf", HierarchyItemKind::Vector),
                ]),
            ]),
        ];

        assert!(!move_hierarchy_node(&mut nodes, 1, Some(3), 0, Some(3)));
        assert_eq!(nodes[0].id, 1);
        assert_eq!(nodes[0].children[0].id, 2);
        assert_eq!(nodes[0].children[0].children[0].id, 3);
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

    #[test]
    fn renders_hierarchy_tree_in_icon_mode() {
        let context = Context::default();
        let mut selected_id = Some(1usize);
        let mut nodes = vec![
            HierarchyNode::new(1, "Gameplay_Systems", HierarchyItemKind::Folder).children(vec![
                HierarchyNode::new(2, "Player_Controller", HierarchyItemKind::Player),
                HierarchyNode::new(3, "Iron_Sword_01", HierarchyItemKind::Weapon),
                HierarchyNode::new(4, "Hitbox_Main", HierarchyItemKind::Hitbox),
            ]),
        ];

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = draw_hierarchy(
                    ui,
                    &mut selected_id,
                    Hierarchy::new(Id::new("hierarchy_icon_mode_test"), &mut nodes)
                        .icon_style(HierarchyIconStyle::Icons),
                );
            });
        });
    }

    #[test]
    fn renders_hierarchy_tree_in_component_style() {
        let context = Context::default();
        let mut selected_id = Some(1usize);
        let mut nodes = vec![
            HierarchyNode::new(1, "Gameplay_Systems", HierarchyItemKind::Folder).children(vec![
                HierarchyNode::new(2, "Character_Rig_A", HierarchyItemKind::Group).children(vec![
                    HierarchyNode::new(3, "Player_Controller", HierarchyItemKind::Player),
                ]),
            ]),
        ];

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = draw_hierarchy(
                    ui,
                    &mut selected_id,
                    Hierarchy::new(Id::new("hierarchy_component_style_test"), &mut nodes)
                        .icon_style(HierarchyIconStyle::Icons)
                        .style(HierarchyStyle::Component),
                );
            });
        });
    }
}
