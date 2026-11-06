//! The tracker's own pictures. Every picture is drawn here, at start-up: smooth shapes (circles, curves, polygons,
//! gradients) painted with anti-aliasing, the monsters from small pixel patterns made smooth with the Scale2x method,
//! and the overworld from the terrain table below. Nothing is read from files or from the original program.
//!
//! The pictures are drawn at several times the size the tracker shows them (the sheet's "scale"), so they stay sharp
//! when the window is large. Each sheet keeps the layout the drawing code expects in its own units (e.g.
//! "icons7x7.png": 32 item pictures of 7x7 units, here 28x28 pixels each), so the rest of the tracker just asks for
//! "picture n of sheet s".

use eframe::egui::{Color32, ColorImage};

// ------------------------------------------------------------------------------------------------ colours

/// A colour with alpha, each part 0..1 (not premultiplied).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct C(pub f32, pub f32, pub f32, pub f32);

const fn hex(v: u32) -> C {
    C(((v >> 16) & 255) as f32 / 255.0, ((v >> 8) & 255) as f32 / 255.0, (v & 255) as f32 / 255.0, 1.0)
}

impl C {
    fn mix(self, o: C, t: f32) -> C {
        let t = t.clamp(0.0, 1.0);
        C(self.0 + (o.0 - self.0) * t, self.1 + (o.1 - self.1) * t, self.2 + (o.2 - self.2) * t, self.3 + (o.3 - self.3) * t)
    }
    fn shade(self, f: f32) -> C {
        C((self.0 * f).min(1.0), (self.1 * f).min(1.0), (self.2 * f).min(1.0), self.3)
    }
    fn alpha(self, a: f32) -> C {
        C(self.0, self.1, self.2, self.3 * a)
    }
    fn from32(c: Color32) -> C {
        C(c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, 1.0)
    }
}

const WHITE: C = hex(0xffffff);
const DARK: C = hex(0x1c1c24);
const OUTLINE: C = C(0.04, 0.04, 0.06, 0.95);
const CREAM: C = hex(0xf8eed4);
const SKIN: C = hex(0xf6c49c);
const WOOD_L: C = hex(0xe2a668);
const WOOD: C = hex(0xb06c2e);
const WOOD_D: C = hex(0x6a3c16);
const GOLD_L: C = hex(0xfff0a0);
const GOLD: C = hex(0xf4c434);
const GOLD_D: C = hex(0xb07810);
const RED_L: C = hex(0xff8070);
const RED: C = hex(0xe0302a);
const RED_D: C = hex(0x861414);
const BLUE_L: C = hex(0x96c0ff);
const BLUE: C = hex(0x3c70ea);
const BLUE_D: C = hex(0x182f8c);
const SILVER_L: C = hex(0xffffff);
const SILVER: C = hex(0xd2d6e2);
const SILVER_D: C = hex(0x868c9c);
const CYAN_L: C = hex(0xe0f8ff);
const CYAN: C = hex(0x66cff8);
const GREEN_L: C = hex(0x9ef08a);
const GREEN: C = hex(0x3cbc4c);
const GREEN_D: C = hex(0x186a28);
const ORANGE: C = hex(0xf6922c);

/// How a shape is coloured: one colour, a top-to-bottom or left-to-right blend, or a blend outwards from a point.
#[derive(Clone, Copy)]
enum Paint {
    S(C),
    V(C, C, f32, f32),
    H(C, C, f32, f32),
    R(C, C, f32, f32, f32),
}

impl Paint {
    fn at(&self, x: f32, y: f32) -> C {
        match *self {
            Paint::S(c) => c,
            Paint::V(a, b, y0, y1) => a.mix(b, (y - y0) / (y1 - y0)),
            Paint::H(a, b, x0, x1) => a.mix(b, (x - x0) / (x1 - x0)),
            Paint::R(a, b, cx, cy, r) => a.mix(b, ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() / r),
        }
    }
}

// ------------------------------------------------------------------------------------------------ pictures

/// A picture being drawn.
pub struct Img {
    w: usize,
    h: usize,
    px: Vec<C>,
}

impl Img {
    pub fn new(w: usize, h: usize) -> Self {
        Img { w, h, px: vec![C(0.0, 0.0, 0.0, 0.0); w * h] }
    }
    fn blend(&mut self, x: usize, y: usize, c: C, cover: f32) {
        if x >= self.w || y >= self.h {
            return;
        }
        let a = c.3 * cover;
        if a <= 0.0 {
            return;
        }
        let i = y * self.w + x;
        let d = self.px[i];
        let oa = a + d.3 * (1.0 - a);
        let mixc = |s: f32, t: f32| (s * a + t * d.3 * (1.0 - a)) / oa;
        self.px[i] = C(mixc(c.0, d.0), mixc(c.1, d.1), mixc(c.2, d.2), oa);
    }
    /// Lays `src` on top of this picture at (x, y).
    fn put(&mut self, src: &Img, x: usize, y: usize) {
        for yy in 0..src.h {
            for xx in 0..src.w {
                let c = src.px[yy * src.w + xx];
                self.blend(x + xx, y + yy, c, 1.0);
            }
        }
    }
    /// A dark edge around everything drawn so far (`r` pixels wide), drawn underneath it.
    fn outline(&mut self, r: f32, col: C) {
        let src = self.px.clone();
        let n = r.ceil() as i32;
        for y in 0..self.h as i32 {
            for x in 0..self.w as i32 {
                let mut best: f32 = 0.0;
                for dy in -n..=n {
                    for dx in -n..=n {
                        let d = ((dx * dx + dy * dy) as f32).sqrt();
                        let (xx, yy) = (x + dx, y + dy);
                        if d > r + 0.5 || xx < 0 || yy < 0 || xx >= self.w as i32 || yy >= self.h as i32 {
                            continue;
                        }
                        let a = src[yy as usize * self.w + xx as usize].3 * (r + 0.5 - d).clamp(0.0, 1.0);
                        best = best.max(a);
                    }
                }
                let i = y as usize * self.w + x as usize;
                let top = src[i];
                // the edge underneath, then the picture on top of it
                let mut c = col.alpha(best);
                let oa = top.3 + c.3 * (1.0 - top.3);
                if oa > 0.0 {
                    let m = |s: f32, t: f32| (s * top.3 + t * c.3 * (1.0 - top.3)) / oa;
                    c = C(m(top.0, c.0), m(top.1, c.1), m(top.2, c.2), oa);
                }
                self.px[i] = c;
            }
        }
    }
    /// Lighter at the top and darker at the bottom, for a little roundness.
    fn light_from_above(&mut self, top: f32, bottom: f32) {
        for y in 0..self.h {
            let f = top + (bottom - top) * y as f32 / (self.h.max(2) - 1) as f32;
            for x in 0..self.w {
                let i = y * self.w + x;
                self.px[i] = self.px[i].shade(f);
            }
        }
    }
    pub fn image(&self) -> ColorImage {
        let bytes: Vec<u8> = self
            .px
            .iter()
            .flat_map(|c| [(c.0 * 255.0).round() as u8, (c.1 * 255.0).round() as u8, (c.2 * 255.0).round() as u8, (c.3 * 255.0).round() as u8])
            .collect();
        ColorImage::from_rgba_unmultiplied([self.w, self.h], &bytes)
    }
}

/// Draws smooth shapes on a picture. Shapes are given in design units; (ox, oy) is where unit (0, 0) lands and `k` is
/// how many pixels one unit is. Every pixel is sampled 4 x 4 times, so edges come out smooth.
struct Pen<'a> {
    img: &'a mut Img,
    ox: f32,
    oy: f32,
    k: f32,
}

impl<'a> Pen<'a> {
    fn new(img: &'a mut Img, ox: f32, oy: f32, k: f32) -> Self {
        Pen { img, ox, oy, k }
    }
    /// A pen for a square of `size` units at (x, y), in which 16 design units fit.
    fn sub(&mut self, x: f32, y: f32, size: f32) -> Pen<'_> {
        Pen { ox: self.ox + x * self.k, oy: self.oy + y * self.k, k: self.k * size / 16.0, img: &mut *self.img }
    }
    fn shape(&mut self, b: (f32, f32, f32, f32), paint: Paint, inside: impl Fn(f32, f32) -> bool) {
        let px0 = (self.ox + b.0 * self.k).floor().max(0.0) as usize;
        let py0 = (self.oy + b.1 * self.k).floor().max(0.0) as usize;
        let px1 = ((self.ox + b.2 * self.k).ceil().max(0.0) as usize).min(self.img.w);
        let py1 = ((self.oy + b.3 * self.k).ceil().max(0.0) as usize).min(self.img.h);
        for py in py0..py1 {
            for px in px0..px1 {
                let (mut n, mut sx, mut sy) = (0, 0.0, 0.0);
                for j in 0..4 {
                    for i in 0..4 {
                        let x = (px as f32 + (i as f32 + 0.5) / 4.0 - self.ox) / self.k;
                        let y = (py as f32 + (j as f32 + 0.5) / 4.0 - self.oy) / self.k;
                        if inside(x, y) {
                            n += 1;
                            sx += x;
                            sy += y;
                        }
                    }
                }
                if n > 0 {
                    let c = paint.at(sx / n as f32, sy / n as f32);
                    self.img.blend(px, py, c, n as f32 / 16.0);
                }
            }
        }
    }
    fn circle(&mut self, cx: f32, cy: f32, r: f32, p: Paint) {
        self.shape((cx - r, cy - r, cx + r, cy + r), p, move |x, y| (x - cx).powi(2) + (y - cy).powi(2) <= r * r);
    }
    fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, p: Paint) {
        self.shape((cx - rx, cy - ry, cx + rx, cy + ry), p, move |x, y| ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2) <= 1.0);
    }
    fn ring(&mut self, cx: f32, cy: f32, ro: f32, ri: f32, p: Paint) {
        self.shape((cx - ro, cy - ro, cx + ro, cy + ro), p, move |x, y| {
            let d = (x - cx).powi(2) + (y - cy).powi(2);
            d <= ro * ro && d >= ri * ri
        });
    }
    /// A thick line with round ends.
    fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, w: f32, p: Paint) {
        let r = w / 2.0;
        let b = (x0.min(x1) - r, y0.min(y1) - r, x0.max(x1) + r, y0.max(y1) + r);
        self.shape(b, p, move |x, y| seg_dist(x, y, x0, y0, x1, y1) <= r);
    }
    fn poly(&mut self, pts: &[(f32, f32)], p: Paint) {
        let pts = pts.to_vec();
        let b = pts.iter().fold((f32::MAX, f32::MAX, f32::MIN, f32::MIN), |b, &(x, y)| (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y)));
        self.shape(b, p, move |x, y| in_poly(&pts, x, y));
    }
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, p: Paint) {
        self.shape((x, y, x + w, y + h), p, move |px, py| px >= x && px <= x + w && py >= y && py <= y + h);
    }
    fn rrect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, p: Paint) {
        self.shape((x, y, x + w, y + h), p, move |px, py| in_rrect(px, py, x, y, w, h, r));
    }
    /// A curved band of radius `r` and width `w`, from angle a0 to a1 (degrees, clockwise from the right, a0 < a1).
    fn arc(&mut self, cx: f32, cy: f32, r: f32, w: f32, a0: f32, a1: f32, p: Paint) {
        let o = r + w / 2.0;
        self.shape((cx - o, cy - o, cx + o, cy + o), p, move |x, y| {
            let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
            if (d - r).abs() > w / 2.0 {
                return false;
            }
            let mut a = (y - cy).atan2(x - cx).to_degrees();
            while a < a0 {
                a += 360.0;
            }
            while a >= a0 + 360.0 {
                a -= 360.0;
            }
            a <= a1
        });
    }
    fn heart(&mut self, cx: f32, cy: f32, s: f32, p: Paint) {
        self.shape((cx - 6.4 * s, cy - 4.8 * s, cx + 6.4 * s, cy + 6.2 * s), p, move |x, y| in_heart((x - cx) / s, (y - cy) / s));
    }
}

