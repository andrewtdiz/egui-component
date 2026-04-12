mod parse;
mod parse_color_typography;
mod parse_layout;
mod tokens;
mod types;

use std::collections::{HashMap, VecDeque};
use std::sync::{OnceLock, RwLock};

use crate::theme::{ColorRole, ThemeRuntime};

#[cfg(test)]
use types::ThemeColorRef;
pub(crate) use types::{
    pack_rgba, unpack_rgba, Color, ColorAsk, ColorRef, FontWeight, Height, PaddingValue,
    SideValues, Spec, TextSize, ThemeStyle, UiRuntimeBackground, Width,
};

const SPEC_CACHE_MAX_ENTRIES: usize = 4096;
const SPEC_CACHE_EVICT_BATCH: usize = 64;

struct SpecCache {
    max_entries: usize,
    evict_batch: usize,
    map: HashMap<String, Spec>,
    order: VecDeque<String>,
}

impl SpecCache {
    fn new(max_entries: usize, evict_batch: usize) -> Self {
        Self {
            max_entries,
            evict_batch,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn get(&self, classes: &str) -> Option<&Spec> {
        self.map.get(classes)
    }

    fn insert(&mut self, classes: &str, spec: Spec) {
        if classes.is_empty() || self.max_entries == 0 {
            return;
        }
        if self.map.contains_key(classes) {
            return;
        }
        let key = classes.to_owned();
        self.map.insert(key.clone(), spec);
        self.order.push_back(key);
        self.evict_if_needed();
    }

    #[cfg(test)]
    fn count(&self) -> usize {
        self.map.len()
    }

    #[cfg(test)]
    fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }

    fn evict_if_needed(&mut self) {
        let count = self.map.len();
        if count <= self.max_entries {
            return;
        }

        let mut to_remove = count - self.max_entries;
        if to_remove < self.evict_batch {
            to_remove = self.evict_batch;
        }
        if to_remove > count {
            to_remove = count;
        }

        while to_remove > 0 {
            let batch = to_remove.min(self.evict_batch);
            for _ in 0..batch {
                let Some(key) = self.order.pop_front() else {
                    return;
                };
                if self.map.remove(&key).is_some() {
                    to_remove -= 1;
                    if to_remove == 0 {
                        return;
                    }
                }
            }
        }
    }
}

fn spec_cache() -> &'static RwLock<SpecCache> {
    static CACHE: OnceLock<RwLock<SpecCache>> = OnceLock::new();
    CACHE.get_or_init(|| {
        RwLock::new(SpecCache::new(
            SPEC_CACHE_MAX_ENTRIES,
            SPEC_CACHE_EVICT_BATCH,
        ))
    })
}

pub(crate) trait ThemeColorResolver {
    fn resolve_theme_color(&self, style: ThemeStyle, ask: ColorAsk) -> Color;
}

impl ThemeColorResolver for ThemeRuntime {
    fn resolve_theme_color(&self, style: ThemeStyle, ask: ColorAsk) -> Color {
        let role = match style {
            ThemeStyle::Content => match ask {
                ColorAsk::Fill | ColorAsk::FillHover => ColorRole::Card,
                ColorAsk::Text | ColorAsk::TextHover => ColorRole::Foreground,
                ColorAsk::Border => ColorRole::Border,
            },
            ThemeStyle::Role(role) => role,
        };
        crate::theme::resolved_color(*self, role)
    }
}

pub(crate) fn resolve_color<R: ThemeColorResolver>(resolver: &R, color_ref: ColorRef) -> Color {
    match color_ref {
        ColorRef::PalettePacked(packed_value) => unpack_rgba(packed_value),
        ColorRef::Theme(role) => resolver.resolve_theme_color(role.style, role.ask),
    }
}

pub(crate) fn lookup_color_token(ask: ColorAsk, token: &str) -> Option<ColorRef> {
    let token = token.trim();
    if token.is_empty() {
        return None;
    }
    if let Some(packed) = lookup_palette_color(token) {
        return Some(ColorRef::PalettePacked(packed));
    }
    if token == "content" {
        return Some(ColorRef::Theme(types::ThemeColorRef {
            style: ThemeStyle::Content,
            ask,
        }));
    }
    lookup_color_role(token).map(|role| {
        ColorRef::Theme(types::ThemeColorRef {
            style: ThemeStyle::Role(role),
            ask,
        })
    })
}

