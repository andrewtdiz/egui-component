use std::path::PathBuf;

use crate::editorui::component_showcase::{
    parse_component_kind, supported_component_ids_csv, ComponentKind,
};
use crate::{ClayError, Result};

const DEFAULT_THEME_SNAPSHOT_PATH: &str = "ui/editorui.theme.ron";
const DEFAULT_WIDTH: i32 = 420;
const DEFAULT_HEIGHT: i32 = 320;
const DEFAULT_HEADLESS_FRAMES: u64 = 2;

#[derive(Debug, Clone)]
pub(crate) struct ComponentRunOptions {
    pub(crate) component: ComponentKind,
    pub(crate) headless: bool,
    pub(crate) verify_png_path: Option<PathBuf>,
    pub(crate) max_frames: Option<u64>,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) theme_snapshot_path: PathBuf,
}

impl ComponentRunOptions {
    pub(crate) fn parse(args: &[String]) -> Result<Self> {
        let mut component_name = None;
        let mut headless = false;
        let mut verify_png_path = None;
        let mut max_frames = None;
        let mut width = DEFAULT_WIDTH;
        let mut height = DEFAULT_HEIGHT;
        let mut theme_snapshot_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_THEME_SNAPSHOT_PATH);
        let mut index = 0usize;

        while index < args.len() {
            match args[index].as_str() {
                "--" => {
                    index += 1;
                }
                "--headless" => {
                    headless = true;
                    index += 1;
                }
                "--verify-png" => {
                    let value = next_arg(args, index, "--verify-png")?;
                    verify_png_path = Some(PathBuf::from(value));
                    index += 2;
                }
                "--max-frames" => {
                    let value = next_arg(args, index, "--max-frames")?;
                    max_frames = Some(parse_u64(value, "--max-frames")?);
                    index += 2;
                }
                "--width" => {
                    let value = next_arg(args, index, "--width")?;
                    width = parse_positive_i32(value, "--width")?;
                    index += 2;
                }
                "--height" => {
                    let value = next_arg(args, index, "--height")?;
                    height = parse_positive_i32(value, "--height")?;
                    index += 2;
                }
                "--theme-snapshot" => {
                    let value = next_arg(args, index, "--theme-snapshot")?;
                    theme_snapshot_path = PathBuf::from(value);
                    index += 2;
                }
                value if value.starts_with("--") => {
                    return Err(ClayError::PlatformError(format!(
                        "unknown component argument: {value}"
                    )));
                }
                value => {
                    if component_name.is_some() {
                        return Err(ClayError::PlatformError(format!(
                            "unexpected positional argument: {value}"
                        )));
                    }
                    component_name = Some(value.to_owned());
                    index += 1;
                }
            }
        }

        let Some(component_name) = component_name else {
            return Err(ClayError::PlatformError(format!(
                "missing component name. Supported: {}",
                supported_component_ids_csv()
            )));
        };

        let component = parse_component_kind(component_name.as_str()).ok_or_else(|| {
            ClayError::PlatformError(format!(
                "unknown component '{}'. Supported: {}",
                component_name,
                supported_component_ids_csv()
            ))
        })?;

        if !headless {
            if verify_png_path.is_some() {
                return Err(ClayError::PlatformError(
                    "--verify-png requires --headless".to_owned(),
                ));
            }
            if max_frames.is_some() {
                return Err(ClayError::PlatformError(
                    "--max-frames requires --headless".to_owned(),
                ));
            }
        }

        if headless {
            if verify_png_path.is_none() {
                return Err(ClayError::PlatformError(
                    "--headless requires --verify-png <path>".to_owned(),
                ));
            }
            if max_frames.is_none() {
                max_frames = Some(DEFAULT_HEADLESS_FRAMES);
            }
        }

        Ok(Self {
            component,
            headless,
            verify_png_path,
            max_frames,
            width,
            height,
            theme_snapshot_path,
        })
    }
}

fn next_arg<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str> {
    args.get(index + 1)
        .map(|value| value.as_str())
        .ok_or_else(|| ClayError::PlatformError(format!("{flag} requires a value")))
}

fn parse_u64(value: &str, flag: &str) -> Result<u64> {
    value.parse::<u64>().map_err(|err| {
        ClayError::PlatformError(format!("invalid value for {flag}: {value} ({err})"))
    })
}

fn parse_positive_i32(value: &str, flag: &str) -> Result<i32> {
    let parsed = value.parse::<i32>().map_err(|err| {
        ClayError::PlatformError(format!("invalid value for {flag}: {value} ({err})"))
    })?;
    if parsed <= 0 {
        return Err(ClayError::PlatformError(format!(
            "invalid value for {flag}: {value} (must be > 0)"
        )));
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::ComponentRunOptions;
    use crate::editorui::component_showcase::ComponentKind;

    #[test]
    fn parse_accepts_lenient_component_aliases() {
        let args = vec!["Button_Group".to_owned()];
        let options = ComponentRunOptions::parse(args.as_slice()).expect("expected alias to parse");
        assert_eq!(options.component, ComponentKind::ButtonGroup);
    }

    #[test]
    fn parse_errors_for_unknown_component_name() {
        let args = vec!["unknown-component".to_owned()];
        let error = ComponentRunOptions::parse(args.as_slice()).expect_err("expected parse error");
        assert!(error.to_string().contains("unknown component"));
    }

    #[test]
    fn parse_errors_when_headless_missing_verify_png() {
        let args = vec!["button".to_owned(), "--headless".to_owned()];
        let error = ComponentRunOptions::parse(args.as_slice()).expect_err("expected parse error");
        assert!(error
            .to_string()
            .contains("--headless requires --verify-png"));
    }

    #[test]
    fn parse_sets_default_headless_max_frames() {
        let args = vec![
            "button".to_owned(),
            "--headless".to_owned(),
            "--verify-png".to_owned(),
            "out.png".to_owned(),
        ];
        let options = ComponentRunOptions::parse(args.as_slice()).expect("expected headless parse");
        assert_eq!(options.max_frames, Some(2));
        assert!(options.headless);
    }

    #[test]
    fn parse_applies_custom_numeric_options() {
        let args = vec![
            "slider".to_owned(),
            "--width".to_owned(),
            "512".to_owned(),
            "--height".to_owned(),
            "384".to_owned(),
        ];
        let options = ComponentRunOptions::parse(args.as_slice()).expect("expected parse");
        assert_eq!(options.width, 512);
        assert_eq!(options.height, 384);
    }

    #[test]
    fn parse_rejects_non_headless_max_frames() {
        let args = vec![
            "slider".to_owned(),
            "--max-frames".to_owned(),
            "8".to_owned(),
        ];
        let error = ComponentRunOptions::parse(args.as_slice()).expect_err("expected parse error");
        assert!(error
            .to_string()
            .contains("--max-frames requires --headless"));
    }

    #[test]
    fn parse_ignores_standalone_double_dash() {
        let args = vec!["--".to_owned(), "button".to_owned()];
        let options = ComponentRunOptions::parse(args.as_slice()).expect("expected parse");
        assert_eq!(options.component, ComponentKind::Button);
    }
}
