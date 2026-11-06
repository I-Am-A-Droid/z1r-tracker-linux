//! The tracker's pictures (map, icons, monsters) are not shipped with this app.
//! They are read at start-up from the person's own copy of the original Z1R_WPF.exe
//! (its embedded files are read; nothing in it is run). If no copy is found, the
//! tracker still works and draws simple shapes and letters instead.

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, TextureHandle, TextureOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn logo_from_ico(bytes: &[u8]) -> Option<egui::IconData> {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Ico).ok()?.to_rgba8();
    Some(egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() })
}

pub struct Sheet {
    pub image: ColorImage,
    pub texture: Option<TextureHandle>,
}

#[derive(Default)]
pub struct Assets {
    pub original: Option<PathBuf>,
    sheets: HashMap<String, Sheet>,
}

/// The original's logo, which it gives its "Launch Z-Tracker" shortcut (ZTracker.lnk): icons/ztlogo64x64.ico in the
/// original's folder (Program/MyWindow::.ctor, token 0x0600004d, RVA 0x50f0, IShellLink::SetIconLocation).
pub const LOGO_FILE: &str = "icons/ztlogo64x64.ico";

impl Assets {
    pub fn loaded(&self) -> bool {
        !self.sheets.is_empty()
    }

    /// The original's logo for the window and taskbar, read from the folder of the person's Z1R_WPF.exe (the best
    /// full-colour picture in the .ico: 48x48, as its 64x64 one has only 16 colours). None when that file isn't
    /// there; the system's usual icon is used then.
    pub fn window_icon(&self) -> Option<egui::IconData> {
        let bytes = std::fs::read(self.original.as_ref()?.parent()?.join(LOGO_FILE)).ok()?;
        logo_from_ico(&bytes)
    }

    pub fn candidate_paths(explicit: Option<&str>) -> Vec<PathBuf> {
        let mut v = vec![];
        if let Some(p) = explicit.filter(|p| !p.is_empty()) {
            v.push(PathBuf::from(p));
        }
        if let Ok(p) = std::env::var("Z1R_ORIGINAL") {
            v.push(PathBuf::from(p));
        }
        if let Some(d) = crate::platform::program_dir() {
            v.push(d.join("Z1R_WPF.exe"));
        }
        if let Ok(d) = std::env::current_dir() {
            v.push(d.join("Z1R_WPF.exe"));
        }
        {
            let home = crate::platform::home();
            for root in [home.join("Downloads"), home.clone(), home.join("Games"), home.join("Desktop"), PathBuf::from("/opt")] {
                let Ok(rd) = std::fs::read_dir(&root) else { continue };
                for e in rd.flatten() {
                    let name = e.file_name().to_string_lossy().to_lowercase();
                    if !(name.contains("tracker") || name.contains("z1r")) || !e.path().is_dir() {
                        continue;
                    }
                    v.push(e.path().join("Z1R_WPF.exe"));
                    if let Ok(sub) = std::fs::read_dir(e.path()) {
                        for s in sub.flatten() {
                            if s.path().is_dir() {
                                v.push(s.path().join("Z1R_WPF.exe"));
                            }
                        }
                    }
                }
            }
        }
        v
    }

