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
    pub config_path: PathBuf,
    pub cache_path: PathBuf,
}
