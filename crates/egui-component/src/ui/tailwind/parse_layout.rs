use crate::ui::tailwind::lookup_color_token;
use crate::ui::tailwind::tokens::{
    BORDER_WIDTH_DEFAULT, DIMENSION_UNIT, RADIUS_TOKENS, SPACING_UNIT,
};
use crate::ui::tailwind::types::{
    AspectDominantAxis, FlexBasis, Height, Inset, PaddingValue, SideTarget, Spec, Width,
};

enum SizeAxisValue {
    Pixels(f32),
    Percent(f32),
}

pub fn handle_width(spec: &mut Spec, suffix: &str) -> bool {
    let Some(width) = parse_width_value(suffix) else {
        return false;
    };
    spec.width = Some(width);
    true
}

pub fn handle_height(spec: &mut Spec, suffix: &str) -> bool {
    let Some(height) = parse_height_value(suffix) else {
        return false;
    };
    spec.height = Some(height);
    true
}

pub fn handle_min_width(spec: &mut Spec, suffix: &str) -> bool {
    let Some(width) = parse_width_value(suffix) else {
        return false;
    };
    spec.min_width = Some(width);
    true
}

pub fn handle_min_height(spec: &mut Spec, suffix: &str) -> bool {
    let Some(height) = parse_height_value(suffix) else {
        return false;
    };
    spec.min_height = Some(height);
    true
}

pub fn handle_max_width(spec: &mut Spec, suffix: &str) -> bool {
    let Some(width) = parse_width_value(suffix) else {
        return false;
    };
    spec.max_width = Some(width);
    true
}

pub fn handle_max_height(spec: &mut Spec, suffix: &str) -> bool {
    let Some(height) = parse_height_value(suffix) else {
        return false;
    };
    spec.max_height = Some(height);
    true
}

pub fn handle_aspect(spec: &mut Spec, suffix: &str) -> bool {
    if suffix == "width" {
        spec.aspect_dominant_axis = Some(AspectDominantAxis::Width);
        return true;
    }
    if suffix == "height" {
        spec.aspect_dominant_axis = Some(AspectDominantAxis::Height);
        return true;
    }
    if let Some(value) = parse_aspect_ratio_value(suffix) {
        spec.aspect_ratio = Some(value);
        return true;
    }
    false
}

pub fn handle_gap(spec: &mut Spec, token: &str) -> bool {
    let Some(mut suffix) = token.strip_prefix("gap-") else {
        return false;
    };
    if suffix.is_empty() {
        return false;
    }

    if suffix.starts_with("x-") {
        suffix = &suffix[2..];
        let Some(value) = parse_spacing_value(suffix) else {
            return false;
        };
        spec.gap_col = Some(value);
        return true;
    }

    if suffix.starts_with("y-") {
        suffix = &suffix[2..];
        let Some(value) = parse_spacing_value(suffix) else {
            return false;
        };
        spec.gap_row = Some(value);
        return true;
    }

    let Some(value) = parse_spacing_value(suffix) else {
        return false;
    };
    spec.gap_col = Some(value);
    spec.gap_row = Some(value);
    true
}

pub fn handle_grid(spec: &mut Spec, token: &str) -> bool {
    if token == "grid" {
        spec.is_grid = true;
        return true;
    }

    if let Some(value) = token.strip_prefix("grid-cols-") {
        let Some(count) = parse_grid_track_count(value) else {
            return false;
        };
        spec.is_grid = true;
        spec.grid_cols = Some(count);
        return true;
    }

    if let Some(value) = token.strip_prefix("grid-rows-") {
        let Some(count) = parse_grid_track_count(value) else {
            return false;
        };
        spec.is_grid = true;
        spec.grid_rows = Some(count);
        return true;
    }

    false
}

pub fn handle_flex(spec: &mut Spec, token: &str) -> bool {
    handle_flex_grow(spec, token)
        || handle_flex_shrink(spec, token)
        || handle_flex_basis(spec, token)
}

