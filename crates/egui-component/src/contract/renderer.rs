use super::*;
use crate::layout::{taffy, tui, TuiBuilderLogic};
use crate::primitives::{draw_swatch, surface_frame, ScrollAreaExt, SurfaceFrame, Swatch};
use crate::runtime_components::{
    AudioPlayback, Button, ButtonLabelWeight, ButtonVariant, Checkbox, CollabCursor, ComponentUi,
    ComponentUiExt, ContextMenu, ControlSize, DialogueHeader, DialogueModal, DragBoard,
    DragBoardItem, DropdownMenu, DropdownMenuAction, DropdownMenuEntry, FileTree, FileTreeNode,
    Hierarchy as HierarchyWidget, HierarchyNode as HierarchyWidgetNode, Icon, Image, Label,
    LabelTone, LabelWeight, NumberInput, Popover, Radio, RadioGroup, RadioOption, Select, Sidebar,
    Slider, Switch, TextInput, Tooltip,
};
use crate::theme::ColorRole;
use crate::ui::{tailwind, tokens, twemoji};
use egui::{
    Align, Align2, Color32, CornerRadius, Id, Key, Layout, Margin, Order, Sense, Stroke,
    StrokeKind, Vec2,
};
use std::collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet, HashMap};
use std::hash::{Hash, Hasher};

const SHOWCASE_IMAGE_BYTES: &[u8] = include_bytes!("../../assets/images/showcase-image.png");

pub fn render_tree(ui: &mut egui::Ui, tree: &ContractTree) -> Vec<ContractEvent> {
    let mut ui = ui.components();
    render_component_tree(&mut ui, tree)
}

pub fn render_component_tree(ui: &mut ComponentUi<'_>, tree: &ContractTree) -> Vec<ContractEvent> {
    let mut renderer = FrameRenderer::new();
    renderer.render_node(ui, &tree.root);
    renderer.events
}

struct FrameRenderer {
    events: Vec<ContractEvent>,
    class_spec_cache: HashMap<u64, Option<tailwind::Spec>>,
    effective_layout_cache: HashMap<u64, Option<ContractLayout>>,
    #[cfg(test)]
    style_cache_enabled: bool,
}

#[derive(Debug, Clone, Default)]
struct ResolvedCommonStyle {
    class_spec: Option<tailwind::Spec>,
    layout: Option<ContractLayout>,
}

#[derive(Debug)]
struct MenuActionBinding<'a> {
    item_id: &'a str,
    label: &'a str,
    action_id: Option<&'a ActionId>,
}

#[derive(Debug)]
struct HierarchyBinding<'a> {
    item_id: &'a str,
    label: &'a str,
    action_id: Option<&'a ActionId>,
    open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ToastLifecycleState {
    shown_at: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ToastViewportState {
    items: BTreeMap<String, ToastLifecycleState>,
    suppressed: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy)]
struct ToastPalette {
    fill: Color32,
    stroke: Stroke,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LayoutScopeMode {
    Full,
    TaffyItem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaffyDisplay {
    Flex,
    Grid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ContainerLayoutPlan {
    display: TaffyDisplay,
    direction: ContractDirection,
    justify: ContractJustify,
    align: ContractAlign,
    gap_x: f32,
    gap_y: f32,
    wrap: bool,
    overflow_x: ContractOverflow,
    overflow_y: ContractOverflow,
}

impl FrameRenderer {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            class_spec_cache: HashMap::new(),
            effective_layout_cache: HashMap::new(),
            #[cfg(test)]
            style_cache_enabled: true,
        }
    }

    #[cfg(test)]
    fn with_style_cache_enabled(style_cache_enabled: bool) -> Self {
        let mut renderer = Self::new();
        renderer.style_cache_enabled = style_cache_enabled;
        renderer
    }

    fn resolve_common_style(&mut self, common: &ContractCommon) -> ResolvedCommonStyle {
        #[cfg(test)]
        if !self.style_cache_enabled {
            let class_spec = class_spec(common);
            let layout = effective_layout(common, class_spec.as_ref());
            return ResolvedCommonStyle { class_spec, layout };
        }

        let class_hash = class_inputs_hash(common.class.as_deref(), &common.class_list);
        let class_spec = if let Some(cached) = self.class_spec_cache.get(&class_hash) {
            cached.clone()
        } else {
            let parsed = class_spec(common);
            self.class_spec_cache.insert(class_hash, parsed.clone());
            parsed
        };

        let explicit_layout_hash = contract_layout_hash(common.layout.as_ref());
        let mut layout_hasher = DefaultHasher::new();
        layout_hasher.write_u64(class_hash);
        layout_hasher.write_u64(explicit_layout_hash);
        let layout_hash = layout_hasher.finish();

        let layout = if let Some(cached) = self.effective_layout_cache.get(&layout_hash) {
            cached.clone()
        } else {
            let derived = effective_layout(common, class_spec.as_ref());
            self.effective_layout_cache
                .insert(layout_hash, derived.clone());
            derived
        };

        ResolvedCommonStyle { class_spec, layout }
    }

    fn render_node(&mut self, ui: &mut ComponentUi<'_>, node: &ContractNode) {
        self.render_node_with_layout_mode(ui, node, LayoutScopeMode::Full);
    }

    fn render_node_with_layout_mode(
        &mut self,
        ui: &mut ComponentUi<'_>,
        node: &ContractNode,
        scope_mode: LayoutScopeMode,
    ) {
        let common = node.common();
        if !common.visible {
            return;
        }

        // Common-field execution truth:
        // - visible, enabled, the supported subset of layout, and class/class_list are executed here
        // - slot_classes and common.actions remain metadata-only
        ignore_metadata_only_common_fields(common);

        let style = self.resolve_common_style(common);
        let class_spec = style.class_spec;
        let layout = style.layout;

        if common.enabled {
            with_layout_scope(ui, layout.as_ref(), scope_mode, |ui| {
                self.render_node_inner(ui, node, class_spec.as_ref(), layout.as_ref());
            });
        } else {
            ui.ui_mut().add_enabled_ui(false, |ui| {
                let mut ui = ui.components();
                with_layout_scope(&mut ui, layout.as_ref(), scope_mode, |ui| {
                    self.render_node_inner(ui, node, class_spec.as_ref(), layout.as_ref());
                });
            });
        }
    }

    fn render_node_inner(
        &mut self,
        ui: &mut ComponentUi<'_>,
        node: &ContractNode,
        class_spec: Option<&tailwind::Spec>,
        layout: Option<&ContractLayout>,
    ) {
        match node {
            ContractNode::Row(props) => self.render_row(ui, props, layout),
            ContractNode::Column(props) => self.render_column(ui, props, layout),
            ContractNode::Inset(props) => self.render_inset(ui, props, layout),
            ContractNode::SizedBox(props) => self.render_sized_box(ui, props),
            ContractNode::Spacer(props) => self.render_spacer(ui, props),
            ContractNode::Card(props) => self.render_card(ui, props, class_spec, layout),
            ContractNode::Sidebar(props) => self.render_sidebar(ui, props),
            ContractNode::Toolbar(props) => self.render_toolbar(ui, props),
            ContractNode::MenuBar(props) => self.render_menu_bar(ui, props),
            ContractNode::Tabs(props) => self.render_tabs(ui, props),
            ContractNode::Label(props) => self.render_label(ui, props, class_spec),
            ContractNode::Button(props) => self.render_button(ui, props, class_spec),
            ContractNode::ButtonGroup(props) => self.render_button_group(ui, props),
            ContractNode::Input(props) => self.render_input(ui, props),
            ContractNode::NumberInput(props) => self.render_number_input(ui, props),
            ContractNode::Checkbox(props) => self.render_checkbox(ui, props),
            ContractNode::Switch(props) => self.render_switch(ui, props),
            ContractNode::Select(props) => self.render_select(ui, props),
            ContractNode::Field(props) => self.render_field(ui, props),
            ContractNode::Separator(_) => {
                let _ = ui.separator();
            }
            ContractNode::Collapsible(props) => self.render_collapsible(ui, props),
            ContractNode::DialogueModal(props) => self.render_dialogue_modal(ui, props),
            ContractNode::Hierarchy(props) => self.render_hierarchy(ui, props),
            ContractNode::Spinner(props) => {
                let _ = ui
                    .ui_mut()
                    .add(egui::Spinner::new().size(props.size.max(1.0)));
            }
            ContractNode::Progress(props) => {
                let _ = ui.ui_mut().add_sized(
                    egui::vec2(props.width.max(1.0), props.height.max(2.0)),
                    egui::ProgressBar::new(props.value.clamp(0.0, 1.0)),
                );
            }
            ContractNode::ToastViewport(props) => self.render_toast_viewport(ui, props),
            ContractNode::Color(props) => self.render_color(ui, props),
            ContractNode::Icon(props) => self.render_icon(ui, props, class_spec),
            ContractNode::Image(props) => self.render_image(ui, props),
            ContractNode::Twemoji(props) => self.render_twemoji(ui, props),
            ContractNode::Kbd(props) => self.render_kbd(ui, props),
            ContractNode::Skeleton(props) => self.render_skeleton(ui, props),
            ContractNode::Slider(props) => self.render_slider(ui, props),
            ContractNode::Radio(props) => self.render_radio(ui, props),
            ContractNode::RadioGroup(props) => self.render_radio_group(ui, props),
            ContractNode::Combobox(props) => self.render_combobox(ui, props),
            ContractNode::EmojiSelector(props) => self.render_emoji_selector(ui, props),
            ContractNode::Pagination(props) => self.render_pagination(ui, props),
            ContractNode::Tooltip(props) => self.render_tooltip(ui, props),
            ContractNode::Popover(props) => self.render_popover(ui, props),
            ContractNode::DropdownMenu(props) => self.render_dropdown_menu(ui, props),
            ContractNode::ContextMenu(props) => self.render_context_menu(ui, props),
            ContractNode::OpenWith(props) => self.render_open_with(ui, props),
            ContractNode::CollabCursor(props) => self.render_collab_cursor(ui, props),
            ContractNode::IconToolbar(props) => self.render_icon_toolbar(ui, props),
            ContractNode::FileTree(props) => self.render_file_tree(ui, props),
            ContractNode::DragBoard(props) => self.render_drag_board(ui, props),
            ContractNode::AudioPlayback(props) => self.render_audio_playback(ui, props),
            ContractNode::ImageTile(props) => self.render_image_tile(ui, props),
            ContractNode::Command(props) => self.render_command(ui, props),
        }
    }

    fn render_children(&mut self, ui: &mut egui::Ui, children: &[ContractNode]) {
        let mut ui = ui.components();
        for child in children {
            self.render_node(&mut ui, child);
        }
    }

    fn render_taffy_container(
        &mut self,
        ui: &mut egui::Ui,
        common: &ContractCommon,
        children: &[ContractNode],
        layout: Option<&ContractLayout>,
        default_direction: ContractDirection,
        default_gap: f32,
        default_justify: ContractJustify,
        default_align: ContractAlign,
    ) {
        let plan = container_layout_plan(
            layout,
            default_direction,
            default_gap,
            default_justify,
            default_align,
        );
        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(ui.available_width().max(0.0)),
            height: taffy::AvailableSpace::Definite(ui.available_height().max(0.0)),
        };

        tui(ui, make_id(&common.node_id, "layout"))
            .with_available_space(available_space)
            .style(container_taffy_style(layout, plan))
            .show(|tui| {
                for child in children {
                    self.render_taffy_child(tui, child);
                }
            });
    }

    fn render_taffy_child(&mut self, tui: &mut crate::layout::Tui, child: &ContractNode) {
        let common = child.common();
        let style = self.resolve_common_style(common);
        let class_spec = style.class_spec;
        let layout = style.layout;
        let builder = tui
            .id(common.node_id.as_str())
            .style(taffy_item_style(child, layout.as_ref()));

        if matches!(child, ContractNode::Spacer(_)) {
            builder.add_empty();
            return;
        }

        if let ContractNode::Label(props) = child {
            builder.ui_manual(|ui, _container| {
                let (response, outer_size) = if common.enabled {
                    self.render_taffy_label_item(ui, props, class_spec.as_ref(), layout.as_ref())
                } else {
                    ui.add_enabled_ui(false, |ui| {
                        self.render_taffy_label_item(
                            ui,
                            props,
                            class_spec.as_ref(),
                            layout.as_ref(),
                        )
                    })
                    .inner
                };
                let response_size = response.rect.size();
                let padding = egui::vec2(
                    (outer_size.x - response_size.x).max(0.0),
                    (outer_size.y - response_size.y).max(0.0),
                );
                let intrinsic_size = response.intrinsic_size.map(|intrinsic| {
                    egui::vec2(
                        intrinsic.x + padding.x,
                        intrinsic.y.max(response_size.y) + padding.y,
                    )
                });
                let max_size = intrinsic_size.unwrap_or(outer_size).max(outer_size);
                crate::layout::TuiContainerResponse {
                    inner: (),
                    min_size: outer_size,
                    intrinsic_size,
                    max_size,
                    infinite: egui::Vec2b::FALSE,
                }
            });
            return;
        }

        builder.ui_manual(|ui, _container| {
            let mut ui = ui.components();
            self.render_node_with_layout_mode(&mut ui, child, LayoutScopeMode::TaffyItem);
            crate::layout::TuiContainerResponse {
                inner: (),
                min_size: ui.ui().min_size(),
                intrinsic_size: None,
                max_size: ui.ui().min_size(),
                infinite: egui::Vec2b::FALSE,
            }
        });
    }

    fn render_taffy_label_item(
        &mut self,
        ui: &mut egui::Ui,
        props: &ContractLabel,
        class_spec: Option<&tailwind::Spec>,
        layout: Option<&ContractLayout>,
    ) -> (egui::Response, egui::Vec2) {
        if let Some(frame) = layout_frame(layout, LayoutScopeMode::TaffyItem) {
            let rendered = frame.show(ui, |ui| {
                let mut components = ui.components();
                self.draw_contract_label(&mut components, props, class_spec)
            });
            return (rendered.inner, rendered.response.rect.size());
        }

        ui.scope(|ui| {
            let mut components = ui.components();
            let response = self.draw_contract_label(&mut components, props, class_spec);
            (response, ui.min_rect().size())
        })
        .inner
    }

    fn render_row(
        &mut self,
        ui: &mut ComponentUi<'_>,
        props: &ContractRow,
        layout: Option<&ContractLayout>,
    ) {
        let _ = ui.ui_mut().scope(|ui| {
            self.render_taffy_container(
                ui,
                &props.common,
                &props.children,
                layout,
                ContractDirection::Row,
                props.gap,
                props.justify,
                props.align,
            );
        });
    }

    fn render_column(
        &mut self,
        ui: &mut ComponentUi<'_>,
        props: &ContractColumn,
        layout: Option<&ContractLayout>,
    ) {
        let _ = ui.ui_mut().scope(|ui| {
            self.render_taffy_container(
                ui,
                &props.common,
                &props.children,
                layout,
                ContractDirection::Column,
                props.gap,
                props.justify,
                props.align,
            );
        });
    }

