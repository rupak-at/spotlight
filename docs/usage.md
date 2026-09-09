# Use Spotlight

Launch **Spotlight** from Ubuntu's application menu, run `spotlight`, or press the configured shortcut after [shortcut setup](installation.md#configure-super--space).

## Search and open

Spotlight opens as a single 680×96 command bar centered on the current screen, with the search field already focused. Type an application, file, or folder name and the window expands to show a highlighted top result followed by grouped **Applications**, **Files**, and **Folders**, then recenters at 680×460. Choose **See all** beside a group to narrow the results; choose **All results** to return. It does not show suggestions, status controls, or an empty result panel before you type. Results update from the local index; no network service is involved.

Completed results and decoded icons stay visible while the next query or refreshed index is processed, then the complete replacement appears at once. Index-start notifications update status without triggering a duplicate search. Opening is temporarily disabled during that short replacement, which prevents an older result from launching for newer text.

Application icons come from Ubuntu's installed desktop entries and icon theme. Files use the system icon for their detected type; folders use the theme's folder icon. A type-specific fallback appears while loading or when no theme icon is available. File types are inferred from names without reading file contents.

| Key/action                               | Behavior                                   |
| ---------------------------------------- | ------------------------------------------ |
| Super+Space, or your configured shortcut | Show/hide the launcher on X11              |
| Up / Down                                | Move through results                       |
| Enter in the search field                | Open the selected result                   |
| Click a result                           | Open that result                           |
| Escape, with or without a query          | Hide the launcher immediately              |
| Ctrl+Tab / Ctrl+Shift+Tab in search      | Cycle grouped result types                 |
| Ctrl+,                                   | Open/close settings                        |
| Escape in settings                       | Hide the launcher immediately              |
| Drag the top strip                       | Move the launcher                          |
| Close button                             | Hide the window and keep Spotlight running |
| Power button in the footer               | Quit Spotlight                             |

Apps launch through GIO desktop entries. Files open in their default associated application, and folders open in your file manager. If an item was deleted or moved, refresh and search again; an opening error leaves the launcher visible.

## Drag results into another app

In the desktop app, search for a file or folder, then hold the left mouse button on its result row and drag it onto the destination. For browser uploads, drop onto the website’s file upload area. The destination decides which file types it accepts; folder support depends on the app or website. Drag one result at a time. Application results cannot be dragged. Native GTK transfers and Chromium uploads have been verified on X11. A standalone WebKitGTK target did not expose the drop as an uploadable file; Firefox and Wayland transfers remain unverified.

Spotlight offers a copy operation and leaves the original in place. A drag does not open the result. Release over an unsupported destination or press Escape during the native drag to cancel. Click or press Enter to open results normally afterward. Dragging is temporarily disabled while search results are being replaced. Removed files and files outside the current search folders produce an error. Browser-only preview cannot transfer real local files; use `npm run desktop`.

## Choose what gets indexed

Open the sliders button in the footer, then **Search & indexing**:

- **Search folders:** add absolute paths such as `/home/you/Documents`. Remove all roots for application-only search.
- **Excluded names:** comma-separated names such as `node_modules, target, dist`. These exclusions apply during traversal.
- **Include hidden items:** allow dotfiles/dotfolders except explicit exclusions.
- **Results:** maximum returned matches, from 1 to 100.
- **Folder depth:** recursion depth from 1 to 32; the default is 12. Set **0** for no depth limit.
- **Index limit:** maximum indexed files/folders, from 100 to 200,000; the default is 50,000. Set **0** for no item limit. Installed applications are added separately.

For all normal files under `/home/you`, add that root and set both limits to **0**, keeping **Include hidden items** off and the default exclusions. A root does not override these limits: a large alphabetically earlier folder can consume the item budget before later folders are scanned. Index notices are also shown in Search & indexing. Unlimited scanning uses more memory and takes longer; inaccessible items and symlinks remain skipped.

Click **Save changes** to apply. Invalid or unavailable newly entered roots produce an error. A saved root that later becomes unavailable stays in your configuration and is reported as unavailable on startup.

By default, existing Desktop, Documents, Downloads, Pictures, and `projects` folders under your home are searched. Symlinks are skipped. The launcher searches names, paths, and application descriptions; it does not search file contents.

## Appearance

Under **Appearance & shortcuts**, choose dark, light, or system appearance; an accent color; and compact result rows. Compact rows are enabled by default. The selected result keeps its path/description visible so similarly named items can be distinguished. Changes take effect when saved.

**Background opacity** ranges from 80% to 100%, with 94% as the default. It affects only the panel background; text and icons remain fully opaque. Dark mode uses neutral charcoal and silver, without glow effects. Native transparency requires compositing; actual blur of other desktop windows depends on the compositor and is not guaranteed by the webview's CSS backdrop filter. Set opacity to 100% for a solid background.

The native window starts at 680×96 and expands to 680×460 for results or Settings. Its size is fixed so the rounded translucent surface stays consistent. File/app icons follow Ubuntu's system icon theme; the appearance setting changes the launcher itself.

## Refresh and indexing notices

The rotating-arrow button requests a refresh. A background worker also watches discovered directories for changes. It rebuilds one snapshot at a time while searches continue using the previous snapshot. Changing search folders clears the old snapshot so removed roots are no longer offered.

Live watching covers up to 8,192 directories. Large trees, inaccessible folders, size/depth limits, and watcher failures produce a notice. Open **View details** or Search & indexing to inspect notices. If watch coverage is incomplete, a full rescan starts after 60 seconds without a refresh request; allow additional time for that scan to finish. Use Refresh for an immediate request. Item and depth limits still apply to every rescan.

Shortcut notices explain GNOME binding conflicts or Wayland setup. The launcher does not silently reassign your desktop shortcuts.

## Local data

Settings are saved in `~/.config/io.github.rupak.spotlight/settings.json` and the index cache in `~/.cache/io.github.rupak.spotlight/index.sqlite3`, unless XDG config/cache directories are customized. The SQLite database contains indexed names and paths, not file contents.

Stop Spotlight before manually editing settings or removing the cache. A missing cache is rebuilt at the next start. A malformed settings file produces a notice and temporary defaults; the original file is preserved until settings are explicitly saved.

For autostart, upgrades, removal, and troubleshooting, see [installation](installation.md). For source changes, see [development](development.md).
