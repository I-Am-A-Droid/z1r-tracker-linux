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
Shop markers (pictures 23–31 of `s_icon_overworld_strip39.png`) have no letter "B" (asked for; the original's start with
a black "B"). Since the tracker now draws its own pictures (see "Own pictures"), they show just the item on orange.

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
- Applied: when the rebuild's program sits in a folder with a Z1R_Tracker_settings.json (e.g. the original's folder),
  it keeps settings, HotKeys.txt and saves in that folder like the original, so the two share them; otherwise it
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

## Window position remembered (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "MainWindowLT": 0 matches, native_function_pseudocode
0x792d6e is only `_CorExeMain@thunk`; the rule below is from the .NET IL.
- Start: `Program/MyWindow::.ctor` (token 0x0600004d, RVA 0x50f0) reads the setting `MainWindowLT` (default `""`,
  `<StartupCode$Z1R_Tracker>.$TrackerModelOptions` static initialiser) and, only if it matches `^(-?\d+),(-?\d+)$`,
  sets Window.Left and Window.Top to the two numbers (double.Parse, invariant culture). Otherwise the system places it.
- Moving: the handler `Program/-ctor@354-61::Invoke` (token 0x060000b5, RVA 0x7294) is attached to
  `Window.LocationChanged` (`-ctor@354-57` add_LocationChanged). Each move it reads Left and Top; if either is below
  -15999 (a minimised window) it does nothing, else it stores `sprintf "%d,%d" (int Left) (int Top)` (conv.i4, so the
  fraction is cut off) with `set_MainWindowLT` and calls `TrackerModelOptions::writeSettings()`.
- The rebuild does the same (`parse_window_lt`, `format_window_lt`, `remember_window_position` in src/main.rs), writing
  the settings only when the text changes. Wayland never tells programs their window position, so there nothing is
  saved or restored; X11, Windows and macOS work. `--screenshot` ignores the saved position.

## Mac disk image (changed on request)
`macos/make-dmg.sh` (run on a Mac, and by the `macos-dmg` job on GitHub) builds the Apple Silicon and Intel programs,
joins them with `lipo` into one, wraps it in Z-Tracker.app (`macos/Info.plist`), gives it an ad-hoc signature
(Apple Silicon won't run unsigned programs at all) and packs it with an Applications shortcut into
`z1r-tracker-macos.dmg` (hdiutil, compressed UDZO). It can't be made on Linux: linking a Mac program needs Apple's
MacOSX SDK (cargo zigbuild stops at "unable to find system framework AppKit"). Inside an app bundle the program's own
folder is taken to be the one holding Z-Tracker.app (`platform::program_dir`) for the original-folder mode, matching where the original looks (AppDomain.BaseDirectory).

## Window icon (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "BaseDirectory": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rule is from the .NET IL.
`Program/MyWindow::.ctor` (token 0x0600004d, RVA 0x50f0) creates a shell link (CLSID 00021401-0000-0000-C000-000000000046)
in its own folder (Process.MainModule.FileName's folder): description "Launch Z-Tracker", path Z1R_WPF.exe, working
folder = that folder, icon `icons/ztlogo64x64.ico` index 0, saved as `ZTracker.lnk`. That file holds 16, 24, 32 and
48 px pictures in 32-bit colour and a 64 px one in 16 colours. Applied as: the program has a logo of its own and uses
it as the window, taskbar and Dock icon (`art::window_icon`). (This first read the original's .ico; that was replaced
when every picture became the tracker's own, see below. On Wayland the desktop chooses the icon from the app id.) The rebuild doesn't write ZTracker.lnk; the Windows installer makes its own shortcuts.

## Own pictures (changed on request)
Nothing is read from Z1R_WPF.exe any more: every picture is drawn by `src/art.rs` at start-up from pixel patterns
written for this rebuild, and the reader for the original's embedded files, the search for it, `--original` and the
Options entry are gone. The sheets keep the original's names, sizes and order (from the IL: room pictures by
`RoomTypeGraphics::GetBmpPair`, monsters by `MonsterGraphics::LoadBitmaps`, markers by
`$OverworldMapTileCustomization::.cctor` and the TILE_STRIP_INDEX table), so every rule above that names a picture
number still holds:
- `icons7x7.png` 32 items of 7×7; `icons3x7.png` 8 shop items of 3×7; `icons10x10.png` 10 pictures (triforce, Take Any
  hearts, save disk …); `icons8x8.png` a heart.
- `zelda_bosses16x16.png` 32 monsters: the original sprite sheet itself since 2026-11-08 (see "Monster sprites" below); `zelda_items16x16.png` 8 floor drops, drawn as 8×8 patterns at twice the size.
- `s_icon_overworld_strip39.png` 39 map markers of 16×11: a coloured box with letters from a 3×5 alphabet and the item
  pictures above; `s_btn_tr_dungeon_num_strip18.png` level numbers; `new_icons13x9.png` 42 room pictures × 2 (done).
- `s_map_overworld_strip8.png`: the overworld as 16×8 screens of 16×11 pixels, coloured from a table of which ground
  covers each quarter-by-third of each screen (rock, water, forest, graveyard, sand, open ground; `art::TERRAIN`), with
  a texture per ground kind. The four overworlds share it; the markers show where everything is.
- `ZTlogo64x64.png` and the window icon: a gold triforce over "ZT" on a dark blue tile.
Settings and HotKeys.txt are no longer copied from the original's folder on first start; the original-folder mode now
looks only for Z1R_Tracker_settings.json. Save files stay compatible both ways.

## Grey pictures for items not had yet (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "icons7x7": 0 matches (the names are .NET
user strings), native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the .NET IL.
- `Graphics::greyscale` (token 0x06002b03, RVA 0x9a278) maps every pixel with `Graphics/greyscale@248::Invoke`
  (token 0x06002c7b, RVA 0x9cd28): a pixel equal to Color.Transparent stays; otherwise s = (R + G + B) / 5 (integer
  division) and the pixel becomes grey (v, v, v) with v = 0 if s = 0, else s + 60. (White becomes 213.)
- `UIComponents::MakeItemProgressBar` (token in the IL listing at line 82959) shows an item not had yet as its greyscale
  picture, e.g. the candle slot is [greyscale red candle, blue candle, red candle].
- Also in `Graphics`: `darken` = `darkenImpl 0.5` (token 0x06002b07, RVA 0x9a3cc; each of R, G, B times the factor,
  cut to a whole number), and `desaturateColor(c, f)` (token 0x06002b04, RVA 0x9a288): L = 0.3 R + 0.6 G + 0.1 B, each
  channel = channel + f × (L − channel).
- Applied: the item progress row now draws items not had yet with the greyscale rule (`art::grey_level`, sheet
  "icons7x7 grey"), instead of the earlier plain dimming to 25 %. Pixels that are see-through in the tracker's own
  pictures stay see-through (the original's PNG transparency is 0x00000000, which isn't Color.Transparent, so there it
  turns black; on the black row that looks the same).

## Higher-quality pictures (changed on request)
The tracker's own pictures are now drawn in much more detail: `src/art.rs` paints smooth shapes (circles, curves,
polygons, rounded boxes, colour blends) with 4 x 4 samples per pixel, gives every picture a dark edge, smooths the
monster pixel patterns with Scale2x (8x8 to 32x32) and lights them from above, and writes letters with the 3x5
alphabet made 4 times bigger and smoothed the same way (the original also uses a 3x5 alphabet,
`Graphics::paintAlphanumerics3x5`, token 0x06002b17, RVA 0x9a614). Items, markers, rooms and level numbers are drawn
at 4 pixels per unit (an item is 28 x 28 pixels), monsters and floor drops at 2, the logo at 128 pixels.
The overworld is drawn at 4 pixels per unit (1024 x 352): the edges between kinds of ground are bent with smooth
noise, forests get round trees, rock gets boulders, the graveyard gravestones, water a light shore, shallows and
waves, sand ripples; one kind of land meeting another gets a darker line. All four overworlds share it.
Each sheet records its scale, so the drawing code still works in the original's units; pictures are scaled smoothly
(linear filtering with mipmaps) instead of as blocks. Drawing all of them takes about 0.1 s at start-up.