fn lookup_color_role(token: &str) -> Option<ColorRole> {
    match token {
        "background" => Some(ColorRole::Background),
        "foreground" => Some(ColorRole::Foreground),
        "card" => Some(ColorRole::Card),
        "card-foreground" => Some(ColorRole::CardForeground),
        "popover" => Some(ColorRole::Popover),
        "popover-foreground" => Some(ColorRole::PopoverForeground),
        "primary" => Some(ColorRole::Primary),
        "primary-foreground" => Some(ColorRole::PrimaryForeground),
        "secondary" => Some(ColorRole::Secondary),
        "secondary-foreground" => Some(ColorRole::SecondaryForeground),
        "muted" => Some(ColorRole::Muted),
        "muted-foreground" => Some(ColorRole::MutedForeground),
        "accent" => Some(ColorRole::Accent),
        "accent-foreground" => Some(ColorRole::AccentForeground),
        "destructive" => Some(ColorRole::Destructive),
        "destructive-foreground" => Some(ColorRole::DestructiveForeground),
        "border" => Some(ColorRole::Border),
        "input" => Some(ColorRole::Input),
        "ring" => Some(ColorRole::Ring),
        "chart-1" => Some(ColorRole::Chart1),
        "chart-2" => Some(ColorRole::Chart2),
        "chart-3" => Some(ColorRole::Chart3),
        "chart-4" => Some(ColorRole::Chart4),
        "chart-5" => Some(ColorRole::Chart5),
        "sidebar" => Some(ColorRole::Sidebar),
        "sidebar-foreground" => Some(ColorRole::SidebarForeground),
        "sidebar-primary" => Some(ColorRole::SidebarPrimary),
        "sidebar-primary-foreground" => Some(ColorRole::SidebarPrimaryForeground),
        "sidebar-accent" => Some(ColorRole::SidebarAccent),
        "sidebar-accent-foreground" => Some(ColorRole::SidebarAccentForeground),
        "sidebar-border" => Some(ColorRole::SidebarBorder),
        "sidebar-ring" => Some(ColorRole::SidebarRing),
        _ => None,
    }
}

fn lookup_palette_color(token: &str) -> Option<u32> {
    let (family, shade) = token.rsplit_once('-').unwrap_or((token, ""));
    match (family, shade) {
        ("transparent", "") => Some(pack_rgba(0, 0, 0, 0)),
        ("black", "") => Some(pack_rgba(0, 0, 0, 255)),
        ("white", "") => Some(pack_rgba(255, 255, 255, 255)),
        ("slate", "50") => Some(pack_rgba(248, 250, 252, 255)),
        ("slate", "100") => Some(pack_rgba(241, 245, 249, 255)),
        ("slate", "200") => Some(pack_rgba(226, 232, 240, 255)),
        ("slate", "300") => Some(pack_rgba(203, 213, 225, 255)),
        ("slate", "400") => Some(pack_rgba(148, 163, 184, 255)),
        ("slate", "500") => Some(pack_rgba(100, 116, 139, 255)),
        ("slate", "600") => Some(pack_rgba(71, 85, 105, 255)),
        ("slate", "700") => Some(pack_rgba(51, 65, 85, 255)),
        ("slate", "800") => Some(pack_rgba(30, 41, 59, 255)),
        ("slate", "900") => Some(pack_rgba(15, 23, 42, 255)),
        ("slate", "950") => Some(pack_rgba(2, 6, 23, 255)),
        ("red", "500") => Some(pack_rgba(239, 68, 68, 255)),
        ("green", "500") => Some(pack_rgba(34, 197, 94, 255)),
        ("blue", "500") => Some(pack_rgba(59, 130, 246, 255)),
        ("zinc", "900") => Some(pack_rgba(24, 24, 27, 255)),
        ("neutral", "900") => Some(pack_rgba(23, 23, 23, 255)),
        ("stone", "900") => Some(pack_rgba(28, 25, 23, 255)),
        _ => None,
    }
}

pub fn init() {
    let _ = spec_cache();
}

