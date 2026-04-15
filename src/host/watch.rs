use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
};

use notify_debouncer_mini::notify::{
    event::ModifyKind, Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode,
    Result as NotifyResult, Watcher,
};

pub struct ReloadWatcher {
    pending_reload_rx: Receiver<()>,
    tracked_files: Arc<Mutex<BTreeSet<PathBuf>>>,
    watched_paths: BTreeMap<PathBuf, RecursiveMode>,
    watcher: RecommendedWatcher,
}

impl std::fmt::Debug for ReloadWatcher {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReloadWatcher")
            .field("watched_paths", &self.watched_paths)
            .finish_non_exhaustive()
    }
}

impl ReloadWatcher {
    pub fn new<F>(request_repaint: F) -> NotifyResult<Self>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let request_repaint = Arc::new(request_repaint);
        let tracked_files = Arc::new(Mutex::new(BTreeSet::new()));
        let tracked_files_for_events = Arc::clone(&tracked_files);
        let request_repaint_for_events = Arc::clone(&request_repaint);
        let (pending_reload_tx, pending_reload_rx) = mpsc::sync_channel(1);
        let watcher = RecommendedWatcher::new(
            move |result| match result {
                Ok(event) => {
                    if !event_requires_reload(&event) {
                        return;
                    }
                    let tracked_files = tracked_files_for_events
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    if !event_matches_tracked_files(&event, &tracked_files) {
                        return;
                    }
                    queue_reload(&pending_reload_tx, request_repaint_for_events.as_ref());
                }
                Err(_error) => {
                    queue_reload(&pending_reload_tx, request_repaint_for_events.as_ref());
                }
            },
            NotifyConfig::default(),
        )?;

        Ok(Self {
            pending_reload_rx,
            tracked_files,
            watched_paths: BTreeMap::new(),
            watcher,
        })
    }

    pub fn set_tracked_files(
        &mut self,
        tracked_files: impl IntoIterator<Item = PathBuf>,
    ) -> NotifyResult<()> {
        let tracked_files = tracked_files
            .into_iter()
            .map(normalize_watch_path)
            .collect::<BTreeSet<_>>();
        let next_watched_paths = watch_paths_for_files(&tracked_files);

        for path in self.watched_paths.keys() {
            if !next_watched_paths.contains_key(path) {
                self.watcher.unwatch(path)?;
            }
        }

        for (path, mode) in &next_watched_paths {
            if self.watched_paths.get(path) == Some(mode) {
                continue;
            }
            if self.watched_paths.contains_key(path) {
                self.watcher.unwatch(path)?;
            }
            self.watcher.watch(path, *mode)?;
        }

        *self
            .tracked_files
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = tracked_files;
        self.watched_paths = next_watched_paths;
        Ok(())
    }

    pub fn take_pending_reload(&mut self) -> bool {
        let mut pending = false;
        while self.pending_reload_rx.try_recv().is_ok() {
            pending = true;
        }
        pending
    }
}

pub fn normalize_watch_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        std::fs::canonicalize(&path).unwrap_or(path)
    } else {
        let absolute = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path);
        std::fs::canonicalize(&absolute).unwrap_or(absolute)
    }
}

pub fn watch_paths_for_files(files: &BTreeSet<PathBuf>) -> BTreeMap<PathBuf, RecursiveMode> {
    let mut watched_paths = BTreeMap::new();
    for path in files {
        if path.exists() {
            watched_paths.insert(path.clone(), RecursiveMode::NonRecursive);
            continue;
        }

        watched_paths.insert(
            nearest_existing_watch_ancestor(path),
            RecursiveMode::NonRecursive,
        );
    }
    watched_paths
}

fn nearest_existing_watch_ancestor(path: &Path) -> PathBuf {
    for ancestor in path.ancestors() {
        if ancestor.exists() {
            return ancestor.to_path_buf();
        }
    }

    path.to_path_buf()
}

