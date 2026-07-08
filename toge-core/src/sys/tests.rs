use super::*;
#[cfg(target_os = "linux")]
use notify::event::{EventAttributes, ModifyKind, RenameMode};
#[cfg(target_os = "linux")]
use notify::{Event, EventKind};
use std::fs;
use std::path::Path;
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};

/// A fake watcher for testing higher-level code without touching inotify.
pub struct FakeWatcher {
    pub watches: Vec<String>,
    pub pending: Vec<WatchEvent>,
}

impl FakeWatcher {
    pub fn new() -> Self {
        Self {
            watches: Vec::new(),
            pending: Vec::new(),
        }
    }

    pub fn push(&mut self, event: WatchEvent) {
        self.pending.push(event);
    }
}

impl FsWatcher for FakeWatcher {
    fn watch(&mut self, path: &Path) -> io::Result<()> {
        self.watches.push(path.to_string_lossy().to_string());
        Ok(())
    }

    fn unwatch(&mut self, path: &Path) -> io::Result<()> {
        let s = path.to_string_lossy().to_string();
        self.watches.retain(|w| w != &s);
        Ok(())
    }

    fn poll_events(&mut self) -> io::Result<Vec<WatchEvent>> {
        Ok(std::mem::take(&mut self.pending))
    }
}

#[test]
fn fake_watcher_records_watches() {
    let mut w = FakeWatcher::new();
    w.watch(Path::new("/tmp")).unwrap();
    w.watch(Path::new("/home")).unwrap();
    assert_eq!(w.watches, vec!["/tmp", "/home"]);
}

#[test]
fn fake_watcher_returns_pending_events() {
    let mut w = FakeWatcher::new();
    w.push(WatchEvent::Create {
        path: "/tmp/x".into(),
        is_dir: false,
    });
    let events = w.poll_events().unwrap();
    assert_eq!(events.len(), 1);
    assert!(w.poll_events().unwrap().is_empty());
}

#[test]
#[cfg(target_os = "linux")]
fn inotify_watcher_can_be_constructed() {
    // May fail inside restricted containers; allow it.
    let _ = InotifyWatcher::new();
}

