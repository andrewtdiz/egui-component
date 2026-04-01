use super::api::{with_component_overrides, ComponentUi};
use crate::{
    primitives::{playback_button_response, PlaybackButtonState, PlaybackButtonStyle},
    ui::tokens,
};
use egui::{Align, CornerRadius, Id, Rect, Response, Sense, Stroke, Ui, UiBuilder, Vec2};
use std::time::Duration;

const AUDIO_PLAYBACK_HEIGHT: f32 = 32.0;
const AUDIO_PLAYBACK_BUTTON_SIZE: f32 = 30.0;
const AUDIO_PLAYBACK_BUTTON_ICON_SIZE: f32 = 13.0;
const AUDIO_PLAYBACK_BUTTON_BORDER_WIDTH: f32 = 2.0;
const AUDIO_PLAYBACK_CONTENT_GAP: f32 = 10.0;
const AUDIO_PLAYBACK_ACTION_GAP: f32 = 6.0;
const AUDIO_PLAYBACK_WAVEFORM_HEIGHT: f32 = 20.0;
const AUDIO_PLAYBACK_DURATION_SECS: f64 = 8.0;
const AUDIO_PLAYBACK_FRAME_TIME: Duration = Duration::from_millis(16);
const AUDIO_PLAYBACK_BAR_PATTERN: [f32; 42] = [
    0.26, 0.54, 0.82, 0.36, 0.64, 0.48, 0.72, 0.30, 0.42, 0.60, 0.80, 0.34, 0.56, 0.38, 0.68, 0.44,
    0.76, 0.28, 0.52, 0.66, 0.40, 0.58, 0.74, 0.32, 0.50, 0.70, 0.88, 0.46, 0.62, 0.36, 0.54, 0.78,
    0.42, 0.60, 0.34, 0.72, 0.48, 0.64, 0.30, 0.56, 0.40, 0.68,
];

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AudioPlaybackState {
    Paused,
    Playing,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct AudioPlaybackResult {
    pub play_pause_clicked: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct AudioPlayback<'a> {
    pub id: Id,
    pub playback_state: AudioPlaybackState,
    pub duration_seconds: Option<f64>,
    pub waveform: Option<&'a [u8]>,
}

impl<'a> AudioPlayback<'a> {
    pub fn new(id: Id, playback_state: AudioPlaybackState) -> Self {
        Self {
            id,
            playback_state,
            duration_seconds: None,
            waveform: None,
        }
    }

    pub fn duration_seconds(mut self, duration_seconds: f64) -> Self {
        if duration_seconds.is_finite() && duration_seconds > 0.0 {
            self.duration_seconds = Some(duration_seconds);
        }
        self
    }

    pub fn waveform(mut self, waveform: &'a [u8]) -> Self {
        self.waveform = Some(waveform);
        self
    }
}

impl ComponentUi<'_> {
    pub fn audio_playback<'a>(
        &mut self,
        props: impl Into<AudioPlayback<'a>>,
    ) -> (Response, AudioPlaybackResult) {
        draw_audio_playback(self, props.into(), false, |_| {})
    }

    pub fn audio_playback_with_actions<'a>(
        &mut self,
        props: impl Into<AudioPlayback<'a>>,
        add_actions: impl FnOnce(&mut Ui),
    ) -> (Response, AudioPlaybackResult) {
        draw_audio_playback(self, props.into(), true, add_actions)
    }
}

