use crate::components::{Button, ButtonStyle, ComponentUiExt, Select};
use crate::theme;
use crate::ui::{icons, tokens};
use crate::{ComponentLibraryError, Result};
use egui::{
    Align, Align2, CentralPanel, Context, CornerRadius, FontFamily, FontId, Frame, Layout, Margin,
    RichText, ScrollArea, Sense, SidePanel, Stroke, StrokeKind, TopBottomPanel, Ui, Vec2,
};
use std::time::{Duration, Instant};

const SIDEBAR_WIDTH: f32 = 220.0;
const CONTENT_MAX_WIDTH: f32 = 680.0;
const COMPOSER_MAX_WIDTH: f32 = 680.0;
const THREAD_ROW_HEIGHT: f32 = 52.0;
const COMPOSER_HEIGHT: f32 = 72.0;
const ROUND_ICON_BUTTON_SIZE: f32 = 30.0;
const STICK_TO_END_THRESHOLD: f32 = 18.0;
const BODY_STREAM_INTERVAL: Duration = Duration::from_millis(18);
const COMMAND_STREAM_INTERVAL: Duration = Duration::from_millis(140);
const BODY_STREAM_CHUNK: usize = 4;
const NEW_THREAD_TITLE: &str = "New thread";
const MODEL_OPTIONS: &[&str] = &["GPT-5 Codex", "GPT-5", "o4-mini"];
const REASONING_OPTIONS: &[&str] = &["Balanced", "Fast", "Deep"];
const MOCK_COMMANDS: &[&str] = &["cargo run chat", "cargo fmt", "cargo check --examples"];

