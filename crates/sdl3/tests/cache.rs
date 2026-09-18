use native_ui::{Key, Ui};
use native_ui_sdl3::SdlBackend;
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::{BlendMode, Canvas, FRect},
    surface::Surface,
};

fn surface(width: u32, height: u32) -> Canvas<Surface<'static>> {
    Surface::new(width, height, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap()
}
fn pixels(canvas: &Canvas<Surface<'_>>) -> Vec<u8> {
    let surface = canvas
        .surface()
        .convert_format(PixelFormat::RGBA32)
        .unwrap();
    surface.with_lock(|bytes| bytes.to_vec())
}
fn scene() -> Ui {
    Ui::from_html_css(
        "<canvas id=first></canvas><button>Overlay</button><canvas id=second></canvas><input id=text value=Hello><select id=pick><option>One</option><option>Two</option></select>",
        "canvas,button,input,select{position:absolute;left:10px;top:10px;width:110px;height:30px}canvas{height:90px;border:2px solid red;padding:3px;background:#fff8}#second{left:65px;top:50px}button{left:25px;top:30px;background:linear-gradient(#88f8,#fff4);border-radius:8px}#text{top:110px}#pick{left:130px;top:0;width:70px}option{background:#246;color:white}",
    ).unwrap()
}
fn native<R: sdl3::render::RenderTarget>(
    region: native_ui::CanvasRegion<'_>,
    canvas: &mut Canvas<R>,
    frame: u8,
) -> Result<(), native_ui_sdl3::BackendError> {
    canvas.set_draw_color(if region.id == Some("first") {
        Color::RGB(frame, 160, 30)
    } else {
        Color::RGB(50, 40, frame)
    });
    let area = region.content;
    canvas.fill_rect(FRect::new(area.x, area.y, area.w, area.h))?;
    // A native callback may change state; the cache must restore it too.
    canvas.set_scale(3.0, 2.0)?;
    canvas.set_clip_rect(None);
    canvas.set_viewport(Rect::new(2, 3, 20, 20));
    canvas.set_blend_mode(BlendMode::Add);
    Ok(())
}

#[test]
fn cached_frames_match_direct_paint_with_live_canvases_alpha_and_popups() {
    for scale in [1.0, 1.5, 2.0] {
        let mut ui = scene();
        let mut direct = surface((240.0 * scale) as u32, (180.0 * scale) as u32);
        let mut cached = surface((240.0 * scale) as u32, (180.0 * scale) as u32);
        let direct_creator = direct.texture_creator();
        let cached_creator = cached.texture_creator();
        let mut a = SdlBackend::new(&direct_creator).unwrap();
        let mut b = SdlBackend::new(&cached_creator).unwrap();
        for frame in 0..6 {
            if frame == 2 {
                ui.pointer_down(140.0, 15.0);
                ui.pointer_up(140.0, 15.0);
                ui.key_down(Key::Down, false);
            }
            if frame == 4 {
                ui.key_down(Key::Escape, false);
            }
            for canvas in [&mut direct, &mut cached] {
                canvas.set_draw_color(Color::RGB(90 + frame, 20, 80));
                canvas.clear();
                canvas.set_scale(scale, scale).unwrap();
                canvas.set_viewport(Rect::new(3, 4, 225, 160));
                canvas.set_clip_rect(Rect::new(0, 0, 205, 150));
                canvas.set_blend_mode(BlendMode::None);
            }
            a.render_with_canvases(&ui, &mut direct, |r, c| native(r, c, frame * 30))
                .unwrap();
            let state = (
                cached.scale(),
                cached.viewport(),
                cached.clip_rect(),
                cached.draw_color(),
                cached.logical_size(),
            );
            let mut calls = Vec::new();
            b.render_cached_with_canvases(&ui, &mut cached, |r, c| {
                calls.push(r.id.unwrap().to_owned());
                native(r, c, frame * 30)
            })
            .unwrap();
            assert_eq!(calls, ["first", "second"]);
            assert_eq!(
                (
                    cached.scale(),
                    cached.viewport(),
                    cached.clip_rect(),
                    cached.draw_color(),
                    cached.logical_size()
                ),
                state
            );
            assert_eq!(cached.blend_mode(), BlendMode::None);
            direct.present();
            cached.present();
            let a_pixels = pixels(&direct);
            let b_pixels = pixels(&cached);
            // Layer compositing introduces at most small 8-bit rounding differences.
            let worst = a_pixels
                .iter()
                .zip(&b_pixels)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            assert!(
                worst <= 4,
                "scale {scale}, frame {frame}: channel error {worst}; {:?}",
                b.cache_stats()
            );
            ui.end_frame();
        }
        assert_eq!(b.cache_stats().rebuilds, 3);
        assert_eq!(b.cache_stats().reuses, 3);
    }
}

