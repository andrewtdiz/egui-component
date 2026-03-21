use egui_component::{ComponentLibraryError, Result};
use libloading::Library;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

pub(crate) fn run_hot_showcase_window() -> Result {
    ensure_hot_library_built()?;

    let window_title = "Component Library Showcase (Hot)";
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(window_title)
            .with_inner_size([1200.0, 760.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        window_title,
        native_options,
        Box::new(move |creation_context| {
            Ok(Box::new(HotShowcaseWindowApp::new(
                creation_context.egui_ctx.clone(),
            )?))
        }),
    )
    .map_err(|error| ComponentLibraryError::Runtime(error.to_string()))
}

struct HotShowcaseWindowApp {
    snapshot_json: String,
    reload_generation: u64,
    shadow_counter: u64,
    pending_reload: bool,
    build_events: Receiver<HotBuildEvent>,
    hot_lib: Option<LoadedHotShowcaseLib>,
}

impl HotShowcaseWindowApp {
    fn new(context: egui::Context) -> Result<Self> {
        egui_component::theme::install_context_resources(&context);
        if let Err(error) = cleanup_hot_shadow_directory() {
            eprintln!("hot reload shadow cleanup failed:\n{error}");
        }

        let mut shadow_counter = 0;
        let hot_lib = LoadedHotShowcaseLib::load(next_shadow_library_path(&mut shadow_counter)?)?;

        Ok(Self {
            snapshot_json: hot_lib.default_snapshot(),
            reload_generation: 0,
            shadow_counter,
            pending_reload: false,
            build_events: spawn_build_watcher(context),
            hot_lib: Some(hot_lib),
        })
    }

    fn handle_build_events(&mut self) {
        while let Ok(event) = self.build_events.try_recv() {
            match event {
                HotBuildEvent::Building => {}
                HotBuildEvent::Ready => {
                    self.pending_reload = true;
                    eprintln!("hot reload: rebuilt showcase library");
                }
                HotBuildEvent::Failed(message) => {
                    eprintln!("hot reload build failed:\n{message}");
                }
            }
        }
    }

    fn apply_pending_reload(&mut self, context: &egui::Context) {
        if !self.pending_reload {
            return;
        }

        context.memory_mut(|memory| *memory = Default::default());
        eprintln!("hot reload: preparing showcase reload");

        let load_result =
            next_shadow_library_path(&mut self.shadow_counter).and_then(LoadedHotShowcaseLib::load);

        let new_hot_lib = match load_result {
            Ok(hot_lib) => hot_lib,
            Err(error) => {
                eprintln!("hot reload load failed:\n{}", error);
                self.pending_reload = false;
                return;
            }
        };

        if let Some(previous_hot_lib) = self.hot_lib.replace(new_hot_lib) {
            remove_hot_lib(previous_hot_lib);
        }
        self.reload_generation = self.reload_generation.saturating_add(1);
        self.pending_reload = false;
        eprintln!(
            "hot reload: loaded showcase generation {}",
            self.reload_generation
        );
        context.request_repaint();
    }
}

impl eframe::App for HotShowcaseWindowApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        context.request_repaint_after(Duration::from_millis(100));
        self.handle_build_events();
        self.apply_pending_reload(context);
        let hot_lib = self
            .hot_lib
            .as_ref()
            .expect("hot showcase library should stay loaded for the app lifetime");
        self.snapshot_json =
            hot_lib.frame(context, self.snapshot_json.as_str(), self.reload_generation);
    }
}

impl Drop for HotShowcaseWindowApp {
    fn drop(&mut self) {
        if let Some(hot_lib) = self.hot_lib.take() {
            remove_hot_lib(hot_lib);
        }
    }
}

struct LoadedHotShowcaseLib {
    _library: Library,
    shadow_path: PathBuf,
    default_snapshot_fn: unsafe fn() -> String,
    frame_fn: unsafe fn(&egui::Context, &str, u64) -> String,
}

impl LoadedHotShowcaseLib {
    fn load(shadow_path: PathBuf) -> Result<Self> {
        let source_path = built_library_path();
        copy_library_to_shadow(source_path.as_path(), shadow_path.as_path())?;

        let load_result = (|| {
            let library = unsafe { Library::new(shadow_path.as_path()) }.map_err(|error| {
                ComponentLibraryError::Runtime(format!(
                    "failed to load hot library {}: {error}",
                    shadow_path.display()
                ))
            })?;

            let default_snapshot_fn = unsafe {
                *library
                    .get::<unsafe fn() -> String>(b"hot_showcase_default_snapshot")
                    .map_err(symbol_error)?
            };
            let frame_fn = unsafe {
                *library
                    .get::<unsafe fn(&egui::Context, &str, u64) -> String>(b"hot_showcase_frame")
                    .map_err(symbol_error)?
            };

            Ok(Self {
                _library: library,
                shadow_path: shadow_path.clone(),
                default_snapshot_fn,
                frame_fn,
            })
        })();

        if load_result.is_err() {
            let _ = remove_shadow_library_file(shadow_path.as_path());
        }

        load_result
    }