pub fn parse(classes: &str) -> Spec {
    init();
    if let Ok(cache) = spec_cache().read() {
        if let Some(cached) = cache.get(classes) {
            return cached.clone();
        }
    }

    let spec = parse::parse(classes);

    if let Ok(mut cache) = spec_cache().write() {
        if let Some(cached) = cache.get(classes) {
            return cached.clone();
        }
        cache.insert(classes, spec.clone());
    }

    spec
}

#[cfg(test)]
pub fn apply_hover(spec: &mut Spec, hovered: bool) {
    if !hovered {
        return;
    }
    if let Some(bg) = spec.hover_background {
        spec.background = Some(bg);
    }
    if let Some(tc) = spec.hover_text {
        spec.text = Some(tc);
    }
    if let Some(tc) = spec.hover_text_outline_color {
        spec.text_outline_color = Some(tc);
    }
    if let Some(value) = spec.hover_text_outline_thickness {
        spec.text_outline_thickness = Some(value);
    }
    if let Some(value) = spec.hover_opacity {
        spec.opacity = Some(value);
    }
    if let Some(value) = spec.hover_scale {
        spec.scale = Some(value);
    }
    if let Some(cursor) = spec.hover_cursor {
        spec.cursor = Some(cursor);
    }
    if spec.hover_margin.any() {
        if let Some(v) = spec.hover_margin.left {
            spec.margin.left = Some(v);
        }
        if let Some(v) = spec.hover_margin.right {
            spec.margin.right = Some(v);
        }
        if let Some(v) = spec.hover_margin.top {
            spec.margin.top = Some(v);
        }
        if let Some(v) = spec.hover_margin.bottom {
            spec.margin.bottom = Some(v);
        }
    }
    if spec.hover_padding.any() {
        if let Some(v) = spec.hover_padding.left {
            spec.padding.left = Some(v);
        }
        if let Some(v) = spec.hover_padding.right {
            spec.padding.right = Some(v);
        }
        if let Some(v) = spec.hover_padding.top {
            spec.padding.top = Some(v);
        }
        if let Some(v) = spec.hover_padding.bottom {
            spec.padding.bottom = Some(v);
        }
    }
    if spec.hover_border.any() {
        if let Some(v) = spec.hover_border.left {
            spec.border.left = Some(v);
        }
        if let Some(v) = spec.hover_border.right {
            spec.border.right = Some(v);
        }
        if let Some(v) = spec.hover_border.top {
            spec.border.top = Some(v);
        }
        if let Some(v) = spec.hover_border.bottom {
            spec.border.bottom = Some(v);
        }
    }
    if let Some(color) = spec.hover_border_color {
        spec.border_color = Some(color);
    }
}

#[cfg(test)]
pub fn apply_group_hover(spec: &mut Spec, hovered: bool) {
    if !hovered {
        return;
    }
    if let Some(bg) = spec.group_hover_background {
        spec.background = Some(bg);
    }
    if let Some(tc) = spec.group_hover_text {
        spec.text = Some(tc);
    }
    if let Some(tc) = spec.group_hover_text_outline_color {
        spec.text_outline_color = Some(tc);
    }
    if let Some(value) = spec.group_hover_text_outline_thickness {
        spec.text_outline_thickness = Some(value);
    }
    if let Some(value) = spec.group_hover_opacity {
        spec.opacity = Some(value);
    }
    if let Some(value) = spec.group_hover_scale {
        spec.scale = Some(value);
    }
    if let Some(cursor) = spec.group_hover_cursor {
        spec.cursor = Some(cursor);
    }
    if spec.group_hover_margin.any() {
        if let Some(v) = spec.group_hover_margin.left {
            spec.margin.left = Some(v);
        }
        if let Some(v) = spec.group_hover_margin.right {
            spec.margin.right = Some(v);
        }
        if let Some(v) = spec.group_hover_margin.top {
            spec.margin.top = Some(v);
        }
        if let Some(v) = spec.group_hover_margin.bottom {
            spec.margin.bottom = Some(v);
        }
    }
    if spec.group_hover_padding.any() {
        if let Some(v) = spec.group_hover_padding.left {
            spec.padding.left = Some(v);
        }
        if let Some(v) = spec.group_hover_padding.right {
            spec.padding.right = Some(v);
        }
        if let Some(v) = spec.group_hover_padding.top {
            spec.padding.top = Some(v);
        }
        if let Some(v) = spec.group_hover_padding.bottom {
            spec.padding.bottom = Some(v);
        }
    }
    if spec.group_hover_border.any() {
        if let Some(v) = spec.group_hover_border.left {
            spec.border.left = Some(v);
        }
        if let Some(v) = spec.group_hover_border.right {
            spec.border.right = Some(v);
        }
        if let Some(v) = spec.group_hover_border.top {
            spec.border.top = Some(v);
        }
        if let Some(v) = spec.group_hover_border.bottom {
            spec.border.bottom = Some(v);
        }
    }
    if let Some(color) = spec.group_hover_border_color {
        spec.border_color = Some(color);
    }
}

