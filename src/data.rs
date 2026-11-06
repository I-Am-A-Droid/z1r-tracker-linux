//! Fixed tables. Values and names come from the mechanics sheet (MECHANICS.md),
//! which records where each one was found in the original program.

pub const VERSION: &str = "2.1.0";

// ---------- Item boxes (dungeon / white sword / ladder / armos boxes) ----------
// Index = value stored in a box's "CellCurrent"; -1 = empty box.
pub const ITEM_HOTKEY_NAMES: [&str; 15] = [
    "BookOrShield", "Boomerang", "Bow", "PowerBracelet", "Ladder", "MagicBoomerang", "AnyKey",
    "Raft", "Recorder", "RedCandle", "RedRing", "SilverArrow", "Wand", "WhiteSword", "HeartContainer",
];
pub const ITEM_NAMES: [&str; 15] = [
    "Book", "Wooden Boomerang", "Bow", "Power Bracelet", "Ladder", "Magic Boomerang", "Magic Key",
    "Raft", "Recorder", "Red Candle", "Red Ring", "Silver Arrow", "Wand", "White Sword", "Heart Container",
];
/// How many of each item exist in a seed.
pub const ITEM_MAX_COUNT: [usize; 15] = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9];
pub const HEART_CONTAINER: i32 = 14;
/// icons7x7.png picture for each item box value.
pub const ITEM_ICON7: [i32; 15] = [8, 0, 1, 16, 4, 2, 9, 3, 5, 7, 12, 10, 6, 17, 15];

// ---------- Overworld map marks ----------
pub const DARK_X: i32 = 35;
pub const ARMOS_MARK: i32 = 31;
/// Screen F16 (column 16, row F): the item off the coast, reached with the ladder. In the original this tile shows the
/// "ladder box" (WPFUI/makeAll@514-73: i = 15, j = 5) and takes no other mark.
pub const COAST_X: usize = 15;
pub const COAST_Y: usize = 5;
pub fn is_coast(x: usize, y: usize) -> bool {
    x == COAST_X && y == COAST_Y
}
pub const WHITE_SWORD_CAVE_MARK: i32 = 14;
pub const WOOD_SWORD_CAVE_MARK: i32 = 15;
pub const TAKE_ANY_MARK: i32 = 33;
pub const SHOP_FIRST: i32 = 16;
pub const SHOP_LAST: i32 = 23;

pub struct Tile {
    pub hotkey: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub max_1q: usize,
    pub max_2q: usize,
}

const fn t(hotkey: &'static str, name: &'static str, description: &'static str, max_1q: usize, max_2q: usize) -> Tile {
    Tile { hotkey, name, description, max_1q, max_2q }
}

/// Index = value stored in the save file's overworld "Map".
pub const TILES: [Tile; 38] = [
    t("Level1", "Level 1", "Dungeon", 1, 1),
    t("Level2", "Level 2", "Dungeon", 1, 1),
    t("Level3", "Level 3", "Dungeon", 1, 1),
    t("Level4", "Level 4", "Dungeon", 1, 1),
    t("Level5", "Level 5", "Dungeon", 1, 1),
    t("Level6", "Level 6", "Dungeon", 1, 1),
    t("Level7", "Level 7", "Dungeon", 1, 1),
    t("Level8", "Level 8", "Dungeon", 1, 1),
    t("Level9", "Level 9", "Final Dungeon", 1, 1),
    t("AnyRoad1", "Any Road 1", "Any Road 1/4", 1, 1),
    t("AnyRoad2", "Any Road 2", "Any Road 2/4", 1, 1),
    t("AnyRoad3", "Any Road 3", "Any Road 3/4", 1, 1),
    t("AnyRoad4", "Any Road 4", "Any Road 4/4", 1, 1),
    t("Sword3", "Magical Sword Cave", "Magical Sword Cave\n(10-14 hearts to lift)", 1, 1),
    t("Sword2", "White Sword Cave", "'White Sword' Cave\n(4-6 hearts to lift)\nNote: might have any item, not just white sword", 1, 1),
    t("Sword1", "Wood Sword Cave", "Wood Sword Cave\n(can always lift)", 1, 1),
    t("ArrowShop", "Wood Arrow shop", "Shop with Wood Arrows\n(60-100 rupees)", 999, 999),
    t("BombShop", "Bomb shop", "Shop with 4 Bomb Pack\n(1-40 rupees)", 999, 999),
    t("BookShop", "Book shop", "Shop with (Boomstick) Book\n(180-220 rupees)", 999, 999),
    t("CandleShop", "Candle shop", "Shop with Blue Candle\n(40-80 rupees)", 999, 999),
    t("BlueRingShop", "Blue ring shop", "Shop with Blue Ring\n(230-255 rupees)", 999, 999),
    t("MeatShop", "Meat shop", "Shop with Meat\n(40-120 rupees)", 999, 999),
    t("KeyShop", "Key shop", "Shop with Key\n(60-120 rupees)", 999, 999),
    t("ShieldShop", "Shield shop", "Shop with Magical Shield\n(70-180 rupees)", 999, 999),
    t("UnknownSecret", "Unknown Secret", "Unknown Secret", 999, 999),
    t("LargeSecret", "Large Secret", "Large Secret\n(50-150 rupees)", 999, 999),
    t("MediumSecret", "Medium Secret", "Medium Secret\n(25-40 rupees)", 999, 999),
    t("SmallSecret", "Small Secret", "Small Secret\n(1-20 rupees)", 999, 999),
    t("DoorRepairCharge", "Door Repair", "Door Repair Charge\n(15-25 rupees)", 9, 10),
    t("MoneyMakingGame", "Money Making Game", "Money Making Game\n(gambling)", 5, 6),
    t("Letter", "The Letter", "The Letter\n(for buying potions)", 1, 1),
    t("Armos", "Armos", "Armos Item", 1, 1),
    t("HintShop", "Hint Shop", "Hint Shop\n(10-60 rupees each)\nor free hint for white/magical sword cave", 4, 4),
    t("TakeAny", "Take Any", "Take Any One\nYou Want", 4, 4),
    t("PotionShop", "Potion Shop", "Potion Shop\n(20-60, 48-88 rupees)", 7, 9),
    t("DarkX", "Don't Care", "Don't Care", 999, 999),
    t("PersonalPref1", "Personal Pref 1", "Personal Preference 1\n(custom marker)", 999, 999),
    t("PersonalPref2", "Personal Pref 2", "Personal Preference 2\n(custom marker)", 999, 999),
];

