//! The existing canvas studio, using its original HTML/CSS themes with raylib.
use native_ui::{CanvasRegion, Ui};
use native_ui_raylib::{BackendError, RaylibBackend, handle_input};
use raylib::prelude::*;
use std::{path::PathBuf, time::Instant};

fn load_ui(theme: &str, width: f32, height: f32) -> Result<Ui, BackendError> {
    let skin = match theme {
        "xp" => include_str!("../../../examples/showcase/ui/themes/windows-xp-luna.css"),
        "macos" => include_str!("../../../examples/showcase/ui/themes/macos-light.css"),
        "win11" => include_str!("../../../examples/showcase/ui/themes/windows-11-fluent.css"),
        _ => return Err("theme must be xp, macos or win11".into()),
    };
    Ok(Ui::from_html_css_with_viewport(
        include_str!("../../../examples/showcase/ui/studio.html"),
        &format!(
            "{skin}\n{}",
            include_str!("../../../examples/showcase/ui/studio.css")
        ),
        width,
        height,
    )?)
}
// The host creates/decodes assets once. The UI only refers to the registered key.
fn register_demo_images(backend: &mut RaylibBackend<'_>) -> Result<(), BackendError> {
    let (width, height) = (128u32, 80u32);
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let xf = x as f32;
            let yf = y as f32;
            // Rounded, transparent tile with two wave traces.
            let dx = (xf - xf.clamp(12.0, 115.0)).abs();
            let dy = (yf - yf.clamp(12.0, 67.0)).abs();
            let alpha = ((12.0 - dx.hypot(dy)).clamp(0.0, 1.0) * 255.0) as u8;
            let mut color = [
                23 + (y / 8) as u8,
                38 + (x / 12) as u8,
                58 + (x / 8) as u8,
                alpha,
            ];
            for (phase, line) in [(0.0, [82, 202, 246]), (1.7, [143, 227, 174])] {
                let wave = 40.0 + 20.0 * (xf / 128.0 * std::f32::consts::TAU + phase).sin();
                if (yf - wave).abs() < 1.6 {
                    color[..3].copy_from_slice(&line);
                }
            }
            let offset = ((y * width + x) * 4) as usize;
            pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }
    backend.register_image_rgba("studio-mark", width, height, &pixels)
}
fn reset(ui: &mut Ui) -> Result<(), BackendError> {
    ui.set_value("pattern", "waves")?;
    ui.set_value("palette", "ocean")?;
    ui.set_value("rate", "1.0")?;
    ui.set_value("amplitude", "0.25")?;
    ui.set_value("detail", "720")?;
    ui.set_checked("grid", true)?;
    ui.set_checked("animate", true)?;
    Ok(())
}

fn line<D: RaylibDraw>(draw: &mut D, from: (f32, f32), to: (f32, f32), color: Color) {
    draw.draw_line_ex(
        Vector2::new(from.0, from.1),
        Vector2::new(to.0, to.1),
        1.0,
        color,
    );
}

