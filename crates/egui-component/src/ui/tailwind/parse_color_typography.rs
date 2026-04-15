use crate::ui::tailwind::lookup_color_token;
use crate::ui::tailwind::tokens::TYPOGRAPHY_TOKENS;
use crate::ui::tailwind::types::{
    ColorAsk, FontFamily, FontWeight, Spec, TextShadow, TextUnit, UiRuntimeBackground,
};

pub fn handle_background(spec: &mut Spec, suffix: &str) -> bool {
    if let Some(color_value) = lookup_color_token(ColorAsk::Fill, suffix) {
        spec.background = Some(UiRuntimeBackground::Solid(color_value));
        return true;
    }
    false
}

pub fn handle_text(spec: &mut Spec, suffix: &str) -> bool {
    if let Some(rest) = suffix.strip_prefix("shadow") {
        return handle_text_shadow(spec, rest);
    }
    if let Some(rest) = suffix.strip_prefix("outline-") {
        if rest.is_empty() {
            return false;
        }
        if let Some(value) = parse_outline_thickness(rest) {
            spec.text_outline_thickness = Some(value);
            return true;
        }
        if let Some(color_value) = lookup_color_token(ColorAsk::Text, rest) {
            spec.text_outline_color = Some(color_value);
            return true;
        }
        return false;
    }
    if let Some(color_value) = lookup_color_token(ColorAsk::Text, suffix) {
        spec.text = Some(color_value);
        return true;
    }
    false
}

fn handle_text_shadow(spec: &mut Spec, rest: &str) -> bool {
    let shadow = spec.text_shadow.get_or_insert_with(TextShadow::default);
    let Some(rest) = rest.strip_prefix('-') else {
        return false;
    };
    if let Some(value) = rest
        .strip_prefix("x-")
        .and_then(parse_shadow_offset)
        .filter(|value| value.is_finite())
    {
        shadow.x = value;
        return true;
    }
    if let Some(value) = rest
        .strip_prefix("y-")
        .and_then(parse_shadow_offset)
        .filter(|value| value.is_finite())
    {
        shadow.y = value;
        return true;
    }
    if let Some(value) = rest
        .strip_prefix("opacity-")
        .and_then(parse_opacity_value)
        .filter(|value| value.is_finite())
    {
        shadow.opacity = value;
        return true;
    }
    if let Some(color) = lookup_color_token(ColorAsk::Text, rest) {
        shadow.color = color;
        return true;
    }
    false
}

pub fn handle_typography(spec: &mut Spec, token: &str) -> bool {
    for rule in TYPOGRAPHY_TOKENS {
        if token == rule.token {
            spec.text_size = Some(rule.text_size);
            spec.font_scale = None;
            return true;
        }
    }
    if let Some(value) = parse_typography_scale_percent(token) {
        spec.text_size = None;
        spec.font_scale = Some(value);
        return true;
    }
    false
}

pub fn handle_font_token(spec: &mut Spec, token: &str) -> bool {
    let Some(suffix) = token.strip_prefix("font-") else {
        return false;
    };
    if let Some(custom) = parse_custom_font_id(suffix) {
        spec.font_id = Some(custom);
        return true;
    }
    match suffix {
        "sans" => {
            spec.font_family = Some(FontFamily::Sans);
            spec.font_id = None;
            true
        }
        "serif" => {
            spec.font_family = Some(FontFamily::Serif);
            spec.font_id = None;
            true
        }
        "mono" => {
            spec.font_family = Some(FontFamily::Mono);
            spec.font_id = None;
            true
        }
        "normal" | "regular" => {
            spec.font_weight = Some(FontWeight::Regular);
            true
        }
        "medium" => {
            spec.font_weight = Some(FontWeight::Medium);
            true
        }
        "semibold" => {
            spec.font_weight = Some(FontWeight::Semibold);
            true
        }
        "bold" => {
            spec.font_weight = Some(FontWeight::Bold);
            true
        }
        _ => false,
    }
}

pub fn handle_text_tracking(spec: &mut Spec, token: &str) -> bool {
    let Some(suffix) = token.strip_prefix("tracking-") else {
        return false;
    };
    let Some(value) = parse_text_unit(suffix) else {
        return false;
    };
    spec.text_tracking = Some(value);
    true
}

pub fn handle_text_leading(spec: &mut Spec, token: &str) -> bool {
    let Some(suffix) = token.strip_prefix("leading-") else {
        return false;
    };
    let Some(value) = parse_text_unit(suffix) else {
        return false;
    };
    spec.text_leading = Some(value);
    true
}

