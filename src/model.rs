//! Everything the tracker remembers about one seed, and the save-file format
//! (the same layout as the original's zt-save-*.json files, so saves move between them).

use crate::data::*;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Has {
    #[default]
    No = 0,
    Yes = 1,
    Skipped = 2,
}

impl Has {
    pub fn from_i64(v: i64) -> Has {
        match v {
            1 => Has::Yes,
            2 => Has::Skipped,
            _ => Has::No,
        }
    }
    pub fn next(self) -> Has {
        match self {
            Has::No => Has::Yes,
            Has::Yes => Has::Skipped,
            Has::Skipped => Has::No,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Box_ {
    pub cell: i32, // item index, -1 = empty
    pub has: Has,
    pub ident: String,
}
impl Box_ {
    fn new(ident: &str) -> Self {
        Box_ { cell: -1, has: Has::No, ident: ident.to_string() }
    }
    /// An item is chosen and it's either had or skipped (TrackerModel/Box::IsDone).
    pub fn is_done(&self) -> bool {
        self.cell != -1 && self.has != Has::No
    }
}

/// Identifies one item box.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoxId {
    Dungeon(usize, usize),
    Level1or4Third,
    WhiteSword,
    Ladder,
    Armos,
}

#[derive(Clone, Debug, Default)]
pub struct Dungeon {
    pub triforce: bool,
    pub color: i64,
    pub label_char: String,
    pub has_map: bool,
    pub boxes: Vec<Box_>,
}

#[derive(Clone, Debug)]
pub struct Blocker {
    pub kind: usize,
    pub applies_to: [bool; 6],
}

#[derive(Clone, Debug, Default)]
pub struct Room {
    pub completed: bool,
    pub kind: usize,
    pub monster: usize,
    pub floor_drop: usize,
    pub floor_drop_bright: bool,
}
impl Room {
    pub fn new() -> Self {
        Room { completed: false, kind: 0, monster: 0, floor_drop: 0, floor_drop_bright: true }
    }
}

#[derive(Clone, Debug)]
pub struct DungeonMap {
    pub hdoors: [[i32; 8]; 7], // [x][y]: door between (x,y) and (x+1,y)
    pub vdoors: [[i32; 7]; 8], // [x][y]: door between (x,y) and (x,y+1)
    pub circled: [[bool; 8]; 8],
    pub rooms: Vec<Vec<Room>>, // [x][y]
    pub vanilla_overlay: i64,
}
impl DungeonMap {
    fn new() -> Self {
        DungeonMap { hdoors: [[0; 8]; 7], vdoors: [[0; 7]; 8], circled: [[false; 8]; 8], rooms: vec![vec![Room::new(); 8]; 8], vanilla_overlay: 0 }
    }
}

pub const PROGRESS_KEYS: [&str; 9] = [
    "PlayerHasBoomBook", "PlayerHasWoodSword", "PlayerHasWoodArrow", "PlayerHasBlueRing", "PlayerHasBlueCandle",
    "PlayerHasMagicalSword", "PlayerHasDefeatedGanon", "PlayerHasRescuedZelda", "PlayerHasBombs",
];
pub const STARTING_KEYS: [&str; 14] = [
    "PlayerHasWhiteSword", "PlayerHasSilverArrow", "PlayerHasBow", "PlayerHasWand", "PlayerHasRedCandle", "PlayerHasBoomerang",
    "PlayerHasMagicBoomerang", "PlayerHasRedRing", "PlayerHasPowerBracelet", "PlayerHasLadder", "PlayerHasRaft", "PlayerHasRecorder",
    "PlayerHasAnyKey", "PlayerHasBook",
];
pub const HINT_FLAG_KEYS: [&str; 6] = ["NoFeatOfStrengthHint", "SailNotHint", "SailToHint", "PlayMelodyHint", "StepOverWaterHint", "FireArrowsHint"];

/// What was taken at a Take Any cave or the Wood Sword cave (stored per map tile).
pub struct TileChoice;
impl TileChoice {
    pub const LEFT: i32 = 0;
    pub const POTION: i32 = 1;
    pub const CANDLE: i32 = 2;
    pub const HEART: i32 = 3;
    pub const SWORD: i32 = 4;
}

pub struct Tracker {
    // overworld
    pub quest: i64, // 0 first, 1 second, 2 mixed-first, 3 mixed-second, 4 blank
    pub mirror: bool,
    pub start_icon: (i64, i64),
    pub waypoint: (i64, i64),
    pub mark: [[i32; 8]; 16],
    pub extra: [[i32; 8]; 16],
    pub circle: [[i32; 8]; 16],
    /// What was taken at a Take Any or Wood Sword cave tile (see TileChoice); -1 = not recorded.
    pub tile_choice: [[i32; 8]; 16],
    /// How many take-any hearts are currently filled in from Take Any tiles on the map.
    pub take_any_linked: usize,
    // items
    pub hidden_dungeon_numbers: bool,
    pub second_quest_dungeons: bool, // the shared third box belongs to level 4 instead of level 1
    pub white_sword_box: Box_,
    pub ladder_box: Box_,
    pub armos_box: Box_,
    pub third_box: Box_,
    pub dungeons: Vec<Dungeon>,
    // player progress
    pub take_any_hearts: [i64; 4], // 0 unknown, 1 taken, 2 skipped
    pub progress: BTreeMap<&'static str, bool>,
    pub starting_triforces: [bool; 8],
    pub starting: BTreeMap<&'static str, bool>,
    pub max_hearts_diff: i64,
    pub blockers: Vec<[Blocker; 3]>,
    pub location_hints: [usize; 11],
    pub hint_flags: BTreeMap<&'static str, bool>,
    pub notes: String,
    pub recorder_index: i64,
    pub recorder_to_new: bool,
    pub recorder_to_unbeaten: bool,
    pub boomstick_seed: bool,
    pub atlas_seed: bool,
    pub wsms_bu_seed: bool,
    pub tab: usize,
    pub maps: Vec<DungeonMap>,
    pub drawing_layer_icons: Value,
    pub alt_map_file: String,
    pub hide_map_initially: bool,
    pub spots_over_time: Vec<i64>,
    pub timeline: BTreeMap<String, i64>, // ident -> seconds
    pub seconds: i64,
    pub started: bool,
}

impl Tracker {
    pub fn new() -> Self {
        let mut t = Tracker {
            quest: 0, mirror: false, start_icon: (-1, -1), waypoint: (-1, -1),
            mark: [[-1; 8]; 16], extra: [[-1; 8]; 16], circle: [[0; 8]; 16], tile_choice: [[-1; 8]; 16], take_any_linked: 0,
            hidden_dungeon_numbers: false, second_quest_dungeons: false,
            white_sword_box: Box_::new("WhiteSwordBox"), ladder_box: Box_::new("LadderBox"), armos_box: Box_::new("ArmosBox"),
            third_box: Box_::new("Level1or4Box3"), dungeons: vec![],
            take_any_hearts: [0; 4], progress: BTreeMap::new(), starting_triforces: [false; 8], starting: BTreeMap::new(),
            max_hearts_diff: 0, blockers: vec![], location_hints: [0; 11], hint_flags: BTreeMap::new(),
            notes: String::new(), recorder_index: 0, recorder_to_new: true, recorder_to_unbeaten: false,
            boomstick_seed: false, atlas_seed: false, wsms_bu_seed: false, tab: 0, maps: vec![],
            drawing_layer_icons: json!([]), alt_map_file: String::new(), hide_map_initially: false,
            spots_over_time: vec![], timeline: BTreeMap::new(), seconds: 0, started: false,
        };
        t.reset(0);
        t
    }

    pub fn reset(&mut self, quest: i64) {
        let started = self.started;
        let mut fresh = Tracker::blank();
        fresh.quest = quest;
        fresh.started = started;
        // screens that never hold anything in this quest start out marked "don't care", as in the original's saves
        for x in 0..16 {
            for y in 0..8 {
                if fresh.always_empty(x, y) {
                    fresh.mark[x][y] = DARK_X;
                    fresh.extra[x][y] = 0;
                }
            }
        }
        *self = fresh;
    }

    fn blank() -> Tracker {
        let box_counts = [2, 2, 2, 2, 2, 2, 2, 3, 2];
        let dungeons = (0..9)
            .map(|i| Dungeon {
                label_char: "?".into(),
                boxes: (0..box_counts[i]).map(|b| Box_::new(&format!("Level{}Box{}", i + 1, b + 1))).collect(),
                ..Default::default()
            })
            .collect();
        let blank_blocker = || Blocker { kind: BLOCKER_NOTHING, applies_to: [false; 6] };
        Tracker {
            quest: 0, mirror: false, start_icon: (-1, -1), waypoint: (-1, -1),
            mark: [[-1; 8]; 16], extra: [[-1; 8]; 16], circle: [[0; 8]; 16], tile_choice: [[-1; 8]; 16], take_any_linked: 0,
            hidden_dungeon_numbers: false, second_quest_dungeons: false,
            white_sword_box: Box_::new("WhiteSwordBox"), ladder_box: Box_::new("LadderBox"), armos_box: Box_::new("ArmosBox"),
            third_box: Box_::new("Level1or4Box3"), dungeons,
            take_any_hearts: [0; 4],
            progress: PROGRESS_KEYS.iter().map(|k| (*k, false)).collect(),
            starting_triforces: [false; 8],
            starting: STARTING_KEYS.iter().map(|k| (*k, false)).collect(),
            max_hearts_diff: 0,
            blockers: (0..8).map(|_| [blank_blocker(), blank_blocker(), blank_blocker()]).collect(),
            location_hints: [0; 11],
            hint_flags: HINT_FLAG_KEYS.iter().map(|k| (*k, false)).collect(),
            notes: "Notes\n".into(), recorder_index: 0, recorder_to_new: true, recorder_to_unbeaten: false,
            boomstick_seed: false, atlas_seed: false, wsms_bu_seed: false, tab: 0,
            maps: (0..9).map(|_| DungeonMap::new()).collect(),
            drawing_layer_icons: json!([]), alt_map_file: String::new(), hide_map_initially: false,
            spots_over_time: vec![], timeline: BTreeMap::new(), seconds: 0, started: false,
        }
    }

    // ---------------- boxes ----------------
    pub fn third_box_owner(&self) -> usize {
        if self.second_quest_dungeons { 3 } else { 0 }
    }

    /// Boxes shown for a dungeon, including the shared third box of level 1 or 4.
    pub fn box_ids_of(&self, d: usize) -> Vec<BoxId> {
        let mut v: Vec<BoxId> = (0..self.dungeons[d].boxes.len()).map(|b| BoxId::Dungeon(d, b)).collect();
        if d == self.third_box_owner() {
            v.push(BoxId::Level1or4Third);
        }
        v
    }

    pub fn bx(&self, id: BoxId) -> &Box_ {
        match id {
            BoxId::Dungeon(d, b) => &self.dungeons[d].boxes[b],
            BoxId::Level1or4Third => &self.third_box,
            BoxId::WhiteSword => &self.white_sword_box,
            BoxId::Ladder => &self.ladder_box,
            BoxId::Armos => &self.armos_box,
        }
    }
    pub fn bx_mut(&mut self, id: BoxId) -> &mut Box_ {
        match id {
            BoxId::Dungeon(d, b) => &mut self.dungeons[d].boxes[b],
            BoxId::Level1or4Third => &mut self.third_box,
            BoxId::WhiteSword => &mut self.white_sword_box,
            BoxId::Ladder => &mut self.ladder_box,
            BoxId::Armos => &mut self.armos_box,
        }
    }

    pub fn all_boxes(&self) -> Vec<&Box_> {
        let mut v: Vec<&Box_> = self.dungeons.iter().flat_map(|d| d.boxes.iter()).collect();
        v.extend([&self.third_box, &self.white_sword_box, &self.ladder_box, &self.armos_box]);
        v
    }

    pub fn uses_of_item(&self, item: i32) -> usize {
        self.all_boxes().iter().filter(|b| b.cell == item).count()
    }

    pub fn player_has_item(&self, item: i32) -> bool {
        self.all_boxes().iter().any(|b| b.cell == item && b.has == Has::Yes) || self.starting_has(item)
    }

    fn starting_has(&self, item: i32) -> bool {
        let key = match item {
            0 => "PlayerHasBook", 1 => "PlayerHasBoomerang", 2 => "PlayerHasBow", 3 => "PlayerHasPowerBracelet",
            4 => "PlayerHasLadder", 5 => "PlayerHasMagicBoomerang", 6 => "PlayerHasAnyKey", 7 => "PlayerHasRaft",
            8 => "PlayerHasRecorder", 9 => "PlayerHasRedCandle", 10 => "PlayerHasRedRing", 11 => "PlayerHasSilverArrow",
            12 => "PlayerHasWand", 13 => "PlayerHasWhiteSword", _ => return false,
        };
        self.starting[key]
    }

    /// The original's `ShouldHide` (OverworldMapTileCustomization::ShouldHide, token 0x06001ab3, RVA 0x5975c): a kind of
    /// tile is hidden when its "HideOverworldTile_…" option is on, but never once Zelda is rescued.
    fn should_hide(&self, mark: i32, opt: &dyn Fn(&str) -> bool) -> bool {
        !self.prog("PlayerHasRescuedZelda") && crate::data::hide_option_for(mark).is_some_and(opt)
    }

    /// A shop item the player no longer needs (OverworldMapTileCustomization::IsHideableShopItem, token 0x06001ab4,
    /// RVA 0x59790): arrows once any arrow is had, the book once "boomerang or book" is, a candle, a ring, a key
    /// (magic key), and meat only with "HideOverworldTile_AlwaysHideMeatShops". Bombs and the shield always stay.
    fn shop_item_no_longer_needed(&self, item: i32, opt: &dyn Fn(&str) -> bool) -> bool {
        match item {
            0 => self.prog("PlayerHasWoodArrow") || self.player_has_item(11),
            2 => self.prog("PlayerHasBoomBook"),
            3 => self.prog("PlayerHasBlueCandle") || self.player_has_item(9),
            4 => self.prog("PlayerHasBlueRing") || self.player_has_item(10),
            6 => self.player_has_item(6),
            5 => opt("HideOverworldTile_AlwaysHideMeatShops"),
            _ => false,
        }
    }

    /// The items a shop shows (0-7, see SHOP_ITEM_NAMES): its own and the second one recorded with it; with
    /// "HideOverworldTile_Shop" on, the ones no longer needed are left out (GetIconBMPAndExtraDecorations$cont@215).
    pub fn shop_items_shown(&self, x: usize, y: usize, opt: &dyn Fn(&str) -> bool) -> Vec<i32> {
        let mark = self.mark[x][y];
        if !(crate::data::SHOP_FIRST..=crate::data::SHOP_LAST).contains(&mark) {
            return vec![];
        }
        let first = mark - crate::data::SHOP_FIRST;
        let extra = self.extra[x][y];
        let mut items = vec![first];
        if (1..=8).contains(&extra) && extra - 1 != first {
            items.push(extra - 1);
        }
        if self.should_hide(crate::data::SHOP_FIRST, opt) {
            items.retain(|&i| !self.shop_item_no_longer_needed(i, opt));
        }
        items
    }

    /// Whether a marked screen is shown as "don't care" because its kind is hidden
    /// (OverworldMapTileCustomization::GetIconBMPAndExtraDecorations, IL listing line 124227): a shop when none of
    /// its items is left to show; any other hideable kind unless something was taken there (extra data = the mark).
    pub fn tile_hidden(&self, x: usize, y: usize, opt: &dyn Fn(&str) -> bool) -> bool {
        let mark = self.mark[x][y];
        if (crate::data::SHOP_FIRST..=crate::data::SHOP_LAST).contains(&mark) {
            return self.shop_items_shown(x, y, opt).is_empty();
        }
        self.extra[x][y] != mark && self.should_hide(mark, opt)
    }

    pub fn prog(&self, key: &str) -> bool {
        self.progress.get(key).copied().unwrap_or(false)
    }
    pub fn set_prog(&mut self, key: &'static str, v: bool) {
        self.progress.insert(key, v);
    }

    // ---------------- overworld ----------------
    fn first_quest_counts(&self) -> bool {
        self.quest == 0 || self.quest == 2
    }
    pub fn is_second_quest(&self) -> bool {
        self.quest == 1 || self.quest == 3
    }
    pub fn max_uses(&self, mark: i32) -> usize {
        let t = &TILES[mark as usize];
        if self.first_quest_counts() { t.max_1q } else { t.max_2q }
    }
    pub fn uses_of_mark(&self, mark: i32) -> usize {
        self.mark.iter().flat_map(|c| c.iter()).filter(|m| **m == mark).count()
    }
    pub fn can_use(&self, mark: i32, x: usize, y: usize) -> bool {
        mark < 0 || self.mark[x][y] == mark || self.uses_of_mark(mark) < self.max_uses(mark)
    }

    pub fn always_empty(&self, x: usize, y: usize) -> bool {
        match self.quest {
            0 => at(&ALWAYS_EMPTY_1Q, x, y),
            1 => at(&ALWAYS_EMPTY_2Q, x, y),
            2 | 3 => at(&ALWAYS_EMPTY_1Q, x, y) && at(&ALWAYS_EMPTY_2Q, x, y),
            _ => false,
        }
    }

    pub fn dungeon_located(&self, d: usize) -> bool {
        self.mark.iter().flat_map(|c| c.iter()).any(|m| *m == d as i32)
    }

    pub fn spots_remaining(&self) -> i64 {
        let mut n = 0;
        for x in 0..16 {
            for y in 0..8 {
                if self.mark[x][y] == -1 && !self.always_empty(x, y) {
                    n += 1;
                }
            }
        }
        n
    }

    /// Unmarked spots the player can reach with what they have now.
    pub fn gettable_remaining(&self) -> i64 {
        let q2 = self.is_second_quest();
        let raft = self.player_has_item(7);
        let ladder = self.player_has_item(4);
        let recorder = self.player_has_item(8);
        let bracelet = self.player_has_item(3);
        let candle = self.player_has_item(9) || self.prog("PlayerHasBlueCandle");
        let mut n = 0;
        for x in 0..16 {
            for y in 0..8 {
                if self.mark[x][y] != -1 || self.always_empty(x, y) {
                    continue;
                }
                if at(&RAFTABLE, x, y) && !raft { continue; }
                if q2 && at(&LADDERABLE_2Q, x, y) && !ladder { continue; }
                if at(if q2 { &WHISTLE_2Q } else { &WHISTLE_1Q }, x, y) && !recorder { continue; }
                if at(if q2 { &BRACELET_2Q } else { &BRACELET_1Q }, x, y) && !bracelet { continue; }
                if at(if q2 { &BURNABLE_2Q } else { &BURNABLE_1Q }, x, y) && !candle { continue; }
                n += 1;
            }
        }
        n
    }

    pub fn max_hearts(&self) -> i64 {
        let boxes = self.all_boxes().iter().filter(|b| b.cell == HEART_CONTAINER && b.has == Has::Yes).count() as i64;
        3 + boxes + self.take_any_hearts.iter().filter(|t| **t == 1).count() as i64 + self.max_hearts_diff
    }

    pub fn triforce_count(&self) -> usize {
        self.dungeons.iter().take(8).filter(|d| d.triforce).count()
    }

    // ---------------- timeline ----------------
    /// The Take Any tiles on the map, in reading order (top row first), at most 4.
    pub fn take_any_tiles(&self) -> Vec<(usize, usize)> {
        let mut v = vec![];
        for y in 0..8 {
            for x in 0..16 {
                if self.mark[x][y] == TAKE_ANY_MARK {
                    v.push((x, y));
                }
            }
        }
        v.truncate(4);
        v
    }

    /// The four take-any hearts in the header follow the Take Any tiles on the map: the first tile is heart 1, the
    /// second heart 2, and so on. Heart taken = 1, potion or candle taken = 2 (heart skipped), left / not yet = 0.
    /// Hearts with no tile behind them keep whatever was clicked in the header; a tile whose choice isn't recorded
    /// (e.g. from the original's save files) leaves its heart as it is.
    pub fn sync_take_any_hearts(&mut self) {
        let tiles = self.take_any_tiles();
        for (i, &(x, y)) in tiles.iter().enumerate() {
            let c = self.tile_choice[x][y];
            if c >= 0 {
                self.take_any_hearts[i] = match c {
                    TileChoice::HEART => 1,
                    TileChoice::POTION | TileChoice::CANDLE => 2,
                    _ => 0,
                };
            }
        }
        // hearts that a removed tile used to fill go back to unknown
        for i in tiles.len()..self.take_any_linked.min(4) {
            self.take_any_hearts[i] = 0;
        }
        self.take_any_linked = tiles.len();
        for i in 0..4 {
            let had = self.take_any_hearts[i] == 1;
            self.stamp(&format!("TakeAny{}", i + 1), had);
        }
    }

    /// Is something other than tile (x, y) giving the player this choice (e.g. another cave's blue candle)?
    pub fn other_tile_gave(&self, x: usize, y: usize, choice: i32) -> bool {
        (0..16).any(|xx| (0..8).any(|yy| (xx, yy) != (x, y) && self.tile_choice[xx][yy] == choice))
    }

    pub fn stamp(&mut self, ident: &str, has: bool) {
        if has {
            self.timeline.entry(ident.to_string()).or_insert(self.seconds);
        } else {
            self.timeline.remove(ident);
        }
    }

    // ================= saving =================
    pub fn to_json(&self) -> String {
        let mut map = vec![];
        for y in 0..8 {
            for x in 0..16 {
                let m = self.mark[x][y];
                let extra = if m == -1 { -1 } else { self.extra[x][y].max(0) };
                map.extend([json!(m), json!(extra), json!(self.circle[x][y])]);
            }
        }
        let box_json = |b: &Box_| json!({"CellCurrent": b.cell, "PlayerHas": b.has as i32});
        let dungeons: Vec<Value> = (0..9)
            .map(|i| {
                let d = &self.dungeons[i];
                let mut boxes: Vec<Value> = d.boxes.iter().map(box_json).collect();
                if i == self.third_box_owner() {
                    boxes.push(box_json(&self.third_box));
                }
                json!({"Triforce": d.triforce, "Color": d.color, "LabelChar": d.label_char, "PlayerHasMap": d.has_map, "Boxes": boxes})
            })
            .collect();
        let mut progress = Map::new();
        progress.insert("TakeAnyHearts".into(), json!(self.take_any_hearts));
        for k in PROGRESS_KEYS {
            progress.insert(k.into(), json!(self.prog(k)));
        }
        let mut starting = Map::new();
        starting.insert("HDNStartingTriforcePieces".into(), json!(self.starting_triforces));
        for k in STARTING_KEYS {
            starting.insert(k.into(), json!(self.starting[k]));
        }
        starting.insert("MaxHeartsDifferential".into(), json!(self.max_hearts_diff));
        let blockers: Vec<Value> = self
            .blockers
            .iter()
            .map(|row| Value::Array(row.iter().map(|b| json!({"Kind": BLOCKER_KINDS[b.kind], "AppliesTo": b.applies_to})).collect()))
            .collect();
        let mut hints = Map::new();
        hints.insert("LocationHints".into(), json!(self.location_hints));
        for k in HINT_FLAG_KEYS {
            hints.insert(k.into(), json!(self.hint_flags[k]));
        }
        let maps: Vec<Value> = self
            .maps
            .iter()
            .map(|m| {
                let hd: Vec<Vec<i32>> = (0..8).map(|y| (0..7).map(|x| m.hdoors[x][y]).collect()).collect();
                let vd: Vec<Vec<i32>> = (0..7).map(|y| (0..8).map(|x| m.vdoors[x][y]).collect()).collect();
                let rc: Vec<Vec<bool>> = (0..8).map(|y| (0..8).map(|x| m.circled[x][y]).collect()).collect();
                let rs: Vec<Vec<Value>> = (0..8)
                    .map(|y| {
                        (0..8)
                            .map(|x| {
                                let r = &m.rooms[x][y];
                                json!({
                                    "IsCompleted": r.completed,
                                    "RoomType": format!("RoomType_{}", ROOM_TYPES[r.kind]),
                                    "MonsterDetail": format!("MonsterDetail_{}", MONSTERS[r.monster]),
                                    "FloorDropDetail": format!("FloorDropDetail_{}", FLOOR_DROPS[r.floor_drop]),
                                    "FloorDropShouldAppearBright": r.floor_drop_bright,
                                })
                            })
                            .collect()
                    })
                    .collect();
                json!({"HorizontalDoors": hd, "VerticalDoors": vd, "RoomIsCircled": rc, "RoomStates": rs, "VanillaMapOverlay": m.vanilla_overlay})
            })
            .collect();
        let mut tl: Vec<(&String, &i64)> = self.timeline.iter().collect();
        tl.sort_by_key(|(_, s)| **s);
        let timeline: Vec<Value> = tl.iter().map(|(i, s)| json!({"Ident": i, "Seconds": s, "Has": 1})).collect();

        let mut root = Map::new();
        root.insert("Version".into(), json!(VERSION));
        root.insert("TimeInSeconds".into(), json!(self.seconds));
        root.insert(
            "Overworld".into(),
            json!({"Quest": self.quest, "MirrorOverworld": self.mirror, "StartIconX": self.start_icon.0, "StartIconY": self.start_icon.1,
                   "CustomWaypointX": self.waypoint.0, "CustomWaypointY": self.waypoint.1, "Map": map}),
        );
        root.insert(
            "Items".into(),
            json!({"HiddenDungeonNumbers": self.hidden_dungeon_numbers, "SecondQuestDungeons": self.second_quest_dungeons,
                   "WhiteSwordBox": box_json(&self.white_sword_box), "LadderBox": box_json(&self.ladder_box), "ArmosBox": box_json(&self.armos_box),
                   "Dungeons": dungeons}),
        );
        root.insert("PlayerProgressAndTakeAnyHearts".into(), Value::Object(progress));
        root.insert("StartingItemsAndExtras".into(), Value::Object(starting));
        root.insert("Blockers".into(), Value::Array(blockers));
        root.insert("Hints".into(), Value::Object(hints));
        root.insert("Notes".into(), json!(self.notes));
        root.insert("CurrentRecorderDestinationIndex".into(), json!(self.recorder_index));
        root.insert("RecorderToNewDungeons".into(), json!(self.recorder_to_new));
        root.insert("RecorderToUnbeatenDungeons".into(), json!(self.recorder_to_unbeaten));
        root.insert("IsBoomstickSeed".into(), json!(self.boomstick_seed));
        root.insert("IsAtlasSeed".into(), json!(self.atlas_seed));
        root.insert("IsWSMSReplacedByBUSeed".into(), json!(self.wsms_bu_seed));
        root.insert("DungeonTabSelected".into(), json!(self.tab));
        root.insert("DungeonMaps".into(), Value::Array(maps));
        root.insert("DrawingLayerIcons".into(), self.drawing_layer_icons.clone());
        root.insert("AlternativeOverworldMapFilename".into(), json!(self.alt_map_file));
        root.insert("ShouldInitiallyHideOverworldMap".into(), json!(self.hide_map_initially));
        root.insert("OverworldSpotsRemainingOverTime".into(), json!(self.spots_over_time));
        root.insert("Timeline".into(), Value::Array(timeline));
        // extra: what was taken at each Take Any / Wood Sword cave (the original ignores keys it doesn't know)
        let mut choices = vec![];
        for y in 0..8 {
            for x in 0..16 {
                if self.tile_choice[x][y] >= 0 {
                    choices.push(json!([x, y, self.tile_choice[x][y]]));
                }
            }
        }
        root.insert("LinuxTileChoices".into(), Value::Array(choices));
        serde_json::to_string_pretty(&Value::Object(root)).unwrap_or_default()
    }

    // ================= loading (also reads the original's own save files) =================
    pub fn load_json(&mut self, text: &str) -> Result<(), String> {
        let cleaned = strip_trailing_commas(text.trim_start_matches('\u{feff}'));
        let root: Value = serde_json::from_str(&cleaned).map_err(|e| e.to_string())?;
        let ow = root.get("Overworld").ok_or("this file has no Overworld section")?;
        let mut t = Tracker::blank();
        t.quest = int(ow, "Quest", 0);
        t.started = true;
        t.seconds = int(&root, "TimeInSeconds", 0);
        t.mirror = boolv(ow, "MirrorOverworld", false);
        t.start_icon = (int(ow, "StartIconX", -1), int(ow, "StartIconY", -1));
        t.waypoint = (int(ow, "CustomWaypointX", -1), int(ow, "CustomWaypointY", -1));
        if let Some(map) = ow.get("Map").and_then(|m| m.as_array()) {
            let v: Vec<i32> = map.iter().map(|e| e.as_i64().unwrap_or(-1) as i32).collect();
            for y in 0..8 {
                for x in 0..16 {
                    let i = (y * 16 + x) * 3;
                    if i + 2 < v.len() {
                        t.mark[x][y] = if (-1..38).contains(&v[i]) { v[i] } else { -1 };
                        t.extra[x][y] = v[i + 1];
                        t.circle[x][y] = v[i + 2];
                    }
                }
            }
        }
        if let Some(items) = root.get("Items") {
            t.hidden_dungeon_numbers = boolv(items, "HiddenDungeonNumbers", false);
            t.second_quest_dungeons = boolv(items, "SecondQuestDungeons", false);
            read_box(items, "WhiteSwordBox", &mut t.white_sword_box);
            read_box(items, "LadderBox", &mut t.ladder_box);
            read_box(items, "ArmosBox", &mut t.armos_box);
            if let Some(ds) = items.get("Dungeons").and_then(|d| d.as_array()) {
                for (i, d) in ds.iter().enumerate().take(9) {
                    let dd = &mut t.dungeons[i];
                    dd.triforce = boolv(d, "Triforce", false);
                    dd.color = int(d, "Color", 0);
                    dd.has_map = boolv(d, "PlayerHasMap", false);
                    dd.label_char = d.get("LabelChar").and_then(|v| v.as_str()).unwrap_or("?").to_string();
                    if let Some(bs) = d.get("Boxes").and_then(|b| b.as_array()) {
                        for (bi, b) in bs.iter().enumerate() {
                            let target = if bi < dd.boxes.len() { &mut dd.boxes[bi] } else { &mut t.third_box };
                            target.cell = int(b, "CellCurrent", -1) as i32;
                            target.has = Has::from_i64(int(b, "PlayerHas", 0));
                        }
                    }
                }
            }
        }
        if let Some(pp) = root.get("PlayerProgressAndTakeAnyHearts") {
            if let Some(a) = pp.get("TakeAnyHearts").and_then(|a| a.as_array()) {
                for (i, v) in a.iter().enumerate().take(4) {
                    t.take_any_hearts[i] = v.as_i64().unwrap_or(0);
                }
            }
            for k in PROGRESS_KEYS {
                t.progress.insert(k, boolv(pp, k, false));
            }
        }
        if let Some(si) = root.get("StartingItemsAndExtras") {
            if let Some(a) = si.get("HDNStartingTriforcePieces").and_then(|a| a.as_array()) {
                for (i, v) in a.iter().enumerate().take(8) {
                    t.starting_triforces[i] = v.as_bool().unwrap_or(false);
                }
            }
            for k in STARTING_KEYS {
                t.starting.insert(k, boolv(si, k, false));
            }
            t.max_hearts_diff = int(si, "MaxHeartsDifferential", 0);
        }
        if let Some(bl) = root.get("Blockers").and_then(|b| b.as_array()) {
            for (d, row) in bl.iter().enumerate().take(8) {
                if let Some(row) = row.as_array() {
                    for (s, b) in row.iter().enumerate().take(3) {
                        let kind = b.get("Kind").and_then(|k| k.as_str()).unwrap_or("");
                        t.blockers[d][s].kind = BLOCKER_KINDS.iter().position(|k| *k == kind).unwrap_or(BLOCKER_NOTHING);
                        if let Some(ap) = b.get("AppliesTo").and_then(|a| a.as_array()) {
                            for (i, v) in ap.iter().enumerate().take(6) {
                                t.blockers[d][s].applies_to[i] = v.as_bool().unwrap_or(false);
                            }
                        }
                    }
                }
            }
        }
        if let Some(h) = root.get("Hints") {
            if let Some(a) = h.get("LocationHints").and_then(|a| a.as_array()) {
                for (i, v) in a.iter().enumerate().take(11) {
                    t.location_hints[i] = (v.as_i64().unwrap_or(0) as usize).min(10);
                }
            }
            for k in HINT_FLAG_KEYS {
                t.hint_flags.insert(k, boolv(h, k, false));
            }
        }
        t.notes = root.get("Notes").and_then(|n| n.as_str()).unwrap_or("").to_string();
        t.recorder_index = int(&root, "CurrentRecorderDestinationIndex", 0);
        t.recorder_to_new = boolv(&root, "RecorderToNewDungeons", true);
        t.recorder_to_unbeaten = boolv(&root, "RecorderToUnbeatenDungeons", false);
        t.boomstick_seed = boolv(&root, "IsBoomstickSeed", false);
        t.atlas_seed = boolv(&root, "IsAtlasSeed", false);
        t.wsms_bu_seed = boolv(&root, "IsWSMSReplacedByBUSeed", false);
        t.tab = (int(&root, "DungeonTabSelected", 0).clamp(0, 8)) as usize;
        if let Some(dms) = root.get("DungeonMaps").and_then(|d| d.as_array()) {
            for (li, dm) in dms.iter().enumerate().take(9) {
                let m = &mut t.maps[li];
                for (y, row) in arr(dm, "HorizontalDoors").iter().enumerate().take(8) {
                    for (x, v) in row.as_array().into_iter().flatten().enumerate().take(7) {
                        m.hdoors[x][y] = v.as_i64().unwrap_or(0) as i32;
                    }
                }
                for (y, row) in arr(dm, "VerticalDoors").iter().enumerate().take(7) {
                    for (x, v) in row.as_array().into_iter().flatten().enumerate().take(8) {
                        m.vdoors[x][y] = v.as_i64().unwrap_or(0) as i32;
                    }
                }
                for (y, row) in arr(dm, "RoomIsCircled").iter().enumerate().take(8) {
                    for (x, v) in row.as_array().into_iter().flatten().enumerate().take(8) {
                        m.circled[x][y] = v.as_bool().unwrap_or(false);
                    }
                }
                for (y, row) in arr(dm, "RoomStates").iter().enumerate().take(8) {
                    for (x, r) in row.as_array().into_iter().flatten().enumerate().take(8) {
                        let room = &mut m.rooms[x][y];
                        room.completed = boolv(r, "IsCompleted", false);
                        room.kind = lookup(&ROOM_TYPES, r, "RoomType", "RoomType_");
                        room.monster = lookup(&MONSTERS, r, "MonsterDetail", "MonsterDetail_");
                        room.floor_drop = lookup(&FLOOR_DROPS, r, "FloorDropDetail", "FloorDropDetail_");
                        room.floor_drop_bright = boolv(r, "FloorDropShouldAppearBright", true);
                    }
                }
                m.vanilla_overlay = int(dm, "VanillaMapOverlay", 0);
            }
        }
        for c in arr(&root, "LinuxTileChoices") {
            if let Some(a) = c.as_array() {
                let v: Vec<i64> = a.iter().filter_map(|n| n.as_i64()).collect();
                if v.len() == 3 && (0..16).contains(&v[0]) && (0..8).contains(&v[1]) {
                    t.tile_choice[v[0] as usize][v[1] as usize] = v[2] as i32;
                }
            }
        }
        t.take_any_linked = t.take_any_tiles().len();
        t.drawing_layer_icons = root.get("DrawingLayerIcons").cloned().unwrap_or(json!([]));
        t.alt_map_file = root.get("AlternativeOverworldMapFilename").and_then(|v| v.as_str()).unwrap_or("").to_string();
        t.hide_map_initially = boolv(&root, "ShouldInitiallyHideOverworldMap", false);
        t.spots_over_time = arr(&root, "OverworldSpotsRemainingOverTime").iter().filter_map(|v| v.as_i64()).collect();
        for e in arr(&root, "Timeline") {
            let secs = int(e, "Seconds", -1);
            if let (Some(id), true) = (e.get("Ident").and_then(|v| v.as_str()), secs >= 0) {
                t.timeline.insert(id.to_string(), secs);
            }
        }
        *self = t;
        Ok(())
    }
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key).and_then(|a| a.as_array()).map(|a| a.as_slice()).unwrap_or(&[])
}
fn int(v: &Value, key: &str, def: i64) -> i64 {
    v.get(key).and_then(|x| x.as_i64()).unwrap_or(def)
}
fn boolv(v: &Value, key: &str, def: bool) -> bool {
    v.get(key).and_then(|x| x.as_bool()).unwrap_or(def)
}
fn read_box(parent: &Value, key: &str, b: &mut Box_) {
    if let Some(e) = parent.get(key) {
        b.cell = int(e, "CellCurrent", -1) as i32;
        b.has = Has::from_i64(int(e, "PlayerHas", 0));
    }
}
fn lookup(names: &[&str], v: &Value, key: &str, prefix: &str) -> usize {
    let s = v.get(key).and_then(|x| x.as_str()).unwrap_or("");
    let s = s.strip_prefix(prefix).unwrap_or(s);
    names.iter().position(|n| *n == s).unwrap_or(0)
}

/// The original writes JSON with trailing commas (e.g. `"MaxHeartsDifferential": 0,\n}`); drop them.
pub fn strip_trailing_commas(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let (mut in_str, mut esc) = (false, false);
    for (i, &c) in b.iter().enumerate() {
        if in_str {
            if esc {
                esc = false;
            } else if c == b'\\' {
                esc = true;
            } else if c == b'"' {
                in_str = false;
            }
            out.push(c);
        } else if c == b'"' {
            in_str = true;
            out.push(c);
        } else if c == b',' {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if !(j < b.len() && (b[j] == b'}' || b[j] == b']')) {
                out.push(c);
            }
        } else {
            out.push(c);
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_first_quest_has_73_spots_to_check() {
        let t = Tracker::new();
        assert_eq!(t.spots_remaining(), 73);
    }

    #[test]
    fn hidden_tiles_follow_the_originals_rules() {
        let mut t = Tracker::new();
        t.mark[1][1] = 25; // large secret
        t.mark[2][2] = SHOP_FIRST; // arrow shop ...
        t.extra[2][2] = 2; // ... also selling bombs
        t.mark[3][3] = SHOP_FIRST + 3; // candle shop
        let none = |_: &str| false;
        assert!(!t.tile_hidden(1, 1, &none));
        let on = |k: &str| k == "HideOverworldTile_LargeSecret" || k == "HideOverworldTile_Shop";
        assert!(t.tile_hidden(1, 1, &on));
        // shops: items still needed stay
        assert_eq!(t.shop_items_shown(2, 2, &on), vec![0, 1]);
        t.set_prog("PlayerHasWoodArrow", true);
        assert_eq!(t.shop_items_shown(2, 2, &on), vec![1]); // bombs are never hidden
        assert!(!t.tile_hidden(2, 2, &on));
        assert!(!t.tile_hidden(3, 3, &on));
        t.set_prog("PlayerHasBlueCandle", true);
        assert!(t.tile_hidden(3, 3, &on)); // nothing left to show
        // something taken there (extra data = the mark) keeps it shown
        t.extra[1][1] = 25;
        assert!(!t.tile_hidden(1, 1, &on));
        t.extra[1][1] = 0;
        // nothing is hidden once Zelda is rescued
        t.set_prog("PlayerHasRescuedZelda", true);
        assert!(!t.tile_hidden(1, 1, &on));
        assert!(!t.tile_hidden(3, 3, &on));
    }

    #[test]
    fn save_and_load_round_trip() {
        let mut t = Tracker::new();
        t.started = true;
        t.mark[3][4] = 5;
        t.mark[0][0] = SHOP_FIRST;
        t.extra[0][0] = 2;
        t.circle[1][1] = 165;
        t.dungeons[2].triforce = true;
        t.dungeons[0].boxes[1].cell = 13;
        t.dungeons[0].boxes[1].has = Has::Yes;
        t.third_box.cell = 2;
        t.blockers[4][2].kind = 3;
        t.maps[6].hdoors[2][5] = DOOR_YELLOW;
        t.maps[6].vdoors[7][1] = DOOR_NO;
        t.maps[6].rooms[4][3] = Room { completed: true, kind: 23, monster: 6, floor_drop: 5, floor_drop_bright: false };
        t.location_hints[9] = 7;
        t.notes = "line one\n\"quoted\"".into();
        t.seconds = 754;
        t.stamp("Triforce3", true);
        let json = t.to_json();
        let mut u = Tracker::new();
        u.load_json(&json).unwrap();
        assert_eq!(u.to_json(), json);
        assert_eq!(u.mark[3][4], 5);
        assert_eq!(u.extra[0][0], 2);
        assert_eq!(u.third_box.cell, 2);
        assert_eq!(u.maps[6].rooms[4][3].monster, 6);
        assert_eq!(u.maps[6].hdoors[2][5], DOOR_YELLOW);
        assert_eq!(u.notes, "line one\n\"quoted\"");
    }

    #[test]
    fn reads_the_originals_own_save_files() {
        let path = "/home/jason/Downloads/ZTracker_v2.1.0.2/zt-save-zz-autosave.json";
        let Ok(text) = std::fs::read_to_string(path) else { return };
        let mut t = Tracker::new();
        t.load_json(&text).unwrap();
        assert_eq!(t.seconds, 120);
        assert_eq!(t.uses_of_mark(DARK_X), 55);
        // a fresh first-quest tracker starts the same way
        assert_eq!(Tracker::new().mark, t.mark);
        assert_eq!(t.maps[0].vdoors[6][2], DOOR_YELLOW);
    }

    #[test]
    fn trailing_commas_are_dropped_but_not_inside_text() {
        assert_eq!(strip_trailing_commas("{\"a\": [1, 2,], \"b\": \"x,}\",}"), "{\"a\": [1, 2], \"b\": \"x,}\"}");
    }
}