// Same scene geometry and controls as the SDL studio, drawn with raylib primitives.
fn draw_scene<D: RaylibDraw>(
    ui: &Ui,
    region: CanvasRegion<'_>,
    draw: &mut D,
    time: f32,
    marker: (f32, f32),
) {
    if region.id != Some("scene") {
        return;
    }
    let area = region.content;
    if ui.checked("grid") == Some(true) {
        let color = Color::new(38, 55, 73, 255);
        for x in (0..area.w as u32).step_by(32) {
            line(
                draw,
                (area.x + x as f32, area.y),
                (area.x + x as f32, area.y + area.h),
                color,
            );
        }
        for y in (0..area.h as u32).step_by(32) {
            line(
                draw,
                (area.x, area.y + y as f32),
                (area.x + area.w, area.y + y as f32),
                color,
            );
        }
    }
    let colors = match ui.get_value("palette") {
        Some("citrus") => [
            Color::new(255, 202, 102, 255),
            Color::new(161, 220, 112, 255),
        ],
        Some("orchid") => [
            Color::new(215, 147, 250, 255),
            Color::new(247, 161, 187, 255),
        ],
        _ => [
            Color::new(82, 202, 246, 255),
            Color::new(143, 227, 174, 255),
        ],
    };
    let amplitude = ui
        .get_value("amplitude")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.25);
    let detail = ui
        .get_value("detail")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(720);
    for (layer, color) in colors.into_iter().enumerate() {
        let mut previous = None;
        for step in 0..=detail {
            let t = step as f32 / detail as f32;
            let phase = layer as f32 * 1.7;
            let (x, y) = match ui.get_value("pattern") {
                Some("orbit") => {
                    let angle = t * std::f32::consts::TAU;
                    let r = amplitude + layer as f32 * 0.10;
                    (
                        0.5 + r * (angle + time).cos(),
                        0.5 + r * (angle + time * 0.65 + phase).sin(),
                    )
                }
                Some("lissajous") => {
                    let angle = t * std::f32::consts::TAU;
                    (
                        0.5 + amplitude * 1.6 * (angle * 3.0 + time + phase).sin(),
                        0.5 + amplitude * 1.4 * (angle * 2.0 + time * 0.4).sin(),
                    )
                }
                _ => (
                    t,
                    0.5 + amplitude * (t * std::f32::consts::TAU * 1.5 + time + phase).sin(),
                ),
            };
            let point = (area.x + x * area.w, area.y + y * area.h);
            if let Some(previous) = previous {
                line(draw, previous, point, color);
            }
            previous = Some(point);
        }
    }
    let (x, y) = (area.x + marker.0 * area.w, area.y + marker.1 * area.h);
    let color = Color::new(255, 200, 105, 255);
    line(draw, (x - 12.0, y), (x + 12.0, y), color);
    line(draw, (x, y - 12.0), (x, y + 12.0), color);
    draw.draw_rectangle_rec(Rectangle::new(x - 2.0, y - 2.0, 4.0, 4.0), color);
}

