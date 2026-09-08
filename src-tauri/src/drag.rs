use gio::prelude::*;
use gtk::prelude::*;
use spotlight_core::{files::allowed_path, model::Kind, Result};
use std::{path::Path, sync::Mutex};
use tauri::{Emitter, Manager};

use crate::state::AppState;

#[derive(Default)]
pub struct DragState(Mutex<Option<String>>);

pub fn setup(window: &tauri::WebviewWindow) -> Result<()> {
    let native = window.gtk_window().map_err(|error| error.to_string())?;
    let app = window.app_handle().clone();
    native.connect_drag_data_get(move |_, _, data, _, _| {
        if let Some(uri) = app.state::<DragState>().0.lock().unwrap().as_ref() {
            data.set_uris(&[uri]);
        }
    });
    let app = window.app_handle().clone();
    native.connect_drag_end(move |_, _| {
        *app.state::<DragState>().0.lock().unwrap() = None;
        let _ = app.emit("file-drag-ended", ());
    });
    Ok(())
}

// Synchronous Tauri commands run on the main thread, as required by GTK.
#[tauri::command]
pub fn start_file_drag(app: tauri::AppHandle, id: String) -> Result<()> {
    let state = app.state::<AppState>();
    let settings = state.settings.read().unwrap();
    let entry = state
        .index
        .read()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or("This result is no longer indexed. Search again.")?;
    if entry.kind == Kind::App {
        return Err("Only files and folders can be dragged.".into());
    }
    let path = allowed_path(Path::new(&entry.path), &settings)?;
    let uri = gio::File::for_path(path).uri().to_string();
    let window = app
        .get_webview_window("main")
        .ok_or("Launcher window is unavailable.")?;
    let native = window.gtk_window().map_err(|error| error.to_string())?;
    let drag = app.state::<DragState>();
    {
        let mut current = drag.0.lock().unwrap();
        if current.is_some() {
            return Err("A file drag is already in progress.".into());
        }
        *current = Some(uri);
    }
    let targets = gtk::TargetList::new(&[gtk::TargetEntry::new(
        "text/uri-list",
        gtk::TargetFlags::empty(),
        0,
    )]);
    if let Some(context) =
        native.drag_begin_with_coordinates(&targets, gtk::gdk::DragAction::COPY, 1, None, -1, -1)
    {
        context.drag_set_icon_default();
        Ok(())
    } else {
        *drag.0.lock().unwrap() = None;
        Err("Could not start the file drag. Try dragging the result again.".into())
    }
}
