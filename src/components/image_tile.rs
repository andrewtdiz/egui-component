use super::{
    api::{with_component_overrides, ComponentUi},
    Image,
};
use crate::{
    primitives::{
        playback_button_response,
        surface::{surface_frame_builder, SurfaceFrame},
        PlaybackButtonState, PlaybackButtonStyle,
    },
    ui::tokens,
};
use egui::{CornerRadius, CursorIcon, Layout, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

const PLAYBACK_BUTTON_SIZE: f32 = 30.0;
const PLAYBACK_ICON_SIZE: f32 = 12.0;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ImageTileSize {
    Sm,
    Md,
    Lg,
}

impl ImageTileSize {
    fn image_size(self) -> Vec2 {
        match self {
            Self::Sm => egui::vec2(144.0, 96.0),
            Self::Md => egui::vec2(216.0, 144.0),
            Self::Lg => egui::vec2(288.0, 192.0),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ImageTilePlaybackState {
    Paused,
    Playing,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct ImageTileState {
    pub tile_clicked: bool,
    pub play_pause_clicked: bool,
}

#[derive(Debug, Clone)]
pub struct ImageTile<'a> {
    pub image: Image<'a>,
    pub size: ImageTileSize,
    pub image_size: Option<Vec2>,
    pub image_frame: bool,
    pub selected: bool,
    pub playback_state: Option<ImageTilePlaybackState>,
}

impl<'a> ImageTile<'a> {
    pub fn new(image: impl Into<Image<'a>>) -> Self {
        Self {
            image: image.into(),
            size: ImageTileSize::Md,
            image_size: None,
            image_frame: true,
            selected: false,
            playback_state: None,
        }
    }

    pub fn size(mut self, size: ImageTileSize) -> Self {
        self.size = size;
        self
    }

    pub fn image_size(mut self, image_size: Vec2) -> Self {
        self.image_size = Some(egui::vec2(image_size.x.max(1.0), image_size.y.max(1.0)));
        self
    }

    pub fn image_frame(mut self, image_frame: bool) -> Self {
        self.image_frame = image_frame;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn playback_state(mut self, playback_state: ImageTilePlaybackState) -> Self {
        self.playback_state = Some(playback_state);
        self
    }

    fn resolved_image_size(&self) -> Vec2 {
        self.image_size.unwrap_or_else(|| self.size.image_size())
    }
}

impl<'a> From<Image<'a>> for ImageTile<'a> {
    fn from(image: Image<'a>) -> Self {
        Self::new(image)
    }
}

impl ComponentUi<'_> {
    pub fn image_tile<'a>(
        &mut self,
        props: impl Into<ImageTile<'a>>,
    ) -> (Response, ImageTileState) {
        draw_image_tile(self, props.into(), false, |_| {})
    }

    pub fn image_tile_with_body<'a>(
        &mut self,
        props: impl Into<ImageTile<'a>>,
        add_body: impl FnOnce(&mut Ui),
    ) -> (Response, ImageTileState) {
        draw_image_tile(self, props.into(), true, add_body)
    }
}

fn draw_image_tile(
    ui: &mut ComponentUi<'_>,
    props: ImageTile<'_>,
    show_body: bool,
    add_body: impl FnOnce(&mut Ui),
) -> (Response, ImageTileState) {
    let overrides = ui.overrides();
    let runtime = crate::theme::runtime_for_ui(ui);
    let style = resolve_image_tile_style(runtime, props.selected);
    let image_size = props.resolved_image_size();
    let mut playback_response = None;

    let tile = surface_frame_builder(
        SurfaceFrame::new(style.tile_fill, style.tile_stroke)
            .corner_radius(tokens::radius_lg(runtime))
            .padding(tokens::INPUT_PADDING_X, tokens::INPUT_PADDING_Y),
    )
    .show(ui.ui_mut(), |ui| {
        let _ = ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
            ui.set_min_width(image_size.x);
            ui.set_max_width(image_size.x);

            let image_rect = if props.image_frame {
                surface_frame_builder(
                    SurfaceFrame::new(style.image_fill, style.image_stroke)
                        .corner_radius(tokens::radius_md(runtime)),
                )
                .show(ui, |ui| {
                    let mut components = ComponentUi::with_overrides(ui, overrides);
                    let _ = components.image(
                        props
                            .image
                            .fit_to_exact_size(image_size)
                            .corner_radius(tokens::radius_md(runtime)),
                    );
                })
                .response
                .rect
            } else {
                let mut components = ComponentUi::with_overrides(ui, overrides);
                components
                    .image(
                        props
                            .image
                            .fit_to_exact_size(image_size)
                            .corner_radius(tokens::radius_lg(runtime)),
                    )
                    .rect
            };

            if let Some(playback_state) = props.playback_state {
                playback_response =
                    Some(draw_playback_button(ui, image_rect, playback_state, &style));
            }

            if show_body {
                ui.add_space(tokens::SPACING_ITEM_Y);
                with_component_overrides(ui, overrides, add_body);
            }
        });
    });

    let tile_response = tile.response.clone();
    let tile_rect = tile_response.rect;
    let tile_interaction_response = tile_response.clone().interact(Sense::click());
    let overlay_hovered = playback_response.as_ref().is_some_and(Response::hovered);
    let overlay_pressed = playback_response
        .as_ref()
        .is_some_and(Response::is_pointer_button_down_on);
    let play_pause_clicked = playback_response.is_some_and(|response| response.clicked());

    let outline_stroke = if tile_interaction_response.is_pointer_button_down_on() || overlay_pressed
    {
        style.tile_active_stroke
    } else if tile_interaction_response.hovered() || overlay_hovered {
        style.tile_hover_stroke
    } else {
        Stroke::NONE
    };

    if outline_stroke != Stroke::NONE {
        ui.ui_mut().painter().rect_stroke(
            tile_rect,
            CornerRadius::same(tokens::radius_lg(runtime)),
            outline_stroke,
            StrokeKind::Outside,
        );
    }

    if tile_interaction_response.hovered() || overlay_hovered {
        tile_response.ctx.set_cursor_icon(CursorIcon::PointingHand);
    }

    (
        tile_response,
        ImageTileState {
            tile_clicked: tile_interaction_response.clicked() && !play_pause_clicked,
            play_pause_clicked,
        },
    )
}

