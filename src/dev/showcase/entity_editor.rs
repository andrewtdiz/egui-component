use crate::components::{
    button, card, collapsible, icon, label, number_input, scroll_area, separator, switch, tabs,
    text_input, ButtonProps, ButtonVariant, CardProps, CollapsibleProps, IconProps, LabelProps,
    LabelTone, LabelWeight, NumberInputProps, ScrollAreaProps, SwitchProps, TabOption,
    TextInputProps,
};
use crate::ui::tokens;
use egui::{Color32, CornerRadius, Id, Layout, Stroke, StrokeKind, Ui};

const LEFT_PANEL_TABS: [TabOption<'static>; 2] =
    [TabOption::new(0, "Hierarchy"), TabOption::new(1, "Project")];
const VIEWPORT_TABS: [TabOption<'static>; 3] = [
    TabOption::new(0, "Scene"),
    TabOption::new(1, "Game"),
    TabOption::new(2, "Animation"),
];
const BOTTOM_PANEL_TABS: [TabOption<'static>; 3] = [
    TabOption::new(0, "Console"),
    TabOption::new(1, "Timeline"),
    TabOption::new(2, "Profiler"),
];

const AXIS_X: Color32 = Color32::from_rgb(248, 113, 113);
const AXIS_Y: Color32 = Color32::from_rgb(134, 239, 172);
const AXIS_Z: Color32 = Color32::from_rgb(96, 165, 250);

#[derive(Debug, Clone)]
pub struct EntityEditorStory {
    pub scene_path: String,
    pub hierarchy_query: String,
    pub project_query: String,
    pub inspector_name: String,
    pub left_tab: usize,
    pub viewport_tab: usize,
    pub bottom_tab: usize,
    pub transform_open: bool,
    pub renderer_open: bool,
    pub lighting_open: bool,
    pub use_postfx: bool,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
    pub exposure: f32,
    pub directional_intensity: f32,
}

impl Default for EntityEditorStory {
    fn default() -> Self {
        Self {
            scene_path: "Assets/Scenes/Desert_Outpost.scene".to_owned(),
            hierarchy_query: String::new(),
            project_query: "robot".to_owned(),
            inspector_name: "Player_Robot".to_owned(),
            left_tab: 0,
            viewport_tab: 0,
            bottom_tab: 0,
            transform_open: true,
            renderer_open: true,
            lighting_open: true,
            use_postfx: true,
            cast_shadows: true,
            receive_shadows: true,
            position_x: 12.0,
            position_y: 1.25,
            position_z: -8.5,
            rotation_x: 0.0,
            rotation_y: 182.0,
            rotation_z: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
            exposure: 1.2,
            directional_intensity: 3.8,
        }
    }
}

pub(super) fn render(ui: &mut Ui, story: &mut EntityEditorStory) {
    clamp_story_values(story);
    ui.set_min_size(ui.available_size());

    let _ = egui::TopBottomPanel::top("editor_toolbar_dock")
        .resizable(false)
        .exact_height(56.0)
        .frame(
            egui::Frame::new()
                .fill(tokens::APP_BACKGROUND)
                .inner_margin(egui::Margin::same(8)),
        )
        .show_inside(ui, |ui| {
            draw_toolbar(ui, story);
        });

    let _ = egui::TopBottomPanel::bottom("editor_bottom_dock")
        .resizable(true)
        .default_height(180.0)
        .min_height(140.0)
        .frame(
            egui::Frame::new()
                .fill(tokens::APP_BACKGROUND)
                .inner_margin(egui::Margin::same(8)),
        )
        .show_inside(ui, |ui| {
            draw_bottom_panel(ui, story);
        });

    let _ = egui::SidePanel::left("editor_left_dock")
        .resizable(true)
        .default_width(260.0)
        .min_width(210.0)
        .max_width(420.0)
        .frame(
            egui::Frame::new()
                .fill(tokens::APP_BACKGROUND)
                .inner_margin(egui::Margin::same(8)),
        )
        .show_inside(ui, |ui| {
            draw_left_panel(ui, story);
        });

    let _ = egui::SidePanel::right("editor_right_dock")
        .resizable(true)
        .default_width(316.0)
        .min_width(260.0)
        .max_width(460.0)
        .frame(
            egui::Frame::new()
                .fill(tokens::APP_BACKGROUND)
                .inner_margin(egui::Margin::same(8)),
        )
        .show_inside(ui, |ui| {
            draw_right_panel(ui, story);
        });

    let _ = egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(tokens::APP_BACKGROUND)
                .inner_margin(egui::Margin::same(8)),
        )
        .show_inside(ui, |ui| {
            draw_center_panel(ui, story);
        });
}

