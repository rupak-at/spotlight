# Spotlight for Ubuntu

A keyboard-first desktop launcher inspired by macOS Spotlight. Built for Ubuntu with **Tauri 2, Rust, React, and TypeScript**.

The Rust engine owns file discovery, ranking, and the SQLite index. The desktop host owns GNOME integration and launching. React owns presentation and interaction. See [the architecture](docs/architecture.md) for boundaries, data flow, and production hardening.

## What works

- Installed application discovery through GIO, including desktop entries exported by Snap and Flatpak.
- Case-insensitive filename/folder search with exact, prefix, substring, and subsequence ranking.
- Arrow-key navigation, Enter to open, Escape to clear/hide, Ctrl+, for settings, and Ctrl+Tab for result filters.
- A single resident process with an X11 global shortcut and a `--toggle` entry point for GNOME shortcuts.
- Background indexing, a transactional SQLite cache, and coalesced filesystem change notifications.
- Dark/light/system appearance, accent color, compact results, search roots, exclusions, limits, and shortcut customization.
- Explicit errors and notices for shortcut conflicts, unavailable folders, and incomplete indexing.

This is a working first version with production-oriented boundaries. See [architecture and release gates](docs/architecture.md) and [verification](docs/verification.md) for its current limits.

## Run on Ubuntu

Use Node.js 22+ and Rust 1.94+ with Cargo. Install Tauri's native prerequisites if missing:

```sh
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
npm ci
npm run desktop
```

The first Rust build takes longer while native dependencies compile. Subsequent builds reuse Cargo's cache. [Official Ubuntu prerequisites](https://v2.tauri.app/start/prerequisites/).

`npm run dev` opens a **browser-only UI preview with sample data**. Browsers cannot search your disk or launch desktop applications. Use `npm run desktop` for the real launcher.

## Super + Space

Ubuntu GNOME normally uses Super+Space to switch input sources. In **Settings → Keyboard → View and Customize Shortcuts → Typing**, reassign **Switch to next input source**, then restart Spotlight or save its shortcut setting again. Spotlight defaults to `Super+Space`; another option is `Ctrl+Alt+Space`. The app does not overwrite GNOME preferences.

On **Wayland**, set a GNOME custom keyboard shortcut for `/usr/bin/spotlight --toggle` after installing the package. Before installation, use the absolute path to `target/release/spotlight --toggle`. Global hotkeys from Tauri's underlying library are X11-only; Wayland focus remains controlled by the compositor. [Upstream hotkey support](https://github.com/tauri-apps/global-hotkey), [GNOME input shortcuts](https://help.gnome.org/gnome-help/keyboard-layouts.html).

Launching Spotlight a second time toggles the existing window. Closing the window hides it; use the power button in its footer to quit the process.

## Build and install

```sh
npm run package
sudo apt install ./target/release/bundle/deb/*.deb
spotlight
```

Or run `./target/release/spotlight` directly. The `.deb` provides the app-menu entry and icon. It is built against the local Ubuntu libraries; rebuild on the oldest Ubuntu version you intend to support. [Tauri Debian packaging](https://v2.tauri.app/distribute/debian/).

To start the installed launcher when you sign in:

```sh
mkdir -p ~/.config/autostart
cp packaging/spotlight-autostart.desktop ~/.config/autostart/
```

Remove that copied desktop file to disable autostart. Autostart and GNOME shortcut changes are opt-in setup steps.

## Customize and develop

Open settings with the sliders button or Ctrl+,. Search roots use absolute paths. The initial roots are existing Desktop, Documents, Downloads, Pictures, and `projects` directories under your home folder. Remove all roots for app-only search. Hidden files, dependency directories, and build output are skipped by default; symlinks are not followed. Search matches names and paths, not file contents.

Settings live in `$XDG_CONFIG_HOME/io.github.rupak.spotlight/settings.json` (normally `~/.config/...`). The disposable SQLite cache lives in `$XDG_CACHE_HOME/io.github.rupak.spotlight/index.sqlite3` (normally `~/.cache/...`). Stop the app before editing settings manually or removing a cache. Changes saved through the UI apply immediately.

| Change                           | Start here                                        |
| -------------------------------- | ------------------------------------------------- |
| Colors, spacing, typography      | `src/styles.css`                                  |
| Results and keyboard behavior    | `src/App.tsx`, `src/useSearch.ts`                 |
| Settings UI                      | `src/SettingsPanel.tsx`                           |
| Search ranking                   | `crates/spotlight-core/src/search.rs`             |
| Traversal/exclusions             | `crates/spotlight-core/src/files.rs`              |
| Desktop behavior and launching   | `src-tauri/src/main.rs`, `src-tauri/src/linux.rs` |
| Background indexing and watching | `src-tauri/src/worker.rs`                         |

## Checks

```sh
npm test
npm run build
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run --release -p spotlight-core --example benchmark
```

Build the frontend before workspace Rust tests because the desktop host embeds frontend assets. Core-only tests do not require a display or frontend: `cargo test -p spotlight-core`.

Live updates currently rebuild the bounded snapshot after relevant changes; they do not perform per-file database updates. At most 8,192 directories are watched. Very large or frequently changing trees should use narrower roots/exclusions; notices identify partial coverage. The Refresh button also rescans apps and files.

## Development workflow

Keep changes focused and commit every completed, verified logical change with a descriptive conventional commit message. Never commit generated builds, dependencies, or local search data.

## Why this stack

- **Rust:** background indexing and predictable control over memory and CPU work.
- **Tauri 2:** desktop integration using Ubuntu's system WebKitGTK webview, with React support and no bundled Chromium runtime. [Tauri overview](https://v2.tauri.app/start/)
- **React + TypeScript + Vite:** familiar UI development, explicit API types, and CSS-based customization.
- **SQLite:** a local, transactional metadata cache; no server or external account.

Rust stays in a running process. Search does not start a CLI process or scan the disk for each keystroke. Python is optional for future user-authored actions, and is not a runtime dependency. React Native does not offer an advantage for this webview-based Ubuntu UI.