#[cfg(test)]
pub fn hover_affects_snapshot(spec: &Spec) -> bool {
    spec.hover_margin.any()
        || spec.hover_padding.any()
        || spec.hover_border.any()
        || spec.hover_scale.is_some()
        || spec.group_hover_margin.any()
        || spec.group_hover_padding.any()
        || spec.group_hover_border.any()
        || spec.group_hover_scale.is_some()
}

#[cfg(test)]
pub fn resolve_padding_pixels(
    value: Option<PaddingValue>,
    parent_axis_scaled: f32,
    scale: f32,
    fallback_pixels: f32,
) -> f32 {
    if let Some(padding) = value {
        return match padding {
            PaddingValue::Pixels(px) => px * scale,
            PaddingValue::Percent(fraction) => parent_axis_scaled * fraction,
        };
    }
    fallback_pixels * scale
}

#[cfg(test)]
mod tests {
    use super::{
        apply_group_hover, apply_hover, hover_affects_snapshot, lookup_color_token, parse,
        resolve_padding_pixels, ColorAsk, ColorRef, PaddingValue, Spec, SpecCache, ThemeColorRef,
        ThemeStyle,
    };
    use crate::ui::tailwind::types::{
        AspectDominantAxis, ClipStrategy, Cursor, EasingDirection, EasingStyle, Height,
        UiRuntimeBackground, Width,
    };

    fn assert_approx(lhs: f32, rhs: f32) {
        assert!((lhs - rhs).abs() < 0.0001, "lhs={lhs} rhs={rhs}");
    }

    #[test]
    fn tailwind_percent_size_parsing() {
        let spec = parse("w-[50%] h-[25%]");
        let width = spec.width.expect("width");
        let height = spec.height.expect("height");
        match width {
            Width::Percent(fraction) => assert_approx(0.5, fraction),
            _ => panic!("unexpected width variant"),
        }
        match height {
            Height::Percent(fraction) => assert_approx(0.25, fraction),
            _ => panic!("unexpected height variant"),
        }
    }

    #[test]
    fn tailwind_aspect_ratio_parsing() {
        let width_spec = parse("aspect-[2] aspect-width");
        let width_ratio = width_spec.aspect_ratio.expect("width_ratio");
        assert_approx(2.0, width_ratio);
        let width_axis = width_spec.aspect_dominant_axis.expect("width_axis");
        assert_eq!(AspectDominantAxis::Width, width_axis);

        let height_spec = parse("aspect-[1.5] aspect-height");
        let height_ratio = height_spec.aspect_ratio.expect("height_ratio");
        assert_approx(1.5, height_ratio);
        let height_axis = height_spec.aspect_dominant_axis.expect("height_axis");
        assert_eq!(AspectDominantAxis::Height, height_axis);
    }

    #[test]
    fn tailwind_aspect_ratio_rejects_invalid_values() {
        let zero = parse("aspect-[0]");
        assert_eq!(zero.aspect_ratio, None);

        let negative = parse("aspect-[-2]");
        assert_eq!(negative.aspect_ratio, None);

        let nan = parse("aspect-[nan]");
        assert_eq!(nan.aspect_ratio, None);
    }

    #[test]
    fn tailwind_percent_padding_parsing() {
        let spec = parse("p-[10%] pt-[5%] px-2");
        let left = spec.padding.left.expect("left");
        let right = spec.padding.right.expect("right");
        let top = spec.padding.top.expect("top");
        let bottom = spec.padding.bottom.expect("bottom");

        match left {
            PaddingValue::Pixels(px) => assert_approx(8.0, px),
            _ => panic!("unexpected left variant"),
        }
        match right {
            PaddingValue::Pixels(px) => assert_approx(8.0, px),
            _ => panic!("unexpected right variant"),
        }
        match top {
            PaddingValue::Percent(fraction) => assert_approx(0.05, fraction),
            _ => panic!("unexpected top variant"),
        }
        match bottom {
            PaddingValue::Percent(fraction) => assert_approx(0.10, fraction),
            _ => panic!("unexpected bottom variant"),
        }
    }

