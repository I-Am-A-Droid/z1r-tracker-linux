# Z-Tracker (Z1R_WPF 2.1.0.2) – mechanics sheet for the Linux rebuild

The original is a .NET Framework 4.8 / F# / WPF program (Z1R_WPF.exe + Z1R_Tracker.dll + Z1R_Sync.dll).
There are no machine-code addresses: "source" names the .NET method (Type::Method) whose IL holds the value.
Evidence: assembly overviews ev_a752d073…, ev_c78fc103…; full member/IL listings ev_ca49d80a… (Z1R_WPF.exe),
ev_1429dc1d… (Z1R_Tracker.dll), ev_b1b20883… (Z1R_Sync.dll). Values marked *estimated* were looked for and not found.

## Why there are no native-code addresses
Opened with core_target_open and checked with the native tools (evidence ev_256e9d57…, ev_3ffecd6c…): the only native
function is `_CorExeMain@thunk` at 0x792d6e (6 bytes, `return _CorExeMain();`), which hands the program to the .NET runtime.
native_strings_find finds none of the program's texts, because .NET keeps them in its own #US heap. Ghidra could not open the
file. Everything below was read from the .NET bytecode (IL) of each method instead. Each method is listed with its metadata
token and RVA (its address in the file):

| method | token | RVA |
|---|---|---|
| Z1R_WPF.exe `<StartupCode$Z1R_WPF>.$OverworldMapTileCustomization::.cctor` | 0x06001b52 | 0x5d64c |
| Z1R_WPF.exe `<StartupCode$Z1R_WPF>.$OverworldItemGridUI::.cctor` | 0x060018e1 | 0x53170 |
| Z1R_WPF.exe `<StartupCode$Z1R_WPF>.$Layout::.cctor` | 0x06000e5d | 0x349f8 |
| Z1R_WPF.exe `Layout/ApplicationLayout::AddOverworldCanvas` | 0x06000da1 | 0x321ec |
| Z1R_WPF.exe `Layout/ApplicationLayout::AddItemGridStuff` | 0x06000d9d | 0x31f98 |
| Z1R_WPF.exe `Layout/ApplicationLayout::AddOWRemainingScreens` | 0x06000db1 | 0x32518 |
| Z1R_WPF.exe `OverworldItemGridUI::get_BLOCKERS_AND_NOTES_OFFSET` | 0x060013c6 | 0x48830 |
| Z1R_WPF.exe `OverworldItemGridUI::get_ITEM_PROGRESS_FIRST_ITEM` | 0x060013c7 | 0x48848 |
| Z1R_WPF.exe `MainTrackerTriforcesAndItems::setup$cont@76` | 0x06000bcb | 0x2c3ec |
| Z1R_WPF.exe `MainTrackerTriforcesAndItems::toggleSecondQuestDungeonCanvas$cont@117` | 0x06000bcc | 0x2c4f4 |
| Z1R_WPF.exe `WPFUI/makeAll@552-80::Invoke` (map tile mouse) | 0x060004ca | 0x118a0 |
| Z1R_WPF.exe `WPFUI::makeAll$cont@561` (map tile right-click) | 0x060003b7 | 0xdf74 |
| Z1R_WPF.exe `DungeonUI::installDoorBehavior$cont@917` (door mouse) | — | — (read from the IL listing) |
| Z1R_WPF.exe `Views::MakeBoxItemWithExtraDecorations` | 0x06002186 | 0x79a64 |
| Z1R_WPF.exe `<StartupCode$Z1R_WPF>.$CustomComboBoxes::.cctor` | 0x060028b4 | 0x9124c |
| Z1R_WPF.exe `<StartupCode$Z1R_WPF>.$Dungeon::.cctor` | 0x06002643 | 0x871a4 |
| Z1R_WPF.exe `<StartupCode$Z1R_WPF>.$Graphics::.cctor` | 0x06002d4c | 0xa0e98 |
| Z1R_WPF.exe `<StartupCode$Z1R_WPF>.$DungeonUI::.cctor` | 0x06001fc3 | 0x730c8 |
| Z1R_Tracker.dll `TrackerModel::overworldTiles` | 0x060002da | 0x9528 |
| Z1R_Tracker.dll `TrackerModel/MapSquareChoiceDomainHelper::IsItem` | 0x06000404 | 0xd154 |
| Z1R_Tracker.dll `TrackerModel/PlayerHas::AsInt` | 0x060004af | 0xe16c |
| Z1R_Tracker.dll `TrackerModel/DungeonTrackerInstance::makeDungeons` | 0x06000512 | 0xef94 |
| Z1R_Tracker.dll `TrackerModel/HintZone::ToString` | 0x06000624 | 0x11794 |
| Z1R_Tracker.dll `TrackerModel::toggleOverworldMapCircle` | 0x0600030e | 0xa324 |
| Z1R_Tracker.dll `TrackerModel::nextOverworldMapCircleColor` | 0x0600030f | 0xa380 |
| Z1R_Tracker.dll `<StartupCode$Z1R_Tracker>.$TrackerModel::.cctor` | 0x060006f2 | 0x13554 |
| Z1R_Tracker.dll `<StartupCode$Z1R_Tracker>.$OverworldData::.cctor` | 0x0600096c | 0x18c48 |
| Z1R_Tracker.dll `SaveAndLoad::SaveOverworld` | 0x06000004 | 0x2068 |
| Z1R_Sync.dll `RoomTypeGraphics::GetBmpPair` | 0x06000059 | 0x3780 |
| Z1R_Sync.dll `MonsterGraphics::LoadBitmaps` | 0x0600004e | 0x2ee0 |
| Z1R_Sync.dll `RoomTypeExtensions::DisplayDescription` | 0x0600006f | 0x3e4c |

