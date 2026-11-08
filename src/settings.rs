//! Options are kept in the same file format as the original (Z1R_Tracker_settings.json),
//! in the app's settings folder (see platform.rs). Unknown keys are kept as they are.
//! Hot keys use the original's HotKeys.txt format ("SelectorName = key").

use crate::data::*;
use eframe::egui::{Key, Modifiers};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;

pub use crate::platform::{config_dir, save_dir};

pub const BOOL_OPTIONS: [(&str, &str, bool); 22] = [
    ("WideLayoutWhenRoomy", "Put the dungeons and timeline beside the map when the window is wide", true),
    ("Visual_CoastItem", "Remind me to get the coast item once I have the ladder", true),
    ("HighlightNearby", "Highlight nearby unmarked screens", true),
    ("Gettables", "Count the screens you can get to now", true),
    ("Zones", "Show hint zones on the map", false),
    ("Coords", "Show map coordinates", false),
    ("ShopsFirst", "Shops first in the map picker", false),
    ("DefaultRoomPreferNonDescriptToMaybePushBlock", "Clicking an unmarked room makes a generic room (not push block?)", false),
    ("ReverseScrollWheelDirection", "Reverse scroll wheel", false),
    ("HideTimer", "Hide the timer", false),
    ("SaveOnCompletion", "Save automatically when Zelda is rescued", false),
    ("BOARDInsteadOfLEVEL", "Say BOARD instead of LEVEL", false),
    ("IsSecondQuestDungeons", "Second-quest dungeons (level 4 gets the third box)", false),
    ("SmallerAppWindow", "Smaller app window", false),
    ("ShowBasementInfo", "Show basement info", true),
    ("DisplaySeedAndFlags", "Show seed and flags", true),
    ("AllowItemIconOnNonShopTile", "Right-click a non-shop tile to add an item icon", false),
    ("GiveDungeonTrackerSunglasses", "Dungeon 'sunglasses' (darken the bright dungeon map a little)", false),
    ("AlphabetizeHintZones", "Alphabetize hint zone list (clicking a hint goes through zones A-Z)", false),
    ("ShowMagnifier", "Magnifier: show the map screen under the mouse enlarged over the dungeon", false),
    ("AnimateShopHighlights", "Animate shop highlights (bold cyan settling to green)", true),
    ("StreamToOBS", "Stream the tracker to OBS over the network", false),
];

pub struct Settings {
    obj: Map<String, Value>,
    path: PathBuf,
}

impl Settings {
    pub fn load() -> Settings {
        let dir = config_dir();
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::create_dir_all(save_dir());
        let path = dir.join("Z1R_Tracker_settings.json");
        let obj = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&crate::model::strip_trailing_commas(&s)).ok())
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        Settings { obj, path }
    }

    pub fn save(&self) {
        if let Ok(s) = serde_json::to_string_pretty(&Value::Object(self.obj.clone())) {
            if let Err(e) = std::fs::write(&self.path, s) {
                eprintln!("Settings could not be saved: {e}");
            }
        }
    }

    pub fn get(&self, key: &str) -> bool {
        if let Some(b) = self.obj.get(key).and_then(|v| v.as_bool()) {
            return b;
        }
        BOOL_OPTIONS.iter().find(|o| o.0 == key).map(|o| o.2).unwrap_or(false)
    }
    pub fn set(&mut self, key: &str, v: bool) {
        self.obj.insert(key.into(), Value::Bool(v));
        self.save();
    }
    pub fn get_str(&self, key: &str) -> String {
        self.obj.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
    }
    pub fn set_str(&mut self, key: &str, v: &str) {
        self.obj.insert(key.into(), Value::String(v.into()));
        self.save();
    }
    pub fn get_f64(&self, key: &str, def: f64) -> f64 {
        self.obj.get(key).and_then(|v| v.as_f64()).unwrap_or(def)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct KeyChord {
    pub key: Key,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}
impl KeyChord {
    pub fn new(key: Key, m: Modifiers) -> Self {
        KeyChord { key, shift: m.shift, ctrl: m.ctrl || m.command, alt: m.alt }
    }
    pub fn describe(&self) -> String {
        let mut s = String::new();
        if self.ctrl { s += "CTRL "; }
        if self.alt { s += "ALT "; }
        if self.shift { s += "SHIFT "; }
        s + self.key.name()
    }
}

#[derive(Default)]
pub struct HotKeys {
    pub item: HashMap<KeyChord, String>,
    pub overworld: HashMap<KeyChord, String>,
    pub blocker: HashMap<KeyChord, String>,
    pub room: HashMap<KeyChord, String>,
    pub problems: Vec<String>,
}

pub fn hotkeys_path() -> PathBuf {
    config_dir().join("HotKeys.txt")
}

impl HotKeys {
    /// The key for an overworld selection ("Level1", "Nothing", …), for "HotKey = …" in descriptions (the original
    /// shows the first one, HotKeys.fs AppendHotKeyToDescription; here the first by name when several are set).
    pub fn overworld_key_for(&self, sel: &str) -> Option<String> {
        self.overworld.iter().filter(|(_, v)| v.as_str() == sel).map(|(k, _)| k.describe()).min()
    }

    pub fn load() -> HotKeys {
        let mut hk = HotKeys::default();
        let path = hotkeys_path();
        if !path.exists() {
            let _ = std::fs::write(&path, template());
        }
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        for (n, raw) in text.lines().enumerate() {
            let line = raw.trim().trim_start_matches('\u{feff}');
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some(eq) = line.find('=') else {
                hk.problems.push(format!("line {}: missing '='", n + 1));
                continue;
            };
            let name = line[..eq].trim();
            let key_text = line[eq + 1..].trim();
            if key_text.is_empty() {
                continue;
            }
            let Some(chord) = parse_chord(key_text) else {
                hk.problems.push(format!("line {}: unknown key '{key_text}'", n + 1));
                continue;
            };
            let (table, sel) = if let Some(s) = name.strip_prefix("Item_") {
                (&mut hk.item, s.to_string())
            } else if let Some(s) = name.strip_prefix("Overworld_") {
                (&mut hk.overworld, s.to_string())
            } else if name.starts_with("Blocker_") {
                (&mut hk.blocker, name.to_string())
            } else if let Some(s) = name.strip_prefix("DungeonRoom_") {
                (&mut hk.room, s.to_string())
            } else {
                hk.problems.push(format!("line {}: unknown selector '{name}'", n + 1));
                continue;
            };
            table.insert(chord, sel);
        }
        hk
    }
}

fn parse_chord(text: &str) -> Option<KeyChord> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    let (mods, last) = parts.split_at(parts.len() - 1);
    let mut c = KeyChord { key: Key::A, shift: false, ctrl: false, alt: false };
    for m in mods {
        match m.to_uppercase().as_str() {
            "SHIFT" => c.shift = true,
            "CTRL" => c.ctrl = true,
            "ALT" => c.alt = true,
            _ => return None,
        }
    }
    let k = last[0];
    c.key = if let Some(code) = k.strip_prefix('\\') {
        wpf_key(code.parse().ok()?)?
    } else if k.len() == 1 {
        let ch = k.chars().next()?.to_ascii_uppercase();
        Key::from_name(&ch.to_string())?
    } else {
        Key::from_name(k)?
    };
    Some(c)
}

