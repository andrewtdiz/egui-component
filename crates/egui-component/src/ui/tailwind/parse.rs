use crate::ui::tailwind::parse_color_typography as color;
use crate::ui::tailwind::parse_layout as layout;
use crate::ui::tailwind::tokens::{Z_INDEX_DEFAULT, Z_LAYER_TOKENS};
use crate::ui::tailwind::types::{
    AlignContent, AlignItems, AlignSelf, Animation, ClipStrategy, Cursor, Direction,
    EasingDirection, EasingStyle, FlexWrap, JustifyContent, Position, Spec, SurfaceShadow,
    TextAlign, TextAlignY,
};

#[derive(Clone, Copy)]
enum LiteralKind {
    FlexDisplay,
    FlexRow,
    FlexCol,
    Absolute,
    JustifyStart,
    JustifyCenter,
    JustifyEnd,
    JustifyBetween,
    JustifyAround,
    AlignItemsStart,
    AlignItemsCenter,
    AlignItemsEnd,
    AlignItemsStretch,
    AlignContentStart,
    AlignContentCenter,
    AlignContentEnd,
    Hidden,
    OverflowHidden,
    OverflowClip,
    ClipContents,
    ClipRect,
    OverflowScroll,
    OverflowXScroll,
    OverflowYScroll,
    TextLeft,
    TextCenter,
    TextRight,
    TextTop,
    TextMiddle,
    TextBottom,
    TextNowrap,
    BreakWords,
    Group,
    FlexWrap,
    FlexNoWrap,
    FlexWrapReverse,
    ContentBetween,
    ContentAround,
    SelfStart,
    SelfCenter,
    SelfEnd,
    SelfAuto,
    SelfStretch,
}

const LITERAL_RULES: &[(&str, LiteralKind)] = &[
    ("flex", LiteralKind::FlexDisplay),
    ("flex-row", LiteralKind::FlexRow),
    ("flex-col", LiteralKind::FlexCol),
    ("absolute", LiteralKind::Absolute),
    ("justify-start", LiteralKind::JustifyStart),
    ("justify-center", LiteralKind::JustifyCenter),
    ("justify-end", LiteralKind::JustifyEnd),
    ("justify-between", LiteralKind::JustifyBetween),
    ("justify-around", LiteralKind::JustifyAround),
    ("items-start", LiteralKind::AlignItemsStart),
    ("items-center", LiteralKind::AlignItemsCenter),
    ("items-end", LiteralKind::AlignItemsEnd),
    ("items-stretch", LiteralKind::AlignItemsStretch),
    ("content-start", LiteralKind::AlignContentStart),
    ("content-center", LiteralKind::AlignContentCenter),
    ("content-end", LiteralKind::AlignContentEnd),
    ("content-between", LiteralKind::ContentBetween),
    ("content-around", LiteralKind::ContentAround),
    ("flex-wrap", LiteralKind::FlexWrap),
    ("flex-nowrap", LiteralKind::FlexNoWrap),
    ("flex-wrap-reverse", LiteralKind::FlexWrapReverse),
    ("self-start", LiteralKind::SelfStart),
    ("self-center", LiteralKind::SelfCenter),
    ("self-end", LiteralKind::SelfEnd),
    ("self-auto", LiteralKind::SelfAuto),
    ("self-stretch", LiteralKind::SelfStretch),
    ("hidden", LiteralKind::Hidden),
    ("overflow-hidden", LiteralKind::OverflowHidden),
    ("overflow-clip", LiteralKind::OverflowClip),
    ("clip-contents", LiteralKind::ClipContents),
    ("clip-rect", LiteralKind::ClipRect),
    ("clip-hard", LiteralKind::ClipRect),
    ("clip-hidden", LiteralKind::ClipRect),
    ("clip-soft", LiteralKind::ClipRect),
    ("clip-clip", LiteralKind::ClipRect),
    ("overflow-scroll", LiteralKind::OverflowScroll),
    ("overflow-x-scroll", LiteralKind::OverflowXScroll),
    ("overflow-y-scroll", LiteralKind::OverflowYScroll),
    ("text-left", LiteralKind::TextLeft),
    ("text-center", LiteralKind::TextCenter),
    ("text-right", LiteralKind::TextRight),
    ("text-top", LiteralKind::TextTop),
    ("text-middle", LiteralKind::TextMiddle),
    ("text-bottom", LiteralKind::TextBottom),
    ("text-nowrap", LiteralKind::TextNowrap),
    ("break-words", LiteralKind::BreakWords),
    ("group", LiteralKind::Group),
];

