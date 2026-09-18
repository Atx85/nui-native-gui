use native_ui::Ui;
use native_ui_sdl3::SdlBackend;
use sdl3::{
    pixels::{Color, PixelFormat},
    surface::Surface,
};

fn overlay() -> Ui {
    Ui::from_html_css_with_viewport(
        include_str!("../../../examples/showcase/ui/overlay.html"),
        &format!(
            "{}\n{}\n{}",
            include_str!("../../../examples/showcase/ui/themes/macos-light.css"),
            include_str!("../../../examples/showcase/ui/studio.css"),
            include_str!("../../../examples/showcase/ui/overlay.css")
        ),
        1000.0,
        680.0,
    )
    .unwrap()
}

#[test]
fn floating_controls_cover_canvas_and_intercept_input_after_resize() {
    let mut ui = overlay();
    for (w, h) in [(1000.0, 680.0), (700.0, 600.0), (1280.0, 800.0)] {
        ui.set_viewport(w, h).unwrap();
        let scene = ui.canvas_region("scene").unwrap();
        assert_eq!(
            scene.content,
            native_ui::Rect {
                x: 0.0,
                y: 0.0,
                w,
                h
            }
        );
        let panel = ui.bounds("options").unwrap();
        assert!(panel.y + panel.h <= h - 24.0);
        // A blank piece of the panel also blocks pointer presses.
        let (x, y) = (panel.x + 5.0, panel.y + 5.0);
        assert!(ui.canvas_at(x, y).is_none());
        ui.pointer_down(x, y);
        assert!(ui.canvas_input("scene").unwrap().pressed.is_none());
        ui.pointer_up(x, y);
        ui.end_frame();
        let select = ui.bounds("pattern").unwrap();
        ui.pointer_down(select.x + 10.0, select.y + 10.0);
        ui.pointer_up(select.x + 10.0, select.y + 10.0);
        assert!(ui.select_is_open());
        assert!(ui.canvas_input("scene").unwrap().pressed.is_none());
        ui.key_down(native_ui::Key::Escape, false);
        ui.end_frame();
        // Uncovered canvas remains interactive.
        ui.pointer_down(w - 40.0, h - 40.0);
        assert!(ui.canvas_input("scene").unwrap().pressed.is_some());
        ui.pointer_up(w - 40.0, h - 40.0);
        ui.end_frame();
    }
}

#[test]
fn floating_ui_is_composited_over_live_native_drawing_and_cache_is_reused() {
    let ui = overlay();
    let mut canvas = Surface::new(1000, 680, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap();
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    backend
        .register_image_rgba("studio-mark", 4, 2, &[82, 202, 246, 255].repeat(8))
        .unwrap();
    for frame in 0..2 {
        canvas.set_draw_color(Color::RGB(23, 35, 49));
        canvas.clear();
        let mut calls = 0;
        backend
            .render_cached_with_canvases(&ui, &mut canvas, |r, c| {
                calls += 1;
                c.set_draw_color(Color::RGB(38, 55, 73));
                for x in (0..1000).step_by(32) {
                    c.draw_line((x as f32, 0.0), (x as f32, r.content.h))?;
                }
                for y in (0..680).step_by(32) {
                    c.draw_line((0.0, y as f32), (r.content.w, y as f32))?;
                }
                for (layer, color) in [Color::RGB(82, 202, 246), Color::RGB(143, 227, 174)]
                    .into_iter()
                    .enumerate()
                {
                    c.set_draw_color(color);
                    let mut previous = (0.0, 340.0);
                    for step in 0..=720 {
                        let t = step as f32 / 720.0;
                        let point = (
                            t * 1000.0,
                            340.0
                                + 170.0
                                    * (t * std::f32::consts::TAU * 1.5
                                        + layer as f32 * 1.7
                                        + frame as f32 * 0.02)
                                        .sin(),
                        );
                        if step > 0 {
                            c.draw_line(previous, point)?;
                        }
                        previous = point;
                    }
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(calls, 1);
        canvas.present();
        let surface = canvas
            .surface()
            .convert_format(PixelFormat::RGBA32)
            .unwrap();
        let pitch = surface.pitch() as usize;
        surface.with_lock(|pixels| {
            assert!(
                pixels[120 * pitch + 30 * 4] > 200,
                "floating panel must cover the native scene"
            );
            assert!(
                pixels[120 * pitch + 800 * 4] < 100,
                "uncovered native scene must remain visible"
            );
        });
        if frame == 0
            && let Ok(path) = std::env::var("NATIVE_UI_OVERLAY_SCREENSHOT")
        {
            canvas.surface().save_bmp(path).unwrap();
        }
    }
    assert_eq!(backend.cache_stats().rebuilds, 1);
    assert_eq!(backend.cache_stats().reuses, 1);
}
