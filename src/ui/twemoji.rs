use egui::ImageSource;
use std::borrow::Cow;
use std::fmt::Write;

const TWEMOJI_URI_PREFIX: &str = "bytes://egui-component/twemoji/";

pub fn image_source(emoji: &str) -> Option<ImageSource<'static>> {
    let (lookup_emoji, asset) = resolve_asset(emoji)?;
    Some(ImageSource::Bytes {
        uri: twemoji_uri(lookup_emoji.as_ref()).into(),
        bytes: egui::load::Bytes::Static(asset.as_bytes()),
    })
}

pub fn image(emoji: &str, size: f32) -> Option<egui::Image<'static>> {
    let icon_size = size.max(1.0);
    Some(egui::Image::new(image_source(emoji)?).fit_to_exact_size(egui::vec2(icon_size, icon_size)))
}

pub fn supported(emoji: &str) -> bool {
    resolve_asset(emoji).is_some()
}

fn twemoji_uri(emoji: &str) -> String {
    let mut uri = String::from(TWEMOJI_URI_PREFIX);
    let mut wrote_codepoint = false;

    for scalar in emoji.chars() {
        if wrote_codepoint {
            uri.push('-');
        }
        wrote_codepoint = true;
        let _ = write!(uri, "{:x}", scalar as u32);
    }

    if !wrote_codepoint {
        uri.push_str("empty");
    }

    uri.push_str(".svg");
    uri
}

fn resolve_asset(
    emoji: &str,
) -> Option<(Cow<'_, str>, &'static twemoji_assets::svg::SvgTwemojiAsset)> {
    if let Some(asset) = twemoji_assets::svg::SvgTwemojiAsset::from_emoji(emoji) {
        return Some((Cow::Borrowed(emoji), asset));
    }

    let normalized = strip_variation_selectors(emoji);
    if normalized == emoji {
        return None;
    }

    let asset = twemoji_assets::svg::SvgTwemojiAsset::from_emoji(normalized.as_str())?;
    Some((Cow::Owned(normalized), asset))
}

fn strip_variation_selectors(emoji: &str) -> String {
    emoji
        .chars()
        .filter(|scalar| *scalar != '\u{fe0f}')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{image_source, supported};

    #[test]
    fn resolves_standard_twemoji_assets() {
        assert!(image_source("🔥").is_some());
        assert!(supported("🔥"));
    }

    #[test]
    fn resolves_complex_grapheme_sequences() {
        assert!(image_source("👩‍💻").is_some());
        assert!(image_source("🧑🏽‍🚀").is_some());
        assert!(image_source("❤️").is_some());
        assert!(image_source("1️⃣").is_some());
        assert!(image_source("🇺🇸").is_some());
    }

    #[test]
    fn rejects_non_emoji_text() {
        assert!(image_source("hello").is_none());
        assert!(!supported("hello"));
    }
}
