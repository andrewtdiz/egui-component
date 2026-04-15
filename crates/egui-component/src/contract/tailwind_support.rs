use super::{ContractFamilyId, ContractNode, ContractTree};
use crate::ui::tailwind;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ContractTailwindDiagnosticReason {
    UnknownToken,
    UnsupportedFamily,
}

impl ContractTailwindDiagnosticReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnknownToken => "unknown-token",
            Self::UnsupportedFamily => "unsupported-family",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ContractTailwindDiagnostic {
    pub node_id: String,
    pub family: String,
    pub token: String,
    pub reason: ContractTailwindDiagnosticReason,
}

pub fn audit_tailwind_support(tree: &ContractTree) -> Vec<ContractTailwindDiagnostic> {
    let mut diagnostics = Vec::new();
    audit_node(&tree.root, false, false, &mut diagnostics);
    diagnostics
}

fn audit_node(
    node: &ContractNode,
    parent_supports_item_layout: bool,
    parent_is_grid: bool,
    diagnostics: &mut Vec<ContractTailwindDiagnostic>,
) {
    let common = node.common();
    let family = node.family_id();
    let node_id = common.node_id.as_str();
    let node_spec = merged_class_spec(common);
    let node_is_grid = node
        .common()
        .layout
        .as_ref()
        .and_then(|layout| layout.display)
        .is_some_and(|display| matches!(display, super::ContractDisplay::Grid))
        || node_spec.as_ref().is_some_and(|spec| spec.is_grid);

    for token in common
        .class
        .as_deref()
        .into_iter()
        .flat_map(|classes| classes.split_whitespace())
        .chain(
            common
                .class_list
                .iter()
                .flat_map(|classes| classes.split_whitespace()),
        )
    {
        if token.is_empty() {
            continue;
        }

        let Some(spec) = tailwind::token_spec(token) else {
            diagnostics.push(ContractTailwindDiagnostic {
                node_id: node_id.to_owned(),
                family: family.as_str().to_owned(),
                token: token.to_owned(),
                reason: ContractTailwindDiagnosticReason::UnknownToken,
            });
            continue;
        };

        if !token_supported_for_context(family, &spec, parent_supports_item_layout, parent_is_grid)
        {
            diagnostics.push(ContractTailwindDiagnostic {
                node_id: node_id.to_owned(),
                family: family.as_str().to_owned(),
                token: token.to_owned(),
                reason: ContractTailwindDiagnosticReason::UnsupportedFamily,
            });
        }
    }

    for child in contract_children(node) {
        audit_node(
            child,
            is_flow_container_family(family),
            node_is_grid,
            diagnostics,
        );
    }
}

fn merged_class_spec(common: &super::ContractCommon) -> Option<tailwind::Spec> {
    let mut classes = String::new();
    if let Some(class) = common.class.as_deref() {
        classes.push_str(class);
    }
    for class in &common.class_list {
        if !classes.is_empty() {
            classes.push(' ');
        }
        classes.push_str(class);
    }
    let classes = classes.trim();
    (!classes.is_empty()).then(|| tailwind::parse(classes))
}

