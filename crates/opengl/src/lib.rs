//! Desktop OpenGL 3.3+ renderer for Native UI. The host owns the current context,
//! framebuffer, events and presentation. See the crate README for the state contract.
mod cache;
pub use cache::CacheStats;
mod state;
mod text;
use fontdue::{Font, FontSettings};
pub use glow;
use glow::{self as gl, HasContext};
use native_ui::{CanvasRegion, Color, ImageDraw, PaintCommand, Rect, Renderer, Ui};
use state::State;
use std::{collections::HashMap, error::Error, rc::Rc};

pub type BackendError = Box<dyn Error>;

/// Logical UI dimensions and physical dimensions of the currently bound framebuffer.
#[derive(Clone, Copy, Debug)]
pub struct Surface {
    pub logical: [f32; 2],
    pub pixels: [u32; 2],
}
impl Surface {
    pub fn new(logical: [f32; 2], pixels: [u32; 2]) -> Result<Self, BackendError> {
        if logical.iter().any(|v| !v.is_finite() || *v <= 0.)
            || pixels.iter().any(|v| *v > i32::MAX as u32)
        {
            return Err("invalid OpenGL surface dimensions".into());
        }
        Ok(Self { logical, pixels })
    }
    fn scale(self) -> [f32; 2] {
        [
            self.pixels[0] as f32 / self.logical[0],
            self.pixels[1] as f32 / self.logical[1],
        ]
    }
    fn full(self) -> Rect {
        Rect {
            x: 0.,
            y: 0.,
            w: self.logical[0],
            h: self.logical[1],
        }
    }
}
/// Canvas geometry in both coordinate systems. `viewport` and `scissor` use GL's
/// bottom-left physical pixels; `region` uses top-left logical UI coordinates.
#[derive(Clone, Copy, Debug)]
pub struct CanvasInfo<'a> {
    pub region: CanvasRegion<'a>,
    pub viewport: [i32; 4],
    pub scissor: [i32; 4],
}
impl CanvasInfo<'_> {
    pub fn pixel_size(&self) -> [u32; 2] {
        [self.viewport[2] as u32, self.viewport[3] as u32]
    }
}

struct Texture {
    gl: Rc<gl::Context>,
    id: gl::Texture,
    width: u32,
    height: u32,
}
impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { self.gl.delete_texture(self.id) }
    }
}
struct Pipeline {
    gl: Rc<gl::Context>,
    program: gl::Program,
    vao: gl::VertexArray,
    buffer: gl::Buffer,
    screen: Option<gl::UniformLocation>,
}
impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.buffer);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_program(self.program);
        }
    }
}

