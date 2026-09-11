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
                size_bytes: None,
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
        Kind::Folder => {
            let path = allowed_path(Path::new(&entry.path), settings)?;
            open_folder(&path).map_err(|e| format!("Could not open {}: {e}", entry.name))
        }
        Kind::File => Err("Files must use the file application handler.".into()),
    }
}

// ShowFolders requests the directory contents, rather than selecting it in its parent.
fn folder_parameters(path: &Path) -> gio::glib::Variant {
    (vec![gio::File::for_path(path).uri().to_string()], "").to_variant()
}

fn open_folder(path: &Path) -> Result<()> {
    if !path.is_dir() {
        return Err("This folder is no longer available. Refresh the index.".into());
    }
    if let Ok(bus) = gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>) {
        if bus
            .call_sync(
                Some("org.freedesktop.FileManager1"),
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1",
                "ShowFolders",
                Some(&folder_parameters(path)),
                None,
                gio::DBusCallFlags::NONE,
                2000,
                None::<&gio::Cancellable>,
            )
            .is_ok()
        {
            return Ok(());
        }
    }
    // Desktops without FileManager1 still receive the exact folder as a GFile.
    let app = gio::AppInfo::default_for_type("inode/directory", false)
        .ok_or("No default file manager is configured for folders.")?;
    app.launch(&[gio::File::for_path(path)], None::<&gio::AppLaunchContext>)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_request_preserves_selected_directory_and_escapes_uri_characters() {
        for path in ["/home/example/Downloads", "/home/example/Downloads/a b#c%é"] {
            let parameters = folder_parameters(Path::new(path));
            assert_eq!(parameters.type_().as_str(), "(ass)");
            let (uris, startup_id) = parameters.get::<(Vec<String>, String)>().unwrap();
            assert_eq!(uris.len(), 1);
            assert!(startup_id.is_empty());
            assert_eq!(
                gio::File::for_uri(&uris[0]).path().unwrap(),
                Path::new(path)
            );
            assert!(!uris[0].contains(' '));
            assert!(!uris[0].contains('#'));
        }
    }

    #[test]
    fn missing_folder_is_rejected_before_contacting_file_manager() {
        assert!(open_folder(Path::new("/proc/spotlight-nonexistent-folder")).is_err());
    }
}
