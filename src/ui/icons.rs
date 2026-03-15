use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

const ICON_DIRECTORY: &str = "assets/icons";
const LUCIDE_ICON_SUBDIRECTORY: &str = "lucide";
const BOOTSTRAP_ICON_SUBDIRECTORY: &str = "bootstrap";
const LUCIDE_ICON_URI_PREFIX: &str = "bytes://egui-component/lucide/";
const BOOTSTRAP_ICON_URI_PREFIX: &str = "bytes://egui-component/bootstrap/";
const ICON_STROKE_WIDTH_FROM: &str = "stroke-width=\"2\"";
const ICON_STROKE_WIDTH_TO: &str = "stroke-width=\"1.75\"";

static CACHED_ICON_ASSETS: OnceLock<Mutex<HashMap<String, Arc<CachedIconAsset>>>> = OnceLock::new();

#[derive(Debug)]
struct CachedIconAsset {
    source: Arc<str>,
    raster_bytes: Arc<[u8]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IconFamily {
    Lucide,
    Bootstrap,
}

pub fn setup(egui_context: &egui::Context) {
    egui_extras::install_image_loaders(egui_context);
}

pub fn image(egui_context: &egui::Context, name: &str, size: f32) -> Option<egui::Image<'static>> {
    let uri = ensure_icon_uri(egui_context, name)?;
    let icon_size = size.max(1.0);
    Some(egui::Image::from_uri(uri).fit_to_exact_size(egui::vec2(icon_size, icon_size)))
}

pub fn svg_source(name: &str) -> Option<Arc<str>> {
    let (family, normalized_name) = parse_icon_name(name)?;
    Some(Arc::clone(
        &load_icon_asset(family, normalized_name.as_str())?.source,
    ))
}

fn ensure_icon_uri(egui_context: &egui::Context, name: &str) -> Option<String> {
    let (family, normalized_name) = parse_icon_name(name)?;
    let uri = format!("{}{}.svg", family.uri_prefix(), normalized_name);
    let svg_bytes = Arc::clone(&load_icon_asset(family, normalized_name.as_str())?.raster_bytes);
    egui_context.include_bytes(uri.clone(), svg_bytes);
    Some(uri)
}

fn load_icon_asset(family: IconFamily, name: &str) -> Option<Arc<CachedIconAsset>> {
    let cache = cached_icon_assets();
    let mut guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let cache_key = format!("{}:{name}", family.cache_key());
    if let Some(asset) = guard.get(cache_key.as_str()) {
        return Some(Arc::clone(asset));
    }

    let source = std::fs::read_to_string(icon_path(family, name)).ok()?;
    let asset = Arc::new(CachedIconAsset {
        raster_bytes: Arc::<[u8]>::from(normalize_icon_svg_source(family, source.as_str())),
        source: Arc::<str>::from(source.into_boxed_str()),
    });
    guard.insert(cache_key, Arc::clone(&asset));
    Some(asset)
}

fn cached_icon_assets() -> &'static Mutex<HashMap<String, Arc<CachedIconAsset>>> {
    CACHED_ICON_ASSETS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn parse_icon_name(name: &str) -> Option<(IconFamily, String)> {
    let trimmed = name.trim();
    if let Some(name) = trimmed.strip_prefix("bootstrap:") {
        return Some((IconFamily::Bootstrap, normalize_icon_name(name)?));
    }

    Some((IconFamily::Lucide, normalize_icon_name(trimmed)?))
}

fn normalize_icon_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }

    let without_extension = if trimmed.to_ascii_lowercase().ends_with(".svg") {
        &trimmed[..trimmed.len().saturating_sub(4)]
    } else {
        trimmed
    };

    let mut normalized = String::with_capacity(without_extension.len());
    let mut previous_was_separator = false;
    let mut previous_was_lower_or_digit = false;

    for ch in without_extension.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() {
                if !normalized.is_empty() && previous_was_lower_or_digit && !previous_was_separator
                {
                    normalized.push('-');
                }
                normalized.push(ch.to_ascii_lowercase());
                previous_was_lower_or_digit = false;
            } else {
                normalized.push(ch.to_ascii_lowercase());
                previous_was_lower_or_digit = true;
            }
            previous_was_separator = false;
            continue;
        }

        if ch == '-' || ch == '_' || ch == ' ' {
            if !normalized.is_empty() && !previous_was_separator {
                normalized.push('-');
                previous_was_separator = true;
            }
            previous_was_lower_or_digit = false;
            continue;
        }

        return None;
    }

    while normalized.ends_with('-') {
        normalized.pop();
    }

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn icon_path(family: IconFamily, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(ICON_DIRECTORY)
        .join(family.subdirectory())
        .join(format!("{name}.svg"))
}

fn normalize_icon_svg_source(family: IconFamily, source: &str) -> Vec<u8> {
    let normalized = source.replace("currentColor", "#FFFFFF");
    match family {
        IconFamily::Lucide => normalized
            .replace(ICON_STROKE_WIDTH_FROM, ICON_STROKE_WIDTH_TO)
            .into_bytes(),
        IconFamily::Bootstrap => normalized.into_bytes(),
    }
}