pub(crate) fn run_chat_window() -> Result {
    let window_title = "Chat Example";
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(window_title)
            .with_inner_size([1360.0, 860.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        window_title,
        native_options,
        Box::new(move |creation_context| {
            theme::setup(&creation_context.egui_ctx);
            theme::set_dark_mode(&creation_context.egui_ctx, true);
            Ok(Box::new(ChatWindowApp {
                state: ChatExampleState::default(),
            }))
        }),
    )
    .map_err(|error| ComponentLibraryError::Runtime(error.to_string()))
}

#[derive(Debug)]
struct ChatWindowApp {
    state: ChatExampleState,
}

impl eframe::App for ChatWindowApp {
    fn update(&mut self, context: &Context, _frame: &mut eframe::Frame) {
        self.state.update_streaming(context);
        self.state.draw(context);
    }
}

#[derive(Debug)]
struct ChatExampleState {
    threads: Vec<ChatThread>,
    selected_thread_index: usize,
    next_thread_id: u64,
    composer_text: String,
    selected_model_index: Option<usize>,
    selected_reasoning_index: Option<usize>,
    conversation_scroll: ConversationScrollState,
    streaming: Option<StreamingResponse>,
}

impl Default for ChatExampleState {
    fn default() -> Self {
        let threads = vec![
            ChatThread::new(1, NEW_THREAD_TITLE, "Start a fresh conversation", "now"),
            ChatThread::new(
                2,
                "Refactor toolbar layout",
                "Need a tighter layout pass and cleaner spacing.",
                "12m",
            ),
            ChatThread::new(
                3,
                "Design system sync",
                "Scroll behavior and density still need a final pass.",
                "38m",
            )
            .with_messages(seed_design_system_sync_messages()),
            ChatThread::new(
                4,
                "Dropdown review",
                "Verify keyboard rows and submenu rhythm in light mode.",
                "1h",
            ),
            ChatThread::new(
                5,
                "Release prep",
                "Last pass before handing the component library off.",
                "3h",
            ),
        ];

        Self {
            threads,
            selected_thread_index: 2,
            next_thread_id: 6,
            composer_text: String::new(),
            selected_model_index: Some(0),
            selected_reasoning_index: Some(0),
            conversation_scroll: ConversationScrollState::default(),
            streaming: None,
        }
    }
}

impl ChatExampleState {
    fn draw(&mut self, context: &Context) {
        SidePanel::left("chat_sidebar")
            .resizable(false)
            .exact_width(SIDEBAR_WIDTH)
            .frame(
                Frame::new()
                    .fill(tokens::muted_surface(true))
                    .stroke(Stroke::NONE)
                    .inner_margin(Margin::symmetric(8, 10)),
            )
            .show(context, |ui| {
                theme::apply_component_theme(ui);
                self.draw_sidebar(ui);
            });

        TopBottomPanel::bottom("chat_composer")
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(tokens::app_background(true))
                    .stroke(Stroke::NONE)
                    .inner_margin(Margin::symmetric(14, 10)),
            )
            .show(context, |ui| {
                theme::apply_component_theme(ui);
                self.draw_composer(ui);
            });

        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(tokens::app_background(true))
                    .stroke(Stroke::NONE)
                    .inner_margin(Margin::symmetric(18, 10)),
            )
            .show(context, |ui| {
                theme::apply_component_theme(ui);
                self.draw_conversation(context, ui);
            });
    }

    fn draw_sidebar(&mut self, ui: &mut Ui) {
        ui.spacing_mut().item_spacing.y = 6.0;

        ui.horizontal(|ui| {
            paint_sidebar_icon(ui, "panel-left");
            ui.label(
                RichText::new("Threads")
                    .size(15.0)
                    .color(tokens::text_primary(true)),
            );
        });
        ui.add_space(6.0);
        let new_thread_width = ui.available_width();

        if ui
            .components()
            .button(
                Button::new("New Thread")
                    .icon("square-pen")
                    .style(ButtonStyle::Ghost)
                    .min_size(egui::vec2(new_thread_width, 32.0)),
            )
            .clicked()
        {
            self.create_new_thread();
        }

        ui.add_space(6.0);
        ui.label(
            RichText::new("Recent chats")
                .size(10.0)
                .color(tokens::text_muted(true))
                .family(FontFamily::Proportional),
        );
        ui.add_space(2.0);

        let available_height = ui.available_height().max(0.0);
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(available_height)
            .show(ui, |ui| {
                let mut clicked_index = None;
                for (index, thread) in self.threads.iter().enumerate() {
                    if draw_thread_row(
                        ui,
                        thread,
                        self.selected_thread_index == index,
                        index == 0 && thread.messages.is_empty(),
                    )
                    .clicked()
                    {
                        clicked_index = Some(index);
                    }
                    ui.add_space(2.0);
                }

                if let Some(index) = clicked_index {
                    self.selected_thread_index = index;
                    self.conversation_scroll
                        .reset_for_thread(self.current_thread().id);
                }
            });
    }

    fn draw_conversation(&mut self, context: &Context, ui: &mut Ui) {
        let thread_id = self.current_thread().id;
        self.conversation_scroll.ensure_thread(thread_id);
        let stick_to_end = self.conversation_scroll.stick_to_end;
        let jump_to_end_requested =
            std::mem::take(&mut self.conversation_scroll.jump_to_end_requested);
        let thread = self.current_thread();
        let content_width = ui.available_width().min(CONTENT_MAX_WIDTH);

        let scroll_output = ScrollArea::vertical()
            .id_salt(("chat_conversation_scroll", thread_id))
            .auto_shrink([false, false])
            .stick_to_bottom(stick_to_end)
            .show(ui, |ui| {
                center_column(ui, content_width, |ui| {
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(thread.title.as_str())
                            .size(22.0)
                            .color(tokens::text_primary(true)),
                    );
                    ui.add_space(10.0);

                    if thread.messages.is_empty() {
                        draw_empty_state(ui);
                    } else {
                        for message in &thread.messages {
                            draw_message(ui, message);
                            ui.add_space(14.0);
                        }
                    }

                    let bottom_anchor = ui
                        .allocate_exact_size(egui::vec2(content_width, 1.0), Sense::hover())
                        .0;
                    if jump_to_end_requested {
                        ui.scroll_to_rect(bottom_anchor, Some(Align::BOTTOM));
                    }
                });
            });

        let distance_to_end = conversation_distance_to_end(
            scroll_output.content_size.y,
            scroll_output.inner_rect.height(),
            scroll_output.state.offset.y,
        );
        self.conversation_scroll
            .update_from_distance(distance_to_end);

        if !self.conversation_scroll.stick_to_end
            && draw_jump_to_end_button(context, scroll_output.inner_rect)
        {
            self.conversation_scroll.request_jump_to_end();
        }
    }

    fn draw_composer(&mut self, ui: &mut Ui) {
        let can_send = !self.composer_text.trim().is_empty() && self.streaming.is_none();
        let composer_width = ui.available_width().min(COMPOSER_MAX_WIDTH);
        let mut send_clicked = false;

        center_column(ui, composer_width, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;

            let text_edit = egui::TextEdit::multiline(&mut self.composer_text)
                .hint_text(
                    RichText::new("Ask for follow-up changes").color(tokens::text_muted(true)),
                )
                .frame(false)
                .font(FontId::new(13.0, FontFamily::Proportional))
                .desired_width(f32::INFINITY)
                .desired_rows(3);
            let _ = ui.add_sized([ui.available_width(), COMPOSER_HEIGHT], text_edit);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                let _ = ui.components().select(
                    &mut self.selected_model_index,
                    Select::from_id(egui::Id::new("chat_model_select"), MODEL_OPTIONS).width(142.0),
                );
                let _ = ui.components().select(
                    &mut self.selected_reasoning_index,
                    Select::from_id(egui::Id::new("chat_reasoning_select"), REASONING_OPTIONS)
                        .width(128.0),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if draw_round_icon_button(
                        ui,
                        "arrow-up",
                        ButtonStyle::Primary,
                        can_send,
                        ROUND_ICON_BUTTON_SIZE,
                    )
                    .clicked()
                    {
                        send_clicked = true;
                    }

                    let _ = draw_round_icon_button(
                        ui,
                        "plus",
                        ButtonStyle::Ghost,
                        true,
                        ROUND_ICON_BUTTON_SIZE,
                    )
                    .on_hover_text("Mock attachment action");
                });
            });
        });

        if send_clicked && can_send {
            self.send_message();
        }
    }

    fn create_new_thread(&mut self) {
        let thread_id = self.next_thread_id;
        self.next_thread_id += 1;
        self.threads.insert(
            0,
            ChatThread::new(
                thread_id,
                NEW_THREAD_TITLE,
                "Start a fresh conversation",
                "now",
            ),
        );
        self.selected_thread_index = 0;
        self.composer_text.clear();
        self.conversation_scroll
            .reset_for_thread(self.current_thread().id);
    }

    fn send_message(&mut self) {
        let prompt = self.composer_text.trim().to_owned();
        if prompt.is_empty() || self.streaming.is_some() {
            return;
        }

        let selected_model = selected_option(MODEL_OPTIONS, self.selected_model_index).to_owned();
        let selected_reasoning =
            selected_option(REASONING_OPTIONS, self.selected_reasoning_index).to_owned();

        let thread = self.current_thread_mut();
        if thread.messages.is_empty() || thread.title == NEW_THREAD_TITLE {
            thread.title = thread_title_from_prompt(prompt.as_str());
        }
        thread.preview = truncate_line(prompt.as_str(), 54);
        thread.updated_at = "now".to_owned();
        thread.messages.push(ChatMessage::user(prompt.clone()));

        let message_index = thread.messages.len();
        thread.messages.push(ChatMessage::assistant_streaming());

        let body = mock_response_body(selected_model.as_str(), selected_reasoning.as_str());
        let thread_id = thread.id;
        self.streaming = Some(StreamingResponse::new(thread_id, message_index, body));
        self.composer_text.clear();
        self.conversation_scroll.request_jump_to_end();
    }

    fn update_streaming(&mut self, context: &Context) {
        let Some(mut streaming) = self.streaming.take() else {
            return;
        };

        let now = Instant::now();
        let mut changed = false;
        while now >= streaming.next_tick {
            changed = true;
            if streaming.body_cursor < streaming.body.len() {
                streaming.body_cursor =
                    (streaming.body_cursor + BODY_STREAM_CHUNK).min(streaming.body.len());
                streaming.next_tick += BODY_STREAM_INTERVAL;
                continue;
            }

            if streaming.commands_revealed < MOCK_COMMANDS.len() {
                streaming.commands_revealed += 1;
                streaming.next_tick += COMMAND_STREAM_INTERVAL;
                continue;
            }

            break;
        }

        if changed {
            if let Some(message) =
                self.streaming_message_mut(streaming.thread_id, streaming.message_index)
            {
                message.body = streaming.visible_body();
                message.commands = streaming.visible_commands();
                message.streaming = streaming.is_active();
            }
        }

        if streaming.is_complete() {
            if let Some(message) =
                self.streaming_message_mut(streaming.thread_id, streaming.message_index)
            {
                message.body = streaming.visible_body();
                message.commands = streaming.visible_commands();
                message.streaming = false;
            }
        } else {
            context.request_repaint_after(Duration::from_millis(16));
            self.streaming = Some(streaming);
        }
    }

    fn current_thread(&self) -> &ChatThread {
        &self.threads[self
            .selected_thread_index
            .min(self.threads.len().saturating_sub(1))]
    }

    fn current_thread_mut(&mut self) -> &mut ChatThread {
        let index = self
            .selected_thread_index
            .min(self.threads.len().saturating_sub(1));
        &mut self.threads[index]
    }

    fn streaming_message_mut(
        &mut self,
        thread_id: u64,
        message_index: usize,
    ) -> Option<&mut ChatMessage> {
        self.threads
            .iter_mut()
            .find(|thread| thread.id == thread_id)
            .and_then(|thread| thread.messages.get_mut(message_index))
    }
}

