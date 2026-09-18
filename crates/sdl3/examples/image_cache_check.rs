//! GPU pixel check: cargo run -p native-ui-sdl3 --example image_cache_check
//! Uses a hidden native window; normal tests exercise the software fallback.
use native_ui::{CanvasRegion, Ui};
use native_ui_sdl3::{BackendError, SdlBackend};
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::{Canvas, FRect},
    video::Window,
};

fn pixels(canvas: &mut Canvas<Window>) -> Result<Vec<u8>, BackendError> {
    let scale = canvas.scale();
    let viewport = canvas.viewport();
    canvas.set_scale(1.0, 1.0)?;
    canvas.set_viewport(None);
    let surface = canvas
        .read_pixels(None)?
        .convert_format(PixelFormat::RGBA32)?;
    canvas.set_scale(scale.0, scale.1)?;
    canvas.set_viewport(viewport);
    Ok(surface.with_lock(|bytes| bytes.to_vec()))
}
fn clear(canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(20, 30, 40));
    canvas.clear();
}
fn scene(
    region: CanvasRegion<'_>,
    canvas: &mut Canvas<Window>,
    frame: u8,
) -> Result<(), BackendError> {
    canvas.set_draw_color(if region.id == Some("first") {
        Color::RGB(0, frame, 0)
    } else {
        Color::RGB(0, 0, frame)
    });
    let r = region.content;
    canvas.fill_rect(FRect::new(r.x, r.y, r.w, r.h))?;
    Ok(())
}
// Switch to windows-11-fluent.css or windows-xp-luna.css to use another skin.
const THEME: &str = include_str!("../../../examples/showcase/ui/themes/macos-light.css");

fn main() -> Result<(), BackendError> {
    let sdl = sdl3::init()?;
    let video = sdl.video()?;
    let window = video
        .window("Image cache validation", 320, 240)
        .hidden()
        .build()?;
    let mut canvas = window.into_canvas();
    println!("Renderer: {}", canvas.renderer_name);
    let creator = canvas.texture_creator();
    let mut direct = SdlBackend::new(&creator)?;
    let mut cached = SdlBackend::new(&creator)?;
    let mut rgba = Vec::new();
    for _ in 0..20 {
        for color in [
            [255, 0, 0, 128],
            [0, 255, 0, 192],
            [0, 0, 255, 128],
            [255, 255, 255, 0],
        ] {
            rgba.extend(color.repeat(10));
        }
    }
    for b in [&mut direct, &mut cached] {
        b.register_image_rgba("photo", 40, 20, &rgba)?;
    }
    let ui = Ui::from_html_css(
        "<canvas id=first></canvas><img src=photo id=cover><canvas id=second></canvas><img src=photo id=contain><img src=photo id=fill>",
        &format!(
            "{THEME}{}",
            "canvas,img{position:absolute;left:10px;top:10px;width:70px;height:70px;border:none;background:transparent}#cover{left:20px;top:20px;object-fit:cover}#second{left:60px;top:50px}#contain{left:70px;top:60px;object-fit:contain}#fill{left:100px;top:10px;width:40px;height:40px}"
        ),
    )?;
    for scale in [1.0, 1.5, 2.0] {
        canvas.set_scale(scale, scale)?;
        canvas.set_viewport(Rect::new(2, 3, 150, 115));
        canvas.set_clip_rect(Rect::new(5, 5, 132, 110));
        for frame in [60, 170, 240] {
            clear(&mut canvas);
            direct.render_with_canvases(&ui, &mut canvas, |r, c| scene(r, c, frame))?;
            let expected = pixels(&mut canvas)?;
            clear(&mut canvas);
            cached.render_cached_with_canvases(&ui, &mut canvas, |r, c| scene(r, c, frame))?;
            let actual = pixels(&mut canvas)?;
            assert_eq!(expected.len(), actual.len());
            let worst = expected
                .iter()
                .zip(&actual)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            assert!(
                worst <= 4,
                "cached/direct image error {worst} at scale {scale}, frame {frame}"
            );
        }
    }
    assert_eq!(cached.cache_stats().rebuilds, 3);
    assert_eq!(cached.cache_stats().reuses, 6);
    assert_eq!(cached.cache_stats().image_uploads, 1);
    assert!(
        cached.cache_stats().texture_layers > 0,
        "this check requires GPU texture caching"
    );
    cached.clear_caches();
    clear(&mut canvas);
    cached.render_cached_with_canvases(&ui, &mut canvas, |r, c| scene(r, c, 240))?;
    assert_eq!(cached.cache_stats().image_uploads, 2);
    println!(
        "Image alpha, contain/cover/fill, host clipping, canvas order and warm GPU cache passed at 1x/1.5x/2x. {:?}",
        cached.cache_stats()
    );
    let mut scrolling = Ui::from_html_css_with_viewport(
        "<aside id=p><canvas id=first></canvas><img src=photo><input type=range value=35></aside>",
        &format!(
            "{THEME}{}",
            "aside{width:110px;height:90px;overflow:auto}canvas{width:150px;height:100px}img{width:150px;height:40px}input{height:30px}"
        ),
        150.0,
        115.0,
    )?;
    let before = cached.cache_stats();
    for scale in [1.0, 1.5, 2.0] {
        canvas.set_scale(scale, scale)?;
        for offset in [0.0, 0.0, 60.0, 60.0, 100.0, 100.0] {
            scrolling.set_scroll_offset("p", 20.0, offset)?;
            clear(&mut canvas);
            direct.render_with_canvases(&scrolling, &mut canvas, |r, c| scene(r, c, 170))?;
            let expected = pixels(&mut canvas)?;
            clear(&mut canvas);
            cached.render_cached_with_canvases(&scrolling, &mut canvas, |r, c| scene(r, c, 170))?;
            let actual = pixels(&mut canvas)?;
            let worst = expected
                .iter()
                .zip(&actual)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            assert!(
                worst <= 4,
                "scrolled GPU cache error {worst}, scale {scale}, offset {offset}"
            );
        }
    }
    assert_eq!(cached.cache_stats().rebuilds - before.rebuilds, 9);
    assert_eq!(cached.cache_stats().reuses - before.reuses, 9);
    assert_eq!(cached.cache_stats().image_uploads, before.image_uploads);
    println!(
        "Scrolling, native canvas clips, styled sliders and scrollbar GPU caching passed at 1x/1.5x/2x."
    );
    Ok(())
}
