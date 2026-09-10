use serde::Serialize;
use spotlight_core::{config::Settings, search::Index};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64},
        mpsc::SyncSender,
        Arc, Mutex, RwLock,
    },
};

#[derive(Clone, Default, Serialize)]
pub struct Status {
    pub indexing: bool,
    pub total: usize,
    pub apps: usize,
    pub truncated: bool,
    pub warnings: Vec<String>,
    pub shortcut_message: Option<String>,
    pub shortcut_active: bool,
    pub session: String,
    pub last_scan_ms: f64,
    pub watched_directories: usize,
}

pub struct AppState {
    pub icons: Mutex<HashMap<String, Option<String>>>,
    pub index: RwLock<Arc<Index>>,
    pub settings: RwLock<Settings>,
    pub status: Mutex<Status>,
    pub revision: AtomicU64,
    pub shortcut_ready: AtomicBool,
    pub refresh: SyncSender<()>,
    pub last_discovery: Mutex<Option<std::time::Instant>>,
    pub config_path: PathBuf,
    pub cache_path: PathBuf,
}

// Called under the discovery mutex so concurrent misses cannot bypass the cooldown.
pub fn request_discovery(
    last: &mut Option<std::time::Instant>,
    sender: &SyncSender<()>,
    indexing: bool,
    now: std::time::Instant,
) {
    if !indexing
        && last.is_none_or(|time| now.duration_since(time) >= std::time::Duration::from_secs(60))
        && sender.try_send(()).is_ok()
    {
        *last = Some(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn discovery_coalesces_misses_and_respects_busy_worker_and_cooldown() {
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let mut last = None;
        let now = Instant::now();
        request_discovery(&mut last, &sender, true, now);
        assert!(receiver.try_recv().is_err());
        assert!(last.is_none());
        request_discovery(&mut last, &sender, false, now);
        receiver.try_recv().unwrap();
        request_discovery(&mut last, &sender, false, now + Duration::from_secs(59));
        assert!(receiver.try_recv().is_err());
        request_discovery(&mut last, &sender, false, now + Duration::from_secs(60));
        receiver.try_recv().unwrap();
        // A queued filesystem/manual refresh is enough; do not block or reset the timer.
        sender.try_send(()).unwrap();
        request_discovery(&mut last, &sender, false, now + Duration::from_secs(120));
        assert_eq!(last, Some(now + Duration::from_secs(60)));
        receiver.try_recv().unwrap();
        assert!(receiver.try_recv().is_err());
    }
}
