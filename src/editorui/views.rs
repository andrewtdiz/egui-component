use std::path::PathBuf;

use crate::editor::ui::icons;
use crate::editor::ui::tokens;
use crate::editorui::component_run_options::ComponentRunOptions;
use crate::editorui::component_showcase::{
    render_component_preview, supported_component_definitions_by_group, ComponentGroup,
    ComponentKind, ComponentStoryState,
};
use crate::editorui::style::setup_editor_context;
use crate::editorui::theme_snapshot::ThemeSnapshot;
use crate::{ClayError, Result};
use egui::{CornerRadius, CursorIcon, Stroke, StrokeKind};

pub(crate) fn run_components_window(snapshot_path: PathBuf) -> Result {
    let window_title = "Clay Engine Components";
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(window_title)
            .with_inner_size([1200.0, 760.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        window_title,
        native_options,
        Box::new(move |creation_context| {
            setup_editor_context(&creation_context.egui_ctx);
            Ok(Box::new(ComponentsWindowApp {
                surface: ComponentsSurface::from_snapshot_path(snapshot_path.clone()),
            }))
        }),
    )
    .map_err(|error| ClayError::PlatformError(error.to_string()))
}

pub(crate) fn run_single_component_window(options: ComponentRunOptions) -> Result {
    let window_title = format!("Clay Engine Component - {}", options.component.display_name());
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(window_title.clone())
            .with_inner_size([options.width as f32, options.height as f32])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        window_title.as_str(),
        native_options,
        Box::new(move |creation_context| {
            setup_editor_context(&creation_context.egui_ctx);
            Ok(Box::new(SingleComponentWindowApp {
                surface: SingleComponentSurface::from_options(options.clone()),
            }))
        }),
    )
    .map_err(|error| ClayError::PlatformError(error.to_string()))
}

pub(crate) struct ComponentsSurface {
    startup_snapshot: Option<ThemeSnapshot>,
    startup_snapshot_applied: bool,
    startup_error: Option<String>,
    selected_component: ComponentKind,
    story: ComponentStoryState,
}

impl ComponentsSurface {
    pub(crate) fn from_snapshot_path(snapshot_path: PathBuf) -> Self {
        let (startup_snapshot, startup_error) =
            match ThemeSnapshot::load_from_path(snapshot_path.as_path()) {
                Ok(snapshot) => (snapshot, None),
                Err(error) => (None, Some(error.to_string())),
            };

        Self {
            startup_snapshot,
            startup_snapshot_applied: false,
            startup_error,
            selected_component: ComponentKind::Button,
            story: ComponentStoryState::default(),
        }
    }

    fn apply_startup_snapshot(&mut self, egui_context: &egui::Context) {
        if self.startup_snapshot_applied {
            return;
        }
        self.startup_snapshot_applied = true;
        if let Some(snapshot) = self.startup_snapshot.take() {
            snapshot.apply(egui_context);
        }
    }

    pub(crate) fn draw(&mut self, egui_context: &egui::Context) {
        icons::setup(egui_context);
        self.apply_startup_snapshot(egui_context);

        egui::SidePanel::left("components_sidebar")
            .default_width(230.0)
            .min_width(180.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(tokens::SIDEBAR_BACKGROUND)
                    .stroke(egui::Stroke::new(1.0, tokens::CARD_BORDER)),
            )
            .show(egui_context, |ui| {
                ui.heading("Components");
                ui.label(
                    egui::RichText::new("Focused subset with room to grow")
                        .size(11.0)
                        .color(tokens::TEXT_MUTED),
                );
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for group in [ComponentGroup::PrimaryPrimitive, ComponentGroup::Composed] {
                            ui.label(
                                egui::RichText::new(group.title())
                                    .size(11.0)
                                    .color(tokens::TEXT_MUTED),
                            );
                            ui.add_space(3.0);
                            for definition in supported_component_definitions_by_group(group) {
                                let selected = self.selected_component == definition.kind;
                                if draw_sidebar_component_row(ui, definition.label, selected)
                                    .clicked()
                                {
                                    self.selected_component = definition.kind;
                                }
                            }
                            ui.add_space(8.0);
                        }
                    });
                if let Some(error) = self.startup_error.as_ref() {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(error)
                            .size(11.0)
                            .color(egui::Color32::from_rgb(220, 120, 120)),
                    );
                }
            });

        egui::CentralPanel::default().show(egui_context, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, tokens::APP_BACKGROUND);

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let width = ui.available_width().min(420.0).max(240.0);
                egui::Frame::new()
                    .fill(tokens::CARD_BACKGROUND)
                    .stroke(egui::Stroke::new(1.0, tokens::CARD_BORDER))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(12, 12))
                    .show(ui, |ui| {
                        ui.set_width(width);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(self.selected_component.display_name())
                                    .font(egui::FontId::new(
                                        12.0,
                                        egui::FontFamily::Name(
                                            "clay-editor-segoe-semibold".into(),
                                        ),
                                    ))
                                    .color(tokens::TEXT_PRIMARY),
                            );
                        });
                        ui.add_space(10.0);
                        egui::ScrollArea::vertical()
                            .id_salt("components_preview_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                    render_component_preview(
                                        ui,
                                        self.selected_component,
                                        &mut self.story,
                                    );
                                });
                            });
                    });
            });
        });
    }
}