fn token_supported_for_context(
    family: ContractFamilyId,
    spec: &tailwind::Spec,
    parent_supports_item_layout: bool,
    parent_is_grid: bool,
) -> bool {
    if touches_hover_state(spec) || touches_group_hover_state(spec) {
        return false;
    }

    let touches_shared_layout = spec.width.is_some()
        || spec.height.is_some()
        || spec.min_width.is_some()
        || spec.min_height.is_some()
        || spec.max_width.is_some()
        || spec.max_height.is_some()
        || spec.padding.any()
        || spec.margin.any();
    if touches_shared_layout {
        return true;
    }

    let touches_clip_strategy = spec.clip_strategy.is_some();
    if touches_clip_strategy {
        return false;
    }

    let touches_unmappable_alignment = spec.align_content.is_some()
        || spec.align_self.is_some()
        || matches!(
            spec.justify,
            Some(tailwind::JustifyContent::Between | tailwind::JustifyContent::Around)
        )
        || matches!(spec.flex_wrap, Some(tailwind::FlexWrap::WrapReverse));
    if touches_unmappable_alignment {
        return false;
    }

    let touches_flex_flow = spec.is_flex
        || spec.direction.is_some()
        || spec.justify.is_some()
        || spec.align_items.is_some()
        || spec.flex_wrap.is_some()
        || spec.gap_col.is_some()
        || spec.gap_row.is_some();
    if touches_flex_flow {
        return matches!(
            family,
            ContractFamilyId::Row
                | ContractFamilyId::Column
                | ContractFamilyId::Inset
                | ContractFamilyId::Card
        );
    }

    let touches_grid = spec.is_grid || spec.grid_cols.is_some() || spec.grid_rows.is_some();
    if touches_grid {
        return matches!(
            family,
            ContractFamilyId::Row
                | ContractFamilyId::Column
                | ContractFamilyId::Inset
                | ContractFamilyId::Card
        );
    }

    let touches_item_layout =
        spec.flex_grow.is_some() || spec.flex_shrink.is_some() || spec.flex_basis.is_some();
    if touches_item_layout {
        return parent_supports_item_layout && !parent_is_grid;
    }

    let touches_overflow = spec.clip_children.is_some() || spec.scroll_x || spec.scroll_y;
    if touches_overflow {
        return matches!(
            family,
            ContractFamilyId::Row
                | ContractFamilyId::Column
                | ContractFamilyId::Inset
                | ContractFamilyId::Card
        );
    }

    let touches_card_visual = spec.background.is_some()
        || spec.border.any()
        || spec.border_color.is_some()
        || spec.corner_radii.any()
        || spec.shadow.is_some();
    if touches_card_visual {
        return matches!(family, ContractFamilyId::Card);
    }

    let touches_text_color = spec.text.is_some();
    if touches_text_color {
        return matches!(
            family,
            ContractFamilyId::Label | ContractFamilyId::Button | ContractFamilyId::Icon
        );
    }

    let touches_font_weight = spec.font_weight.is_some();
    if touches_font_weight {
        return matches!(family, ContractFamilyId::Label | ContractFamilyId::Button);
    }

    let touches_text_size = spec.text_size.is_some() || spec.font_scale.is_some();
    if touches_text_size {
        return matches!(family, ContractFamilyId::Label);
    }

    let touches_opacity = spec.opacity.is_some();
    if touches_opacity {
        return matches!(
            family,
            ContractFamilyId::Card
                | ContractFamilyId::Label
                | ContractFamilyId::Button
                | ContractFamilyId::Icon
        );
    }

    true
}

fn touches_hover_state(spec: &tailwind::Spec) -> bool {
    spec.hover_background.is_some()
        || spec.hover_text.is_some()
        || spec.hover_text_outline_color.is_some()
        || spec.hover_text_outline_thickness.is_some()
        || spec.hover_opacity.is_some()
        || spec.hover_scale.is_some()
        || spec.hover_cursor.is_some()
        || spec.hover_margin.any()
        || spec.hover_padding.any()
        || spec.hover_border.any()
        || spec.hover_border_color.is_some()
}

fn touches_group_hover_state(spec: &tailwind::Spec) -> bool {
    spec.group_hover_background.is_some()
        || spec.group_hover_text.is_some()
        || spec.group_hover_text_outline_color.is_some()
        || spec.group_hover_text_outline_thickness.is_some()
        || spec.group_hover_opacity.is_some()
        || spec.group_hover_scale.is_some()
        || spec.group_hover_cursor.is_some()
        || spec.group_hover_margin.any()
        || spec.group_hover_padding.any()
        || spec.group_hover_border.any()
        || spec.group_hover_border_color.is_some()
}