fn seg_dist(x: f32, y: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let l = dx * dx + dy * dy;
    let t = if l == 0.0 { 0.0 } else { (((x - x0) * dx + (y - y0) * dy) / l).clamp(0.0, 1.0) };
    ((x - x0 - t * dx).powi(2) + (y - y0 - t * dy).powi(2)).sqrt()
}

fn in_poly(pts: &[(f32, f32)], x: f32, y: f32) -> bool {
    let mut inside = false;
    let mut j = pts.len() - 1;
    for i in 0..pts.len() {
        let (xi, yi) = pts[i];
        let (xj, yj) = pts[j];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn in_rrect(px: f32, py: f32, x: f32, y: f32, w: f32, h: f32, r: f32) -> bool {
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    let qx = ((px - cx).abs() - w / 2.0 + r).max(0.0);
    let qy = ((py - cy).abs() - h / 2.0 + r).max(0.0);
    (px - cx).abs() <= w / 2.0 && (py - cy).abs() <= h / 2.0 && qx * qx + qy * qy <= r * r
}

/// A heart about 12.4 units wide, centred on (0, 0).
fn in_heart(u: f32, v: f32) -> bool {
    let lobe = |cx: f32| (u - cx).powi(2) + (v + 1.5).powi(2) <= 10.0;
    let point = (-0.5..=6.0).contains(&v) && u.abs() <= 6.1 * (6.0 - v) / 6.5;
    lobe(-3.0) || lobe(3.0) || point
}

// ------------------------------------------------------------------------------------------------ smoothing pixel art

/// Scale2x: doubles a pixel grid, rounding off diagonal steps instead of making them blocky.
fn scale2x<T: Copy + PartialEq>(w: usize, h: usize, src: &[T]) -> Vec<T> {
    let at = |x: i32, y: i32| src[(y.clamp(0, h as i32 - 1) as usize) * w + x.clamp(0, w as i32 - 1) as usize];
    let mut out = vec![src[0]; w * h * 4];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let (p, a, b, c, d) = (at(x, y), at(x, y - 1), at(x + 1, y), at(x - 1, y), at(x, y + 1));
            let e = [
                if c == a && c != d && a != b { a } else { p },
                if a == b && a != c && b != d { b } else { p },
                if d == c && d != b && c != a { c } else { p },
                if b == d && b != a && d != c { d } else { p },
            ];
            let (ox, oy) = (x as usize * 2, y as usize * 2);
            out[oy * w * 2 + ox] = e[0];
            out[oy * w * 2 + ox + 1] = e[1];
            out[(oy + 1) * w * 2 + ox] = e[2];
            out[(oy + 1) * w * 2 + ox + 1] = e[3];
        }
    }
    out
}

/// A pixel pattern (see `ink`) made `2^passes` times bigger with Scale2x.
fn smooth_art(rows: &[&str], passes: u32) -> Img {
    let (mut w, mut h) = (rows[0].len(), rows.len());
    let mut grid: Vec<char> = rows.iter().flat_map(|r| r.chars()).collect();
    for _ in 0..passes {
        grid = scale2x(w, h, &grid);
        w *= 2;
        h *= 2;
    }
    let mut img = Img::new(w, h);
    for (i, ch) in grid.iter().enumerate() {
        if let Some(c) = ink(*ch) {
            img.px[i] = C::from32(c);
        }
    }
    img
}

/// Text in the 3x5 letters, made 4 times bigger and smoothed, with its top-left corner at (x, y) pixels.
fn text(img: &mut Img, x: usize, y: usize, s: &str, col: C) {
    let n = s.chars().count();
    if n == 0 {
        return;
    }
    let (w, h) = (n * 4 - 1, 5);
    let mut mask = vec![false; w * h];
    for (i, ch) in s.chars().enumerate() {
        if let Some(g) = glyph(ch) {
            for (yy, row) in g.iter().enumerate() {
                for (xx, b) in row.bytes().enumerate() {
                    if b == b'#' {
                        mask[yy * w + i * 4 + xx] = true;
                    }
                }
            }
        }
    }
    let mask = scale2x(w * 2, h * 2, &scale2x(w, h, &mask));
    for yy in 0..h * 4 {
        for xx in 0..w * 4 {
            if mask[yy * w * 4 + xx] {
                img.blend(x + xx, y + yy, col, 1.0);
            }
        }
    }
}

/// Text centred on (cx, cy) pixels.
fn text_c(img: &mut Img, cx: f32, cy: f32, s: &str, col: C) {
    let w = (s.chars().count() * 4 - 1) * 4;
    text(img, (cx - w as f32 / 2.0).round().max(0.0) as usize, (cy - 10.0).round().max(0.0) as usize, s, col);
}

// ------------------------------------------------------------------------------------------------ items

fn boomerang(p: &mut Pen, l: C, d: C) {
    p.arc(13.0, 13.0, 8.0, 3.6, 180.0, 270.0, Paint::V(l, d, 3.0, 15.0));
    p.circle(5.0, 13.0, 1.8, Paint::S(d));
    p.circle(13.0, 5.0, 1.8, Paint::S(l));
    p.arc(13.0, 13.0, 9.0, 0.7, 195.0, 255.0, Paint::S(WHITE.alpha(0.55)));
}

fn bow(p: &mut Pen, cx: f32, cy: f32, r: f32) {
    let (ex, ey) = (cx - r * 0.5, r * 0.866);
    p.line(ex, cy - ey + 0.3, ex, cy + ey - 0.3, 0.55, Paint::S(CREAM));
    p.arc(cx, cy, r, 1.9, 120.0, 240.0, Paint::H(WOOD_L, WOOD_D, cx - r, ex));
    p.circle(ex, cy - ey, 1.0, Paint::S(GOLD));
    p.circle(ex, cy + ey, 1.0, Paint::S(GOLD));
}

fn candle(p: &mut Pen, l: C, d: C) {
    p.rrect(4.0, 13.0, 8.0, 2.0, 0.9, Paint::V(SILVER, SILVER_D, 13.0, 15.0));
    p.rrect(5.6, 6.4, 4.8, 7.2, 1.0, Paint::H(l, d, 5.6, 10.4));
    p.ellipse(8.0, 6.7, 2.3, 0.7, Paint::S(l.mix(WHITE, 0.35)));
    p.line(8.0, 5.0, 8.0, 6.4, 0.5, Paint::S(DARK));
    p.circle(8.0, 3.9, 1.7, Paint::S(ORANGE));
    p.poly(&[(6.4, 3.6), (9.6, 3.6), (8.0, 0.4)], Paint::S(ORANGE));
    p.ellipse(8.0, 4.1, 0.85, 1.3, Paint::S(GOLD_L));
}

fn arrow(p: &mut Pen, shaft: C, head: C) {
    p.poly(&[(8.0, 11.0), (5.4, 13.6), (5.4, 15.4), (8.0, 13.2)], Paint::S(RED));
    p.poly(&[(8.0, 11.0), (10.6, 13.6), (10.6, 15.4), (8.0, 13.2)], Paint::S(RED_D));
    p.line(8.0, 4.0, 8.0, 14.6, 1.2, Paint::S(shaft));
    p.poly(&[(8.0, 0.6), (10.8, 5.6), (8.0, 4.6), (5.2, 5.6)], Paint::H(SILVER_L, head, 5.2, 10.8));
}

fn ring(p: &mut Pen, l: C, d: C) {
    p.ring(8.0, 9.2, 5.2, 3.2, Paint::V(l, d, 4.0, 14.4));
    p.arc(8.0, 9.2, 4.2, 0.6, 200.0, 260.0, Paint::S(WHITE.alpha(0.6)));
    p.poly(&[(8.0, 1.2), (10.2, 3.6), (8.0, 6.0), (5.8, 3.6)], Paint::V(WHITE, l, 1.2, 6.0));
}

