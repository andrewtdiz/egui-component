
fn draw_toolbar_contents(ui: &mut Ui, toolbar_color_index: &mut usize) {
    let selected_stroke = Stroke::new(1.0, theme::color(ui, ColorRole::Foreground));

    let _ = show_row(ui, 6.0, |ui| {
        for (index, label) in TOOLBAR_ACTION_OPTIONS.iter().copied().enumerate() {
            let _ = ui
                .components()
                .button(Button::new(label).variant(ButtonVariant::Ghost));
            if index == 1 {
                let _ = ui.components().button(
                    Button::icon_only("crown")
                        .variant(ButtonVariant::Ghost)
                        .icon_size(13.0)
                        .icon_tint(Color32::from_rgb(216, 168, 83)),
                );
            }
        }

        draw_toolbar_divider(ui);

        for (index, fill) in TOOLBAR_SWATCHES.iter().copied().enumerate() {
            let stroke = if *toolbar_color_index == index {
                selected_stroke
            } else {
                Stroke::NONE
            };
            if ui
                .components()
                .button(Button::color_only(
                    Color::new(fill).size(16.0).stroke(stroke),
                ))
                .clicked()
            {
                *toolbar_color_index = index;
            }
        }

        draw_toolbar_divider(ui);
        let _ = ui
            .components()
            .button(Button::icon_only("square-menu").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::icon_only("rotate-ccw").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::icon_only("crop").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::new("Flip").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::icon_only("grid-3x3").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::new("Animate").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::new("Position").variant(ButtonVariant::Ghost));
        let _ = ui
            .components()
            .button(Button::icon_only("paint-roller").variant(ButtonVariant::Ghost));
    });
}

fn draw_toolbar_divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(1.0, 16.0), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(1.0, theme::color(ui, ColorRole::Border)),
    );
}

fn draw_canva_brand_nav_item(
    ui: &mut Ui,
    label: &str,
    selected: bool,
    width: f32,
) -> egui::Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let normal_fill = Color32::TRANSPARENT;
    let selected_fill = CANVA_BRAND_ACCENT.linear_multiply(0.28);
    let hover_fill = crate::ui::tokens::button_secondary_hover_bg(runtime).linear_multiply(0.7);
    let foreground = if selected {
        theme::color(ui, ColorRole::Foreground)
    } else {
        text_secondary(ui)
    };
    let (rect, response) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click());

    let fill = if selected {
        selected_fill
    } else if response.hovered() {
        hover_fill
    } else {
        normal_fill
    };

    if selected || response.hovered() {
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(radius_md(ui)), fill);
    }

    ui.painter().text(
        egui::pos2(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        crate::ui::typography::body_font(),
        foreground,
    );

    response
}

fn draw_canva_brand_vertical_divider(ui: &mut Ui, height: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(1.0, height), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(1.0, theme::color(ui, ColorRole::Border)),
    );
}

fn render_canva_brand_detail(ui: &mut Ui, selected_index: usize) {
    match selected_index {
        0 => render_canva_brand_overview(ui),
        1 => render_canva_brand_guidelines(ui),
        2 => render_canva_brand_logo_grid(ui),
        3 => render_canva_brand_color_swatches(ui),
        4 => render_canva_brand_font_list(ui),
        5 => render_canva_brand_voice_cards(ui),
        6 | 7 => render_canva_brand_media_grid(ui, selected_index == 6),
        8 => render_canva_brand_icon_grid(ui),
        _ => render_canva_brand_chart_cards(ui),
    }
}

fn render_canva_brand_overview(ui: &mut Ui) {
    let hero_height = 192.0;
    let full_width = ui.available_width();
    let hero_rect = ui
        .allocate_exact_size(vec2(full_width, hero_height), Sense::hover())
        .0;
    paint_canva_brand_hero(ui, hero_rect);

    ui.add_space(12.0);
    let _ = ui.components().label(
        Label::new(
            "Apply your brand colors, fonts, logo, and much more effortlessly to every design",
        )
        .tone(LabelTone::Secondary)
        .weight(LabelWeight::Semibold),
    );
    ui.add_space(12.0);
    let _ = ui.components().button(
        Button::new("Try Business for 30 days")
            .variant(ButtonVariant::Primary)
            .leading_icon("crown")
            .icon_size(15.0)
            .min_size(vec2(full_width, 36.0))
            .label_weight(ButtonLabelWeight::Medium),
    );
    ui.add_space(10.0);
    let _ = ui.components().button(
        Button::new("Start with 3 free colors")
            .variant(ButtonVariant::Secondary)
            .leading_icon("plus")
            .icon_size(16.0)
            .min_size(vec2(full_width, 36.0))
            .label_weight(ButtonLabelWeight::Medium),
    );
}

