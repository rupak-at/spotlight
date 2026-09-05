# Verification

Environment: Ubuntu 24.04.4, GNOME on X11, x86_64, Rust 1.94.0, Node.js 25.9.0. Validation is local; the CI workflow has been added but has not run on a remote service.

## Completed

- Core tests for ranking, filters, Unicode, bounded results, traversal exclusions, overlapping roots, symlink escapes, configuration, unavailable saved roots, and SQLite rollback.
- Four frontend tests covering stale asynchronous replies, errors, keyboard navigation, launching the selected ID, filtering, and opening settings.
- TypeScript checks and optimized frontend build.
- Native desktop compile check and workspace Clippy with warnings denied.
- Browser interaction checks at 760×560 and 560×460: search, filters, themes, exclusion editing, settings save, and no horizontal overflow.
- Release compilation and creation of `target/release/bundle/deb/Spotlight_0.1.0_amd64.deb`.

## Runtime validation in progress

Native smoke tests, the full linked workspace test run, and release-mode search measurements are being completed. A sandboxed GUI launch cannot access the display; native verification requires the desktop session.

## Limits

This is a first working implementation, not a completed production release qualification. Wayland focus/shortcuts, packaged installation on a clean machine, multiple monitors/scaling, long-running filesystem churn, assistive technologies, and real-world memory/battery measurements remain release gates. See [architecture](architecture.md).

Browser validation uses explicitly labeled sample data; it is not evidence that native file/app launching works. Synthetic engine benchmarks exclude IPC, rendering, window activation, and disk indexing.