    fn default_snapshot(&self) -> String {
        unsafe { (self.default_snapshot_fn)() }
    }

    fn frame(
        &self,
        context: &egui::Context,
        snapshot_json: &str,
        reload_generation: u64,
    ) -> String {
        unsafe { (self.frame_fn)(context, snapshot_json, reload_generation) }
    }
}

fn remove_hot_lib(hot_lib: LoadedHotShowcaseLib) {
    let shadow_path = hot_lib.shadow_path.clone();
    drop(hot_lib);
    if let Err(error) = remove_shadow_library_file(shadow_path.as_path()) {
        eprintln!(
            "hot reload shadow cleanup failed for {}:\n{error}",
            shadow_path.display()
        );
    }
}

#[derive(Debug)]
enum HotBuildEvent {
    Building,
    Ready,
    Failed(String),
}

fn ensure_hot_library_built() -> Result {
    run_hot_library_build(manifest_dir()).map_err(|message| {
        ComponentLibraryError::Runtime(format!("hot startup build failed\n{message}"))
    })
}

fn spawn_build_watcher(context: egui::Context) -> Receiver<HotBuildEvent> {
    let manifest_dir = manifest_dir().to_path_buf();
    let (status_tx, status_rx) = mpsc::channel();

    thread::spawn(move || {
        let (event_tx, event_rx) = mpsc::channel();
        let mut watcher = match notify::recommended_watcher(move |result| {
            let _ = event_tx.send(result);
        }) {
            Ok(watcher) => watcher,
            Err(error) => {
                let _ = status_tx.send(HotBuildEvent::Failed(format!(
                    "watcher setup failed: {error}"
                )));
                return;
            }
        };

        if let Err(error) = configure_watch_paths(&mut watcher, manifest_dir.as_path()) {
            let _ = status_tx.send(HotBuildEvent::Failed(format!(
                "watch registration failed: {error}"
            )));
            return;
        }

        while let Ok(result) = event_rx.recv() {
            let event = match result {
                Ok(event) if is_relevant_event(&event) => event,
                Ok(_) => continue,
                Err(error) => {
                    let _ =
                        status_tx.send(HotBuildEvent::Failed(format!("file watch error: {error}")));
                    context.request_repaint();
                    continue;
                }
            };

            drain_debounced_events(&event_rx);
            let _ = status_tx.send(HotBuildEvent::Building);
            context.request_repaint();

            let status = match run_hot_library_build(manifest_dir.as_path()) {
                Ok(()) => HotBuildEvent::Ready,
                Err(message) => HotBuildEvent::Failed(message),
            };
            let _ = status_tx.send(status);
            context.request_repaint();

            if matches!(event.kind, EventKind::Remove(_)) {
                thread::sleep(Duration::from_millis(50));
            }
        }
    });

    status_rx
}

fn configure_watch_paths(
    watcher: &mut RecommendedWatcher,
    manifest_dir: &Path,
) -> notify::Result<()> {
    watcher.watch(&manifest_dir.join("src"), RecursiveMode::Recursive)?;

    let assets_dir = manifest_dir.join("assets");
    if assets_dir.exists() {
        watcher.watch(&assets_dir, RecursiveMode::Recursive)?;
    }

    watcher.watch(
        &manifest_dir.join("Cargo.toml"),
        RecursiveMode::NonRecursive,
    )?;

    let cargo_config = manifest_dir.join(".cargo/config.toml");
    if cargo_config.exists() {
        watcher.watch(&cargo_config, RecursiveMode::NonRecursive)?;
    }

    Ok(())
}

fn is_relevant_event(event: &Event) -> bool {
    if !matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) {
        return false;
    }

    event.paths.iter().any(|path| {
        path.components()
            .all(|component| component.as_os_str() != "target")
    })
}

fn drain_debounced_events(event_rx: &mpsc::Receiver<notify::Result<Event>>) {
    while let Ok(result) = event_rx.recv_timeout(Duration::from_millis(150)) {
        match result {
            Ok(event) if is_relevant_event(&event) => continue,
            Ok(_) => continue,
            Err(_) => continue,
        }
    }
}

