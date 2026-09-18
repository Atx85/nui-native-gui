//! Explicit host-owned image registration; no file or network loading in the renderer.
use super::*;

pub(super) struct ImageAsset<'a> {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    texture: Option<Texture<'a>>,
}

fn upload<'a, T>(
    creator: &'a TextureCreator<T>,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Result<Texture<'a>, BackendError> {
    let mut texture = creator.create_texture_static(PixelFormat::RGBA32, width, height)?;
    texture.update(None, pixels, width as usize * 4)?;
    texture.set_blend_mode(BlendMode::Blend);
    Ok(texture)
}

impl<T> SdlBackend<'_, T> {
    /// Register tightly packed, straight-alpha RGBA8 pixels under an exact `src` key.
    /// Uploads once. Replacing an asset invalidates cached UI, including fitted image
    /// geometry. Validation/upload failures leave the previous asset and cache intact.
    /// Retains a CPU copy so `clear_caches` can discard GPU resources after a device
    /// reset and lazily recreate them on the next draw. Asset storage is separate
    /// from the paint-layer texture budget. No paths/URLs are fetched automatically.
    pub fn register_image_rgba(
        &mut self,
        src: impl Into<String>,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<(), BackendError> {
        let src = src.into();
        if src.is_empty() || width == 0 || height == 0 {
            return Err("image src and dimensions must be nonempty".into());
        }
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or("image dimensions overflow the pixel buffer size")?;
        if pixels.len() != expected {
            return Err("image pixels must contain exactly width * height * 4 RGBA bytes".into());
        }
        let texture = upload(self.creator, width, height, pixels)?;
        let asset = ImageAsset {
            width,
            height,
            pixels: pixels.to_vec(),
            texture: Some(texture),
        };
        self.images.insert(src, asset);
        self.cache_stats.image_uploads += 1;
        self.invalidate_image_paint();
        Ok(())
    }

    /// Remove an asset and invalidate cached UI so its alt fallback is painted.
    /// Returns false for an unknown key without invalidating the cache.
    pub fn remove_image(&mut self, src: &str) -> bool {
        let removed = self.images.remove(src).is_some();
        if removed {
            self.invalidate_image_paint();
        }
        removed
    }

    fn invalidate_image_paint(&mut self) {
        self.paint_cache = None;
        self.cache_stats.texture_bytes = 0;
        self.cache_stats.texture_layers = 0;
        self.cache_stats.command_layers = 0;
    }

    pub(super) fn reset_image_textures(&mut self) {
        for asset in self.images.values_mut() {
            asset.texture = None;
        }
    }

    pub(super) fn registered_image_size(&self, src: &str) -> Option<(u32, u32)> {
        self.images
            .get(src)
            .map(|asset| (asset.width, asset.height))
    }

    pub(super) fn draw_registered_image<R: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<R>,
        image: ImageDraw<'_>,
    ) -> Result<(), BackendError> {
        if image.clip.w <= 0.0
            || image.clip.h <= 0.0
            || image.destination.w <= 0.0
            || image.destination.h <= 0.0
            || image.source.w <= 0.0
            || image.source.h <= 0.0
        {
            return Ok(());
        }
        // SDL clips are integral UI coordinates; use inward rounding, matching
        // text/native-canvas clipping so image pixels stay inside the content box.
        let x = image.clip.x.ceil() as i32;
        let y = image.clip.y.ceil() as i32;
        let w = ((image.clip.x + image.clip.w).floor() - x as f32).max(0.0) as u32;
        let h = ((image.clip.y + image.clip.h).floor() - y as f32).max(0.0) as u32;
        if w == 0 || h == 0 {
            return Ok(());
        }
        let saved_clip = canvas.clip_rect();
        let clip = saved_clip.intersection(ClippingRect::Some(Rect::new(x, y, w, h)));
        if clip == ClippingRect::Zero {
            return Ok(());
        }
        let Some(asset) = self.images.get_mut(image.src) else {
            return Ok(());
        };
        if asset.texture.is_none() {
            asset.texture = Some(upload(
                self.creator,
                asset.width,
                asset.height,
                &asset.pixels,
            )?);
            self.cache_stats.image_uploads += 1;
        }
        let source = image.source;
        let destination = image.destination;
        canvas.set_clip_rect(clip);
        let result = canvas.copy(
            asset.texture.as_ref().unwrap(),
            FRect::new(source.x, source.y, source.w, source.h),
            FRect::new(destination.x, destination.y, destination.w, destination.h),
        );
        canvas.set_clip_rect(saved_clip);
        result?;
        Ok(())
    }
}