#[derive(Debug)]
struct ConversationScrollState {
    active_thread_id: Option<u64>,
    stick_to_end: bool,
    jump_to_end_requested: bool,
}

impl Default for ConversationScrollState {
    fn default() -> Self {
        Self {
            active_thread_id: None,
            stick_to_end: true,
            jump_to_end_requested: true,
        }
    }
}

impl ConversationScrollState {
    fn ensure_thread(&mut self, thread_id: u64) {
        if self.active_thread_id != Some(thread_id) {
            self.reset_for_thread(thread_id);
        }
    }

    fn reset_for_thread(&mut self, thread_id: u64) {
        self.active_thread_id = Some(thread_id);
        self.stick_to_end = true;
        self.jump_to_end_requested = true;
    }

    fn request_jump_to_end(&mut self) {
        self.stick_to_end = true;
        self.jump_to_end_requested = true;
    }

    fn update_from_distance(&mut self, distance_to_end: f32) {
        self.stick_to_end = distance_to_end <= STICK_TO_END_THRESHOLD;
        if self.stick_to_end {
            self.jump_to_end_requested = false;
        }
    }
}

#[derive(Debug)]
struct ChatThread {
    id: u64,
    title: String,
    preview: String,
    updated_at: String,
    messages: Vec<ChatMessage>,
}

