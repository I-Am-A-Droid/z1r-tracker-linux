# Z-Tracker for Linux, Windows and macOS

A native rebuild of **Z-Tracker** (Z1R_WPF 2.1.0), the item and map tracker for the Zelda 1 Randomizer,
that runs on Linux, Windows and macOS.

It tracks the overworld map (every marker type, with the original's limits), the dungeon item boxes, triforces,
hearts and Take Any caves, the dungeon room and door maps, blockers, hint zones, a timer and timeline, and notes.
It reads and writes the same `zt-save-*.json` files as the original, so saves move between the two.

## Pictures

The overworld map and the magnifier are made from the 128 map screen pictures in `assets/overworld` and the 128
enlarged ones in `assets/magnifier` (taken from the original Z-Tracker's `Overworld` and `Magnifier` folders, first
quest; the other overworld choices use the same pictures). They are built into the program, as are the original's two vanilla dungeon item reference pictures
(`assets/reference`), shown with the Starting Items and Extra Drops panel. Every other picture
(items, map markers, dungeon rooms, monsters and the logo) is drawn by the program itself, in high detail with smooth
edges, shading and outlines. It needs no other files when it runs: the original Z-Tracker doesn't have to be
installed.

## Download

**Linux:** download `Z-Tracker-x86_64.AppImage`, make it executable (right-click > Properties > Permissions >
"Allow executing", or `chmod +x Z-Tracker-x86_64.AppImage`) and run it. It works on most 64-bit Linux systems from
about 2018 on (glibc 2.28 or newer). If it says FUSE is missing, run it with `--appimage-extract-and-run`, or install
your system's `fuse2`/`libfuse2` package. To make it yourself: `linux/make-appimage.sh` (needs appimagetool; with
cargo-zigbuild it is built for glibc 2.28).

**Windows:** run `z1r-tracker-setup.exe` to install (no administrator rights needed; it adds a Start Menu entry, an
optional desktop shortcut and an uninstaller), or unzip `z1r-tracker-windows-x86_64.zip` and run `z1r-tracker.exe`
from anywhere. Windows may warn that the program is from an unknown publisher, because it isn't signed:
choose **More info** > **Run anyway**. Put it in the original Z-Tracker's folder if you want both to share settings
and saves.

To make those files yourself on Linux: `cargo zigbuild --release --target x86_64-pc-windows-gnu`
(needs `pip install ziglang` and `cargo install cargo-zigbuild`), then compile `windows/installer.iss` with Inno Setup.

Every upload to GitHub builds the program for Linux (also as an AppImage), Windows (`z1r-tracker.exe` and the installer) and macOS (the `.dmg`):
open the latest run under the repository's **Actions** tab and download it from **Artifacts**. Tagged versions
(e.g. `v2.1.0`) are also published under **Releases**.

**Mac:** open `z1r-tracker-macos.dmg` and drag **Z-Tracker** onto **Applications**. It runs on both Apple Silicon and
Intel Macs (macOS 10.13 or later). It isn't signed by a registered Apple developer, so the first time macOS refuses to
open it: right-click it in Applications and choose **Open**, or on newer macOS go to System Settings > Privacy &
Security and click **Open Anyway**.

The `.dmg` can only be made on a Mac (it needs Apple's tools): GitHub makes it on every upload (artifact
`z1r-tracker-macos-dmg`), or run `macos/make-dmg.sh` on a Mac with `rustup target add aarch64-apple-darwin
x86_64-apple-darwin`.

## Build and run

Needs the Rust toolchain (https://rustup.rs); the same commands work on all three systems.

```
cargo build --release
./target/release/z1r-tracker
```

Options:

```
--quest 1|2|mixed1|mixed2|blank   start straight away with that overworld
--load <file.json>                open a saved tracker (the original's save files work too)
--screenshot <file.png>           draw the window, save a picture of it and exit
```

Settings and `HotKeys.txt` are kept in:

| | settings | saves and the once-a-minute autosave |
|---|---|---|
| Linux | `~/.config/z1r-tracker/` | `~/.local/share/z1r-tracker/saves/` |
| Windows | `%APPDATA%\z1r-tracker\` | `%LOCALAPPDATA%\z1r-tracker\saves\` |
| macOS | `~/Library/Application Support/z1r-tracker/` | `~/Library/Application Support/z1r-tracker/saves/` |

If the program's folder holds a `Z1R_Tracker_settings.json` (for example when you put it in the original Z-Tracker's
folder), it keeps settings, hot keys and saves in that folder instead, exactly like the original, so both programs
share them.

With the Magnifier option on (Options), pointing at a map screen shows it enlarged over the dungeon rooms, with a
key to the coloured cave, bomb, burn, recorder and pushable spots.

The window can be resized freely and everything scales; when the window is wide, the dungeon maps and timeline move
beside the overworld map.

## How it matches the original

`MECHANICS.md` lists the values and rules taken from the original program and where each was found;
`mechanics.json` holds the same values in a machine-readable form.

## Not included yet

Voice commands and spoken reminders, online co-op sync, gamepad input, the broadcast/streaming window, route
drawing and drawing-layer icons (these are kept in save files but not shown).

## Credits

Z-Tracker was originally written by Dr. Brian Lorgon111, with updates by Neononane
(https://github.com/Neononane/Zelda1RandoTools). This is an independent rebuild for Linux, Windows and macOS.