    fn render_inset(
        &mut self,
        ui: &mut ComponentUi<'_>,
        props: &ContractInset,
        layout: Option<&ContractLayout>,
    ) {
        let _ = egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(
                props.padding_x as i8,
                props.padding_y as i8,
            ))
            .show(ui.ui_mut(), |ui| {
                self.render_taffy_container(
                    ui,
                    &props.common,
                    &props.children,
                    layout,
                    ContractDirection::Column,
                    0.0,
                    ContractJustify::Start,
                    ContractAlign::Start,
                );
            });
    }

    fn render_sized_box(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSizedBox) {
        let _ = ui.ui_mut().scope(|ui| {
            if let Some(width) = props.width {
                ui.set_min_width(width);
                ui.set_max_width(width);
            }
            if let Some(height) = props.height {
                ui.set_min_height(height);
                ui.set_max_height(height);
            }
            self.render_children(ui, &props.children);
        });
    }

    fn render_spacer(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSpacer) {
        let ui = ui.ui_mut();
        let size = if props.flex {
            match ui.layout().main_dir() {
                egui::Direction::LeftToRight | egui::Direction::RightToLeft => {
                    egui::vec2(ui.available_width().max(0.0), props.height.unwrap_or(0.0))
                }
                egui::Direction::TopDown | egui::Direction::BottomUp => {
                    egui::vec2(props.width.unwrap_or(0.0), ui.available_height().max(0.0))
                }
            }
        } else {
            egui::vec2(props.width.unwrap_or(0.0), props.height.unwrap_or(0.0))
        };
        let _ = ui.allocate_exact_size(size, egui::Sense::hover());
    }

    fn render_card(
        &mut self,
        ui: &mut ComponentUi<'_>,
        props: &crate::contract::ContractCard,
        class_spec: Option<&tailwind::Spec>,
        layout: Option<&ContractLayout>,
    ) {
        let runtime = crate::theme::runtime_for_ui(ui.raw());
        let default_corner_radius = CornerRadius::same(tokens::radius_lg(runtime));
        let _ = compat_card_frame(
            ui,
            class_background_color(class_spec, runtime),
            class_border_stroke(class_spec, runtime),
            class_corner_radius(class_spec, default_corner_radius),
            class_shadow(class_spec, runtime),
            props.padding_x as i8,
            props.padding_y as i8,
            |ui| {
                self.render_taffy_container(
                    ui,
                    &props.common,
                    &props.children,
                    layout,
                    ContractDirection::Column,
                    0.0,
                    ContractJustify::Start,
                    ContractAlign::Start,
                );
            },
        );
    }

    fn render_sidebar(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSidebar) {
        let mut open = props.open;
        let mut sidebar = Sidebar::new(make_id(&props.common.node_id, "sidebar"))
            .side(compat_sidebar_side(props.side))
            .width(props.width);
        if let Some(title) = props.title.as_deref() {
            sidebar = sidebar.title(title);
        }
        ui.sidebar(&mut open, sidebar, |ui, _close_requested| {
            self.render_children(ui, &props.children);
        });
        if props.open && !open {
            self.emit_basic(&props.common.node_id, EventKind::Closed, None);
        }
    }

    fn render_toolbar(&mut self, ui: &mut ComponentUi<'_>, props: &ContractToolbar) {
        let runtime = crate::theme::runtime_for_ui(ui.raw());
        let parent_rect = ui.ui_mut().max_rect();
        let anchor = map_anchor(props.anchor);
        let anchor_pos =
            compat_anchored_pos(parent_rect, anchor) + egui::vec2(props.offset_x, props.offset_y);
        let _ = egui::Area::new(make_id(&props.common.node_id, "toolbar_area"))
            .order(Order::Foreground)
            .pivot(anchor)
            .fixed_pos(anchor_pos)
            .constrain_to(parent_rect)
            .fade_in(false)
            .show(ui.ctx(), |ui| {
                let _ = surface_frame(
                    ui,
                    SurfaceFrame::new(
                        tokens::card_background(runtime),
                        Stroke::new(1.0, tokens::separator(runtime)),
                    )
                    .corner_radius(crate::theme::radius(ui, crate::theme::RadiusRole::Xl))
                    .padding(8, 6)
                    .shadow(tokens::tailwind_shadow_sm(runtime)),
                    |ui| {
                        ui.scope(|ui| {
                            ui.spacing_mut().item_spacing.x = tokens::LAYOUT_GAP_XS;
                            ui.horizontal(|ui| {
                                let mut components = ui.components();
                                self.render_children(components.ui_mut(), &props.children);
                            })
                            .inner
                        })
                        .inner
                    },
                );
            });
    }

    fn render_menu_bar(&mut self, ui: &mut ComponentUi<'_>, props: &ContractMenuBar) {
        let mut selected = None;
        ui.ui_mut().horizontal(|ui| {
            for menu in &props.menus {
                let menu_id = make_item_id(&props.common.node_id, "menu", menu.menu_id.as_str());
                let _ = ui.push_id(menu_id, |ui| {
                    ui.menu_button(menu.label.as_str(), |ui| {
                        ui.set_min_width(menu.width.max(160.0));
                        render_contract_menu_entries(ui, menu.entries.as_slice(), &mut selected);
                    });
                });
            }
        });

        if let Some(binding) = selected {
            self.emit_item(
                &props.common.node_id,
                EventKind::CommandInvoked,
                binding.action_id,
                binding.item_id,
                binding.label,
            );
        }
    }

    fn render_tabs(&mut self, ui: &mut ComponentUi<'_>, props: &ContractTabs) {
        if props.items.is_empty() {
            return;
        }

        let mut current = props
            .selected_item_id
            .as_deref()
            .and_then(|item_id| props.items.iter().position(|item| item.item_id == item_id))
            .unwrap_or(0);
        let previous = current;
        let vertical = matches!(
            props.style,
            ContractTabsStyle::Stacked | ContractTabsStyle::Rail
        );
        let gap = match props.style {
            ContractTabsStyle::BlenderTopbar => 1.0,
            ContractTabsStyle::Stacked => 8.0,
            ContractTabsStyle::Rail => 6.0,
            ContractTabsStyle::Segmented => 4.0,
            ContractTabsStyle::Underline => 6.0,
        };
        let draw = |ui: &mut egui::Ui, current: &mut usize| {
            let mut components = ui.components();
            for (index, item) in props.items.iter().enumerate() {
                let selected = *current == index;
                let mut button = if item.icon_only {
                    Button::icon_only(item.icon.as_deref().unwrap_or("circle"))
                } else {
                    Button::new(item.label.as_str())
                };
                if let Some(icon) = item.icon.as_deref().filter(|_| !item.icon_only) {
                    button = button.leading_icon(icon);
                }
                button = button
                    .id(make_item_id(
                        &props.common.node_id,
                        "tab",
                        item.item_id.as_str(),
                    ))
                    .variant(match props.style {
                        ContractTabsStyle::Underline => {
                            if selected {
                                ButtonVariant::Link
                            } else {
                                ButtonVariant::Ghost
                            }
                        }
                        _ => {
                            if selected {
                                ButtonVariant::Secondary
                            } else {
                                ButtonVariant::Ghost
                            }
                        }
                    })
                    .selected(selected);
                if vertical {
                    button = button.min_size(egui::vec2(96.0, 40.0));
                }
                if components.button(button).clicked() {
                    *current = index;
                }
            }
        };

        if vertical {
            ui.ui_mut().scope(|ui| {
                ui.spacing_mut().item_spacing.y = gap;
                ui.vertical(|ui| draw(ui, &mut current)).inner
            });
        } else {
            ui.ui_mut().scope(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                ui.horizontal(|ui| draw(ui, &mut current)).inner
            });
        }

        if current != previous {
            if let Some(item) = props.items.get(current) {
                self.emit_item(
                    &props.common.node_id,
                    EventKind::Selected,
                    item.action_id.as_ref(),
                    item.item_id.as_str(),
                    item.label.as_str(),
                );
            }
        }
    }

    fn render_label(
        &mut self,
        ui: &mut ComponentUi<'_>,
        props: &crate::contract::ContractLabel,
        class_spec: Option<&tailwind::Spec>,
    ) {
        let _ = self.draw_contract_label(ui, props, class_spec);
    }

    fn draw_contract_label(
        &mut self,
        ui: &mut ComponentUi<'_>,
        props: &crate::contract::ContractLabel,
        class_spec: Option<&tailwind::Spec>,
    ) -> egui::Response {
        let mut label = Label::new(props.text.as_str());
        let runtime = crate::theme::runtime_for_ui(ui.raw());
        if props.tone.is_none() {
            if let Some(color) = class_text_color(class_spec, runtime) {
                label = label.color(color);
            }
        }
        if props.weight.is_none() {
            if let Some(weight) = class_label_weight(class_spec) {
                label = label.weight(weight);
            }
        }
        if props.size.is_none() {
            if let Some(size) = class_text_size(class_spec) {
                label = label.size(size);
            }
        }
        if let Some(tone) = props.tone {
            label = label.tone(compat_label_tone(tone));
        }
        if let Some(weight) = props.weight {
            label = label.weight(compat_label_weight(weight));
        }
        if let Some(size) = props.size {
            label = label.size(size);
        }
        if props.truncate {
            label = label.truncate();
        }
        ui.label(label)
    }

    fn render_button(
        &mut self,
        ui: &mut ComponentUi<'_>,
        props: &ContractButton,
        class_spec: Option<&tailwind::Spec>,
    ) {
        let mut button = if props.icon_only {
            if let Some(icon) = props.leading_icon.as_deref() {
                Button::icon_only(icon)
            } else {
                Button::new(props.label.as_str())
            }
        } else {
            Button::new(props.label.as_str())
        };

        let runtime = crate::theme::runtime_for_ui(ui.raw());
        if let Some(label_color) = class_text_color(class_spec, runtime) {
            button = button.label_color(label_color);
        }
        if let Some(label_weight) = class_button_label_weight(class_spec) {
            button = button.label_weight(label_weight);
        }

        if let Some(variant) = props.variant {
            button = button.variant(compat_button_variant(variant));
        }
        if let Some(size) = props.size {
            button = button.size(compat_control_size(size));
        }
        if let Some(icon) = props.leading_icon.as_deref() {
            if !props.icon_only {
                button = button.leading_icon(icon);
            }
        }
        if let Some(trailing_text) = props.trailing_text.as_deref() {
            button = button.trailing_text(trailing_text);
        }
        if let Some(trailing_icon) = props.trailing_icon.as_deref() {
            button = button.trailing_icon(trailing_icon);
        }
        if props.selected {
            button = button.selected(true);
        }

        if ui.button(button).clicked() {
            self.emit_basic(
                &props.common.node_id,
                EventKind::Clicked,
                props.action_id.as_ref(),
            );
        }
    }

    fn render_button_group(&mut self, ui: &mut ComponentUi<'_>, props: &ContractButtonGroup) {
        if props.items.is_empty() {
            return;
        }

        let _ = compat_card_frame(ui, None, None, None, None, 4, 4, |ui| {
            ui.scope(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.horizontal(|ui| {
                    let mut components = ui.components();
                    for item in &props.items {
                        if components
                            .button(Button::new(item.label.as_str()).variant(ButtonVariant::Ghost))
                            .clicked()
                        {
                            self.emit_item(
                                &props.common.node_id,
                                EventKind::CommandInvoked,
                                item.action_id.as_ref(),
                                item.item_id.as_str(),
                                item.label.as_str(),
                            );
                        }
                    }
                })
                .inner
            })
            .inner
        });
    }

    fn render_input(&mut self, ui: &mut ComponentUi<'_>, props: &ContractInput) {
        let mut value = props.value.clone();
        let mut input = TextInput::new()
            .id(make_id(&props.common.node_id, "input"))
            .width(props.width);
        if let Some(placeholder) = props.placeholder.as_deref() {
            input = input.placeholder(placeholder);
        }
        if let Some(icon) = props.leading_icon.as_deref() {
            input = input.leading_icon(icon);
        }

        let response = ui.text_input(&mut value, input);
        if value != props.value {
            self.emit_value(
                &props.common.node_id,
                EventKind::Changed,
                props.action_id.as_ref(),
                Some(EventValue::Text(value.clone())),
                None,
            );
        }
        if response.lost_focus() && ui.ui().input(|input| input.key_pressed(Key::Enter)) {
            self.emit_value(
                &props.common.node_id,
                EventKind::Submitted,
                props.action_id.as_ref(),
                Some(EventValue::Text(value)),
                None,
            );
        }
    }

    fn render_number_input(&mut self, ui: &mut ComponentUi<'_>, props: &ContractNumberInput) {
        let mut value = props.value;
        let mut number = NumberInput::new(make_id(&props.common.node_id, "number_input"))
            .width(props.width)
            .range(props.min..=props.max)
            .speed(props.speed)
            .decimals(props.decimals);
        if let Some(prefix) = props.prefix.as_deref() {
            number = number.prefix(prefix);
        }
        if let Some(suffix) = props.suffix.as_deref() {
            number = number.suffix(suffix);
        }
        if let Some(axis) = props.axis {
            number = number.axis(compat_number_input_axis(axis));
        }

        let _ = ui.number_input(&mut value, number);
        if (value - props.value).abs() > f32::EPSILON {
            self.emit_value(
                &props.common.node_id,
                EventKind::Changed,
                props.action_id.as_ref(),
                Some(EventValue::Number(value)),
                None,
            );
        }
    }

    fn render_checkbox(&mut self, ui: &mut ComponentUi<'_>, props: &ContractCheckbox) {
        let mut value = props.value;
        let checkbox = match props.label.as_deref() {
            Some(label) => Checkbox::new()
                .id(make_id(&props.common.node_id, "checkbox"))
                .label(label),
            None => Checkbox::new().id(make_id(&props.common.node_id, "checkbox")),
        };
        let _ = ui.checkbox(&mut value, checkbox);
        if value != props.value {
            self.emit_value(
                &props.common.node_id,
                EventKind::Toggled,
                props.action_id.as_ref(),
                Some(EventValue::Boolean(value)),
                None,
            );
        }
    }

    fn render_switch(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSwitch) {
        let mut value = props.value;
        let mut switch = Switch::new().id(make_id(&props.common.node_id, "switch"));
        if let Some(label) = props.label.as_deref() {
            switch = switch.label(label);
        }
        if let Some(size) = props.size {
            switch = switch.size(compat_control_size(size));
        }
        let _ = ui.switch(&mut value, switch);
        if value != props.value {
            self.emit_value(
                &props.common.node_id,
                EventKind::Toggled,
                props.action_id.as_ref(),
                Some(EventValue::Boolean(value)),
                None,
            );
        }
    }

    fn render_select(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSelect) {
        if props.items.is_empty() {
            return;
        }

        let labels = props
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        let mut selected_index = props
            .selected_item_id
            .as_deref()
            .and_then(|item_id| props.items.iter().position(|item| item.item_id == item_id));
        let previous = selected_index;
        let mut select =
            Select::from_id(make_id(&props.common.node_id, "select"), labels.as_slice());
        if let Some(width) = props.width {
            select = select.width(width);
        }
        if let Some(placeholder) = props.placeholder.as_deref() {
            select = select.placeholder(placeholder);
        }
        if let Some(variant) = props.variant {
            select = select.variant(compat_select_variant(variant));
        }
        if let Some(icon) = props.leading_icon.as_deref() {
            select = select.leading_icon(icon);
        }
        let _ = ui.select(&mut selected_index, select);

        if selected_index != previous {
            if let Some(index) = selected_index {
                if let Some(item) = props.items.get(index) {
                    self.emit_item(
                        &props.common.node_id,
                        EventKind::Selected,
                        item.action_id.as_ref().or(props.action_id.as_ref()),
                        item.item_id.as_str(),
                        item.label.as_str(),
                    );
                }
            }
        }
    }

    fn render_field(&mut self, ui: &mut ComponentUi<'_>, props: &ContractField) {
        let mut value = props.value.clone();
        ui.ui_mut().vertical(|ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new(props.label.as_str())
                    .tone(LabelTone::Secondary)
                    .weight(LabelWeight::Semibold),
            );
            let response = components.text_input(
                &mut value,
                TextInput::new()
                    .id(make_id(&props.common.node_id, "field_input"))
                    .width(props.width)
                    .placeholder(props.placeholder.as_deref().unwrap_or_default()),
            );
            if let Some(helper_text) = props.helper_text.as_deref() {
                components.add_space(4.0);
                let _ = components.label(
                    Label::new(helper_text)
                        .tone(LabelTone::Muted)
                        .weight(LabelWeight::Regular)
                        .size(12.0),
                );
            }
            if value != props.value {
                self.emit_value(
                    &props.common.node_id,
                    EventKind::Changed,
                    props.action_id.as_ref(),
                    Some(EventValue::Text(value.clone())),
                    None,
                );
            }
            let _ = response;
        });
    }

    fn render_collapsible(&mut self, ui: &mut ComponentUi<'_>, props: &ContractCollapsible) {
        let mut toggled = None;
        let _ = compat_card_frame(ui, None, None, None, None, 12, 12, |ui| {
            let mut components = ui.components();
            let leading_icon = props.leading_icon.as_deref().unwrap_or(if props.open {
                "chevron-down"
            } else {
                "chevron-right"
            });
            let mut trigger = Button::new(props.title.as_str())
                .variant(ButtonVariant::Ghost)
                .selected(props.open)
                .leading_icon(leading_icon);
            if let Some(trailing_icon) = props.trailing_icon.as_deref() {
                trigger = trigger.trailing_icon(trailing_icon);
            }
            if components.button(trigger).clicked() {
                toggled = Some(!props.open);
            }
            if props.open {
                components.add_space(8.0);
                self.render_children(components.ui_mut(), &props.children);
            }
        });

        if let Some(open) = toggled {
            self.emit_value(
                &props.common.node_id,
                EventKind::Toggled,
                props.action_id.as_ref(),
                Some(EventValue::Boolean(open)),
                None,
            );
            self.emit_basic(
                &props.common.node_id,
                if open {
                    EventKind::Opened
                } else {
                    EventKind::Closed
                },
                props.action_id.as_ref(),
            );
        }
    }

    fn render_dialogue_modal(&mut self, ui: &mut ComponentUi<'_>, props: &ContractDialogueModal) {
        let mut open = props.open;
        let mut confirmed = false;
        let mut cancelled = false;
        ui.dialogue_modal(
            &mut open,
            DialogueModal::new(make_id(&props.common.node_id, "dialogue")).width(props.width),
            |ui, close_requested| {
                let mut ui = ui.components();
                ui.dialogue_header_with_close(
                    DialogueHeader::new(props.title.as_str())
                        .description(props.description.as_deref().unwrap_or_default())
                        .intent(compat_dialogue_intent(
                            props.intent.unwrap_or(super::DialogueIntent::Default),
                        )),
                    close_requested,
                );

                if !props.children.is_empty() {
                    ui.add_space(12.0);
                    self.render_children(ui.ui_mut(), &props.children);
                }

                ui.add_space(12.0);
                let footer_size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                let _ = ui.ui_mut().allocate_ui_with_layout(
                    footer_size,
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        let mut ui = ui.components();
                        let confirm_label = props.confirm_label.as_deref().unwrap_or("Confirm");
                        let cancel_label = props.cancel_label.as_deref().unwrap_or("Cancel");
                        if ui
                            .button(Button::new(confirm_label).variant(ButtonVariant::Primary))
                            .clicked()
                        {
                            *close_requested = true;
                            confirmed = true;
                        }
                        if ui
                            .button(Button::new(cancel_label).variant(ButtonVariant::Secondary))
                            .clicked()
                        {
                            *close_requested = true;
                            cancelled = true;
                        }
                    },
                );
            },
        );

        self.emit_dialogue_events(props, open, confirmed, cancelled);
    }

    fn emit_dialogue_events(
        &mut self,
        props: &ContractDialogueModal,
        open: bool,
        confirmed: bool,
        cancelled: bool,
    ) {
        if confirmed {
            self.emit_basic(
                &props.common.node_id,
                EventKind::Confirmed,
                props.confirm_action_id.as_ref(),
            );
        }
        if cancelled {
            self.emit_basic(
                &props.common.node_id,
                EventKind::Cancelled,
                props.cancel_action_id.as_ref(),
            );
        }
        if should_emit_dialogue_closed(props.open, open, confirmed, cancelled) {
            self.emit_basic(&props.common.node_id, EventKind::Closed, None);
        }
    }

    fn render_toast_viewport(&mut self, ui: &mut ComponentUi<'_>, props: &ContractToastViewport) {
        let ctx = ui.ctx().clone();
        let now = ctx.input(|input| input.time);
        let mut state = load_toast_viewport_state(ui.ui(), &props.common.node_id);
        let current_ids = props
            .toasts
            .iter()
            .map(|toast| toast.item_id.clone())
            .collect::<BTreeSet<_>>();
        state
            .items
            .retain(|item_id, _| current_ids.contains(item_id));
        state
            .suppressed
            .retain(|item_id| current_ids.contains(item_id));

        let mut active_ids = BTreeSet::new();
        let mut next_expiration: Option<f64> = None;
        for toast in &props.toasts {
            if state.suppressed.contains(toast.item_id.as_str()) {
                continue;
            }

            let shown_at = state
                .items
                .entry(toast.item_id.clone())
                .or_insert_with(|| {
                    self.emit_item(
                        &props.common.node_id,
                        EventKind::Opened,
                        toast.action_id.as_ref(),
                        toast.item_id.as_str(),
                        toast.title.as_str(),
                    );
                    ToastLifecycleState { shown_at: now }
                })
                .shown_at;

            if toast.duration_secs > 0.0
                && now >= shown_at + f64::from(toast.duration_secs.max(0.0))
            {
                self.emit_item(
                    &props.common.node_id,
                    EventKind::Closed,
                    toast.action_id.as_ref(),
                    toast.item_id.as_str(),
                    toast.title.as_str(),
                );
                state.suppressed.insert(toast.item_id.clone());
                continue;
            }

            active_ids.insert(toast.item_id.clone());
            if toast.duration_secs > 0.0 {
                let expiry = shown_at + f64::from(toast.duration_secs.max(0.0));
                next_expiration = Some(match next_expiration {
                    Some(current) => current.min(expiry),
                    None => expiry,
                });
            }
        }

        if let Some(expiry) = next_expiration {
            let remaining = (expiry - now).max(0.0) as f32;
            ctx.request_repaint_after_secs(remaining.max(0.05));
        }

        let mut dismiss_ids = Vec::new();
        let visible_toasts = props
            .toasts
            .iter()
            .rev()
            .filter(|toast| active_ids.contains(toast.item_id.as_str()))
            .take(props.max_visible.max(1))
            .collect::<Vec<_>>();

        if !visible_toasts.is_empty() {
            let host_rect = ui.ui().max_rect();
            let width = props.width.max(1.0);
            let gap = props.gap.max(0.0);
            let overlap = props.overlap.max(0.0);
            let margin = egui::vec2(props.margin_x.max(0.0), props.margin_y.max(0.0));

            let _ = egui::Area::new(make_id(&props.common.node_id, "toast_viewport"))
                .order(Order::Foreground)
                .anchor(
                    toast_anchor(props.placement),
                    toast_anchor_offset(props.placement, margin),
                )
                .constrain_to(host_rect)
                .layout(toast_layout(props.placement))
                .show(&ctx, |ui| {
                    ui.set_min_width(width);
                    ui.set_max_width(width);
                    ui.spacing_mut().item_spacing.y = gap;

                    let _ = ui.with_layout(toast_layout(props.placement), |ui| {
                        for (depth, toast) in visible_toasts.iter().enumerate() {
                            let _ =
                                ui.push_id((&props.common.node_id, toast.item_id.as_str()), |ui| {
                                    if draw_contract_toast(ui, toast, width, depth) {
                                        dismiss_ids.push(toast.item_id.clone());
                                    }
                                });

                            if depth + 1 < visible_toasts.len() && overlap > 0.0 {
                                ui.add_space(-(gap + overlap));
                            }
                        }
                    });
                });
        }

        for dismiss_id in dismiss_ids {
            if let Some(toast) = props
                .toasts
                .iter()
                .find(|toast| toast.item_id == dismiss_id)
            {
                self.emit_item(
                    &props.common.node_id,
                    EventKind::Closed,
                    toast.action_id.as_ref(),
                    toast.item_id.as_str(),
                    toast.title.as_str(),
                );
                state.suppressed.insert(toast.item_id.clone());
            }
        }

        store_toast_viewport_state(ui.ui_mut(), &props.common.node_id, state);
    }

    fn render_hierarchy(&mut self, ui: &mut ComponentUi<'_>, props: &ContractHierarchy) {
        let runtime_ids = hierarchy_runtime_ids(&props.items);
        let mut bindings = BTreeMap::new();
        let mut runtime_nodes = build_runtime_hierarchy(&props.items, &runtime_ids, &mut bindings);
        let mut selected_runtime = props
            .selected_item_id
            .as_deref()
            .and_then(|item_id| runtime_ids.get(item_id).copied());
        let previous_selected = selected_runtime;
        let mut hierarchy = HierarchyWidget::new(
            make_id(&props.common.node_id, "hierarchy"),
            &mut runtime_nodes,
        )
        .width(props.width)
        .row_height(props.row_height)
        .indent_width(props.indent_width);
        if let Some(icon_style) = props.icon_style {
            hierarchy = hierarchy.icon_style(compat_hierarchy_icon_style(icon_style));
        }
        if let Some(style) = props.style {
            hierarchy = hierarchy.style(compat_hierarchy_style(style));
        }

        let _ = ui.hierarchy(&mut selected_runtime, hierarchy);

        if selected_runtime != previous_selected {
            match selected_runtime.and_then(|id| bindings.get(&id)) {
                Some(binding) => self.emit_item(
                    &props.common.node_id,
                    EventKind::Selected,
                    binding.action_id.or(props.action_id.as_ref()),
                    binding.item_id,
                    binding.label,
                ),
                None => self.emit_value(
                    &props.common.node_id,
                    EventKind::Selected,
                    props.action_id.as_ref(),
                    None,
                    None,
                ),
            }
        }

        self.emit_hierarchy_open_events(
            &props.common.node_id,
            props.action_id.as_ref(),
            runtime_nodes.as_slice(),
            &bindings,
        );
    }

    fn render_color(&mut self, ui: &mut ComponentUi<'_>, props: &ContractColor) {
        let mut swatch = Swatch::new(parse_contract_color(&props.fill).unwrap_or(Color32::WHITE))
            .size(props.size);
        if let Some(stroke) = props.stroke.as_ref().and_then(parse_contract_stroke) {
            swatch = swatch.stroke(stroke);
        }
        if let Some(corner_radius) = props.corner_radius {
            swatch = swatch.rounded(corner_radius);
        }
        let _ = draw_swatch(ui.raw_mut(), swatch);
    }

    fn render_icon(
        &mut self,
        ui: &mut ComponentUi<'_>,
        props: &ContractIcon,
        class_spec: Option<&tailwind::Spec>,
    ) {
        let mut icon = Icon::new(props.name.as_str()).size(props.size);
        let runtime = crate::theme::runtime_for_ui(ui.raw());
        if let Some(tint) = resolved_icon_tint(props, class_spec, runtime) {
            icon = icon.tint(tint);
        }
        let _ = ui.icon(icon);
    }

    fn render_image(&mut self, ui: &mut ComponentUi<'_>, props: &ContractImage) {
        let mut image = contract_image(props.source.as_str())
            .fit_to_exact_size(egui::vec2(props.width.max(1.0), props.height.max(1.0)));
        if let Some(corner_radius) = props.corner_radius {
            image = image.corner_radius(egui::CornerRadius::same(corner_radius));
        }
        let _ = ui.image(image);
    }

    fn render_twemoji(&mut self, ui: &mut ComponentUi<'_>, props: &ContractTwemoji) {
        if let Some(image) = twemoji::image(props.emoji.as_str(), props.size) {
            let _ = ui.ui_mut().add(image);
        } else {
            let _ = ui.ui_mut().allocate_exact_size(
                egui::vec2(props.size.max(1.0), props.size.max(1.0)),
                Sense::hover(),
            );
        }
    }

    fn render_kbd(&mut self, ui: &mut ComponentUi<'_>, props: &ContractKbd) {
        let _ = draw_compat_keycap(
            ui.ui_mut(),
            props.text.as_str(),
            props.min_width,
            props.height,
            None,
            None,
            None,
            10.0,
            4.0,
        );
    }

    fn render_skeleton(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSkeleton) {
        draw_compat_skeleton(
            ui.ui_mut(),
            props.width,
            props.height,
            props.corner_radius,
            props.circle,
            props.animated,
        );
    }

    fn render_slider(&mut self, ui: &mut ComponentUi<'_>, props: &ContractSlider) {
        let mut value = props.value;
        let _ = ui.slider(
            &mut value,
            Slider::new(props.min..=props.max)
                .id(make_id(&props.common.node_id, "slider"))
                .width(props.width),
        );
        if (value - props.value).abs() > f32::EPSILON {
            self.emit_value(
                &props.common.node_id,
                EventKind::Changed,
                props.action_id.as_ref(),
                Some(EventValue::Number(value)),
                None,
            );
        }
    }

    fn render_radio(&mut self, ui: &mut ComponentUi<'_>, props: &ContractRadio) {
        let mut value = props.value;
        let mut radio = Radio::new().id(make_id(&props.common.node_id, "radio"));
        if let Some(label) = props.label.as_deref() {
            radio = radio.label(label);
        }
        if let Some(description) = props.description.as_deref() {
            radio = radio.description(description);
        }
        let _ = ui.radio(&mut value, radio);
        if value != props.value {
            self.emit_value(
                &props.common.node_id,
                EventKind::Toggled,
                props.action_id.as_ref(),
                Some(EventValue::Boolean(value)),
                None,
            );
        }
    }

    fn render_radio_group(&mut self, ui: &mut ComponentUi<'_>, props: &ContractRadioGroup) {
        let options = props
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let mut option = RadioOption::new(index, item.label.as_str());
                if let Some(description) = item.description.as_deref() {
                    option = option.description(description);
                }
                option
            })
            .collect::<Vec<_>>();
        let mut current = props
            .selected_item_id
            .as_deref()
            .and_then(|item_id| props.items.iter().position(|item| item.item_id == item_id));
        let previous = current;
        let _ = ui.radio_group(
            &mut current,
            RadioGroup::new(
                make_id(&props.common.node_id, "radio_group"),
                options.as_slice(),
            )
            .gap(props.gap),
        );
        if current != previous {
            match current.and_then(|index| props.items.get(index)) {
                Some(item) => self.emit_item(
                    &props.common.node_id,
                    EventKind::Selected,
                    item.action_id.as_ref().or(props.action_id.as_ref()),
                    item.item_id.as_str(),
                    item.label.as_str(),
                ),
                None => self.emit_value(
                    &props.common.node_id,
                    EventKind::Selected,
                    props.action_id.as_ref(),
                    None,
                    None,
                ),
            }
        }
    }

    fn render_combobox(&mut self, ui: &mut ComponentUi<'_>, props: &ContractCombobox) {
        let mut query = props.query.clone();
        let previous_query = query.clone();
        let mut selected_item_ids = props.selected_item_ids.clone();
        let previous_item_ids = selected_item_ids.clone();
        let summary = compat_combobox_summary(selected_item_ids.as_slice(), props);

        let combobox_id = make_id(&props.common.node_id, "combobox");
        let _ = ui.ui_mut().push_id(combobox_id, |ui| {
            ui.menu_button(summary, |ui| {
                ui.set_min_width(props.width.max(180.0));
                if props.searchable {
                    let mut components = ui.components();
                    let _ = components.text_input(
                        &mut query,
                        TextInput::new()
                            .id(make_id(&props.common.node_id, "combobox_query"))
                            .width(props.width.max(180.0))
                            .placeholder(props.filter_placeholder.as_deref().unwrap_or("Filter")),
                    );
                    components.add_space(6.0);
                }

                let query_lower = query.trim().to_ascii_lowercase();
                for item in &props.items {
                    if !query_lower.is_empty()
                        && !item
                            .label
                            .to_ascii_lowercase()
                            .contains(query_lower.as_str())
                    {
                        continue;
                    }

                    let selected = selected_item_ids.iter().any(|value| value == &item.item_id);
                    let label = if selected {
                        format!("✓ {}", item.label)
                    } else {
                        item.label.clone()
                    };
                    let response = ui.push_id(
                        make_item_id(
                            &props.common.node_id,
                            "combobox_item",
                            item.item_id.as_str(),
                        ),
                        |ui| ui.button(label),
                    );
                    if response.inner.clicked() {
                        if selected {
                            selected_item_ids.retain(|value| value != &item.item_id);
                        } else {
                            selected_item_ids.push(item.item_id.clone());
                        }
                    }
                }
            });
        });

        if selected_item_ids != previous_item_ids {
            self.emit_value(
                &props.common.node_id,
                EventKind::Selected,
                props.action_id.as_ref(),
                Some(EventValue::ItemIds(selected_item_ids)),
                None,
            );
        } else if query != previous_query {
            self.emit_value(
                &props.common.node_id,
                EventKind::Changed,
                props.action_id.as_ref(),
                Some(EventValue::Text(query)),
                None,
            );
        }
    }

    fn render_emoji_selector(&mut self, ui: &mut ComponentUi<'_>, props: &ContractEmojiSelector) {
        const COMPAT_EMOJIS: &[&str] = &[
            "😀", "😁", "😂", "🙂", "😍", "😎", "🤔", "😢", "🔥", "✨", "🎉", "🚀",
        ];
        let label = props.placeholder.as_deref().unwrap_or(props.value.as_str());
        let mut next_value = None;
        let _ = ui
            .ui_mut()
            .push_id(make_id(&props.common.node_id, "emoji_selector"), |ui| {
                ui.menu_button(label, |ui| {
                    for emoji in COMPAT_EMOJIS {
                        let response = ui
                            .push_id(make_item_id(&props.common.node_id, "emoji", emoji), |ui| {
                                ui.button(*emoji)
                            });
                        if response.inner.clicked() {
                            next_value = Some((*emoji).to_owned());
                            ui.close();
                        }
                    }
                });
            });
        if let Some(value) = next_value.filter(|value| value != &props.value) {
            self.emit_value(
                &props.common.node_id,
                EventKind::Selected,
                props.action_id.as_ref(),
                Some(EventValue::Text(value)),
                None,
            );
        }
    }

    fn render_pagination(&mut self, ui: &mut ComponentUi<'_>, props: &ContractPagination) {
        let pages = compat_pagination_items(
            props.current_page.max(1),
            props.page_count.max(1),
            props.sibling_count,
        );
        ui.ui_mut().horizontal(|ui| {
            let mut components = ui.components();
            if props.current_page > 1
                && components
                    .button(
                        Button::new("Previous")
                            .variant(ButtonVariant::Ghost)
                            .leading_icon("chevron-left"),
                    )
                    .clicked()
            {
                self.emit_value(
                    &props.common.node_id,
                    EventKind::Selected,
                    props.action_id.as_ref(),
                    Some(EventValue::Number((props.current_page - 1) as f32)),
                    None,
                );
            }
            for page in pages {
                match page {
                    CompatPaginationItem::Ellipsis => {
                        let _ = components.label(
                            Label::new("...")
                                .tone(LabelTone::Muted)
                                .weight(LabelWeight::Regular),
                        );
                    }
                    CompatPaginationItem::Page(page) => {
                        let selected = page == props.current_page;
                        if components
                            .button(
                                Button::new(page.to_string().as_str())
                                    .variant(if selected {
                                        ButtonVariant::Secondary
                                    } else {
                                        ButtonVariant::Ghost
                                    })
                                    .selected(selected),
                            )
                            .clicked()
                        {
                            self.emit_value(
                                &props.common.node_id,
                                EventKind::Selected,
                                props.action_id.as_ref(),
                                Some(EventValue::Number(page as f32)),
                                None,
                            );
                        }
                    }
                }
            }
            if props.current_page < props.page_count
                && components
                    .button(
                        Button::new("Next")
                            .variant(ButtonVariant::Ghost)
                            .trailing_icon("chevron-right"),
                    )
                    .clicked()
            {
                self.emit_value(
                    &props.common.node_id,
                    EventKind::Selected,
                    props.action_id.as_ref(),
                    Some(EventValue::Number((props.current_page + 1) as f32)),
                    None,
                );
            }
        });
    }

    fn render_tooltip(&mut self, ui: &mut ComponentUi<'_>, props: &ContractTooltip) {
        let _ = ui.tooltip(
            Tooltip::new(props.trigger_label.as_str(), props.text.as_str())
                .width(props.width)
                .delay_ms(props.delay_ms)
                .placement(compat_tooltip_placement(props.placement)),
        );
    }

    fn render_popover(&mut self, ui: &mut ComponentUi<'_>, props: &ContractPopover) {
        let mut open = props.open;
        let mut popover = Popover::new(make_id(&props.common.node_id, "popover"))
            .side(compat_popover_side(props.side))
            .align(compat_popover_align(props.align))
            .side_offset(props.side_offset)
            .padding(props.padding_x as i8, props.padding_y as i8);
        if let Some(width) = props.width {
            popover = popover.width(width);
        }
        let trigger_label = props.trigger_label.as_deref().unwrap_or("Open");
        let _ = ui.popover(
            &mut open,
            popover,
            |ui| {
                ui.components()
                    .button(Button::new(trigger_label).variant(ButtonVariant::Secondary))
            },
            |ui, _open| self.render_children(ui, &props.children),
        );
        if open != props.open {
            self.emit_value(
                &props.common.node_id,
                if open {
                    EventKind::Opened
                } else {
                    EventKind::Closed
                },
                props.action_id.as_ref(),
                Some(EventValue::Boolean(open)),
                None,
            );
        }
    }

    fn render_dropdown_menu(&mut self, ui: &mut ComponentUi<'_>, props: &ContractDropdownMenu) {
        let mut action_bindings = Vec::new();
        let mut runtime_entries = Vec::new();
        append_runtime_menu_entries(&props.entries, &mut runtime_entries, &mut action_bindings);
        let mut dropdown = DropdownMenu::new(props.trigger_label.as_str())
            .entries(runtime_entries.as_slice())
            .width(props.width);
        if let Some(variant) = props.trigger_variant {
            dropdown = dropdown.trigger_variant(compat_button_variant(variant));
        }
        let (_, state) = ui
            .ui_mut()
            .push_id(make_id(&props.common.node_id, "dropdown_menu"), |ui| {
                ComponentUi::new(ui).dropdown_menu(dropdown)
            })
            .inner;
        self.emit_menu_action(
            &props.common.node_id,
            props.action_id.as_ref(),
            state.action,
            &action_bindings,
        );
    }

    fn render_context_menu(&mut self, ui: &mut ComponentUi<'_>, props: &ContractContextMenu) {
        let mut action_bindings = Vec::new();
        let mut runtime_entries = Vec::new();
        append_runtime_menu_entries(&props.entries, &mut runtime_entries, &mut action_bindings);
        let (_, state) = ui.context_menu(
            ContextMenu::new(
                make_id(&props.common.node_id, "context_menu"),
                runtime_entries.as_slice(),
            )
            .width(props.width)
            .size(egui::vec2(props.region_width, props.region_height))
            .padding(props.padding_x as i8, props.padding_y as i8),
            |ui| self.render_children(ui, &props.children),
        );
        self.emit_menu_action(
            &props.common.node_id,
            props.action_id.as_ref(),
            state.action,
            &action_bindings,
        );
    }

    fn render_open_with(&mut self, ui: &mut ComponentUi<'_>, props: &ContractOpenWith) {
        let mut action_bindings = Vec::new();
        let mut runtime_entries = Vec::new();
        append_runtime_menu_entries(&props.entries, &mut runtime_entries, &mut action_bindings);
        let trigger_label =
            find_contract_menu_label(props.entries.as_slice(), props.selected_item_id.as_deref())
                .or(props.placeholder.as_deref())
                .unwrap_or("Open With");
        let mut dropdown = DropdownMenu::new(trigger_label)
            .entries(runtime_entries.as_slice())
            .width(props.width);
        if let Some(variant) = props.trigger_variant {
            dropdown = dropdown.trigger_variant(compat_button_variant(variant));
        }
        let (_, state) = ui
            .ui_mut()
            .push_id(make_id(&props.common.node_id, "open_with"), |ui| {
                ComponentUi::new(ui).dropdown_menu(dropdown)
            })
            .inner;
        self.emit_menu_action(
            &props.common.node_id,
            props.action_id.as_ref(),
            state.action,
            &action_bindings,
        );
    }

    fn render_collab_cursor(&mut self, ui: &mut ComponentUi<'_>, props: &ContractCollabCursor) {
        let mut cursor = CollabCursor::new(
            make_id(&props.common.node_id, "collab_cursor"),
            props.name.as_str(),
            egui::pos2(props.x, props.y),
        )
        .size(props.size);
        if let Some(color) = props.color.as_ref().and_then(parse_contract_color) {
            cursor = cursor.color(color);
        }
        let _ = ui.collab_cursor(cursor);
    }

    fn render_icon_toolbar(&mut self, ui: &mut ComponentUi<'_>, props: &ContractIconToolbar) {
        let _ = compat_card_frame(ui, None, None, None, None, 6, 4, |ui| {
            ui.scope(|ui| {
                ui.spacing_mut().item_spacing.x = props.gap;
                ui.horizontal(|ui| {
                    let mut components = ui.components();
                    for item in &props.items {
                        let selected = props
                            .selected_item_id
                            .as_deref()
                            .is_some_and(|item_id| item_id == item.item_id);
                        if components
                            .button(
                                Button::icon_only(item.icon.as_str())
                                    .variant(if selected {
                                        ButtonVariant::Primary
                                    } else {
                                        ButtonVariant::Ghost
                                    })
                                    .selected(selected),
                            )
                            .clicked()
                        {
                            self.emit_item(
                                &props.common.node_id,
                                EventKind::Selected,
                                item.action_id.as_ref().or(props.action_id.as_ref()),
                                item.item_id.as_str(),
                                item.tooltip.as_deref().unwrap_or(item.icon.as_str()),
                            );
                        }
                    }
                })
                .inner
            })
            .inner
        });
    }

    fn render_file_tree(&mut self, ui: &mut ComponentUi<'_>, props: &ContractFileTree) {
        let runtime_ids = file_tree_runtime_ids(&props.items);
        let mut bindings = BTreeMap::new();
        let mut runtime_nodes = build_runtime_file_tree(&props.items, &runtime_ids, &mut bindings);
        let mut selected_id = props
            .selected_item_id
            .as_deref()
            .and_then(|item_id| runtime_ids.get(item_id).copied());
        let previous_selected_id = selected_id;
        let _ = ui.file_tree(
            &mut selected_id,
            FileTree::new(
                make_id(&props.common.node_id, "file_tree"),
                &mut runtime_nodes,
            )
            .width(props.width)
            .row_height(props.row_height)
            .indent_width(props.indent_width),
        );
        if selected_id != previous_selected_id {
            if let Some(binding) = selected_id.and_then(|id| bindings.get(&id)) {
                self.emit_item(
                    &props.common.node_id,
                    EventKind::Selected,
                    binding.action_id.or(props.action_id.as_ref()),
                    binding.item_id,
                    binding.label,
                );
            }
        }
        self.emit_file_tree_open_events(
            &props.common.node_id,
            props.action_id.as_ref(),
            runtime_nodes.as_slice(),
            &bindings,
        );
    }

    fn render_drag_board(&mut self, ui: &mut ComponentUi<'_>, props: &ContractDragBoard) {
        let items =
            props
                .items
                .iter()
                .map(|item| {
                    let mut runtime_item = DragBoardItem::new(item.title.as_str()).id(
                        make_item_id(&props.common.node_id, "drag_item", item.item_id.as_str()),
                    );
                    if let Some(description) = item.description.as_deref() {
                        runtime_item = runtime_item.description(description);
                    }
                    runtime_item
                })
                .collect::<Vec<_>>();
        let mut regions = props
            .items
            .iter()
            .map(|item| compat_drag_board_region(item.region))
            .collect::<Vec<_>>();
        let previous_regions = regions.clone();
        let _ = ui.drag_board(
            regions.as_mut_slice(),
            DragBoard::new(
                make_id(&props.common.node_id, "drag_board"),
                props.left_title.as_str(),
                props.right_title.as_str(),
                items.as_slice(),
            )
            .height(props.height),
        );
        if let Some((index, next_region)) = regions
            .iter()
            .enumerate()
            .find(|(index, region)| previous_regions.get(*index) != Some(*region))
        {
            if let Some(item) = props.items.get(index) {
                self.emit_value(
                    &props.common.node_id,
                    EventKind::Changed,
                    item.action_id.as_ref().or(props.action_id.as_ref()),
                    Some(EventValue::ItemMove(ContractItemMove {
                        item_id: item.item_id.clone(),
                        from: drag_region_id(item.region).to_owned(),
                        to: drag_region_id(contract_drag_board_region(*next_region)).to_owned(),
                    })),
                    Some(EventMetadata::item(
                        item.item_id.as_str(),
                        item.title.as_str(),
                    )),
                );
            }
        }
    }

    fn render_audio_playback(&mut self, ui: &mut ComponentUi<'_>, props: &ContractAudioPlayback) {
        let mut playback = AudioPlayback::new(
            make_id(&props.common.node_id, "audio_playback"),
            compat_audio_playback_state(props.playback_state),
        );
        if let Some(duration_seconds) = props.duration_seconds {
            playback = playback.duration_seconds(duration_seconds);
        }
        let (_, result) = if props.children.is_empty() {
            ui.audio_playback(playback)
        } else {
            ui.audio_playback_with_actions(playback, |ui| {
                self.render_children(ui, &props.children);
            })
        };
        if result.play_pause_clicked {
            self.emit_value(
                &props.common.node_id,
                EventKind::Toggled,
                props.action_id.as_ref(),
                Some(EventValue::Boolean(matches!(
                    props.playback_state,
                    super::AudioPlaybackState::Paused
                ))),
                None,
            );
        }
    }

    fn render_image_tile(&mut self, ui: &mut ComponentUi<'_>, props: &ContractImageTile) {
        let (default_width, default_height) = compat_image_tile_dimensions(props.size);
        let image_width = props.image_width.unwrap_or(default_width);
        let image_height = props.image_height.unwrap_or(default_height);
        let mut play_pause_clicked = false;
        let card = compat_card_frame(ui, None, None, None, None, 10, 10, |ui| {
            let mut components = ui.components();
            let mut image = contract_image(props.source.as_str())
                .fit_to_exact_size(egui::vec2(image_width, image_height));
            if props.image_frame {
                image = image.corner_radius(CornerRadius::same(8));
            }
            let _ = components.image(image);
            if let Some(playback_state) = props.playback_state {
                components.add_space(6.0);
                let icon = match playback_state {
                    ImageTilePlaybackState::Paused => "play",
                    ImageTilePlaybackState::Playing => "pause",
                };
                if components
                    .button(Button::icon_only(icon).variant(ButtonVariant::Primary))
                    .clicked()
                {
                    play_pause_clicked = true;
                }
            }
            if !props.children.is_empty() {
                components.add_space(8.0);
                self.render_children(components.ui_mut(), &props.children);
            }
        });
        if play_pause_clicked {
            self.emit_basic(
                &props.common.node_id,
                EventKind::Toggled,
                props
                    .play_pause_action_id
                    .as_ref()
                    .or(props.action_id.as_ref()),
            );
        } else if card.response.clicked() {
            self.emit_basic(
                &props.common.node_id,
                EventKind::Clicked,
                props.action_id.as_ref(),
            );
        }
    }

    fn render_command(&mut self, ui: &mut ComponentUi<'_>, props: &ContractCommand) {
        let mut query = props.query.clone();
        let previous_query = query.clone();
        let placeholder = props
            .placeholder
            .as_deref()
            .unwrap_or("Execute a command...");

        if props.preview {
            let runtime = crate::theme::runtime_for_ui(ui.raw());
            let _ = compat_card_frame(
                ui,
                Some(tokens::muted_surface(runtime)),
                Some(Stroke::new(1.0, tokens::separator(runtime))),
                None,
                None,
                12,
                12,
                |ui| {
                    ui.set_min_height(props.preview_height.max(1.0));
                    render_contract_command_panel(
                        ui,
                        &props.common.node_id,
                        &mut query,
                        props,
                        placeholder,
                    );
                },
            );
        } else {
            render_contract_command_panel(
                ui.raw_mut(),
                &props.common.node_id,
                &mut query,
                props,
                placeholder,
            );
        }

        if query != previous_query {
            self.emit_value(
                &props.common.node_id,
                EventKind::Changed,
                props.action_id.as_ref(),
                Some(EventValue::Text(query)),
                None,
            );
        }
    }

    fn emit_menu_action(
        &mut self,
        node_id: &NodeId,
        fallback_action: Option<&ActionId>,
        action_index: Option<usize>,
        bindings: &[MenuActionBinding<'_>],
    ) {
        if let Some(binding) = action_index.and_then(|index| bindings.get(index)) {
            self.emit_item(
                node_id,
                EventKind::CommandInvoked,
                binding.action_id.or(fallback_action),
                binding.item_id,
                binding.label,
            );
        }
    }

    fn emit_hierarchy_open_events(
        &mut self,
        node_id: &crate::contract::NodeId,
        hierarchy_action: Option<&ActionId>,
        nodes: &[HierarchyWidgetNode],
        bindings: &BTreeMap<usize, HierarchyBinding<'_>>,
    ) {
        for node in nodes {
            if let Some(binding) = bindings.get(&node.id) {
                if node.expanded != binding.open {
                    self.emit_item(
                        node_id,
                        if node.expanded {
                            EventKind::Opened
                        } else {
                            EventKind::Closed
                        },
                        binding.action_id.or(hierarchy_action),
                        binding.item_id,
                        binding.label,
                    );
                }
                self.emit_hierarchy_open_events(
                    node_id,
                    hierarchy_action,
                    node.children.as_slice(),
                    bindings,
                );
            }
        }
    }

    fn emit_file_tree_open_events(
        &mut self,
        node_id: &NodeId,
        file_tree_action: Option<&ActionId>,
        nodes: &[FileTreeNode<'_>],
        bindings: &BTreeMap<usize, HierarchyBinding<'_>>,
    ) {
        for node in nodes {
            if let Some(binding) = bindings.get(&node.id) {
                if node.expanded != binding.open {
                    self.emit_item(
                        node_id,
                        if node.expanded {
                            EventKind::Opened
                        } else {
                            EventKind::Closed
                        },
                        binding.action_id.or(file_tree_action),
                        binding.item_id,
                        binding.label,
                    );
                }
                self.emit_file_tree_open_events(
                    node_id,
                    file_tree_action,
                    node.children.as_slice(),
                    bindings,
                );
            }
        }
    }

    fn emit_basic(
        &mut self,
        node_id: &crate::contract::NodeId,
        kind: EventKind,
        action: Option<&ActionId>,
    ) {
        self.events.push(
            ContractEvent::new(node_id.clone(), kind)
                .action(action.cloned())
                .value(None)
                .metadata(None),
        );
    }

    fn emit_value(
        &mut self,
        node_id: &crate::contract::NodeId,
        kind: EventKind,
        action: Option<&ActionId>,
        value: Option<EventValue>,
        metadata: Option<EventMetadata>,
    ) {
        self.events.push(
            ContractEvent::new(node_id.clone(), kind)
                .action(action.cloned())
                .value(value)
                .metadata(metadata),
        );
    }

    fn emit_item(
        &mut self,
        node_id: &crate::contract::NodeId,
        kind: EventKind,
        action: Option<&ActionId>,
        item_id: &str,
        label: &str,
    ) {
        self.emit_value(
            node_id,
            kind,
            action,
            Some(EventValue::ItemId(item_id.to_owned())),
            Some(EventMetadata::item(item_id, label)),
        );
    }
}

fn class_spec(common: &ContractCommon) -> Option<tailwind::Spec> {
    class_spec_from_parts(common.class.as_deref(), &common.class_list)
}

fn class_spec_from_parts(class: Option<&str>, class_list: &[String]) -> Option<tailwind::Spec> {
    let mut classes = String::new();
    if let Some(class) = class {
        classes.push_str(class);
    }
    for class in class_list {
        if !classes.is_empty() {
            classes.push(' ');
        }
        classes.push_str(class);
    }
    let classes = classes.trim();
    (!classes.is_empty()).then(|| tailwind::parse(classes))
}

fn class_inputs_hash(class: Option<&str>, class_list: &[String]) -> u64 {
    let mut hasher = DefaultHasher::new();
    class.hash(&mut hasher);
    class_list.len().hash(&mut hasher);
    for class in class_list {
        class.hash(&mut hasher);
    }
    hasher.finish()
}

fn contract_layout_hash(layout: Option<&ContractLayout>) -> u64 {
    let mut hasher = DefaultHasher::new();
    hash_contract_layout(layout, &mut hasher);
    hasher.finish()
}

fn hash_contract_layout<H: Hasher>(layout: Option<&ContractLayout>, state: &mut H) {
    let Some(layout) = layout else {
        state.write_u8(0);
        return;
    };

    state.write_u8(1);
    hash_option_display(layout.display, state);
    hash_option_direction(layout.direction, state);
    hash_option_f32(layout.grow, state);
    hash_option_f32(layout.shrink, state);
    hash_option_length(layout.basis.as_ref(), state);
    hash_option_length(layout.width.as_ref(), state);
    hash_option_length(layout.height.as_ref(), state);
    hash_option_length(layout.min_width.as_ref(), state);
    hash_option_length(layout.min_height.as_ref(), state);
    hash_option_length(layout.max_width.as_ref(), state);
    hash_option_length(layout.max_height.as_ref(), state);
    hash_option_f32(layout.gap_x, state);
    hash_option_f32(layout.gap_y, state);
    hash_option_edges(layout.padding.as_ref(), state);
    hash_option_edges(layout.margin.as_ref(), state);
    hash_option_align(layout.align, state);
    hash_option_justify(layout.justify, state);
    hash_option_bool(layout.wrap, state);

    layout.columns.len().hash(state);
    for column in &layout.columns {
        hash_track(column, state);
    }

    layout.rows.len().hash(state);
    for row in &layout.rows {
        hash_track(row, state);
    }

    hash_option_u16(layout.col_span, state);
    hash_option_u16(layout.row_span, state);
    hash_option_overflow(layout.overflow_x, state);
    hash_option_overflow(layout.overflow_y, state);
}

fn hash_option_display<H: Hasher>(value: Option<ContractDisplay>, state: &mut H) {
    let Some(value) = value else {
        state.write_u8(0);
        return;
    };
    state.write_u8(match value {
        ContractDisplay::Flow => 1,
        ContractDisplay::Flex => 2,
        ContractDisplay::Grid => 3,
        ContractDisplay::Overlay => 4,
    });
}

fn hash_option_direction<H: Hasher>(value: Option<ContractDirection>, state: &mut H) {
    let Some(value) = value else {
        state.write_u8(0);
        return;
    };
    state.write_u8(match value {
        ContractDirection::Row => 1,
        ContractDirection::Column => 2,
    });
}

fn hash_option_align<H: Hasher>(value: Option<ContractAlign>, state: &mut H) {
    let Some(value) = value else {
        state.write_u8(0);
        return;
    };
    state.write_u8(match value {
        ContractAlign::Start => 1,
        ContractAlign::Center => 2,
        ContractAlign::End => 3,
        ContractAlign::Stretch => 4,
    });
}

fn hash_option_justify<H: Hasher>(value: Option<ContractJustify>, state: &mut H) {
    let Some(value) = value else {
        state.write_u8(0);
        return;
    };
    state.write_u8(match value {
        ContractJustify::Start => 1,
        ContractJustify::Center => 2,
        ContractJustify::End => 3,
    });
}

fn hash_option_overflow<H: Hasher>(value: Option<ContractOverflow>, state: &mut H) {
    let Some(value) = value else {
        state.write_u8(0);
        return;
    };
    state.write_u8(match value {
        ContractOverflow::Visible => 1,
        ContractOverflow::Hidden => 2,
        ContractOverflow::Scroll => 3,
    });
}

fn hash_option_u16<H: Hasher>(value: Option<u16>, state: &mut H) {
    match value {
        Some(value) => {
            state.write_u8(1);
            state.write_u16(value);
        }
        None => state.write_u8(0),
    }
}

fn hash_option_bool<H: Hasher>(value: Option<bool>, state: &mut H) {
    match value {
        Some(false) => state.write_u8(1),
        Some(true) => state.write_u8(2),
        None => state.write_u8(0),
    }
}

fn hash_option_f32<H: Hasher>(value: Option<f32>, state: &mut H) {
    match value {
        Some(value) => {
            state.write_u8(1);
            state.write_u32(value.to_bits());
        }
        None => state.write_u8(0),
    }
}

fn hash_option_edges<H: Hasher>(value: Option<&ContractEdges>, state: &mut H) {
    let Some(edges) = value else {
        state.write_u8(0);
        return;
    };
    state.write_u8(1);
    state.write_u32(edges.top.to_bits());
    state.write_u32(edges.right.to_bits());
    state.write_u32(edges.bottom.to_bits());
    state.write_u32(edges.left.to_bits());
}

fn hash_option_length<H: Hasher>(value: Option<&ContractLength>, state: &mut H) {
    let Some(length) = value else {
        state.write_u8(0);
        return;
    };

    state.write_u8(1);
    hash_length(length, state);
}

fn hash_length<H: Hasher>(value: &ContractLength, state: &mut H) {
    match value {
        ContractLength::Auto => state.write_u8(0),
        ContractLength::Px { value } => {
            state.write_u8(1);
            state.write_u32(value.to_bits());
        }
        ContractLength::Percent { value } => {
            state.write_u8(2);
            state.write_u32(value.to_bits());
        }
    }
}

fn hash_track<H: Hasher>(value: &ContractTrack, state: &mut H) {
    match value {
        ContractTrack::Auto => state.write_u8(0),
        ContractTrack::Fr { value } => {
            state.write_u8(1);
            state.write_u32(value.to_bits());
        }
        ContractTrack::Px { value } => {
            state.write_u8(2);
            state.write_u32(value.to_bits());
        }
        ContractTrack::Percent { value } => {
            state.write_u8(3);
            state.write_u32(value.to_bits());
        }
    }
}

fn effective_layout(
    common: &ContractCommon,
    class_spec: Option<&tailwind::Spec>,
) -> Option<ContractLayout> {
    let mut layout = common.layout.clone().unwrap_or_default();
    let mut changed = common.layout.is_some();

    if let Some(spec) = class_spec {
        if layout.display.is_none() {
            if let Some(display) = class_display(spec) {
                layout.display = Some(display);
                changed = true;
            }
        }
        if layout.direction.is_none() {
            if let Some(direction) = spec.direction.map(class_direction) {
                layout.direction = Some(direction);
                changed = true;
            } else if spec.is_flex {
                layout.direction = Some(ContractDirection::Row);
                changed = true;
            }
        }
        if layout.grow.is_none() {
            if let Some(grow) = spec.flex_grow {
                layout.grow = Some(grow);
                changed = true;
            }
        }
        if layout.shrink.is_none() {
            if let Some(shrink) = spec.flex_shrink {
                layout.shrink = Some(shrink);
                changed = true;
            }
        }
        if layout.basis.is_none() {
            if let Some(basis) = spec.flex_basis.and_then(class_flex_basis) {
                layout.basis = Some(basis);
                changed = true;
            }
        }
        if layout.width.is_none() {
            if let Some(width) = spec.width.and_then(class_width) {
                layout.width = Some(width);
                changed = true;
            }
        }
        if layout.height.is_none() {
            if let Some(height) = spec.height.and_then(class_height) {
                layout.height = Some(height);
                changed = true;
            }
        }
        if layout.min_width.is_none() {
            if let Some(min_width) = spec.min_width.and_then(class_width) {
                layout.min_width = Some(min_width);
                changed = true;
            }
        }
        if layout.min_height.is_none() {
            if let Some(min_height) = spec.min_height.and_then(class_height) {
                layout.min_height = Some(min_height);
                changed = true;
            }
        }
        if layout.max_width.is_none() {
            if let Some(max_width) = spec.max_width.and_then(class_width) {
                layout.max_width = Some(max_width);
                changed = true;
            }
        }
        if layout.max_height.is_none() {
            if let Some(max_height) = spec.max_height.and_then(class_height) {
                layout.max_height = Some(max_height);
                changed = true;
            }
        }
        if layout.gap_x.is_none() {
            if let Some(gap) = spec.gap_col {
                layout.gap_x = Some(gap);
                changed = true;
            }
        }
        if layout.gap_y.is_none() {
            if let Some(gap) = spec.gap_row {
                layout.gap_y = Some(gap);
                changed = true;
            }
        }
        if layout.padding.is_none() {
            if let Some(padding) = class_padding_edges(spec.padding) {
                layout.padding = Some(padding);
                changed = true;
            }
        }
        if layout.margin.is_none() {
            if let Some(margin) = class_margin_edges(spec.margin) {
                layout.margin = Some(margin);
                changed = true;
            }
        }
        if layout.align.is_none() {
            if let Some(align) = spec.align_items.map(class_align_items) {
                layout.align = Some(align);
                changed = true;
            } else if spec.is_flex {
                layout.align = Some(ContractAlign::Stretch);
                changed = true;
            }
        }
        if layout.justify.is_none() {
            if let Some(justify) = spec.justify.and_then(class_justify) {
                layout.justify = Some(justify);
                changed = true;
            }
        }
        if layout.wrap.is_none() {
            if let Some(wrap) = spec.flex_wrap.and_then(class_flex_wrap) {
                layout.wrap = Some(wrap);
                changed = true;
            }
        }
        if layout.columns.is_empty() {
            if let Some(columns) = class_grid_tracks(spec.grid_cols) {
                layout.columns = columns;
                changed = true;
            }
        }
        if layout.rows.is_empty() {
            if let Some(rows) = class_grid_tracks(spec.grid_rows) {
                layout.rows = rows;
                changed = true;
            }
        }
        if layout.overflow_x.is_none() {
            if let Some(overflow_x) = class_overflow_x(spec) {
                layout.overflow_x = Some(overflow_x);
                changed = true;
            }
        }
        if layout.overflow_y.is_none() {
            if let Some(overflow_y) = class_overflow_y(spec) {
                layout.overflow_y = Some(overflow_y);
                changed = true;
            }
        }
    }

    changed.then_some(layout)
}

fn class_display(spec: &tailwind::Spec) -> Option<ContractDisplay> {
    if spec.is_grid {
        Some(ContractDisplay::Grid)
    } else if spec.is_flex {
        Some(ContractDisplay::Flex)
    } else {
        None
    }
}

fn class_direction(direction: tailwind::Direction) -> ContractDirection {
    match direction {
        tailwind::Direction::Horizontal => ContractDirection::Row,
        tailwind::Direction::Vertical => ContractDirection::Column,
    }
}

fn class_flex_basis(basis: tailwind::FlexBasis) -> Option<ContractLength> {
    Some(match basis {
        tailwind::FlexBasis::Auto => ContractLength::Auto,
        tailwind::FlexBasis::Full => ContractLength::Percent { value: 1.0 },
        tailwind::FlexBasis::Pixels(value) => ContractLength::Px { value },
        tailwind::FlexBasis::Percent(value) => ContractLength::Percent { value },
    })
}

fn class_grid_tracks(track_count: Option<usize>) -> Option<Vec<ContractTrack>> {
    let count = track_count?;
    (count > 0).then(|| vec![ContractTrack::Fr { value: 1.0 }; count])
}

fn class_align_items(align: tailwind::AlignItems) -> ContractAlign {
    match align {
        tailwind::AlignItems::Start => ContractAlign::Start,
        tailwind::AlignItems::Center => ContractAlign::Center,
        tailwind::AlignItems::End => ContractAlign::End,
        tailwind::AlignItems::Stretch => ContractAlign::Stretch,
    }
}

fn class_justify(justify: tailwind::JustifyContent) -> Option<ContractJustify> {
    match justify {
        tailwind::JustifyContent::Start => Some(ContractJustify::Start),
        tailwind::JustifyContent::Center => Some(ContractJustify::Center),
        tailwind::JustifyContent::End => Some(ContractJustify::End),
        tailwind::JustifyContent::Between | tailwind::JustifyContent::Around => None,
    }
}

fn class_flex_wrap(wrap: tailwind::FlexWrap) -> Option<bool> {
    match wrap {
        tailwind::FlexWrap::NoWrap => Some(false),
        tailwind::FlexWrap::Wrap => Some(true),
        tailwind::FlexWrap::WrapReverse => None,
    }
}

fn class_width(width: tailwind::Width) -> Option<ContractLength> {
    Some(match width {
        tailwind::Width::Full => ContractLength::Percent { value: 1.0 },
        tailwind::Width::Pixels(value) => ContractLength::Px { value },
        tailwind::Width::Percent(value) => ContractLength::Percent { value },
    })
}

fn class_height(height: tailwind::Height) -> Option<ContractLength> {
    Some(match height {
        tailwind::Height::Full => ContractLength::Percent { value: 1.0 },
        tailwind::Height::Pixels(value) => ContractLength::Px { value },
        tailwind::Height::Percent(value) => ContractLength::Percent { value },
    })
}

fn class_overflow_x(spec: &tailwind::Spec) -> Option<ContractOverflow> {
    if spec.scroll_x {
        return Some(ContractOverflow::Scroll);
    }
    spec.clip_children
        .filter(|clip_children| *clip_children)
        .map(|_| ContractOverflow::Hidden)
}

fn class_overflow_y(spec: &tailwind::Spec) -> Option<ContractOverflow> {
    if spec.scroll_y {
        return Some(ContractOverflow::Scroll);
    }
    spec.clip_children
        .filter(|clip_children| *clip_children)
        .map(|_| ContractOverflow::Hidden)
}

fn class_padding_edges(
    edges: tailwind::SideValues<tailwind::PaddingValue>,
) -> Option<ContractEdges> {
    if !edges.any() {
        return None;
    }
    Some(ContractEdges {
        top: class_padding_value(edges.top),
        right: class_padding_value(edges.right),
        bottom: class_padding_value(edges.bottom),
        left: class_padding_value(edges.left),
    })
}

fn class_padding_value(value: Option<tailwind::PaddingValue>) -> f32 {
    match value {
        Some(tailwind::PaddingValue::Pixels(value)) => value.max(0.0),
        Some(tailwind::PaddingValue::Percent(_)) | None => 0.0,
    }
}

fn class_margin_edges(edges: tailwind::SideValues<f32>) -> Option<ContractEdges> {
    if !edges.any() {
        return None;
    }
    Some(ContractEdges {
        top: edges.top.unwrap_or(0.0).max(0.0),
        right: edges.right.unwrap_or(0.0).max(0.0),
        bottom: edges.bottom.unwrap_or(0.0).max(0.0),
        left: edges.left.unwrap_or(0.0).max(0.0),
    })
}

fn class_text_color(
    class_spec: Option<&tailwind::Spec>,
    runtime: crate::theme::ThemeRuntime,
) -> Option<Color32> {
    class_spec
        .and_then(|spec| spec.text)
        .map(|color| tailwind::resolve_color(&runtime, color))
        .map(|color| apply_class_opacity(color, class_spec))
}

fn resolved_icon_tint(
    props: &ContractIcon,
    class_spec: Option<&tailwind::Spec>,
    runtime: crate::theme::ThemeRuntime,
) -> Option<Color32> {
    match props.tint.as_ref() {
        Some(tint) => parse_contract_color(tint),
        None => class_text_color(class_spec, runtime),
    }
}

fn class_background_color(
    class_spec: Option<&tailwind::Spec>,
    runtime: crate::theme::ThemeRuntime,
) -> Option<Color32> {
    match class_spec.and_then(|spec| spec.background) {
        Some(tailwind::UiRuntimeBackground::Solid(color)) => Some(apply_class_opacity(
            tailwind::resolve_color(&runtime, color),
            class_spec,
        )),
        None => None,
    }
}

fn class_border_stroke(
    class_spec: Option<&tailwind::Spec>,
    runtime: crate::theme::ThemeRuntime,
) -> Option<Stroke> {
    let spec = class_spec?;
    let width = class_border_width(spec).or_else(|| spec.border_color.map(|_| 1.0))?;
    let color = spec
        .border_color
        .map(|color| tailwind::resolve_color(&runtime, color))
        .unwrap_or_else(|| tokens::separator(runtime));
    Some(Stroke::new(
        width.max(0.0),
        apply_class_opacity(color, class_spec),
    ))
}

fn class_shadow(
    class_spec: Option<&tailwind::Spec>,
    runtime: crate::theme::ThemeRuntime,
) -> Option<egui::Shadow> {
    let mut shadow = match class_spec.and_then(|spec| spec.shadow) {
        Some(tailwind::SurfaceShadow::Sm) => tokens::tailwind_shadow_sm(runtime),
        Some(tailwind::SurfaceShadow::Md) => tokens::tailwind_shadow_md(runtime),
        Some(tailwind::SurfaceShadow::Lg) => tokens::tailwind_shadow_lg(runtime),
        None => return None,
    };
    shadow.color = apply_class_opacity(shadow.color, class_spec);
    Some(shadow)
}

fn apply_class_opacity(color: Color32, class_spec: Option<&tailwind::Spec>) -> Color32 {
    match class_spec.and_then(|spec| spec.opacity) {
        Some(opacity) => color.gamma_multiply(opacity.clamp(0.0, 1.0)),
        None => color,
    }
}

fn class_border_width(spec: &tailwind::Spec) -> Option<f32> {
    [
        spec.border.top,
        spec.border.right,
        spec.border.bottom,
        spec.border.left,
    ]
    .into_iter()
    .flatten()
    .reduce(f32::max)
}

fn class_corner_radius(
    class_spec: Option<&tailwind::Spec>,
    fallback: CornerRadius,
) -> Option<CornerRadius> {
    let radii = class_spec?.corner_radii;
    radii.any().then(|| CornerRadius {
        nw: radii
            .nw
            .unwrap_or(f32::from(fallback.nw))
            .round()
            .clamp(0.0, 255.0) as u8,
        ne: radii
            .ne
            .unwrap_or(f32::from(fallback.ne))
            .round()
            .clamp(0.0, 255.0) as u8,
        sw: radii
            .sw
            .unwrap_or(f32::from(fallback.sw))
            .round()
            .clamp(0.0, 255.0) as u8,
        se: radii
            .se
            .unwrap_or(f32::from(fallback.se))
            .round()
            .clamp(0.0, 255.0) as u8,
    })
}

fn class_label_weight(class_spec: Option<&tailwind::Spec>) -> Option<LabelWeight> {
    match class_spec.and_then(|spec| spec.font_weight) {
        Some(tailwind::FontWeight::Regular) => Some(LabelWeight::Regular),
        Some(tailwind::FontWeight::Medium | tailwind::FontWeight::Semibold) => {
            Some(LabelWeight::Semibold)
        }
        Some(tailwind::FontWeight::Bold) => Some(LabelWeight::Bold),
        None => None,
    }
}

fn class_button_label_weight(class_spec: Option<&tailwind::Spec>) -> Option<ButtonLabelWeight> {
    match class_spec.and_then(|spec| spec.font_weight) {
        Some(tailwind::FontWeight::Regular) => Some(ButtonLabelWeight::Regular),
        Some(tailwind::FontWeight::Medium | tailwind::FontWeight::Semibold) => {
            Some(ButtonLabelWeight::Medium)
        }
        Some(tailwind::FontWeight::Bold) => Some(ButtonLabelWeight::Bold),
        None => None,
    }
}

fn class_text_size(class_spec: Option<&tailwind::Spec>) -> Option<f32> {
    let spec = class_spec?;
    if let Some(scale) = spec.font_scale {
        return Some((12.0 * scale).max(1.0));
    }
    spec.text_size.map(|size| match size {
        tailwind::TextSize::Xs => 12.0,
        tailwind::TextSize::Sm => 14.0,
        tailwind::TextSize::Base => 16.0,
        tailwind::TextSize::Lg => 18.0,
        tailwind::TextSize::Xl => 20.0,
        tailwind::TextSize::X2l => 24.0,
        tailwind::TextSize::X3l => 30.0,
    })
}

fn with_layout_scope<R>(
    ui: &mut ComponentUi<'_>,
    layout: Option<&ContractLayout>,
    mode: LayoutScopeMode,
    add: impl FnOnce(&mut ComponentUi<'_>) -> R,
) -> R {
    if layout.is_none() {
        return add(ui);
    }

    ui.ui_mut()
        .scope(|ui| {
            let render = |ui: &mut egui::Ui| {
                apply_layout_sizing(ui, layout, mode);
                let mut components = ui.components();
                add(&mut components)
            };
            if let Some(frame) = layout_frame(layout, mode) {
                frame.show(ui, render).inner
            } else {
                render(ui)
            }
        })
        .inner
}

fn layout_frame(layout: Option<&ContractLayout>, mode: LayoutScopeMode) -> Option<egui::Frame> {
    let layout = layout?;
    let include_margin = matches!(mode, LayoutScopeMode::Full);
    if layout.padding.is_none() && (!include_margin || layout.margin.is_none()) {
        return None;
    }
    let mut frame = egui::Frame::new();
    if let Some(padding) = layout.padding {
        frame = frame.inner_margin(edges_to_margin(padding));
    }
    if include_margin {
        if let Some(margin) = layout.margin {
            frame = frame.outer_margin(edges_to_margin(margin));
        }
    }
    Some(frame)
}

fn edges_to_margin(edges: ContractEdges) -> Margin {
    Margin {
        left: edge_to_i8(edges.left),
        right: edge_to_i8(edges.right),
        top: edge_to_i8(edges.top),
        bottom: edge_to_i8(edges.bottom),
    }
}

fn edge_to_i8(value: f32) -> i8 {
    value.round().clamp(0.0, i8::MAX as f32) as i8
}

fn apply_layout_sizing(ui: &mut egui::Ui, layout: Option<&ContractLayout>, mode: LayoutScopeMode) {
    let Some(layout) = layout else {
        return;
    };

    if matches!(mode, LayoutScopeMode::TaffyItem) {
        return;
    }

    if let Some(width) = resolve_contract_length(layout.width.as_ref(), ui.available_width()) {
        ui.set_min_width(width);
        ui.set_max_width(width);
    }
    if let Some(height) = resolve_contract_length(layout.height.as_ref(), ui.available_height()) {
        ui.set_min_height(height);
        ui.set_max_height(height);
    }
    if let Some(min_width) =
        resolve_contract_length(layout.min_width.as_ref(), ui.available_width())
    {
        ui.set_min_width(min_width);
    }
    if let Some(max_width) =
        resolve_contract_length(layout.max_width.as_ref(), ui.available_width())
    {
        ui.set_max_width(max_width);
    }
    if let Some(min_height) =
        resolve_contract_length(layout.min_height.as_ref(), ui.available_height())
    {
        ui.set_min_height(min_height);
    }
    if let Some(max_height) =
        resolve_contract_length(layout.max_height.as_ref(), ui.available_height())
    {
        ui.set_max_height(max_height);
    }
}

fn compat_card_frame<R>(
    ui: &mut ComponentUi<'_>,
    fill: Option<Color32>,
    stroke: Option<Stroke>,
    corner_radius: Option<CornerRadius>,
    shadow: Option<egui::Shadow>,
    padding_x: i8,
    padding_y: i8,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let runtime = crate::theme::runtime_for_ui(ui.raw());
    surface_frame(
        ui.raw_mut(),
        SurfaceFrame::new(
            fill.unwrap_or(tokens::muted_surface(runtime)),
            stroke.unwrap_or(Stroke::new(1.0, tokens::separator(runtime))),
        )
        .corner_radius(corner_radius.unwrap_or(CornerRadius::same(tokens::radius_lg(runtime))))
        .shadow(shadow.unwrap_or(egui::Shadow::NONE))
        .padding(padding_x, padding_y),
        add,
    )
}

fn ignore_metadata_only_common_fields(common: &ContractCommon) {
    let _ = (&common.slot_classes, &common.actions);
}

fn container_layout_plan(
    layout: Option<&ContractLayout>,
    default_direction: ContractDirection,
    default_gap: f32,
    default_justify: ContractJustify,
    default_align: ContractAlign,
) -> ContainerLayoutPlan {
    let display = match layout.and_then(|layout| layout.display) {
        Some(ContractDisplay::Grid) => TaffyDisplay::Grid,
        _ => TaffyDisplay::Flex,
    };
    let direction = layout
        .and_then(|layout| layout.direction)
        .unwrap_or(default_direction);
    let justify = layout
        .and_then(|layout| layout.justify)
        .unwrap_or(default_justify);
    let align = layout
        .and_then(|layout| layout.align)
        .unwrap_or(default_align);
    let gap_x = layout
        .and_then(|layout| layout.gap_x.or(layout.gap_y))
        .unwrap_or(default_gap);
    let gap_y = layout
        .and_then(|layout| layout.gap_y.or(layout.gap_x))
        .unwrap_or(default_gap);
    let wrap = layout.and_then(|layout| layout.wrap).unwrap_or(false);
    let overflow_x = layout
        .and_then(|layout| layout.overflow_x)
        .unwrap_or(ContractOverflow::Visible);
    let overflow_y = layout
        .and_then(|layout| layout.overflow_y)
        .unwrap_or(ContractOverflow::Visible);

    ContainerLayoutPlan {
        display,
        direction,
        justify,
        align,
        gap_x,
        gap_y,
        wrap,
        overflow_x,
        overflow_y,
    }
}

fn container_taffy_style(
    layout: Option<&ContractLayout>,
    plan: ContainerLayoutPlan,
) -> taffy::Style {
    let mut style = taffy::Style {
        display: match plan.display {
            TaffyDisplay::Flex => taffy::Display::Flex,
            TaffyDisplay::Grid => taffy::Display::Grid,
        },
        overflow: taffy::Point {
            x: map_taffy_overflow(plan.overflow_x),
            y: map_taffy_overflow(plan.overflow_y),
        },
        gap: taffy::Size {
            width: taffy::style_helpers::length(plan.gap_x.max(0.0)),
            height: taffy::style_helpers::length(plan.gap_y.max(0.0)),
        },
        size: taffy::Size {
            width: contract_length_to_dimension(layout.and_then(|layout| layout.width.as_ref())),
            height: contract_length_to_dimension(layout.and_then(|layout| layout.height.as_ref())),
        },
        min_size: taffy::Size {
            width: contract_length_to_dimension(
                layout.and_then(|layout| layout.min_width.as_ref()),
            ),
            height: contract_length_to_dimension(
                layout.and_then(|layout| layout.min_height.as_ref()),
            ),
        },
        max_size: taffy::Size {
            width: contract_length_to_dimension(
                layout.and_then(|layout| layout.max_width.as_ref()),
            ),
            height: contract_length_to_dimension(
                layout.and_then(|layout| layout.max_height.as_ref()),
            ),
        },
        ..Default::default()
    };

    if matches!(plan.display, TaffyDisplay::Flex) {
        style.flex_direction = match plan.direction {
            ContractDirection::Row => taffy::FlexDirection::Row,
            ContractDirection::Column => taffy::FlexDirection::Column,
        };
        style.align_items = Some(map_taffy_align_items(plan.align));
        style.justify_content = Some(map_taffy_justify_content(plan.justify));
        style.flex_wrap = if plan.wrap {
            taffy::FlexWrap::Wrap
        } else {
            taffy::FlexWrap::NoWrap
        };
    } else {
        style.align_items = Some(map_taffy_align_items(plan.align));
        style.justify_content = Some(map_taffy_justify_content(plan.justify));
        if let Some(layout) = layout {
            style.grid_template_columns =
                layout.columns.iter().map(contract_track_to_taffy).collect();
            style.grid_template_rows = layout.rows.iter().map(contract_track_to_taffy).collect();
        }
    }

    style
}

fn taffy_item_style(child: &ContractNode, layout: Option<&ContractLayout>) -> taffy::Style {
    let mut style = taffy::Style {
        size: taffy::Size {
            width: contract_length_to_dimension(layout.and_then(|layout| layout.width.as_ref())),
            height: contract_length_to_dimension(layout.and_then(|layout| layout.height.as_ref())),
        },
        min_size: taffy::Size {
            width: contract_length_to_dimension(
                layout.and_then(|layout| layout.min_width.as_ref()),
            ),
            height: contract_length_to_dimension(
                layout.and_then(|layout| layout.min_height.as_ref()),
            ),
        },
        max_size: taffy::Size {
            width: contract_length_to_dimension(
                layout.and_then(|layout| layout.max_width.as_ref()),
            ),
            height: contract_length_to_dimension(
                layout.and_then(|layout| layout.max_height.as_ref()),
            ),
        },
        margin: contract_margin_to_taffy(layout.and_then(|layout| layout.margin)),
        ..Default::default()
    };

    if let Some(layout) = layout {
        if let Some(grow) = layout.grow {
            style.flex_grow = grow.max(0.0);
        }
        if let Some(shrink) = layout.shrink {
            style.flex_shrink = shrink.max(0.0);
        }
        if let Some(basis) = layout.basis.as_ref() {
            style.flex_basis = contract_length_to_dimension(Some(basis));
        }
        if let Some(col_span) = layout.col_span.filter(|span| *span > 1) {
            style.grid_column = taffy::style_helpers::span(col_span);
        }
        if let Some(row_span) = layout.row_span.filter(|span| *span > 1) {
            style.grid_row = taffy::style_helpers::span(row_span);
        }
    }

    if let ContractNode::Spacer(props) = child {
        if props.flex {
            style.flex_grow = layout
                .and_then(|layout| layout.grow)
                .unwrap_or(1.0)
                .max(0.0);
            style.flex_basis = taffy::style_helpers::length(0.0);
            if props.width.is_none() {
                style.min_size.width = taffy::style_helpers::length(0.0);
            }
            if props.height.is_none() {
                style.min_size.height = taffy::style_helpers::length(0.0);
            }
        }
        if let Some(width) = props.width {
            style.size.width = taffy::style_helpers::length(width.max(0.0));
        }
        if let Some(height) = props.height {
            style.size.height = taffy::style_helpers::length(height.max(0.0));
        }
    }

    if let ContractNode::SizedBox(props) = child {
        if let Some(width) = props.width {
            style.size.width = taffy::style_helpers::length(width.max(0.0));
        }
        if let Some(height) = props.height {
            style.size.height = taffy::style_helpers::length(height.max(0.0));
        }
    }

    style
}

fn resolve_contract_length(length: Option<&ContractLength>, available: f32) -> Option<f32> {
    match length? {
        ContractLength::Auto => None,
        ContractLength::Px { value } => Some(value.max(0.0)),
        ContractLength::Percent { value } => {
            let ratio = normalized_percent(*value);
            Some((available.max(0.0) * ratio).max(0.0))
        }
    }
}

fn contract_length_to_dimension(length: Option<&ContractLength>) -> taffy::Dimension {
    match length {
        Some(ContractLength::Auto) | None => taffy::Dimension::Auto,
        Some(ContractLength::Px { value }) => taffy::Dimension::Length(value.max(0.0)),
        Some(ContractLength::Percent { value }) => {
            taffy::Dimension::Percent(normalized_percent(*value))
        }
    }
}

fn normalized_percent(value: f32) -> f32 {
    let ratio = if value > 1.0 { value / 100.0 } else { value };
    ratio.max(0.0)
}

fn contract_margin_to_taffy(
    edges: Option<ContractEdges>,
) -> taffy::Rect<taffy::LengthPercentageAuto> {
    let Some(edges) = edges else {
        return taffy::Rect::zero();
    };
    taffy::Rect {
        left: taffy::LengthPercentageAuto::Length(edges.left.max(0.0)),
        right: taffy::LengthPercentageAuto::Length(edges.right.max(0.0)),
        top: taffy::LengthPercentageAuto::Length(edges.top.max(0.0)),
        bottom: taffy::LengthPercentageAuto::Length(edges.bottom.max(0.0)),
    }
}

fn contract_track_to_taffy(track: &ContractTrack) -> taffy::TrackSizingFunction {
    match track {
        ContractTrack::Auto => taffy::style_helpers::auto(),
        ContractTrack::Fr { value } => taffy::style_helpers::fr(value.max(0.0)),
        ContractTrack::Px { value } => taffy::style_helpers::length(value.max(0.0)),
        ContractTrack::Percent { value } => {
            taffy::style_helpers::percent(normalized_percent(*value))
        }
    }
}

fn map_taffy_overflow(value: ContractOverflow) -> taffy::Overflow {
    match value {
        ContractOverflow::Visible => taffy::Overflow::Visible,
        ContractOverflow::Hidden => taffy::Overflow::Hidden,
        ContractOverflow::Scroll => taffy::Overflow::Scroll,
    }
}

fn map_taffy_justify_content(value: ContractJustify) -> taffy::JustifyContent {
    match value {
        ContractJustify::Start => taffy::JustifyContent::Start,
        ContractJustify::Center => taffy::JustifyContent::Center,
        ContractJustify::End => taffy::JustifyContent::End,
    }
}

fn map_taffy_align_items(value: ContractAlign) -> taffy::AlignItems {
    match value {
        ContractAlign::Start => taffy::AlignItems::Start,
        ContractAlign::Center => taffy::AlignItems::Center,
        ContractAlign::End => taffy::AlignItems::End,
        ContractAlign::Stretch => taffy::AlignItems::Stretch,
    }
}

fn map_anchor(value: ContractAnchor) -> Align2 {
    match value {
        ContractAnchor::TopLeft => Align2::LEFT_TOP,
        ContractAnchor::TopCenter => Align2::CENTER_TOP,
        ContractAnchor::TopRight => Align2::RIGHT_TOP,
        ContractAnchor::LeftCenter => Align2::LEFT_CENTER,
        ContractAnchor::Center => Align2::CENTER_CENTER,
        ContractAnchor::RightCenter => Align2::RIGHT_CENTER,
        ContractAnchor::BottomLeft => Align2::LEFT_BOTTOM,
        ContractAnchor::BottomCenter => Align2::CENTER_BOTTOM,
        ContractAnchor::BottomRight => Align2::RIGHT_BOTTOM,
    }
}

fn make_id(node_id: &crate::contract::NodeId, suffix: &str) -> Id {
    Id::new(("contract", node_id.as_str(), suffix))
}

fn make_item_id(node_id: &crate::contract::NodeId, scope: &str, item_id: &str) -> Id {
    Id::new(("contract", node_id.as_str(), scope, item_id))
}

fn hierarchy_runtime_ids<'a>(items: &'a [ContractHierarchyItem]) -> BTreeMap<&'a str, usize> {
    let mut item_ids = BTreeSet::new();
    collect_hierarchy_item_ids(items, &mut item_ids);
    item_ids
        .into_iter()
        .enumerate()
        .map(|(index, item_id)| (item_id, index))
        .collect()
}

