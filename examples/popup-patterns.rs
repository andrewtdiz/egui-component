use egui::{CentralPanel, Context, Id, ScrollArea};
use egui_component::layout;
use egui_component::prelude::*;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("egui-component - Popup Patterns")
            .with_inner_size([1120.0, 820.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui-component - Popup Patterns",
        options,
        Box::new(|creation_context| {
            egui_component::theme::install(
                &creation_context.egui_ctx,
                ThemeSpec::preset(BaseColor::Neutral),
                ThemeMode::Dark,
            );
            Ok(Box::<PopupPatternsApp>::default())
        }),
    )
}

const TOOLTIP_ENTRIES: [(&str, TooltipPlacement, &str); 4] = [
    (
        "Top",
        TooltipPlacement::Top,
        "Appears above the trigger with a short delay.",
    ),
    (
        "Right",
        TooltipPlacement::Right,
        "Useful for tight vertical stacks where the right side is open.",
    ),
    (
        "Bottom",
        TooltipPlacement::Bottom,
        "Keeps the hint close to the control when the header sits below.",
    ),
    (
        "Left",
        TooltipPlacement::Left,
        "Good for dense toolbars where the action cluster is right-aligned.",
    ),
];

const MENU_ENTRIES: [DropdownMenuEntry<'static>; 5] = [
    DropdownMenuEntry::action_with_icon_and_shortcut(0, "Duplicate", "copy", "Ctrl+D"),
    DropdownMenuEntry::action_with_icon(1, "Rename", "pencil"),
    DropdownMenuEntry::submenu_with_icon(
        "Share",
        "share-2",
        &[
            DropdownMenuEntry::action(2, "Copy Link"),
            DropdownMenuEntry::action(3, "Invite by Email"),
            DropdownMenuEntry::action(4, "Export PDF"),
        ],
    ),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action_with_icon(5, "Delete", "trash-2"),
];

const SUPPORT_MENU_ENTRIES: [DropdownMenuEntry<'static>; 4] = [
    DropdownMenuEntry::action_with_icon(10, "Open in New Window", "external-link"),
    DropdownMenuEntry::action_with_icon(11, "Move to Folder", "folder-open"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action_with_icon(12, "Archive", "archive"),
];

struct PopupPatternsApp {
    last_action: Option<usize>,
    archive_dialogue_open: bool,
}

impl Default for PopupPatternsApp {
    fn default() -> Self {
        Self {
            last_action: None,
            archive_dialogue_open: false,
        }
    }
}

impl eframe::App for PopupPatternsApp {
    fn update(&mut self, context: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(context, |ui| {
            ui.set_max_width(1080.0);
            ui.add_space(12.0);

            let _ = layout::column().gap(10.0).show(ui, |ui| {
                let mut components = ui.components();
                let _ = components.label(
                    Label::new("Popup Patterns")
                        .weight(LabelWeight::Bold)
                        .size(20.0),
                );
                let _ = components.label(
                    Label::new("Tooltips, menus, and modals built with the same popup primitives.")
                        .tone(LabelTone::Muted),
                );
            });

            ui.add_space(16.0);

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(980.0);
                    render_tooltip_section(ui);
                    ui.add_space(16.0);
                    render_menu_section(self, ui);
                    ui.add_space(16.0);
                    render_dialogue_section(self, ui);
                    ui.add_space(16.0);
                    render_status(self, ui);
                });
        });
    }
}

fn render_tooltip_section(ui: &mut egui::Ui) {
    let _ = ui.components().card(Card::new().padding(16, 16), |ui| {
        let _ = layout::column().gap(10.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new("Tooltip")
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Secondary),
            );
            let _ = components.label(
                Label::new(
                    "Use hover hints for labels that need a little context but not a full menu.",
                )
                .tone(LabelTone::Muted),
            );

            let _ = layout::row().gap(10.0).show(ui, |ui| {
                for (label, placement, text) in TOOLTIP_ENTRIES {
                    let tooltip = Tooltip::new(label, text)
                        .placement(placement)
                        .width(236.0)
                        .delay_ms(250);
                    let _ = ui.components().tooltip(tooltip);
                }
            });
        });
    });
}

fn render_menu_section(app: &mut PopupPatternsApp, ui: &mut egui::Ui) {
    let _ = ui.components().card(Card::new().padding(16, 16), |ui| {
        let _ = layout::column().gap(10.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new("Dropdown Menu")
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Secondary),
            );
            let _ = components.label(
                Label::new("Menus handle action sets, separators, nested submenus, and keyboard shortcuts.")
                    .tone(LabelTone::Muted),
            );

            let _ = layout::row().gap(10.0).show(ui, |ui| {
                let mut components = ui.components();
                let (_, state) = components.dropdown_menu(
                    DropdownMenu::new("Actions")
                        .entries(&MENU_ENTRIES)
                        .width(260.0)
                        .trigger_variant(ButtonVariant::Secondary),
                );
                if state.action.is_some() {
                    app.last_action = state.action;
                }

                let (_, support_state) = components.dropdown_menu(
                    DropdownMenu::new("More")
                        .entries(&SUPPORT_MENU_ENTRIES)
                        .width(220.0)
                        .trigger_variant(ButtonVariant::Ghost),
                );
                if support_state.action.is_some() {
                    app.last_action = support_state.action;
                }
            });
        });
    });
}

fn render_dialogue_section(app: &mut PopupPatternsApp, ui: &mut egui::Ui) {
    let _ = ui.components().card(Card::new().padding(16, 16), |ui| {
        let _ = layout::column().gap(10.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new("Dialogue")
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Secondary),
            );
            let _ = components.label(
                Label::new("The trigger button opens a modal with a destructive intent and explicit actions.")
                    .tone(LabelTone::Muted),
            );

            {
                let response = ui.components().dialogue(
                    &mut app.archive_dialogue_open,
                    Dialogue::new(Id::new("archive_dialogue"), "Archive project")
                        .description("This popup demonstrates the typed modal builder and the confirm/cancel footer.")
                        .trigger_label("Open archive dialogue")
                        .cancel_label("Keep it open")
                        .confirm_label("Archive")
                        .intent(DialogueIntent::Alert)
                        .width(380.0),
                );

                if response.clicked() {
                    app.last_action = Some(99);
                }
            }

            ui.add_space(4.0);
            let _ = layout::row().gap(10.0).show(ui, |ui| {
                let _ = ui.components().kbd_group((), |ui| {
                    let mut components = ui.components();
                    let _ = components.kbd("Esc");
                    let _ = components.kbd("Enter");
                });
                let mut components = ui.components();
                let _ = components.label(
                    Label::new("Keyboard users can close the modal with the Escape key or commit with Enter.")
                        .tone(LabelTone::Muted),
                );
            });
        });
    });
}

fn render_status(app: &PopupPatternsApp, ui: &mut egui::Ui) {
    let _ = ui.components().card(Card::new().padding(14, 14), |ui| {
        let _ = layout::row().gap(10.0).show(ui, |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new("Status")
                    .tone(LabelTone::Secondary)
                    .weight(LabelWeight::Semibold),
            );
            let status = app
                .last_action
                .map(|action| format!("last popup action {action}"))
                .unwrap_or_else(|| "waiting for an action".to_owned());
            let _ = components.label(Label::new(status.as_str()).tone(LabelTone::Muted));
        });
    });
}