fn draw_toolbar(ui: &mut Ui, story: &mut EntityEditorStory) {
    let _ = card(
        ui,
        CardProps::new()
            .fill(tokens::MUTED_SURFACE)
            .stroke(Stroke::new(1.0, tokens::SEPARATOR))
            .padding(8, 8),
        |ui| {
            ui.horizontal(|ui| {
                let _ = icon(
                    ui,
                    IconProps::new("gamepad-2")
                        .size(14.0)
                        .tint(tokens::TEXT_SECONDARY),
                );
                let _ = label(
                    ui,
                    LabelProps::new("Nebula Editor")
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold),
                );

                ui.add_space(8.0);
                let _ = button(ui, ButtonProps::new("Play").variant(ButtonVariant::Primary));
                let _ = button(
                    ui,
                    ButtonProps::new("Pause").variant(ButtonVariant::Secondary),
                );
                let _ = button(ui, ButtonProps::new("Step").variant(ButtonVariant::Ghost));

                ui.add_space(8.0);
                let _ = text_input(
                    ui,
                    &mut story.scene_path,
                    TextInputProps::new().width(330.0).hint_text("Scene path"),
                );

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let _ = toolbar_chip(ui, "settings-2");
                    let _ = toolbar_chip(ui, "save");
                    let _ = toolbar_chip(ui, "redo-2");
                    let _ = toolbar_chip(ui, "undo-2");
                });
            });
        },
    );
}

fn toolbar_chip(ui: &mut Ui, icon_name: &str) {
    let _ = card(
        ui,
        CardProps::new()
            .fill(tokens::INPUT_BACKGROUND)
            .stroke(Stroke::new(1.0, tokens::INPUT_BORDER))
            .padding(6, 4),
        |ui| {
            let _ = icon(
                ui,
                IconProps::new(icon_name)
                    .size(13.0)
                    .tint(tokens::TEXT_SECONDARY),
            );
        },
    );
}

fn draw_left_panel(ui: &mut Ui, story: &mut EntityEditorStory) {
    let _ = card(ui, CardProps::new(), |ui| {
        tabs(
            ui,
            Id::new("game_editor_left_tabs"),
            &mut story.left_tab,
            &LEFT_PANEL_TABS,
        );
        ui.add_space(8.0);

        if story.left_tab == 0 {
            let _ = text_input(
                ui,
                &mut story.hierarchy_query,
                TextInputProps::new()
                    .width(ui.available_width())
                    .hint_text("Search hierarchy"),
            );
            ui.add_space(8.0);
            scroll_area(
                ui,
                ScrollAreaProps::new(Id::new("game_editor_hierarchy_scroll"))
                    .max_height(ui.available_height().max(220.0)),
                |ui| {
                    draw_tree_row(ui, 0.0, "folder-open", "Desert_Outpost", true);
                    draw_tree_row(ui, 14.0, "camera", "Main Camera", false);
                    draw_tree_row(ui, 14.0, "speaker", "Ambience Audio", false);
                    draw_tree_row(ui, 14.0, "sun", "Directional Light", false);
                    draw_tree_row(ui, 14.0, "box", "Environment", false);
                    draw_tree_row(ui, 28.0, "mountain-snow", "Cliffs", false);
                    draw_tree_row(ui, 28.0, "image", "Sand Dunes", false);
                    draw_tree_row(ui, 14.0, "bot", "Player_Robot", true);
                    draw_tree_row(ui, 28.0, "aperture", "Weapon Mount", false);
                    draw_tree_row(ui, 14.0, "bug", "Enemy_Scout_01", false);
                    draw_tree_row(ui, 14.0, "bug", "Enemy_Scout_02", false);
                },
            );
        } else {
            let _ = text_input(
                ui,
                &mut story.project_query,
                TextInputProps::new()
                    .width(ui.available_width())
                    .hint_text("Search assets"),
            );
            ui.add_space(8.0);
            scroll_area(
                ui,
                ScrollAreaProps::new(Id::new("game_editor_project_scroll"))
                    .max_height(ui.available_height().max(220.0)),
                |ui| {
                    draw_asset_row(ui, "folder", "Models");
                    draw_asset_row(ui, "folder", "Materials");
                    draw_asset_row(ui, "folder", "Textures");
                    draw_asset_row(ui, "folder", "Animations");
                    draw_asset_row(ui, "box", "Player_Robot.fbx");
                    draw_asset_row(ui, "image", "Terrain_Albedo.png");
                    draw_asset_row(ui, "file-code", "RobotController.cs");
                },
            );
        }
    });
}

