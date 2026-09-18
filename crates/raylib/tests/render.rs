use native_ui::{Key, Ui};
use native_ui_raylib::RaylibBackend;
use raylib::prelude::*;

fn render_frame(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    draw: impl FnOnce(&mut RaylibDrawHandle<'_>),
) -> Image {
    {
        let mut frame = rl.begin_drawing(thread);
        draw(&mut frame);
        // Read the back buffer before EndDrawing swaps it.
        // SAFETY: an active drawing frame owns a valid raylib graphics context.
        unsafe { raylib::ffi::rlDrawRenderBatchActive() };
        let mut image = frame.load_image_from_screen(thread);
        image.resize_nn(frame.get_screen_width(), frame.get_screen_height());
        image
    }
}

fn assert_pixel(image: &Image, x: i32, y: i32, expected: Color, tolerance: u8) {
    let p = image.get_color(x, y);
    assert!(
        p.r.abs_diff(expected.r) <= tolerance
            && p.g.abs_diff(expected.g) <= tolerance
            && p.b.abs_diff(expected.b) <= tolerance,
        "pixel ({x}, {y}): expected {expected:?}, got {p:?}"
    );
}

// A standalone test executable keeps Cocoa/GLFW initialization on the main thread.
fn main() {
    let mut builder = raylib::init();
    builder
        .size(320, 240)
        .title("raylib connector pixel checks")
        .hidden();
    if std::env::args().any(|arg| arg == "--highdpi") {
        builder.highdpi();
    }
    let (mut rl, thread) = builder.build();
    {
        let (rl, thread) = (&mut rl, &thread);
        let mut backend = RaylibBackend::new(rl, thread).unwrap();
        let mut ui = Ui::from_html_css(
            "<button id=button>Café</button><img id=image src=tile><canvas id=scene></canvas><input id=text value=Hello>",
            "button,img,canvas,input{position:absolute} button{left:10px;top:10px;width:140px;height:50px;background:#4080c0;color:#ff0000;padding:10px} img{left:170px;top:10px;width:40px;height:40px;object-fit:contain} canvas{left:10px;top:80px;width:100px;height:70px;padding:10px;border:2px solid #000000;background:#ffffff} input{left:10px;top:180px;width:200px;height:30px;background:#ffffff}",
        ).unwrap();
        backend
            .register_image_rgba("tile", 2, 1, &[0, 255, 0, 255, 0, 255, 0, 255])
            .unwrap();
        assert!(backend.register_image_rgba("tile", 2, 1, &[0]).is_err());
        assert!(
            backend
                .register_image_rgba("bad", u32::MAX, 2, &[])
                .is_err()
        );
        let mut calls = 0;
        let image = render_frame(rl, thread, |draw| {
            draw.clear_background(Color::WHITE);
            backend
                .render_with_canvases(&ui, draw, |region, draw| {
                    calls += 1;
                    assert_eq!(region.id, Some("scene"));
                    // Deliberately oversize: only the content box should be red.
                    draw.draw_rectangle(0, 0, 320, 240, Color::RED);
                    Ok(())
                })
                .unwrap();
            // Verify the callback's clip has been removed before host drawing.
            draw.draw_rectangle(280, 200, 10, 10, Color::BLUE);
        });
        assert_eq!(calls, 1);
        assert_pixel(&image, 12, 12, Color::new(64, 128, 192, 255), 0);
        assert_pixel(&image, 180, 30, Color::new(0, 255, 0, 255), 0);
        assert_pixel(&image, 180, 12, Color::WHITE, 0); // object-fit letterboxing
        assert_pixel(&image, 30, 100, Color::RED, 0);
        assert_pixel(&image, 15, 85, Color::WHITE, 0); // canvas padding
        assert_pixel(&image, 10, 80, Color::BLACK, 0); // canvas border
        assert_pixel(&image, 280, 200, Color::BLUE, 0);
        let mut red_glyphs = 0;
        for y in 10..60 {
            for x in 10..150 {
                let p = image.get_color(x, y);
                if p.r > 150 && p.g < 80 && p.b < 100 {
                    red_glyphs += 1;
                    assert!((20..140).contains(&x) && (20..50).contains(&y));
                }
            }
        }
        assert!(red_glyphs > 20, "bundled font should paint colored glyphs");
        let built = backend.cache_stats();
        let cached = render_frame(rl, thread, |draw| {
            draw.clear_background(Color::WHITE);
            backend
                .render_with_canvases(&ui, draw, |_, draw| {
                    calls += 1;
                    draw.draw_rectangle(0, 0, 320, 240, Color::BLUE);
                    Ok(())
                })
                .unwrap();
        });
        assert_eq!(calls, 2, "native animation must run on every cached frame");
        assert_eq!(backend.cache_stats().rebuilds, built.rebuilds);
        assert_eq!(
            backend.cache_stats().text_measurements,
            built.text_measurements
        );
        assert_eq!(
            backend.cache_stats().paint_commands_built,
            built.paint_commands_built
        );
        assert_eq!(backend.cache_stats().reuses, built.reuses + 1);
        assert_pixel(&cached, 30, 100, Color::BLUE, 0);
        assert_pixel(&cached, 15, 85, Color::WHITE, 0);
        let direct = render_frame(rl, thread, |draw| {
            draw.clear_background(Color::WHITE);
            backend
                .render_uncached_with_canvases(&ui, draw, |_, draw| {
                    draw.draw_rectangle(0, 0, 320, 240, Color::BLUE);
                    Ok(())
                })
                .unwrap();
        });
        assert_eq!(
            cached.get_image_data_u8(false),
            direct.get_image_data_u8(false)
        );
        // Replacing an image must invalidate both fitted geometry and painted pixels.
        backend
            .register_image_rgba("tile", 1, 2, &[0, 0, 255, 255, 0, 0, 255, 255])
            .unwrap();
        let replaced = render_frame(rl, thread, |draw| {
            draw.clear_background(Color::WHITE);
            backend.render(&ui, draw).unwrap();
        });
        assert_pixel(&replaced, 190, 12, Color::BLUE, 0);
        assert_pixel(&replaced, 172, 30, Color::WHITE, 0);
        let changed = backend.cache_stats().rebuilds;
        ui.pointer_down(40.0, 35.0);
        ui.pointer_up(40.0, 35.0);
        assert!(ui.clicked("button"));
        ui.end_frame();
        ui.pointer_down(40.0, 195.0);
        ui.pointer_up(40.0, 195.0);
        ui.key_down(Key::SelectAll, false);
        ui.text_input("éλ");
        assert_eq!(ui.get_value("text"), Some("éλ"));

        let image = render_frame(rl, thread, |draw| {
            draw.clear_background(Color::WHITE);
            let result =
                backend.render_with_canvases(&ui, draw, |_, _| Err("callback failed".into()));
            assert!(result.is_err());
            draw.draw_rectangle(280, 200, 10, 10, Color::BLUE);
        });
        assert_pixel(&image, 280, 200, Color::BLUE, 0);
        assert_eq!(
            backend.cache_stats().rebuilds,
            changed + 1,
            "input edits refresh cached UI"
        );
        assert!(backend.remove_image("tile"));
        assert!(!backend.remove_image("tile"));
        backend.clear_text_cache();
    }
    println!(
        "raylib pixel checks passed: text, images, canvas clipping, callback errors and UI input"
    );
}
