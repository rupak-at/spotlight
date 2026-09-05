# Install on Ubuntu

Spotlight is a local desktop application for Ubuntu 24.04, built with Tauri, Rust, and React. X11 is the primary tested session. On Wayland, use a GNOME custom shortcut and see the focus limitation below.

## Install a published package

If the repository owner has attached a `.deb` to a GitHub Release, download the package matching your Ubuntu version and architecture. The currently built package is `Spotlight_0.1.0_amd64.deb` for x86_64 Ubuntu 24.04.

From the folder containing the downloaded package:

```sh
sudo apt install ./Spotlight_0.1.0_amd64.deb
spotlight
```

You can also open **Spotlight** from Ubuntu's application menu. Package installation does not require Node.js, Rust, or a local development server. A Git push alone does not publish a downloadable package; the owner must attach the `.deb` to a release separately.

## Build after cloning the repository

Replace `<repository-url>` with this project's Git URL. A remote has not been configured in this checkout, so no repository address is assumed here.

```sh
git clone <repository-url> spotlight
cd spotlight
```

Install Git, a C build toolchain, and the native libraries:

```sh
sudo apt update
sudo apt install git build-essential curl wget file pkg-config \
  libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev
```

Install **Node.js 22.22.1 or newer** and **Rust 1.94 or newer**. If you already use nvm, `nvm install 22` and `nvm use 22` select a supported Node 22 release. Install Rust with the instructions at [rustup.rs](https://rustup.rs/), then enable the development checks:

```sh
rustup component add rustfmt clippy
node --version
rustc --version
npm ci
npm run package
```

`npm ci` installs the exact JavaScript lockfile and enables this repository's formatting hook. Cargo uses `Cargo.lock` for native dependencies. The first build downloads dependencies and compiles GTK/WebKit bindings; later builds reuse the local cache.

Install the generated package:

```sh
sudo apt install ./target/release/bundle/deb/Spotlight_0.1.0_amd64.deb
spotlight
```

For a quick check without installation, run `./target/release/spotlight`. See [development](development.md) for live UI and Rust development.

Native prerequisites follow [Tauri's Ubuntu instructions](https://v2.tauri.app/start/prerequisites/). Build on the oldest Ubuntu release you intend to support: binaries built against newer glibc/WebKit libraries may not run on older Ubuntu versions. [Debian packaging details](https://v2.tauri.app/distribute/debian/).

## Configure Super + Space

GNOME reserves Super+Space for changing keyboard input sources by default:

1. Open **Settings → Keyboard → View and Customize Shortcuts → Typing**.
2. Reassign **Switch to next input source** to another shortcut.
3. In Spotlight, open **Settings** with Ctrl+,, enter `Super+Space` under **Open Spotlight**, and save.

On X11, Spotlight registers its shortcut directly. To keep GNOME's default, choose something else such as `Ctrl+Alt+Space` in Spotlight. [GNOME input shortcuts](https://help.gnome.org/gnome-help/keyboard-layouts.html).

On Wayland, add a GNOME custom shortcut:

| Field    | Value                                               |
| -------- | --------------------------------------------------- |
| Name     | Spotlight                                           |
| Command  | `/usr/bin/spotlight --toggle`                       |
| Shortcut | Super+Space, after reassigning the existing binding |

Before package installation, replace `/usr/bin/spotlight` with the absolute path to `target/release/spotlight`. Do not use `npm run desktop` as a desktop shortcut: that is the development build command.

Tauri's underlying global-hotkey library supports Linux X11 only. The custom command reuses the existing process on Wayland, but GNOME's compositor decides whether an application receives focus. Wayland has not yet completed release qualification. [Upstream support](https://github.com/tauri-apps/global-hotkey).

## Start automatically at login

After installing the `.deb`, run these commands from the cloned repository:

```sh
mkdir -p ~/.config/autostart
cp packaging/spotlight-autostart.desktop ~/.config/autostart/
```

The entry runs `/usr/bin/spotlight --background`: it keeps the search service ready with the window hidden. If you downloaded only the package, create `~/.config/autostart/spotlight-autostart.desktop` with the contents of [the autostart template](../packaging/spotlight-autostart.desktop).

To disable it, remove only `~/.config/autostart/spotlight-autostart.desktop`.

## Update or uninstall

Quit Spotlight using the power button before installing a newer package. Install the new `.deb` with the same `sudo apt install ./<package>.deb` command; normal updates retain your settings.

To build an update from Git:

```sh
git pull --ff-only
npm ci
npm run package
```

Then install the newly generated package. If you have local changes, commit or reconcile them before pulling.

To uninstall the installed application, run `sudo apt remove spotlight` and remove the optional autostart entry. Your settings remain under `~/.config/io.github.rupak.spotlight` and the disposable cache under `~/.cache/io.github.rupak.spotlight`, unless you use custom XDG locations.

## Troubleshooting

| Symptom                                    | What to check                                                                                                                                   |
| ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| Super+Space does nothing                   | Resolve the GNOME binding conflict, save the shortcut again, and check Spotlight's shortcut notice. On Wayland, use the custom command above.   |
| Missing file/folder results                | Add the containing absolute path in Settings → Search & indexing. Check hidden-item settings, exclusions, and depth/size notices, then refresh. |
| `npm run dev` cannot launch a result       | This is the browser preview with sample data. Run `npm run desktop` for native search and opening.                                              |
| Native build cannot find GTK/WebKit        | Install the development packages above; verify `pkg-config --modversion webkit2gtk-4.1 gtk+-3.0`.                                               |
| GUI cannot initialize GTK                  | Run inside your logged-in desktop session, with a valid display. A headless shell or restricted sandbox cannot display the app.                 |
| App still shows old code after a build     | Quit the existing Spotlight process before starting the new build; single-instance mode otherwise toggles the old process.                      |
| Index is incomplete or live watching fails | Inspect the notice, narrow the roots, or increase the configured index/depth limit. Refresh manually for folders beyond the watcher limit.      |

See [usage](usage.md) for daily operation and [verification](verification.md) for tested behavior and remaining limits.
