# Local development

Use the same repository and dependencies described in [installation](installation.md). Node.js must be at least 22.22.1; Rust must be at least 1.94. Commit both lockfiles when dependencies change.

## Run the actual desktop application

```sh
npm ci
npm run desktop
```

This starts Vite at `http://127.0.0.1:1420` and the Rust/Tauri host. React/CSS changes update through Vite. Changes in the Rust project trigger native rebuilds through Tauri. Keep this terminal open while developing; Ctrl+C stops the development processes.

Quit an already running installed/release Spotlight first. The single-instance plugin deliberately prevents a second independent launcher from starting.

If port 1420 is occupied by a separately started `npm run dev`, stop that server before running `npm run desktop`. Tauri owns the development server through `beforeDevCommand` in `src-tauri/tauri.conf.json`.

## UI-only preview

```sh
npm run dev
```

Open the printed localhost address in a browser. This mode is labeled **UI preview with sample data**. It allows layout, keyboard, and settings-form work without a running GTK application. It cannot index your home folder, open applications, or display native desktop icons. Preview settings are in-memory and reset on reload.

Use the native application to verify system icons, shortcuts, filesystem watching, window behavior, and launching. A successful browser check is not a native integration test.

## Format, check, and commit

```sh
npm run format
npm run lint
npm test
npm run build
npm run release:check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
git add <relevant-paths>
git commit -m "feat: describe the completed change"
```

Choose checks relevant to the change; the list above is the complete verification set. `npm run build` runs TypeScript checks and builds frontend assets before the desktop host's linked workspace tests. For engine-only work, `cargo test -p spotlight-core` runs without a display or frontend build.

`npm ci` runs the `prepare` script to enable Husky. Each commit runs lint-staged:

- JavaScript/TypeScript: Prettier, then ESLint fixes/checks.
- JSON, CSS, Markdown, YAML, and HTML: Prettier. Unsupported formats are skipped.
- Rust: rustfmt on staged files, with child-module recursion disabled so unrelated files are not changed.

lint-staged stages the formatting results and protects partially staged work with a backup. A failed formatter/linter stops the commit; fix the reported problem and commit again. Tests are explicit commands and CI checks, not hidden long-running pre-commit steps. Keep the hook enabled. If a checkout has no hook configured, run `npm run prepare` inside the repository.

CI runs formatting checks, ESLint, frontend tests/build, Rust tests, and Clippy on Ubuntu 24.04. Check its result after pushing; the presence of a workflow file is not evidence that remote CI has passed.

## Project map

| Directory/file              | Responsibility                                                                  |
| --------------------------- | ------------------------------------------------------------------------------- |
| `src`                       | React UI, typed IPC client, search state, settings, fallback icons              |
| `crates/spotlight-core`     | Portable-within-Linux core models, settings, file scan, ranking, SQLite storage |
| `src-tauri/src/main.rs`     | Tauri commands, window lifecycle, shortcut registration                         |
| `src-tauri/src/linux.rs`    | Installed app discovery and GIO launching                                       |
| `src-tauri/src/icons.rs`    | GTK application/MIME icon resolution                                            |
| `src-tauri/src/worker.rs`   | Serial indexing, watcher events, snapshot publication                           |
| `src-tauri/tauri.conf.json` | Window, CSP, dev URL, and package settings                                      |
| `docs/architecture.md`      | Module boundaries, data flow, tradeoffs, release gates                          |

The core has no Tauri dependency and is reusable by a future CLI. The React UI never scans the filesystem or executes shell text. See [architecture](architecture.md) before adding a provider or new command.

## Use isolated development data

The normal app uses XDG config/cache locations. To keep a native test separate from your usual Spotlight settings, run on Linux:

```sh
XDG_CONFIG_HOME=/tmp/spotlight-dev/config \
XDG_CACHE_HOME=/tmp/spotlight-dev/cache \
npm run desktop
```

This isolates settings and cache, but does **not** change the default search roots under your actual home directory. Choose fixture roots in Settings for controlled tests. Single-instance identity is still shared, so quit other Spotlight processes first.

## Benchmark and package

```sh
cargo run --release -p spotlight-core --example benchmark
npm run package
```

The benchmark reports synthetic in-memory search latency for 50,000 entries. It does not measure disk scans, IPC, rendering, or show-to-focus latency. Record the environment and profile alongside any published results.

The package command builds `target/release/spotlight` and a `.deb` under `target/release/bundle/deb/`. Use the native binary or package to check the final assets; browser hot reload alone does not verify the release build.

`npm run release:check` verifies that `package.json`, the root of `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` use one version. When given a tag, it also requires the exact `v<version>` form. CI runs this check on every branch.

For a release, update those versions and refresh both lockfiles as needed. Commit and push the verified change, then push an annotated tag such as `v0.1.0`. The tag starts `.github/workflows/release.yml`, which checks the tag, builds on Ubuntu 24.04 amd64 with `tauri-apps/tauri-action`, and creates a public GitHub Release containing the `.deb`. Exact maintainer commands and end-user installation steps are in [installation](installation.md).

## Update the application icon

Edit the source at `assets/icon.svg`, then regenerate the four Linux icon sizes:

```sh
npm run tauri -- icon assets/icon.svg -o /tmp/spotlight-icons
cp /tmp/spotlight-icons/32x32.png /tmp/spotlight-icons/128x128.png \
  /tmp/spotlight-icons/128x128@2x.png /tmp/spotlight-icons/icon.png src-tauri/icons/
```

Rebuild the package to include the new icons. Commit the SVG and these PNG files together; other platforms' generated icons are not needed for this Ubuntu app.

## Documentation is part of a change

When setup, scripts, settings, shortcuts, architecture, packaging, or user behavior changes, update the relevant guide in the same logical change. Verify that examples match the actual package scripts, flags, paths, and settings labels. Update [verification](verification.md) with completed checks and retain explicit untested limits. This is also a persistent rule in [AGENTS.md](../AGENTS.md).