fn is_flow_container_family(family: ContractFamilyId) -> bool {
    matches!(
        family,
        ContractFamilyId::Row
            | ContractFamilyId::Column
            | ContractFamilyId::Inset
            | ContractFamilyId::Card
    )
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
        | ContractNode::Command(_) => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::{
        audit_tailwind_support, ContractTailwindDiagnostic, ContractTailwindDiagnosticReason,
    };
    use crate::contract::{
        ContractButton, ContractCard, ContractCommon, ContractInput, ContractNode, ContractRow,
        ContractTooltip, ContractTree,
    };

    #[test]
    fn flags_unknown_tailwind_tokens() {
        let tree = ContractTree::new(ContractNode::Button(ContractButton {
            common: ContractCommon {
                node_id: "button".into(),
                class: Some("totally-unknown-class".into()),
                ..ContractCommon::new("button")
            },
            label: "Open".into(),
            action_id: None,
            variant: None,
            size: None,
            leading_icon: None,
            trailing_text: None,
            trailing_icon: None,
            selected: false,
            icon_only: false,
        }));

        assert_eq!(
            audit_tailwind_support(&tree),
            vec![ContractTailwindDiagnostic {
                node_id: "button".into(),
                family: "button".into(),
                token: "totally-unknown-class".into(),
                reason: ContractTailwindDiagnosticReason::UnknownToken,
            }]
        );
    }

    #[test]
    fn flags_known_tokens_on_unsupported_families() {
        let tree = ContractTree::new(ContractNode::Input(ContractInput {
            common: ContractCommon {
                node_id: "input".into(),
                class: Some("bg-card".into()),
                ..ContractCommon::new("input")
            },
            value: String::new(),
            action_id: None,
            placeholder: None,
            leading_icon: None,
            width: 220.0,
        }));

        assert_eq!(
            audit_tailwind_support(&tree),
            vec![ContractTailwindDiagnostic {
                node_id: "input".into(),
                family: "input".into(),
                token: "bg-card".into(),
                reason: ContractTailwindDiagnosticReason::UnsupportedFamily,
            }]
        );
    }

    #[test]
    fn flags_hover_tokens_as_unsupported_until_renderer_consumes_them() {
        let tree = ContractTree::new(ContractNode::Button(ContractButton {
            common: ContractCommon {
                node_id: "button".into(),
                class: Some("hover:bg-card group-hover:opacity-80".into()),
                ..ContractCommon::new("button")
            },
            label: "Open".into(),
            action_id: None,
            variant: None,
            size: None,
            leading_icon: None,
            trailing_text: None,
            trailing_icon: None,
            selected: false,
            icon_only: false,
        }));

        assert_eq!(
            audit_tailwind_support(&tree),
            vec![
                ContractTailwindDiagnostic {
                    node_id: "button".into(),
                    family: "button".into(),
                    token: "hover:bg-card".into(),
                    reason: ContractTailwindDiagnosticReason::UnsupportedFamily,
                },
                ContractTailwindDiagnostic {
                    node_id: "button".into(),
                    family: "button".into(),
                    token: "group-hover:opacity-80".into(),
                    reason: ContractTailwindDiagnosticReason::UnsupportedFamily,
                },
            ]
        );
    }

    #[test]
    fn accepts_card_visual_tokens_on_cards() {
        let tree = ContractTree::new(ContractNode::Card(ContractCard {
            common: ContractCommon {
                node_id: "card".into(),
                class: Some("bg-card shadow-md".into()),
                ..ContractCommon::new("card")
            },
            padding_x: 16,
            padding_y: 16,
            children: Vec::new(),
        }));

        assert!(audit_tailwind_support(&tree).is_empty());
    }

    #[test]
    fn flags_text_tokens_on_families_that_do_not_consume_them() {
        let tree = ContractTree::new(ContractNode::Tooltip(ContractTooltip {
            common: ContractCommon {
                node_id: "tooltip".into(),
                class: Some("text-muted-foreground".into()),
                ..ContractCommon::new("tooltip")
            },
            trigger_label: "Hover".into(),
            text: "Tooltip".into(),
            width: 220.0,
            delay_ms: 0,
            placement: crate::runtime_components::TooltipPlacement::Top,
        }));

        assert_eq!(
            audit_tailwind_support(&tree),
            vec![ContractTailwindDiagnostic {
                node_id: "tooltip".into(),
                family: "tooltip".into(),
                token: "text-muted-foreground".into(),
                reason: ContractTailwindDiagnosticReason::UnsupportedFamily,
            }]
        );
    }

    #[test]
    fn accepts_child_layout_tokens_inside_flow_containers() {
        let tree = ContractTree::new(ContractNode::Row(ContractRow {
            common: ContractCommon::new("row"),
            gap: 8.0,
            justify: Default::default(),
            align: Default::default(),
            children: vec![ContractNode::Button(ContractButton {
                common: ContractCommon {
                    node_id: "button".into(),
                    class: Some("grow basis-[50%]".into()),
                    ..ContractCommon::new("button")
                },
                label: "Open".into(),
                action_id: None,
                variant: None,
                size: None,
                leading_icon: None,
                trailing_text: None,
                trailing_icon: None,
                selected: false,
                icon_only: false,
            })],
        }));

        assert!(audit_tailwind_support(&tree).is_empty());
    }

    #[test]
    fn flags_clip_tokens_until_contract_gains_clip_mode() {
        let tree = ContractTree::new(ContractNode::Card(ContractCard {
            common: ContractCommon {
                node_id: "card".into(),
                class: Some("overflow-clip".into()),
                ..ContractCommon::new("card")
            },
            padding_x: 16,
            padding_y: 16,
            children: Vec::new(),
        }));

        assert_eq!(
            audit_tailwind_support(&tree),
            vec![ContractTailwindDiagnostic {
                node_id: "card".into(),
                family: "card".into(),
                token: "overflow-clip".into(),
                reason: ContractTailwindDiagnosticReason::UnsupportedFamily,
            }]
        );
    }
}
