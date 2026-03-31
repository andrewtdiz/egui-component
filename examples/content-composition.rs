use egui::{Align, CentralPanel, Id, Layout};
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

const TAB_OPTIONS: [TabOption<'static>; 3] = [
    TabOption::new(0, "Stack"),
    TabOption::new(1, "Overrides"),
    TabOption::new(2, "Notes"),
];

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Content Composition")
            .with_inner_size([1220.0, 860.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Content Composition",
        options,
        Box::new(|creation_context| {
            egui_component::theme::install(
                &creation_context.egui_ctx,
                ThemeSpec::preset(BaseColor::Stone),
                ThemeMode::Light,
            );
            Ok(Box::<ContentCompositionApp>::default())
        }),
    )
}

#[derive(Debug, Default)]
struct ContentCompositionApp {
    newsletter_email: String,
    search_query: String,
    active_view: usize,
    accent_label: String,
}

impl eframe::App for ContentCompositionApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        CentralPanel::default().show_inside(ui, |ui| {
            let _ = column(ui, 16.0, |ui| {
                hero_block(ui, self.active_view);

                {
                    let mut components = ui.components();
                    components.tabs(Id::new("composition-tabs"), &mut self.active_view, &TAB_OPTIONS);
                }

                let _ = row(ui, 16.0, |ui| {
                    let _ = column(ui, 16.0, |ui| {
                        section_card(ui, "Reusable layout", |ui| {
                            composition_toolbar(ui);
                            ui.add_space(12.0);
                            feature_grid(ui);
                        });

                        section_card(ui, "Scoped overrides", |ui| {
                            let mut components = ui.components();
                            let _ = components.with_override(
                                (
                                    CardOverride::new().padding(18, 18),
                                    ButtonOverride::new().variant(ButtonVariant::Secondary),
                                    LabelOverride::new().tone(LabelTone::Muted),
                                ),
                                |ui| {
                                    let _ = ui.components().card(Card::new(), |ui| {
                                        let _ = column(ui, 10.0, |ui| {
                                            {
                                                let mut components = ui.components();
                                                let _ = components.label(
                                                    Label::new("Overrides are local to the section")
                                                        .weight(LabelWeight::Semibold),
                                                );
                                                let _ = components.label(
                                                    Label::new(
                                                        "This block uses a secondary button tone and muted copy.",
                                                    )
                                                    .size(11.0),
                                                );
                                            }
                                            let _ = row(ui, 10.0, |ui| {
                                                let mut components = ui.components();
                                                let _ = components.button(
                                                    Button::new("Preview").leading_icon("eye"),
                                                );
                                                let _ = components.button(
                                                    Button::new("Publish").leading_icon("rocket"),
                                                );
                                            });
                                        });
                                    });
                                },
                            );
                        });
                    });

                    let _ = column(ui, 16.0, |ui| {
                        section_card(ui, "Content sample", |ui| {
                            content_stack(
                                ui,
                                &self.accent_label,
                                &mut self.newsletter_email,
                                &mut self.search_query,
                            );
                        });

                        section_card(ui, "Action rail", |ui| {
                            rail_actions(ui);
                        });
                    });
                });
            });
        });
    }
}

fn hero_block(ui: &mut egui::Ui, active_view: usize) {
    let _ = ui.components().card(Card::new().padding(18, 18), |ui| {
        let _ = column(ui, 10.0, |ui| {
            {
                let mut components = ui.components();
                let _ = components.label(
                    Label::new("Content Composition")
                        .weight(LabelWeight::Bold)
                        .size(18.0),
                );
                let _ = components.label(
                    Label::new(
                        "A single file can still be modular when layout helpers, cards, labels, buttons, and shortcuts are composed carefully.",
                    )
                    .tone(LabelTone::Muted),
                );
                let _ = components.label(
                    Label::new(active_view_label(active_view))
                        .tone(LabelTone::Secondary)
                        .size(11.0),
                );
            }

            let _ = row(ui, 10.0, |ui| {
                let mut components = ui.components();
                let _ = components.button(
                    Button::new("Create section")
                        .variant(ButtonVariant::Primary)
                        .leading_icon("plus"),
                );
                let _ = components.button(
                    Button::new("Export notes")
                        .variant(ButtonVariant::Secondary)
                        .leading_icon("download"),
                );
                let _ = components.kbd_group((), |ui| {
                    let mut components = ui.components();
                    let _ = components.kbd("Ctrl");
                    let _ = components.kbd("Shift");
                    let _ = components.kbd("K");
                });
            });
        });
    });
}

fn composition_toolbar(ui: &mut egui::Ui) {
    let _ = row(ui, 10.0, |ui| {
        let mut components = ui.components();
        let _ = components.label(Label::new("Primary").weight(LabelWeight::Semibold));
        let _ = components.label(Label::new("Secondary").tone(LabelTone::Secondary));
        let _ = components.label(Label::new("Muted").tone(LabelTone::Muted));
    });
}

