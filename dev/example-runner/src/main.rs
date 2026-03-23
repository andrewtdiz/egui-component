use egui::{self, CentralPanel, Pos2, RawInput, Rect, ViewportId};
use egui_component::example_apps::showcase;
use egui_component::{
    component_definitions, parse_component_kind, supported_component_ids_csv, ComponentKind,
    ThemeMode,
};
use egui_kittest::{wgpu::WgpuTestRenderer, TestRenderer};
use image::RgbaImage;
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Child, ExitStatus};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

type AppResult<T = ()> = Result<T, Box<dyn Error>>;

const WATCH_DEBOUNCE: Duration = Duration::from_millis(250);
const WATCH_POLL: Duration = Duration::from_millis(200);
const SNAPSHOT_OUTPUT_DIR: &str = "artifacts/ui-snapshots";
const SNAPSHOT_WINDOW_SIZE: [f32; 2] = [1280.0, 1100.0];
const SNAPSHOT_RENDER_FRAMES: usize = 4;
const SNAPSHOT_MIN_RENDER_FRAMES: usize = 2;
const SNAPSHOT_FRAME_DT_SECS: f32 = 1.0 / 60.0;
const SNAPSHOT_PIXELS_PER_POINT: f32 = 1.0;
const SNAPSHOT_CROP_PADDING_PX: u32 = 16;
const SHOWCASE_USAGE: &str = "usage: cargo example showcase [--hot]";
const SNAPSHOT_USAGE: &str =
    "usage: cargo example snapshot (--component <id> | --all) [--theme light|dark|both] [--output <dir>]\n   or: cargo screenshot (--component <id> | --all) [--theme light|dark|both] [--output <dir>]";

fn main() -> AppResult {
    let args = Args::parse(env::args_os().skip(1))?;
    match args.command {
        CommandKind::Showcase { hot } => {
            if hot {
                run_hot("showcase")
            } else {
                run_once("showcase")
            }
        }
        CommandKind::Snapshot(snapshot) => run_snapshot(snapshot),
    }
}

#[derive(Debug)]
struct Args {
    command: CommandKind,
}

#[derive(Debug)]
enum CommandKind {
    Showcase { hot: bool },
    Snapshot(SnapshotArgs),
}

#[derive(Debug, Clone)]
struct SnapshotArgs {
    components: SnapshotComponents,
    theme: SnapshotTheme,
    output_dir: PathBuf,
}

#[derive(Debug, Clone)]
enum SnapshotComponents {
    One(ComponentKind),
    All,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SnapshotTheme {
    Light,
    Dark,
    Both,
}

impl SnapshotTheme {
    fn modes(self) -> &'static [ThemeMode] {
        match self {
            Self::Light => &[ThemeMode::Light],
            Self::Dark => &[ThemeMode::Dark],
            Self::Both => &[ThemeMode::Light, ThemeMode::Dark],
        }
    }
}

impl Args {
    fn parse(args: impl IntoIterator<Item = OsString>) -> AppResult<Self> {
        let mut args = args.into_iter();
        let Some(command) = args.next() else {
            return Err(format!("missing command\n{SHOWCASE_USAGE}\n{SNAPSHOT_USAGE}").into());
        };

        let command = match command.to_string_lossy().as_ref() {
            "showcase" => CommandKind::Showcase {
                hot: parse_showcase_args(args)?,
            },
            "snapshot" => CommandKind::Snapshot(parse_snapshot_args(args)?),
            other => {
                return Err(format!(
                    "unsupported command `{other}`\n{SHOWCASE_USAGE}\n{SNAPSHOT_USAGE}"
                )
                .into())
            }
        };

        Ok(Self { command })
    }
}

fn parse_showcase_args(args: impl IntoIterator<Item = OsString>) -> AppResult<bool> {
    let mut hot = false;

    for arg in args {
        match arg.to_string_lossy().as_ref() {
            "--hot" => hot = true,
            other => return Err(format!("unsupported argument `{other}`\n{SHOWCASE_USAGE}").into()),
        }
    }

    Ok(hot)
}

