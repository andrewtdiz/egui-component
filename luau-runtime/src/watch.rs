use std::{
    collections::BTreeSet,
    env, fs,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use log::{debug, trace, warn};
use notify::{
    event::ModifyKind, Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode,
    Watcher,
};

use crate::runtime::{ReloadQueue, RuntimeError};

/// Watches a Luau project rooted at one root script path and enqueues coalesced dirty module paths.
pub struct RootScriptWatcher {
    _watcher: RecommendedWatcher,
    watched_root: PathBuf,
    watched_dir: PathBuf,
}

impl RootScriptWatcher {
    /// Start watching the root script path. The callback is invoked whenever the project becomes dirty.
    pub fn new(
        root_path: impl Into<PathBuf>,
        reload_queue: ReloadQueue,
        on_dirty: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, RuntimeError> {
        let watched_root = normalize_path(&root_path.into())?;
        let watched_dir = resolve_watched_dir(&watched_root)?;

        let dirty_root = watched_root.clone();
        let callback = Arc::new(on_dirty);
        let callback_root = watched_root.clone();
        let callback_dir = watched_dir.clone();
        let callback_queue = reload_queue.clone();
        let callback_signal = Arc::clone(&callback);

        let mut watcher = RecommendedWatcher::new(
            move |event_result| {
                handle_notify_event(
                    &callback_root,
                    &callback_dir,
                    &callback_queue,
                    callback_signal.as_ref(),
                    event_result,
                );
            },
            NotifyConfig::default(),
        )
        .map_err(|err| {
            RuntimeError::io(format!(
                "failed to create watcher for {}: {err}",
                dirty_root.display()
            ))
        })?;

        watcher
            .watch(&watched_dir, RecursiveMode::Recursive)
            .map_err(|err| {
                RuntimeError::io(format!("failed to watch {}: {err}", watched_dir.display()))
            })?;

        Ok(Self {
            _watcher: watcher,
            watched_root,
            watched_dir,
        })
    }

    /// Return the normalized root path tracked by this watcher.
    pub fn watched_root(&self) -> &Path {
        &self.watched_root
    }

    /// Return the normalized directory tracked by this watcher.
    pub fn watched_dir(&self) -> &Path {
        &self.watched_dir
    }
}

fn resolve_watched_dir(watched_root: &Path) -> Result<PathBuf, RuntimeError> {
    let root_parent = watched_root.parent().ok_or_else(|| {
        RuntimeError::config(format!(
            "root script path `{}` has no parent directory",
            watched_root.display()
        ))
    })?;

    let mut current = Some(root_parent);
    while let Some(candidate) = current {
        if candidate.join(".luaurc").is_file() {
            return Ok(candidate.to_path_buf());
        }
        current = candidate.parent();
    }

    Ok(root_parent.to_path_buf())
}

fn handle_notify_event(
    watched_root: &Path,
    watched_dir: &Path,
    reload_queue: &ReloadQueue,
    on_dirty: &(dyn Fn() + Send + Sync),
    event_result: notify::Result<Event>,
) {
    match event_result {
        Ok(event) => {
            let dirty_paths = dirty_module_paths(watched_root, watched_dir, &event);
            if dirty_paths.is_empty() {
                trace!(
                    "luau runtime watcher ignored event for {}: {:?}",
                    watched_root.display(),
                    event
                );
                return;
            }

            for path in &dirty_paths {
                debug!(
                    "luau_runtime event=file_changed root={} path={}",
                    watched_root.display(),
                    path.display()
                );
                reload_queue.push(path.clone());
            }
            on_dirty();
        }
        Err(err) => {
            warn!(
                "luau runtime watcher error for {}: {}",
                watched_root.display(),
                err
            );
        }
    }
}

fn dirty_module_paths(watched_root: &Path, watched_dir: &Path, event: &Event) -> Vec<PathBuf> {
    if event.need_rescan() {
        return vec![watched_root.to_path_buf()];
    }

    if !event_kind_marks_root_dirty(&event.kind) {
        return Vec::new();
    }

    let mut dirty_paths = BTreeSet::new();
    for path in &event.paths {
        let Ok(normalized) = normalize_path(path) else {
            continue;
        };
        if normalized.starts_with(watched_dir)
            && normalized.extension().is_some_and(|ext| ext == "luau")
        {
            dirty_paths.insert(normalized);
        }
    }

    dirty_paths.into_iter().collect()
}

fn event_kind_marks_root_dirty(kind: &EventKind) -> bool {
    match kind {
        EventKind::Any => true,
        EventKind::Create(_) | EventKind::Remove(_) => true,
        EventKind::Modify(ModifyKind::Data(_))
        | EventKind::Modify(ModifyKind::Name(_))
        | EventKind::Modify(ModifyKind::Any)
        | EventKind::Modify(ModifyKind::Other) => true,
        EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_)) | EventKind::Other => {
            false
        }
    }
}

