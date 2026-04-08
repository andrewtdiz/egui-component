pub fn prepare_frame(app: &mut ShowcaseApp, ctx: &egui::Context) {
    clamp_state(app);
    theme::set_theme(ctx, ThemeSpec::preset(SHOWCASE_BASE_COLOR));
    theme::set_mode(ctx, app.theme_mode);
}

pub fn configure_snapshot(app: &mut ShowcaseApp, component: ComponentKind, theme_mode: ThemeMode) {
    app.selected_component = component;
    app.theme_mode = theme_mode;

    match component {
        ComponentKind::CanvaBrandKit => {
            app.canva_brand_query.clear();
            app.canva_brand_select_index = Some(0);
            app.canva_brand_category_index = 0;
        }
        ComponentKind::CanvaEditImage => {
            app.canva_edit_tool_index = 0;
            app.canva_edit_filter_index = 0;
        }
        ComponentKind::Command => {
            app.command_query = "view".to_owned();
        }
        ComponentKind::CollabCursor => {
            app.collab_cursor_name = "Lisa Chen".to_owned();
            app.collab_cursor_color = COLLAB_CURSOR_DEFAULT_COLOR;
            app.collab_cursor_preview_position = COLLAB_CURSOR_DEFAULT_PREVIEW_POSITION;
        }
        ComponentKind::ContextMenu => {
            app.context_menu_action = None;
        }
        ComponentKind::EmojiSelector => {
            app.emoji_selector_value = "🍕".to_owned();
        }
        ComponentKind::IconToolbar => {
            app.icon_toolbar_selected_index = 0;
        }
        ComponentKind::Dialogue => {
            app.dialogue_open = true;
        }
        ComponentKind::DragBoard => {
            app.drag_board_regions = DRAG_BOARD_DEFAULT_REGIONS;
        }
        ComponentKind::OpenWith => {
            app.open_with_action = Some(0);
        }
        ComponentKind::FileTree => {
            app.file_tree_nodes = default_file_tree_nodes();
            app.file_tree_selected_id = Some(FILE_TREE_DEFAULT_SELECTED_ID);
        }
        ComponentKind::Hierarchy => {
            app.hierarchy_nodes = default_hierarchy_nodes();
            app.hierarchy_selected_id = Some(HIERARCHY_DEFAULT_SELECTED_ID);
            app.hierarchy_style_index = 0;
            app.hierarchy_icon_style_index = 0;
        }
        ComponentKind::Popover => {
            app.popover_open = true;
        }
        ComponentKind::Progress => {
            app.progress_value = 0.68;
        }
        ComponentKind::CanvaPosition => {
            app.canva_position_tab_index = 1;
            app.canva_layer_filter_index = 0;
            app.canva_selected_layer_id = 1;
            app.canva_layer_order = CANVA_LAYER_DEFAULT_ORDER;
        }
        ComponentKind::Sidebar => {
            app.sidebar_preview_open = true;
        }
        ComponentKind::Spinner => {
            app.spinner_demo_until = Some(f64::MAX);
        }
        ComponentKind::Toast => {
            app.toast_stack.clear();
            app.toast_stack.push(
                Toast::new("Changes saved")
                    .description("Your layout tokens were published successfully.")
                    .intent(ToastIntent::Success),
            );
            app.toast_stack.push(
                Toast::new("Heads up")
                    .description("Neutral notifications stack cleanly without blocking the UI."),
            );
        }
        _ => {}
    }
}

pub fn update(app: &mut ShowcaseApp, ctx: &egui::Context) {
    prepare_frame(app, ctx);
    let runtime = theme::runtime_for_context(ctx);
    #[allow(
        deprecated,
        reason = "eframe App::update still renders top-level panels from Context"
    )]
    {
        TopBottomPanel::top("component_showcase_topbar")
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(showcase_header_fill(runtime))
                    .stroke(Stroke::NONE),
            )
            .show(ctx, |ui| app.render_topbar(ui));

        SidePanel::left("component_showcase_sidebar")
            .resizable(true)
            .default_width(SIDEBAR_WIDTH)
            .min_width(200.0)
            .max_width(320.0)
            .show(ctx, |ui| app.render_sidebar(ui));

        CentralPanel::default().show(ctx, |ui| app.render_center(ui));
    }
}

impl ShowcaseApp {
    fn render_sidebar(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);

