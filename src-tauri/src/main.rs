mod drag;
mod icons;
mod linux;
mod state;
mod worker;

use gtk::prelude::*;
use spotlight_core::{
    config::{self, Settings},
    model::{Kind, SearchResponse},
    search::Index,
    Result,
};
use state::{AppState, Status};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc, Arc, Mutex, RwLock,
    },
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

fn resize_launcher(window: &tauri::WebviewWindow, expanded: bool) -> Result<()> {
    let height = if expanded { 460.0 } else { 96.0 };
    window
        .set_size(tauri::LogicalSize::new(680.0, height))
        .map_err(|error| error.to_string())?;
    window.center().map_err(|error| error.to_string())
}

fn toggle(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false) {
            let _ = window.hide();
        } else {
            // Map before focusing: Tao's queued show request can otherwise leave
            // the window hidden when its following focus request is checked.
            let _ = app.run_on_main_thread(move || {
                let _ = resize_launcher(&window, false);
                if let Ok(native) = window.gtk_window() {
                    native.show_all();
                    native.deiconify();
                    let timestamp = native
                        .window()
                        .and_then(|window| window.downcast::<gdkx11::X11Window>().ok())
                        .map(|window| gdkx11::functions::x11_get_server_time(&window))
                        .unwrap_or(0);
                    native.present_with_time(timestamp);
                }
                let _ = window.set_focus();
                let _ = window.emit("launcher-shown", ());
            });
        }
    }
}

fn bind_shortcut(app: &tauri::AppHandle, settings: &Settings) {
    let state = app.state::<AppState>();
    let mut status = state.status.lock().unwrap();
    if !state.shortcut_ready.load(Ordering::SeqCst) {
        status.shortcut_active = false;
        if status.shortcut_message.is_none() {
            status.shortcut_message = Some("Set a GNOME custom shortcut to run: spotlight --toggle. See README for Wayland setup.".into());
        }
        return;
    }
    let result = app
        .global_shortcut()
        .unregister_all()
        .and_then(|_| app.global_shortcut().register(settings.shortcut.as_str()));
    status.shortcut_active = result.is_ok();
    status.shortcut_message = result.err().map(|error| format!("Could not register {}: {error}. GNOME may reserve Super+Space for switching input sources. Reassign it in Settings → Keyboard, or choose another shortcut.", settings.shortcut));
}

