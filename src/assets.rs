//! The tracker's pictures. They are all drawn by this program itself (see art.rs) when it starts: nothing is read
//! from other files or programs.

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, TextureHandle, TextureOptions};
use std::collections::HashMap;

pub struct Sheet {
    pub image: ColorImage,
    pub texture: Option<TextureHandle>,
    /// pixels per unit: the pictures are drawn in more detail than the units the drawing code uses
    pub scale: f32,
    /// shown with sharp pixels when enlarged (the overworld map pictures)
    pub pixel_art: bool,
}

/// Smooth scaling, with smaller copies for when a picture is shown much smaller than it is drawn.
const SMOOTH: TextureOptions = TextureOptions {
    magnification: egui::TextureFilter::Linear,
    minification: egui::TextureFilter::Linear,
    wrap_mode: egui::TextureWrapMode::ClampToEdge,
    mipmap_mode: Some(egui::TextureFilter::Linear),
};

pub struct Assets {
    sheets: HashMap<String, Sheet>,
    /// the overworld picture last asked for with the blank map ("" = none)
    custom_map_path: String,
}

/// The person's own overworld picture for the blank map.
const CUSTOM_MAP: &str = "custom overworld";
/// The original makes the picture 768 x 264 and cuts it into 16 x 8 screens of 48 x 33
/// (`Graphics::overworldMapBMPs`, token 0x06002bc9, RVA 0x9aecc).
pub const CUSTOM_MAP_SIZE: [u32; 2] = [768, 264];

impl Default for Assets {
    fn default() -> Self {
        Self::new()
    }
}

impl Assets {
    pub fn new() -> Self {
        let sheets = crate::art::sheets().into_iter().map(|(name, image, scale, pixel_art)| (name.to_string(), Sheet { image, texture: None, scale, pixel_art })).collect();
        Assets { sheets, custom_map_path: String::new() }
    }

    /// The sheet's texture and its size in units.
    fn texture(&mut self, ctx: &egui::Context, sheet: &str) -> Option<(egui::TextureId, [f32; 2])> {
        let s = self.sheets.get_mut(sheet)?;
        let size = [s.image.size[0] as f32 / s.scale, s.image.size[1] as f32 / s.scale];
        if s.texture.is_none() {
            let options = if s.pixel_art { TextureOptions { magnification: egui::TextureFilter::Nearest, ..SMOOTH } } else { SMOOTH };
            s.texture = Some(ctx.load_texture(sheet, s.image.clone(), options));
        }
        Some((s.texture.as_ref().unwrap().id(), size))
    }

