use crate::dev::showcase::{render_component_showcase, ComponentShowcaseState};
use crate::theme::{self, ThemeMode, ThemeRuntime};
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
            theme::install(
                &creation_context.egui_ctx,
                theme::ThemeSpec::default(),
                ThemeMode::Dark,
            );
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
        draw_showcase_surface(egui_context, &mut self.story, 0);
    }
}

pub(crate) fn draw_showcase_surface(
    egui_context: &egui::Context,
    state: &mut ComponentShowcaseState,
    reload_generation: u64,
) {
    let runtime = crate::theme::runtime_for_context(egui_context);
    draw_showcase_surface_with_runtime(egui_context, state, reload_generation, runtime);
}

pub(crate) fn draw_showcase_surface_with_runtime(
    egui_context: &egui::Context,
    state: &mut ComponentShowcaseState,
    reload_generation: u64,
    runtime: ThemeRuntime,
) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(tokens::app_background(runtime))
                .inner_margin(egui::Margin::ZERO)
                .outer_margin(egui::Margin::ZERO)
                .stroke(egui::Stroke::NONE),
        )
        .show(egui_context, |ui| {
            let rect = ui.max_rect();
            ui.painter()
                .rect_filled(rect, 0.0, tokens::app_background(runtime));
            ui.set_min_size(rect.size());
            theme::with_theme(ui, runtime.spec, runtime.mode, |ui| {
                render_component_showcase(ui, state, reload_generation, runtime);
            });
        });
}

struct ShowcaseWindowApp {
    surface: ShowcaseSurface,
}

impl eframe::App for ShowcaseWindowApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.surface.draw(context);
    }
}