fn parse_snapshot_args(args: impl IntoIterator<Item = OsString>) -> AppResult<SnapshotArgs> {
    let mut component = None;
    let mut all = false;
    let mut theme = SnapshotTheme::Both;
    let mut output_dir = PathBuf::from(SNAPSHOT_OUTPUT_DIR);

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--component" => {
                let Some(value) = args.next() else {
                    return Err(format!("missing value for `--component`\n{SNAPSHOT_USAGE}").into());
                };
                let component_id = value.to_string_lossy();
                let parsed = parse_component_kind(component_id.as_ref()).ok_or_else(|| {
                    format!(
                        "unsupported component `{}`\nsupported: {}",
                        component_id,
                        supported_component_ids_csv()
                    )
                })?;
                component = Some(parsed);
            }
            "--all" => all = true,
            "--theme" => {
                let Some(value) = args.next() else {
                    return Err(format!("missing value for `--theme`\n{SNAPSHOT_USAGE}").into());
                };
                theme = match value.to_string_lossy().as_ref() {
                    "light" => SnapshotTheme::Light,
                    "dark" => SnapshotTheme::Dark,
                    "both" => SnapshotTheme::Both,
                    other => {
                        return Err(format!(
                            "unsupported theme `{other}`\nexpected one of: light, dark, both"
                        )
                        .into())
                    }
                };
            }
            "--output" => {
                let Some(value) = args.next() else {
                    return Err(format!("missing value for `--output`\n{SNAPSHOT_USAGE}").into());
                };
                output_dir = PathBuf::from(value);
            }
            other => return Err(format!("unsupported argument `{other}`\n{SNAPSHOT_USAGE}").into()),
        }
    }

    let components = match (component, all) {
        (Some(kind), false) => SnapshotComponents::One(kind),
        (None, true) => SnapshotComponents::All,
        (Some(_), true) => {
            return Err("use either `--component` or `--all`, not both".into());
        }
        (None, false) => {
            return Err(format!("missing target\n{SNAPSHOT_USAGE}").into());
        }
    };

    Ok(SnapshotArgs {
        components,
        theme,
        output_dir,
    })
}

fn run_once(example: &str) -> AppResult {
    let status = example_command(example).status()?;
    exit_for_status(status)
}

fn run_snapshot(args: SnapshotArgs) -> AppResult {
    let repo_root = repo_root();
    let output_dir = if args.output_dir.is_absolute() {
        args.output_dir.clone()
    } else {
        repo_root.join(args.output_dir)
    };
    std::fs::create_dir_all(&output_dir)?;

    let components = match args.components {
        SnapshotComponents::One(kind) => vec![kind],
        SnapshotComponents::All => component_definitions()
            .map(|definition| definition.kind)
            .collect(),
    };

    for component in components {
        for theme in args.theme.modes() {
            let output_path = output_dir.join(format!(
                "{}--{}.png",
                component_id(component),
                theme_suffix(*theme)
            ));
            eprintln!(
                "[example-snapshot] rendering `{}` ({}) -> {}",
                component_id(component),
                theme_suffix(*theme),
                output_path.display()
            );
            capture_snapshot(component, *theme, output_path)?;
        }
    }

    Ok(())
}

fn capture_snapshot(
    component: ComponentKind,
    theme_mode: ThemeMode,
    output_path: PathBuf,
) -> AppResult {
    let image = render_snapshot_image(component, theme_mode)?;
    save_rgba_image(&output_path, &image)
}

fn render_snapshot_image(component: ComponentKind, theme_mode: ThemeMode) -> AppResult<RgbaImage> {
    let context = egui::Context::default();
    showcase::install_context(&context);

    let mut renderer = WgpuTestRenderer::default();
    let mut state = showcase::ShowcaseApp::default();
    let mut last_frame = None;

    for frame_index in 0..SNAPSHOT_RENDER_FRAMES {
        let raw_input = snapshot_raw_input(frame_index);
        let (output, crop_rect) =
            render_snapshot_frame(&context, &mut state, component, theme_mode, raw_input);
        renderer.handle_delta(&output.textures_delta);
        let image = renderer
            .render(&context, &output)
            .map_err(|error| format!("headless snapshot render failed: {error}"))?;
        let cropped = crop_snapshot_image(&image, crop_rect)?;
        let pending_images = context.has_pending_images();

        last_frame = Some(cropped);
        if frame_index + 1 >= SNAPSHOT_MIN_RENDER_FRAMES && !pending_images {
            break;
        }
    }

    last_frame.ok_or_else(|| "snapshot render produced no frames".into())
}