    #[test]
    fn tailwind_transition_parsing() {
        let a = parse("transition duration-200 ease-sine ease-in");
        assert!(a.transition.enabled);
        assert!(a.transition.props.layout);
        assert!(a.transition.props.transform);
        assert!(a.transition.props.colors);
        assert!(a.transition.props.opacity);
        assert_eq!(200_000, a.transition.duration_us);
        assert_eq!(EasingStyle::Sine, a.transition.easing_style);
        assert_eq!(EasingDirection::In, a.transition.easing_dir);

        let b = parse("transition-opacity duration-75 ease-linear ease-in-out");
        assert!(b.transition.enabled);
        assert!(b.transition.props.opacity);
        assert!(!b.transition.props.layout);
        assert!(!b.transition.props.transform);
        assert!(!b.transition.props.colors);
        assert_eq!(75_000, b.transition.duration_us);
        assert_eq!(EasingStyle::Linear, b.transition.easing_style);

        let c1 = parse("transition ease-in-out ease-quad");
        let c2 = parse("transition ease-quad ease-in-out");
        assert_eq!(c1.transition.easing_style, c2.transition.easing_style);
        assert_eq!(c1.transition.easing_dir, c2.transition.easing_dir);
    }

    #[test]
    fn tailwind_overflow_scroll_parsing() {
        let a = parse("overflow-scroll");
        assert!(a.scroll_x);
        assert!(a.scroll_y);

        let b = parse("overflow-y-scroll");
        assert!(!b.scroll_x);
        assert!(b.scroll_y);

        let c = parse("overflow-x-scroll");
        assert!(c.scroll_x);
        assert!(!c.scroll_y);
    }

    #[test]
    fn tailwind_clip_strategy_parsing() {
        let a = parse("overflow-hidden rounded-full");
        assert_eq!(Some(true), a.clip_children);
        assert_eq!(None, a.clip_strategy);

        let b = parse("overflow-hidden clip-contents rounded-full");
        assert_eq!(Some(true), b.clip_children);
        assert_eq!(Some(ClipStrategy::Contents), b.clip_strategy);

        let c = parse("clip-contents");
        assert_eq!(None, c.clip_children);
        assert_eq!(Some(ClipStrategy::Contents), c.clip_strategy);

        let d = parse("overflow-clip");
        assert_eq!(Some(true), d.clip_children);
        assert_eq!(Some(ClipStrategy::Rect), d.clip_strategy);

        let e = parse("clip-soft");
        assert_eq!(None, e.clip_children);
        assert_eq!(Some(ClipStrategy::Rect), e.clip_strategy);
    }

    #[test]
    fn tailwind_palette_color_refs() {
        let Some(ColorRef::PalettePacked(expected)) =
            lookup_color_token(ColorAsk::Fill, "slate-900")
        else {
            panic!("slate-900 should resolve to a palette color");
        };
        let spec = parse("bg-slate-900");
        let bg = spec.background.expect("background");
        match bg {
            UiRuntimeBackground::Solid(ColorRef::PalettePacked(packed_value)) => {
                assert_eq!(expected, packed_value)
            }
            _ => panic!("unexpected color ref"),
        }
    }

    #[test]
    fn tailwind_theme_role_refs() {
        let bg_spec = parse("bg-content");
        let bg = bg_spec.background.expect("bg");
        match bg {
            UiRuntimeBackground::Solid(ColorRef::Theme(role)) => {
                assert_eq!(ThemeStyle::Content, role.style);
                assert_eq!(ColorAsk::Fill, role.ask);
            }
            _ => panic!("unexpected bg color"),
        }

        let text_spec = parse("text-content");
        let text = text_spec.text.expect("text");
        match text {
            ColorRef::Theme(role) => {
                assert_eq!(ThemeStyle::Content, role.style);
                assert_eq!(ColorAsk::Text, role.ask);
            }
            _ => panic!("unexpected text color"),
        }
    }