pub fn handle_line_clamp(spec: &mut Spec, token: &str) -> bool {
    let Some(suffix) = token.strip_prefix("line-clamp-") else {
        return false;
    };
    let Some(value) = parse_line_clamp_value(suffix) else {
        return false;
    };
    spec.text_max_lines = Some(value);
    true
}

pub fn handle_opacity(spec: &mut Spec, token: &str) -> bool {
    let Some(suffix) = token.strip_prefix("opacity-") else {
        return false;
    };
    if suffix.is_empty() {
        return false;
    }
    let Ok(int_value) = suffix.parse::<u8>() else {
        return false;
    };
    if int_value > 100 {
        return false;
    }
    spec.opacity = Some(f32::from(int_value) / 100.0);
    true
}

pub fn handle_hover_background(spec: &mut Spec, suffix: &str) -> bool {
    if let Some(color_value) = lookup_color_token(ColorAsk::FillHover, suffix) {
        spec.hover_background = Some(UiRuntimeBackground::Solid(color_value));
        return true;
    }
    false
}

pub fn handle_hover_text(spec: &mut Spec, suffix: &str) -> bool {
    if let Some(rest) = suffix.strip_prefix("outline-") {
        if rest.is_empty() {
            return false;
        }
        if let Some(value) = parse_outline_thickness(rest) {
            spec.hover_text_outline_thickness = Some(value);
            return true;
        }
        if let Some(color_value) = lookup_color_token(ColorAsk::TextHover, rest) {
            spec.hover_text_outline_color = Some(color_value);
            return true;
        }
        return false;
    }
    if let Some(color_value) = lookup_color_token(ColorAsk::TextHover, suffix) {
        spec.hover_text = Some(color_value);
        return true;
    }
    false
}

pub fn handle_hover_opacity(spec: &mut Spec, token: &str) -> bool {
    let Some(suffix) = token.strip_prefix("opacity-") else {
        return false;
    };
    if suffix.is_empty() {
        return false;
    }
    let Ok(int_value) = suffix.parse::<u8>() else {
        return false;
    };
    if int_value > 100 {
        return false;
    }
    spec.hover_opacity = Some(f32::from(int_value) / 100.0);
    true
}

pub fn handle_group_hover_background(spec: &mut Spec, suffix: &str) -> bool {
    if let Some(color_value) = lookup_color_token(ColorAsk::FillHover, suffix) {
        spec.group_hover_background = Some(UiRuntimeBackground::Solid(color_value));
        return true;
    }
    false
}

pub fn handle_group_hover_text(spec: &mut Spec, suffix: &str) -> bool {
    if let Some(rest) = suffix.strip_prefix("outline-") {
        if rest.is_empty() {
            return false;
        }
        if let Some(value) = parse_outline_thickness(rest) {
            spec.group_hover_text_outline_thickness = Some(value);
            return true;
        }
        if let Some(color_value) = lookup_color_token(ColorAsk::TextHover, rest) {
            spec.group_hover_text_outline_color = Some(color_value);
            return true;
        }
        return false;
    }
    if let Some(color_value) = lookup_color_token(ColorAsk::TextHover, suffix) {
        spec.group_hover_text = Some(color_value);
        return true;
    }
    false
}

pub fn handle_group_hover_opacity(spec: &mut Spec, token: &str) -> bool {
    let Some(suffix) = token.strip_prefix("opacity-") else {
        return false;
    };
    if suffix.is_empty() {
        return false;
    }
    let Ok(int_value) = suffix.parse::<u8>() else {
        return false;
    };
    if int_value > 100 {
        return false;
    }
    spec.group_hover_opacity = Some(f32::from(int_value) / 100.0);
    true
}

fn parse_text_unit(token: &str) -> Option<TextUnit> {
    let token = parse_bracket_token(token).unwrap_or(token).trim();
    if token.is_empty() {
        return None;
    }
    if let Some(percent_text) = token.strip_suffix('%') {
        let value = parse_float(percent_text.trim())?;
        return Some(TextUnit::FontPercent(value / 100.0));
    }
    let value = parse_float(token.strip_suffix("px").unwrap_or(token).trim())?;
    Some(TextUnit::Pixels(value))
}

fn parse_shadow_offset(token: &str) -> Option<f32> {
    let token = parse_bracket_token(token).unwrap_or(token).trim();
    if token.is_empty() {
        return None;
    }
    parse_float(token.strip_suffix("px").unwrap_or(token).trim())
}

fn parse_opacity_value(token: &str) -> Option<f32> {
    if token.is_empty() {
        return None;
    }
    let value = token.parse::<u8>().ok()?;
    if value > 100 {
        return None;
    }
    Some(f32::from(value) / 100.0)
}

