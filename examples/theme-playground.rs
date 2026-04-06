use egui::{CentralPanel, Context, Id, ScrollArea, ViewportBuilder};
use egui_component::prelude::*;

fn row<R>(
    ui: &mut egui::Ui,
    gap: f32,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = gap.max(0.0);
        ui.horizontal(add)
    })
    .inner
}

fn column<R>(
    ui: &mut egui::Ui,
    gap: f32,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = gap.max(0.0);
        ui.vertical(add)
    })
    .inner
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("Theme Playground")
            .with_inner_size([1280.0, 880.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Theme Playground",
        options,
        Box::new(|creation_context| {
            egui_component::theme::install(
                &creation_context.egui_ctx,
                ThemeSpec::preset(BaseColor::Neutral),
                ThemeMode::System,
            );
            Ok(Box::<ThemePlaygroundApp>::default())
        }),
    )
}

struct ThemePlaygroundApp {
    draft_base: BaseColor,
    draft_mode: ThemeMode,
    draft_radius: f32,
    applied_spec: ThemeSpec,
    applied_mode: ThemeMode,
    scoped_base: BaseColor,
    scoped_mode: ThemeMode,
    scoped_radius: f32,
    status: String,
}

impl Default for ThemePlaygroundApp {
    fn default() -> Self {
        let spec = ThemeSpec::preset(BaseColor::Neutral).with_radius(10.0);
        Self {
            draft_base: BaseColor::Neutral,
            draft_mode: ThemeMode::System,
            draft_radius: 10.0,
            applied_spec: spec,
            applied_mode: ThemeMode::System,
            scoped_base: BaseColor::Mauve,
            scoped_mode: ThemeMode::Light,
            scoped_radius: 12.0,
            status: "Adjust the controls to explore the theme API.".to_owned(),
        }
    }
}

impl eframe::App for ThemePlaygroundApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.sync_theme(ctx);

        CentralPanel::default().show(ctx, |ui| {
            let ctx = ui.ctx().clone();
            ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.set_min_width(1180.0);
                ui.add_space(12.0);

                let _ = column(ui, 8.0, |ui| {
                    let mut components = ui.components();
                    let _ = components.label(
                        Label::new("Theme Playground").weight(LabelWeight::Bold).size(22.0),
                    );
                    let _ = components.label(
                        Label::new("Live controls for ThemeSpec, ThemeMode, and scoped theme previews.")
                            .tone(LabelTone::Muted)
                            .size(13.0),
                    );
                });

                ui.add_space(16.0);

                let _ = row(ui, 16.0, |ui| {
                    self.render_controls(ui, &ctx);
                    self.render_previews(ui);
                });
            });
        });
    }
}

impl ThemePlaygroundApp {
    fn sync_theme(&mut self, ctx: &Context) {
        let next_spec = ThemeSpec::preset(self.draft_base).with_radius(self.draft_radius);
        if next_spec != self.applied_spec {
            egui_component::theme::set_theme(ctx, next_spec);
            self.applied_spec = next_spec;
            self.status = format!(
                "Applied {} with radius {:.1}.",
                self.draft_base, self.draft_radius
            );
        }

        if self.draft_mode != self.applied_mode {
            egui_component::theme::set_mode(ctx, self.draft_mode);
            self.applied_mode = self.draft_mode;
            self.status = format!("Applied {} mode.", mode_label(self.draft_mode));
        }
    }

    fn render_controls(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        let _ = ui.components().card(Card::new().padding(18, 18), |ui| {
            let _ = column(ui, 18.0, |ui| {
                {
                    let mut components = ui.components();
                    let _ = components
                        .label(Label::new("Theme controls").weight(LabelWeight::Bold).size(16.0));
                    let _ = components.label(
                        Label::new("Switch the global runtime, then compare it with a scoped override below.")
                            .tone(LabelTone::Muted),
                    );
                }

                self.base_color_picker(ui, ctx);
                self.mode_picker(ui, ctx);
                self.radius_control(ui, ctx);

                let _ = row(ui, 8.0, |ui| {
                    let mut components = ui.components();
                    if components
                        .button(
                            Button::new("Reapply theme")
                                .variant(ButtonVariant::Secondary)
                                .leading_icon("palette"),
                        )
                        .clicked()
                    {
                        let spec = ThemeSpec::preset(self.draft_base).with_radius(self.draft_radius);
                        egui_component::theme::set_theme(ctx, spec);
                        self.applied_spec = spec;
                        self.status = format!(
                            "Reapplied {} with radius {:.1}.",
                            self.draft_base, self.draft_radius
                        );
                    }
                    if components
                        .button(
                            Button::new("Reapply mode")
                                .variant(ButtonVariant::Secondary)
                                .leading_icon(mode_icon(self.draft_mode)),
                        )
                        .clicked()
                    {
                        egui_component::theme::set_mode(ctx, self.draft_mode);
                        self.applied_mode = self.draft_mode;
                        self.status = format!("Reapplied {} mode.", mode_label(self.draft_mode));
                    }
                });

                {
                    let mut components = ui.components();
                    let _ = components.label(Label::new(self.status.as_str()).tone(LabelTone::Muted));
                }
            });
        });
    }