#[test]
fn warm_game_frames_do_no_ui_calculation_and_invalidation_rebuilds() {
    let mut ui = scene();
    let mut canvas = surface(240, 180);
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    ui.pointer_move(20.0, 20.0);
    backend.render_cached(&ui, &mut canvas).unwrap();
    let cold = backend.cache_stats();
    assert!(cold.paint_commands_built > 100);
    assert!(cold.text_measurements > 0);
    let mut calls = 0;
    for frame in 0..60 {
        ui.pointer_move(20.0 + frame as f32 * 0.01, 20.0);
        ui.end_frame();
        canvas.clear();
        backend
            .render_cached_with_canvases(&ui, &mut canvas, |_, _| {
                calls += 1;
                Ok(())
            })
            .unwrap();
    }
    let warm = backend.cache_stats();
    assert_eq!(calls, 120);
    assert_eq!(warm.rebuilds, 1);
    assert_eq!(warm.reuses, 60);
    assert_eq!(warm.paint_commands_built, cold.paint_commands_built);
    assert_eq!(warm.text_measurements, cold.text_measurements);
    ui.set_value("text", "Updated").unwrap();
    backend.render_cached(&ui, &mut canvas).unwrap();
    assert_eq!(backend.cache_stats().rebuilds, 2);
    canvas.set_scale(1.5, 1.5).unwrap();
    backend.render_cached(&ui, &mut canvas).unwrap();
    assert_eq!(backend.cache_stats().rebuilds, 3);
    canvas.set_clip_rect(Rect::new(0, 0, 50, 50));
    backend.render_cached(&ui, &mut canvas).unwrap();
    assert_eq!(backend.cache_stats().rebuilds, 4);
    backend.render_cached(&scene(), &mut canvas).unwrap();
    assert_eq!(
        backend.cache_stats().rebuilds,
        5,
        "replacing the UI invalidates the cache"
    );
    backend.clear_caches();
    assert_eq!(backend.cache_stats().texture_bytes, 0);
    backend.render_cached(&ui, &mut canvas).unwrap();
    assert_eq!(backend.cache_stats().rebuilds, 6);
}

#[test]
fn cached_callback_errors_restore_state_and_leave_cache_reusable() {
    let ui = scene();
    let mut canvas = surface(240, 180);
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    canvas.set_draw_color(Color::RGBA(10, 20, 30, 200));
    canvas.set_blend_mode(BlendMode::None);
    let before = (
        canvas.scale(),
        canvas.viewport(),
        canvas.clip_rect(),
        canvas.draw_color(),
    );
    let result = backend.render_cached_with_canvases(&ui, &mut canvas, |_, c| {
        c.set_scale(4.0, 3.0)?;
        c.set_viewport(Rect::new(2, 3, 20, 20));
        c.set_clip_rect(Rect::new(1, 2, 3, 4));
        Err("native failure".into())
    });
    assert_eq!(result.unwrap_err().to_string(), "native failure");
    assert_eq!(
        (
            canvas.scale(),
            canvas.viewport(),
            canvas.clip_rect(),
            canvas.draw_color()
        ),
        before
    );
    assert_eq!(canvas.blend_mode(), BlendMode::None);
    assert!(!unsafe { sdl3::sys::render::SDL_RenderViewportSet(canvas.raw()) });
    backend.render_cached(&ui, &mut canvas).unwrap();
    assert_eq!(backend.cache_stats().rebuilds, 1);
    assert_eq!(backend.cache_stats().reuses, 1);
}

