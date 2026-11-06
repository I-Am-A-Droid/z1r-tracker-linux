//! What the mouse and keyboard do on the tracker surface.

use crate::data::*;
use crate::model::*;
use crate::settings::{KeyChord, BOOL_OPTIONS};
use crate::view::*;
use eframe::egui::{self, Color32, Key, Modifiers, Pos2};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Btn {
    Left,
    Right,
    Middle,
}

const STARTING_ICONS: [i32; 14] = [17, 10, 1, 6, 7, 0, 2, 12, 16, 4, 3, 5, 9, 8];

impl View {
    /// Handles this frame's mouse and keyboard events (the notes box gets its own).
    pub fn handle_input(&mut self, ctx: &egui::Context, notes_focused: bool) {
        let (events, hover_pos) = ctx.input(|i| (i.events.clone(), i.pointer.hover_pos()));
        let notes = self.notes_rect();
        let mut changed = false;
        if let Some(hp) = hover_pos {
            let lp = hp - self.origin;
            if lp != self.mouse {
                changed = true;
            }
            self.mouse = lp;
            if let Some(pop) = &mut self.popup {
                pop.hover = pop.hit(lp);
            } else {
                self.hover = if notes.contains(lp) { Target::None } else { self.hit_test(lp) };
            }
        } else if self.hover != Target::None {
            self.hover = Target::None;
            changed = true;
        }
        for e in events {
            match e {
                egui::Event::PointerButton { pos, button, pressed: true, modifiers } => {
                    let lp = pos - self.origin;
                    if self.popup.is_none() && notes.contains(lp) {
                        continue;
                    }
                    let btn = match button {
                        egui::PointerButton::Secondary => Btn::Right,
                        egui::PointerButton::Middle => Btn::Middle,
                        egui::PointerButton::Primary => Btn::Left,
                        _ => continue,
                    };
                    self.click(lp, btn, modifiers);
                    changed = true;
                }
                egui::Event::MouseWheel { delta, .. } => {
                    if delta.y.abs() < 0.01 {
                        continue;
                    }
                    if let Some(hp) = hover_pos {
                        let lp = hp - self.origin;
                        if !notes.contains(lp) {
                            self.wheel(lp, if delta.y > 0.0 { -1 } else { 1 });
                            changed = true;
                        }
                    }
                }
                egui::Event::Key { key, pressed: true, repeat: false, modifiers, .. } => {
                    if key == Key::Escape && self.popup.is_some() {
                        self.popup = None;
                        changed = true;
                    } else if !notes_focused && self.key(key, modifiers) {
                        changed = true;
                    }
                }
                _ => {}
            }
        }
        if changed {
            ctx.request_repaint();
        }
    }

    fn click(&mut self, p: Pos2, btn: Btn, mods: Modifiers) {
        let shift = mods.shift;
        if let Some(pop) = self.popup.take() {
            match pop.hit(p) {
                Some(i) if pop.choices[i].enabled => {
                    let action = pop.action.clone();
                    if pop.keep_open {
                        self.popup = Some(pop);
                    }
                    self.choose(action, i, btn == Btn::Right);
                }
                _ if pop.bounds.contains(p) => self.popup = Some(pop),
                _ => {} // clicked outside: cancel
            }
            return;
        }
        let t = self.hit_test(p);
        if !self.m.started && !matches!(t, Target::StartChoice(_) | Target::Button(_)) {
            return;
        }
        match t {
            Target::MapTile(x, y) => self.click_map_tile(x, y, btn, shift, p),
            Target::Box(id) => self.click_box(id, btn, p),
            Target::Triforce(d) => self.click_triforce(d, btn),
            Target::DungeonLabel(d) => {
                if btn == Btn::Right {
                    self.m.location_hints[d] = 0;
                } else {
                    self.open_hint_zone(d, p);
                }
            }
            Target::Toggle14 => self.m.second_quest_dungeons = !self.m.second_quest_dungeons,
            Target::Progress(k) => {
                let v = !self.m.prog(k);
                self.m.set_prog(k, v);
                let ident = k.trim_start_matches("PlayerHas").replace("DefeatedGanon", "Ganon").replace("RescuedZelda", "Zelda");
                self.m.stamp(&ident, v);
                if k == "PlayerHasRescuedZelda" {
                    self.zelda_changed();
                }
            }
            Target::TakeAny(i) => {
                let v = &mut self.m.take_any_hearts[i];
                *v = if btn == Btn::Right { (*v + 2) % 3 } else { (*v + 1) % 3 };
                let has = *v == 1;
                self.m.stamp(&format!("TakeAny{}", i + 1), has);
            }
            Target::HeartsAdjust => self.m.max_hearts_diff += if btn == Btn::Right { -1 } else { 1 },
            Target::Timer => {
                if btn == Btn::Right {
                    self.confirm("Reset the timer to 0:00:00?", Action::ResetTimer, p);
                } else {
                    self.paused = !self.paused;
                }
            }
            Target::Button(name) => match name {
                "Options" => self.open_options(),
                "Hints" => self.open_hints(),
                _ => self.commands.push(name),
            },
            Target::Tab(i) => self.m.tab = i,
            Target::Room(x, y) => self.click_room(x, y, btn, shift, p),
            Target::HDoor(x, y) => self.click_door(true, x, y, btn, shift),
            Target::VDoor(x, y) => self.click_door(false, x, y, btn, shift),
            Target::Blocker(d, s) => {
                if btn == Btn::Right {
                    self.m.blockers[d][s].kind = BLOCKER_NOTHING;
                } else {
                    self.open_blocker(d, s, p);
                }
            }
            Target::StartChoice(i) => self.click_start(i),
            Target::StartingItems => self.open_starting_items(p),
            Target::None => {}
        }
    }