fn draw_tree_row(ui: &mut Ui, indent: f32, icon_name: &str, title: &str, selected: bool) {
    let desired = egui::vec2(ui.available_width(), 26.0);
    let (rect, _response) = ui.allocate_exact_size(desired, egui::Sense::hover());

    let fill = if selected {
        tokens::ROW_HOVER_BG
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(tokens::RADIUS_SM),
        fill,
        Stroke::NONE,
        StrokeKind::Outside,
    );

    let _ = ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(egui::Align::Center)),
        |ui| {
            ui.add_space(indent + 8.0);
            let _ = icon(
                ui,
                IconProps::new(icon_name)
                    .size(12.0)
                    .tint(tokens::TEXT_MUTED),
            );
            let _ = label(
                ui,
                LabelProps::new(title).tone(if selected {
                    LabelTone::Primary
                } else {
                    LabelTone::Secondary
                }),
            );
        },
    );
}

fn draw_asset_row(ui: &mut Ui, icon_name: &str, title: &str) {
    ui.horizontal(|ui| {
        let _ = icon(
            ui,
            IconProps::new(icon_name)
                .size(12.0)
                .tint(tokens::TEXT_MUTED),
        );
        let _ = label(ui, LabelProps::new(title).tone(LabelTone::Secondary));
    });
    ui.add_space(6.0);
}

fn draw_center_panel(ui: &mut Ui, story: &mut EntityEditorStory) {
    let viewport_height = (ui.available_height() - 96.0).max(300.0);

    ui.scope(|ui| {
        let _ = card(
            ui,
            CardProps::new()
                .fill(tokens::MUTED_SURFACE)
                .stroke(Stroke::new(1.0, tokens::SEPARATOR))
                .padding(8, 8),
            |ui| {
                ui.horizontal(|ui| {
                    tabs(
                        ui,
                        Id::new("game_editor_viewport_tabs"),
                        &mut story.viewport_tab,
                        &VIEWPORT_TABS,
                    );
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let _ = toolbar_chip(ui, "panel-right");
                        let _ = toolbar_chip(ui, "panel-bottom");
                        let _ = toolbar_chip(ui, "panel-left");
                    });
                });
            },
        );

        ui.add_space(8.0);
        draw_viewport_canvas(ui, viewport_height);

        ui.add_space(8.0);
        let _ = card(
            ui,
            CardProps::new()
                .fill(tokens::MUTED_SURFACE)
                .stroke(Stroke::new(1.0, tokens::SEPARATOR))
                .padding(8, 8),
            |ui| {
                ui.horizontal(|ui| {
                    let _ = icon(
                        ui,
                        IconProps::new("camera").size(12.0).tint(tokens::TEXT_MUTED),
                    );
                    let _ = label(
                        ui,
                        LabelProps::new("Camera: Perspective").tone(LabelTone::Muted),
                    );
                    ui.add_space(16.0);
                    let _ = icon(
                        ui,
                        IconProps::new("move-3d")
                            .size(12.0)
                            .tint(tokens::TEXT_MUTED),
                    );
                    let _ = label(
                        ui,
                        LabelProps::new("Transform Gizmo: Local").tone(LabelTone::Muted),
                    );
                });
            },
        );
    });
}