fn sword(p: &mut Pen, bl: C, bd: C, guard: C) {
    p.poly(&[(8.0, 0.6), (9.4, 2.4), (9.4, 10.2), (6.6, 10.2), (6.6, 2.4)], Paint::H(bl, bd, 6.6, 9.4));
    p.line(8.0, 2.8, 8.0, 9.4, 0.35, Paint::S(bd.alpha(0.7)));
    p.rrect(4.0, 10.0, 8.0, 1.8, 0.9, Paint::V(guard.mix(WHITE, 0.3), guard.shade(0.6), 10.0, 11.8));
    p.rect(7.1, 11.7, 1.8, 2.7, Paint::H(WOOD, WOOD_D, 7.1, 8.9));
    p.circle(8.0, 14.9, 1.15, Paint::R(guard.mix(WHITE, 0.4), guard, 7.6, 14.5, 1.4));
}

fn rupee(p: &mut Pen, l: C, d: C) {
    p.poly(&[(8.0, 0.8), (12.6, 4.6), (12.6, 11.4), (8.0, 15.2), (3.4, 11.4), (3.4, 4.6)], Paint::H(l, d, 3.4, 12.6));
    p.poly(&[(8.0, 3.4), (10.2, 5.4), (10.2, 10.6), (8.0, 12.6), (5.8, 10.6), (5.8, 5.4)], Paint::V(l.mix(WHITE, 0.45), l.mix(d, 0.4), 3.4, 12.6));
    p.line(5.0, 5.2, 5.0, 9.2, 0.6, Paint::S(WHITE.alpha(0.7)));
}

fn triforce(p: &mut Pen, l: C, m: C, d: C) {
    p.poly(&[(8.0, 0.8), (15.4, 14.8), (0.6, 14.8)], Paint::V(m, d, 0.8, 14.8));
    p.poly(&[(8.0, 4.0), (12.8, 13.0), (3.2, 13.0)], Paint::V(l, m, 4.0, 13.0));
}

fn heart_icon(p: &mut Pen, l: C, d: C) {
    p.heart(8.0, 8.4, 1.05, Paint::R(l, d, 5.0, 5.0, 10.0));
    p.ellipse(5.0, 5.8, 1.3, 0.85, Paint::S(WHITE.alpha(0.75)));
}

fn bomb(p: &mut Pen) {
    p.circle(8.0, 9.8, 5.4, Paint::R(BLUE_L, BLUE_D, 6.0, 7.6, 7.0));
    p.rrect(6.5, 3.4, 3.0, 1.8, 0.4, Paint::V(SILVER, SILVER_D, 3.4, 5.2));
    p.line(9.0, 3.6, 11.0, 1.8, 0.75, Paint::S(hex(0xc8a070)));
    p.circle(11.6, 1.4, 1.2, Paint::S(ORANGE));
    p.circle(11.6, 1.4, 0.6, Paint::S(GOLD_L));
    p.ellipse(5.9, 7.6, 1.4, 0.9, Paint::S(WHITE.alpha(0.7)));
}

fn key(p: &mut Pen) {
    p.ring(8.0, 4.6, 3.3, 1.5, Paint::V(GOLD_L, GOLD_D, 1.3, 7.9));
    p.rrect(7.1, 7.2, 1.8, 7.8, 0.5, Paint::H(GOLD_L, GOLD_D, 7.1, 8.9));
    p.rect(8.8, 11.0, 2.4, 1.3, Paint::S(GOLD));
    p.rect(8.8, 13.2, 3.0, 1.3, Paint::S(GOLD));
}

fn compass(p: &mut Pen) {
    p.circle(8.0, 8.4, 6.2, Paint::V(RED_L, RED_D, 2.2, 14.6));
    p.circle(8.0, 8.4, 4.8, Paint::V(WHITE, CREAM, 3.6, 13.2));
    p.poly(&[(8.0, 4.2), (9.3, 8.4), (6.7, 8.4)], Paint::S(RED));
    p.poly(&[(8.0, 12.6), (9.3, 8.4), (6.7, 8.4)], Paint::S(BLUE_D));
    p.circle(8.0, 8.4, 0.7, Paint::S(GOLD));
}

fn map_scroll(p: &mut Pen) {
    p.rect(2.6, 3.2, 10.8, 9.6, Paint::V(CREAM, hex(0xdcc08a), 3.2, 12.8));
    p.line(2.6, 2.6, 2.6, 13.4, 2.2, Paint::H(WOOD_L, WOOD_D, 1.5, 3.7));
    p.line(13.4, 2.6, 13.4, 13.4, 2.2, Paint::H(WOOD_L, WOOD_D, 12.3, 14.5));
    for (x, y) in [(5.0, 10.6), (6.4, 9.2), (7.8, 8.4), (9.2, 7.2)] {
        p.circle(x, y, 0.45, Paint::S(RED_D));
    }
    p.line(9.6, 4.6, 11.4, 6.4, 0.7, Paint::S(RED));
    p.line(11.4, 4.6, 9.6, 6.4, 0.7, Paint::S(RED));
}

fn star(p: &mut Pen) {
    let mut pts = vec![];
    for i in 0..10 {
        let a = (i as f32 * 36.0 - 90.0).to_radians();
        let r = if i % 2 == 0 { 7.4 } else { 3.2 };
        pts.push((8.0 + r * a.cos(), 8.6 + r * a.sin()));
    }
    p.poly(&pts, Paint::R(GOLD_L, GOLD_D, 7.0, 7.0, 8.0));
}