    pub fn try_load(&mut self, explicit: Option<&str>) -> bool {
        for p in Self::candidate_paths(explicit) {
            if !p.is_file() {
                continue;
            }
            match read_manifest_resources(&p) {
                Ok(res) if res.contains_key("s_map_overworld_strip8.png") => {
                    self.sheets.clear();
                    for (name, bytes) in res {
                        if !name.to_lowercase().ends_with(".png") {
                            continue;
                        }
                        if let Ok(img) = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png) {
                            let rgba = img.to_rgba8();
                            let size = [rgba.width() as usize, rgba.height() as usize];
                            let mut image = ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                            if name == "s_icon_overworld_strip39.png" {
                                remove_shop_letter_b(&mut image);
                            }
                            if name == "icons3x7.png" {
                                // black is "see-through" in the shop item pictures (makeTwoItemShopBmp skips black pixels)
                                for px in image.pixels.iter_mut() {
                                    if px.r() == 0 && px.g() == 0 && px.b() == 0 {
                                        *px = Color32::TRANSPARENT;
                                    }
                                }
                            }
                            self.sheets.insert(name, Sheet { image, texture: None });
                        }
                    }
                    self.original = Some(p);
                    return true;
                }
                Ok(_) => {}
                Err(e) => eprintln!("Could not read pictures from {}: {e}", p.display()),
            }
        }
        false
    }

    fn texture(&mut self, ctx: &egui::Context, sheet: &str) -> Option<(egui::TextureId, [usize; 2])> {
        let s = self.sheets.get_mut(sheet)?;
        let size = s.image.size;
        if s.texture.is_none() {
            s.texture = Some(ctx.load_texture(sheet, s.image.clone(), TextureOptions::NEAREST));
        }
        Some((s.texture.as_ref().unwrap().id(), size))
    }

    /// Draws part of a picture sheet (pixel rectangle sx,sy,sw,sh) into `dst`.
    /// `shade` < 1 darkens; `mirror` flips left-right.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&mut self, painter: &egui::Painter, sheet: &str, sx: f32, sy: f32, sw: f32, sh: f32, dst: Rect, shade: f32, mirror: bool) -> bool {
        let Some((id, size)) = self.texture(painter.ctx(), sheet) else { return false };
        let (w, h) = (size[0] as f32, size[1] as f32);
        if sx < 0.0 || sy < 0.0 || sx + sw > w + 0.01 || sy + sh > h + 0.01 {
            return false;
        }
        let (mut u0, mut u1) = (sx / w, (sx + sw) / w);
        if mirror {
            std::mem::swap(&mut u0, &mut u1);
        }
        let uv = Rect::from_min_max(Pos2::new(u0, sy / h), Pos2::new(u1, (sy + sh) / h));
        let v = (shade.clamp(0.0, 1.0) * 255.0) as u8;
        painter.image(id, dst, uv, Color32::from_rgb(v, v, v));
        true
    }

    // Convenience drawing helpers for the sheets the tracker uses. Each returns false if
    // the picture isn't available, so the caller can draw a stand-in.
    pub fn item7(&mut self, p: &egui::Painter, i: i32, dst: Rect, shade: f32) -> bool {
        i >= 0 && self.draw(p, "icons7x7.png", i as f32 * 7.0, 0.0, 7.0, 7.0, dst, shade, false)
    }
    /// Shop item picture (3x7 game pixels): 0 arrow, 1 bomb, 2 book, 3 candle, 4 blue ring, 5 meat, 6 key, 7 shield.
    pub fn shop_item(&mut self, p: &egui::Painter, i: i32, dst: Rect) -> bool {
        (0..8).contains(&i) && self.draw(p, "icons3x7.png", i as f32 * 3.0, 0.0, 3.0, 7.0, dst, 1.0, false)
    }
    pub fn icon10(&mut self, p: &egui::Painter, i: i32, dst: Rect, shade: f32) -> bool {
        self.draw(p, "icons10x10.png", i as f32 * 10.0, 0.0, 10.0, 10.0, dst, shade, false)
    }
    pub fn boss16(&mut self, p: &egui::Painter, i: i32, dst: Rect) -> bool {
        i >= 0 && self.draw(p, "zelda_bosses16x16.png", i as f32 * 16.0, 0.0, 16.0, 16.0, dst, 1.0, false)
    }
    pub fn zitem16(&mut self, p: &egui::Painter, i: i32, dst: Rect, shade: f32) -> bool {
        i >= 0 && self.draw(p, "zelda_items16x16.png", i as f32 * 16.0, 0.0, 16.0, 16.0, dst, shade, false)
    }
    pub fn ow_icon(&mut self, p: &egui::Painter, i: i32, dst: Rect) -> bool {
        i >= 0 && self.draw(p, "s_icon_overworld_strip39.png", i as f32 * 16.0, 0.0, 16.0, 11.0, dst, 1.0, false)
    }
    pub fn dungeon_num(&mut self, p: &egui::Painter, level: usize, found: bool, dst: Rect) -> bool {
        let i = level + if found { 9 } else { 0 };
        self.draw(p, "s_btn_tr_dungeon_num_strip18.png", i as f32 * 9.0, 0.0, 9.0, 9.0, dst, 1.0, false)
    }
    pub fn room(&mut self, p: &egui::Painter, icon: i32, completed: bool, dst: Rect) -> bool {
        self.draw(p, "new_icons13x9.png", icon as f32 * 13.0, if completed { 9.0 } else { 0.0 }, 13.0, 9.0, dst, 1.0, false)
    }
    pub fn heart8(&mut self, p: &egui::Painter, dst: Rect) -> bool {
        self.draw(p, "icons8x8.png", 0.0, 0.0, 8.0, 8.0, dst, 1.0, false)
    }
    pub fn logo(&mut self, p: &egui::Painter, dst: Rect) -> bool {
        self.draw(p, "ZTlogo64x64.png", 0.0, 0.0, 64.0, 64.0, dst, 1.0, false)
    }
    /// One overworld screen from the map strip (4 maps of 256x88: 1Q, 2Q, mixed-1Q, mixed-2Q).
    pub fn map_screen(&mut self, p: &egui::Painter, quest: i64, x: usize, y: usize, dst: Rect, shade: f32, mirror: bool) -> bool {
        let q = quest.clamp(0, 3) as f32;
        self.draw(p, "s_map_overworld_strip8.png", q * 256.0 + x as f32 * 16.0, y as f32 * 11.0, 16.0, 11.0, dst, shade, mirror)
    }
}