## Window layout (pixels, 96 dpi)
| value | number | source |
|---|---|---|
| overworld tile width (OMTW) | 48 | `<StartupCode$Z1R_WPF>.$OverworldMapTileCustomization::.cctor` |
| overworld tile height | 33 (11 game pixels × 3) | map strip is 16×11 per screen, `s_map_overworld_strip8.png` |
| map top | 150 | `Layout/ApplicationLayout::AddOverworldCanvas` |
| THRU_MAIN_MAP_H | 414 | `$OverworldItemGridUI::.cctor` |
| legend height | 33 (→447) | same |
| item-progress row height | 30 (→477) | same |
| dungeon tab height TH | 24 | same |
| dungeon area | 24+30+306+3 (→840) | same |
| timeline height | 127 (→967) | same |
| blockers / notes column x | 408+42 = 450 | `OverworldItemGridUI::get_BLOCKERS_AND_NOTES_OFFSET` |
| blocker row height | 36 (×3) | `$OverworldItemGridUI::.cctor` |
| first item-progress icon x | 129 | `OverworldItemGridUI::get_ITEM_PROGRESS_FIRST_ITEM` |
| main tracker grid | 9 cols × 5 rows of 30 px at (0,0) | `MainTrackerTriforcesAndItems::toggleSecondQuestDungeonCanvas$cont@117` (makeGrid 9,5,30,30) |
| item grid origin | (280, 30) | `Layout/ApplicationLayout::AddItemGridStuff` |
| WEBCAM_LINE / timer x | 16×48−200 = 568, timer at 572 | `$Layout::.cctor`, `AddItemGridStuff` |
| timer box | 148×56, font 42, LightGreen | `$OverworldItemGridUI::.cctor` |
| remaining / gettable / max hearts text | x 410, y 90 / 110 / 130 (drawn at x 468 here, *estimated* shift to clear the 6-column item grid) | `AddOWRemainingScreens`, `AddOWGettableScreens`, `AddCurrentMaxHearts` |
| dungeon room | 39×27, pitch 51×39 (door gap 12) | constants in `DungeonUI::*` |
| L1/L4 third-box toggle colour | rgb(55,55,85) | `toggleSecondQuestDungeonCanvas$cont@117` |

