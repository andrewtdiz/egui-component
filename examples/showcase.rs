use egui::{vec2, Align2, CentralPanel, Color32, Context, Id, ScrollArea, SidePanel, Stroke};
use egui_component::layout;
use egui_component::prelude::*;

const SIDEBAR_WIDTH: f32 = 220.0;
const THEME_PANEL_WIDTH: f32 = 220.0;
const SHOWCASE_IMAGE_BYTES: &[u8] = include_bytes!("../assets/images/showcase-image.png");
const STATUS_OPTIONS: [&str; 4] = ["Draft", "Review", "Approved", "Archived"];
const BUTTON_GROUP_OPTIONS: [&str; 3] = ["Move", "Rotate", "Scale"];
const COMBOBOX_OPTIONS: [&str; 5] = [
    "Primary Action",
    "Secondary Action",
    "Accent Border",
    "Muted Surface",
    "Success State",
];
const TAB_OPTIONS: [TabOption<'static>; 3] = [
    TabOption::new(0, "Overview"),
    TabOption::new(1, "Tokens"),
    TabOption::new(2, "Publish"),
];
const STACKED_TAB_OPTIONS: [TabOption<'static>; 2] = [
    TabOption::with_icon(0, "Files", "folder-open"),
    TabOption::with_icon(1, "History", "history"),
];
const RAIL_TAB_OPTIONS: [TabOption<'static>; 3] = [
    TabOption::with_icon(0, "Home", "house"),
    TabOption::with_icon(1, "Assets", "image"),
    TabOption::with_icon(2, "Deploy", "rocket"),
];
const COMMAND_ITEMS: [CommandItem<'static>; 5] = [
    CommandItem::new("Build", "cargo check").shortcut("Ctrl+Shift+B"),
    CommandItem::new("Build", "cargo test").shortcut("Ctrl+T"),
    CommandItem::new("Release", "cargo publish --dry-run").shortcut("Ctrl+P"),
    CommandItem::new("Docs", "cargo doc --no-deps"),
    CommandItem::new("Lint", "cargo clippy --all-targets"),
];
const DROPDOWN_INVITE_ENTRIES: [DropdownMenuEntry<'static>; 3] = [
    DropdownMenuEntry::action(3, "Email"),
    DropdownMenuEntry::action(4, "Message"),
    DropdownMenuEntry::action(5, "Webhook"),
];
const DROPDOWN_ENTRIES: [DropdownMenuEntry<'static>; 7] = [
    DropdownMenuEntry::action_with_icon(0, "Open Docs", "book-open"),
    DropdownMenuEntry::action_with_shortcut(1, "Run Tests", "Ctrl+T"),
    DropdownMenuEntry::submenu_with_icon("Invite", "users", &DROPDOWN_INVITE_ENTRIES),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action_with_icon(2, "Deploy", "rocket"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action(6, "Archive"),
];
const DROPDOWN_ACTION_LABELS: [&str; 7] = [
    "Open Docs",
    "Run Tests",
    "Deploy",
    "Email",
    "Message",
    "Webhook",
    "Archive",
];
const MENU_FILE_ENTRIES: [DropdownMenuEntry<'static>; 4] = [
    DropdownMenuEntry::action_with_icon(0, "New File", "file-plus"),
    DropdownMenuEntry::action_with_icon(1, "Open...", "folder-open"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action_with_icon(2, "Export", "download"),
];
const MENU_EDIT_ENTRIES: [DropdownMenuEntry<'static>; 4] = [
    DropdownMenuEntry::action_with_icon(3, "Undo", "undo-2"),
    DropdownMenuEntry::action_with_icon(4, "Redo", "redo-2"),
    DropdownMenuEntry::separator(),
    DropdownMenuEntry::action_with_icon(5, "Duplicate", "copy"),
];
const MENU_BAR_ITEMS: [MenuBarItem<'static>; 2] = [
    MenuBarItem::new("File", &MENU_FILE_ENTRIES),
    MenuBarItem::new("Edit", &MENU_EDIT_ENTRIES),
];
const MENU_ACTION_LABELS: [&str; 6] =
    ["New File", "Open...", "Export", "Undo", "Redo", "Duplicate"];

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("egui-component Showcase")
            .with_inner_size([1280.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui-component Showcase",
        options,
        Box::new(|creation_context| {
            egui_component::theme::install(
                &creation_context.egui_ctx,
                ThemeSpec::preset(BaseColor::Neutral),
                ThemeMode::Dark,
            );
            Ok(Box::<ShowcaseApp>::default())
        }),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShowcaseComponent {
    AudioPlayback,
    Button,
    ButtonGroup,
    Card,
    Checkbox,
    Color,
    ColorInput,
    Collapsible,
    Combobox,
    Command,
    Dialogue,
    DropdownMenu,
    Field,
    Icon,
    Image,
    ImageTile,
    Input,
    Kbd,
    Label,
    MenuBar,
    NumberInput,
    Pagination,
    Progress,
    Select,
    Separator,
    Slider,
    Switch,
    Tabs,
    Toolbar,
    Tooltip,
}

impl ShowcaseComponent {
    const ALL: [Self; 30] = [
        Self::AudioPlayback,
        Self::Button,
        Self::ButtonGroup,
        Self::Card,
        Self::Checkbox,
        Self::Color,
        Self::ColorInput,
        Self::Collapsible,
        Self::Combobox,
        Self::Command,
        Self::Dialogue,
        Self::DropdownMenu,
        Self::Field,
        Self::Icon,
        Self::Image,
        Self::ImageTile,
        Self::Input,
        Self::Kbd,
        Self::Label,
        Self::MenuBar,
        Self::NumberInput,
        Self::Pagination,
        Self::Progress,
        Self::Select,
        Self::Separator,
        Self::Slider,
        Self::Switch,
        Self::Tabs,
        Self::Toolbar,
        Self::Tooltip,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::AudioPlayback => "Audio Playback",
            Self::Button => "Button",
            Self::ButtonGroup => "Button Group",
            Self::Card => "Card",
            Self::Checkbox => "Checkbox",
            Self::Color => "Color",
            Self::ColorInput => "Color Input",
            Self::Collapsible => "Collapsible",
            Self::Combobox => "Combobox",
            Self::Command => "Command",
            Self::Dialogue => "Dialogue",
            Self::DropdownMenu => "Dropdown Menu",
            Self::Field => "Field",
            Self::Icon => "Icon",
            Self::Image => "Image",
            Self::ImageTile => "Image Tile",
            Self::Input => "Input",
            Self::Kbd => "Kbd",
            Self::Label => "Label",
            Self::MenuBar => "Menu Bar",
            Self::NumberInput => "Number Input",
            Self::Pagination => "Pagination",
            Self::Progress => "Progress",
            Self::Select => "Select",
            Self::Separator => "Separator",
            Self::Slider => "Slider",
            Self::Switch => "Switch",
            Self::Tabs => "Tabs",
            Self::Toolbar => "Toolbar",
            Self::Tooltip => "Tooltip",
        }
    }

    fn summary(self) -> &'static str {
        match self {
            Self::AudioPlayback => "Playback row with a waveform and optional trailing actions.",
            Self::Button => "Primary, secondary, ghost, and link button variants.",
            Self::ButtonGroup => "Segmented row of tightly grouped button actions.",
            Self::Card => "Framed surface container for composing content.",
            Self::Checkbox => "Boolean input with optional label text.",
            Self::Color => "Circular color swatch with optional border.",
            Self::ColorInput => "Interactive RGBA color picker button.",
            Self::Collapsible => "Expandable section with icons and nested content.",
            Self::Combobox => "Filterable list backed by a text query.",
            Self::Command => "Command palette surface with grouped actions.",
            Self::Dialogue => "Modal confirmation flow with header and footer actions.",
            Self::DropdownMenu => "Button-triggered menu with actions and submenus.",
            Self::Field => "Label, text input, and helper text stacked together.",
            Self::Icon => "Named icon rendering from bundled icon assets.",
            Self::Image => "Raster image rendering with resizing and rounding.",
            Self::ImageTile => "Media tile with optional playback and body content.",
            Self::Input => "Single-line text input using the library theme.",
            Self::Kbd => "Keyboard keycaps and grouped shortcut display.",
            Self::Label => "Typed text styles with tone and weight.",
            Self::MenuBar => "Desktop-style menubar with nested dropdown menus.",
            Self::NumberInput => "Numeric text field with prefix and drag axes.",
            Self::Pagination => "Page navigator with truncation and next/previous controls.",
            Self::Progress => "Determinate progress bar.",
            Self::Select => "Popup select input with 1-of-N choices.",
            Self::Separator => "Simple horizontal rule for visual grouping.",
            Self::Slider => "Horizontal numeric range input.",
            Self::Switch => "Toggle control with optional label.",
            Self::Tabs => "Underline, segmented, stacked, and rail tab variants.",
            Self::Toolbar => "Floating action bar anchored inside a canvas.",
            Self::Tooltip => "Hover-triggered popup hint surface.",
        }
    }
}

struct ShowcaseApp {
    selected_component: ShowcaseComponent,
    theme_base: BaseColor,
    theme_mode: ThemeMode,
    text_value: String,
    field_value: String,
    checkbox_value: bool,
    switch_value: bool,
    select_value: Option<usize>,
    combobox_query: String,
    combobox_selected: usize,
    command_query: String,
    current_page: usize,
    progress: f32,
    slider_value: f32,
    number_x: f32,
    number_y: f32,
    tabs_value: usize,
    segmented_tabs_value: usize,
    stacked_tabs_value: usize,
    rail_tabs_value: usize,
    toggle_rail_tabs_value: Option<usize>,
    button_group_selection: Option<usize>,
    collapsible_open: bool,
    dialogue_open: bool,
    dropdown_action: Option<usize>,
    menu_action: Option<usize>,
    audio_state: AudioPlaybackState,
    image_tile_selected: bool,
    image_tile_playback: ImageTilePlaybackState,
    accent_color: Color32,
}

impl Default for ShowcaseApp {
    fn default() -> Self {
        Self {
            selected_component: ShowcaseComponent::Button,
            theme_base: BaseColor::Neutral,
            theme_mode: ThemeMode::Dark,
            text_value: "egui-component".to_owned(),
            field_value: "mainline".to_owned(),
            checkbox_value: true,
            switch_value: false,
            select_value: Some(1),
            combobox_query: String::new(),
            combobox_selected: 0,
            command_query: String::new(),
            current_page: 3,
            progress: 0.42,
            slider_value: 64.0,
            number_x: 12.0,
            number_y: 24.0,
            tabs_value: 0,
            segmented_tabs_value: 1,
            stacked_tabs_value: 0,
            rail_tabs_value: 0,
            toggle_rail_tabs_value: Some(1),
            button_group_selection: Some(0),
            collapsible_open: true,
            dialogue_open: false,
            dropdown_action: None,
            menu_action: None,
            audio_state: AudioPlaybackState::Paused,
            image_tile_selected: false,
            image_tile_playback: ImageTilePlaybackState::Paused,
            accent_color: Color32::from_rgb(59, 130, 246),
        }
    }
}

impl eframe::App for ShowcaseApp {
    fn update(&mut self, context: &Context, _frame: &mut eframe::Frame) {
        egui_component::theme::set_theme(context, ThemeSpec::preset(self.theme_base));
        egui_component::theme::set_mode(context, self.theme_mode);

        SidePanel::left("showcase_sidebar")
            .resizable(false)
            .exact_width(SIDEBAR_WIDTH)
            .show(context, |ui| self.render_sidebar(ui));

        SidePanel::right("theme_sidebar")
            .resizable(false)
            .exact_width(THEME_PANEL_WIDTH)
            .show(context, |ui| self.render_theme_panel(ui));

        CentralPanel::default().show(context, |ui| self.render_center(ui));
    }
}

impl ShowcaseApp {
    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        ui.heading("Components");
        ui.label("Select a component to preview it.");
        ui.add_space(10.0);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for component in ShowcaseComponent::ALL {
                    let selected = self.selected_component == component;
                    let button = Button::new(component.title())
                        .variant(if selected {
                            ButtonVariant::Secondary
                        } else {
                            ButtonVariant::Ghost
                        })
                        .selected(selected)
                        .min_size(vec2(ui.available_width(), 32.0));
                    if ui.components().button(button).clicked() {
                        self.selected_component = component;
                    }
                }
            });
    }

    fn render_theme_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        ui.heading("Theme");
        ui.label("Apply one theme to every component preview.");
        ui.add_space(10.0);

        let _ = ui.components().card(Card::new().padding(12, 12), |ui| {
            let mut components = ui.components();
            let _ = components.label(Label::new("Mode").weight(LabelWeight::Semibold));
            ui.add_space(8.0);
            let _ = layout::row().gap(8.0).show(ui, |ui| {
                let mut components = ui.components();
                if components
                    .button(
                        Button::new("Dark")
                            .variant(ButtonVariant::Secondary)
                            .selected(self.theme_mode == ThemeMode::Dark),
                    )
                    .clicked()
                {
                    self.theme_mode = ThemeMode::Dark;
                }
                if components
                    .button(
                        Button::new("Light")
                            .variant(ButtonVariant::Secondary)
                            .selected(self.theme_mode == ThemeMode::Light),
                    )
                    .clicked()
                {
                    self.theme_mode = ThemeMode::Light;
                }
            });

            ui.add_space(12.0);
            let mut components = ui.components();
            let _ = components.label(Label::new("Preset").weight(LabelWeight::Semibold));
            ui.add_space(8.0);
            let preset_button_width = ui.available_width();

            for base in BaseColor::ALL {
                let selected = self.theme_base == base;
                if ui
                    .components()
                    .button(
                        Button::new(base.label())
                            .variant(ButtonVariant::Secondary)
                            .selected(selected)
                            .min_size(vec2(preset_button_width, 30.0)),
                    )
                    .clicked()
                {
                    self.theme_base = base;
                }
                ui.add_space(6.0);
            }
        });
    }

    fn render_center(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_max_width(980.0);
                ui.add_space(16.0);
                ui.heading(self.selected_component.title());
                ui.label(self.selected_component.summary());
                ui.add_space(16.0);

                match self.selected_component {
                    ShowcaseComponent::AudioPlayback => self.render_audio_playback(ui),
                    ShowcaseComponent::Button => self.render_button(ui),
                    ShowcaseComponent::ButtonGroup => self.render_button_group(ui),
                    ShowcaseComponent::Card => self.render_card(ui),
                    ShowcaseComponent::Checkbox => self.render_checkbox(ui),
                    ShowcaseComponent::Color => self.render_color(ui),
                    ShowcaseComponent::ColorInput => self.render_color_input(ui),
                    ShowcaseComponent::Collapsible => self.render_collapsible(ui),
                    ShowcaseComponent::Combobox => self.render_combobox(ui),
                    ShowcaseComponent::Command => self.render_command(ui),
                    ShowcaseComponent::Dialogue => self.render_dialogue(ui),
                    ShowcaseComponent::DropdownMenu => self.render_dropdown_menu(ui),
                    ShowcaseComponent::Field => self.render_field(ui),
                    ShowcaseComponent::Icon => self.render_icon(ui),
                    ShowcaseComponent::Image => self.render_image(ui),
                    ShowcaseComponent::ImageTile => self.render_image_tile(ui),
                    ShowcaseComponent::Input => self.render_input(ui),
                    ShowcaseComponent::Kbd => self.render_kbd(ui),
                    ShowcaseComponent::Label => self.render_label(ui),
                    ShowcaseComponent::MenuBar => self.render_menu_bar(ui),
                    ShowcaseComponent::NumberInput => self.render_number_input(ui),
                    ShowcaseComponent::Pagination => self.render_pagination(ui),
                    ShowcaseComponent::Progress => self.render_progress(ui),
                    ShowcaseComponent::Select => self.render_select(ui),
                    ShowcaseComponent::Separator => self.render_separator(ui),
                    ShowcaseComponent::Slider => self.render_slider(ui),
                    ShowcaseComponent::Switch => self.render_switch(ui),
                    ShowcaseComponent::Tabs => self.render_tabs(ui),
                    ShowcaseComponent::Toolbar => self.render_toolbar(ui),
                    ShowcaseComponent::Tooltip => self.render_tooltip(ui),
                }
            });
    }

    fn render_audio_playback(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Audio Playback", |ui| {
            let (_, playback_result) = ui.components().audio_playback_with_actions(
                AudioPlayback::new(Id::new("audio_playback"), self.audio_state),
                |ui| {
                    let mut components = ui.components();
                    let _ = components.button(
                        Button::icon_only("skip-back")
                            .variant(ButtonVariant::Ghost)
                            .icon_size(12.0),
                    );
                    let _ = components.button(
                        Button::icon_only("skip-forward")
                            .variant(ButtonVariant::Ghost)
                            .icon_size(12.0),
                    );
                },
            );
            if playback_result.play_pause_clicked {
                self.audio_state = match self.audio_state {
                    AudioPlaybackState::Paused => AudioPlaybackState::Playing,
                    AudioPlaybackState::Playing => AudioPlaybackState::Paused,
                };
            }
        });
    }

    fn render_button(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Button Variants", |ui| {
            let _ = layout::row().gap(10.0).show(ui, |ui| {
                let mut components = ui.components();
                let _ = components.button(
                    Button::new("Primary")
                        .variant(ButtonVariant::Primary)
                        .leading_icon("check"),
                );
                let _ = components.button(
                    Button::new("Secondary")
                        .variant(ButtonVariant::Secondary)
                        .trailing_hint("K"),
                );
                let _ = components.button(
                    Button::new("Ghost")
                        .variant(ButtonVariant::Ghost)
                        .trailing_icon("chevron-right"),
                );
                let _ = components.button(
                    Button::new("Docs")
                        .variant(ButtonVariant::Link)
                        .trailing_icon("arrow-up-right"),
                );
                let _ = components.button(Button::color_only(Color::new(self.accent_color)));
            });
        });
    }

    fn render_button_group(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Button Group", |ui| {
            let clicked = ui.components().button_group(ButtonGroup::new(
                Id::new("button_group"),
                &BUTTON_GROUP_OPTIONS,
            ));
            if clicked.is_some() {
                self.button_group_selection = clicked;
            }
            ui.add_space(10.0);
            let status = button_group_text(self.button_group_selection, &BUTTON_GROUP_OPTIONS);
            let mut components = ui.components();
            let _ = components.label(Label::new(status.as_str()).tone(LabelTone::Muted));
        });
    }

    fn render_card(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Card", |ui| {
            let _ = ui.components().card(Card::new().padding(16, 16), |ui| {
                let mut components = ui.components();
                let _ = components.label(Label::new("Card Title").weight(LabelWeight::Bold));
                let _ = components.label(
                    Label::new("Cards are useful for grouping related UI.").tone(LabelTone::Muted),
                );
            });
        });
    }

    fn render_checkbox(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Checkbox", |ui| {
            let _ = ui.components().checkbox(
                &mut self.checkbox_value,
                Checkbox::new().label("Enable release checks"),
            );
        });
    }

    fn render_color(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Color", |ui| {
            let _ = layout::row().gap(10.0).show(ui, |ui| {
                let mut components = ui.components();
                let _ = components
                    .color(Color::new(self.accent_color).stroke(Stroke::new(1.0, Color32::WHITE)));
                let _ = components.color(Color::new(Color32::from_rgb(34, 197, 94)).size(24.0));
                let _ = components.color(Color::new(Color32::from_rgb(244, 114, 182)).size(28.0));
            });
        });
    }

    fn render_color_input(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Color Input", |ui| {
            let _ = ui.components().color_input(
                &mut self.accent_color,
                ColorInput::new().id(Id::new("accent_color")),
            );
        });
    }

    fn render_collapsible(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Collapsible", |ui| {
            let _ = ui.components().collapsible(
                &mut self.collapsible_open,
                Collapsible::new(Id::new("collapsible"), "Component Details")
                    .leading_icon("package")
                    .trailing_icon("sparkles"),
                |ui| {
                    let _ = ui.components().card(Card::new().padding(12, 12), |ui| {
                        let mut components = ui.components();
                        let _ = components.label(
                            Label::new("Release Configuration").weight(LabelWeight::Semibold),
                        );
                        let _ = components.label(
                            Label::new(
                                "This content should disappear when the collapsible closes.",
                            )
                            .tone(LabelTone::Muted),
                        );
                        ui.add_space(10.0);
                        let _ = layout::row().gap(10.0).show(ui, |ui| {
                            let mut components = ui.components();
                            let _ = components.button(
                                Button::new("Preview")
                                    .variant(ButtonVariant::Secondary)
                                    .leading_icon("eye"),
                            );
                            let _ = components.button(
                                Button::new("Publish")
                                    .variant(ButtonVariant::Primary)
                                    .leading_icon("rocket"),
                            );
                        });
                    });
                },
            );
        });
    }

    fn render_combobox(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Combobox", |ui| {
            let _ = ui.components().combobox(
                &mut self.combobox_query,
                &mut self.combobox_selected,
                Combobox::new(Id::new("combobox"), &COMBOBOX_OPTIONS).width(280.0),
            );
        });
    }

    fn render_command(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Command", |ui| {
            let _ = ui.components().command(
                &mut self.command_query,
                &COMMAND_ITEMS,
                Command::new(Id::new("command"))
                    .width(360.0)
                    .preview(true)
                    .preview_height(220.0),
            );
        });
    }

    fn render_dialogue(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Dialogue", |ui| {
            let _ = ui.components().dialogue(
                &mut self.dialogue_open,
                Dialogue::new(Id::new("dialogue"), "Publish Build")
                    .description("This demonstrates the dialogue component.")
                    .trigger_label("Open Dialogue")
                    .confirm_label("Publish")
                    .cancel_label("Not Now"),
            );
        });
    }

    fn render_dropdown_menu(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Dropdown Menu", |ui| {
            let (_, dropdown_state) = ui.components().dropdown_menu(
                DropdownMenu::new("Actions")
                    .entries(&DROPDOWN_ENTRIES)
                    .width(220.0),
            );
            if dropdown_state.action.is_some() {
                self.dropdown_action = dropdown_state.action;
            }
            ui.add_space(10.0);
            let status = dropdown_action_text(self.dropdown_action, &DROPDOWN_ACTION_LABELS);
            let _ = ui
                .components()
                .label(Label::new(status.as_str()).tone(LabelTone::Muted));
        });
    }

    fn render_field(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Field", |ui| {
            let _ = ui.components().field(
                &mut self.field_value,
                Field::new("Package Name")
                    .width(280.0)
                    .helper_text("Helper text under the input"),
            );
        });
    }

    fn render_icon(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Icon", |ui| {
            let _ = layout::row().gap(10.0).show(ui, |ui| {
                let mut components = ui.components();
                let _ = components.icon(Icon::new("search").size(18.0));
                let _ = components.icon(Icon::new("sparkles").size(18.0).tint(self.accent_color));
                let _ = components.icon(Icon::new("bootstrap:play-fill").size(18.0));
            });
        });
    }

    fn render_image(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Image", |ui| {
            let _ = ui.components().image(
                showcase_image()
                    .fit_to_exact_size(vec2(220.0, 146.0))
                    .corner_radius(10),
            );
        });
    }

    fn render_image_tile(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Image Tile", |ui| {
            let (_, tile_state) = ui.components().image_tile_with_body(
                ImageTile::new(showcase_image())
                    .size(ImageTileSize::Md)
                    .selected(self.image_tile_selected)
                    .playback_state(self.image_tile_playback),
                |ui| {
                    let mut components = ui.components();
                    let _ =
                        components.label(Label::new("Preview Asset").weight(LabelWeight::Semibold));
                    let _ = components
                        .label(Label::new("Image Tile with body content").tone(LabelTone::Muted));
                },
            );
            if tile_state.tile_clicked {
                self.image_tile_selected = !self.image_tile_selected;
            }
            if tile_state.play_pause_clicked {
                self.image_tile_playback = match self.image_tile_playback {
                    ImageTilePlaybackState::Paused => ImageTilePlaybackState::Playing,
                    ImageTilePlaybackState::Playing => ImageTilePlaybackState::Paused,
                };
            }
        });
    }

    fn render_input(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Input", |ui| {
            let _ = ui.components().text_input(
                &mut self.text_value,
                TextInput::new()
                    .width(280.0)
                    .placeholder("Component library"),
            );
        });
    }

    fn render_kbd(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Kbd", |ui| {
            let _ = ui.components().kbd_group((), |ui| {
                let mut components = ui.components();
                let _ = components.kbd("Ctrl");
                let _ = components.kbd("Shift");
                let _ = components.kbd("P");
            });
        });
    }

    fn render_label(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Label", |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new("Primary Label")
                    .weight(LabelWeight::Bold)
                    .size(14.0),
            );
            let _ = components.label(Label::new("Secondary Label").tone(LabelTone::Secondary));
            let _ = components.label(Label::new("Muted Label").tone(LabelTone::Muted));
            let _ = components.label(Label::new("Destructive Label").tone(LabelTone::Destructive));
        });
    }

    fn render_menu_bar(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Menu Bar", |ui| {
            let (_, menu_state) = ui
                .components()
                .menu_bar(MenuBar::new(Id::new("menu_bar"), &MENU_BAR_ITEMS));
            if menu_state.action.is_some() {
                self.menu_action = menu_state.action;
            }
            let status = menu_action_text(self.menu_action, &MENU_ACTION_LABELS);
            ui.add_space(10.0);
            let _ = ui
                .components()
                .label(Label::new(status.as_str()).tone(LabelTone::Muted));
        });
    }

    fn render_number_input(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Number Input", |ui| {
            let _ = layout::row().gap(10.0).show(ui, |ui| {
                let mut components = ui.components();
                let _ = components.number_input(
                    &mut self.number_x,
                    NumberInput::new(Id::new("number_x"))
                        .prefix("X")
                        .prefix_align_left()
                        .range(0.0..=100.0),
                );
                let _ = components.number_input(
                    &mut self.number_y,
                    NumberInput::new(Id::new("number_y"))
                        .prefix("Y")
                        .axis(NumberInputAxis::Vertical)
                        .range(0.0..=100.0),
                );
            });
        });
    }

    fn render_pagination(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Pagination", |ui| {
            let _ = ui.components().pagination(
                &mut self.current_page,
                Pagination::new(Id::new("pagination"), 12),
            );
        });
    }

    fn render_progress(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Progress", |ui| {
            let _ = ui
                .components()
                .progress(self.progress, Progress::new().width(280.0));
            ui.add_space(10.0);
            let _ = layout::row().gap(8.0).show(ui, |ui| {
                let mut components = ui.components();
                if components
                    .button(Button::new("Advance").variant(ButtonVariant::Primary))
                    .clicked()
                {
                    self.progress = (self.progress + 0.1).clamp(0.0, 1.0);
                }
                if components
                    .button(Button::new("Reset").variant(ButtonVariant::Secondary))
                    .clicked()
                {
                    self.progress = 0.0;
                }
            });
        });
    }

    fn render_select(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Select", |ui| {
            let _ = ui.components().select(
                &mut self.select_value,
                Select::from_id(Id::new("status"), &STATUS_OPTIONS).width(220.0),
            );
        });
    }

    fn render_separator(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Separator", |ui| {
            let mut components = ui.components();
            let _ = components.label("Above separator");
            let _ = components.separator();
            let _ = components.label("Below separator");
        });
    }

    fn render_slider(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Slider", |ui| {
            let _ = ui.components().slider(
                &mut self.slider_value,
                Slider::new(0.0..=100.0).width(220.0),
            );
        });
    }

    fn render_switch(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Switch", |ui| {
            let _ = ui.components().switch(
                &mut self.switch_value,
                Switch::new()
                    .label("Publish immediately")
                    .size(ControlSize::Sm),
            );
        });
    }

    fn render_tabs(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Tabs", |ui| {
            let _ = layout::column().gap(12.0).show(ui, |ui| {
                let mut components = ui.components();
                components.tabs(Id::new("tabs"), &mut self.tabs_value, &TAB_OPTIONS);
                components.segmented_tabs(
                    Id::new("segmented_tabs"),
                    &mut self.segmented_tabs_value,
                    &TAB_OPTIONS,
                );
                components.stacked_tabs(
                    Id::new("stacked_tabs"),
                    &mut self.stacked_tabs_value,
                    &STACKED_TAB_OPTIONS,
                );
                components.rail_tabs(
                    Id::new("rail_tabs"),
                    &mut self.rail_tabs_value,
                    &RAIL_TAB_OPTIONS,
                );
                components.toggle_rail_tabs(
                    Id::new("toggle_rail_tabs"),
                    &mut self.toggle_rail_tabs_value,
                    &RAIL_TAB_OPTIONS,
                );
            });
        });
    }

    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Toolbar", |ui| {
            let (host_rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 112.0), egui::Sense::hover());
            let _ = ui.scope_builder(egui::UiBuilder::new().max_rect(host_rect), |ui| {
                let _ = ui.components().toolbar(
                    Toolbar::new(Id::new("toolbar"))
                        .anchor(Align2::CENTER_CENTER)
                        .offset(vec2(0.0, 0.0)),
                    |ui| {
                        let mut components = ui.components();
                        let _ = components.button(
                            Button::new("Edit")
                                .variant(ButtonVariant::Ghost)
                                .leading_icon("pencil"),
                        );
                        let _ = components.button(
                            Button::new("Share")
                                .variant(ButtonVariant::Ghost)
                                .leading_icon("share-2"),
                        );
                        let _ = components.button(
                            Button::new("Export")
                                .variant(ButtonVariant::Secondary)
                                .trailing_icon("arrow-right"),
                        );
                    },
                );
            });
        });
    }

    fn render_tooltip(&mut self, ui: &mut egui::Ui) {
        surface_demo(ui, "Tooltip", |ui| {
            let _ = ui.components().tooltip(
                Tooltip::new("Hover For Tooltip", "Tooltip component example")
                    .placement(TooltipPlacement::Right)
                    .width(180.0),
            );
        });
    }
}