fn collect_hierarchy_item_ids<'a>(
    items: &'a [ContractHierarchyItem],
    item_ids: &mut BTreeSet<&'a str>,
) {
    for item in items {
        item_ids.insert(item.item_id.as_str());
        collect_hierarchy_item_ids(item.children.as_slice(), item_ids);
    }
}

fn file_tree_runtime_ids<'a>(items: &'a [ContractFileTreeItem]) -> BTreeMap<&'a str, usize> {
    let mut item_ids = BTreeSet::new();
    collect_file_tree_item_ids(items, &mut item_ids);
    item_ids
        .into_iter()
        .enumerate()
        .map(|(index, item_id)| (item_id, index))
        .collect()
}

fn collect_file_tree_item_ids<'a>(
    items: &'a [ContractFileTreeItem],
    item_ids: &mut BTreeSet<&'a str>,
) {
    for item in items {
        item_ids.insert(item.item_id.as_str());
        collect_file_tree_item_ids(item.children.as_slice(), item_ids);
    }
}

fn build_runtime_hierarchy<'a>(
    items: &'a [ContractHierarchyItem],
    runtime_ids: &BTreeMap<&'a str, usize>,
    bindings: &mut BTreeMap<usize, HierarchyBinding<'a>>,
) -> Vec<HierarchyWidgetNode> {
    items
        .iter()
        .map(|item| {
            let runtime_id = *runtime_ids
                .get(item.item_id.as_str())
                .expect("missing runtime hierarchy id");
            let _ = bindings.insert(
                runtime_id,
                HierarchyBinding {
                    item_id: item.item_id.as_str(),
                    label: item.label.as_str(),
                    action_id: item.action_id.as_ref(),
                    open: item.open,
                },
            );
            HierarchyWidgetNode::new(
                runtime_id,
                item.label.as_str(),
                compat_hierarchy_item_kind(item.kind),
            )
            .expanded(item.open)
            .locked(item.locked)
            .children(build_runtime_hierarchy(
                item.children.as_slice(),
                runtime_ids,
                bindings,
            ))
        })
        .collect()
}