/// Owns GPU resources only. Must be created, used and dropped on the GL context's
/// owning thread while that context is current. Rc deliberately makes this !Send.
pub struct OpenGlBackend {
    gl: Rc<gl::Context>,
    pipeline: Pipeline,
    white: Texture,
    font: Font,
    text: HashMap<(String, u32), text::TextTexture>,
    images: HashMap<String, Texture>,
    paint_cache: Option<cache::CachedUi>,
    cache_stats: CacheStats,
    cache_texture_limit: u64,
}
impl OpenGlBackend {
    /// # Safety
    /// `gl` must refer to a current desktop OpenGL 3.3+ context. Keep it current for
    /// every operation and destruction, and do not delete this backend's resources.
    pub unsafe fn new(gl: Rc<gl::Context>) -> Result<Self, BackendError> {
        unsafe { Self::with_font(gl, include_bytes!("../../text/assets/NotoSans.ttf")) }
    }
    /// # Safety
    /// Same context/lifetime requirements as `new`.
    pub unsafe fn with_font(gl: Rc<gl::Context>, bytes: &[u8]) -> Result<Self, BackendError> {
        unsafe {
            let version = gl.version();
            if version.is_embedded || (version.major, version.minor) < (3, 3) {
                return Err("desktop OpenGL 3.3 or newer is required".into());
            }
            let _state = State::save(&gl);
            let font = Font::from_bytes(bytes, FontSettings::default())?;
            let pipeline = Pipeline::new(&gl)?;
            let white = upload(&gl, 1, 1, &[255; 4])?;
            Ok(Self {
                gl,
                pipeline,
                white,
                font,
                text: HashMap::new(),
                images: HashMap::new(),
                paint_cache: None,
                cache_stats: CacheStats::default(),
                cache_texture_limit: 64 * 1024 * 1024,
            })
        }
    }
    pub fn gl(&self) -> &Rc<gl::Context> {
        &self.gl
    }
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
        let _state = unsafe { State::save(&self.gl) };
        let texture = unsafe { upload(&self.gl, width, height, pixels)? };
        self.images.insert(src, texture);
        self.cache_stats.image_uploads += 1;
        self.paint_cache = None;
        Ok(())
    }
    pub fn remove_image(&mut self, src: &str) -> bool {
        let removed = self.images.remove(src).is_some();
        if removed {
            self.paint_cache = None;
        }
        removed
    }
    pub fn clear_text_cache(&mut self) {
        self.paint_cache = None;
        self.text.clear();
    }
    pub fn render(&mut self, ui: &Ui, surface: Surface) -> Result<(), BackendError> {
        self.render_with_canvases(ui, surface, |_, _| Ok(()))
    }
    /// Draw into the currently bound draw framebuffer, without clearing/presenting.
    /// The callback receives an already positioned/clipped GL canvas. Keep scissor
    /// enabled and do not change the context or delete backend resources. Bind your
    /// own program/VAO; enable depth testing/writes explicitly for 3D. See README.
    pub fn render_with_canvases<F>(
        &mut self,
        ui: &Ui,
        surface: Surface,
        mut callback: F,
    ) -> Result<(), BackendError>
    where
        F: FnMut(CanvasInfo<'_>, &CanvasPainter<'_>) -> Result<(), BackendError>,
    {
        self.paint_cache = None;
        Surface::new(surface.logical, surface.pixels)?;
        if surface.pixels.contains(&0) {
            return Ok(());
        }
        let _state = unsafe { State::save(&self.gl) };
        // Evict only between frames, so queued glyph draws never reference freed textures.
        if self.text.len() > 256 {
            self.text.clear();
        }
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
        let result = ui.render(&mut frame);
        frame.flush();
        result
    }
}