    #[cfg(test)]
    fn wheel_on_coast_for_test(&mut self) {
        let r = rect(COAST_X as f32 * TILE_W, MAP_Y + COAST_Y as f32 * TILE_H, TILE_W, TILE_H);
        self.hits.push((r, Target::MapTile(COAST_X, COAST_Y)));
        self.wheel(r.center(), 1);
    }

    fn wheel(&mut self, p: Pos2, mut dir: i32) {
        if self.popup.is_some() || !self.m.started {
            return;
        }
        if self.settings.get("ReverseScrollWheelDirection") {
            dir = -dir;
        }
        let tab = self.m.tab;
        match self.hit_test(p) {
            Target::MapTile(x, y) if is_coast(x, y) => self.cycle_box(BoxId::Ladder, dir),
            Target::MapTile(x, y) => self.cycle_mark(x, y, dir),
            Target::Box(id) => self.cycle_box(id, dir),
            Target::Blocker(d, s) => {
                let k = &mut self.m.blockers[d][s].kind;
                *k = (*k as i32 + dir).rem_euclid(BLOCKER_KINDS.len() as i32) as usize;
            }
            // wheel up = monster pop-up, wheel down = floor-drop pop-up (as the original's option text describes)
            Target::Room(x, y) => {
                if dir < 0 {
                    self.open_monster(x, y, p)
                } else {
                    self.open_floor_drop(x, y, p)
                }
            }
            Target::HDoor(x, y) => {
                let d = &mut self.m.maps[tab].hdoors[x][y];
                *d = (*d + dir).rem_euclid(5);
            }
            Target::VDoor(x, y) => {
                let d = &mut self.m.maps[tab].vdoors[x][y];
                *d = (*d + dir).rem_euclid(5);
            }
            Target::Tab(_) => self.m.tab = (tab as i32 + dir).rem_euclid(9) as usize,
            _ => {}
        }
    }

    // ------------------------------------------------------------------ the chosen pop-up entry
    fn choose(&mut self, action: Action, i: usize, right: bool) {
        match action {
            Action::Mark(x, y, order) => {
                self.set_mark(x, y, if i == 0 { -1 } else { order[i - 1] });
                self.after_mark(x, y);
            }
            Action::ShopSecond(x, y) => {
                if i == 9 {
                    let a = Pos2::new(x as f32 * TILE_W, MAP_Y + y as f32 * TILE_H);
                    self.open_mark_picker(x, y, a);
                } else {
                    self.m.extra[x][y] = i as i32;
                }
            }
            Action::Item(id) => {
                let b = self.m.bx_mut(id);
                b.cell = if i < ITEM_NAMES.len() { i as i32 } else { -1 };
                if b.cell >= 0 && b.has == Has::No && !right {
                    b.has = Has::Yes;
                }
                if b.cell < 0 {
                    b.has = Has::No;
                }
                let (ident, has) = (b.ident.clone(), b.has == Has::Yes && b.cell >= 0);
                self.m.stamp(&ident, has);
            }
            Action::HintZone(d) => self.m.location_hints[d] = i,
            Action::RoomType(x, y) => {
                let r = &mut self.m.maps[self.m.tab].rooms[x][y];
                r.kind = i;
                r.completed = i != 0;
                if i == 0 {
                    r.monster = 0;
                    r.floor_drop = 0;
                }
            }
            Action::Monster(x, y) => {
                let r = &mut self.m.maps[self.m.tab].rooms[x][y];
                r.monster = i;
                if r.kind == ROOM_UNMARKED && i > 0 {
                    r.kind = ROOM_NON_DESCRIPT;
                    r.completed = true;
                }
            }
            Action::FloorDrop(x, y) => {
                let r = &mut self.m.maps[self.m.tab].rooms[x][y];
                r.floor_drop = i;
                r.floor_drop_bright = !right;
                if r.kind == ROOM_UNMARKED && i > 0 {
                    r.kind = ROOM_NON_DESCRIPT;
                    r.completed = true;
                }
            }
            Action::Blocker(d, s) => self.m.blockers[d][s].kind = i,
            Action::ResetTimer => {
                if i == 0 {
                    self.m.seconds = 0;
                    self.show_toast("Timer reset", 2.0);
                }
            }
            Action::StartOver => {
                if i == 0 {
                    self.m.started = false;
                    self.paused = false;
                }
            }
            Action::Options => self.choose_option(i),
            Action::Hints => {
                if i < 11 {
                    self.m.location_hints[i] = (self.m.location_hints[i] + if right { 10 } else { 1 }) % 11;
                } else {
                    let k = HINT_FLAG_KEYS[i - 11];
                    let v = !self.m.hint_flags[k];
                    self.m.hint_flags.insert(k, v);
                }
                let c = self.hints_choices();
                if let Some(p) = &mut self.popup {
                    p.choices = c;
                }
            }
            Action::Starting => {
                if i < STARTING_KEYS.len() {
                    let k = STARTING_KEYS[i];
                    let v = !self.m.starting[k];
                    self.m.starting.insert(k, v);
                } else {
                    let t = &mut self.m.starting_triforces[i - STARTING_KEYS.len()];
                    *t = !*t;
                }
                let c = self.starting_choices();
                if let Some(p) = &mut self.popup {
                    p.choices = c;
                }
            }
            Action::TakeAny(x, y) => {
                // potion / candle / heart / leave (PieMenus::TakeAnyPieMenuAsync); the header hearts follow the map
                let choice = [TileChoice::POTION, TileChoice::CANDLE, TileChoice::HEART, TileChoice::LEFT][i.min(3)];
                self.set_tile_choice(x, y, choice);
                // setOverworldMapExtraData: the cave remembers it was used
                self.m.extra[x][y] = if choice != TileChoice::LEFT { TAKE_ANY_MARK } else { 0 };
            }
            Action::TakeThis(x, y) => {
                // wood sword / candle / leave (PieMenus::TakeThisPieMenuAsync)
                let choice = [TileChoice::SWORD, TileChoice::CANDLE, TileChoice::LEFT][i.min(2)];
                self.set_tile_choice(x, y, choice);
                self.m.extra[x][y] = if choice != TileChoice::LEFT { WOOD_SWORD_CAVE_MARK } else { 0 };
            }
            Action::LoadList(files) => {
                if let Some(f) = files.get(i) {
                    self.load_request = Some(f.clone());
                }
            }
        }
    }