#[test]
fn failed_cache_build_restores_state_and_recovers() {
    let mut canvas = surface(240, 180);
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    let too_large = Ui::from_html_css(
        &format!("<button>{}</button>", "W".repeat(1000)),
        "button{position:absolute;left:0;top:0;width:200px;height:40px;font-size:256px}",
    )
    .unwrap();
    canvas.set_scale(1.5, 1.5).unwrap();
    canvas.set_clip_rect(Rect::new(0, 0, 120, 100));
    canvas.set_draw_color(Color::RGB(12, 34, 56));
    let before = (
        canvas.scale(),
        canvas.viewport(),
        canvas.clip_rect(),
        canvas.draw_color(),
    );
    assert!(backend.render_cached(&too_large, &mut canvas).is_err());
    assert_eq!(
        (
            canvas.scale(),
            canvas.viewport(),
            canvas.clip_rect(),
            canvas.draw_color()
        ),
        before
    );
    assert_eq!(backend.cache_stats().texture_bytes, 0);
    backend.render_cached(&scene(), &mut canvas).unwrap();
    assert_eq!(backend.cache_stats().rebuilds, 1);
}

#[test]
fn cached_text_keeps_pointer_editing_and_host_updates_working() {
    let mut canvas = surface(240, 180);
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    let mut ui = scene();
    backend.render_cached(&ui, &mut canvas).unwrap();
    backend.render_cached(&ui, &mut canvas).unwrap();
    ui.pointer_down(11.0, 125.0);
    ui.pointer_up(11.0, 125.0);
    ui.text_input("X");
    assert_eq!(ui.get_value("text"), Some("XHello"));
    backend.render_cached(&ui, &mut canvas).unwrap();
    ui.key_down(Key::SelectAll, false);
    backend.render_cached(&ui, &mut canvas).unwrap();
    assert_eq!(ui.selected_text(), Some("XHello"));
    ui.set_value("text", "Updated").unwrap();
    backend.render_cached(&ui, &mut canvas).unwrap();
    ui.pointer_down(11.0, 125.0);
    ui.pointer_up(11.0, 125.0);
    ui.text_input("Y");
    assert_eq!(ui.get_value("text"), Some("YUpdated"));
}