fn build_runtime_file_tree<'a>(
    items: &'a [ContractFileTreeItem],
    runtime_ids: &BTreeMap<&'a str, usize>,
    bindings: &mut BTreeMap<usize, HierarchyBinding<'a>>,
) -> Vec<FileTreeNode<'a>> {
    items
        .iter()
        .map(|item| {
            let runtime_id = *runtime_ids
                .get(item.item_id.as_str())
                .expect("missing runtime file-tree id");
            let _ = bindings.insert(
                runtime_id,
                HierarchyBinding {
                    item_id: item.item_id.as_str(),
                    label: item.label.as_str(),
                    action_id: item.action_id.as_ref(),
                    open: item.open,
                },
            );
            FileTreeNode::new(
                runtime_id,
                item.label.as_str(),
                compat_file_tree_item_kind(item.kind),
            )
            .expanded(item.open)
            .children(build_runtime_file_tree(
                item.children.as_slice(),
                runtime_ids,
                bindings,
            ))
        })
        .collect()
}

fn render_contract_menu_entries<'a>(
    ui: &mut egui::Ui,
    entries: &'a [ContractMenuEntry],
    selected: &mut Option<MenuActionBinding<'a>>,
) {
    for entry in entries {
        match entry {
            ContractMenuEntry::Action(action) => {
                let mut label = action.label.clone();
                if let Some(shortcut) = action.shortcut.as_deref() {
                    label.push_str("    ");
                    label.push_str(shortcut);
                }
                if ui.button(label).clicked() {
                    *selected = Some(MenuActionBinding {
                        item_id: action.item_id.as_str(),
                        label: action.label.as_str(),
                        action_id: action.action_id.as_ref(),
                    });
                    ui.close();
                }
            }
            ContractMenuEntry::Separator => {
                ui.separator();
            }
            ContractMenuEntry::Submenu(submenu) => {
                ui.menu_button(submenu.label.as_str(), |ui| {
                    render_contract_menu_entries(ui, submenu.entries.as_slice(), selected);
                });
            }
        }
    }
}