fn handle_flex_grow(spec: &mut Spec, token: &str) -> bool {
    if token == "grow" {
        spec.flex_grow = Some(1.0);
        return true;
    }

    let Some(mut suffix) = token.strip_prefix("grow-") else {
        return false;
    };
    if suffix.is_empty() {
        return false;
    }
    if suffix == "0" {
        spec.flex_grow = Some(0.0);
        return true;
    }

    if suffix.starts_with('[') && suffix.ends_with(']') {
        suffix = &suffix[1..suffix.len() - 1];
        if suffix.is_empty() {
            return false;
        }
        let Some(value) = parse_float(suffix) else {
            return false;
        };
        if value < 0.0 || !value.is_finite() {
            return false;
        }
        spec.flex_grow = Some(value);
        return true;
    }

    let Some(value) = parse_float(suffix) else {
        return false;
    };
    if value < 0.0 || !value.is_finite() {
        return false;
    }
    spec.flex_grow = Some(value);
    true
}

fn handle_flex_shrink(spec: &mut Spec, token: &str) -> bool {
    if token == "shrink" {
        spec.flex_shrink = Some(1.0);
        return true;
    }

    let Some(mut suffix) = token.strip_prefix("shrink-") else {
        return false;
    };
    if suffix.is_empty() {
        return false;
    }
    if suffix == "0" {
        spec.flex_shrink = Some(0.0);
        return true;
    }

    if suffix.starts_with('[') && suffix.ends_with(']') {
        suffix = &suffix[1..suffix.len() - 1];
        if suffix.is_empty() {
            return false;
        }
        let Some(value) = parse_float(suffix) else {
            return false;
        };
        if value < 0.0 || !value.is_finite() {
            return false;
        }
        spec.flex_shrink = Some(value);
        return true;
    }

    let Some(value) = parse_float(suffix) else {
        return false;
    };
    if value < 0.0 || !value.is_finite() {
        return false;
    }
    spec.flex_shrink = Some(value);
    true
}

fn handle_flex_basis(spec: &mut Spec, token: &str) -> bool {
    let Some(suffix) = token.strip_prefix("basis-") else {
        return false;
    };
    if suffix.is_empty() {
        return false;
    }

    if suffix == "auto" {
        spec.flex_basis = Some(FlexBasis::Auto);
        return true;
    }
    if suffix == "full" {
        spec.flex_basis = Some(FlexBasis::Full);
        return true;
    }
    if suffix == "px" {
        spec.flex_basis = Some(FlexBasis::Pixels(1.0));
        return true;
    }
    if let Some(value) = parse_bracket_size_value(suffix) {
        spec.flex_basis = Some(match value {
            SizeAxisValue::Pixels(px) => FlexBasis::Pixels(px),
            SizeAxisValue::Percent(fraction) => FlexBasis::Percent(fraction),
        });
        return true;
    }
    let Some(value) = parse_float(suffix) else {
        return false;
    };
    if value < 0.0 || !value.is_finite() {
        return false;
    }
    spec.flex_basis = Some(FlexBasis::Pixels(value * DIMENSION_UNIT));
    true
}

pub fn handle_scale(spec: &mut Spec, token: &str) -> bool {
    let Some(suffix) = token.strip_prefix("scale-") else {
        return false;
    };
    let Some(value) = parse_scale_value(suffix) else {
        return false;
    };
    spec.scale = Some(value);
    true
}

pub fn handle_translate(spec: &mut Spec, token: &str) -> bool {
    let (axis, negative, suffix) = if let Some(suffix) = token.strip_prefix("-translate-x-") {
        ('x', true, suffix)
    } else if let Some(suffix) = token.strip_prefix("translate-x-") {
        ('x', false, suffix)
    } else if let Some(suffix) = token.strip_prefix("-translate-y-") {
        ('y', true, suffix)
    } else if let Some(suffix) = token.strip_prefix("translate-y-") {
        ('y', false, suffix)
    } else {
        return false;
    };
    let Some(value) = parse_translate_value(suffix, negative) else {
        return false;
    };
    let mut translate = spec.translate.unwrap_or_default();
    match axis {
        'x' => translate.x = value,
        'y' => translate.y = value,
        _ => return false,
    }
    spec.translate = Some(translate);
    true
}

