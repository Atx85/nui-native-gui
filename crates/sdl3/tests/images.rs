use native_ui::Ui;
use native_ui_sdl3::SdlBackend;
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::{BlendMode, Canvas, ClippingRect},
    surface::Surface,
};

fn surface(scale: u32) -> Canvas<Surface<'static>> {
    Surface::new(60 * scale, 60 * scale, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap()
}
fn pixel(canvas: &Canvas<Surface<'_>>, x: usize, y: usize) -> [u8; 4] {
    let surface = canvas
        .surface()
        .convert_format(PixelFormat::RGBA32)
        .unwrap();
    let at = y * surface.pitch() as usize + x * 4;
    surface.with_lock(|bytes| bytes[at..at + 4].try_into().unwrap())
}
fn clear(canvas: &mut Canvas<Surface<'_>>) {
    canvas.set_draw_color(Color::RGB(10, 20, 30));
    canvas.clear();
}
fn image_ui(fit: &str) -> Ui {
    Ui::from_html_css("<img id=photo src=asset alt=Missing>", &format!(
        "img{{position:absolute;left:10px;top:10px;width:20px;height:20px;padding:0;border:none;background:transparent;object-fit:{fit};font-size:8px;color:white}}"
    )).unwrap()
}
fn solid(width: usize, height: usize, color: [u8; 4]) -> Vec<u8> {
    color.repeat(width * height)
}

#[test]
fn images_honor_fill_contain_and_cover_in_direct_and_cached_rendering() {
    let colors = [
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [255, 255, 255, 255],
    ];
    let mut stripes = Vec::new();
    for _ in 0..20 {
        for color in colors {
            stripes.extend(color.repeat(10));
        }
    }
    for cached in [false, true] {
        for fit in ["fill", "contain", "cover"] {
            let ui = image_ui(fit);
            let mut canvas = surface(1);
            let creator = canvas.texture_creator();
            let mut backend = SdlBackend::new(&creator).unwrap();
            backend
                .register_image_rgba("asset", 40, 20, &stripes)
                .unwrap();
            for _ in 0..2 {
                clear(&mut canvas);
                if cached {
                    backend.render_cached(&ui, &mut canvas).unwrap();
                } else {
                    backend.render(&ui, &mut canvas).unwrap();
                }
                canvas.present();
                assert_eq!(pixel(&canvas, 9, 20), [10, 20, 30, 255]);
                if fit == "cover" {
                    assert_eq!(pixel(&canvas, 12, 20), colors[1]);
                    assert_eq!(pixel(&canvas, 27, 20), colors[2]);
                } else {
                    assert_eq!(pixel(&canvas, 12, 20), colors[0]);
                    assert_eq!(pixel(&canvas, 17, 20), colors[1]);
                    assert_eq!(pixel(&canvas, 22, 20), colors[2]);
                    assert_eq!(pixel(&canvas, 27, 20), colors[3]);
                    assert_eq!(
                        pixel(&canvas, 12, 11),
                        if fit == "contain" {
                            [10, 20, 30, 255]
                        } else {
                            colors[0]
                        }
                    );
                }
            }
            assert_eq!(backend.cache_stats().image_uploads, 1);
            if cached {
                assert_eq!(backend.cache_stats().rebuilds, 1);
            }
        }
    }
}