pub fn parse(classes: &str) -> Spec {
    let mut spec = Spec::default();
    for token in classes.split_whitespace() {
        if token.is_empty() {
            continue;
        }
        let _ = apply_token(&mut spec, token);
    }
    spec
}

pub(crate) fn token_spec(token: &str) -> Option<Spec> {
    let mut spec = Spec::default();
    apply_token(&mut spec, token).then_some(spec)
}

pub(crate) fn apply_token(spec: &mut Spec, token: &str) -> bool {
    handle_hover(spec, token)
        || handle_group_hover(spec, token)
        || handle_literal(spec, token)
        || handle_anchor(spec, token)
        || layout::handle_flex(spec, token)
        || layout::handle_grid(spec, token)
        || layout::handle_spacing(spec, token)
        || layout::handle_inset(spec, token)
        || layout::handle_gap(spec, token)
        || layout::handle_scale(spec, token)
        || layout::handle_translate(spec, token)
        || layout::handle_rotation(spec, token)
        || layout::handle_border(spec, token)
        || layout::handle_rounded(spec, token)
        || handle_shadow(spec, token)
        || color::handle_typography(spec, token)
        || color::handle_font_token(spec, token)
        || color::handle_text_tracking(spec, token)
        || color::handle_text_leading(spec, token)
        || color::handle_line_clamp(spec, token)
        || color::handle_opacity(spec, token)
        || handle_z_index(spec, token)
        || handle_cursor(spec, token)
        || handle_animation(spec, token)
        || handle_transition(spec, token)
        || handle_duration(spec, token)
        || handle_ease(spec, token)
        || handle_prefixed(spec, token)
}

fn handle_hover(spec: &mut Spec, token: &str) -> bool {
    let Some(inner) = token.strip_prefix("hover:") else {
        return false;
    };
    if inner.is_empty() {
        return false;
    }
    color::handle_hover_opacity(spec, inner)
        || layout::handle_hover_spacing(spec, inner)
        || layout::handle_hover_border(spec, inner)
        || layout::handle_hover_scale(spec, inner)
        || handle_hover_cursor(spec, inner)
        || handle_hover_prefixed(spec, inner)
}

fn handle_group_hover(spec: &mut Spec, token: &str) -> bool {
    let Some(inner) = token.strip_prefix("group-hover:") else {
        return false;
    };
    if inner.is_empty() {
        return false;
    }
    color::handle_group_hover_opacity(spec, inner)
        || layout::handle_group_hover_spacing(spec, inner)
        || layout::handle_group_hover_border(spec, inner)
        || layout::handle_group_hover_scale(spec, inner)
        || handle_group_hover_cursor(spec, inner)
        || handle_group_hover_prefixed(spec, inner)
}

fn handle_hover_prefixed(spec: &mut Spec, token: &str) -> bool {
    if let Some(suffix) = token.strip_prefix("bg-") {
        return color::handle_hover_background(spec, suffix);
    }
    if let Some(suffix) = token.strip_prefix("text-") {
        return color::handle_hover_text(spec, suffix);
    }
    false
}

fn handle_group_hover_prefixed(spec: &mut Spec, token: &str) -> bool {
    if let Some(suffix) = token.strip_prefix("bg-") {
        return color::handle_group_hover_background(spec, suffix);
    }
    if let Some(suffix) = token.strip_prefix("text-") {
        return color::handle_group_hover_text(spec, suffix);
    }
    false
}

fn handle_hover_cursor(spec: &mut Spec, token: &str) -> bool {
    let Some(name) = token.strip_prefix("cursor-") else {
        return false;
    };
    let Some(cursor) = parse_cursor_name(name) else {
        return false;
    };
    spec.hover_cursor = Some(cursor);
    true
}

fn handle_group_hover_cursor(spec: &mut Spec, token: &str) -> bool {
    let Some(name) = token.strip_prefix("cursor-") else {
        return false;
    };
    let Some(cursor) = parse_cursor_name(name) else {
        return false;
    };
    spec.group_hover_cursor = Some(cursor);
    true
}