    #[test]
    fn tailwind_hover_theme_role_ask_preserved() {
        let mut spec = parse("hover:bg-content");
        assert_eq!(None, spec.background);
        let hover_bg = spec.hover_background.expect("hover_bg");
        match hover_bg {
            UiRuntimeBackground::Solid(ColorRef::Theme(role)) => {
                assert_eq!(ColorAsk::FillHover, role.ask)
            }
            _ => panic!("unexpected hover bg"),
        }

        apply_hover(&mut spec, true);
        let bg = spec.background.expect("bg");
        match bg {
            UiRuntimeBackground::Solid(ColorRef::Theme(role)) => {
                assert_eq!(ColorAsk::FillHover, role.ask)
            }
            _ => panic!("unexpected applied bg"),
        }
    }

    #[test]
    fn tailwind_hover_cursor_parsing_and_apply() {
        let mut spec = parse("cursor-default hover:cursor-pointer");
        assert_eq!(spec.cursor, Some(Cursor::Arrow));
        assert_eq!(spec.hover_cursor, Some(Cursor::Hand));

        apply_hover(&mut spec, true);
        assert_eq!(spec.cursor, Some(Cursor::Hand));
    }

    #[test]
    fn tailwind_hover_scale_parsing_and_apply() {
        let mut spec = parse("scale-90 hover:scale-110");
        assert!((spec.scale.expect("scale") - 0.9).abs() < 0.0001);
        assert!((spec.hover_scale.expect("hover_scale") - 1.1).abs() < 0.0001);

        apply_hover(&mut spec, true);
        assert!((spec.scale.expect("scale") - 1.1).abs() < 0.0001);
    }

    #[test]
    fn tailwind_group_hover_parsing() {
        let spec = parse(
            "group group-hover:bg-content group-hover:p-2 group-hover:border-content group-hover:opacity-80",
        );
        assert!(spec.group);
        assert!(spec.group_hover_background.is_some());
        assert!(spec.group_hover_padding.any());
        assert!(spec.group_hover_border_color.is_some());
        assert!(hover_affects_snapshot(&spec));
        assert!(spec.group_hover_opacity.is_some());
        assert_eq!(None, spec.background);
        assert!(!spec.padding.any());
    }

    #[test]
    fn tailwind_group_hover_theme_role_ask_preserved() {
        let mut spec = parse("group-hover:bg-content");
        assert_eq!(None, spec.background);
        let group_hover_bg = spec.group_hover_background.expect("group_hover_bg");
        match group_hover_bg {
            UiRuntimeBackground::Solid(ColorRef::Theme(role)) => {
                assert_eq!(ColorAsk::FillHover, role.ask)
            }
            _ => panic!("unexpected group hover bg"),
        }

        apply_group_hover(&mut spec, true);
        let bg = spec.background.expect("bg");
        match bg {
            UiRuntimeBackground::Solid(ColorRef::Theme(role)) => {
                assert_eq!(ColorAsk::FillHover, role.ask)
            }
            _ => panic!("unexpected applied group bg"),
        }
    }

    #[test]
    fn tailwind_group_hover_cursor_parsing_and_apply() {
        let mut spec = parse("cursor-default group-hover:cursor-wait");
        assert_eq!(spec.cursor, Some(Cursor::Arrow));
        assert_eq!(spec.group_hover_cursor, Some(Cursor::Wait));

        apply_group_hover(&mut spec, true);
        assert_eq!(spec.cursor, Some(Cursor::Wait));
    }

    #[test]
    fn tailwind_group_hover_scale_parsing_and_apply() {
        let mut spec = parse("scale-90 group-hover:scale-[1.25]");
        assert!((spec.scale.expect("scale") - 0.9).abs() < 0.0001);
        assert!((spec.group_hover_scale.expect("group_hover_scale") - 1.25).abs() < 0.0001);

        apply_group_hover(&mut spec, true);
        assert!((spec.scale.expect("scale") - 1.25).abs() < 0.0001);
    }

    #[test]
    fn resolve_padding_pixels_applies_units() {
        assert_approx(
            resolve_padding_pixels(Some(PaddingValue::Pixels(4.0)), 100.0, 2.0, 0.0),
            8.0,
        );
        assert_approx(
            resolve_padding_pixels(Some(PaddingValue::Percent(0.1)), 200.0, 1.0, 0.0),
            20.0,
        );
        assert_approx(resolve_padding_pixels(None, 100.0, 2.0, 3.0), 6.0);
    }