fn draw_audio_playback(
    ui: &mut ComponentUi<'_>,
    props: AudioPlayback<'_>,
    show_actions: bool,
    add_actions: impl FnOnce(&mut Ui),
) -> (Response, AudioPlaybackResult) {
    let runtime = crate::theme::runtime_for_ui(ui);
    let style = resolve_audio_playback_style(runtime);
    let playback_duration_seconds = resolve_playback_duration_seconds(props.duration_seconds);
    let playback_progress = playback_progress(
        ui.ui_mut(),
        props.id,
        props.playback_state,
        playback_duration_seconds,
    );
    let row_width = ui.ui().available_width().max(AUDIO_PLAYBACK_BUTTON_SIZE);
    let (row_rect, response) = ui
        .ui_mut()
        .allocate_exact_size(egui::vec2(row_width, AUDIO_PLAYBACK_HEIGHT), Sense::hover());
    let button_rect = egui::Rect::from_center_size(
        egui::pos2(
            row_rect.left() + (AUDIO_PLAYBACK_BUTTON_SIZE * 0.5),
            row_rect.center().y,
        ),
        Vec2::splat(AUDIO_PLAYBACK_BUTTON_SIZE),
    );
    let playback_response = playback_button_response(
        ui.ui_mut(),
        button_rect,
        props.id.with("playback"),
        match props.playback_state {
            AudioPlaybackState::Paused => PlaybackButtonState::Paused,
            AudioPlaybackState::Playing => PlaybackButtonState::Playing,
        },
        style.playback_button,
    );

    let actions_left = if show_actions {
        let overrides = ui.overrides();
        draw_audio_playback_actions(ui.ui_mut(), row_rect, overrides, add_actions)
            .map(|rect| rect.left())
            .unwrap_or(row_rect.right())
    } else {
        row_rect.right()
    };

    let waveform_left = button_rect.right() + AUDIO_PLAYBACK_CONTENT_GAP;
    let waveform_right = (actions_left - AUDIO_PLAYBACK_CONTENT_GAP).max(waveform_left);
    let waveform_rect = egui::Rect::from_min_max(
        egui::pos2(
            waveform_left,
            row_rect.center().y - (AUDIO_PLAYBACK_WAVEFORM_HEIGHT * 0.5),
        ),
        egui::pos2(
            waveform_right,
            row_rect.center().y + (AUDIO_PLAYBACK_WAVEFORM_HEIGHT * 0.5),
        ),
    );

    paint_waveform(
        ui.ui_mut(),
        waveform_rect,
        style.waveform_tint,
        style.waveform_active_tint,
        playback_progress,
        props.waveform,
    );

    (
        response,
        AudioPlaybackResult {
            play_pause_clicked: playback_response.clicked(),
        },
    )
}

#[derive(Debug, Clone, Copy)]
struct AudioPlaybackStyle {
    playback_button: PlaybackButtonStyle,
    waveform_tint: egui::Color32,
    waveform_active_tint: egui::Color32,
}