## Black or white letters on coloured boxes (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "alphanumerics": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rule is from the .NET IL.
- `Graphics::makeColor(int)` (token 0x06002b00, RVA 0x9a124): a colour number 0xRRGGBB split as R = (n & 0xFF0000) / 65536,
  G = (n & 0xFF00) / 256, B = n & 0xFF.
- `Graphics::isBlackGoodContrast(int)` (token 0x06002b01, RVA 0x9a164): with R, G, B divided by 255, black letters
  are used when 0.2126 R² + 0.7152 G² + 0.0722 B² > 0.5 × 0.5; otherwise white.
- Used for the labels drawn on each dungeon's colour: `MainTrackerTriforcesAndItems/setup@213-18` (token 0x06000c48,
  RVA 0x2e050), `DungeonUI/makeDungeonTabs@739-61` (0x06001d25, RVA 0x6521c), `DungeonUI/update@1338-9`
  (0x06001e50, RVA 0x6822c), `DungeonUI/makeDungeonTabs@1344-164` (0x06001e54, RVA 0x6833c), `Views::redraw@204-6`
  (0x0600217c, RVA 0x79268).
- Applied: the letters and numbers on the tracker's own map markers and on found level numbers now take black or
  white by this rule from their box colour (`art::black_is_good_contrast`), instead of always black.

## Overworld map pictures (changed on request)
The overworld map is now made from the person's map screen pictures, copied from the original's `Overworld` folder
(`quest.0.ow.XX. Y.bmp`, 128 PNG files of 48 x 33 pixels: column XX 0-15, row Y 0-7, first quest only) into
`assets/overworld/quest0_xXX_yY.png`. `build.rs` embeds every file there in the program (`OVERWORLD_TILES`), and
`art::overworld` puts them together into the 768 x 264 map sheet at 3 pixels per unit, shown with sharp pixels when
enlarged. All four overworld choices use these first-quest pictures; a screen without a picture would be drawn by
the tracker's own map drawing (`art::drawn_overworld`). Nothing is read from the original's folder at run time.

## How the original makes its overworld pictures (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "Overworld": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the .NET IL.
- `Graphics::overworldMapBMPs(quest)` (token 0x06002bc9, RVA 0x9aecc) builds the map as a 16 x 8 array of 48 x 33
  pictures. For each screen it calls `Graphics::readCacheFileOrCreateBmp` (IL listing line 219004) with
  BaseDirectory + `Overworld\quest.%d.ow.%2d.%2d.bmp` (quest, x, y; `%2d` pads with a space): if the file exists it
  is read, otherwise the screen is made from the built-in map strip, its folder created, and the file saved (despite
  the name, as PNG). So the `Overworld` folder is the original's cache of the screens it has shown; it only holds
  the first quest when only that overworld was used.
- For quest 4 (the blank map), when `alternativeOverworldMapFilename` is set, it instead opens that picture, makes it
  768 x 264 (`new Bitmap(image, Size(768, 264))`) and cuts screen (x, y) from pixels x·16·3 .. +48, y·11·3 .. +33.
- The file is chosen right after a start choice (`Program/-ctor@1318-144::Invoke`, token 0x0600031a, RVA 0xc328):
  it is first set to "", then an open-file dialog "Overworld map images|*.png", starting in the program's folder,
  sets it. Saves keep it as `AlternativeOverworldMapFilename` (restored by `WPFUI/makeAll@82::Invoke`).
- Applied: choosing "Blank map" now offers the same choice (a window asking for the .png's path; also under Options:
  "Overworld picture for the blank map..."). The picture is stretched to 768 x 264 and each 48 x 33 part is one screen
  (`Assets::use_custom_map`); the path is saved and loaded with `AlternativeOverworldMapFilename`, so saves from the
  original that use one show it too. Without a picture the blank map stays plain, as before.

