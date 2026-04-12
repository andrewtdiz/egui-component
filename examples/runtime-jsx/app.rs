use std::{path::PathBuf, time::SystemTime};

use clay_jsx_egui_bridge::{JsxRuntimeSession, MotionFrame};
use egui::{CentralPanel, Context, ScrollArea, TopBottomPanel};
use egui_component::{
    contract::{render_tree, ContractEvent, ContractTree},
    prelude::*,
    theme::{self, BaseColor, ThemeMode, ThemeSpec},
};

use super::default_entry_path;

#[derive(Debug)]
pub struct RuntimeJsxApp {
    entry_path: PathBuf,
    last_seen_modified: Option<SystemTime>,
    initialized: bool,
    session: Option<JsxRuntimeSession>,
    rendered: Option<ContractTree>,
    motion: MotionFrame,
    error: Option<String>,
}

impl Default for RuntimeJsxApp {
    fn default() -> Self {
        Self::new(default_entry_path())
    }
}

impl RuntimeJsxApp {
    pub fn new(entry_path: impl Into<PathBuf>) -> Self {
        Self {
            entry_path: entry_path.into(),
            last_seen_modified: None,
            initialized: false,
            session: None,
            rendered: None,
            motion: MotionFrame::default(),
            error: None,
        }
    }

    fn update_frame(&mut self, ctx: &Context) {
        self.reload_initial_or_external_changes();
        self.tick_runtime_motion(ctx);

        TopBottomPanel::top("jsx_runtime_topbar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let entry_path = self.entry_path.display().to_string();
                    let _ = ui.components().label(
                        Label::new("JSX egui runtime")
                            .weight(LabelWeight::Semibold)
                            .tone(LabelTone::Primary),
                    );
                    let _ = ui.components().label(
                        Label::new(entry_path.as_str())
                            .tone(LabelTone::Muted)
                            .size(11.0),
                    );
                });
                let _ = ui.components().separator();
            });

        CentralPanel::default().show(ctx, |ui| self.render_preview(ui));
    }

    fn render_preview(&mut self, ui: &mut egui::Ui) {
        let mut frame_events = Vec::new();

        ui.add_space(16.0);
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if let Some(error) = &self.error {
                    let _ = ui.components().label(
                        Label::new("Runtime error")
                            .tone(LabelTone::Destructive)
                            .weight(LabelWeight::Semibold),
                    );
                    let _ = ui
                        .components()
                        .label(Label::new(error.as_str()).tone(LabelTone::Muted).size(11.0));
                    return;
                }

                match self.rendered.as_ref() {
                    Some(tree) => {
                        let events = render_tree(ui, tree);
                        if !events.is_empty() {
                            frame_events = events;
                        }
                    }
                    None => {
                        let _ = ui.components().label(
                            Label::new("No JSX tree has been rendered yet.").tone(LabelTone::Muted),
                        );
                    }
                }
            });

        if !frame_events.is_empty() {
            self.dispatch_events_to_runtime(&frame_events);
        }
    }

    fn reload_initial_or_external_changes(&mut self) {
        if !self.initialized {
            self.initialized = true;
            self.reload_from_disk();
            return;
        }

        let modified = modified_time(&self.entry_path);
        if modified != self.last_seen_modified {
            self.reload_from_disk();
        }
    }

    fn reload_from_disk(&mut self) {
        match JsxRuntimeSession::load(&self.entry_path) {
            Ok((session, rendered)) => {
                self.session = Some(session);
                self.rendered = rendered.tree;
                self.motion = rendered.motion;
                self.error = None;
                self.last_seen_modified = modified_time(&self.entry_path);
            }
            Err(error) => {
                self.session = None;
                self.motion = MotionFrame::default();
                self.rendered = None;
                self.error = Some(format!(
                    "Failed to load {}: {error}",
                    self.entry_path.display()
                ));
                self.last_seen_modified = modified_time(&self.entry_path);
            }
        }
    }

    fn dispatch_events_to_runtime(&mut self, events: &[ContractEvent]) {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        match session.dispatch_events(events) {
            Ok(rendered) => {
                if let Some(tree) = rendered.tree {
                    self.rendered = Some(tree);
                }
                self.motion = rendered.motion;
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }

    fn tick_runtime_motion(&mut self, ctx: &Context) {
        if !self.motion.active {
            return;
        }
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let now_secs = ctx.input(|input| input.time);
        match session.tick_motion(now_secs) {
            Ok(rendered) => {
                self.motion = rendered.motion;
                if self.motion.active {
                    ctx.request_repaint();
                }
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }
}

impl eframe::App for RuntimeJsxApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        theme::set_theme(ctx, ThemeSpec::preset(BaseColor::Neutral));
        theme::set_mode(ctx, ThemeMode::System);
        self.update_frame(ctx);
    }
}

fn modified_time(path: &std::path::Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}