#[test]
fn scrolling_clips_native_images_and_controls_and_invalidates_only_when_moved() {
    for scale in [1.0, 1.5, 2.0] {
        let mut ui=Ui::from_html_css_with_viewport("<aside id=p><canvas id=scene></canvas><img src=photo><button>Bottom</button></aside>","aside{width:100px;height:80px;overflow:auto;background:white}canvas{width:130px;height:100px}img{width:130px;height:40px}button{height:40px;background:red}::scrollbar{width:10px;height:10px}::scrollbar-thumb{background:blue}",160.0,140.0).unwrap();
        let mut direct = surface((160.0 * scale) as u32, (140.0 * scale) as u32);
        let mut cached = surface((160.0 * scale) as u32, (140.0 * scale) as u32);
        let ca = direct.texture_creator();
        let cb = cached.texture_creator();
        let mut a = SdlBackend::new(&ca).unwrap();
        let mut b = SdlBackend::new(&cb).unwrap();
        for backend in [&mut a, &mut b] {
            backend
                .register_image_rgba("photo", 1, 1, &[240, 180, 30, 255])
                .unwrap();
        }
        for offset in [0.0, 0.0, 60.0, 60.0, 110.0, 110.0] {
            ui.set_scroll_offset("p", 20.0, offset).unwrap();
            for c in [&mut direct, &mut cached] {
                c.set_draw_color(Color::RGB(10, 20, 30));
                c.clear();
                c.set_scale(scale, scale).unwrap();
            }
            let draw = |r: native_ui::CanvasRegion<'_>,
                        c: &mut Canvas<Surface<'static>>|
             -> Result<(), native_ui_sdl3::BackendError> {
                assert_eq!(r.content.w, 130.0);
                assert_eq!(r.content.h, 100.0);
                assert!(
                    r.clip.x >= 0.0
                        && r.clip.y >= 0.0
                        && r.clip.x + r.clip.w <= 90.0
                        && r.clip.y + r.clip.h <= 70.0
                );
                c.set_draw_color(Color::RGB(0, 200, 0));
                c.fill_rect(FRect::new(-100.0, -100.0, 400.0, 400.0))?;
                Ok(())
            };
            a.render_with_canvases(&ui, &mut direct, draw).unwrap();
            b.render_cached_with_canvases(&ui, &mut cached, draw)
                .unwrap();
            direct.present();
            cached.present();
            assert_eq!(pixels(&direct), pixels(&cached));
            let surface = cached
                .surface()
                .convert_format(PixelFormat::RGBA32)
                .unwrap();
            let pitch = surface.pitch() as usize;
            surface.with_lock(|bytes| {
                for (x, y) in [(110.0, 20.0), (20.0, 90.0)] {
                    let at = (y * scale) as usize * pitch + (x * scale) as usize * 4;
                    assert_eq!(
                        &bytes[at..at + 4],
                        &[10, 20, 30, 255],
                        "overflow must not leak at {offset}"
                    );
                }
            });
        }
        assert_eq!(b.cache_stats().rebuilds, 3);
        assert_eq!(b.cache_stats().reuses, 3);
        assert_eq!(b.cache_stats().image_uploads, 1);
    }
}

#[test]
fn replacing_select_options_rebuilds_cached_text_and_removes_old_popup() {
    use native_ui::SelectItem;
    let mut ui = Ui::from_html_css("<select id=pick><option>One</option><option>Two</option></select>", "select{position:absolute;left:130px;top:0;width:100px;height:30px;background:white}option{background:#246;color:white}").unwrap();
    let mut canvas = surface(240, 180);
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    ui.pointer_down(140.0, 15.0);
    ui.pointer_up(140.0, 15.0);
    backend.render_cached(&ui, &mut canvas).unwrap();
    canvas.present();
    let old = pixels(&canvas);
    ui.set_select_options("pick", &[SelectItem::new("new", "New label")])
        .unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    backend.render_cached(&ui, &mut canvas).unwrap();
    assert_eq!(backend.cache_stats().rebuilds, 2);
    canvas.present();
    let updated = pixels(&canvas);
    assert!(old != updated, "replacement must change cached pixels");
    backend.render_cached(&ui, &mut canvas).unwrap();
    assert_eq!(backend.cache_stats().reuses, 1);
    canvas.clear();
    backend.render(&ui, &mut canvas).unwrap();
    canvas.present();
    assert!(
        updated == pixels(&canvas),
        "updated cache must match direct paint"
    );
}

#[test]
fn typed_insertion_rebuilds_cached_layers_and_then_resumes_reuse() {
    use native_ui::Button;
    let mut ui = Ui::from_html_css_with_viewport(
        "<div id=parent></div>",
        "button{height:30px;background:white;color:black}",
        240.0,
        180.0,
    )
    .unwrap();
    let mut canvas = surface(240, 180);
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    backend.render_cached(&ui, &mut canvas).unwrap();
    canvas.present();
    let old = pixels(&canvas);
    ui.get("parent")
        .unwrap()
        .add_button(Button::new("newButton"))
        .unwrap();
    canvas.clear();
    backend.render_cached(&ui, &mut canvas).unwrap();
    canvas.present();
    let updated = pixels(&canvas);
    assert!(old != updated);
    assert_eq!(backend.cache_stats().rebuilds, 2);
    backend.render_cached(&ui, &mut canvas).unwrap();
    assert_eq!(backend.cache_stats().reuses, 1);
    canvas.clear();
    backend.render(&ui, &mut canvas).unwrap();
    canvas.present();
    assert!(updated == pixels(&canvas));
}