/// Draws item picture `i` (the order of "icons7x7.png", see ITEM_NAMES) in a 16 x 16 unit square.
fn draw_item(p: &mut Pen, i: usize) {
    match i {
        0 => boomerang(p, WOOD_L, WOOD_D),
        1 => bow(p, 13.0, 8.0, 7.5),
        2 => boomerang(p, BLUE_L, BLUE_D),
        3 => {
            for n in 0..5 {
                let x = 1.9 + n as f32 * 2.5;
                p.rrect(x, 2.8, 2.35, 10.4, 1.15, Paint::H(WOOD_L, WOOD_D, x, x + 2.35));
            }
            p.rect(1.6, 5.0, 12.8, 1.1, Paint::S(hex(0x5a3818)));
            p.rect(1.6, 10.0, 12.8, 1.1, Paint::S(hex(0x5a3818)));
        }
        4 => {
            for j in 0..4 {
                p.rect(4.0, 3.4 + j as f32 * 3.1, 8.0, 1.3, Paint::V(WOOD_L, WOOD, 3.4, 14.0));
            }
            p.line(4.0, 1.4, 4.0, 14.6, 1.8, Paint::H(WOOD_L, WOOD_D, 3.1, 4.9));
            p.line(12.0, 1.4, 12.0, 14.6, 1.8, Paint::H(WOOD_L, WOOD_D, 11.1, 12.9));
        }
        5 => {
            p.line(3.4, 13.6, 12.0, 3.2, 2.8, Paint::H(hex(0xf0d8a0), WOOD, 3.0, 13.0));
            p.line(12.0, 3.2, 13.0, 1.9, 2.0, Paint::S(hex(0xf4e4c0)));
            for t in [0.3f32, 0.45, 0.6, 0.75] {
                p.circle(3.4 + (12.0 - 3.4) * t, 13.6 + (3.2 - 13.6) * t, 0.5, Paint::S(WOOD_D));
            }
        }
        6 => {
            p.line(4.4, 14.6, 10.0, 5.6, 1.9, Paint::H(BLUE_L, BLUE_D, 4.0, 10.6));
            p.circle(11.0, 3.8, 2.6, Paint::R(WHITE, CYAN, 10.4, 3.2, 2.8));
            p.line(11.0, 0.4, 11.0, 7.2, 0.5, Paint::S(WHITE.alpha(0.8)));
            p.line(7.6, 3.8, 14.4, 3.8, 0.5, Paint::S(WHITE.alpha(0.8)));
        }
        7 => candle(p, RED_L, RED_D),
        8 => {
            p.rrect(3.8, 3.0, 9.8, 11.4, 0.6, Paint::S(CREAM));
            p.rrect(2.4, 1.8, 9.8, 11.6, 1.0, Paint::V(RED_L, RED_D, 1.8, 13.4));
            p.rect(2.4, 1.8, 1.7, 11.6, Paint::S(RED_D));
            p.poly(&[(7.6, 5.0), (9.6, 7.6), (7.6, 10.2), (5.6, 7.6)], Paint::V(GOLD_L, GOLD_D, 5.0, 10.2));
        }
        9 => key(p),
        10 => arrow(p, SILVER, SILVER),
        11 => arrow(p, WOOD, SILVER_D),
        12 => ring(p, RED_L, RED_D),
        13 => {
            let rim = [(2.0, 1.4), (14.0, 1.4), (14.0, 8.4), (8.0, 15.4), (2.0, 8.4)];
            p.poly(&rim, Paint::V(SILVER_L, SILVER_D, 1.4, 15.4));
            p.poly(&[(3.4, 2.8), (12.6, 2.8), (12.6, 8.0), (8.0, 13.4), (3.4, 8.0)], Paint::V(BLUE_L, BLUE_D, 2.8, 13.4));
            p.line(8.0, 4.6, 8.0, 10.8, 1.4, Paint::S(GOLD));
            p.line(5.4, 7.0, 10.6, 7.0, 1.4, Paint::S(GOLD));
        }
        14 => {
            p.rrect(2.6, 4.4, 6.4, 9.0, 0.5, Paint::S(CREAM));
            p.rrect(1.4, 3.6, 6.4, 9.0, 0.8, Paint::V(RED_L, RED_D, 3.6, 12.6));
            p.arc(15.6, 13.2, 5.8, 2.6, 180.0, 270.0, Paint::V(BLUE_L, BLUE_D, 6.0, 15.0));
            p.circle(9.8, 13.2, 1.3, Paint::S(BLUE_D));
            p.circle(15.6, 7.4, 1.3, Paint::S(BLUE_L));
        }
        15 => heart_icon(p, RED_L, RED_D),
        16 => {
            p.shape((1.5, 3.5, 14.5, 12.5), Paint::V(RED_L, RED_D, 3.5, 12.5), |x, y| {
                let (u, v) = (x - 8.0, y - 8.0);
                (u / 6.4).powi(2) + (v / 4.2).powi(2) <= 1.0 && (u / 4.4).powi(2) + (v / 2.4).powi(2) >= 1.0
            });
            for (x, y) in [(2.4, 8.0), (13.6, 8.0), (8.0, 4.7), (8.0, 11.3)] {
                p.circle(x, y, 0.9, Paint::R(GOLD_L, GOLD_D, x - 0.3, y - 0.3, 1.0));
            }
        }
        17 => sword(p, SILVER_L, SILVER_D, BLUE),
        18 => {
            let stone = Paint::V(hex(0xf4be7c), hex(0x9c5c26), 2.0, 15.0);
            p.rrect(3.4, 2.4, 9.2, 11.0, 2.6, stone);
            p.rect(2.6, 12.6, 10.8, 2.6, Paint::V(hex(0xb47440), hex(0x6a3a14), 12.6, 15.2));
            p.circle(6.0, 6.4, 1.1, Paint::S(RED));
            p.circle(10.0, 6.4, 1.1, Paint::S(RED));
            p.rect(6.0, 9.2, 4.0, 1.0, Paint::S(hex(0x6a3a14)));
        }
        19 => sword(p, hex(0xf0d8a8), hex(0xa07040), WOOD),
        20 => sword(p, CYAN_L, CYAN, RED),
        21 => candle(p, BLUE_L, BLUE_D),
        22 => ring(p, BLUE_L, BLUE_D),
        23 => {
            p.poly(&[(2.2, 1.2), (5.6, 4.6), (3.0, 6.0)], Paint::S(SILVER));
            p.poly(&[(13.8, 1.2), (10.4, 4.6), (13.0, 6.0)], Paint::S(SILVER));
            p.circle(8.0, 8.4, 5.6, Paint::V(BLUE_L, BLUE_D, 2.8, 14.0));
            p.circle(5.9, 7.2, 1.1, Paint::S(RED));
            p.circle(10.1, 7.2, 1.1, Paint::S(RED));
            p.circle(5.9, 7.2, 0.45, Paint::S(GOLD_L));
            p.circle(10.1, 7.2, 0.45, Paint::S(GOLD_L));
            p.ellipse(8.0, 11.0, 2.7, 1.9, Paint::V(hex(0xf8b0c0), hex(0xc06080), 9.1, 12.9));
            p.circle(7.1, 11.0, 0.45, Paint::S(DARK));
            p.circle(8.9, 11.0, 0.45, Paint::S(DARK));
            p.poly(&[(4.6, 11.4), (5.4, 13.8), (5.8, 11.6)], Paint::S(WHITE));
            p.poly(&[(11.4, 11.4), (10.6, 13.8), (10.2, 11.6)], Paint::S(WHITE));
        }
        24 => {
            p.circle(8.0, 7.8, 5.8, Paint::V(GOLD_L, GOLD_D, 2.0, 13.6));
            p.poly(&[(3.6, 16.0), (12.4, 16.0), (10.4, 12.6), (5.6, 12.6)], Paint::V(RED_L, RED_D, 12.6, 16.0));
            p.ellipse(8.0, 8.8, 3.4, 3.8, Paint::V(SKIN, SKIN.shade(0.85), 5.0, 12.6));
            p.ellipse(8.0, 5.6, 4.0, 1.9, Paint::V(GOLD_L, GOLD, 3.7, 7.5));
            p.circle(6.7, 9.0, 0.55, Paint::S(DARK));
            p.circle(9.3, 9.0, 0.55, Paint::S(DARK));
            p.line(7.4, 11.0, 8.6, 11.0, 0.4, Paint::S(RED_D));
            p.poly(&[(5.8, 3.6), (6.6, 1.8), (7.4, 3.0), (8.0, 1.4), (8.6, 3.0), (9.4, 1.8), (10.2, 3.6)], Paint::S(GOLD_L));
        }
        25 => bomb(p),
        26 => {
            bow(p, 11.4, 8.0, 7.0);
            p.line(1.6, 8.0, 13.0, 8.0, 1.0, Paint::S(WOOD));
            p.poly(&[(15.6, 8.0), (12.4, 6.0), (13.2, 8.0), (12.4, 10.0)], Paint::S(SILVER));
            p.poly(&[(1.0, 6.4), (3.4, 8.0), (1.0, 9.6)], Paint::S(RED));
        }
        27 => {
            p.line(3.6, 12.4, 6.6, 9.4, 1.7, Paint::S(CREAM));
            p.circle(2.6, 12.4, 1.1, Paint::S(CREAM));
            p.circle(3.6, 13.4, 1.1, Paint::S(CREAM));
            p.ellipse(9.4, 6.8, 5.0, 4.4, Paint::R(hex(0xf07850), hex(0x8a2a14), 8.0, 5.0, 6.5));
            p.ellipse(7.8, 5.0, 1.6, 0.9, Paint::S(WHITE.alpha(0.5)));
        }
        28 => {
            p.arc(8.0, 5.6, 3.2, 2.1, 180.0, 450.0, Paint::V(GOLD_L, GOLD_D, 2.0, 9.0));
            p.line(8.0, 8.8, 8.0, 10.6, 2.1, Paint::S(GOLD));
            p.circle(8.0, 13.6, 1.3, Paint::S(GOLD));
        }
        29 => rupee(p, CYAN, BLUE_D),
        30 => {
            p.shape((2.0, 2.0, 14.0, 14.0), Paint::S(SILVER_D), |x, y| in_rrect(x, y, 2.0, 2.0, 12.0, 12.0, 1.5) && !in_rrect(x, y, 3.5, 3.5, 9.0, 9.0, 0.8));
        }
        _ => compass(p),
    }
}

/// One item picture, `size` pixels square, with a dark edge.
fn item_img(i: usize, size: usize) -> Img {
    let mut img = Img::new(size, size);
    draw_item(&mut Pen::new(&mut img, 0.0, 0.0, size as f32 / 16.0), i);
    img.outline(size as f32 / 22.0, OUTLINE);
    img
}

/// "icons10x10.png": 0 triforce (missing), 1 triforce, 2 Take Any heart skipped, 3 Take Any heart unknown,
/// 4 Take Any heart taken, 5 arrow, 6 tick, 7 starting items, 8 save disk, 9 big heart. 40 pixels each.
fn icon10(i: usize) -> Img {
    let mut img = Img::new(40, 40);
    {
        let mut p = Pen::new(&mut img, 0.0, 0.0, 2.5);
        let grey = hex(0x8a8a94);
        match i {
            0 => triforce(&mut p, hex(0x7a7a84), hex(0x5a5a64), hex(0x3a3a42)),
            1 => triforce(&mut p, GOLD_L, GOLD, GOLD_D),
            2 => {
                heart_icon(&mut p, grey, hex(0x4a4a52));
                p.line(3.4, 3.4, 12.6, 13.0, 1.6, Paint::S(GOLD));
                p.line(12.6, 3.4, 3.4, 13.0, 1.6, Paint::S(GOLD));
            }
            3 => {
                p.heart(8.0, 8.4, 1.05, Paint::S(hex(0x6a6a74)));
                p.heart(8.0, 8.2, 0.8, Paint::V(hex(0x44444c), hex(0x2a2a30), 4.0, 13.0));
            }
            4 | 9 => heart_icon(&mut p, RED_L, RED_D),
            5 => p.poly(&[(1.0, 6.0), (8.5, 6.0), (8.5, 2.5), (15.0, 8.0), (8.5, 13.5), (8.5, 10.0), (1.0, 10.0)], Paint::V(GOLD_L, GOLD_D, 2.5, 13.5)),
            6 => {
                p.line(2.6, 8.6, 6.2, 12.4, 2.6, Paint::S(GREEN));
                p.line(6.2, 12.4, 13.6, 3.6, 2.6, Paint::V(GREEN_L, GREEN, 3.6, 12.4));
            }
            7 => {
                p.rrect(1.0, 4.0, 14.0, 8.0, 1.8, Paint::V(hex(0x6a8cff), hex(0x2a44b0), 4.0, 12.0));
                for x in [4.6, 8.0, 11.4] {
                    p.circle(x, 8.0, 1.1, Paint::S(WHITE));
                }
            }
            _ => {
                p.rrect(1.6, 1.6, 12.8, 12.8, 1.2, Paint::V(hex(0x3a3a4a), hex(0x1a1a24), 1.6, 14.4));
                p.rect(4.6, 1.6, 6.8, 4.6, Paint::V(SILVER_L, SILVER_D, 1.6, 6.2));
                p.rect(8.8, 2.4, 1.4, 3.0, Paint::S(hex(0x2a2a34)));
                p.rrect(3.6, 8.4, 8.8, 6.0, 0.6, Paint::S(CREAM));
                p.line(5.0, 10.4, 11.0, 10.4, 0.4, Paint::S(SILVER_D));
                p.line(5.0, 12.2, 11.0, 12.2, 0.4, Paint::S(SILVER_D));
            }
        }
    }
    img.outline(1.6, OUTLINE);
    img
}

// ------------------------------------------------------------------------------------------------ monsters and floor drops

/// A monster from its 8x8 pattern: smoothed to 32x32, lit from above, with a dark edge.
fn monster_img(i: usize) -> Img {
    let mut img = smooth_art(&MONSTERS8[i], 2);
    img.light_from_above(1.12, 0.8);
    img.outline(1.2, OUTLINE);
    img
}