    #[test]
    fn spec_cache_insert_hit_evict_and_clear() {
        let mut cache = SpecCache::new(2, 1);
        let mut a = Spec::default();
        a.hidden = true;
        cache.insert("a", a.clone());
        let cached_a = cache.get("a").cloned().expect("cached a");
        assert_eq!(cached_a, a);

        let mut replacement = Spec::default();
        replacement.group = true;
        cache.insert("a", replacement);
        assert_eq!(cache.get("a"), Some(&a));

        let mut b = Spec::default();
        b.group = true;
        cache.insert("b", b);

        let mut c = Spec::default();
        c.scroll_x = true;
        cache.insert("c", c);
        assert!(cache.count() <= 2);

        cache.clear();
        assert_eq!(cache.count(), 0);
    }

    #[test]
    fn spec_cache_evict_batch_respected() {
        let mut cache = SpecCache::new(4, 2);
        for idx in 0..7 {
            cache.insert(&format!("k-{idx}"), Spec::default());
        }
        assert!(cache.count() <= 4);
    }

    #[test]
    fn spec_cache_fifo_evicts_oldest_entries_first() {
        let mut cache = SpecCache::new(3, 1);
        cache.insert("a", Spec::default());
        cache.insert("b", Spec::default());
        cache.insert("c", Spec::default());
        cache.insert("d", Spec::default());

        assert!(cache.get("a").is_none());
        assert!(cache.get("b").is_some());
        assert!(cache.get("c").is_some());
        assert!(cache.get("d").is_some());
    }

    #[test]
    fn spec_cache_fifo_evict_batch_is_deterministic() {
        let mut cache = SpecCache::new(3, 2);
        cache.insert("a", Spec::default());
        cache.insert("b", Spec::default());
        cache.insert("c", Spec::default());
        cache.insert("d", Spec::default());

        assert!(cache.get("a").is_none());
        assert!(cache.get("b").is_none());
        assert!(cache.get("c").is_some());
        assert!(cache.get("d").is_some());

        cache.insert("e", Spec::default());
        assert!(cache.get("c").is_some());
        assert!(cache.get("d").is_some());
        assert!(cache.get("e").is_some());
    }

    #[test]
    fn parse_cache_hit_returns_same_spec() {
        let a = parse("w-[50%] h-[25%] bg-content");
        let b = parse("w-[50%] h-[25%] bg-content");
        assert_eq!(a, b);
    }

    #[test]
    fn malformed_tokens_do_not_panic_or_overwrite_state() {
        let spec = parse("w-[] h-[nan] text-[bad%] hover:unknown group-hover:unknown");
        assert_eq!(spec.width, None);
        assert_eq!(spec.height, None);
        assert_eq!(spec.font_scale, None);
    }

    #[test]
    fn parser_precedence_last_token_wins_for_same_field() {
        let spec = parse("text-left text-right");
        assert_eq!(
            spec.text_align,
            Some(crate::ui::tailwind::types::TextAlign::Right)
        );
    }

    #[test]
    fn theme_ask_preserved_for_group_hover_text_outline() {
        let mut spec = parse("group-hover:text-outline-content");
        assert!(spec.group_hover_text_outline_color.is_some());
        apply_group_hover(&mut spec, true);
        let outline = spec.text_outline_color.expect("text_outline_color");
        match outline {
            ColorRef::Theme(ThemeColorRef { ask, .. }) => assert_eq!(ask, ColorAsk::TextHover),
            _ => panic!("unexpected outline color"),
        }
    }

    #[test]
    fn text_shadow_tokens_resolve_static_shadow_style() {
        let spec = parse("text-shadow text-shadow-x-[-2] text-shadow-y-[3px] text-shadow-opacity-50 text-shadow-black");
        let shadow = spec.text_shadow.expect("text shadow");
        assert_eq!(shadow.x, -2.0);
        assert_eq!(shadow.y, 3.0);
        assert_eq!(shadow.opacity, 0.5);
        assert!(matches!(shadow.color, ColorRef::PalettePacked(_)));
    }
}
