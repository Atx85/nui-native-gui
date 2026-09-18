use native_ui::Ui;
use native_ui_sdl3::SdlBackend;
use sdl3::{
    pixels::{Color, PixelFormat},
    surface::Surface,
};

#[test]
fn rounded_borders_match_physical_pixel_coverage_without_fill_seams() {
    let ui=Ui::from_html_css("<label></label>",
        "label{position:absolute;left:10.25px;top:10.5px;width:20px;height:20px;border:3px solid #ff0000;border-radius:10px;background:#0000ff}").unwrap();
    for scale in [1., 1.5, 2., 3.] {
        let mut canvas = Surface::new(120, 120, PixelFormat::RGBA32)
            .unwrap()
            .into_canvas()
            .unwrap();
        canvas.set_scale(scale, scale).unwrap();
        let creator = canvas.texture_creator();
        let mut backend = SdlBackend::new(&creator).unwrap();
        for cached in [false, true] {
            canvas.set_draw_color(Color::BLACK);
            canvas.clear();
            if cached {
                backend.render_cached(&ui, &mut canvas).unwrap();
            } else {
                backend.render(&ui, &mut canvas).unwrap();
            }
            canvas.present();
            let surface = canvas
                .surface()
                .convert_format(PixelFormat::RGBA32)
                .unwrap();
            let pitch = surface.pitch() as usize;
            let pixels = surface.with_lock(|p| p.to_vec());
            let mut softened = 0;
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
                    let expected = [(red * 255 / 256) as u8, 0, (blue * 255 / 256) as u8];
                    let got = &pixels[y * pitch + x * 4..][..3];
                    assert!(
                        got.iter().zip(expected).all(|(a, b)| a.abs_diff(b) <= 5),
                        "scale {scale}, cached {cached}, ({x},{y}): got {got:?}, expected {expected:?}"
                    );
                    if got[0] > 0 && got[0] < 250 && got[2] == 0 {
                        softened += 1;
                    }
                }
            }
            assert!(
                softened > 15,
                "curves need coverage pixels at scale {scale}"
            );
        }
    }
}

#[test]
fn checkbox_stroke_is_antialiased_at_the_actual_scale() {
    let ui=Ui::from_html_css("<input type=checkbox checked>",
        "input{position:absolute;left:10px;top:10px;width:20px;height:20px;background:transparent;border:none;padding:0px;color:#ffffff}").unwrap();
    let mut previous = 0;
    for scale in [1., 2., 3.] {
        let mut canvas = Surface::new(120, 120, PixelFormat::RGBA32)
            .unwrap()
            .into_canvas()
            .unwrap();
        canvas.set_scale(scale, scale).unwrap();
        let creator = canvas.texture_creator();
        let mut backend = SdlBackend::new(&creator).unwrap();
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        backend.render(&ui, &mut canvas).unwrap();
        canvas.present();
        let image = canvas
            .surface()
            .convert_format(PixelFormat::RGBA32)
            .unwrap();
        let edge =
            image.with_lock(|p| p.chunks_exact(4).filter(|p| p[0] > 0 && p[0] < 255).count());
        assert!(
            edge > previous,
            "higher DPI should add actual stroke edge samples"
        );
        previous = edge;
    }
}
