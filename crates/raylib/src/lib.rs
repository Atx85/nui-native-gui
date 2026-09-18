//! Optional raylib connector. The UI core has no raylib dependency.
//!
//! Create after raylib initialization and drop before closing its window. Render in
//! an ordinary screen-space drawing frame, outside camera, shader, blend, texture,
//! and scissor modes. Coordinates are logical window pixels (including on HiDPI).
mod cache;
mod input;
mod text;
pub use cache::CacheStats;
use fontdue::{Font, FontSettings};
pub use input::{handle_input, handle_input_snapshot};
use native_ui::{CanvasRegion, Color, ImageDraw, PaintCommand, Rect, Renderer, Ui};
use raylib::prelude::*;
use std::{collections::HashMap, error::Error};
use text::TextTexture;

pub type BackendError = Box<dyn Error>;

/// Owns registered images, a bounded text cache and the last UI's paint commands.
/// The host owns the window.
pub struct RaylibBackend<'a> {
    _thread: &'a RaylibThread,
    font: Font,
    text: HashMap<(String, u32), TextTexture>,
    images: HashMap<String, Texture2D>,
    paint_cache: Option<cache::CachedUi>,
    cache_stats: CacheStats,
}

impl<'a> RaylibBackend<'a> {
    pub fn new(rl: &RaylibHandle, thread: &'a RaylibThread) -> Result<Self, BackendError> {
        Self::with_font(rl, thread, include_bytes!("../../text/assets/NotoSans.ttf"))
    }

    pub fn with_font(
        rl: &RaylibHandle,
        thread: &'a RaylibThread,
        bytes: &[u8],
    ) -> Result<Self, BackendError> {
        if !rl.is_window_ready() {
            return Err("initialize raylib before creating its UI connector".into());
        }
        Ok(Self {
            _thread: thread,
            font: Font::from_bytes(bytes, FontSettings::default())?,
            text: HashMap::new(),
            images: HashMap::new(),
            paint_cache: None,
            cache_stats: CacheStats::default(),
        })
    }

