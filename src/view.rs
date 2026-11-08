//! The whole tracker, drawn by hand on one 768 x 967 surface (the original's own canvas size).
//! What the mouse and keyboard do is in input.rs.

use crate::assets::Assets;
use crate::data::*;
use crate::model::*;
use crate::settings::{HotKeys, Settings};
use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};
use std::path::PathBuf;

pub const W: f32 = 768.0;
pub const H: f32 = 967.0;
/// The wide arrangement: map part on the left, the lower part (967 - 477 = 490 tall) beside it.
pub const WIDE_W: f32 = 2.0 * W + DIVIDER_W;
/// The line between the two halves of the wide arrangement: 3 px, like the original's RED_LINE in
/// Layout/ShorterApplicationLayout::.ctor (token 0x06000dcc, RVA 0x32b50); its colour is taken from that name.
pub const DIVIDER_W: f32 = 3.0;
pub const DIVIDER_COLOR: Color32 = Color32::RED;
pub const WIDE_H: f32 = H - DUNGEON_Y;

// Layout (see MECHANICS.md for where each number comes from)
pub const CELL: f32 = 30.0; // main tracker / item grid cell
pub const ITEM_GRID_X: f32 = 280.0;
pub const ITEM_GRID_Y: f32 = 30.0;
pub const MAP_Y: f32 = 150.0;
pub const TILE_W: f32 = 48.0;
pub const TILE_H: f32 = 33.0; // map 16x8 tiles, through y = 414
/// A mark on a map tile: the 16x11 marker picture at twice its size (the tile shows it at three times).
pub const MARK_BADGE_W: f32 = 32.0;
pub const MARK_BADGE_H: f32 = 22.0;
pub const LEGEND_Y: f32 = 414.0;
pub const PROGRESS_Y: f32 = 447.0;
pub const FIRST_PROGRESS_X: f32 = 129.0;
pub const DUNGEON_Y: f32 = 477.0;
pub const TAB_H: f32 = 24.0;
pub const DUNGEON_HEADER_H: f32 = 30.0;
pub const ROOM_W: f32 = 39.0;
pub const ROOM_H: f32 = 27.0;
/// A room's monster and floor-drop pictures (DungeonRoomState.CurrentDisplayEx$cont@191, token 0x06002a83, RVA
/// 0x977e0, in a 39 x 27 room canvas): 18 x 18, the monster at (-5,-3), the floor drop at (44-18, 30-18), and the black
/// square laid over them when darkened at opacity 0.5.
pub const ROOM_DETAIL_SIZE: f32 = 18.0;
pub const MONSTER_AT: (f32, f32) = (-5.0, -3.0);
pub const FLOOR_DROP_AT: (f32, f32) = (44.0 - 18.0, 30.0 - 18.0);
pub const DONE_DARKEN: f32 = 0.5;
/// Monsters that stay bright in a completed room: BlueBubble (4), RedBubble (5), Traps (29), Other (30), Other2 (31)
/// (the same function's switches on MonsterDetail − 4 and − 29).
pub fn monster_stays_bright(m: usize) -> bool {
    matches!(m, 4 | 5 | 29 | 30 | 31)
}
pub const ROOM_PITCH_X: f32 = 51.0;
pub const ROOM_PITCH_Y: f32 = 39.0;
pub const ROOM_X0: f32 = 6.0;
pub const BLOCKERS_X: f32 = 450.0;
pub const BLOCKER_ROW_H: f32 = 36.0;
pub const TIMELINE_Y: f32 = 840.0;
pub const TIMER_X: f32 = 572.0;
pub const NOTES_RECT: (f32, f32, f32, f32) = (450.0, 600.0, 310.0, 232.0);
/// Disk icon shown after an autosave: x = OMTW*16 - 40, y = START_TIMELINE_H + 60 (Layout/ApplicationLayout::AddDiskIcon).
pub const DISK_ICON_X: f32 = TILE_W * 16.0 - 40.0;
pub const DISK_ICON_Y: f32 = TIMELINE_Y + 60.0;
pub const DISK_FLASH_MS: u64 = 300;
/// Reminder timing: settings "ReminderIntervalMinutes" (5 in the original's settings file), times 0.6 for the
/// coast item reminder (Reminders/PeriodicReminders::Check, token 0x06000d4a, RVA 0x31178).
pub const REMINDER_INTERVAL_MINUTES: f64 = 5.0;
pub const REMINDER_INTERVAL_FACTOR: f64 = 0.6;

#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    None,
    MapTile(usize, usize),
    Box(BoxId),
    Triforce(usize),
    DungeonLabel(usize),
    Toggle14,
    Progress(&'static str),
    TakeAny(usize),
    HeartsAdjust,
    Timer,
    Button(&'static str),
    Tab(usize),
    Room(usize, usize),
    HDoor(usize, usize),
    VDoor(usize, usize),
    Blocker(usize, usize),
    StartChoice(usize),
    StartingItems,
}

#[derive(Clone)]
pub enum Icon {
    None,
    Item7(i32, f32),
    Ow(i32),
    Room(i32, bool),
    Boss(i32),
    Zitem(i32),
    Icon10(i32, f32),
}

#[derive(Clone)]
pub struct Choice {
    pub icon: Icon,
    pub text: String,
    pub label: String,
    pub enabled: bool,
    pub selected: bool,
    pub fill: Option<Color32>,
}
impl Choice {
    pub fn new(icon: Icon, text: impl Into<String>, label: impl Into<String>) -> Self {
        Choice { icon, text: text.into(), label: label.into(), enabled: true, selected: false, fill: None }
    }
    pub fn sel(mut self, s: bool) -> Self {
        self.selected = s;
        self
    }
    pub fn en(mut self, e: bool) -> Self {
        self.enabled = e;
        self
    }
}

/// What a pop-up does when one of its choices is clicked.
#[derive(Clone)]
pub enum Action {
    Mark(usize, usize, Vec<i32>),
    ShopSecond(usize, usize),
    Item(BoxId),
    HintZone(usize),
    RoomType(usize, usize),
    Monster(usize, usize),
    FloorDrop(usize, usize),
    Blocker(usize, usize),
    ResetTimer,
    StartOver,
    ResetKeepMaps,
    ResetMenu,
    /// true: first-quest-only spots; false: second-quest-only
    QuestOnlyDontCare(bool),
    Options,
    HideTiles,
    Hints,
    Starting,
    RecorderSettings,
    LoadList(Vec<PathBuf>),
    TakeAny(usize, usize),
    TakeThis(usize, usize),
}

pub struct Popup {
    pub title: String,
    pub choices: Vec<Choice>,
    pub cols: usize,
    pub cell: Vec2,
    pub anchor: Pos2,
    pub keep_open: bool,
    pub hover: Option<usize>,
    pub bounds: Rect,
    pub action: Action,
    /// how many lines the description at the bottom may take
    pub foot_lines: usize,
}

const PAD: f32 = 8.0;
const TITLE_H: f32 = 22.0;
const FOOT_H: f32 = 22.0;
const FOOT_LINE_H: f32 = 16.0;

impl Popup {
    pub fn new(title: impl Into<String>, cols: usize, cw: f32, ch: f32, anchor: Pos2, action: Action) -> Self {
        Popup { title: title.into(), choices: vec![], cols, cell: Vec2::new(cw, ch), anchor, keep_open: false, hover: None, bounds: Rect::NOTHING, action, foot_lines: 1 }
    }
    pub fn layout(&mut self, size: Vec2) {
        let rows = self.choices.len().div_ceil(self.cols);
        let w = self.cols as f32 * self.cell.x + PAD * 2.0;
        let h = rows as f32 * self.cell.y + PAD * 2.0 + TITLE_H + FOOT_H + (self.foot_lines.max(1) - 1) as f32 * FOOT_LINE_H;
        let x = (self.anchor.x - w / 2.0).clamp(2.0, (size.x - w - 2.0).max(2.0));
        let y = self.anchor.y.clamp(2.0, (size.y - h - 2.0).max(2.0));
        self.bounds = Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h));
    }
    pub fn cell_rect(&self, i: usize) -> Rect {
        let (c, r) = (i % self.cols, i / self.cols);
        Rect::from_min_size(
            Pos2::new(self.bounds.min.x + PAD + c as f32 * self.cell.x, self.bounds.min.y + PAD + TITLE_H + r as f32 * self.cell.y),
            self.cell,
        )
    }
    pub fn hit(&self, p: Pos2) -> Option<usize> {
        (0..self.choices.len()).find(|&i| self.cell_rect(i).contains(p))
    }
}

pub struct View {
    pub m: Tracker,
    pub assets: Assets,
    pub settings: Settings,
    pub hotkeys: HotKeys,
    pub hits: Vec<(Rect, Target)>,
    pub hover: Target,
    pub mouse: Pos2,
    pub popup: Option<Popup>,
    toast: String,
    toast_until: f64,
    disk_until: f64,
    ladder_reminder_at: f64,
    now: f64,
    pub paused: bool,
    pub commands: Vec<&'static str>,
    pub load_request: Option<PathBuf>,
    pub origin: Vec2,
    /// drawing offset for the part being drawn (non-zero for the lower part in the wide arrangement)
    shift: Vec2,
    pub wide: bool,
    /// the shop item being located on the map, and when it started (for the original's brief animation)
    pub locating: Option<(Locate, f64)>,
}

pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h))
}

/// The timer text, as the original writes it (Program/MyWindow::Update, token 0x0600004e, RVA 0x68a4):
/// "%2d:%02d:%02d" of TimeSpan.Hours / Minutes / Seconds, so hours are padded to two places with a space and
/// wrap after 23 (TimeSpan.Hours leaves out whole days).
pub fn timer_text(seconds: i64) -> String {
    let s = seconds.max(0);
    format!("{:>2}:{:02}:{:02}", (s / 3600) % 24, (s / 60) % 60, s % 60)
}

/// A w x h box centered in `outer`, on whole pixels so the pictures stay sharp.
pub fn centered_in(outer: Rect, w: f32, h: f32) -> Rect {
    rect((outer.min.x + (outer.width() - w) / 2.0).floor(), (outer.min.y + (outer.height() - h) / 2.0).floor(), w, h)
}
pub const ORANGE: Color32 = Color32::from_rgb(255, 165, 0);
pub const LIGHT_GREEN: Color32 = Color32::from_rgb(144, 238, 144);
pub const LIME: Color32 = Color32::from_rgb(50, 205, 50);
/// The shop locator outline (Graphics.fs TileHighlightRectangle): ends 4 px lime; animated from 12 px cyan over
/// 0.5 s; without animation "green" = desaturateColor(Lime, 0.5) = rgb(76,204,76).
pub const SHOP_HIGHLIGHT_W: f32 = 4.0;
pub const SHOP_HIGHLIGHT_SECS: f64 = 0.5;
pub const SHOP_HIGHLIGHT_PLAIN: Color32 = Color32::from_rgb(76, 204, 76);

/// Which shop item (0-7, see SHOP_ITEM_NAMES) a top-section toggle locates (OverworldItemGridUI.fs MouseEnter
/// handlers): wood arrow, bombs, boomerang-or-book (the book), blue ring; the blue candle only while no candle is
/// had (CandleLevel = 0; otherwise the original shows burnable bushes instead).
pub fn shop_item_for_progress(key: &str, have_candle: bool) -> Option<i32> {
    match key {
        "PlayerHasWoodArrow" => Some(0),
        "PlayerHasBombs" => Some(1),
        "PlayerHasBoomBook" => Some(2),
        "PlayerHasBlueCandle" if !have_candle => Some(3),
        "PlayerHasBlueRing" => Some(4),
        _ => None,
    }
}

/// The book's picture in icons7x7 (used for the "boomerang or book" box, which shows simply as the book).
pub const BOOK_ICON7: i32 = 8;

/// The name shown for a top-section item: "Book" for the boomerang-or-book box, else the save name without "PlayerHas".
pub fn progress_name(key: &str) -> &str {
    match key {
        "PlayerHasBoomBook" => "Book",
        k => k.strip_prefix("PlayerHas").unwrap_or(k),
    }
}

/// Where the Spot Summary panel goes (Layout.fs AddSpotSummary: canvasAdd(…, 50., 3.)).
pub const SPOT_SUMMARY_X: f32 = 50.0;
pub const SPOT_SUMMARY_Y: f32 = 3.0;

/// What the mouse is locating on the map: shops selling an item (0-7), or a sword cave (by its mark).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Locate {
    Shop(i32),
    Cave(i32),
}

