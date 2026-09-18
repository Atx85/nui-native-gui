//! Restore the state touched by UI rendering, including on callback errors/panics.
use glow::{self as gl, HasContext};
use std::rc::Rc;
const CAPS: [u32; 8] = [
    gl::BLEND,
    gl::CULL_FACE,
    gl::DEPTH_TEST,
    gl::STENCIL_TEST,
    gl::SCISSOR_TEST,
    gl::RASTERIZER_DISCARD,
    gl::PRIMITIVE_RESTART,
    gl::COLOR_LOGIC_OP,
];
pub(crate) struct State {
    gl: Rc<gl::Context>,
    program: Option<gl::Program>,
    vao: Option<gl::VertexArray>,
    buffer: Option<gl::Buffer>,
    draw_fb: Option<gl::Framebuffer>,
    read_fb: Option<gl::Framebuffer>,
    active: u32,
    texture: Option<gl::Texture>,
    sampler: Option<gl::Sampler>,
    viewport: [i32; 4],
    scissor: [i32; 4],
    blend: [u32; 6],
    enabled: [bool; 8],
    color_mask: [bool; 4],
    depth_mask: bool,
    polygon: u32,
    clear_color: [f32; 4],
    depth_func: u32,
    unpack: [i32; 4],
    unpack_buffer: Option<gl::Buffer>,
}
impl State {
    pub(crate) unsafe fn save(gl: &Rc<gl::Context>) -> Self {
        unsafe {
            let active = gl.get_parameter_i32(gl::ACTIVE_TEXTURE) as u32;
            gl.active_texture(gl::TEXTURE0);
            let mut viewport = [0; 4];
            gl.get_parameter_i32_slice(gl::VIEWPORT, &mut viewport);
            let mut scissor = [0; 4];
            gl.get_parameter_i32_slice(gl::SCISSOR_BOX, &mut scissor);
            let mut clear_color = [0.; 4];
            gl.get_parameter_f32_slice(gl::COLOR_CLEAR_VALUE, &mut clear_color);
            let mut polygon = [0; 2];
            gl.get_parameter_i32_slice(gl::POLYGON_MODE, &mut polygon);
            Self {
                gl: gl.clone(),
                active,
                program: gl.get_parameter_program(gl::CURRENT_PROGRAM),
                vao: gl.get_parameter_vertex_array(gl::VERTEX_ARRAY_BINDING),
                buffer: gl.get_parameter_buffer(gl::ARRAY_BUFFER_BINDING),
                draw_fb: gl.get_parameter_framebuffer(gl::DRAW_FRAMEBUFFER_BINDING),
                read_fb: gl.get_parameter_framebuffer(gl::READ_FRAMEBUFFER_BINDING),
                texture: gl.get_parameter_texture(gl::TEXTURE_BINDING_2D),
                sampler: gl.get_parameter_sampler(gl::SAMPLER_BINDING),
                viewport,
                scissor,
                clear_color,
                polygon: polygon[0] as u32,
                blend: [
                    gl::BLEND_SRC_RGB,
                    gl::BLEND_DST_RGB,
                    gl::BLEND_SRC_ALPHA,
                    gl::BLEND_DST_ALPHA,
                    gl::BLEND_EQUATION_RGB,
                    gl::BLEND_EQUATION_ALPHA,
                ]
                .map(|p| gl.get_parameter_i32(p) as u32),
                enabled: CAPS.map(|p| gl.is_enabled(p)),
                color_mask: gl.get_parameter_bool_array(gl::COLOR_WRITEMASK),
                depth_mask: gl.get_parameter_bool(gl::DEPTH_WRITEMASK),
                depth_func: gl.get_parameter_i32(gl::DEPTH_FUNC) as u32,
                unpack: [
                    gl::UNPACK_ALIGNMENT,
                    gl::UNPACK_ROW_LENGTH,
                    gl::UNPACK_SKIP_PIXELS,
                    gl::UNPACK_SKIP_ROWS,
                ]
                .map(|p| gl.get_parameter_i32(p)),
                unpack_buffer: gl.get_parameter_buffer(gl::PIXEL_UNPACK_BUFFER_BINDING),
            }
        }
    }
}
impl Drop for State {
    fn drop(&mut self) {
        unsafe {
            let g = &self.gl;
            g.use_program(self.program);
            g.bind_vertex_array(self.vao);
            g.bind_buffer(gl::ARRAY_BUFFER, self.buffer);
            g.bind_framebuffer(gl::DRAW_FRAMEBUFFER, self.draw_fb);
            g.bind_framebuffer(gl::READ_FRAMEBUFFER, self.read_fb);
            g.active_texture(gl::TEXTURE0);
            g.bind_texture(gl::TEXTURE_2D, self.texture);
            g.bind_sampler(0, self.sampler);
            g.active_texture(self.active);
            let [x, y, w, h] = self.viewport;
            g.viewport(x, y, w, h);
            let [x, y, w, h] = self.scissor;
            g.scissor(x, y, w, h);
            let [a, b, c, d, e, f] = self.blend;
            g.blend_func_separate(a, b, c, d);
            g.blend_equation_separate(e, f);
            for (cap, enabled) in CAPS.into_iter().zip(self.enabled) {
                if enabled {
                    g.enable(cap)
                } else {
                    g.disable(cap)
                }
            }
            let [r, b, c, a] = self.color_mask;
            g.color_mask(r, b, c, a);
            g.depth_mask(self.depth_mask);
            g.depth_func(self.depth_func);
            g.polygon_mode(gl::FRONT_AND_BACK, self.polygon);
            let [r, b, c, a] = self.clear_color;
            g.clear_color(r, b, c, a);
            for (p, v) in [
                gl::UNPACK_ALIGNMENT,
                gl::UNPACK_ROW_LENGTH,
                gl::UNPACK_SKIP_PIXELS,
                gl::UNPACK_SKIP_ROWS,
            ]
            .into_iter()
            .zip(self.unpack)
            {
                g.pixel_store_i32(p, v);
            }
            g.bind_buffer(gl::PIXEL_UNPACK_BUFFER, self.unpack_buffer);
        }
    }
}