    fn choose_option(&mut self, i: usize) {
        let n = BOOL_OPTIONS.len();
        if i < n {
            let k = BOOL_OPTIONS[i].0;
            let v = !self.settings.get(k);
            self.settings.set(k, v);
            if k == "IsSecondQuestDungeons" {
                self.m.second_quest_dungeons = v;
            }
            if k == "SmallerAppWindow" {
                self.commands.push("Zoom");
            }
        } else if i == n {
            self.m.mirror = !self.m.mirror;
        } else if i == n + 1 {
            self.m.boomstick_seed = !self.m.boomstick_seed;
        } else if i == n + 2 {
            self.m.quest = (self.m.quest + 1) % 5;
        } else if i == n + 3 {
            self.commands.push("OpenSaves");
        } else if i == n + 4 {
            self.confirm("Discard this seed and start over?", Action::StartOver, Pos2::new(W / 2.0, 200.0));
            return;
        }
        let c = self.options_choices();
        if let Some(p) = &mut self.popup {
            p.choices = c;
        }
    }

    // ------------------------------------------------------------------ overworld
    fn click_map_tile(&mut self, x: usize, y: usize, btn: Btn, shift: bool, p: Pos2) {
        let mark = self.m.mark[x][y];
        let c = &mut self.m.circle[x][y];
        if btn == Btn::Middle {
            // middle-click toggles a circle (keeping its colour)
            *c = if *c % 100 == 0 { *c + 1 } else { *c - *c % 100 };
            return;
        }
        if btn == Btn::Left && shift {
            if *c % 100 == 0 {
                *c += 1;
            }
            self.show_toast("Circled. Press a letter or number now to label it (middle-click removes it).", 3.0);
            return;
        }
        if btn == Btn::Right && shift {
            // shift-right-click: next circle colour
            *c = (*c + 100) % 300;
            return;
        }
        if is_coast(x, y) {
            // F16 only takes the coast item: left-click picks it, right-click marks it had / skipped
            self.click_box(BoxId::Ladder, btn, p);
            if let Some(pop) = &mut self.popup {
                pop.title = "Coast item (F16, needs the ladder): which item is it?".into();
            }
            return;
        }
        if btn == Btn::Right {
            // right-click only does something when item icons are allowed on non-shop tiles
            if mark >= 0 && !(SHOP_FIRST..=SHOP_LAST).contains(&mark) && self.settings.get("AllowItemIconOnNonShopTile") {
                self.open_shop_second(x, y, p);
            }
            return;
        }
        if (SHOP_FIRST..=SHOP_LAST).contains(&mark) {
            self.open_shop_second(x, y, p);
        } else {
            self.open_mark_picker(x, y, p);
        }
    }

    pub fn set_mark(&mut self, x: usize, y: usize, mark: i32) {
        if !self.m.can_use(mark, x, y) {
            self.show_toast(format!("All {} marks are already used", TILES[mark as usize].name), 3.0);
            return;
        }
        let before = self.m.mark[x][y];
        self.m.mark[x][y] = mark;
        self.m.extra[x][y] = if mark == -1 { -1 } else { 0 };
        if before != mark && (before == TAKE_ANY_MARK || before == WOOD_SWORD_CAVE_MARK) {
            // the cave is gone: forget what was taken there
            self.set_tile_choice(x, y, -1);
        } else if mark == TAKE_ANY_MARK || before == TAKE_ANY_MARK {
            self.m.sync_take_any_hearts();
        }
    }

    /// Records what was taken at a Take Any / Wood Sword cave tile and keeps everything that depends on it in step:
    /// the take-any hearts in the header, and the blue candle / wood sword when this tile was where they came from.
    fn set_tile_choice(&mut self, x: usize, y: usize, choice: i32) {
        let old = self.m.tile_choice[x][y];
        self.m.tile_choice[x][y] = choice;
        for (gift, key, ident) in [(TileChoice::CANDLE, "PlayerHasBlueCandle", "BlueCandle"), (TileChoice::SWORD, "PlayerHasWoodSword", "WoodSword")] {
            if choice == gift && old != gift {
                self.m.set_prog(key, true);
                self.m.stamp(ident, true);
            } else if old == gift && choice != gift && !self.m.other_tile_gave(x, y, gift) {
                self.m.set_prog(key, false);
                self.m.stamp(ident, false);
            }
        }
        self.m.sync_take_any_hearts();
    }