/// The original's showLocator for the sword icons (OverworldItemGridUI.fs: wood_sword_box → Sword1, white sword →
/// Sword2, mags_box → Sword3): the cave's mark.
pub fn sword_cave_for(key: &str) -> Option<Locate> {
    match key {
        "PlayerHasWoodSword" => Some(Locate::Cave(15)),
        "PlayerHasMagicalSword" => Some(Locate::Cave(13)),
        _ => None,
    }
}

/// The hint used when a sword cave isn't marked (WPFUI.fs showLocator: Sword2 → GetLevelHint(9), Sword3 → 10; the
/// wood sword cave has none).
pub fn sword_hint_index(mark: i32) -> Option<usize> {
    match mark {
        14 => Some(9),
        13 => Some(10),
        _ => None,
    }
}

pub fn sword_cave_name(mark: i32) -> &'static str {
    match mark {
        13 => "magical sword cave",
        14 => "white sword cave",
        _ => "wood sword cave",
    }
}

/// How dark the screens outside a hinted zone get (WPFUI.fs showLocatorHintedZone: opacity 0.7).
pub const HINTED_ZONE_DARKEN: u8 = 179;

/// Which shop item a dungeon blocker locates (UIComponents.fs blocker MouseEnter): meat, keys, bombs, bow and arrow
/// (the "maybe" kinds too).
pub fn shop_item_for_blocker(kind: &str) -> Option<i32> {
    let k = kind.trim_start_matches("Blocker_").trim_start_matches("Maybe_");
    match k {
        "Bait" => Some(5),
        "Key" => Some(6),
        "Bomb" => Some(1),
        "Bow_And_Arrow" => Some(0),
        _ => None,
    }
}
pub const INDIAN_RED: Color32 = Color32::from_rgb(205, 92, 92);
// item box borders and door colours, as in the original (see MECHANICS.md)
/// Orange behind shop items on the map (Graphics .cctor: itemBackgroundColor = Color.FromArgb(239, 131, 0)).
pub const SHOP_ITEM_BACKGROUND: Color32 = Color32::from_rgb(239, 131, 0);
pub const BOX_NO: Color32 = Color32::from_rgb(168, 0, 0);
pub const BOX_YES: Color32 = Color32::from_rgb(50, 168, 50);
pub const BOX_SKIPPED: Color32 = Color32::from_rgb(147, 112, 219);
pub const DOOR_COLORS: [Color32; 5] = [
    Color32::from_rgb(30, 30, 45),   // unknown
    Color32::from_rgb(145, 0, 0),    // no
    Color32::from_rgb(60, 120, 60),  // yes
    Color32::from_rgb(160, 160, 40), // yellow
    Color32::from_rgb(140, 30, 140), // purple
];
/// Circle colours on the overworld, picked by the saved circle value (WPFUI::owCircleColor@361):
/// below 100 = bright blue, 100-199 = pink, 200 and up = yellow (brushes made in WPFUI/makeAll@318-46).
pub const CIRCLE_COLORS: [Color32; 3] = [Color32::from_rgb(0, 208, 255), Color32::from_rgb(255, 64, 153), Color32::YELLOW];
/// Circle size and line (WPFUI::makeOwCircle@358: 33 - 2 = 31 px across, line 3 px) and its label font (12 pt bold).
pub const CIRCLE_SIZE: f32 = 31.0;
pub const CIRCLE_LINE: f32 = 3.0;
pub const CIRCLE_LABEL_SIZE: f32 = 12.0;
/// "Don't care" tiles: a black overlay at opacity 0.55 (WPFUI::drawDarkening@404), so the map keeps 45% of its brightness.
pub const DONT_CARE_OPACITY: f32 = 0.55;
/// The recorder destination diamond: a square this wide, turned 45 degrees (WPFUI.fs recorderingCanvas).
pub const RECORDER_DIAMOND: f32 = 35.0;
/// Size of one triangle step in the triforce reference diagram (the original uses TRIFORCE_SIZE = 80 in its own,
/// larger window; scaled to fit beside the item reference here).
pub const TRIFORCE_REF_SIZE: f32 = 52.0;
/// Where the turned square's corner is placed in the screen (WPFUI/makeAll@1074-166 RoutingInfo: OMTW*i+23, 33*j-9).
pub const RECORDER_DIAMOND_X: f32 = 23.0;
pub const RECORDER_DIAMOND_Y: f32 = -9.0;
/// Its colour: rgb(220,220,220) for "Recorder to new dungeons", white for the vanilla locations
/// (UIComponents.fs RecorderEllipseColor).
pub fn recorder_color(to_new_dungeons: bool) -> Color32 {
    if to_new_dungeons { Color32::from_rgb(220, 220, 220) } else { Color32::WHITE }
}

/// The "Dungeon 'sunglasses'" option: how dark, and how far past the room map it reaches.
pub const SUNGLASSES_OPACITY: f32 = 0.15;
pub const SUNGLASSES_MARGIN: f32 = 3.0;

/// Whether a map screen is shown darkened, as in the original (`WPFUI::redrawGridSpot$cont@602`, token 0x060003b8,
/// RVA 0xe074): a screen with a mark gets `drawDarkening` (black at opacity 0.55) under its marker, and a "don't
/// care" screen the same darkness (the original lays an X picture over it at opacity 0.55; here, on request, it is
/// just darkened, without the X). Screens that never hold anything are darkened too.
pub fn tile_is_shaded(mark: i32, always_empty: bool) -> bool {
    mark >= 0 || always_empty
}
const PANEL: Color32 = Color32::from_rgb(20, 20, 28);

impl View {
    pub fn new(m: Tracker, assets: Assets, settings: Settings, hotkeys: HotKeys) -> Self {
        View {
            m, assets, settings, hotkeys, hits: vec![], hover: Target::None, mouse: Pos2::ZERO, popup: None,
            toast: String::new(), toast_until: 0.0, disk_until: 0.0, ladder_reminder_at: 0.0, now: 0.0, paused: false, commands: vec![], load_request: None, origin: Vec2::ZERO, shift: Vec2::ZERO, wide: false,
            locating: None,
        }
    }

    /// After an autosave a disk icon shows for 300 ms (WPFUI/makeAll@2022-416: opacity 0.7, Sleep 300).
    pub fn flash_disk_icon(&mut self) {
        self.disk_until = self.now + DISK_FLASH_MS as f64 / 1000.0;
    }

    /// Reminders play again from the start (TrackerModel.ResetForGroundhogOrRoutersOrFourPlusFourEtc).
    pub fn restart_reminders(&mut self) {
        self.ladder_reminder_at = 0.0;
    }

    pub fn show_toast(&mut self, s: impl Into<String>, secs: f64) {
        self.toast = s.into();
        self.toast_until = self.now + secs;
    }

    // ------------------------------------------------------------------ drawing helpers (local coordinates)
    pub fn r(&self, r: Rect) -> Rect {
        r.translate(self.origin + self.shift)
    }
    fn fill(&self, p: &Painter, r: Rect, c: Color32) {
        p.rect_filled(self.r(r), 0.0, c);
    }
    fn fill_round(&self, p: &Painter, r: Rect, c: Color32, rad: u8) {
        p.rect_filled(self.r(r), CornerRadius::same(rad), c);
    }
    fn stroke(&self, p: &Painter, r: Rect, w: f32, c: Color32) {
        p.rect_stroke(self.r(r), 0.0, Stroke::new(w, c), StrokeKind::Inside);
    }
    fn stroke_round(&self, p: &Painter, r: Rect, w: f32, c: Color32, rad: u8) {
        p.rect_stroke(self.r(r), CornerRadius::same(rad), Stroke::new(w, c), StrokeKind::Inside);
    }
    fn line(&self, p: &Painter, a: Pos2, b: Pos2, w: f32, c: Color32) {
        p.line_segment([a + self.origin + self.shift, b + self.origin + self.shift], Stroke::new(w, c));
    }
    fn text(&self, p: &Painter, x: f32, y: f32, size: f32, c: Color32, s: &str) {
        p.text(Pos2::new(x, y) + self.origin + self.shift, Align2::LEFT_TOP, s, FontId::proportional(size), c);
    }
    fn text_c(&self, p: &Painter, r: Rect, size: f32, c: Color32, s: &str) {
        p.text(self.r(r).center(), Align2::CENTER_CENTER, s, FontId::proportional(size), c);
    }
    fn measure(p: &Painter, size: f32, s: &str) -> Vec2 {
        p.layout_no_wrap(s.to_string(), FontId::proportional(size), Color32::WHITE).size()
    }
    fn hit(&mut self, r: Rect, t: Target) {
        self.hits.push((r.translate(self.shift), t));
    }

    /// The whole tracker's size: tall (768 x 967), or wide (1536 x 490) with the dungeon maps, blockers, notes and
    /// timeline to the right of the map.
    pub fn size(&self) -> Vec2 {
        if self.wide { Vec2::new(WIDE_W, WIDE_H) } else { Vec2::new(W, H) }
    }
    /// How far the lower part (from the dungeon tabs down) moves in the wide arrangement.
    pub fn lower_shift(&self) -> Vec2 {
        if self.wide { Vec2::new(W + DIVIDER_W, -DUNGEON_Y) } else { Vec2::ZERO }
    }
    /// Where the notes box is, in tracker coordinates.
    pub fn notes_rect(&self) -> Rect {
        rect(NOTES_RECT.0, NOTES_RECT.1, NOTES_RECT.2, NOTES_RECT.3).translate(self.lower_shift())
    }
    pub fn hit_test(&self, p: Pos2) -> Target {
        self.hits.iter().rev().find(|(r, _)| r.contains(p)).map(|(_, t)| t.clone()).unwrap_or(Target::None)
    }
    fn centered(r: Rect, w: f32, h: f32) -> Rect {
        Rect::from_center_size(r.center(), Vec2::new(w, h))
    }

    /// The coast item reminder (Reminders/PeriodicReminders::Check): every ReminderIntervalMinutes × 0.6 minutes, if the
    /// player has the ladder but the coast box isn't done, remind them. Shown on screen when "Visual_CoastItem" is on.
    pub fn coast_reminder_text(&self) -> Option<String> {
        if !self.m.started || !self.m.player_has_item(4) || self.m.ladder_box.is_done() {
            return None;
        }
        let cell = self.m.ladder_box.cell;
        if cell == -1 {
            return Some("Get the coast item with the ladder".into());
        }
        // a white sword off the coast doesn't matter once you have the magical sword
        if cell == 13 && self.m.prog("PlayerHasMagicalSword") {
            return None;
        }
        Some(format!("Get the {} off the coast", ITEM_NAMES[cell as usize].to_lowercase()))
    }

    fn check_coast_reminder(&mut self) {
        let interval_minutes = self.settings.get_f64("ReminderIntervalMinutes", REMINDER_INTERVAL_MINUTES);
        let wait = interval_minutes * REMINDER_INTERVAL_FACTOR * 60.0;
        if self.now - self.ladder_reminder_at < wait {
            return;
        }
        self.ladder_reminder_at = self.now;
        if !self.settings.get("Visual_CoastItem") {
            return;
        }
        if let Some(text) = self.coast_reminder_text() {
            self.show_toast(text, 6.0);
        }
    }

