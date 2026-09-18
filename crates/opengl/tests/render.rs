mod cache;
use native_ui::Ui;
use native_ui_opengl::{
    OpenGlBackend, Surface,
    glow::{self as gl, HasContext},
};
use std::rc::Rc;

fn pixel(g: &gl::Context, x: i32, y: i32) -> [u8; 4] {
    let mut p = [0; 4];
    unsafe {
        g.read_pixels(
            x,
            240 - y - 1,
            1,
            1,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            gl::PixelPackData::Slice(Some(&mut p)),
        );
    }
    p
}
fn assert_pixel(g: &gl::Context, x: i32, y: i32, rgb: [u8; 3]) {
    assert_eq!(&pixel(g, x, y)[..3], &rgb, "pixel ({x},{y})");
}
fn verify_antialias(backend: &mut OpenGlBackend, g: &gl::Context) {
    for scale in [1., 1.5, 2., 3.] {
        let surface = Surface::new([320. / scale, 240. / scale], [320, 240]).unwrap();
        let ui = Ui::from_html_css_with_viewport(
            "<label></label><input type=checkbox checked>",
            "label{position:absolute;left:10.25px;top:10.5px;width:20px;height:20px;border:3px solid #ff0000;border-radius:10px;background:#0000ff}input{position:absolute;left:55px;top:10px;width:20px;height:20px;background:transparent;border:none;padding:0px;color:#ffffff}",
            surface.logical[0], surface.logical[1],
        ).unwrap();
        unsafe {
            g.disable(gl::SCISSOR_TEST);
            g.color_mask(true, true, true, true);
            g.clear_color(0., 0., 0., 1.);
            g.clear(gl::COLOR_BUFFER_BIT);
        }
        backend.render(&ui, surface).unwrap();
        let mut pixels = vec![0; 320 * 240 * 4];
        unsafe {
            g.read_pixels(
                0,
                0,
                320,
                240,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                gl::PixelPackData::Slice(Some(&mut pixels)),
            );
        }
        for y in 0..100 {
            for x in 0..100 {
                let mut red = 0;
                let mut blue = 0;
                for sy in 0..16 {
                    for sx in 0..16 {
                        let px = (x as f32 + (sx as f32 + 0.5) / 16.) / scale - 20.25;
                        let py = (y as f32 + (sy as f32 + 0.5) / 16.) / scale - 20.5;
                        let d = px * px + py * py;
                        if d < 49. {
                            blue += 1;
                        } else if d < 100. {
                            red += 1;
                        }
                    }
                }
                // Skip the separate checkmark; verify circle coverage against an
                // independent area reference, including the border/fill junction.
                if x as f32 >= 50. * scale {
                    continue;
                }
                let expected = [(red * 255 / 256) as u8, 0, (blue * 255 / 256) as u8];
                let got = &pixels[((239 - y) * 320 + x) * 4..][..3];
                assert!(
                    got.iter().zip(expected).all(|(a, b)| a.abs_diff(b) <= 5),
                    "OpenGL scale {scale}, ({x},{y}): {got:?} vs {expected:?}"
                );
            }
        }
        let mut soft_stroke = 0;
        for y in (10. * scale) as usize..(30. * scale) as usize {
            for x in (55. * scale) as usize..(75. * scale) as usize {
                let p = &pixels[((239 - y) * 320 + x) * 4..];
                if p[0] > 0 && p[0] < 255 {
                    soft_stroke += 1;
                }
            }
        }
        assert!(
            soft_stroke > 20,
            "checkmark should have antialiased edges at {scale}x"
        );
    }
}

