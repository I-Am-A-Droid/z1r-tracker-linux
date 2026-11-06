//! The tracker's pictures. They are all drawn by this program itself (see art.rs) when it starts: nothing is read
//! from other files or programs.

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, TextureHandle, TextureOptions};
use std::collections::HashMap;

pub struct Sheet {
    pub image: ColorImage,
    pub texture: Option<TextureHandle>,
    /// pixels per unit: the pictures are drawn in more detail than the units the drawing code uses
    pub scale: f32,
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
}

impl Default for Assets {
    fn default() -> Self {
        Self::new()
    }
}

impl Assets {
    pub fn new() -> Self {
        let sheets = crate::art::sheets().into_iter().map(|(name, image, scale)| (name.to_string(), Sheet { image, texture: None, scale })).collect();
        Assets { sheets }
    }

    /// The sheet's texture and its size in units.
    fn texture(&mut self, ctx: &egui::Context, sheet: &str) -> Option<(egui::TextureId, [f32; 2])> {
        let s = self.sheets.get_mut(sheet)?;
        let size = [s.image.size[0] as f32 / s.scale, s.image.size[1] as f32 / s.scale];
        if s.texture.is_none() {
            s.texture = Some(ctx.load_texture(sheet, s.image.clone(), SMOOTH));
        }
        Some((s.texture.as_ref().unwrap().id(), size))
    }

    /// Draws part of a picture sheet (rectangle sx,sy,sw,sh in the sheet's units) into `dst`.
    /// `shade` < 1 darkens; `mirror` flips left-right.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&mut self, painter: &egui::Painter, sheet: &str, sx: f32, sy: f32, sw: f32, sh: f32, dst: Rect, shade: f32, mirror: bool) -> bool {
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
        let v = (shade.clamp(0.0, 1.0) * 255.0) as u8;
        painter.image(id, dst, uv, Color32::from_rgb(v, v, v));
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
    /// One overworld screen (the map is 16 x 8 screens of 16 x 11 units; all four overworlds share its ground).
    pub fn map_screen(&mut self, p: &egui::Painter, _quest: i64, x: usize, y: usize, dst: Rect, shade: f32, mirror: bool) -> bool {
        self.draw(p, "s_map_overworld_strip8.png", x as f32 * 16.0, y as f32 * 11.0, 16.0, 11.0, dst, shade, mirror)
    }
}
