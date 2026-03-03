#![allow(dead_code)]

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use egui::{Button, Color32, CornerRadius, Response, Stroke, Ui};

const LUCIDE_ICON_DIRECTORY: &str = "assets/icons/lucide";
const LUCIDE_ICON_URI_PREFIX: &str = "bytes://clay/lucide/";
const DEFAULT_ICON_SIZE: f32 = 14.0;
const DEFAULT_ICON_BUTTON_WIDTH: f32 = 22.0;

static REGISTERED_LUCIDE_URIS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

#[derive(Clone, Copy)]
pub(crate) enum IconButtonKind {
    Action,
    Toggle,
    Segmented,
}

pub(crate) fn setup(egui_context: &egui::Context) {
    egui_extras::install_image_loaders(egui_context);
}

pub(crate) fn image(
    egui_context: &egui::Context,
    name: &str,
    size: f32,
) -> Option<egui::Image<'static>> {
    let uri = ensure_lucide_icon_uri(egui_context, name)?;
    let icon_size = size.max(1.0);
    Some(egui::Image::from_uri(uri).fit_to_exact_size(egui::vec2(icon_size, icon_size)))
}

pub(crate) fn icon_button(
    ui: &mut Ui,
    name: &str,
    selected: bool,
    kind: IconButtonKind,
) -> Response {
    let corner_radius = ui.visuals().widgets.inactive.corner_radius;
    icon_button_with_corner(
        ui,
        name,
        selected,
        kind,
        corner_radius,
        DEFAULT_ICON_BUTTON_WIDTH,
    )
}

pub(crate) fn icon_button_with_corner(
    ui: &mut Ui,
    name: &str,
    selected: bool,
    kind: IconButtonKind,
    corner_radius: CornerRadius,
    width: f32,
) -> Response {
    ui.scope(|ui| {
        apply_icon_button_visuals(ui, kind, selected);
        let interact_height = ui.spacing().interact_size.y;
        let button = if let Some(image) = image(ui.ctx(), name, DEFAULT_ICON_SIZE) {
            Button::image(image).image_tint_follows_text_color(true)
        } else {
            Button::new("")
        };
        ui.add_sized(
            [width, interact_height],
            button
                .frame(true)
                .corner_radius(corner_radius)
                .min_size(egui::vec2(width, interact_height)),
        )
    })
    .inner
}

fn apply_icon_button_visuals(ui: &mut Ui, kind: IconButtonKind, selected: bool) {
    let visuals = &mut ui.style_mut().visuals.widgets;
    match kind {
        IconButtonKind::Action => {
            visuals.inactive.bg_fill = Color32::TRANSPARENT;
            visuals.inactive.weak_bg_fill = Color32::TRANSPARENT;
            visuals.inactive.bg_stroke = Stroke::NONE;
            visuals.hovered.bg_fill = Color32::from_gray(54);
            visuals.hovered.weak_bg_fill = Color32::from_gray(54);
            visuals.hovered.bg_stroke = Stroke::NONE;
            visuals.active.bg_fill = Color32::from_gray(58);
            visuals.active.weak_bg_fill = Color32::from_gray(58);
            visuals.active.bg_stroke = Stroke::NONE;
            visuals.open.bg_fill = Color32::from_gray(58);
            visuals.open.weak_bg_fill = Color32::from_gray(58);
            visuals.open.bg_stroke = Stroke::NONE;
        }
        IconButtonKind::Toggle => {
            let inactive_fill = if selected {
                Color32::from_gray(60)
            } else {
                Color32::TRANSPARENT
            };
            let hovered_fill = if selected {
                Color32::from_gray(64)
            } else {
                Color32::from_gray(54)
            };
            let active_fill = if selected {
                Color32::from_gray(68)
            } else {
                Color32::from_gray(58)
            };
            let inactive_stroke = if selected {
                Stroke::new(1.0, Color32::from_gray(90))
            } else {
                Stroke::NONE
            };
            let hovered_stroke = if selected {
                Stroke::new(1.0, Color32::from_gray(98))
            } else {
                Stroke::NONE
            };
            let active_stroke = if selected {
                Stroke::new(1.0, Color32::from_gray(106))
            } else {
                Stroke::NONE
            };
            visuals.inactive.bg_fill = inactive_fill;
            visuals.inactive.weak_bg_fill = inactive_fill;
            visuals.inactive.bg_stroke = inactive_stroke;
            visuals.hovered.bg_fill = hovered_fill;
            visuals.hovered.weak_bg_fill = hovered_fill;
            visuals.hovered.bg_stroke = hovered_stroke;
            visuals.active.bg_fill = active_fill;
            visuals.active.weak_bg_fill = active_fill;
            visuals.active.bg_stroke = active_stroke;
            visuals.open.bg_fill = active_fill;
            visuals.open.weak_bg_fill = active_fill;
            visuals.open.bg_stroke = active_stroke;
        }
        IconButtonKind::Segmented => {
            let inactive_fill = if selected {
                Color32::from_gray(60)
            } else {
                Color32::from_gray(46)
            };
            let hovered_fill = if selected {
                Color32::from_gray(64)
            } else {
                Color32::from_gray(50)
            };
            let active_fill = if selected {
                Color32::from_gray(68)
            } else {
                Color32::from_gray(54)
            };
            visuals.inactive.bg_fill = inactive_fill;
            visuals.inactive.weak_bg_fill = inactive_fill;
            visuals.inactive.bg_stroke = Stroke::NONE;
            visuals.hovered.bg_fill = hovered_fill;
            visuals.hovered.weak_bg_fill = hovered_fill;
            visuals.hovered.bg_stroke = Stroke::NONE;
            visuals.active.bg_fill = active_fill;
            visuals.active.weak_bg_fill = active_fill;
            visuals.active.bg_stroke = Stroke::NONE;
            visuals.open.bg_fill = active_fill;
            visuals.open.weak_bg_fill = active_fill;
            visuals.open.bg_stroke = Stroke::NONE;
        }
    }
}

fn ensure_lucide_icon_uri(egui_context: &egui::Context, name: &str) -> Option<String> {
    if !is_valid_lucide_name(name) {
        return None;
    }

    let uri = format!("{LUCIDE_ICON_URI_PREFIX}{name}.svg");
    if is_uri_registered(uri.as_str()) {
        return Some(uri);
    }

    let svg_bytes = match std::fs::read(lucide_icon_path(name)) {
        Ok(bytes) => normalize_lucide_svg_bytes(bytes),
        Err(_) => {
            unregister_uri(uri.as_str());
            return None;
        }
    };

    egui_context.include_bytes(uri.clone(), svg_bytes);
    Some(uri)
}

fn is_uri_registered(uri: &str) -> bool {
    let registry = registered_lucide_uris();
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
    let registry = registered_lucide_uris();
    let mut guard = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.remove(uri);
}

fn registered_lucide_uris() -> &'static Mutex<HashSet<String>> {
    REGISTERED_LUCIDE_URIS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn is_valid_lucide_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn lucide_icon_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(LUCIDE_ICON_DIRECTORY)
        .join(format!("{name}.svg"))
}

fn normalize_lucide_svg_bytes(bytes: Vec<u8>) -> Vec<u8> {
    match String::from_utf8(bytes) {
        Ok(svg) => svg.replace("currentColor", "#FFFFFF").into_bytes(),
        Err(err) => err.into_bytes(),
    }
}