fn paint_canva_brand_hero(ui: &Ui, rect: egui::Rect) {
    let painter = ui.painter();
    let radius = egui::CornerRadius::same(radius_md(ui));
    painter.rect_filled(rect, radius, Color32::from_rgb(92, 180, 206));

    painter.text(
        egui::pos2(rect.left() + 18.0, rect.center().y + 6.0),
        Align2::LEFT_CENTER,
        "Aa",
        crate::ui::typography::bold_font(74.0),
        Color32::from_rgb(226, 246, 247),
    );

    let swatch_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 18.0, rect.bottom() - 34.0),
        vec2(140.0, 18.0),
    );
    let swatch_width = swatch_rect.width() / 3.0;
    for (index, fill) in [
        Color32::from_rgb(25, 73, 208),
        Color32::from_rgb(189, 86, 230),
        Color32::from_rgb(246, 198, 78),
    ]
    .iter()
    .enumerate()
    {
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(
                    swatch_rect.left() + swatch_width * index as f32,
                    swatch_rect.top(),
                ),
                vec2(swatch_width, swatch_rect.height()),
            ),
            egui::CornerRadius::same(6),
            *fill,
        );
    }

    let portrait_center = egui::pos2(rect.right() - 64.0, rect.center().y - 2.0);
    painter.circle_filled(portrait_center, 42.0, CANVA_BRAND_ACCENT);
    painter.circle_filled(
        egui::pos2(portrait_center.x + 8.0, portrait_center.y + 2.0),
        28.0,
        Color32::from_rgb(166, 116, 63),
    );
    painter.rect_filled(
        egui::Rect::from_min_size(
            egui::pos2(rect.right() - 104.0, rect.bottom() - 62.0),
            vec2(68.0, 36.0),
        ),
        egui::CornerRadius::same(18),
        Color32::from_rgb(234, 173, 124),
    );

    let badge_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 112.0, rect.top() + 48.0),
        vec2(40.0, 22.0),
    );
    painter.rect_filled(
        badge_rect,
        egui::CornerRadius::same(10),
        CANVA_BRAND_ACCENT.linear_multiply(0.94),
    );
    painter.text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        "Andy",
        crate::ui::typography::semibold_font(10.0),
        Color32::WHITE,
    );
}

