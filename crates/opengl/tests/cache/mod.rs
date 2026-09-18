use native_ui::{Color, Key, Ui};
use native_ui_opengl::{
    OpenGlBackend, Surface,
    glow::{self as gl, HasContext},
};
use std::rc::Rc;

fn clear(g: &gl::Context, frame: u8) {
    unsafe {
        g.disable(gl::SCISSOR_TEST);
        g.color_mask(true, true, true, true);
        g.clear_color(0.2, frame as f32 / 255., 0.4, 0.7);
        g.clear(gl::COLOR_BUFFER_BIT);
    }
}
fn pixels(g: &gl::Context) -> Vec<u8> {
    let mut p = vec![0; 320 * 240 * 4];
    unsafe {
        g.read_pixels(
            0,
            0,
            320,
            240,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            gl::PixelPackData::Slice(Some(&mut p)),
        );
    }
    p
}
fn scene() -> Ui {
    Ui::from_html_css_with_viewport(
        "<canvas id=first></canvas><button>Overlay</button><canvas id=second></canvas><input id=text value=Hello><select id=pick><option>One</option><option>Two</option></select><img src=tile>",
        "canvas,button,input,select,img{position:absolute;left:10px;top:10px;width:110px;height:30px}canvas{height:90px;border:2px solid red;padding:3px;background:#fff8}#second{left:65px;top:50px}button{left:25px;top:30px;background:linear-gradient(#88f8,#fff4);border-radius:8px}#text{top:110px}#pick{left:130px;top:0;width:70px}option{background:#246;color:white}img{left:10px;top:150px;width:30px;height:20px}",320.,240.,
    ).unwrap()
}
pub fn verify(g: &Rc<gl::Context>) {
    let mut direct = unsafe { OpenGlBackend::new(g.clone()) }.unwrap();
    let mut cached = unsafe { OpenGlBackend::new(g.clone()) }.unwrap();
    for backend in [&mut direct, &mut cached] {
        backend
            .register_image_rgba("tile", 1, 1, &[230, 50, 90, 180])
            .unwrap();
    }
    // Exercise texture layers, partial budget fallback, command-only fallback and
    // host sRGB policy. Every cached frame is compared with independently drawn UI.
    for (budget, srgb) in [
        (64 * 1024 * 1024, false),
        (320 * 240 * 4, false),
        (0, false),
        (64 * 1024 * 1024, true),
    ] {
        cached.set_cache_texture_limit(budget);
        unsafe {
            if srgb {
                g.enable(gl::FRAMEBUFFER_SRGB)
            } else {
                g.disable(gl::FRAMEBUFFER_SRGB)
            }
        }
        for scale in [1., 1.5, 2., 3.] {
            let surface = Surface::new([320. / scale, 240. / scale], [320, 240]).unwrap();
            let mut ui = scene();
            let before = cached.cache_stats();
            for frame in 0..8 {
                if frame == 2 {
                    ui.set_value("text", "Updated").unwrap();
                }
                if frame == 4 {
                    ui.pointer_down(140., 15.);
                    ui.pointer_up(140., 15.);
                    ui.key_down(Key::Down, false);
                }
                if frame == 6 {
                    ui.key_down(Key::Escape, false);
                }
                let mut draws = 0;
                let mut draw =
                    |info: native_ui_opengl::CanvasInfo<'_>,
                     painter: &native_ui_opengl::CanvasPainter<'_>| {
                        draws += 1;
                        let color = if info.region.id == Some("first") {
                            Color(frame * 25, 150, 25, 255)
                        } else {
                            Color(30, 40, frame * 25, 255)
                        };
                        painter.fill(info.region.content, color);
                        Ok(())
                    };
                clear(g, frame * 10);
                direct
                    .render_with_canvases(&ui, surface, &mut draw)
                    .unwrap();
                let reference = pixels(g);
                clear(g, frame * 10);
                cached
                    .render_cached_with_canvases(&ui, surface, &mut draw)
                    .unwrap();
                let actual = pixels(g);
                assert_eq!(draws, 4, "two live canvases per renderer, even on reuse");
                let worst = reference
                    .iter()
                    .zip(&actual)
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap();
                assert!(
                    worst <= 3,
                    "cached pixels differ by {worst}: scale={scale}, frame={frame}, budget={budget}, srgb={srgb}"
                );
                ui.end_frame();
            }
            let after = cached.cache_stats();
            assert_eq!(after.rebuilds - before.rebuilds, 4);
            assert_eq!(after.reuses - before.reuses, 4);
            if budget == 0 || srgb {
                assert_eq!(after.texture_layers, 0);
                assert!(after.command_layers > 0);
            } else {
                assert!(after.texture_layers > 0);
            }
            if budget == 320 * 240 * 4 {
                assert_eq!(after.texture_layers, 1);
                assert!(after.command_layers > 0);
            }
            let stable = cached.cache_stats();
            cached.render_cached(&ui, surface).unwrap();
            let reused = cached.cache_stats();
            assert_eq!(stable.paint_commands_built, reused.paint_commands_built);
            assert_eq!(stable.text_measurements, reused.text_measurements);
        }
    }
    unsafe {
        g.disable(gl::FRAMEBUFFER_SRGB);
    }
    let surface = Surface::new([320., 240.], [320, 240]).unwrap();
    let ui = scene();
    cached.render_cached(&ui, surface).unwrap();
    let baseline = cached.cache_stats().rebuilds;
    cached
        .register_image_rgba("tile", 1, 1, &[0, 255, 0, 255])
        .unwrap();
    clear(g, 0);
    cached.render_cached(&ui, surface).unwrap();
    let p = pixels(g);
    let index = ((239 - 155) * 320 + 15) * 4;
    assert_eq!(&p[index..index + 3], &[0, 255, 0]);
    assert_eq!(cached.cache_stats().rebuilds, baseline + 1);
    assert!(cached.remove_image("tile"));
    clear(g, 0);
    cached.render_cached(&ui, surface).unwrap();
    assert_ne!(&pixels(g)[index..index + 3], &[0, 255, 0]);
    assert_eq!(cached.cache_stats().rebuilds, baseline + 2);
    cached.clear_caches();
    assert_eq!(cached.cache_stats().texture_bytes, 0);
    cached.render_cached(&ui, surface).unwrap();
    assert_eq!(cached.cache_stats().rebuilds, baseline + 3);
    // Replacing the Ui instance and changing output geometry both invalidate.
    cached.render_cached(&scene(), surface).unwrap();
    assert_eq!(cached.cache_stats().rebuilds, baseline + 4);
    cached
        .render_cached(&ui, Surface::new([320., 240.], [160, 120]).unwrap())
        .unwrap();
    assert_eq!(cached.cache_stats().rebuilds, baseline + 5);
    let stats = cached.cache_stats();
    cached
        .render_cached(&ui, Surface::new([320., 240.], [0, 0]).unwrap())
        .unwrap();
    assert_eq!(stats, cached.cache_stats());
    println!(
        "OpenGL cache checks passed: reuse counters, live canvases, alpha, popups, edits, image lifecycle, DPI, resize, replacement, budgets and sRGB fallback"
    );
}