    /// Draws part of a picture sheet (rectangle sx,sy,sw,sh in the sheet's units) into `dst`.
    /// `shade` < 1 darkens; `mirror` flips left-right.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&mut self, painter: &egui::Painter, sheet: &str, sx: f32, sy: f32, sw: f32, sh: f32, dst: Rect, shade: f32, mirror: bool) -> bool {
        let v = (shade.clamp(0.0, 1.0) * 255.0) as u8;
        self.draw_tinted(painter, sheet, sx, sy, sw, sh, dst, Color32::from_rgb(v, v, v), mirror)
    }

    /// Like `draw`, with any tint (e.g. see-through).
    #[allow(clippy::too_many_arguments)]
    fn draw_tinted(&mut self, painter: &egui::Painter, sheet: &str, sx: f32, sy: f32, sw: f32, sh: f32, dst: Rect, tint: Color32, mirror: bool) -> bool {
        let Some((id, size)) = self.texture(painter.ctx(), sheet) else { return false };
        let (w, h) = (size[0], size[1]);
        if sx < 0.0 || sy < 0.0 || sx + sw > w + 0.01 || sy + sh > h + 0.01 {
            return false;
        }
        let (mut u0, mut u1) = (sx / w, (sx + sw) / w);
        if mirror {
            std::mem::swap(&mut u0, &mut u1);
        }
        let uv = Rect::from_min_max(Pos2::new(u0, sy / h), Pos2::new(u1, (sy + sh) / h));
        painter.image(id, dst, uv, tint);
        true
    }

    // Convenience drawing helpers for the sheets the tracker uses. Each returns false if
    // the picture isn't available, so the caller can draw a stand-in.
    pub fn item7(&mut self, p: &egui::Painter, i: i32, dst: Rect, shade: f32) -> bool {
        i >= 0 && self.draw(p, "icons7x7.png", i as f32 * 7.0, 0.0, 7.0, 7.0, dst, shade, false)
    }
    /// An item picture in the original's grey (for items not had yet).
    pub fn item7_grey(&mut self, p: &egui::Painter, i: i32, dst: Rect) -> bool {
        i >= 0 && self.draw(p, crate::art::GREY_ITEMS, i as f32 * 7.0, 0.0, 7.0, 7.0, dst, 1.0, false)
    }
    /// Shop item picture (3x7 game pixels): 0 arrow, 1 bomb, 2 book, 3 candle, 4 blue ring, 5 meat, 6 key, 7 shield.
    pub fn shop_item(&mut self, p: &egui::Painter, i: i32, dst: Rect) -> bool {
        (0..8).contains(&i) && self.draw(p, "icons3x7.png", i as f32 * 3.0, 0.0, 3.0, 7.0, dst, 1.0, false)
    }
    pub fn icon10(&mut self, p: &egui::Painter, i: i32, dst: Rect, shade: f32) -> bool {
        self.draw(p, "icons10x10.png", i as f32 * 10.0, 0.0, 10.0, 10.0, dst, shade, false)
    }
    pub fn boss16(&mut self, p: &egui::Painter, i: i32, dst: Rect) -> bool {
        // one 18x18 cell each: the original sprite on black inside its border
        let c = crate::art::MONSTER_CELL as f32;
        i >= 0 && self.draw(p, "zelda_bosses16x16.png", i as f32 * c, 0.0, c, c, dst, 1.0, false)
    }
    pub fn zitem16(&mut self, p: &egui::Painter, i: i32, dst: Rect, shade: f32) -> bool {
        i >= 0 && self.draw(p, "zelda_items16x16.png", i as f32 * 16.0, 0.0, 16.0, 16.0, dst, shade, false)
    }
    /// A map marker picture at the given opacity, optionally in the original's greyscale (the Spot Summary's icons).
    pub fn ow_icon_faded(&mut self, p: &egui::Painter, i: i32, dst: Rect, opacity: f32, grey: bool) -> bool {
        let tint = Color32::from_white_alpha((opacity.clamp(0.0, 1.0) * 255.0) as u8);
        let sheet = if grey { crate::art::GREY_MARKERS } else { "s_icon_overworld_strip39.png" };
        i >= 0 && self.draw_tinted(p, sheet, i as f32 * 16.0, 0.0, 16.0, 11.0, dst, tint, false)
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
    /// Uses the picture at `path` as the blank map's overworld ("" = none). Like the original, any size is stretched
    /// to 768 x 264 and each 48 x 33 part is one screen.
    pub fn use_custom_map(&mut self, path: &str) -> Result<(), String> {
        self.custom_map_path = path.to_string();
        self.sheets.remove(CUSTOM_MAP);
        if path.is_empty() {
            return Ok(());
        }
        let img = image::open(path).map_err(|e| e.to_string())?.to_rgba8();
        let [w, h] = CUSTOM_MAP_SIZE;
        let img = image::imageops::resize(&img, w, h, image::imageops::FilterType::Triangle);
        let image = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], img.as_raw());
        self.sheets.insert(CUSTOM_MAP.into(), Sheet { image, texture: None, scale: 3.0, pixel_art: false });
        Ok(())
    }

    /// Makes sure the blank map's picture is the one named in the tracker (e.g. after loading a save).
    pub fn follow_custom_map(&mut self, path: &str) {
        if path != self.custom_map_path {
            if let Err(e) = self.use_custom_map(path) {
                eprintln!("Could not read the overworld picture {path}: {e}");
            }
        }
    }

    /// One screen of the blank map's own picture, if one is chosen.
    #[allow(clippy::too_many_arguments)]
    pub fn custom_map_screen(&mut self, p: &egui::Painter, x: usize, y: usize, dst: Rect, shade: f32, mirror: bool) -> bool {
        self.draw(p, CUSTOM_MAP, x as f32 * 16.0, y as f32 * 11.0, 16.0, 11.0, dst, shade, mirror)
    }

    /// A whole sheet drawn into `dst` at the given opacity (e.g. the item reference pictures).
    pub fn sheet_faded(&mut self, p: &egui::Painter, sheet: &str, dst: Rect, opacity: f32) -> bool {
        let Some(size) = self.sheets.get(sheet).map(|s| [s.image.size[0] as f32 / s.scale, s.image.size[1] as f32 / s.scale]) else { return false };
        let tint = Color32::from_white_alpha((opacity.clamp(0.0, 1.0) * 255.0) as u8);
        self.draw_tinted(p, sheet, 0.0, 0.0, size[0], size[1], dst, tint, false)
    }

    /// One map screen enlarged, for the magnifier (false if there is no enlarged picture).
    pub fn magnifier_screen(&mut self, p: &egui::Painter, x: usize, y: usize, dst: Rect, mirror: bool) -> bool {
        self.draw(p, crate::art::MAGNIFIER, x as f32 * 16.0, y as f32 * 11.0, 16.0, 11.0, dst, 1.0, mirror)
    }

    /// One overworld screen (the map is 16 x 8 screens of 16 x 11 units; all four overworlds share its ground).
    /// quest: 0 first, 1 second, 2 mixed (first-quest map), 3 mixed (second-quest map), 4 blank.
    pub fn map_screen(&mut self, p: &egui::Painter, quest: i64, x: usize, y: usize, dst: Rect, shade: f32, mirror: bool) -> bool {
        let q = quest.clamp(0, crate::art::OVERWORLD_MAPS as i64 - 1) as f32;
        self.draw(p, "s_map_overworld_strip8.png", q * 256.0 + x as f32 * 16.0, y as f32 * 11.0, 16.0, 11.0, dst, shade, mirror)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_custom_overworld_picture_is_stretched_to_the_originals_size() {
        let dir = std::env::temp_dir().join("z1r-custom-map-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("map.png");
        image::save_buffer(&path, &vec![200u8; 100 * 50 * 4], 100, 50, image::ColorType::Rgba8).unwrap();
        let mut a = super::Assets::new();
        a.use_custom_map(path.to_str().unwrap()).unwrap();
        let s = &a.sheets[super::CUSTOM_MAP];
        assert_eq!(s.image.size, [768, 264]);
        assert_eq!(s.image.size[0] as f32 / s.scale, 256.0); // 16 screens of 16 units
        assert!(a.use_custom_map("/no/such/picture.png").is_err());
        a.use_custom_map("").unwrap();
        assert!(!a.sheets.contains_key(super::CUSTOM_MAP));
    }
}