fn handle_literal(spec: &mut Spec, token: &str) -> bool {
    for (rule_token, kind) in LITERAL_RULES {
        if token == *rule_token {
            apply_literal(spec, *kind);
            return true;
        }
    }
    false
}

fn handle_anchor(spec: &mut Spec, token: &str) -> bool {
    let Some(anchor) = parse_anchor_token(token) else {
        return false;
    };
    spec.layout_anchor = Some(anchor);
    true
}

fn parse_anchor_token(token: &str) -> Option<[f32; 2]> {
    match token {
        "anchor-top-left" => Some([0.0, 0.0]),
        "anchor-top" => Some([0.5, 0.0]),
        "anchor-top-right" => Some([1.0, 0.0]),
        "anchor-left" => Some([0.0, 0.5]),
        "anchor-center" => Some([0.5, 0.5]),
        "anchor-right" => Some([1.0, 0.5]),
        "anchor-bottom-left" => Some([0.0, 1.0]),
        "anchor-bottom" => Some([0.5, 1.0]),
        "anchor-bottom-right" => Some([1.0, 1.0]),
        _ => None,
    }
}

fn handle_prefixed(spec: &mut Spec, token: &str) -> bool {
    if let Some(suffix) = token.strip_prefix("bg-") {
        if !suffix.is_empty() {
            return color::handle_background(spec, suffix);
        }
    }
    if let Some(suffix) = token.strip_prefix("text-") {
        if !suffix.is_empty() {
            return color::handle_text(spec, suffix);
        }
    }
    if let Some(suffix) = token.strip_prefix("w-") {
        if !suffix.is_empty() {
            return layout::handle_width(spec, suffix);
        }
    }
    if let Some(suffix) = token.strip_prefix("h-") {
        if !suffix.is_empty() {
            return layout::handle_height(spec, suffix);
        }
    }
    if let Some(suffix) = token.strip_prefix("min-w-") {
        if !suffix.is_empty() {
            return layout::handle_min_width(spec, suffix);
        }
    }
    if let Some(suffix) = token.strip_prefix("min-h-") {
        if !suffix.is_empty() {
            return layout::handle_min_height(spec, suffix);
        }
    }
    if let Some(suffix) = token.strip_prefix("max-w-") {
        if !suffix.is_empty() {
            return layout::handle_max_width(spec, suffix);
        }
    }
    if let Some(suffix) = token.strip_prefix("max-h-") {
        if !suffix.is_empty() {
            return layout::handle_max_height(spec, suffix);
        }
    }
    if let Some(suffix) = token.strip_prefix("aspect-") {
        if !suffix.is_empty() {
            return layout::handle_aspect(spec, suffix);
        }
    }
    false
}

fn handle_shadow(spec: &mut Spec, token: &str) -> bool {
    spec.shadow = Some(match token {
        "shadow-sm" => SurfaceShadow::Sm,
        "shadow-md" => SurfaceShadow::Md,
        "shadow-lg" => SurfaceShadow::Lg,
        _ => return false,
    });
    true
}