fn draw_audio_playback_actions(
    ui: &mut Ui,
    row_rect: Rect,
    overrides: crate::components::api::ComponentOverrides,
    add_actions: impl FnOnce(&mut Ui),
) -> Option<Rect> {
    let mut actions_rect = Rect::NOTHING;
    let _ = ui.scope_builder(
        UiBuilder::new()
            .max_rect(row_rect)
            .layout(egui::Layout::right_to_left(Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = AUDIO_PLAYBACK_ACTION_GAP;
            with_component_overrides(ui, overrides, add_actions);
            actions_rect = ui.min_rect();
        },
    );

    if actions_rect.is_positive() {
        Some(actions_rect)
    } else {
        None
    }
}

fn resolve_audio_playback_style(runtime: crate::theme::ThemeRuntime) -> AudioPlaybackStyle {
    let dark_mode = runtime.mode.is_dark();
    AudioPlaybackStyle {
        playback_button: PlaybackButtonStyle {
            icon_size: AUDIO_PLAYBACK_BUTTON_ICON_SIZE,
            fill: tokens::TRANSPARENT,
            hover_fill: tokens::button_secondary_hover_bg(runtime),
            stroke: Stroke::new(
                AUDIO_PLAYBACK_BUTTON_BORDER_WIDTH,
                tokens::text_primary(runtime).gamma_multiply(if dark_mode { 0.58 } else { 0.42 }),
            ),
            icon_tint: tokens::text_primary(runtime),
            paused_icon_name: "bootstrap:play-fill",
            playing_icon_name: "bootstrap:pause-fill",
        },
        waveform_tint: tokens::text_muted(runtime).gamma_multiply(if dark_mode {
            0.9
        } else {
            0.82
        }),
        waveform_active_tint: tokens::text_primary(runtime),
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct AudioPlaybackProgressMemory {
    started_at_secs: f64,
    progress: f32,
    was_playing: bool,
}

fn resolve_playback_duration_seconds(duration_seconds: Option<f64>) -> f64 {
    duration_seconds
        .filter(|duration_seconds| duration_seconds.is_finite() && *duration_seconds > 0.0)
        .unwrap_or(AUDIO_PLAYBACK_DURATION_SECS)
}

fn playback_progress(
    ui: &mut egui::Ui,
    id: Id,
    playback_state: AudioPlaybackState,
    duration_seconds: f64,
) -> f32 {
    let memory_id = id.with("mock_playback_progress");
    let now = ui.ctx().input(|input| input.time);
    let mut memory = ui
        .data(|data| data.get_temp::<AudioPlaybackProgressMemory>(memory_id))
        .unwrap_or_default();

    match playback_state {
        AudioPlaybackState::Playing => {
            if !memory.was_playing {
                let resume_progress = if memory.progress >= 1.0 {
                    0.0
                } else {
                    memory.progress
                };
                memory.started_at_secs = now - (f64::from(resume_progress) * duration_seconds);
                memory.progress = resume_progress;
            }

            memory.progress =
                playback_progress_for_elapsed(now - memory.started_at_secs, duration_seconds);
            memory.was_playing = true;

            ui.data_mut(|data| data.insert_temp(memory_id, memory));

            if memory.progress < 1.0 {
                ui.ctx().request_repaint_after(AUDIO_PLAYBACK_FRAME_TIME);
            }
        }
        AudioPlaybackState::Paused => {
            if memory.was_playing {
                memory.progress =
                    playback_progress_for_elapsed(now - memory.started_at_secs, duration_seconds);
                memory.was_playing = false;
            }

            ui.data_mut(|data| {
                if memory.progress > 0.0 {
                    data.insert_temp(memory_id, memory);
                } else {
                    data.remove::<AudioPlaybackProgressMemory>(memory_id);
                }
            });
        }
    }

    memory.progress
}

fn playback_progress_for_elapsed(elapsed_secs: f64, duration_seconds: f64) -> f32 {
    (elapsed_secs / duration_seconds).clamp(0.0, 1.0) as f32
}

fn paint_waveform(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    tint: egui::Color32,
    active_tint: egui::Color32,
    playback_progress: f32,
    waveform: Option<&[u8]>,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }

    let ideal_bar_count = (rect.width() / 4.0).floor() as usize;
    let bar_count = ideal_bar_count.clamp(8, AUDIO_PLAYBACK_BAR_PATTERN.len());
    let step = rect.width() / bar_count as f32;
    let bar_width = (step * 0.58).clamp(1.5, 3.0);
    let progress_x = egui::lerp(
        rect.left()..=rect.right(),
        playback_progress.clamp(0.0, 1.0),
    );

    for index in 0..bar_count {
        let bar_height = waveform_bar_height(
            rect.height(),
            waveform_bar_factor(waveform, index, bar_count),
        );
        let bar_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + (step * (index as f32 + 0.5)), rect.center().y),
            egui::vec2(bar_width, bar_height),
        );
        ui.painter()
            .rect_filled(bar_rect, CornerRadius::same(2), tint);

        let fill_width = filled_bar_width(bar_rect, progress_x);
        if fill_width > 0.0 {
            let filled_rect = egui::Rect::from_min_max(
                bar_rect.min,
                egui::pos2(bar_rect.left() + fill_width, bar_rect.bottom()),
            );
            ui.painter()
                .rect_filled(filled_rect, CornerRadius::same(2), active_tint);
        }
    }
}

fn waveform_bar_factor(waveform: Option<&[u8]>, index: usize, bar_count: usize) -> f32 {
    let Some(waveform) = waveform.filter(|waveform| !waveform.is_empty()) else {
        let pattern_index = ((index as f32 / bar_count as f32)
            * (AUDIO_PLAYBACK_BAR_PATTERN.len().saturating_sub(1) as f32))
            .round() as usize;
        return AUDIO_PLAYBACK_BAR_PATTERN[pattern_index];
    };

    let waveform_index = ((index as f32 / bar_count as f32)
        * (waveform.len().saturating_sub(1) as f32))
        .round() as usize;
    f32::from(waveform[waveform_index]) / f32::from(u8::MAX)
}

