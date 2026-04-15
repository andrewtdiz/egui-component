use super::api::ComponentUi;
use crate::ui::{tokens, typography};
use egui::{
    vec2, Color32, CornerRadius, Id, Order, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    Vec2,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

const CURSOR_SVG_WIDTH: f32 = 18.0;
const CURSOR_SVG_HEIGHT: f32 = 18.0;
const CURSOR_TIP_X: f32 = 2.9;
const CURSOR_TIP_Y: f32 = 3.2;
const DEFAULT_CURSOR_HEIGHT: f32 = 30.0;
const CURSOR_FILL_PATH: &str = "M14.082 2.182a.5.5 0 0 1 .103.557L8.528 15.467a.5.5 0 0 1-.917-.007L5.57 10.694.803 8.652a.5.5 0 0 1-.006-.916l12.728-5.657a.5.5 0 0 1 .556.103z";
const CURSOR_TRANSFORM: &str = "translate(16 0) scale(-1 1)";

static CURSOR_SVG_CACHE: OnceLock<Mutex<HashMap<u32, Arc<[u8]>>>> = OnceLock::new();

#[derive(Debug, Clone, Copy)]
pub struct CollabCursor<'a> {
    pub id: Id,
    pub name: &'a str,
    pub position: Pos2,
    pub color: Color32,
    pub size: f32,
}

impl<'a> CollabCursor<'a> {
    pub fn new(id: Id, name: &'a str, position: Pos2) -> Self {
        Self {
            id,
            name,
            position,
            color: Color32::from_rgb(255, 122, 36),
            size: DEFAULT_CURSOR_HEIGHT,
        }
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(1.0);
        self
    }
}

impl ComponentUi<'_> {
    pub fn collab_cursor<'a>(&mut self, props: impl Into<CollabCursor<'a>>) -> Response {
        draw_collab_cursor(self.ui_mut(), props.into())
    }
}

fn draw_collab_cursor(ui: &mut Ui, props: CollabCursor<'_>) -> Response {
    let cursor_size = cursor_size(props.size);
    let badge_metrics = measure_badge(ui, props.name, props.size);
    let badge_origin = badge_origin(cursor_size, badge_metrics.total_size, props.size);
    let total_size = total_size(cursor_size, badge_origin, badge_metrics.total_size);
    let area_position = props.position - cursor_tip_offset(props.size);
    let cursor_uri = ensure_cursor_uri(ui.ctx(), props.color);
    let runtime = crate::theme::runtime_for_ui(ui);
    let badge_fill = props.color;
    let badge_stroke = Stroke::new(1.0, props.color.lerp_to_gamma(Color32::BLACK, 0.28));
    let badge_shadow = props
        .color
        .lerp_to_gamma(Color32::BLACK, 0.58)
        .linear_multiply(if runtime.mode.is_dark() { 0.34 } else { 0.22 });
    let badge_corner_radius = CornerRadius::same(tokens::radius_md(runtime));

    egui::Area::new(props.id.with("area"))
        .order(Order::Foreground)
        .fixed_pos(area_position)
        .show(ui.ctx(), |ui| {
            let (rect, response) = ui.allocate_exact_size(total_size, Sense::hover());
            let cursor_rect = Rect::from_min_size(rect.min, cursor_size);
            let badge_rect = Rect::from_min_size(rect.min + badge_origin, badge_metrics.total_size);

            let _ = ui.put(
                cursor_rect,
                egui::Image::from_uri(cursor_uri.clone()).fit_to_exact_size(cursor_size),
            );

            ui.painter().rect_filled(
                badge_rect.translate(vec2(0.0, 2.0)),
                badge_corner_radius,
                badge_shadow,
            );
            ui.painter().rect(
                badge_rect,
                badge_corner_radius,
                badge_fill,
                badge_stroke,
                StrokeKind::Outside,
            );
            ui.painter().text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                props.name,
                badge_metrics.font,
                Color32::WHITE,
            );

            response
        })
        .inner
}

#[derive(Clone)]
struct BadgeMetrics {
    total_size: Vec2,
    font: egui::FontId,
}

fn measure_badge(ui: &Ui, name: &str, cursor_height: f32) -> BadgeMetrics {
    let font_size = (cursor_height * 0.42).clamp(11.0, 14.0);
    let font = typography::semibold_font(font_size);
    let galley = ui
        .painter()
        .layout_no_wrap(name.to_owned(), font.clone(), Color32::WHITE);
    let padding_x = (cursor_height * 0.3).clamp(8.0, 12.0);
    let padding_y = (cursor_height * 0.18).clamp(4.0, 7.0);
    let badge_height = (galley.size().y + (padding_y * 2.0)).max(cursor_height * 0.46);
    let badge_width = (galley.size().x + (padding_x * 2.0)).max(badge_height);

    BadgeMetrics {
        total_size: vec2(badge_width, badge_height),
        font,
    }
}

fn badge_origin(cursor_size: Vec2, badge_size: Vec2, cursor_height: f32) -> Vec2 {
    vec2(
        cursor_size.x - (cursor_height * 0.08),
        (cursor_size.y - badge_size.y + (cursor_height * 0.04)).max(cursor_height * 0.7),
    )
}