fn compat_combobox_summary(selected_item_ids: &[String], props: &ContractCombobox) -> String {
    let labels = selected_item_ids
        .iter()
        .filter_map(|item_id| props.items.iter().find(|item| &item.item_id == item_id))
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>();
    if labels.is_empty() {
        props
            .placeholder
            .clone()
            .unwrap_or_else(|| "Select options".to_owned())
    } else {
        labels.join(", ")
    }
}

fn render_contract_command_panel(
    ui: &mut egui::Ui,
    node_id: &crate::contract::NodeId,
    query: &mut String,
    props: &ContractCommand,
    placeholder: &str,
) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let _ = surface_frame(
        ui,
        SurfaceFrame::new(
            tokens::card_background(runtime),
            Stroke::new(1.0, tokens::separator(runtime)),
        )
        .corner_radius(tokens::radius_lg(runtime))
        .padding(10, 8)
        .shadow(tokens::tailwind_shadow_lg(runtime)),
        |ui| {
            ui.set_min_width(props.width);
            ui.set_max_width(props.width);
            let mut components = ui.components();
            let _ = components.text_input(
                query,
                TextInput::new()
                    .id(make_id(node_id, "command_query"))
                    .width(props.width)
                    .placeholder(placeholder),
            );
            components.add_space(6.0);
            let _ = components.separator();
            components.add_space(4.0);

            let query_lower = query.trim().to_ascii_lowercase();
            let visible_items = props
                .items
                .iter()
                .filter(|item| {
                    query_lower.is_empty()
                        || item
                            .label
                            .to_ascii_lowercase()
                            .contains(query_lower.as_str())
                        || item
                            .group
                            .to_ascii_lowercase()
                            .contains(query_lower.as_str())
                })
                .collect::<Vec<_>>();

            let _ = egui::ScrollArea::vertical()
                .id_salt(make_id(node_id, "command_scroll"))
                .no_drag_to_scroll()
                .max_height(props.max_height)
                .auto_shrink([false, false])
                .show(components.ui_mut(), |ui: &mut egui::Ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    if visible_items.is_empty() {
                        let mut components = ui.components();
                        let _ = components
                            .label(Label::new("No matches").tone(LabelTone::Muted).size(12.0));
                    } else {
                        let mut components = ui.components();
                        for item in visible_items {
                            let display = if item.group.is_empty() {
                                item.label.clone()
                            } else {
                                format!("{} - {}", item.group, item.label)
                            };
                            let mut button = Button::new(display.as_str())
                                .id(make_item_id(node_id, "command_item", item.item_id.as_str()))
                                .variant(ButtonVariant::Ghost);
                            if let Some(shortcut) = item.shortcut.as_deref() {
                                button = button.trailing_text(shortcut);
                            }
                            let _ = components.button(button);
                        }
                    }
                });
        },
    );
}