    /// Marking the Armos screen or the White Sword cave goes straight on to "which item did you get there?",
    /// like the original's overworldAcceleratorTable ($OverworldMapTileCustomization::.cctor: marks 31 and 14).
    /// Picking the item marks it as had, so it shows on the timeline at the current time.
    fn after_mark(&mut self, x: usize, y: usize) {
        let below_tile = Pos2::new(x as f32 * TILE_W + TILE_W / 2.0, MAP_Y + (y + 1) as f32 * TILE_H);
        let id = match self.m.mark[x][y] {
            m if m == ARMOS_MARK => BoxId::Armos,
            m if m == WHITE_SWORD_CAVE_MARK => BoxId::WhiteSword,
            m if m == TAKE_ANY_MARK => {
                // PieMenus::TakeAnyPieMenuAsync
                let mut pop = Popup::new("Take Any: what did you take?", 4, 70.0, 44.0, below_tile, Action::TakeAny(x, y));
                pop.choices.push(Choice::new(Icon::None, "Potion", "Took the potion"));
                pop.choices.push(Choice::new(Icon::Item7(21, 1.0), "Candle", "Took the blue candle"));
                pop.choices.push(Choice::new(Icon::Icon10(4, 1.0), "Heart", "Took the heart container"));
                pop.choices.push(Choice::new(Icon::None, "Leave", "Took nothing (come back later)"));
                self.popup = Some(pop);
                return;
            }
            m if m == WOOD_SWORD_CAVE_MARK => {
                // PieMenus::TakeThisPieMenuAsync
                let mut pop = Popup::new("Wood Sword cave: what did you take?", 3, 70.0, 44.0, below_tile, Action::TakeThis(x, y));
                pop.choices.push(Choice::new(Icon::Item7(19, 1.0), "Sword", "Took the wood sword"));
                pop.choices.push(Choice::new(Icon::Item7(21, 1.0), "Candle", "Took the blue candle"));
                pop.choices.push(Choice::new(Icon::None, "Leave", "Took nothing (come back later)"));
                self.popup = Some(pop);
                return;
            }
            _ => return,
        };
        self.click_box(id, Btn::Left, below_tile);
        if let Some(p) = &mut self.popup {
            p.title = format!("{}: which item did you get?", TILES[self.m.mark[x][y] as usize].name);
        }
    }

    fn mark_order(&self) -> Vec<i32> {
        let all: Vec<i32> = (0..TILES.len() as i32).collect();
        if self.settings.get("ShopsFirst") {
            let (shops, rest): (Vec<i32>, Vec<i32>) = all.into_iter().partition(|i| (SHOP_FIRST..=SHOP_LAST).contains(i));
            shops.into_iter().chain(rest).collect()
        } else {
            all
        }
    }

    fn cycle_mark(&mut self, x: usize, y: usize, dir: i32) {
        let mut order = vec![-1];
        order.extend(self.mark_order());
        let n = order.len() as i32;
        let mut i = order.iter().position(|m| *m == self.m.mark[x][y]).unwrap_or(0) as i32;
        for _ in 0..n {
            i = (i + dir).rem_euclid(n);
            if self.m.can_use(order[i as usize], x, y) {
                self.set_mark(x, y, order[i as usize]);
                return;
            }
        }
    }

    fn open_mark_picker(&mut self, x: usize, y: usize, p: Pos2) {
        let order = self.mark_order();
        let title = format!("Mark {}{} ({})", (b'A' + y as u8) as char, x + 1, HINT_ZONE_NAMES[zone_at(x, y)]);
        let mut pop = Popup::new(title, 8, 54.0, 39.0, Pos2::new(p.x, (p.y + 20.0).min(H - 330.0)), Action::Mark(x, y, order.clone()));
        pop.choices.push(Choice::new(Icon::None, "none", "Unmarked").sel(self.m.mark[x][y] == -1));
        for &m in &order {
            let t = &TILES[m as usize];
            let (used, max) = (self.m.uses_of_mark(m), self.m.max_uses(m));
            let label = if max < 999 { format!("{}  ({used}/{max} used)", t.name) } else { t.name.to_string() };
            let text = match m {
                36 => "P1",
                37 => "P2",
                _ => t.name,
            };
            pop.choices.push(Choice::new(Icon::Ow(TILE_STRIP_INDEX[m as usize]), text, label).en(self.m.can_use(m, x, y)).sel(self.m.mark[x][y] == m));
        }
        self.popup = Some(pop);
    }

    fn open_shop_second(&mut self, x: usize, y: usize, p: Pos2) {
        let mark = self.m.mark[x][y];
        let what = if (SHOP_FIRST..=SHOP_LAST).contains(&mark) { "second item for sale" } else { "item icon on this tile" };
        let title = format!("{}: {what}", TILES[mark as usize].name);
        let mut pop = Popup::new(title, 5, 60.0, 40.0, Pos2::new(p.x, p.y + 20.0), Action::ShopSecond(x, y));
        pop.choices.push(Choice::new(Icon::None, "none", "No second item").sel(self.m.extra[x][y] <= 0));
        for i in 0..8 {
            pop.choices.push(
                Choice::new(Icon::Item7(SHOP_ITEM_ICON7[i], 1.0), SHOP_ITEM_NAMES[i], SHOP_ITEM_NAMES[i])
                    .sel(self.m.extra[x][y] == i as i32 + 1)
                    .en(i as i32 + 1 != mark - 15),
            );
        }
        pop.choices.push(Choice::new(Icon::None, "change", "Change what this tile is"));
        self.popup = Some(pop);
    }

    // ------------------------------------------------------------------ item boxes
    fn click_box(&mut self, id: BoxId, btn: Btn, p: Pos2) {
        if btn != Btn::Left {
            let b = self.m.bx_mut(id);
            b.has = b.has.next(); // no -> yes -> skipped
            let (ident, has) = (b.ident.clone(), b.has == Has::Yes && b.cell >= 0);
            self.m.stamp(&ident, has);
            return;
        }
        let cur = self.m.bx(id).cell;
        let mut pop = Popup::new("Which item is in this box?", 8, 40.0, 40.0, Pos2::new(p.x, p.y + 20.0), Action::Item(id));
        for i in 0..ITEM_NAMES.len() {
            let used = self.m.uses_of_item(i as i32);
            let label = if ITEM_MAX_COUNT[i] > 1 { format!("{} ({used}/{})", ITEM_NAMES[i], ITEM_MAX_COUNT[i]) } else { ITEM_NAMES[i].to_string() };
            pop.choices.push(
                Choice::new(Icon::Item7(ITEM_ICON7[i], 1.0), &ITEM_NAMES[i][..2], label)
                    .en(cur == i as i32 || used < ITEM_MAX_COUNT[i])
                    .sel(cur == i as i32),
            );
        }
        pop.choices.push(Choice::new(Icon::None, "none", "Empty box").sel(cur < 0));
        self.popup = Some(pop);
    }

