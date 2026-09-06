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

## Choose what gets indexed

Open the sliders button in the footer, then **Search & indexing**:

- **Search folders:** add absolute paths such as `/home/you/Documents`. Remove all roots for application-only search.
- **Excluded names:** comma-separated names such as `node_modules, target, dist`. These exclusions apply during traversal.
- **Include hidden items:** allow dotfiles/dotfolders except explicit exclusions.
- **Results:** maximum returned matches, from 1 to 100.
- **Folder depth:** recursion depth from 1 to 32; the default is 12.
- **Index limit:** maximum indexed files/folders, from 100 to 200,000; the default is 50,000. Installed applications are added separately.

Click **Save changes** to apply. Invalid or unavailable newly entered roots produce an error. A saved root that later becomes unavailable stays in your configuration and is reported as unavailable on startup.

By default, existing Desktop, Documents, Downloads, Pictures, and `projects` folders under your home are searched. Symlinks are skipped. The launcher searches names, paths, and application descriptions; it does not search file contents.

## Appearance

Under **Appearance & shortcuts**, choose dark, light, or system appearance; an accent color; and compact result rows. Compact rows are enabled by default. The selected result keeps its path/description visible so similarly named items can be distinguished. Changes take effect when saved.

**Background opacity** ranges from 80% to 100%, with 94% as the default. It affects only the panel background; text and icons remain fully opaque. Dark mode uses neutral charcoal and silver, without glow effects. Native transparency requires compositing; actual blur of other desktop windows depends on the compositor and is not guaranteed by the webview's CSS backdrop filter. Set opacity to 100% for a solid background.

The native window starts at 680×96 and expands to 680×460 for results or Settings. Its size is fixed so the rounded translucent surface stays consistent. File/app icons follow Ubuntu's system icon theme; the appearance setting changes the launcher itself.

## Refresh and indexing notices

The rotating-arrow button requests a refresh. A background worker also watches discovered directories for changes. It rebuilds one bounded snapshot at a time while searches continue using the previous snapshot. Changing search folders clears the old snapshot so removed roots are no longer offered.

Live watching covers up to 8,192 directories. Large trees, inaccessible folders, size/depth limits, and watcher failures produce a notice. Open **View details**, adjust search roots/limits if needed, and use Refresh for changes outside live coverage.

Shortcut notices explain GNOME binding conflicts or Wayland setup. The launcher does not silently reassign your desktop shortcuts.

## Local data

Settings are saved in `~/.config/io.github.rupak.spotlight/settings.json` and the index cache in `~/.cache/io.github.rupak.spotlight/index.sqlite3`, unless XDG config/cache directories are customized. The SQLite database contains indexed names and paths, not file contents.

Stop Spotlight before manually editing settings or removing the cache. A missing cache is rebuilt at the next start. A malformed settings file produces a notice and temporary defaults; the original file is preserved until settings are explicitly saved.

For autostart, upgrades, removal, and troubleshooting, see [installation](installation.md). For source changes, see [development](development.md).