fn render_canva_brand_guidelines(ui: &mut Ui) {
    let _ = ui.components().label(
        Label::new("Guidelines")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    for (title, body) in [
        (
            "Logo spacing",
            "Keep a minimum clear-space area equal to the icon height.",
        ),
        (
            "Tone of voice",
            "Friendly, plainspoken, and direct across marketing and support copy.",
        ),
        (
            "Primary usage",
            "Use the soft orange accent only for primary actions and highlights.",
        ),
    ] {
        draw_canva_brand_info_card(ui, title, body);
        ui.add_space(10.0);
    }
}

fn draw_canva_brand_info_card(ui: &mut Ui, title: &str, body: &str) {
    let fill = theme::color(ui, ColorRole::Card);
    let _ = ui
        .components()
        .card(Card::new().padding(12, 12).fill(fill), |ui| {
            let _ = ui.components().label(
                Label::new(title)
                    .tone(LabelTone::Primary)
                    .weight(LabelWeight::Semibold),
            );
            ui.add_space(6.0);
            let _ = ui
                .components()
                .label(Label::new(body).tone(LabelTone::Secondary));
        });
}

fn render_canva_brand_logo_grid(ui: &mut Ui) {
    let _ = ui.components().label(
        Label::new("Logos")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    draw_canva_brand_asset_grid(ui, 2, 6, false, "logo");
}

fn render_canva_brand_color_swatches(ui: &mut Ui) {
    let border = Stroke::new(1.0, theme::color(ui, ColorRole::Border));
    let _ = ui.components().label(
        Label::new("Colors")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(12.0);
    for (name, hex, fill) in [
        ("Primary Purple", "#7C3AED", Color32::from_rgb(124, 58, 237)),
        ("Ocean Blue", "#3B82F6", Color32::from_rgb(59, 130, 246)),
        ("Warm Gold", "#F5C04A", Color32::from_rgb(245, 192, 74)),
        ("Ink", "#111827", Color32::from_rgb(17, 24, 39)),
    ] {
        let _ = show_row(ui, 10.0, |ui| {
            let _ = ui
                .components()
                .color(Color::new(fill).size(28.0).stroke(border));
            let _ = show_column(ui, 2.0, |ui| {
                let _ = ui.components().label(
                    Label::new(name)
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold),
                );
                let _ = ui
                    .components()
                    .label(Label::new(hex).tone(LabelTone::Muted));
            });
        });
        ui.add_space(10.0);
    }
}

fn render_canva_brand_font_list(ui: &mut Ui) {
    let _ = ui.components().label(
        Label::new("Fonts")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    for (title, sample) in [
        ("Display Serif", "The quick brown fox"),
        ("UI Sans", "Design systems that scale"),
        ("Editorial Italic", "Bold ideas, clean layouts"),
    ] {
        draw_canva_brand_info_card(ui, title, sample);
        ui.add_space(10.0);
    }
}

fn render_canva_brand_voice_cards(ui: &mut Ui) {
    let _ = ui.components().label(
        Label::new("Brand voice")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    for (title, body) in [
        (
            "Confident",
            "Clear calls to action and high-clarity product language.",
        ),
        ("Warm", "Conversational phrasing without overexplaining."),
        (
            "Modern",
            "Short sentences, active verbs, and clean visual hierarchy.",
        ),
    ] {
        draw_canva_brand_info_card(ui, title, body);
        ui.add_space(10.0);
    }
}

fn render_canva_brand_media_grid(ui: &mut Ui, photo_mode: bool) {
    let title = if photo_mode { "Photos" } else { "Graphics" };
    let prefix = if photo_mode { "photo" } else { "graphic" };
    let _ = ui.components().label(
        Label::new(title)
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    draw_canva_brand_asset_grid(ui, 2, 6, true, prefix);
}

fn render_canva_brand_icon_grid(ui: &mut Ui) {
    let _ = ui.components().label(
        Label::new("Icons")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    draw_canva_brand_asset_grid(ui, 3, 9, false, "icon");
}

fn render_canva_brand_chart_cards(ui: &mut Ui) {
    let _ = ui.components().label(
        Label::new("Charts")
            .tone(LabelTone::Primary)
            .weight(LabelWeight::Semibold),
    );
    ui.add_space(10.0);
    draw_canva_brand_asset_grid(ui, 2, 4, false, "chart");
}

fn draw_canva_brand_asset_grid(ui: &mut Ui, columns: usize, count: usize, tall: bool, kind: &str) {
    let tile_height = if tall { 120.0 } else { 92.0 };
    let _ = show_tile_grid(
        ui,
        Some(columns),
        120.0,
        10.0,
        Some(tile_height),
        None,
        count,
        |ui, index, rect| {
            paint_canva_brand_asset_tile(ui, rect, kind, index);
        },
    );
}

fn paint_canva_brand_asset_tile(ui: &Ui, rect: egui::Rect, kind: &str, index: usize) {
    let painter = ui.painter();
    let radius = egui::CornerRadius::same(radius_md(ui));
    let border = Stroke::new(1.0, theme::color(ui, ColorRole::Border));

    let (fill, accent) = match (kind, index % 5) {
        ("logo", 0) => (
            Color32::from_rgb(38, 49, 79),
            Color32::from_rgb(124, 173, 255),
        ),
        ("logo", 1) => (
            Color32::from_rgb(243, 244, 246),
            Color32::from_rgb(79, 70, 229),
        ),
        ("photo", 0) => (
            Color32::from_rgb(200, 209, 223),
            Color32::from_rgb(102, 126, 173),
        ),
        ("photo", 1) => (
            Color32::from_rgb(184, 159, 122),
            Color32::from_rgb(237, 218, 185),
        ),
        ("graphic", 0) => (
            Color32::from_rgb(114, 84, 196),
            Color32::from_rgb(255, 202, 58),
        ),
        ("graphic", 1) => (
            Color32::from_rgb(61, 166, 177),
            Color32::from_rgb(255, 255, 255),
        ),
        ("icon", 0) => (
            Color32::from_rgb(32, 38, 56),
            Color32::from_rgb(182, 197, 255),
        ),
        ("icon", 1) => (
            Color32::from_rgb(49, 58, 91),
            Color32::from_rgb(255, 205, 86),
        ),
        ("chart", 0) => (
            Color32::from_rgb(35, 46, 66),
            Color32::from_rgb(114, 152, 255),
        ),
        ("chart", 1) => (
            Color32::from_rgb(70, 54, 105),
            Color32::from_rgb(222, 133, 255),
        ),
        _ => (
            Color32::from_rgb(74, 80, 96),
            Color32::from_rgb(212, 219, 233),
        ),
    };

    painter.rect_filled(rect, radius, fill);
    match kind {
        "logo" => {
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Aa",
                crate::ui::typography::bold_font(36.0),
                accent,
            );
        }
        "icon" => {
            painter.circle_filled(rect.center(), rect.width() * 0.22, accent);
            painter.line_segment(
                [
                    egui::pos2(rect.center().x, rect.top() + rect.height() * 0.18),
                    egui::pos2(rect.center().x, rect.bottom() - rect.height() * 0.18),
                ],
                Stroke::new(2.0, fill.linear_multiply(0.6)),
            );
        }
        "chart" => {
            for step in 0..4 {
                let bar_width = rect.width() * 0.12;
                let x = rect.left() + rect.width() * (0.18 + step as f32 * 0.18);
                let height = rect.height() * (0.2 + step as f32 * 0.12);
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(x, rect.bottom() - height - 12.0),
                        vec2(bar_width, height),
                    ),
                    egui::CornerRadius::same(4),
                    accent,
                );
            }
        }
        _ => {
            painter.rect_filled(
                rect.shrink2(vec2(rect.width() * 0.18, rect.height() * 0.18)),
                egui::CornerRadius::same(8),
                accent.linear_multiply(0.85),
            );
            painter.circle_filled(
                egui::pos2(
                    rect.right() - rect.width() * 0.14,
                    rect.top() + rect.height() * 0.18,
                ),
                rect.width() * 0.08,
                Color32::from_rgba_unmultiplied(255, 255, 255, 110),
            );
        }
    }
    painter.rect_stroke(rect, radius, border, egui::StrokeKind::Inside);
}

fn draw_canva_edit_tool_row(ui: &mut Ui, selected_index: &mut usize) {
    ScrollArea::horizontal()
        .id_salt("component_showcase_canva_edit_tool_row")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let _ = show_row(ui, 8.0, |ui| {
                for (index, option) in CANVA_EDIT_SELECTION_OPTIONS.iter().enumerate() {
                    let selected = *selected_index == index;
                    if draw_canva_edit_tool_chip(ui, option, selected).clicked() {
                        *selected_index = index;
                    }
                }
            });
        });
}

fn draw_canva_edit_tool_chip(
    ui: &mut Ui,
    option: &CanvaEditChip<'_>,
    selected: bool,
) -> egui::Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let muted_foreground = crate::ui::tokens::text_muted(runtime);
    let border = if selected {
        CANVA_EDIT_ACCENT
    } else {
        crate::ui::tokens::button_secondary_border(runtime)
    };
    let fill = if selected {
        CANVA_EDIT_ACCENT.linear_multiply(0.22)
    } else {
        crate::ui::tokens::button_secondary_bg(runtime)
    };
    let foreground = if selected {
        theme::color(ui, ColorRole::Foreground)
    } else {
        muted_foreground.lerp_to_gamma(theme::color(ui, ColorRole::Foreground), 0.55)
    };
    let width = (option.label.len() as f32 * 7.4 + 44.0).max(76.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, 40.0), Sense::click());

    let fill = if response.is_pointer_button_down_on() {
        fill.linear_multiply(0.92)
    } else if response.hovered() && !selected {
        fill.linear_multiply(1.08)
    } else {
        fill
    };

    ui.painter().rect(
        rect,
        egui::CornerRadius::same(radius_md(ui)),
        fill,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );

    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 18.0, rect.center().y),
        vec2(18.0, 18.0),
    );
    if let Some(image) = crate::icons::image(ui.ctx(), option.icon, 18.0) {
        let _ = image.tint(foreground).paint_at(ui, icon_rect);
    }

    ui.painter().text(
        egui::pos2(rect.left() + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        option.label,
        crate::ui::typography::semibold_font(crate::ui::typography::BODY_SIZE),
        foreground,
    );

    response
}