fn parse_line_clamp_value(token: &str) -> Option<usize> {
    let token = parse_bracket_token(token).unwrap_or(token).trim();
    if token.is_empty() {
        return None;
    }
    let value = token.parse::<usize>().ok()?;
    if value == 0 {
        return None;
    }
    Some(value)
}

fn parse_bracket_token(token: &str) -> Option<&str> {
    token.strip_prefix('[')?.strip_suffix(']')
}

fn parse_outline_thickness(token: &str) -> Option<TextUnit> {
    let value = parse_text_unit(token)?;
    match value {
        TextUnit::Pixels(value) | TextUnit::FontPercent(value) if value < 0.0 => None,
        _ => Some(value),
    }
}

fn parse_typography_scale_percent(token: &str) -> Option<f32> {
    let rest = token.strip_prefix("text-[")?;
    if rest.len() < 2 || !rest.ends_with(']') {
        return None;
    }
    let inner = &rest[..rest.len() - 1];
    let percent_text = inner.strip_suffix('%')?;
    if percent_text.is_empty() {
        return None;
    }
    let percent = parse_float(percent_text)?;
    if percent <= 0.0 {
        return None;
    }
    Some(percent / 100.0)
}

fn parse_custom_font_id(token: &str) -> Option<String> {
    let inner = token.strip_prefix('[')?.strip_suffix(']')?;
    let inner = inner.trim();
    if inner.is_empty() {
        return None;
    }
    Some(inner.to_owned())
}

fn parse_float(token: &str) -> Option<f32> {
    let value = token.parse::<f32>().ok()?;
    if !value.is_finite() {
        return None;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::{
        handle_font_token, handle_line_clamp, handle_text_leading, handle_text_tracking,
        handle_typography,
    };
    use crate::ui::tailwind::types::{FontWeight, Spec, TextSize, TextUnit};

    #[test]
    fn handle_typography_parses_percent_scale_token() {
        let mut spec = Spec::default();
        assert!(handle_typography(&mut spec, "text-[125%]"));
        assert_eq!(spec.text_size, None);
        assert!(spec.font_scale.is_some());
        assert!((spec.font_scale.expect("font_scale") - 1.25).abs() < 0.0001);
    }

    #[test]
    fn handle_typography_rejects_invalid_percent_scale_token() {
        let mut spec = Spec::default();
        assert!(!handle_typography(&mut spec, "text-[nope%]"));
        assert!(spec.font_scale.is_none());
    }

    #[test]
    fn handle_typography_parses_supported_named_token() {
        let mut spec = Spec::default();
        assert!(handle_typography(&mut spec, "text-2xl"));
        assert_eq!(spec.text_size, Some(TextSize::X2l));
        assert_eq!(spec.font_scale, None);
    }

    #[test]
    fn handle_font_token_parses_supported_weights() {
        let mut spec = Spec::default();
        assert!(handle_font_token(&mut spec, "font-normal"));
        assert_eq!(spec.font_weight, Some(FontWeight::Regular));
        assert!(handle_font_token(&mut spec, "font-medium"));
        assert_eq!(spec.font_weight, Some(FontWeight::Medium));
        assert!(handle_font_token(&mut spec, "font-semibold"));
        assert_eq!(spec.font_weight, Some(FontWeight::Semibold));
        assert!(handle_font_token(&mut spec, "font-bold"));
        assert_eq!(spec.font_weight, Some(FontWeight::Bold));
    }

    #[test]
    fn handle_text_tracking_parses_pixels_and_percent() {
        let mut spec = Spec::default();
        assert!(handle_text_tracking(&mut spec, "tracking-[1px]"));
        assert_eq!(spec.text_tracking, Some(TextUnit::Pixels(1.0)));
        assert!(handle_text_tracking(&mut spec, "tracking-[25%]"));
        assert_eq!(spec.text_tracking, Some(TextUnit::FontPercent(0.25)));
    }

    #[test]
    fn handle_text_leading_parses_percent() {
        let mut spec = Spec::default();
        assert!(handle_text_leading(&mut spec, "leading-[120%]"));
        assert_eq!(spec.text_leading, Some(TextUnit::FontPercent(1.2)));
    }

    #[test]
    fn handle_line_clamp_parses_plain_and_bracketed_values() {
        let mut spec = Spec::default();
        assert!(handle_line_clamp(&mut spec, "line-clamp-2"));
        assert_eq!(spec.text_max_lines, Some(2));
        assert!(handle_line_clamp(&mut spec, "line-clamp-[3]"));
        assert_eq!(spec.text_max_lines, Some(3));
        assert!(!handle_line_clamp(&mut spec, "line-clamp-0"));
    }
}