    fn cycle_box(&mut self, id: BoxId, dir: i32) {
        let n = ITEM_NAMES.len() as i32 + 1; // items + empty
        let cur = self.m.bx(id).cell;
        let mut i = if cur < 0 { n - 1 } else { cur };
        for _ in 0..n {
            i = (i + dir).rem_euclid(n);
            if i == n - 1 {
                self.m.bx_mut(id).cell = -1;
                break;
            }
            if self.m.uses_of_item(i) < ITEM_MAX_COUNT[i as usize] {
                self.m.bx_mut(id).cell = i;
                break;
            }
        }
        let b = self.m.bx(id);
        let (ident, has) = (b.ident.clone(), b.has == Has::Yes && b.cell >= 0);
        self.m.stamp(&ident, has);
    }

    // ------------------------------------------------------------------ triforces, zelda
    fn click_triforce(&mut self, d: usize, btn: Btn) {
        if d < 8 {
            let v = !self.m.dungeons[d].triforce;
            self.m.dungeons[d].triforce = v;
            self.m.stamp(&format!("Triforce{}", d + 1), v);
            return;
        }
        // level 9: nothing -> Ganon defeated -> Zelda rescued -> nothing
        let (mut g, mut z) = (self.m.prog("PlayerHasDefeatedGanon"), self.m.prog("PlayerHasRescuedZelda"));
        if btn == Btn::Right || (g && z) {
            g = false;
            z = false;
        } else if !g {
            g = true;
        } else {
            z = true;
        }
        self.m.set_prog("PlayerHasDefeatedGanon", g);
        self.m.set_prog("PlayerHasRescuedZelda", z);
        self.m.stamp("Ganon", g);
        self.m.stamp("Zelda", z);
        self.zelda_changed();
    }

    fn zelda_changed(&mut self) {
        if self.m.prog("PlayerHasRescuedZelda") {
            self.paused = true;
            let s = self.m.seconds;
            self.show_toast(format!("Seed complete in {}:{:02}:{:02}!", s / 3600, (s / 60) % 60, s % 60), 6.0);
            if self.settings.get("SaveOnCompletion") {
                self.commands.push("SaveFinished");
            }
        } else {
            self.paused = false;
        }
    }

    fn open_hint_zone(&mut self, d: usize, p: Pos2) {
        let title = format!("Level {}: which zone does the hint point to?", d + 1);
        let mut pop = Popup::new(title, 4, 110.0, 30.0, Pos2::new(p.x + 150.0, p.y + 20.0), Action::HintZone(d));
        for (z, name) in HINT_ZONE_NAMES.iter().enumerate() {
            pop.choices.push(Choice::new(Icon::None, *name, *name).sel(self.m.location_hints[d] == z));
        }
        self.popup = Some(pop);
    }

    // ------------------------------------------------------------------ dungeon rooms and doors
    fn click_room(&mut self, x: usize, y: usize, btn: Btn, shift: bool, p: Pos2) {
        let tab = self.m.tab;
        if btn == Btn::Middle {
            let c = &mut self.m.maps[tab].circled[x][y];
            *c = !*c;
            return;
        }
        if btn == Btn::Left && shift {
            self.m.maps[tab].rooms[x][y] = Room::new();
            return;
        }
        if btn == Btn::Right {
            self.open_room_type(x, y, p);
            return;
        }
        let prefer_generic = self.settings.get("DefaultRoomPreferNonDescriptToMaybePushBlock");
        let room = &mut self.m.maps[tab].rooms[x][y];
        if room.kind == ROOM_UNMARKED {
            room.kind = if prefer_generic { ROOM_NON_DESCRIPT } else { ROOM_MAYBE_PUSH_BLOCK };
            room.completed = true;
        } else if is_entrance(room.kind) {
            // entrance arrow cycles S -> W -> N -> E
            room.kind = match room.kind {
                30 => 28,
                28 => 29,
                29 => 27,
                _ => 30,
            };
        } else {
            room.completed = !room.completed;
        }
    }

    fn open_room_type(&mut self, x: usize, y: usize, p: Pos2) {
        let cur = self.m.maps[self.m.tab].rooms[x][y].kind;
        let mut pop = Popup::new("Room type", 9, 46.0, 36.0, Pos2::new(p.x, (p.y + 20.0).min(H - 200.0)), Action::RoomType(x, y));
        for i in 0..ROOM_TYPES.len() {
            let icon = if i == 0 { Icon::None } else { Icon::Room(ROOM_ICON[i], i >= 32) };
            pop.choices.push(Choice::new(icon, if i == 0 { "none" } else { ROOM_TYPE_NAMES[i] }, ROOM_TYPE_NAMES[i]).sel(cur == i));
        }
        self.popup = Some(pop);
    }

    fn open_monster(&mut self, x: usize, y: usize, p: Pos2) {
        let cur = self.m.maps[self.m.tab].rooms[x][y].monster;
        let mut pop = Popup::new("Monster in this room", 8, 40.0, 40.0, Pos2::new(p.x, (p.y + 20.0).min(H - 200.0)), Action::Monster(x, y));
        for i in 0..MONSTERS.len() {
            let text = if i == 0 { "none".to_string() } else { MONSTER_NAMES[i][..2].to_string() };
            pop.choices.push(Choice::new(Icon::Boss(MONSTER_ICON[i]), text, MONSTER_NAMES[i]).sel(cur == i));
        }
        self.popup = Some(pop);
    }