pub fn handle_rotation(spec: &mut Spec, token: &str) -> bool {
    let (negative, suffix) = if let Some(suffix) = token.strip_prefix("-rotate-") {
        (true, suffix)
    } else if let Some(suffix) = token.strip_prefix("rotate-") {
        (false, suffix)
    } else {
        return false;
    };
    let Some(mut value) = parse_rotation_value(suffix) else {
        return false;
    };
    if negative {
        value = -value;
    }
    spec.rotation_degrees = Some(value);
    true
}

pub fn handle_hover_scale(spec: &mut Spec, token: &str) -> bool {
    let Some(suffix) = token.strip_prefix("scale-") else {
        return false;
    };
    let Some(value) = parse_scale_value(suffix) else {
        return false;
    };
    spec.hover_scale = Some(value);
    true
}

pub fn handle_group_hover_scale(spec: &mut Spec, token: &str) -> bool {
    let Some(suffix) = token.strip_prefix("scale-") else {
        return false;
    };
    let Some(value) = parse_scale_value(suffix) else {
        return false;
    };
    spec.group_hover_scale = Some(value);
    true
}

fn parse_scale_value(mut suffix: &str) -> Option<f32> {
    if suffix.is_empty() {
        return None;
    }

    if suffix.starts_with('[') && suffix.ends_with(']') {
        suffix = &suffix[1..suffix.len() - 1];
        if suffix.is_empty() {
            return None;
        }
        let value = parse_float(suffix)?;
        if value <= 0.0 {
            return None;
        }
        return Some(value);
    }

    let value = parse_float(suffix)?;
    if value <= 0.0 {
        return None;
    }
    Some(if value >= 10.0 { value / 100.0 } else { value })
}

fn parse_translate_value(suffix: &str, negative: bool) -> Option<f32> {
    if suffix.is_empty() {
        return None;
    }
    let value = if suffix == "px" {
        1.0
    } else if let Some(value) = parse_bracket_axis_value(suffix, true) {
        match value {
            SizeAxisValue::Pixels(px) => px,
            SizeAxisValue::Percent(_) => return None,
        }
    } else {
        let value = parse_float(suffix)?;
        if value < 0.0 {
            return None;
        }
        value * SPACING_UNIT
    };
    Some(if negative { -value.abs() } else { value })
}

fn parse_rotation_value(suffix: &str) -> Option<f32> {
    match suffix {
        "0" => Some(0.0),
        "45" => Some(45.0),
        "90" => Some(90.0),
        "180" => Some(180.0),
        _ => parse_bracket_rotation_value(suffix),
    }
}

fn parse_bracket_rotation_value(suffix: &str) -> Option<f32> {
    let inner = suffix.strip_prefix('[')?.strip_suffix(']')?;
    let degrees = inner.strip_suffix("deg")?;
    let value = parse_float(degrees)?;
    value.is_finite().then_some(value)
}