fn render_snapshot_frame(
    context: &egui::Context,
    state: &mut showcase::ShowcaseApp,
    component: ComponentKind,
    theme_mode: ThemeMode,
    raw_input: RawInput,
) -> (egui::FullOutput, Rect) {
    let mut snapshot_rect = Rect::NOTHING;
    let output = context.run(raw_input, |ctx| {
        showcase::configure_snapshot(state, component, theme_mode);
        showcase::prepare_frame(state, ctx);
        snapshot_rect = CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| showcase::render_snapshot_component(state, ui))
            .inner
            .rect;
    });

    let crop_rect = snapshot_crop_rect(context, snapshot_rect).unwrap_or(snapshot_rect);
    (output, crop_rect)
}

fn snapshot_raw_input(frame_index: usize) -> RawInput {
    let mut input = RawInput {
        screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(SNAPSHOT_WINDOW_SIZE[0], SNAPSHOT_WINDOW_SIZE[1]),
        )),
        time: Some(frame_index as f64 * f64::from(SNAPSHOT_FRAME_DT_SECS)),
        predicted_dt: SNAPSHOT_FRAME_DT_SECS,
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&ViewportId::ROOT)
        .expect("root viewport")
        .native_pixels_per_point = Some(SNAPSHOT_PIXELS_PER_POINT);
    input
}

fn snapshot_crop_rect(context: &egui::Context, base_rect: Rect) -> Option<Rect> {
    let mut used_rect = if base_rect.is_positive() {
        Some(base_rect)
    } else {
        None
    };

    context.memory(|memory| {
        for layer_id in memory.areas().visible_layer_ids() {
            if layer_id.order == egui::Order::Background {
                continue;
            }
            if let Some(area_rect) = memory.area_rect(layer_id.id) {
                used_rect = Some(match used_rect {
                    Some(mut rect) => {
                        rect |= area_rect;
                        rect
                    }
                    None => area_rect,
                });
            }
        }
    });

    used_rect
}

fn crop_snapshot_image(image: &RgbaImage, crop_rect: Rect) -> AppResult<RgbaImage> {
    let stage_rect = Rect::from_min_size(
        Pos2::ZERO,
        egui::vec2(SNAPSHOT_WINDOW_SIZE[0], SNAPSHOT_WINDOW_SIZE[1]),
    );
    let padded_rect = crop_rect
        .expand(SNAPSHOT_CROP_PADDING_PX as f32 / SNAPSHOT_PIXELS_PER_POINT)
        .intersect(stage_rect);

    if !padded_rect.is_positive() {
        return Err("snapshot crop rect was empty".into());
    }

    let image_width = image.width();
    let image_height = image.height();
    let pixels_per_point = SNAPSHOT_PIXELS_PER_POINT;

    let left = (padded_rect.left() * pixels_per_point)
        .floor()
        .clamp(0.0, image_width as f32) as u32;
    let top = (padded_rect.top() * pixels_per_point)
        .floor()
        .clamp(0.0, image_height as f32) as u32;
    let right = (padded_rect.right() * pixels_per_point)
        .ceil()
        .clamp((left + 1) as f32, image_width as f32) as u32;
    let bottom = (padded_rect.bottom() * pixels_per_point)
        .ceil()
        .clamp((top + 1) as f32, image_height as f32) as u32;

    Ok(image::imageops::crop_imm(
        image,
        left,
        top,
        right.saturating_sub(left),
        bottom.saturating_sub(top),
    )
    .to_image())
}

fn save_rgba_image(path: &Path, image: &RgbaImage) -> AppResult {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    image.save(path)?;
    Ok(())
}

fn theme_suffix(theme_mode: ThemeMode) -> &'static str {
    match theme_mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    }
}

fn component_id(kind: ComponentKind) -> &'static str {
    component_definitions()
        .find(|definition| definition.kind == kind)
        .map(|definition| definition.id)
        .expect("component definition")
}

