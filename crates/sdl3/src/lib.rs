//! Optional SDL3 adapter. The native-ui core has no SDL dependency.
mod cache;
mod images;
pub use cache::CacheStats;
pub mod text;
use native_ui::{CanvasRegion, Color, ImageDraw, Key, PaintCommand, Renderer, Ui};
use sdl3::{
    event::{Event, WindowEvent},
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
    pixels::{Color as SdlColor, PixelFormat},
    rect::Rect,
    render::{BlendMode, Canvas, ClippingRect, FRect, RenderTarget, Texture, TextureCreator},
    video::{Window, WindowContext},
};
use std::{collections::HashMap, error::Error};
use text::TextRasterizer;
pub type BackendError = Box<dyn Error>;

struct TextTexture<'a> {
    texture: Texture<'a>,
    width: f32,
    height: f32,
    advance: f32,
}
pub struct SdlBackend<'a, T> {
    creator: &'a TextureCreator<T>,
    font: TextRasterizer,
    text: HashMap<(String, u32), TextTexture<'a>>,
    paint_cache: Option<cache::CachedUi<'a>>,
    cache_stats: CacheStats,
    images: HashMap<String, images::ImageAsset<'a>>,
}
impl<'a, T> SdlBackend<'a, T> {
    pub fn new(creator: &'a TextureCreator<T>) -> Result<Self, BackendError> {
        Self::with_font(creator, include_bytes!("../../text/assets/NotoSans.ttf"))
    }
    pub fn with_font(creator: &'a TextureCreator<T>, bytes: &[u8]) -> Result<Self, BackendError> {
        Ok(Self {
            creator,
            font: TextRasterizer::new(bytes)?,
            text: HashMap::new(),
            paint_cache: None,
            cache_stats: CacheStats::default(),
            images: HashMap::new(),
        })
    }
    fn text(&mut self, text: &str, size: f32) -> Result<&mut TextTexture<'a>, BackendError> {
        if !size.is_finite() || size <= 0.0 || size > 1024.0 {
            return Err("invalid text size".into());
        }
        let key = (text.to_owned(), size.to_bits());
        if !self.text.contains_key(&key) {
            let text::RasterizedText {
                pixels,
                width,
                height,
                advance,
            } = self.font.rasterize(text, size)?;
            let mut texture =
                self.creator
                    .create_texture_static(PixelFormat::RGBA32, width, height)?;
            texture.update(None, &pixels, width as usize * 4)?;
            texture.set_blend_mode(BlendMode::Blend);
            if self.text.len() >= 256 {
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
    /// Uses the host's current scale, viewport, target and clipping. Restores draw state even on error.
    pub fn render<R: RenderTarget>(
        &mut self,
        ui: &Ui,
        canvas: &mut Canvas<R>,
    ) -> Result<(), BackendError> {
        self.render_with_canvases(ui, canvas, |_, _| Ok(()))
    }
    /// Draw native canvas content directly on the existing SDL target, in UI paint order.
    /// The callback uses UI-space coordinates and the host's scale/viewport. Its clip is
    /// the content box intersected with the host clip; use `region.content` as the origin.
    /// Do not call `clear`/`present`, destroy the renderer/target, or remove the clip to
    /// draw outside the region. SDL clear ignores clipping. Native code is trusted.
    /// Target, viewport, scale, logical presentation, clip, blend and draw color are
    /// restored after callbacks, including callbacks returning an error.
    pub fn render_with_canvases<R, F>(
        &mut self,
        ui: &Ui,
        canvas: &mut Canvas<R>,
        mut draw_canvas: F,
    ) -> Result<(), BackendError>
    where
        R: RenderTarget,
        F: FnMut(CanvasRegion<'_>, &mut Canvas<R>) -> Result<(), BackendError>,
    {
        // Direct painting can refresh text hit positions using different metrics.
        // Discard the retained paint snapshot when callers switch rendering paths.
        self.paint_cache = None;
        self.cache_stats.texture_bytes = 0;
        self.cache_stats.texture_layers = 0;
        self.cache_stats.command_layers = 0;
        let color = canvas.draw_color();
        let blend = canvas.blend_mode();
        let clip = canvas.clip_rect();
        canvas.set_blend_mode(BlendMode::Blend);
        let scale = canvas.scale().0.max(canvas.scale().1).max(1.0);
        let result = ui.render(&mut Frame {
            backend: self,
            canvas,
            scale,
            draw_canvas: &mut draw_canvas,
        });
        canvas.set_clip_rect(clip);
        canvas.set_blend_mode(blend);
        canvas.set_draw_color(color);
        result
    }
}
struct Frame<'b, 'a, T, R: RenderTarget, F> {
    backend: &'b mut SdlBackend<'a, T>,
    canvas: &'b mut Canvas<R>,
    scale: f32,
    draw_canvas: &'b mut F,
}
impl<T, R: RenderTarget, F> Renderer for Frame<'_, '_, T, R, F>
where
    F: FnMut(CanvasRegion<'_>, &mut Canvas<R>) -> Result<(), BackendError>,
{
    type Error = BackendError;
    fn raster_scale(&self) -> (f32, f32) {
        self.canvas.scale()
    }
    fn image_size(&mut self, src: &str) -> Option<(u32, u32)> {
        self.backend.registered_image_size(src)
    }
    fn draw_image(&mut self, image: ImageDraw<'_>) -> Result<(), BackendError> {
        self.backend.draw_registered_image(self.canvas, image)
    }
    fn draw_canvas(&mut self, region: CanvasRegion<'_>) -> Result<(), BackendError> {
        // SDL clips use integer render coordinates. Round inward so native pixels
        // never overwrite CSS padding/borders, including at fractional CSS edges.
        let rect = region.clip;
        let x = rect.x.ceil() as i32;
        let y = rect.y.ceil() as i32;
        let w = ((rect.x + rect.w).floor() - x as f32).max(0.0) as u32;
        let h = ((rect.y + rect.h).floor() - y as f32).max(0.0) as u32;
        if w == 0 || h == 0 {
            return Ok(());
        }
        let clip = self
            .canvas
            .clip_rect()
            .intersection(ClippingRect::Some(Rect::new(x, y, w, h)));
        if clip == ClippingRect::Zero {
            return Ok(());
        }
        let saved = NativeState::capture(self.canvas);
        self.canvas.set_clip_rect(clip);
        let result = (self.draw_canvas)(region, self.canvas);
        let restored = saved.restore(self.canvas);
        result.and(restored)
    }
    fn measure_text(&mut self, text: &str, size: f32) -> Result<(f32, f32), BackendError> {
        let text = self.backend.text(text, size * self.scale)?;
        Ok((text.advance / self.scale, text.height / self.scale))
    }
    fn draw(&mut self, command: PaintCommand<'_>) -> Result<(), BackendError> {
        match command {
            PaintCommand::FillRect {
                rect,
                color: Color(r, g, b, a),
            } => {
                if rect.w <= 0.0 || rect.h <= 0.0 {
                    return Ok(());
                }
                self.canvas.set_draw_color(SdlColor::RGBA(r, g, b, a));
                // Clipping can turn a physical 1px span into 0.999999px after
                // CSS-space subtraction. SDL's software backend truncates that
                // to zero. Stabilize grid-aligned coordinates before submission.
                let (sx, sy) = self.canvas.scale();
                let stable = |v: f32, scale: f32| {
                    let pixel = v * scale;
                    let nearest = pixel.round();
                    if scale > 0.0 && (pixel - nearest).abs() < 0.001 {
                        (nearest + 0.0001_f32.copysign(nearest)) / scale
                    } else {
                        v
                    }
                };
                self.canvas.fill_rect(FRect::new(
                    stable(rect.x, sx),
                    stable(rect.y, sy),
                    stable(rect.w, sx),
                    stable(rect.h, sy),
                ))?;
            }
            PaintCommand::Text {
                text,
                x,
                y,
                size,
                color: Color(r, g, b, a),
                clip,
            } => {
                if text.is_empty() || clip.w <= 0.0 || clip.h <= 0.0 {
                    return Ok(());
                }
                let old_clip = self.canvas.clip_rect();
                // Round inward: glyph pixels must stay within the button interior.
                let left = clip.x.ceil() as i32;
                let top = clip.y.ceil() as i32;
                let width = ((clip.x + clip.w).floor() - left as f32).max(0.0) as u32;
                let height = ((clip.y + clip.h).floor() - top as f32).max(0.0) as u32;
                if width == 0 || height == 0 {
                    return Ok(());
                }
                let clip =
                    old_clip.intersection(ClippingRect::Some(Rect::new(left, top, width, height)));
                let text = self.backend.text(text, size * self.scale)?;
                text.texture.set_color_mod(r, g, b);
                text.texture.set_alpha_mod(a);
                self.canvas.set_clip_rect(clip);
                let result = self.canvas.copy(
                    &text.texture,
                    None,
                    FRect::new(x, y, text.width / self.scale, text.height / self.scale),
                );
                self.canvas.set_clip_rect(old_clip);
                result?;
            }
        }
        Ok(())
    }
}

// No texture allocation, CPU readback or pixel upload is involved in canvas dispatch.
#[derive(Clone, Copy)]
struct NativeState {
    target: *mut sdl3::sys::render::SDL_Texture,
    viewport: Option<Rect>,
    scale: (f32, f32),
    logical: (u32, u32, sdl3::sys::render::SDL_RendererLogicalPresentation),
    clip: ClippingRect,
    color: SdlColor,
    blend: BlendMode,
}
impl NativeState {
    fn capture<R: RenderTarget>(canvas: &Canvas<R>) -> Self {
        // SAFETY: the borrowed Canvas owns a live SDL renderer. No ownership is transferred.
        let (target, viewport_set) = unsafe {
            (
                sdl3::sys::render::SDL_GetRenderTarget(canvas.raw()),
                sdl3::sys::render::SDL_RenderViewportSet(canvas.raw()),
            )
        };
        Self {
            target,
            viewport: viewport_set.then(|| canvas.viewport()),
            scale: canvas.scale(),
            logical: canvas.logical_size(),
            clip: canvas.clip_rect(),
            color: canvas.draw_color(),
            blend: canvas.blend_mode(),
        }
    }
    fn restore<R: RenderTarget>(self, canvas: &mut Canvas<R>) -> Result<(), BackendError> {
        // SAFETY: callbacks must not destroy the renderer or the target borrowed for this frame.
        let target_ok = unsafe {
            sdl3::sys::render::SDL_GetRenderTarget(canvas.raw()) == self.target
                || sdl3::sys::render::SDL_SetRenderTarget(canvas.raw(), self.target)
        };
        if !target_ok {
            return Err(sdl3::get_error().into());
        }
        let logical = if canvas.logical_size() != self.logical {
            canvas
                .set_logical_size(self.logical.0, self.logical.1, self.logical.2)
                .map_err(|e| -> BackendError { e.into() })
        } else {
            Ok(())
        };
        let scale = canvas.set_scale(self.scale.0, self.scale.1);
        canvas.set_viewport(self.viewport);
        canvas.set_clip_rect(self.clip);
        canvas.set_blend_mode(self.blend);
        canvas.set_draw_color(self.color);
        logical?;
        scale?;
        Ok(())
    }
}

/// Translate a window-space point with SDL so input matches scale, viewport and logical presentation.
pub fn pointer_position(
    canvas: &Canvas<Window>,
    x: f32,
    y: f32,
) -> Result<(f32, f32), BackendError> {
    let (mut rx, mut ry) = (0.0, 0.0);
    // SAFETY: Canvas owns a valid renderer and both output pointers refer to live local f32s.
    let success = unsafe {
        sdl3::sys::render::SDL_RenderCoordinatesFromWindow(canvas.raw(), x, y, &mut rx, &mut ry)
    };
    if !success {
        return Err(sdl3::get_error().into());
    }
    Ok((rx, ry))
}
/// Feed events from this canvas's window only. The host continues to handle quit and its own input.
pub fn handle_event(
    ui: &mut Ui,
    event: &Event,
    canvas: &Canvas<Window>,
) -> Result<(), BackendError> {
    handle_mapped_event(ui, event, canvas.window(), |x, y| {
        pointer_position(canvas, x, y)
    })
}
/// Route SDL window events in logical window coordinates, without an SDL renderer.
/// Useful when the same window hosts OpenGL. Quit remains the host's responsibility.
pub fn handle_window_event(
    ui: &mut Ui,
    event: &Event,
    window: &Window,
) -> Result<(), BackendError> {
    handle_mapped_event(ui, event, window, |x, y| Ok((x, y)))
}
fn handle_mapped_event(
    ui: &mut Ui,
    event: &Event,
    window: &Window,
    map: impl Fn(f32, f32) -> Result<(f32, f32), BackendError>,
) -> Result<(), BackendError> {
    match *event {
        Event::MouseMotion {
            window_id, x, y, ..
        } if window_id == window.id() => {
            let (x, y) = map(x, y)?;
            ui.pointer_move(x, y);
        }
        Event::MouseButtonDown {
            window_id,
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } if window_id == window.id() => {
            let (x, y) = map(x, y)?;
            ui.pointer_down(x, y);
        }
        Event::MouseButtonUp {
            window_id,
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } if window_id == window.id() => {
            let (x, y) = map(x, y)?;
            ui.pointer_up(x, y);
        }
        Event::TextInput {
            window_id,
            ref text,
            ..
        } if window_id == window.id() => ui.text_input(text),
        Event::MouseWheel {
            window_id,
            x,
            y,
            direction,
            ..
        } if window_id == window.id() => {
            let sign = if direction == sdl3::mouse::MouseWheelDirection::Flipped {
                1.0
            } else {
                -1.0
            };
            ui.scroll_wheel(x * sign * 40.0, y * sign * 40.0);
        }
        Event::Window {
            window_id,
            win_event: WindowEvent::FocusLost,
            ..
        } if window_id == window.id() => ui.cancel_input(),
        Event::Window {
            window_id,
            win_event: WindowEvent::MouseLeave,
            ..
        } if window_id == window.id() => ui.pointer_leave(),
        Event::KeyDown {
            window_id,
            keycode: Some(key),
            keymod,
            repeat,
            ..
        } if window_id == window.id() => {
            let shortcut =
                keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD | Mod::LGUIMOD | Mod::RGUIMOD);
            if shortcut && matches!(key, Keycode::C | Keycode::X) {
                if let Some(text) = ui.selected_text() {
                    window.subsystem().clipboard().set_clipboard_text(text)?;
                    if key == Keycode::X {
                        ui.key_down(Key::Delete, repeat);
                    }
                }
            } else if shortcut && key == Keycode::V && ui.wants_text_input() {
                ui.text_input(&window.subsystem().clipboard().clipboard_text()?);
            } else if let Some(key) = ui_key(key, keymod) {
                ui.key_down_with_shift(
                    key,
                    repeat,
                    keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD),
                );
            }
        }
        Event::KeyUp {
            window_id,
            keycode: Some(key),
            keymod,
            ..
        } if window_id == window.id() => {
            if let Some(key) = ui_key(key, keymod) {
                ui.key_up(key);
            }
        }
        _ => {}
    }
    sync_text_input(ui, window);
    Ok(())
}
/// Synchronize SDL text events after routing input (also callable after host-driven focus changes).
pub fn sync_text_input(ui: &Ui, window: &Window) {
    let text_input = window.subsystem().text_input();
    if ui.wants_text_input() && !text_input.is_active(window) {
        text_input.start(window);
    } else if !ui.wants_text_input() && text_input.is_active(window) {
        text_input.stop(window);
    }
}
fn ui_key(key: Keycode, modifiers: Mod) -> Option<Key> {
    match key {
        Keycode::PageUp => Some(Key::PageUp),
        Keycode::PageDown => Some(Key::PageDown),
        Keycode::Tab => Some(if modifiers.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
            Key::BackTab
        } else {
            Key::Tab
        }),
        Keycode::Return | Keycode::KpEnter => Some(Key::Enter),
        Keycode::Space => Some(Key::Space),
        Keycode::Left => Some(Key::Left),
        Keycode::Right => Some(Key::Right),
        Keycode::Up => Some(Key::Up),
        Keycode::Down => Some(Key::Down),
        Keycode::Home => Some(Key::Home),
        Keycode::End => Some(Key::End),
        Keycode::Backspace => Some(Key::Backspace),
        Keycode::Delete => Some(Key::Delete),
        Keycode::Escape => Some(Key::Escape),
        Keycode::A
            if modifiers
                .intersects(Mod::LCTRLMOD | Mod::RCTRLMOD | Mod::LGUIMOD | Mod::RGUIMOD) =>
        {
            Some(Key::SelectAll)
        }
        _ => None,
    }
}

/// Convenience alias for a normal SDL window renderer.
pub type WindowBackend<'a> = SdlBackend<'a, WindowContext>;
