use gio::prelude::*;
use gtk::prelude::*;
use spotlight_core::{config, files::allowed_path, model::Entry, Result};
use std::path::Path;
use tauri::Manager;

use crate::state::AppState;

fn content_type(path: &Path) -> String {
    gio::content_type_guess(Some(path), &[]).0.to_string()
}

// Extensions keep choices for .js and .py independent even on systems which
// classify both as plain text. Extensionless files use their MIME content type.
fn association_key(path: &Path) -> String {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if !extension.is_empty() => format!("ext:{}", extension.to_lowercase()),
        _ => format!("mime:{}", content_type(path)),
    }
}

fn is_development_file(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(
        extension.as_str(),
        "js" | "jsx"
            | "ts"
            | "tsx"
            | "mjs"
            | "cjs"
            | "py"
            | "pyw"
            | "rs"
            | "css"
            | "scss"
            | "sass"
            | "less"
            | "html"
            | "htm"
            | "json"
            | "jsonc"
            | "toml"
            | "yaml"
            | "yml"
            | "xml"
            | "md"
            | "mdx"
            | "sh"
            | "bash"
            | "zsh"
            | "c"
            | "h"
            | "cpp"
            | "hpp"
            | "cc"
            | "go"
            | "java"
            | "kt"
            | "swift"
            | "rb"
            | "php"
            | "vue"
            | "svelte"
            | "sql"
            | "lua"
            | "r"
            | "dart"
            | "ipynb"
    ) || matches!(
        path.file_name().and_then(|value| value.to_str()),
        Some("Dockerfile" | "Makefile" | "Justfile" | ".gitignore" | ".env")
    )
}

fn default_app(path: &Path, mime: &str) -> Option<gio::AppInfo> {
    if is_development_file(path) {
        for id in [
            "antigravity.desktop",
            "code.desktop",
            "com.visualstudio.code.desktop",
            "codium.desktop",
        ] {
            if let Some(app) = gio::DesktopAppInfo::new(id).filter(|app| app.should_show()) {
                return Some(app.upcast());
            }
        }
    }
    if mime == "application/pdf" {
        if let Some(browser) = gio::AppInfo::default_for_uri_scheme("https") {
            return Some(browser);
        }
    }
    gio::AppInfo::default_for_type(mime, false)
}

// Some custom editor desktop entries (including this host's Antigravity entry)
// omit %f/%F/%u/%U. Add a file placeholder to an in-memory copy, never a shell.
fn file_capable_app(app: &gio::AppInfo) -> Result<gio::AppInfo> {
    if app.supports_files() || app.supports_uris() {
        return Ok(app.clone());
    }
    let desktop = app
        .clone()
        .downcast::<gio::DesktopAppInfo>()
        .map_err(|_| "This application cannot accept files. Choose another application.")?;
    let filename = desktop
        .filename()
        .ok_or("The application's desktop entry is unavailable.")?;
    let key = gio::glib::KeyFile::new();
    key.load_from_file(filename, gio::glib::KeyFileFlags::NONE)
        .map_err(|error| error.to_string())?;
    let exec = desktop
        .string("Exec")
        .ok_or("The application has no launch command.")?;
    key.set_string("Desktop Entry", "Exec", &format!("{exec} %F"));
    key.set_boolean("Desktop Entry", "DBusActivatable", false);
    gio::DesktopAppInfo::from_keyfile(&key)
        .map(|app| app.upcast())
        .ok_or_else(|| "Could not prepare this application to open a file.".into())
}

fn launch_file(app: &gio::AppInfo, path: &Path) -> Result<()> {
    file_capable_app(app)?
        .launch(&[gio::File::for_path(path)], None::<&gio::AppLaunchContext>)
        .map_err(|error| format!("Could not open the file in {}: {error}", app.display_name()))
}

fn choose_application(
    native: &gtk::ApplicationWindow,
    name: &str,
    mime: &str,
) -> Result<Option<(gio::AppInfo, bool)>> {
    let dialog = gtk::AppChooserDialog::for_content_type(
        Some(native),
        gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
        mime,
    );
    dialog.set_title("Open with");
    dialog.set_heading(&gio::glib::markup_escape_text(name));
    if let Ok(chooser) = dialog.widget().downcast::<gtk::AppChooserWidget>() {
        chooser.set_show_other(true);
        chooser.set_show_all(true);
    }
    let remember = gtk::CheckButton::with_label("Always use for this file type in Spotlight");
    dialog.content_area().pack_end(&remember, false, false, 8);
    dialog.show_all();
    let response = dialog.run();
    let selected = dialog.app_info();
    let save = remember.is_active();
    dialog.close();
    if response != gtk::ResponseType::Ok {
        return Ok(None);
    }
    let selected = selected.ok_or("Choose an application first.")?;
    Ok(Some((selected, save)))
}