#[derive(Debug, Clone, Copy)]
struct ImageTileStyle {
    tile_fill: egui::Color32,
    tile_stroke: Stroke,
    tile_hover_stroke: Stroke,
    tile_active_stroke: Stroke,
    image_fill: egui::Color32,
    image_stroke: Stroke,
    playback_fill: egui::Color32,
    playback_hover_fill: egui::Color32,
    playback_active_fill: egui::Color32,
    playback_stroke: Stroke,
    playback_icon_tint: egui::Color32,
}

fn resolve_image_tile_style(runtime: crate::theme::ThemeRuntime, selected: bool) -> ImageTileStyle {
    let light_runtime = runtime.with_mode(crate::theme::ThemeMode::Light);
    ImageTileStyle {
        tile_fill: if selected {
            tokens::image_tile_selected_fill(runtime)
        } else {
            tokens::muted_surface(runtime)
        },
        tile_stroke: Stroke::new(
            1.0,
            if selected {
                tokens::image_tile_selected_stroke(runtime)
            } else {
                tokens::separator(runtime)
            },
        ),
        tile_hover_stroke: Stroke::new(1.0, tokens::input_hover_border(runtime)),
        tile_active_stroke: tokens::input_focus_stroke(runtime),
        image_fill: tokens::card_background(runtime),
        image_stroke: Stroke::new(1.0, tokens::separator(runtime)),
        playback_fill: tokens::primary_bg(light_runtime),
        playback_hover_fill: tokens::primary_hover_bg(light_runtime),
        playback_active_fill: tokens::primary_active_bg(light_runtime),
        playback_stroke: Stroke::new(1.0, tokens::separator(light_runtime)),
        playback_icon_tint: tokens::primary_fg(light_runtime),
    }
}

fn draw_playback_button(
    ui: &mut Ui,
    image_rect: Rect,
    playback_state: ImageTilePlaybackState,
    style: &ImageTileStyle,
) -> Response {
    let button_rect = playback_button_rect(image_rect);
    playback_button_response(
        ui,
        button_rect,
        ui.auto_id_with("image_tile_playback"),
        match playback_state {
            ImageTilePlaybackState::Paused => PlaybackButtonState::Paused,
            ImageTilePlaybackState::Playing => PlaybackButtonState::Playing,
        },
        PlaybackButtonStyle {
            icon_size: PLAYBACK_ICON_SIZE,
            fill: style.playback_fill,
            hover_fill: style.playback_hover_fill,
            active_fill: style.playback_active_fill,
            stroke: style.playback_stroke,
            icon_tint: style.playback_icon_tint,
            paused_icon_name: "play",
            playing_icon_name: "pause",
        },
    )
}