fn waveform_bar_height(total_height: f32, factor: f32) -> f32 {
    (total_height * factor).max(2.0)
}

fn filled_bar_width(bar_rect: egui::Rect, progress_x: f32) -> f32 {
    (progress_x - bar_rect.left()).clamp(0.0, bar_rect.width())
}

#[cfg(test)]
mod tests {
    use super::{
        filled_bar_width, playback_progress_for_elapsed, waveform_bar_factor, waveform_bar_height,
        AudioPlayback, AudioPlaybackProgressMemory, AudioPlaybackResult, AudioPlaybackState,
        AUDIO_PLAYBACK_BUTTON_SIZE, AUDIO_PLAYBACK_DURATION_SECS,
    };
    use crate::components::{Button, ButtonVariant, ComponentUiExt, ControlSize};
    use egui::{
        pos2, vec2, CentralPanel, Context, Event, Id, Modifiers, PointerButton, Pos2, RawInput,
        Rect,
    };

    #[test]
    fn renders_audio_playback_without_panic() {
        let context = Context::default();
        let (rect, _) = render_audio_playback(
            &context,
            RawInput::default(),
            AudioPlayback::new(Id::new("audio_playback_render"), AudioPlaybackState::Paused),
            false,
        );

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
    }

    #[test]
    fn renders_audio_playback_with_actions_without_panic() {
        let context = Context::default();
        let (rect, _) = render_audio_playback(
            &context,
            RawInput::default(),
            AudioPlayback::new(
                Id::new("audio_playback_actions"),
                AudioPlaybackState::Playing,
            ),
            true,
        );

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
    }

