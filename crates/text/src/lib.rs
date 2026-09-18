//! Renderer-independent font rasterization. No windowing or GPU dependency.
pub type BackendError = Box<dyn std::error::Error>;
use fontdue::{
    Font, FontSettings,
    layout::{CoordinateSystem, Layout, TextStyle},
};

pub struct TextRasterizer {
    font: Font,
}
pub struct RasterizedText {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub advance: f32,
}
impl TextRasterizer {
    pub fn new(bytes: &[u8]) -> Result<Self, BackendError> {
        Ok(Self {
            font: Font::from_bytes(bytes, FontSettings::default())?,
        })
    }
    pub fn bundled() -> Result<Self, BackendError> {
        Self::new(include_bytes!("../assets/NotoSans.ttf"))
    }
    pub fn rasterize(&self, text: &str, size: f32) -> Result<RasterizedText, BackendError> {
        if !size.is_finite() || size <= 0.0 || size > 1024.0 {
            return Err("invalid text size".into());
        }
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
        Ok(RasterizedText {
            pixels,
            width,
            height,
            advance,
        })
    }
}