/// The original numbers keys like .NET's System.Windows.Input.Key (e.g. \75 = NumPad1).
fn wpf_key(code: u32) -> Option<Key> {
    let name = match code {
        34..=43 => ((b'0' + (code - 34) as u8) as char).to_string(),
        44..=69 => ((b'A' + (code - 44) as u8) as char).to_string(),
        74..=83 => ((b'0' + (code - 74) as u8) as char).to_string(), // numpad digits act like digits
        90..=109 => format!("F{}", code - 89),
        2 => "Backspace".into(),
        3 => "Tab".into(),
        6 => "Enter".into(),
        13 => "Escape".into(),
        18 => "Space".into(),
        23 => "ArrowLeft".into(),
        24 => "ArrowUp".into(),
        25 => "ArrowRight".into(),
        26 => "ArrowDown".into(),
        84 => "*".into(),
        85 => "+".into(),
        87 => "-".into(),
        89 => "/".into(),
        _ => return None,
    };
    Key::from_name(&name)
}

fn template() -> String {
    let mut s = String::from(
        "# Z-Tracker for Linux - HotKeys\n\
         # Form: 'SelectorName = key'. key is 0-9 or a-z, or \\nnn (a numeric key code as in the original),\n\
         # optionally preceded by SHIFT, CTRL or ALT. Leave the key blank for no hotkey.\n\
         # Lines that are blank or start with '#' are ignored.\n\n\
         # ITEMS - work while the mouse is over an item box\n",
    );
    for n in ITEM_HOTKEY_NAMES {
        s += &format!("Item_{n} = \n");
    }
    s += "Item_Nothing = \n\n# OVERWORLD - work while the mouse is over an overworld map tile\n";
    for t in TILES.iter() {
        s += &format!("Overworld_{} = \n", t.hotkey);
    }
    s += "Overworld_Nothing = \n\n# BLOCKERS - work while the mouse is over a blocker box\n";
    for b in BLOCKER_KINDS {
        s += &format!("{b} = \n");
    }
    s += "\n# DUNGEON ROOMS - work while the mouse is over a dungeon room\n";
    for r in ROOM_TYPES {
        s += &format!("DungeonRoom_RoomType_{r} = \n");
    }
    for m in MONSTERS {
        s += &format!("DungeonRoom_MonsterDetail_{m} = \n");
    }
    for f in FLOOR_DROPS {
        s += &format!("DungeonRoom_FloorDropDetail_{f} = \n");
    }
    s
}