pub fn handle_spacing(spec: &mut Spec, token: &str) -> bool {
    if token.len() < 3 {
        return false;
    }
    let mut chars = token.chars();
    let Some(kind) = chars.next() else {
        return false;
    };
    let is_margin = kind == 'm';
    let is_padding = kind == 'p';
    if !is_margin && !is_padding {
        return false;
    }

    let bytes = token.as_bytes();
    let mut idx = 1;
    let mut target = SideTarget::All;
    if idx < bytes.len() && bytes[idx] != b'-' {
        target = match bytes[idx] {
            b'x' => SideTarget::Horizontal,
            b'y' => SideTarget::Vertical,
            b't' => SideTarget::Top,
            b'r' => SideTarget::Right,
            b'b' => SideTarget::Bottom,
            b'l' => SideTarget::Left,
            _ => return false,
        };
        idx += 1;
    }
    if idx >= bytes.len() || bytes[idx] != b'-' {
        return false;
    }
    idx += 1;
    if idx >= bytes.len() {
        return false;
    }
    let value_text = &token[idx..];
    if is_margin {
        let Some(value) = parse_spacing_value(value_text) else {
            return false;
        };
        spec.margin.set(target, value);
    } else {
        let Some(value) = parse_padding_value(value_text) else {
            return false;
        };
        spec.padding.set(target, value);
    }
    true
}

pub fn handle_border(spec: &mut Spec, token: &str) -> bool {
    if token == "border" {
        spec.border.set(SideTarget::All, BORDER_WIDTH_DEFAULT);
        return true;
    }
    let Some(suffix) = token.strip_prefix("border-") else {
        return false;
    };
    if suffix.is_empty() {
        return false;
    }

    if suffix.len() == 1 {
        if let Some(target) = direction_target(suffix.as_bytes()[0]) {
            spec.border.set(target, BORDER_WIDTH_DEFAULT);
            return true;
        }
    }

    if let Some((target, consume_len)) = parse_direction_prefix(suffix) {
        let mut rest = &suffix[consume_len..];
        if rest.is_empty() {
            spec.border.set(target, BORDER_WIDTH_DEFAULT);
            return true;
        }
        if !rest.starts_with('-') {
            return false;
        }
        rest = &rest[1..];
        if rest.is_empty() {
            return false;
        }
        if let Some(value) = parse_border_width(rest) {
            spec.border.set(target, value);
            return true;
        }
        if let Some(color_value) =
            lookup_color_token(crate::ui::tailwind::types::ColorAsk::Border, rest)
        {
            spec.border_color = Some(color_value);
            return true;
        }
        return false;
    }

    if let Some(width_value) = parse_border_width(suffix) {
        spec.border.set(SideTarget::All, width_value);
        return true;
    }

    if let Some(color_value) =
        lookup_color_token(crate::ui::tailwind::types::ColorAsk::Border, suffix)
    {
        spec.border_color = Some(color_value);
        return true;
    }

    false
}

pub fn handle_rounded(spec: &mut Spec, token: &str) -> bool {
    for rule in RADIUS_TOKENS {
        if token == rule.token {
            spec.corner_radii.set_all(rule.radius);
            return true;
        }
    }

    if let Some(radius) = parse_segmented_radius(token, "rounded-l", "rounded-l-") {
        spec.corner_radii.set_left(radius);
        return true;
    }
    if let Some(radius) = parse_segmented_radius(token, "rounded-r", "rounded-r-") {
        spec.corner_radii.set_right(radius);
        return true;
    }
    if let Some(radius) = parse_segmented_radius(token, "rounded-t", "rounded-t-") {
        spec.corner_radii.set_top(radius);
        return true;
    }
    if let Some(radius) = parse_segmented_radius(token, "rounded-b", "rounded-b-") {
        spec.corner_radii.set_bottom(radius);
        return true;
    }
    if let Some(radius) = parse_segmented_radius(token, "rounded-tl", "rounded-tl-") {
        spec.corner_radii.nw = Some(radius);
        return true;
    }
    if let Some(radius) = parse_segmented_radius(token, "rounded-tr", "rounded-tr-") {
        spec.corner_radii.ne = Some(radius);
        return true;
    }
    if let Some(radius) = parse_segmented_radius(token, "rounded-bl", "rounded-bl-") {
        spec.corner_radii.sw = Some(radius);
        return true;
    }
    if let Some(radius) = parse_segmented_radius(token, "rounded-br", "rounded-br-") {
        spec.corner_radii.se = Some(radius);
        return true;
    }

    false
}