/// "zelda_items16x16.png": what lies on a dungeon room's floor. 0 triforce, 1 heart container, 2 bombs, 3 key,
/// 4 rupee, 5 map, 6 compass, 7 another key item (8-12 unused). 32 pixels each.
fn floor_img(i: usize) -> Img {
    let mut img = Img::new(32, 32);
    {
        let mut p = Pen::new(&mut img, 0.0, 0.0, 2.0);
        match i {
            0 => triforce(&mut p, GOLD_L, GOLD, GOLD_D),
            1 => heart_icon(&mut p, RED_L, RED_D),
            2 => bomb(&mut p),
            3 => key(&mut p),
            4 => rupee(&mut p, GOLD_L, GOLD_D),
            5 => map_scroll(&mut p),
            6 => compass(&mut p),
            7 => star(&mut p),
            _ => {}
        }
    }
    img.outline(1.4, OUTLINE);
    img
}

// ------------------------------------------------------------------------------------------------ map markers

/// The colour of each dungeon level's marker (levels 1-9).
const LEVEL_COLORS: [C; 9] = [
    hex(0x5adcf0), hex(0x828cfa), hex(0x46c896), hex(0xf0dc28), hex(0x8cd23c),
    hex(0xfabe32), hex(0x78dc5a), hex(0xaaaab4), hex(0xbebec8),
];
const SHOP_COLOR: C = hex(0xef8300);

/// Black or white letters on a coloured background, by the original's rule (`Graphics::isBlackGoodContrast`,
/// token 0x06002b01, RVA 0x9a164): black when 0.2126 R² + 0.7152 G² + 0.0722 B² > 0.5 × 0.5 (R, G, B from 0 to 1),
/// otherwise white. The original uses it for the labels on each dungeon's colour.
pub fn black_is_good_contrast(bg: C) -> bool {
    0.2126 * bg.0 * bg.0 + 0.7152 * bg.1 * bg.1 + 0.0722 * bg.2 * bg.2 > 0.5 * 0.5
}

fn letters_on(bg: C) -> C {
    if black_is_good_contrast(bg) { hex(0x101014) } else { WHITE }
}

/// A marker's box: rounded, with a darker edge and a soft blend from top to bottom. Units are the 16 x 11 marker.
fn marker_box(p: &mut Pen, bg: C) {
    p.rrect(0.15, 0.15, 15.7, 10.7, 2.0, Paint::S(bg.shade(0.45)));
    p.rrect(0.8, 0.8, 14.4, 9.4, 1.5, Paint::V(bg.mix(WHITE, 0.25), bg.shade(0.82), 0.8, 10.2));
    p.rrect(1.4, 1.1, 13.2, 1.6, 0.8, Paint::S(WHITE.alpha(0.22)));
}

fn bottle(p: &mut Pen, cx: f32, l: C, d: C) {
    p.rect(cx - 0.8, 1.6, 1.6, 2.6, Paint::S(SILVER));
    p.rect(cx - 1.0, 1.0, 2.0, 1.0, Paint::S(WOOD));
    p.circle(cx, 6.6, 2.9, Paint::S(SILVER_L.alpha(0.9)));
    p.circle(cx, 6.8, 2.4, Paint::R(l, d, cx - 0.8, 6.0, 3.0));
    p.ellipse(cx - 1.0, 5.8, 0.6, 0.9, Paint::S(WHITE.alpha(0.7)));
}

/// One 16x11 marker at 4 times the size (64 x 44 pixels). Order of "s_icon_overworld_strip39.png": 0 nothing,
/// 1 unknown level, 2-10 levels 1-9, 11-14 any roads 1-4, 15 Take Any, 16 money game, 17 letter, 18 Armos,
/// 19-21 sword caves (magical, white, wood), 22 hint, 23-31 shops (shield, blue ring, meat, key, heart, candle, book,
/// bomb, arrow), 32 potion shop, 33 door repair, 34-37 secrets (unknown, large, medium, small), 38 "don't care".
fn marker_img(i: usize) -> Img {
    let mut img = Img::new(64, 44);
    let black = hex(0x101014);
    let item = |img: &mut Img, n: usize, x: usize, y: usize, size: usize| {
        let it = item_img(n, size);
        img.put(&it, x, y);
    };
    match i {
        0 => marker_box(&mut Pen::new(&mut img, 0.0, 0.0, 4.0), hex(0x2a1834)),
        1 => {
            marker_box(&mut Pen::new(&mut img, 0.0, 0.0, 4.0), hex(0x9a9aa4));
            text_c(&mut img, 32.0, 22.0, "L?", letters_on(hex(0x9a9aa4)));
        }
        2..=10 => {
            marker_box(&mut Pen::new(&mut img, 0.0, 0.0, 4.0), LEVEL_COLORS[i - 2]);
            text_c(&mut img, 32.0, 22.0, &format!("L{}", i - 1), letters_on(LEVEL_COLORS[i - 2]));
        }
        11..=14 => {
            let mut p = Pen::new(&mut img, 0.0, 0.0, 4.0);
            marker_box(&mut p, hex(0xe496ec));
            for s in 0..3 {
                let s = s as f32;
                p.rect(8.0 + s * 1.6, 3.0 + s * 1.9, 5.6 - s * 1.6, 1.5, Paint::V(hex(0x6a4ac8), hex(0x3c2878), 3.0, 9.0));
            }
            text_c(&mut img, 18.0, 22.0, &format!("{}", i - 10), letters_on(hex(0xe496ec)));
        }
        15 => {
            marker_box(&mut Pen::new(&mut img, 0.0, 0.0, 4.0), hex(0xc0242e));
            item(&mut img, 15, 6, 6, 32);
            text_c(&mut img, 48.0, 22.0, "?", letters_on(hex(0xc0242e)));
        }
        16 => {
            let mut p = Pen::new(&mut img, 0.0, 0.0, 4.0);
            marker_box(&mut p, hex(0x78c86e));
            p.line(2.6, 4.2, 6.6, 4.2, 1.0, Paint::S(black));
            p.line(4.6, 2.2, 4.6, 6.2, 1.0, Paint::S(black));
            p.line(2.6, 8.0, 6.6, 8.0, 1.0, Paint::S(black));
            item(&mut img, 29, 30, 6, 32);
        }
        17 => {
            let mut p = Pen::new(&mut img, 0.0, 0.0, 4.0);
            marker_box(&mut p, hex(0xc8dcf0));
            p.rrect(3.0, 2.6, 10.0, 6.0, 0.6, Paint::S(hex(0x3c508c)));
            p.rrect(3.4, 3.0, 9.2, 5.2, 0.4, Paint::V(WHITE, CREAM, 3.0, 8.2));
            p.line(3.6, 3.2, 8.0, 6.0, 0.5, Paint::S(hex(0x3c508c)));
            p.line(12.4, 3.2, 8.0, 6.0, 0.5, Paint::S(hex(0x3c508c)));
        }
        18 => {
            marker_box(&mut Pen::new(&mut img, 0.0, 0.0, 4.0), hex(0xdcaa6e));
            item(&mut img, 18, 15, 4, 36);
        }
        19..=21 => {
            let (sword, bg) = [(20, hex(0xe66e6e)), (17, hex(0x6e96f0)), (19, hex(0xc8aa6e))][i - 19];
            marker_box(&mut Pen::new(&mut img, 0.0, 0.0, 4.0), bg);
            text_c(&mut img, 16.0, 22.0, &format!("{}", 3 - (i - 19)), letters_on(bg));
            item(&mut img, sword, 26, 4, 36);
        }
        22 => {
            marker_box(&mut Pen::new(&mut img, 0.0, 0.0, 4.0), hex(0xa082dc));
            text_c(&mut img, 32.0, 22.0, "???", letters_on(hex(0xa082dc)));
        }
        23..=31 => {
            marker_box(&mut Pen::new(&mut img, 0.0, 0.0, 4.0), SHOP_COLOR);
            let n = [13, 22, 27, 9, 15, 21, 8, 25, 11][i - 23];
            item(&mut img, n, 14, 4, 36);
        }
        32 => {
            let mut p = Pen::new(&mut img, 0.0, 0.0, 4.0);
            marker_box(&mut p, hex(0xc8e6fa));
            bottle(&mut p, 5.0, RED_L, RED_D);
            bottle(&mut p, 11.0, BLUE_L, BLUE_D);
        }
        33 => {
            let mut p = Pen::new(&mut img, 0.0, 0.0, 4.0);
            marker_box(&mut p, hex(0xe6e6b4));
            p.line(2.4, 5.5, 6.0, 5.5, 1.1, Paint::S(black));
            item(&mut img, 29, 28, 6, 32);
        }
        34..=37 => {
            let mut p = Pen::new(&mut img, 0.0, 0.0, 4.0);
            marker_box(&mut p, hex(0xe6e6b4));
            let (l, d) = [(hex(0xb4b4bc), hex(0x60606a)), (RED_L, RED_D), (BLUE_L, BLUE_D), (GREEN_L, GREEN_D)][i - 34];
            let mut r = Img::new(32, 32);
            rupee(&mut Pen::new(&mut r, 0.0, 0.0, 2.0), l, d);
            r.outline(1.4, OUTLINE);
            img.put(&r, 28, 6);
            text_c(&mut img, 16.0, 22.0, if i == 34 { "?" } else { "+" }, black);
        }
        _ => {
            let mut p = Pen::new(&mut img, 0.0, 0.0, 4.0);
            marker_box(&mut p, hex(0x2c2c32));
            p.line(5.0, 2.4, 11.0, 8.6, 1.0, Paint::S(hex(0x9a9aa6)));
            p.line(11.0, 2.4, 5.0, 8.6, 1.0, Paint::S(hex(0x9a9aa6)));
        }
    }
    img
}

