# Verification

Environment: Ubuntu 24.04.4, GNOME on X11, x86_64, Rust 1.94.0, Node.js 25.9.0. Validation is local; the CI workflow has been added but has not run on a remote service.

## Completed

- Eight Rust tests for ranking, filters, Unicode, bounded results, traversal exclusions, overlapping roots, symlink escapes, configuration, unavailable saved roots, and SQLite rollback. The full linked workspace test command passes.
- Five frontend tests covering atomic replacement and late asynchronous replies, no empty-query search/suggestions, errors, keyboard navigation, launching the selected ID, filtering, and immediate Escape dismissal from both search and settings.
- TypeScript checks and optimized frontend build.
- Native desktop compile check and workspace Clippy with warnings denied.
- Prettier/rustfmt checks and ESLint pass. The staged-file formatting hook ran successfully during real commits.
- Browser interaction checks include the collapsed search-only surface at 96 px and a fractional-scale 77 CSS px, expanded results, dark/light themes, exclusion editing, background opacity, settings save, centered search controls, and no overflow.
- Release compilation and creation of `target/release/bundle/deb/Spotlight_0.1.0_amd64.deb`.

## Native runtime checks

The release binary was run inside the Ubuntu X11 desktop using isolated XDG config/cache directories and a temporary search folder. Checks confirmed:

- Real installed application discovery and filename/folder search through Rust IPC.
- Native themed application and folder icons, transparent background, a 680×96 search-only window, and 680×460 expansion for results.
- Automatic SQLite index updates when a fixture file was created and deleted.
- The configured `Ctrl+Alt+Space` shortcut showing and focusing the launcher, including reopening a hidden window.
- Escape hiding the native launcher immediately while a query was present.
- Enter opening the temporary fixture folder in Files and launching Calculator through GIO; Spotlight hid after successful opening.

Super+Space itself was not reassigned on the host, because GNOME reserves it for input-source switching. Follow [installation](installation.md) to configure it. `.deb` contents were inspected for the executable, desktop entry, icons, and runtime dependencies; installation on a clean machine remains untested.

## Search measurements

Command: `cargo run --release -p spotlight-core --example benchmark`.

| Measurement                             | Result   |
| --------------------------------------- | -------- |
| Synthetic entries                       | 50,000   |
| Index construction                      | 10.66 ms |
| Warm query median, 100 measured queries | 2.06 ms  |
| Warm query p95                          | 3.25 ms  |
| Maximum measured warm query             | 3.42 ms  |

The query mix includes exact/prefix, subsequence, substring, multiple tokens, no matches, and empty queries. These are local engine measurements, not end-to-end performance guarantees. IPC, rendering, window activation, disk indexing, idle process memory, and battery use need separate measurement.

## Limits

This is a first working implementation, not a completed production release qualification. Wayland focus/shortcuts, packaged installation on a clean machine, multiple monitors/scaling, long-running filesystem churn, assistive technologies, and real-world memory/battery measurements remain release gates. See [architecture](architecture.md).

Browser validation uses explicitly labeled sample data; it is not evidence that native file/app launching works. Synthetic engine benchmarks exclude IPC, rendering, window activation, and disk indexing.