// SDL/Cocoa context creation must run on the main thread, not Rust's test workers.
fn main() {
    let sdl = sdl3::init().unwrap();
    let video = sdl.video().unwrap();
    let attr = video.gl_attr();
    attr.set_context_profile(sdl3::video::GLProfile::Core);
    attr.set_context_version(3, 3);
    let window = video
        .window("OpenGL adapter pixel checks", 320, 240)
        .opengl()
        .hidden()
        .build()
        .unwrap();
    let _context = window.gl_create_context().unwrap();
    let g = Rc::new(unsafe {
        gl::Context::from_loader_function(|name| {
            video
                .gl_get_proc_address(name)
                .map_or(std::ptr::null(), |p| p as *const _)
        })
    });
    let mut backend = unsafe { OpenGlBackend::new(g.clone()) }.unwrap();
    let surface = Surface::new([320., 240.], [320, 240]).unwrap();
    let mut ui=Ui::from_html_css_with_viewport(
        "<button>Café λ</button><img src=tile><div id=clip><canvas id=scene></canvas></div><label id=overlay></label>",
        "button,img,canvas,div,label{position:absolute} button{left:10px;top:10px;width:140px;height:50px;background:#4080c0;color:#ff0000;padding:10px} img{left:170px;top:10px;width:40px;height:40px;object-fit:contain} #clip{left:10px;top:80px;width:80px;height:70px;overflow:hidden} canvas{left:0px;top:0px;width:100px;height:70px;padding:10px;border:2px solid #000000;background:#ffffff} #overlay{left:40px;top:100px;width:12px;height:12px;background:#0000ff}",320.,240.).unwrap();
    backend
        .register_image_rgba("tile", 2, 1, &[0, 255, 0, 255, 0, 255, 0, 255])
        .unwrap();
    assert!(backend.register_image_rgba("tile", 2, 1, &[0]).is_err());
    unsafe {
        g.clear_color(1., 1., 1., 1.);
        g.clear(gl::COLOR_BUFFER_BIT);
        // Deliberately awkward host state. Uploads must not interpret pixels as PBO offsets.
        g.viewport(3, 4, 19, 23);
        g.scissor(5, 6, 7, 8);
        g.enable(gl::SCISSOR_TEST);
        g.enable(gl::DEPTH_TEST);
        g.depth_mask(true);
        g.color_mask(false, true, false, true);
        g.polygon_mode(gl::FRONT_AND_BACK, gl::LINE);
        g.active_texture(gl::TEXTURE3);
        g.pixel_store_i32(gl::UNPACK_ROW_LENGTH, 9);
        g.pixel_store_i32(gl::UNPACK_ALIGNMENT, 8);
    }
    let mut calls = 0;
    backend
        .render_cached_with_canvases(&ui, surface, |info, painter| {
            calls += 1;
            assert_eq!(info.region.id, Some("scene"));
            assert_eq!(info.pixel_size(), [76, 46]);
            assert_eq!(info.viewport, [22, 102, 76, 46]);
            assert_eq!(info.scissor, [22, 102, 68, 46]);
            let gl = painter.gl();
            unsafe {
                // Clear ignores the viewport: the adapter's scissor must protect padding,
                // borders, ancestors and all surrounding UI even for this oversized draw.
                gl.clear_color(1., 0., 0., 1.);
                gl.clear(gl::COLOR_BUFFER_BIT);
                gl.use_program(None);
                gl.bind_vertex_array(None);
                gl.disable(gl::BLEND);
                gl.viewport(0, 0, 1, 1);
                gl.color_mask(false, false, false, false);
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(calls, 1);
    unsafe {
        let mut v = [0; 4];
        g.get_parameter_i32_slice(gl::VIEWPORT, &mut v);
        assert_eq!(v, [3, 4, 19, 23]);
        g.get_parameter_i32_slice(gl::SCISSOR_BOX, &mut v);
        assert_eq!(v, [5, 6, 7, 8]);
        assert!(g.is_enabled(gl::SCISSOR_TEST));
        assert!(g.is_enabled(gl::DEPTH_TEST));
        assert!(g.get_parameter_bool(gl::DEPTH_WRITEMASK));
        assert_eq!(
            g.get_parameter_bool_array::<4>(gl::COLOR_WRITEMASK),
            [false, true, false, true]
        );
        assert_eq!(g.get_parameter_i32(gl::ACTIVE_TEXTURE), gl::TEXTURE3 as i32);
        assert_eq!(g.get_parameter_i32(gl::UNPACK_ROW_LENGTH), 9);
        assert_eq!(g.get_parameter_i32(gl::UNPACK_ALIGNMENT), 8);
        assert_eq!(g.get_error(), gl::NO_ERROR);
    }
    assert_pixel(&g, 12, 12, [64, 128, 192]);
    assert_pixel(&g, 180, 30, [0, 255, 0]);
    assert_pixel(&g, 180, 12, [255, 255, 255]);
    assert_pixel(&g, 30, 100, [255, 0, 0]);
    assert_pixel(&g, 15, 85, [255, 255, 255]);
    assert_pixel(&g, 10, 80, [0, 0, 0]);
    assert_pixel(&g, 95, 100, [255, 255, 255]);
    assert_pixel(&g, 44, 104, [0, 0, 255]);
    let mut glyphs = 0;
    for y in 20..50 {
        for x in 20..140 {
            let p = pixel(&g, x, y);
            if p[0] > 150 && p[1] < 80 && p[2] < 100 {
                glyphs += 1;
            }
        }
    }
    assert!(glyphs > 20);
    assert!(
        backend
            .render_cached_with_canvases(&ui, surface, |_, _| Err("callback failed".into()))
            .is_err()
    );
    unsafe {
        assert_eq!(g.get_parameter_i32(gl::ACTIVE_TEXTURE), gl::TEXTURE3 as i32);
        assert!(g.is_enabled(gl::DEPTH_TEST));
    }
    // HiDPI: same logical canvas geometry, twice the viewport/scissor in pixels.
    ui.set_viewport(160., 120.).unwrap();
    let mut hidpi = false;
    backend
        .render_cached_with_canvases(
            &ui,
            Surface::new([160., 120.], [320, 240]).unwrap(),
            |info, _| {
                hidpi = true;
                assert_eq!(info.pixel_size(), [152, 92]);
                Ok(())
            },
        )
        .unwrap();
    assert!(hidpi);
    // Minimized frame skips native callbacks without dividing by zero.
    backend
        .render_cached_with_canvases(&ui, Surface::new([320., 240.], [0, 0]).unwrap(), |_, _| {
            panic!("zero surface")
        })
        .unwrap();
    ui.set_viewport(320., 240.).unwrap();
    // Rendering into an existing offscreen framebuffer must never redirect later
    // UI paint to the window, even if the native callback temporarily binds it.
    unsafe {
        let texture = g.create_texture().unwrap();
        g.bind_texture(gl::TEXTURE_2D, Some(texture));
        g.tex_image_2d(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as i32,
            320,
            240,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            gl::PixelUnpackData::Slice(None),
        );
        let framebuffer = g.create_framebuffer().unwrap();
        g.bind_framebuffer(gl::FRAMEBUFFER, Some(framebuffer));
        g.framebuffer_texture_2d(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            Some(texture),
            0,
        );
        assert_eq!(
            g.check_framebuffer_status(gl::FRAMEBUFFER),
            gl::FRAMEBUFFER_COMPLETE
        );
        backend
            .render_cached_with_canvases(&ui, surface, |_, painter| {
                let g = painter.gl();
                g.clear_color(1., 0., 0., 1.);
                g.clear(gl::COLOR_BUFFER_BIT);
                g.bind_framebuffer(gl::FRAMEBUFFER, None);
                Ok(())
            })
            .unwrap();
        assert_eq!(
            g.get_parameter_framebuffer(gl::DRAW_FRAMEBUFFER_BINDING),
            Some(framebuffer)
        );
        assert_eq!(
            g.get_parameter_framebuffer(gl::READ_FRAMEBUFFER_BINDING),
            Some(framebuffer)
        );
        assert_pixel(&g, 44, 104, [0, 0, 255]);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = backend.render_cached_with_canvases(&ui, surface, |_, painter| {
                painter.gl().bind_framebuffer(gl::FRAMEBUFFER, None);
                panic!("intentional canvas panic");
            });
        }));
        assert!(result.is_err());
        assert_eq!(
            g.get_parameter_framebuffer(gl::DRAW_FRAMEBUFFER_BINDING),
            Some(framebuffer)
        );
        backend.render(&ui, surface).unwrap();
        g.bind_framebuffer(gl::FRAMEBUFFER, None);
        g.delete_framebuffer(framebuffer);
        g.delete_texture(texture);
    }
    assert!(backend.remove_image("tile"));
    assert!(!backend.remove_image("tile"));
    backend.clear_text_cache();
    verify_antialias(&mut backend, &g);
    cache::verify(&g);
    unsafe {
        assert_eq!(g.get_error(), gl::NO_ERROR);
    }
    println!(
        "OpenGL pixel checks passed: text, image fitting, canvas/ancestor clipping, overlay order, HiDPI, minimized surfaces, host state, callback errors and 1x/1.5x/2x/3x shape antialiasing"
    );
}