fn apply_literal(spec: &mut Spec, kind: LiteralKind) {
    match kind {
        LiteralKind::FlexDisplay => spec.is_flex = true,
        LiteralKind::FlexRow => spec.direction = Some(Direction::Horizontal),
        LiteralKind::FlexCol => spec.direction = Some(Direction::Vertical),
        LiteralKind::Absolute => spec.position = Some(Position::Absolute),
        LiteralKind::JustifyStart => spec.justify = Some(JustifyContent::Start),
        LiteralKind::JustifyCenter => spec.justify = Some(JustifyContent::Center),
        LiteralKind::JustifyEnd => spec.justify = Some(JustifyContent::End),
        LiteralKind::JustifyBetween => spec.justify = Some(JustifyContent::Between),
        LiteralKind::JustifyAround => spec.justify = Some(JustifyContent::Around),
        LiteralKind::AlignItemsStart => spec.align_items = Some(AlignItems::Start),
        LiteralKind::AlignItemsCenter => spec.align_items = Some(AlignItems::Center),
        LiteralKind::AlignItemsEnd => spec.align_items = Some(AlignItems::End),
        LiteralKind::AlignItemsStretch => spec.align_items = Some(AlignItems::Stretch),
        LiteralKind::AlignContentStart => spec.align_content = Some(AlignContent::Start),
        LiteralKind::AlignContentCenter => spec.align_content = Some(AlignContent::Center),
        LiteralKind::AlignContentEnd => spec.align_content = Some(AlignContent::End),
        LiteralKind::ContentBetween => spec.align_content = Some(AlignContent::Between),
        LiteralKind::ContentAround => spec.align_content = Some(AlignContent::Around),
        LiteralKind::FlexWrap => spec.flex_wrap = Some(FlexWrap::Wrap),
        LiteralKind::FlexNoWrap => spec.flex_wrap = Some(FlexWrap::NoWrap),
        LiteralKind::FlexWrapReverse => spec.flex_wrap = Some(FlexWrap::WrapReverse),
        LiteralKind::SelfStart => spec.align_self = Some(AlignSelf::Start),
        LiteralKind::SelfCenter => spec.align_self = Some(AlignSelf::Center),
        LiteralKind::SelfEnd => spec.align_self = Some(AlignSelf::End),
        LiteralKind::SelfAuto => spec.align_self = Some(AlignSelf::Auto),
        LiteralKind::SelfStretch => spec.align_self = Some(AlignSelf::Stretch),
        LiteralKind::Hidden => spec.hidden = true,
        LiteralKind::OverflowHidden => spec.clip_children = Some(true),
        LiteralKind::OverflowClip => {
            spec.clip_children = Some(true);
            spec.clip_strategy = Some(ClipStrategy::Rect);
        }
        LiteralKind::ClipContents => spec.clip_strategy = Some(ClipStrategy::Contents),
        LiteralKind::ClipRect => spec.clip_strategy = Some(ClipStrategy::Rect),
        LiteralKind::OverflowScroll => {
            spec.scroll_x = true;
            spec.scroll_y = true;
        }
        LiteralKind::OverflowXScroll => spec.scroll_x = true,
        LiteralKind::OverflowYScroll => spec.scroll_y = true,
        LiteralKind::TextLeft => spec.text_align = Some(TextAlign::Left),
        LiteralKind::TextCenter => spec.text_align = Some(TextAlign::Center),
        LiteralKind::TextRight => spec.text_align = Some(TextAlign::Right),
        LiteralKind::TextTop => spec.text_align_y = Some(TextAlignY::Top),
        LiteralKind::TextMiddle => spec.text_align_y = Some(TextAlignY::Middle),
        LiteralKind::TextBottom => spec.text_align_y = Some(TextAlignY::Bottom),
        LiteralKind::TextNowrap => spec.text_wrap = false,
        LiteralKind::BreakWords => spec.break_words = true,
        LiteralKind::Group => spec.group = true,
    }
}

fn handle_z_index(spec: &mut Spec, token: &str) -> bool {
    let (negative, suffix) = if let Some(suffix) = token.strip_prefix("-z-") {
        (true, suffix)
    } else if let Some(suffix) = token.strip_prefix("z-") {
        (false, suffix)
    } else {
        return false;
    };

    if suffix.is_empty() {
        return false;
    }
    if suffix == "auto" {
        spec.z_index = Z_INDEX_DEFAULT;
        return true;
    }

    if let Some(layer_value) = lookup_z_layer(suffix) {
        spec.z_index = if negative { -layer_value } else { layer_value };
        return true;
    }

    if suffix.starts_with('[') && suffix.ends_with(']') {
        let inner = &suffix[1..suffix.len() - 1];
        if inner.is_empty() {
            return false;
        }
        let Ok(mut value) = inner.parse::<i16>() else {
            return false;
        };
        if negative && value > 0 {
            value = -value;
        }
        spec.z_index = value;
        return true;
    }

    let Ok(mut value) = suffix.parse::<i16>() else {
        return false;
    };
    if negative {
        value = -value;
    }
    spec.z_index = value;
    true
}

fn lookup_z_layer(name: &str) -> Option<i16> {
    for layer in Z_LAYER_TOKENS {
        if name == layer.token {
            return Some(layer.value);
        }
    }
    None
}

fn handle_cursor(spec: &mut Spec, token: &str) -> bool {
    let Some(name) = token.strip_prefix("cursor-") else {
        return false;
    };
    let Some(cursor) = parse_cursor_name(name) else {
        return false;
    };
    spec.cursor = Some(cursor);
    true
}