        let _ = show_column(ui, 8.0, |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new("Components")
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Primary),
            );
        });

        ui.add_space(8.0);
        let _ = ui.components().separator();
        ui.add_space(8.0);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for section in [
                    ShowcaseSection::PrimaryPrimitive,
                    ShowcaseSection::DerivedComposed,
                    ShowcaseSection::Examples,
                    ShowcaseSection::Canva,
                ] {
                    let _ = ui.components().label(
                        Label::new(section.title())
                            .tone(LabelTone::Muted)
                            .size(SMALL_TEXT)
                            .weight(LabelWeight::Semibold),
                    );
                    ui.add_space(4.0);

                    for definition in showcase_component_definitions_by_section(section) {
                        let selected = self.selected_component == definition.kind;
                        if draw_showcase_sidebar_item(ui, definition.label, selected).clicked() {
                            self.selected_component = definition.kind;
                        }
                    }

                    ui.add_space(8.0);
                }
            });
    }

    fn render_topbar(&mut self, ui: &mut Ui) {
        let _ = show_inset(ui, 16, 10, |ui| {
            show_showcase_topbar_row(ui, "egui-component Showcase", &mut self.theme_mode);
        });
        let _ = ui.components().separator();
    }

    fn render_center(&mut self, ui: &mut Ui) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                    self.render_preview_surface(ui);
                });
            });
    }

    fn render_preview_surface(&mut self, ui: &mut Ui) {
        let definition = catalog_component_definition(self.selected_component);
        let width = preview_surface_width(self.selected_component, ui.available_width());

        let _ = ui.components().card(Card::new().padding(14, 14), |ui| {
            ui.set_width(width);

            let mut components = ui.components();
            let _ = components.label(
                Label::new(definition.label)
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Primary),
            );
            let _ = components.label(
                Label::new(showcase_description(self.selected_component))
                    .tone(LabelTone::Muted)
                    .size(SMALL_TEXT),
            );
            ui.add_space(8.0);
            let _ = ui.components().separator();
            ui.add_space(10.0);

            self.render_selected_preview(ui);
        });
    }

    fn render_selected_preview(&mut self, ui: &mut Ui) {
        match self.selected_component {
            ComponentKind::Label => self.render_label_preview(ui),
            ComponentKind::Color => self.render_color_preview(ui),
            ComponentKind::Image => self.render_image_preview(ui),
            ComponentKind::Icon => self.render_icon_preview(ui),
            ComponentKind::IconToolbar => self.render_icon_toolbar_preview(ui),
            ComponentKind::Twemoji => self.render_twemoji_preview(ui),
            ComponentKind::EmojiSelector => self.render_emoji_selector_preview(ui),
            ComponentKind::Kbd => self.render_kbd_preview(ui),
            ComponentKind::Input => self.render_input_preview(ui),
            ComponentKind::Field => self.render_field_preview(ui),
            ComponentKind::Button => self.render_button_preview(ui),
            ComponentKind::ButtonGroup => self.render_button_group_preview(ui),
            ComponentKind::CanvaBackgrounds => self.render_canva_backgrounds_preview(ui),
            ComponentKind::CanvaBrandKit => self.render_canva_brand_kit_preview(ui),
            ComponentKind::CanvaEditImage => self.render_canva_edit_image_preview(ui),
            ComponentKind::CanvaPosition => self.render_canva_position_preview(ui),
            ComponentKind::Checkbox => self.render_checkbox_preview(ui),
            ComponentKind::CollabCursor => self.render_collab_cursor_preview(ui),
            ComponentKind::Switch => self.render_switch_preview(ui),
            ComponentKind::Slider => self.render_slider_preview(ui),
            ComponentKind::NumberInput => self.render_number_input_preview(ui),
            ComponentKind::Select => self.render_select_preview(ui),
            ComponentKind::Tabs => self.render_tabs_preview(ui),
            ComponentKind::Separator => self.render_separator_preview(ui),
            ComponentKind::Card => self.render_card_preview(ui),
            ComponentKind::Progress => self.render_progress_preview(ui),
            ComponentKind::Radio => self.render_radio_preview(ui),
            ComponentKind::RadioGroup => self.render_radio_group_preview(ui),
            ComponentKind::Popover => self.render_popover_preview(ui),
            ComponentKind::Tooltip => self.render_tooltip_preview(ui),
            ComponentKind::DropdownMenu => self.render_dropdown_menu_preview(ui),
            ComponentKind::OpenWith => self.render_open_with_preview(ui),
            ComponentKind::Collapsible => self.render_collapsible_preview(ui),
            ComponentKind::AudioPlayback => self.render_audio_playback_preview(ui),
            ComponentKind::Combobox => self.render_combobox_preview(ui),
            ComponentKind::ContextMenu => self.render_context_menu_preview(ui),
            ComponentKind::Command => self.render_command_preview(ui),
            ComponentKind::Dialogue => self.render_dialogue_preview(ui),
            ComponentKind::DragBoard => self.render_drag_board_preview(ui),
            ComponentKind::FileTree => self.render_file_tree_preview(ui),
            ComponentKind::Hierarchy => self.render_hierarchy_preview(ui),
            ComponentKind::ImageTile => self.render_image_tile_preview(ui),
            ComponentKind::MenuBar => self.render_menu_bar_preview(ui),
            ComponentKind::Sidebar => self.render_sidebar_preview(ui),
            ComponentKind::Skeleton => self.render_skeleton_preview(ui),
            ComponentKind::Spinner => self.render_spinner_preview(ui),
            ComponentKind::Toast => self.render_toast_preview(ui),
            ComponentKind::Toolbar => self.render_toolbar_preview(ui),
            ComponentKind::Pagination => self.render_pagination_preview(ui),
        }
    }
}

