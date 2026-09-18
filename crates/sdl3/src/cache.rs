//! Cached paint layers alternate with live native canvas callbacks in document order.
use super::*;
use native_ui::Rect as UiRect;

const MAX_TEXTURE_BYTES: u64 = 64 * 1024 * 1024;

/// Cumulative counters plus the memory/layer counts of the current cache.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub rebuilds: u64,
    pub reuses: u64,
    pub paint_commands_built: u64,
    pub text_measurements: u64,
    /// Successful registered-image uploads, including lazy uploads after a reset.
    pub image_uploads: u64,
    pub texture_bytes: u64,
    pub texture_layers: usize,
    pub command_layers: usize,
}

#[derive(PartialEq)]
struct Key {
    revision: u64,
    size: (u32, u32),
    viewport: Option<Rect>,
    scale: (f32, f32),
    logical: (u32, u32, sdl3::sys::render::SDL_RendererLogicalPresentation),
    clip: ClippingRect,
}

enum Command {
    Image {
        src: String,
        source: UiRect,
        destination: UiRect,
        clip: UiRect,
    },
    Fill {
        rect: UiRect,
        color: Color,
    },
    Text {
        text: String,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        clip: UiRect,
    },
}
impl Command {
    fn paint<R: Renderer>(&self, renderer: &mut R) -> Result<(), R::Error> {
        match self {
            Self::Image {
                src,
                source,
                destination,
                clip,
            } => renderer.draw_image(ImageDraw {
                src,
                source: *source,
                destination: *destination,
                clip: *clip,
            }),
            Self::Fill { rect, color } => renderer.draw(PaintCommand::FillRect {
                rect: *rect,
                color: *color,
            }),
            Self::Text {
                text,
                x,
                y,
                size,
                color,
                clip,
            } => renderer.draw(PaintCommand::Text {
                text,
                x: *x,
                y: *y,
                size: *size,
                color: *color,
                clip: *clip,
            }),
        }
    }
}
enum Layer<'a> {
    Commands(Vec<Command>),
    Texture(Texture<'a>),
    Canvas {
        id: Option<String>,
        bounds: UiRect,
        content: UiRect,
        clip: UiRect,
    },
}
pub(super) struct CachedUi<'a> {
    key: Key,
    layers: Vec<Layer<'a>>,
}

struct Recorder<'b, 'a, T> {
    backend: &'b mut SdlBackend<'a, T>,
    scale: f32,
    raster_scale: (f32, f32),
    layers: Vec<Layer<'a>>,
}
impl<T> Recorder<'_, '_, T> {
    fn push(&mut self, command: Command) {
        if !matches!(self.layers.last(), Some(Layer::Commands(_))) {
            self.layers.push(Layer::Commands(Vec::new()));
        }
        if let Some(Layer::Commands(commands)) = self.layers.last_mut() {
            commands.push(command);
        }
        self.backend.cache_stats.paint_commands_built += 1;
    }
}
impl<T> Renderer for Recorder<'_, '_, T> {
    type Error = BackendError;
    fn raster_scale(&self) -> (f32, f32) {
        self.raster_scale
    }
    fn image_size(&mut self, src: &str) -> Option<(u32, u32)> {
        self.backend.registered_image_size(src)
    }
    fn draw_image(&mut self, image: ImageDraw<'_>) -> Result<(), BackendError> {
        self.push(Command::Image {
            src: image.src.to_owned(),
            source: image.source,
            destination: image.destination,
            clip: image.clip,
        });
        Ok(())
    }
    fn measure_text(&mut self, text: &str, size: f32) -> Result<(f32, f32), BackendError> {
        self.backend.cache_stats.text_measurements += 1;
        let text = self.backend.text(text, size * self.scale)?;
        Ok((text.advance / self.scale, text.height / self.scale))
    }
    fn draw(&mut self, command: PaintCommand<'_>) -> Result<(), BackendError> {
        let command = match command {
            PaintCommand::FillRect { rect, color } => Command::Fill { rect, color },
            PaintCommand::Text {
                text,
                x,
                y,
                size,
                color,
                clip,
            } => Command::Text {
                text: text.to_owned(),
                x,
                y,
                size,
                color,
                clip,
            },
        };
        self.push(command);
        Ok(())
    }
    fn draw_canvas(&mut self, region: CanvasRegion<'_>) -> Result<(), BackendError> {
        self.layers.push(Layer::Canvas {
            id: region.id.map(str::to_owned),
            bounds: region.bounds,
            content: region.content,
            clip: region.clip,
        });
        Ok(())
    }
}