pub fn event_requires_reload(event: &Event) -> bool {
    if event.need_rescan() {
        return true;
    }

    if event.paths.is_empty() {
        return false;
    }

    match event.kind {
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

pub fn event_matches_tracked_files(event: &Event, tracked_files: &BTreeSet<PathBuf>) -> bool {
    if tracked_files.is_empty() {
        return false;
    }
    if event.need_rescan() {
        return true;
    }
    event
        .paths
        .iter()
        .map(|path| normalize_watch_path(path.to_path_buf()))
        .any(|path| tracked_files.contains(&path))
}

fn queue_reload(pending_reload_tx: &SyncSender<()>, request_repaint: &dyn Fn()) {
    if pending_reload_tx.try_send(()).is_ok() {
        request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use notify_debouncer_mini::notify::event::{
        AccessKind, AccessMode, CreateKind, DataChange, Flag, MetadataKind, ModifyKind, RemoveKind,
        RenameMode,
    };

    use super::*;

    #[test]
    fn real_change_predicate_matches_example_runner_behavior() {
        let path = std::path::Path::new("/tmp/example.tsx").to_path_buf();

        assert!(event_requires_reload(
            &Event::new(EventKind::Any).add_path(path.clone())
        ));
        assert!(event_requires_reload(
            &Event::new(EventKind::Create(CreateKind::File)).add_path(path.clone())
        ));
        assert!(event_requires_reload(
            &Event::new(EventKind::Remove(RemoveKind::File)).add_path(path.clone())
        ));
        assert!(event_requires_reload(
            &Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Content)))
                .add_path(path.clone())
        ));
        assert!(event_requires_reload(
            &Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Any)))
                .add_path(path.clone())
        ));
        assert!(!event_requires_reload(
            &Event::new(EventKind::Access(AccessKind::Close(AccessMode::Write)))
                .add_path(path.clone())
        ));
        assert!(!event_requires_reload(
            &Event::new(EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any))).add_path(path)
        ));
    }

    #[test]
    fn tracked_file_matching_is_exact_and_rescan_friendly() {
        let tracked = BTreeSet::from([
            normalize_watch_path(Path::new("/tmp/a.tsx").to_path_buf()),
            normalize_watch_path(Path::new("/tmp/missing.tsx").to_path_buf()),
        ]);

        assert!(event_matches_tracked_files(
            &Event::new(EventKind::Create(CreateKind::File))
                .add_path(Path::new("/tmp/a.tsx").to_path_buf()),
            &tracked,
        ));
        assert!(!event_matches_tracked_files(
            &Event::new(EventKind::Create(CreateKind::File))
                .add_path(Path::new("/tmp/b.tsx").to_path_buf()),
            &tracked,
        ));
        assert!(event_matches_tracked_files(
            &Event::new(EventKind::Other).set_flag(Flag::Rescan),
            &tracked,
        ));
    }

    #[test]
    fn watch_paths_follow_missing_files_by_parent_directory() {
        let temp = tempfile::tempdir().expect("temp dir should exist");
        let existing = temp.path().join("existing.tsx");
        std::fs::write(&existing, "export {}").expect("existing file should be written");
        let missing = temp.path().join("missing.tsx");

        let watched = watch_paths_for_files(&BTreeSet::from([
            normalize_watch_path(existing.clone()),
            normalize_watch_path(missing.clone()),
        ]));

        assert_eq!(
            watched.get(&normalize_watch_path(existing)),
            Some(&RecursiveMode::NonRecursive)
        );
        assert_eq!(watched.get(temp.path()), Some(&RecursiveMode::NonRecursive));
        assert!(
            !watched.contains_key(&normalize_watch_path(missing)),
            "missing files are watched via the parent directory"
        );
    }

    #[test]
    fn watch_paths_for_nested_missing_imports_fallback_to_nearest_existing_ancestor() {
        let temp = tempfile::tempdir().expect("temp dir should exist");
        let existing = temp.path().join("existing.tsx");
        let nested = temp.path().join("nested");
        std::fs::write(&existing, "export {}").expect("existing file should be written");
        std::fs::create_dir_all(&nested).expect("nested directory should be created");

        let missing = nested.join("deeper").join("missing.tsx");
        let missing_parent = missing
            .parent()
            .expect("missing test fixture should have a parent")
            .to_path_buf();

        let watched = watch_paths_for_files(&BTreeSet::from([
            normalize_watch_path(existing.clone()),
            normalize_watch_path(missing.clone()),
        ]));

        assert_eq!(
            watched.get(&normalize_watch_path(existing)),
            Some(&RecursiveMode::NonRecursive)
        );
        assert_eq!(
            watched.get(&normalize_watch_path(nested)),
            Some(&RecursiveMode::NonRecursive)
        );
        assert!(
            !watched.contains_key(&normalize_watch_path(missing_parent)),
            "missing intermediate parent should not be registered directly"
        );
        assert!(
            !watched.contains_key(&normalize_watch_path(missing)),
            "missing file should be watched through the nearest existing ancestor"
        );
    }

    #[test]
    fn reload_bursts_collapse_into_one_pending_signal_and_one_repaint_edge() {
        let (pending_reload_tx, pending_reload_rx) = mpsc::sync_channel(1);
        let repaint_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let repaint_count_for_callback = Arc::clone(&repaint_count);
        let request_repaint = move || {
            repaint_count_for_callback.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        };

        queue_reload(&pending_reload_tx, &request_repaint);
        queue_reload(&pending_reload_tx, &request_repaint);
        queue_reload(&pending_reload_tx, &request_repaint);

        let mut pending = 0usize;
        while pending_reload_rx.try_recv().is_ok() {
            pending += 1;
        }

        assert_eq!(pending, 1);
        assert_eq!(repaint_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