// Runs on the GTK main thread. No index/settings lock is held while the modal
// dialog processes events. Revalidate the path and current scope after it closes.
pub fn open(handle: &tauri::AppHandle, entry: &Entry, choose: bool) -> Result<bool> {
    let state = handle.state::<AppState>();
    let settings = state.settings.read().unwrap().clone();
    let path = allowed_path(Path::new(&entry.path), &settings)?;
    if !path.is_file() {
        return Err("This file is no longer available. Refresh the index.".into());
    }
    let mime = content_type(&path);
    let key = association_key(&path);
    let preferred = settings
        .file_associations
        .get(&key)
        .and_then(|id| gio::DesktopAppInfo::new(id))
        .filter(|app| app.should_show())
        .map(|app| app.upcast());
    if !choose {
        if let Some(app) = preferred.or_else(|| default_app(&path, &mime)) {
            launch_file(&app, &path)?;
            return Ok(true);
        }
    }
    let window = handle
        .get_webview_window("main")
        .ok_or("Launcher window is unavailable.")?;
    let native = window.gtk_window().map_err(|error| error.to_string())?;
    let Some((selected, save)) = choose_application(&native, &entry.name, &mime)? else {
        native.present();
        return Ok(false);
    };
    let mut current = state.settings.write().unwrap();
    let validated = allowed_path(Path::new(&entry.path), &current)?;
    if validated != path || !validated.is_file() {
        return Err("This file changed while choosing an application. Search again.".into());
    }
    launch_file(&selected, &validated)?;
    if save {
        let id = selected
            .id()
            .ok_or("This application has no desktop ID and cannot be remembered.")?;
        let mut updated = current.clone();
        updated.file_associations.insert(key, id.to_string());
        config::save(&state.config_path, &updated).map_err(|error| {
            format!("The file opened, but the preference could not be saved: {error}")
        })?;
        *current = updated;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_source_files_without_mistaking_documents_and_photos() {
        for name in [
            "main.js",
            "main.PY",
            "lib.rs",
            "style.css",
            "Dockerfile",
            "App.tsx",
        ] {
            assert!(is_development_file(Path::new(name)), "{name}");
        }
        for name in ["photo.jpg", "report.pdf", "letter.docx", "movie.mp4"] {
            assert!(!is_development_file(Path::new(name)), "{name}");
        }
    }

    #[test]
    fn associations_are_case_insensitive_and_specific_to_extension() {
        assert_eq!(
            association_key(Path::new("a.PY")),
            association_key(Path::new("b.py"))
        );
        assert_ne!(
            association_key(Path::new("a.py")),
            association_key(Path::new("a.js"))
        );
        assert!(association_key(Path::new("README")).starts_with("mime:"));
    }
    #[test]
    fn launch_passes_exact_file_to_desktop_entries_with_and_without_placeholders() {
        use std::{
            fs, thread,
            time::{Duration, Instant},
        };
        let directory = tempfile::tempdir().unwrap();
        let script = directory.path().join("capture.sh");
        fs::write(
            &script,
            "output=$1\nshift\nprintf '%s\\n' \"$@\" > \"$output\"\n",
        )
        .unwrap();
        let path = directory.path().join("code file # % é $(not-a-command).py");
        fs::write(&path, "print('fixture')").unwrap();
        for (index, placeholder) in [" %F", ""].into_iter().enumerate() {
            let output = directory.path().join(format!("args-{index}"));
            let desktop = directory.path().join(format!("capture-{index}.desktop"));
            fs::write(&desktop, format!(
                "[Desktop Entry]\nType=Application\nName=Capture\nExec=/bin/sh {} {}{placeholder}\n",
                script.display(), output.display()
            )).unwrap();
            let app = gio::DesktopAppInfo::from_filename(desktop)
                .unwrap()
                .upcast();
            launch_file(&app, &path).unwrap();
            let expected = format!("{}\n", path.display());
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                if fs::read_to_string(&output).ok().as_ref() == Some(&expected) {
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "File argument was missing or altered: {:?}",
                    fs::read_to_string(&output)
                );
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    #[test]
    #[ignore = "requires a display; run with xvfb-run and --test-threads=1"]
    fn native_chooser_cancel_returns_without_selecting_an_application() {
        gtk::init().unwrap();
        let parent = gtk::ApplicationWindow::builder().build();
        gio::glib::idle_add_local_once(|| {
            let dialog = gtk::Window::list_toplevels()
                .into_iter()
                .find_map(|window| window.downcast::<gtk::AppChooserDialog>().ok())
                .unwrap();
            assert_eq!(dialog.title().as_deref(), Some("Open with"));
            dialog.response(gtk::ResponseType::Cancel);
        });
        assert!(choose_application(&parent, "example.py", "text/x-python")
            .unwrap()
            .is_none());
        parent.close();
    }
}