fn draw_canva_edit_navigation_row(ui: &mut Ui, icon: &str, label: &str) {
    let _ = show_section_link_row(ui, icon, label);
}

fn draw_canva_edit_rail(
    ui: &mut Ui,
    id: Id,
    items: &[CanvaEditRailItem<'_>],
    mut selected_index: Option<&mut usize>,
    text_below: bool,
) {
    ScrollArea::horizontal()
        .id_salt(id)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let _ = show_row(ui, 12.0, |ui| {
                for (index, item) in items.iter().enumerate() {
                    let selected = selected_index
                        .as_ref()
                        .is_some_and(|current| **current == index);
                    let response = draw_canva_edit_rail_item(ui, item, selected, text_below, index);
                    if response.clicked() {
                        if let Some(current) = selected_index.as_deref_mut() {
                            *current = index;
                        }
                    }
                }
            });
        });
}

fn draw_canva_edit_rail_item(
    ui: &mut Ui,
    item: &CanvaEditRailItem<'_>,
    selected: bool,
    text_below: bool,
    pattern_index: usize,
) -> egui::Response {
    let tile_size = if text_below { 92.0 } else { 88.0 };
    let total_height = if text_below { 116.0 } else { 112.0 };
    let sense = if selected || text_below {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(tile_size, total_height), sense);
    let tile_rect = egui::Rect::from_min_size(rect.min, vec2(tile_size, tile_size));

    paint_canva_edit_tile(ui, tile_rect, item, pattern_index, selected);

    let label_color = if selected {
        theme::color(ui, ColorRole::Foreground)
    } else {
        text_secondary(ui)
    };
    ui.painter().text(
        egui::pos2(tile_rect.center().x, tile_rect.bottom() + 16.0),
        Align2::CENTER_CENTER,
        item.label,
        crate::ui::typography::semibold_font(SMALL_TEXT),
        label_color,
    );

    response
}