impl ChatThread {
    fn new(id: u64, title: &str, preview: &str, updated_at: &str) -> Self {
        Self {
            id,
            title: title.to_owned(),
            preview: preview.to_owned(),
            updated_at: updated_at.to_owned(),
            messages: Vec::new(),
        }
    }

    fn with_messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.messages = messages;
        self
    }
}

#[derive(Debug)]
struct ChatMessage {
    role: MessageRole,
    body: String,
    commands: Vec<String>,
    streaming: bool,
}

impl ChatMessage {
    fn user(body: String) -> Self {
        Self {
            role: MessageRole::User,
            body,
            commands: Vec::new(),
            streaming: false,
        }
    }

    fn assistant_streaming() -> Self {
        Self {
            role: MessageRole::Assistant,
            body: String::new(),
            commands: Vec::new(),
            streaming: true,
        }
    }

    fn assistant_complete(body: String, commands: Vec<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            body,
            commands,
            streaming: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug)]
struct StreamingResponse {
    thread_id: u64,
    message_index: usize,
    body: Vec<char>,
    body_cursor: usize,
    commands_revealed: usize,
    next_tick: Instant,
}

impl StreamingResponse {
    fn new(thread_id: u64, message_index: usize, body: String) -> Self {
        Self {
            thread_id,
            message_index,
            body: body.chars().collect(),
            body_cursor: 0,
            commands_revealed: 0,
            next_tick: Instant::now(),
        }
    }