fn draw_viewport_canvas(ui: &mut Ui, viewport_height: f32) {
    let _ = card(
        ui,
        CardProps::new()
            .fill(Color32::from_rgb(10, 12, 18))
            .stroke(Stroke::new(1.0, tokens::SEPARATOR))
            .padding(0, 0),
        |ui| {
            let desired = egui::vec2(ui.available_width(), viewport_height);
            let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());

            ui.painter().rect_filled(
                rect,
                CornerRadius::same(tokens::RADIUS_MD),
                Color32::from_rgb(10, 12, 18),
            );

            let columns = 18;
            let rows = 12;
            for i in 1..columns {
                let x = rect.left() + rect.width() * (i as f32 / columns as f32);
                let stroke = Stroke::new(1.0, Color32::from_rgba_premultiplied(68, 74, 92, 52));
                ui.painter().line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    stroke,
                );
            }
            for i in 1..rows {
                let y = rect.top() + rect.height() * (i as f32 / rows as f32);
                let stroke = Stroke::new(1.0, Color32::from_rgba_premultiplied(68, 74, 92, 52));
                ui.painter().line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    stroke,
                );
            }

            let selection = egui::Rect::from_center_size(
                rect.center() + egui::vec2(42.0, 16.0),
                egui::vec2(132.0, 92.0),
            );
            ui.painter().rect(
                selection,
                CornerRadius::same(tokens::RADIUS_SM),
                Color32::from_rgba_premultiplied(56, 189, 248, 30),
                Stroke::new(1.0, Color32::from_rgb(56, 189, 248)),
                StrokeKind::Outside,
            );

            ui.painter().text(
                rect.left_top() + egui::vec2(12.0, 12.0),
                egui::Align2::LEFT_TOP,
                "Scene View - Desert Outpost",
                egui::FontId::new(12.0, egui::FontFamily::Proportional),
                tokens::TEXT_SECONDARY,
            );
            ui.painter().text(
                rect.left_bottom() + egui::vec2(12.0, -12.0),
                egui::Align2::LEFT_BOTTOM,
                "WASD to navigate  |  RMB to orbit",
                egui::FontId::new(11.0, egui::FontFamily::Proportional),
                tokens::TEXT_MUTED,
            );
        },
    );
}

fn draw_right_panel(ui: &mut Ui, story: &mut EntityEditorStory) {
    ui.scope(|ui| {
        scroll_area(
            ui,
            ScrollAreaProps::new(Id::new("game_editor_inspector_scroll"))
                .max_height(ui.available_height().max(220.0)),
            |ui| {
                let _ = card(ui, CardProps::new(), |ui| {
                    ui.horizontal(|ui| {
                        let _ = icon(
                            ui,
                            IconProps::new("sliders-horizontal")
                                .size(12.0)
                                .tint(tokens::TEXT_SECONDARY),
                        );
                        let _ = label(
                            ui,
                            LabelProps::new("Inspector")
                                .tone(LabelTone::Primary)
                                .weight(LabelWeight::Semibold),
                        );
                    });

                    ui.add_space(8.0);
                    let _ = text_input(
                        ui,
                        &mut story.inspector_name,
                        TextInputProps::new()
                            .width(ui.available_width())
                            .hint_text("Selected object"),
                    );

                    ui.add_space(8.0);
                    draw_inspector_section(
                        ui,
                        "editor_inspector_transform",
                        "Transform",
                        "move-3d",
                        Color32::from_rgb(234, 179, 8),
                        &mut story.transform_open,
                        |ui| {
                            draw_vec3_row(
                                ui,
                                "Position",
                                "inspector_pos_x",
                                &mut story.position_x,
                                "inspector_pos_y",
                                &mut story.position_y,
                                "inspector_pos_z",
                                &mut story.position_z,
                                -2000.0..=2000.0,
                            );
                            draw_vec3_row(
                                ui,
                                "Rotation",
                                "inspector_rot_x",
                                &mut story.rotation_x,
                                "inspector_rot_y",
                                &mut story.rotation_y,
                                "inspector_rot_z",
                                &mut story.rotation_z,
                                -360.0..=360.0,
                            );
                            draw_vec3_row(
                                ui,
                                "Scale",
                                "inspector_scale_x",
                                &mut story.scale_x,
                                "inspector_scale_y",
                                &mut story.scale_y,
                                "inspector_scale_z",
                                &mut story.scale_z,
                                0.01..=100.0,
                            );
                        },
                    );

                    ui.add_space(8.0);
                    draw_inspector_section(
                        ui,
                        "editor_inspector_renderer",
                        "Renderer",
                        "box",
                        Color32::from_rgb(96, 165, 250),
                        &mut story.renderer_open,
                        |ui| {
                            let _ = switch(
                                ui,
                                &mut story.cast_shadows,
                                SwitchProps::new().label("Cast Shadows"),
                            );
                            let _ = switch(
                                ui,
                                &mut story.receive_shadows,
                                SwitchProps::new().label("Receive Shadows"),
                            );
                            ui.add_space(6.0);
                            let _ = text_input(
                                ui,
                                &mut story.project_query,
                                TextInputProps::new()
                                    .width(ui.available_width())
                                    .hint_text("Material: M_Robot_Body"),
                            );
                        },
                    );

                    ui.add_space(8.0);
                    draw_inspector_section(
                        ui,
                        "editor_inspector_lighting",
                        "Lighting",
                        "sun",
                        Color32::from_rgb(250, 204, 21),
                        &mut story.lighting_open,
                        |ui| {
                            let _ = switch(
                                ui,
                                &mut story.use_postfx,
                                SwitchProps::new().label("Enable Post FX"),
                            );
                            ui.add_space(6.0);
                            draw_single_number(
                                ui,
                                "Exposure",
                                "inspector_exposure",
                                &mut story.exposure,
                                0.1..=5.0,
                            );
                            draw_single_number(
                                ui,
                                "Sun Intensity",
                                "inspector_sun_intensity",
                                &mut story.directional_intensity,
                                0.0..=12.0,
                            );
                        },
                    );
                });
            },
        );
    });
}