    fn open_floor_drop(&mut self, x: usize, y: usize, p: Pos2) {
        let cur = self.m.maps[self.m.tab].rooms[x][y].floor_drop;
        let title = "Item on the floor (right-click: dim it = already taken)";
        let mut pop = Popup::new(title, 9, 40.0, 40.0, Pos2::new(p.x, (p.y + 20.0).min(H - 140.0)), Action::FloorDrop(x, y));
        for i in 0..FLOOR_DROPS.len() {
            let text = if i == 0 { "none".to_string() } else { FLOOR_DROP_NAMES[i][..2].to_string() };
            pop.choices.push(Choice::new(Icon::Zitem(FLOOR_DROP_ICON[i]), text, FLOOR_DROP_NAMES[i]).sel(cur == i));
        }
        self.popup = Some(pop);
    }

    fn click_door(&mut self, horizontal: bool, x: usize, y: usize, btn: Btn, shift: bool) {
        let tab = self.m.tab;
        let door = if horizontal { &mut self.m.maps[tab].hdoors[x][y] } else { &mut self.m.maps[tab].vdoors[x][y] };
        // DungeonUI::installDoorBehavior$cont@917: shift-left = previous state, shift-right = next state;
        // otherwise left = yes, right = no, middle = yellow, and the same click again clears it
        if shift && btn != Btn::Middle {
            *door = (*door + if btn == Btn::Left { -1 } else { 1 }).rem_euclid(DOOR_PURPLE + 1);
            return;
        }
        let want = match btn {
            Btn::Left => DOOR_YES,
            Btn::Right => DOOR_NO,
            Btn::Middle => DOOR_YELLOW,
        };
        *door = if *door == want { DOOR_UNKNOWN } else { want };
    }

    // ------------------------------------------------------------------ blockers
    fn open_blocker(&mut self, d: usize, s: usize, p: Pos2) {
        let cur = self.m.blockers[d][s].kind;
        let title = format!("Level {}: what blocks you?", d + 1);
        let mut pop = Popup::new(title, 8, 44.0, 40.0, Pos2::new(p.x.min(W - 200.0), p.y + 20.0), Action::Blocker(d, s));
        for i in 0..BLOCKER_KINDS.len() {
            let maybe = blocker_is_maybe(i);
            let icon = if BLOCKER_ICON7[i] >= 0 { Icon::Item7(BLOCKER_ICON7[i], if maybe { 0.6 } else { 1.0 }) } else { Icon::None };
            let text = if i == BLOCKER_NOTHING { "none".to_string() } else { BLOCKER_KINDS[i]["Blocker_".len()..][..2].to_string() };
            let mut c = Choice::new(icon, text, BLOCKER_NAMES[i]).sel(cur == i);
            if maybe {
                c.fill = Some(Color32::from_rgb(50, 50, 20));
            }
            pop.choices.push(c);
        }
        self.popup = Some(pop);
    }

    // ------------------------------------------------------------------ start, options, hints, starting items
    fn click_start(&mut self, i: usize) {
        match i {
            0..=4 => {
                self.m.reset(i as i64);
                self.m.second_quest_dungeons = self.settings.get("IsSecondQuestDungeons");
                self.m.started = true;
                self.paused = false;
                let msg = if i <= 3 { "Good luck! Left-click map tiles to mark them." } else { "Blank map ready." };
                self.show_toast(msg, 3.0);
            }
            5 => self.commands.push("LoadAutosave"),
            _ => self.commands.push("Load"),
        }
    }

    fn confirm(&mut self, q: &str, action: Action, p: Pos2) {
        let mut pop = Popup::new(q, 2, 120.0, 30.0, Pos2::new(p.x.min(W - 160.0), p.y + 10.0), action);
        pop.choices.push(Choice::new(Icon::None, "Yes", "Yes"));
        pop.choices.push(Choice::new(Icon::None, "No", "No"));
        self.popup = Some(pop);
    }

    fn options_choices(&self) -> Vec<Choice> {
        let check = |b: bool| if b { "[x] " } else { "[  ] " };
        let mut v: Vec<Choice> = BOOL_OPTIONS.iter().map(|(k, l, _)| Choice::new(Icon::None, format!("{}{l}", check(self.settings.get(k))), *l)).collect();
        v.push(Choice::new(Icon::None, format!("{}Mirror the overworld (this seed)", check(self.m.mirror)), "Mirror overworld"));
        v.push(Choice::new(Icon::None, format!("{}Boomstick seed (this seed)", check(self.m.boomstick_seed)), "Boomstick seed"));
        let q = match self.m.quest {
            0 => "First Quest",
            1 => "Second Quest",
            2 => "Mixed (first map)",
            3 => "Mixed (second map)",
            _ => "Blank",
        };
        v.push(Choice::new(Icon::None, format!("Overworld: {q}  (click to change)"), "Change the overworld map"));
        v.push(Choice::new(Icon::None, "Open the saves folder", crate::settings::save_dir().display().to_string()));
        v.push(Choice::new(Icon::None, "Start over with a new seed", "Clears everything"));
        v
    }

    fn open_options(&mut self) {
        let mut pop = Popup::new("Options (changes are saved right away)", 1, 470.0, 24.0, Pos2::new(W / 2.0, 60.0), Action::Options);
        pop.keep_open = true;
        pop.choices = self.options_choices();
        self.popup = Some(pop);
    }

    fn hints_choices(&self) -> Vec<Choice> {
        let mut v = vec![];
        for (i, (hint, meaning)) in HINT_MEANINGS.iter().enumerate() {
            let text = format!("\"{hint}\"  =  {meaning}   ->   {}", HINT_ZONE_NAMES[self.m.location_hints[i]]);
            v.push(Choice::new(Icon::None, text, *meaning));
        }
        let flag_text = [
            "No feat of strength (no power-bracelet spots)",
            "Sail not (no raft spots)",
            "Sail to (raft spot matters)",
            "Play melody (recorder spot)",
            "Step over water (ladder)",
            "Fire arrows (burnable)",
        ];
        for (i, k) in HINT_FLAG_KEYS.iter().enumerate() {
            let on = self.m.hint_flags[k];
            v.push(Choice::new(Icon::None, format!("{}{}", if on { "[x] " } else { "[  ] " }, flag_text[i]), *k));
        }
        v
    }