/// "s_btn_tr_dungeon_num_strip18.png": the level numbers, 0-8 = level not found yet, 9-17 = found. 36 pixels each.
fn level_number_img(i: usize) -> Img {
    let (l, found) = (i % 9, i >= 9);
    let col = LEVEL_COLORS[l];
    let mut img = Img::new(36, 36);
    {
        let mut p = Pen::new(&mut img, 0.0, 0.0, 4.0);
        p.rrect(0.1, 0.1, 8.8, 8.8, 1.6, Paint::S(col.shade(if found { 0.45 } else { 0.6 })));
        let face = if found { Paint::V(col.mix(WHITE, 0.25), col.shade(0.8), 0.8, 8.2) } else { Paint::V(hex(0x24242e), hex(0x101016), 0.8, 8.2) };
        p.rrect(0.8, 0.8, 7.4, 7.4, 1.1, face);
    }
    text_c(&mut img, 18.0, 18.0, &format!("{}", l + 1), if found { letters_on(col) } else { col });
    img
}

// ------------------------------------------------------------------------------------------------ dungeon rooms

/// "new_icons13x9.png": room pictures (13 x 9 units, 52 x 36 pixels), top row = not finished, bottom row = finished.
/// 0 unmarked, 1 plain room, 2 double moat, 3 Chevy, 4 right moat, 5 top moat, 6 circle moat, 7 vertical chute,
/// 8 horizontal chute, 9 Tee, 10 push block?, 11 item basement, 12 old man hint, 13 meat block, 14 life or money,
/// 15 bomb upgrade, 16 turnstile, 17-24 transports 1-8, 25 staircase, 26-29 start (entered from W, N, S, E),
/// 30 off the map, 31 Ganon, 32 Zelda, 33 lava moat, 34-41 numbers 1-8.
fn room_img(i: usize, done: bool) -> Img {
    let mut img = Img::new(52, 36);
    let f = if done { 0.55 } else { 1.0 };
    let water = Paint::V(hex(0x78b4ff).shade(f), hex(0x2c64d8).shade(f), 1.0, 8.0);
    let line_c = Paint::S(hex(0x3a3a46).shade(f.max(0.7)));
    let special = i >= 12 && i <= 15;
    if i == 0 {
        let mut p = Pen::new(&mut img, 0.0, 0.0, 4.0);
        p.rrect(0.1, 0.1, 12.8, 8.8, 1.2, Paint::S(hex(0x34345c)));
        p.rrect(0.6, 0.6, 11.8, 7.8, 0.9, Paint::S(hex(0x10102a)));
        return img;
    }
    {
        let mut p = Pen::new(&mut img, 0.0, 0.0, 4.0);
        if i == 30 {
            p.rrect(0.1, 0.1, 12.8, 8.8, 1.2, Paint::S(hex(0x000000)));
        } else {
            let wall = if special { RED.shade(f * 0.9) } else { hex(0x6c6e80).shade(f) };
            p.rrect(0.1, 0.1, 12.8, 8.8, 1.2, Paint::V(wall.mix(WHITE, 0.15), wall.shade(0.7), 0.0, 9.0));
            p.rrect(1.0, 1.0, 11.0, 7.0, 0.6, Paint::V(hex(0xa6a8b6).shade(f), hex(0x7e8090).shade(f), 1.0, 8.0));
        }
        match i {
            2 => {
                p.line(1.6, 2.6, 11.4, 2.6, 1.0, water);
                p.line(1.6, 6.4, 11.4, 6.4, 1.0, water);
            }
            3 => {
                p.shape((2.6, 1.6, 10.4, 7.4), water, |x, y| in_rrect(x, y, 2.6, 1.8, 7.8, 5.4, 1.6) && !in_rrect(x, y, 3.6, 2.8, 5.8, 3.4, 0.8) && (x - 6.5).abs() > 0.6);
            }
            4 => p.line(9.4, 1.6, 9.4, 7.4, 1.0, water),
            5 => p.line(1.6, 3.2, 11.4, 3.2, 1.0, water),
            6 => p.shape((2.6, 1.6, 10.4, 7.4), water, |x, y| in_rrect(x, y, 2.6, 1.8, 7.8, 5.4, 1.2) && !in_rrect(x, y, 3.6, 2.8, 5.8, 3.4, 0.6)),
            7 => {
                p.line(4.4, 1.6, 4.4, 7.4, 0.8, line_c);
                p.line(8.6, 1.6, 8.6, 7.4, 0.8, line_c);
            }
            8 => {
                p.line(1.6, 3.4, 11.4, 3.4, 0.8, line_c);
                p.line(1.6, 5.6, 11.4, 5.6, 0.8, line_c);
            }
            9 => {
                p.line(2.4, 2.4, 10.6, 2.4, 1.0, water);
                p.line(6.5, 2.4, 6.5, 7.2, 1.0, water);
            }
            10 => {
                p.rrect(5.0, 3.0, 3.0, 3.0, 0.4, Paint::V(hex(0x9a9cac).shade(f), hex(0x50525e).shade(f), 3.0, 6.0));
                for (x, y) in [(3.0, 2.4), (10.0, 2.4), (3.0, 6.6), (10.0, 6.6)] {
                    p.circle(x, y, 0.5, line_c);
                }
            }
            11 => {
                for s in 0..3 {
                    let s = s as f32;
                    p.rect(6.2 + s * 1.2, 2.6 + s * 1.5, 4.6 - s * 1.2, 1.2, line_c);
                }
            }
            12 => {
                p.poly(&[(4.4, 7.6), (8.6, 7.6), (6.5, 3.6)], Paint::V(RED_L, RED_D, 3.6, 7.6));
                p.circle(6.5, 3.2, 1.3, Paint::S(SKIN));
                p.ellipse(6.5, 4.6, 0.9, 1.0, Paint::S(WHITE));
            }
            13 => draw_item(&mut p.sub(3.3, 0.9, 7.2), 27),
            14 => {
                draw_item(&mut p.sub(1.6, 1.6, 5.8), 15);
                draw_item(&mut p.sub(6.6, 1.6, 5.8), 29);
            }
            15 => draw_item(&mut p.sub(3.4, 0.9, 7.2), 25),
            16 => {
                p.line(3.0, 4.5, 10.0, 4.5, 0.8, line_c);
                p.line(6.5, 1.6, 6.5, 7.4, 0.8, line_c);
                p.circle(6.5, 4.5, 1.0, line_c);
            }
            17..=24 | 34..=41 => p.rrect(3.7, 1.4, 5.6, 6.2, 0.8, Paint::S(hex(0x0c0c10))),
            25 => {
                for s in 0..4 {
                    let s = s as f32;
                    p.rect(2.6 + s * 1.9, 2.0 + s * 1.3, 7.8 - s * 1.9, 1.1, water);
                }
            }
            26..=29 => {
                let right = [(2.6, 3.7), (6.6, 3.7), (6.6, 2.0), (10.4, 4.5), (6.6, 7.0), (6.6, 5.3), (2.6, 5.3)];
                let pts: Vec<(f32, f32)> = right
                    .iter()
                    .map(|&(x, y)| {
                        let (dx, dy) = (x - 6.5, y - 4.5);
                        match i {
                            26 => (x, y),
                            27 => (6.5 - dy * 0.85, 4.5 + dx * 0.85),
                            28 => (6.5 + dy * 0.85, 4.5 - dx * 0.85),
                            _ => (6.5 - dx, y),
                        }
                    })
                    .collect();
                p.poly(&pts, Paint::V(GREEN_L.shade(f), GREEN_D.shade(f), 1.0, 8.0));
            }
            33 => {
                let lava = Paint::V(hex(0xffc040).shade(f), hex(0xe04010).shade(f), 1.0, 8.0);
                p.line(3.0, 2.4, 3.0, 6.8, 1.0, lava);
                p.line(10.0, 2.4, 10.0, 6.8, 1.0, lava);
                p.line(3.0, 2.4, 10.0, 2.4, 1.0, lava);
            }
            _ => {}
        }
    }
    let colour = |n: usize| [hex(0xfa3c3c), hex(0x50c8fa), hex(0xfa8228), hex(0x3cdc50), hex(0xf046dc), hex(0xfafaa0), hex(0xb4f03c), hex(0xfac83c)][n - 1].shade(if done { 0.7 } else { 1.0 });
    match i {
        11 => text(&mut img, 8, 8, "I", GOLD.shade(f)),
        17..=24 => text_c(&mut img, 26.0, 18.0, &format!("{}", i - 16), colour(i - 16)),
        34..=41 => text_c(&mut img, 26.0, 18.0, &format!("{}", i - 33), colour(i - 33)),
        30 => text_c(&mut img, 26.0, 18.0, "OFF", hex(0xf02828).shade(if done { 0.55 } else { 1.0 })),
        31 => text_c(&mut img, 26.0, 18.0, "G", hex(0x50c8fa).shade(f)),
        32 => text_c(&mut img, 26.0, 18.0, "Z", hex(0xf03c3c).shade(f)),
        _ => {}
    }
    img
}

// ------------------------------------------------------------------------------------------------ overworld map

/// Smooth random values (0..1) that change gently from place to place.
fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263) ^ seed.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0xffff) as f32 / 65535.0
}

fn noise(x: f32, y: f32, seed: u32) -> f32 {
    let (xi, yi) = (x.floor(), y.floor());
    let (fx, fy) = (x - xi, y - yi);
    let (u, v) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xi as i32, yi as i32);
    let a = hash(xi, yi, seed) + (hash(xi + 1, yi, seed) - hash(xi, yi, seed)) * u;
    let b = hash(xi, yi + 1, seed) + (hash(xi + 1, yi + 1, seed) - hash(xi, yi + 1, seed)) * u;
    a + (b - a) * v
}

/// The ground kind (see TERRAIN) at a point of the map, in map units (256 x 88 for the whole overworld).
fn kind_at(x: f32, y: f32) -> u8 {
    let (x, y) = (x.clamp(0.0, 255.99), y.clamp(0.0, 87.99));
    let sy = (y / 11.0) as usize;
    let yy = y - sy as f32 * 11.0;
    let by = if yy < 4.0 { 0 } else if yy < 8.0 { 1 } else { 2 };
    TERRAIN[sy * 3 + by].as_bytes()[(x / 4.0) as usize]
}