## Overworld marks (index = value in the save file's `Map`)
From `TrackerModel::overworldTiles(bool isFirstQuest, bool)` and `TrackerModel/MapSquareChoiceDomainHelper::get_*`:
0–8 Level1–9 (1 each) · 9–12 Any Road 1–4 · 13 Sword3 (magical sword cave) · 14 Sword2 · 15 Sword1 ·
16–23 shops: Arrow, Bomb, Book, Candle, Blue Ring, Meat, Key, Shield (unlimited) · 24 Unknown secret ·
25/26/27 Large/Medium/Small secret · 28 Door repair (9 in 1Q, 10 in 2Q) · 29 Money-making game (5 / 6) ·
30 Letter · 31 Armos · 32 Hint shop (4) · 33 Take Any (4) · 34 Potion shop (7 / 9) · 35 Dark X "don't care" ·
36/37 personal preference markers. Mixed-first counts as first quest, mixed-second as second (`overworldTiles` call sites).
Save: three numbers per screen, rows top→bottom, columns left→right: mark, extra data (shop's second item 1–8,
0 otherwise, −1 when unmarked), circle (`SaveAndLoad::SaveOverworld`, format `"%2d,%2d,%3d"`).
Circle value (`TrackerModel::toggleOverworldMapCircle`, `nextOverworldMapCircleColor`): value mod 100 = 0 means no circle,
1 = plain circle, otherwise the label character (the label meaning is *estimated*); the hundreds (0/100/200) pick one of three colours.
Mouse on a map tile (`WPFUI/makeAll@552-80::Invoke`, `WPFUI::makeAll$cont@561`): middle-click toggles the circle,
shift-left-click labels it, shift-right-click cycles its colour, plain right-click opens an item-icon picker only when the
"AllowItemIconOnNonShopTile" option is on, left-click opens the mark picker (a second left-click on a shop picks its second item).

Item box (`Views::MakeBoxItemWithExtraDecorations`, `$CustomComboBoxes::.cctor`, `$Graphics::.cctor`): 30×30, border 3 px,
not had = rgb(168,0,0), have = rgb(50,168,50), skipped = MediumPurple.
Dungeon doors (`$Dungeon::.cctor`): unknown rgb(30,30,45), no rgb(145,0,0), yes rgb(60,120,60), yellow rgb(160,160,40), purple rgb(140,30,140).

## Item boxes
Items (index = `CellCurrent`): BookOrShield, Boomerang, Bow, PowerBracelet, Ladder, MagicBoomerang, AnyKey, Raft,
Recorder, RedCandle, RedRing, SilverArrow, Wand, WhiteSword, HeartContainer; counts 1 each except HeartContainer 9
(`$TrackerModel::.cctor`, itemNamesAndCounts). PlayerHas: NO=0, YES=1, SKIPPED=2 (`TrackerModel/PlayerHas::AsInt/FromInt`).
Boxes per dungeon 2,2,2,2,2,2,2,3,2 plus a shared third box shown under level 1, or level 4 when "second quest dungeons"
(`TrackerModel/DungeonTrackerInstance::makeDungeons`, toggle text in `toggleSecondQuestDungeonCanvas$cont@117`).

## Overworld geography (16×8)
Zones, armos, raft, bomb, burn, power bracelet, ladder, recorder, grave and always-empty grids are copied as data from
`<StartupCode$Z1R_Tracker>.$OverworldData::.cctor` (owMapZone, owMapSquares*). With the first-quest grids,
an empty map leaves 73 screens to check, which matches the original's own save file (`OverworldSpotsRemainingOverTime: [1,73]`).
Zone letters → hint zones: M Death Mountain, L Lake, H Lost Hills, R River, G Grave, D Desert, C Coast, W Dead Woods,
S Close to Start, F Forest (`TrackerModel/HintZone::ToString/AsDisplayTwoChars`).
Vanilla dungeon screens 1Q: (7,3)(12,3)(4,7)(5,4)(11,0)(2,2)(2,4)(13,6)(5,0); 2Q: (7,3)(4,3)(12,3)(11,1)(5,4)(0,3)(12,6)(8,1)(0,0).
Hint texts: "Aquamentus Awaits" = Level 1 … "entrance to death" = Level 9, "(npc) has (item) at" = White Sword item,
"Meet (npc) at" = Magical Sword.

