# Verification

Environment: Ubuntu 24.04.4, GNOME on X11, x86_64, Rust 1.94.0, Node.js 25.9.0. Validation is local; the CI workflow has been added but has not run on a remote service.

## Completed

- Eight Rust tests for ranking, filters, Unicode, bounded results, traversal exclusions, overlapping roots, symlink escapes, configuration, unavailable saved roots, and SQLite rollback. The full linked workspace test command passes.
- Seven frontend tests covering atomic replacement and late asynchronous replies, no empty-query search/suggestions, errors, initial and post-Settings input focus, keyboard navigation, launching the selected ID, filtering, immediate Escape dismissal from both search and settings, native drag requests, suppression of post-drag opening, stale/app drag rejection, and drag errors.
- TypeScript checks and optimized frontend build.
- Native desktop compile check and workspace Clippy with warnings denied.
- Prettier/rustfmt checks and ESLint pass. The staged-file formatting hook ran successfully during real commits.
- Browser interaction checks include the collapsed command bar at 96 px and a fractional-scale 77 CSS px, grouped results and group switching, rapid query replacement, dark/light themes, Settings return focus, centered search controls, and no overflow.
- Release compilation and creation of `target/release/bundle/deb/Spotlight_0.1.0_amd64.deb`.
- Release metadata alignment check, including rejection of a tag that does not match the application version.

## Native runtime checks

The release binary was run inside the Ubuntu X11 desktop using isolated XDG config/cache directories and a temporary search folder. Checks confirmed:

- Real installed application discovery and filename/folder search through Rust IPC.
- Native themed application and folder icons, transparent background, a centered 680×96 search-only window, and centered 680×460 expansion for results.
- Automatic SQLite index updates when a fixture file was created and deleted.
- The configured `Ctrl+Alt+Space` shortcut showing and focusing the launcher, including reopening a hidden window.
- Escape hiding the native launcher immediately while a query was present.
- Enter opening the temporary fixture folder in Files and launching Calculator through GIO; Spotlight hid after successful opening.

The original native checks did not reassign Super+Space. A later GNOME X11 diagnostic, after the input-source shortcut had been reassigned, found that right Super+Space opened the installed launcher on its first press while left Super+Space did not. Temporarily setting `org.gnome.mutter overlay-key` to an empty string made left Super+Space open and focus it on the first press; the original `Super_L` value was restored after the test. This isolates the standalone Activities binding as an additional conflict on this host. Follow [installation](installation.md) for the opt-in setup and its effect on Super alone. `.deb` contents were inspected for the executable, desktop entry, icons, license metadata, and runtime dependencies; installation on a clean machine remains untested. The tag-triggered GitHub Release workflow is configured locally; its first remote run must still be observed.

## Outbound drag checks

The new drag integration was tested with the desktop build in an isolated X11 virtual display and a separate GTK drop target. A real pointer drag transferred the expected `text/uri-list` file URI, including correct encoding of spaces, `#`, and accented characters in the filename. A separate Chromium window running a local upload page received the matching `File` object and read its expected contents. Chromium used a temporary profile and `--no-sandbox` for this isolated local-page test because the host blocks its user namespaces.

A standalone WebKitGTK upload page received URI/HTML data but an empty `FileList`; file upload compatibility with that target remains unresolved. Firefox and Wayland drag transfers were not verified.

Manual regression steps:

1. Run `npm run desktop`, search for a file, and drag its row into a file manager or an accepting browser upload area. Confirm the original remains and the destination receives the file.
2. Repeat with a folder in a destination that supports folders.
3. Cancel a drag with Escape or release outside an accepting target, then drag again and click/Enter to open normally.
4. Rename or remove a result before dragging; confirm an error instead of a transfer. Change the query and immediately try dragging a pending result; confirm no old file is transferred.
5. Verify application rows cannot be dragged. Repeat native checks on Wayland and in the browsers you use; destination support varies.

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