fn draw_inspector_section(
    ui: &mut Ui,
    id: &'static str,
    title: &'static str,
    icon_name: &'static str,
    icon_tint: Color32,
    open: &mut bool,
    add: impl FnOnce(&mut Ui),
) {
    let _ = card(
        ui,
        CardProps::new()
            .fill(tokens::MUTED_SURFACE)
            .stroke(Stroke::new(1.0, tokens::SEPARATOR))
            .padding(8, 8),
        |ui| {
            let is_open = *open;
            let _ = collapsible(
                ui,
                open,
                CollapsibleProps::new(Id::new(id), title)
                    .open(is_open)
                    .leading_icon(icon_name)
                    .leading_icon_tint(icon_tint)
                    .trailing_icon("ellipsis_vertical")
                    .trailing_icon_tint(tokens::TEXT_MUTED),
                |ui| {
                    ui.add_space(6.0);
                    add(ui);
                },
            );
        },
    );
}

#[expect(clippy::too_many_arguments)]
fn draw_vec3_row(
    ui: &mut Ui,
    label_text: &'static str,
    x_id: &'static str,
    x: &mut f32,
    y_id: &'static str,
    y: &mut f32,
    z_id: &'static str,
    z: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) {
    let row_label_width = 64.0;
    ui.horizontal(|ui| {
        let _ = ui.allocate_ui_with_layout(
            egui::vec2(row_label_width, ui.spacing().interact_size.y),
            Layout::left_to_right(egui::Align::Center),
            |ui| {
                let _ = label(
                    ui,
                    LabelProps::new(label_text)
                        .tone(LabelTone::Secondary)
                        .weight(LabelWeight::Semibold),
                );
            },
        );

        ui.spacing_mut().item_spacing.x = 4.0;
        let width_each = ((ui.available_width() - 8.0) / 3.0).max(78.0);
        let _ = number_input(
            ui,
            x,
            NumberInputProps::new(Id::new(x_id))
                .width(width_each)
                .range(range.clone())
                .decimals(2)
                .prefix("X")
                .prefix_tint(AXIS_X)
                .prefix_align_left(),
        );
        let _ = number_input(
            ui,
            y,
            NumberInputProps::new(Id::new(y_id))
                .width(width_each)
                .range(range.clone())
                .decimals(2)
                .prefix("Y")
                .prefix_tint(AXIS_Y)
                .prefix_align_left(),
        );
        let _ = number_input(
            ui,
            z,
            NumberInputProps::new(Id::new(z_id))
                .width(width_each)
                .range(range)
                .decimals(2)
                .prefix("Z")
                .prefix_tint(AXIS_Z)
                .prefix_align_left(),
        );
    });
}

fn draw_single_number(
    ui: &mut Ui,
    label_text: &'static str,
    id: &'static str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) {
    ui.horizontal(|ui| {
        let _ = ui.allocate_ui_with_layout(
            egui::vec2(92.0, ui.spacing().interact_size.y),
            Layout::left_to_right(egui::Align::Center),
            |ui| {
                let _ = label(
                    ui,
                    LabelProps::new(label_text)
                        .tone(LabelTone::Secondary)
                        .weight(LabelWeight::Semibold),
                );
            },
        );
        let _ = number_input(
            ui,
            value,
            NumberInputProps::new(Id::new(id))
                .width((ui.available_width() - 2.0).max(120.0))
                .range(range)
                .decimals(2),
        );
    });
}

