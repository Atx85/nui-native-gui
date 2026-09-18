use native_ui::{Rect as UiRect, Ui};
use native_ui_sdl3::SdlBackend;
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::{BlendMode, ClippingRect},
    surface::{Surface, SurfaceRef},
};
fn pixel(surface: &SurfaceRef, x: usize, y: usize) -> [u8; 4] {
    let converted = surface.convert_format(PixelFormat::RGBA32).unwrap();
    let pitch = converted.pitch() as usize;
    converted.with_lock(|bytes| {
        bytes[y * pitch + x * 4..y * pitch + x * 4 + 4]
            .try_into()
            .unwrap()
    })
}
#[test]
fn software_rendering_honors_css_padding_colors_clipping_and_host_state() {
    let mut canvas = Surface::new(300, 160, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap();
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    let ui=Ui::from_html_css("<button id=save>Save &amp; café</button>",
        "button{position:absolute;left:20px;top:30px;width:140px;height:60px;padding:12px 15px;background:#4080c0;color:#ff0000}").unwrap();
    canvas.set_draw_color(Color::RGB(20, 24, 34));
    canvas.clear();
    canvas.set_clip_rect(Rect::new(0, 0, 200, 140));
    canvas.set_draw_color(Color::RGB(7, 8, 9));
    canvas.set_blend_mode(BlendMode::None);
    let color = canvas.draw_color();
    let clip = canvas.clip_rect();
    let scale = canvas.scale();
    let viewport = canvas.viewport();
    backend.render(&ui, &mut canvas).unwrap();
    canvas.present();
    assert_eq!(canvas.draw_color(), color);
    assert_eq!(canvas.clip_rect(), clip);
    assert_eq!(canvas.blend_mode(), BlendMode::None);
    assert_eq!(canvas.scale(), scale);
    assert_eq!(canvas.viewport(), viewport);
    assert_eq!(pixel(canvas.surface(), 25, 35), [64, 128, 192, 255]);
    assert_eq!(pixel(canvas.surface(), 200, 150), [20, 24, 34, 255]);
    let surface = canvas
        .surface()
        .convert_format(PixelFormat::RGBA32)
        .unwrap();
    let pitch = surface.pitch() as usize;
    let red_pixels = surface.with_lock(|bytes| {
        let mut count = 0;
        for y in 0..160 {
            for x in 0..300 {
                let p = &bytes[y * pitch + x * 4..][..4];
                if p[0] > 150 && p[1] < 80 && p[2] < 100 {
                    count += 1;
                    assert!((36..144).contains(&x));
                    assert!((43..77).contains(&y));
                }
            }
        }
        count
    });
    assert!(red_pixels > 30, "font should render actual colored glyphs");
}
#[test]
fn transparent_background_reveals_host_and_empty_clip_stays_empty() {
    let mut canvas = Surface::new(160, 100, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap();
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    let ui=Ui::from_html_css("<button></button>","button{position:absolute;left:10px;top:10px;width:100px;height:50px;background:transparent}").unwrap();
    canvas.set_draw_color(Color::RGB(200, 100, 40));
    canvas.clear();
    backend.render(&ui, &mut canvas).unwrap();
    canvas.present();
    assert_eq!(pixel(canvas.surface(), 30, 30), [200, 100, 40, 255]);
    canvas.set_clip_rect(ClippingRect::Zero);
    backend.render(&ui, &mut canvas).unwrap();
    assert_eq!(canvas.clip_rect(), ClippingRect::Zero);
}
#[test]
fn output_scales_and_demo_can_be_rendered_without_a_window() {
    let html = include_str!("../../../examples/showcase/ui/main.html");
    let layout = include_str!("../../../examples/showcase/ui/layout.css");
    let xp = include_str!("../../../examples/showcase/ui/themes/windows-xp-luna.css");
    let macos = include_str!("../../../examples/showcase/ui/themes/macos-light.css");
    let win11 = include_str!("../../../examples/showcase/ui/themes/windows-11-fluent.css");
    let baseline = Ui::from_html_css(html, &format!("{xp}\n{layout}")).unwrap();
    for (name, theme, arrangement, background) in [
        (
            "xp",
            xp,
            include_str!("../../../examples/showcase/ui/layouts/windows-xp-luna.css"),
            [236, 233, 216, 255],
        ),
        (
            "macos",
            macos,
            include_str!("../../../examples/showcase/ui/layouts/macos-light.css"),
            [243, 243, 243, 255],
        ),
        (
            "win11",
            win11,
            include_str!("../../../examples/showcase/ui/layouts/windows-11-fluent.css"),
            [243, 243, 243, 255],
        ),
    ] {
        let ui = Ui::from_html_css(html, &format!("{theme}\n{layout}\n{arrangement}")).unwrap();
        for id in [
            "name-label",
            "name",
            "step-label",
            "step",
            "counting",
            "counting-label",
            "demo",
            "reset",
        ] {
            assert!(
                ui.contains_id(id),
                "theme {name} must preserve the existing {id} control"
            );
            if name == "xp" {
                assert_eq!(ui.bounds(id), baseline.bounds(id));
            }
        }
        for scale in [1.0, 2.0] {
            let mut canvas = Surface::new(
                (640.0 * scale) as u32,
                (400.0 * scale) as u32,
                PixelFormat::RGBA32,
            )
            .unwrap()
            .into_canvas()
            .unwrap();
            let creator = canvas.texture_creator();
            let mut backend = SdlBackend::new(&creator).unwrap();
            let native_ui::Color(r, g, b, a) = ui.background_color().unwrap();
            canvas.set_draw_color(Color::RGBA(r, g, b, a));
            canvas.clear();
            canvas.set_scale(scale, scale).unwrap();
            backend.render(&ui, &mut canvas).unwrap();
            canvas.present();
            assert_eq!(
                pixel(
                    canvas.surface(),
                    (20.0 * scale) as usize,
                    (20.0 * scale) as usize
                ),
                background
            );
            let rect = ui.bounds("demo").unwrap();
            let top = pixel(
                canvas.surface(),
                ((rect.x + 8.0) * scale) as usize,
                ((rect.y + 8.0) * scale) as usize,
            );
            let bottom = pixel(
                canvas.surface(),
                ((rect.x + 8.0) * scale) as usize,
                ((rect.y + rect.h - 8.0) * scale) as usize,
            );
            if name == "xp" {
                assert!(
                    top[0] > bottom[0] && top[2] > bottom[2],
                    "XP buttons retain their bevel shading"
                );
            } else if name == "macos" {
                assert_eq!(top, [8, 107, 223, 255]);
                assert_eq!(bottom, top, "Mac primary button has a flat fill");
                assert_eq!(ui.bounds("demo").unwrap().h, 30.0);
                assert_eq!(ui.bounds("counting").unwrap().w, 16.0);
                assert_eq!(
                    pixel(
                        canvas.surface(),
                        (497.0 * scale) as usize,
                        (167.0 * scale) as usize
                    ),
                    [8, 122, 255, 255],
                    "Mac dropdown has a compact accent indicator"
                );
            } else {
                assert_eq!(top, [0, 95, 184, 255]);
                assert_eq!(bottom, top);
                assert_eq!(ui.bounds("name").unwrap().h, 36.0);
                assert_eq!(ui.bounds("counting").unwrap().w, 20.0);
            }
            if scale == 2.0
                && let Ok(path) = std::env::var("NATIVE_UI_TEST_IMAGE")
            {
                let path = if name == "macos" {
                    std::path::Path::new(&path).with_file_name("native-ui-macos-light.bmp")
                } else if name == "win11" {
                    std::path::Path::new(&path).with_file_name("native-ui-windows-11.bmp")
                } else {
                    path.into()
                };
                canvas.surface().save_bmp(path).unwrap();
            }
        }
    }
}
#[test]
fn font_errors_propagate_and_failed_paint_restores_state() {
    let mut canvas = Surface::new(200, 100, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap();
    let creator = canvas.texture_creator();
    assert!(SdlBackend::with_font(&creator, b"bad font").is_err());
    let mut backend = SdlBackend::new(&creator).unwrap();
    let mut ui = Ui::new();
    ui.add_button_at(
        "huge",
        &"W".repeat(2000),
        UiRect {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 40.0,
        },
    )
    .unwrap();
    canvas.set_draw_color(Color::RGB(1, 2, 3));
    canvas.set_clip_rect(Rect::new(0, 0, 10, 10));
    canvas.set_blend_mode(BlendMode::None);
    let clip = canvas.clip_rect();
    assert!(backend.render(&ui, &mut canvas).is_err());
    assert_eq!(canvas.draw_color(), Color::RGB(1, 2, 3));
    assert_eq!(canvas.clip_rect(), clip);
    assert_eq!(canvas.blend_mode(), BlendMode::None);
}

#[test]
fn sdl_paints_css_state_colors_exactly_and_restores_the_base() {
    let mut canvas = Surface::new(160, 100, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap();
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    let mut ui=Ui::from_html_css("<button id=save></button>","button{position:absolute;left:10px;top:10px;width:100px;height:50px;background:#123456}button:hover{background:#abcdef}button:active{background:#654321}").unwrap();
    for (step, expected) in [
        [18, 52, 86, 255],
        [171, 205, 239, 255],
        [101, 67, 33, 255],
        [171, 205, 239, 255],
        [18, 52, 86, 255],
    ]
    .into_iter()
    .enumerate()
    {
        match step {
            1 => ui.pointer_move(20.0, 20.0),
            2 => ui.pointer_down(20.0, 20.0),
            3 => ui.pointer_up(20.0, 20.0),
            4 => ui.pointer_leave(),
            _ => {}
        }
        backend.render(&ui, &mut canvas).unwrap();
        canvas.present();
        assert_eq!(pixel(canvas.surface(), 30, 30), expected);
    }
}

#[test]
fn form_controls_render_edits_checked_state_and_popup_overlay() {
    let mut canvas = Surface::new(300, 240, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap();
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    let mut ui = Ui::from_html_css(
        "<input id=text placeholder=Name><input id=check type=checkbox checked><select id=choice><option>First</option><option>Second</option></select><button id=back>Behind</button>",
        "input,select,button{position:absolute;left:10px;top:10px;width:200px;height:32px;padding:4px;background:white;color:black}#check{top:50px;width:30px;height:30px}#check:checked{background:red}#choice{top:90px;background:blue;color:white}#back{top:154px;background:green}",
    ).unwrap();
    ui.set_value("text", &"é🙂abc ".repeat(1000)).unwrap();
    ui.key_down(native_ui::Key::Tab, false);
    backend.render(&ui, &mut canvas).unwrap();
    canvas.present();
    assert_eq!(pixel(canvas.surface(), 12, 52), [255, 0, 0, 255]);
    ui.set_checked("check", false).unwrap();
    ui.pointer_down(20.0, 100.0);
    ui.pointer_up(20.0, 100.0);
    backend.render(&ui, &mut canvas).unwrap();
    canvas.present();
    assert_eq!(pixel(canvas.surface(), 12, 52), [255, 255, 255, 255]);
    assert_eq!(
        pixel(canvas.surface(), 12, 160),
        [0, 0, 255, 255],
        "popup must paint over the green button"
    );
    ui.key_down(native_ui::Key::End, false);
    ui.key_down(native_ui::Key::Enter, false);
    assert_eq!(ui.get_value("choice"), Some("Second"));
}

#[test]
fn decorations_preserve_transparency_host_clip_and_state_at_both_scales() {
    for scale in [1.0, 2.0] {
        let mut canvas = Surface::new(
            (180.0 * scale) as u32,
            (100.0 * scale) as u32,
            PixelFormat::RGBA32,
        )
        .unwrap()
        .into_canvas()
        .unwrap();
        let creator = canvas.texture_creator();
        let mut backend = SdlBackend::new(&creator).unwrap();
        let ui=Ui::from_html_css("<button></button>","button{position:absolute;left:10px;top:10px;width:120px;height:60px;border:2px solid blue;border-radius:10px;background:linear-gradient(to right,rgba(255,0,0,0.5),rgba(0,0,255,0.5));box-shadow:inset 0 0 0 2px white;outline:1px dotted black;outline-offset:2px}").unwrap();
        canvas.set_scale(scale, scale).unwrap();
        canvas.set_draw_color(Color::RGB(0, 200, 0));
        canvas.clear();
        canvas.set_clip_rect(Rect::new(0, 0, 100, 100));
        canvas.set_blend_mode(BlendMode::None);
        let clip = canvas.clip_rect();
        backend.render(&ui, &mut canvas).unwrap();
        canvas.present();
        let at =
            |x: f32, y: f32| pixel(canvas.surface(), (x * scale) as usize, (y * scale) as usize);
        assert_eq!(
            at(10.0, 10.0),
            [0, 200, 0, 255],
            "rounded corners must reveal the host"
        );
        assert_eq!(at(60.0, 10.0), [0, 0, 255, 255]);
        assert_eq!(at(60.0, 12.0), [255, 255, 255, 255]);
        let blended = at(60.0, 40.0);
        assert!(
            (95..=105).contains(&blended[1]),
            "gradient alpha must composite once over the host: {blended:?}"
        );
        assert_eq!(
            at(120.0, 40.0),
            [0, 200, 0, 255],
            "all decoration obeys the host clip"
        );
        assert_eq!(canvas.clip_rect(), clip);
        assert_eq!(canvas.blend_mode(), BlendMode::None);
        assert_eq!(canvas.scale(), (scale, scale));
        assert_eq!(canvas.draw_color(), Color::RGB(0, 200, 0));
    }
}

#[test]
fn luna_interaction_states_change_decoration_without_changing_geometry() {
    let mut ui = Ui::from_html_css(
        include_str!("../../../examples/showcase/ui/main.html"),
        concat!(
            include_str!("../../../examples/showcase/ui/layout.css"),
            "\n",
            include_str!("../../../examples/showcase/ui/themes/windows-xp-luna.css")
        ),
    )
    .unwrap();
    let mut canvas = Surface::new(640, 400, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap();
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    let button = ui.bounds("demo").unwrap();
    assert_eq!(
        button,
        UiRect {
            x: 132.0,
            y: 294.0,
            w: 176.0,
            h: 56.0
        }
    );
    assert_eq!(
        ui.bounds("name"),
        Some(UiRect {
            x: 132.0,
            y: 58.0,
            w: 376.0,
            h: 44.0
        })
    );
    assert_eq!(
        ui.bounds("step"),
        Some(UiRect {
            x: 132.0,
            y: 150.0,
            w: 376.0,
            h: 44.0
        })
    );
    assert_eq!(
        ui.bounds("counting"),
        Some(UiRect {
            x: 132.0,
            y: 228.0,
            w: 26.0,
            h: 26.0
        })
    );
    for (state, expected) in [
        (0, [255, 255, 255, 255]),
        (1, [246, 198, 90, 255]),
        (2, [185, 181, 167, 255]),
        (3, [165, 197, 237, 255]),
    ] {
        match state {
            1 => ui.pointer_move(button.x + 10.0, button.y + 10.0),
            2 => ui.pointer_down(button.x + 10.0, button.y + 10.0),
            3 => {
                ui.pointer_up(button.x + 10.0, button.y + 10.0);
                ui.pointer_leave();
            }
            _ => {}
        }
        canvas.set_draw_color(Color::RGB(236, 233, 216));
        canvas.clear();
        backend.render(&ui, &mut canvas).unwrap();
        canvas.present();
        assert_eq!(
            pixel(canvas.surface(), 200, 295),
            expected,
            "wrong highlight in state {state}"
        );
        assert_eq!(ui.bounds("demo"), Some(button));
        if let Ok(path) = std::env::var("NATIVE_UI_TEST_IMAGE") {
            let path = std::path::Path::new(&path)
                .with_file_name(format!("native-ui-xp-state-{state}.bmp"));
            canvas.surface().save_bmp(path).unwrap();
        }
    }
    ui.pointer_down(145.0, 165.0);
    ui.pointer_up(145.0, 165.0);
    ui.key_down(native_ui::Key::Down, false);
    backend.render(&ui, &mut canvas).unwrap();
    canvas.present();
    assert_eq!(
        pixel(canvas.surface(), 140, 240),
        [49, 106, 197, 255],
        "highlighted option uses CSS blue"
    );
    assert_eq!(
        ui.get_value("step"),
        Some("1"),
        "previewing an option must not commit it"
    );
    if let Ok(path) = std::env::var("NATIVE_UI_TEST_IMAGE") {
        canvas
            .surface()
            .save_bmp(std::path::Path::new(&path).with_file_name("native-ui-xp-select.bmp"))
            .unwrap();
    }
}

#[test]
fn modern_theme_focus_states_and_comparison_render() {
    let html = include_str!("../../../examples/showcase/ui/main.html");
    let layout = include_str!("../../../examples/showcase/ui/layout.css");
    let themes = [
        (
            include_str!("../../../examples/showcase/ui/themes/windows-xp-luna.css"),
            include_str!("../../../examples/showcase/ui/layouts/windows-xp-luna.css"),
        ),
        (
            include_str!("../../../examples/showcase/ui/themes/macos-light.css"),
            include_str!("../../../examples/showcase/ui/layouts/macos-light.css"),
        ),
        (
            include_str!("../../../examples/showcase/ui/themes/windows-11-fluent.css"),
            include_str!("../../../examples/showcase/ui/layouts/windows-11-fluent.css"),
        ),
    ];
    let mut comparison = Surface::new(1920, 400, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap();
    let creator = comparison.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    for (index, (theme, arrangement)) in themes.iter().enumerate() {
        let mut ui = Ui::from_html_css(html, &format!("{theme}\n{layout}\n{arrangement}")).unwrap();
        comparison.set_viewport(Rect::new(index as i32 * 640, 0, 640, 400));
        let native_ui::Color(r, g, b, a) = ui.background_color().unwrap();
        comparison.set_draw_color(Color::RGBA(r, g, b, a));
        comparison.fill_rect(Rect::new(0, 0, 640, 400)).unwrap();
        // Show focus on the same existing field to expose the different focus treatments.
        ui.key_down(native_ui::Key::Tab, false);
        backend.render(&ui, &mut comparison).unwrap();
    }
    comparison.present();
    assert_eq!(
        pixel(comparison.surface(), 640 + 150, 84),
        [165, 202, 243, 255],
        "Mac focus ring surrounds the field"
    );
    assert_eq!(
        pixel(comparison.surface(), 1280 + 150, 106),
        [0, 95, 184, 255],
        "Windows focus accent is on the lower edge"
    );
    assert_eq!(
        pixel(comparison.surface(), 1280 + 150, 70),
        [243, 243, 243, 255],
        "Windows field has no outer focus halo"
    );
    if let Ok(path) = std::env::var("NATIVE_UI_TEST_IMAGE") {
        comparison
            .surface()
            .save_bmp(std::path::Path::new(&path).with_file_name("native-ui-theme-comparison.bmp"))
            .unwrap();
    }
}

#[test]
fn native_canvas_draws_directly_with_clipping_scale_and_state_restoration() {
    let ui = Ui::from_html_css(
        "<canvas id=scene></canvas><button>Overlay</button><canvas id=hidden></canvas>",
        "canvas{position:absolute;left:10px;top:10px;width:80px;height:60px;padding:4px;border:2px solid red;background:white}button{position:absolute;left:30px;top:30px;width:20px;height:20px;border:none;background:blue;color:blue}#hidden{left:100px}",
    ).unwrap();
    for scale in [1.0, 2.0] {
        let mut canvas = Surface::new(
            (160.0 * scale) as u32,
            (100.0 * scale) as u32,
            PixelFormat::RGBA32,
        )
        .unwrap()
        .into_canvas()
        .unwrap();
        let creator = canvas.texture_creator();
        let mut backend = SdlBackend::new(&creator).unwrap();
        canvas.set_draw_color(Color::RGB(11, 12, 13));
        canvas.clear();
        canvas.set_scale(scale, scale).unwrap();
        canvas.set_clip_rect(Rect::new(0, 0, 70, 60));
        canvas.set_blend_mode(BlendMode::None);
        let viewport = canvas.viewport();
        let clip = canvas.clip_rect();
        let logical = canvas.logical_size();
        let mut calls = 0;
        backend
            .render_with_canvases(&ui, &mut canvas, |region, native| {
                calls += 1;
                assert_eq!(region.id, Some("scene"));
                assert_eq!(
                    region.content,
                    native_ui::Rect {
                        x: 16.0,
                        y: 16.0,
                        w: 68.0,
                        h: 48.0
                    }
                );
                assert_eq!(native.scale(), (scale, scale));
                assert_eq!(
                    native.clip_rect(),
                    ClippingRect::Some(Rect::new(16, 16, 54, 44))
                );
                native.set_draw_color(Color::RGB(0, 200, 0));
                native.fill_rect(sdl3::render::FRect::new(-100.0, -100.0, 500.0, 500.0))?;
                // The following UI commands must recover even when native code alters state.
                native.set_clip_rect(None);
                native.set_scale(3.0, 4.0)?;
                native.set_viewport(Rect::new(2, 3, 40, 40));
                native.set_blend_mode(BlendMode::Add);
                native.set_draw_color(Color::RGB(200, 0, 200));
                Ok(())
            })
            .unwrap();
        assert_eq!(calls, 1, "fully clipped canvases do not dispatch");
        assert_eq!(canvas.viewport(), viewport);
        assert_eq!(canvas.clip_rect(), clip);
        assert_eq!(canvas.scale(), (scale, scale));
        assert_eq!(canvas.logical_size(), logical);
        assert_eq!(canvas.blend_mode(), BlendMode::None);
        assert_eq!(canvas.draw_color(), Color::RGB(11, 12, 13));
        assert!(
            !unsafe { sdl3::sys::render::SDL_RenderViewportSet(canvas.raw()) },
            "automatic viewport remains automatic"
        );
        canvas.present();
        let at =
            |x: f32, y: f32| pixel(canvas.surface(), (x * scale) as usize, (y * scale) as usize);
        assert_eq!(
            at(11.0, 11.0),
            [255, 0, 0, 255],
            "border survives native drawing"
        );
        assert_eq!(
            at(13.0, 13.0),
            [255, 255, 255, 255],
            "padding survives native drawing"
        );
        assert_eq!(at(20.0, 20.0), [0, 200, 0, 255]);
        assert_eq!(
            at(35.0, 35.0),
            [0, 0, 255, 255],
            "later UI overlays native drawing"
        );
        assert_eq!(at(75.0, 20.0), [11, 12, 13, 255], "host clip still applies");
    }
}

#[test]
fn native_canvas_error_restores_host_and_popup_paints_above_native_content() {
    let mut ui=Ui::from_html_css("<canvas id=scene></canvas><select><option>One</option><option>Two</option></select>","canvas{position:absolute;left:10px;top:20px;width:100px;height:70px}select{position:absolute;left:10px;top:0;width:100px;height:20px}option{background:blue;color:blue;border:none}").unwrap();
    let mut canvas = Surface::new(200, 150, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap();
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    canvas.set_viewport(Rect::new(5, 7, 160, 120));
    canvas.set_draw_color(Color::RGB(10, 20, 30));
    canvas.set_blend_mode(BlendMode::None);
    let viewport = canvas.viewport();
    let logical = canvas.logical_size();
    let error = backend
        .render_with_canvases(&ui, &mut canvas, |_, native| {
            native.set_scale(2.0, 2.0)?;
            native.set_clip_rect(Rect::new(1, 2, 3, 4));
            native.set_viewport(None);
            native.set_logical_size(
                100,
                75,
                sdl3::sys::render::SDL_LOGICAL_PRESENTATION_LETTERBOX,
            )?;
            native.set_draw_color(Color::RGB(200, 0, 0));
            native.set_blend_mode(BlendMode::Add);
            Err("native callback error".into())
        })
        .unwrap_err();
    assert_eq!(error.to_string(), "native callback error");
    assert_eq!(canvas.viewport(), viewport);
    assert_eq!(canvas.scale(), (1.0, 1.0));
    assert_eq!(canvas.clip_rect(), ClippingRect::None);
    assert_eq!(canvas.logical_size(), logical);
    assert_eq!(canvas.draw_color(), Color::RGB(10, 20, 30));
    assert_eq!(canvas.blend_mode(), BlendMode::None);
    ui.key_down(native_ui::Key::Tab, false);
    ui.key_down(native_ui::Key::Enter, false);
    backend
        .render_with_canvases(&ui, &mut canvas, |_, native| {
            native.set_draw_color(Color::RGB(0, 200, 0));
            native.fill_rect(Rect::new(10, 20, 100, 70))?;
            Ok(())
        })
        .unwrap();
    canvas.present();
    assert_eq!(
        pixel(canvas.surface(), 5 + 20, 7 + 30),
        [0, 0, 255, 255],
        "popup overlays callback in a translated host viewport"
    );
    assert_eq!(pixel(canvas.surface(), 5 + 20, 7 + 75), [0, 200, 0, 255]);
    canvas.set_clip_rect(ClippingRect::Zero);
    backend
        .render_with_canvases(&ui, &mut canvas, |_, _| {
            panic!("empty clip must skip callback")
        })
        .unwrap();
}

#[test]
fn fractional_scrollbar_thumb_has_no_scanline_gaps() {
    let ui=Ui::from_html_css_with_viewport("<aside><label>long</label></aside>","aside{width:100px;height:97px;overflow:auto}label{height:173px}::scrollbar{width:12px;background:white}::scrollbar-thumb{background:#808080;border-radius:6px;border:3px solid white}",150.0,120.0).unwrap();
    let mut canvas = Surface::new(300, 240, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap();
    canvas.set_scale(2.0, 2.0).unwrap();
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    backend.render(&ui, &mut canvas).unwrap();
    canvas.present();
    for y in 16..98 {
        assert_eq!(
            pixel(canvas.surface(), 188, y),
            [128, 128, 128, 255],
            "gap at {y}"
        );
    }
}