fn paint_canva_edit_tile(
    ui: &Ui,
    rect: egui::Rect,
    item: &CanvaEditRailItem<'_>,
    pattern_index: usize,
    selected: bool,
) {
    let radius = egui::CornerRadius::same(radius_md(ui));
    let border = if selected {
        Stroke::new(2.0, CANVA_EDIT_ACCENT)
    } else {
        Stroke::new(1.0, theme::color(ui, ColorRole::Border))
    };
    let painter = ui.painter();

    painter.rect_filled(rect, radius, item.fill);

    match pattern_index % 5 {
        0 => {
            painter.circle_filled(
                egui::pos2(rect.center().x, rect.center().y + rect.height() * 0.08),
                rect.width() * 0.28,
                item.accent,
            );
            painter.line_segment(
                [
                    egui::pos2(rect.center().x, rect.top() + rect.height() * 0.12),
                    egui::pos2(rect.center().x, rect.bottom() - rect.height() * 0.14),
                ],
                Stroke::new(2.0, Color32::from_rgba_unmultiplied(20, 20, 20, 64)),
            );
        }
        1 => {
            painter.rect_filled(
                rect.shrink2(vec2(rect.width() * 0.17, rect.height() * 0.17)),
                radius,
                item.accent,
            );
            painter.circle_filled(
                egui::pos2(
                    rect.right() - rect.width() * 0.2,
                    rect.top() + rect.height() * 0.22,
                ),
                rect.width() * 0.1,
                Color32::from_rgba_unmultiplied(255, 255, 255, 128),
            );
        }
        2 => {
            painter.circle_filled(
                egui::pos2(rect.center().x, rect.center().y),
                rect.width() * 0.34,
                item.accent,
            );
            painter.circle_filled(
                egui::pos2(rect.center().x, rect.center().y),
                rect.width() * 0.15,
                Color32::from_rgba_unmultiplied(255, 255, 255, 120),
            );
        }
        3 => {
            for step in 0..5 {
                let x = rect.left() + rect.width() * step as f32 / 4.0;
                painter.line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    Stroke::new(8.0, item.accent.linear_multiply(0.7)),
                );
            }
        }
        _ => {
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.left(), rect.bottom() - rect.height() * 0.38),
                    vec2(rect.width(), rect.height() * 0.38),
                ),
                radius,
                item.accent,
            );
            painter.circle_filled(
                egui::pos2(rect.center().x, rect.top() + rect.height() * 0.28),
                rect.width() * 0.18,
                Color32::from_rgba_unmultiplied(255, 255, 255, 110),
            );
        }
    }

    if let Some(badge) = item.badge {
        let badge_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 8.0, rect.bottom() - 22.0),
            vec2(34.0, 16.0),
        );
        painter.rect_filled(badge_rect, egui::CornerRadius::same(8), CANVA_EDIT_ACCENT);
        painter.text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            badge,
            crate::ui::typography::semibold_font(10.0),
            Color32::WHITE,
        );
    }

    painter.rect_stroke(rect, radius, border, egui::StrokeKind::Inside);
}

fn draw_canva_position_button_grid(ui: &mut Ui, actions: &[(&str, &str, bool)]) {
    let _ = show_tile_grid(
        ui,
        Some(2),
        120.0,
        8.0,
        Some(40.0),
        None,
        actions.len(),
        |ui, index, rect| {
            let (label, icon, enabled) = actions[index];
            let _ = draw_canva_position_action_button(ui, rect, index, label, icon, enabled);
        },
    );
}

fn draw_canva_position_action_button(
    ui: &mut Ui,
    rect: egui::Rect,
    index: usize,
    label: &str,
    icon: &str,
    enabled: bool,
) -> egui::Response {
    let runtime = crate::theme::runtime_for_ui(ui);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let response = ui.interact(
        rect,
        ui.id().with(("canva_position_action_button", index, label)),
        sense,
    );

    let is_pressed = enabled && response.is_pointer_button_down_on();
    let is_hovered = enabled && response.hovered();
    let fill = if is_pressed {
        crate::ui::tokens::button_secondary_active_bg(runtime)
    } else if is_hovered {
        crate::ui::tokens::button_secondary_hover_bg(runtime)
    } else {
        crate::ui::tokens::button_secondary_bg(runtime)
    };
    let stroke_color = if is_pressed {
        crate::ui::tokens::button_secondary_active_border(runtime)
    } else if is_hovered {
        crate::ui::tokens::button_secondary_hover_border(runtime)
    } else {
        crate::ui::tokens::button_secondary_border(runtime)
    };
    let foreground = if enabled {
        crate::ui::tokens::text_primary(runtime)
    } else {
        crate::ui::tokens::text_muted(runtime)
    };

    ui.painter().rect(
        rect,
        egui::CornerRadius::same(radius_md(ui)),
        fill,
        Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );

    let content_left = rect.left() + 12.0;
    let icon_size = 18.0;
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(content_left + (icon_size * 0.5), rect.center().y),
        vec2(icon_size, icon_size),
    );
    if let Some(image) = crate::icons::image(ui.ctx(), icon, icon_size) {
        let _ = image.tint(foreground).paint_at(ui, icon_rect);
    }

    ui.painter().text(
        egui::pos2(content_left + icon_size + 8.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        crate::ui::typography::semibold_font(crate::ui::typography::LABEL_SIZE),
        foreground,
    );

    response
}

fn draw_canva_background_placeholder_grid(ui: &mut Ui) {
    let _ = show_tile_grid(ui, None, 84.0, 8.0, None, None, 15, |ui, index, rect| {
        paint_canva_background_placeholder(ui, rect, index);
    });
}