fn run_hot(example: &str) -> AppResult {
    let repo_root = repo_root();
    let (watch_tx, watch_rx) = mpsc::channel::<DebounceEventResult>();
    let (signal_tx, signal_rx) = mpsc::channel::<()>();
    let child = Arc::new(Mutex::new(Some(spawn_example(example)?)));
    let signal_child = Arc::clone(&child);

    ctrlc::set_handler(move || {
        let _ = stop_child(&signal_child);
        let _ = signal_tx.send(());
    })?;

    let mut debouncer = new_debouncer(WATCH_DEBOUNCE, watch_tx)?;
    for (path, mode) in watch_targets(&repo_root) {
        debouncer.watcher().watch(path.as_path(), mode)?;
    }

    loop {
        if signal_rx.try_recv().is_ok() {
            return Ok(());
        }

        match watch_rx.recv_timeout(WATCH_POLL) {
            Ok(Ok(events)) => {
                if events.is_empty() {
                    continue;
                }
                eprintln!("[example-hot] change detected; restarting `{example}`");
                restart_child(&child, example)?;
            }
            Ok(Err(error)) => {
                eprintln!("[example-hot] watcher error: {error}");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        match try_wait_child(&child)? {
            Some(status) if status.success() => return Ok(()),
            Some(status) => {
                eprintln!(
                    "[example-hot] `{example}` exited with {status}; waiting for the next change"
                );
            }
            None => {}
        }
    }

    Ok(())
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn watch_targets(root: &Path) -> Vec<(PathBuf, RecursiveMode)> {
    vec![
        (root.join("src"), RecursiveMode::Recursive),
        (root.join("examples"), RecursiveMode::Recursive),
        (root.join("assets"), RecursiveMode::Recursive),
        (root.join(".cargo"), RecursiveMode::Recursive),
        (root.join("Cargo.toml"), RecursiveMode::NonRecursive),
        (root.join("Cargo.lock"), RecursiveMode::NonRecursive),
    ]
}

fn example_command(example: &str) -> std::process::Command {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let mut command = std::process::Command::new(cargo);
    command
        .current_dir(repo_root())
        .arg("run")
        .arg("--example")
        .arg(example)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());
    command
}

fn spawn_example(example: &str) -> AppResult<Child> {
    eprintln!("[example-hot] launching `{example}`");
    Ok(example_command(example).spawn()?)
}

fn restart_child(child: &Arc<Mutex<Option<Child>>>, example: &str) -> AppResult {
    stop_child(child)?;
    let mut guard = child.lock().map_err(|_| "child process lock poisoned")?;
    *guard = Some(spawn_example(example)?);
    Ok(())
}

fn try_wait_child(child: &Arc<Mutex<Option<Child>>>) -> AppResult<Option<ExitStatus>> {
    let mut guard = child.lock().map_err(|_| "child process lock poisoned")?;
    let Some(running) = guard.as_mut() else {
        return Ok(None);
    };

    let Some(status) = running.try_wait()? else {
        return Ok(None);
    };

    *guard = None;
    Ok(Some(status))
}

fn stop_child(child: &Arc<Mutex<Option<Child>>>) -> AppResult {
    let mut guard = child.lock().map_err(|_| "child process lock poisoned")?;
    if let Some(mut running) = guard.take() {
        let _ = running.kill();
        let _ = running.wait();
    }
    Ok(())
}

fn exit_for_status(status: ExitStatus) -> AppResult {
    if status.success() {
        return Ok(());
    }

    Err(format!("example process exited with {status}").into())
}

#[cfg(test)]
mod tests {
    use super::{Args, CommandKind, SnapshotComponents, SnapshotTheme};
    use std::ffi::OsString;

    #[test]
    fn parses_showcase_hot_mode() {
        let args = Args::parse(["showcase", "--hot"].map(OsString::from)).expect("args");
        assert!(matches!(args.command, CommandKind::Showcase { hot: true }));
    }

    #[test]
    fn parses_single_component_snapshot_defaults() {
        let args = Args::parse(["snapshot", "--component", "canva-position"].map(OsString::from))
            .expect("args");
        let CommandKind::Snapshot(snapshot) = args.command else {
            panic!("expected snapshot command");
        };

        assert!(matches!(snapshot.components, SnapshotComponents::One(_)));
        assert_eq!(snapshot.theme, SnapshotTheme::Both);
    }

    #[test]
    fn parses_all_snapshot_with_theme_and_output() {
        let args = Args::parse(
            [
                "snapshot",
                "--all",
                "--theme",
                "dark",
                "--output",
                "tmp/snaps",
            ]
            .map(OsString::from),
        )
        .expect("args");
        let CommandKind::Snapshot(snapshot) = args.command else {
            panic!("expected snapshot command");
        };

        assert!(matches!(snapshot.components, SnapshotComponents::All));
        assert_eq!(snapshot.theme, SnapshotTheme::Dark);
        assert_eq!(snapshot.output_dir, std::path::PathBuf::from("tmp/snaps"));
    }
}