    #[test]
    fn action_strip_stays_right_aligned_and_leaves_waveform_room() {
        let context = Context::default();
        let mut actions_rect = Rect::NOTHING;
        let row_rect =
            Rect::from_min_size(pos2(20.0, 20.0), vec2(420.0, super::AUDIO_PLAYBACK_HEIGHT));

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                actions_rect = super::draw_audio_playback_actions(
                    ui,
                    row_rect,
                    crate::components::api::ComponentOverrides::default(),
                    |ui| {
                        let mut ui = ui.components();
                        let _ = ui.button(
                            Button::icon_only("share-2")
                                .variant(ButtonVariant::Ghost)
                                .size(ControlSize::Sm),
                        );
                        let _ = ui.button(
                            Button::icon_only("download")
                                .variant(ButtonVariant::Ghost)
                                .size(ControlSize::Sm),
                        );
                        let _ = ui.button(
                            Button::icon_only("ellipsis")
                                .variant(ButtonVariant::Ghost)
                                .size(ControlSize::Sm),
                        );
                    },
                )
                .unwrap();
            });
        });

        let waveform_left =
            row_rect.left() + super::AUDIO_PLAYBACK_BUTTON_SIZE + super::AUDIO_PLAYBACK_CONTENT_GAP;
        assert!(actions_rect.right() <= row_rect.right() + 0.5);
        assert!(actions_rect.left() > waveform_left + 40.0);
    }

    #[test]
    fn playback_button_reports_click() {
        let context = Context::default();
        let (rect, _) = render_audio_playback(
            &context,
            RawInput::default(),
            AudioPlayback::new(Id::new("audio_playback_click"), AudioPlaybackState::Paused),
            false,
        );
        let button_center = pos2(
            rect.left() + (AUDIO_PLAYBACK_BUTTON_SIZE * 0.5),
            rect.center().y,
        );

        let _ = render_audio_playback(
            &context,
            pointer_input(button_center, true),
            AudioPlayback::new(Id::new("audio_playback_click"), AudioPlaybackState::Paused),
            false,
        );
        let (_, result) = render_audio_playback(
            &context,
            pointer_input(button_center, false),
            AudioPlayback::new(Id::new("audio_playback_click"), AudioPlaybackState::Paused),
            false,
        );

        assert!(result.play_pause_clicked);
    }

    #[test]
    fn playback_progress_clamps_across_duration() {
        assert_eq!(
            playback_progress_for_elapsed(0.0, AUDIO_PLAYBACK_DURATION_SECS),
            0.0
        );
        assert_eq!(
            playback_progress_for_elapsed(
                AUDIO_PLAYBACK_DURATION_SECS * 0.5,
                AUDIO_PLAYBACK_DURATION_SECS,
            ),
            0.5
        );
        assert_eq!(
            playback_progress_for_elapsed(
                AUDIO_PLAYBACK_DURATION_SECS * 2.0,
                AUDIO_PLAYBACK_DURATION_SECS,
            ),
            1.0
        );
    }

    #[test]
    fn provided_waveform_overrides_generic_pattern() {
        assert_eq!(waveform_bar_factor(Some(&[0, 255]), 0, 2), 0.0);
        assert_eq!(waveform_bar_factor(Some(&[0, 255]), 1, 2), 1.0);
    }

    #[test]
    fn waveform_bar_height_comes_only_from_pattern() {
        assert_eq!(waveform_bar_height(20.0, 0.5), 10.0);
        assert_eq!(waveform_bar_height(20.0, 0.04), 2.0);
    }

    #[test]
    fn filled_bar_width_stops_at_progress_boundary() {
        let bar_rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(4.0, 12.0));

        assert_eq!(filled_bar_width(bar_rect, 8.0), 0.0);
        assert_eq!(filled_bar_width(bar_rect, 12.0), 2.0);
        assert_eq!(filled_bar_width(bar_rect, 18.0), 4.0);
    }

    #[test]
    fn paused_playback_keeps_existing_progress() {
        let context = Context::default();
        let playback_id = Id::new("audio_playback_pause_progress");

        let _ = render_audio_playback(
            &context,
            timed_input(0.0),
            AudioPlayback::new(playback_id, AudioPlaybackState::Playing),
            false,
        );
        let _ = render_audio_playback(
            &context,
            timed_input(AUDIO_PLAYBACK_DURATION_SECS * 0.25),
            AudioPlayback::new(playback_id, AudioPlaybackState::Playing),
            false,
        );
        let _ = render_audio_playback(
            &context,
            timed_input(AUDIO_PLAYBACK_DURATION_SECS * 0.25),
            AudioPlayback::new(playback_id, AudioPlaybackState::Paused),
            false,
        );
        let stored_progress = stored_progress(&context, playback_id);

        assert_eq!(stored_progress, Some(0.25));
    }

    fn render_audio_playback(
        context: &Context,
        input: RawInput,
        props: AudioPlayback<'_>,
        with_actions: bool,
    ) -> (Rect, AudioPlaybackResult) {
        let mut rect = Rect::NOTHING;
        let mut result = AudioPlaybackResult::default();

        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                ui.set_width(420.0);
                let (response, playback_result) = if with_actions {
                    ui.components().audio_playback_with_actions(props, |ui| {
                        let mut ui = ui.components();
                        let _ = ui.button(
                            Button::icon_only("share-2")
                                .variant(ButtonVariant::Ghost)
                                .size(ControlSize::Sm),
                        );
                        let _ = ui.button(
                            Button::icon_only("download")
                                .variant(ButtonVariant::Ghost)
                                .size(ControlSize::Sm),
                        );
                    })
                } else {
                    ui.components().audio_playback(props)
                };
                rect = response.rect;
                result = playback_result;
            });
        });

        (rect, result)
    }

    fn pointer_input(pos: Pos2, pressed: bool) -> RawInput {
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(480.0, 160.0))),
            events: vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..Default::default()
        }
    }

    fn timed_input(time_secs: f64) -> RawInput {
        RawInput {
            time: Some(time_secs),
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(480.0, 160.0))),
            ..Default::default()
        }
    }

    fn stored_progress(context: &Context, id: Id) -> Option<f32> {
        context.data(|data| {
            data.get_temp::<AudioPlaybackProgressMemory>(id.with("mock_playback_progress"))
                .map(|memory| memory.progress)
        })
    }
}
