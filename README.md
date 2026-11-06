# Z-Tracker for Linux, Windows and macOS

A native rebuild of **Z-Tracker** (Z1R_WPF 2.1.0), the item and map tracker for the Zelda 1 Randomizer,
that runs on Linux, Windows and macOS.

It tracks the overworld map (every marker type, with the original's limits), the dungeon item boxes, triforces,
hearts and Take Any caves, the dungeon room and door maps, blockers, hint zones, a timer and timeline, and notes.
It reads and writes the same `zt-save-*.json` files as the original, so saves move between the two.

## Pictures

No pictures from the original are included. At start-up the tracker reads them from your own copy of the original
program, `Z1R_WPF.exe` (it looks next to the app, in the current folder, and in folders under your home and
`~/Downloads` whose names contain "Tracker" or "Z1R"; or pass `--original /path/to/Z1R_WPF.exe`, or choose it in
Options). Nothing in that file is run. Without it the tracker still works and draws plain shapes and letters.

## Download

**Windows:** run `z1r-tracker-setup.exe` to install (no administrator rights needed; it adds a Start Menu entry, an
optional desktop shortcut and an uninstaller), or unzip `z1r-tracker-windows-x86_64.zip` and run `z1r-tracker.exe`
from anywhere. Windows may warn that the program is from an unknown publisher, because it isn't signed:
choose **More info** > **Run anyway**. Put it in the original Z-Tracker's folder if you want both to share settings
and saves.

To make those files yourself on Linux: `cargo zigbuild --release --target x86_64-pc-windows-gnu`
(needs `pip install ziglang` and `cargo install cargo-zigbuild`), then compile `windows/installer.iss` with Inno Setup.

Every upload to GitHub builds the program for Linux, Windows (`z1r-tracker.exe`) and macOS (Apple Silicon and Intel):
open the latest run under the repository's **Actions** tab and download it from **Artifacts**. Tagged versions
(e.g. `v2.1.0`) are also published under **Releases**.

On macOS the program isn't signed: the first time, right-click it and choose **Open** (or allow it under
System Settings > Privacy & Security).

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
--original <Z1R_WPF.exe>          where to read the tracker pictures from
--screenshot <file.png>           draw the window, save a picture of it and exit
```

Settings and `HotKeys.txt` are copied from the original's folder the first time (if found) and kept in:

| | settings | saves and the once-a-minute autosave |
|---|---|---|
| Linux | `~/.config/z1r-tracker/` | `~/.local/share/z1r-tracker/saves/` |
| Windows | `%APPDATA%\z1r-tracker\` | `%LOCALAPPDATA%\z1r-tracker\saves\` |
| macOS | `~/Library/Application Support/z1r-tracker/` | `~/Library/Application Support/z1r-tracker/saves/` |

If you put the program in the original Z-Tracker's folder (next to `Z1R_WPF.exe`), it keeps settings, hot keys and
saves in that folder instead, exactly like the original, so both programs share them.

The window can be resized freely and everything scales; when the window is wide, the dungeon maps and timeline move
beside the overworld map.

## How it matches the original

`MECHANICS.md` lists the values and rules taken from the original program and where each was found;
`mechanics.json` holds the same values in a machine-readable form.

## Not included yet

Voice commands and spoken reminders, online co-op sync, gamepad input, the broadcast/streaming window, the mouse
magnifier, route drawing and drawing-layer icons (these are kept in save files but not shown).

## Credits

Z-Tracker was originally written by Dr. Brian Lorgon111, with updates by Neononane
(https://github.com/Neononane/Zelda1RandoTools). This is an independent rebuild for Linux, Windows and macOS.
