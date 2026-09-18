//! Retained static UI layers interleaved with live native canvas callbacks.
use super::*;
const MAX_TEXTURE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub rebuilds: u64,
    pub reuses: u64,
    pub paint_commands_built: u64,
    pub text_measurements: u64,
    pub image_uploads: u64,
    pub texture_bytes: u64,
    pub texture_layers: usize,
    pub command_layers: usize,
}
#[derive(PartialEq)]
struct Key {
    revision: u64,
    logical: [f32; 2],
    pixels: [u32; 2],
    srgb: bool,
}
enum Command {
    Image {
        src: String,
        source: Rect,
        destination: Rect,
        clip: Rect,
    },
    Fill {
        rect: Rect,
        color: Color,
    },
    Text {
        text: String,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        clip: Rect,
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

enum Layer {
    Commands(Vec<Command>),
    Texture(Texture),
    Canvas {
        id: Option<String>,
        bounds: Rect,
        content: Rect,
        clip: Rect,
    },
}
pub(super) struct CachedUi {
    key: Key,
    layers: Vec<Layer>,
}
struct Recorder<'a> {
    backend: &'a mut OpenGlBackend,
    surface: Surface,
    layers: Vec<Layer>,
}
impl Recorder<'_> {
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
impl Renderer for Recorder<'_> {
    type Error = BackendError;
    fn raster_scale(&self) -> (f32, f32) {
        let [x, y] = self.surface.scale();
        (x, y)
    }
    fn image_size(&mut self, src: &str) -> Option<(u32, u32)> {
        self.backend.images.get(src).map(|t| (t.width, t.height))
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
        let scale = self.surface.scale().into_iter().fold(0., f32::max);
        let text = self.backend.text(text, size * scale)?;
        Ok((text.advance / scale, text.texture.height as f32 / scale))
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

impl OpenGlBackend {
    /// Cumulative work counters and currently retained layer memory.
    pub fn cache_stats(&self) -> CacheStats {
        let mut stats = self.cache_stats;
        stats.texture_bytes = 0;
        stats.texture_layers = 0;
        stats.command_layers = 0;
        if let Some(cache) = &self.paint_cache {
            for layer in &cache.layers {
                match layer {
                    Layer::Texture(t) => {
                        stats.texture_layers += 1;
                        stats.texture_bytes += u64::from(t.width) * u64::from(t.height) * 4;
                    }
                    Layer::Commands(_) => stats.command_layers += 1,
                    Layer::Canvas { .. } => {}
                }
            }
        }
        stats
    }
    /// Release derived paint layers and text textures. Registered images survive.
    /// The original GL context must still be current; recreate the backend after context loss.
    pub fn clear_caches(&mut self) {
        self.paint_cache = None;
        self.text.clear();
    }
    /// Set the texture budget (capped at 64 MiB). Zero retains commands only.
    pub fn set_cache_texture_limit(&mut self, bytes: u64) {
        let limit = bytes.min(MAX_TEXTURE_BYTES);
        if limit != self.cache_texture_limit {
            self.paint_cache = None;
            self.cache_texture_limit = limit;
        }
    }
    pub fn render_cached(&mut self, ui: &Ui, surface: Surface) -> Result<(), BackendError> {
        self.render_cached_with_canvases(ui, surface, |_, _| Ok(()))
    }
    /// Reuse unchanged UI layers while calling native canvases every frame in paint order.
    /// Texture layers use premultiplied alpha; oversized layers and sRGB targets replay
    /// retained commands instead. The host still owns clearing and presentation.
    pub fn render_cached_with_canvases<F>(
        &mut self,
        ui: &Ui,
        surface: Surface,
        mut callback: F,
    ) -> Result<(), BackendError>
    where
        F: FnMut(CanvasInfo<'_>, &CanvasPainter<'_>) -> Result<(), BackendError>,
    {
        Surface::new(surface.logical, surface.pixels)?;
        if surface.pixels.contains(&0) {
            return Ok(());
        }
        let _state = unsafe { State::save(&self.gl) };
        if self.text.len() > 256 {
            self.text.clear();
        }
        let key = Key {
            revision: ui.paint_revision(),
            logical: surface.logical,
            pixels: surface.pixels,
            srgb: unsafe { self.gl.is_enabled(gl::FRAMEBUFFER_SRGB) },
        };
        if self.paint_cache.as_ref().is_none_or(|c| c.key != key) {
            self.paint_cache = None;
            let mut recorder = Recorder {
                backend: self,
                surface,
                layers: Vec::new(),
            };
            ui.render(&mut recorder)?;
            let mut layers = recorder.layers;
            let bytes = u64::from(surface.pixels[0]) * u64::from(surface.pixels[1]) * 4;
            let mut remaining = self.cache_texture_limit;
            for layer in &mut layers {
                if let Layer::Commands(commands) = layer
                    && !key.srgb
                    && bytes <= remaining
                    && let Some(texture) = self.rasterize_layer(commands, surface)?
                {
                    *layer = Layer::Texture(texture);
                    remaining -= bytes;
                }
            }
            self.paint_cache = Some(CachedUi { key, layers });
            self.cache_stats.rebuilds += 1;
        } else {
            self.cache_stats.reuses += 1;
        }
        // Taking ownership permits text-cache updates during command playback. On unwind
        // the layers are dropped safely; the next frame rebuilds them.
        let cache = self.paint_cache.take().unwrap();
        let result = (|| {
            unsafe {
                self.pipeline.setup(surface);
            }
            let mut frame = Frame {
                backend: self,
                surface,
                callback: &mut callback,
                vertices: Vec::new(),
                texture: None,
                clip: surface.full(),
            };
            for layer in &cache.layers {
                match layer {
                    Layer::Commands(commands) => {
                        for command in commands {
                            command.paint(&mut frame)?;
                        }
                        frame.flush();
                    }
                    Layer::Texture(texture) => unsafe {
                        // FBO textures have a bottom-left origin and premultiplied RGB.
                        frame.backend.gl.blend_func_separate(
                            gl::ONE,
                            gl::ONE_MINUS_SRC_ALPHA,
                            gl::ONE,
                            gl::ONE_MINUS_SRC_ALPHA,
                        );
                        frame.queue(
                            surface.full(),
                            [0., 1., 1., 0.],
                            Color(255, 255, 255, 255),
                            texture.id,
                            surface.full(),
                        );
                        frame.flush();
                        frame.backend.pipeline.setup(surface);
                    },
                    Layer::Canvas {
                        id,
                        bounds,
                        content,
                        clip,
                    } => frame.draw_canvas(CanvasRegion {
                        id: id.as_deref(),
                        bounds: *bounds,
                        content: *content,
                        clip: *clip,
                    })?,
                }
            }
            Ok(())
        })();
        self.paint_cache = Some(cache);
        result
    }
    fn rasterize_layer(
        &mut self,
        commands: &[Command],
        surface: Surface,
    ) -> Result<Option<Texture>, BackendError> {
        unsafe {
            let g = self.gl.clone();
            let max = g.get_parameter_i32(gl::MAX_TEXTURE_SIZE) as u32;
            if surface.pixels.iter().any(|v| *v > max) {
                return Ok(None);
            }
            let _state = State::save(&g);
            let Ok(id) = g.create_texture() else {
                return Ok(None);
            };
            let texture = Texture {
                gl: g.clone(),
                id,
                width: surface.pixels[0],
                height: surface.pixels[1],
            };
            g.active_texture(gl::TEXTURE0);
            g.bind_texture(gl::TEXTURE_2D, Some(id));
            g.bind_buffer(gl::PIXEL_UNPACK_BUFFER, None);
            for p in [gl::TEXTURE_MIN_FILTER, gl::TEXTURE_MAG_FILTER] {
                g.tex_parameter_i32(gl::TEXTURE_2D, p, gl::NEAREST as i32);
            }
            g.tex_image_2d(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                texture.width as i32,
                texture.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                gl::PixelUnpackData::Slice(None),
            );
            let Ok(id) = g.create_framebuffer() else {
                return Ok(None);
            };
            let _framebuffer = Framebuffer { gl: g.clone(), id };
            g.bind_framebuffer(gl::DRAW_FRAMEBUFFER, Some(id));
            g.framebuffer_texture_2d(
                gl::DRAW_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                Some(texture.id),
                0,
            );
            if g.check_framebuffer_status(gl::DRAW_FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                return Ok(None);
            }
            self.pipeline.setup(surface);
            g.disable(gl::SCISSOR_TEST);
            g.clear_color(0., 0., 0., 0.);
            g.clear(gl::COLOR_BUFFER_BIT);
            g.enable(gl::SCISSOR_TEST);
            let mut noop = |_: CanvasInfo<'_>, _: &CanvasPainter<'_>| Ok(());
            let mut frame = Frame {
                backend: self,
                surface,
                callback: &mut noop,
                vertices: Vec::new(),
                texture: None,
                clip: surface.full(),
            };
            for command in commands {
                command.paint(&mut frame)?;
            }
            frame.flush();
            Ok(Some(texture))
        }
    }
}
struct Framebuffer {
    gl: Rc<gl::Context>,
    id: gl::Framebuffer,
}
impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.id);
        }
    }
}