fn paint_canva_background_placeholder(ui: &Ui, rect: egui::Rect, index: usize) {
    let radius = egui::CornerRadius::ZERO;
    let painter = ui.painter();
    let border = Stroke::new(1.0, theme::color(ui, ColorRole::Border));

    match index {
        0 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(241, 240, 236));
            painter.rect_filled(
                egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.center().y)),
                radius,
                Color32::from_rgb(252, 252, 250),
            );
        }
        1 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(228, 222, 208));
            for step in 1..6 {
                let y = rect.top() + (rect.height() * step as f32 / 6.0);
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)),
                );
            }
        }
        2 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(118, 90, 109));
            painter.rect_filled(
                egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.center().y)),
                radius,
                Color32::from_rgb(222, 206, 202),
            );
        }
        3 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(73, 159, 183));
            for step in 0..5 {
                let x = rect.left() + rect.width() * step as f32 / 4.0;
                painter.circle_filled(
                    egui::pos2(x, rect.top() + rect.height() * 0.4),
                    rect.width() * 0.18,
                    Color32::from_rgba_unmultiplied(173, 230, 237, 48),
                );
            }
        }
        4 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(210, 187, 163));
            painter.circle_stroke(
                egui::pos2(
                    rect.center().x + rect.width() * 0.16,
                    rect.center().y + rect.height() * 0.1,
                ),
                rect.width() * 0.34,
                Stroke::new(3.0, Color32::from_rgb(188, 105, 52)),
            );
            painter.line_segment(
                [
                    egui::pos2(rect.center().x - rect.width() * 0.04, rect.top()),
                    egui::pos2(rect.center().x + rect.width() * 0.2, rect.bottom()),
                ],
                Stroke::new(2.0, Color32::from_rgb(76, 45, 26)),
            );
        }
        5 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(224, 198, 129));
            for center in [
                egui::pos2(
                    rect.left() + rect.width() * 0.25,
                    rect.top() + rect.height() * 0.34,
                ),
                egui::pos2(
                    rect.left() + rect.width() * 0.62,
                    rect.top() + rect.height() * 0.52,
                ),
                egui::pos2(
                    rect.left() + rect.width() * 0.46,
                    rect.top() + rect.height() * 0.78,
                ),
            ] {
                painter.circle_filled(
                    center,
                    rect.width() * 0.18,
                    Color32::from_rgba_unmultiplied(255, 243, 195, 72),
                );
            }
        }
        6 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(209, 101, 154));
            for center in [
                egui::pos2(
                    rect.left() + rect.width() * 0.18,
                    rect.top() + rect.height() * 0.2,
                ),
                egui::pos2(
                    rect.left() + rect.width() * 0.62,
                    rect.top() + rect.height() * 0.38,
                ),
                egui::pos2(
                    rect.left() + rect.width() * 0.78,
                    rect.top() + rect.height() * 0.68,
                ),
            ] {
                painter.circle_filled(
                    center,
                    rect.width() * 0.16,
                    Color32::from_rgba_unmultiplied(255, 210, 230, 64),
                );
            }
        }
        7 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(130, 171, 86));
            painter.rect_stroke(
                rect.shrink(rect.width() * 0.18),
                radius,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(248, 247, 220, 120)),
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [rect.center_top(), rect.center_bottom()],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(248, 247, 220, 100)),
            );
        }
        8 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(191, 203, 219));
            for step in 0..4 {
                let y = rect.top() + rect.height() * (0.58 + (step as f32 * 0.08));
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 88)),
                );
            }
        }
        9 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(239, 238, 232));
            for step in 0..8 {
                let y = rect.top() + rect.height() * step as f32 / 8.0;
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 32)),
                );
            }
        }
        10 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(13, 29, 92));
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.top() + rect.height() * 0.68),
                    rect.max,
                ),
                radius,
                Color32::from_rgb(16, 20, 40),
            );
            painter.circle_filled(
                egui::pos2(rect.center().x, rect.top() + rect.height() * 0.36),
                rect.width() * 0.14,
                Color32::from_rgb(245, 213, 136),
            );
        }
        11 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(198, 177, 151));
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.top() + rect.height() * 0.72),
                    rect.max,
                ),
                radius,
                Color32::from_rgb(143, 96, 71),
            );
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(
                        rect.left() + rect.width() * 0.12,
                        rect.top() + rect.height() * 0.62,
                    ),
                    vec2(rect.width() * 0.36, rect.height() * 0.12),
                ),
                radius,
                Color32::from_rgb(112, 73, 52),
            );
        }
        12 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(166, 153, 135));
            for step in 1..6 {
                let x = rect.left() + (rect.width() * step as f32 / 6.0);
                painter.line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    Stroke::new(2.0, Color32::from_rgba_unmultiplied(84, 69, 56, 72)),
                );
            }
        }
        13 => {
            painter.rect_filled(rect, radius, Color32::from_rgb(222, 210, 195));
            for step in 0..6 {
                let y = rect.top() + rect.height() * (0.16 + step as f32 * 0.12);
                painter.line_segment(
                    [
                        egui::pos2(rect.left() + rect.width() * 0.1, y),
                        egui::pos2(rect.right() - rect.width() * 0.1, y - rect.height() * 0.05),
                    ],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(120, 104, 96, 80)),
                );
            }
        }
        _ => {
            painter.rect_filled(rect, radius, Color32::from_rgb(203, 158, 126));
            for step in 0..5 {
                let x = rect.left() + rect.width() * step as f32 / 5.0;
                painter.line_segment(
                    [
                        egui::pos2(x, rect.top() + rect.height() * 0.08),
                        egui::pos2(
                            x + rect.width() * 0.08,
                            rect.bottom() - rect.height() * 0.08,
                        ),
                    ],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(247, 222, 205, 72)),
                );
            }
        }
    }

    painter.rect_stroke(rect, radius, border, egui::StrokeKind::Inside);
}