pub fn render_snapshot_component(app: &mut ShowcaseApp, ui: &mut Ui) -> egui::Response {
    ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
        show_width(
            ui,
            preview_surface_width(app.selected_component, ui.available_width()),
            |ui| {
                app.render_selected_preview(ui);
            },
        )
        .response
    })
    .inner
}

pub fn render_snapshot_surface(app: &mut ShowcaseApp, ui: &mut Ui) {
    let _ = render_snapshot_component(app, ui);
}

pub(crate) fn show_showcase_topbar_row(ui: &mut Ui, title: &str, theme_mode: &mut ThemeMode) {
    let _ = show_leading_trailing(
        ui,
        12.0,
        30.0,
        egui::Align::Center,
        |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new(title)
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Primary),
            );
        },
        |ui| {
            show_showcase_theme_mode_selector(ui, theme_mode);
        },
    );
}

fn show_showcase_theme_mode_selector(ui: &mut Ui, theme_mode: &mut ThemeMode) {
    let options = [
        TabOption::icon_only(0, "Light", "sun-medium"),
        TabOption::icon_only(1, "Dark", "moon-star"),
        TabOption::icon_only(2, "System", "monitor"),
    ];
    let mut selected_mode = match *theme_mode {
        ThemeMode::Light => 0,
        ThemeMode::Dark => 1,
        ThemeMode::System => 2,
    };
    ui.components().segmented_tabs(
        Id::new("component_showcase_theme_mode"),
        &mut selected_mode,
        &options,
    );
    *theme_mode = match selected_mode {
        0 => ThemeMode::Light,
        1 => ThemeMode::Dark,
        _ => ThemeMode::System,
    };
}

fn show_canva_panel_header(ui: &mut Ui, title: &str) -> egui::Response {
    show_leading_trailing(
        ui,
        8.0,
        28.0,
        egui::Align::Center,
        |ui| {
            let _ = ui.components().label(
                Label::new(title)
                    .tone(LabelTone::Primary)
                    .weight(LabelWeight::Semibold),
            );
        },
        |ui| {
            let _ = ui.components().button(
                Button::icon_only("x")
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Sm)
                    .icon_size(14.0)
                    .min_size(egui::vec2(28.0, 28.0)),
            );
        },
    )
}

fn show_section_title_with_trailing_label(
    ui: &mut Ui,
    title: &str,
    trailing_label: &str,
) -> egui::Response {
    show_leading_trailing(
        ui,
        8.0,
        30.0,
        egui::Align::Center,
        |ui| {
            let _ = ui.components().label(
                Label::new(title)
                    .tone(LabelTone::Primary)
                    .weight(LabelWeight::Semibold),
            );
        },
        |ui| {
            let _ = ui.components().button(
                Button::new(trailing_label)
                    .variant(ButtonVariant::Ghost)
                    .label_weight(ButtonLabelWeight::Regular),
            );
        },
    )
}

