use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const ICON_DIRECTORY: &str = "assets/icons/lucide";
const ICON_URI_PREFIX: &str = "bytes://egui-component/icons/";
const ICON_STROKE_WIDTH_FROM: &str = "stroke-width=\"2\"";
const ICON_STROKE_WIDTH_TO: &str = "stroke-width=\"1.75\"";

static REGISTERED_ICON_URIS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

pub(crate) fn setup(egui_context: &egui::Context) {
    egui_extras::install_image_loaders(egui_context);
}

pub(crate) fn image(
    egui_context: &egui::Context,
    name: &str,
    size: f32,
) -> Option<egui::Image<'static>> {
    let uri = ensure_icon_uri(egui_context, name)?;
    let icon_size = size.max(1.0);
    Some(egui::Image::from_uri(uri).fit_to_exact_size(egui::vec2(icon_size, icon_size)))
}

fn ensure_icon_uri(egui_context: &egui::Context, name: &str) -> Option<String> {
    let normalized_name = normalize_icon_name(name)?;

    let uri = format!("{ICON_URI_PREFIX}{normalized_name}.svg");
    if is_uri_registered(uri.as_str()) {
        return Some(uri);
    }

    let svg_bytes = match std::fs::read(icon_path(normalized_name.as_str())) {
        Ok(bytes) => normalize_icon_svg_bytes(bytes),
        Err(_) => {
            unregister_uri(uri.as_str());
            return None;
        }
    };

    egui_context.include_bytes(uri.clone(), svg_bytes);
    Some(uri)
}

fn is_uri_registered(uri: &str) -> bool {
    let registry = registered_icon_uris();
    let mut guard = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if guard.contains(uri) {
        true
    } else {
        guard.insert(uri.to_owned());
        false
    }
}

fn unregister_uri(uri: &str) {
    let registry = registered_icon_uris();
    let mut guard = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.remove(uri);
}

fn registered_icon_uris() -> &'static Mutex<HashSet<String>> {
    REGISTERED_ICON_URIS.get_or_init(|| Mutex::new(HashSet::new()))
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

fn icon_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(ICON_DIRECTORY)
        .join(format!("{name}.svg"))
}

fn normalize_icon_svg_bytes(bytes: Vec<u8>) -> Vec<u8> {
    match String::from_utf8(bytes) {
        Ok(svg) => svg
            .replace("currentColor", "#FFFFFF")
            .replace(ICON_STROKE_WIDTH_FROM, ICON_STROKE_WIDTH_TO)
            .into_bytes(),
        Err(error) => error.into_bytes(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_icon_uri, icon_path, normalize_icon_name, normalize_icon_svg_bytes, setup,
        ICON_STROKE_WIDTH_FROM, ICON_STROKE_WIDTH_TO, ICON_URI_PREFIX,
    };

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
        let svg =
            format!("<svg stroke=\"currentColor\" {ICON_STROKE_WIDTH_FROM}></svg>").into_bytes();
        let normalized = String::from_utf8(normalize_icon_svg_bytes(svg)).unwrap();

        assert!(normalized.contains("#FFFFFF"));
        assert!(normalized.contains(ICON_STROKE_WIDTH_TO));
        assert!(!normalized.contains(ICON_STROKE_WIDTH_FROM));
    }

    #[test]
    fn resolves_play_fill_icon_asset() {
        assert!(icon_path("play-fill").is_file());

        let context = egui::Context::default();
        setup(&context);

        let uri = ensure_icon_uri(&context, "PlayFill");
        let expected_uri = format!("{ICON_URI_PREFIX}play-fill.svg");
        assert_eq!(uri.as_deref(), Some(expected_uri.as_str()));
    }
}