fn main() -> Result<(), BackendError> {
    // macOS is the default. Pass --theme win11 or --theme xp to switch skins.
    let mut theme = "macos".to_owned();
    let mut smoke = false;
    let mut screenshot = None;
    let mut benchmark = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--theme" => theme = args.next().ok_or("missing theme")?,
            "--smoke-test" => smoke = true,
            "--benchmark" => {
                let frames = args
                    .next()
                    .ok_or("missing benchmark frame count")?
                    .parse::<usize>()?;
                if frames < 30 {
                    return Err("benchmark needs at least 30 frames".into());
                }
                benchmark = Some(frames);
            }
            "--screenshot" => {
                screenshot = Some(PathBuf::from(args.next().ok_or("missing screenshot path")?));
                smoke = true;
            }
            "--help" | "-h" => {
                println!(
                    "cargo run -p native-ui-raylib --example studio -- [--theme macos|xp|win11] [--smoke-test] [--screenshot image.png] [--benchmark frames]"
                );
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {arg}").into()),
        }
    }
    let mut ui = load_ui(&theme, 1000.0, 680.0)?;
    let mut builder = raylib::init();
    builder
        .size(1000, 680)
        .title("Canvas studio — raylib")
        .resizable()
        .highdpi();
    if smoke || benchmark.is_some() {
        builder.hidden();
    }
    let (mut rl, thread) = builder.build();
    rl.set_window_min_size(700, 600);
    rl.set_target_fps(if benchmark.is_some() { 0 } else { 60 });
    rl.set_exit_key(None);
    let mut backend = RaylibBackend::new(&rl, &thread)?;
    register_demo_images(&mut backend)?;
    let mut time = 0.0;
    let mut marker = (0.5, 0.5);
    let mut timings = Vec::new();
    for frame in 0.. {
        if rl.window_should_close() || (smoke && frame == 8) || benchmark == Some(frame) {
            break;
        }
        let frame_start = Instant::now();
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) && !ui.select_is_open() {
            break;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE)
            && ui.canvas_input("scene").is_some_and(|input| input.focused)
        {
            ui.set_checked("animate", !ui.checked("animate").unwrap_or(true))?;
        }
        handle_input(&mut ui, &mut rl)?;
        if ui.clicked("reset") {
            reset(&mut ui)?;
            time = 0.0;
            marker = (0.5, 0.5);
        }
        if let Some(input) = ui.canvas_input("scene") {
            let point = input.released.or(input.pointer.filter(|_| input.captured));
            if let Some((x, y)) = point {
                let area = ui.canvas_region("scene").unwrap().content;
                if area.w > 0.0 && area.h > 0.0 {
                    marker = ((x / area.w).clamp(0.0, 1.0), (y / area.h).clamp(0.0, 1.0));
                }
            }
        }
        let speed = ui
            .get_value("rate")
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .unwrap_or(1.0)
            .clamp(0.1, 4.0);
        if ui.checked("animate") == Some(true) {
            time += rl.get_frame_time() * speed;
        }
        let native_ui::Color(r, g, b, a) = ui
            .background_color()
            .unwrap_or(native_ui::Color(243, 243, 243, 255));
        let render_start = Instant::now();
        let mut canvas_ms = 0.0;
        let render_ms;
        {
            let mut draw = rl.begin_drawing(&thread);
            draw.clear_background(Color::new(r, g, b, a));
            backend.render_with_canvases(&ui, &mut draw, |region, draw| {
                let start = Instant::now();
                draw_scene(&ui, region, draw, if smoke { 0.8 } else { time }, marker);
                canvas_ms += start.elapsed().as_secs_f64() * 1000.0;
                Ok(())
            })?;
            render_ms = render_start.elapsed().as_secs_f64() * 1000.0;
            if frame == 7
                && let Some(path) = &screenshot
            {
                // Read the completed back buffer before EndDrawing swaps it.
                // SAFETY: the host owns this active raylib drawing frame.
                unsafe { raylib::ffi::rlDrawRenderBatchActive() };
                draw.load_image_from_screen(&thread)
                    .export_image(path.to_str().ok_or("invalid screenshot path")?);
                if !path.is_file() {
                    return Err("screenshot export failed".into());
                }
            }
        }
        ui.end_frame();
        if benchmark.is_some() && frame >= 20 {
            timings.push((
                frame_start.elapsed().as_secs_f64() * 1000.0,
                render_ms,
                canvas_ms,
            ));
        }
    }
    if benchmark.is_some() {
        println!("UI cache: {:?}", backend.cache_stats());
        for (label, index) in [("frame", 0), ("UI submission", 1), ("canvas", 2)] {
            let mut values: Vec<_> = timings
                .iter()
                .map(|&(frame, render, canvas)| match index {
                    0 => frame,
                    1 => render - canvas,
                    _ => canvas,
                })
                .collect();
            values.sort_by(f64::total_cmp);
            println!(
                "{label}: median {:.3} ms, p95 {:.3} ms",
                values[values.len() / 2],
                values[values.len() * 95 / 100]
            );
        }
    }
    if smoke {
        println!("raylib studio ({theme}): 8 frames rendered successfully");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_themes_layout_and_controls_work_in_the_raylib_host() {
        for theme in ["macos", "xp", "win11"] {
            let mut ui = load_ui(theme, 1000.0, 680.0).unwrap();
            for (width, height) in [(1000.0, 680.0), (1280.0, 720.0), (700.0, 600.0)] {
                ui.set_viewport(width, height).unwrap();
                let scene = ui.canvas_region("scene").unwrap();
                let options = ui.bounds("options").unwrap();
                assert_eq!(options.x + options.w, width - 24.0);
                assert_eq!(scene.bounds.x + scene.bounds.w + 20.0, options.x);
                assert!(scene.content.w > 0.0 && scene.content.h > 0.0);
            }
            ui.set_value("pattern", "lissajous").unwrap();
            ui.set_value("rate", "3.5").unwrap();
            ui.set_checked("grid", false).unwrap();
            ui.set_scroll_offset("options", 0.0, 10000.0).unwrap();
            let rect = ui.bounds("reset").unwrap();
            ui.pointer_down(rect.x + 5.0, rect.y + 5.0);
            ui.pointer_up(rect.x + 5.0, rect.y + 5.0);
            assert!(ui.clicked("reset"));
            reset(&mut ui).unwrap();
            assert_eq!(ui.get_value("pattern"), Some("waves"));
            assert_eq!(ui.get_value("rate"), Some("1"));
            assert_eq!(ui.checked("grid"), Some(true));
        }
    }
}