fn playback_button_rect(image_rect: Rect) -> Rect {
    let inset = f32::from(tokens::INPUT_PADDING_Y);
    Rect::from_min_size(
        egui::pos2(
            image_rect.left() + inset,
            image_rect.bottom() - inset - PLAYBACK_BUTTON_SIZE,
        ),
        Vec2::splat(PLAYBACK_BUTTON_SIZE),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        draw_playback_button, playback_button_rect, resolve_image_tile_style, ImageTile,
        ImageTilePlaybackState, ImageTileSize, ImageTileState,
    };
    use crate::components::{ComponentUiExt, Image};
    use egui::{
        pos2, vec2, CentralPanel, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect,
        Sense,
    };

    const SHOWCASE_PNG_BYTES: &[u8] = include_bytes!("../../assets/images/showcase-image.png");

    #[test]
    fn explicit_image_size_overrides_preset_size() {
        let tile = ImageTile::new(Image::from_bytes(
            "bytes://tests/showcase-image-tile.png",
            SHOWCASE_PNG_BYTES,
        ))
        .size(ImageTileSize::Lg)
        .image_size(vec2(180.0, 120.0));

        assert_eq!(tile.resolved_image_size(), vec2(180.0, 120.0));
    }

    #[test]
    fn preset_sizes_map_to_expected_dimensions() {
        let base = ImageTile::new(Image::from_bytes(
            "bytes://tests/showcase-image-tile.png",
            SHOWCASE_PNG_BYTES,
        ));

        assert_eq!(
            base.clone().size(ImageTileSize::Sm).resolved_image_size(),
            vec2(144.0, 96.0)
        );
        assert_eq!(
            base.clone().size(ImageTileSize::Md).resolved_image_size(),
            vec2(216.0, 144.0)
        );
        assert_eq!(
            base.size(ImageTileSize::Lg).resolved_image_size(),
            vec2(288.0, 192.0)
        );
    }

    #[test]
    fn renders_image_tile_without_panic() {
        let context = Context::default();
        egui_extras::install_image_loaders(&context);

        let tile = ImageTile::new(Image::from_bytes(
            "bytes://tests/showcase-image-tile.png",
            SHOWCASE_PNG_BYTES,
        ));
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = ui.components().image_tile(tile.clone()).0.rect;
            });
        });

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
    }

    #[test]
    fn renders_image_tile_with_body_without_panic() {
        let context = Context::default();
        egui_extras::install_image_loaders(&context);

        let tile = ImageTile::new(Image::from_bytes(
            "bytes://tests/showcase-image-tile.png",
            SHOWCASE_PNG_BYTES,
        ))
        .size(ImageTileSize::Sm);
        let mut rect = Rect::NOTHING;

        let _ = context.run(RawInput::default(), |context| {
            CentralPanel::default().show(context, |ui| {
                rect = ui
                    .components()
                    .image_tile_with_body(tile.clone(), |ui| {
                        let _ = ui.label("Untitled Design");
                    })
                    .0
                    .rect;
            });
        });

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 96.0);
    }

    #[test]
    fn playback_button_reports_click() {
        let context = Context::default();
        let (image_rect, _) = render_playback_button(&context, RawInput::default());
        let button_center = playback_button_rect(image_rect).center();

        let _ = render_playback_button(&context, pointer_input(button_center, true));
        let (_, clicked) = render_playback_button(&context, pointer_input(button_center, false));

        assert!(clicked);
    }

    #[test]
    fn clicking_tile_background_sets_tile_clicked_without_triggering_play_pause_state() {
        let context = Context::default();
        egui_extras::install_image_loaders(&context);

        let tile = ImageTile::new(Image::from_bytes(
            "bytes://tests/showcase-image-tile.png",
            SHOWCASE_PNG_BYTES,
        ))
        .playback_state(ImageTilePlaybackState::Paused);

        let (tile_rect, _) = render_tile(&context, RawInput::default(), tile.clone());
        let background_center = tile_rect.center();

        let _ = render_tile(
            &context,
            pointer_input(background_center, true),
            tile.clone(),
        );
        let (_, state) = render_tile(&context, pointer_input(background_center, false), tile);

        assert!(state.tile_clicked);
        assert!(!state.play_pause_clicked);
    }

    fn render_tile(
        context: &Context,
        input: RawInput,
        tile: ImageTile<'static>,
    ) -> (Rect, ImageTileState) {
        let mut rect = Rect::NOTHING;
        let mut state = ImageTileState::default();

        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                let (response, image_tile_state) = ui.components().image_tile(tile.clone());
                rect = response.rect;
                state = image_tile_state;
            });
        });

        (rect, state)
    }

    fn render_playback_button(context: &Context, input: RawInput) -> (Rect, bool) {
        let mut image_rect = Rect::NOTHING;
        let mut clicked = false;

        let _ = context.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                let style = resolve_image_tile_style(crate::theme::runtime_for_ui(ui), false);
                image_rect = ui.allocate_exact_size(vec2(216.0, 144.0), Sense::hover()).0;
                clicked =
                    draw_playback_button(ui, image_rect, ImageTilePlaybackState::Paused, &style)
                        .clicked();
            });
        });

        (image_rect, clicked)
    }

    fn pointer_input(pos: Pos2, pressed: bool) -> RawInput {
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 400.0))),
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
}
