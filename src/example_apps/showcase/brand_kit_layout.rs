fn render_canva_brand_kit_layout(app: &mut ShowcaseApp, ui: &mut Ui, panel_width: f32) {
    let left_width = 170.0;
    let gap = 14.0;
    let divider_width = 1.0;

    tui(ui, Id::new("component_showcase_canva_brand_kit_taffy"))
        .reserve_width(panel_width)
        .style(crate::internal_taffy::taffy::Style {
            flex_direction: crate::internal_taffy::taffy::FlexDirection::Row,
            align_items: Some(crate::internal_taffy::taffy::AlignItems::Stretch),
            gap: length(gap),
            size: crate::internal_taffy::taffy::Size {
                width: length(panel_width),
                height: auto(),
            },
            ..Default::default()
        })
        .show(|tui| {
            tui.id("left-pane")
                .style(crate::internal_taffy::taffy::Style {
                    size: crate::internal_taffy::taffy::Size {
                        width: length(left_width),
                        height: auto(),
                    },
                    ..Default::default()
                })
                .egui_layout(Layout::top_down(egui::Align::Min))
                .ui(|ui| {
                    ui.set_width(left_width);

                    let _ = ui.components().text_input(
                        &mut app.canva_brand_query,
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
                        &mut app.canva_brand_select_index,
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
                        let selected = app.canva_brand_category_index == index;
                        if draw_canva_brand_nav_item(ui, category, selected, left_width).clicked() {
                            app.canva_brand_category_index = index;
                        }
                        if index + 1 < CANVA_BRAND_CATEGORIES.len() {
                            ui.add_space(2.0);
                        }
                    }
                });

            tui.id("divider")
                .style(crate::internal_taffy::taffy::Style {
                    size: crate::internal_taffy::taffy::Size {
                        width: length(divider_width),
                        height: length(520.0),
                    },
                    ..Default::default()
                })
                .ui(|ui| {
                    draw_canva_brand_vertical_divider(ui, 520.0);
                });

            tui.id("right-pane")
                .style(crate::internal_taffy::taffy::Style {
                    flex_grow: 1.0,
                    flex_basis: length(0.0),
                    min_size: crate::internal_taffy::taffy::Size {
                        width: length(260.0),
                        height: auto(),
                    },
                    size: crate::internal_taffy::taffy::Size {
                        width: percent(1.0),
                        height: auto(),
                    },
                    ..Default::default()
                })
                .egui_layout(Layout::top_down(egui::Align::Min))
                .ui(|ui| {
                    render_canva_brand_detail_taffy(ui, app.canva_brand_category_index);
                });
        });
}

fn render_canva_brand_detail_taffy(ui: &mut Ui, selected_index: usize) {
    match selected_index {
        2 => render_canva_brand_asset_grid_section_taffy(ui, "Logos", 2, 6, 92.0, "logo"),
        6 => render_canva_brand_asset_grid_section_taffy(ui, "Photos", 2, 6, 120.0, "photo"),
        7 => render_canva_brand_asset_grid_section_taffy(ui, "Graphics", 2, 6, 120.0, "graphic"),
        8 => render_canva_brand_asset_grid_section_taffy(ui, "Icons", 3, 9, 92.0, "icon"),
        9 => render_canva_brand_asset_grid_section_taffy(ui, "Charts", 2, 4, 92.0, "chart"),
        _ => render_canva_brand_detail(ui, selected_index),
    }
}

fn render_canva_brand_asset_grid_section_taffy(
    ui: &mut Ui,
    title: &str,
    columns: usize,
    count: usize,
    tile_height: f32,
    kind: &str,
) {
    let _ = ui.components().label(
        Label::new(title)
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    draw_canva_brand_asset_grid_taffy(ui, columns, count, tile_height, kind);
}

fn draw_canva_brand_asset_grid_taffy(
    ui: &mut Ui,
    columns: usize,
    count: usize,
    tile_height: f32,
    kind: &str,
) {
    tui(
        ui,
        Id::new("component_showcase_canva_brand_asset_grid").with((kind, columns, count)),
    )
    .reserve_available_width()
    .style(crate::internal_taffy::taffy::Style {
        display: crate::internal_taffy::taffy::Display::Grid,
        grid_template_columns: vec![fr(1.0); columns],
        gap: length(10.0),
        size: percent(1.0),
        align_items: Some(crate::internal_taffy::taffy::AlignItems::Stretch),
        justify_items: Some(crate::internal_taffy::taffy::AlignItems::Stretch),
        ..Default::default()
    })
    .show(|tui| {
        for index in 0..count {
            tui.id(tid(("tile", kind, index)))
                .style(crate::internal_taffy::taffy::Style {
                    size: crate::internal_taffy::taffy::Size {
                        width: auto(),
                        height: length(tile_height),
                    },
                    ..Default::default()
                })
                .ui(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), tile_height), Sense::hover());
                    paint_canva_brand_asset_tile(ui, rect, kind, index);
                });
        }
    });
}