    fn visible_body(&self) -> String {
        self.body.iter().take(self.body_cursor).collect()
    }

    fn visible_commands(&self) -> Vec<String> {
        MOCK_COMMANDS
            .iter()
            .take(self.commands_revealed)
            .map(|command| (*command).to_owned())
            .collect()
    }

    fn is_active(&self) -> bool {
        self.body_cursor < self.body.len() || self.commands_revealed < MOCK_COMMANDS.len()
    }

    fn is_complete(&self) -> bool {
        !self.is_active()
    }
}

fn draw_thread_row(
    ui: &mut Ui,
    thread: &ChatThread,
    selected: bool,
    is_placeholder: bool,
) -> egui::Response {
    let desired_size = egui::vec2(ui.available_width(), THREAD_ROW_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());
    let fill = tokens::row_bg(
        selected,
        response.is_pointer_button_down_on(),
        response.hovered(),
        true,
    );

    ui.painter().rect(
        rect,
        CornerRadius::same(12),
        fill,
        Stroke::NONE,
        StrokeKind::Outside,
    );

    let title_pos = egui::pos2(rect.left() + 12.0, rect.top() + 12.0);
    let preview_pos = egui::pos2(rect.left() + 12.0, rect.top() + 31.0);
    let time_pos = egui::pos2(rect.right() - 12.0, rect.top() + 13.0);
    let preview_text = if is_placeholder {
        "No messages yet".to_owned()
    } else {
        truncate_line(thread.preview.as_str(), 34)
    };

    ui.painter().text(
        title_pos,
        Align2::LEFT_TOP,
        thread.title.as_str(),
        FontId::new(12.5, FontFamily::Proportional),
        tokens::text_primary(true),
    );
    ui.painter().text(
        preview_pos,
        Align2::LEFT_TOP,
        preview_text,
        FontId::new(10.5, FontFamily::Proportional),
        tokens::text_muted(true),
    );
    ui.painter().text(
        time_pos,
        Align2::RIGHT_TOP,
        thread.updated_at.as_str(),
        FontId::new(11.0, FontFamily::Proportional),
        tokens::text_muted(true),
    );

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn draw_message(ui: &mut Ui, message: &ChatMessage) {
    match message.role {
        MessageRole::User => draw_user_message(ui, message),
        MessageRole::Assistant => draw_assistant_message(ui, message),
    }
}

fn draw_user_message(ui: &mut Ui, message: &ChatMessage) {
    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
        ui.set_max_width(460.0);
        ui.label(
            RichText::new(message.body.as_str())
                .size(14.0)
                .color(tokens::text_primary(true)),
        );
    });
}

fn draw_assistant_message(ui: &mut Ui, message: &ChatMessage) {
    let mut body = message.body.clone();
    if message.streaming {
        body.push_str("  ");
        body.push('\u{258b}');
    }

    for paragraph in body
        .split("\n\n")
        .filter(|paragraph| !paragraph.trim().is_empty())
    {
        ui.label(
            RichText::new(paragraph.trim_end())
                .size(14.0)
                .color(tokens::text_secondary(true)),
        );
        ui.add_space(8.0);
    }

    if !message.commands.is_empty() || message.streaming {
        ui.add_space(4.0);
        ui.label(
            RichText::new("Mock commands")
                .size(11.0)
                .color(tokens::text_muted(true)),
        );

        if message.commands.is_empty() {
            ui.label(
                RichText::new("Waiting for generated command suggestions...")
                    .size(12.0)
                    .color(tokens::text_muted(true)),
            );
        } else {
            for command in &message.commands {
                draw_command_row(ui, command.as_str());
            }
        }
    }
}

fn draw_command_row(ui: &mut Ui, command: &str) {
    let dark_mode = true;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        paint_sidebar_icon(ui, "sparkles");
        ui.label(
            RichText::new(command)
                .size(12.5)
                .family(FontFamily::Monospace)
                .color(tokens::text_primary(dark_mode)),
        );
    });
}

