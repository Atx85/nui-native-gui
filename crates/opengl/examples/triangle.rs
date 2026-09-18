//! A host-owned OpenGL window: normal GL scene drawing inside a CSS canvas.
use native_ui::Ui;
use native_ui_opengl::{
    BackendError, OpenGlBackend, Surface,
    glow::{self as gl, HasContext},
};
use sdl3::{
    event::Event,
    keyboard::Keycode,
    video::{GLProfile, SwapInterval},
};
use std::{rc::Rc, time::Instant};

struct Triangle {
    g: Rc<gl::Context>,
    program: gl::Program,
    vao: gl::VertexArray,
    angle: Option<gl::UniformLocation>,
    aspect: Option<gl::UniformLocation>,
}
impl Triangle {
    unsafe fn new(g: Rc<gl::Context>) -> Result<Self, BackendError> {
        unsafe {
            let program = g.create_program()?;
            for (kind, source) in [(gl::VERTEX_SHADER, VERTEX), (gl::FRAGMENT_SHADER, FRAGMENT)] {
                let shader = g.create_shader(kind)?;
                g.shader_source(shader, source);
                g.compile_shader(shader);
                if !g.get_shader_compile_status(shader) {
                    return Err(g.get_shader_info_log(shader).into());
                }
                g.attach_shader(program, shader);
                g.delete_shader(shader);
            }
            g.link_program(program);
            if !g.get_program_link_status(program) {
                return Err(g.get_program_info_log(program).into());
            }
            Ok(Self {
                vao: g.create_vertex_array()?,
                angle: g.get_uniform_location(program, "angle"),
                aspect: g.get_uniform_location(program, "aspect"),
                g,
                program,
            })
        }
    }
    fn draw(&self, pixels: [u32; 2], angle: f32) {
        unsafe {
            self.g.use_program(Some(self.program));
            self.g.bind_vertex_array(Some(self.vao));
            self.g.uniform_1_f32(self.angle.as_ref(), angle);
            self.g
                .uniform_1_f32(self.aspect.as_ref(), pixels[0] as f32 / pixels[1] as f32);
            self.g.draw_arrays(gl::TRIANGLES, 0, 3);
        }
    }
}
impl Drop for Triangle {
    fn drop(&mut self) {
        unsafe {
            self.g.delete_vertex_array(self.vao);
            self.g.delete_program(self.program);
        }
    }
}
fn main() -> Result<(), BackendError> {
    let snapshot = std::env::var("NUI_SNAPSHOT").ok();
    let mut smoke = snapshot.is_some();
    // Defaults to macOS; pass --theme win11 or --theme xp to choose another skin.
    let mut theme = "macos".to_owned();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke-test" => smoke = true,
            "--theme" => theme = args.next().ok_or("--theme needs xp, macos or win11")?,
            _ => return Err("Usage: triangle [--theme xp|macos|win11] [--smoke-test]".into()),
        }
    }
    let skin = match theme.as_str() {
        "xp" => include_str!("../../../examples/showcase/ui/themes/windows-xp-luna.css"),
        "macos" => include_str!("../../../examples/showcase/ui/themes/macos-light.css"),
        "win11" => include_str!("../../../examples/showcase/ui/themes/windows-11-fluent.css"),
        _ => return Err("theme must be xp, macos or win11".into()),
    };
    let sdl = sdl3::init()?;
    let video = sdl.video()?;
    let attr = video.gl_attr();
    attr.set_context_profile(GLProfile::Core);
    attr.set_context_version(3, 3);
    attr.set_depth_size(24);
    let mut builder = video.window("Native UI · OpenGL canvas", 900, 600);
    builder.opengl().resizable().high_pixel_density();
    if smoke {
        builder.hidden();
    }
    let window = builder.build()?;
    let _context = window.gl_create_context()?;
    let _ = video.gl_set_swap_interval(SwapInterval::VSync);
    let gl = Rc::new(unsafe {
        gl::Context::from_loader_function(|name| {
            video
                .gl_get_proc_address(name)
                .map_or(std::ptr::null(), |p| p as *const _)
        })
    });
    let triangle = unsafe { Triangle::new(gl.clone())? };
    let mut backend = unsafe { OpenGlBackend::new(gl.clone())? };
    let mut ui = Ui::from_html_css_with_viewport(HTML, &format!("{skin}\n{LAYOUT}"), 900., 600.)?;
    let mut events = sdl.event_pump()?;
    let mut last = Instant::now();
    let mut angle = 0.;
    'running: for frame in 0.. {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => native_ui_sdl3::handle_window_event(&mut ui, &event, &window)?,
            }
        }
        let (w, h) = window.size();
        let (pw, ph) = window.size_in_pixels();
        if w == 0 || h == 0 {
            continue;
        }
        ui.set_viewport(w as f32, h as f32)?;
        let now = Instant::now();
        let dt = (now - last).as_secs_f32();
        last = now;
        if ui.checked("animate") == Some(true) {
            let speed = ui
                .get_value("speed")
                .unwrap_or("1")
                .parse::<f32>()
                .unwrap_or(1.);
            angle += dt * speed;
        }
        if ui.clicked("reset") {
            angle = 0.;
        }
        unsafe {
            gl.disable(gl::SCISSOR_TEST);
            gl.clear_color(0.94, 0.95, 0.97, 1.);
            gl.clear(gl::COLOR_BUFFER_BIT);
        }
        backend.render_cached_with_canvases(
            &ui,
            Surface::new([w as f32, h as f32], [pw, ph])?,
            |info, _| {
                if info.region.id == Some("scene") {
                    triangle.draw(info.pixel_size(), angle);
                }
                Ok(())
            },
        )?;
        if smoke {
            unsafe {
                assert_eq!(gl.get_error(), gl::NO_ERROR);
            }
        }
        if frame == 7
            && let Some(path) = &snapshot
        {
            use std::io::Write;
            let mut pixels = vec![0u8; pw as usize * ph as usize * 4];
            unsafe {
                gl.read_pixels(
                    0,
                    0,
                    pw as i32,
                    ph as i32,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    gl::PixelPackData::Slice(Some(&mut pixels)),
                );
            }
            let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
            write!(file, "P6\n{pw} {ph}\n255\n")?;
            for row in pixels.chunks_exact(pw as usize * 4).rev() {
                for pixel in row.chunks_exact(4) {
                    file.write_all(&pixel[..3])?;
                }
            }
        }
        window.gl_swap_window();
        ui.end_frame();
        if smoke && frame >= 7 {
            break;
        }
    }
    if smoke {
        let stats = backend.cache_stats();
        assert!(stats.reuses > 0);
        println!(
            "OpenGL UI cache: {} rebuilds, {} reuses, {} texture layers",
            stats.rebuilds, stats.reuses, stats.texture_layers
        );
    }
    Ok(())
}
const HTML: &str = r#"<canvas id="scene"></canvas><aside class="panel"><label class="title">OpenGL canvas</label><label>Native scene, CSS UI</label><label>Rotation speed</label><input id="speed" type="range" min="0" max="3" step="0.1" value="1"><input id="animate" type="checkbox" checked><label for="animate">Animate</label><button id="reset" class="primary">Reset rotation</button><label>Resize the window.</label></aside><label id="badge" class="panel">UI over the canvas</label>"#;
// Application geometry only. Controls and panel colours come from the shared skin.
const LAYOUT: &str = r#"
canvas{position:absolute;left:0px;top:0px;width:100%;height:100%;padding:24px;background:#101a2b}
aside{position:absolute;left:24px;top:24px;width:230px;height:380px;padding:20px}
label{height:28px;margin-bottom:12px}.title{height:36px}
button,input[type=range]{width:190px;margin-bottom:14px}
input[type=checkbox]{margin-bottom:14px}
#badge{position:absolute;left:320px;top:24px;width:210px;height:36px;padding:8px}
"#;
const VERTEX: &str = r#"#version 330 core
uniform float angle;uniform float aspect;out vec3 color;
void main(){vec2 p[3]=vec2[3](vec2(0.,.72),vec2(-.62,-.36),vec2(.62,-.36));vec3 c[3]=vec3[3](vec3(.35,.65,1.),vec3(.9,.35,.55),vec3(.2,.9,.7));
vec2 v=mat2(cos(angle),sin(angle),-sin(angle),cos(angle))*p[gl_VertexID];v.x/=aspect;v.x+=.22;
gl_Position=vec4(v,0.,1.);color=c[gl_VertexID];}"#;
const FRAGMENT: &str = "#version 330 core\nin vec3 color;out vec4 output_color;void main(){output_color=vec4(color,1.);}";
