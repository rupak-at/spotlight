use gio::prelude::*;
use spotlight_core::{
    config::Settings,
    files::allowed_path,
    model::{Entry, Kind},
    Result,
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub fn applications() -> Vec<Entry> {
    let mut seen = HashSet::new();
    let mut entries: Vec<_> = gio::AppInfo::all()
        .into_iter()
        .filter(|app| app.should_show())
        .filter_map(|app| {
            let id = app.id()?.to_string();
            if !seen.insert(id.clone()) {
                return None;
            }
            Some(Entry {
                id: format!("app:{id}"),
                name: app.display_name().to_string(),
                path: id,
                kind: Kind::App,
                keywords: format!(
                    "{} {}",
                    app.description().unwrap_or_default(),
                    app.executable().display()
                ),
            })
        })
        .collect();
    entries.sort_by_key(|entry| entry.name.to_lowercase());
    entries
}

pub fn application_dirs() -> Vec<PathBuf> {
    let mut bases: Vec<_> = std::env::var_os("XDG_DATA_DIRS")
        .map(|value| std::env::split_paths(&value).collect())
        .unwrap_or_else(|| vec!["/usr/local/share".into(), "/usr/share".into()]);
    if let Some(base) = std::env::var_os("XDG_DATA_HOME") {
        bases.push(base.into());
    } else if let Some(home) = std::env::var_os("HOME") {
        bases.push(PathBuf::from(home).join(".local/share"));
    }
    bases
        .into_iter()
        .map(|base| base.join("applications"))
        .filter(|path| path.is_dir())
        .collect()
}

pub fn launch(entry: &Entry, settings: &Settings) -> Result<()> {
    match entry.kind {
        Kind::App => {
            let app = gio::DesktopAppInfo::new(&entry.path)
                .ok_or("This application is no longer installed. Refresh the index.")?;
            if !app.should_show() {
                return Err("This application is no longer visible in your desktop menu.".into());
            }
            app.launch(&[], None::<&gio::AppLaunchContext>)
                .map_err(|e| format!("Could not launch {}: {e}", entry.name))
        }
        Kind::File | Kind::Folder => {
            let path = allowed_path(Path::new(&entry.path), settings)?;
            let uri = gio::File::for_path(path).uri();
            gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>)
                .map_err(|e| format!("Could not open {}: {e}", entry.name))
        }
    }
}