fn show_section_link_row(ui: &mut Ui, icon: &str, label: &str) -> egui::Response {
    let primary_tint = text_secondary(ui);
    let muted_tint = text_muted(ui);
    show_leading_trailing(
        ui,
        10.0,
        22.0,
        egui::Align::Center,
        |ui| {
            let _ = show_row(ui, 10.0, |ui| {
                let _ = ui
                    .components()
                    .icon(Icon::new(icon).size(18.0).tint(primary_tint));
                let _ = ui.components().label(
                    Label::new(label)
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold),
                );
            });
        },
        |ui| {
            let _ = ui
                .components()
                .icon(Icon::new("chevron-right").size(18.0).tint(muted_tint));
        },
    )
}

pub(crate) fn draw_showcase_sidebar_item(
    ui: &mut Ui,
    label: &str,
    selected: bool,
) -> egui::Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let hover_fill = crate::ui::tokens::button_secondary_hover_bg(runtime).linear_multiply(0.78);
    let label_font = crate::ui::typography::label_font();
    let foreground = if selected {
        theme::color(ui, ColorRole::Foreground)
    } else {
        text_secondary(ui)
    };
    let label_width = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(label.to_owned(), label_font.clone(), foreground)
            .size()
            .x
    });
    let button_width = (label_width + 20.0).ceil().min(ui.available_width());
    let (rect, response) = ui.allocate_exact_size(vec2(button_width, 30.0), Sense::click());
    let pressed = response.is_pointer_button_down_on();

    let fill = if selected {
        hover_fill
    } else if response.hovered() || pressed {
        hover_fill
    } else {
        Color32::TRANSPARENT
    };

    if selected || response.hovered() || pressed {
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(radius_md(ui)), fill);
    }

    ui.painter().text(
        egui::pos2(rect.left() + 10.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        label_font,
        foreground,
    );

    response
}

fn show_preview_host(
    ui: &mut Ui,
    size: egui::Vec2,
    fill: Color32,
    stroke: Stroke,
    sense: Sense,
    add: impl FnOnce(&mut Ui, Rect, &Response),
) -> Response {
    let (host_rect, response) = ui.allocate_exact_size(size, sense);
    ui.painter().rect(
        host_rect,
        egui::CornerRadius::ZERO,
        fill,
        stroke,
        egui::StrokeKind::Outside,
    );
    let _ = ui.scope_builder(egui::UiBuilder::new().max_rect(host_rect), |ui| {
        add(ui, host_rect, &response);
    });
    response
}

fn draw_collab_cursor_preview_guides(ui: &Ui, host_rect: Rect) {
    let runtime = theme::runtime_for_ui(ui);
    let guide = tokens::text_muted(runtime).linear_multiply(if runtime.mode.is_dark() {
        0.26
    } else {
        0.18
    });
    let center = host_rect.center();
    let inset = 14.0;
    let stroke = Stroke::new(1.0, guide);

    ui.painter().line_segment(
        [
            egui::pos2(host_rect.left() + inset, center.y),
            egui::pos2(host_rect.right() - inset, center.y),
        ],
        stroke,
    );
    ui.painter().line_segment(
        [
            egui::pos2(center.x, host_rect.top() + inset),
            egui::pos2(center.x, host_rect.bottom() - inset),
        ],
        stroke,
    );
    ui.painter()
        .circle_filled(center, 3.0, guide.linear_multiply(1.25));
}

fn collab_cursor_preview_position(host_rect: Rect, normalized: egui::Vec2) -> egui::Pos2 {
    let normalized_x = normalized.x.clamp(0.0, 1.0);
    let normalized_y = normalized.y.clamp(0.0, 1.0);

    egui::pos2(
        host_rect.left() + (host_rect.width() * normalized_x),
        host_rect.top() + (host_rect.height() * normalized_y),
    )
}

fn collab_cursor_preview_anchor(host_rect: Rect, pointer_position: egui::Pos2) -> egui::Vec2 {
    let width = host_rect.width().max(1.0);
    let height = host_rect.height().max(1.0);

    vec2(
        ((pointer_position.x - host_rect.left()) / width).clamp(0.0, 1.0),
        ((pointer_position.y - host_rect.top()) / height).clamp(0.0, 1.0),
    )
}

fn sidebar_preview_toggle_rect(
    host_rect: Rect,
    side_index: usize,
    preview_animation: f32,
    sidebar_width: f32,
) -> Rect {
    let x = if side_index == 0 {
        host_rect.left() + 8.0 + (preview_animation * (sidebar_width - 44.0))
    } else {
        host_rect.right() - 44.0 - (preview_animation * (sidebar_width - 44.0))
    };
    Rect::from_min_size(egui::pos2(x, host_rect.top() + 8.0), vec2(36.0, 36.0))
}