fn total_size(cursor_size: Vec2, badge_origin: Vec2, badge_size: Vec2) -> Vec2 {
    vec2(
        cursor_size.x.max(badge_origin.x + badge_size.x),
        cursor_size.y.max(badge_origin.y + badge_size.y),
    )
}

fn cursor_size(cursor_height: f32) -> Vec2 {
    vec2(
        (cursor_height * CURSOR_SVG_WIDTH) / CURSOR_SVG_HEIGHT,
        cursor_height,
    )
}

fn cursor_tip_offset(cursor_height: f32) -> Vec2 {
    let scale = cursor_height / CURSOR_SVG_HEIGHT;
    vec2(CURSOR_TIP_X * scale, CURSOR_TIP_Y * scale)
}

fn ensure_cursor_uri(ctx: &egui::Context, color: Color32) -> String {
    let uri = collab_cursor_uri(color);
    let svg_bytes = Arc::clone(&load_cursor_svg_bytes(color));
    ctx.include_bytes(uri.clone(), svg_bytes);
    uri
}

fn load_cursor_svg_bytes(color: Color32) -> Arc<[u8]> {
    let cache = collab_cursor_cache();
    let mut guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let key = color_cache_key(color);

    if let Some(bytes) = guard.get(&key) {
        return Arc::clone(bytes);
    }

    let bytes = Arc::<[u8]>::from(collab_cursor_svg(color).into_bytes().into_boxed_slice());
    guard.insert(key, Arc::clone(&bytes));
    bytes
}

fn collab_cursor_cache() -> &'static Mutex<HashMap<u32, Arc<[u8]>>> {
    CURSOR_SVG_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn color_cache_key(color: Color32) -> u32 {
    let [r, g, b, a] = color.to_array();
    u32::from_be_bytes([r, g, b, a])
}

fn collab_cursor_uri(color: Color32) -> String {
    format!(
        "bytes://egui-component/collab-cursor/{}.svg",
        color_hex_rgba(color)
    )
}

fn color_hex_rgba(color: Color32) -> String {
    let [r, g, b, a] = color.to_array();
    format!("{r:02x}{g:02x}{b:02x}{a:02x}")
}

fn collab_cursor_svg(color: Color32) -> String {
    let fill = format!("#{}", color_hex_rgba(color));
    format!(
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"18\" height=\"18\" viewBox=\"-1 -1 18 18\" fill=\"none\">",
            "<defs>",
            "<filter id=\"shadow\" x=\"-40%\" y=\"-30%\" width=\"220%\" height=\"220%\" color-interpolation-filters=\"sRGB\">",
            "<feDropShadow dx=\"0.8\" dy=\"1.2\" stdDeviation=\"0.75\" flood-color=\"#0B1220\" flood-opacity=\"0.34\"/>",
            "</filter>",
            "</defs>",
            "<g filter=\"url(#shadow)\">",
            "<path d=\"{fill_path}\" fill=\"none\" stroke=\"#FFFFFF\" stroke-width=\"1.75\" stroke-linejoin=\"round\" stroke-linecap=\"round\" transform=\"{transform}\"/>",
            "<path d=\"{fill_path}\" fill=\"{fill}\" transform=\"{transform}\"/>",
            "</g>",
            "</svg>"
        ),
        fill_path = CURSOR_FILL_PATH,
        transform = CURSOR_TRANSFORM,
        fill = fill,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        collab_cursor_svg, collab_cursor_uri, CollabCursor, CURSOR_FILL_PATH, CURSOR_TRANSFORM,
    };
    use crate::runtime_components::ComponentUiExt;
    use crate::theme::{self, BaseColor, ThemeMode, ThemeSpec};
    use egui::{pos2, vec2, CentralPanel, Color32, Context, Id, RawInput, Rect};

    #[test]
    fn collab_cursor_uri_is_color_stable() {
        assert_eq!(
            collab_cursor_uri(Color32::from_rgba_unmultiplied(255, 122, 36, 255)),
            "bytes://egui-component/collab-cursor/ff7a24ff.svg"
        );
    }

    #[test]
    fn collab_cursor_svg_uses_white_outline() {
        let svg = collab_cursor_svg(Color32::from_rgb(12, 200, 140));
        assert!(svg.contains("fill=\"#0cc88cff\""));
        assert!(svg.contains(CURSOR_FILL_PATH));
        assert!(svg.contains("stroke=\"#FFFFFF\""));
        assert!(svg.contains(CURSOR_TRANSFORM));
        assert!(svg.contains("feDropShadow"));
    }

    #[test]
    fn renders_collab_cursor_near_requested_position() {
        let context = Context::default();
        theme::install(
            &context,
            ThemeSpec::preset(BaseColor::Neutral),
            ThemeMode::Dark,
        );
        let target = pos2(120.0, 84.0);
        let mut rect = Rect::NOTHING;

        for _ in 0..2 {
            let _ = context.run(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(320.0, 240.0))),
                    ..Default::default()
                },
                |context| {
                    CentralPanel::default().show(context, |ui| {
                        rect = ui
                            .components()
                            .collab_cursor(CollabCursor::new(
                                Id::new("collab_cursor_test"),
                                "Lisa Chen",
                                target,
                            ))
                            .rect;
                    });
                },
            );
        }

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
        assert!(rect.left() <= target.x);
        assert!(rect.top() <= target.y);
        assert!(rect.right() > target.x);
        assert!(rect.bottom() > target.y);
    }
}