impl<'a, T> SdlBackend<'a, T> {
    pub fn cache_stats(&self) -> CacheStats {
        self.cache_stats
    }

    /// Discard derived GPU resources after SDL renderer/device reset events.
    /// Registered image dimensions and source pixels survive; textures are uploaded
    /// lazily when next drawn. Use `remove_image` to release an asset completely.
    pub fn clear_caches(&mut self) {
        self.paint_cache = None;
        self.text.clear();
        self.reset_image_textures();
        self.cache_stats.texture_bytes = 0;
        self.cache_stats.texture_layers = 0;
        self.cache_stats.command_layers = 0;
    }

    /// Paint with cached static layers, without native canvas content.
    pub fn render_cached<R: RenderTarget>(
        &mut self,
        ui: &Ui,
        canvas: &mut Canvas<R>,
    ) -> Result<(), BackendError> {
        self.render_cached_with_canvases(ui, canvas, |_, _| Ok(()))
    }

    /// Refresh UI layers only when the UI revision or host rendering geometry changes.
    /// Native drawing callbacks still run on every call, in the original paint order.
    /// Up to 64 MiB of target textures are retained. Software rendering, unsupported
    /// target textures or premultiplied blending use cached drawing commands instead.
    /// The caller still clears/presents the host target. Do not mix renderers with a
    /// backend: its texture creator and the borrowed canvas must belong together.
    pub fn render_cached_with_canvases<R, F>(
        &mut self,
        ui: &Ui,
        canvas: &mut Canvas<R>,
        mut draw_canvas: F,
    ) -> Result<(), BackendError>
    where
        R: RenderTarget,
        F: FnMut(CanvasRegion<'_>, &mut Canvas<R>) -> Result<(), BackendError>,
    {
        let saved = NativeState::capture(canvas);
        canvas.set_blend_mode(BlendMode::Blend);
        let result = self.paint_cached(ui, canvas, &mut draw_canvas);
        let restored = saved.restore(canvas);
        if self.paint_cache.is_none() {
            // Failed builds drop their partial layers; report retained memory only.
            self.cache_stats.texture_bytes = 0;
            self.cache_stats.texture_layers = 0;
            self.cache_stats.command_layers = 0;
        }
        result.and(restored)
    }

    fn paint_cached<R, F>(
        &mut self,
        ui: &Ui,
        canvas: &mut Canvas<R>,
        draw_canvas: &mut F,
    ) -> Result<(), BackendError>
    where
        R: RenderTarget,
        F: FnMut(CanvasRegion<'_>, &mut Canvas<R>) -> Result<(), BackendError>,
    {
        let host = NativeState::capture(canvas);
        let key = Key {
            revision: ui.paint_revision(),
            size: physical_size(canvas, host.target)?,
            viewport: host.viewport,
            scale: host.scale,
            logical: host.logical,
            clip: host.clip,
        };
        let scale = host.scale.0.max(host.scale.1).max(1.0);
        if self
            .paint_cache
            .as_ref()
            .is_none_or(|cache| cache.key != key)
        {
            // Drop old GPU allocations before building their replacements.
            self.paint_cache = None;
            self.cache_stats.texture_bytes = 0;
            self.cache_stats.texture_layers = 0;
            self.cache_stats.command_layers = 0;
            let mut recorder = Recorder {
                backend: self,
                scale,
                raster_scale: host.scale,
                layers: Vec::new(),
            };
            ui.render(&mut recorder)?;
            let mut layers = recorder.layers;
            let bytes = u64::from(key.size.0) * u64::from(key.size.1) * 4;
            for layer in &mut layers {
                let Layer::Commands(commands) = layer else {
                    continue;
                };
                if bytes > 0
                    && bytes <= MAX_TEXTURE_BYTES - self.cache_stats.texture_bytes
                    && let Some(texture) =
                        self.rasterize(commands, canvas, host, key.size, scale)?
                {
                    *layer = Layer::Texture(texture);
                    self.cache_stats.texture_bytes += bytes;
                    self.cache_stats.texture_layers += 1;
                } else {
                    self.cache_stats.command_layers += 1;
                }
            }
            self.paint_cache = Some(CachedUi { key, layers });
            self.cache_stats.rebuilds += 1;
        } else {
            self.cache_stats.reuses += 1;
        }
        // Take ownership temporarily so Frame can borrow the backend's text cache.
        let cache = self.paint_cache.take().unwrap();
        let result = (|| {
            for layer in &cache.layers {
                match layer {
                    Layer::Texture(texture) => composite(canvas, texture)?,
                    Layer::Commands(commands) => {
                        let mut frame = Frame {
                            backend: self,
                            canvas,
                            scale,
                            draw_canvas,
                        };
                        for command in commands {
                            command.paint(&mut frame)?;
                        }
                    }
                    Layer::Canvas {
                        id,
                        bounds,
                        content,
                        clip,
                    } => {
                        Frame {
                            backend: self,
                            canvas,
                            scale,
                            draw_canvas,
                        }
                        .draw_canvas(CanvasRegion {
                            id: id.as_deref(),
                            bounds: *bounds,
                            content: *content,
                            clip: *clip,
                        })?;
                    }
                }
            }
            Ok(())
        })();
        self.paint_cache = Some(cache);
        result
    }

    fn rasterize<R: RenderTarget>(
        &mut self,
        commands: &[Command],
        canvas: &mut Canvas<R>,
        host: NativeState,
        size: (u32, u32),
        scale: f32,
    ) -> Result<Option<Texture<'a>>, BackendError> {
        // Software text blits can force destination alpha opaque inside the text
        // rectangle. Replay commands there to preserve transparency exactly; it
        // also avoids copying full-window CPU surfaces for small control panels.
        if canvas.renderer_name == "software" {
            return Ok(None);
        }
        let Ok(texture) = self
            .creator
            .create_texture_target(PixelFormat::RGBA32, size.0, size.1)
        else {
            return Ok(None);
        };
        // Drawing straight-alpha primitives over transparent black produces
        // premultiplied pixels. Using ordinary alpha blending again would darken
        // translucent backgrounds, text edges, and rounded corners.
        // SAFETY: texture is live and belongs to this backend's renderer.
        if !unsafe {
            sdl3::sys::render::SDL_SetTextureBlendMode(
                texture.raw(),
                sdl3::sys::blendmode::SDL_BLENDMODE_BLEND_PREMULTIPLIED,
            )
        } {
            return Ok(None);
        }
        let result = (|| {
            NativeState {
                target: texture.raw(),
                ..host
            }
            .restore(canvas)?;
            canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 0));
            canvas.clear();
            let mut noop = |_: CanvasRegion<'_>, _: &mut Canvas<R>| Ok(());
            let mut frame = Frame {
                backend: self,
                canvas,
                scale,
                draw_canvas: &mut noop,
            };
            for command in commands {
                command.paint(&mut frame)?;
            }
            Ok(())
        })();
        let restored = host.restore(canvas);
        result.and(restored)?;
        Ok(Some(texture))
    }
}