fn parse_width_value(suffix: &str) -> Option<Width> {
    if suffix == "full" || suffix == "screen" {
        return Some(Width::Full);
    }
    if suffix == "px" {
        return Some(Width::Pixels(1.0));
    }
    if let Some(value) = parse_bracket_size_value(suffix) {
        return Some(match value {
            SizeAxisValue::Pixels(px) => Width::Pixels(px),
            SizeAxisValue::Percent(fraction) => Width::Percent(fraction),
        });
    }
    let value = parse_float(suffix)?;
    (value >= 0.0).then_some(Width::Pixels(value * DIMENSION_UNIT))
}

fn parse_height_value(suffix: &str) -> Option<Height> {
    if suffix == "full" || suffix == "screen" {
        return Some(Height::Full);
    }
    if suffix == "px" {
        return Some(Height::Pixels(1.0));
    }
    if let Some(value) = parse_bracket_size_value(suffix) {
        return Some(match value {
            SizeAxisValue::Pixels(px) => Height::Pixels(px),
            SizeAxisValue::Percent(fraction) => Height::Percent(fraction),
        });
    }
    let value = parse_float(suffix)?;
    (value >= 0.0).then_some(Height::Pixels(value * DIMENSION_UNIT))
}

fn parse_radius_value(suffix: &str) -> Option<f32> {
    if suffix.is_empty() {
        return RADIUS_TOKENS
            .iter()
            .find(|rule| rule.token == "rounded")
            .map(|rule| rule.radius);
    }
    RADIUS_TOKENS
        .iter()
        .find(|rule| rule.token.strip_prefix("rounded-") == Some(suffix))
        .map(|rule| rule.radius)
}

fn parse_segmented_radius(token: &str, exact: &str, prefixed: &str) -> Option<f32> {
    if token == exact {
        return parse_radius_value("");
    }
    token.strip_prefix(prefixed).and_then(parse_radius_value)
}

pub fn handle_inset(spec: &mut Spec, token: &str) -> bool {
    if let Some(value_text) = token.strip_prefix("top-") {
        let Some(value) = parse_inset_value(value_text) else {
            return false;
        };
        spec.top = Some(value);
        return true;
    }
    if let Some(value_text) = token.strip_prefix("right-") {
        let Some(value) = parse_inset_value(value_text) else {
            return false;
        };
        spec.right = Some(value);
        return true;
    }
    if let Some(value_text) = token.strip_prefix("bottom-") {
        let Some(value) = parse_inset_value(value_text) else {
            return false;
        };
        spec.bottom = Some(value);
        return true;
    }
    if let Some(value_text) = token.strip_prefix("left-") {
        let Some(value) = parse_inset_value(value_text) else {
            return false;
        };
        spec.left = Some(value);
        return true;
    }
    false
}

pub fn handle_hover_spacing(spec: &mut Spec, token: &str) -> bool {
    handle_state_spacing(
        token,
        |target, value| {
            spec.hover_margin.set(target, value);
        },
        |target, value| {
            spec.hover_padding.set(target, value);
        },
    )
}

pub fn handle_group_hover_spacing(spec: &mut Spec, token: &str) -> bool {
    handle_state_spacing(
        token,
        |target, value| {
            spec.group_hover_margin.set(target, value);
        },
        |target, value| {
            spec.group_hover_padding.set(target, value);
        },
    )
}

pub fn handle_hover_border(spec: &mut Spec, token: &str) -> bool {
    handle_state_border(
        token,
        |target, value| {
            spec.hover_border.set(target, value);
        },
        |color_value| {
            spec.hover_border_color = Some(color_value);
        },
    )
}

pub fn handle_group_hover_border(spec: &mut Spec, token: &str) -> bool {
    handle_state_border(
        token,
        |target, value| {
            spec.group_hover_border.set(target, value);
        },
        |color_value| {
            spec.group_hover_border_color = Some(color_value);
        },
    )
}