fn append_runtime_menu_entries<'a>(
    entries: &'a [ContractMenuEntry],
    runtime_entries: &mut Vec<DropdownMenuEntry<'a>>,
    bindings: &mut Vec<MenuActionBinding<'a>>,
) {
    for entry in entries {
        match entry {
            ContractMenuEntry::Action(action) => {
                let action_index = bindings.len();
                bindings.push(MenuActionBinding {
                    item_id: action.item_id.as_str(),
                    label: action.label.as_str(),
                    action_id: action.action_id.as_ref(),
                });
                runtime_entries.push(runtime_menu_action_entry(action, action_index));
            }
            ContractMenuEntry::Separator => {
                runtime_entries.push(DropdownMenuEntry::separator());
            }
            ContractMenuEntry::Submenu(submenu) => {
                runtime_entries.push(DropdownMenuEntry::separator());
                let label_index = bindings.len();
                bindings.push(MenuActionBinding {
                    item_id: submenu.label.as_str(),
                    label: submenu.label.as_str(),
                    action_id: None,
                });
                let mut label_action =
                    DropdownMenuAction::new(label_index, submenu.label.as_str()).enabled(false);
                if let Some(icon) = submenu.leading_icon.as_deref() {
                    label_action = label_action.icon(icon);
                }
                runtime_entries.push(DropdownMenuEntry::Action(label_action));
                append_runtime_menu_entries(submenu.entries.as_slice(), runtime_entries, bindings);
            }
        }
    }
}

fn runtime_menu_action_entry<'a>(
    action: &'a ContractMenuAction,
    action_index: usize,
) -> DropdownMenuEntry<'a> {
    match (action.leading_icon.as_deref(), action.shortcut.as_deref()) {
        (Some(icon), Some(shortcut)) => DropdownMenuEntry::action_with_icon_and_shortcut(
            action_index,
            action.label.as_str(),
            icon,
            shortcut,
        ),
        (Some(icon), None) => {
            DropdownMenuEntry::action_with_icon(action_index, action.label.as_str(), icon)
        }
        (None, Some(shortcut)) => {
            DropdownMenuEntry::action_with_shortcut(action_index, action.label.as_str(), shortcut)
        }
        (None, None) => DropdownMenuEntry::action(action_index, action.label.as_str()),
    }
}

fn parse_contract_color(value: &ContractColorValue) -> Option<Color32> {
    parse_color(value.as_str())
}

fn parse_color(value: &str) -> Option<Color32> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("transparent") {
        return Some(Color32::TRANSPARENT);
    }
    if value.eq_ignore_ascii_case("black") {
        return Some(Color32::BLACK);
    }
    if value.eq_ignore_ascii_case("white") {
        return Some(Color32::WHITE);
    }

    let hex = value.strip_prefix('#')?;
    match hex.len() {
        6 => {
            let rgb = u32::from_str_radix(hex, 16).ok()?;
            Some(Color32::from_rgb(
                ((rgb >> 16) & 0xff) as u8,
                ((rgb >> 8) & 0xff) as u8,
                (rgb & 0xff) as u8,
            ))
        }
        8 => {
            let rgba = u32::from_str_radix(hex, 16).ok()?;
            Some(Color32::from_rgba_unmultiplied(
                ((rgba >> 24) & 0xff) as u8,
                ((rgba >> 16) & 0xff) as u8,
                ((rgba >> 8) & 0xff) as u8,
                (rgba & 0xff) as u8,
            ))
        }
        _ => None,
    }
}

fn parse_contract_stroke(stroke: &ContractStroke) -> Option<Stroke> {
    Some(Stroke::new(
        stroke.width.max(0.0),
        parse_contract_color(&stroke.color)?,
    ))
}

fn contract_image(source: &str) -> Image<'static> {
    match source {
        "showcase" | "showcase-image" | "builtin:showcase-image" => {
            Image::from_bytes("bytes://contract/showcase-image.png", SHOWCASE_IMAGE_BYTES)
        }
        source if source.starts_with("builtin:twemoji:") => {
            let emoji = source.trim_start_matches("builtin:twemoji:");
            twemoji::image_source(emoji)
                .map(Image::new)
                .unwrap_or_else(|| Image::from_uri(source.to_owned()))
        }
        source => Image::from_uri(source.to_owned()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompatPaginationItem {
    Page(usize),
    Ellipsis,
}

fn compat_pagination_items(
    current_page: usize,
    page_count: usize,
    sibling_count: usize,
) -> Vec<CompatPaginationItem> {
    if page_count == 0 {
        return Vec::new();
    }

    let current = current_page.clamp(1, page_count);
    let visible_page_slots = sibling_count.saturating_mul(2) + 5;
    if page_count <= visible_page_slots {
        return (1..=page_count).map(CompatPaginationItem::Page).collect();
    }

    let left_sibling = current.saturating_sub(sibling_count).max(1);
    let right_sibling = (current + sibling_count).min(page_count);
    let show_left_ellipsis = left_sibling > 2;
    let show_right_ellipsis = right_sibling < page_count.saturating_sub(1);

    if !show_left_ellipsis && show_right_ellipsis {
        let last_left_page = (sibling_count * 2 + 2).min(page_count.saturating_sub(1));
        let mut items = (1..=last_left_page)
            .map(CompatPaginationItem::Page)
            .collect::<Vec<_>>();
        items.push(CompatPaginationItem::Ellipsis);
        items.push(CompatPaginationItem::Page(page_count));
        return items;
    }

    if show_left_ellipsis && !show_right_ellipsis {
        let start_page = page_count.saturating_sub(sibling_count * 2 + 1).max(2);
        let mut items = vec![
            CompatPaginationItem::Page(1),
            CompatPaginationItem::Ellipsis,
        ];
        items.extend((start_page..=page_count).map(CompatPaginationItem::Page));
        return items;
    }

    if show_left_ellipsis && show_right_ellipsis {
        let mut items = vec![
            CompatPaginationItem::Page(1),
            CompatPaginationItem::Ellipsis,
        ];
        items.extend((left_sibling..=right_sibling).map(CompatPaginationItem::Page));
        items.push(CompatPaginationItem::Ellipsis);
        items.push(CompatPaginationItem::Page(page_count));
        return items;
    }

    (1..=page_count).map(CompatPaginationItem::Page).collect()
}

fn compat_anchored_pos(rect: egui::Rect, anchor: Align2) -> egui::Pos2 {
    egui::pos2(
        rect.left() + rect.width() * anchor.x().to_factor(),
        rect.top() + rect.height() * anchor.y().to_factor(),
    )
}

fn compat_image_tile_dimensions(size: Option<ImageTileSize>) -> (f32, f32) {
    match size.unwrap_or(ImageTileSize::Md) {
        ImageTileSize::Sm => (144.0, 96.0),
        ImageTileSize::Md => (216.0, 144.0),
        ImageTileSize::Lg => (288.0, 192.0),
    }
}

fn find_contract_menu_label<'a>(
    entries: &'a [ContractMenuEntry],
    item_id: Option<&str>,
) -> Option<&'a str> {
    let item_id = item_id?;
    for entry in entries {
        match entry {
            ContractMenuEntry::Action(action) if action.item_id == item_id => {
                return Some(action.label.as_str())
            }
            ContractMenuEntry::Submenu(submenu) => {
                if let Some(label) =
                    find_contract_menu_label(submenu.entries.as_slice(), Some(item_id))
                {
                    return Some(label);
                }
            }
            ContractMenuEntry::Action(_) | ContractMenuEntry::Separator => {}
        }
    }
    None
}

fn draw_compat_keycap(
    ui: &mut egui::Ui,
    text: &str,
    min_width: f32,
    height: f32,
    fill: Option<Color32>,
    stroke: Option<Stroke>,
    text_color: Option<Color32>,
    text_size: f32,
    padding_x: f32,
) -> egui::Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let resolved_text_color = text_color.unwrap_or(tokens::text_muted(runtime));
    let font_id = egui::FontId::new(text_size.max(1.0), egui::FontFamily::Monospace);
    let text_width = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font_id.clone(), resolved_text_color)
            .size()
            .x
    });
    let width = (text_width + (padding_x.max(0.0) * 2.0)).max(min_width.max(1.0));
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, height.max(1.0)), Sense::hover());

    ui.painter().rect(
        rect,
        CornerRadius::same(tokens::radius_sm(runtime)),
        fill.unwrap_or(tokens::input_background(runtime)),
        stroke.unwrap_or(Stroke::new(1.0, tokens::input_border(runtime))),
        StrokeKind::Outside,
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        font_id,
        resolved_text_color,
    );

    response
}

fn draw_compat_skeleton(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    corner_radius: Option<u8>,
    circle: bool,
    animated: bool,
) -> egui::Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let size = if circle {
        let diameter = width.min(height).max(1.0);
        egui::vec2(diameter, diameter)
    } else {
        egui::vec2(width.max(1.0), height.max(1.0))
    };
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let base = tokens::muted_surface(runtime);
    let fill = if animated {
        let phase = (ui.input(|input| input.time) as f32 / 2.0).fract();
        let opacity = if phase < 0.5 {
            1.0 - (0.5 * (phase * 2.0))
        } else {
            0.5 + (0.5 * ((phase - 0.5) * 2.0))
        };
        ui.ctx().request_repaint_after_secs(1.0 / 30.0);
        base.gamma_multiply(opacity)
    } else {
        base
    };
    let radius = if circle {
        ((size.x.min(size.y) * 0.5).round()).clamp(0.0, 255.0) as u8
    } else {
        corner_radius.unwrap_or(tokens::radius_md(runtime))
    };

    ui.painter().rect(
        rect,
        CornerRadius::same(radius),
        fill,
        Stroke::NONE,
        StrokeKind::Outside,
    );

    response
}

fn compat_sidebar_side(side: super::SidebarSide) -> crate::runtime_components::SidebarSide {
    match side {
        super::SidebarSide::Left => crate::runtime_components::SidebarSide::Left,
        super::SidebarSide::Right => crate::runtime_components::SidebarSide::Right,
    }
}

fn compat_label_tone(tone: super::LabelTone) -> crate::runtime_components::LabelTone {
    match tone {
        super::LabelTone::Primary => crate::runtime_components::LabelTone::Primary,
        super::LabelTone::Secondary => crate::runtime_components::LabelTone::Secondary,
        super::LabelTone::Muted => crate::runtime_components::LabelTone::Muted,
        super::LabelTone::Destructive => crate::runtime_components::LabelTone::Destructive,
    }
}

fn compat_label_weight(weight: super::LabelWeight) -> crate::runtime_components::LabelWeight {
    match weight {
        super::LabelWeight::Regular => crate::runtime_components::LabelWeight::Regular,
        super::LabelWeight::Semibold => crate::runtime_components::LabelWeight::Semibold,
        super::LabelWeight::Bold => crate::runtime_components::LabelWeight::Bold,
    }
}

fn compat_button_variant(
    variant: super::ButtonVariant,
) -> crate::runtime_components::ButtonVariant {
    match variant {
        super::ButtonVariant::Primary => crate::runtime_components::ButtonVariant::Primary,
        super::ButtonVariant::Secondary => crate::runtime_components::ButtonVariant::Secondary,
        super::ButtonVariant::Ghost => crate::runtime_components::ButtonVariant::Ghost,
        super::ButtonVariant::Link => crate::runtime_components::ButtonVariant::Link,
    }
}

fn compat_control_size(size: super::ControlSize) -> crate::runtime_components::ControlSize {
    match size {
        super::ControlSize::Sm => crate::runtime_components::ControlSize::Sm,
        super::ControlSize::Md => crate::runtime_components::ControlSize::Md,
    }
}

fn compat_number_input_axis(
    axis: super::NumberInputAxis,
) -> crate::runtime_components::NumberInputAxis {
    match axis {
        super::NumberInputAxis::Horizontal => {
            crate::runtime_components::NumberInputAxis::Horizontal
        }
        super::NumberInputAxis::Vertical => crate::runtime_components::NumberInputAxis::Vertical,
    }
}

fn compat_select_variant(
    variant: super::SelectVariant,
) -> crate::runtime_components::SelectVariant {
    match variant {
        super::SelectVariant::Default => crate::runtime_components::SelectVariant::Default,
        super::SelectVariant::Secondary => crate::runtime_components::SelectVariant::Secondary,
    }
}

fn compat_dialogue_intent(
    intent: super::DialogueIntent,
) -> crate::runtime_components::DialogueIntent {
    match intent {
        super::DialogueIntent::Default => crate::runtime_components::DialogueIntent::Default,
        super::DialogueIntent::Alert => crate::runtime_components::DialogueIntent::Alert,
    }
}

fn compat_hierarchy_icon_style(
    icon_style: super::HierarchyIconStyle,
) -> crate::runtime_components::HierarchyIconStyle {
    match icon_style {
        super::HierarchyIconStyle::Emoji => crate::runtime_components::HierarchyIconStyle::Emoji,
        super::HierarchyIconStyle::Icons => crate::runtime_components::HierarchyIconStyle::Icons,
    }
}

fn compat_hierarchy_style(
    style: super::HierarchyStyle,
) -> crate::runtime_components::HierarchyStyle {
    match style {
        super::HierarchyStyle::Normal => crate::runtime_components::HierarchyStyle::Normal,
        super::HierarchyStyle::Component => crate::runtime_components::HierarchyStyle::Component,
    }
}

fn compat_tooltip_placement(
    placement: super::TooltipPlacement,
) -> crate::runtime_components::TooltipPlacement {
    match placement {
        super::TooltipPlacement::Auto => crate::runtime_components::TooltipPlacement::Auto,
        super::TooltipPlacement::Top => crate::runtime_components::TooltipPlacement::Top,
        super::TooltipPlacement::Right => crate::runtime_components::TooltipPlacement::Right,
        super::TooltipPlacement::Bottom => crate::runtime_components::TooltipPlacement::Bottom,
        super::TooltipPlacement::Left => crate::runtime_components::TooltipPlacement::Left,
    }
}

fn compat_popover_side(side: super::PopoverSide) -> crate::runtime_components::PopoverSide {
    match side {
        super::PopoverSide::Top => crate::runtime_components::PopoverSide::Top,
        super::PopoverSide::Right => crate::runtime_components::PopoverSide::Right,
        super::PopoverSide::Bottom => crate::runtime_components::PopoverSide::Bottom,
        super::PopoverSide::Left => crate::runtime_components::PopoverSide::Left,
    }
}

fn compat_popover_align(align: super::PopoverAlign) -> crate::runtime_components::PopoverAlign {
    match align {
        super::PopoverAlign::Start => crate::runtime_components::PopoverAlign::Start,
        super::PopoverAlign::Center => crate::runtime_components::PopoverAlign::Center,
        super::PopoverAlign::End => crate::runtime_components::PopoverAlign::End,
    }
}

fn compat_drag_board_region(
    region: super::DragBoardRegion,
) -> crate::runtime_components::DragBoardRegion {
    match region {
        super::DragBoardRegion::Left => crate::runtime_components::DragBoardRegion::Left,
        super::DragBoardRegion::Right => crate::runtime_components::DragBoardRegion::Right,
    }
}

fn contract_drag_board_region(
    region: crate::runtime_components::DragBoardRegion,
) -> super::DragBoardRegion {
    match region {
        crate::runtime_components::DragBoardRegion::Left => super::DragBoardRegion::Left,
        crate::runtime_components::DragBoardRegion::Right => super::DragBoardRegion::Right,
    }
}

fn compat_audio_playback_state(
    state: super::AudioPlaybackState,
) -> crate::runtime_components::AudioPlaybackState {
    match state {
        super::AudioPlaybackState::Paused => crate::runtime_components::AudioPlaybackState::Paused,
        super::AudioPlaybackState::Playing => {
            crate::runtime_components::AudioPlaybackState::Playing
        }
    }
}

fn compat_hierarchy_item_kind(
    kind: super::HierarchyItemKind,
) -> crate::runtime_components::HierarchyItemKind {
    match kind {
        super::HierarchyItemKind::Folder => crate::runtime_components::HierarchyItemKind::Folder,
        super::HierarchyItemKind::GameObject => {
            crate::runtime_components::HierarchyItemKind::GameObject
        }
        super::HierarchyItemKind::Frame => crate::runtime_components::HierarchyItemKind::Frame,
        super::HierarchyItemKind::Group => crate::runtime_components::HierarchyItemKind::Group,
        super::HierarchyItemKind::Player => crate::runtime_components::HierarchyItemKind::Player,
        super::HierarchyItemKind::Weapon => crate::runtime_components::HierarchyItemKind::Weapon,
        super::HierarchyItemKind::Clothing => {
            crate::runtime_components::HierarchyItemKind::Clothing
        }
        super::HierarchyItemKind::Hitbox => crate::runtime_components::HierarchyItemKind::Hitbox,
        super::HierarchyItemKind::Vector => crate::runtime_components::HierarchyItemKind::Vector,
    }
}

fn compat_file_tree_item_kind(
    kind: super::FileTreeItemKind,
) -> crate::runtime_components::FileTreeItemKind {
    match kind {
        super::FileTreeItemKind::Folder => crate::runtime_components::FileTreeItemKind::Folder,
        super::FileTreeItemKind::Collection => {
            crate::runtime_components::FileTreeItemKind::Collection
        }
        super::FileTreeItemKind::Script => crate::runtime_components::FileTreeItemKind::Script,
        super::FileTreeItemKind::Project => crate::runtime_components::FileTreeItemKind::Project,
        super::FileTreeItemKind::Markdown => crate::runtime_components::FileTreeItemKind::Markdown,
        super::FileTreeItemKind::File => crate::runtime_components::FileTreeItemKind::File,
    }
}

fn drag_region_id(region: super::DragBoardRegion) -> &'static str {
    match region {
        super::DragBoardRegion::Left => "left",
        super::DragBoardRegion::Right => "right",
    }
}

fn should_emit_dialogue_closed(
    was_open: bool,
    open: bool,
    confirmed: bool,
    cancelled: bool,
) -> bool {
    was_open && !open && !confirmed && !cancelled
}

fn load_toast_viewport_state(
    ui: &egui::Ui,
    node_id: &crate::contract::NodeId,
) -> ToastViewportState {
    ui.data(|data| {
        data.get_temp::<ToastViewportState>(make_id(node_id, "toast_state"))
            .unwrap_or_default()
    })
}

fn store_toast_viewport_state(
    ui: &mut egui::Ui,
    node_id: &crate::contract::NodeId,
    state: ToastViewportState,
) {
    ui.data_mut(|data| {
        let state_id = make_id(node_id, "toast_state");
        if state == ToastViewportState::default() {
            data.remove::<ToastViewportState>(state_id);
        } else {
            data.insert_temp(state_id, state);
        }
    });
}

fn toast_anchor(placement: super::ToastPlacement) -> Align2 {
    match placement {
        super::ToastPlacement::TopLeft => Align2::LEFT_TOP,
        super::ToastPlacement::TopCenter => Align2::CENTER_TOP,
        super::ToastPlacement::TopRight => Align2::RIGHT_TOP,
        super::ToastPlacement::CenterLeft => Align2::LEFT_CENTER,
        super::ToastPlacement::Center => Align2::CENTER_CENTER,
        super::ToastPlacement::CenterRight => Align2::RIGHT_CENTER,
        super::ToastPlacement::BottomLeft => Align2::LEFT_BOTTOM,
        super::ToastPlacement::BottomCenter => Align2::CENTER_BOTTOM,
        super::ToastPlacement::BottomRight => Align2::RIGHT_BOTTOM,
    }
}

fn toast_layout(placement: super::ToastPlacement) -> Layout {
    match placement {
        super::ToastPlacement::TopLeft => Layout::top_down(Align::Min),
        super::ToastPlacement::TopCenter => Layout::top_down(Align::Center),
        super::ToastPlacement::TopRight => Layout::top_down(Align::Max),
        super::ToastPlacement::CenterLeft => Layout::top_down(Align::Min),
        super::ToastPlacement::Center => Layout::top_down(Align::Center),
        super::ToastPlacement::CenterRight => Layout::top_down(Align::Max),
        super::ToastPlacement::BottomLeft => Layout::bottom_up(Align::Min),
        super::ToastPlacement::BottomCenter => Layout::bottom_up(Align::Center),
        super::ToastPlacement::BottomRight => Layout::bottom_up(Align::Max),
    }
}