fn surface_demo(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    let _ = ui.components().card(Card::new().padding(16, 16), |ui| {
        let mut components = ui.components();
        let _ = components.label(Label::new(title).weight(LabelWeight::Bold).size(15.0));
        let _ = components.separator();
        ui.add_space(12.0);
        add(ui);
    });
}

fn showcase_image() -> Image<'static> {
    Image::from_bytes("bytes://examples/showcase-image.png", SHOWCASE_IMAGE_BYTES)
}

fn button_group_text(selection: Option<usize>, labels: &[&str]) -> String {
    selection
        .and_then(|index| labels.get(index).copied())
        .map(|label| format!("Button Group: {label}"))
        .unwrap_or_else(|| "Button Group: nothing selected".to_owned())
}

fn dropdown_action_text(selection: Option<usize>, labels: &[&str]) -> String {
    selection
        .and_then(|index| labels.get(index).copied())
        .map(|label| format!("Dropdown Action: {label}"))
        .unwrap_or_else(|| "Dropdown Action: none selected yet".to_owned())
}

fn menu_action_text(selection: Option<usize>, labels: &[&str]) -> String {
    selection
        .and_then(|index| labels.get(index).copied())
        .map(|label| format!("Menu Action: {label}"))
        .unwrap_or_else(|| "Menu Action: none selected yet".to_owned())
}