## Marked screens darkened (changed on request)
A map screen with any mark on it is now drawn darkened by the same amount as a "don't care" screen (shown at
1 − 0.55 brightness, `view::tile_is_shaded`); the marker itself is drawn on top at full brightness. (This turned out
to be the original's own rule too, see the next section.)

## How the original darkens map screens (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "dontcare": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rule is from the .NET IL.
- `WPFUI::redrawGridSpot@588` (IL listing) clears the screen, draws the map picture, and asks
  `OverworldMapTileCustomization::GetIconBMPAndExtraDecorations` for the mark's picture; with no mark nothing more is
  drawn. Otherwise it calls `WPFUI::redrawGridSpot$cont@602` (token 0x060003b8, RVA 0xe074):
  - mark 35 ("don't care", or the "show as don't care" flag): unless its extra data is also 35 (then a blank tile),
    the full-size X tile picture (`theFullTileBmpTable[35]`) is laid over the screen at opacity 0.55;
  - any other mark: `WPFUI::drawDarkening@404` (token 0x060003b2, RVA 0xdc0c; black 48 x 33 box, 3-px black edge,
    opacity 0.55) and then the marker picture on top at opacity 1.
- So in the original a marked screen is darkened exactly as much as a "don't care" one; my earlier note that it
  darkens only "don't care" screens was wrong.
- Applied: marked screens keep the darkening; "don't care" screens keep the same darkening (so both look equally
  dark, as asked) and now also show a faint X across the screen, standing in for the original's X tile picture
  (`view::dont_care_overlay`).

## No X on "don't care" screens (changed on request)
The faint X on "don't care" map screens is removed: they are only darkened (by the same 0.55 as marked screens). The
"don't care" choice in the pop-up and the key under the map still show the X marker picture.

## Hiding kinds of map tiles until Zelda is rescued (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "GetIconBMPAndExtraDecorations": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the .NET IL.
- Settings `HideOverworldTile_Sword3/Sword2/Sword1/LargeSecret/MediumSecret/SmallSecret/DoorRepair/MoneyMakingGame/
  TheLetter/Armos/HintShop/TakeAny/Shop` and `HideOverworldTile_AlwaysHideMeatShops` (TrackerModelOptions/
  OverworldTilesToHide; all false by default). Mark → option:
  `TrackerModel/MapSquareChoiceDomainHelper::AsTrackerModelOptionsOverworldTilesToHide` (token 0x06000416, RVA 0xd1f4):
  13-15 sword caves 3/2/1, 25-27 secrets large/medium/small, 28 door repair, 29 money game, 30 letter, 31 Armos,
  32 hint shop, 33 Take Any, 16 (every shop) Shop.
- `OverworldMapTileCustomization::ShouldHide` (token 0x06001ab3, RVA 0x5975c): false if the screen is temporarily
  shown (a press-and-hold "show hidden" feature, not rebuilt) or Zelda is rescued; otherwise the option's value.
- `OverworldMapTileCustomization::GetIconBMPAndExtraDecorations` (IL listing line 124227): for a hideable kind, if
  the tile's extra data equals the mark (something was taken there) it is shown; else it is hidden when ShouldHide.
  A hidden tile is drawn by `redrawGridSpot$cont@602` like "don't care" (no marker).
- Shops (`GetIconBMPAndExtraDecorations$cont@215`): the shop's items are its own and the extra one; with ShouldHide(16)
  the ones `IsHideableShopItem` (token 0x06001ab4, RVA 0x59790) says are no longer needed are dropped — arrow when
  ArrowLevel > 0, book when PlayerHasBoomBook, candle when CandleLevel > 0, ring when RingLevel > 0, key when
  HaveAnyKey, meat only with AlwaysHideMeatShops; bombs and shield never. Three, two or one left: that shop picture;
  none left: hidden.
- Applied: `Tracker::tile_hidden` and `Tracker::shop_items_shown` (src/model.rs), `data::hide_option_for`; hidden
  tiles are darkened without their marker, shops show only the items still needed. The options are under Options >
  "Hide map tiles until Zelda is rescued..." and use the original's setting names, so a shared settings file works
  both ways. Arrow level counts the wood arrow (progress) or silver arrow, candle the blue or red candle, ring the blue
  or red ring, key the magic key.

## Z-Tracker 1.2.3 compared (checked 2026-11-06)
The person pointed to /home/jason/Code/Zelda1RandoTools/UserCustomAssets/ZTracker_v1.2.3/Z1R_WPF.exe as an updated
version. It is older than the program this rebuild follows: no file version in its resources, README "v1.2", dated
2022-08-15; the source code beside it (git, latest commit "Version 1.3.1 full release", `OverworldData.VersionString =
"1.3.1"`) is newer than it, and Z1R_WPF 2.1.0.2 newer still. Compared by the text in both programs (UTF-16 user
strings, and 2.1.0's resource and translation files):
- Same map screen pictures in its `Overworld` folder (all 128 files identical) and the same settings keys.
- Gone in 2.1.0 on purpose (not brought back): a "Day %d / Night %d" timer joke mode ("terraria"), already removed
  in the 1.3.1 source; the "Remaining Locations Summary" (commented out in 1.3.1: OverworldMapTileCustomization.fs).
- Worded differently: 1.2.3 describes room types more fully (Z1R_WPF/DungeonRoomState.fs, `RoomType.DisplayDescription`,
  e.g. "Chevy (four-way moat-ladder-block)", "Tee (moat isolating south exit)", "Transport staircase #1 (one of
  matched pair)"); 2.1.0 shortened them. Applied: the room pop-up keeps the short names, and pointing at a room or a
  choice now shows the fuller description (`data::ROOM_TYPE_DESCRIPTIONS`).
- Also in that source (`BmpPair`): Ganon and Zelda rooms use their "done" picture in both states; the old-man hint
  room uses its "done" picture unless "BookForHelpfulHints" — matching the pictures the rebuild already draws.
Everything else in 1.2.3 is already in 2.1.0 and so in this rebuild.

## Dungeon "sunglasses" and the original's Options menu (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "BookForHelpfulHints": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the .NET IL.
- `<StartupCode$Z1R_WPF>.$OptionsMenu::.cctor` (token 0x06002511, RVA 0x828c4) lists the dungeon options as
  (label, explanation, setting): "Do door inference", "Book for Helpful Hints" (left-click old-man hint rooms to mark
  them read), "Alphabetize hint zone list", "Left-drag auto-inverts", "Default to NonDescript", "Dungeon 'sunglasses'"
  ("The dungeon tracker has high contrast (bright colors against black); Turn this on to darken the colors somewhat
  to reduce bright contrast"), "Reverse scroll wheel".
- `DungeonUI::makeDungeonTabs@1936-213` adds a black rectangle, not clickable, 6 px wider and taller than the dungeon
  canvas, at (−3, −3) on top of it; `DungeonUI::doSunglasses@1961` (token 0x06001be4, RVA 0x6183c) sets its opacity to
  0.15 when `GiveDungeonTrackerSunglasses` is on, else 0 (and again whenever the option changes).
- Applied: the option "Dungeon 'sunglasses'" (same setting name; the person's settings file has it on) lays black at
  0.15 over the room map, 3 px past it on every side (`SUNGLASSES_OPACITY`, `SUNGLASSES_MARGIN` in src/view.rs).

## "Start" marker (changed on request)
The first personal marker (mark 36, saved as the original's `PersonalPref1`) now says "Start" on the map and in the
marking pop-up instead of "P1" (`data::START_MARK_TEXT`); words longer than three letters use a 10-pt font so they
fit the 32 x 22 badge. Saves are unchanged, so the original still reads it as its Personal Preference 1 marker.

## Alphabetical hint zones (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "PersonalPref": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rule is from the .NET IL.
- `UIComponents::activatePopup@687-41` (IL listing line 83897) builds the hint-zone list; when
  `AlphabetizeHintZones` is on it keeps entry 0 ("(Unknown)") first and appends the rest sorted by name
  (`Array.sortBy` on the name, ordinal string order). Options menu text: "When checked, the hint zone dropdown
  (where you mark which zone a dungeon/sword is in) shows zones in alphabetical order instead of geographic order."
- Applied: option "Alphabetize hint zone list" (same setting name); clicking a hint in the Hints pop-up then steps
  through the zones in that order (left click forward, right click back) (`data::hint_zone_order`).

## Magnifier (asked for; checked 2026-11-06)
The person asked for a magnifier using the enlarged screens from Z-Tracker 1.2.3's `Magnifier` folder.
- In the original: setting `ShowMagnifier` (TrackerModelOptions/Overworld; the person's settings have it on), used
  while the mouse moves over the map (`WPFUI/makeAll@463-65::Invoke`, token 0x060004a0, RVA 0x1113c). Its enlarged
  screens come from `Magnifier\quest.%d.ow.%2d.%2d.bmp` (the same read-or-create cache as the map, IL listing line
  81318), 128 x 88 each (8 times a 16 x 11 screen). `UIComponents::MakeMagnifier$cont@31` (IL listing line 81101)
  shows a key to the coloured spots: open cave = cyan square on black; bomb spot, burn spot, recorder spot, pushable
  spot = black squares edged cyan, red, yellow, magenta (8 x 8, 2-px edge).
- Applied: the 128 pictures are copied to `assets/magnifier/quest0_xXX_yY.png` and built into the program
  (`build.rs` → `MAGNIFIER_TILES`; sheet "magnifier", 2048 x 704, 8 pixels per unit, sharp pixels). With the option
  "Magnifier" (same setting name) on, while the mouse is over a map screen that screen is drawn as large as fits
  over the dungeon rooms (keeping its 16:11 shape), labelled with its coordinates, with the key underneath
  (`View::draw_magnifier`). It follows "mirror the overworld"; with the blank map it enlarges the person's own map
  picture. Like the map pictures, these are first-quest screens and are used for every overworld choice.

## The "..." button: Starting Items and Extra Drops (asked for)
The person pointed at the "..." icon of Z-Tracker 1.2.3. In the original (source beside 1.2.3:
Z1R_WPF/MainTrackerTriforcesAndItems.fs, `Graphics.iconExtras_bmp` = picture 7 of icons10x10, tooltip "Starting items
and extra drops") it opens the panel built by OverworldItemGridUI.fs `makeExtrasPanelAndepRefresh`:
- header "Starting Items and Extra Drops", "Mark any items you start the game with or get as monster drops/extra
  dungeon drops in this section";
- the 8 triforces: clicking toggles the dungeon's own triforce (`TrackerModel.GetDungeon(i).ToggleTriforce`), or the
  starting piece (`HDNStartingTriforcePieces`) when dungeon numbers are hidden;
- weapons row: Wood sword, White sword, Magical sword, Wood arrow, Silver arrow, Bow, Wand, Blue candle, Red candle,
  Boomerang, Magic boomerang; utility row: Blue ring, Red ring, Power bracelet, Ladder, Raft, Recorder, Magic key,
  Book. Wood sword, wood arrow, blue candle, blue ring (and, in 2.1.0's saves, magical sword) are progress items
  (`PlayerProgressAndTakeAnyHearts`); the rest are `StartingItemsAndExtras`;
- "Max Hearts: N" with +1 / −1 buttons changing `MaxHeartsDifferential`;
- a picture reference of vanilla dungeon items and the hidden-dungeon triforce decoder (not rebuilt: they need the
  original's pictures).
Applied: the "..." button now opens that panel in the same rows (`input::extras_cells`, 11 columns): triforces,
weapons, utility items, then the hearts count with +1 / −1. Before, it listed only the 14 starting items and 8
starting triforce pieces.

## Starting Items and Extra Drops in 2.1.0 (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "Extra Drops": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the .NET IL.
- `OverworldItemGridUI::makeExtrasPanelAndepRefresh@496` (IL listing line 99987) in 2.1.0 has the same texts as the
  1.3.1 source: "Starting Items and Extra Drops", "Mark any items you start the game with / or get as monster
  drops/extra dungeon drops / in this section", "Max Hearts: %d", " You can adjust hearts here:", " +1 ", " -1 ",
  "Vanilla dungeon item reference / Don't click here", "1Q", "2Q".
- Its item rows (IL listing line 101842 on) are the same order; the magical sword there is
  `StartingItemsAndExtras.PlayerHasMagicalSword`, a flag separate from the main tracker's progress one. 2.1.0 never
  writes it to save files (`SaveAndLoad::SaveStartingItemsAndExtras`, token 0x06000008, RVA 0x270c, lists
  HDNStartingTriforcePieces, White sword … Book, MaxHeartsDifferential but no magical sword), nor reads it back, and
  `TrackerModel::recomputePlayerStateSummary` (token 0x06000306, RVA 0x9d70) counts only the progress flag for the
  sword level (3 = magical sword, unless white/magical sword are replaced by bomb upgrades). So in the original that
  tick is lost on reload and changes nothing; the rebuild deliberately ties the panel's magical sword to the saved
  progress flag (PlayerHasMagicalSword in PlayerProgressAndTakeAnyHearts), which does count and survive a reload.
- Max hearts (same function): 3 + heart containers had + Take Any hearts taken (state 1) + MaxHeartsDifferential —
  the rebuild's `Tracker::max_hearts` is the same formula.

## The legend's "..." button: Recorder settings and destinations (asked for)
The person pointed again at 2.1.0's "..." icon. Besides the Starting Items icon (done before), 2.1.0 has one text
"..." button, in the map legend beside "Recorder Destination" (`UIComponents::MakeLegend`, IL listing line 82570).
It opens "Recorder settings" (`UIComponents/MakeLegend@283-5::Invoke`, token 0x060010b2, RVA 0x42298) with two
check boxes and their explanations: "Recorder to new dungeons" ("If unchecked, the recorder goes to vanilla first
quest dungeon locations") and "Recorder to unbeaten dungeons" ("If checked, the recorder goes to dungeons where you
do NOT yet have the triforce"). Saved as `RecorderToNewDungeons` (default true) / `RecorderToUnbeatenDungeons`
(default false), which the rebuild already kept in saves but never showed.
- Destinations (source TrackerModel.fs `recorderDests`): only with the recorder; for each triforce t whose "have"
  is not equal to RecorderToUnbeatenDungeons: with RecorderToNewDungeons, dungeon t's marked screen (if found); else
  the vanilla first-quest location (OverworldData.vanilla1QDungeonLocations: (7,3) (12,3) (4,7) (5,4) (11,0) (2,2)
  (2,4) (13,6)).
- Drawing (WPFUI.fs RoutingInfo / recorderingCanvas; UIComponents.fs RecorderEllipseColor): a 35 x 35 square turned
  45 degrees, 3-px edge, centred on the screen, over a blurred black copy; rgb(220,220,220) with new dungeons, white
  with vanilla spots, which are also numbered (white on black, bold 12, bottom-left).
- Applied: `Tracker::recorder_destinations`, `View::draw_recorder_destinations`; the legend is now one tighter row
  ending with the diamond, "Recorder" and a "..." button that opens "Recorder settings" (the explanations show as
  each choice's description).

## Recorder destinations in 2.1.0 (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "Recorder settings": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the .NET IL.
- `TrackerModel/recorderDests@1668::GenerateNext` (token 0x0600064c, RVA 0x11c48): only if haveRecorder; for t in
  0..7 with (hasTri, idx) = doesPlayerHaveTriforceAndWhichDungeonIndexIsIt(t), skipped when hasTri equals
  recorderToUnbeatenDungeons; with recorderToNewDungeons it yields (dungeonLocations[idx], idx) unless idx is -1 or
  the location is NOTFOUND; otherwise (vanilla location t, t). Same as the 1.3.1 source.
- `<StartupCode$Z1R_Tracker>.$OverworldData` initialiser: vanilla1QDungeonLocations = (7,3) (12,3) (4,7) (5,4) (11,0)
  (2,2) (2,4) (13,6) (5,0); the rebuild's table for levels 1-8 matches.
- `WPFUI/makeAll@1074-166` RoutingInfo (IL listing line 29663): black square 35 x 35, edge 3, turned 45 degrees,
  BlurEffect radius 5; the coloured one the same without blur; both at (OMTW*i + 23, 33*j - 9); vanilla numbers: font
  12, bold, white on black, 2 px from the left and bottom.
- Applied: the diamond is now placed exactly there, centre (23, 15.75) in the screen instead of the screen's middle
  (`RECORDER_DIAMOND_X`, `RECORDER_DIAMOND_Y` in src/view.rs).

## Reference diagram with Starting Items and Extra Drops (asked for; checked 2026-11-06)
In the original the Starting Items and Extra Drops panel comes with two reference areas
(OverworldItemGridUI.makeExtrasPanelAndepRefresh; 2.1.0 `makeExtrasPanelAndepRefresh@496`, IL listing line 99987):
- "Vanilla dungeon item reference / Don't click here", opacity 0.6, with "1Q" and "2Q" and the pictures
  `first-quest-item-reference.png` / `second-quest-item-reference.png` (270 x 117, embedded in Z1R_WPF.exe; copied to
  assets/reference and built into the program).
- `Dungeon.MakeTriforceDecoderDiagram` (2.1.0 IL listing line 184750, same texts): opacity 0.8, "Reference diagram -
  don't click here", "(Note: in Mixed Quest dungeons, 7 and 8 may be swapped)", "First Quest or Shapes dungeons" with
  `DrawTriforceMapCore(haves, "12345678")`, "Second Quest dungeons" with the haves reordered
  [h0, h2, h1, h4, h3, h5, h7, h6] and labels "13254687". DrawTriforceMapCore: points on a 4N x 2N grid
  (p1 = (2N,0); p2..p4 = (N..3N, N); p5..p9 = (0..4N, 2N)), eight triangles [p1p2p3, p1p3p4, p2p5p6, p4p8p9, p2p3p7,
  p2p6p7, p3p4p7, p4p7p8], edged DarkSlateBlue 2 px, filled orange with a black label when that triforce is had,
  else black with a white label; labels bold, N/3 tall, in an N/2 box at (1.5,0.5) (2,0.5) (0.5,1.5) (3,1.5) (1.5,1)
  (1,1.5) (2,1) (2.5,1.5) × N. N = 80 there; 52 here so both areas fit under the panel.
- Clicks on both areas are swallowed (they don't close the panel).
Applied: `View::draw_extras_references`, `View::draw_triforce_map`, `View::extras_reference_rects`.

## Triforce "haves" with hidden dungeon numbers (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "quest-item-reference": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the .NET IL.
- `TrackerModel::GetTriforceHaves` (token 0x060002fc, RVA 0x9c4c): normally haves[i] = dungeon i's triforce. With
  hidden dungeon numbers: for each dungeon whose triforce is ticked and whose label is '1'..'8', haves[label − '1'] =
  true; and haves[i] = true for each starting piece i.
- `TrackerModel::doesPlayerHaveTriforceAndWhichDungeonIndexIsIt` (token 0x06000378, RVA 0xb150): normally (dungeon t's
  triforce, t). Hidden numbers: idx = the last dungeon labelled '1'+t, or −1; had = (idx ≠ −1 and its triforce) or
  starting piece t. The recorder destinations use idx (skipped when −1).
- `Dungeon::DrawTriforceMapCore` (token 0x06002533, RVA 0x831bc), `Dungeon::tri@447` (token 0x06002532, RVA 0x83174),
  `Dungeon::get_TRIFORCE_SIZE` = 80: triangles DarkSlateBlue edge 2 on black, orange with black label when had, labels
  bold white on transparent otherwise — as the rebuild draws them.
- Applied: `Tracker::triforce_haves` (used by the reference diagram) and `Tracker::triforce_for` (used by the recorder
  destinations). Before, with hidden dungeon numbers both used the dungeon's tab position instead of its label.

## Coast item box stays on F16 (changed on request)
The original removes the coast item box from F16 once its item is chosen and had or skipped (`WPFUI/makeAll@523-74`,
ladderBox.IsDone). On request it now stays, showing the chosen item with its green (had) or purple (skipped) border;
clicking it still changes the item, and the right-click / wheel still cycle had / skipped. The coast reminder still
stops once the box is done.

## The coast box in the original (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "ladderBox": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the .NET IL.
- `WPFUI/makeAll@523-74::Invoke` (token 0x060004b2, RVA 0x11574): if ladderBox.IsDone the coast box on F16 gets
  opacity 0 and stops taking clicks, else opacity 1 and clickable. `TrackerModel/Box::IsDone` (token 0x060004f6,
  RVA 0xec48): the box's cell is not −1 and playerHas is not NO.
- The rebuild's `Box_::is_done` is the same rule (now tested). The coast box itself stays visible on request (see
  "Coast item box stays on F16"); `is_done` still ends the coast reminder, as in the original.

## Shops for the item pointed at (asked for)
While the mouse is over the wood arrow, bombs, blue candle or blue ring in the top section, every map screen whose
shop sells that item (its own item or its second item; with "AllowItemIconOnNonShopTile" also any screen showing it as
an item icon) is outlined in bold green (4 px, rgb(0,255,0)) (`view::shop_item_for_progress`, `Tracker::tile_sells`).
The original's Mouse Hover Explainer describes the same areas as "Show locations of shops containing each item"
(UIComponents.fs MakeMouseHoverExplainer, 1.3.1 source); there it uses its own locator highlight, here the outline
asked for.

## The original's shop locator (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "shops containing": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the source beside 1.2.3
(1.3.1) and its texts in 2.1.0.
- WPFUI.fs `showShopLocatorInstanceFunc item`: for every screen whose mark is a shop (IsThreeItemShop) and whose
  mark is the item or whose extra data (second item) is it: `owLocatorTileRectangles.MakeGreenWithBriefAnimation()`,
  and the screen's hidden marker is shown for the moment (temporarilyDisplayHiddenOverworldTileMarks). None found:
  `showLocatorNoneFound` (shows the "ghost buster" icon).
- Graphics.fs TileHighlightRectangle: 48 x 33, MakeGreen = desaturateColor(Lime, 0.5) (rgb 76,204,76) at 4 px;
  MakeGreenWithBriefAnimation with `AnimateShopHighlights`: thickness 12 → 4 and colour Cyan → Lime over 0.5 s;
  MakeBoldGreen is the same green at 6 px.
- Who locates what (OverworldItemGridUI.fs MouseEnter, UIComponents.fs blockers): wood arrow box → arrow; bombs → bomb;
  boomerang-or-book → book; blue candle → candle only while CandleLevel = 0 (else burnable bushes); blue ring → ring;
  blockers bait → meat, key → key, bomb → bomb, bow and arrow → arrow.
- Applied: the bold-green outline asked for now follows these rules (only shops; book, blockers and the candle rule
  added; hidden shop markers shown while located; the 0.5 s 12 → 4 px cyan → lime animation, setting
  "AnimateShopHighlights", default on; "no … shop marked yet" when none). `View::locating`, `shop_item_for_progress`,
  `shop_item_for_blocker`, `Tracker::tile_sells`.

## Sword icons locate their caves (asked for)
Source beside 1.2.3 (OverworldItemGridUI.fs MouseEnter: wood_sword_box → Sword1, white sword → Sword2, mags_box →
Sword3; WPFUI.fs showLocator): a marked sword cave is shown (the original: magenta tile and crosshair lines via
showLocatorExactLocation); white / magical sword caves not marked yet fall back to their hint (GetLevelHint 9 / 10):
showLocatorHintedZone darkens every screen outside the hinted zone (black at opacity 0.7) and outlines the zone with a
dotted light-grey line. Applied, as asked "the same" as the shops: pointing at the wood sword, white sword box or
magical sword outlines that cave in the same bold green (animated) outline; if not marked, white/magical darken the
screens outside their hinted zone (0.7); otherwise "no … cave marked yet" (`Locate::Cave`, `sword_cave_for`,
`sword_hint_index`).

## The "boomerang or book" box shows as the book (asked for)
The top section's PlayerHasBoomBook box now uses the book picture and is named "Book" (hover text, timeline); the
save name stays PlayerHasBoomBook and it still locates book shops.

## Spot Summary (asked for; from the source beside 1.2.3, also in 2.1.0)
- Button "Spot Summary" (OverworldItemGridUI.fs; Layout.fs at 12.8·OMTW, 90; only for a standard overworld). While
  hovered, `OverworldMapTileCustomization.MakeRemainderSummaryDisplay` is shown at (50, 3) (Layout.fs AddSpotSummary);
  2.1.0 keeps it (WPFUI spotSummaryCanvas, IL listing line 32086). Clicking opens a separate window there (not rebuilt).
- Contents (icons with a 3-px black border; OPA 0.4, OPA2 0.75): "Unique Locations": marks 0-8, then 9-15, 30, 31,
  each at 0.4 once it can't be used again; secrets: the actual counts (first-quest overworld 3 large / 7 medium /
  4 small, otherwise 1 / 7 / 6), with the person's large/medium/small/unknown marks (at most 14 in that order) put in
  their own bins first, overflow into the other bin with more holes, unknowns into the bin with most holes; actual
  secrets dim to 0.4 when matched by a mark and 0.75 when matched by an unknown; "Non-Unique Locations": door repair,
  money game, hint shop, Take Any, potion shop — still-open ones on the left, used ones dimmed on the right;
  "Shop summary (only first item shown)": the marked shops sorted, then greyed hint-shop icons up to 12 (first-quest
  overworld) or 15 shops.
- Applied: `View::draw_spot_summary`, `Tracker::secret_summary` (tested), button beside "Triforces". The original's
  ↓/↑ arrows in the secret headings are written as words (the font has no arrows).

## Spot Summary in 2.1.0 (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "Spot Summary": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the .NET IL.
- `OverworldMapTileCustomization::MakeRemainderSummaryDisplay` (token 0x06001ac6, RVA 0x5a4e8) in 2.1.0 has the
  same texts ("Unique Locations", "↓ Actual Secrets ↓", "↑ Your Secret Marks ↑", "Non-Unique Locations", "Shop
  summary (only first item shown)"), unique lists 0-8 and 9-15, 30, 31, secret marks 25/26/27/24 capped at 14,
  bins 3/7/4 (first-quest overworld) or 1/7/6, opacities 0.4 / 0.75 / 0.01 (padding), non-unique marks 28, 29, 32,
  33, 34, shop count 12 / 15 and the empty shop slots as `Graphics::greyscale` of the hint-shop icon (32) at 0.4.
- `OverworldData/OWQuest::get_IsFirstQuestOW` (token 0x0600092b, RVA 0x17b94): true for FIRST and MIXED_FIRST, false
  for SECOND, MIXED_SECOND and BLANK — the rebuild's first-quest test (overworld 0 or 2) matches.
- Applied: the empty shop slots now use a greyscale copy of the markers made with the original's formula
  (`art::GREY_MARKERS`, s = (R+G+B)/5, grey = s + 60), instead of a darker tint.

## Map-marking pop-up in 9 columns (changed on request)
The pop-up for marking a map screen is laid out in 9 columns (`MARK_PICKER_COLS`), and "none" moved from the first
to the last cell, so levels 1-9 fill the first row, any roads and sword caves the second, and so on.

## The original's marking pop-up (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "ShopsFirst": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the .NET IL and the 1.3.1 source.
- `WPFUI::activatePopup@703-45` (IL listing line 17280; WPFUI.fs in the source): a grid of GCOL = 8 columns and
  GROW = 5 rows (6 with `PersonalPrefMarkersEnabled`, "When enabled, the tile selector popup includes two
  personal-preference marker tiles (asterisk and diamond)… Off by default"), cells 15 x 27 (the inner pictures):
  marks 0-23 in three rows; [gap, 24-29, gap]; [30-34, "don't care" 35, Unmarked, gap]. "Unmarked" is near the end.
- Placement, relative to the tile: left (−3) when the tile's x < 2 tiles, right-aligned (OMTW − gridWidth) when > 13
  tiles, else centred ((OMTW − gridWidth)/2 rounded to a multiple of 3); just below the tile (11·3 + border).
- "Shops before dungeons" (`ShopsFirst`, OptionsMenu.fs: "In the overworld map tile popup, the grid starts with shops
  when this is checked"): the grid is rotated, [grid.[16..]; grid.[..15]].
- Applied: the pop-up (9 columns, as asked) is now placed by the tile with the original's alignment
  (`input::mark_picker_anchor`, tested), and "Shops first" puts the shops and everything after them before the
  dungeons and any roads. The personal markers ("Start", "P2") stay in the pop-up even though the original hides them
  unless PersonalPrefMarkersEnabled is on, because the person uses the Start marker.

## Marking pop-up rows (changed on request)
The pop-up now has fixed rows (`input::MARK_PICKER_LAYOUT`, tested; rearranged on a second request): don't care,
none, arrow, bomb, key, meat, candle, shield, ring; unknown, large, medium, small secret, door repair, money making
game, Take Any, potion shop, hint shop; levels 1-9; any roads 1-4, wood sword, white sword, magical sword caves,
Armos, the letter; Start, P2. The book
shop (mark 18) is no longer offered (saves that have one still load and show it). With this fixed layout "Shops
first" no longer changes the pop-up; it still sets the order the mouse wheel steps through marks.

## Marking pop-up descriptions (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "After marking a shop": 0 matches (it is a
.NET user string), native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the IL and
the 1.3.1 source.
- The original's description box under the grid (WPFUI.fs; 2.1.0 IL listing line 24688): the choice's name, with
  `HotKeys.HotKeyProcessor.AppendHotKeyToDescription` adding "\nHotKey = <first key>" when a key is set for it
  (HotKeys.fs), and for a shop (marks 16-23) a second line "After marking a shop, left-click it again to mark a
  second item" (font 10, under a 1-px grey rule).
- Applied: the marking pop-up's description (bottom, now up to three lines, `Popup::foot_lines`) shows the name and
  uses, the shop reminder for shops, and "HotKey = ..." from HotKeys.txt (`HotKeys::overworld_key_for`,
  `input::with_hotkey`). With several keys for one mark, the first by name is shown.

## Which marks may go on a screen (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "Unmarked": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rule is from the 1.3.1 source (WPFUI.fs) and
OverworldData.fs.
- A choice in the tile pop-up is selectable when (it is the tile's current mark, or CanAddUse) and isLegalHere;
  isLegalHere(state) = if state = ARMOS then owInstance.HasArmos(i, j) else true. HasArmos is false on the blank map,
  otherwise the owMapSquaresArmos grid: M2, E3, E4, N4, O5 (the rebuild's `data::ARMOS` grid is the same).
- Applied: `Tracker::legal_here` / `Tracker::can_use` (tested): the Armos mark is greyed out in the pop-up and refused
  (with "Armos can't go here (no Armos statues on this screen)") on other screens and on the blank map. Before, it
  could be put anywhere.

## Linux AppImage (asked for)
`linux/make-appimage.sh` builds the program against glibc 2.28 with `cargo zigbuild --target
x86_64-unknown-linux-gnu.2.28` (it needs only libc, libm, libpthread, libdl, librt at start; the graphics and window
libraries are loaded when it runs), saves its logo with `--write-icon`, writes Z-Tracker.AppDir (AppRun,
z1r-tracker.desktop, icon) and packs it with appimagetool into `dist/Z-Tracker-x86_64.AppImage` (about 7 MB). The
GitHub job `linux-appimage` does the same on every upload. Tested here: the AppImage starts and draws the tracker.

## Names for the AppImage menu entry (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "ztlogo": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`. The original's version resource has only
OriginalFilename Z1R_WPF.exe and ProductVersion 2.1.0.0 (FileVersion 2.1.0.2), no description. `Program/MyWindow::.ctor`
(token 0x0600004d, RVA 0x50f0) names its shortcut ZTracker.lnk with the description "Launch Z-Tracker" and sets the
window title "Z-Tracker for Zelda 1 Randomizer". Applied: the AppImage's z1r-tracker.desktop uses Name=Z-Tracker,
GenericName=Z-Tracker for Zelda 1 Randomizer, Comment=Launch Z-Tracker.

## Reset the tracker but keep the maps (checked 2026-11-06)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "zt-save": 0 matches,
native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rule is from the source beside 1.2.3 (1.3.1).
- The original's "Pause/Reset timer" pop-up has "Click here to Reset the TRACKER, (remove inventory but preserve maps)
  for groundhog/routers/4+4 purposes." (OverworldItemGridUI.fs resetTrackerButton; the text is in 2.1.0 too). It
  makes a manual save ("Z-Tracker data saved to file … The tracker will now be reset.", or an error and no reset),
  then: removes every triforce; sets every item box to not had unless skipped; PlayerProgressAndTakeAnyHearts.ResetAll
  (Take Any hearts 0; boomerang-or-book, wood sword, wood arrow, blue ring, blue candle, magical sword, Ganon, Zelda,
  bombs off); Take Any and wood sword caves back to untaken (extra data 0), large/medium/small secrets and the letter
  bright (extra data = the mark); dungeon maps' floor drops bright again; reminders play again. Blockers are kept.
- Applied: Options > "Reset the tracker, keep the maps (groundhog / routers / 4+4)…" with a confirmation; it saves
  first and resets only if the save worked (`Tracker::reset_keep_maps`, tested; the coast reminder restarts). Map
  marks, dungeon maps, blockers, hints, notes and the timer are kept.

## Each overworld choice has its own map (asked for; checked 2026-11-08)
- `<StartupCode$Z1R_WPF>.$Graphics::.cctor` (token 0x06002d4c, RVA 0xa0e98) loads the overworld picture from the
  resource `s_map_overworld_vanilla_strip8.png` (a list of one file, picked at random). It is 1280 x 88: five maps of
  256 x 88 side by side — first quest, second quest, mixed (first-quest map), mixed (second-quest map), blank (plain
  sand). The second-quest map differs from the first in 140 pixels (its caves and secret spots); the mixed maps in 61.
- `Graphics/overworldMapBMPs@842::Invoke` (token 0x06002d28, RVA 0xa0850): screen (x, y) of map n is 48 x 33 with
  pixel (px, py) = strip(256·n + (48·x + px) div 3, (33·y + py) div 3), i.e. the 16 x 11 piece enlarged 3 times.
  These are what the original caches in its Overworld folder (only the maps it has shown, so the person's folder held
  only the first quest).
- Applied: the screens of maps 1-4 were made with that rule from the person's copy of the original (the rule
  reproduces all 128 first-quest screens in the Overworld folder exactly) and added to assets/overworld as
  quest1..quest4_xXX_yY.png, built into the program. The map sheet now holds all five maps
  (`art::OVERWORLD_MAPS`), `Assets::map_screen` picks the chosen overworld's, and the blank map shows the sand map
  when no picture of the person's own is chosen. The magnifier's enlarged pictures are first-quest only, so for the
  other overworlds it enlarges that overworld's own screen instead.

## Mixed quest: first/second-quest-only spots (checked 2026-11-08)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "vanilla_strip": 0 matches (a .NET user
string), native_function_pseudocode 0x792d6e is only `_CorExeMain@thunk`; the rules are from the source beside 1.2.3
(OverworldData.fs, OverworldItemGridUI.fs), whose texts are in 2.1.0.
- Always-empty screens: the rebuild's first- and second-quest tables equal owMapSquaresFirstQuestAlwaysEmpty /
  SecondQuestAlwaysEmpty; mixed overworlds use owMapSquaresMixedQuestAlwaysEmpty = empty in both (as the rebuild
  does); the blank overworld has none.
- owMapSquaresFirstQuestOnly = not always-empty in the first quest but always-empty in the second;
  owMapSquaresSecondQuestOnly = the reverse.
- The "FQ/SQ..." pop-up ("Warning! These actions cannot be undone! Don't click unless you are sure!"), for mixed
  overworlds: "Mark all First Quest Only spots as Don't Care. I am certain this is Second Quest." and the same for the
  Second Quest; each sets every such spot to DARK_X with AttemptToSet, replacing any mark, and its button is red when
  some of those spots are already marked (else yellow).
- Applied: Options > "Mixed quest: mark all First (Second) Quest Only spots Don't Care..." with a confirmation that
  says how many marked spots would be replaced (`Tracker::quest_only`, `Tracker::mark_quest_only_dont_care`, tested);
  refused on non-mixed overworlds. The confirmation box now sizes itself to its question. (Not rebuilt: the HFQ/HSQ
  check boxes that only shade those spots.)

## Reset button (asked for)
The original's "Pause/Reset timer" button opens a pop-up with: "Timer has been Paused. Click here to Resume.",
"Click here to confirm you want to Reset the TIMER to 0:00:00.", "Click here to Reset the TRACKER, (remove inventory
but preserve maps) for groundhog/routers/4+4 purposes." and "Click here to close Z-Tracker, and restart the app. All
your current work will be discarded!" (OverworldItemGridUI.fs, timerResetButton). Applied: a "Reset..." button beside
Options and Hints (the top buttons are now three per row) opens the same four choices, each confirmed first. "Start
over" puts the program back to its starting state without restarting it: a fresh tracker waiting for the overworld
choice; settings, hot keys and the window stay. Unlike the original, the current work is first saved as a manual save,
so it can be loaded again. The Options entry "Start over with a new seed" now does the same.

## Timer reset and the spots-remaining history (checked 2026-11-08)
Native check again: core_target_open on Z1R_WPF.exe, native_strings_find "Reset the TIMER": 0 matches (a .NET user
string), native_function_pseudocode 0x792d6e: Ghidra (Deep mode) now ran, and shows only the `_CorExeMain` entry stub;
the rules are from the .NET IL.
- "Reset the TIMER" (`OverworldItemGridUI/MakeItemGrid@655-186::Invoke`, token 0x06001761, RVA 0x50758): fires
  resetTimerEvent (clock to 0:00:00), clears timelineDataOverworldSpotsRemain and adds (0, OwSpotsRemain), calls
  ResetTotalSeconds on every timeline item and redraws the timeline.
- The history (`TrackerModel::allUIEventingLogic`, token 0x0600037b, RVA 0xb324): whenever owSpotsRemain differs from
  priorOWSpotsRemain, it adds (seconds since the start, owSpotsRemain). Saved flattened as
  [seconds, spots, seconds, spots, ...] (`SaveAndLoad/flat@375::GenerateNext`, token 0x060001f8, RVA 0x50c8) and read
  back in pairs (`WPFUI/makeAll@1956-400`, token 0x06000808, RVA 0x1b5dc). The person's save shows [1,73].
- Fixed: the rebuild kept one bare count per minute and saved that, which the original would read as wrong pairs. It
  now keeps (seconds, spots) pairs added when the count changes (`Tracker::note_spots_remaining`), saves and loads them
  flattened like the original, and "Reset the TIMER" does all three steps above (`Tracker::reset_timer`, tested).

## Streaming to OBS (asked for; rebuild-only)

The original has no network stream. For streaming it can open a separate "broadcast" window (option
`ShowBroadcastWindow`, built by `Broadcast.MakeBroadcastWindow`; also `BroadcastWindowSize`, `BroadcastWindowLT`,
`BroadcastWindowIncludesOverworldMagnifier`), meant to be captured from the screen. Asked for here: streaming the
tracker to an OBS media source over the network. Option `StreamToOBS` (off by default, kept in
Z1R_Tracker_settings.json; also "Streaming on" in the Stream button's window, under Reset...) starts a small web
server, port 8765 (`stream::STREAM_PORT`), on one address (`stream::chosen_ip`): 127.0.0.1 when
`StreamOnNetwork` is off ("Local", the default), or the network adapter chosen in the Stream window
(`StreamAddress`; by default the address the system routes outside traffic from). Changing the address restarts
the server there. Checked by running it: Local answered only on 127.0.0.1, Network only on the adapter's address.
The original has no such button or window; these keys are the rebuild's own.

- `/stream.mjpg`: motion JPEG (`multipart/x-mixed-replace; boundary=frame`), JPEG quality 85. OBS Media Source:
  Local File off, Input = that address, Input Format = `mpjpeg`.
- `/`: a page holding that stream, for a Browser Source.
- `/frame.jpg`: one picture.

Frames are the window as drawn (egui `request_screenshot`), captured at most `STREAM_FPS` = 15 times a second and
only while a stream or picture is being asked for. Encoding runs on its own thread, and a frame that arrives while
the previous one is still being encoded is dropped. When nothing new is drawn the last frame is sent again each
second, so players don't give up. Turning the option off closes the server.

The mouse pointer (checked 2026-11-08): screen capture doesn't show the real pointer, so the original's broadcast
window draws its own. `Broadcast/addFakeMouse@168-1` (token 0x06000f8a, RVA 0x3895c) adds a `Polygon` with
`Fill = Brushes.White`, no outline, and points (0,0), (12,6), (6,12) in tracker units. The MouseMove handler
`Broadcast/makeBroadcastWindow@204-18::Invoke` (token 0x06000fb8, RVA 0x398e8) puts the polygon's (0,0) corner at
the mouse position `GetPosition(appMainCanvas)` (`Canvas.SetLeft = X`, `SetTop = Y`; in the lower half the Y becomes
Y − THRU_MAIN_MAP_AND_ITEM_PROGRESS_H + 230, because that window shows the lower part moved up). It only moves on
MouseMove and is never hidden, so it stays where the mouse last was. The native entry point (0x792d6e) is only the
`_CorExeMain` start-up stub, so these come from the IL. The stream now draws the same triangle into every frame
(`stream::FAKE_POINTER`, `stream::draw_pointer`, tested), with its tip at egui's last known mouse position and
1 tracker unit = pixels-per-point pixels, smoothed at the edges.

Broadcast size (checked 2026-11-08). The original's broadcast window has three sizes, setting `BroadcastWindowSize`
(an integer in Z1R_Tracker_settings.json). In `Broadcast/makeBroadcastWindow@45` (token 0x06000f8b, RVA 0x389f0)
size 1 gives width 256 and `ScaleTransform` 0.333333, size 2 gives 512 and 0.666666, anything else 768 and no
scaling (the window height becomes content height × scale + 40). Its default is 3 (`TrackerModelOptions..cctor`,
token 0x060008bd: `ldc.i4.3; stsfld BroadcastWindowSize@131`), and reading the settings brings the value into 1..3
(`TrackerModelOptions.read`, token 0x06000759, RVA 0x148d4: `min(3, max(1, v))`). The Options menu labels are
"Full size broadcast" (3), "2/3 size broadcast" (2) and "1/3 size broadcast" (1) (OptionsMenu.makeOptionsCanvas,
handlers at RVA 0x81f50, 0x8201c, 0x820e8). The Stream window now has the same choice (Size: Full size / 2/3 size /
1/3 size), kept in the same `BroadcastWindowSize` key with the same default and limits (`stream::broadcast_size`,
`stream::broadcast_scale`, `stream::BROADCAST_SIZES`). Each streamed frame, pointer included, is shrunk by that
scale before it is sent (tested; running it at size 1 sent 282 × 355 for an 845 × 1064 window).

Blocking (modal) windows (checked 2026-11-08). In the original a pop-up blocks the main window:
`CanvasManager.CreatePopup(blackSunglassesOpacity)` (token 0x0600278b, RVA 0x8c834) pushes the opacity on a
stack, sets a black "sunglasses" canvas (Background = Black, IsHitTestVisible = false, made in
`CanvasManager..ctor`, token 0x06002784) to the largest opacity on the stack, and puts a new full-window canvas on
top (Background = Transparent, IsHitTestVisible = true, Opacity 1) that takes every click meant for the tracker;
`DismissPopup` (token 0x0600278c, RVA 0x8c9fc) pops both. `DoModal` (token 0x06002751, RVA 0x8bbac) and
`DoModalDocked` (token 0x06002753, RVA 0x8bc10) ask for opacity 0.5, the value most pop-ups use (DoModalOpa is
called with 0.5 in 16 places; others use 0.35, 0.25, 0.7 or 0.01). The Stream window is a modal like this: the
tracker behind it takes no clicks, wheel or keys (main.rs skips `View::handle_input` and the F5/Ctrl+S keys and
makes the notes box inert), and is darkened with black at opacity 0.5 (`MODAL_SUNGLASSES_OPACITY`). The rebuild's
own item and marking pop-ups don't dim the tracker yet. Checked 2026-11-08 by running the
tracker: `/frame.jpg` answered in 0.07 s (an 845 × 1064 JPEG) and `/stream.mjpg` gave 29 frames in 3 s.

## Monster sprites (asked for; checked 2026-11-08)

The monster pictures are now the original Zelda sprites from the original tracker's own sheet,
`Z1R_WPF/icons/zelda_bosses16x16.png` (512 × 16: 32 sprites of 16 × 16, solid black background), copied to
`assets/monsters` and built into the program. They are sliced as the original's Graphics.fs does (the
`digdogger_bmp … other_monster2` tuple): each sprite goes on its own 18 × 18 bitmap whose one-pixel edge is
`bg16x16` = RGB(45, 50, 0) and whose inside is black, and every sprite pixel that isn't black is copied in at (1,1).
The order is unchanged (0 Digdogger, 1 Gleeok, 2 Gohma, 3 Manhandla, 4 Wizzrobe, 5 Patra, 6 Dodongo, 7 red bubble,
8 blue bubble, 9 Darknut, 10 other, 11 old man, 12 Vire, … 29 rupee boss, 30 traps, 31 other 2), so `MONSTER_ICON`
still applies. `art::original_monsters` builds the sheet at 4 pixels per unit with each sprite pixel a sharp 4 × 4
block, and the room corners and the monster picker draw whole 18-unit cells (`art::MONSTER_CELL`). The drawn 8 × 8
patterns are kept only as a fallback if the sheet can't be read (tested: border, black inside and sprite pixels
match the sheet).

Where a room shows its monster and floor drop (checked 2026-11-08 in the shipped program; the native entry point
0x792d6e is only the `_CorExeMain` stub, so this is from the IL). `DungeonRoomState.CurrentDisplayEx$cont@191`
(token 0x06002a83, RVA 0x977e0) builds each room on a 13·3 × 9·3 = 39 × 27 canvas:
- the monster's 18 × 18 picture (`MonsterDetailExtensions.Bmp`) at (−5, −3), hanging off the top-left corner;
- if the room is completed, a black 18 × 18 `DockPanel` at opacity 0.5 over it at the same place, except when the
  monster is BlueBubble or RedBubble (`switch` on MonsterDetail − 4: values 4, 5) or Traps, Other or Other2 (`switch`
  on MonsterDetail − 29: values 29, 30, 31), which stay bright;
- the floor drop's 18 × 18 picture at (44 − 18, 30 − 18) = (26, 12), the bottom right, with the same black 0.5 square
  over it while the drop isn't marked bright.
The source of the same version (DungeonRoomState.fs, `canvasAdd(c, monsterIcon, -5., -3.)`) agrees. The rebuild
had these swapped (monster bottom right at 16 × 16, floor drop top left) and didn't darken monsters; it now uses the
original's places and sizes (`view::MONSTER_AT`, `FLOOR_DROP_AT`, `ROOM_DETAIL_SIZE`), the 0.5 darkening
(`DONE_DARKEN`, drawn as a 0.5 tint, which looks the same as a half-black square) and the stay-bright list
(`monster_stays_bright`, tested against the monster names). Its rooms are already 39 × 27, so the numbers carry over
unchanged.

## Version text, version button and About box (checked 2026-11-08)

The native entry point (0x792d6e) is only the `_CorExeMain` stub; these come from the IL of Z1R_Tracker.dll and
Z1R_WPF.exe.
- `<StartupCode$Z1R_Tracker>.$OverworldData..cctor` (Z1R_Tracker.dll, token 0x0600096c, RVA 0x18c48):
  `version = Assembly.GetExecutingAssembly().GetName().Version` (2.1.0.2 in the shipped files; Z1R_WPF.exe's
  AssemblyFileVersion is also 2.1.0.2 and its AssemblyInformationalVersion 2.1.0);
  `VersionString = sprintf "%d.%d.%d" version.Major version.Minor version.Build` = "2.1.0";
  `ProgramNameString = sprintf "Z-Tracker v%s" VersionString`;
  `AboutBody = sprintf "BETA RELEASE version %s of %s originally by Dr. Brian Lorgon111 with updates by
  Neononane\n\nLearn more at\n%s\n" (version.ToString()) ProgramNameString Website`, with `Website` =
  "https://github.com/Neononane/Zelda1RandoTools" (`OverworldData.get_Website`, token 0x060008e5).
  VersionString is what saves record as "Version" and what loading compares.
- The "v2.1.0" text is a button (`CustomComboBoxes.makeVersionButtonWithBehavior`: `makeButton(sprintf "v%s"
  VersionString, Some 12.0, Some Brushes.Orange)`); clicking it shows a modal message box with AboutBody and the
  buttons "Go to website" (opens Website) and "Ok".
The rebuild had the version as fixed grey text. Now `data::VERSION` and `PROGRAM_NAME` are made by the same
formulas from Cargo.toml's version (so the version is set in one place), the text is an orange size-12 button, and
clicking it opens the About box (`data::about_body`, a modal with 0.5 sunglasses like the Stream window) with "Go to
website" and "Ok". Rebuild-only: the About text ends with the date and time this copy was built (build.rs sets
`Z1R_BUILD_DATE`), so an old AppImage or installer can be told from a new one (tested).

## Windows program resources (checked 2026-11-08)

Z1R_WPF.exe's PE resources (its native entry point 0x792d6e is only the `_CorExeMain` stub):
- RT_GROUP_ICON 32512 (RVA 0x399078, 76 bytes) with RT_ICON 1–5: 16 × 16, 24 × 24, 32 × 32 and 48 × 48 at 32 bits
  and 64 × 64 at 4 bits (1128, 2440, 4264, 9640 and 2664 bytes; RVAs 0x394180, 0x3945f8, 0x394f90, 0x396048,
  0x398600). The same five pictures are in the original's `images/icon_32512_0.ico`.
- RT_VERSION 1 (RVA 0x3990d4): FileVersion 2.1.0.2, ProductVersion 2.1.0.0, OriginalFilename Z1R_WPF.exe.
- RT_MANIFEST 1 (RVA 0x39929c): `requestedExecutionLevel level="asInvoker" uiAccess="false"` (runs with the
  user's own rights; no administrator prompt).
The rebuild's z1r-tracker.exe had no resources at all (no icon in Explorer, nothing under Properties → Details).
build.rs now writes a .res file for Windows builds and links it (Microsoft's link.exe and Zig's lld both accept a
.res input): icon group 32512 from `windows/z1r-tracker.ico` (a copy of the original's icon file; checked: the five
RT_ICON entries and the group come out byte-for-byte the same as the original's), RT_VERSION with FileVersion and
ProductVersion = Cargo.toml's version plus ".0" (2.1.0.0), FileDescription "Z-Tracker for Zelda 1 Randomizer",
ProductName "Z-Tracker", OriginalFilename "z1r-tracker.exe", and the same asInvoker manifest. The installer uses
the same icon (`SetupIconFile`).

Icon picture (asked for, 2026-11-08): the Windows icon is now the tracker's own logo instead of the original's, the
same picture as the AppImage's icon (`linux/Z-Tracker.AppDir/z1r-tracker.png`, 128 × 128, made by `--write-icon`
from `art::logo`). It is kept as `windows/z1r-tracker.png`, and `windows/z1r-tracker.ico` is made from it with 16,
24, 32, 48, 64 and 128 pixel pictures (PNG-compressed, which Windows Vista and later read). It is still icon group
32512, so the rest of the rule above is unchanged; checked that the exe and the installer contain all six pictures
and no longer the original's.
