# Install on Ubuntu

Spotlight is a local desktop application for Ubuntu 24.04, built with Tauri, Rust, and React. X11 is the primary tested session. On Wayland, use a GNOME custom shortcut and see the focus limitation below.

## Install a published package

Open the [latest GitHub Release](https://github.com/rupak-at/spotlight/releases/latest) and download the `.deb` asset. Releases currently provide an x86_64 package built on Ubuntu 24.04, named like `Spotlight_0.1.0_amd64.deb`.

From the folder containing the downloaded package:

```sh
install -m 0644 ./Spotlight_*_amd64.deb /tmp/spotlight.deb
sudo apt install /tmp/spotlight.deb
rm /tmp/spotlight.deb
spotlight
```

You can also open **Spotlight** from Ubuntu's application menu. Package installation does not require Node.js, Rust, or a local development server. GitHub verifies the Release asset transport; this project does not yet publish a separate package signature or APT repository.

## Build after cloning the repository

Replace `<repository-url>` with the repository's Git clone URL from GitHub.

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
install -m 0644 ./target/release/bundle/deb/Spotlight_*_amd64.deb /tmp/spotlight.deb
sudo apt install /tmp/spotlight.deb
rm /tmp/spotlight.deb
spotlight
```

APT downloads and verifies packages as the restricted `_apt` user. Staging the package in `/tmp` prevents a harmless permission warning when `_apt` cannot traverse your home directory. Do not make your home directory globally accessible to suppress the warning.

For a quick check without installation, run `./target/release/spotlight`. See [development](development.md) for live UI and Rust development.

Native prerequisites follow [Tauri's Ubuntu instructions](https://v2.tauri.app/start/prerequisites/). Build on the oldest Ubuntu release you intend to support: binaries built against newer glibc/WebKit libraries may not run on older Ubuntu versions. [Debian packaging details](https://v2.tauri.app/distribute/debian/).

## Publish a release

This repository publishes packages from version tags. Before tagging, update the same semantic version in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. Then verify and push the normal change commit:

```sh
npm run release:check
npm test
npm run lint
npm run format:check
npm run package
git push origin main
```

Create and push an annotated tag that exactly matches the application version:

```sh
git tag -a v0.1.0 -m "Spotlight v0.1.0"
git push origin v0.1.0
```

The `Release` GitHub Actions workflow rejects a mismatched tag, runs the frontend checks, builds the `.deb` on Ubuntu 24.04 amd64, and creates a public GitHub Release. Follow the workflow in the repository's **Actions** tab. When it succeeds, verify the release page and install its downloaded asset on a clean supported machine. A regular branch push runs CI but does not publish a package.

## Configure Super + Space

GNOME reserves Super+Space for changing keyboard input sources by default:

1. Open **Settings → Keyboard → View and Customize Shortcuts → Typing**.
2. Reassign **Switch to next input source** to another shortcut.
3. In Spotlight, open **Settings** with Ctrl+,, enter `Super+Space` under **Open Spotlight**, and save.

On X11, Spotlight registers its shortcut directly. To keep GNOME's default, choose something else such as `Ctrl+Alt+Space` in Spotlight. [GNOME input shortcuts](https://help.gnome.org/gnome-help/keyboard-layouts.html).

### Left Super opens Activities or needs another press

On GNOME X11, the standalone left-Super **Activities overview** binding can intercept Spotlight’s shortcut even after the input-source binding is reassigned. Compare **right Super+Space** with **left Super+Space**. If only the right key opens Spotlight with one press, check:

```sh
gsettings get org.gnome.mutter overlay-key
```

When this is `'Super_L'`, one option is to disable the standalone overview binding:

```sh
gsettings set org.gnome.mutter overlay-key ''
```

This changes GNOME behavior: pressing Super alone will no longer open Activities. The Activities button remains available. Alternatively, use `gsettings set org.gnome.mutter overlay-key 'Super_R'` to move that standalone action to right Super and use left Super+Space for Spotlight. Record the previous value before changing it; restore it to undo the change (normally `gsettings set org.gnome.mutter overlay-key 'Super_L'`). These are opt-in desktop changes; Spotlight does not apply them automatically. [GNOME overview key setting](https://discourse.gnome.org/t/right-super-key-does-not-show-the-activities-overview-but-left-super-key-does/8029).

### Wayland custom shortcut

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

Quit Spotlight using the power button before installing a newer package. Copy the new `.deb` to `/tmp/spotlight.deb` with `install -m 0644`, install it with `sudo apt install /tmp/spotlight.deb`, then remove the temporary copy. Normal updates retain your settings.

To build an update from Git:

```sh
git pull --ff-only
npm ci
npm run package
```

Then install the newly generated package. If you have local changes, commit or reconcile them before pulling.

To uninstall the installed application, run `sudo apt remove spotlight` and remove the optional autostart entry. Your settings remain under `~/.config/io.github.rupak.spotlight` and the disposable cache under `~/.cache/io.github.rupak.spotlight`, unless you use custom XDG locations.

## Troubleshooting

| Symptom                                         | What to check                                                                                                                                                                                                                                   |
| ----------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Super+Space does nothing or needs another press | On GNOME X11, compare left and right Super and check the standalone Activities binding described above, as well as the input-source binding. Quit older Spotlight builds and save the shortcut again. On Wayland, use the custom command above. |
| Results flash or disappear while typing         | Quit an older running binary and start the current build. The current UI keeps the last completed results until their atomic replacement is ready.                                                                                              |
| Missing file/folder results                     | Add the containing absolute path in Settings → Search & indexing. Check hidden-item settings, exclusions, and depth/size notices. For full normal-file coverage, set Index limit and Folder depth to 0, save, and wait for the scan.            |
| `npm run dev` cannot launch a result            | This is the browser preview with sample data. Run `npm run desktop` for native search and opening.                                                                                                                                              |
| Native build cannot find GTK/WebKit             | Install the development packages above; verify `pkg-config --modversion webkit2gtk-4.1 gtk+-3.0`.                                                                                                                                               |
| GUI cannot initialize GTK                       | Run inside your logged-in desktop session, with a valid display. A headless shell or restricted sandbox cannot display the app.                                                                                                                 |
| App still shows old code after a build          | Quit the existing Spotlight process before starting the new build; single-instance mode otherwise toggles the old process.                                                                                                                      |
| APT reports `_apt` permission denied            | Copy the `.deb` to `/tmp/spotlight.deb` with `install -m 0644`, then install that path. The warning comes from `_apt` being unable to traverse a private home directory.                                                                        |
| Index is incomplete or live watching fails      | Inspect the notice, narrow the roots, or increase the configured index/depth limit. Refresh manually for folders beyond the watcher limit.                                                                                                      |

See [usage](usage.md) for daily operation and [verification](verification.md) for tested behavior and remaining limits.