fn draw_empty_state(ui: &mut Ui) {
    ui.label(
        RichText::new("Start a new thread")
            .size(16.0)
            .color(tokens::text_primary(true)),
    );
    ui.add_space(4.0);
    ui.label(
        RichText::new(
            "Type a message below and the assistant will fake-stream a response with a mock commands section.",
        )
        .size(13.0)
        .color(tokens::text_secondary(true)),
    );
}

fn center_column(ui: &mut Ui, width: f32, add: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        let leading_space = ((ui.available_width() - width) * 0.5).max(0.0);
        if leading_space > 0.0 {
            ui.add_space(leading_space);
        }
        ui.allocate_ui_with_layout(egui::vec2(width, 0.0), Layout::top_down(Align::Min), |ui| {
            add(ui)
        });
    });
}

fn paint_sidebar_icon(ui: &mut Ui, icon_name: &str) {
    if let Some(image) = icons::image(ui.ctx(), icon_name, 14.0) {
        let _ = ui.add(image.tint(tokens::text_muted(true)));
    }
}

fn draw_jump_to_end_button(context: &Context, conversation_rect: egui::Rect) -> bool {
    let button_size = ROUND_ICON_BUTTON_SIZE;
    let button_rect = egui::Rect::from_min_size(
        egui::pos2(
            conversation_rect.right() - button_size - 10.0,
            conversation_rect.bottom() - button_size - 10.0,
        ),
        Vec2::splat(button_size),
    );

    egui::Area::new(egui::Id::new("chat_jump_to_end"))
        .order(egui::Order::Foreground)
        .fixed_pos(button_rect.min)
        .interactable(true)
        .show(context, |ui| {
            theme::apply_component_theme(ui);
            draw_round_icon_button(ui, "arrow-down", ButtonStyle::Secondary, true, button_size)
                .clicked()
        })
        .inner
}

fn draw_round_icon_button(
    ui: &mut Ui,
    icon_name: &str,
    style: ButtonStyle,
    enabled: bool,
    size: f32,
) -> egui::Response {
    let dark_mode = ui.visuals().dark_mode;
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), sense);
    let (base_fill, hover_fill, active_fill, icon_fill) = match style {
        ButtonStyle::Primary => (
            tokens::primary_bg(dark_mode),
            tokens::primary_hover_bg(dark_mode),
            tokens::primary_active_bg(dark_mode),
            tokens::primary_fg(dark_mode),
        ),
        ButtonStyle::Secondary => (
            tokens::muted_surface(dark_mode),
            tokens::row_hover_bg(dark_mode),
            tokens::row_selected_bg(dark_mode),
            tokens::text_primary(dark_mode),
        ),
        ButtonStyle::Ghost | ButtonStyle::Link => (
            tokens::TRANSPARENT,
            tokens::row_hover_bg(dark_mode),
            tokens::row_active_bg(dark_mode),
            tokens::text_secondary(dark_mode),
        ),
    };
    let fill = if !enabled {
        tokens::muted_surface(dark_mode)
    } else if response.is_pointer_button_down_on() {
        active_fill
    } else if response.hovered() {
        hover_fill
    } else {
        base_fill
    };
    let icon_tint = if enabled {
        icon_fill
    } else {
        tokens::text_muted(dark_mode)
    };

    if fill != tokens::TRANSPARENT {
        ui.painter()
            .circle_filled(rect.center(), rect.width() * 0.5, fill);
    }

    if let Some(image) = icons::image(ui.ctx(), icon_name, 14.0) {
        let icon_rect = egui::Rect::from_center_size(rect.center(), Vec2::splat(14.0));
        let _ = image.tint(icon_tint).paint_at(ui, icon_rect);
    }

    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

fn conversation_distance_to_end(content_height: f32, viewport_height: f32, offset_y: f32) -> f32 {
    (content_height - viewport_height - offset_y).max(0.0)
}