pub(crate) struct SingleComponentSurface {
    startup_snapshot: Option<ThemeSnapshot>,
    startup_snapshot_applied: bool,
    startup_error: Option<String>,
    component: ComponentKind,
    story: ComponentStoryState,
}

impl SingleComponentSurface {
    pub(crate) fn from_options(options: ComponentRunOptions) -> Self {
        let (startup_snapshot, startup_error) =
            match ThemeSnapshot::load_from_path(options.theme_snapshot_path.as_path()) {
                Ok(snapshot) => (snapshot, None),
                Err(error) => (None, Some(error.to_string())),
            };

        Self {
            startup_snapshot,
            startup_snapshot_applied: false,
            startup_error,
            component: options.component,
            story: ComponentStoryState::default(),
        }
    }

    fn apply_startup_snapshot(&mut self, egui_context: &egui::Context) {
        if self.startup_snapshot_applied {
            return;
        }
        self.startup_snapshot_applied = true;
        if let Some(snapshot) = self.startup_snapshot.take() {
            snapshot.apply(egui_context);
        }
    }

    pub(crate) fn draw(&mut self, egui_context: &egui::Context) {
        icons::setup(egui_context);
        self.apply_startup_snapshot(egui_context);

        egui::CentralPanel::default().show(egui_context, |ui| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, tokens::APP_BACKGROUND);

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let width = ui.available_width().min(360.0).max(200.0);
                egui::Frame::new()
                    .fill(tokens::CARD_BACKGROUND)
                    .stroke(egui::Stroke::new(1.0, tokens::CARD_BORDER))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(12, 12))
                    .show(ui, |ui| {
                        ui.set_width(width);
                        egui::ScrollArea::vertical()
                            .id_salt("component_preview_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                    render_component_preview(ui, self.component, &mut self.story);
                                });
                                if let Some(error) = self.startup_error.as_ref() {
                                    ui.add_space(8.0);
                                    ui.label(
                                        egui::RichText::new(error)
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(220, 120, 120)),
                                    );
                                }
                            });
                    });
            });
        });
    }
}

fn draw_sidebar_component_row(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let desired_size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let fill = if selected {
        tokens::neutral(700)
    } else if response.hovered() {
        tokens::neutral(800)
    } else {
        egui::Color32::TRANSPARENT
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(5),
        fill,
        Stroke::NONE,
        StrokeKind::Outside,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 7.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::new(12.0, egui::FontFamily::Proportional),
        if selected {
            tokens::TEXT_PRIMARY
        } else {
            tokens::TEXT_SECONDARY
        },
    );
    response.on_hover_cursor(CursorIcon::PointingHand)
}

struct ComponentsWindowApp {
    surface: ComponentsSurface,
}

impl eframe::App for ComponentsWindowApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.surface.draw(context);
    }
}

struct SingleComponentWindowApp {
    surface: SingleComponentSurface,
}

impl eframe::App for SingleComponentWindowApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.surface.draw(context);
    }
}