fn toast_anchor_offset(placement: super::ToastPlacement, margin: Vec2) -> Vec2 {
    match placement {
        super::ToastPlacement::TopLeft => margin,
        super::ToastPlacement::TopCenter => egui::vec2(0.0, margin.y),
        super::ToastPlacement::TopRight => egui::vec2(-margin.x, margin.y),
        super::ToastPlacement::CenterLeft => egui::vec2(margin.x, 0.0),
        super::ToastPlacement::Center => Vec2::ZERO,
        super::ToastPlacement::CenterRight => egui::vec2(-margin.x, 0.0),
        super::ToastPlacement::BottomLeft => egui::vec2(margin.x, -margin.y),
        super::ToastPlacement::BottomCenter => egui::vec2(0.0, -margin.y),
        super::ToastPlacement::BottomRight => egui::vec2(-margin.x, -margin.y),
    }
}

fn draw_contract_toast(
    ui: &mut egui::Ui,
    toast: &ContractToastItem,
    width: f32,
    depth: usize,
) -> bool {
    const TOAST_FRAME_PADDING_X: i8 = 10;
    const TOAST_FRAME_PADDING_Y: i8 = 10;
    const TOAST_CLOSE_BUTTON_SIZE: f32 = 22.0;

    let runtime = crate::theme::runtime_for_ui(ui);
    let palette = contract_toast_palette(runtime, toast.intent, depth);
    let shadow = contract_toast_shadow(runtime, depth);
    let inner_width = (width - f32::from(TOAST_FRAME_PADDING_X * 2)).max(1.0);
    let mut dismissed = false;

    let _ = ui.with_layout(Layout::top_down(Align::Min), |ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);

        let _ = surface_frame(
            ui,
            SurfaceFrame::new(palette.fill, palette.stroke)
                .corner_radius(tokens::radius_lg(runtime))
                .padding(TOAST_FRAME_PADDING_X, TOAST_FRAME_PADDING_Y)
                .shadow(shadow),
            |ui| {
                ui.set_min_width(inner_width);
                ui.set_max_width(inner_width);

                let header_width = ui.available_width();
                let _ = ui.allocate_ui_with_layout(
                    egui::vec2(header_width, TOAST_CLOSE_BUTTON_SIZE),
                    Layout::left_to_right(Align::Center),
                    |ui: &mut egui::Ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let mut components = ui.components();
                        let _ = components.label(
                            Label::new(toast.title.as_str())
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );

                        let trailing_width = ui.available_width().max(0.0);
                        let _ = ui.allocate_ui_with_layout(
                            egui::vec2(trailing_width, TOAST_CLOSE_BUTTON_SIZE),
                            Layout::right_to_left(Align::Center),
                            |ui: &mut egui::Ui| {
                                let mut components = ui.components();
                                if components
                                    .button(
                                        Button::icon_only("x")
                                            .variant(ButtonVariant::Ghost)
                                            .size(ControlSize::Sm)
                                            .icon_size(12.0)
                                            .icon_tint(tokens::text_muted(runtime))
                                            .min_size(egui::vec2(
                                                TOAST_CLOSE_BUTTON_SIZE,
                                                TOAST_CLOSE_BUTTON_SIZE,
                                            )),
                                    )
                                    .clicked()
                                {
                                    dismissed = true;
                                }
                            },
                        );
                    },
                );

                if let Some(description) = toast.description.as_deref() {
                    ui.add_space(4.0);
                    let mut components = ui.components();
                    let _ =
                        components.label(Label::new(description).tone(LabelTone::Muted).size(12.0));
                }
            },
        );
    });

    dismissed
}

fn contract_toast_palette(
    runtime: crate::theme::ThemeRuntime,
    intent: super::ToastIntent,
    depth: usize,
) -> ToastPalette {
    let card_fill = tokens::card_background(runtime);
    let border = tokens::separator(runtime);
    let mut palette = match intent {
        super::ToastIntent::Neutral => ToastPalette {
            fill: card_fill,
            stroke: Stroke::new(1.0, border),
        },
        super::ToastIntent::Success => {
            let accent = Color32::from_rgb(34, 197, 94);
            ToastPalette {
                fill: card_fill
                    .lerp_to_gamma(accent, if runtime.mode.is_dark() { 0.18 } else { 0.1 }),
                stroke: Stroke::new(1.0, border.lerp_to_gamma(accent, 0.55)),
            }
        }
        super::ToastIntent::Destructive => {
            let accent = crate::theme::resolved_color(runtime, ColorRole::Destructive);
            ToastPalette {
                fill: card_fill
                    .lerp_to_gamma(accent, if runtime.mode.is_dark() { 0.2 } else { 0.12 }),
                stroke: Stroke::new(1.0, border.lerp_to_gamma(accent, 0.6)),
            }
        }
    };

    let shade = (depth as f32 * 0.075).clamp(0.0, 0.28);
    let stroke_shade = (depth as f32 * 0.05).clamp(0.0, 0.18);
    palette.fill = palette
        .fill
        .lerp_to_gamma(tokens::muted_surface(runtime), shade * 0.8)
        .gamma_multiply(1.0 - (shade * 0.22));
    palette.stroke.color = palette
        .stroke
        .color
        .lerp_to_gamma(tokens::input_border(runtime), stroke_shade);
    palette
}

fn contract_toast_shadow(runtime: crate::theme::ThemeRuntime, depth: usize) -> egui::Shadow {
    let mut shadow = tokens::tailwind_shadow_md(runtime);
    let fade = (1.0 - (depth as f32 * 0.18)).clamp(0.3, 1.0);
    shadow.color = shadow.color.gamma_multiply(fade);
    shadow.blur = (shadow.blur as f32 * (1.0 - (depth as f32 * 0.12)).clamp(0.45, 1.0)) as u8;
    shadow
}

#[cfg(test)]
mod tests {
    use super::{
        class_background_color, class_corner_radius, class_label_weight, class_shadow, class_spec,
        class_text_color, class_text_size, container_layout_plan, contract_toast_shadow,
        effective_layout, make_id, render_tree, resolved_icon_tint, should_emit_dialogue_closed,
        taffy_item_style, FrameRenderer, TaffyDisplay,
    };
    use crate::contract::{
        ContractActions, ContractAlign, ContractButton, ContractColumn, ContractCommon,
        ContractDirection, ContractDisplay, ContractDropdownMenu, ContractEdges, ContractIcon,
        ContractInput, ContractJustify, ContractLabel, ContractLayout, ContractLength,
        ContractMenuAction, ContractMenuEntry, ContractNode, ContractOverflow, ContractRow,
        ContractSizedBox, ContractToastItem, ContractToastViewport, ContractTrack, ContractTree,
        EventKind, LabelTone as ContractLabelTone, LabelWeight as ContractLabelWeight, NodeId,
        ToastIntent as ContractToastIntent, ToastPlacement as ContractToastPlacement,
    };
    use crate::layout::taffy;
    use crate::runtime_components::{ComponentUiExt, LabelWeight as UiLabelWeight};
    use crate::theme::{self, ColorRole, ThemeMode, ThemeSpec};
    use crate::ui::tokens;
    use egui::{
        pos2, CentralPanel, Context, CornerRadius, Event, Modifiers, PointerButton, RawInput, Shape,
    };
    use std::collections::BTreeMap;
    use std::time::{Duration, Instant};

