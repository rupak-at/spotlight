# Spotlight for Ubuntu

A keyboard-first desktop launcher inspired by macOS Spotlight. Built for Ubuntu with **Tauri 2, Rust, React, and TypeScript**.

The Rust engine owns file discovery, ranking, and the SQLite index. The desktop host owns GNOME integration and launching. React owns presentation and interaction. See [the architecture](docs/architecture.md) for boundaries, data flow, and production hardening.

## Status

First working version under development. This repository uses production-oriented boundaries; it is not yet a production-certified release.

## Development workflow

Keep changes focused and commit every completed, verified logical change with a descriptive conventional commit message. Never commit generated builds, dependencies, or local search data.

## Why this stack

- **Rust:** background indexing and predictable control over memory and CPU work.
- **Tauri 2:** desktop integration using Ubuntu's system WebKitGTK webview, with React support and no bundled Chromium runtime. [Tauri overview](https://v2.tauri.app/start/)
- **React + TypeScript + Vite:** familiar UI development, explicit API types, and CSS-based customization.
- **SQLite:** a local, transactional metadata cache; no server or external account.

Rust stays in a running process. Search does not start a CLI process or scan the disk for each keystroke. Python is optional for future user-authored actions, and is not a runtime dependency. React Native does not offer an advantage for this webview-based Ubuntu UI.