include!(concat!(env!("OUT_DIR"), "/overworld_tiles.rs"));

/// "s_map_overworld_strip8.png": the overworld, 16 x 8 screens of 16 x 11 units. Made from the map screen pictures
/// in assets/overworld (first quest, 48 x 33 pixels each, so 3 pixels per unit), which are built into the program; the
/// other overworlds use the same pictures. Any screen without a picture is drawn by `drawn_overworld` instead.
/// Returns the picture and its pixels per unit.
fn overworld() -> (Img, usize) {
    let decode = |bytes: &[u8]| image::load_from_memory_with_format(bytes, image::ImageFormat::Png).ok().map(|i| i.to_rgba8());
    let tiles: Vec<(usize, usize, image::RgbaImage)> = OVERWORLD_TILES
        .iter()
        .filter(|t| t.0 == 0 && t.1 < 16 && t.2 < 8)
        .filter_map(|&(_, x, y, bytes)| decode(bytes).map(|img| (x, y, img)))
        .collect();
    let scale = tiles.first().map(|t| t.2.width() as usize / 16).filter(|&s| s > 0).unwrap_or(4);
    let tiles: Vec<_> = tiles.into_iter().filter(|t| t.2.width() as usize == 16 * scale && t.2.height() as usize == 11 * scale).collect();
    let mut img = if tiles.len() == 16 * 8 { Img::new(256 * scale, 88 * scale) } else { drawn_overworld(scale) };
    for (x, y, tile) in &tiles {
        for (px, py, p) in tile.enumerate_pixels() {
            let [r, g, b, a] = p.0;
            let i = (y * 11 * scale + py as usize) * img.w + x * 16 * scale + px as usize;
            img.px[i] = C(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0);
        }
    }
    (img, scale)
}

/// "magnifier": the enlarged map screens (assets/magnifier, first quest, 128 x 88 pixels each = 8 pixels per unit),
/// put together like the map: 16 x 8 screens of 16 x 11 units. None if there are no such pictures.
fn magnifier() -> Option<(Img, usize)> {
    let tiles: Vec<(usize, usize, image::RgbaImage)> = MAGNIFIER_TILES
        .iter()
        .filter(|t| t.0 == 0 && t.1 < 16 && t.2 < 8)
        .filter_map(|&(_, x, y, bytes)| image::load_from_memory_with_format(bytes, image::ImageFormat::Png).ok().map(|i| (x, y, i.to_rgba8())))
        .collect();
    let scale = tiles.first()?.2.width() as usize / 16;
    let mut img = Img::new(256 * scale, 88 * scale);
    for (x, y, tile) in &tiles {
        if tile.width() as usize != 16 * scale || tile.height() as usize != 11 * scale {
            continue;
        }
        for (px, py, p) in tile.enumerate_pixels() {
            let [r, g, b, a] = p.0;
            let i = (y * 11 * scale + py as usize) * img.w + x * 16 * scale + px as usize;
            img.px[i] = C(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0);
        }
    }
    Some((img, scale))
}

/// The tracker's own drawing of the overworld (16 x 8 screens of 16 x 11 units), `scale` pixels per unit, for screens
/// that have no picture in assets/overworld. The edges
/// between kinds of ground are bent a little with smooth noise so they look natural; forests get trees, rock gets
/// boulders, the graveyard gravestones, water a shore and waves, sand ripples.
fn drawn_overworld(scale: usize) -> Img {
    let (w, h) = (256 * scale, 88 * scale);
    let s = scale as f32;
    // the ground kind of every pixel, with bent edges
    let mut kinds = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let (mx, my) = ((x as f32 + 0.5) / s, (y as f32 + 0.5) / s);
            let dx = (noise(mx * 0.42, my * 0.42, 1) - 0.5) * 3.0 + (noise(mx * 1.3, my * 1.3, 3) - 0.5) * 0.9;
            let dy = (noise(mx * 0.42, my * 0.42, 2) - 0.5) * 3.0 + (noise(mx * 1.3, my * 1.3, 4) - 0.5) * 0.9;
            kinds[y * w + x] = kind_at(mx + dx, my + dy);
        }
    }
    let kind = |x: i32, y: i32| kinds[(y.clamp(0, h as i32 - 1) as usize) * w + x.clamp(0, w as i32 - 1) as usize];
    // round things (trees, boulders) on a jittered grid: the nearest one covering (x, y), if any
    let blob = |x: f32, y: f32, spacing: f32, radius: f32, seed: u32, want: u8| -> Option<(f32, f32, f32)> {
        let (gx, gy) = ((x / spacing).floor() as i32, (y / spacing).floor() as i32);
        let mut best: Option<(f32, f32, f32)> = None;
        for cy in gy - 1..=gy + 1 {
            for cx in gx - 1..=gx + 1 {
                let px = (cx as f32 + 0.2 + hash(cx, cy, seed) * 0.6) * spacing;
                let py = (cy as f32 + 0.2 + hash(cx, cy, seed + 7) * 0.6) * spacing;
                let r = radius * (0.8 + hash(cx, cy, seed + 13) * 0.4);
                let (dx, dy) = (x - px, y - py);
                let d = (dx * dx + dy * dy).sqrt();
                if d < r && kind(px as i32, py as i32) == want && best.is_none_or(|b| py > b.1) {
                    best = Some((dx / r, dy / r, py));
                }
            }
        }
        best
    };
    let mut img = Img::new(w, h);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let k = kind(x, y);
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let n = noise(fx * 0.12, fy * 0.12, 9);
            let fine = hash(x, y, 11);
            let mut c = match k {
                b'W' => {
                    let near = |d: i32| (-1..=1).any(|j| (-1..=1).any(|i| kind(x + i * d, y + j * d) != b'W'));
                    if near(1) {
                        hex(0xd6ecff)
                    } else if near(3) {
                        hex(0x6aa8ec)
                    } else if near(6) {
                        hex(0x3f7fd6)
                    } else {
                        let wave = ((fx * 0.35 + fy * 0.9 + n * 9.0).sin() > 0.93) as u8 as f32;
                        hex(0x2c62bc).mix(hex(0x6a9ce4), wave * 0.6 + n * 0.15)
                    }
                }
                b'F' => match blob(fx, fy, 7.0, 4.2, 21, b'F') {
                    Some((dx, dy, _)) => {
                        let lit = (-dx - dy) * 0.5;
                        let d = (dx * dx + dy * dy).sqrt();
                        hex(0x3e9a44).mix(hex(0x8ad070), lit.max(0.0) * 0.8).mix(hex(0x1a5422), ((d - 0.75) * 4.0).max(0.0))
                    }
                    None => hex(0x2a6a30).mix(hex(0x1c4c22), n),
                },
                b'R' => {
                    let ground = hex(0x8e5e34).mix(hex(0xa8744a), n).mix(hex(0x6a4224), (fine > 0.9) as u8 as f32 * 0.5);
                    match blob(fx, fy, 9.0, 4.4, 31, b'R') {
                        Some((dx, dy, _)) => {
                            let lit = (-dx * 0.6 - dy) * 0.5;
                            let d = (dx * dx + dy * dy).sqrt();
                            hex(0x9c6c44).mix(hex(0xd4a878), lit.max(0.0)).mix(hex(0x4a2e16), ((d - 0.7) * 3.3).max(0.0))
                        }
                        None => ground,
                    }
                }
                b'G' => {
                    let ground = hex(0x6c7466).mix(hex(0x5a6256), n);
                    let (cx, cy) = ((fx / 8.0).floor(), (fy / 8.0).floor());
                    let (sx, sy) = (fx - cx * 8.0, fy - cy * 8.0);
                    let stone = in_rrect(sx, sy, 2.4, 1.6, 3.2, 4.6, 1.2) && kind((cx * 8.0 + 4.0) as i32, (cy * 8.0 + 4.0) as i32) == b'G';
                    let shadow = in_rrect(sx, sy, 3.2, 2.6, 3.2, 4.6, 1.2) && kind((cx * 8.0 + 4.0) as i32, (cy * 8.0 + 4.0) as i32) == b'G';
                    if stone {
                        hex(0xe4e4ea).mix(hex(0x9a9aa4), (sx - 2.4) / 3.2)
                    } else if shadow {
                        ground.shade(0.7)
                    } else {
                        ground
                    }
                }
                b'S' => {
                    let ripple = ((fx * 0.25 + fy * 0.8 + n * 7.0).sin() * 0.5 + 0.5).powi(6);
                    hex(0xdcbc7c).mix(hex(0xb8965a), ripple * 0.6).mix(hex(0xf0d8a0), (fine > 0.94) as u8 as f32 * 0.6)
                }
                _ => {
                    let tuft = (fine > 0.965) as u8 as f32;
                    hex(0xc8b88e).mix(hex(0xb4a678), n * 0.7).mix(hex(0x8e9a5a), tuft * 0.7)
                }
            };
            // a darker line where one kind of land meets another, lighter on the side facing the light
            if k != b'W' {
                let (r, d) = (kind(x + 1, y), kind(x, y + 1));
                if (r != k && r != b'W') || (d != k && d != b'W') {
                    c = c.shade(0.72);
                }
            }
            img.px[y as usize * w + x as usize] = c;
        }
    }
    img
}

// ------------------------------------------------------------------------------------------------ logo