    fn run_frame(
        context: &Context,
        input: RawInput,
        tree: &ContractTree,
    ) -> Vec<crate::contract::ContractEvent> {
        let mut events = Vec::new();
        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                let new_events = render_tree(ui, tree);
                if !new_events.is_empty() {
                    events = new_events;
                }
            });
        });
        events
    }

    fn run_frame_with_style_cache(
        context: &Context,
        input: RawInput,
        tree: &ContractTree,
        style_cache_enabled: bool,
    ) -> Vec<crate::contract::ContractEvent> {
        let mut events = Vec::new();
        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                let mut components = ui.components();
                let mut renderer = FrameRenderer::with_style_cache_enabled(style_cache_enabled);
                renderer.render_node(&mut components, &tree.root);
                if !renderer.events.is_empty() {
                    events = renderer.events;
                }
            });
        });
        events
    }

    fn run_frame_output(
        context: &Context,
        input: RawInput,
        tree: &ContractTree,
    ) -> egui::FullOutput {
        context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = render_tree(ui, tree);
            });
        })
    }

    fn press_at(position: egui::Pos2) -> RawInput {
        RawInput {
            events: vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..RawInput::default()
        }
    }

    fn release_at(position: egui::Pos2) -> RawInput {
        RawInput {
            events: vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..RawInput::default()
        }
    }

    fn input_tree(node_id: &str, value: &str) -> ContractTree {
        ContractTree::new(ContractNode::Input(ContractInput {
            common: ContractCommon::new(node_id),
            value: value.to_owned(),
            action_id: None,
            placeholder: Some("Search".to_owned()),
            leading_icon: None,
            width: 240.0,
        }))
    }

    fn dropdown_menu_tree(node_id: &str) -> ContractTree {
        ContractTree::new(ContractNode::DropdownMenu(ContractDropdownMenu {
            common: ContractCommon::new(node_id),
            action_id: None,
            trigger_label: "Actions".to_owned(),
            width: 220.0,
            trigger_variant: None,
            entries: vec![ContractMenuEntry::Action(ContractMenuAction {
                item_id: "settings".to_owned(),
                label: "Settings".to_owned(),
                action_id: None,
                leading_icon: None,
                shortcut: None,
            })],
        }))
    }

    fn large_render_tree_benchmark_fixture(
        section_count: usize,
        controls_per_section: usize,
    ) -> ContractTree {
        let mut sections = Vec::with_capacity(section_count);
        for section_idx in 0..section_count {
            let mut section_children = Vec::with_capacity(controls_per_section * 2);
            for control_idx in 0..controls_per_section {
                let mut button_common = ContractCommon::new(format!(
                    "benchmark.section.{section_idx}.control.{control_idx}.button"
                ));
                button_common.class = Some(String::from(
                    "w-[50%] h-[25%] min-w-44 max-h-[25%] overflow-hidden border rounded-md bg-card text-card-foreground",
                ));
                button_common.class_list = vec![
                    String::from("flex"),
                    String::from("items-center"),
                    String::from("justify-center"),
                    String::from("grow"),
                    String::from("shrink-0"),
                    String::from("basis-[33%]"),
                    String::from("p-2"),
                    String::from("m-1"),
                ];
                section_children.push(ContractNode::Button(ContractButton {
                    common: button_common,
                    label: format!("Open {section_idx}:{control_idx}"),
                    action_id: None,
                    variant: None,
                    size: None,
                    leading_icon: None,
                    trailing_text: None,
                    trailing_icon: None,
                    icon_only: false,
                    selected: false,
                }));

                let mut label_common = ContractCommon::new(format!(
                    "benchmark.section.{section_idx}.control.{control_idx}.label"
                ));
                label_common.class = Some(String::from("text-sm text-muted-foreground"));
                label_common.class_list = vec![
                    String::from("w-full"),
                    String::from("min-h-6"),
                    String::from("break-words"),
                ];
                section_children.push(ContractNode::Label(ContractLabel {
                    common: label_common,
                    text: format!("Section {section_idx} · Item {control_idx}"),
                    tone: None,
                    weight: None,
                    size: None,
                    truncate: false,
                }));
            }

            let mut section_common =
                ContractCommon::new(format!("benchmark.section.{section_idx}"));
            section_common.class = Some(String::from(
                "flex flex-row items-stretch justify-start gap-2 overflow-hidden",
            ));
            section_common.class_list = vec![
                String::from("w-full"),
                String::from("p-2"),
                String::from("m-1"),
            ];

            sections.push(ContractNode::Row(ContractRow {
                common: section_common,
                gap: 8.0,
                justify: ContractJustify::Start,
                align: ContractAlign::Stretch,
                children: section_children,
            }));
        }

        let mut root_common = ContractCommon::new("benchmark.root");
        root_common.class = Some(String::from(
            "flex flex-col items-stretch justify-start gap-2 overflow-y-scroll",
        ));
        root_common.class_list = vec![
            String::from("w-full"),
            String::from("h-full"),
            String::from("p-2"),
        ];

        ContractTree::new(ContractNode::Column(ContractColumn {
            common: root_common,
            gap: 6.0,
            justify: ContractJustify::Start,
            align: ContractAlign::Stretch,
            children: sections,
        }))
    }

    fn benchmark_render_tree_duration(
        tree: &ContractTree,
        style_cache_enabled: bool,
        frame_count: usize,
    ) -> Duration {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);

        for _ in 0..3 {
            let _ = run_frame_with_style_cache(
                &context,
                RawInput::default(),
                tree,
                style_cache_enabled,
            );
        }

        let started = Instant::now();
        for _ in 0..frame_count {
            let _ = run_frame_with_style_cache(
                &context,
                RawInput::default(),
                tree,
                style_cache_enabled,
            );
        }
        started.elapsed()
    }

    fn benchmark_render_tree_median_duration(
        tree: &ContractTree,
        style_cache_enabled: bool,
        frame_count: usize,
        samples: usize,
    ) -> Duration {
        let mut durations = Vec::with_capacity(samples);
        for _ in 0..samples {
            durations.push(benchmark_render_tree_duration(
                tree,
                style_cache_enabled,
                frame_count,
            ));
        }
        durations.sort_unstable();
        durations[samples / 2]
    }

    #[test]
    fn button_click_emits_clicked_event() {
        let context = Context::default();
        let tree = ContractTree::new(ContractNode::Button(ContractButton {
            common: ContractCommon::new("save"),
            label: "Save".to_owned(),
            action_id: Some("save.clicked".into()),
            variant: None,
            size: None,
            leading_icon: None,
            trailing_text: None,
            trailing_icon: None,
            icon_only: false,
            selected: false,
        }));

        let mut events = run_frame(&context, RawInput::default(), &tree);
        assert!(events.is_empty());

        let click_input = RawInput {
            events: vec![
                Event::PointerMoved(pos2(20.0, 20.0)),
                Event::PointerButton {
                    pos: pos2(20.0, 20.0),
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..RawInput::default()
        };
        let _ = run_frame(&context, click_input, &tree);

        let release_input = RawInput {
            events: vec![
                Event::PointerMoved(pos2(20.0, 20.0)),
                Event::PointerButton {
                    pos: pos2(20.0, 20.0),
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..RawInput::default()
        };
        events = run_frame(&context, release_input, &tree);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Clicked);
    }

    #[test]
    fn text_input_focus_persists_for_a_stable_node_id_across_rerenders() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let focus_id = make_id(&NodeId::from("search"), "input");
        let tree = input_tree("search", "alpha");

        let _ = run_frame(&context, RawInput::default(), &tree);
        let _ = run_frame(&context, press_at(pos2(20.0, 20.0)), &tree);
        let _ = run_frame(&context, release_at(pos2(20.0, 20.0)), &tree);
        assert_eq!(context.memory(|mem| mem.focused()), Some(focus_id));

        let rerendered_tree = input_tree("search", "beta");
        let _ = run_frame(&context, RawInput::default(), &rerendered_tree);
        assert_eq!(context.memory(|mem| mem.focused()), Some(focus_id));
    }

    #[test]
    fn changing_text_input_node_id_resets_focus_state() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let original_focus_id = make_id(&NodeId::from("search-a"), "input");
        let next_focus_id = make_id(&NodeId::from("search-b"), "input");
        let tree = input_tree("search-a", "alpha");

        let _ = run_frame(&context, RawInput::default(), &tree);
        let _ = run_frame(&context, press_at(pos2(20.0, 20.0)), &tree);
        let _ = run_frame(&context, release_at(pos2(20.0, 20.0)), &tree);
        assert_eq!(context.memory(|mem| mem.focused()), Some(original_focus_id));

        let rerendered_tree = input_tree("search-b", "alpha");
        let _ = run_frame(&context, RawInput::default(), &rerendered_tree);
        let _ = run_frame(&context, RawInput::default(), &rerendered_tree);
        assert!(!context.memory(|mem| mem.has_focus(original_focus_id)));
        assert!(!context.memory(|mem| mem.has_focus(next_focus_id)));
    }

    #[test]
    fn dropdown_menu_open_state_tracks_node_id_identity() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let tree = dropdown_menu_tree("menu-a");

        let _ = run_frame(&context, RawInput::default(), &tree);
        let _ = run_frame(&context, press_at(pos2(20.0, 20.0)), &tree);
        let _ = run_frame(&context, release_at(pos2(20.0, 20.0)), &tree);

        let stable_output = run_frame_output(&context, RawInput::default(), &tree);
        assert!(
            find_text_shape(&stable_output.shapes, "Settings").is_some(),
            "same node_id should retain the open dropdown menu"
        );

        let changed_output =
            run_frame_output(&context, RawInput::default(), &dropdown_menu_tree("menu-b"));
        assert!(
            find_text_shape(&changed_output.shapes, "Settings").is_none(),
            "changing node_id should reset the open dropdown menu state"
        );
    }

    #[test]
    fn dialogue_closed_only_emits_for_non_confirm_non_cancel_paths() {
        assert!(!should_emit_dialogue_closed(true, false, true, false));
        assert!(!should_emit_dialogue_closed(true, false, false, true));
        assert!(should_emit_dialogue_closed(true, false, false, false));
        assert!(!should_emit_dialogue_closed(false, false, false, false));
    }

    #[test]
    fn contract_toast_shadow_keeps_md_offset_and_fades_with_depth() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);

        let runtime = theme::runtime_for_context(&context);
        let base = tokens::tailwind_shadow_md(runtime);

        assert_eq!(contract_toast_shadow(runtime, 0), base);

        let stacked = contract_toast_shadow(runtime, 2);
        assert_eq!(stacked.offset, base.offset);
        assert!(stacked.blur < base.blur);
        assert!(stacked.color.a() < base.color.a());
    }

    #[test]
    fn toast_viewport_emits_opened_then_closed_once_on_expiry() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);

        let toast_tree = ContractTree::new(ContractNode::ToastViewport(ContractToastViewport {
            common: ContractCommon::new("toast.viewport"),
            placement: ContractToastPlacement::BottomRight,
            width: 320.0,
            margin_x: 16.0,
            margin_y: 16.0,
            gap: 8.0,
            overlap: 0.0,
            max_visible: 4,
            toasts: vec![ContractToastItem {
                item_id: "toast.saved".to_owned(),
                title: "Saved".to_owned(),
                description: Some("The contract tree round-tripped successfully.".to_owned()),
                intent: ContractToastIntent::Success,
                duration_secs: 0.05,
                action_id: Some("toast.saved".into()),
            }],
        }));

        let events = run_frame(
            &context,
            RawInput {
                time: Some(0.0),
                ..RawInput::default()
            },
            &toast_tree,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Opened);

        let events = run_frame(
            &context,
            RawInput {
                time: Some(0.1),
                ..RawInput::default()
            },
            &toast_tree,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Closed);

        let events = run_frame(
            &context,
            RawInput {
                time: Some(0.2),
                ..RawInput::default()
            },
            &toast_tree,
        );
        assert!(events.is_empty());

        let cleared_tree = ContractTree::new(ContractNode::ToastViewport(ContractToastViewport {
            common: ContractCommon::new("toast.viewport"),
            placement: ContractToastPlacement::BottomRight,
            width: 320.0,
            margin_x: 16.0,
            margin_y: 16.0,
            gap: 8.0,
            overlap: 0.0,
            max_visible: 4,
            toasts: Vec::new(),
        }));
        let events = run_frame(
            &context,
            RawInput {
                time: Some(0.3),
                ..RawInput::default()
            },
            &cleared_tree,
        );
        assert!(events.is_empty());

        let events = run_frame(
            &context,
            RawInput {
                time: Some(0.4),
                ..RawInput::default()
            },
            &toast_tree,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Opened);
    }

    #[test]
    fn common_actions_and_slot_classes_remain_metadata_only() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let mut slot_classes = BTreeMap::new();
        slot_classes.insert(String::from("icon"), String::from("text-lg"));

        let tree = ContractTree::new(ContractNode::Button(ContractButton {
            common: ContractCommon {
                node_id: "save".into(),
                visible: true,
                enabled: true,
                class: Some(String::from("font-semibold")),
                class_list: vec![String::from("font-semibold"), String::from("text-lg")],
                slot_classes,
                actions: ContractActions {
                    click: Some("common.click".into()),
                    ..ContractActions::default()
                },
                layout: None,
            },
            label: "Save".to_owned(),
            action_id: Some("button.click".into()),
            variant: None,
            size: None,
            leading_icon: None,
            trailing_text: None,
            trailing_icon: None,
            icon_only: false,
            selected: false,
        }));

        let _ = run_frame(&context, RawInput::default(), &tree);
        let _ = run_frame(
            &context,
            RawInput {
                events: vec![
                    Event::PointerMoved(pos2(20.0, 20.0)),
                    Event::PointerButton {
                        pos: pos2(20.0, 20.0),
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    },
                ],
                ..RawInput::default()
            },
            &tree,
        );
        let events = run_frame(
            &context,
            RawInput {
                events: vec![
                    Event::PointerMoved(pos2(20.0, 20.0)),
                    Event::PointerButton {
                        pos: pos2(20.0, 20.0),
                        button: PointerButton::Primary,
                        pressed: false,
                        modifiers: Modifiers::NONE,
                    },
                ],
                ..RawInput::default()
            },
            &tree,
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Clicked);
        assert_eq!(
            events[0].action_id.as_ref().map(|id| id.as_str()),
            Some("button.click")
        );
    }

    #[test]
    fn class_name_styles_flow_into_label_rendering_when_props_are_absent() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);

        let mut common = ContractCommon::new("styled-label");
        common.class = Some(String::from("text-lg font-bold text-destructive"));
        let tree = ContractTree::new(ContractNode::Label(ContractLabel {
            common,
            text: "Styled label".to_owned(),
            tone: None,
            weight: None,
            size: None,
            truncate: false,
        }));

        let frame_output = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = render_tree(ui, &tree);
            });
        });

        let text_shape = find_text_shape(&frame_output.shapes, "Styled label")
            .expect("styled label text shape should render");
        let format = text_shape
            .galley
            .job
            .sections
            .first()
            .expect("styled label should have a text section")
            .format
            .clone();
        let runtime = theme::runtime_for_context(&context);
        assert_eq!(format.font_id.size, 18.0);
        assert_eq!(
            format.font_id.family,
            egui::FontFamily::Name(crate::ui::typography::BOLD_FAMILY.into())
        );
        assert_eq!(
            format.color,
            theme::resolved_color(runtime, ColorRole::Destructive)
        );
    }

    #[test]
    fn class_list_merges_after_class_and_explicit_label_props_win() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let runtime = theme::runtime_for_context(&context);

        let mut common = ContractCommon::new("styled-label");
        common.class = Some(String::from("text-sm font-normal text-muted-foreground"));
        common.class_list = vec![
            String::from("text-xl"),
            String::from("font-bold"),
            String::from("text-destructive"),
        ];
        let spec = class_spec(&common).expect("class spec should parse");

        assert_eq!(class_text_size(Some(&spec)), Some(20.0));
        assert_eq!(class_label_weight(Some(&spec)), Some(UiLabelWeight::Bold));
        assert_eq!(
            class_text_color(Some(&spec), runtime),
            Some(theme::resolved_color(runtime, ColorRole::Destructive))
        );

        let tree = ContractTree::new(ContractNode::Label(ContractLabel {
            common,
            text: "Explicit label".to_owned(),
            tone: Some(ContractLabelTone::Muted),
            weight: Some(ContractLabelWeight::Regular),
            size: Some(11.0),
            truncate: false,
        }));

        let frame_output = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = render_tree(ui, &tree);
            });
        });

        let text_shape = find_text_shape(&frame_output.shapes, "Explicit label")
            .expect("explicit label text shape should render");
        let format = text_shape
            .galley
            .job
            .sections
            .first()
            .expect("explicit label should have a text section")
            .format
            .clone();
        assert_eq!(format.font_id.size, 11.0);
        assert_eq!(format.font_id.family, egui::FontFamily::Proportional);
        assert_eq!(format.color, tokens::text_muted(runtime));
    }

    #[test]
    fn class_name_layout_merges_only_when_explicit_layout_is_absent() {
        let mut common = ContractCommon::new("layout.node");
        common.class = Some(String::from("w-[50%] h-[25%] p-2 m-1"));

        let spec = class_spec(&common).expect("class spec should parse");
        let layout = effective_layout(&common, Some(&spec)).expect("class layout");
        assert_eq!(layout.width, Some(ContractLength::Percent { value: 0.5 }));
        assert_eq!(layout.height, Some(ContractLength::Percent { value: 0.25 }));
        assert_eq!(
            layout.padding,
            Some(ContractEdges {
                top: 8.0,
                right: 8.0,
                bottom: 8.0,
                left: 8.0,
            })
        );
        assert_eq!(
            layout.margin,
            Some(ContractEdges {
                top: 4.0,
                right: 4.0,
                bottom: 4.0,
                left: 4.0,
            })
        );

        common.layout = Some(ContractLayout {
            width: Some(ContractLength::Px { value: 99.0 }),
            padding: Some(ContractEdges {
                top: 1.0,
                right: 2.0,
                bottom: 3.0,
                left: 4.0,
            }),
            ..ContractLayout::default()
        });
        let layout = effective_layout(&common, Some(&spec)).expect("merged layout");
        assert_eq!(layout.width, Some(ContractLength::Px { value: 99.0 }));
        assert_eq!(
            layout.padding,
            Some(ContractEdges {
                top: 1.0,
                right: 2.0,
                bottom: 3.0,
                left: 4.0,
            })
        );
        assert_eq!(layout.height, Some(ContractLength::Percent { value: 0.25 }));
    }

    #[test]
    fn style_cache_reuses_class_and_layout_derivation_for_equivalent_nodes() {
        let mut renderer = FrameRenderer::with_style_cache_enabled(true);

        let mut common_a = ContractCommon::new("cache.a");
        common_a.class = Some(String::from("flex gap-2 items-center"));
        common_a.class_list = vec![String::from("w-full"), String::from("p-2")];

        let mut common_b = ContractCommon::new("cache.b");
        common_b.class = Some(String::from("flex gap-2 items-center"));
        common_b.class_list = vec![String::from("w-full"), String::from("p-2")];

        let first = renderer.resolve_common_style(&common_a);
        let second = renderer.resolve_common_style(&common_b);

        assert_eq!(renderer.class_spec_cache.len(), 1);
        assert_eq!(renderer.effective_layout_cache.len(), 1);
        assert_eq!(first.class_spec, second.class_spec);
        assert_eq!(first.layout, second.layout);
    }

    #[test]
    fn style_cache_keeps_layout_derivation_distinct_when_layout_inputs_differ() {
        let mut renderer = FrameRenderer::with_style_cache_enabled(true);

        let mut common_a = ContractCommon::new("cache.a");
        common_a.class = Some(String::from("flex gap-2 items-center"));
        common_a.layout = Some(ContractLayout {
            width: Some(ContractLength::Percent { value: 0.5 }),
            ..ContractLayout::default()
        });

        let mut common_b = ContractCommon::new("cache.b");
        common_b.class = Some(String::from("flex gap-2 items-center"));
        common_b.layout = Some(ContractLayout {
            width: Some(ContractLength::Percent { value: 0.75 }),
            ..ContractLayout::default()
        });

        let first = renderer.resolve_common_style(&common_a);
        let second = renderer.resolve_common_style(&common_b);

        assert_eq!(renderer.class_spec_cache.len(), 1);
        assert_eq!(renderer.effective_layout_cache.len(), 2);
        assert_ne!(first.layout, second.layout);
    }

    #[test]
    fn class_name_grid_overflow_and_min_max_materialize_into_layout() {
        let mut common = ContractCommon::new("layout.node");
        common.class = Some(String::from(
            "grid grid-cols-3 grid-rows-2 min-w-44 max-h-[25%] overflow-hidden",
        ));

        let spec = class_spec(&common).expect("class spec should parse");
        let layout = effective_layout(&common, Some(&spec)).expect("class layout");

        assert_eq!(layout.display, Some(ContractDisplay::Grid));
        assert_eq!(
            layout.columns,
            vec![
                ContractTrack::Fr { value: 1.0 },
                ContractTrack::Fr { value: 1.0 },
                ContractTrack::Fr { value: 1.0 }
            ]
        );
        assert_eq!(
            layout.rows,
            vec![
                ContractTrack::Fr { value: 1.0 },
                ContractTrack::Fr { value: 1.0 }
            ]
        );
        assert_eq!(layout.min_width, Some(ContractLength::Px { value: 176.0 }));
        assert_eq!(
            layout.max_height,
            Some(ContractLength::Percent { value: 0.25 })
        );
        assert_eq!(layout.overflow_x, Some(ContractOverflow::Hidden));
        assert_eq!(layout.overflow_y, Some(ContractOverflow::Hidden));
    }

    #[test]
    fn class_name_flex_layout_merges_into_container_plan() {
        let mut common = ContractCommon::new("layout.node");
        common.class = Some(String::from(
            "flex flex-col gap-x-2 gap-y-3 items-stretch justify-center flex-wrap grow basis-[50%]",
        ));

        let spec = class_spec(&common).expect("class spec should parse");
        let layout = effective_layout(&common, Some(&spec)).expect("class layout");

        assert_eq!(layout.display, Some(ContractDisplay::Flex));
        assert_eq!(layout.direction, Some(ContractDirection::Column));
        assert_eq!(layout.gap_x, Some(8.0));
        assert_eq!(layout.gap_y, Some(12.0));
        assert_eq!(layout.align, Some(ContractAlign::Stretch));
        assert_eq!(layout.justify, Some(ContractJustify::Center));
        assert_eq!(layout.wrap, Some(true));
        assert_eq!(layout.grow, Some(1.0));
        assert_eq!(layout.basis, Some(ContractLength::Percent { value: 0.5 }));

        let plan = container_layout_plan(
            Some(&layout),
            ContractDirection::Row,
            4.0,
            ContractJustify::Start,
            ContractAlign::Start,
        );

        assert_eq!(plan.display, TaffyDisplay::Flex);
        assert_eq!(plan.direction, ContractDirection::Column);
        assert_eq!(plan.gap_x, 8.0);
        assert_eq!(plan.gap_y, 12.0);
        assert_eq!(plan.align, ContractAlign::Stretch);
        assert_eq!(plan.justify, ContractJustify::Center);
        assert!(plan.wrap);
    }

    #[test]
    fn class_name_flex_layout_defaults_cross_axis_alignment_to_stretch() {
        let mut common = ContractCommon::new("layout.node");
        common.class = Some(String::from("flex flex-col gap-3"));

        let spec = class_spec(&common).expect("class spec should parse");
        let layout = effective_layout(&common, Some(&spec)).expect("class layout");

        assert_eq!(layout.display, Some(ContractDisplay::Flex));
        assert_eq!(layout.direction, Some(ContractDirection::Column));
        assert_eq!(layout.align, Some(ContractAlign::Stretch));
    }

    #[test]
    fn taffy_item_style_maps_flex_and_grid_item_fields() {
        let button = ContractNode::Button(ContractButton {
            common: ContractCommon::new("button"),
            label: "Save".into(),
            action_id: None,
            variant: None,
            size: None,
            leading_icon: None,
            trailing_text: None,
            trailing_icon: None,
            selected: false,
            icon_only: false,
        });
        let layout = ContractLayout {
            width: Some(ContractLength::Percent { value: 0.5 }),
            min_height: Some(ContractLength::Px { value: 24.0 }),
            margin: Some(ContractEdges {
                top: 1.0,
                right: 2.0,
                bottom: 3.0,
                left: 4.0,
            }),
            grow: Some(2.0),
            shrink: Some(0.0),
            basis: Some(ContractLength::Px { value: 120.0 }),
            col_span: Some(2),
            row_span: Some(3),
            ..ContractLayout::default()
        };

        let style = taffy_item_style(&button, Some(&layout));
        assert_eq!(style.size.width, taffy::Dimension::Percent(0.5));
        assert_eq!(style.min_size.height, taffy::Dimension::Length(24.0));
        assert_eq!(style.margin.left, taffy::LengthPercentageAuto::Length(4.0));
        assert_eq!(style.flex_grow, 2.0);
        assert_eq!(style.flex_shrink, 0.0);
        assert_eq!(style.flex_basis, taffy::Dimension::Length(120.0));
        assert_eq!(
            style.grid_column,
            taffy::style_helpers::span::<taffy::Line<taffy::GridPlacement>>(2)
        );
        assert_eq!(
            style.grid_row,
            taffy::style_helpers::span::<taffy::Line<taffy::GridPlacement>>(3)
        );
    }

    #[test]
    fn taffy_item_style_honors_sized_box_explicit_dimensions() {
        let sized_box = ContractNode::SizedBox(ContractSizedBox {
            common: ContractCommon::new("catalog.sidepanel"),
            width: Some(232.0),
            height: Some(480.0),
            children: Vec::new(),
        });

        let style = taffy_item_style(&sized_box, None);

        assert_eq!(style.size.width, taffy::Dimension::Length(232.0));
        assert_eq!(style.size.height, taffy::Dimension::Length(480.0));
    }

    #[test]
    fn taffy_column_labels_keep_intrinsic_text_width_instead_of_collapsing_per_character() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);

        let tree = ContractTree::new(ContractNode::SizedBox(ContractSizedBox {
            common: ContractCommon::new("catalog.sidepanel"),
            width: Some(232.0),
            height: None,
            children: vec![ContractNode::Column(ContractColumn {
                common: ContractCommon::new("catalog.nav"),
                gap: 8.0,
                justify: ContractJustify::Start,
                align: ContractAlign::Start,
                children: vec![ContractNode::Label(ContractLabel {
                    common: ContractCommon::new("catalog.nav.label"),
                    text: "Component authoring".to_owned(),
                    tone: None,
                    weight: None,
                    size: None,
                    truncate: false,
                })],
            })],
        }));

        let frame_output = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                let _ = render_tree(ui, &tree);
            });
        });

        let text_shape = find_text_shape(&frame_output.shapes, "Component authoring")
            .expect("catalog nav text should render");
        assert!(
            text_shape.galley.rows.len() <= 2,
            "expected catalog nav label to render in at most two rows, got {} rows",
            text_shape.galley.rows.len()
        );
    }

    #[test]
    fn card_class_background_resolves_theme_roles() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let runtime = theme::runtime_for_context(&context);
        let mut common = ContractCommon::new("styled-card");
        common.class = Some(String::from("bg-card"));
        let spec = class_spec(&common).expect("class spec should parse");

        assert_eq!(
            class_background_color(Some(&spec), runtime),
            Some(theme::resolved_color(runtime, ColorRole::Card))
        );
    }

    #[test]
    fn class_opacity_modulates_supported_tailwind_colors() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let runtime = theme::runtime_for_context(&context);
        let mut common = ContractCommon::new("styled-card");
        common.class = Some(String::from("bg-card text-destructive opacity-80"));
        let spec = class_spec(&common).expect("class spec should parse");

        assert_eq!(
            class_background_color(Some(&spec), runtime),
            Some(theme::resolved_color(runtime, ColorRole::Card).gamma_multiply(0.8))
        );
        assert_eq!(
            class_text_color(Some(&spec), runtime),
            Some(theme::resolved_color(runtime, ColorRole::Destructive).gamma_multiply(0.8))
        );
    }

    #[test]
    fn card_class_shadow_maps_tailwind_shadow_tokens() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let runtime = theme::runtime_for_context(&context);
        let mut common = ContractCommon::new("styled-card");
        common.class = Some(String::from("shadow-md"));
        let spec = class_spec(&common).expect("class spec should parse");

        assert_eq!(
            class_shadow(Some(&spec), runtime),
            Some(tokens::tailwind_shadow_md(runtime))
        );
    }

    #[test]
    fn card_class_radius_merges_segmented_tokens() {
        let mut common = ContractCommon::new("styled-card");
        common.class = Some(String::from("rounded-lg rounded-l-none rounded-tr-xl"));
        let spec = class_spec(&common).expect("class spec should parse");

        assert_eq!(
            class_corner_radius(Some(&spec), CornerRadius::same(8)),
            Some(CornerRadius {
                nw: 0,
                ne: 12,
                sw: 0,
                se: 8,
            })
        );
    }

    #[test]
    fn card_class_radius_preserves_default_corners_for_partial_segmented_tokens() {
        let mut common = ContractCommon::new("styled-card");
        common.class = Some(String::from("rounded-l-none"));
        let spec = class_spec(&common).expect("class spec should parse");

        assert_eq!(
            class_corner_radius(Some(&spec), CornerRadius::same(8)),
            Some(CornerRadius {
                nw: 0,
                ne: 8,
                sw: 0,
                se: 8,
            })
        );

        common.class = Some(String::from("rounded-tr-xl"));
        let spec = class_spec(&common).expect("class spec should parse");

        assert_eq!(
            class_corner_radius(Some(&spec), CornerRadius::same(8)),
            Some(CornerRadius {
                nw: 8,
                ne: 12,
                sw: 8,
                se: 8,
            })
        );
    }

    #[test]
    fn icon_class_text_color_resolves_when_explicit_tint_is_absent() {
        let context = Context::default();
        theme::install(&context, ThemeSpec::default(), ThemeMode::Dark);
        let runtime = theme::runtime_for_context(&context);
        let mut common = ContractCommon::new("styled-icon");
        common.class = Some(String::from("text-destructive"));
        let icon = ContractIcon {
            common,
            name: "bot".to_owned(),
            size: 18.0,
            tint: None,
        };
        let spec = class_spec(&icon.common).expect("class spec should parse");

        assert_eq!(
            resolved_icon_tint(&icon, Some(&spec), runtime),
            Some(theme::resolved_color(runtime, ColorRole::Destructive))
        );
    }

    #[test]
    fn partial_layout_plan_applies_flow_container_overrides() {
        let mut common = ContractCommon::new("layout.node");
        common.layout = Some(ContractLayout {
            direction: Some(ContractDirection::Column),
            gap_y: Some(14.0),
            align: Some(ContractAlign::End),
            justify: Some(ContractJustify::Center),
            ..ContractLayout::default()
        });

        let plan = container_layout_plan(
            common.layout.as_ref(),
            ContractDirection::Row,
            8.0,
            ContractJustify::Start,
            ContractAlign::Start,
        );

        assert_eq!(plan.display, TaffyDisplay::Flex);
        assert_eq!(plan.direction, ContractDirection::Column);
        assert_eq!(plan.gap_x, 14.0);
        assert_eq!(plan.gap_y, 14.0);
        assert_eq!(plan.align, ContractAlign::End);
        assert_eq!(plan.justify, ContractJustify::Center);
        assert!(!plan.wrap);
    }

    #[test]
    fn remaining_unsupported_layout_fields_are_ignored_without_breaking_interaction() {
        let context = Context::default();
        let tree = ContractTree::new(ContractNode::Button(ContractButton {
            common: ContractCommon {
                node_id: "gridy".into(),
                visible: true,
                enabled: true,
                class: None,
                class_list: Vec::new(),
                slot_classes: BTreeMap::new(),
                actions: ContractActions::default(),
                layout: Some(ContractLayout {
                    display: Some(ContractDisplay::Grid),
                    grow: Some(1.0),
                    shrink: Some(1.0),
                    basis: Some(ContractLength::Px { value: 120.0 }),
                    padding: Some(ContractEdges {
                        top: 4.0,
                        right: 8.0,
                        bottom: 4.0,
                        left: 8.0,
                    }),
                    margin: Some(ContractEdges {
                        top: 3.0,
                        right: 3.0,
                        bottom: 3.0,
                        left: 3.0,
                    }),
                    wrap: Some(true),
                    columns: vec![ContractTrack::Fr { value: 1.0 }],
                    rows: vec![ContractTrack::Auto],
                    col_span: Some(2),
                    row_span: Some(2),
                    overflow_x: Some(crate::contract::ContractOverflow::Scroll),
                    overflow_y: Some(crate::contract::ContractOverflow::Hidden),
                    ..ContractLayout::default()
                }),
            },
            label: "Save".to_owned(),
            action_id: Some("button.click".into()),
            variant: None,
            size: None,
            leading_icon: None,
            trailing_text: None,
            trailing_icon: None,
            icon_only: false,
            selected: false,
        }));

        let _ = run_frame(&context, RawInput::default(), &tree);
        let _ = run_frame(
            &context,
            RawInput {
                events: vec![
                    Event::PointerMoved(pos2(20.0, 20.0)),
                    Event::PointerButton {
                        pos: pos2(20.0, 20.0),
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    },
                ],
                ..RawInput::default()
            },
            &tree,
        );
        let events = run_frame(
            &context,
            RawInput {
                events: vec![
                    Event::PointerMoved(pos2(20.0, 20.0)),
                    Event::PointerButton {
                        pos: pos2(20.0, 20.0),
                        button: PointerButton::Primary,
                        pressed: false,
                        modifiers: Modifiers::NONE,
                    },
                ],
                ..RawInput::default()
            },
            &tree,
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Clicked);
    }

    #[test]
    fn render_tree_style_cache_reduces_cpu_time_on_large_tree() {
        const SECTION_COUNT: usize = 72;
        const CONTROLS_PER_SECTION: usize = 10;
        const FRAMES_PER_SAMPLE: usize = 14;
        const SAMPLE_COUNT: usize = 3;

        let tree = large_render_tree_benchmark_fixture(SECTION_COUNT, CONTROLS_PER_SECTION);

        let baseline =
            benchmark_render_tree_median_duration(&tree, false, FRAMES_PER_SAMPLE, SAMPLE_COUNT);
        let cached =
            benchmark_render_tree_median_duration(&tree, true, FRAMES_PER_SAMPLE, SAMPLE_COUNT);

        let baseline_ns = baseline.as_nanos();
        let cached_ns = cached.as_nanos();
        let improvement = if baseline_ns > 0 {
            ((baseline_ns.saturating_sub(cached_ns) as f64) * 100.0) / baseline_ns as f64
        } else {
            0.0
        };

        println!(
            "render_tree microbenchmark (median of {SAMPLE_COUNT} samples, {FRAMES_PER_SAMPLE} frames): baseline={baseline:?}, cached={cached:?}, improvement={improvement:.2}%"
        );

        assert!(
            cached < baseline,
            "style cache should reduce render_tree CPU time on a large tree (baseline={baseline:?}, cached={cached:?})"
        );
    }

    fn find_text_shape<'a>(
        shapes: &'a [egui::epaint::ClippedShape],
        text: &str,
    ) -> Option<&'a egui::epaint::TextShape> {
        for clipped_shape in shapes {
            if let Some(shape) = find_text_shape_in_shape(&clipped_shape.shape, text) {
                return Some(shape);
            }
        }
        None
    }

    fn find_text_shape_in_shape<'a>(
        shape: &'a Shape,
        text: &str,
    ) -> Option<&'a egui::epaint::TextShape> {
        match shape {
            Shape::Text(text_shape) if text_shape.galley.job.text == text => Some(text_shape),
            Shape::Vec(shapes) => {
                for nested_shape in shapes {
                    if let Some(shape) = find_text_shape_in_shape(nested_shape, text) {
                        return Some(shape);
                    }
                }
                None
            }
            _ => None,
        }
    }
}
