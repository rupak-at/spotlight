use crate::{linux, state::AppState};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use spotlight_core::{files, repository, search::Index};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{
        atomic::Ordering,
        mpsc::{Receiver, SyncSender},
        Arc,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager};

fn watcher(sender: SyncSender<()>) -> notify::Result<RecommendedWatcher> {
    notify::recommended_watcher(move |event: notify::Result<Event>| {
        // Access notifications caused by our own scan must never schedule another scan.
        if event.as_ref().map_or(true, |event| {
            matches!(
                event.kind,
                EventKind::Create(_)
                    | EventKind::Remove(_)
                    | EventKind::Modify(_)
                    | EventKind::Other
            )
        }) {
            let _ = sender.try_send(());
        }
    })
}

pub fn start(app: AppHandle, receiver: Receiver<()>) {
    std::thread::spawn(move || {
        let state = app.state::<AppState>();
        let mut watcher = watcher(state.refresh.clone());
        let mut watched = HashSet::<PathBuf>::new();
        let mut startup_warnings = state.status.lock().unwrap().warnings.clone();
        // Loading a cached snapshot must not delay window creation.
        let settings = state.settings.read().unwrap().clone();
        match repository::load(&state.cache_path, &settings.index_key()) {
            Ok(entries) => {
                let current = state.settings.read().unwrap();
                if current.index_key() == settings.index_key() {
                    let total = entries.len();
                    *state.index.write().unwrap() = Arc::new(Index::new(entries));
                    state.status.lock().unwrap().total = total;
                }
            }
            Err(error) => startup_warnings.push(format!("Index cache could not be read: {error}")),
        }
        while receiver.recv().is_ok() {
            let start = Instant::now();
            let (settings, revision) = {
                let settings = state.settings.read().unwrap();
                (settings.clone(), state.revision.load(Ordering::SeqCst))
            };
            state.status.lock().unwrap().indexing = true;
            let _ = app.emit("index-status-changed", ());
            let mut scan = files::scan(&settings, || {
                state.revision.load(Ordering::SeqCst) != revision
            });
            if state.revision.load(Ordering::SeqCst) != revision {
                continue;
            }
            let mut entries = linux::applications();
            let apps = entries.len();
            entries.append(&mut scan.entries);
            if scan.truncated {
                scan.warnings.push("Some items were skipped at your index size or depth limit. Adjust limits or choose smaller search folders.".into());
            }
            let desired: HashSet<_> = scan
                .directories
                .into_iter()
                .chain(linux::application_dirs())
                .take(8192)
                .collect();
            if desired.len() == 8192 {
                scan.warnings.push("Live watching is limited to 8,192 directories. Use Refresh for changes beyond that limit.".into());
            }
            match &mut watcher {
                Ok(watcher) => {
                    for removed in watched.difference(&desired) {
                        let _ = watcher.unwatch(removed);
                    }
                    watched.retain(|path| desired.contains(path));
                    let mut failures = 0;
                    for added in desired.difference(&watched.clone()) {
                        if watcher.watch(added, RecursiveMode::NonRecursive).is_ok() {
                            watched.insert(added.clone());
                        } else {
                            failures += 1;
                        }
                    }
                    if failures > 0 {
                        scan.warnings.push(format!("Live watching could not cover {failures} directories. Use Refresh after changes there."));
                    }
                }
                Err(error) => scan.warnings.push(format!(
                    "Live watching unavailable: {error}. Use Refresh after changes."
                )),
            }
            if let Err(error) =
                repository::replace(&state.cache_path, &settings.index_key(), &entries)
            {
                scan.warnings
                    .push(format!("Could not save index cache: {error}"));
            }
            // Disk work finishes before taking the publication lock. An obsolete
            // cache is harmless: its configuration key will not match on restart.
            let current = state.settings.read().unwrap();
            if state.revision.load(Ordering::SeqCst) != revision {
                continue;
            }
            let total = entries.len();
            *state.index.write().unwrap() = Arc::new(Index::new(entries));
            state.icons.lock().unwrap().clear();
            {
                let mut status = state.status.lock().unwrap();
                status.indexing = false;
                status.total = total;
                status.apps = apps;
                status.truncated = scan.truncated;
                status.warnings = startup_warnings
                    .iter()
                    .cloned()
                    .chain(scan.warnings)
                    .collect();
                status.last_scan_ms = start.elapsed().as_secs_f64() * 1000.0;
                status.watched_directories = watched.len();
            }
            drop(current);
            let _ = app.emit("index-changed", ());
            // A bounded delay coalesces filesystem bursts without starving refreshes.
            std::thread::sleep(Duration::from_millis(700));
        }
    });
}
