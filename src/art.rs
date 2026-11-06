//! The tracker's own pictures. Every picture is drawn here, at start-up, from the small pixel patterns and the
//! terrain table below: nothing is read from files or from the original program. Each sheet keeps the size and
//! order the drawing code expects (e.g. "icons7x7.png": 32 item pictures of 7x7), so the rest of the tracker just
//! asks for "picture n of sheet s".

use eframe::egui::{Color32, ColorImage};

/// A picture being drawn, pixel by pixel.
pub struct Canvas {
    w: usize,
    h: usize,
    px: Vec<Color32>,
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Self {
        Canvas { w, h, px: vec![Color32::TRANSPARENT; w * h] }
    }
    fn set(&mut self, x: i32, y: i32, c: Color32) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.px[y as usize * self.w + x as usize] = c;
        }
    }
    fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color32) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }
    fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color32) {
        self.fill(x, y, w, 1, c);
        self.fill(x, y + h - 1, w, 1, c);
        self.fill(x, y, 1, h, c);
        self.fill(x + w - 1, y, 1, h, c);
    }
    fn hline(&mut self, x0: i32, x1: i32, y: i32, c: Color32) {
        self.fill(x0, y, x1 - x0 + 1, 1, c);
    }
    fn vline(&mut self, x: i32, y0: i32, y1: i32, c: Color32) {
        self.fill(x, y0, 1, y1 - y0 + 1, c);
    }
    /// Draws a pixel pattern (one string per row, one letter per pixel, see `ink`), each pixel `scale` x `scale`.
    fn art(&mut self, x0: i32, y0: i32, rows: &[&str], scale: i32) {
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                if let Some(c) = ink(ch) {
                    self.fill(x0 + x as i32 * scale, y0 + y as i32 * scale, scale, scale, c);
                }
            }
        }
    }
    /// Writes text in the 3x5 letters below, one pixel apart.
    fn text(&mut self, x0: i32, y0: i32, s: &str, c: Color32) {
        let mut x = x0;
        for ch in s.chars() {
            if let Some(g) = glyph(ch) {
                for (y, row) in g.iter().enumerate() {
                    for (i, b) in row.bytes().enumerate() {
                        if b == b'#' {
                            self.set(x + i as i32, y0 + y as i32, c);
                        }
                    }
                }
            }
            x += 4;
        }
    }
    pub fn image(&self) -> ColorImage {
        let bytes: Vec<u8> = self.px.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect();
        ColorImage::from_rgba_unmultiplied([self.w, self.h], &bytes)
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// The colours of the pixel patterns.
fn ink(ch: char) -> Option<Color32> {
    Some(match ch {
        'k' => rgb(0, 0, 0),
        'w' => rgb(255, 255, 255),
        's' => rgb(200, 202, 212),
        'S' => rgb(128, 130, 142),
        'd' => rgb(68, 70, 80),
        'r' => rgb(226, 44, 44),
        'R' => rgb(140, 22, 22),
        'o' => rgb(242, 142, 34),
        'y' => rgb(250, 220, 60),
        'n' => rgb(156, 98, 46),
        'N' => rgb(92, 56, 26),
        't' => rgb(222, 192, 132),
        'p' => rgb(250, 190, 162),
        'P' => rgb(156, 84, 206),
        'b' => rgb(52, 104, 232),
        'B' => rgb(22, 42, 140),
        'c' => rgb(112, 212, 250),
        'g' => rgb(62, 190, 72),
        'G' => rgb(22, 112, 42),
        'l' => rgb(176, 232, 64),
        _ => return None,
    })
}

/// 3x5 letters and digits.
fn glyph(ch: char) -> Option<[&'static str; 5]> {
    Some(match ch {
        '0' | 'O' => ["###", "#.#", "#.#", "#.#", "###"],
        '1' => [".#.", "##.", ".#.", ".#.", "###"],
        '2' => ["###", "..#", "###", "#..", "###"],
        '3' => ["###", "..#", ".##", "..#", "###"],
        '4' => ["#.#", "#.#", "###", "..#", "..#"],
        '5' | 'S' => ["###", "#..", "###", "..#", "###"],
        '6' => ["###", "#..", "###", "#.#", "###"],
        '7' => ["###", "..#", ".#.", ".#.", ".#."],
        '8' => ["###", "#.#", "###", "#.#", "###"],
        '9' => ["###", "#.#", "###", "..#", "###"],
        'L' => ["#..", "#..", "#..", "#..", "###"],
        'F' => ["###", "#..", "##.", "#..", "#.."],
        'G' => ["###", "#..", "#.#", "#.#", "###"],
        'Z' => ["###", "..#", ".#.", "#..", "###"],
        'X' => ["#.#", "#.#", ".#.", "#.#", "#.#"],
        'I' => ["###", ".#.", ".#.", ".#.", "###"],
        'T' => ["###", ".#.", ".#.", ".#.", ".#."],
        '?' => ["###", "..#", ".##", "...", ".#."],
        '+' => ["...", ".#.", "###", ".#.", "..."],
        '-' => ["...", "...", "###", "...", "..."],
        _ => return None,
    })
}

// ------------------------------------------------------------------------------------------------ item pictures

/// "icons7x7.png": 0 wooden boomerang, 1 bow, 2 magic boomerang, 3 raft, 4 ladder, 5 recorder, 6 wand, 7 red candle,
/// 8 book, 9 key, 10 silver arrow, 11 wood arrow, 12 red ring, 13 magical shield, 14 boomerang-or-book,
/// 15 heart container, 16 power bracelet, 17 white sword, 18 armos statue, 19 wood sword, 20 magical sword,
/// 21 blue candle, 22 blue ring, 23 Ganon, 24 Zelda, 25 bomb, 26 bow and arrow, 27 meat, 28 unknown, 29 money,
/// 30 empty box, 31 compass.
const ITEMS7: [[&str; 7]; 32] = [
    [".oooo..", "on..o..", "o......", "o......", "on.....", ".o.....", "......."],
    ["..nnw..", ".n..w..", "n...w..", "n...w..", "n...w..", ".n..w..", "..nnw.."],
    [".bbbb..", "bc..b..", "b......", "b......", "bc.....", ".b.....", "......."],
    ["nnnnnnn", "NnNnNnN", "nnnnnnn", "NnNnNnN", "nnnnnnn", "NnNnNnN", "nnnnnnn"],
    ["t.....t", "ttttttt", "t.....t", "ttttttt", "t.....t", "ttttttt", "t.....t"],
    ["...t...", "..ttt..", "...k...", "...t...", "...k...", "...t...", "..ttt.."],
    ["...w...", "..wcw..", "...b...", "...b...", "...b...", "...b...", "...b..."],
    ["...y...", "...o...", "..rrr..", "..rwr..", "..rrr..", "..rrr..", ".rrrrr."],
    ["rrrrrr.", "rryyrrw", "rryyrrw", "rrrrrrw", "rrrrrrw", "rrrrrrw", "rrrrrr."],
    [".yyy...", "y...y..", ".yyy...", "..y....", "..yy...", "..y....", "..yy..."],
    ["...w...", "..www..", "...s...", "...s...", "...s...", "..sss..", "..s.s.."],
    ["...s...", "..sss..", "...n...", "...n...", "...n...", "..nnn..", "..n.n.."],
    [".......", "..rrr..", ".r...r.", ".r...r.", ".r...r.", "..rrr..", "......."],
    ["sssssss", "sbbybbs", "sbyyybs", "sbbybbs", ".sbbbs.", "..sbs..", "...s..."],
    [".......", "rrr.bbb", "ryr.b..", "rrr.b..", "rrr.b..", "rrr....", "......."],
    [".rr.rr.", "rwrrrrr", "rrrrrrr", "rrrrrrr", ".rrrrr.", "..rrr..", "...r..."],
    [".......", ".ooooo.", "oRRRRRo", "o.....o", "oRRRRRo", ".ooooo.", "......."],
    ["...w...", "...w...", "...w...", "...w...", ".bbbbb.", "...b...", "...b..."],
    [".ooooo.", "okoooko", "ooooooo", ".ooooo.", ".onono.", ".ooooo.", "oo...oo"],
    ["...t...", "...t...", "...t...", "...t...", ".nnnnn.", "...n...", "...n..."],
    ["...c...", "...c...", "...c...", "...c...", ".rrrrr.", "...r...", "...r..."],
    ["...y...", "...o...", "..bbb..", "..bwb..", "..bbb..", "..bbb..", ".bbbbb."],
    [".......", "..bbb..", ".b...b.", ".b...b.", ".b...b.", "..bbb..", "......."],
    ["b.....b", "bbbbbbb", "bbrbrbb", "bbbbbbb", ".bwbwb.", ".bbbbb.", "..b.b.."],
    ["..yyy..", ".yyyyy.", "yykpkyy", "yyppp.y", ".ypppy.", "..ppp..", ".rrrrr."],
    ["....y..", "...o...", "..bbb..", ".bbbbb.", ".bwbbb.", ".bbbbb.", "..bbb.."],
    [".nw....", "n.w....", "n.w..o.", "nwwwwoo", "n.w..o.", "n.w....", ".nw...."],
    ["...rrr.", "..rrrrr", "..rrrrr", "..rrrr.", ".ww....", "ww.....", "w......"],
    [".yyyyy.", "yy...yy", "....yy.", "...yy..", "...yy..", ".......", "...yy.."],
    ["...c...", "..cwc..", ".cwcbc.", ".cbcbc.", ".cbcbc.", "..cbc..", "...c..."],
    ["SSSSSSS", "S.....S", "S.....S", "S.....S", "S.....S", "S.....S", "SSSSSSS"],
    [".rrrrr.", "rwwrwwr", "rwwrwwr", "rwkkwwr", "rwwwwwr", "rwwwwwr", ".rrrrr."],
];

/// "icons3x7.png": the narrow shop pictures, 0 arrow, 1 bomb, 2 book, 3 candle, 4 blue ring, 5 meat, 6 key, 7 shield.
const SHOP3: [[&str; 7]; 8] = [
    [".w.", "www", ".n.", ".n.", ".n.", ".n.", "w.w"],
    ["..y", ".o.", "bbb", "bbb", "bwb", "bbb", ".b."],
    ["rrr", "ryr", "ryr", "rrr", "rrr", "rrr", "rrr"],
    [".y.", ".o.", "bbb", "bwb", "bbb", "bbb", "bbb"],
    ["...", ".b.", "b.b", "b.b", "b.b", ".b.", "..."],
    ["rrr", "rrr", "rrr", "rrr", ".w.", ".w.", "w.w"],
    ["yyy", "y.y", "yyy", ".y.", ".yy", ".y.", ".yy"],
    ["sss", "byb", "yyy", "byb", "bbb", "sbs", ".s."],
];

const HEART10: [&str; 9] = [
    ".xxx..xxx.", "xxxxxxxxxx", "xxxxxxxxxx", "xxxxxxxxxx", "xxxxxxxxxx", ".xxxxxxxx.", "..xxxxxx..", "...xxxx...", "....xx....",
];
const TRIANGLE10: [&str; 10] = [
    "....xx....", "....xx....", "...xxxx...", "...xxxx...", "..xxxxxx..", "..xxxxxx..", ".xxxxxxxx.", ".xxxxxxxx.", "xxxxxxxxxx", "xxxxxxxxxx",
];

fn shape(c: &mut Canvas, x0: i32, y0: i32, rows: &[&str], col: Color32) {
    for (y, row) in rows.iter().enumerate() {
        for (x, b) in row.bytes().enumerate() {
            if b == b'x' {
                c.set(x0 + x as i32, y0 + y as i32, col);
            }
        }
    }
}

/// "icons10x10.png": 0 triforce (missing), 1 triforce, 2 Take Any heart skipped, 3 Take Any heart unknown,
/// 4 Take Any heart taken, 5 arrow, 6 tick, 7 starting items, 8 save disk, 9 big heart.
fn icons10() -> Canvas {
    let mut c = Canvas::new(100, 10);
    shape(&mut c, 0, 0, &TRIANGLE10, rgb(90, 90, 96));
    shape(&mut c, 10, 0, &TRIANGLE10, rgb(250, 168, 30));
    shape(&mut c, 21, 1, &HEART10[..], rgb(110, 110, 118));
    for i in 0..8 {
        c.set(21 + i + 1, 1 + i, rgb(250, 220, 60));
        c.set(21 + 8 - i, 1 + i, rgb(250, 220, 60));
    }
    shape(&mut c, 30, 1, &HEART10, rgb(70, 70, 76));
    shape(&mut c, 40, 1, &HEART10, rgb(226, 44, 44));
    c.set(42, 2, rgb(255, 255, 255));
    c.art(50, 1, &["....y....", "....yy...", "yyyyyyy..", "yyyyyyyy.", "yyyyyyy..", "....yy...", "....y...."], 1);
    c.art(60, 1, &[".........g", "........gg", ".......gg.", "g.....gg..", "gg...gg...", ".gg.gg....", "..ggg.....", "...g......"], 1);
    c.fill(70, 2, 10, 6, rgb(60, 80, 200));
    for x in [72, 74, 76] {
        c.fill(x + 1, 5, 1, 1, rgb(255, 255, 255));
    }
    c.fill(80, 0, 10, 10, rgb(30, 30, 40));
    c.fill(82, 0, 6, 4, rgb(200, 202, 212));
    c.fill(86, 1, 1, 2, rgb(30, 30, 40));
    c.fill(81, 6, 8, 4, rgb(240, 240, 240));
    shape(&mut c, 90, 0, &HEART10, rgb(226, 44, 44));
    c
}

// ------------------------------------------------------------------------------------------------ monsters

/// "zelda_bosses16x16.png": 8x8 patterns drawn twice the size. Order: 0 Digdogger, 1 Gleeok, 2 Gohma, 3 Manhandla,
/// 4 Wizzrobe, 5 Patra, 6 Dodongo, 7 red bubble, 8 blue bubble, 9 Darknut, 10 other, 11 (unused), 12 Vire, 13 Zol,
/// 14 Pols Voice, 15 Tektite, 16 Goriya, 17 Rope, 18 Stalfos, 19 Wallmaster, 20 Gel, 21 Keese, 22 Like-like, 23 Gibdo,
/// 24 Lynel, 25 Moblin, 26 Aquamentus, 27 Lanmola, 28 Moldorm, 29 rupee boss, 30 traps, 31 other (2).
const MONSTERS8: [[&str; 8]; 32] = [
    ["..BBBB..", ".BBBBBB.", "BBwwwwBB", "BBwkkwBB", "BBwwwwBB", ".BBBBBB.", "..B..B..", ".BB..BB."],
    ["gg....gg", "gkg..gkg", ".g....g.", "..g..g..", "...gg...", "..gggg..", ".gGggGg.", ".gg..gg."],
    ["o.o..o.o", ".oooooo.", "oowwwwoo", "oowkkwoo", ".oooooo.", "o.o..o.o", "o......o", "........"],
    ["...gg...", "...gg...", ".r.gg.r.", "gggrrggg", "gggrrggg", ".r.gg.r.", "...gg...", "...gg..."],
    ["...bb...", "..bbbb..", "..pkpk..", "..bbbb..", ".bbwbbb.", ".bbwbbb.", "bbbbbbbb", "bb.bb.bb"],
    ["..P..P..", "P......P", "...PP...", "..PwwP..", "..PkkP..", "...PP...", "P......P", "..P..P.."],
    ["........", ".....tt.", ".tttttkt", "tttttttt", "tttttttw", ".tt..tt.", ".tt..tt.", "........"],
    [".rrrrrr.", "rwwwwwwr", "rwkwwkwr", "rwwwwwwr", ".rwkkwr.", "..rwwr..", "..r..r..", "........"],
    [".bbbbbb.", "bwwwwwwb", "bwkwwkwb", "bwwwwwwb", ".bwkkwb.", "..bwwb..", "..b..b..", "........"],
    ["..bbbb..", ".bkbbkb.", ".bbbbbb.", "sbbbbbbs", "sbbwwbbs", "sbbbbbbs", ".bb..bb.", ".bb..bb."],
    ["..wwww..", ".ww..ww.", ".....ww.", "....ww..", "...ww...", "...ww...", "........", "...ww..."],
    ["........", "........", "........", "........", "........", "........", "........", "........"],
    ["b.b..b.b", "bbbbbbbb", ".bkbbkb.", "..bbbb..", ".brbbrb.", "..bbbb..", "...bb...", "..b..b.."],
    ["........", "...gg...", "..gggg..", ".ggkgkg.", ".gggggg.", "gggggggg", "gggggggg", "........"],
    [".w....w.", ".w....w.", ".ww..ww.", ".wwwwww.", "wwkwwkww", "wwwppwww", ".wwwwww.", ".w....w."],
    ["........", "..rrrr..", ".rrwwrr.", ".rrkkrr.", "r.rrrr.r", "r......r", ".r....r.", "........"],
    [".r....r.", "rrr..rrr", ".rrrrrr.", ".rkrrkr.", ".rrrrrr.", "..rwwr..", ".rrrrrr.", ".r.rr.r."],
    ["........", "....ooo.", "...okoo.", "...ooo..", "..oo....", ".oo.....", ".oooooo.", "..oooo.."],
    ["..wwww..", ".wkwwkw.", ".wwwwww.", "..wkkw..", ".wwwwww.", "...ww...", "..w..w..", ".ww..ww."],
    [".G.G.G..", ".G.G.G..", ".GGGGG.G", ".GGGGGG.", ".GGGGG..", ".GGGGG..", "..GGG...", "........"],
    ["........", "........", "........", "...cc...", "..cccc..", ".cckkcc.", ".cccccc.", "........"],
    ["........", "b......b", "bb.bb.bb", ".bbkkbb.", "..bbbb..", "...bb...", "........", "........"],
    ["..oooo..", ".oRRRRo.", ".orrrro.", ".oRRRRo.", ".orrrro.", ".oRRRRo.", ".orrrro.", "..oooo.."],
    ["..tttt..", ".tkttkt.", ".nnnnnn.", ".tttttt.", ".nnnnnn.", ".tttttt.", ".tt..tt.", ".nn..nn."],
    [".rr...s.", "rkrr..s.", "rrrr..s.", ".rrrrrs.", ".rrrrrr.", ".rrrrr..", ".r.r.r..", ".r.r.r.."],
    [".b....b.", ".bbbbbb.", ".bkbbkb.", ".bbppbb.", "..bppb..", ".bbbbbb.", ".bb..bb.", ".bb..bb."],
    ["......w.", "....ggg.", "...ggkg.", "..gggg..", ".ggggg..", "gggggg..", ".g.g.g..", ".g.g.g.."],
    [".bb.....", "bbbb....", ".bbbb...", "..bbbb..", "...bbbb.", "....bbbb", ".....bkb", "......b."],
    ["oo......", "oo.oo...", "...oo.oo", "......oo", "....oo..", "....oo..", "........", "........"],
    ["...gg...", "..gllg..", ".gllggg.", ".glgggg.", ".glgggg.", ".gggggg.", "..gggg..", "...gg..."],
    ["s......s", ".s.ss.s.", "..ssss..", ".sskkss.", ".sskkss.", "..ssss..", ".s.ss.s.", "s......s"],
    ["..yyyy..", ".yy..yy.", ".....yy.", "....yy..", "...yy...", "...yy...", "........", "...yy..."],
];

/// "zelda_items16x16.png": what lies on a dungeon room's floor. 0 triforce, 1 heart container, 2 bombs, 3 key,
/// 4 rupee, 5 map, 6 compass, 7 another key item (8-12 unused).
const FLOOR8: [[&str; 8]; 8] = [
    ["........", "...yy...", "...yy...", "..yyyy..", "..yyyy..", ".yyyyyy.", ".yyyyyy.", "yyyyyyyy"],
    [".rr..rr.", "rwrrrrrr", "rrrrrrrr", "rrrrrrrr", ".rrrrrr.", "..rrrr..", "...rr...", "........"],
    [".....y..", "....o...", "..bbbb..", ".bbbbbb.", ".bwbbbb.", ".bbbbbb.", "..bbbb..", "........"],
    [".yyy....", "y...y...", "y...y...", ".yyy....", "..y.....", "..yyy...", "..y.....", "..yy...."],
    ["...cc...", "..cwwc..", ".cwccbc.", ".cwcbbc.", ".ccbbbc.", ".ccbbbc.", "..cbbc..", "...cc..."],
    [".tttttt.", "tnttttnt", ".tttttt.", ".tnnntt.", ".tttttt.", ".ttnntt.", "tnttttnt", ".tttttt."],
    ["..rrrr..", ".rwwwwr.", "rwwwrwwr", "rwwrrwwr", "rwwkwwwr", "rwwwwwwr", ".rwwwwr.", "..rrrr.."],
    ["........", ".wwwwww.", "...ww...", "...ww...", "...ww...", "...ww...", ".wwwwww.", "........"],
];

// ------------------------------------------------------------------------------------------------ map markers

/// The colour of each dungeon level's marker (levels 1-9).
const LEVEL_COLORS: [Color32; 9] = [
    rgb(90, 220, 240), rgb(130, 140, 250), rgb(70, 200, 150), rgb(240, 220, 40), rgb(140, 210, 60),
    rgb(250, 190, 50), rgb(120, 220, 90), rgb(170, 170, 180), rgb(190, 190, 200),
];
const SHOP_COLOR: Color32 = rgb(239, 131, 0);

fn darker(c: Color32, f: f32) -> Color32 {
    rgb((c.r() as f32 * f) as u8, (c.g() as f32 * f) as u8, (c.b() as f32 * f) as u8)
}

/// A 16x11 marker: a coloured box with a darker edge.
fn marker_box(c: &mut Canvas, i: i32, bg: Color32) -> i32 {
    let x0 = i * 16;
    c.fill(x0, 0, 16, 11, bg);
    c.frame(x0, 0, 16, 11, darker(bg, 0.55));
    x0
}

fn item7_at(c: &mut Canvas, x: i32, y: i32, i: usize) {
    c.art(x, y, &ITEMS7[i], 1);
}

/// "s_icon_overworld_strip39.png": the map markers. 0 nothing, 1 unknown level, 2-10 levels 1-9, 11-14 any roads
/// 1-4, 15 Take Any, 16 money game, 17 letter, 18 Armos, 19-21 sword caves (magical, white, wood), 22 hint,
/// 23-31 shops (shield, blue ring, meat, key, heart, candle, book, bomb, arrow), 32 potion shop, 33 door repair,
/// 34-37 secrets (unknown, large, medium, small), 38 "don't care".
fn markers() -> Canvas {
    let mut c = Canvas::new(39 * 16, 11);
    let black = rgb(0, 0, 0);
    let white = rgb(255, 255, 255);
    marker_box(&mut c, 0, rgb(40, 20, 50));
    let x = marker_box(&mut c, 1, rgb(150, 150, 160));
    c.text(x + 4, 3, "L?", black);
    for l in 0..9 {
        let x = marker_box(&mut c, 2 + l, LEVEL_COLORS[l as usize]);
        c.text(x + 4, 3, &format!("L{}", l + 1), black);
    }
    for r in 0..4 {
        let x = marker_box(&mut c, 11 + r, rgb(228, 150, 236));
        c.text(x + 3, 3, &format!("{}", r + 1), black);
        // stairs going down
        for s in 0..3 {
            c.fill(x + 8 + s * 2, 3 + s * 2, 6 - s * 2, 2, rgb(60, 40, 120));
        }
    }
    let x = marker_box(&mut c, 15, rgb(190, 30, 40));
    item7_at(&mut c, x + 1, 2, 15);
    c.text(x + 10, 3, "?", white);
    let x = marker_box(&mut c, 16, rgb(120, 200, 110));
    // a plus-minus sign
    c.hline(x + 2, x + 6, 4, black);
    c.vline(x + 4, 2, 6, black);
    c.hline(x + 2, x + 6, 8, black);
    item7_at(&mut c, x + 8, 2, 29);
    let x = marker_box(&mut c, 17, rgb(200, 220, 240));
    c.fill(x + 3, 3, 10, 6, white);
    c.frame(x + 3, 3, 10, 6, rgb(60, 80, 140));
    for i in 0..5 {
        c.set(x + 4 + i, 4 + i / 2, rgb(60, 80, 140));
        c.set(x + 11 - i, 4 + i / 2, rgb(60, 80, 140));
    }
    let x = marker_box(&mut c, 18, rgb(220, 170, 110));
    item7_at(&mut c, x + 4, 2, 18);
    for (n, (sword, bg)) in [(20, rgb(230, 110, 110)), (17, rgb(110, 150, 240)), (19, rgb(200, 170, 110))].into_iter().enumerate() {
        let x = marker_box(&mut c, 19 + n as i32, bg);
        c.text(x + 2, 3, &format!("{}", 3 - n), black);
        item7_at(&mut c, x + 7, 2, sword);
    }
    let x = marker_box(&mut c, 22, rgb(160, 130, 220));
    c.text(x + 2, 3, "???", black);
    for (n, item) in [13, 22, 27, 9, 15, 21, 8, 25, 11].into_iter().enumerate() {
        let x = marker_box(&mut c, 23 + n as i32, SHOP_COLOR);
        item7_at(&mut c, x + 4, 2, item);
    }
    let x = marker_box(&mut c, 32, rgb(200, 230, 250));
    // a potion bottle
    c.art(x + 2, 2, &["..s..", ".sss.", ".srs.", "srrrs", "srrrs", "srrrs", ".sss."], 1);
    c.art(x + 9, 2, &["..s..", ".sss.", ".sbs.", "sbbbs", "sbbbs", "sbbbs", ".sss."], 1);
    let x = marker_box(&mut c, 33, rgb(230, 230, 180));
    c.text(x + 2, 3, "-", black);
    item7_at(&mut c, x + 7, 2, 29);
    let secrets: [(&str, Option<char>); 4] = [("?", None), ("+", Some('r')), ("+", Some('b')), ("+", Some('g'))];
    for (n, (sign, rupee)) in secrets.into_iter().enumerate() {
        let x = marker_box(&mut c, 34 + n as i32, rgb(230, 230, 180));
        c.text(x + 2, 3, sign, black);
        let rows = ITEMS7[29];
        let colour = rupee.and_then(ink).unwrap_or(rgb(130, 130, 140));
        for (yy, row) in rows.iter().enumerate() {
            for (xx, ch) in row.chars().enumerate() {
                if ch != '.' {
                    c.set(x + 7 + xx as i32, 2 + yy as i32, if ch == 'w' { white } else { colour });
                }
            }
        }
    }
    let x = marker_box(&mut c, 38, rgb(40, 40, 46));
    for i in 0..7 {
        c.set(x + 5 + i, 2 + i, rgb(150, 150, 160));
        c.set(x + 11 - i, 2 + i, rgb(150, 150, 160));
    }
    c
}

/// "s_btn_tr_dungeon_num_strip18.png": the 9x9 level numbers, 0-8 = level not found yet, 9-17 = found.
fn dungeon_numbers() -> Canvas {
    let mut c = Canvas::new(18 * 9, 9);
    for l in 0..9 {
        let col = LEVEL_COLORS[l];
        let x = l as i32 * 9;
        c.fill(x, 0, 9, 9, rgb(16, 16, 22));
        c.frame(x, 0, 9, 9, darker(col, 0.5));
        c.text(x + 3, 2, &format!("{}", l + 1), col);
        let x = x + 81;
        c.fill(x, 0, 9, 9, col);
        c.frame(x, 0, 9, 9, darker(col, 0.5));
        c.text(x + 3, 2, &format!("{}", l + 1), rgb(0, 0, 0));
    }
    c
}

// ------------------------------------------------------------------------------------------------ dungeon rooms

/// "new_icons13x9.png": room pictures, top row = not finished, bottom row = finished. 0 unmarked, 1 plain room,
/// 2 double moat, 3 Chevy, 4 right moat, 5 top moat, 6 circle moat, 7 vertical chute, 8 horizontal chute, 9 Tee,
/// 10 push block?, 11 item basement, 12 old man hint, 13 meat block, 14 life or money, 15 bomb upgrade,
/// 16 turnstile, 17-24 transports 1-8, 25 staircase, 26-29 start (entered from W, N, S, E), 30 off the map,
/// 31 Ganon, 32 Zelda, 33 lava moat, 34-41 numbers 1-8.
fn rooms() -> Canvas {
    let mut c = Canvas::new(42 * 13, 18);
    for done in [false, true] {
        let y0 = if done { 9 } else { 0 };
        let floor = if done { rgb(66, 66, 74) } else { rgb(132, 132, 140) };
        let wall = if done { rgb(36, 36, 42) } else { rgb(78, 78, 88) };
        let water = if done { rgb(40, 70, 130) } else { rgb(52, 120, 240) };
        let line = if done { rgb(30, 30, 34) } else { rgb(50, 50, 58) };
        let special = rgb(200, 40, 40);
        for i in 0..42 {
            let x0 = i * 13;
            let room = |c: &mut Canvas| {
                c.fill(x0, y0, 13, 9, floor);
                c.frame(x0, y0, 13, 9, wall);
            };
            match i {
                0 => {
                    c.fill(x0, y0, 13, 9, rgb(14, 14, 34));
                    c.frame(x0, y0, 13, 9, rgb(52, 52, 92));
                }
                1 => room(&mut c),
                2 => {
                    room(&mut c);
                    c.hline(x0 + 1, x0 + 11, y0 + 2, water);
                    c.hline(x0 + 1, x0 + 11, y0 + 6, water);
                }
                3 => {
                    room(&mut c);
                    c.frame(x0 + 3, y0 + 2, 7, 5, water);
                    c.set(x0 + 6, y0 + 2, floor);
                    c.set(x0 + 6, y0 + 6, floor);
                }
                4 => {
                    room(&mut c);
                    c.vline(x0 + 9, y0 + 1, y0 + 7, water);
                }
                5 => {
                    room(&mut c);
                    c.hline(x0 + 1, x0 + 11, y0 + 3, water);
                }
                6 => {
                    room(&mut c);
                    c.frame(x0 + 3, y0 + 2, 7, 5, water);
                }
                7 => {
                    room(&mut c);
                    c.vline(x0 + 4, y0 + 1, y0 + 7, line);
                    c.vline(x0 + 8, y0 + 1, y0 + 7, line);
                }
                8 => {
                    room(&mut c);
                    c.hline(x0 + 1, x0 + 11, y0 + 3, line);
                    c.hline(x0 + 1, x0 + 11, y0 + 5, line);
                }
                9 => {
                    room(&mut c);
                    c.hline(x0 + 2, x0 + 10, y0 + 2, water);
                    c.vline(x0 + 6, y0 + 2, y0 + 7, water);
                }
                10 => {
                    room(&mut c);
                    for (dx, dy) in [(3, 2), (6, 2), (9, 2), (3, 6), (6, 6), (9, 6)] {
                        c.set(x0 + dx, y0 + dy, line);
                    }
                    c.fill(x0 + 5, y0 + 3, 3, 3, line);
                }
                11 => {
                    room(&mut c);
                    c.text(x0 + 2, y0 + 2, "I", rgb(250, 220, 60));
                    for s in 0..3 {
                        c.fill(x0 + 6 + s, y0 + 3 + s, 4 - s, 1, line);
                    }
                }
                12..=15 => {
                    room(&mut c);
                    c.frame(x0, y0, 13, 9, if done { darker(special, 0.5) } else { special });
                    let pattern: &[&str] = match i {
                        12 => &["..r..", ".rpr.", "..p..", ".rrr.", ".r.r."],
                        13 => &[".rrr.", "rrrrr", ".rrr.", "..w..", ".w..."],
                        14 => &["rr.rr", "rrrrr", ".rrr.", "..r..", "....."],
                        _ => &["...y.", "..o..", ".bbb.", "bbbbb", ".bbb."],
                    };
                    c.art(x0 + 4, y0 + 2, pattern, 1);
                }
                16 => {
                    room(&mut c);
                    c.hline(x0 + 3, x0 + 9, y0 + 4, line);
                    c.vline(x0 + 6, y0 + 1, y0 + 7, line);
                }
                17..=24 | 34..=41 => {
                    room(&mut c);
                    let n = if i >= 34 { i - 33 } else { i - 16 };
                    let col = [rgb(250, 60, 60), rgb(80, 200, 250), rgb(250, 130, 40), rgb(60, 220, 80), rgb(240, 70, 220), rgb(250, 250, 160), rgb(180, 240, 60), rgb(250, 200, 60)][(n - 1) as usize];
                    c.fill(x0 + 4, y0 + 1, 5, 7, rgb(0, 0, 0));
                    c.text(x0 + 5, y0 + 2, &format!("{n}"), if done { darker(col, 0.7) } else { col });
                }
                25 => {
                    room(&mut c);
                    for s in 0..4 {
                        c.fill(x0 + 3 + s * 2, y0 + 2 + s, 8 - s * 2, 1, water);
                    }
                }
                26..=29 => {
                    room(&mut c);
                    let g = rgb(40, 200, 60);
                    let rows: [&str; 5] = match i {
                        26 => ["..x..", "...x.", "xxxxx", "...x.", "..x.."],
                        27 => ["..x..", "..x..", "x.x.x", ".xxx.", "..x.."],
                        28 => ["..x..", ".xxx.", "x.x.x", "..x..", "..x.."],
                        _ => ["..x..", ".x...", "xxxxx", ".x...", "..x.."],
                    };
                    shape(&mut c, x0 + 4, y0 + 2, &rows, if done { darker(g, 0.6) } else { g });
                }
                30 => {
                    c.fill(x0, y0, 13, 9, rgb(0, 0, 0));
                    c.text(x0 + 1, y0 + 2, "OFF", if done { rgb(130, 20, 20) } else { rgb(240, 40, 40) });
                }
                31 | 32 => {
                    room(&mut c);
                    c.text(x0 + 5, y0 + 2, if i == 31 { "G" } else { "Z" }, if i == 31 { rgb(80, 200, 250) } else { rgb(240, 60, 60) });
                }
                33 => {
                    room(&mut c);
                    let lava = if done { rgb(130, 60, 20) } else { rgb(250, 110, 30) };
                    c.vline(x0 + 3, y0 + 2, y0 + 6, lava);
                    c.vline(x0 + 9, y0 + 2, y0 + 6, lava);
                    c.hline(x0 + 3, x0 + 9, y0 + 2, lava);
                }
                _ => room(&mut c),
            }
        }
    }
    c
}

// ------------------------------------------------------------------------------------------------ overworld map

/// What covers each part of the overworld: 16 x 8 screens, each split into 4 x 3 blocks (64 letters per row,
/// 24 rows). R rock, W water, F forest, G graveyard, S sand, O open ground. The four overworlds (first and second
/// quest, and the two mixed ones) share this layout; the tracker's markers show where the caves are.
const TERRAIN: [&str; 24] = [
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRORRRRRWWWWRRRR",
    "RRRRSSSSRRSSSRRRRSSSSRRSRSSRRRRRRSSRSSSSOWWOOOOORROOOOORRRRWOOOO",
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRORRRRRRRRRRRRRWRROR",
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRORRRORRRRRRRRRORRRR",
    "SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSOOOOOOOOOOOOOOROOOOOOOOOOOOOOOOOOO",
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRORRRRRRRRRRRRRRORRRRRRORWWWWRRRRRR",
    "OOOOOOOGRRRRRRRRRRGRGRGRRRRRRRORRRRRRRRRRRRRRRRRSRRRROWWWWWWOOOO",
    "OGGGGGGGRGGRGGRGGGGGGGGGOOOOSWOOSSSSSSSSSSSSSSSSSRRSOOOOOOWWGGGO",
    "OGGGGGGGRRRRRRRRRRRRRRRRRRWWWWWWWWSRRRRRSSSSSSSSRRRRROOROOWWOOOO",
    "OGGGGGGGRRRRRGGRRRRRRRRRRRWWRRRRWWORRRRRRSSSSSSSFFFFFOOFROWWWWWW",
    "OGGGGGGGGGGRGGGGOOOWROOOOOWWOOOOWWOOOWWORSSSSSSSOOOOOOOOROOOOOWW",
    "OGGGGGGGRRRRRRRRRRWWWWWWWWWWRRRRWWRRROORRSSRRRRSFOFFOOOORRRRROWW",
    "OGGGGGGGRRRRRRRRROWWRRRRWWWWWWWWWWOFFFFFFSSFFFFSFFFFOOOOFFFFROWW",
    "OGGGGGGGOOWOOWWOOOWWOOOOWWOOOOOOOOOFFSSSSSSSSSFSOOOFOOOOOOOOROWW",
    "OOOOOOOGROORROORROWWRRORWWFFFFFFFOOFFFFFFFFFSSFSFFFFFFFFFFFFROWW",
    "OOOORRRRRRRRROORFOWWWWWWWWOFFFFFFOOFFFFFFFFFOOOOFFFFFFFFFOFFROWW",
    "GGGGOOOOORRRROOOOOOOOWOOOOOOOOOOOOOOOOOOOOOOOOOOFFFOOOOOOOOOROWW",
    "GGGGRRORROOOROORFOOFFOOFFFFFFFFFOOOOFFWWWWOOOOOOFFFFFFFFFFFFROWW",
    "GGGGRRRRROOOROORFOOOOOOOFFFFFFFFFOOOFOWWWWOOOOOFFFFFFFFFFFFFOOWW",
    "GGGGOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOFFFFOOOOOOOWW",
    "RRRRRRRRRRRRROORFFFFFWFFFFFOFOFFFOOOFFFFFFFFFFOOFFFFFFFFFFFFROWW",
    "RRRRROORRRRRRRORFFFFFWFFFOFOFOFFFOOORRRRRRRRRROORORRRORRRRRRROWW",
    "ROOOOOOOORRRRROOOOOOOOOOOOOOOOOFOOOOOOORROORROOOOOOOOOOOOOOOOOWW",
    "RRRRRRRRRRRRRRRRFFFFFWFFFFFFFFFFFFFFRRRRRRRRRRWWWWWWWWWWWWWWWWWW",
];

/// One overworld pixel, from the ground kind with a little texture of its own.
fn ground(kind: u8, x: i32, y: i32) -> Color32 {
    match kind {
        b'R' => if (x + 2 * y) % 5 == 0 { rgb(104, 66, 36) } else { rgb(138, 92, 50) },
        b'W' => if (x + 3 * y) % 7 == 0 { rgb(90, 140, 220) } else { rgb(44, 86, 170) },
        b'F' => if x % 3 == 1 && y % 3 == 1 { rgb(24, 74, 30) } else { rgb(58, 120, 56) },
        b'G' => if x % 4 == 1 && y % 3 == 0 { rgb(210, 210, 216) } else { rgb(104, 104, 112) },
        b'S' => if (x * 3 + y) % 7 == 0 { rgb(180, 150, 90) } else { rgb(212, 182, 118) },
        _ => if (x + y) % 9 == 0 { rgb(168, 160, 132) } else { rgb(196, 184, 150) },
    }
}

/// "s_map_overworld_strip8.png": four overworlds side by side, each 16 x 8 screens of 16 x 11 pixels.
fn overworld() -> Canvas {
    const BLOCK_X: [i32; 5] = [0, 4, 8, 12, 16];
    const BLOCK_Y: [i32; 4] = [0, 4, 8, 11];
    let mut c = Canvas::new(1024, 88);
    for q in 0..4 {
        for (row, line) in TERRAIN.iter().enumerate() {
            let (sy, by) = (row / 3, row % 3);
            for (col, kind) in line.bytes().enumerate() {
                let (sx, bx) = (col / 4, col % 4);
                for y in sy as i32 * 11 + BLOCK_Y[by]..sy as i32 * 11 + BLOCK_Y[by + 1] {
                    for x in sx as i32 * 16 + BLOCK_X[bx]..sx as i32 * 16 + BLOCK_X[bx + 1] {
                        c.set(q * 256 + x, y, ground(kind, x, y));
                    }
                }
            }
        }
    }
    c
}

// ------------------------------------------------------------------------------------------------ logo

/// "ZTlogo64x64.png", also the window icon: a gold triforce on a dark blue tile, with "ZT" under it.
pub fn logo() -> Canvas {
    let mut c = Canvas::new(64, 64);
    let edge = rgb(250, 200, 60);
    for y in 0..64 {
        for x in 0..64 {
            // rounded corners
            let (dx, dy) = ((x as f32 - 31.5).abs() - 24.0, (y as f32 - 31.5).abs() - 24.0);
            let outside = dx > 0.0 && dy > 0.0 && dx * dx + dy * dy > 64.0;
            if !outside {
                let shade = 30 + (y / 4) as u8;
                c.set(x, y, rgb(14, 20, shade + 40));
            }
        }
    }
    // three triangles, each 18 wide and 16 tall
    let tri = |c: &mut Canvas, x0: i32, y0: i32| {
        for row in 0..16 {
            let half = (row + 2) / 2 + row / 2;
            for x in 9 - half / 2 - 1..9 + half / 2 + 1 {
                c.set(x0 + x, y0 + row, if row == 15 { rgb(200, 140, 20) } else { edge });
            }
        }
    };
    tri(&mut c, 23, 6);
    tri(&mut c, 14, 22);
    tri(&mut c, 32, 22);
    // "ZT" in letters three times the size
    let white = rgb(255, 255, 255);
    for (n, ch) in ['Z', 'T'].into_iter().enumerate() {
        if let Some(g) = glyph(ch) {
            for (y, row) in g.iter().enumerate() {
                for (x, b) in row.bytes().enumerate() {
                    if b == b'#' {
                        c.fill(19 + n as i32 * 14 + x as i32 * 3, 43 + y as i32 * 3, 3, 3, white);
                    }
                }
            }
        }
    }
    c
}

// ------------------------------------------------------------------------------------------------ all sheets

/// The original's grey version of a picture (`Graphics::greyscale`, token 0x06002b03, RVA 0x9a278, pixel rule
/// `Graphics/greyscale@248::Invoke`, token 0x06002c7b, RVA 0x9cd28): s = (R + G + B) / 5 (whole numbers), grey level
/// 0 if s is 0, else s + 60; see-through pixels stay see-through.
pub fn grey_level(c: Color32) -> u8 {
    let s = (c.r() as u32 + c.g() as u32 + c.b() as u32) / 5;
    if s == 0 { 0 } else { (s + 60) as u8 }
}

fn greyscale(img: &ColorImage) -> ColorImage {
    let mut out = img.clone();
    for px in out.pixels.iter_mut() {
        if px.a() > 0 {
            let v = grey_level(*px);
            *px = rgb(v, v, v);
        }
    }
    out
}

/// The item pictures in grey, for items not had yet in the progress row.
pub const GREY_ITEMS: &str = "icons7x7 grey";

/// Every picture sheet, by the name the drawing code asks for.
pub fn sheets() -> Vec<(&'static str, ColorImage)> {
    let mut icons7 = Canvas::new(32 * 7, 7);
    for (i, rows) in ITEMS7.iter().enumerate() {
        icons7.art(i as i32 * 7, 0, rows, 1);
    }
    let mut shop3 = Canvas::new(8 * 3, 7);
    for (i, rows) in SHOP3.iter().enumerate() {
        shop3.art(i as i32 * 3, 0, rows, 1);
    }
    let mut heart8 = Canvas::new(16, 8);
    shape(&mut heart8, 0, 0, &[".xx.xx..", "xxxxxxx.", "xxxxxxx.", "xxxxxxx.", ".xxxxx..", "..xxx...", "...x...."], rgb(226, 44, 44));
    heart8.set(1, 1, rgb(255, 255, 255));
    let mut monsters = Canvas::new(32 * 16, 16);
    for (i, rows) in MONSTERS8.iter().enumerate() {
        monsters.art(i as i32 * 16, 0, rows, 2);
    }
    let mut floor = Canvas::new(13 * 16, 16);
    for (i, rows) in FLOOR8.iter().enumerate() {
        floor.art(i as i32 * 16, 0, rows, 2);
    }
    vec![
        ("icons7x7.png", icons7.image()),
        (GREY_ITEMS, greyscale(&icons7.image())),
        ("icons3x7.png", shop3.image()),
        ("icons8x8.png", heart8.image()),
        ("icons10x10.png", icons10().image()),
        ("zelda_bosses16x16.png", monsters.image()),
        ("zelda_items16x16.png", floor.image()),
        ("s_icon_overworld_strip39.png", markers().image()),
        ("s_btn_tr_dungeon_num_strip18.png", dungeon_numbers().image()),
        ("new_icons13x9.png", rooms().image()),
        ("s_map_overworld_strip8.png", overworld().image()),
        ("ZTlogo64x64.png", logo().image()),
    ]
}

/// The window, taskbar and Dock icon: the logo.
pub fn window_icon() -> eframe::egui::IconData {
    let c = logo();
    let rgba = c.px.iter().flat_map(|p| p.to_srgba_unmultiplied()).collect();
    eframe::egui::IconData { width: c.w as u32, height: c.h as u32, rgba }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_sheet_has_the_size_the_drawing_code_expects() {
        let want = [
            ("icons7x7.png", [224, 7]),
            ("icons3x7.png", [24, 7]),
            ("icons10x10.png", [100, 10]),
            ("zelda_bosses16x16.png", [512, 16]),
            ("zelda_items16x16.png", [208, 16]),
            ("s_icon_overworld_strip39.png", [624, 11]),
            ("s_btn_tr_dungeon_num_strip18.png", [162, 9]),
            ("new_icons13x9.png", [546, 18]),
            ("s_map_overworld_strip8.png", [1024, 88]),
            ("ZTlogo64x64.png", [64, 64]),
        ];
        let sheets = super::sheets();
        for (name, size) in want {
            let s = sheets.iter().find(|s| s.0 == name).unwrap_or_else(|| panic!("{name} missing"));
            assert_eq!(s.1.size, size, "{name}");
        }
    }

    #[test]
    fn grey_follows_the_originals_formula() {
        use eframe::egui::Color32;
        assert_eq!(super::grey_level(Color32::from_rgb(0, 0, 0)), 0);
        assert_eq!(super::grey_level(Color32::from_rgb(255, 255, 255)), 213); // 765 / 5 + 60
        assert_eq!(super::grey_level(Color32::from_rgb(226, 44, 44)), 122); // 314 / 5 = 62, + 60
        assert_eq!(super::grey_level(Color32::from_rgb(1, 1, 2)), 0); // 4 / 5 = 0
    }

    #[test]
    fn every_pattern_is_the_right_size() {
        for (i, p) in super::ITEMS7.iter().enumerate() {
            assert!(p.iter().all(|r| r.len() == 7), "item {i}");
        }
        for (i, p) in super::MONSTERS8.iter().chain(super::FLOOR8.iter()).enumerate() {
            assert!(p.iter().all(|r| r.len() == 8), "8x8 pattern {i}");
        }
        for p in super::SHOP3.iter() {
            assert!(p.iter().all(|r| r.len() == 3));
        }
        assert!(super::TERRAIN.iter().all(|r| r.len() == 64 && r.bytes().all(|b| b"RWFGSO".contains(&b))));
    }

    /// Saves every sheet as a PNG to look at: Z1R_ART_DIR=/some/folder cargo test -- --ignored save_sheets
    #[test]
    #[ignore]
    fn save_sheets() {
        let dir = std::path::PathBuf::from(std::env::var("Z1R_ART_DIR").expect("set Z1R_ART_DIR"));
        for (name, img) in super::sheets() {
            let rgba: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect();
            image::save_buffer(dir.join(name), &rgba, img.size[0] as u32, img.size[1] as u32, image::ColorType::Rgba8).unwrap();
        }
    }
}