    fn open_hints(&mut self) {
        let title = "Hint decoder: click a hint to set its zone (right-click goes back)";
        let mut pop = Popup::new(title, 1, 540.0, 24.0, Pos2::new(W / 2.0, 120.0), Action::Hints);
        pop.keep_open = true;
        pop.choices = self.hints_choices();
        self.popup = Some(pop);
    }

    fn starting_choices(&self) -> Vec<Choice> {
        let mut v = vec![];
        for (i, k) in STARTING_KEYS.iter().enumerate() {
            let on = self.m.starting[k];
            let label = k.replace("PlayerHas", "Start with ");
            v.push(Choice::new(Icon::Item7(STARTING_ICONS[i], if on { 1.0 } else { 0.3 }), &k[9..11], label).sel(on));
        }
        for i in 0..8 {
            let on = self.m.starting_triforces[i];
            v.push(Choice::new(Icon::Icon10(if on { 1 } else { 0 }, 1.0), format!("T{}", i + 1), format!("Start with triforce {}", i + 1)).sel(on));
        }
        v
    }

    fn open_starting_items(&mut self, p: Pos2) {
        let title = "Starting items and extra drops (click to toggle)";
        let mut pop = Popup::new(title, 8, 44.0, 40.0, Pos2::new(p.x, p.y + 20.0), Action::Starting);
        pop.keep_open = true;
        pop.choices = self.starting_choices();
        self.popup = Some(pop);
    }

    pub fn open_load_list(&mut self, files: Vec<PathBuf>) {
        let mut pop = Popup::new("Load which save? (newest first)", 1, 600.0, 24.0, Pos2::new(W / 2.0, 160.0), Action::LoadList(files.clone()));
        for f in files.iter().take(30) {
            let name = f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let folder = f.parent().map(|d| d.display().to_string()).unwrap_or_default();
            pop.choices.push(Choice::new(Icon::None, name, folder));
        }
        if pop.choices.is_empty() {
            pop.choices.push(Choice::new(Icon::None, "No saves found yet", "").en(false));
        }
        self.popup = Some(pop);
    }

