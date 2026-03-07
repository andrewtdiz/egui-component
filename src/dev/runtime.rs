use crate::dev::showcase::{render_component_showcase, ComponentShowcaseState};
use crate::theme;
use crate::ui::tokens;
use crate::{ComponentLibraryError, Result};

pub(crate) fn run_showcase_window() -> Result {
    let window_title = "Component Library Showcase";
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
            theme::setup(&creation_context.egui_ctx);
            Ok(Box::new(ShowcaseWindowApp {
                surface: ShowcaseSurface::new(),
            }))
        }),
    )
    .map_err(|error| ComponentLibraryError::Runtime(error.to_string()))
}

pub(crate) struct ShowcaseSurface {
    story: ComponentShowcaseState,
}

impl ShowcaseSurface {
    pub(crate) fn new() -> Self {
        Self {
            story: ComponentShowcaseState::default(),
        }
    }

    pub(crate) fn draw(&mut self, egui_context: &egui::Context) {
        let dark_mode = self.story.dark_mode();
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(tokens::app_background(dark_mode))
                    .inner_margin(egui::Margin::ZERO)
                    .outer_margin(egui::Margin::ZERO)
                    .stroke(egui::Stroke::NONE),
            )
            .show(egui_context, |ui| {
                let rect = ui.max_rect();
                ui.painter()
                    .rect_filled(rect, 0.0, tokens::app_background(dark_mode));
                ui.set_min_size(rect.size());
                render_component_showcase(ui, &mut self.story);
            });
    }
}

struct ShowcaseWindowApp {
    surface: ShowcaseSurface,
}

impl eframe::App for ShowcaseWindowApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.surface.draw(context);
    }
}