## Dungeons
Room types (34), monsters (32), floor drops (9), door states Unknown/No/Yes/Yellow/Purple = 0–4: enums in Z1R_Sync.dll
(`Z1R_Tracker.Models.Z1R_TrackerInterop.*`). Room picture index in `new_icons13x9.png` (top half = not done, bottom
half = done): `RoomTypeGraphics::GetBmpPair`. Monster picture index in `zelda_bosses16x16.png`: `MonsterGraphics::LoadBitmaps`.
Doors arrays: horizontal doors 7 per row × 8 rows, vertical 8 per row × 7 rows (`$DungeonUI::.cctor`).
Mouse rules (from the original's own tips text in `$DungeonData::.cctor` and option texts):
left-click door = green, right-click = red, middle-click = yellow, scroll/shift-click = purple; left-click entrance
room cycles the arrow, other rooms toggle completed; unmarked room becomes "generic" or "push block?" depending on the
option; scroll up = monster picker, scroll down = floor-drop picker.
Doors (`DungeonUI::installDoorBehavior$cont@917`): left = YES, right = NO, middle = YELLOW, and the same click on a door
already in that state sets it back to UNKNOWN; shift+left = previous state (`Door::Prev`), shift+right = next (`Door::Next`).

## Blockers
16 kinds (`TrackerModel/DungeonBlocker`), 3 per level for levels 1–8, saved as `{ "Kind": "Blocker_…", "AppliesTo": [6 bools] }`.

## Not rebuilt in this version (Windows-only services)
Voice commands and spoken reminders (System.Speech), online co-op sync (SignalR host), gamepad (DirectInput),
broadcast/streaming window, mouse magnifier, route drawing, drawing-layer icons (kept in saves, not shown).

## Changed on request
Map marks are drawn as a 32×22 badge (the 16×11 marker picture at 2×) in the tile's top-left corner instead of covering the
whole 48×33 tile, so the map stays visible under a mark; a shop's second item sits in the tile's bottom-right corner.

## Saving clock (checked again in the bytecode)
The native tools were run again on 2026-11-05 (core_target_open, native_strings_find "autosave": 0 matches,
native_function_pseudocode 0x792d6e = `_CorExeMain@thunk` only), so the values come from the IL again:
- `WPFUI/makeAll@1981-404` creates a `DispatcherTimer` with `Interval = TimeSpan.FromSeconds 60`; its tick
  (`WPFUI/makeAll@2016-415::Invoke`, token 0x06000859, RVA 0x1d23c) calls `SaveAndLoad::SaveAll(..., AutoSave)`. It runs on
  real time, so autosaves continue while the game timer is paused.
- After saving, `WPFUI/makeAll@2022-416` (token 0x06000857) shows the disk icon at opacity 0.7, waits 300 ms, then hides it.
  The icon sits at x = OMTW×16 − 40 = 728, y = START_TIMELINE_H + 60 = 900 (`Layout/ApplicationLayout::AddDiskIcon`,
  token 0x06000dbe, RVA 0x327c0).
- `Program/MyWindow::Update` (token 0x0600004e, RVA 0x68a4) shows the time as `%2d:%02d:%02d` and once per new whole minute
  calls `updateTimeline` (this is where the remaining-spots history grows).

## Follow-up picker after marking (overworldAcceleratorTable)
`<StartupCode$Z1R_WPF>.$OverworldMapTileCustomization::.cctor` (token 0x06001b52, RVA 0x5d64c) fills
`overworldAcceleratorTable` with entries for marks 33 (Take Any), 15 (Wood Sword cave), 31 (Armos) and 14 (White Sword cave):
choosing one of these marks goes straight on to a second picker. Rebuilt for Armos (31 → the Armos item box) and the
White Sword cave (14 → the White Sword item box): the item picker opens below the tile, and picking an item marks it as had,
so it shows on the timeline at the current time.

Each entry, read from the IL (native tools rechecked 2026-11-05: still only `_CorExeMain@thunk` at 0x792d6e):
| mark | handler | follow-up |
|---|---|---|
| 31 Armos | `clo@101-16` (token 0x06001aef, RVA 0x5be68) → `clo@102-17` (0x06001aee, RVA 0x5bdf0) | `DoRemoteItemComboBox` on `armosBox` |
| 14 White Sword cave | `clo@105-19` (0x06001af7, RVA 0x5bf84) → `clo@106-20` (0x06001af6, RVA 0x5bf0c) | `DoRemoteItemComboBox` on `sword2Box` |
| 33 Take Any | `clo@93-10` (0x06001ae0, RVA 0x5bc4c) → `clo@94-11` (0x06001adf, RVA 0x5bc18) | `PieMenus::TakeAnyPieMenuAsync` (0x060026e4, RVA 0x89b28) |
| 15 Wood Sword cave | `clo@97-13` (0x06001ae7, RVA 0x5bd48) → `clo@98-14` (0x06001ae6, RVA 0x5bd14) | `PieMenus::TakeThisPieMenuAsync` (0x060026e8, RVA 0x8a0dc) |

Take Any choices: `PieMenus::whichHeart@250` (0x060026e3, RVA 0x89ac8) picks the first take-any heart slot still 0 (or none);
potion (`potionBehavior@266`) sets it to 2, heart (`heartBehavior@271`) to 1, candle (`candleBehavior@260`) to 2 and turns on
PlayerHasBlueCandle; leave changes nothing. Take This: wood sword (`swordBehavior@337`) turns on PlayerHasWoodSword, candle
(`candleBehavior@334-1`) turns on PlayerHasBlueCandle. Afterwards `clo@95-12` / `clo@99-15` store the tile's extra data =
the mark (33 / 15) when something was taken, else 0 (`TrackerModel::setOverworldMapExtraData`).
Shop markers (pictures 23–31 of `s_icon_overworld_strip39.png`) are drawn without their black "B" (columns 1–3, rows 3–7 of each
16×11 picture), which is painted over with the marker's own background colour when the pictures are loaded; the shop's
own letter and item picture stay.

## Shops on the map, as the original draws them (checked 2026-11-05)
Native tools rechecked: still only `_CorExeMain@thunk` at 0x792d6e and no strings, so this is from the IL.
- `OverworldMapTileCustomization::makeTwoItemShopBmp$cont@133` (token 0x06001aae, RVA 0x593d4): a 48×33 tile, see-through
  except an orange box at game pixels x 3–11, y 1–9 (27×27 px at (9,3)); the two shop items (3×7 pictures from
  `icons3x7.png`, black = see-through) at game pixels x 4–6 and x 8–10, y 2–8.
- One-item shop (`<StartupCode$Z1R_WPF>.$Graphics::.cctor`, token 0x06002d4c, RVA 0xa0e98, theInteriorBmpTable): a 5×9
  orange interior with the item at x 1–3, y 1–7, placed at (3,3) in the tile (15×27 px box, item at (6,6)).
- Orange = `itemBackgroundColor` = rgb(239,131,0) (same .cctor); see-through = `TRANS_BG` = Color.FromArgb(1, Black).
- Shop item picture order in icons3x7.png: arrow, bomb, book, candle, blue ring, meat, key, shield (= shop marks 16–23).
So on the map a shop shows no letters at all; the lettered marker pictures (with the black "B" removed) are only used in
the mark picker and the legend.

## Centered markers (changed on request)
All map markers are centered on their 48×33 tile (rounded down to whole pixels): the 32×22 marker badge, the 15×27
one-item shop box and the 27×27 two-item shop box (shop items stay 3 px and 15 px inside the box, as in the original).
The original puts shop boxes at (3,3) / (9,3) and marker pictures over the whole tile.

## Circles and "don't care" darkening (checked 2026-11-05)
Native tools rechecked again (only `_CorExeMain@thunk` at 0x792d6e; native_strings_find "circle": 0 matches); from the IL:
- `WPFUI::makeOwCircle@358` (token 0x060003af, RVA 0xdb80): an ellipse 33−2 = 31 px wide and tall, line 3 px.
- `WPFUI::owCircleColor@361` (token 0x060003b0, RVA 0xdbd8): value ≥ 200 → Yellow, ≥ 100 → pink, else bright blue;
  pink = rgb(255,64,153) and bright blue = rgb(0,208,255) (`WPFUI/makeAll@318-46`).
- `WPFUI::activateCircleLabelPopup$cont@426`: the circle sits 11.5−3 = 8.5 px from the tile's left, at the top; the label
  is shown only for value mod 100 in '0'–'9' (48–57) or 'A'–'Z' (65–90), as a 12-pt bold character on black at the
  tile's bottom-left.
- `WPFUI::drawDarkening@404` (token 0x060003b2, RVA 0xdc0c): a "don't care" tile gets a 48×33 black rectangle (3-px black
  border) at opacity 0.55, i.e. the map keeps 45 % of its brightness.

## The coast item at F16 (checked 2026-11-05)
- `WPFUI/makeAll@514-73::Invoke` (token 0x060004dc, RVA 0x11c68): for the tile i = 15, j = 5 (F16) the original adds the
  ladder box (`TrackerModel::get_ladderBox`, made with `Views::MakeBoxItemWithExtraDecorations`) onto the map tile at
  (OMTW − 31, 1) = (17, 1). The item grid's text says: "The item box to the right is for the item found off the coast, at
  coords F16", so the "ladder box" is the coast item box.
- `WPFUI/makeAll@523-74::Invoke` (token 0x060004b2, RVA 0x11574): the box on the map is shown and clickable while
  `ladderBox.IsDone()` is false, hidden once it's done.
- `TrackerModel/Box::IsDone` (Z1R_Tracker.dll token 0x060004f6, RVA 0xec48): an item is chosen (cell ≠ −1) and PlayerHas ≠ NO.
- In this rebuild, on request, F16 only takes the coast item: left-click opens the item picker for the coast box,
  right-click cycles have/skip, the wheel cycles the item, item hot keys set it, and overworld marks can't be put there.
  The box is centered on the tile like the other marks (the original puts it at (17,1)).

## Coast item reminder (checked 2026-11-05)
Native tools rechecked (only `_CorExeMain@thunk` at 0x792d6e; native_strings_find "coast": 0 matches); from the IL:
- `Reminders/PeriodicReminders::Check` (token 0x06000d4a, RVA 0x31178): once (now − last ladder reminder) ≥
  ReminderIntervalMinutes × 0.6 minutes (setting, 5 in the original's settings file → every 3 minutes), if the player has
  the ladder and doesn't have the coast item: with the coast box empty, "Get the coast item with the ladder"; with an item
  chosen, "Get the %s off the coast" — skipped when that item is the white sword (13) and the magical sword is had.
- `TrackerModel::recomputePlayerStateSummary` (Z1R_Tracker.dll): haveCoastItem = ladderBox.IsDone().
- Shown as an on-screen message when "Visual_CoastItem" is on (true in the original's settings); spoken reminders are
  not rebuilt.

## Take Any hearts follow the map (changed on request)
In the original a Take Any choice fills the first unknown heart (`PieMenus::whichHeart@250`) and isn't tied to its tile.
Here, on request, each Take Any / Wood Sword cave tile remembers what was taken there (saved as "LinuxTileChoices",
[x, y, choice] with 0 left, 1 potion, 2 candle, 3 heart, 4 wood sword; the original ignores this key). The four header
hearts are worked out from the Take Any tiles in reading order (top row first): heart = taken (1), potion or candle =
skipped (2), left = unknown (0). Changing a choice, re-marking, adding or removing a Take Any tile updates them at once;
hearts with no tile behind them keep what was clicked in the header, and tiles from old saves without a recorded choice
leave their heart unchanged. A blue candle or wood sword given by a tile is taken back when that tile's choice changes,
unless another tile also gave it.
Header heart clicks (checked 2026-11-06; native tools still see only `_CorExeMain@thunk` at 0x792d6e and no text):
`OverworldItemGridUI::f@207-19` (token 0x06001419, RVA 0x48d08) sets heart i to (value ± 1 + 3) mod 3, i.e. forward
0 unknown → 1 taken → 2 skipped → 0, and back the other way; the rebuild does the same (left-click forward, right-click back).
The only other places that change a heart are the Take Any choices (`PieMenus/potionBehavior@266`, `heartBehavior@271`,
`candleBehavior@260`).

## Resizable window (changed on request)
The whole 768×967 tracker scales to the window: every frame it is zoomed to the largest size that fits (keeping its
shape, between 0.25× and 6×) and centered in any space left over, so the map, boxes, text, pop-ups and notes all grow and
shrink together. The "Smaller app window" option now just sets the window to that fraction of the full size.
(The original offers only fixed sizes: "SmallerAppWindow" with SmallerAppWindowScaleFactor 0.666 in its settings.)
Window size in the original (checked 2026-11-06; native tools still see only `_CorExeMain@thunk` at 0x792d6e,
native_strings_find "SmallerAppWindow": 0 matches): `Program/MyWindow::.ctor` (token 0x0600004d, RVA 0x50f0)
- with "SmallerAppWindow" on, scales the whole canvas by `ScaleTransform(f, f)` (f = SmallerAppWindowScaleFactor, 0.666…)
  and sizes the window round(width × f + 16) × round(height × f + 39), the 16 × 39 being the Windows frame;
- reads `SystemParameters.WorkArea.Height` and, when it is below 1000 × f (f = 1 when not smaller), colours a bar
  rgb(120,30,30) instead of rgb(50,50,50) (40 px tall when scaled, 30 otherwise) as a warning that the screen is too short.
The rebuild uses the same 1000 × f test, but instead of warning it starts the window at 90 % of the screen height,
since the tracker scales to any window size.

## Wide arrangement (changed on request)
When the window is wide enough that it shows the tracker bigger, the lower part (from the dungeon tabs at y 477 down:
dungeon maps, blockers, notes, timeline — 490 px tall) moves to the right of the 768-px-wide upper part, giving a
1536 × 490 tracker instead of 768 × 967. It switches once the wide arrangement is at least 2 % bigger, and back the same
way. Option "WideLayoutWhenRoomy" (on by default) turns it off. The original has only the stacked arrangement
(plus its own "ShorterAppWindow" variant, `Layout/ShorterApplicationLayout`).
The original's own short arrangement (checked 2026-11-06; native tools still see only `_CorExeMain@thunk` at 0x792d6e,
native_strings_find "ShorterAppWindow": 0 matches): `Layout/ShorterApplicationLayout::.ctor` (token 0x06000dcc, RVA
0x32b50) keeps the 768-px width and makes the window 477 + 3 (RED_LINE) + 108 (blocker grid) + 127 (timeline) = 715 tall,
showing either the map part or the dungeon part in the same place (`upper` / `lower` canvases) with a 3-px line between.
The rebuild's wide arrangement keeps both parts visible side by side instead, and uses the same 3-px line as the divider
(colour red, inferred from the name RED_LINE); the right half starts at x = 768 + 3, so the wide tracker is 1539 × 490.

## Window title, sizes and save versions (checked 2026-11-06)
Native tools rechecked (only `_CorExeMain@thunk` at 0x792d6e; native_strings_find "Z-Tracker": 0 matches); from the IL:
- Window title "Z-Tracker for Zelda 1 Randomizer" (`Program/MyWindow::.ctor`, token 0x0600004d, RVA 0x50f0). Used as is.
- Window sizes: "Z-Tracker can run in a few window sizes: '4/3 size', 'Default', '5/6 size', or '2/3 size'."
  (`Program::menu@873`): SmallerAppWindowScaleFactor from 2/3 to 4/3 is accepted (it was capped at 1 before).
- Loading a save from another version (`Program/startButtonBehavior@1214-3::Invoke`, token 0x060002b3, RVA 0xb1…):
  the file's "Version" is compared with the program's; if different, versions starting "1.2" or "1.3" get "Loading this
  file might not work", others "Loading this file will probably fail", inside "You are running Z-Tracker version '%s'
  but the save file was created using version '%s'. %s, but Z-Tracker will attempt to load it anyway." The original
  asks with an "Attempt Load" button; the rebuild shows the same message and loads.

## Timer text (checked 2026-11-06)
Native tools rechecked (only `_CorExeMain@thunk` at 0x792d6e; native_strings_find "%2d:%02d": 0 matches); from the IL:
`Program/MyWindow::Update` (token 0x0600004e, RVA 0x68a4) writes `sprintf "%2d:%02d:%02d"` of TimeSpan.Hours, Minutes and
Seconds to the timer box (and the broadcast timer): hours padded to two places with a space, and wrapping after 23
because TimeSpan.Hours leaves out whole days. The rebuild now writes the timer the same way (it used to show unpadded,
unwrapped hours).

## Windows and macOS (changed on request)
The same code now builds for Linux, Windows (x86_64) and macOS (Apple Silicon and Intel); `cargo check` passes for
x86_64-pc-windows-gnu, aarch64-apple-darwin and x86_64-apple-darwin. Only `src/platform.rs` differs by system: settings
and save folders, opening files/links (xdg-open / explorer / open) and the local time for save names. The tracker's
behaviour and the save-file format are the same everywhere. `.github/workflows/build.yml` builds and tests all of them
on GitHub and offers the programs as downloads.

## Where the original keeps its files (checked 2026-11-06)
Native tools rechecked (only `_CorExeMain@thunk` at 0x792d6e; native_strings_find "zt-save": 0 matches); from the IL:
- `TrackerModelOptions::readSettings` (Z1R_Tracker.dll token 0x0600075c, RVA 0x1521c): settings file =
  AppDomain.BaseDirectory + "Z1R_Tracker_settings.json" (the program's own folder). HotKeys.txt is read from there too.
- `SaveAndLoad::SaveAll` (token 0x06000013): saves go to BaseDirectory + "zt-save-manual-" / "zt-save-completed-" +
  DateTime.Now.ToString("yyyy-MM-dd-HH-mm-ss") + ".json"; the autosave is BaseDirectory + "zt-save-zz-autosave.json"
  (`$SaveAndLoad::.cctor`, token 0x06000223). The rebuild uses the same names and the same local-time format.
- Applied: when the rebuild's program sits in the original's folder (Z1R_WPF.exe or Z1R_Tracker_settings.json next to
  it), it keeps settings, HotKeys.txt and saves in that folder like the original, so the two share them; otherwise it
  uses the per-user folders listed in README.md.

## Show/Run Custom file (checked 2026-11-06)
Native tools rechecked (only `_CorExeMain@thunk` at 0x792d6e; native_strings_find "ShowRunCustom": 0 matches); from the IL:
`ShowRunCustom::ParseShowRunCustomFile` (token 0x06002649, RVA 0x87b8c) accepts only these lines (case-sensitive):
`^\s*$` (blank), `^#(.*)$` (comment), `^\s*SHOW\s*"(.*)"\s*$`, `^\s*(-?\d+),(-?\d+),(\d+),(\d+)\s*SHOW\s*"(.*)"\s*$`
(window left, top, width, height) and `^\s*RUN\s*"(.*)"\s*(.*)$`. Anything else stops with "Error parsing '%s', line %d"
+ "You should fix this error by editing the text file. Or you can delete it, and pressing the button again will create an
empty template file in its place." The rebuild now uses the same patterns and message (it used to look for SHOW/RUN
anywhere, in any case, and skip bad lines). Pictures open in the system's viewer, so the window position is not used.

## Windows program and installer (changed on request)
On Windows the window is drawn with wgpu (Direct3D 12 / Vulkan) instead of OpenGL, because OpenGL set-up failed under
Wine ("failed to find a matching configuration for creating glutin config") and is unreliable in virtual machines and
remote desktops; Linux and macOS keep OpenGL. Z1R_RENDERER=glow|wgpu overrides the choice. The Windows .exe is built on
Linux with `cargo zigbuild --target x86_64-pc-windows-gnu`; `windows/installer.iss` makes a per-user Inno Setup installer
(Start Menu entry, optional desktop shortcut, uninstaller). Both were tested under Wine 64-bit: the program draws the full
tracker with the original's pictures, and the installer installs, adds shortcuts and the installed program runs.
