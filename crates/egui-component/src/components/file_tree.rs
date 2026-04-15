use super::api::ComponentUi;
use crate::ui::{icons, tokens, typography};
use egui::{vec2, Align2, Color32, Id, Rect, Response, Sense, Ui};

const FILE_TREE_MIN_WIDTH: f32 = 140.0;
const FILE_TREE_ROW_HEIGHT: f32 = 20.0;
const FILE_TREE_INDENT_WIDTH: f32 = 14.0;
const FILE_TREE_ROW_PADDING_X: f32 = 4.0;
const FILE_TREE_ROW_PADDING_RIGHT: f32 = 8.0;
const FILE_TREE_DISCLOSURE_SLOT_WIDTH: f32 = 10.0;
const FILE_TREE_DISCLOSURE_GLYPH_SIZE: f32 = 8.0;
const FILE_TREE_ICON_SIZE: f32 = 13.0;
const FILE_TREE_ICON_GAP: f32 = 6.0;

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum FileTreeItemKind {
    Folder,
    Collection,
    Script,
    Project,
    Markdown,
    File,
}

impl FileTreeItemKind {
    fn icon(self, expanded: bool) -> &'static str {
        match self {
            Self::Folder if expanded => "bootstrap:folder2-open",
            Self::Folder => "bootstrap:folder2",
            Self::Collection => "bootstrap:collection-fill",
            Self::Script => "bootstrap:file-earmark-code-fill",
            Self::Project => "bootstrap:gear-fill",
            Self::Markdown => "bootstrap:filetype-md",
            Self::File => "bootstrap:file-earmark-text-fill",
        }
    }

    fn tint(self, runtime: crate::theme::ThemeRuntime) -> Color32 {
        match self {
            Self::Folder => {
                if runtime.mode.is_dark() {
                    Color32::from_rgb(176, 183, 194)
                } else {
                    tokens::text_secondary(runtime)
                }
            }
            Self::Collection => Color32::from_rgb(84, 163, 255),
            Self::Script => Color32::from_rgb(106, 201, 110),
            Self::Project => Color32::from_rgb(126, 212, 128),
            Self::Markdown => {
                if runtime.mode.is_dark() {
                    Color32::from_rgb(186, 191, 200)
                } else {
                    tokens::text_secondary(runtime)
                }
            }
            Self::File => tokens::text_muted(runtime),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileTreeNode<'a> {
    pub id: usize,
    pub label: &'a str,
    pub kind: FileTreeItemKind,
    pub expanded: bool,
    pub children: Vec<FileTreeNode<'a>>,
}

impl<'a> FileTreeNode<'a> {
    pub fn new(id: usize, label: &'a str, kind: FileTreeItemKind) -> Self {
        Self {
            id,
            label,
            kind,
            expanded: true,
            children: Vec::new(),
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn children(mut self, children: Vec<FileTreeNode<'a>>) -> Self {
        self.children = children;
        self
    }
}

#[derive(Debug)]
pub struct FileTree<'a, 'nodes> {
    pub id: Id,
    pub nodes: &'nodes mut Vec<FileTreeNode<'a>>,
    pub width: f32,
    pub row_height: f32,
    pub indent_width: f32,
}

impl<'a, 'nodes> FileTree<'a, 'nodes> {
    pub fn new(id: Id, nodes: &'nodes mut Vec<FileTreeNode<'a>>) -> Self {
        Self {
            id,
            nodes,
            width: f32::INFINITY,
            row_height: FILE_TREE_ROW_HEIGHT,
            indent_width: FILE_TREE_INDENT_WIDTH,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(FILE_TREE_MIN_WIDTH);
        self
    }

    pub fn row_height(mut self, row_height: f32) -> Self {
        self.row_height = row_height.max(16.0);
        self
    }

    pub fn indent_width(mut self, indent_width: f32) -> Self {
        self.indent_width = indent_width.max(8.0);
        self
    }
}

impl ComponentUi<'_> {
    pub fn file_tree<'a, 'nodes>(
        &mut self,
        selected_id: &mut Option<usize>,
        props: FileTree<'a, 'nodes>,
    ) -> Response {
        draw_file_tree(self.ui_mut(), selected_id, props)
    }
}

fn draw_file_tree(
    ui: &mut Ui,
    selected_id: &mut Option<usize>,
    props: FileTree<'_, '_>,
) -> Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let mut changed = false;
    let width = props
        .width
        .min(ui.available_width().max(FILE_TREE_MIN_WIDTH));
    let mut response = ui
        .scope(|ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing.y = 0.0;
            draw_file_tree_list(
                ui,
                props.id,
                props.nodes,
                0,
                selected_id,
                runtime,
                props.row_height,
                props.indent_width,
                &mut changed,
            );
        })
        .response;

    if changed {
        response.mark_changed();
    }

    response
}

#[allow(clippy::too_many_arguments)]
fn draw_file_tree_list(
    ui: &mut Ui,
    tree_id: Id,
    nodes: &mut Vec<FileTreeNode<'_>>,
    depth: usize,
    selected_id: &mut Option<usize>,
    runtime: crate::theme::ThemeRuntime,
    row_height: f32,
    indent_width: f32,
    changed: &mut bool,
) {
    for node in nodes.iter_mut() {
        let expanded = draw_file_tree_row(
            ui,
            tree_id,
            node,
            depth,
            selected_id,
            runtime,
            row_height,
            indent_width,
            changed,
        );

        if expanded && !node.children.is_empty() {
            draw_file_tree_list(
                ui,
                tree_id,
                &mut node.children,
                depth + 1,
                selected_id,
                runtime,
                row_height,
                indent_width,
                changed,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_file_tree_row(
    ui: &mut Ui,
    tree_id: Id,
    node: &mut FileTreeNode<'_>,
    depth: usize,
    selected_id: &mut Option<usize>,
    runtime: crate::theme::ThemeRuntime,
    row_height: f32,
    indent_width: f32,
    changed: &mut bool,
) -> bool {
    let row_width = ui.available_width();
    let (rect, row_response) = ui.allocate_exact_size(vec2(row_width, row_height), Sense::click());
    let selected = *selected_id == Some(node.id);
    let hovered = row_response.hovered();
    let has_children = !node.children.is_empty();

    let fill = if selected {
        file_tree_selected_fill(runtime)
    } else if hovered {
        file_tree_hover_fill(runtime)
    } else {
        Color32::TRANSPARENT
    };

    if fill != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 0, fill);
    }

    let row_left = rect.left() + FILE_TREE_ROW_PADDING_X + depth as f32 * indent_width;
    let disclosure_rect = Rect::from_center_size(
        egui::pos2(
            row_left + FILE_TREE_DISCLOSURE_SLOT_WIDTH * 0.5,
            rect.center().y,
        ),
        vec2(FILE_TREE_DISCLOSURE_SLOT_WIDTH, row_height),
    );
    let disclosure_response = has_children.then(|| {
        ui.interact(
            disclosure_rect,
            tree_id.with(("file_tree_disclosure", node.id)),
            Sense::click(),
        )
    });

    if let Some(disclosure_response) = disclosure_response.as_ref() {
        let disclosure_icon = if node.expanded {
            "bootstrap:caret-down-fill"
        } else {
            "bootstrap:caret-right-fill"
        };
        if let Some(image) =
            icons::image(ui.ctx(), disclosure_icon, FILE_TREE_DISCLOSURE_GLYPH_SIZE)
        {
            let tint = if disclosure_response.hovered() || selected {
                tokens::text_primary(runtime)
            } else {
                tokens::text_muted(runtime)
            };
            let _ = image.tint(tint).paint_at(
                ui,
                Rect::from_center_size(
                    disclosure_rect.center(),
                    vec2(
                        FILE_TREE_DISCLOSURE_GLYPH_SIZE,
                        FILE_TREE_DISCLOSURE_GLYPH_SIZE,
                    ),
                ),
            );
        }
    }

    let icon_left = row_left + FILE_TREE_DISCLOSURE_SLOT_WIDTH + 2.0;
    let icon_rect = Rect::from_center_size(
        egui::pos2(icon_left + FILE_TREE_ICON_SIZE * 0.5, rect.center().y),
        vec2(FILE_TREE_ICON_SIZE, FILE_TREE_ICON_SIZE),
    );
    if let Some(image) = icons::image(ui.ctx(), node.kind.icon(node.expanded), FILE_TREE_ICON_SIZE)
    {
        let _ = image.tint(node.kind.tint(runtime)).paint_at(ui, icon_rect);
    }

    let label_left = icon_left + FILE_TREE_ICON_SIZE + FILE_TREE_ICON_GAP;
    let label_rect = Rect::from_min_max(
        egui::pos2(label_left, rect.top()),
        egui::pos2(
            (rect.right() - FILE_TREE_ROW_PADDING_RIGHT).max(label_left + 1.0),
            rect.bottom(),
        ),
    );
    ui.painter().with_clip_rect(label_rect).text(
        egui::pos2(label_rect.left(), label_rect.center().y),
        Align2::LEFT_CENTER,
        node.label,
        typography::label_font(),
        file_tree_label_color(runtime, selected),
    );

    if disclosure_response.as_ref().is_some_and(Response::clicked) {
        node.expanded = !node.expanded;
        *changed = true;
    } else if row_response.clicked() && *selected_id != Some(node.id) {
        *selected_id = Some(node.id);
        *changed = true;
    }

    node.expanded
}

fn file_tree_hover_fill(runtime: crate::theme::ThemeRuntime) -> Color32 {
    if runtime.mode.is_dark() {
        Color32::from_rgba_unmultiplied(255, 255, 255, 14)
    } else {
        tokens::button_secondary_hover_bg(runtime).linear_multiply(0.6)
    }
}

fn file_tree_selected_fill(runtime: crate::theme::ThemeRuntime) -> Color32 {
    if runtime.mode.is_dark() {
        Color32::from_rgb(72, 78, 88)
    } else {
        tokens::button_secondary_hover_bg(runtime).linear_multiply(0.95)
    }
}

fn file_tree_label_color(runtime: crate::theme::ThemeRuntime, selected: bool) -> Color32 {
    if selected {
        tokens::text_primary(runtime)
    } else {
        tokens::text_secondary(runtime).lerp_to_gamma(tokens::text_primary(runtime), 0.18)
    }
}

#[cfg(test)]
mod tests {
    use super::{draw_file_tree, FileTree, FileTreeItemKind, FileTreeNode};
    use egui::{CentralPanel, Context, Id, RawInput};

    #[test]
    fn folder_icons_follow_expansion_state() {
        assert_eq!(FileTreeItemKind::Folder.icon(false), "bootstrap:folder2");
        assert_eq!(
            FileTreeItemKind::Folder.icon(true),
            "bootstrap:folder2-open"
        );
    }

    #[test]
    fn renders_file_tree() {
        let context = Context::default();
        let mut selected_id = Some(3usize);
        let mut nodes = vec![
            FileTreeNode::new(1, "SpaceShooter", FileTreeItemKind::Folder).children(vec![
                FileTreeNode::new(2, "assets", FileTreeItemKind::Folder)
                    .expanded(false)
                    .children(vec![FileTreeNode::new(
                        4,
                        "player.png",
                        FileTreeItemKind::File,
                    )]),
                FileTreeNode::new(3, "main.collection", FileTreeItemKind::Collection),
            ]),
        ];

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = draw_file_tree(
                    ui,
                    &mut selected_id,
                    FileTree::new(Id::new("file_tree_test"), &mut nodes).width(220.0),
                );
            });
        });
    }
}
