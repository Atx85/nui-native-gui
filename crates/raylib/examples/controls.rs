//! cargo run -p native-ui-raylib --example controls
use native_ui::Ui;
use native_ui_raylib::{BackendError, RaylibBackend, handle_input};
use raylib::prelude::*;

// Use windows-11-fluent.css or windows-xp-luna.css here to switch the shared skin.
const THEME: &str = include_str!("../../../examples/showcase/ui/themes/macos-light.css");

fn main() -> Result<(), BackendError> {
    let smoke = std::env::args().any(|arg| arg == "--smoke-test");
    let mut builder = raylib::init();
    builder
        .size(640, 440)
        .title("Native UI — raylib")
        .resizable()
        .highdpi();
    if smoke {
        builder.hidden();
    }
    let (mut rl, thread) = builder.build();
    rl.set_target_fps(60);
    rl.set_exit_key(None);
    let mut backend = RaylibBackend::new(&rl, &thread)?;
    backend.register_image_rgba(
        "badge",
        2,
        2,
        &[
            55, 110, 230, 255, 90, 200, 170, 255, 90, 200, 170, 255, 55, 110, 230, 255,
        ],
    )?;
    let mut ui = Ui::from_html_css_with_viewport(
        r#"<label id="heading">Native UI · raylib</label>
        <input id="name" value="Hello, café!">
        <button id="hello" class="primary">Say hello</button>
        <input id="animate" type="checkbox" checked><label id="caption" for="animate">Animate canvas</label>
        <input id="speed" type="range" min="0" max="100" value="50">
        <select id="color"><option value="blue">Blue</option><option value="green">Green</option></select>
        <img id="badge" src="badge" alt="Badge"><canvas id="scene"></canvas>
        <label id="status">One UI core, another native renderer.</label>"#,
        &format!(
            "{THEME}{}",
            r#"label, button, input, select, canvas, img { position: absolute; left: 28px; }
        #heading { top: 22px; width: 540px; height: 34px; font-size: 25px; }
        #name { top: 76px; width: 350px; }
        #hello { left: 398px; top: 76px; width: 214px; }
        #animate { top: 134px; }
        #caption { left: 62px; top: 132px; width: 220px; height: 28px; }
        #speed { left: 290px; top: 132px; width: 130px; }
        #color { left: 446px; top: 126px; width: 166px; }
        #badge { left: 546px; top: 192px; width: 50px; height: 50px; }
        #scene { top: 188px; width: 496px; height: 170px; padding: 12px; }
        #status { top: 386px; width: 580px; height: 28px; }"#
        ),
        640.0,
        440.0,
    )?;
    let mut phase = 0.0_f32;
    for frame in 0.. {
        if rl.window_should_close() || (smoke && frame >= 8) {
            break;
        }
        handle_input(&mut ui, &mut rl)?;
        if ui.clicked("hello") {
            ui.set_value(
                "status",
                &format!("Hello, {}", ui.get_value("name").unwrap_or_default()),
            )?;
        }
        if ui.checked("animate").unwrap_or(false) {
            let speed = ui
                .get_value("speed")
                .unwrap_or("50")
                .parse::<f32>()
                .unwrap_or(50.0);
            phase += rl.get_frame_time() * speed / 20.0;
        }
        let color = if ui.get_value("color") == Some("green") {
            Color::SEAGREEN
        } else {
            Color::ROYALBLUE
        };
        {
            let mut draw = rl.begin_drawing(&thread);
            draw.clear_background(Color::RAYWHITE);
            backend.render_with_canvases(&ui, &mut draw, |region, draw| {
                let r = region.content;
                draw.draw_circle_v(
                    Vector2::new(r.x + r.w * (0.5 + 0.4 * phase.sin()), r.y + r.h / 2.0),
                    24.0,
                    color,
                );
                Ok(())
            })?;
        }
        ui.end_frame();
    }
    if smoke {
        println!("raylib connector: 8 frames rendered successfully");
    }
    Ok(())
}