/// "ZTlogo64x64.png" (at 128 pixels) and the window icon: a gold triforce over "ZT" on a dark blue tile.
pub fn logo() -> Img {
    let mut img = Img::new(128, 128);
    {
        let mut p = Pen::new(&mut img, 0.0, 0.0, 8.0);
        p.rrect(0.2, 0.2, 15.6, 15.6, 3.0, Paint::S(hex(0xc89a30)));
        p.rrect(0.6, 0.6, 14.8, 14.8, 2.7, Paint::V(hex(0x2a3c8c), hex(0x0c1236), 0.6, 15.4));
        for (x, y) in [(8.0, 1.6), (4.6, 6.6), (11.4, 6.6)] {
            p.poly(&[(x, y), (x + 3.3, y + 5.0), (x - 3.3, y + 5.0)], Paint::V(GOLD_L, GOLD_D, y, y + 5.0));
            p.poly(&[(x, y + 1.6), (x + 1.9, y + 4.4), (x - 1.9, y + 4.4)], Paint::V(WHITE.alpha(0.55), GOLD_L.alpha(0.0), y + 1.6, y + 4.4));
        }
    }
    text_c(&mut img, 64.0 + 1.0, 112.0 + 1.0, "ZT", hex(0x05060c));
    text_c(&mut img, 64.0, 112.0, "ZT", WHITE);
    img
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
            let [r, g, b, a] = px.to_srgba_unmultiplied();
            let v = grey_level(Color32::from_rgb(r, g, b));
            *px = Color32::from_rgba_unmultiplied(v, v, v, a);
        }
    }
    out
}

/// Pictures side by side.
fn strip(cells: Vec<Img>) -> Img {
    let w = cells.iter().map(|c| c.w).sum();
    let h = cells.iter().map(|c| c.h).max().unwrap_or(0);
    let mut out = Img::new(w, h);
    let mut x = 0;
    for c in cells {
        out.put(&c, x, 0);
        x += c.w;
    }
    out
}

/// The item pictures in grey, for items not had yet in the progress row.
pub const GREY_ITEMS: &str = "icons7x7 grey";
/// The map markers in the original's grey (the Spot Summary's empty shop slots: Graphics.greyscale of the hint shop).
pub const GREY_MARKERS: &str = "markers grey";

/// Every picture sheet: its name, the picture, how many pixels make one of the units the drawing code uses, and
/// whether it is pixel art (shown with sharp pixels when enlarged).
pub fn sheets() -> Vec<(&'static str, ColorImage, f32, bool)> {
    let markers = strip((0..39).map(marker_img).collect()).image();
    let (map, map_scale) = overworld();
    let items = strip((0..32).map(|i| item_img(i, 28)).collect()).image();
    let grey = greyscale(&items);
    let shop3 = strip(
        SHOP3
            .iter()
            .map(|rows| {
                let mut a = smooth_art(rows, 2);
                a.outline(1.0, OUTLINE);
                a
            })
            .collect(),
    );
    let mut heart8 = Img::new(64, 32);
    heart8.put(&item_img(15, 32), 0, 0);
    let mut rooms = Img::new(42 * 52, 72);
    for i in 0..42 {
        rooms.put(&room_img(i, false), i * 52, 0);
        rooms.put(&room_img(i, true), i * 52, 36);
    }
    let mut sheets = vec![
        ("icons7x7.png", items, 4.0, false),
        (GREY_ITEMS, grey, 4.0, false),
        ("icons3x7.png", shop3.image(), 4.0, false),
        ("icons8x8.png", heart8.image(), 4.0, false),
        ("icons10x10.png", strip((0..10).map(icon10).collect()).image(), 4.0, false),
        ("zelda_bosses16x16.png", strip((0..32).map(monster_img).collect()).image(), 2.0, false),
        ("zelda_items16x16.png", strip((0..13).map(floor_img).collect()).image(), 2.0, false),
        ("s_icon_overworld_strip39.png", markers.clone(), 4.0, false),
        (GREY_MARKERS, greyscale(&markers), 4.0, false),
        ("s_btn_tr_dungeon_num_strip18.png", strip((0..18).map(level_number_img).collect()).image(), 4.0, false),
        ("new_icons13x9.png", rooms.image(), 4.0, false),
        ("s_map_overworld_strip8.png", map.image(), map_scale as f32, true),
        ("ZTlogo64x64.png", logo().image(), 2.0, false),
    ];
    // the original's vanilla dungeon item reference pictures (Graphics.firstQuestItemReferenceBMP / second…), copied
    // into assets/reference from Z-Tracker 2.1.0 and built into the program
    for (name, bytes) in [
        (REFERENCE_1Q, &include_bytes!("../assets/reference/first-quest-item-reference.png")[..]),
        (REFERENCE_2Q, &include_bytes!("../assets/reference/second-quest-item-reference.png")[..]),
    ] {
        if let Ok(img) = image::load_from_memory_with_format(bytes, image::ImageFormat::Png) {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            sheets.push((name, ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()), 1.0, true));
        }
    }
    if let Some((mag, scale)) = magnifier() {
        sheets.push((MAGNIFIER, mag.image(), scale as f32, true));
    }
    sheets
}

/// The vanilla dungeon item reference pictures (first and second quest), 270 x 117 each.
pub const REFERENCE_1Q: &str = "first-quest-item-reference";
pub const REFERENCE_2Q: &str = "second-quest-item-reference";

/// The sheet of enlarged map screens for the magnifier.
pub const MAGNIFIER: &str = "magnifier";

/// The window, taskbar and Dock icon: the logo.
pub fn window_icon() -> eframe::egui::IconData {
    let img = logo().image();
    let rgba = img.pixels.iter().flat_map(|p| p.to_srgba_unmultiplied()).collect();
    eframe::egui::IconData { width: img.size[0] as u32, height: img.size[1] as u32, rgba }
}

// ------------------------------------------------------------------------------------------------ patterns
/// The colours of the pixel patterns.
fn ink(ch: char) -> Option<Color32> {
    let rgb = Color32::from_rgb;
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

/// "zelda_bosses16x16.png": 8x8 patterns, smoothed to 32x32. Order: 0 Digdogger, 1 Gleeok, 2 Gohma, 3 Manhandla,
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

#[cfg(test)]
mod tests {
    #[test]
    fn every_sheet_has_the_size_the_drawing_code_expects() {
        // in the drawing code's units (the picture's pixels divided by the sheet's scale)
        let want = [
            ("icons7x7.png", [224, 7]),
            ("icons3x7.png", [24, 7]),
            ("icons8x8.png", [16, 8]),
            ("icons10x10.png", [100, 10]),
            ("zelda_bosses16x16.png", [512, 16]),
            ("zelda_items16x16.png", [208, 16]),
            ("s_icon_overworld_strip39.png", [624, 11]),
            ("s_btn_tr_dungeon_num_strip18.png", [162, 9]),
            ("new_icons13x9.png", [546, 18]),
            ("s_map_overworld_strip8.png", [256, 88]),
            ("ZTlogo64x64.png", [64, 64]),
        ];
        let sheets = super::sheets();
        for (name, size) in want {
            let s = sheets.iter().find(|s| s.0 == name).unwrap_or_else(|| panic!("{name} missing"));
            let units = [(s.1.size[0] as f32 / s.2) as usize, (s.1.size[1] as f32 / s.2) as usize];
            assert_eq!(units, size, "{name}");
        }
    }

    #[test]
    fn black_or_white_letters_follow_the_originals_rule() {
        use super::{black_is_good_contrast, hex};
        assert!(black_is_good_contrast(hex(0xffffff)));
        assert!(!black_is_good_contrast(hex(0x000000)));
        assert!(black_is_good_contrast(hex(0xf0dc28))); // yellow
        assert!(!black_is_good_contrast(hex(0x0000ff))); // blue: 0.0722 < 0.25
        assert!(!black_is_good_contrast(hex(0xff0000))); // red: 0.2126 < 0.25
        assert!(black_is_good_contrast(hex(0x00ff00))); // green: 0.7152 > 0.25
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
    fn the_map_is_made_from_the_overworld_pictures() {
        assert_eq!(super::OVERWORLD_TILES.len(), 128);
        let (map, scale) = super::overworld();
        assert_eq!((scale, map.w, map.h), (3, 768, 264));
        // every pixel came from a picture (nothing see-through left)
        assert!(map.px.iter().all(|c| c.3 > 0.0));
    }

    #[test]
    fn the_magnifier_has_every_screen_at_8_pixels_per_unit() {
        assert_eq!(super::MAGNIFIER_TILES.len(), 128);
        let (img, scale) = super::magnifier().unwrap();
        assert_eq!((scale, img.w, img.h), (8, 2048, 704));
        assert!(img.px.iter().all(|c| c.3 > 0.0));
    }

    #[test]
    fn every_pattern_is_the_right_size() {
        for (i, p) in super::MONSTERS8.iter().enumerate() {
            assert!(p.iter().all(|r| r.len() == 8), "monster {i}");
        }
        for p in super::SHOP3.iter() {
            assert!(p.iter().all(|r| r.len() == 3));
        }
        assert!(super::TERRAIN.iter().all(|r| r.len() == 64 && r.bytes().all(|b| b"RWFGSO".contains(&b))));
    }

    #[test]
    fn shapes_are_smooth_at_the_edges() {
        // a circle's edge pixels are partly covered, not just on or off
        let mut img = super::Img::new(20, 20);
        super::Pen::new(&mut img, 0.0, 0.0, 1.0).circle(10.0, 10.0, 7.3, super::Paint::S(super::WHITE));
        assert!(img.px.iter().any(|c| c.3 > 0.1 && c.3 < 0.9));
        assert_eq!(img.px[10 * 20 + 10].3, 1.0);
    }

    /// Saves every sheet as a PNG to look at: Z1R_ART_DIR=/some/folder cargo test -- --ignored save_sheets
    #[test]
    #[ignore]
    fn save_sheets() {
        let dir = std::path::PathBuf::from(std::env::var("Z1R_ART_DIR").expect("set Z1R_ART_DIR"));
        for (name, img, _, _) in super::sheets() {
            let rgba: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect();
            image::save_buffer(dir.join(format!("{}.png", name.trim_end_matches(".png").replace(' ', "_"))), &rgba, img.size[0] as u32, img.size[1] as u32, image::ColorType::Rgba8).unwrap();
        }
    }
}