#[test]
#[cfg(target_os = "linux")]
fn inotify_watcher_trait_object() {
    fn takes_watcher(_: &mut dyn FsWatcher) {}
    if let Ok(mut w) = InotifyWatcher::new() {
        takes_watcher(&mut w);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn notify_maps_rename_from_to_delete() {
    let events = InotifyWatcher::map_event(Event {
        kind: EventKind::Modify(ModifyKind::Name(RenameMode::From)),
        paths: vec!["/tmp/movie.mkv".into()],
        attrs: EventAttributes::new(),
    });

    assert_eq!(
        events,
        vec![WatchEvent::Delete {
            path: "/tmp/movie.mkv".into()
        }]
    );
}

#[test]
#[cfg(target_os = "linux")]
fn notify_maps_rename_both_to_move() {
    let events = InotifyWatcher::map_event(Event {
        kind: EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
        paths: vec!["/tmp/movie.part".into(), "/tmp/movie.mkv".into()],
        attrs: EventAttributes::new(),
    });

    assert_eq!(
        events,
        vec![WatchEvent::Move {
            from: "/tmp/movie.part".into(),
            to: "/tmp/movie.mkv".into()
        }]
    );
}

#[test]
#[cfg(target_os = "linux")]
fn inotify_watcher_observes_real_create_and_delete_events() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("live-watch.mkv");

    let Ok(mut watcher) = InotifyWatcher::new() else {
        return;
    };
    watcher.watch(dir.path()).expect("watch temp dir");

    fs::write(&path, b"hello").unwrap();

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut saw_create = false;
    while Instant::now() < deadline {
        let events = watcher.poll_events().unwrap();
        if events.iter().any(|event| {
            matches!(
                event,
                WatchEvent::Create { path: event_path, is_dir: false }
                    if event_path == path.to_string_lossy().as_ref()
            )
        }) {
            saw_create = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(saw_create, "expected create event for {}", path.display());

    fs::remove_file(&path).unwrap();

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut saw_delete = false;
    while Instant::now() < deadline {
        let events = watcher.poll_events().unwrap();
        if events.iter().any(|event| {
            matches!(
                event,
                WatchEvent::Delete { path: event_path }
                    if event_path == path.to_string_lossy().as_ref()
            )
        }) {
            saw_delete = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(saw_delete, "expected delete event for {}", path.display());
}

#[test]
fn fake_watcher_unwatch_removes_watch() {
    let mut w = FakeWatcher::new();
    w.watch(Path::new("/tmp/a")).unwrap();
    w.watch(Path::new("/tmp/b")).unwrap();
    assert_eq!(w.watches, vec!["/tmp/a", "/tmp/b"]);

    w.unwatch(Path::new("/tmp/a")).unwrap();
    assert_eq!(w.watches, vec!["/tmp/b"]);
}

#[test]
fn fake_watcher_unwatch_is_idempotent() {
    let mut w = FakeWatcher::new();
    w.watch(Path::new("/tmp/a")).unwrap();
    w.unwatch(Path::new("/tmp/nonexistent")).unwrap();
    assert_eq!(w.watches, vec!["/tmp/a"]);
}

#[test]
fn simulate_delete_event_unwatches_directory() {
    let mut watcher = FakeWatcher::new();
    let mut watched: Vec<String> = Vec::new();

    watcher.watch(Path::new("/project/src")).unwrap();
    watched.push("/project/src".to_string());
    watcher.watch(Path::new("/project/lib")).unwrap();
    watched.push("/project/lib".to_string());
    assert_eq!(watcher.watches.len(), 2);

    let event = WatchEvent::Delete {
        path: "/project/src".into(),
    };

    match &event {
        WatchEvent::Delete { path } => {
            watched.retain(|w| w != path);
            let _ = watcher.unwatch(&Path::new(path));
        }
        _ => {}
    }

    assert_eq!(watcher.watches, vec!["/project/lib"]);
    assert_eq!(watched, vec!["/project/lib"]);
}

#[test]
fn simulate_move_event_unwatches_source() {
    let mut watcher = FakeWatcher::new();
    let mut watched: Vec<String> = Vec::new();

    watcher.watch(Path::new("/project/old_dir")).unwrap();
    watched.push("/project/old_dir".to_string());
    assert_eq!(watcher.watches, vec!["/project/old_dir"]);

    let event = WatchEvent::Move {
        from: "/project/old_dir".into(),
        to: "/project/new_dir".into(),
    };

    match &event {
        WatchEvent::Move { from, .. } => {
            watched.retain(|w| w != from);
            let _ = watcher.unwatch(&Path::new(from));
        }
        _ => {}
    }

    assert_eq!(watcher.watches, Vec::<String>::new());
    assert_eq!(watched, Vec::<String>::new());
}

#[test]
fn simulate_move_event_watches_destination_if_exists() {
    let dir = tempfile::tempdir().unwrap();
    let old_path = dir.path().join("old_dir");
    let new_path = dir.path().join("new_dir");
    fs::create_dir(&old_path).unwrap();
    fs::create_dir(&new_path).unwrap();

    let mut watcher = FakeWatcher::new();
    let mut watched: Vec<String> = Vec::new();

    watcher.watch(&old_path).unwrap();
    watched.push(old_path.to_string_lossy().to_string());

    let event = WatchEvent::Move {
        from: old_path.to_string_lossy().to_string(),
        to: new_path.to_string_lossy().to_string(),
    };

    match &event {
        WatchEvent::Move { from, to } => {
            watched.retain(|w| w != from);
            let _ = watcher.unwatch(&Path::new(from));

            if Path::new(to).is_dir() {
                let _ = watcher.watch(&Path::new(to));
                watched.push(to.clone());
            }
        }
        _ => {}
    }

    assert_eq!(watcher.watches.len(), 1);
    assert!(watcher.watches.contains(&new_path.to_string_lossy().to_string()));
    assert_eq!(watched.len(), 1);
    assert!(watched.contains(&new_path.to_string_lossy().to_string()));
}

#[test]
fn simulate_overflow_reindex_does_not_accumulate_watches() {
    let mut watcher = FakeWatcher::new();

    let dirs = vec![
        "/project/src".to_string(),
        "/project/lib".to_string(),
        "/project/test".to_string(),
    ];

    for dir in &dirs {
        watcher.watch(Path::new(dir)).unwrap();
    }
    assert_eq!(watcher.watches.len(), 3);

    let new_dirs = vec![
        "/project/src".to_string(),
        "/project/lib".to_string(),
        "/project/test".to_string(),
        "/project/docs".to_string(),
    ];

    for dir in &dirs {
        let _ = watcher.unwatch(&Path::new(dir));
    }
    assert_eq!(watcher.watches.len(), 0);

    for dir in &new_dirs {
        watcher.watch(Path::new(dir)).unwrap();
    }
    assert_eq!(watcher.watches.len(), 4);

    assert!(watcher.watches.contains(&"/project/src".to_string()));
    assert!(watcher.watches.contains(&"/project/docs".to_string()));
}

#[test]
fn simulate_creating_nested_directory_watches_it() {
    let mut watcher = FakeWatcher::new();
    let mut watched: Vec<String> = Vec::new();

    watcher.watch(Path::new("/project")).unwrap();
    watched.push("/project".to_string());

    let event = WatchEvent::Create {
        path: "/project/src".into(),
        is_dir: true,
    };

    match &event {
        WatchEvent::Create { path, is_dir } => {
            if *is_dir {
                let _ = watcher.watch(&Path::new(path));
                watched.push(path.clone());
            }
        }
        _ => {}
    }

    assert_eq!(watcher.watches.len(), 2);
    assert!(watcher.watches.contains(&"/project".to_string()));
    assert!(watcher.watches.contains(&"/project/src".to_string()));
}

#[test]
fn simulate_file_create_does_not_add_watch() {
    let mut watcher = FakeWatcher::new();
    let mut watched: Vec<String> = Vec::new();

    watcher.watch(Path::new("/project")).unwrap();
    watched.push("/project".to_string());

    let event = WatchEvent::Create {
        path: "/project/file.txt".into(),
        is_dir: false,
    };

    match &event {
        WatchEvent::Create { path, is_dir } => {
            if *is_dir {
                let _ = watcher.watch(&Path::new(path));
                watched.push(path.clone());
            }
        }
        _ => {}
    }

    assert_eq!(watcher.watches.len(), 1);
    assert_eq!(watched.len(), 1);
}