fn parse_cursor_name(name: &str) -> Option<Cursor> {
    match name {
        "auto" | "default" => Some(Cursor::Arrow),
        "pointer" => Some(Cursor::Hand),
        "text" => Some(Cursor::Ibeam),
        "move" => Some(Cursor::ArrowAll),
        "wait" => Some(Cursor::Wait),
        "progress" => Some(Cursor::WaitArrow),
        "crosshair" => Some(Cursor::Crosshair),
        "not-allowed" => Some(Cursor::Bad),
        "none" => Some(Cursor::Hidden),
        "grab" | "grabbing" => Some(Cursor::Hand),
        "col-resize" | "e-resize" | "w-resize" => Some(Cursor::ArrowWE),
        "row-resize" | "n-resize" | "s-resize" => Some(Cursor::ArrowNS),
        "ne-resize" | "sw-resize" => Some(Cursor::ArrowNESW),
        "nw-resize" | "se-resize" => Some(Cursor::ArrowNWSE),
        _ => None,
    }
}

fn handle_transition(spec: &mut Spec, token: &str) -> bool {
    match token {
        "transition" => {
            spec.transition.enabled = true;
            spec.transition.props.layout = true;
            spec.transition.props.transform = true;
            spec.transition.props.colors = true;
            spec.transition.props.opacity = true;
            true
        }
        "transition-none" => {
            spec.transition = Default::default();
            true
        }
        "transition-layout" => {
            spec.transition.enabled = true;
            spec.transition.props = Default::default();
            spec.transition.props.layout = true;
            true
        }
        "transition-transform" => {
            spec.transition.enabled = true;
            spec.transition.props = Default::default();
            spec.transition.props.transform = true;
            true
        }
        "transition-colors" => {
            spec.transition.enabled = true;
            spec.transition.props = Default::default();
            spec.transition.props.colors = true;
            true
        }
        "transition-opacity" => {
            spec.transition.enabled = true;
            spec.transition.props = Default::default();
            spec.transition.props.opacity = true;
            true
        }
        _ => false,
    }
}

fn handle_animation(spec: &mut Spec, token: &str) -> bool {
    match token {
        "animate-spin" => {
            spec.animation = Some(Animation::Spin);
            true
        }
        _ => false,
    }
}

fn handle_duration(spec: &mut Spec, token: &str) -> bool {
    let Some(suffix) = token.strip_prefix("duration-") else {
        return false;
    };
    if suffix.is_empty() {
        return false;
    }
    let Ok(ms) = suffix.parse::<i32>() else {
        return false;
    };
    let clamped_ms = ms.clamp(0, 10_000);
    spec.transition.duration_us = clamped_ms * 1000;
    true
}