    // ------------------------------------------------------------------ hot keys (while hovering)
    fn key(&mut self, key: Key, mods: Modifiers) -> bool {
        if self.popup.is_some() || !self.m.started {
            return false;
        }
        let chord = KeyChord::new(key, mods);
        // on F16 the item hot keys set the coast item; overworld marks don't apply
        let hover = match self.hover.clone() {
            Target::MapTile(x, y) if is_coast(x, y) => Target::Box(BoxId::Ladder),
            h => h,
        };
        match hover {
            Target::MapTile(x, y) => {
                if let Some(sel) = self.hotkeys.overworld.get(&chord).cloned() {
                    let idx = if sel == "Nothing" { Some(-1) } else { TILES.iter().position(|t| t.hotkey == sel).map(|i| i as i32) };
                    if let Some(i) = idx {
                        self.set_mark(x, y, i);
                        self.after_mark(x, y);
                        return true;
                    }
                }
                if self.m.circle[x][y] % 100 != 0 && !mods.any() {
                    let name = key.name();
                    if name.len() == 1 && name.chars().all(|c| c.is_ascii_alphanumeric()) {
                        let c = &mut self.m.circle[x][y];
                        *c = *c - *c % 100 + name.chars().next().unwrap().to_ascii_uppercase() as i32;
                        return true;
                    }
                }
            }
            Target::Box(id) => {
                if let Some(sel) = self.hotkeys.item.get(&chord).cloned() {
                    let idx = if sel == "Nothing" { Some(-1) } else { ITEM_HOTKEY_NAMES.iter().position(|n| *n == sel).map(|i| i as i32) };
                    if let Some(i) = idx {
                        if i < 0 || self.m.bx(id).cell == i || self.m.uses_of_item(i) < ITEM_MAX_COUNT[i as usize] {
                            self.m.bx_mut(id).cell = i;
                            return true;
                        }
                    }
                }
            }
            Target::Blocker(d, s) => {
                if let Some(sel) = self.hotkeys.blocker.get(&chord) {
                    if let Some(i) = BLOCKER_KINDS.iter().position(|k| k == sel) {
                        self.m.blockers[d][s].kind = i;
                        return true;
                    }
                }
            }
            Target::Room(x, y) => {
                if let Some(sel) = self.hotkeys.room.get(&chord).cloned() {
                    let room = &mut self.m.maps[self.m.tab].rooms[x][y];
                    if let Some(t) = sel.strip_prefix("RoomType_") {
                        if let Some(i) = ROOM_TYPES.iter().position(|r| *r == t) {
                            room.kind = i;
                            room.completed = i != 0;
                        }
                    } else if let Some(m) = sel.strip_prefix("MonsterDetail_") {
                        if let Some(i) = MONSTERS.iter().position(|r| *r == m) {
                            room.monster = i;
                        }
                    } else if let Some(f) = sel.strip_prefix("FloorDropDetail_") {
                        if let Some(i) = FLOOR_DROPS.iter().position(|r| *r == f) {
                            room.floor_drop = i;
                        }
                    }
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::Assets;
    use crate::model::Tracker;
    use crate::settings::{HotKeys, Settings};

    fn view() -> View {
        let mut v = View::new(Tracker::new(), Assets::default(), Settings::load(), HotKeys::default());
        v.m.started = true;
        v
    }

    #[test]
    fn marking_armos_asks_for_the_item_and_puts_it_on_the_timeline() {
        let mut v = view();
        v.m.seconds = 125;
        let order: Vec<i32> = (0..TILES.len() as i32).collect();
        // pick "Armos" in the mark picker for screen (12,1)
        v.choose(Action::Mark(12, 1, order), ARMOS_MARK as usize + 1, false);
        assert_eq!(v.m.mark[12][1], ARMOS_MARK);
        let pop = v.popup.take().expect("the item picker opens right away");
        assert!(matches!(pop.action, Action::Item(BoxId::Armos)));
        // pick the ladder (item 4)
        v.choose(pop.action, 4, false);
        assert_eq!(v.m.armos_box.cell, 4);
        assert_eq!(v.m.armos_box.has, Has::Yes);
        assert_eq!(v.m.timeline.get("ArmosBox"), Some(&125));
    }

    fn mark_take_any(v: &mut View, x: usize, y: usize, pick: usize) {
        let order: Vec<i32> = (0..TILES.len() as i32).collect();
        v.choose(Action::Mark(x, y, order), TAKE_ANY_MARK as usize + 1, false);
        let pop = v.popup.take().expect("the take-any menu opens");
        v.choose(pop.action, pick, false);
    }

    #[test]
    fn header_hearts_follow_the_take_any_tiles_on_the_map() {
        let mut v = view();
        // 0 potion, 1 candle, 2 heart, 3 leave
        mark_take_any(&mut v, 9, 1, 2); // row B: heart
        mark_take_any(&mut v, 4, 3, 0); // row D: potion
        assert_eq!(v.m.take_any_hearts, [1, 2, 0, 0]);
        assert!(v.m.timeline.contains_key("TakeAny1"));
        // change the selection on the first cave: heart -> candle
        mark_take_any(&mut v, 9, 1, 1);
        assert_eq!(v.m.take_any_hearts, [2, 2, 0, 0]);
        assert!(v.m.prog("PlayerHasBlueCandle"));
        assert!(!v.m.timeline.contains_key("TakeAny1"));
        // change it again to "leave": the candle came only from there, so it goes too
        mark_take_any(&mut v, 9, 1, 3);
        assert_eq!(v.m.take_any_hearts, [0, 2, 0, 0]);
        assert!(!v.m.prog("PlayerHasBlueCandle"));
        // a third cave in between (row C) moves the potion cave to heart 3
        mark_take_any(&mut v, 2, 2, 2);
        assert_eq!(v.m.take_any_hearts, [0, 1, 2, 0]);
        // removing the first cave shifts the others up and clears the last heart
        v.set_mark(9, 1, -1);
        assert_eq!(v.m.take_any_hearts, [1, 2, 0, 0]);
    }

    #[test]
    fn take_any_choices_are_saved() {
        let mut v = view();
        mark_take_any(&mut v, 9, 1, 2);
        let json = v.m.to_json();
        let mut t = crate::model::Tracker::new();
        t.load_json(&json).unwrap();
        assert_eq!(t.tile_choice[9][1], TileChoice::HEART);
        assert_eq!(t.take_any_hearts[0], 1);
    }

    #[test]
    fn wood_sword_cave_can_give_the_sword_or_be_left() {
        let mut v = view();
        let order: Vec<i32> = (0..TILES.len() as i32).collect();
        v.choose(Action::Mark(7, 7, order.clone()), WOOD_SWORD_CAVE_MARK as usize + 1, false);
        let pop = v.popup.take().expect("the take-this menu opens");
        v.choose(pop.action, 0, false); // wood sword
        assert!(v.m.prog("PlayerHasWoodSword"));
        assert!(v.m.timeline.contains_key("WoodSword"));
        v.choose(Action::TakeThis(7, 7), 2, false); // leave
        assert_eq!(v.m.extra[7][7], 0);
    }

    #[test]
    fn f16_only_takes_the_coast_item() {
        let mut v = view();
        let before = v.m.mark[COAST_X][COAST_Y];
        v.click_map_tile(COAST_X, COAST_Y, Btn::Left, false, Pos2::new(700.0, 330.0));
        let pop = v.popup.take().expect("the coast item picker opens");
        assert!(matches!(pop.action, Action::Item(BoxId::Ladder)));
        v.choose(pop.action, 5, false); // magic boomerang
        assert_eq!(v.m.ladder_box.cell, 5);
        assert!(v.m.ladder_box.is_done());
        assert_eq!(v.m.mark[COAST_X][COAST_Y], before, "F16 never gets an overworld mark");
        v.wheel_on_coast_for_test();
        assert_eq!(v.m.mark[COAST_X][COAST_Y], before);
    }

    #[test]
    fn coast_reminder_follows_the_original_rules() {
        let mut v = view();
        assert_eq!(v.coast_reminder_text(), None, "no ladder yet");
        v.m.dungeons[2].boxes[0].cell = 4; // ladder found in level 3
        v.m.dungeons[2].boxes[0].has = Has::Yes;
        assert_eq!(v.coast_reminder_text().as_deref(), Some("Get the coast item with the ladder"));
        v.m.ladder_box.cell = 13; // the coast holds the white sword
        assert_eq!(v.coast_reminder_text().as_deref(), Some("Get the white sword off the coast"));
        v.m.set_prog("PlayerHasMagicalSword", true);
        assert_eq!(v.coast_reminder_text(), None, "white sword no longer matters");
        v.m.ladder_box.cell = 7;
        v.m.ladder_box.has = Has::Yes;
        assert_eq!(v.coast_reminder_text(), None, "coast item taken");
    }

    #[test]
    fn timer_reads_like_the_original() {
        assert_eq!(crate::view::timer_text(0), " 0:00:00");
        assert_eq!(crate::view::timer_text(3725), " 1:02:05");
        assert_eq!(crate::view::timer_text(12 * 3600 + 59), "12:00:59");
        assert_eq!(crate::view::timer_text(25 * 3600), " 1:00:00");
    }

    #[test]
    fn other_marks_do_not_open_the_item_picker() {
        let mut v = view();
        let order: Vec<i32> = (0..TILES.len() as i32).collect();
        v.choose(Action::Mark(3, 3, order), 1, false); // level 1
        assert!(v.popup.is_none());
    }
}