#[derive(Clone, Copy)]
struct CanvaLayerDragPayload {
    list_id: Id,
    item_id: usize,
}

fn draw_canva_layers_list(
    ui: &mut Ui,
    list_id: Id,
    order: &mut [usize; 3],
    selected_layer_id: &mut usize,
    items: &[CanvaLayerItem],
) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let mut pending_move = None;
    let ordered_ids = ordered_canva_layer_ids(order, items);

    let _ = show_column(ui, 14.0, |ui| {
        for (row_index, item_id) in ordered_ids.iter().copied().enumerate() {
            let item = items
                .iter()
                .find(|candidate| candidate.id == item_id)
                .expect("missing canva layer item");
            let response =
                draw_canva_layer_row(ui, list_id, item, *selected_layer_id == item.id, runtime);

            if response.clicked() {
                *selected_layer_id = item.id;
            }

            if let Some(payload) =
                hovered_canva_layer_payload(ui.ctx(), &response, list_id).filter(|payload| {
                    payload.item_id != item.id && ui.ctx().pointer_interact_pos().is_some()
                })
            {
                let insert_after = ui
                    .ctx()
                    .pointer_interact_pos()
                    .is_some_and(|pointer| pointer.y > response.rect.center().y);
                let y = if insert_after {
                    response.rect.bottom()
                } else {
                    response.rect.top()
                };
                ui.painter().line_segment(
                    [
                        egui::pos2(response.rect.left() + 6.0, y),
                        egui::pos2(response.rect.right() - 6.0, y),
                    ],
                    Stroke::new(3.0, CANVA_EDIT_ACCENT),
                );
                if released_canva_layer_payload(ui.ctx(), &response, list_id).is_some() {
                    pending_move = Some((payload.item_id, row_index + usize::from(insert_after)));
                }
            }
        }
    });

    if let Some((item_id, target_index)) = pending_move {
        move_canva_layer_item(order, item_id, target_index);
        *selected_layer_id = item_id;
    }
}

fn draw_canva_layer_row(
    ui: &mut Ui,
    list_id: Id,
    item: &CanvaLayerItem,
    selected: bool,
    runtime: crate::theme::ThemeRuntime,
) -> Response {
    let row_height = 84.0;
    let row_width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(row_width, row_height), Sense::click());
    let hovered_payload = hovered_canva_layer_payload(ui.ctx(), &response, list_id);
    let fill = if hovered_payload.is_some_and(|payload| payload.item_id != item.id) {
        tokens::button_secondary_hover_bg(runtime)
    } else {
        tokens::button_secondary_bg(runtime)
    };
    let stroke = if selected {
        Stroke::new(2.0, CANVA_EDIT_ACCENT)
    } else {
        Stroke::new(1.0, tokens::separator(runtime))
    };
    paint_canva_layer_row(ui, rect, item, runtime, selected, fill, stroke);

    let payload = CanvaLayerDragPayload {
        list_id,
        item_id: item.id,
    };
    let mut row_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    let _ = row_ui.dnd_drag_source(list_id.with(("layer_row", item.id)), payload, |ui| {
        ui.set_min_size(rect.size());
        paint_canva_layer_row(ui, ui.max_rect(), item, runtime, selected, fill, stroke);
    });

    response.on_hover_cursor(CursorIcon::Grab)
}