fn run_hot_library_build(manifest_dir: &Path) -> std::result::Result<(), String> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .current_dir(manifest_dir)
        .args(["build", "--features", "showcase", "--lib"])
        .output()
        .map_err(|error| format!("failed to spawn cargo build: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stderr}\n{stdout}");
    Err(trim_error_output(combined.as_str()))
}

fn trim_error_output(message: &str) -> String {
    let lines = message
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let start = lines.len().saturating_sub(14);
    lines[start..].join("\n")
}

fn built_library_path() -> PathBuf {
    manifest_dir()
        .join("target/debug")
        .join(platform_library_name())
}

fn next_shadow_library_path(counter: &mut u64) -> Result<PathBuf> {
    let directory = manifest_dir().join("target/hot");
    fs::create_dir_all(&directory).map_err(|error| {
        ComponentLibraryError::Runtime(format!(
            "failed to create hot reload shadow dir {}: {error}",
            directory.display()
        ))
    })?;

    *counter = counter.saturating_add(1);
    Ok(directory.join(format!(
        "{}-shadow-{}-{}",
        library_stem(),
        std::process::id(),
        shadow_filename_suffix(*counter)
    )))
}

fn shadow_filename_suffix(counter: u64) -> String {
    format!("{counter}.{}", platform_library_extension())
}

fn cleanup_hot_shadow_directory() -> std::io::Result<()> {
    cleanup_hot_shadow_directory_at(manifest_dir().join("target/hot").as_path())
}

fn cleanup_hot_shadow_directory_at(directory: &Path) -> std::io::Result<()> {
    if !directory.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(is_hot_shadow_file_name)
        {
            let _ = fs::remove_file(&path);
        }
    }

    Ok(())
}

fn is_hot_shadow_file_name(file_name: &str) -> bool {
    file_name.starts_with(&format!("{}-shadow-", library_stem()))
        && file_name.ends_with(&format!(".{}", platform_library_extension()))
}

fn copy_library_to_shadow(source_path: &Path, shadow_path: &Path) -> Result {
    fs::copy(source_path, shadow_path).map_err(|error| {
        ComponentLibraryError::Runtime(format!(
            "failed to copy hot library from {} to {}: {error}",
            source_path.display(),
            shadow_path.display()
        ))
    })?;
    Ok(())
}

fn remove_shadow_library_file(shadow_path: &Path) -> std::io::Result<()> {
    match fs::remove_file(shadow_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn symbol_error(error: libloading::Error) -> ComponentLibraryError {
    ComponentLibraryError::Runtime(format!("failed to load hot library symbol: {error}"))
}

fn manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn platform_library_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "egui_component.dll"
    }
    #[cfg(target_os = "macos")]
    {
        "libegui_component.dylib"
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        "libegui_component.so"
    }
}

fn platform_library_extension() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "dll"
    }
    #[cfg(target_os = "macos")]
    {
        "dylib"
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        "so"
    }
}

fn library_stem() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "egui_component"
    }
    #[cfg(any(
        target_os = "macos",
        all(not(target_os = "windows"), not(target_os = "macos"))
    ))]
    {
        "libegui_component"
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cleanup_hot_shadow_directory_at, is_hot_shadow_file_name, library_stem,
        platform_library_extension,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn cleanup_hot_shadow_directory_removes_only_shadow_libraries() {
        let directory = unique_test_directory();
        fs::create_dir_all(&directory).unwrap();

        let shadow_file = directory.join(format!(
            "{}-shadow-1.{}",
            library_stem(),
            platform_library_extension()
        ));
        let unrelated_file = directory.join("keep-me.txt");
        fs::write(&shadow_file, b"shadow").unwrap();
        fs::write(&unrelated_file, b"keep").unwrap();

        cleanup_hot_shadow_directory_at(&directory).unwrap();

        assert!(!shadow_file.exists());
        assert!(unrelated_file.exists());

        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn shadow_file_name_detection_requires_expected_pattern() {
        assert!(is_hot_shadow_file_name(&format!(
            "{}-shadow-7.{}",
            library_stem(),
            platform_library_extension()
        )));
        assert!(!is_hot_shadow_file_name("egui_component.dll"));
        assert!(!is_hot_shadow_file_name("notes.txt"));
    }

    fn unique_test_directory() -> PathBuf {
        std::env::temp_dir().join(format!(
            "egui-component-hot-tests-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
}
