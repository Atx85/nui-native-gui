//! The same bundled font and rasterization metrics as the SDL connector.
use super::*;
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};

pub(super) struct TextTexture {
    pub texture: Texture2D,
    pub width: f32,
    pub height: f32,
    pub advance: f32,
}
impl RaylibBackend<'_> {
    pub(super) fn text(&mut self, text: &str, size: f32) -> Result<&mut TextTexture, BackendError> {
        if !size.is_finite() || size <= 0.0 || size > 1024.0 {
            return Err("invalid text size".into());
        }
        let key = (text.to_owned(), size.to_bits());
        if !self.text.contains_key(&key) {
            let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
            layout.append(&[&self.font], &TextStyle::new(text, size, 0));
            let advance: f32 = layout
                .glyphs()
                .iter()
                .map(|g| {
                    self.font
                        .metrics_indexed(g.key.glyph_index, size)
                        .advance_width
                        .ceil()
                })
                .sum();
            let glyphs: Vec<_> = layout
                .glyphs()
                .iter()
                .filter(|g| g.width > 0 && g.height > 0)
                .collect();
            let left = glyphs
                .iter()
                .map(|g| g.x)
                .fold(f32::INFINITY, f32::min)
                .min(0.0);
            let top = glyphs.iter().map(|g| g.y).fold(f32::INFINITY, f32::min);
            let right = glyphs
                .iter()
                .map(|g| g.x + g.width as f32)
                .fold(0.0, f32::max);
            let bottom = glyphs
                .iter()
                .map(|g| g.y + g.height as f32)
                .fold(0.0, f32::max);
            let width = (right - left).ceil().max(1.0) as u32;
            let height = (bottom - top).ceil().max(1.0) as u32;
            // Bound cache/texture allocation for very long labels or repeated host-provided text.
            if width > 16384 || height > 4096 {
                return Err("text exceeds texture limits".into());
            }
            let mut pixels = vec![255u8; width as usize * height as usize * 4];
            for alpha in pixels.iter_mut().skip(3).step_by(4) {
                *alpha = 0;
            }
            for glyph in glyphs {
                let (_, bitmap) = self.font.rasterize_config(glyph.key);
                for y in 0..glyph.height {
                    for x in 0..glyph.width {
                        let px = (glyph.x - left) as usize + x;
                        let py = (glyph.y - top) as usize + y;
                        let index = (py * width as usize + px) * 4 + 3;
                        let coverage = bitmap[y * glyph.width + x] as u16;
                        let old = pixels[index] as u16;
                        pixels[index] = (old + coverage * (255 - old) / 255) as u8;
                    }
                }
            }
            let texture = upload(width, height, &pixels)?;
            if self.text.len() >= 256 {
                // raylib batches draws; flush before releasing textures used this frame.
                flush();
                self.text.clear();
            }
            self.text.insert(
                key.clone(),
                TextTexture {
                    texture,
                    width: width as f32,
                    height: height as f32,
                    advance,
                },
            );
        }
        Ok(self.text.get_mut(&key).unwrap())
    }
}