fn paint_canva_layer_row(
    ui: &Ui,
    rect: Rect,
    item: &CanvaLayerItem,
    runtime: crate::theme::ThemeRuntime,
    selected: bool,
    fill: Color32,
    stroke: Stroke,
) {
    let painter = ui.painter();
    painter.rect(
        rect,
        CornerRadius::same(tokens::radius_lg(runtime)),
        fill,
        stroke,
        egui::StrokeKind::Outside,
    );
    let dot_color = if selected {
        Color32::from_rgba_unmultiplied(255, 255, 255, 164)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 136)
    };
    let handle_x = rect.left() + 22.0;
    let handle_y = rect.center().y - 14.0;
    for row in 0..3 {
        for column in 0..2 {
            painter.circle_filled(
                egui::pos2(
                    handle_x + column as f32 * 10.0,
                    handle_y + row as f32 * 10.0,
                ),
                2.4,
                dot_color,
            );
        }
    }

    match item.visual {
        CanvaLayerVisual::Sprite => {
            let sprite_rect =
                Rect::from_center_size(rect.center() + vec2(0.0, -1.0), vec2(26.0, 34.0));
            painter.rect_filled(
                sprite_rect,
                CornerRadius::same(8),
                Color32::from_rgb(72, 84, 136),
            );
            painter.rect_filled(
                Rect::from_center_size(sprite_rect.center(), vec2(14.0, 18.0)),
                CornerRadius::same(5),
                Color32::from_rgb(206, 168, 92),
            );
            painter.rect_stroke(
                sprite_rect,
                CornerRadius::same(8),
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 32)),
                egui::StrokeKind::Inside,
            );
        }
        CanvaLayerVisual::Frame => {
            let frame_rect =
                Rect::from_center_size(rect.center() + vec2(0.0, -1.0), vec2(54.0, 54.0));
            painter.rect_stroke(
                frame_rect,
                CornerRadius::same(10),
                Stroke::new(3.0, Color32::from_rgb(159, 191, 235)),
                egui::StrokeKind::Inside,
            );
            painter.rect_filled(
                frame_rect.shrink(6.0),
                CornerRadius::same(8),
                Color32::from_rgb(37, 38, 45),
            );
        }
        CanvaLayerVisual::Text => {
            let text_rect = Rect::from_center_size(
                rect.center() + vec2(-12.0, -1.0),
                vec2(rect.width() * 0.62, 28.0),
            );
            painter.rect_filled(
                text_rect,
                CornerRadius::same(4),
                Color32::from_rgb(36, 35, 42),
            );
            let accent_rect = Rect::from_center_size(
                egui::pos2(rect.right() - 30.0, rect.center().y),
                vec2(20.0, 20.0),
            );
            for step in 0..5 {
                let x = accent_rect.left() + step as f32 * 4.0;
                painter.line_segment(
                    [
                        egui::pos2(x, accent_rect.bottom()),
                        egui::pos2(x + 8.0, accent_rect.top()),
                    ],
                    Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 172)),
                );
            }
        }
    }

    if selected {
        painter.rect_stroke(
            rect.shrink(1.0),
            CornerRadius::same(tokens::radius_lg(runtime)),
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 26)),
            egui::StrokeKind::Inside,
        );
    }
}

fn ordered_canva_layer_ids(order: &[usize; 3], items: &[CanvaLayerItem]) -> Vec<usize> {
    order
        .iter()
        .copied()
        .filter(|item_id| items.iter().any(|candidate| candidate.id == *item_id))
        .collect()
}

fn move_canva_layer_item(order: &mut [usize; 3], item_id: usize, target_index: usize) {
    let Some(source_index) = order.iter().position(|candidate| *candidate == item_id) else {
        return;
    };

    let mut reordered = order.to_vec();
    let item = reordered.remove(source_index);
    let mut target_index = target_index.min(reordered.len());
    if source_index < target_index {
        target_index = target_index.saturating_sub(1);
    }
    reordered.insert(target_index, item);
    order.copy_from_slice(&reordered);
}

fn hovered_canva_layer_payload(
    ctx: &egui::Context,
    response: &Response,
    list_id: Id,
) -> Option<CanvaLayerDragPayload> {
    if !response.contains_pointer() {
        return None;
    }

    egui::DragAndDrop::payload::<CanvaLayerDragPayload>(ctx)
        .map(|payload| *payload)
        .filter(|payload| payload.list_id == list_id)
}

fn released_canva_layer_payload(
    ctx: &egui::Context,
    response: &Response,
    list_id: Id,
) -> Option<CanvaLayerDragPayload> {
    if !response.contains_pointer() || !ctx.input(|input| input.pointer.any_released()) {
        return None;
    }

    let payload = egui::DragAndDrop::payload::<CanvaLayerDragPayload>(ctx)
        .map(|payload| *payload)
        .filter(|payload| payload.list_id == list_id)?;
    let _ = egui::DragAndDrop::take_payload::<CanvaLayerDragPayload>(ctx);
    Some(payload)
}

fn draw_canva_number_field(ui: &mut Ui, label: &str, value: &mut f32, input: NumberInput) {
    let _ = show_column(ui, 6.0, |ui| {
        let _ = ui.components().label(
            Label::new(label)
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT)
                .weight(LabelWeight::Semibold),
        );
        let _ = ui.components().number_input(value, input);
    });
}

fn draw_canva_ratio_field(ui: &mut Ui, width: f32, locked: &mut bool) {
    let _ = show_column(ui, 6.0, |ui| {
        let _ = ui.components().label(
            Label::new("Ratio")
                .tone(LabelTone::Muted)
                .size(SMALL_TEXT)
                .weight(LabelWeight::Semibold),
        );
        if ui
            .components()
            .button(
                Button::icon_only(if *locked { "lock" } else { "lock-open" })
                    .variant(ButtonVariant::Secondary)
                    .icon_size(18.0)
                    .min_size(vec2(width, 30.0)),
            )
            .clicked()
        {
            *locked = !*locked;
        }
    });
}

fn draw_image_tile_metadata_row(ui: &mut Ui, text: &str) {
    let _ = show_row(ui, 6.0, |ui| {
        let mut components = ui.components();
        let _ = components.icon(Icon::new("globe").size(12.0).tint(IMAGE_TILE_META_ACCENT));
        let _ = components.label(Label::new("•").tone(LabelTone::Muted));
        let _ = components.label(Label::new(text).tone(LabelTone::Muted).size(SMALL_TEXT));
    });
}