    // ------------------------------------------------------------------ the frame
    pub fn paint(&mut self, ui: &egui::Ui, origin: Vec2) {
        self.origin = origin;
        self.now = ui.ctx().input(|i| i.time);
        self.check_coast_reminder();
        // what (if anything) the mouse is locating on the map; restart the brief animation when it changes
        let have_candle = self.m.player_has_item(9) || self.m.prog("PlayerHasBlueCandle");
        let item = match self.hover {
            Target::Progress(key) => sword_cave_for(key).or(shop_item_for_progress(key, have_candle).map(Locate::Shop)),
            Target::Box(BoxId::WhiteSword) => Some(Locate::Cave(WHITE_SWORD_CAVE_MARK)),
            Target::Blocker(d, s) => self.m.blockers.get(d).and_then(|b| b.get(s)).and_then(|b| BLOCKER_KINDS.get(b.kind)).and_then(|k| shop_item_for_blocker(k)).map(Locate::Shop),
            _ => None,
        };
        self.locating = match (item, self.locating) {
            (Some(i), Some((j, since))) if i == j => Some((j, since)),
            (Some(i), _) => Some((i, self.now)),
            (None, _) => None,
        };
        if self.locating.is_some_and(|(_, since)| self.now - since < SHOP_HIGHLIGHT_SECS) {
            ui.ctx().request_repaint();
        }
        self.hits.clear();
        let p = ui.painter().clone();
        let size = self.size();
        self.fill(&p, rect(0.0, 0.0, size.x, size.y), Color32::BLACK);
        self.draw_main_tracker(&p);
        self.draw_item_grid(&p);
        self.draw_timer_area(&p);
        self.draw_overworld(&p);
        self.draw_recorder_destinations(&p);
        self.draw_legend(&p);
        self.draw_item_progress(&p);
        self.shift = self.lower_shift();
        self.draw_dungeon(&p);
        self.draw_magnifier(&p);
        self.draw_blockers(&p);
        self.draw_timeline(&p);
        self.shift = Vec2::ZERO;
        if self.wide {
            self.fill(&p, rect(W, 0.0, DIVIDER_W, WIDE_H), DIVIDER_COLOR);
        }
        if self.now < self.disk_until {
            let r = rect(DISK_ICON_X, DISK_ICON_Y, 30.0, 30.0);
            if !self.assets.icon10(&p, 8, self.r(r), 0.7) {
                self.text_c(&p, r, 11.0, Color32::LIGHT_GRAY, "save");
            }
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(DISK_FLASH_MS));
        }
        if !self.m.started {
            self.draw_start_choices(&p);
        }
        // pop-ups, hover text and messages go above everything (including the notes box)
        let top = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::unique("z1r-top")));
        self.draw_hover_info(&top);
        if self.hover == Target::Button("SpotSummary") && self.m.quest != 4 {
            let saved = self.shift;
            self.shift = Vec2::ZERO;
            self.draw_spot_summary(&top);
            self.shift = saved;
        }
        self.draw_popup(&top);
        if !self.toast.is_empty() && self.now < self.toast_until {
            let sz = Self::measure(&top, 14.0, &self.toast);
            let r = rect(W / 2.0 - sz.x / 2.0 - 10.0, MAP_Y + 4.0, sz.x + 20.0, sz.y + 10.0);
            self.fill_round(&top, r, Color32::from_rgba_unmultiplied(0, 60, 0, 235), 4);
            self.stroke_round(&top, r, 1.0, LIME, 4);
            let t = self.toast.clone();
            self.text(&top, r.min.x + 10.0, r.min.y + 5.0, 14.0, Color32::WHITE, &t);
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
        }
    }

    // ------------------------------------------------------------------ top-left: dungeons
    fn draw_main_tracker(&mut self, p: &Painter) {
        for d in 0..9 {
            let x = d as f32 * CELL;
            // row 0: dungeon number, white once its entrance is marked on the map
            let found = self.m.dungeon_located(d);
            let r0 = rect(x, 0.0, CELL, CELL);
            if !self.assets.dungeon_num(p, d, found, self.r(rect(x + 1.5, 1.5, 27.0, 27.0))) {
                let c = if found { Color32::WHITE } else { Color32::GRAY };
                self.stroke(p, r0.shrink(2.0), 2.0, c);
                self.text_c(p, r0, 16.0, c, &(d + 1).to_string());
            }
            let hint = self.m.location_hints[d];
            if !found && hint > 0 {
                self.fill(p, rect(x + 3.0, 19.0, 24.0, 10.0), Color32::BLACK);
                self.text_c(p, rect(x, 19.0, CELL, 11.0), 9.0, Color32::YELLOW, HINT_ZONE_SHORT[hint]);
            }
            if self.hover == Target::DungeonLabel(d) {
                self.stroke(p, r0, 2.0, Color32::YELLOW);
            }
            self.hit(r0, Target::DungeonLabel(d));

            // row 1: triforce (level 9 shows Ganon / Zelda instead)
            let r1 = rect(x, CELL, CELL, CELL);
            if d < 8 {
                let has = self.m.dungeons[d].triforce;
                let dst = self.r(rect(x + 1.0, CELL + 1.0, 28.0, 28.0));
                let ok = if has {
                    self.assets.icon10(p, 1, dst, 1.0)
                } else if found {
                    self.assets.icon10(p, 1, dst, 0.45)
                } else {
                    self.assets.icon10(p, 0, dst, 1.0)
                };
                if !ok {
                    let c = if has { ORANGE } else if found { Color32::from_rgb(139, 69, 19) } else { Color32::DARK_GRAY };
                    let rr = self.r(r1);
                    p.add(egui::Shape::convex_polygon(
                        vec![Pos2::new(rr.center().x, rr.min.y + 4.0), Pos2::new(rr.max.x - 4.0, rr.max.y - 4.0), Pos2::new(rr.min.x + 4.0, rr.max.y - 4.0)],
                        c,
                        Stroke::NONE,
                    ));
                }
            } else {
                let z = self.m.prog("PlayerHasRescuedZelda");
                let g = self.m.prog("PlayerHasDefeatedGanon");
                let dst = self.r(rect(x + 4.5, CELL + 2.0, 21.0, 21.0));
                if !self.assets.item7(p, if z { 24 } else { 23 }, dst, if z || g { 1.0 } else { 0.45 }) {
                    self.text_c(p, rect(x, CELL, CELL, 20.0), 11.0, Color32::WHITE, if z { "Z" } else { "G" });
                }
                let tc = format!("{}/8", self.m.triforce_count());
                self.text_c(p, rect(x, CELL + 19.0, CELL, 11.0), 9.0, ORANGE, &tc);
            }
            if self.hover == Target::Triforce(d) {
                self.stroke(p, r1, 2.0, Color32::YELLOW);
            }
            self.hit(r1, Target::Triforce(d));

            // rows 2-4: item boxes
            let ids = self.m.box_ids_of(d);
            for b in 0..3 {
                let rb = rect(x, CELL * (2 + b) as f32, CELL, CELL);
                if b < ids.len() {
                    self.draw_box(p, rb, ids[b]);
                    self.hit(rb, Target::Box(ids[b]));
                } else if d == 8 && b == 2 {
                    self.fill(p, rb.shrink(1.0), Color32::from_rgb(55, 55, 85));
                    let which = if self.m.second_quest_dungeons { "4" } else { "1" };
                    self.text_c(p, rect(rb.min.x, rb.min.y + 1.0, CELL, 16.0), 13.0, Color32::WHITE, which);
                    self.text_c(p, rect(rb.min.x, rb.min.y + 16.0, CELL, 12.0), 9.0, Color32::LIGHT_GRAY, "3rd");
                    if self.hover == Target::Toggle14 {
                        self.stroke(p, rb, 2.0, Color32::YELLOW);
                    }
                    self.hit(rb, Target::Toggle14);
                }
            }
        }
        self.line(p, Pos2::new(270.5, 0.0), Pos2::new(270.5, 150.0), 1.0, Color32::GRAY);
    }

    fn draw_box(&mut self, p: &Painter, cell: Rect, id: BoxId) {
        // 30x30 box with a 3-pixel border: red = not had yet, green = have it, purple = skipped
        let r = cell;
        let b = self.m.bx(id).clone();
        let c = match b.has {
            Has::Yes => BOX_YES,
            Has::Skipped => BOX_SKIPPED,
            Has::No => BOX_NO,
        };
        self.fill(p, r, Color32::BLACK);
        self.stroke(p, r, 3.0, c);
        if b.cell >= 0 && !self.assets.item7(p, ITEM_ICON7[b.cell as usize], self.r(Self::centered(r, 21.0, 21.0)), 1.0) {
            self.text_c(p, r, 11.0, Color32::WHITE, &ITEM_NAMES[b.cell as usize][..2]);
        }
        if self.hover == Target::Box(id) {
            self.stroke(p, cell, 2.0, Color32::YELLOW);
        }
    }

    // ------------------------------------------------------------------ item grid (swords, ladder, armos...)
    fn grid_rect(c: usize, r: usize) -> Rect {
        rect(ITEM_GRID_X + c as f32 * CELL, ITEM_GRID_Y + r as f32 * CELL, CELL, CELL)
    }

    fn label_icon(&mut self, p: &Painter, c: usize, r: usize, icon: i32) {
        let rr = Self::grid_rect(c, r);
        self.assets.item7(p, icon, self.r(Self::centered(rr, 14.0, 14.0)), 1.0);
    }

    fn toggle(&mut self, p: &Painter, c: usize, r: usize, icon: i32, key: &'static str) {
        let rr = Self::grid_rect(c, r);
        let has = self.m.prog(key);
        self.fill(p, rr.shrink(2.0), Color32::BLACK);
        self.stroke(p, rr.shrink(2.0), if has { 2.0 } else { 1.0 }, if has { LIME } else { Color32::GRAY });
        if !self.assets.item7(p, icon, self.r(Self::centered(rr, 21.0, 21.0)), if has { 1.0 } else { 0.35 }) {
            let short = &key[9..11];
            self.text_c(p, rr, 10.0, if has { Color32::WHITE } else { Color32::GRAY }, short);
        }
        if self.hover == Target::Progress(key) {
            self.stroke(p, rr, 2.0, Color32::YELLOW);
        }
        self.hit(rr, Target::Progress(key));
    }

    fn draw_item_grid(&mut self, p: &Painter) {
        self.label_icon(p, 0, 0, 17);
        self.box_at(p, 1, 0, BoxId::WhiteSword);
        self.toggle(p, 2, 0, 20, "PlayerHasMagicalSword");
        self.toggle(p, 3, 0, 19, "PlayerHasWoodSword");
        self.toggle(p, 4, 0, 22, "PlayerHasBlueRing");
        // the original's "boomerang or book" box, shown simply as the book (asked for); saved as PlayerHasBoomBook
        self.toggle(p, 5, 0, BOOK_ICON7, "PlayerHasBoomBook");
        self.label_icon(p, 0, 1, 4);
        self.box_at(p, 1, 1, BoxId::Ladder);
        self.toggle(p, 2, 1, 21, "PlayerHasBlueCandle");
        self.toggle(p, 3, 1, 11, "PlayerHasWoodArrow");
        self.toggle(p, 5, 1, 25, "PlayerHasBombs");
        self.label_icon(p, 0, 2, 18);
        self.box_at(p, 1, 2, BoxId::Armos);
        self.toggle(p, 2, 2, 23, "PlayerHasDefeatedGanon");
        self.toggle(p, 3, 2, 24, "PlayerHasRescuedZelda");
        // starting items / extra drops
        let ex = Self::grid_rect(4, 2);
        self.fill(p, ex.shrink(2.0), Color32::BLACK);
        self.stroke(p, ex.shrink(2.0), 1.0, if self.hover == Target::StartingItems { Color32::YELLOW } else { Color32::GRAY });
        if !self.assets.icon10(p, 7, self.r(Self::centered(ex, 20.0, 20.0)), 1.0) {
            self.text_c(p, ex, 16.0, Color32::WHITE, "+");
        }
        self.hit(ex, Target::StartingItems);
        // take-any hearts
        for i in 0..4 {
            let rr = Self::grid_rect(i, 3);
            let st = self.m.take_any_hearts[i];
            let icon = match st {
                1 => 4,
                2 => 2,
                _ => 3,
            };
            if !self.assets.icon10(p, icon, self.r(Self::centered(rr, 20.0, 20.0)), 1.0) {
                self.text_c(p, rr, 13.0, Color32::RED, match st {
                    1 => "<3",
                    2 => "x",
                    _ => "o",
                });
            }
            if self.hover == Target::TakeAny(i) {
                self.stroke(p, rr, 2.0, Color32::YELLOW);
            }
            self.hit(rr, Target::TakeAny(i));
        }
    }

    fn box_at(&mut self, p: &Painter, c: usize, r: usize, id: BoxId) {
        let rr = Self::grid_rect(c, r);
        self.draw_box(p, rr, id);
        self.hit(rr, Target::Box(id));
    }

    // ------------------------------------------------------------------ top right: counters, timer, buttons
    fn draw_timer_area(&mut self, p: &Painter) {
        let rem = format!("{} OW spots left", self.m.spots_remaining());
        self.text(p, 468.0, 92.0, 12.0, Color32::WHITE, &rem);
        if self.settings.get("Gettables") {
            let g = format!("{} gettable", self.m.gettable_remaining());
            self.text(p, 468.0, 110.0, 12.0, Color32::LIGHT_GRAY, &g);
        }
        if !self.assets.heart8(p, self.r(rect(468.0, 131.0, 16.0, 16.0))) {
            self.text(p, 468.0, 130.0, 12.0, Color32::RED, "<3");
        }
        let mh = format!("x {} max hearts", self.m.max_hearts());
        self.text(p, 488.0, 130.0, 12.0, Color32::WHITE, &mh);
        self.hit(rect(466.0, 128.0, 104.0, 20.0), Target::HeartsAdjust);

        if !self.settings.get("HideTimer") {
            let s = self.m.seconds;
            let txt = timer_text(s);
            self.text(p, TIMER_X, 0.0, 42.0, if self.paused { Color32::GRAY } else { LIGHT_GREEN }, &txt);
        }
        if self.paused {
            self.text(p, TIMER_X, 50.0, 10.0, ORANGE, "paused - click to resume");
        }
        self.hit(rect(TIMER_X, 2.0, 148.0, 56.0), Target::Timer);
        // three columns: Options, Hints, Reset / Save, Load, Stream (the stream button is the rebuild's own: it opens
        // the OBS streaming window, and a red dot shows while it streams)
        let names: [(&'static str, &'static str); 6] = [("Options", "Options"), ("Hints", "Hints"), ("Reset...", "Reset"), ("Save", "Save"), ("Load", "Load"), ("Stream", "Stream")];
        for (i, (label, n)) in names.iter().enumerate() {
            let r = rect(TIMER_X + (i % 3) as f32 * 65.0, 66.0 + (i / 3) as f32 * 28.0, 62.0, 24.0);
            self.button(p, r, label, Target::Button(n));
            if *n == "Stream" && self.settings.get("StreamToOBS") {
                self.fill_round(p, rect(r.max.x - 9.0, r.min.y + 3.0, 6.0, 6.0), Color32::from_rgb(230, 40, 40), 3);
            }
        }
        let tf = format!("Triforces: {}/8", self.m.triforce_count());
        self.text(p, TIMER_X, 124.0, 12.0, ORANGE, &tf);
        // the original's "Spot Summary" button (OverworldItemGridUI.fs; shown only for a standard overworld)
        if self.m.quest != 4 {
            self.button(p, rect(TIMER_X + 96.0, 121.0, 92.0, 22.0), "Spot Summary", Target::Button("SpotSummary"));
        }
    }

    fn button(&mut self, p: &Painter, r: Rect, label: &str, t: Target) {
        let hov = self.hover == t;
        self.fill_round(p, r, if hov { Color32::from_rgb(60, 60, 70) } else { Color32::from_rgb(36, 36, 44) }, 3);
        self.stroke_round(p, r, 1.0, if hov { Color32::YELLOW } else { Color32::GRAY }, 3);
        self.text_c(p, r, 12.0, Color32::WHITE, label);
        self.hit(r, t);
    }

    // ------------------------------------------------------------------ overworld map
    pub fn data_x(&self, col: usize) -> usize {
        if self.m.mirror {
            15 - col
        } else {
            col
        }
    }

    fn draw_overworld(&mut self, p: &Painter) {
        let zones = self.settings.get("Zones");
        let coords = self.settings.get("Coords");
        for col in 0..16 {
            for y in 0..8 {
                let x = self.data_x(col);
                let r = rect(col as f32 * TILE_W, MAP_Y + y as f32 * TILE_H, TILE_W, TILE_H);
                let mark = self.m.mark[x][y];
                let shade = if tile_is_shaded(mark, self.m.always_empty(x, y)) { 1.0 - DONT_CARE_OPACITY } else { 1.0 };
                // the blank map shows the person's own overworld picture, if one is chosen (the original's
                // overworldMapBMPs uses alternativeOverworldMapFilename only for quest 4)
                let drawn = if self.m.quest == 4 {
                    self.assets.follow_custom_map(&self.m.alt_map_file);
                    // without a picture of the person's own, the original's blank map (plain sand)
                    self.assets.custom_map_screen(p, x, y, self.r(r), shade, self.m.mirror)
                        || self.assets.map_screen(p, 4, x, y, self.r(r), shade, self.m.mirror)
                } else {
                    self.assets.map_screen(p, self.m.quest, x, y, self.r(r), shade, self.m.mirror)
                };
                if !drawn {
                    let v = |c: f32| (c * shade) as u8;
                    self.fill(p, r, Color32::from_rgb(v(40.0), v(90.0), v(40.0)));
                    self.stroke(p, r, 1.0, Color32::BLACK);
                }
                // a hidden kind of tile looks like "don't care": darkened, without its marker
                let settings = &self.settings;
                let located = self.locating.is_some_and(|(what, _)| self.locates(what, x, y));
                let hidden = self.m.tile_hidden(x, y, &|k| settings.get(k)) && !located;
                if mark >= 0 && mark != DARK_X && !hidden {
                    let shop_items = self.m.shop_items_shown(x, y, &|k| settings.get(k));
                    self.draw_mark(p, r, mark, self.m.extra[x][y], shop_items);
                }
                // circle value: last two digits 0 = no circle, 1 = plain circle, else the label's character; hundreds = colour
                let circle = self.m.circle[x][y];
                if circle % 100 != 0 {
                    let rr = self.r(r);
                    let color = CIRCLE_COLORS[((circle / 100).clamp(0, 2)) as usize];
                    // a 31x31 circle, centered across the tile and touching its top (activateCircleLabelPopup$cont@426)
                    let center = Pos2::new(rr.center().x, rr.min.y + CIRCLE_SIZE / 2.0);
                    p.circle_stroke(center, (CIRCLE_SIZE - CIRCLE_LINE) / 2.0, Stroke::new(CIRCLE_LINE, color));
                    // the label: '0'-'9' or 'A'-'Z', bold, on black, in the tile's bottom-left corner
                    let label = circle % 100;
                    if (48..=57).contains(&label) || (65..=90).contains(&label) {
                        let ch = (label as u8 as char).to_string();
                        let galley = p.layout_no_wrap(ch, FontId::proportional(CIRCLE_LABEL_SIZE), color);
                        let sz = galley.size();
                        let lr = Rect::from_min_size(Pos2::new(rr.min.x, rr.max.y - sz.y), sz + Vec2::new(2.0, 0.0));
                        p.rect_filled(lr, 0.0, Color32::BLACK);
                        p.galley(lr.min + Vec2::new(1.0, 0.0), galley, color);
                        // bold: draw a second time one pixel to the right
                        let galley2 = p.layout_no_wrap((label as u8 as char).to_string(), FontId::proportional(CIRCLE_LABEL_SIZE), color);
                        p.galley(lr.min + Vec2::new(2.0, 0.0), galley2, color);
                    }
                }
                if zones {
                    let z = ZONES[y].as_bytes()[x] as char;
                    self.text(p, r.min.x + 2.0, r.min.y + 1.0, 10.0, Color32::WHITE, &z.to_string());
                }
                if coords {
                    let c = format!("{}{}", (b'A' + y as u8) as char, x + 1);
                    self.text(p, r.max.x - 18.0, r.max.y - 13.0, 9.0, Color32::WHITE, &c);
                }
                self.hit(r, Target::MapTile(x, y));
                // F16 holds the coast item box (WPFUI/makeAll@514-73). The original hides it once ladderBox.IsDone
                // (WPFUI/makeAll@523-74); on request it stays, showing the chosen item and its had/skipped border.
                // The original puts it at (+17,+1); here it is centered like the other marks.
                if is_coast(x, y) {
                    let b = centered_in(r, CELL, CELL);
                    self.draw_box(p, b, BoxId::Ladder);
                }
            }
        }
        // the original's shop locator (WPFUI.fs showShopLocatorInstanceFunc): pointing at the wood arrow, bombs,
        // boomerang-or-book, blue candle (while no candle is had) or blue ring in the top section, or at a dungeon
        // blocker needing meat / keys / bombs / bow and arrow, outlines every shop selling it
        // (TileHighlightRectangle.MakeGreenWithBriefAnimation: 12 -> 4 px and cyan -> lime over 0.5 s, or plain
        // green 4 px without "AnimateShopHighlights"); if none is marked, says so
        if let Some((what, since)) = self.locating {
            let t = ((self.now - since) / SHOP_HIGHLIGHT_SECS).clamp(0.0, 1.0) as f32;
            let (w, c) = if self.settings.get("AnimateShopHighlights") {
                let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
                (12.0 + (SHOP_HIGHLIGHT_W - 12.0) * t, Color32::from_rgb(0, lerp(255, 255), lerp(255, 0)))
            } else {
                (SHOP_HIGHLIGHT_W, SHOP_HIGHLIGHT_PLAIN)
            };
            let mut any = false;
            for col in 0..16 {
                for y in 0..8 {
                    let x = self.data_x(col);
                    if self.locates(what, x, y) {
                        any = true;
                        let r = rect(col as f32 * TILE_W, MAP_Y + y as f32 * TILE_H, TILE_W, TILE_H);
                        self.stroke(p, r.shrink(w.min(16.0) / 2.0), w, c);
                    }
                }
            }
            // a sword cave not marked yet but hinted: the original's showLocatorHintedZone darkens every screen
            // outside the hinted zone (black, opacity 0.7)
            let hint = match what {
                Locate::Cave(m) => sword_hint_index(m).map(|i| self.m.location_hints[i]).filter(|&z| z != 0),
                Locate::Shop(_) => None,
            };
            if !any && hint.is_some() {
                let zone = hint.unwrap_or(0);
                for col in 0..16 {
                    for y in 0..8 {
                        if zone_at(self.data_x(col), y) != zone {
                            self.fill(p, rect(col as f32 * TILE_W, MAP_Y + y as f32 * TILE_H, TILE_W, TILE_H), Color32::from_black_alpha(HINTED_ZONE_DARKEN));
                        }
                    }
                }
            } else if !any {
                let what_name = match what {
                    Locate::Shop(item) => format!("no {} shop marked yet", SHOP_ITEM_NAMES[item as usize].to_lowercase()),
                    Locate::Cave(m) => format!("no {} marked yet", sword_cave_name(m)),
                };
                let w = Self::measure(p, 12.0, &what_name).x + 16.0;
                let r = rect(16.0 * TILE_W - w - 4.0, MAP_Y + 4.0, w, 20.0);
                self.fill(p, r, Color32::from_black_alpha(200));
                self.text_c(p, r, 12.0, Color32::YELLOW, &what_name);
            }
        }
        if let Target::MapTile(hx, hy) = self.hover {
            let hcol = (if self.m.mirror { 15 - hx } else { hx }) as i32;
            if self.settings.get("HighlightNearby") {
                for dx in -1..=1i32 {
                    for dy in -1..=1i32 {
                        let (c, yy) = (hcol + dx, hy as i32 + dy);
                        if (dx == 0 && dy == 0) || !(0..16).contains(&c) || !(0..8).contains(&yy) {
                            continue;
                        }
                        let x = self.data_x(c as usize);
                        if self.m.mark[x][yy as usize] == -1 && !self.m.always_empty(x, yy as usize) {
                            let hr = rect(c as f32 * TILE_W + 2.0, MAP_Y + yy as f32 * TILE_H + 2.0, TILE_W - 4.0, TILE_H - 4.0);
                            self.stroke(p, hr, 1.0, Color32::from_rgb(0, 255, 255));
                        }
                    }
                }
            }
            self.stroke(p, rect(hcol as f32 * TILE_W, MAP_Y + hy as f32 * TILE_H, TILE_W, TILE_H), 2.0, Color32::YELLOW);
        }
        // hovering a dungeon number or triforce whose location is hinted outlines that zone
        if let Target::DungeonLabel(d) | Target::Triforce(d) = self.hover {
            if d < 9 && self.m.location_hints[d] > 0 && !self.m.dungeon_located(d) {
                let zone = self.m.location_hints[d];
                for col in 0..16 {
                    for y in 0..8 {
                        if zone_at(self.data_x(col), y) == zone {
                            let zr = rect(col as f32 * TILE_W, MAP_Y + y as f32 * TILE_H, TILE_W, TILE_H);
                            self.fill(p, zr, Color32::from_rgba_unmultiplied(255, 255, 0, 60));
                        }
                    }
                }
            }
        }
    }

    /// `shop_items`: for a shop, the items it shows (see Tracker::shop_items_shown).
    fn draw_mark(&mut self, p: &Painter, tile: Rect, mark: i32, extra: i32, shop_items: Vec<i32>) {
        if (SHOP_FIRST..=SHOP_LAST).contains(&mark) && self.draw_shop(p, tile, &shop_items) {
            return;
        }
        // the mark is a small badge centered on the tile, so the map stays visible around it
        let strip = TILE_STRIP_INDEX[mark as usize];
        let r = centered_in(tile, MARK_BADGE_W, MARK_BADGE_H);
        self.stroke(p, r.expand(1.0), 1.0, Color32::BLACK);
        if !self.assets.ow_icon(p, strip, self.r(r)) {
            let fill = match mark {
                0..=8 => Color32::from_rgb(70, 130, 180),
                9..=12 => Color32::from_rgb(221, 160, 221),
                13..=15 => INDIAN_RED,
                16..=23 => Color32::from_rgb(255, 140, 0),
                24..=27 => Color32::from_rgb(154, 205, 50),
                36 => Color32::from_rgb(147, 112, 219),
                37 => Color32::from_rgb(0, 128, 128),
                _ => Color32::from_rgb(244, 164, 96),
            };
            self.fill(p, r, fill);
            let s = match mark {
                0..=8 => format!("L{}", mark + 1),
                9..=12 => format!("R{}", mark - 8),
                13 => "S3".into(),
                14 => "S2".into(),
                15 => "S1".into(),
                16..=23 => SHOP_ITEM_NAMES[(mark - 16) as usize][..1].to_string(),
                24 => "?".into(),
                25 => "+L".into(),
                26 => "+M".into(),
                27 => "+S".into(),
                28 => "-".into(),
                29 => "+-".into(),
                30 => "LT".into(),
                31 => "AR".into(),
                32 => "??".into(),
                33 => "TA".into(),
                34 => "PO".into(),
                36 => START_MARK_TEXT.into(),
                37 => "P2".into(),
                _ => String::new(),
            };
            // longer words get a smaller font so they fit the 32 x 22 badge
            let size = if s.chars().count() > 3 { 10.0 } else { 12.0 };
            self.text_c(p, r, size, Color32::BLACK, &s);
        }
        // a shop can show a second item, in the tile's bottom-right corner
        let shop = (SHOP_FIRST..=SHOP_LAST).contains(&mark);
        if (shop || self.settings.get("AllowItemIconOnNonShopTile")) && (1..=8).contains(&extra) {
            let b = rect(tile.max.x - 17.0, tile.max.y - 17.0, 16.0, 16.0);
            self.fill(p, b, Color32::BLACK);
            self.stroke(p, b, 1.0, Color32::WHITE);
            if !self.assets.item7(p, SHOP_ITEM_ICON7[(extra - 1) as usize], self.r(b.shrink(1.0)), 1.0) {
                self.text_c(p, b, 10.0, Color32::WHITE, &SHOP_ITEM_NAMES[(extra - 1) as usize][..1]);
            }
        }
    }

    /// A shop on the map, drawn as the original does: no letters, just the item pictures on an orange box,
    /// the rest of the tile left see-through.
    /// One item: a 15x27 box with the item 3 px in (Graphics .cctor, theInteriorBmpTable). Two items: a 27x27 box with
    /// the items 3 px and 15 px in (OverworldMapTileCustomization::makeTwoItemShopBmp$cont@133).
    /// On request the box is centered on the tile (the original puts it at (3,3) / (9,3)).
    fn draw_shop(&mut self, p: &Painter, tile: Rect, items: &[i32]) -> bool {
        let (box_r, spots) = match *items {
            [] => return false,
            [one] => (centered_in(tile, 15.0, 27.0), vec![(one, 3.0)]),
            [a, b, ..] => (centered_in(tile, 27.0, 27.0), vec![(a, 3.0), (b, 15.0)]),
        };
        self.stroke(p, box_r.expand(1.0), 1.0, Color32::BLACK);
        self.fill(p, box_r, SHOP_ITEM_BACKGROUND);
        for (item, dx) in spots {
            let item_r = rect(box_r.min.x + dx, box_r.min.y + 3.0, 9.0, 21.0);
            if !self.assets.shop_item(p, item, self.r(item_r)) {
                // no pictures: the item's first letter
                self.text_c(p, item_r, 11.0, Color32::BLACK, &SHOP_ITEM_NAMES[item as usize][..1]);
            }
        }
        true
    }

    // ------------------------------------------------------------------ legend and item progress
    fn draw_legend(&mut self, p: &Painter) {
        // one row, ending with the recorder destination and the original's "..." settings button beside it
        let legend: [(i32, &str); 7] = [(2, "Dungeon"), (11, "Any Road"), (21, "Sword Cave"), (31, "Shop"), (35, "Secret"), (22, "Hint/Other"), (38, "Don't care")];
        let (icon_w, icon_h, size) = (22.0, 15.0, 9.0);
        let mut x = 2.0;
        for (strip, label) in legend {
            let dst = rect(x, LEGEND_Y + 7.0, icon_w, icon_h);
            if !self.assets.draw(p, "s_icon_overworld_strip39.png", strip as f32 * 16.0, 0.0, 16.0, 11.0, self.r(dst), 1.0, false) {
                self.fill(p, dst, Color32::GRAY);
            }
            self.text(p, x + icon_w + 3.0, LEGEND_Y + 9.0, size, Color32::WHITE, label);
            x += icon_w + 3.0 + Self::measure(p, size, label).x + 7.0;
        }
        let r = self.r(rect(x, LEGEND_Y + 7.0, icon_w, icon_h));
        let c = r.center();
        let col = recorder_color(self.m.recorder_to_new);
        p.add(egui::Shape::closed_line(vec![c + Vec2::new(0.0, -7.0), c + Vec2::new(7.0, 0.0), c + Vec2::new(0.0, 7.0), c + Vec2::new(-7.0, 0.0)], Stroke::new(2.0, col)));
        let label = "Recorder";
        self.text(p, x + icon_w + 3.0, LEGEND_Y + 9.0, size, Color32::WHITE, label);
        let bx = x + icon_w + 3.0 + Self::measure(p, size, label).x + 4.0;
        self.button(p, rect(bx, LEGEND_Y + 5.0, 22.0, 19.0), "...", Target::Button("RecorderSettings"));
        self.button(p, rect(590.0, LEGEND_Y + 2.0, 96.0, 20.0), "HotKeys", Target::Button("HotKeys"));
        self.button(p, rect(590.0, LEGEND_Y + 24.0, 96.0, 20.0), "Show/Run", Target::Button("ShowRun"));
        if !self.assets.logo(p, self.r(rect(W - 63.0 - 12.0, LEGEND_Y, 60.0, 60.0))) {
            self.text(p, W - 66.0, LEGEND_Y + 14.0, 26.0, Color32::RED, "ZT");
        }
    }

    fn draw_item_progress(&mut self, p: &Painter) {
        // the original's version button (CustomComboBoxes.makeVersionButtonWithBehavior): "v2.1.0" in orange at size 12;
        // clicking it opens the About box
        let vr = rect(2.0, PROGRESS_Y + 4.0, 58.0, 20.0);
        let hov = self.hover == Target::Button("About");
        self.fill_round(p, vr, if hov { Color32::from_rgb(60, 60, 70) } else { Color32::from_rgb(36, 36, 44) }, 3);
        self.stroke_round(p, vr, 1.0, if hov { Color32::YELLOW } else { Color32::GRAY }, 3);
        self.text_c(p, vr, 12.0, ORANGE, &format!("v{VERSION}"));
        self.hit(vr, Target::Button("About"));
        self.text(p, 70.0, PROGRESS_Y + 8.0, 12.0, Color32::LIGHT_GRAY, "Items:");
        let m = &self.m;
        // a negative number means "not had yet": draw icon (-v - 1) dimmed
        let pick = |have: bool, icon: i32| if have { icon } else { -icon - 1 };
        let sword = if m.prog("PlayerHasMagicalSword") {
            20
        } else if m.player_has_item(13) {
            17
        } else {
            pick(m.prog("PlayerHasWoodSword"), 19)
        };
        let ring = if m.player_has_item(10) { 12 } else { pick(m.prog("PlayerHasBlueRing"), 22) };
        let candle = if m.player_has_item(9) { 7 } else { pick(m.prog("PlayerHasBlueCandle"), 21) };
        let arrow = if m.player_has_item(11) { 10 } else { pick(m.prog("PlayerHasWoodArrow"), 11) };
        let boom = if m.player_has_item(5) { 2 } else { pick(m.player_has_item(1), 0) };
        let list = [
            sword, ring, candle, arrow,
            pick(m.player_has_item(2), 1), boom, pick(m.player_has_item(12), 6), pick(m.player_has_item(0), 8),
            pick(m.player_has_item(7), 3), pick(m.player_has_item(4), 4), pick(m.player_has_item(8), 5),
            pick(m.player_has_item(3), 16), pick(m.player_has_item(6), 9), pick(m.prog("PlayerHasBombs"), 25),
        ];
        let mut x = FIRST_PROGRESS_X;
        for v in list {
            let (icon, have) = if v >= 0 { (v, true) } else { (-v - 1, false) };
            // not had yet: the original's grey picture (UIComponents::MakeItemProgressBar uses Graphics::greyscale)
            let dst = self.r(rect(x + 4.0, PROGRESS_Y + 4.0, 21.0, 21.0));
            if have {
                self.assets.item7(p, icon, dst, 1.0);
            } else {
                self.assets.item7_grey(p, icon, dst);
            }
            x += CELL;
        }
    }

    // ------------------------------------------------------------------ dungeon maps
    pub fn room_rect(x: usize, y: usize) -> Rect {
        rect(ROOM_X0 + x as f32 * ROOM_PITCH_X, DUNGEON_Y + TAB_H + DUNGEON_HEADER_H + 3.0 + y as f32 * ROOM_PITCH_Y, ROOM_W, ROOM_H)
    }

    fn draw_dungeon(&mut self, p: &Painter) {
        let sel = self.m.tab.min(8);
        let tab_w = 408.0 / 9.0;
        for i in 0..9 {
            let r = rect(i as f32 * tab_w, DUNGEON_Y, tab_w - 2.0, TAB_H);
            let on = i == sel;
            let done = if i < 8 { self.m.dungeons[i].triforce } else { self.m.prog("PlayerHasRescuedZelda") };
            self.fill_round(p, r, if on { Color32::from_rgb(70, 70, 110) } else { PANEL }, 3);
            self.stroke_round(p, r, 1.0, if on { Color32::WHITE } else { Color32::GRAY }, 3);
            self.text_c(p, r, 13.0, if done { ORANGE } else { Color32::WHITE }, &(i + 1).to_string());
            self.hit(r, Target::Tab(i));
        }
        let word = if self.settings.get("BOARDInsteadOfLEVEL") { "BOARD" } else { "LEVEL" };
        self.text(p, 8.0, DUNGEON_Y + TAB_H + 4.0, 18.0, Color32::WHITE, &format!("{word}-{}", sel + 1));
        let (info, size, color) = if let Target::Room(x, y) = self.hover {
            let room = &self.m.maps[sel].rooms[x][y];
            let mut d = ROOM_TYPE_DESCRIPTIONS[room.kind].to_string();
            if room.monster > 0 {
                d += &format!(" / {}", MONSTER_NAMES[room.monster]);
            }
            if room.floor_drop > 0 {
                d += &format!(" / {}", FLOOR_DROP_NAMES[room.floor_drop]);
            }
            (d, 12.0, Color32::LIGHT_GRAY)
        } else {
            ("L-click: mark/complete   R-click: room type   wheel: monster/drop".to_string(), 10.0, Color32::GRAY)
        };
        self.text(p, 110.0, DUNGEON_Y + TAB_H + 8.0, size, color, &info);

        for x in 0..7 {
            for y in 0..8 {
                let a = Self::room_rect(x, y);
                let r = rect(a.max.x, a.min.y + 8.0, ROOM_PITCH_X - ROOM_W, 11.0);
                let st = self.m.maps[sel].hdoors[x][y];
                self.draw_door(p, r, st, self.hover == Target::HDoor(x, y));
                self.hit(r, Target::HDoor(x, y));
            }
        }
        for x in 0..8 {
            for y in 0..7 {
                let a = Self::room_rect(x, y);
                let r = rect(a.min.x + 12.0, a.max.y, 15.0, ROOM_PITCH_Y - ROOM_H);
                let st = self.m.maps[sel].vdoors[x][y];
                self.draw_door(p, r, st, self.hover == Target::VDoor(x, y));
                self.hit(r, Target::VDoor(x, y));
            }
        }
        for x in 0..8 {
            for y in 0..8 {
                let r = Self::room_rect(x, y);
                let room = self.m.maps[sel].rooms[x][y].clone();
                let gz = room.kind == 32 || room.kind == 33;
                if room.kind == ROOM_UNMARKED {
                    self.fill(p, r, Color32::from_rgb(16, 16, 40));
                    self.stroke(p, r, 1.0, Color32::from_rgb(40, 40, 70));
                } else if !self.assets.room(p, ROOM_ICON[room.kind], room.completed || gz, self.r(r)) {
                    self.fill(p, r, if room.completed { Color32::DARK_GRAY } else { Color32::LIGHT_GRAY });
                    let n = ROOM_TYPE_NAMES[room.kind];
                    self.text_c(p, r, 10.0, Color32::BLACK, &n[..n.len().min(3)]);
                }
                // as the original (DungeonRoomState.CurrentDisplayEx$cont@191): the monster's 18x18 picture hangs off the
                // room's top-left corner, darkened once the room is done unless it is a bubble, traps or "other"; the
                // floor drop sits at the bottom right, darkened while it hasn't been marked bright
                if room.monster > 0 {
                    let shade = if room.completed && !monster_stays_bright(room.monster) { 1.0 - DONE_DARKEN } else { 1.0 };
                    let at = rect(r.min.x + MONSTER_AT.0, r.min.y + MONSTER_AT.1, ROOM_DETAIL_SIZE, ROOM_DETAIL_SIZE);
                    if !self.assets.boss_shaded(p, MONSTER_ICON[room.monster], self.r(at), shade) {
                        self.text(p, at.min.x + 2.0, at.min.y + 2.0, 10.0, Color32::RED, &MONSTER_NAMES[room.monster][..2]);
                    }
                }
                if room.floor_drop > 0 {
                    let shade = if room.floor_drop_bright { 1.0 } else { 1.0 - DONE_DARKEN };
                    let at = rect(r.min.x + FLOOR_DROP_AT.0, r.min.y + FLOOR_DROP_AT.1, ROOM_DETAIL_SIZE, ROOM_DETAIL_SIZE);
                    if !self.assets.zitem16(p, FLOOR_DROP_ICON[room.floor_drop], self.r(at.shrink(1.0)), shade) {
                        self.text(p, at.min.x + 2.0, at.min.y + 2.0, 10.0, Color32::YELLOW, &FLOOR_DROP_NAMES[room.floor_drop][..1]);
                    }
                }
                if self.m.maps[sel].circled[x][y] {
                    let rr = self.r(r);
                    p.add(egui::Shape::ellipse_stroke(rr.center(), Vec2::new(ROOM_W / 2.0 + 3.0, ROOM_H / 2.0 + 4.0), Stroke::new(2.0, Color32::YELLOW)));
                }
                if self.hover == Target::Room(x, y) {
                    self.stroke(p, r.expand(1.0), 2.0, Color32::YELLOW);
                }
                self.hit(r, Target::Room(x, y));
            }
        }
        // Dungeon "sunglasses": a black box 3 px larger than the room map on every side, over it at opacity 0.15
        // (DungeonUI::doSunglasses@1961, token 0x06001be4, RVA 0x6183c; set up in makeDungeonTabs@1936-213)
        if self.settings.get("GiveDungeonTrackerSunglasses") {
            let area = Self::room_rect(0, 0).union(Self::room_rect(7, 7)).expand(SUNGLASSES_MARGIN);
            self.fill(p, area, Color32::from_black_alpha((SUNGLASSES_OPACITY * 255.0).round() as u8));
        }
    }

    /// The magnifier: while the mouse is over a map screen, that screen enlarged over the dungeon rooms, with the
    /// original's key to the coloured spots in the pictures (UIComponents::MakeMagnifier$cont@31: open cave = cyan
    /// square, bomb / burn / recorder / pushable spot = black square edged cyan / red / yellow / magenta).
    fn draw_magnifier(&mut self, p: &Painter) {
        if !self.settings.get("ShowMagnifier") {
            return;
        }
        let Target::MapTile(x, y) = self.hover else { return };
        let area = Self::room_rect(0, 0).union(Self::room_rect(7, 7)).expand(3.0);
        let key_h = 22.0;
        let avail = Rect::from_min_max(area.min + Vec2::splat(4.0), Pos2::new(area.max.x - 4.0, area.max.y - key_h));
        let s = (avail.width() / 16.0).min(avail.height() / 11.0);
        let pic = Rect::from_center_size(avail.center(), Vec2::new(16.0 * s, 11.0 * s));
        self.fill(p, area, Color32::BLACK);
        self.stroke(p, area, 1.0, Color32::GRAY);
        let mirror = self.m.mirror;
        // the enlarged pictures are of the first quest's map; other overworlds show their own screen enlarged
        let drawn = if self.m.quest == 4 {
            self.assets.custom_map_screen(p, x, y, self.r(pic), 1.0, mirror) || self.assets.map_screen(p, 4, x, y, self.r(pic), 1.0, mirror)
        } else if self.m.quest == 0 {
            self.assets.magnifier_screen(p, x, y, self.r(pic), mirror) || self.assets.map_screen(p, 0, x, y, self.r(pic), 1.0, mirror)
        } else {
            self.assets.map_screen(p, self.m.quest, x, y, self.r(pic), 1.0, mirror)
        };
        if !drawn {
            self.text_c(p, pic, 14.0, Color32::GRAY, "no picture of this screen");
        }
        self.stroke(p, pic.expand(1.0), 1.0, Color32::WHITE);
        let label = format!("{}{}", (b'A' + y as u8) as char, x + 1);
        self.text(p, pic.min.x + 4.0, pic.min.y + 3.0, 14.0, ORANGE, &label);
        let key: [(Color32, Color32, &str); 5] = [
            (Color32::from_rgb(0, 255, 255), Color32::BLACK, "open cave"),
            (Color32::BLACK, Color32::from_rgb(0, 255, 255), "bomb spot"),
            (Color32::BLACK, Color32::RED, "burn spot"),
            (Color32::BLACK, Color32::YELLOW, "recorder spot"),
            (Color32::BLACK, Color32::from_rgb(255, 0, 255), "pushable spot"),
        ];
        let step = area.width() / key.len() as f32;
        for (i, (fill, edge, text)) in key.iter().enumerate() {
            let x0 = area.min.x + 6.0 + i as f32 * step;
            let y0 = area.max.y - key_h + 6.0;
            let sq = rect(x0, y0, 8.0, 8.0);
            self.fill(p, sq, *fill);
            self.stroke(p, sq, 2.0, *edge);
            self.text(p, x0 + 12.0, y0 - 2.0, 10.0, Color32::LIGHT_GRAY, text);
        }
    }

    /// Where the recorder can take the player: a diamond (a 35 x 35 square turned 45 degrees, 3-px edge, over a
    /// blurred black one) centred on each destination screen (WPFUI.fs RoutingInfo, recorderingCanvas); with the
    /// vanilla setting each is numbered in white on black at the screen's bottom-left.
    fn draw_recorder_destinations(&mut self, p: &Painter) {
        let col = recorder_color(self.m.recorder_to_new);
        for ((x, y), t) in self.m.recorder_destinations() {
            let screen_col = if self.m.mirror { 15 - x } else { x };
            let tile = rect(screen_col as f32 * TILE_W, MAP_Y + y as f32 * TILE_H, TILE_W, TILE_H);
            // the original turns the square about its top-left corner, placed at (+23, -9) in the screen, so the
            // diamond's centre is at (23, -9 + 35/sqrt 2) = (23, 15.75)
            let h = RECORDER_DIAMOND / std::f32::consts::SQRT_2;
            let c = self.r(tile).min + Vec2::new(RECORDER_DIAMOND_X, RECORDER_DIAMOND_Y + h);
            let pts = vec![c + Vec2::new(0.0, -h), c + Vec2::new(h, 0.0), c + Vec2::new(0.0, h), c + Vec2::new(-h, 0.0)];
            p.add(egui::Shape::closed_line(pts.clone(), Stroke::new(7.0, Color32::from_black_alpha(110))));
            p.add(egui::Shape::closed_line(pts, Stroke::new(3.0, col)));
            if !self.m.recorder_to_new {
                let n = (t + 1).to_string();
                let lr = rect(tile.min.x + 2.0, tile.max.y - 17.0, 10.0, 15.0);
                self.fill(p, lr, Color32::BLACK);
                self.text(p, lr.min.x + 1.0, lr.min.y, 12.0, Color32::WHITE, &n);
            }
        }
    }

    /// Whether map screen (x, y) is one of the places being located.
    fn locates(&self, what: Locate, x: usize, y: usize) -> bool {
        match what {
            Locate::Shop(item) => self.m.tile_sells(x, y, item),
            Locate::Cave(mark) => self.m.mark[x][y] == mark,
        }
    }

    fn draw_door(&self, p: &Painter, r: Rect, state: i32, hovered: bool) {
        let c = DOOR_COLORS[(state.clamp(0, 4)) as usize];
        let inner = if r.width() > r.height() { r.shrink2(Vec2::new(1.0, 3.0)) } else { r.shrink2(Vec2::new(3.0, 1.0)) };
        self.fill(p, inner, c);
        if hovered {
            self.stroke(p, r, 1.0, Color32::YELLOW);
        }
    }

    // ------------------------------------------------------------------ blockers
    fn draw_blockers(&mut self, p: &Painter) {
        self.line(p, Pos2::new(BLOCKERS_X - 20.0, DUNGEON_Y), Pos2::new(BLOCKERS_X - 20.0, TIMELINE_Y - 4.0), 1.0, Color32::GRAY);
        for d in 0..9 {
            let (c, row) = (d % 3, d / 3);
            let x = BLOCKERS_X + c as f32 * 106.0;
            let y = DUNGEON_Y + row as f32 * BLOCKER_ROW_H;
            if d == 8 {
                self.text(p, x + 4.0, y + 9.0, 14.0, Color32::LIGHT_GRAY, "Blockers");
                continue;
            }
            self.text(p, x, y + 8.0, 14.0, if self.m.dungeons[d].triforce { ORANGE } else { Color32::WHITE }, &(d + 1).to_string());
            for s in 0..3 {
                let r = rect(x + 14.0 + s as f32 * 29.0, y + 5.0, 26.0, 26.0);
                let kind = self.m.blockers[d][s].kind;
                let maybe = blocker_is_maybe(kind);
                self.fill(p, r, if maybe { Color32::from_rgb(50, 50, 20) } else { Color32::BLACK });
                self.stroke(p, r, 1.0, if kind == BLOCKER_NOTHING { Color32::GRAY } else { Color32::WHITE });
                if kind != BLOCKER_NOTHING {
                    if !self.assets.item7(p, BLOCKER_ICON7[kind], self.r(Self::centered(r, 21.0, 21.0)), if maybe { 0.6 } else { 1.0 }) {
                        let n = BLOCKER_KINDS[kind].trim_start_matches("Blocker_").trim_start_matches("Maybe_");
                        self.text_c(p, r, 10.0, Color32::WHITE, &n[..2]);
                    }
                    if maybe {
                        self.text(p, r.max.x - 7.0, r.min.y - 1.0, 10.0, Color32::YELLOW, "?");
                    }
                }
                if self.hover == Target::Blocker(d, s) {
                    self.stroke(p, r, 2.0, Color32::YELLOW);
                }
                self.hit(r, Target::Blocker(d, s));
            }
        }
        self.text(p, BLOCKERS_X, DUNGEON_Y + 3.0 * BLOCKER_ROW_H + 2.0, 12.0, Color32::GRAY, "Notes");
    }

    // ------------------------------------------------------------------ timeline
    fn draw_timeline(&mut self, p: &Painter) {
        self.fill(p, rect(0.0, TIMELINE_Y, W, H - TIMELINE_Y), Color32::from_rgb(10, 10, 14));
        self.line(p, Pos2::new(0.0, TIMELINE_Y + 0.5), Pos2::new(W, TIMELINE_Y + 0.5), 1.0, Color32::GRAY);
        let (left, right, axis_y) = (14.0, W - 14.0, H - 24.0);
        let secs = self.m.seconds;
        let max_min = (((secs / 60 / 30) + 1) * 30).max(60);
        self.line(p, Pos2::new(left, axis_y), Pos2::new(right, axis_y), 1.0, Color32::WHITE);
        let step = if max_min > 120 { 15 } else { 5 };
        let major = if max_min > 120 { 60 } else { 30 };
        let mut mm = 0;
        while mm <= max_min {
            let x = left + (right - left) * mm as f32 / max_min as f32;
            let big = mm % major == 0;
            self.line(p, Pos2::new(x, axis_y), Pos2::new(x, axis_y + if big { 8.0 } else { 4.0 }), 1.0, Color32::WHITE);
            if big {
                self.text(p, x - 10.0, axis_y + 8.0, 10.0, Color32::GRAY, &format!("{}:{:02}", mm / 60, mm % 60));
            }
            mm += step;
        }
        let now_x = left + (right - left) * (secs as f32 / 60.0 / max_min as f32).min(1.0);
        self.line(p, Pos2::new(now_x, TIMELINE_Y + 6.0), Pos2::new(now_x, axis_y), 1.0, LIGHT_GREEN);
        let mut entries: Vec<(String, i64)> = self.m.timeline.iter().map(|(k, v)| (k.clone(), *v)).collect();
        entries.sort_by_key(|e| e.1);
        let mut used = std::collections::HashMap::new();
        for (ident, s) in entries {
            let x = left + (right - left) * (s as f32 / 60.0 / max_min as f32).min(1.0);
            let bucket = (x / 18.0) as i32;
            let stack = used.entry(bucket).or_insert(0);
            let y = axis_y - 24.0 - *stack as f32 * 22.0;
            *stack += 1;
            if y < TIMELINE_Y + 4.0 {
                continue;
            }
            self.line(p, Pos2::new(x, y + 20.0), Pos2::new(x, axis_y), 1.0, Color32::GRAY);
            if !self.draw_timeline_icon(p, &ident, rect(x - 10.0, y, 20.0, 20.0)) {
                self.fill(p, rect(x - 4.0, y + 6.0, 8.0, 8.0), ORANGE);
            }
        }
        self.text(p, 6.0, TIMELINE_Y + 4.0, 11.0, Color32::GRAY, "Timeline");
    }

    fn draw_timeline_icon(&mut self, p: &Painter, ident: &str, r: Rect) -> bool {
        let dst = self.r(r);
        if ident.starts_with("Triforce") {
            return self.assets.icon10(p, 1, dst, 1.0);
        }
        if ident.starts_with("TakeAny") {
            return self.assets.icon10(p, 4, dst, 1.0);
        }
        let box_cell = match ident {
            "WhiteSwordBox" => Some(self.m.white_sword_box.cell),
            "LadderBox" => Some(self.m.ladder_box.cell),
            "ArmosBox" => Some(self.m.armos_box.cell),
            "Level1or4Box3" => Some(self.m.third_box.cell),
            _ => ident.strip_prefix("Level").and_then(|rest| {
                let mut it = rest.split("Box");
                let d: usize = it.next()?.parse().ok()?;
                let b: usize = it.next()?.parse().ok()?;
                self.m.dungeons.get(d.wrapping_sub(1))?.boxes.get(b.wrapping_sub(1)).map(|bx| bx.cell)
            }),
        };
        if let Some(c) = box_cell {
            return c >= 0 && self.assets.item7(p, ITEM_ICON7[c as usize], dst, 1.0);
        }
        let icon = match ident {
            "WoodSword" => 19,
            "MagicalSword" => 20,
            "BlueRing" => 22,
            "BlueCandle" => 21,
            "WoodArrow" => 11,
            "BoomBook" => BOOK_ICON7,
            "Bombs" => 25,
            "Ganon" => 23,
            "Zelda" => 24,
            _ => -1,
        };
        icon >= 0 && self.assets.item7(p, icon, dst, 1.0)
    }

    // ------------------------------------------------------------------ start-up choice (which overworld)
    pub const START_LABELS: [&'static str; 7] = [
        "First Quest overworld",
        "Second Quest overworld",
        "Mixed Quest (first-quest map)",
        "Mixed Quest (second-quest map)",
        "Blank map (no overworld picture)",
        "Load the last autosave",
        "Load a saved game...",
    ];

    fn draw_start_choices(&mut self, p: &Painter) {
        self.fill(p, rect(0.0, MAP_Y, W, 264.0), Color32::from_rgba_unmultiplied(0, 0, 0, 215));
        self.text(p, 230.0, MAP_Y + 10.0, 18.0, ORANGE, "Which overworld is this seed using?");
        for (i, l) in Self::START_LABELS.iter().enumerate() {
            self.button(p, rect(234.0, MAP_Y + 40.0 + i as f32 * 31.0, 300.0, 26.0), l, Target::StartChoice(i));
        }
    }

    // ------------------------------------------------------------------ hover descriptions
    /// The Spot Summary (OverworldMapTileCustomization.MakeRemainderSummaryDisplay), shown at (50, 3) while the
    /// button is hovered: unique locations, the secrets (actual vs. your marks), non-unique locations, and shops.
    /// Icons 39 x 27 with a 3-px black border; dimmed to 0.4 once used.
    fn draw_spot_summary(&mut self, p: &Painter) {
        const CELL_W: f32 = 45.0;
        const CELL_H: f32 = 33.0;
        const OPA: f32 = 0.4;
        let (x0, y0, w) = (SPOT_SUMMARY_X, SPOT_SUMMARY_Y, 700.0);
        let mut items: Vec<(f32, Vec<(i32, f32, bool)>)> = vec![]; // (y, icons) rows, drawn after the panel size is known
        let mut texts: Vec<(f32, String)> = vec![];
        let mut rules: Vec<f32> = vec![];
        let mut y = y0 + 6.0;
        let header = |y: &mut f32, t: &str, texts: &mut Vec<(f32, String)>| {
            texts.push((*y, t.to_string()));
            *y += 22.0;
        };
        let used = |s: &Self, m: i32| s.m.uses_of_mark(m) >= s.m.max_uses(m);
        header(&mut y, "Unique Locations", &mut texts);
        for row in [vec![0, 1, 2, 3, 4, 5, 6, 7, 8], vec![9, 10, 11, 12, 13, 14, 15, 30, 31]] {
            items.push((y, row.into_iter().map(|m| (m, if used(self, m) { OPA } else { 1.0 }, false)).collect()));
            y += CELL_H;
        }
        rules.push(y + 3.0);
        y += 9.0;
        let (actual, marks) = self.m.secret_summary();
        // the original's arrows (down / up) aren't in this font, so the headings say it in words
        header(&mut y, "Actual Secrets (your marks below)", &mut texts);
        items.push((y, actual.into_iter().map(|(m, o)| (m, o, false)).collect()));
        y += CELL_H;
        items.push((y, marks.into_iter().map(|e| e.map(|(m, o)| (m, o, false)).unwrap_or((-1, 0.0, false))).collect()));
        y += CELL_H;
        header(&mut y, "Your Secret Marks (matched to the secrets above)", &mut texts);
        rules.push(y + 3.0);
        y += 9.0;
        header(&mut y, "Non-Unique Locations", &mut texts);
        let mut multi_rows = vec![];
        for m in [28, 29, 32, 33, 34] {
            let (max, n) = (self.m.max_uses(m), self.m.uses_of_mark(m));
            multi_rows.push((y, m, max.saturating_sub(n.min(max)), n.min(max)));
            y += CELL_H;
        }
        rules.push(y + 3.0);
        y += 9.0;
        header(&mut y, "Shop summary (only first item shown)", &mut texts);
        let shop_count: usize = if self.m.first_quest_overworld() { 12 } else { 15 };
        let mut found: Vec<i32> = self.m.mark.iter().flat_map(|c| c.iter()).copied().filter(|m| (SHOP_FIRST..=SHOP_LAST).contains(m)).collect();
        found.sort();
        let mut shops: Vec<(i32, f32, bool)> = found.iter().map(|&m| (m, 1.0, false)).collect();
        shops.extend(std::iter::repeat_n((32, OPA, true), shop_count.saturating_sub(found.len())));
        items.push((y, shops));
        y += CELL_H + 6.0;
        // the panel
        let panel = rect(x0, y0, w, y - y0);
        self.fill(p, panel, Color32::BLACK);
        self.stroke(p, panel, 3.0, Color32::GRAY);
        for (ty, t) in texts {
            self.text_c(p, rect(x0, ty, w, 20.0), 14.0, ORANGE, &t);
        }
        for ry in rules {
            self.fill(p, rect(x0 + 6.0, ry, w - 12.0, 3.0), Color32::GRAY);
        }
        let icon = |s: &mut Self, x: f32, y: f32, (m, o, grey): (i32, f32, bool)| {
            if m >= 0 {
                let r = s.r(rect(x + 3.0, y + 3.0, CELL_W - 6.0, CELL_H - 6.0));
                s.assets.ow_icon_faded(p, TILE_STRIP_INDEX[m as usize], r, o, grey);
            }
        };
        for (ry, row) in items {
            let x = x0 + (w - row.len() as f32 * CELL_W) / 2.0;
            for (i, e) in row.into_iter().enumerate() {
                icon(self, x + i as f32 * CELL_W, ry, e);
            }
        }
        // non-unique: still-open ones right-aligned in a left column, used ones (dimmed) left-aligned in a right
        // column, the two columns centred together (the original's allLefts / allRights stack panels)
        let left_w = multi_rows.iter().map(|r| r.2).max().unwrap_or(0) as f32 * CELL_W;
        let right_w = multi_rows.iter().map(|r| r.3).max().unwrap_or(0) as f32 * CELL_W;
        let mid = x0 + (w - left_w - right_w) / 2.0 + left_w;
        for (ry, m, open, n) in multi_rows {
            for i in 0..open {
                icon(self, mid - (i + 1) as f32 * CELL_W, ry, (m, 1.0, false));
            }
            for i in 0..n {
                icon(self, mid + i as f32 * CELL_W, ry, (m, OPA, false));
            }
        }
    }

    fn draw_hover_info(&mut self, p: &Painter) {
        if self.popup.is_some() {
            return;
        }
        let text = match &self.hover {
            Target::MapTile(x, y) => self.map_tile_text(*x, *y),
            Target::Box(id) => {
                let b = self.m.bx(*id);
                if b.cell >= 0 {
                    let st = match b.has {
                        Has::Yes => "have it",
                        Has::Skipped => "skipped",
                        Has::No => "not taken yet",
                    };
                    format!("{} ({st})\nL-click: choose item   R-click: have / skip", ITEM_NAMES[b.cell as usize])
                } else {
                    "Empty box\nL-click: choose the item   wheel: cycle".into()
                }
            }
            Target::Triforce(d) if *d < 8 => format!("Level {} triforce - click to toggle", d + 1),
            Target::Triforce(_) => "Level 9: click for Ganon defeated, then Zelda rescued".into(),
            Target::DungeonLabel(d) => {
                let h = self.m.location_hints[*d];
                let hint = if h > 0 { format!(" - hint: {}", HINT_ZONE_NAMES[h]) } else { String::new() };
                format!("Level {}{hint}\nclick: set the hinted zone   R-click: clear", d + 1)
            }
            Target::Toggle14 => "Click to choose which dungeon (1 or 4)\nhas the third item box".into(),
            Target::Progress(k) => format!("{} - click to toggle", progress_name(k)),
            Target::TakeAny(_) => "Take Any heart: click to cycle taken / skipped / unknown".into(),
            Target::HeartsAdjust => "Max hearts: L-click +1, R-click -1 (for extra hearts not in a box)".into(),
            Target::Blocker(d, s) => format!("Level {} blocker: {}", d + 1, BLOCKER_NAMES[self.m.blockers[*d][*s].kind]),
            Target::StartingItems => "Starting items and extra drops".into(),
            Target::Timer => {
                if self.paused {
                    "Click to resume the timer".into()
                } else {
                    "Click to pause the timer (right-click to reset)".into()
                }
            }
            _ => String::new(),
        };
        if text.is_empty() {
            return;
        }
        let lines: Vec<&str> = text.lines().collect();
        let w = lines.iter().map(|l| Self::measure(p, 12.0, l).x).fold(0.0, f32::max) + 12.0;
        let h = lines.len() as f32 * 16.0 + 8.0;
        let size = self.size();
        let x = (self.mouse.x + 16.0).min(size.x - w - 2.0);
        let mut y = self.mouse.y + 18.0;
        if y + h > size.y - 2.0 {
            y = self.mouse.y - h - 8.0;
        }
        let r = rect(x, y, w, h);
        self.fill_round(p, r, Color32::from_rgba_unmultiplied(30, 30, 30, 235), 3);
        self.stroke_round(p, r, 1.0, Color32::GRAY, 3);
        for (i, l) in lines.iter().enumerate() {
            self.text(p, x + 6.0, y + 4.0 + i as f32 * 16.0, 12.0, Color32::WHITE, l);
        }
    }

    fn map_tile_text(&self, x: usize, y: usize) -> String {
        let mut s = format!("{}{}  {}", (b'A' + y as u8) as char, x + 1, HINT_ZONE_NAMES[zone_at(x, y)]);
        if is_coast(x, y) {
            let b = &self.m.ladder_box;
            let what = if b.cell >= 0 { ITEM_NAMES[b.cell as usize] } else { "not chosen yet" };
            return format!("{s}\nCoast item (needs the ladder): {what}\nL-click: choose the item   R-click: have / skip");
        }
        let m = self.m.mark[x][y];
        if m >= 0 {
            s += &format!("\n{}", TILES[m as usize].description);
        }
        let e = self.m.extra[x][y];
        if (SHOP_FIRST..=SHOP_LAST).contains(&m) && (1..=8).contains(&e) {
            s += &format!("\nalso sells: {}", SHOP_ITEM_NAMES[(e - 1) as usize]);
        }
        if m == -1 && self.m.always_empty(x, y) {
            s += "\n(nothing is ever here in this quest)";
        }
        let q2 = self.m.is_second_quest();
        let vanilla = if q2 { &VANILLA_2Q_DUNGEONS } else { &VANILLA_1Q_DUNGEONS };
        if let Some(d) = vanilla.iter().position(|&(vx, vy)| vx == x && vy == y) {
            s += &format!("\n(level {} is here in the unshuffled game)", d + 1);
        }
        let mut how = vec![];
        if at(if q2 { &BOMBABLE_2Q } else { &BOMBABLE_1Q }, x, y) {
            how.push("bomb");
        }
        if at(if q2 { &BURNABLE_2Q } else { &BURNABLE_1Q }, x, y) {
            how.push("burn");
        }
        if at(if q2 { &BRACELET_2Q } else { &BRACELET_1Q }, x, y) {
            how.push("power bracelet");
        }
        if at(if q2 { &WHISTLE_2Q } else { &WHISTLE_1Q }, x, y) {
            how.push("recorder");
        }
        if at(&RAFTABLE, x, y) {
            how.push("raft");
        }
        if q2 && at(&LADDERABLE_2Q, x, y) {
            how.push("ladder");
        }
        if at(&ARMOS, x, y) {
            how.push("armos");
        }
        if at(if q2 { &GRAVE_2Q } else { &GRAVE_1Q }, x, y) {
            how.push("push grave");
        }
        if !how.is_empty() {
            s += &format!("\nspots here: {}", how.join(", "));
        }
        s
    }

    // ------------------------------------------------------------------ pop-ups
    /// Shown with the Starting Items and Extra Drops panel, as in the original (OverworldItemGridUI
    /// makeExtrasPanelAndepRefresh): the vanilla dungeon item reference (faded, opacity 0.6) and the triforce
    /// "Reference diagram" (Dungeon.MakeTriforceDecoderDiagram, opacity 0.8), below the panel.
    /// Where the two reference areas go, below the Starting Items panel: (item reference, triforce diagram).
    pub fn extras_reference_rects(&self, panel: Rect) -> (Rect, Rect) {
        let top = panel.max.y + 8.0;
        let left = rect(panel.min.x.max(2.0), top, 318.0, 296.0);
        let n = TRIFORCE_REF_SIZE;
        let right = rect((left.max.x + 8.0).min(self.size().x - n * 4.0 - 30.0), top, n * 4.0 + 26.0, n * 4.0 + 150.0);
        (left, right)
    }

    fn draw_extras_references(&mut self, p: &Painter, panel: Rect) {
        let (left, right) = self.extras_reference_rects(panel);
        // left: the vanilla dungeon item reference
        self.fill(p, left, Color32::BLACK);
        self.stroke(p, left, 4.0, Color32::GRAY);
        let faded = Color32::from_rgba_unmultiplied(255, 165, 0, 153);
        self.text(p, left.min.x + 10.0, left.min.y + 8.0, 12.0, faded, "Vanilla dungeon item reference");
        self.text(p, left.min.x + 10.0, left.min.y + 24.0, 12.0, faded, "Don't click here");
        for (i, (q, sheet)) in [("1Q", crate::art::REFERENCE_1Q), ("2Q", crate::art::REFERENCE_2Q)].into_iter().enumerate() {
            let y = left.min.y + 46.0 + i as f32 * 123.0;
            self.text(p, left.min.x + 8.0, y + 46.0, 18.0, faded, q);
            let dst = self.r(rect(left.min.x + 40.0, y, 270.0, 117.0));
            self.assets.sheet_faded(p, sheet, dst, 0.6);
        }
        // right: the triforce reference diagram
        let n = TRIFORCE_REF_SIZE;
        self.fill(p, right, Color32::BLACK);
        self.stroke(p, right, 4.0, Color32::GRAY);
        let orange = Color32::from_rgba_unmultiplied(255, 165, 0, 204);
        let line = |s: &mut Self, y: f32, t: &str| s.text_c(p, rect(right.min.x, y, right.width(), 16.0), 12.0, orange, t);
        line(self, right.min.y + 6.0, "Reference diagram - don't click here");
        line(self, right.min.y + 24.0, "(Note: in Mixed Quest dungeons,");
        line(self, right.min.y + 40.0, "7 and 8 may be swapped)");
        let haves = self.m.triforce_haves();
        line(self, right.min.y + 66.0, "First Quest or Shapes dungeons");
        self.draw_triforce_map(p, Pos2::new(right.min.x + 13.0, right.min.y + 86.0), haves, "12345678");
        line(self, right.min.y + 92.0 + n * 2.0, "Second Quest dungeons");
        let h = haves;
        let second = [h[0], h[2], h[1], h[4], h[3], h[5], h[7], h[6]];
        self.draw_triforce_map(p, Pos2::new(right.min.x + 13.0, right.min.y + 112.0 + n * 2.0), second, "13254687");
    }

    /// Dungeon.DrawTriforceMapCore: eight triangles on a 4N x 2N grid, edged dark slate blue, filled orange (with a
    /// black number) when had, else black (white number).
    fn draw_triforce_map(&mut self, p: &Painter, at: Pos2, haves: [bool; 8], labels: &str) {
        let n = TRIFORCE_REF_SIZE;
        let pt = |x: f32, y: f32| at + Vec2::new(x * n, y * n);
        let (p1, p2, p3, p4) = (pt(2.0, 0.0), pt(1.0, 1.0), pt(2.0, 1.0), pt(3.0, 1.0));
        let (p5, p6, p7, p8, p9) = (pt(0.0, 2.0), pt(1.0, 2.0), pt(2.0, 2.0), pt(3.0, 2.0), pt(4.0, 2.0));
        let tris = [[p1, p2, p3], [p1, p3, p4], [p2, p5, p6], [p4, p8, p9], [p2, p3, p7], [p2, p6, p7], [p3, p4, p7], [p4, p7, p8]];
        let label_at = [(1.5, 0.5), (2.0, 0.5), (0.5, 1.5), (3.0, 1.5), (1.5, 1.0), (1.0, 1.5), (2.0, 1.0), (2.5, 1.5)];
        let slate = Color32::from_rgb(72, 61, 139);
        for (i, t) in tris.iter().enumerate() {
            let pts: Vec<Pos2> = t.iter().map(|q| self.r(Rect::from_min_size(*q, Vec2::ZERO)).min).collect();
            let fill = if haves[i] { Color32::from_rgb(255, 165, 0) } else { Color32::BLACK };
            p.add(egui::Shape::convex_polygon(pts, fill, Stroke::new(2.0, slate)));
            let (lx, ly) = label_at[i];
            let ch = labels.chars().nth(i).unwrap_or('?').to_string();
            let r = Rect::from_min_size(at + Vec2::new(lx * n, ly * n), Vec2::splat(n / 2.0));
            self.text_c(p, r, n / 3.0, if haves[i] { Color32::BLACK } else { Color32::WHITE }, &ch);
        }
    }

    fn draw_popup(&mut self, p: &Painter) {
        let Some(mut pop) = self.popup.take() else { return };
        pop.layout(self.size());
        self.fill_round(p, pop.bounds, Color32::from_rgba_unmultiplied(16, 16, 24, 242), 3);
        self.stroke_round(p, pop.bounds, 2.0, ORANGE, 3);
        self.text(p, pop.bounds.min.x + PAD, pop.bounds.min.y + 4.0, 13.0, ORANGE, &pop.title);
        for i in 0..pop.choices.len() {
            let ch = pop.choices[i].clone();
            let r = pop.cell_rect(i).shrink(2.0);
            let bg = ch.fill.unwrap_or(if ch.selected { Color32::from_rgb(40, 70, 40) } else { Color32::BLACK });
            self.fill(p, r, bg);
            let shade = if ch.enabled { 1.0 } else { 0.3 };
            let dst = |w: f32, h: f32| self.r(Self::centered(r, w.min(r.width() - 2.0), h.min(r.height() - 2.0)));
            let drawn = match ch.icon {
                Icon::None => false,
                Icon::Item7(i, s) => {
                    let d = dst(21.0, 21.0);
                    self.assets.item7(p, i, d, s * shade)
                }
                Icon::Icon10(i, s) => {
                    let d = dst(30.0, 30.0);
                    self.assets.icon10(p, i, d, s * shade)
                }
                Icon::Ow(i) => {
                    let d = dst(48.0, 33.0);
                    i >= 0 && self.assets.draw(p, "s_icon_overworld_strip39.png", i as f32 * 16.0, 0.0, 16.0, 11.0, d, shade, false)
                }
                Icon::Room(i, done) => {
                    let d = dst(39.0, 27.0);
                    self.assets.draw(p, "new_icons13x9.png", i as f32 * 13.0, if done { 9.0 } else { 0.0 }, 13.0, 9.0, d, shade, false)
                }
                Icon::Boss(i) => {
                    let d = dst(32.0, 32.0);
                    let c = crate::art::MONSTER_CELL as f32;
                    i >= 0 && self.assets.draw(p, "zelda_bosses16x16.png", i as f32 * c, 0.0, c, c, d, shade, false)
                }
                Icon::Zitem(i) => {
                    let d = dst(32.0, 32.0);
                    i >= 0 && self.assets.draw(p, "zelda_items16x16.png", i as f32 * 16.0, 0.0, 16.0, 16.0, d, shade, false)
                }
            };
            if !drawn {
                let c = if ch.enabled { Color32::WHITE } else { Color32::DARK_GRAY };
                if pop.cols == 1 {
                    self.text(p, r.min.x + 6.0, r.min.y + (r.height() - 15.0) / 2.0, 12.0, c, &ch.text);
                } else {
                    self.text_c(p, r, 10.0, c, &ch.text);
                }
            }
            if ch.selected {
                self.stroke(p, r, 2.0, LIME);
            }
            if pop.hover == Some(i) {
                self.stroke(p, r, 2.0, Color32::YELLOW);
            }
        }
        if matches!(pop.action, Action::Starting) {
            self.draw_extras_references(p, pop.bounds);
        }
        let foot = pop.hover.and_then(|h| pop.choices.get(h)).map(|c| c.label.clone()).unwrap_or_else(|| "Click outside to cancel".into());
        if pop.foot_lines > 1 {
            let top = pop.bounds.max.y - FOOT_H - (pop.foot_lines - 1) as f32 * FOOT_LINE_H;
            for (i, line) in foot.lines().take(pop.foot_lines).enumerate() {
                self.text(p, pop.bounds.min.x + PAD, top + 3.0 + i as f32 * FOOT_LINE_H, 12.0, Color32::WHITE, line);
            }
        } else {
            self.text(p, pop.bounds.min.x + PAD, pop.bounds.max.y - FOOT_H + 3.0, 12.0, Color32::WHITE, &foot.replace('\n', " "));
        }
        self.popup = Some(pop);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::MONSTERS;

    #[test]
    fn version_texts_are_made_as_the_original_makes_them() {
        use crate::data::{about_body, PROGRAM_NAME, VERSION, WEBSITE};
        // "%d.%d.%d" of the program's version, "Z-Tracker v%s", and the About text round them
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION").split('.').take(3).collect::<Vec<_>>().join("."));
        assert_eq!(PROGRAM_NAME, format!("Z-Tracker v{VERSION}"));
        let about = about_body();
        assert!(about.starts_with(&format!("BETA RELEASE version {} of {PROGRAM_NAME} originally by Dr. Brian Lorgon111 with updates by Neononane\n\nLearn more at\n{WEBSITE}\n", env!("CARGO_PKG_VERSION"))));
        assert!(about.contains(crate::data::BUILD_DATE) && crate::data::BUILD_DATE.ends_with(" UTC"));
    }

    #[test]
    fn only_bubbles_traps_and_others_stay_bright_in_a_done_room() {
        let bright: Vec<&str> = (0..MONSTERS.len()).filter(|&m| monster_stays_bright(m)).map(|m| MONSTERS[m]).collect();
        assert_eq!(bright, ["BlueBubble", "RedBubble", "Traps", "Other", "Other2"]);
        // the original's places in its 39 x 27 room
        assert_eq!((MONSTER_AT, FLOOR_DROP_AT, ROOM_DETAIL_SIZE), ((-5.0, -3.0), (26.0, 12.0), 18.0));
        assert_eq!((ROOM_W, ROOM_H), (13.0 * 3.0, 9.0 * 3.0));
    }
}