impl IconFamily {
    fn cache_key(self) -> &'static str {
        match self {
            Self::Lucide => "lucide",
            Self::Bootstrap => "bootstrap",
        }
    }

    fn subdirectory(self) -> &'static str {
        match self {
            Self::Lucide => LUCIDE_ICON_SUBDIRECTORY,
            Self::Bootstrap => BOOTSTRAP_ICON_SUBDIRECTORY,
        }
    }

    fn uri_prefix(self) -> &'static str {
        match self {
            Self::Lucide => LUCIDE_ICON_URI_PREFIX,
            Self::Bootstrap => BOOTSTRAP_ICON_URI_PREFIX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_icon_uri, icon_path, normalize_icon_name, normalize_icon_svg_source,
        parse_icon_name, setup, svg_source, IconFamily, ICON_STROKE_WIDTH_FROM,
        ICON_STROKE_WIDTH_TO, LUCIDE_ICON_URI_PREFIX,
    };
    use egui::load::{ImagePoll, SizeHint};

    #[test]
    fn accepts_standard_lucide_kebab_case() {
        assert_eq!(
            normalize_icon_name("chevrons-up-down"),
            Some("chevrons-up-down".to_owned())
        );
    }

    #[test]
    fn normalizes_common_name_variants() {
        assert_eq!(
            normalize_icon_name("ChevronDown"),
            Some("chevron-down".to_owned())
        );
        assert_eq!(
            normalize_icon_name("chevron_down"),
            Some("chevron-down".to_owned())
        );
        assert_eq!(
            normalize_icon_name("chevron-down.svg"),
            Some("chevron-down".to_owned())
        );
        assert_eq!(
            normalize_icon_name("PlayFill"),
            Some("play-fill".to_owned())
        );
    }

    #[test]
    fn rejects_invalid_characters() {
        assert_eq!(normalize_icon_name(""), None);
        assert_eq!(normalize_icon_name("chevron/down"), None);
    }

    #[test]
    fn normalizes_icon_stroke_width() {
        let svg = format!("<svg stroke=\"currentColor\" {ICON_STROKE_WIDTH_FROM}></svg>");
        let normalized =
            String::from_utf8(normalize_icon_svg_source(IconFamily::Lucide, svg.as_str())).unwrap();

        assert!(normalized.contains("#FFFFFF"));
        assert!(normalized.contains(ICON_STROKE_WIDTH_TO));
        assert!(!normalized.contains(ICON_STROKE_WIDTH_FROM));
    }

    #[test]
    fn resolves_play_fill_icon_asset() {
        assert!(icon_path(IconFamily::Lucide, "play-fill").is_file());

        let context = egui::Context::default();
        setup(&context);

        let uri = ensure_icon_uri(&context, "PlayFill");
        let expected_uri = format!("{LUCIDE_ICON_URI_PREFIX}play-fill.svg");
        assert_eq!(uri.as_deref(), Some(expected_uri.as_str()));
    }

    #[test]
    fn resolves_fire_icon_asset() {
        assert!(icon_path(IconFamily::Lucide, "fire").is_file());

        let context = egui::Context::default();
        setup(&context);

        let uri = ensure_icon_uri(&context, "fire");
        let expected_uri = format!("{LUCIDE_ICON_URI_PREFIX}fire.svg");
        assert_eq!(uri.as_deref(), Some(expected_uri.as_str()));
    }

    #[test]
    fn resolves_requested_sidebar_icon_assets() {
        for (name, family, icon) in [
            (
                "bootstrap:globe-americas",
                IconFamily::Bootstrap,
                "globe-americas",
            ),
            (
                "bootstrap:music-note-beamed",
                IconFamily::Bootstrap,
                "music-note-beamed",
            ),
            ("bootstrap:box", IconFamily::Bootstrap, "box"),
            ("image", IconFamily::Lucide, "image"),
        ] {
            assert!(
                icon_path(family, icon).is_file(),
                "missing icon file for {icon}"
            );

            let context = egui::Context::default();
            setup(&context);

            let uri = ensure_icon_uri(&context, name);
            let expected_uri = format!("{}{}.svg", family.uri_prefix(), icon);
            assert_eq!(uri.as_deref(), Some(expected_uri.as_str()));
        }
    }

    #[test]
    fn rasterizes_requested_sidebar_icons_with_visible_pixels_in_each_context() {
        let first_context = egui::Context::default();
        let second_context = egui::Context::default();
        setup(&first_context);
        setup(&second_context);

        for context in [&first_context, &second_context] {
            for icon in [
                "bootstrap:globe-americas",
                "bootstrap:fire",
                "bootstrap:music-note-beamed",
                "bootstrap:box",
            ] {
                let uri = ensure_icon_uri(context, icon).expect("icon uri");
                let image = context
                    .try_load_image(uri.as_str(), SizeHint::default())
                    .expect("image load");

                let ImagePoll::Ready { image } = image else {
                    panic!("icon image should be ready for {icon}");
                };

                assert!(
                    image.pixels.iter().any(|pixel| pixel.a() > 0),
                    "icon should contain visible pixels for {icon}"
                );
            }
        }
    }

    #[test]
    fn parses_bootstrap_prefixed_names() {
        assert_eq!(
            parse_icon_name("bootstrap:MusicNoteBeamed"),
            Some((IconFamily::Bootstrap, "music-note-beamed".to_owned()))
        );
    }

    #[test]
    fn exposes_svg_source_from_shared_assets() {
        let source = svg_source("bootstrap:fire").expect("svg source");

        assert!(source.contains("<svg"));
        assert!(source.contains("<path") || source.contains("<g"));
    }
}