fn normalize_path(path: &Path) -> Result<PathBuf, RuntimeError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|err| RuntimeError::io(format!("failed to resolve current directory: {err}")))?
            .join(path)
    };

    let lexical = normalize_lexical_path(&absolute);
    match fs::canonicalize(&lexical) {
        Ok(canonical) => Ok(canonical),
        Err(_) => Ok(lexical),
    }
}

fn normalize_lexical_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{
        CreateKind, DataChange, EventAttributes, ModifyKind, RemoveKind, RenameMode,
    };
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("luau-runtime crate lives inside workspace root")
            .to_path_buf()
    }

    fn runtime_example_path(relative: &str) -> PathBuf {
        workspace_root()
            .join("examples")
            .join("runtime-luau")
            .join(relative)
    }

    #[test]
    fn modify_create_remove_and_rename_mark_project_modules_dirty() {
        let root = normalize_path(&runtime_example_path("apps/demo/main.luau")).unwrap();
        let watched_dir = resolve_watched_dir(&root).unwrap();
        let leaf = normalize_path(&runtime_example_path("leaf.luau")).unwrap();

        for kind in [
            EventKind::Create(CreateKind::File),
            EventKind::Remove(RemoveKind::File),
            EventKind::Modify(ModifyKind::Any),
            EventKind::Modify(ModifyKind::Data(DataChange::Any)),
            EventKind::Modify(ModifyKind::Name(RenameMode::Any)),
        ] {
            let event = Event {
                kind,
                paths: vec![leaf.clone()],
                attrs: EventAttributes::default(),
            };
            assert_eq!(
                dirty_module_paths(&root, &watched_dir, &event),
                vec![leaf.clone()]
            );
        }
    }

    #[test]
    fn metadata_only_events_are_ignored() {
        let root = normalize_path(&runtime_example_path("apps/demo/main.luau")).unwrap();
        let watched_dir = resolve_watched_dir(&root).unwrap();
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Metadata(notify::event::MetadataKind::Any)),
            paths: vec![root.clone()],
            attrs: notify::event::EventAttributes::default(),
        };

        assert!(dirty_module_paths(&root, &watched_dir, &event).is_empty());
    }

    #[test]
    fn non_luau_or_out_of_tree_paths_are_ignored() {
        let root = normalize_path(&runtime_example_path("apps/demo/main.luau")).unwrap();
        let watched_dir = resolve_watched_dir(&root).unwrap();
        let png = normalize_path(&runtime_example_path("preview.png")).unwrap();
        let elsewhere = normalize_path(&workspace_root().join("Cargo.toml")).unwrap();
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Any),
            paths: vec![png, elsewhere],
            attrs: EventAttributes::default(),
        };

        assert!(dirty_module_paths(&root, &watched_dir, &event).is_empty());
    }

    #[test]
    fn rescan_marks_root_dirty() {
        let root = normalize_path(&runtime_example_path("apps/demo/main.luau")).unwrap();
        let watched_dir = resolve_watched_dir(&root).unwrap();
        let mut event = Event {
            kind: EventKind::Any,
            paths: Vec::new(),
            attrs: EventAttributes::default(),
        };
        event.attrs.set_flag(notify::event::Flag::Rescan);

        assert_eq!(dirty_module_paths(&root, &watched_dir, &event), vec![root]);
    }

    #[test]
    fn duplicate_paths_are_coalesced_per_event() {
        let root = normalize_path(&runtime_example_path("apps/demo/main.luau")).unwrap();
        let watched_dir = resolve_watched_dir(&root).unwrap();
        let leaf = normalize_path(&runtime_example_path("leaf.luau")).unwrap();
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Any),
            paths: vec![leaf.clone(), leaf.clone(), leaf.clone()],
            attrs: EventAttributes::default(),
        };

        assert_eq!(dirty_module_paths(&root, &watched_dir, &event), vec![leaf]);
    }

    #[test]
    fn nested_root_uses_luaurc_project_root_for_watch_scope() {
        let root = normalize_path(&runtime_example_path("apps/showcase/main.luau")).unwrap();
        let watched_dir = resolve_watched_dir(&root).unwrap();
        let expected = normalize_path(&runtime_example_path("")).unwrap();

        assert_eq!(watched_dir, expected);
    }

    #[test]
    fn nested_root_detects_sibling_modules_under_project_root() {
        let root = normalize_path(&runtime_example_path("apps/showcase/main.luau")).unwrap();
        let watched_dir = resolve_watched_dir(&root).unwrap();
        let helper =
            normalize_path(&runtime_example_path("ui/recipes/profile_panel.luau")).unwrap();
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Any),
            paths: vec![helper.clone()],
            attrs: EventAttributes::default(),
        };

        assert_eq!(
            dirty_module_paths(&root, &watched_dir, &event),
            vec![helper]
        );
    }
}