fn composite<R: RenderTarget>(
    canvas: &mut Canvas<R>,
    texture: &Texture<'_>,
) -> Result<(), BackendError> {
    let saved = NativeState::capture(canvas);
    let result = (|| {
        // The layer already includes the host transform and clip. Copy its physical
        // pixels 1:1; restore the host coordinate system before native callbacks.
        canvas.set_logical_size(0, 0, sdl3::sys::render::SDL_LOGICAL_PRESENTATION_DISABLED)?;
        canvas.set_scale(1.0, 1.0)?;
        canvas.set_viewport(None);
        canvas.set_clip_rect(None);
        canvas.copy(texture, None, None)?;
        Ok(())
    })();
    let restored = saved.restore(canvas);
    result.and(restored)
}

fn physical_size<R: RenderTarget>(
    canvas: &Canvas<R>,
    target: *mut sdl3::sys::render::SDL_Texture,
) -> Result<(u32, u32), BackendError> {
    // SDL_GetCurrentRenderOutputSize (Canvas::output_size) excludes letterbox
    // margins. A cached layer must cover the entire physical target, otherwise
    // logical presentation gets applied to the wrong aspect ratio on rebuild.
    // SAFETY: renderer and any current target are live; all output pointers are valid.
    unsafe {
        if target.is_null() {
            let (mut w, mut h) = (0, 0);
            if !sdl3::sys::render::SDL_GetRenderOutputSize(canvas.raw(), &mut w, &mut h) {
                return Err(sdl3::get_error().into());
            }
            Ok((w.max(0) as u32, h.max(0) as u32))
        } else {
            let (mut w, mut h) = (0.0, 0.0);
            if !sdl3::sys::render::SDL_GetTextureSize(target, &mut w, &mut h) {
                return Err(sdl3::get_error().into());
            }
            Ok((w as u32, h as u32))
        }
    }
}