fn selected_option<'a>(options: &'a [&'a str], index: Option<usize>) -> &'a str {
    index
        .and_then(|value| options.get(value).copied())
        .unwrap_or(options[0])
}

fn mock_response_body(model: &str, reasoning: &str) -> String {
    format!(
        "Mock response streaming in for the `{model}` model with `{reasoning}` reasoning enabled.\n\nThis is a placeholder LLM conversation surface, so the assistant is intentionally faking latency while keeping the layout restrained and neutral.\n\nWhen the real backend arrives, the composer text and the two select values can flow straight into the request payload without changing the shell of this interface."
    )
}

fn thread_title_from_prompt(prompt: &str) -> String {
    let normalized = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return NEW_THREAD_TITLE.to_owned();
    }

    let max_chars = 28;
    if normalized.chars().count() <= max_chars {
        return normalized;
    }

    let trimmed = normalized
        .char_indices()
        .take_while(|(index, _)| *index < max_chars)
        .map(|(_, ch)| ch)
        .collect::<String>();
    let word_boundary = trimmed.rfind(' ').unwrap_or(trimmed.len());
    format!("{} ...", trimmed[..word_boundary].trim_end())
}

fn truncate_line(text: &str, max_chars: usize) -> String {
    let mut result = String::new();
    let mut count = 0usize;
    for ch in text.chars() {
        if count >= max_chars {
            result.push_str("...");
            return result;
        }
        result.push(ch);
        count += 1;
    }
    result
}

fn seed_design_system_sync_messages() -> Vec<ChatMessage> {
    vec![
        ChatMessage::user("Tighten the density across the chat shell and keep it neutral.".to_owned()),
        ChatMessage::assistant_complete(
            "I collapsed the heavier surfaces first so the app reads like a working tool instead of a component demo. The sidebar and composer now rely on spacing and contrast instead of card borders.".to_owned(),
            vec![],
        ),
        ChatMessage::user("Make the conversation stick to the end by default, but stop doing that once I scroll upward.".to_owned()),
        ChatMessage::assistant_complete(
            "That behavior is now scoped to the conversation scroll area. It starts locked to the end, releases as soon as the viewport drifts away from the bottom threshold, and relocks when the user returns to the end.".to_owned(),
            vec!["cargo run chat".to_owned()],
        ),
        ChatMessage::user("The composer still feels too padded and the send affordance is too square.".to_owned()),
        ChatMessage::assistant_complete(
            "I reduced the textarea height, removed the outer card, and swapped the send control over to a circular icon button so the composer feels lighter and more direct.".to_owned(),
            vec![],
        ),
        ChatMessage::user("Keep the model and reasoning controls, but drop the labels.".to_owned()),
        ChatMessage::assistant_complete(
            "Done. The selected values are descriptive enough on their own, so the extra labels were just adding height. The controls now sit on the same compact row as the action buttons.".to_owned(),
            vec![],
        ),
        ChatMessage::user("Include a mock commands section in the response stream so I can evaluate spacing with mixed content.".to_owned()),
        ChatMessage::assistant_complete(
            "The assistant stream now grows in two phases: text first, then command suggestions. That keeps the interaction close to a real tool-use transcript without needing a backend yet.".to_owned(),
            vec![
                "cargo fmt".to_owned(),
                "cargo check".to_owned(),
                "cargo test".to_owned(),
            ],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::{conversation_distance_to_end, thread_title_from_prompt, truncate_line};

    #[test]
    fn truncates_sidebar_lines_consistently() {
        assert_eq!(truncate_line("short", 10), "short");
        assert_eq!(truncate_line("123456", 4), "1234...");
    }

    #[test]
    fn builds_thread_titles_from_prompt_text() {
        assert_eq!(thread_title_from_prompt(""), "New thread");
        assert_eq!(
            thread_title_from_prompt("  tighten the toolbar spacing for mobile  "),
            "tighten the toolbar spacing ..."
        );
    }

    #[test]
    fn conversation_distance_clamps_to_zero() {
        assert_eq!(conversation_distance_to_end(400.0, 300.0, 120.0), 0.0);
        assert_eq!(conversation_distance_to_end(400.0, 300.0, 60.0), 40.0);
    }
}