    /// Register tightly packed straight-alpha RGBA8 under an exact HTML `src` key.
    /// No image paths or URLs are loaded automatically. Failed replacements preserve
    /// the old asset. Call between frames while the raylib context is alive.
    pub fn register_image_rgba(
        &mut self,
        src: impl Into<String>,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<(), BackendError> {
        let src = src.into();
        if src.is_empty() {
            return Err("image src must be nonempty".into());
        }
        let texture = upload(width, height, pixels)?;
        flush();
        self.images.insert(src, texture);
        self.paint_cache = None;
        Ok(())
    }

    pub fn remove_image(&mut self, src: &str) -> bool {
        flush();
        let removed = self.images.remove(src).is_some();
        if removed {
            self.paint_cache = None;
        }
        removed
    }

    /// Release cached text textures between frames. Registered images are retained.
    pub fn clear_text_cache(&mut self) {
        flush();
        self.text.clear();
        self.paint_cache = None;
    }

    pub fn cache_stats(&self) -> CacheStats {
        self.cache_stats
    }

    /// Paint into the host's open screen-space frame without clearing or presenting.
    pub fn render(&mut self, ui: &Ui, draw: &mut RaylibDrawHandle<'_>) -> Result<(), BackendError> {
        self.render_with_canvases(ui, draw, |_, _| Ok(()))
    }

    /// Draw native content every frame in UI paint order, clipped to each canvas
    /// content box. Retain UI commands and text metrics until paint invalidation.
    /// The callback must not clear/present, alter scissor state, or close the window.
    /// Balanced camera/shader/blend scopes inside a callback are allowed. Its clip
    /// is removed even if the callback returns an error or unwinds.
    pub fn render_with_canvases<D: RaylibDraw, F>(
        &mut self,
        ui: &Ui,
        draw: &mut D,
        mut draw_canvas: F,
    ) -> Result<(), BackendError>
    where
        F: FnMut(CanvasRegion<'_>, &mut RaylibScissorMode<'_, D>) -> Result<(), BackendError>,
    {
        // SAFETY: callers render only within an active raylib drawing frame.
        let dpi = unsafe { raylib::ffi::GetWindowScaleDPI() };
        let scale = dpi.x.max(dpi.y).max(1.0);
        let key = (ui.paint_revision(), scale.to_bits());
        let cached = self.paint_cache.take();
        let cached = if let Some(cached) = cached.filter(|cached| cached.key == key) {
            self.cache_stats.reuses += 1;
            cached
        } else {
            cache::record(self, ui, scale, key)?
        };
        let result = cached.paint(&mut Frame {
            backend: self,
            draw,
            scale,
            draw_canvas: &mut draw_canvas,
        });
        self.paint_cache = Some(cached);
        // Keep texture destruction/replacement safe even before EndDrawing.
        flush();
        result
    }

    /// Reference path for diagnostics: regenerate UI paint commands every frame.
    pub fn render_uncached_with_canvases<D: RaylibDraw, F>(
        &mut self,
        ui: &Ui,
        draw: &mut D,
        mut draw_canvas: F,
    ) -> Result<(), BackendError>
    where
        F: FnMut(CanvasRegion<'_>, &mut RaylibScissorMode<'_, D>) -> Result<(), BackendError>,
    {
        // SAFETY: callers own an active screen-space raylib drawing frame.
        let dpi = unsafe { raylib::ffi::GetWindowScaleDPI() };
        let scale = dpi.x.max(dpi.y).max(1.0);
        self.paint_cache = None;
        let result = ui.render(&mut Frame {
            backend: self,
            draw,
            scale,
            draw_canvas: &mut draw_canvas,
        });
        flush();
        result
    }
}

struct Frame<'a, 'b, D, F> {
    backend: &'a mut RaylibBackend<'b>,
    draw: &'a mut D,
    scale: f32,
    draw_canvas: &'a mut F,
}

impl<D: RaylibDraw, F> Renderer for Frame<'_, '_, D, F>
where
    F: FnMut(CanvasRegion<'_>, &mut RaylibScissorMode<'_, D>) -> Result<(), BackendError>,
{
    type Error = BackendError;
    fn raster_scale(&self) -> (f32, f32) {
        (self.scale, self.scale)
    }

    fn measure_text(&mut self, value: &str, size: f32) -> Result<(f32, f32), BackendError> {
        let text = self.backend.text(value, size * self.scale)?;
        Ok((text.advance / self.scale, text.height / self.scale))
    }

    fn draw(&mut self, command: PaintCommand<'_>) -> Result<(), BackendError> {
        match command {
            PaintCommand::FillRect { rect, color } => {
                if rect.w > 0.0 && rect.h > 0.0 {
                    self.draw.draw_rectangle_rec(rectangle(rect), tint(color));
                }
            }
            PaintCommand::Text {
                text,
                x,
                y,
                size,
                color,
                clip,
            } => {
                let Some((cx, cy, w, h)) = clip_bounds(clip) else {
                    return Ok(());
                };
                if text.is_empty() {
                    return Ok(());
                }
                let text = self.backend.text(text, size * self.scale)?;
                let mut draw = self.draw.begin_scissor_mode(cx, cy, w, h);
                draw.draw_texture_pro(
                    &text.texture,
                    Rectangle::new(0.0, 0.0, text.width, text.height),
                    Rectangle::new(x, y, text.width / self.scale, text.height / self.scale),
                    Vector2::zero(),
                    0.0,
                    tint(color),
                );
            }
        }
        Ok(())
    }

    fn image_size(&mut self, src: &str) -> Option<(u32, u32)> {
        self.backend
            .images
            .get(src)
            .map(|t| (t.width() as u32, t.height() as u32))
    }

    fn draw_image(&mut self, image: ImageDraw<'_>) -> Result<(), BackendError> {
        let Some((x, y, w, h)) = clip_bounds(image.clip) else {
            return Ok(());
        };
        let Some(texture) = self.backend.images.get(image.src) else {
            return Ok(());
        };
        if image.source.w <= 0.0
            || image.source.h <= 0.0
            || image.destination.w <= 0.0
            || image.destination.h <= 0.0
        {
            return Ok(());
        }
        let mut draw = self.draw.begin_scissor_mode(x, y, w, h);
        draw.draw_texture_pro(
            texture,
            rectangle(image.source),
            rectangle(image.destination),
            Vector2::zero(),
            0.0,
            raylib::color::Color::WHITE,
        );
        Ok(())
    }

    fn draw_canvas(&mut self, region: CanvasRegion<'_>) -> Result<(), BackendError> {
        let Some((x, y, w, h)) = clip_bounds(region.clip) else {
            return Ok(());
        };
        let mut draw = self.draw.begin_scissor_mode(x, y, w, h);
        (self.draw_canvas)(region, &mut draw)
    }
}

fn rectangle(r: Rect) -> Rectangle {
    Rectangle::new(r.x, r.y, r.w, r.h)
}
fn tint(Color(r, g, b, a): Color) -> raylib::color::Color {
    raylib::color::Color::new(r, g, b, a)
}

// Round inward so text, images and native drawing cannot cover CSS padding/borders.
fn clip_bounds(r: Rect) -> Option<(i32, i32, i32, i32)> {
    let x = r.x.ceil() as i32;
    let y = r.y.ceil() as i32;
    let w = ((r.x + r.w).floor() - x as f32).max(0.0) as i32;
    let h = ((r.y + r.h).floor() - y as f32).max(0.0) as i32;
    (w > 0 && h > 0).then_some((x, y, w, h))
}

fn flush() {
    // SAFETY: connector operations run on the owning thread with an active context.
    unsafe { raylib::ffi::rlDrawRenderBatchActive() }
}

fn upload(width: u32, height: u32, pixels: &[u8]) -> Result<Texture2D, BackendError> {
    if width == 0 || height == 0 || width > 16384 || height > 16384 {
        return Err("image dimensions must be between 1 and 16384".into());
    }
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or("image size overflow")?;
    if pixels.len() != expected {
        return Err("image pixels must contain exactly width * height * 4 RGBA bytes".into());
    }
    let image = raylib::ffi::Image {
        data: pixels.as_ptr().cast_mut().cast(),
        width: width as i32,
        height: height as i32,
        mipmaps: 1,
        format: PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
    };
    // SAFETY: LoadTextureFromImage only reads the validated buffer synchronously.
    // It does not own/free the borrowed pixels. The returned texture has one owner.
    let raw = unsafe { raylib::ffi::LoadTextureFromImage(image) };
    if raw.id == 0 {
        return Err("raylib texture upload failed".into());
    }
    Ok(unsafe { Texture2D::from_raw(raw) })
}