#[test]
fn image_alpha_scaling_and_clipping_preserve_host_state() {
    for scale in [1, 2] {
        let ui = image_ui("fill");
        let mut canvas = surface(scale);
        let creator = canvas.texture_creator();
        let mut backend = SdlBackend::new(&creator).unwrap();
        backend
            .register_image_rgba("asset", 2, 2, &solid(2, 2, [255, 0, 0, 128]))
            .unwrap();
        canvas.set_scale(scale as f32, scale as f32).unwrap();
        canvas.set_clip_rect(Rect::new(14, 14, 12, 12));
        canvas.set_blend_mode(BlendMode::None);
        let saved = (
            canvas.scale(),
            canvas.clip_rect(),
            canvas.viewport(),
            canvas.logical_size(),
        );
        for cached in [false, true, true] {
            clear(&mut canvas);
            if cached {
                backend.render_cached(&ui, &mut canvas).unwrap();
            } else {
                backend.render(&ui, &mut canvas).unwrap();
            }
            assert_eq!(
                (
                    canvas.scale(),
                    canvas.clip_rect(),
                    canvas.viewport(),
                    canvas.logical_size()
                ),
                saved
            );
            assert_eq!(canvas.blend_mode(), BlendMode::None);
            assert_eq!(canvas.draw_color(), Color::RGB(10, 20, 30));
            canvas.present();
            let p = pixel(&canvas, 20 * scale as usize, 20 * scale as usize);
            for (actual, expected) in p.into_iter().zip([133_u8, 10, 15, 255]) {
                assert!(actual.abs_diff(expected) <= 1);
            }
            assert_eq!(
                pixel(&canvas, 12 * scale as usize, 20 * scale as usize),
                [10, 20, 30, 255]
            );
        }
        canvas.set_clip_rect(ClippingRect::Zero);
        clear(&mut canvas);
        backend.render_cached(&ui, &mut canvas).unwrap();
        canvas.present();
        assert_eq!(
            pixel(&canvas, 20 * scale as usize, 20 * scale as usize),
            [10, 20, 30, 255]
        );
    }
}

#[test]
fn replacing_removing_and_restoring_assets_invalidates_cached_pixels_and_geometry() {
    let ui = image_ui("contain");
    let revision = ui.paint_revision();
    let mut canvas = surface(1);
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    // Cache the missing asset's alt text, then register it without touching Ui.
    clear(&mut canvas);
    backend.render_cached(&ui, &mut canvas).unwrap();
    backend
        .register_image_rgba("asset", 40, 20, &solid(40, 20, [255, 0, 0, 255]))
        .unwrap();
    clear(&mut canvas);
    backend.render_cached(&ui, &mut canvas).unwrap();
    canvas.present();
    assert_eq!(pixel(&canvas, 12, 20), [255, 0, 0, 255]);
    assert_eq!(pixel(&canvas, 20, 11), [10, 20, 30, 255]);
    let cold = backend.cache_stats();
    for _ in 0..10 {
        backend.render_cached(&ui, &mut canvas).unwrap();
    }
    assert_eq!(backend.cache_stats().rebuilds, cold.rebuilds);
    assert_eq!(backend.cache_stats().image_uploads, cold.image_uploads);
    assert_eq!(
        backend.cache_stats().paint_commands_built,
        cold.paint_commands_built
    );
    backend
        .register_image_rgba("asset", 20, 40, &solid(20, 40, [0, 255, 0, 255]))
        .unwrap();
    clear(&mut canvas);
    backend.render_cached(&ui, &mut canvas).unwrap();
    canvas.present();
    assert_eq!(pixel(&canvas, 12, 20), [10, 20, 30, 255]);
    assert_eq!(pixel(&canvas, 20, 11), [0, 255, 0, 255]);
    let stable = backend.cache_stats();
    assert!(
        backend
            .register_image_rgba("asset", 20, 40, &[0; 8])
            .is_err()
    );
    assert!(backend.register_image_rgba("asset", 0, 1, &[]).is_err());
    assert!(
        backend
            .register_image_rgba("asset", u32::MAX, u32::MAX, &[])
            .is_err()
    );
    assert!(!backend.remove_image("unknown"));
    backend.render_cached(&ui, &mut canvas).unwrap();
    assert_eq!(backend.cache_stats().rebuilds, stable.rebuilds);
    assert_eq!(backend.cache_stats().image_uploads, stable.image_uploads);
    assert_eq!(pixel(&canvas, 20, 11), [0, 255, 0, 255]);
    backend.clear_caches();
    clear(&mut canvas);
    backend.render_cached(&ui, &mut canvas).unwrap();
    canvas.present();
    assert_eq!(pixel(&canvas, 20, 11), [0, 255, 0, 255]);
    assert_eq!(
        backend.cache_stats().image_uploads,
        stable.image_uploads + 1
    );
    assert!(backend.remove_image("asset"));
    clear(&mut canvas);
    backend.render_cached(&ui, &mut canvas).unwrap();
    canvas.present();
    assert_eq!(pixel(&canvas, 20, 11), [10, 20, 30, 255]);
    assert_eq!(
        ui.paint_revision(),
        revision,
        "asset changes invalidate the backend independently"
    );
}