    fn base_color_picker(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        let _ = column(ui, 6.0, |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new("ThemeSpec / base color")
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Secondary),
            );
            let mut selected = Some(base_color_index(self.draft_base));
            let changed = components
                .select(
                    &mut selected,
                    Select::from_id(Id::new("base-color"), &BASE_COLOR_OPTIONS).width(300.0),
                )
                .changed();
            if changed {
                if let Some(index) = selected {
                    self.draft_base = BaseColor::ALL[index];
                    let spec = ThemeSpec::preset(self.draft_base).with_radius(self.draft_radius);
                    egui_component::theme::set_theme(ctx, spec);
                    self.applied_spec = spec;
                    self.status = format!(
                        "Applied {} with radius {:.1}.",
                        self.draft_base, self.draft_radius
                    );
                }
            }
        });
    }

    fn mode_picker(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        let _ = column(ui, 6.0, |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new("ThemeMode")
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Secondary),
            );
            let mut selected_mode = theme_mode_index(self.draft_mode);
            ui.components().segmented_tabs(
                Id::new("theme-mode"),
                &mut selected_mode,
                &THEME_MODE_OPTIONS,
            );
            let next_mode = theme_mode_from_index(selected_mode);
            if next_mode != self.draft_mode {
                self.draft_mode = next_mode;
                egui_component::theme::set_mode(ctx, self.draft_mode);
                self.applied_mode = self.draft_mode;
                self.status = format!("Applied {} mode.", mode_label(self.draft_mode));
            }
        });
    }

    fn radius_control(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        let _ = column(ui, 6.0, |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new("Corner radius")
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Secondary),
            );
            let before = self.draft_radius;
            let _ = components.slider(&mut self.draft_radius, Slider::new(4.0..=18.0).width(300.0));
            if (before - self.draft_radius).abs() > f32::EPSILON {
                let spec = ThemeSpec::preset(self.draft_base).with_radius(self.draft_radius);
                egui_component::theme::set_theme(ctx, spec);
                self.applied_spec = spec;
                self.status = format!(
                    "Applied {} with radius {:.1}.",
                    self.draft_base, self.draft_radius
                );
            }
        });
    }

    fn render_previews(&self, ui: &mut egui::Ui) {
        let _ = column(ui, 16.0, |ui| {
            let _ = ui.components().card(Card::new().padding(18, 18), |ui| {
                let _ = column(ui, 14.0, |ui| {
                    let mut components = ui.components();
                    let _ = components.label(
                        Label::new("Runtime preview")
                            .weight(LabelWeight::Bold)
                            .size(16.0),
                    );
                    let _ = components.label(
                        Label::new(
                            "This section uses the active global theme that was applied above.",
                        )
                        .tone(LabelTone::Muted),
                    );
                    let _ = row(ui, 10.0, |ui| {
                        let mut components = ui.components();
                        let _ = components.button(
                            Button::new("Primary action")
                                .variant(ButtonVariant::Primary)
                                .leading_icon("sparkles"),
                        );
                        let _ = components.button(
                            Button::new("Secondary")
                                .variant(ButtonVariant::Secondary)
                                .trailing_hint("Enter"),
                        );
                    });
                });
            });

            let scoped_spec = ThemeSpec::preset(self.scoped_base).with_radius(self.scoped_radius);
            let _ = egui_component::theme::with_theme(ui, scoped_spec, self.scoped_mode, |ui| {
                let _ = ui.components().card(Card::new().padding(18, 18), |ui| {
                    let _ = column(ui, 14.0, |ui| {
                        let mut components = ui.components();
                        let _ = components.label(
                            Label::new("Scoped preview")
                                .weight(LabelWeight::Bold)
                                .size(16.0),
                        );
                        let _ = components.label(
                            Label::new("This card is rendered through theme::with_theme with its own ThemeSpec and ThemeMode.")
                                .tone(LabelTone::Muted),
                        );
                        let _ = row(ui, 8.0, |ui| {
                            let mut components = ui.components();
                            let _ = components.button(
                                Button::new("Scoped callout")
                                    .variant(ButtonVariant::Primary)
                                    .leading_icon("palette"),
                            );
                            let _ = components.button(
                                Button::new("Scoped reset")
                                    .variant(ButtonVariant::Secondary)
                                    .trailing_hint("Esc"),
                            );
                        });
                    });
                });
            });
        });
    }
}

fn base_color_index(base: BaseColor) -> usize {
    BaseColor::ALL
        .iter()
        .position(|candidate| *candidate == base)
        .unwrap_or(0)
}

fn mode_label(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::System => "System",
        ThemeMode::Light => "Light",
        ThemeMode::Dark => "Dark",
    }
}

fn mode_icon(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::System => "monitor",
        ThemeMode::Light => "sun-medium",
        ThemeMode::Dark => "moon-star",
    }
}

fn theme_mode_index(mode: ThemeMode) -> usize {
    match mode {
        ThemeMode::Light => 0,
        ThemeMode::Dark => 1,
        ThemeMode::System => 2,
    }
}

fn theme_mode_from_index(index: usize) -> ThemeMode {
    match index {
        0 => ThemeMode::Light,
        1 => ThemeMode::Dark,
        _ => ThemeMode::System,
    }
}

const BASE_COLOR_OPTIONS: [&str; 8] = [
    "Neutral", "Stone", "Zinc", "Mauve", "Olive", "Mist", "Slate", "Taupe",
];
const THEME_MODE_OPTIONS: [TabOption<'static>; 3] = [
    TabOption::icon_only(0, "Light", "sun-medium"),
    TabOption::icon_only(1, "Dark", "moon-star"),
    TabOption::icon_only(2, "System", "monitor"),
];