/// Borrowed only for one canvas callback. `gl()` gives ordinary OpenGL access.
/// `fill` is an optional logical-coordinate convenience for small native overlays.
pub struct CanvasPainter<'a> {
    pipeline: &'a Pipeline,
    white: gl::Texture,
    surface: Surface,
    info: CanvasInfo<'a>,
}
impl CanvasPainter<'_> {
    pub fn gl(&self) -> &gl::Context {
        &self.pipeline.gl
    }
    pub fn info(&self) -> CanvasInfo<'_> {
        self.info
    }
    pub fn fill(&self, rect: Rect, color: Color) {
        let _state = unsafe { State::save(&self.pipeline.gl) };
        let mut vertices = Vec::with_capacity(48);
        quad(&mut vertices, rect, [0., 0., 1., 1.], color);
        unsafe {
            self.pipeline.setup(self.surface);
            self.pipeline
                .paint(self.surface, self.info.region.clip, self.white, &vertices);
        }
    }
}
struct Frame<'a, F> {
    backend: &'a mut OpenGlBackend,
    surface: Surface,
    callback: &'a mut F,
    vertices: Vec<f32>,
    texture: Option<gl::Texture>,
    clip: Rect,
}
impl<F> Frame<'_, F> {
    fn flush(&mut self) {
        if let Some(t) = self.texture {
            unsafe {
                self.backend
                    .pipeline
                    .paint(self.surface, self.clip, t, &self.vertices);
            }
        }
        self.vertices.clear();
    }
    fn queue(&mut self, rect: Rect, uv: [f32; 4], color: Color, texture: gl::Texture, clip: Rect) {
        if rect.w <= 0. || rect.h <= 0. || clip.w <= 0. || clip.h <= 0. {
            return;
        }
        if self.texture != Some(texture) || self.clip != clip || self.vertices.len() > 48 * 4096 {
            self.flush();
        }
        self.texture = Some(texture);
        self.clip = clip;
        // Core antialiasing already encodes edge coverage in physical-pixel spans.
        // Align remaining UI primitives (text/images/legacy indicators) to that grid,
        // avoiding half-pixel tie differences between window and texture targets.
        let [sx, sy] = self.surface.scale();
        let x = (rect.x * sx).round() / sx;
        let y = (rect.y * sy).round() / sy;
        let rect = Rect {
            x,
            y,
            w: ((rect.x + rect.w) * sx).round() / sx - x,
            h: ((rect.y + rect.h) * sy).round() / sy - y,
        };
        quad(&mut self.vertices, rect, uv, color);
    }
}
impl<F> Renderer for Frame<'_, F>
where
    F: FnMut(CanvasInfo<'_>, &CanvasPainter<'_>) -> Result<(), BackendError>,
{
    type Error = BackendError;
    fn raster_scale(&self) -> (f32, f32) {
        let [x, y] = self.surface.scale();
        (x, y)
    }
    fn measure_text(&mut self, text: &str, size: f32) -> Result<(f32, f32), BackendError> {
        let scale = self.surface.scale().into_iter().fold(0., f32::max);
        let t = self.backend.text(text, size * scale)?;
        Ok((t.advance / scale, t.texture.height as f32 / scale))
    }
    fn draw(&mut self, command: PaintCommand<'_>) -> Result<(), BackendError> {
        match command {
            PaintCommand::FillRect { rect, color } => self.queue(
                rect,
                [0., 0., 1., 1.],
                color,
                self.backend.white.id,
                self.surface.full(),
            ),
            PaintCommand::Text {
                text,
                x,
                y,
                size,
                color,
                clip,
            } => {
                if text.is_empty() {
                    return Ok(());
                }
                let scale = self.surface.scale().into_iter().fold(0., f32::max);
                let t = self.backend.text(text, size * scale)?;
                let (id, w, h) = (t.texture.id, t.texture.width, t.texture.height);
                self.queue(
                    Rect {
                        x,
                        y,
                        w: w as f32 / scale,
                        h: h as f32 / scale,
                    },
                    [0., 0., 1., 1.],
                    color,
                    id,
                    clip,
                );
            }
        }
        Ok(())
    }
    fn image_size(&mut self, src: &str) -> Option<(u32, u32)> {
        self.backend.images.get(src).map(|t| (t.width, t.height))
    }
    fn draw_image(&mut self, image: ImageDraw<'_>) -> Result<(), BackendError> {
        if let Some(t) = self.backend.images.get(image.src) {
            let r = image.source;
            let uv = [
                r.x / t.width as f32,
                r.y / t.height as f32,
                (r.x + r.w) / t.width as f32,
                (r.y + r.h) / t.height as f32,
            ];
            self.queue(
                image.destination,
                uv,
                Color(255, 255, 255, 255),
                t.id,
                image.clip,
            );
        }
        Ok(())
    }
    fn draw_canvas(&mut self, region: CanvasRegion<'_>) -> Result<(), BackendError> {
        let viewport = pixel_rect(self.surface, region.content, false);
        let scissor = pixel_rect(
            self.surface,
            intersect(region.clip, self.surface.full()),
            true,
        );
        if viewport[2] <= 0 || viewport[3] <= 0 || scissor[2] <= 0 || scissor[3] <= 0 {
            return Ok(());
        }
        self.flush();
        let info = CanvasInfo {
            region,
            viewport,
            scissor,
        };
        let _state = unsafe { State::save(&self.backend.gl) };
        unsafe {
            let g = &self.backend.gl;
            let [x, y, w, h] = viewport;
            g.viewport(x, y, w, h);
            let [x, y, w, h] = scissor;
            g.scissor(x, y, w, h);
            g.enable(gl::SCISSOR_TEST);
            g.disable(gl::BLEND);
        }
        (self.callback)(
            info,
            &CanvasPainter {
                pipeline: &self.backend.pipeline,
                white: self.backend.white.id,
                surface: self.surface,
                info,
            },
        )
    }
}
fn intersect(a: Rect, b: Rect) -> Rect {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    Rect {
        x,
        y,
        w: ((a.x + a.w).min(b.x + b.w) - x).max(0.),
        h: ((a.y + a.h).min(b.y + b.h) - y).max(0.),
    }
}
fn pixel_rect(surface: Surface, r: Rect, inward: bool) -> [i32; 4] {
    let [sx, sy] = surface.scale();
    let (l, t, r, b) = if inward {
        (
            (r.x * sx).ceil(),
            (r.y * sy).ceil(),
            ((r.x + r.w) * sx).floor(),
            ((r.y + r.h) * sy).floor(),
        )
    } else {
        (
            (r.x * sx).round(),
            (r.y * sy).round(),
            ((r.x + r.w) * sx).round(),
            ((r.y + r.h) * sy).round(),
        )
    };
    [
        l as i32,
        surface.pixels[1] as i32 - b as i32,
        (r - l).max(0.) as i32,
        (b - t).max(0.) as i32,
    ]
}
fn quad(v: &mut Vec<f32>, r: Rect, uv: [f32; 4], Color(red, green, blue, alpha): Color) {
    for (x, y, u, w) in [
        (r.x, r.y, uv[0], uv[1]),
        (r.x + r.w, r.y, uv[2], uv[1]),
        (r.x + r.w, r.y + r.h, uv[2], uv[3]),
        (r.x, r.y, uv[0], uv[1]),
        (r.x + r.w, r.y + r.h, uv[2], uv[3]),
        (r.x, r.y + r.h, uv[0], uv[3]),
    ] {
        v.extend_from_slice(&[
            x,
            y,
            u,
            w,
            red as f32 / 255.,
            green as f32 / 255.,
            blue as f32 / 255.,
            alpha as f32 / 255.,
        ]);
    }
}
impl Pipeline {
    unsafe fn new(g: &Rc<gl::Context>) -> Result<Self, BackendError> {
        unsafe {
            let program = g.create_program()?;
            for (kind, source) in [(gl::VERTEX_SHADER, VERTEX), (gl::FRAGMENT_SHADER, FRAGMENT)] {
                let shader = match g.create_shader(kind) {
                    Ok(s) => s,
                    Err(e) => {
                        g.delete_program(program);
                        return Err(e.into());
                    }
                };
                g.shader_source(shader, source);
                g.compile_shader(shader);
                if !g.get_shader_compile_status(shader) {
                    let e = g.get_shader_info_log(shader);
                    g.delete_shader(shader);
                    g.delete_program(program);
                    return Err(e.into());
                }
                g.attach_shader(program, shader);
                g.delete_shader(shader);
            }
            g.link_program(program);
            if !g.get_program_link_status(program) {
                let e = g.get_program_info_log(program);
                g.delete_program(program);
                return Err(e.into());
            }
            let vao = match g.create_vertex_array() {
                Ok(v) => v,
                Err(e) => {
                    g.delete_program(program);
                    return Err(e.into());
                }
            };
            let buffer = match g.create_buffer() {
                Ok(v) => v,
                Err(e) => {
                    g.delete_vertex_array(vao);
                    g.delete_program(program);
                    return Err(e.into());
                }
            };
            g.bind_vertex_array(Some(vao));
            g.bind_buffer(gl::ARRAY_BUFFER, Some(buffer));
            for (index, count, offset) in [(0, 2, 0), (1, 2, 8), (2, 4, 16)] {
                g.enable_vertex_attrib_array(index);
                g.vertex_attrib_pointer_f32(index, count, gl::FLOAT, false, 32, offset);
            }
            g.use_program(Some(program));
            g.uniform_1_i32(g.get_uniform_location(program, "image").as_ref(), 0);
            Ok(Self {
                gl: g.clone(),
                program,
                vao,
                buffer,
                screen: g.get_uniform_location(program, "screen"),
            })
        }
    }
    unsafe fn setup(&self, s: Surface) {
        unsafe {
            let g = &self.gl;
            g.use_program(Some(self.program));
            g.bind_vertex_array(Some(self.vao));
            g.bind_buffer(gl::ARRAY_BUFFER, Some(self.buffer));
            g.active_texture(gl::TEXTURE0);
            g.bind_sampler(0, None);
            for cap in [
                gl::CULL_FACE,
                gl::DEPTH_TEST,
                gl::STENCIL_TEST,
                gl::RASTERIZER_DISCARD,
                gl::PRIMITIVE_RESTART,
                gl::COLOR_LOGIC_OP,
            ] {
                g.disable(cap);
            }
            g.enable(gl::BLEND);
            g.enable(gl::SCISSOR_TEST);
            g.blend_equation_separate(gl::FUNC_ADD, gl::FUNC_ADD);
            g.blend_func_separate(
                gl::SRC_ALPHA,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
            );
            g.color_mask(true, true, true, true);
            g.depth_mask(false);
            g.polygon_mode(gl::FRONT_AND_BACK, gl::FILL);
            g.viewport(0, 0, s.pixels[0] as i32, s.pixels[1] as i32);
            g.uniform_2_f32(self.screen.as_ref(), s.logical[0], s.logical[1]);
        }
    }
    unsafe fn paint(&self, s: Surface, clip: Rect, texture: gl::Texture, vertices: &[f32]) {
        unsafe {
            if vertices.is_empty() {
                return;
            }
            let g = &self.gl;
            let [x, y, w, h] = pixel_rect(s, intersect(clip, s.full()), true);
            g.scissor(x, y, w, h);
            g.bind_texture(gl::TEXTURE_2D, Some(texture));
            let bytes = std::slice::from_raw_parts(
                vertices.as_ptr().cast::<u8>(),
                std::mem::size_of_val(vertices),
            );
            g.buffer_data_u8_slice(gl::ARRAY_BUFFER, bytes, gl::STREAM_DRAW);
            g.draw_arrays(gl::TRIANGLES, 0, (vertices.len() / 8) as i32);
        }
    }
}
unsafe fn upload(
    g: &Rc<gl::Context>,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Result<Texture, BackendError> {
    unsafe {
        let max = g.get_parameter_i32(gl::MAX_TEXTURE_SIZE).min(16384) as u32;
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|v| v.checked_mul(4));
        if width == 0
            || height == 0
            || width > max
            || height > max
            || expected != Some(pixels.len())
        {
            return Err("invalid RGBA8 image dimensions or byte count".into());
        }
        let id = g.create_texture()?;
        g.active_texture(gl::TEXTURE0);
        g.bind_texture(gl::TEXTURE_2D, Some(id));
        g.bind_buffer(gl::PIXEL_UNPACK_BUFFER, None);
        for (p, v) in [
            (gl::UNPACK_ALIGNMENT, 1),
            (gl::UNPACK_ROW_LENGTH, 0),
            (gl::UNPACK_SKIP_PIXELS, 0),
            (gl::UNPACK_SKIP_ROWS, 0),
        ] {
            g.pixel_store_i32(p, v);
        }
        for p in [gl::TEXTURE_MIN_FILTER, gl::TEXTURE_MAG_FILTER] {
            g.tex_parameter_i32(gl::TEXTURE_2D, p, gl::LINEAR as i32);
        }
        for p in [gl::TEXTURE_WRAP_S, gl::TEXTURE_WRAP_T] {
            g.tex_parameter_i32(gl::TEXTURE_2D, p, gl::CLAMP_TO_EDGE as i32);
        }
        g.tex_image_2d(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as i32,
            width as i32,
            height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            gl::PixelUnpackData::Slice(Some(pixels)),
        );
        Ok(Texture {
            gl: g.clone(),
            id,
            width,
            height,
        })
    }
}
const VERTEX: &str = "#version 330 core\nlayout(location=0) in vec2 position;layout(location=1) in vec2 texcoord;layout(location=2) in vec4 tint;uniform vec2 screen;out vec2 uv;out vec4 color;void main(){gl_Position=vec4(position.x/screen.x*2.-1.,1.-position.y/screen.y*2.,0.,1.);uv=texcoord;color=tint;}";
const FRAGMENT: &str = "#version 330 core\nin vec2 uv;in vec4 color;uniform sampler2D image;out vec4 output_color;void main(){output_color=texture(image,uv)*color;}";