fn handle_state_spacing<FMargin, FPadding>(
    token: &str,
    mut set_margin: FMargin,
    mut set_padding: FPadding,
) -> bool
where
    FMargin: FnMut(SideTarget, f32),
    FPadding: FnMut(SideTarget, PaddingValue),
{
    if token.len() < 3 {
        return false;
    }
    let mut chars = token.chars();
    let Some(kind) = chars.next() else {
        return false;
    };
    let is_margin = kind == 'm';
    let is_padding = kind == 'p';
    if !is_margin && !is_padding {
        return false;
    }

    let bytes = token.as_bytes();
    let mut idx = 1;
    let mut target = SideTarget::All;
    if idx < bytes.len() && bytes[idx] != b'-' {
        target = match bytes[idx] {
            b'x' => SideTarget::Horizontal,
            b'y' => SideTarget::Vertical,
            b't' => SideTarget::Top,
            b'r' => SideTarget::Right,
            b'b' => SideTarget::Bottom,
            b'l' => SideTarget::Left,
            _ => return false,
        };
        idx += 1;
    }
    if idx >= bytes.len() || bytes[idx] != b'-' {
        return false;
    }
    idx += 1;
    if idx >= bytes.len() {
        return false;
    }
    let value_text = &token[idx..];
    if is_margin {
        let Some(value) = parse_spacing_value(value_text) else {
            return false;
        };
        set_margin(target, value);
    } else {
        let Some(value) = parse_padding_value(value_text) else {
            return false;
        };
        set_padding(target, value);
    }
    true
}

fn handle_state_border<FBorder, FColor>(
    token: &str,
    mut set_border: FBorder,
    mut set_color: FColor,
) -> bool
where
    FBorder: FnMut(SideTarget, f32),
    FColor: FnMut(crate::ui::tailwind::types::ColorRef),
{
    if token == "border" {
        set_border(SideTarget::All, BORDER_WIDTH_DEFAULT);
        return true;
    }
    let Some(suffix) = token.strip_prefix("border-") else {
        return false;
    };
    if suffix.is_empty() {
        return false;
    }

    if suffix.len() == 1 {
        if let Some(target) = direction_target(suffix.as_bytes()[0]) {
            set_border(target, BORDER_WIDTH_DEFAULT);
            return true;
        }
    }

    if let Some((target, consume_len)) = parse_direction_prefix(suffix) {
        let mut rest = &suffix[consume_len..];
        if rest.is_empty() {
            set_border(target, BORDER_WIDTH_DEFAULT);
            return true;
        }
        if !rest.starts_with('-') {
            return false;
        }
        rest = &rest[1..];
        if rest.is_empty() {
            return false;
        }
        if let Some(value) = parse_border_width(rest) {
            set_border(target, value);
            return true;
        }
        if let Some(color_value) =
            lookup_color_token(crate::ui::tailwind::types::ColorAsk::Border, rest)
        {
            set_color(color_value);
            return true;
        }
        return false;
    }

    if let Some(width_value) = parse_border_width(suffix) {
        set_border(SideTarget::All, width_value);
        return true;
    }

    if let Some(color_value) =
        lookup_color_token(crate::ui::tailwind::types::ColorAsk::Border, suffix)
    {
        set_color(color_value);
        return true;
    }

    false
}

fn parse_direction_prefix(suffix: &str) -> Option<(SideTarget, usize)> {
    let bytes = suffix.as_bytes();
    if bytes.len() < 2 || bytes[1] != b'-' {
        return None;
    }
    let target = match bytes[0] {
        b'x' => SideTarget::Horizontal,
        b'y' => SideTarget::Vertical,
        b't' => SideTarget::Top,
        b'r' => SideTarget::Right,
        b'b' => SideTarget::Bottom,
        b'l' => SideTarget::Left,
        _ => return None,
    };
    Some((target, 1))
}