fn draw_bottom_panel(ui: &mut Ui, story: &mut EntityEditorStory) {
    let _ = card(
        ui,
        CardProps::new()
            .fill(tokens::MUTED_SURFACE)
            .stroke(Stroke::new(1.0, tokens::SEPARATOR))
            .padding(8, 8),
        |ui| {
            tabs(
                ui,
                Id::new("game_editor_bottom_tabs"),
                &mut story.bottom_tab,
                &BOTTOM_PANEL_TABS,
            );
            ui.add_space(6.0);
            let _ = separator(ui);
            ui.add_space(6.0);

            match story.bottom_tab {
                0 => {
                    scroll_area(
                        ui,
                        ScrollAreaProps::new(Id::new("game_editor_console_scroll"))
                            .max_height(124.0),
                        |ui| {
                            draw_console_line(
                                ui,
                                "triangle-alert",
                                "Shader compile warning in M_Sand_Detail",
                            );
                            draw_console_line(ui, "bug", "NavMesh bake complete in 1.28s");
                            draw_console_line(
                                ui,
                                "terminal",
                                "Build pipeline idle - no pending tasks",
                            );
                            draw_console_line(
                                ui,
                                "wrench",
                                "Imported 4 textures to Assets/Textures",
                            );
                        },
                    );
                }
                1 => {
                    draw_timeline_track(ui, "Camera Shake", "00:01.20 - 00:03.00");
                    draw_timeline_track(ui, "Enemy Spawn Wave", "00:02.40 - 00:05.80");
                    draw_timeline_track(ui, "Music Transition", "00:04.10 - 00:08.90");
                }
                _ => {
                    draw_profiler_row(ui, "Frame Time", "11.8 ms");
                    draw_profiler_row(ui, "Draw Calls", "438");
                    draw_profiler_row(ui, "Triangles", "1.92 M");
                    draw_profiler_row(ui, "GPU Memory", "2.4 GB");
                }
            }
        },
    );
}

fn draw_console_line(ui: &mut Ui, icon_name: &str, text: &str) {
    ui.horizontal(|ui| {
        let _ = icon(
            ui,
            IconProps::new(icon_name)
                .size(12.0)
                .tint(tokens::TEXT_MUTED),
        );
        let _ = label(ui, LabelProps::new(text).tone(LabelTone::Secondary));
    });
    ui.add_space(6.0);
}

fn draw_timeline_track(ui: &mut Ui, label_text: &str, range: &str) {
    let _ = card(
        ui,
        CardProps::new()
            .fill(tokens::INPUT_BACKGROUND)
            .stroke(Stroke::new(1.0, tokens::INPUT_BORDER))
            .padding(8, 6),
        |ui| {
            ui.horizontal(|ui| {
                let _ = icon(
                    ui,
                    IconProps::new("film").size(12.0).tint(tokens::TEXT_MUTED),
                );
                let _ = label(
                    ui,
                    LabelProps::new(label_text)
                        .tone(LabelTone::Primary)
                        .weight(LabelWeight::Semibold),
                );
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let _ = label(ui, LabelProps::new(range).tone(LabelTone::Muted));
                });
            });
        },
    );
    ui.add_space(6.0);
}

fn draw_profiler_row(ui: &mut Ui, metric: &str, value: &str) {
    ui.horizontal(|ui| {
        let _ = label(
            ui,
            LabelProps::new(metric)
                .tone(LabelTone::Secondary)
                .weight(LabelWeight::Semibold),
        );
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            let _ = label(ui, LabelProps::new(value).tone(LabelTone::Primary));
        });
    });
    ui.add_space(6.0);
}

fn clamp_story_values(story: &mut EntityEditorStory) {
    if story.left_tab >= LEFT_PANEL_TABS.len() {
        story.left_tab = 0;
    }
    if story.viewport_tab >= VIEWPORT_TABS.len() {
        story.viewport_tab = 0;
    }
    if story.bottom_tab >= BOTTOM_PANEL_TABS.len() {
        story.bottom_tab = 0;
    }

    story.position_x = story.position_x.clamp(-2000.0, 2000.0);
    story.position_y = story.position_y.clamp(-2000.0, 2000.0);
    story.position_z = story.position_z.clamp(-2000.0, 2000.0);
    story.rotation_x = story.rotation_x.clamp(-360.0, 360.0);
    story.rotation_y = story.rotation_y.clamp(-360.0, 360.0);
    story.rotation_z = story.rotation_z.clamp(-360.0, 360.0);
    story.scale_x = story.scale_x.clamp(0.01, 100.0);
    story.scale_y = story.scale_y.clamp(0.01, 100.0);
    story.scale_z = story.scale_z.clamp(0.01, 100.0);
    story.exposure = story.exposure.clamp(0.1, 5.0);
    story.directional_intensity = story.directional_intensity.clamp(0.0, 12.0);
}