fn feature_grid(ui: &mut egui::Ui) {
    let _ = column(ui, 12.0, |ui| {
        feature_row(
            ui,
            "Cards create structure",
            "Use cards as framing surfaces for repeated content blocks.",
            "Built with layout and reusable card helpers.",
            "Shift",
        );
        feature_row(
            ui,
            "Buttons stay local",
            "Scoped overrides let one section differ without leaking styling.",
            "The override block below stays self-contained.",
            "Enter",
        );
        feature_row(
            ui,
            "Keyboard hints fit inline",
            "Kbd components work well in labels, helper copy, and action rows.",
            "They keep shortcut affordances visible.",
            "Cmd",
        );
    });
}

fn feature_row(ui: &mut egui::Ui, title: &str, body: &str, note: &str, key: &str) {
    let _ = ui.components().card(Card::new().padding(14, 14), |ui| {
        let _ = row(ui, 12.0, |ui| {
            let _ = column(ui, 4.0, |ui| {
                let mut components = ui.components();
                let _ =
                    components.label(Label::new(title).weight(LabelWeight::Semibold).size(13.0));
                let _ = components.label(Label::new(body).tone(LabelTone::Muted));
                let _ = components.label(Label::new(note).tone(LabelTone::Secondary).size(11.0));
            });
            let _ = ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let mut components = ui.components();
                let _ = components.kbd(key);
            });
        });
    });
}

fn content_stack(ui: &mut egui::Ui, accent_label: &str, email: &mut String, query: &mut String) {
    let _ = column(ui, 12.0, |ui| {
        let _ = ui.components().card(Card::new().padding(14, 14), |ui| {
            let mut components = ui.components();
            let _ = components.label(Label::new("Article preview").weight(LabelWeight::Semibold));
            let _ = components.label(
                Label::new(
                    "Blocks can be stacked to produce a reusable article, status summary, or settings panel.",
                )
                .tone(LabelTone::Muted),
            );
            let _ = components.label(
                Label::new("Highlights").weight(LabelWeight::Semibold).size(12.0),
            );
            let _ = components.label(Label::new(accent_label).tone(LabelTone::Secondary));
            let _ = components.field(
                email,
                Field::new("Newsletter email").placeholder("name@example.com"),
            );
        });

        let _ = ui.components().card(Card::new().padding(14, 14), |ui| {
            let mut components = ui.components();
            let _ = components.label(Label::new("Search snippet").weight(LabelWeight::Semibold));
            let _ = components.label(Label::new(query.as_str()).tone(LabelTone::Muted));
            let _ = components.field(
                query,
                Field::new("Search").placeholder("Find reusable sections"),
            );
        });
    });
}

fn rail_actions(ui: &mut egui::Ui) {
    let _ = ui.components().card(Card::new().padding(14, 14), |ui| {
        let _ = column(ui, 10.0, |ui| {
            {
                let mut components = ui.components();
                let _ =
                    components.label(Label::new("Keyboard driven").weight(LabelWeight::Semibold));
                let _ = components.label(
                    Label::new(
                        "Compose action rows with shortcut reminders and restrained button groups.",
                    )
                    .tone(LabelTone::Muted),
                );
            }

            let _ = column(ui, 8.0, |ui| {
                action_line(ui, "Save draft", "Ctrl", "S", "Save");
                action_line(ui, "Open palette", "Ctrl", "K", "Open");
                action_line(ui, "Search docs", "Ctrl", "Shift", "F");
            });
        });
    });
}

fn action_line(ui: &mut egui::Ui, label: &str, first: &str, second: &str, trigger: &str) {
    let _ = row(ui, 8.0, |ui| {
        let mut components = ui.components();
        let _ = components.label(Label::new(label).weight(LabelWeight::Semibold));
        let _ = components.kbd(first);
        let _ = components.kbd(second);
        let _ = components.button(Button::new(trigger).variant(ButtonVariant::Secondary));
    });
}

fn section_card(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    let _ = ui.components().card(Card::new().padding(16, 16), |ui| {
        let _ = column(ui, 10.0, |ui| {
            {
                let mut components = ui.components();
                let _ = components.label(Label::new(title).weight(LabelWeight::Bold).size(14.0));
                let _ = components.separator();
            }
            add(ui);
        });
    });
}

fn active_view_label(active_view: usize) -> &'static str {
    match active_view {
        0 => "Stack: compose reusable blocks from smaller sections.",
        1 => "Overrides: scope styling changes to one region.",
        2 => "Notes: keep keyboard hints and labels visible.",
        _ => "Stack: compose reusable blocks from smaller sections.",
    }
}
