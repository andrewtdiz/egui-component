impl ShowcaseApp {
    fn render_label_preview(&mut self, ui: &mut Ui) {
        let mut components = ui.components();
        let _ = components.label(
            Label::new("Primary label")
                .tone(LabelTone::Primary)
                .weight(LabelWeight::Semibold),
        );
        let _ = components.label(Label::new("Secondary label").tone(LabelTone::Secondary));
        let _ = components.label(Label::new("Muted helper text").tone(LabelTone::Muted));
        let _ = components.label(
            Label::new("Destructive text")
                .tone(LabelTone::Destructive)
                .weight(LabelWeight::Semibold),
        );
    }

    fn render_color_preview(&mut self, ui: &mut Ui) {
        let border = Stroke::new(1.0, theme::color(ui, ColorRole::Border));

        let _ = ui.components().label(
            Label::new("Solid swatches")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(6.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            for fill in TOOLBAR_SWATCHES {
                let _ = components.color(Color::new(fill).size(20.0));
            }
        });

        ui.add_space(10.0);
        let _ = ui.components().label(
            Label::new("Bordered swatches")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(6.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            for fill in TOOLBAR_SWATCHES {
                let _ = components.color(Color::new(fill).size(20.0).stroke(border));
            }
        });
    }

    fn render_image_preview(&mut self, ui: &mut Ui) {
        let image = showcase_image("primary");
        let image_bg = input_background(ui);
        let md_radius = radius_md(ui);
        let sm_radius = radius_sm(ui);

        let _ = ui.components().label(
            Label::new("bytes:// PNG")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(6.0);
        let _ = ui.components().image(
            image
                .clone()
                .fit_to_exact_size(vec2(180.0, 180.0))
                .corner_radius(md_radius)
                .bg_fill(image_bg),
        );

        ui.add_space(10.0);
        let _ = ui.components().label(
            Label::new("Rotation")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(6.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.slider(&mut self.image_rotation_degrees, (-180.0..=180.0, 220.0));
            let rotation_label = format!("{:.0}deg", self.image_rotation_degrees.round());
            let _ = components.label(
                Label::new(rotation_label.as_str())
                    .tone(LabelTone::Secondary)
                    .weight(LabelWeight::Semibold),
            );
        });
        ui.add_space(6.0);
        let _ = ui.components().image(
            image
                .fit_to_exact_size(vec2(96.0, 96.0))
                .rotate(self.image_rotation_degrees.to_radians(), vec2(0.5, 0.5))
                .corner_radius(sm_radius)
                .bg_fill(image_bg),
        );
    }

    fn render_icon_preview(&mut self, ui: &mut Ui) {
        let _ = layout::row().gap(10.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.icon(Icon::new("bot").size(16.0));
            let _ = components.icon(Icon::new("settings-2").size(16.0));
            let _ = components.icon(Icon::new("sparkles").size(16.0));
            let _ = components.icon(Icon::new("gamepad-2").size(16.0));
            let _ = components.icon(Icon::new("wand-sparkles").size(16.0));
        });
    }

    fn render_twemoji_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().label(
            Label::new("Scales with the same image loader pipeline used by Image and Icon.")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(6.0);

        let _ = layout::row().gap(12.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.twemoji(Twemoji::new("🔥").size(16.0));
            let _ = components.twemoji(Twemoji::new("🔥").size(24.0));
            let _ = components.twemoji(Twemoji::new("🔥").size(32.0));
            let _ = components.twemoji(Twemoji::new("🔥").size(48.0));
        });

        ui.add_space(12.0);
        let _ = ui.components().label(
            Label::new("Sequence coverage")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(6.0);

        let _ = layout::row().gap(14.0).show(ui, |ui| {
            for (emoji, label) in TWEMOJI_SEQUENCE_SAMPLES {
                let _ = layout::column().gap(6.0).show(ui, |ui| {
                    let _ = ui.components().twemoji(Twemoji::new(emoji).size(32.0));
                    let _ = ui
                        .components()
                        .label(Label::new(label).tone(LabelTone::Muted).size(SMALL_TEXT));
                });
            }
        });
    }

    fn render_emoji_selector_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().emoji_selector(
            &mut self.emoji_selector_value,
            EmojiSelector::new(Id::new("component_showcase_emoji_selector")),
        );
    }

    fn render_kbd_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().kbd_group((), |ui| {
            let mut components = ui.components();
            let _ = components.kbd(Kbd::new("⌘"));
            let _ = components.kbd(Kbd::new("⇧"));
            let _ = components.kbd(Kbd::new("⌥"));
            let _ = components.kbd(Kbd::new("⌃"));
        });
        ui.add_space(6.0);
        let _ = ui.components().kbd_group(KbdGroup::new(), |ui| {
            let mut components = ui.components();
            let _ = components.kbd(Kbd::new("Ctrl"));
            let _ = components.label(Label::new("+").tone(LabelTone::Muted));
            let _ = components.kbd(Kbd::new("B"));
        });
    }

    fn render_input_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().text_input(
            &mut self.input_value,
            TextInput::new()
                .width(280.0)
                .placeholder("Type component name"),
        );
    }

    fn render_field_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().field(
            &mut self.field_value,
            Field::new("Material")
                .width(280.0)
                .helper_text("Assigned material for selected mesh"),
        );
    }

    fn render_button_preview(&mut self, ui: &mut Ui) {
        let selected_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Foreground));

        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.button(Button::new("Primary").variant(ButtonVariant::Primary));
            let _ = components.button(Button::new("Secondary").variant(ButtonVariant::Secondary));
            let _ = components.button(Button::new("Ghost").variant(ButtonVariant::Ghost));
            let _ = components.button(Button::new("Link").variant(ButtonVariant::Link));
        });

        ui.add_space(8.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.button(
                Button::icon_only("wand-sparkles")
                    .icon_size(15.0)
                    .variant(ButtonVariant::Primary),
            );
            let _ = components.button(
                Button::icon_only("wand-sparkles")
                    .icon_size(15.0)
                    .variant(ButtonVariant::Secondary),
            );
            let _ = components.button(
                Button::icon_only("wand-sparkles")
                    .icon_size(15.0)
                    .variant(ButtonVariant::Ghost),
            );
            let _ = components.button(
                Button::icon_only("wand-sparkles")
                    .icon_size(15.0)
                    .variant(ButtonVariant::Link),
            );
        });

        ui.add_space(8.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            for (index, fill) in TOOLBAR_SWATCHES.iter().copied().enumerate() {
                let stroke = if index == 1 {
                    selected_stroke
                } else {
                    Stroke::NONE
                };
                let _ = components.button(
                    Button::color_only(Color::new(fill).size(18.0).stroke(stroke))
                        .variant(ButtonVariant::Ghost),
                );
            }
        });
    }

    fn render_button_group_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().button_group(ButtonGroup::new(
            Id::new("component_showcase_button_group"),
            &BUTTON_GROUP_OPTIONS,
        ));
    }

    fn render_canva_backgrounds_preview(&mut self, ui: &mut Ui) {
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let panel_width = ui.available_width().min(352.0);
            let panel_fill = input_background(ui);
            let _ = ui
                .components()
                .card(Card::new().padding(16, 16).fill(panel_fill), |ui| {
                    ui.set_width(panel_width);
                    let full_width = ui.available_width();

                    let _ = ui.components().text_input(
                        &mut self.canva_background_query,
                        TextInput::new()
                            .width(full_width)
                            .placeholder("Search backgrounds")
                            .leading_icon("search")
                            .border_color(CANVA_BRAND_ACCENT),
                    );

                    ui.add_space(12.0);
                    let _ = layout::leading_trailing().gap(12.0).min_height(36.0).show(
                        ui,
                        |ui| {
                            let _ = layout::row().gap(8.0).show(ui, |ui| {
                                let _ = ui.components().button(
                                    Button::icon_only("palette")
                                        .variant(ButtonVariant::Secondary)
                                        .icon_size(18.0)
                                        .min_size(vec2(40.0, 36.0)),
                                );

                                for (index, fill) in
                                    CANVA_BACKGROUND_SWATCHES.iter().copied().enumerate()
                                {
                                    let stroke = if self.canva_background_color_index == index {
                                        Stroke::new(1.0, theme::color(ui, ColorRole::Foreground))
                                    } else {
                                        Stroke::NONE
                                    };

                                    if ui
                                        .components()
                                        .button(
                                            Button::color_only(
                                                Color::new(fill).size(32.0).stroke(stroke),
                                            )
                                            .variant(ButtonVariant::Ghost),
                                        )
                                        .clicked()
                                    {
                                        self.canva_background_color_index = index;
                                    }
                                }
                            });
                        },
                        |ui| {
                            let _ = ui.components().button(
                                Button::icon_only("chevron-right")
                                    .variant(ButtonVariant::Primary)
                                    .icon_size(18.0)
                                    .min_size(vec2(36.0, 36.0)),
                            );
                        },
                    );

                    ui.add_space(12.0);
                    let _ = ui.components().button(
                        Button::new("Magic Background")
                            .variant(ButtonVariant::Secondary)
                            .leading_icon("sparkles")
                            .trailing_icon("crown")
                            .icon_size(16.0)
                            .label_weight(ButtonLabelWeight::Medium)
                            .min_size(vec2(full_width, 38.0)),
                    );

                    ui.add_space(18.0);
                    let _ = ui.components().label(
                        Label::new("All results")
                            .tone(LabelTone::Primary)
                            .weight(LabelWeight::Semibold),
                    );

                    ui.add_space(10.0);
                    draw_canva_background_placeholder_grid(ui);
                });
        });
    }

    fn render_canva_brand_kit_preview(&mut self, ui: &mut Ui) {
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let panel_width = ui.available_width().min(560.0);
            let panel_fill = input_background(ui);
            let _ = ui
                .components()
                .card(Card::new().padding(14, 14).fill(panel_fill), |ui| {
                    let _ = ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                        ui.set_width(panel_width);

                        let left_width = 170.0;
                        let gap = 14.0;

                        let _ = ui.scope(|ui| {
                            let _ = ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                                ui.set_width(left_width);

                                let _ = ui.components().text_input(
                                    &mut self.canva_brand_query,
                                    TextInput::new()
                                        .width(left_width)
                                        .placeholder("Search")
                                        .leading_icon("search")
                                        .border_color(CANVA_BRAND_ACCENT),
                                );

                                ui.add_space(10.0);
                                let _ = ui.components().label(
                                    Label::new("All Brand Templates")
                                        .tone(LabelTone::Secondary)
                                        .weight(LabelWeight::Semibold),
                                );

                                ui.add_space(10.0);
                                let _ = ui.components().separator();
                                ui.add_space(10.0);
                                let _ = ui.components().select(
                                    &mut self.canva_brand_select_index,
                                    Select::from_id(
                                        Id::new("component_showcase_canva_brand_select"),
                                        &CANVA_BRAND_SELECT_OPTIONS,
                                    )
                                    .width(left_width)
                                    .variant(SelectVariant::Secondary)
                                    .leading_icon("badge-cent"),
                                );

                                ui.add_space(6.0);
                                for (index, category) in CANVA_BRAND_CATEGORIES.iter().enumerate() {
                                    let selected = self.canva_brand_category_index == index;
                                    if draw_canva_brand_nav_item(ui, category, selected, left_width)
                                        .clicked()
                                    {
                                        self.canva_brand_category_index = index;
                                    }
                                    if index + 1 < CANVA_BRAND_CATEGORIES.len() {
                                        ui.add_space(2.0);
                                    }
                                }
                            });
                        });

                        ui.add_space(gap);
                        draw_canva_brand_vertical_divider(ui, 520.0);
                        ui.add_space(gap);

                        let _ = ui.scope(|ui| {
                            let _ = ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                                ui.set_width(
                                    (panel_width - left_width - gap * 2.0 - 1.0).max(260.0),
                                );
                                render_canva_brand_detail(ui, self.canva_brand_category_index);
                            });
                        });
                    });
                });
        });
    }

    fn render_canva_edit_image_preview(&mut self, ui: &mut Ui) {
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let panel_width = ui.available_width().min(356.0);
            let panel_fill = input_background(ui);
            let _ = ui
                .components()
                .card(Card::new().padding(16, 16).fill(panel_fill), |ui| {
                    let _ = ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                        ui.set_width(panel_width);

                        let _ = show_canva_panel_header(ui, "Edit image");

                        ui.add_space(18.0);
                        let _ = ui.components().label(
                            Label::new("Select area")
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );
                        ui.add_space(10.0);
                        draw_canva_edit_tool_row(ui, &mut self.canva_edit_tool_index);

                        ui.add_space(12.0);
                        draw_canva_edit_navigation_row(ui, "sliders-horizontal", "Adjust");

                        ui.add_space(12.0);
                        let _ = ui.components().label(
                            Label::new("Magic Studio")
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );
                        ui.add_space(10.0);
                        draw_canva_edit_rail(
                            ui,
                            Id::new("component_showcase_canva_edit_magic"),
                            &CANVA_MAGIC_STUDIO_ITEMS,
                            None,
                            false,
                        );

                        ui.add_space(14.0);
                        let _ = show_section_title_with_trailing_label(ui, "Filters", "See all");
                        ui.add_space(10.0);
                        draw_canva_edit_rail(
                            ui,
                            Id::new("component_showcase_canva_edit_filters"),
                            &CANVA_FILTER_ITEMS,
                            Some(&mut self.canva_edit_filter_index),
                            true,
                        );

                        ui.add_space(14.0);
                        let _ = show_section_title_with_trailing_label(ui, "Effects", "See all");
                        ui.add_space(10.0);
                        draw_canva_edit_rail(
                            ui,
                            Id::new("component_showcase_canva_edit_effects"),
                            &CANVA_EFFECT_ITEMS,
                            None,
                            true,
                        );
                    });
                });
        });
    }

    fn render_canva_position_preview(&mut self, ui: &mut Ui) {
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let panel_width = ui.available_width().min(352.0);
            let panel_fill = input_background(ui);
            let _ = ui
                .components()
                .card(Card::new().padding(16, 16).fill(panel_fill), |ui| {
                    ui.set_width(panel_width);

                    let _ = show_canva_panel_header(ui, "Position");

                    ui.add_space(10.0);
                    ui.components().tabs(
                        Id::new("component_showcase_canva_position_tabs"),
                        &mut self.canva_position_tab_index,
                        &CANVA_POSITION_TAB_OPTIONS,
                    );

                    if self.canva_position_tab_index == 0 {
                        ui.add_space(14.0);
                        draw_canva_position_button_grid(ui, &CANVA_POSITION_ACTIONS);

                        ui.add_space(18.0);
                        let _ = ui.components().label(
                            Label::new("Align to page")
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );
                        ui.add_space(10.0);
                        draw_canva_position_button_grid(ui, &CANVA_POSITION_ALIGN_ACTIONS);
                        ui.add_space(18.0);
                        let _ = ui.components().label(
                            Label::new("Advanced")
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );
                        ui.add_space(12.0);

                        let field_gap = 8.0;
                        let field_width = ((ui.available_width() - (field_gap * 2.0)) / 3.0)
                            .floor()
                            .max(96.0);

                        let _ = layout::row().gap(field_gap).show(ui, |ui| {
                            draw_canva_number_field(
                                ui,
                                "Width",
                                &mut self.canva_width_value,
                                NumberInput::new(Id::new("component_showcase_canva_width"))
                                    .width(field_width)
                                    .range(1.0..=4_000.0)
                                    .speed(1.0)
                                    .fine_speed(0.01)
                                    .decimals(0)
                                    .fine_decimals(2)
                                    .suffix("px"),
                            );
                            draw_canva_number_field(
                                ui,
                                "Height",
                                &mut self.canva_height_value,
                                NumberInput::new(Id::new("component_showcase_canva_height"))
                                    .width(field_width)
                                    .range(1.0..=4_000.0)
                                    .speed(1.0)
                                    .fine_speed(0.01)
                                    .decimals(0)
                                    .fine_decimals(2)
                                    .suffix("px"),
                            );
                            draw_canva_ratio_field(ui, field_width, &mut self.canva_ratio_locked);
                        });

                        ui.add_space(10.0);
                        let _ = layout::row().gap(field_gap).show(ui, |ui| {
                            draw_canva_number_field(
                                ui,
                                "X",
                                &mut self.canva_x_value,
                                NumberInput::new(Id::new("component_showcase_canva_x"))
                                    .width(field_width)
                                    .range(-4_000.0..=4_000.0)
                                    .speed(1.0)
                                    .fine_speed(0.01)
                                    .decimals(0)
                                    .fine_decimals(2)
                                    .suffix("px"),
                            );
                            draw_canva_number_field(
                                ui,
                                "Y",
                                &mut self.canva_y_value,
                                NumberInput::new(Id::new("component_showcase_canva_y"))
                                    .width(field_width)
                                    .range(-4_000.0..=4_000.0)
                                    .speed(1.0)
                                    .fine_speed(0.01)
                                    .decimals(0)
                                    .fine_decimals(2)
                                    .suffix("px"),
                            );
                            draw_canva_number_field(
                                ui,
                                "Rotate",
                                &mut self.canva_rotate_value,
                                NumberInput::new(Id::new("component_showcase_canva_rotate"))
                                    .width(field_width)
                                    .range(-360.0..=360.0)
                                    .speed(1.0)
                                    .fine_speed(0.01)
                                    .decimals(0)
                                    .fine_decimals(2)
                                    .suffix("°"),
                            );
                        });
                    } else {
                        ui.add_space(14.0);
                        ui.components().segmented_tabs(
                            Id::new("component_showcase_canva_layer_filter"),
                            &mut self.canva_layer_filter_index,
                            &CANVA_LAYER_FILTER_OPTIONS,
                        );
                        ui.add_space(16.0);
                        draw_canva_layers_list(
                            ui,
                            Id::new("component_showcase_canva_layers"),
                            &mut self.canva_layer_order,
                            &mut self.canva_selected_layer_id,
                            &CANVA_LAYER_ITEMS,
                        );
                    }
                });
        });
    }

    fn render_checkbox_preview(&mut self, ui: &mut Ui) {
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.checkbox(&mut self.checkbox_value, Checkbox::new());

            let label_color = if self.checkbox_value {
                text_secondary(ui)
            } else {
                text_muted(ui)
            };
            let response = ui
                .scope(|ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.add(
                        egui::Label::new(egui::RichText::new("Receive Shadows").color(label_color))
                            .selectable(false)
                            .sense(Sense::click()),
                    )
                })
                .inner
                .on_hover_cursor(CursorIcon::PointingHand);
            if response.clicked() {
                self.checkbox_value = !self.checkbox_value;
            }
        });
    }

    fn render_collab_cursor_preview(&mut self, ui: &mut Ui) {
        let _ = layout::row().gap(10.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.label(Label::new("Color").tone(LabelTone::Muted).size(SMALL_TEXT));
            let _ = components.color_input(
                &mut self.collab_cursor_color,
                ColorInput::new().id(Id::new("component_showcase_collab_cursor_color")),
            );
            let _ = components.label(Label::new("Name").tone(LabelTone::Muted).size(SMALL_TEXT));
            let _ = components.text_input(
                &mut self.collab_cursor_name,
                TextInput::new().width(180.0).placeholder("Lisa Chen"),
            );
        });

        ui.add_space(14.0);

        let preview_name = self.collab_cursor_name.clone();
        let preview_color = self.collab_cursor_color;
        let host_fill = input_background(ui);
        let host_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Border));

        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let host_size = vec2(ui.available_width().clamp(320.0, 460.0), 240.0);
            let _ = show_preview_host(
                ui,
                host_size,
                host_fill,
                host_stroke,
                Sense::click(),
                |ui, host_rect, host_response| {
                    if host_response.hovered() {
                        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                    }

                    if host_response.clicked() {
                        if let Some(pointer_position) = host_response.interact_pointer_pos() {
                            self.collab_cursor_preview_position =
                                collab_cursor_preview_anchor(host_rect, pointer_position);
                        }
                    }

                    draw_collab_cursor_preview_guides(ui, host_rect);
                    let _ = ui.components().collab_cursor(
                        CollabCursor::new(
                            Id::new("component_showcase_collab_cursor"),
                            preview_name.as_str(),
                            collab_cursor_preview_position(
                                host_rect,
                                self.collab_cursor_preview_position,
                            ),
                        )
                        .color(preview_color)
                        .size(34.0),
                    );
                },
            );
        });
    }

    fn render_switch_preview(&mut self, ui: &mut Ui) {
        let _ = ui
            .components()
            .switch(&mut self.switch_value, "Enable Post FX");
        let _ = ui.components().switch(
            &mut self.small_switch_value,
            Switch::new()
                .label("Use Compact Handles")
                .size(ControlSize::Sm),
        );
    }

    fn render_slider_preview(&mut self, ui: &mut Ui) {
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.slider(&mut self.slider_value, (0.0..=100.0, 250.0));
            let value_label = format!("{:.0}", self.slider_value.round());
            let _ = components.label(
                Label::new(value_label.as_str())
                    .tone(LabelTone::Secondary)
                    .weight(LabelWeight::Semibold),
            );
        });
    }

    fn render_number_input_preview(&mut self, ui: &mut Ui) {
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.number_input(
                &mut self.number_x_value,
                NumberInput::new(Id::new("component_showcase_number_x"))
                    .width(110.0)
                    .range(0.0..=100.0)
                    .decimals(1)
                    .prefix("X")
                    .prefix_tint(NUMBER_INPUT_GREEN)
                    .prefix_align_left()
                    .axis(NumberInputAxis::Horizontal),
            );
            let _ = components.number_input(
                &mut self.number_y_value,
                NumberInput::new(Id::new("component_showcase_number_y"))
                    .width(110.0)
                    .range(0.0..=100.0)
                    .decimals(1)
                    .prefix("Y")
                    .prefix_tint(NUMBER_INPUT_RED)
                    .prefix_align_left()
                    .axis(NumberInputAxis::Vertical),
            );
        });
    }

    fn render_select_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().select(
            &mut self.select_index,
            Select::from_id(Id::new("component_showcase_select"), &SELECT_OPTIONS).width(280.0),
        );

        let selected_label = self
            .select_index
            .and_then(|index| SELECT_OPTIONS.get(index).copied())
            .unwrap_or("None");
        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new(selected_label)
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
    }

    fn render_tabs_preview(&mut self, ui: &mut Ui) {
        ui.components().tabs(
            Id::new("component_showcase_tabs"),
            &mut self.tab_index,
            &TAB_OPTIONS,
        );

        let selected_tab = TAB_OPTIONS
            .iter()
            .find(|option| option.value == self.tab_index)
            .map(|option| option.label)
            .unwrap_or(TAB_OPTIONS[0].label);
        ui.add_space(6.0);
        let _ = ui.components().label(
            Label::new(selected_tab)
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );

        ui.add_space(18.0);
        ui.components().tabs_variant(
            Id::new("component_showcase_blender_tabs"),
            &mut self.blender_tab_index,
            &BLENDER_TAB_OPTIONS,
            TabsVariant::BlenderTopbar,
        );

        ui.add_space(18.0);
        ui.components().segmented_tabs(
            Id::new("component_showcase_segmented_tabs"),
            &mut self.segmented_tab_index,
            &TAB_OPTIONS,
        );

        ui.add_space(18.0);
        ui.components().stacked_tabs(
            Id::new("component_showcase_stacked_tabs"),
            &mut self.stacked_tab_index,
            &STACKED_TAB_OPTIONS,
        );

        ui.add_space(18.0);
        ui.components().rail_tabs(
            Id::new("component_showcase_rail_tabs"),
            &mut self.rail_tab_index,
            &RAIL_TAB_OPTIONS,
        );
    }

    fn render_separator_preview(&mut self, ui: &mut Ui) {
        let mut components = ui.components();
        let _ = components.label(Label::new("Above separator").tone(LabelTone::Secondary));
        let _ = components.separator();
        let _ = components.label(Label::new("Below separator").tone(LabelTone::Secondary));
    }

    fn render_card_preview(&mut self, ui: &mut Ui) {
        let card_fill = input_background(ui);
        let card_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Border));
        let _ = layout::sized_box()
            .width(ui.available_width())
            .show(ui, |ui| {
                let _ =
                    ui.components()
                        .card(Card::new().fill(card_fill).stroke(card_stroke), |ui| {
                            ui.set_min_width(ui.available_width());

                            let _ = layout::column().gap(6.0).show(ui, |ui| {
                                let _ = ui.components().label(
                                    Label::new("Card Title")
                                        .tone(LabelTone::Primary)
                                        .weight(LabelWeight::Bold)
                                        .size(16.0),
                                );
                                let _ = ui.components().label(
                                    Label::new("Cards wrap related content in a bordered panel.")
                                        .tone(LabelTone::Muted),
                                );
                            });

                            ui.add_space(6.0);
                            let _ = ui.components().separator();
                            ui.add_space(6.0);

                            let footer_size =
                                egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                            let _ = ui.allocate_ui_with_layout(
                                footer_size,
                                Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let mut components = ui.components();
                                    let _ = components.button(
                                        Button::new("Save").variant(ButtonVariant::Primary),
                                    );
                                    let _ = components.button(
                                        Button::new("Cancel").variant(ButtonVariant::Secondary),
                                    );
                                },
                            );
                        });
            });
    }

    fn render_progress_preview(&mut self, ui: &mut Ui) {
        let _ = ui
            .components()
            .progress(self.progress_value, Progress::new().width(280.0));
        ui.add_space(10.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            if components
                .button(Button::new("Advance").variant(ButtonVariant::Primary))
                .clicked()
            {
                self.progress_value = (self.progress_value + 0.1).clamp(0.0, 1.0);
            }
            if components
                .button(Button::new("Reset").variant(ButtonVariant::Secondary))
                .clicked()
            {
                self.progress_value = 0.0;
            }
        });
    }

    fn render_spinner_preview(&mut self, ui: &mut Ui) {
        let spinner_tint = theme::color(ui, ColorRole::Foreground);
        let _ = layout::row().gap(16.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.spinner(Spinner::new().size(16.0));
            let _ = components.spinner(Spinner::new().size(22.0));
            let _ = components.spinner(
                Spinner::new()
                    .size(28.0)
                    .stroke_width(2.4)
                    .color(spinner_tint),
            );
        });

        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new("Indeterminate loading spinner with configurable size, stroke, and tint.")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );

        let now = ui.input(|input| input.time);
        let spinner_demo_active = self.spinner_demo_until.is_some_and(|until| until > now);
        if let Some(until) = self.spinner_demo_until {
            if until > now {
                ui.ctx().request_repaint_after_secs((until - now) as f32);
            } else {
                self.spinner_demo_until = None;
            }
        }

        ui.add_space(12.0);
        let _ = ui.components().label(
            Label::new("Loading button example")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(6.0);
        let trigger = ui
            .add_enabled_ui(!spinner_demo_active, |ui| {
                ui.components().button(
                    Button::new(if spinner_demo_active {
                        "  Publish"
                    } else {
                        "Publish"
                    })
                    .variant(ButtonVariant::Secondary)
                    .min_size(vec2(112.0, 34.0)),
                )
            })
            .inner;
        if trigger.clicked() {
            self.spinner_demo_until = Some(now + 3.0);
        }

        if spinner_demo_active {
            let spinner_color = theme::color(ui, ColorRole::Foreground);
            let spinner_rect = egui::Rect::from_center_size(
                egui::pos2(trigger.rect.left() + 18.0, trigger.rect.center().y),
                vec2(14.0, 14.0),
            );
            let _ = ui.scope_builder(egui::UiBuilder::new().max_rect(spinner_rect), |ui| {
                let _ = ui.components().spinner(
                    Spinner::new()
                        .size(14.0)
                        .stroke_width(2.0)
                        .speed(1.4)
                        .color(spinner_color),
                );
            });
        }
    }

    fn render_skeleton_preview(&mut self, ui: &mut Ui) {
        let toggle_label = if self.skeleton_loading {
            "Show Loaded State"
        } else {
            "Show Loading State"
        };
        if ui
            .components()
            .button(Button::new(toggle_label).variant(ButtonVariant::Secondary))
            .clicked()
        {
            self.skeleton_loading = !self.skeleton_loading;
        }

        ui.add_space(10.0);
        let card_fill = input_background(ui);
        let card_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Border));
        let _ = ui
            .components()
            .card(Card::new().fill(card_fill).stroke(card_stroke), |ui| {
                let primary_tint = theme::color(ui, ColorRole::Primary);
                let _ = layout::row().gap(12.0).show(ui, |ui| {
                    let mut components = ui.components();
                    if self.skeleton_loading {
                        let _ = components.skeleton(Skeleton::new().circle(44.0));
                    } else {
                        let _ = components.icon(
                            Icon::new("sparkles").size(20.0).tint(primary_tint),
                        );
                    }

                    let _ = layout::column().gap(8.0).show(ui, |ui| {
                        let mut components = ui.components();
                        if self.skeleton_loading {
                            let _ = components.skeleton((180.0, 16.0));
                            let _ = components.skeleton((240.0, 12.0));
                            let _ = components.skeleton((212.0, 12.0));
                            ui.add_space(2.0);
                            let _ = layout::row().gap(8.0).show(ui, |ui| {
                                let mut components = ui.components();
                                let _ = components.skeleton((72.0, 28.0));
                                let _ = components.skeleton((96.0, 28.0));
                            });
                        } else {
                            let _ = components.label(
                                Label::new("Loading state complete")
                                    .tone(LabelTone::Primary)
                                    .weight(LabelWeight::Semibold),
                            );
                            let _ = components.label(
                                Label::new(
                                    "Skeleton blocks can be mixed to mirror the final layout while data is in flight.",
                                )
                                .tone(LabelTone::Muted)
                                .size(SMALL_TEXT),
                            );
                            ui.add_space(2.0);
                            let _ = layout::row().gap(8.0).show(ui, |ui| {
                                let mut components = ui.components();
                                let _ = components.button(
                                    Button::new("Inspect").variant(ButtonVariant::Primary),
                                );
                                let _ = components.button(
                                    Button::new("Dismiss").variant(ButtonVariant::Secondary),
                                );
                            });
                        }
                    });
                });
            });
    }

    fn render_radio_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().radio(
            &mut self.radio_value,
            Radio::new()
                .label("Use publish channel")
                .description("Standalone radios stay selected until you explicitly reset them."),
        );

        ui.add_space(8.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            if components
                .button(Button::new("Reset").variant(ButtonVariant::Secondary))
                .clicked()
            {
                self.radio_value = false;
            }
            let state_label = if self.radio_value {
                "Selected"
            } else {
                "Unselected"
            };
            let _ = components.label(
                Label::new(state_label)
                    .tone(LabelTone::Muted)
                    .size(SMALL_TEXT),
            );
        });
    }

    fn render_radio_group_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().radio_group(
            &mut self.radio_group_value,
            RadioGroup::new(
                Id::new("component_showcase_radio_group"),
                &RADIO_GROUP_OPTIONS,
            ),
        );

        ui.add_space(8.0);
        let _ = layout::row().gap(8.0).show(ui, |ui| {
            let mut components = ui.components();
            if components
                .button(Button::new("Clear").variant(ButtonVariant::Secondary))
                .clicked()
            {
                self.radio_group_value = None;
            }
            let selected = self
                .radio_group_value
                .and_then(|value| {
                    RADIO_GROUP_OPTIONS
                        .iter()
                        .find(|option| option.value == value)
                        .map(|option| option.label)
                })
                .unwrap_or("No option selected");
            let _ = components.label(Label::new(selected).tone(LabelTone::Muted).size(SMALL_TEXT));
        });
    }

    fn render_tooltip_preview(&mut self, ui: &mut Ui) {
        let mut placement_index = tooltip_placement_index(self.tooltip_placement);
        let _ = ui.components().toggle_group(
            &mut placement_index,
            ToggleGroup::new(
                Id::new("component_showcase_tooltip_placement"),
                &TOOLTIP_PLACEMENT_OPTIONS,
            )
            .min_segment_width(72.0),
        );
        self.tooltip_placement = tooltip_placement_from_index(placement_index);
        ui.add_space(8.0);
        let _ = ui.components().tooltip(
            Tooltip::new("Hover this trigger", "Tooltip content example")
                .width(220.0)
                .placement(self.tooltip_placement),
        );
    }

    fn render_popover_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().popover(
            &mut self.popover_open,
            Popover::new(Id::new("component_showcase_popover"))
                .width(280.0)
                .side_offset(1.0),
            |ui| {
                ui.components()
                    .button(Button::new("Open Popover").variant(ButtonVariant::Secondary))
            },
            |ui, open| {
                let _ = layout::column().gap(10.0).show(ui, |ui| {
                    let mut components = ui.components();
                    let _ = components.label(
                        Label::new("Layout settings")
                            .weight(LabelWeight::Semibold)
                            .tone(LabelTone::Primary),
                    );
                    let _ = components.label(
                        Label::new("Interactive popovers work well for small settings panels.")
                            .tone(LabelTone::Muted)
                            .size(SMALL_TEXT),
                    );
                    let _ = components.switch(&mut self.popover_compact_mode, "Compact mode");
                    let _ = components.switch(&mut self.switch_value, "Snap to grid");

                    ui.add_space(4.0);
                    let _ = layout::row().gap(8.0).show(ui, |ui| {
                        let mut components = ui.components();
                        if components
                            .button(Button::new("Close").variant(ButtonVariant::Ghost))
                            .clicked()
                        {
                            *open = false;
                        }
                        let _ =
                            components.button(Button::new("Apply").variant(ButtonVariant::Primary));
                    });
                });
            },
        );
        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new(if self.popover_compact_mode {
                "Compact mode enabled"
            } else {
                "Compact mode disabled"
            })
            .tone(LabelTone::Muted)
            .size(SMALL_TEXT),
        );
    }

    fn render_dropdown_menu_preview(&mut self, ui: &mut Ui) {
        let (_, state) = ui.components().dropdown_menu(
            DropdownMenu::new("Open")
                .entries(&DROPDOWN_ENTRIES)
                .width(220.0),
        );
        if let Some(action) = state.action {
            self.dropdown_action = Some(action);
        }
        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new(dropdown_action_label(self.dropdown_action))
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
    }

    fn render_context_menu_preview(&mut self, ui: &mut Ui) {
        let preview_width = 420.0f32.min(ui.available_width());
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let (_, state) = ui.components().context_menu(
                ContextMenu::new(
                    Id::new("component_showcase_context_menu"),
                    &CONTEXT_MENU_ENTRIES,
                )
                .width(220.0)
                .size(vec2(preview_width, 176.0)),
                |ui| {
                    let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                        ui.add_space(18.0);
                        let _ = ui.components().label(
                            Label::new("Scene View")
                                .tone(LabelTone::Muted)
                                .size(SMALL_TEXT),
                        );
                        ui.add_space(8.0);
                        let _ = ui.components().label(
                            Label::new("Right-click anywhere in this region")
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold)
                                .size(18.0),
                        );
                        ui.add_space(6.0);
                        let _ = ui.components().label(
                            Label::new("Open a context menu with scene actions like rename, duplicate, or delete.")
                                .tone(LabelTone::Secondary),
                        );
                    });
                },
            );
            if let Some(action) = state.action {
                self.context_menu_action = Some(action);
            }
        });

        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new(context_menu_action_label(self.context_menu_action))
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
    }

    fn render_collapsible_preview(&mut self, ui: &mut Ui) {
        let collapsible_open = self.collapsible_open;
        let _ = ui.components().collapsible(
            &mut self.collapsible_open,
            Collapsible::new(Id::new("component_showcase_collapsible"), "Transform")
                .open(collapsible_open)
                .leading_icon("move-3d")
                .trailing_icon("ellipsis"),
            |ui| {
                let mut components = ui.components();
                let _ = components.label(Label::new("Position").tone(LabelTone::Secondary));
                let _ = components.label(Label::new("Rotation").tone(LabelTone::Secondary));
                let _ = components.label(Label::new("Scale").tone(LabelTone::Secondary));
            },
        );
    }

    fn render_audio_playback_preview(&mut self, ui: &mut Ui) {
        let (_, playback_result) = ui.components().audio_playback_with_actions(
            AudioPlayback::new(
                Id::new("component_showcase_audio_playback"),
                self.audio_playback_state,
            ),
            |ui| {
                let mut components = ui.components();
                let _ = components.button(
                    Button::icon_only("share-2")
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Sm),
                );
                let _ = components.button(
                    Button::icon_only("download")
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Sm),
                );
                let _ = components.button(
                    Button::icon_only("ellipsis")
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Sm),
                );
            },
        );

        if playback_result.play_pause_clicked {
            self.audio_playback_state = match self.audio_playback_state {
                AudioPlaybackState::Paused => AudioPlaybackState::Playing,
                AudioPlaybackState::Playing => AudioPlaybackState::Paused,
            };
        }

        ui.add_space(8.0);
        let status = match self.audio_playback_state {
            AudioPlaybackState::Paused => "Playback: Paused",
            AudioPlaybackState::Playing => "Playback: Playing",
        };
        let _ = ui
            .components()
            .label(Label::new(status).tone(LabelTone::Muted).size(SMALL_TEXT));
    }

    fn render_combobox_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().combobox(
            &mut self.combobox_query,
            &mut self.combobox_indices,
            Combobox::new(Id::new("component_showcase_combobox"), &COMBOBOX_OPTIONS).width(280.0),
        );
        ui.add_space(8.0);
        let selected_summary = if self.combobox_indices.is_empty() {
            "No options selected".to_owned()
        } else {
            self.combobox_indices
                .iter()
                .filter_map(|index| COMBOBOX_OPTIONS.get(*index))
                .copied()
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = ui.components().label(
            Label::new(&selected_summary)
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
    }

    fn render_command_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().command(
            &mut self.command_query,
            &COMMAND_ITEMS,
            Command::new(Id::new("component_showcase_command"))
                .width(380.0)
                .preview(true)
                .preview_height(220.0),
        );
    }

    fn render_dialogue_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().dialogue(
            &mut self.dialogue_open,
            Dialogue::new(Id::new("component_showcase_dialogue"), "Create Component")
                .description("Adds the selected component to the active object.")
                .trigger_label("Open Dialogue")
                .confirm_label("Create")
                .cancel_label("Cancel")
                .intent(DialogueIntent::Default)
                .width(380.0),
        );
    }

    fn render_image_tile_preview(&mut self, ui: &mut Ui) {
        let featured_image = showcase_image("featured");
        let secondary_image = showcase_image("secondary");

        let _ = ui.components().label(
            Label::new("Featured tile")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(8.0);

        let (_, featured_state) = ui.components().image_tile_with_body(
            ImageTile::new(featured_image.clone()).size(ImageTileSize::Lg),
            |ui| {
                let mut components = ui.components();
                let _ = components.label(
                    Label::new("Untitled Design")
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold)
                        .size(16.0),
                );
                draw_image_tile_metadata_row(ui, "Edited 2 days ago");
            },
        );
        if featured_state.tile_clicked {
            self.image_tile_selected = !self.image_tile_selected;
            self.image_tile_last_action = "Opened Untitled Design".to_owned();
        }

        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new("Audio preview")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(8.0);
        let (_, audio_state) = ui.components().image_tile_with_body(
            ImageTile::new(secondary_image)
                .size(ImageTileSize::Md)
                .selected(self.image_tile_selected)
                .playback_state(self.image_tile_playback_state),
            |ui| {
                let mut components = ui.components();
                let _ = components.label(
                    Label::new("Ambient Preview")
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold),
                );
                let playback_label = match self.image_tile_playback_state {
                    ImageTilePlaybackState::Paused => "Paused • Click play to preview",
                    ImageTilePlaybackState::Playing => "Playing • 0:27 loop",
                };
                draw_image_tile_metadata_row(ui, playback_label);
            },
        );
        if audio_state.play_pause_clicked {
            self.image_tile_playback_state = match self.image_tile_playback_state {
                ImageTilePlaybackState::Paused => ImageTilePlaybackState::Playing,
                ImageTilePlaybackState::Playing => ImageTilePlaybackState::Paused,
            };
            self.image_tile_last_action = match self.image_tile_playback_state {
                ImageTilePlaybackState::Paused => "Paused audio preview".to_owned(),
                ImageTilePlaybackState::Playing => "Started audio preview".to_owned(),
            };
        } else if audio_state.tile_clicked {
            self.image_tile_last_action = "Opened audio preview tile".to_owned();
        }

        ui.add_space(8.0);
        let status = format!("Last action: {}", self.image_tile_last_action);
        let _ = ui.components().label(
            Label::new(status.as_str())
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
    }

    fn render_menu_bar_preview(&mut self, ui: &mut Ui) {
        let card_fill = app_background(ui);
        let card_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Border));
        let menu_items = [
            MenuBarItem::new("File", &MENU_BAR_FILE_ENTRIES).width(220.0),
            MenuBarItem::new("Edit", &MENU_BAR_EDIT_ENTRIES).width(190.0),
            MenuBarItem::new("View", &MENU_BAR_VIEW_ENTRIES).width(196.0),
            MenuBarItem::new("Object", &MENU_BAR_OBJECT_ENTRIES).width(220.0),
        ];

        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let _ = ui
                .components()
                .card(Card::new().fill(card_fill).stroke(card_stroke), |ui| {
                    ui.set_width(MENU_BAR_PREVIEW_WIDTH.min(ui.available_width()));
                    let (_, state) = ui.components().menu_bar(MenuBar::new(
                        Id::new("component_showcase_menu_bar"),
                        &menu_items,
                    ));
                    if let Some(action) = state.action {
                        self.menu_bar_action = Some(action);
                    }
                });
        });

        ui.add_space(8.0);
        let _ = ui.components().label(
            Label::new(menu_bar_action_label(self.menu_bar_action))
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
    }

    fn render_sidebar_preview(&mut self, ui: &mut Ui) {
        if let Some(index) = ui.components().button_group(ButtonGroup::new(
            Id::new("component_showcase_sidebar_side"),
            &SIDEBAR_SIDE_OPTIONS,
        )) {
            self.sidebar_side_index = index;
        }

        ui.add_space(10.0);
        let canvas_fill = if theme::runtime_for_ui(ui).mode.is_dark() {
            app_background(ui)
        } else {
            TOOLBAR_CANVAS_LIGHT_FILL
        };
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let width = 700.0f32.min(ui.available_width());
            let host_size = vec2(width, 360.0);
            let _ = show_preview_host(
                ui,
                host_size,
                canvas_fill,
                Stroke::new(1.0, theme::color(ui, ColorRole::Border)),
                Sense::hover(),
                |ui, host_rect, _| {
                    let preview_animation = ui.ctx().animate_bool_responsive(
                        Id::new("component_showcase_sidebar_preview"),
                        self.sidebar_preview_open,
                    );
                    let sidebar_width = 240.0;
                    let toggle_rect = sidebar_preview_toggle_rect(
                        host_rect,
                        self.sidebar_side_index,
                        preview_animation,
                        sidebar_width,
                    );
                    let toggle_icon = sidebar_preview_toggle_icon(
                        self.sidebar_side_index,
                        self.sidebar_preview_open,
                    );
                    let toggle_response = ui
                        .scope_builder(egui::UiBuilder::new().max_rect(toggle_rect), |ui| {
                            ui.components().button(
                                Button::icon_only(toggle_icon)
                                    .variant(if self.sidebar_preview_open {
                                        ButtonVariant::Secondary
                                    } else {
                                        ButtonVariant::Ghost
                                    })
                                    .icon_size(16.0)
                                    .min_size(vec2(36.0, 36.0)),
                            )
                        })
                        .inner;
                    if toggle_response.clicked() {
                        self.sidebar_preview_open = !self.sidebar_preview_open;
                    }

                    ui.components().sidebar_in(
                        host_rect,
                        &mut self.sidebar_preview_open,
                        Sidebar::new(Id::new("component_showcase_overlay_sidebar"))
                            .title("Workspace")
                            .side(if self.sidebar_side_index == 0 {
                                SidebarSide::Left
                            } else {
                                SidebarSide::Right
                            })
                            .width(sidebar_width)
                            .backdrop(false),
                        |ui, open| {
                            let _ = layout::column().gap(8.0).show(ui, |ui| {
                                let _ = ui.components().button(
                                    Button::new("New Draft")
                                        .variant(ButtonVariant::Primary)
                                        .leading_icon("file-plus"),
                                );
                                let _ = ui.components().button(
                                    Button::new("Command Search")
                                        .variant(ButtonVariant::Ghost)
                                        .leading_icon("search"),
                                );
                                let _ = ui.components().button(
                                    Button::new("Theme Tokens")
                                        .variant(ButtonVariant::Ghost)
                                        .leading_icon("palette"),
                                );
                                let _ = ui.components().button(
                                    Button::new("Exports")
                                        .variant(ButtonVariant::Ghost)
                                        .leading_icon("rocket"),
                                );
                                ui.add_space(4.0);
                                if ui
                                    .components()
                                    .button(
                                        Button::new("Close Sidebar")
                                            .variant(ButtonVariant::Secondary),
                                    )
                                    .clicked()
                                {
                                    *open = false;
                                }
                            });
                        },
                    );
                },
            );
        });
    }

    fn render_toast_preview(&mut self, ui: &mut Ui) {
        let placement_label = self
            .toast_placement_index
            .and_then(|index| TOAST_PLACEMENT_OPTIONS.get(index).copied())
            .unwrap_or(TOAST_PLACEMENT_OPTIONS[8]);
        let _ = ui.components().select(
            &mut self.toast_placement_index,
            Select::from_id(
                Id::new("component_showcase_toast_position"),
                &TOAST_PLACEMENT_OPTIONS,
            )
            .width(220.0),
        );

        ui.add_space(8.0);
        let _ =
            layout::row().gap(8.0).show(ui, |ui| {
                let mut components = ui.components();
                if components
                    .button(Button::new("Neutral").variant(ButtonVariant::Secondary))
                    .clicked()
                {
                    self.toast_stack.push(Toast::new("Heads up").description(
                        "Neutral notifications stack cleanly without blocking the UI.",
                    ));
                }
                if components
                    .button(Button::new("Success").variant(ButtonVariant::Primary))
                    .clicked()
                {
                    self.toast_stack.push(
                        Toast::new("Changes saved")
                            .description("Your layout tokens were published successfully.")
                            .intent(ToastIntent::Success),
                    );
                }
                if components
                    .button(Button::new("Error").variant(ButtonVariant::Ghost))
                    .clicked()
                {
                    self.toast_stack.push(
                        Toast::new("Build failed")
                            .description("A component export is missing a required icon mapping.")
                            .intent(ToastIntent::Destructive),
                    );
                }
                if components
                    .button(Button::new("Clear").variant(ButtonVariant::Ghost))
                    .clicked()
                {
                    self.toast_stack.clear();
                }
            });

        ui.add_space(10.0);
        let canvas_fill = if theme::runtime_for_ui(ui).mode.is_dark() {
            app_background(ui)
        } else {
            TOOLBAR_CANVAS_LIGHT_FILL
        };
        let card_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Border));
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let width = 640.0f32.min(ui.available_width());
            let _ = ui
                .components()
                .card(Card::new().fill(canvas_fill).stroke(card_stroke), |ui| {
                    ui.set_width(width);
                    let (host_rect, _) =
                        ui.allocate_exact_size(vec2(width - 24.0, 300.0), Sense::hover());
                    let _ = ui.scope_builder(egui::UiBuilder::new().max_rect(host_rect), |ui| {
                        let mut components = ui.components();
                        let _ = components.label(
                            Label::new("Toast viewport host")
                                .tone(LabelTone::Muted)
                                .size(SMALL_TEXT),
                        );
                        let subtitle = format!("Current placement: {placement_label}");
                        let _ = components.label(
                            Label::new(subtitle.as_str())
                                .tone(LabelTone::Muted)
                                .size(SMALL_TEXT),
                        );

                        components.toast_viewport(
                            &mut self.toast_stack,
                            ToastViewport::new(Id::new("component_showcase_toast_viewport"))
                                .placement(toast_placement_from_index(
                                    self.toast_placement_index.unwrap_or(8),
                                ))
                                .width(280.0),
                        );
                    });
                });
        });
    }

    fn render_drag_board_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().label(
            Label::new("Drag cards between the two regions to see each column update as a stack.")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(10.0);
        let _ = ui.components().drag_board(
            &mut self.drag_board_regions,
            DragBoard::new(
                Id::new("component_showcase_drag_board"),
                "Backlog",
                "Done",
                &DRAG_BOARD_ITEMS,
            ),
        );
    }

    fn render_hierarchy_preview(&mut self, ui: &mut Ui) {
        let _ = ui.components().label(
            Label::new("Select a node to highlight its subtree. Drag row edges to reorder, or drop on a row body to move into that parent.")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT),
        );
        ui.add_space(10.0);
        ui.components().segmented_tabs(
            Id::new("component_showcase_hierarchy_style"),
            &mut self.hierarchy_style_index,
            &HIERARCHY_STYLE_OPTIONS,
        );
        ui.add_space(10.0);
        ui.components().segmented_tabs(
            Id::new("component_showcase_hierarchy_icon_style"),
            &mut self.hierarchy_icon_style_index,
            &HIERARCHY_ICON_STYLE_OPTIONS,
        );
        ui.add_space(10.0);

        let icon_style = if self.hierarchy_icon_style_index == 1 {
            HierarchyIconStyle::Icons
        } else {
            HierarchyIconStyle::Emoji
        };
        let hierarchy_style = if self.hierarchy_style_index == 1 {
            HierarchyStyle::Component
        } else {
            HierarchyStyle::Normal
        };
        let _ = ui.components().hierarchy(
            &mut self.hierarchy_selected_id,
            Hierarchy::new(
                Id::new("component_showcase_hierarchy"),
                &mut self.hierarchy_nodes,
            )
            .width(360.0)
            .icon_style(icon_style)
            .style(hierarchy_style),
        );
    }

    fn render_toolbar_preview(&mut self, ui: &mut Ui) {
        let canvas_fill = if theme::runtime_for_ui(ui).mode.is_dark() {
            app_background(ui)
        } else {
            TOOLBAR_CANVAS_LIGHT_FILL
        };
        let card_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Border));

        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let width = TOOLBAR_PREVIEW_WIDTH.min(ui.available_width());
            let _ = ui
                .components()
                .card(Card::new().fill(canvas_fill).stroke(card_stroke), |ui| {
                    ui.set_width(width);
                    let (host_rect, _) =
                        ui.allocate_exact_size(vec2(width - 24.0, 220.0), Sense::hover());
                    let _ = ui.scope_builder(egui::UiBuilder::new().max_rect(host_rect), |ui| {
                        let _ = ui.components().toolbar(
                            Toolbar::new(Id::new("component_showcase_toolbar"))
                                .anchor(Align2::CENTER_TOP)
                                .offset(vec2(0.0, 10.0)),
                            |ui| draw_toolbar_contents(ui, &mut self.toolbar_color_index),
                        );
                    });
                });
        });
    }

    fn render_pagination_preview(&mut self, ui: &mut Ui) {
        let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            let _ = ui.vertical_centered(|ui| {
                let _ = ui.components().pagination(
                    &mut self.pagination_page,
                    Pagination::new(
                        Id::new("component_showcase_pagination"),
                        PAGINATION_PAGE_COUNT,
                    ),
                );

                ui.add_space(14.0);
                let summary = format!("Page {} of {}", self.pagination_page, PAGINATION_PAGE_COUNT);
                let _ = ui.components().label(
                    Label::new(summary.as_str())
                        .tone(LabelTone::Muted)
                        .size(SMALL_TEXT),
                );
            });
        });
    }
}