#[tauri::command]
async fn search(
    app: tauri::AppHandle,
    query: String,
    kind: Option<Kind>,
) -> Result<SearchResponse> {
    if query.chars().count() > 256 {
        return Err("Search queries are limited to 256 characters.".into());
    }
    let state = app.state::<AppState>();
    let index = state.index.read().unwrap().clone();
    let limit = state.settings.read().unwrap().result_limit;
    tauri::async_runtime::spawn_blocking(move || index.search(&query, kind, limit))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> Settings {
    state.settings.read().unwrap().clone()
}

#[tauri::command]
fn get_status(state: tauri::State<AppState>) -> Status {
    state.status.lock().unwrap().clone()
}

#[tauri::command]
fn get_icon(state: tauri::State<AppState>, id: String) -> Option<String> {
    let entry = state.index.read().unwrap().get(&id).cloned()?;
    let mut cache = state.icons.lock().unwrap();
    if let Some(icon) = cache.get(&id) {
        return icon.clone();
    }
    let icon = icons::resolve(&entry);
    if cache.len() >= 256 {
        cache.clear();
    }
    cache.insert(id, icon.clone());
    icon
}

#[tauri::command]
fn set_launcher_expanded(app: tauri::AppHandle, expanded: bool) -> Result<()> {
    let window = app
        .get_webview_window("main")
        .ok_or("Launcher window is unavailable.")?;
    resize_launcher(&window, expanded)
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<Settings> {
    settings.validate()?;
    Shortcut::from_str(&settings.shortcut).map_err(|error| format!("Invalid shortcut: {error}"))?;
    let state = app.state::<AppState>();
    {
        let mut current = state.settings.write().unwrap();
        config::save(&state.config_path, &settings)?;
        if current.index_key() != settings.index_key() {
            state.revision.fetch_add(1, Ordering::SeqCst);
            *state.index.write().unwrap() = Arc::new(Index::default());
            let mut status = state.status.lock().unwrap();
            status.indexing = true;
            status.total = 0;
        }
        *current = settings.clone();
    }
    bind_shortcut(&app, &settings);
    let _ = state.refresh.try_send(());
    let _ = app.emit("index-status-changed", ());
    Ok(settings)
}

#[tauri::command]
fn refresh_index(state: tauri::State<AppState>) {
    let _ = state.refresh.try_send(());
}

#[tauri::command]
fn launch(app: tauri::AppHandle, id: String) -> Result<()> {
    let state = app.state::<AppState>();
    let settings = state.settings.read().unwrap();
    let entry = state
        .index
        .read()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or("This result is no longer indexed. Search again.")?;
    linux::launch(&entry, &settings)?;
    hide_window(app.clone())
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) -> Result<()> {
    app.get_webview_window("main")
        .ok_or("Launcher window is unavailable.")?
        .hide()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn quit(app: tauri::AppHandle) {
    app.exit(0);
}

fn main() {
    tauri::Builder::default()
        .manage(drag::DragState::default())
        .plugin(tauri_plugin_single_instance::init(|app, _, _| toggle(app)))
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                drag::setup(&window)?;
                // WebKitGTK's natural request is 200 px high. Let the native
                // window follow the compact 96 px launcher size instead.
                window.with_webview(|webview| webview.inner().set_size_request(1, 1))?;
                if let Ok(native) = window.gtk_window() {
                    native.set_position(gtk::WindowPosition::CenterAlways);
                }
            }
            let config_path = app.path().app_config_dir()?.join("settings.json");
            let cache_path = app.path().app_cache_dir()?.join("index.sqlite3");
            let mut warnings = Vec::new();
            let settings = config::load(&config_path).unwrap_or_else(|error| {
                warnings.push(format!("{error} Using defaults; your existing settings file is preserved until you save."));
                Settings::default()
            });
            let (sender, receiver) = mpsc::sync_channel(1);
            let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".into());
            let wayland = session == "wayland" || std::env::var_os("WAYLAND_DISPLAY").is_some();
            app.manage(AppState {
                icons: Mutex::new(HashMap::new()),
                index: RwLock::new(Arc::new(Index::default())), settings: RwLock::new(settings.clone()),
                status: Mutex::new(Status { indexing: true, session, warnings, ..Status::default() }),
                revision: AtomicU64::new(0), shortcut_ready: AtomicBool::new(false), refresh: sender.clone(), config_path, cache_path,
            });
            if !wayland {
                let last_shortcut = Mutex::new(None::<Instant>);
                let plugin = tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, _, event| {
                        if event.state() != ShortcutState::Pressed {
                            return;
                        }
                        let now = Instant::now();
                        let mut last = last_shortcut.lock().unwrap();
                        if last.is_some_and(|previous| now.duration_since(previous) < Duration::from_millis(450)) {
                            return;
                        }
                        *last = Some(now);
                        toggle(app);
                    })
                    .build();
                match app.handle().plugin(plugin) {
                    Ok(()) => app.state::<AppState>().shortcut_ready.store(true, Ordering::SeqCst),
                    Err(error) => app.state::<AppState>().status.lock().unwrap().shortcut_message = Some(format!("Global shortcuts unavailable: {error}. Configure a GNOME shortcut for spotlight --toggle.")),
                }
            }
            bind_shortcut(app.handle(), &settings);
            worker::start(app.handle().clone(), receiver);
            let _ = sender.try_send(());
            if !std::env::args().any(|arg| arg == "--background") { toggle(app.handle()); }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event { api.prevent_close(); let _ = window.hide(); }
        })
        .invoke_handler(tauri::generate_handler![search, get_settings, get_status, get_icon, set_launcher_expanded, save_settings, refresh_index, launch, drag::start_file_drag, hide_window, quit])
        .run(tauri::generate_context!())
        .expect("Could not start Spotlight. Run from a terminal to inspect desktop or WebKitGTK errors.");
}