fn direction_target(byte: u8) -> Option<SideTarget> {
    match byte {
        b'x' => Some(SideTarget::Horizontal),
        b'y' => Some(SideTarget::Vertical),
        b't' => Some(SideTarget::Top),
        b'r' => Some(SideTarget::Right),
        b'b' => Some(SideTarget::Bottom),
        b'l' => Some(SideTarget::Left),
        _ => None,
    }
}

fn parse_spacing_value(token: &str) -> Option<f32> {
    if token.is_empty() {
        return None;
    }
    if token == "px" {
        return Some(1.0);
    }
    if let Some(value) = parse_bracket_axis_value(token, false) {
        return match value {
            SizeAxisValue::Pixels(px) => Some(px),
            SizeAxisValue::Percent(_) => None,
        };
    }
    let value = parse_float(token)?;
    if value < 0.0 {
        return None;
    }
    Some(value * SPACING_UNIT)
}

fn parse_padding_value(token: &str) -> Option<PaddingValue> {
    if token.is_empty() {
        return None;
    }
    if let Some(value) = parse_bracket_axis_value(token, false) {
        return Some(match value {
            SizeAxisValue::Pixels(px) => PaddingValue::Pixels(px),
            SizeAxisValue::Percent(fraction) => PaddingValue::Percent(fraction),
        });
    }
    let value = parse_spacing_value(token)?;
    Some(PaddingValue::Pixels(value))
}

fn parse_border_width(token: &str) -> Option<f32> {
    if token.is_empty() {
        return None;
    }
    if token == "px" {
        return Some(1.0);
    }
    let value = parse_float(token)?;
    if value < 0.0 {
        return None;
    }
    Some(value)
}

fn parse_inset_value(token: &str) -> Option<Inset> {
    if token.is_empty() {
        return None;
    }
    if let Some(value) = parse_bracket_axis_value(token, true) {
        return Some(match value {
            SizeAxisValue::Pixels(px) => Inset::Pixels(px),
            SizeAxisValue::Percent(fraction) => Inset::Percent(fraction),
        });
    }
    let value = parse_spacing_value(token)?;
    Some(Inset::Pixels(value))
}

fn parse_bracket_size_value(token: &str) -> Option<SizeAxisValue> {
    parse_bracket_axis_value(token, false)
}

fn parse_bracket_axis_value(token: &str, allow_negative: bool) -> Option<SizeAxisValue> {
    if token.len() < 2 || !token.starts_with('[') || !token.ends_with(']') {
        return None;
    }
    let inner = &token[1..token.len() - 1];
    if inner.is_empty() {
        return None;
    }

    if let Some(num_slice) = inner.strip_suffix('%') {
        if num_slice.is_empty() {
            return None;
        }
        let value = parse_float(num_slice)?;
        if !allow_negative && value < 0.0 {
            return None;
        }
        return Some(SizeAxisValue::Percent(value / 100.0));
    }

    let num_slice = inner.strip_suffix("px").unwrap_or(inner);
    let value = parse_float(num_slice)?;
    if !allow_negative && value < 0.0 {
        return None;
    }
    Some(SizeAxisValue::Pixels(value))
}

fn parse_aspect_ratio_value(token: &str) -> Option<f32> {
    if token.len() < 3 || !token.starts_with('[') || !token.ends_with(']') {
        return None;
    }
    let inner = &token[1..token.len() - 1];
    if inner.is_empty() {
        return None;
    }
    let value = parse_float(inner)?;
    if value <= 0.0 {
        return None;
    }
    Some(value)
}

fn parse_grid_track_count(token: &str) -> Option<usize> {
    let value = if token.starts_with('[') && token.ends_with(']') {
        &token[1..token.len() - 1]
    } else {
        token
    };
    let value = value.parse::<usize>().ok()?;
    if value == 0 {
        return None;
    }
    Some(value)
}

fn parse_float(token: &str) -> Option<f32> {
    let value = token.parse::<f32>().ok()?;
    if !value.is_finite() {
        return None;
    }
    Some(value)
}
