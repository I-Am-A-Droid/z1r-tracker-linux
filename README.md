# Z-Tracker for Linux

A native Linux rebuild of **Z-Tracker** (Z1R_WPF 2.1.0), the item and map tracker for the Zelda 1 Randomizer.

It tracks the overworld map (every marker type, with the original's limits), the dungeon item boxes, triforces,
hearts and Take Any caves, the dungeon room and door maps, blockers, hint zones, a timer and timeline, and notes.
It reads and writes the same `zt-save-*.json` files as the original, so saves move between the two.

## Pictures

No pictures from the original are included. At start-up the tracker reads them from your own copy of the original
program, `Z1R_WPF.exe` (it looks next to the app, in the current folder, and in folders under your home and
`~/Downloads` whose names contain "Tracker" or "Z1R"; or pass `--original /path/to/Z1R_WPF.exe`, or choose it in
Options). Nothing in that file is run. Without it the tracker still works and draws plain shapes and letters.

## Build and run

Needs the Rust toolchain (https://rustup.rs).

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

Settings and `HotKeys.txt` live in `~/.config/z1r-tracker/` (copied from the original's folder the first time, if
found); saves and the once-a-minute autosave go to `~/.local/share/z1r-tracker/saves/`.

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
(https://github.com/Neononane/Zelda1RandoTools). This is an independent rebuild for Linux.