/// The shop markers (pictures 23-31 of the 16x11 marker strip) all start with a black letter "B" (for "buy")
/// in columns 1-3, rows 3-7. On request it is removed: those black pixels take the marker's own background colour.
pub const SHOP_STRIP_FIRST: usize = 23;
pub const SHOP_STRIP_LAST: usize = 31;
fn remove_shop_letter_b(img: &mut ColorImage) {
    let w = img.size[0];
    if img.size[1] < 8 || w < (SHOP_STRIP_LAST + 1) * 16 {
        return;
    }
    for icon in SHOP_STRIP_FIRST..=SHOP_STRIP_LAST {
        let x0 = icon * 16;
        for y in 3..=7 {
            let background = img.pixels[y * w + x0];
            for x in x0 + 1..=x0 + 3 {
                let px = &mut img.pixels[y * w + x];
                if px.r() == 0 && px.g() == 0 && px.b() == 0 && px.a() > 0 {
                    *px = background;
                }
            }
        }
    }
}

// ---------------------------------------------------------------- reading a .NET program's embedded files

fn u16_at(b: &[u8], o: usize) -> Result<u16, String> {
    b.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]])).ok_or_else(|| "file too short".to_string())
}
fn u32_at(b: &[u8], o: usize) -> Result<u32, String> {
    b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])).ok_or_else(|| "file too short".to_string())
}