/// Which picture in the overworld icon strip (16x11 pictures) shows each mark; -1 = drawn by the tracker.
pub const TILE_STRIP_INDEX: [i32; 38] = [
    2, 3, 4, 5, 6, 7, 8, 9, 10, // levels 1-9
    11, 12, 13, 14, // any roads
    19, 20, 21, // sword caves 3,2,1
    31, 30, 29, 28, 24, 25, 26, 23, // shops: arrow bomb book candle bluering meat key shield
    34, 35, 36, 37, // secrets: unknown large medium small
    33, 16, 17, 18, 22, 15, 32, // door repair, money game, letter, armos, hint, take any, potion
    38, -1, -1, // dark X, personal prefs
];

/// Shop items (second item of a shop, 1-based) and the icons7x7 picture for each.
pub const SHOP_ITEM_NAMES: [&str; 8] = ["Wood Arrow", "Bomb", "Book", "Blue Candle", "Blue Ring", "Meat", "Key", "Magical Shield"];
pub const SHOP_ITEM_ICON7: [i32; 8] = [11, 25, 8, 21, 22, 27, 9, 13];

// ---------- Overworld geography (16 columns x 8 rows) ----------
pub type Grid = [&'static str; 8];
pub const ZONES: Grid = ["MMMMMMMMMMLHHHCC", "MMMMMMMRRRRHHHCC", "GGGGGGLRDDDDDCCC", "GGGGLLLLLLDDFFCC", "GGLLLLLLLDDDFFFC", "GWWWLLLSSLLFFFFC", "GWWWWRSSSLLFFFFC", "WWWWWRSSSSSCCCCC"];
pub const ARMOS: Grid = ["................", "............X...", "....X...........", "....X........X..", "..............X.", "................", "................", "................"];
pub const RAFTABLE: Grid = ["................", "................", "...............X", "................", ".....X..........", "................", "................", "................"];
pub const BOMBABLE_1Q: Grid = [".X.X.X.X.....X..", "X.XXX.X.......X.", "......XX....XX..", "...X............", "................", "................", ".......X........", ".X....X....XXX.."];
pub const BOMBABLE_2Q: Grid = ["XXXX...X.....X..", "X.XXXXX.XX....X.", "......X......X..", "...X............", "................", "................", "................", "......X.....XX.."];
pub const BURNABLE_1Q: Grid = ["................", "................", "........X.......", "................", "......XXX..X.X..", ".X....X....X....", "..XX....X.XX.X..", "........X......."];
pub const BURNABLE_2Q: Grid = ["................", "................", "........X.......", "................", "......X.X..X.X..", ".X.X..X....X....", "...X....X.X.X...", "........X......."];
pub const BRACELET_1Q: Grid = ["................", ".............X..", "...X............", "................", ".........X......", "................", "................", ".........X......"];
pub const BRACELET_2Q: Grid = [".........X......", ".X.........X.X..", "...X............", "................", ".........X......", "................", "................", ".........X......"];
pub const LADDERABLE_2Q: Grid = ["................", "........XX......", "................", "................", "................", "................", "................", "................"];
pub const WHISTLE_1Q: Grid = ["................", "................", "................", "................", "..X.............", "................", "................", "................"];
pub const WHISTLE_2Q: Grid = ["......X.........", "................", ".........X.X....", "X.........X.X...", "................", "........X.......", "X.............X.", "..X............."];
pub const GRAVE_1Q: Grid = ["................", "................", ".X..............", "................", "................", "................", "................", "................"];
pub const GRAVE_2Q: Grid = ["................", "................", "X...............", "................", "................", "................", "................", "................"];
pub const ALWAYS_EMPTY_1Q: Grid = ["X.X...X.XX......", ".X...X.XXX.X....", "X........XXX..X.", "XXX..XX.XXXX..XX", "XX.X........X..X", "X.XXXX.XXXX.XX.X", "XX...X...X..X.X.", "..XX......X...XX"];
pub const ALWAYS_EMPTY_2Q: Grid = [".....X..X..X....", ".......X........", ".X.....X..X.X.X.", ".XX..XX.XX.X..XX", "XXXX...X....X..X", "X.X.XX.X.XX.XX.X", ".XX..X.X.X.X.X..", ".X.X......XX..XX"];

pub fn at(g: &Grid, x: usize, y: usize) -> bool {
    g[y].as_bytes()[x] == b'X'
}

/// Hint zones, in the order the save file numbers them.
pub const HINT_ZONE_NAMES: [&str; 11] = ["(Unknown)", "Death Mountain", "Lake", "Lost Hills", "River", "Grave", "Desert", "Coast", "Dead Woods", "Close to Start", "Forest"];
pub const HINT_ZONE_SHORT: [&str; 11] = ["UN", "DM", "LK", "LH", "RI", "GR", "DE", "CO", "DW", "ST", "FO"];
const HINT_ZONE_LETTERS: &str = "?MLHRGDCWSF";
pub fn zone_at(x: usize, y: usize) -> usize {
    let c = ZONES[y].as_bytes()[x] as char;
    HINT_ZONE_LETTERS.find(c).unwrap_or(0)
}

pub const HINT_MEANINGS: [(&str, &str); 11] = [
    ("Aquamentus Awaits", "Level 1"), ("Dodongo Dwells", "Level 2"), ("Manhandla Threatens", "Level 3"),
    ("Gleeok Lurks", "Level 4"), ("Digdogger Gazes", "Level 5"), ("Gohma Creeps", "Level 6"),
    ("Goriya Grumbles", "Level 7"), ("Gleeok Returns", "Level 8"), ("entrance to death", "Level 9"),
    ("(npc) has (item) at", "White Sword item"), ("Meet (npc) at", "Magical Sword"),
];

pub const VANILLA_1Q_DUNGEONS: [(usize, usize); 9] = [(7, 3), (12, 3), (4, 7), (5, 4), (11, 0), (2, 2), (2, 4), (13, 6), (5, 0)];
pub const VANILLA_2Q_DUNGEONS: [(usize, usize); 9] = [(7, 3), (4, 3), (12, 3), (11, 1), (5, 4), (0, 3), (12, 6), (8, 1), (0, 0)];

// ---------- Blockers ----------
pub const BLOCKER_KINDS: [&str; 16] = [
    "Blocker_Combat", "Blocker_Bow_And_Arrow", "Blocker_Recorder", "Blocker_Ladder", "Blocker_Bait", "Blocker_Key", "Blocker_Bomb", "Blocker_Money",
    "Blocker_Maybe_Bow_And_Arrow", "Blocker_Maybe_Recorder", "Blocker_Maybe_Ladder", "Blocker_Maybe_Bait", "Blocker_Maybe_Key", "Blocker_Maybe_Bomb", "Blocker_Maybe_Money",
    "Blocker_Nothing",
];
pub const BLOCKER_NAMES: [&str; 16] = [
    "Need better weapon/armor", "Need bow&arrow", "Need recorder", "Need ladder", "Need meat", "Need keys", "Need bombs", "Need money (e.g. mugger)",
    "Might need bow&arrow", "Might need recorder", "Might need ladder", "Might need meat", "Might need keys", "Might need bombs", "Might need money (e.g. mugger)",
    "Unmarked",
];
/// icons7x7 picture for each blocker kind (maybes reuse the same picture, drawn dimmer).
pub const BLOCKER_ICON7: [i32; 16] = [17, 26, 5, 4, 27, 9, 25, 29, 26, 5, 4, 27, 9, 25, 29, -1];
pub const BLOCKER_NOTHING: usize = 15;
pub fn blocker_is_maybe(k: usize) -> bool {
    (8..=14).contains(&k)
}

// ---------- Dungeon rooms ----------
pub const ROOM_TYPES: [&str; 34] = [
    "Unmarked", "NonDescript", "MaybePushBlock", "ItemBasement", "StaircaseToUnknown",
    "Transport1", "Transport2", "Transport3", "Transport4", "Transport5", "Transport6", "Transport7", "Transport8",
    "Chevy", "DoubleMoat", "TopMoat", "RightMoat", "CircleMoat", "Tee", "LavaMoat", "VChute", "HChute", "Turnstile",
    "OldManHint", "BombUpgrade", "LifeOrMoney", "HungryGoriyaMeatBlock",
    "StartEnterFromE", "StartEnterFromW", "StartEnterFromN", "StartEnterFromS", "OffTheMap", "Gannon", "Zelda",
];
pub const ROOM_TYPE_NAMES: [&str; 34] = [
    "(None)", "Generic Room", "Push Block?", "Item Basement", "Staircase",
    "Transport 1", "Transport 2", "Transport 3", "Transport 4", "Transport 5", "Transport 6", "Transport 7", "Transport 8",
    "Chevy Room", "Double Moat", "Top Moat", "Right Moat", "Circle Moat", "Tee Room", "Lava Moat", "Vertical Chute", "Horizontal Chute", "Turnstile",
    "Old Man (Hint)", "Bomb Upgrade", "Life or Money", "Meat Block",
    "Start (E)", "Start (W)", "Start (N)", "Start (S)", "Off the Map", "Ganon", "Zelda",
];
/// Picture in the 13x9 room-icon sheet for each room type (top half = not yet done, bottom half = done).
pub const ROOM_ICON: [i32; 34] = [0, 1, 10, 11, 25, 17, 18, 19, 20, 21, 22, 23, 24, 3, 2, 5, 4, 6, 9, 33, 7, 8, 16, 12, 15, 14, 13, 29, 26, 27, 28, 30, 31, 32];
pub const ROOM_UNMARKED: usize = 0;
pub const ROOM_NON_DESCRIPT: usize = 1;
pub const ROOM_MAYBE_PUSH_BLOCK: usize = 2;
pub fn is_entrance(t: usize) -> bool {
    (27..=30).contains(&t)
}

pub const MONSTERS: [&str; 32] = [
    "Unmarked", "Gleeok", "Bow", "Digdogger", "BlueBubble", "RedBubble", "Dodongo", "Patra", "BlueWizzrobe", "BlueDarknut",
    "Manhandla", "Vire", "Zol", "PolsVoice", "RedTektite", "RedGoriya", "Rope", "Stalfos", "Wallmaster", "Gel",
    "Keese", "Likelike", "Gibdo", "RedLynel", "BlueMoblin", "Aquamentus", "BlueLanmola", "Moldorm", "RupeeBoss", "Traps", "Other", "Other2",
];
pub const MONSTER_NAMES: [&str; 32] = [
    "(None)", "Gleeok", "Gohma", "Digdogger", "Blue Bubble", "Red Bubble", "Dodongo", "Patra", "Wizzrobe", "Darknut",
    "Manhandla", "Vire", "Zol", "Pols Voice", "Tektite", "Goriya", "Rope", "Stalfos", "Wallmaster", "Gel",
    "Keese", "Likelike", "Gibdo", "Lynel", "Moblin", "Aquamentus", "Lanmola", "Moldorm", "Rupee Boss", "Traps", "Other", "Other2",
];
/// Picture in the 16x16 monster sheet for each monster (-1 = none).
pub const MONSTER_ICON: [i32; 32] = [-1, 1, 2, 0, 8, 7, 6, 5, 4, 9, 3, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 10, 31];

pub const FLOOR_DROPS: [&str; 9] = ["Unmarked", "Triforce", "Heart", "OtherKeyItem", "BombPack", "Key", "FiveRupee", "Map", "Compass"];
pub const FLOOR_DROP_NAMES: [&str; 9] = ["(None)", "Triforce", "Heart Container", "Other Key Item", "Bomb Pack", "Key (single use)", "Five-Rupee", "Map", "Compass"];
/// Picture in the 16x16 Zelda item sheet.
pub const FLOOR_DROP_ICON: [i32; 9] = [-1, 0, 1, 7, 2, 3, 4, 5, 6];

// Door states as stored in the save file.
pub const DOOR_UNKNOWN: i32 = 0;
pub const DOOR_NO: i32 = 1;
pub const DOOR_YES: i32 = 2;
pub const DOOR_YELLOW: i32 = 3;
pub const DOOR_PURPLE: i32 = 4;
