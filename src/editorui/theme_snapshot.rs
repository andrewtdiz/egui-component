#![allow(dead_code)]

use std::fs;
use std::path::Path;

use crate::ClayError;
use crate::Result;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct ThemeSnapshot {
    options: egui::Options,
    dark_style: egui::Style,
    light_style: egui::Style,
}

impl ThemeSnapshot {
    pub(crate) fn capture(egui_context: &egui::Context) -> Self {
        Self {
            options: egui_context.options(|options| options.clone()),
            dark_style: (*egui_context.style_of(egui::Theme::Dark)).clone(),
            light_style: (*egui_context.style_of(egui::Theme::Light)).clone(),
        }
    }

    pub(crate) fn apply(&self, egui_context: &egui::Context) {
        let options = self.options.clone();
        egui_context.options_mut(move |target| *target = options);
        egui_context.set_style_of(egui::Theme::Dark, self.dark_style.clone());
        egui_context.set_style_of(egui::Theme::Light, self.light_style.clone());
    }

    pub(crate) fn load_from_path(path: &Path) -> Result<Option<Self>> {
        if !path.is_file() {
            return Ok(None);
        }

        let source = fs::read_to_string(path).map_err(|error| {
            ClayError::PlatformError(format!(
                "failed to read theme snapshot at {}: {error}",
                path.display()
            ))
        })?;

        let snapshot = ron::from_str(&source).map_err(|error| {
            ClayError::PlatformError(format!(
                "failed to parse theme snapshot at {}: {error}",
                path.display()
            ))
        })?;

        Ok(Some(snapshot))
    }

    pub(crate) fn save_to_path(&self, path: &Path) -> Result {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                ClayError::PlatformError(format!(
                    "failed to create theme snapshot directory {}: {error}",
                    parent.display()
                ))
            })?;
        }

        let source =
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::new()).map_err(|error| {
                ClayError::PlatformError(format!(
                    "failed to serialize theme snapshot for {}: {error}",
                    path.display()
                ))
            })?;

        fs::write(path, source).map_err(|error| {
            ClayError::PlatformError(format!(
                "failed to write theme snapshot at {}: {error}",
                path.display()
            ))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::ThemeSnapshot;

    fn unique_temp_path(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("clayengine-editorui-{label}-{unique}.ron"))
    }

    #[test]
    fn roundtrip_preserves_snapshot_payload() {
        let mut options = egui::Options::default();
        options.zoom_factor = 1.35;

        let mut dark_style = egui::Theme::Dark.default_style();
        dark_style.spacing.item_spacing = egui::vec2(9.0, 7.0);

        let mut light_style = egui::Theme::Light.default_style();
        light_style.spacing.item_spacing = egui::vec2(4.0, 3.0);

        let snapshot = ThemeSnapshot {
            options,
            dark_style,
            light_style,
        };

        let encoded = ron::ser::to_string(&snapshot).expect("snapshot should serialize");
        let decoded: ThemeSnapshot = ron::from_str(&encoded).expect("snapshot should deserialize");

        assert!((decoded.options.zoom_factor - snapshot.options.zoom_factor).abs() < 0.0001);
        assert_eq!(
            decoded.dark_style.spacing.item_spacing,
            snapshot.dark_style.spacing.item_spacing
        );
        assert_eq!(
            decoded.light_style.spacing.item_spacing,
            snapshot.light_style.spacing.item_spacing
        );
    }

    #[test]
    fn load_missing_snapshot_returns_none() {
        let path = unique_temp_path("missing");
        let loaded = ThemeSnapshot::load_from_path(path.as_path()).expect("load should succeed");
        assert!(loaded.is_none());
    }

    #[test]
    fn load_invalid_snapshot_reports_parse_error() {
        let path = unique_temp_path("invalid");
        fs::write(path.as_path(), "{ not valid ron").expect("write should succeed");

        let error = ThemeSnapshot::load_from_path(path.as_path()).expect_err("load should fail");
        assert!(error.to_string().contains("failed to parse theme snapshot"));

        let _ = fs::remove_file(path);
    }
}