/// Returns name -> bytes for every resource embedded in a .NET program (ECMA-335 ManifestResource table).
pub fn read_manifest_resources(path: &Path) -> Result<HashMap<String, Vec<u8>>, String> {
    let b = std::fs::read(path).map_err(|e| e.to_string())?;
    if b.get(0..2) != Some(b"MZ") {
        return Err("not a Windows program".into());
    }
    let pe = u32_at(&b, 0x3c)? as usize;
    if b.get(pe..pe + 4) != Some(b"PE\0\0") {
        return Err("no PE header".into());
    }
    let nsec = u16_at(&b, pe + 6)? as usize;
    let opt_size = u16_at(&b, pe + 20)? as usize;
    let opt = pe + 24;
    let magic = u16_at(&b, opt)?;
    let dirs = opt + if magic == 0x20b { 112 } else { 96 };
    let cli_rva = u32_at(&b, dirs + 14 * 8)? as usize;
    if cli_rva == 0 {
        return Err("not a .NET program".into());
    }
    let sections: Vec<(usize, usize, usize, usize)> = (0..nsec)
        .map(|i| {
            let s = opt + opt_size + i * 40;
            Ok((u32_at(&b, s + 12)? as usize, u32_at(&b, s + 8)? as usize, u32_at(&b, s + 20)? as usize, u32_at(&b, s + 16)? as usize))
        })
        .collect::<Result<_, String>>()?;
    let rva_to_off = |rva: usize| -> Result<usize, String> {
        for &(va, vs, raw, rs) in &sections {
            if rva >= va && rva < va + vs.max(rs) {
                return Ok(rva - va + raw);
            }
        }
        Err("address outside the file".into())
    };
    let cli = rva_to_off(cli_rva)?;
    let md = rva_to_off(u32_at(&b, cli + 8)? as usize)?;
    let res_rva = u32_at(&b, cli + 24)? as usize;
    if u32_at(&b, md)? != 0x424A_5342 {
        return Err("no .NET metadata".into());
    }
    let ver_len = u32_at(&b, md + 12)? as usize;
    let mut p = md + 16 + ver_len + 2;
    let nstreams = u16_at(&b, p)? as usize;
    p += 2;
    let (mut tables, mut strings) = (None, None);
    for _ in 0..nstreams {
        let off = u32_at(&b, p)? as usize;
        let size = u32_at(&b, p + 4)? as usize;
        let mut q = p + 8;
        let start = q;
        while q < b.len() && b[q] != 0 {
            q += 1;
        }
        let name = String::from_utf8_lossy(&b[start..q]).to_string();
        q += 1;
        q = (q + 3) & !3;
        match name.as_str() {
            "#~" | "#-" => tables = Some((md + off, size)),
            "#Strings" => strings = Some(md + off),
            _ => {}
        }
        p = q;
    }
    let (t, _) = tables.ok_or("no metadata tables")?;
    let strings = strings.ok_or("no string heap")?;
    let heap_sizes = b[t + 6];
    let valid = u32_at(&b, t + 8)? as u64 | ((u32_at(&b, t + 12)? as u64) << 32);
    let mut rows = [0usize; 64];
    let mut q = t + 24;
    for (i, r) in rows.iter_mut().enumerate() {
        if valid & (1u64 << i) != 0 {
            *r = u32_at(&b, q)? as usize;
            q += 4;
        }
    }
    let str_idx = if heap_sizes & 1 != 0 { 4 } else { 2 };
    let guid_idx = if heap_sizes & 2 != 0 { 4 } else { 2 };
    let blob_idx = if heap_sizes & 4 != 0 { 4 } else { 2 };
    let idx = |table: usize| if rows[table] < 65536 { 2 } else { 4 };
    let coded = |tabs: &[usize], bits: u32| {
        let max = tabs.iter().map(|&t| if t == usize::MAX { 0 } else { rows[t] }).max().unwrap_or(0);
        if max < (1usize << (16 - bits)) { 2 } else { 4 }
    };
    let none = usize::MAX;
    let type_def_or_ref = coded(&[2, 1, 0x1b], 2);
    let has_constant = coded(&[4, 8, 0x17], 2);
    let has_ca = coded(&[6, 4, 1, 2, 8, 9, 10, 0, 0x0e, 0x17, 0x14, 0x11, 0x1a, 0x1b, 0x20, 0x23, 0x26, 0x27, 0x28, 0x2a, 0x2c, 0x2b], 5);
    let has_field_marshal = coded(&[4, 8], 1);
    let has_decl_security = coded(&[2, 6, 0x20], 2);
    let member_ref_parent = coded(&[2, 1, 0x1a, 6, 0x1b], 3);
    let has_semantics = coded(&[0x14, 0x17], 1);
    let method_def_or_ref = coded(&[6, 10], 1);
    let member_forwarded = coded(&[4, 6], 1);
    let implementation = coded(&[0x26, 0x23, 0x27], 2);
    let ca_type = coded(&[none, none, 6, 10, none], 3);
    let resolution_scope = coded(&[0, 0x1a, 0x23, 1], 2);
    let row_size = |table: usize| -> usize {
        match table {
            0x00 => 2 + str_idx + guid_idx * 3,
            0x01 => resolution_scope + str_idx * 2,
            0x02 => 4 + str_idx * 2 + type_def_or_ref + idx(4) + idx(6),
            0x03 => idx(4),
            0x04 => 2 + str_idx + blob_idx,
            0x05 => idx(6),
            0x06 => 4 + 2 + 2 + str_idx + blob_idx + idx(8),
            0x07 => idx(8),
            0x08 => 2 + 2 + str_idx,
            0x09 => idx(2) + type_def_or_ref,
            0x0a => member_ref_parent + str_idx + blob_idx,
            0x0b => 2 + has_constant + blob_idx,
            0x0c => has_ca + ca_type + blob_idx,
            0x0d => has_field_marshal + blob_idx,
            0x0e => 2 + has_decl_security + blob_idx,
            0x0f => 2 + 4 + idx(2),
            0x10 => 4 + idx(4),
            0x11 => blob_idx,
            0x12 => idx(2) + idx(0x14),
            0x13 => idx(0x14),
            0x14 => 2 + str_idx + type_def_or_ref,
            0x15 => idx(2) + idx(0x17),
            0x16 => idx(0x17),
            0x17 => 2 + str_idx + blob_idx,
            0x18 => 2 + idx(6) + has_semantics,
            0x19 => idx(2) + method_def_or_ref * 2,
            0x1a => str_idx,
            0x1b => blob_idx,
            0x1c => 2 + member_forwarded + str_idx + idx(0x1a),
            0x1d => 4 + idx(4),
            0x1e => 8,
            0x1f => 4,
            0x20 => 4 + 8 + 4 + blob_idx + str_idx * 2,
            0x21 => 4,
            0x22 => 12,
            0x23 => 8 + 4 + blob_idx + str_idx * 2 + blob_idx,
            0x24 => 4 + idx(0x23),
            0x25 => 12 + idx(0x23),
            0x26 => 4 + str_idx + blob_idx,
            0x27 => 8 + str_idx * 2 + implementation,
            _ => 0,
        }
    };
    let mut tp = q;
    for table in 0..0x28 {
        tp += rows[table] * row_size(table);
    }
    let read_str = |o: usize| -> String {
        let s = strings + o;
        let mut e = s;
        while e < b.len() && b[e] != 0 {
            e += 1;
        }
        String::from_utf8_lossy(&b[s.min(b.len())..e]).to_string()
    };
    let read_idx = |o: usize, size: usize| -> Result<usize, String> { Ok(if size == 2 { u16_at(&b, o)? as usize } else { u32_at(&b, o)? as usize }) };
    let res_base = if res_rva != 0 { Some(rva_to_off(res_rva)?) } else { None };
    let mut out = HashMap::new();
    for _ in 0..rows[0x28] {
        let offset = u32_at(&b, tp)? as usize;
        let name = read_str(read_idx(tp + 8, str_idx)?);
        let imp = read_idx(tp + 8 + str_idx, implementation)?;
        tp += 8 + str_idx + implementation;
        if imp != 0 {
            continue; // stored in another file
        }
        let Some(base) = res_base else { continue };
        let at = base + offset;
        let len = u32_at(&b, at)? as usize;
        if let Some(data) = b.get(at + 4..at + 4 + len) {
            out.insert(name, data.to_vec());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_pictures_from_the_original_program() {
        let p = std::path::Path::new("/home/jason/Downloads/ZTracker_v2.1.0.2/Z1R_WPF.exe");
        if !p.exists() {
            return;
        }
        let res = super::read_manifest_resources(p).unwrap();
        assert!(res.contains_key("s_map_overworld_strip8.png"));
        assert!(res.contains_key("icons7x7.png"));
        assert_eq!(&res["new_icons13x9.png"][..4], b"\x89PNG");
    }

    #[test]
    fn reads_the_originals_logo() {
        let p = std::path::Path::new("/home/jason/Downloads/ZTracker_v2.1.0.2").join(super::LOGO_FILE);
        let Ok(bytes) = std::fs::read(&p) else { return };
        let logo = super::logo_from_ico(&bytes).unwrap();
        assert!(logo.width >= 48 && logo.height >= 48, "{}x{}", logo.width, logo.height);
        assert_eq!(logo.rgba.len(), (logo.width * logo.height * 4) as usize);
    }
}

#[cfg(test)]
mod shop_icon_tests {
    use eframe::egui::{Color32, ColorImage};

    #[test]
    fn the_black_b_is_gone_but_the_rest_of_the_shop_icon_stays() {
        let p = std::path::Path::new("/home/jason/Downloads/ZTracker_v2.1.0.2/Z1R_WPF.exe");
        if !p.exists() {
            return;
        }
        let mut a = super::Assets::default();
        assert!(a.try_load(Some(p.to_str().unwrap())));
        let img: &ColorImage = &a.sheets["s_icon_overworld_strip39.png"].image;
        let w = img.size[0];
        for icon in super::SHOP_STRIP_FIRST..=super::SHOP_STRIP_LAST {
            for y in 3..=7 {
                for x in icon * 16 + 1..=icon * 16 + 3 {
                    assert_ne!(img.pixels[y * w + x], Color32::BLACK, "icon {icon} still has the B at ({x},{y})");
                }
            }
        }
        // non-shop markers keep their black outlines/letters, e.g. the "L1" dungeon marker
        assert!((0..11).any(|y| (2 * 16..3 * 16).any(|x| img.pixels[y * w + x] == Color32::BLACK)));
    }
}