fn handle_ease(spec: &mut Spec, token: &str) -> bool {
    match token {
        "ease-linear" => {
            spec.transition.easing_style = EasingStyle::Linear;
            true
        }
        "ease-in" => {
            spec.transition.easing_dir = EasingDirection::In;
            true
        }
        "ease-out" => {
            spec.transition.easing_dir = EasingDirection::Out;
            true
        }
        "ease-in-out" => {
            spec.transition.easing_dir = EasingDirection::InOut;
            true
        }
        "ease-sine" => {
            spec.transition.easing_style = EasingStyle::Sine;
            true
        }
        "ease-quad" => {
            spec.transition.easing_style = EasingStyle::Quad;
            true
        }
        "ease-cubic" => {
            spec.transition.easing_style = EasingStyle::Cubic;
            true
        }
        "ease-quart" => {
            spec.transition.easing_style = EasingStyle::Quart;
            true
        }
        "ease-quint" => {
            spec.transition.easing_style = EasingStyle::Quint;
            true
        }
        "ease-expo" => {
            spec.transition.easing_style = EasingStyle::Expo;
            true
        }
        "ease-circ" => {
            spec.transition.easing_style = EasingStyle::Circ;
            true
        }
        "ease-back" => {
            spec.transition.easing_style = EasingStyle::Back;
            true
        }
        "ease-elastic" => {
            spec.transition.easing_style = EasingStyle::Elastic;
            true
        }
        "ease-bounce" => {
            spec.transition.easing_style = EasingStyle::Bounce;
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse, token_spec};
    use crate::ui::tailwind::types::{
        AlignContent, AlignItems, AlignSelf, ClipStrategy, ColorAsk, ColorRef, Cursor, FlexBasis,
        FlexWrap, Height, TextUnit, ThemeStyle, UiRuntimeBackground, Width,
    };

    #[test]
    fn parser_order_last_token_wins() {
        let spec = parse("opacity-10 opacity-80");
        assert_eq!(spec.opacity, Some(0.8));
    }

    #[test]
    fn malformed_tokens_do_not_panic() {
        let spec = parse("hover: hover:text-outline-[] border-[x] z-[x] text-[nope%]");
        assert_eq!(spec.hover_background, None);
        assert_eq!(spec.border_color, None);
        assert_eq!(spec.opacity, None);
    }

    #[test]
    fn parser_reads_layout_anchor_tokens() {
        let top_left = parse("anchor-top-left");
        assert_eq!(top_left.layout_anchor, Some([0.0, 0.0]));

        let center = parse("anchor-center");
        assert_eq!(center.layout_anchor, Some([0.5, 0.5]));

        let bottom_right = parse("anchor-bottom-right");
        assert_eq!(bottom_right.layout_anchor, Some([1.0, 1.0]));
    }

    #[test]
    fn parser_ignores_invalid_anchor_tokens() {
        let spec = parse("anchor-middle-left");
        assert_eq!(spec.layout_anchor, None);
    }

    #[test]
    fn parser_anchor_last_token_wins() {
        let spec = parse("anchor-top-left anchor-bottom-right");
        assert_eq!(spec.layout_anchor, Some([1.0, 1.0]));
    }

    #[test]
    fn hover_prefix_rejects_unknown_inner_tokens() {
        let spec = parse("hover:unknown-token bg-content");
        assert_eq!(
            spec.background,
            Some(UiRuntimeBackground::Solid(ColorRef::Theme(
                crate::ui::tailwind::types::ThemeColorRef {
                    style: ThemeStyle::Content,
                    ask: ColorAsk::Fill,
                },
            )))
        );
        assert_eq!(token_spec("hover:unknown-token"), None);
        assert_eq!(token_spec("group-hover:unknown-token"), None);
    }

    #[test]
    fn parser_allows_flex_wrap_and_wrap_reverse() {
        let spec = parse("flex-wrap flex-wrap-reverse");
        assert_eq!(spec.flex_wrap, Some(FlexWrap::WrapReverse));
    }

    #[test]
    fn parser_preserves_flex_no_wrap() {
        let spec = parse("flex-nowrap");
        assert_eq!(spec.flex_wrap, Some(FlexWrap::NoWrap));
    }

    #[test]
    fn parser_reads_arbitrary_gap_spacing_tokens() {
        let spec = parse("gap-[10px] m-[12px]");
        assert_eq!(spec.gap_col, Some(10.0));
        assert_eq!(spec.gap_row, Some(10.0));
        assert_eq!(spec.margin.top, Some(12.0));
    }

    #[test]
    fn parser_reads_flex_grow_tokens() {
        let spec = parse("grow grow-0 grow-[2]");
        assert_eq!(spec.flex_grow, Some(2.0));
    }

    #[test]
    fn parser_reads_flex_shrink_tokens() {
        let spec = parse("shrink shrink-0 shrink-[3]");
        assert_eq!(spec.flex_shrink, Some(3.0));
    }

    #[test]
    fn parser_reads_flex_basis_tokens() {
        let spec = parse("basis-[120] basis-[50%] basis-full basis-auto");
        assert_eq!(spec.flex_basis, Some(FlexBasis::Auto));
    }

    #[test]
    fn parser_reads_grid_tokens() {
        let spec = parse("grid grid-cols-4 justify-around content-between");
        assert!(spec.is_grid);
        assert_eq!(spec.grid_cols, Some(4));
        assert_eq!(spec.grid_rows, None);
        assert_eq!(
            spec.justify,
            Some(crate::ui::tailwind::types::JustifyContent::Around)
        );
        assert_eq!(spec.align_content, Some(AlignContent::Between));

        let bracket = parse("grid-cols-4 grid-rows-[3]");
        assert!(bracket.is_grid);
        assert_eq!(bracket.grid_rows, Some(3));
        assert_eq!(bracket.grid_cols, Some(4));
    }

    #[test]
    fn parser_reads_min_and_max_size_tokens() {
        let spec = parse("min-w-44 min-h-[25%] max-w-full max-h-32");
        assert_eq!(spec.min_width, Some(Width::Pixels(176.0)));
        assert_eq!(spec.min_height, Some(Height::Percent(0.25)));
        assert_eq!(spec.max_width, Some(Width::Full));
        assert_eq!(spec.max_height, Some(Height::Pixels(128.0)));
    }

    #[test]
    fn parser_reads_segmented_radius_tokens() {
        let spec = parse("rounded-lg rounded-l-none rounded-tr-xl");
        assert_eq!(spec.corner_radii.nw, Some(0.0));
        assert_eq!(spec.corner_radii.sw, Some(0.0));
        assert_eq!(spec.corner_radii.ne, Some(12.0));
        assert_eq!(spec.corner_radii.se, Some(8.0));
    }

    #[test]
    fn parser_reads_plain_segmented_radius_tokens() {
        let spec = parse("rounded-l rounded-tr rounded-br-none");
        assert_eq!(spec.corner_radii.nw, Some(4.0));
        assert_eq!(spec.corner_radii.sw, Some(4.0));
        assert_eq!(spec.corner_radii.ne, Some(4.0));
        assert_eq!(spec.corner_radii.se, Some(0.0));
    }

    #[test]
    fn parser_reads_items_stretch() {
        let spec = parse("items-stretch");
        assert_eq!(spec.align_items, Some(AlignItems::Stretch));
    }
    #[test]
    fn parser_reads_align_self_tokens() {
        let spec = parse("self-start self-center self-end self-auto self-stretch");
        assert_eq!(spec.align_self, Some(AlignSelf::Stretch));
    }

    #[test]
    fn parser_reads_content_tokens() {
        let spec = parse("content-between content-around");
        assert_eq!(spec.align_content, Some(AlignContent::Around));
    }

    #[test]
    fn parser_reads_clip_strategy_tokens() {
        let spec = parse("overflow-clip clip-rect clip-contents");
        assert_eq!(spec.clip_children, Some(true));
        assert_eq!(spec.clip_strategy, Some(ClipStrategy::Contents));
    }

    #[test]
    fn parser_reads_clip_strategy_alias_tokens() {
        let spec = parse("clip-soft clip-hidden");
        assert_eq!(spec.clip_strategy, Some(ClipStrategy::Rect));
    }

    #[test]
    fn parser_reads_hover_cursor_tokens() {
        let spec = parse("hover:cursor-pointer group-hover:cursor-wait");
        assert_eq!(spec.hover_cursor, Some(Cursor::Hand));
        assert_eq!(spec.group_hover_cursor, Some(Cursor::Wait));
    }

    #[test]
    fn parser_reads_hover_scale_tokens() {
        let spec = parse("hover:scale-110 group-hover:scale-[1.25]");
        assert!((spec.hover_scale.expect("hover_scale") - 1.1).abs() < 0.0001);
        assert!((spec.group_hover_scale.expect("group_hover_scale") - 1.25).abs() < 0.0001);
    }

    #[test]
    fn parser_reads_translate_tokens() {
        let spec = parse("translate-x-4 translate-y-[12] -translate-x-2 translate-y-[-6]");
        let translate = spec.translate.expect("translate");
        assert!((translate.x + 8.0).abs() < 0.0001);
        assert!((translate.y + 6.0).abs() < 0.0001);
    }

    #[test]
    fn parser_reads_rotation_tokens() {
        let spec = parse("rotate-90 -rotate-45 rotate-[15deg]");
        assert!((spec.rotation_degrees.expect("rotation") - 15.0).abs() < 0.0001);
    }

    #[test]
    fn parser_reads_spin_animation_token() {
        let spec = parse("animate-spin");
        assert_eq!(
            spec.animation,
            Some(crate::ui::tailwind::types::Animation::Spin)
        );
    }

    #[test]
    fn parser_reads_text_spacing_and_clamp_tokens() {
        let spec = parse("tracking-[1px] leading-[120%] line-clamp-3");
        assert_eq!(spec.text_tracking, Some(TextUnit::Pixels(1.0)));
        assert_eq!(spec.text_leading, Some(TextUnit::FontPercent(1.2)));
        assert_eq!(spec.text_max_lines, Some(3));
    }
}
