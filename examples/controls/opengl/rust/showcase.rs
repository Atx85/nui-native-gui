//! Host-owned window and frame loop; the UI borrows the renderer.
use native_ui::{SelectItem, Ui};
#[allow(dead_code)]
mod themes;
// Use themes::WINDOWS_11 or themes::WINDOWS_XP for another shared skin.
const THEME: &str = themes::MACOS;
#[cfg(test)]
#[path = "tests/checks.rs"]
mod checks;
// BEGIN UI
const HTML: &str = r##"<body>
  <section id="controls" class="panel">
    <label class="title">Controls</label>
    <label for="name">Player name</label>
    <input id="name" value="Ada" maxlength="40" placeholder="Your name">
    <input id="echo" value="Ada" readonly>
    <div class="row"><input id="enabled" type="checkbox" checked><label class="row-label" for="enabled">Enable sound</label></div>
    <label>Export formats</label>
    <div class="row">
      <input id="png" name="formats" type="checkbox" value="png" checked><label class="row-label" for="png">PNG</label>
      <input id="svg" name="formats" type="checkbox" value="svg"><label class="row-label" for="svg">SVG</label>
    </div>
    <label for="volume">Volume</label>
    <input id="volume" type="range" min="0" max="100" step="5" value="55">
    <label for="mode">Quality</label>
    <select id="mode"><option value="normal" selected>Normal</option><option value="high">High</option></select>
    <div class="row"><button id="apply" class="action primary">Read values</button><button id="reset" class="action">Reset values</button></div>
    <div class="row"><button id="options" class="action">Replace options</button><button class="action" disabled>Unavailable</button></div>
  </section>
  <section id="preview" class="panel">
    <label class="title">Images, canvas and scrolling</label>
    <div class="row"><img id="picture" src="first" alt="Color tiles"><button id="swap" class="action">Switch image</button></div>
    <canvas id="scene" tabindex="0"></canvas>
    <label>Click the native canvas above.</label>
    <div id="list" tabindex="0">
      <label class="list-item">Row 1</label><label class="list-item">Row 2</label><label class="list-item">Row 3</label><label class="list-item">Row 4</label>
      <label class="list-item">Row 5</label><label class="list-item">Row 6</label><label class="list-item">Row 7</label><label class="list-item">Row 8</label>
    </div>
    <div class="row"><button id="top" class="action">Scroll to top</button><button id="bottom" class="action">Scroll to bottom</button></div>
    <input id="status" value="Ready" readonly>
  </section>
</body>
"##;
const CSS: &str = r##"/* Geometry only; the canonical macOS theme supplies all control appearance. */
body { display: flex; gap: 24px; padding: 24px; }
.panel { padding: 16px; display: flex; flex-direction: column; gap: 8px; overflow: auto; }
#controls { width: 360px; }
#preview { flex-grow: 1; min-width: 280px; }
.title { font-size: 18px; height: 28px; }
.row { display: flex; align-items: center; gap: 8px; height: 32px; }
.row-label { width: 76px; }
.action { flex-grow: 1; }
#picture { width: 80px; height: 32px; object-fit: contain; }
#scene { height: 110px; }
#list { height: 96px; overflow: auto; display: flex; flex-direction: column; }
.list-item { height: 32px; }
"##;
// END UI

// BEGIN ACTIONS
fn update(ui: &mut Ui, alternate: &mut bool) -> Result<(), native_ui::Error> {
    if ui.changed("name") {
        let value = ui.get_value("name").unwrap_or_default().to_owned();
        ui.set_value("echo", &value)?;
    }
    if ui.changed("enabled") {
        ui.set_value(
            "status",
            if ui.checked("enabled") == Some(true) {
                "Sound on"
            } else {
                "Sound off"
            },
        )?;
    }
    if ui.checkbox_group_changed("formats") == Some(true) {
        let count = ui.checked_values("formats").unwrap_or_default().len();
        ui.set_value("status", &format!("{count} formats selected"))?;
    }
    if ui.changed("volume") || ui.changed("mode") || ui.clicked("apply") {
        ui.set_value(
            "status",
            &format!(
                "Quality: {} / volume: {}",
                ui.get_value("mode").unwrap_or_default(),
                ui.get_value("volume").unwrap_or_default()
            ),
        )?;
    }
    // Application setters do not generate user-change events.
    if ui.clicked("reset") {
        ui.set_value("name", "Ada")?;
        ui.set_value("echo", "Ada")?;
        ui.set_checked("enabled", true)?;
        ui.set_value("volume", "55")?;
        ui.set_value("mode", "normal")?;
        ui.set_checked_values("formats", &["png"])?;
        ui.set_value("status", "Values reset")?;
    }
    if ui.clicked("options") {
        let mut options = vec![
            SelectItem::new("normal", "Balanced"),
            SelectItem::new("high", "Best quality"),
            SelectItem::new("draft", "Draft"),
        ];
        options[0].selected = true;
        ui.set_select_options("mode", &options)?;
    }
    if ui.clicked("swap") {
        *alternate = !*alternate;
        ui.set_image_source("picture", if *alternate { "second" } else { "first" })?;
    }
    if ui.clicked("top") {
        ui.set_scroll_offset("list", 0., 0.)?;
    }
    if ui.clicked("bottom") {
        ui.set_scroll_offset("list", 0., 10000.)?;
    }
    if ui
        .canvas_input("scene")
        .is_some_and(|input| input.pressed.is_some())
    {
        ui.set_value("status", "Canvas pressed")?;
    }
    Ok(())
}
// END ACTIONS

use native_ui_opengl::{
    OpenGlBackend, Surface,
    glow::{self, HasContext},
};
use std::rc::Rc;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke = std::env::var("NUI_EXAMPLE_MODE").as_deref() == Ok("smoke");
    let sdl = sdl3::init()?;
    let video = sdl.video()?;
    video
        .gl_attr()
        .set_context_profile(sdl3::video::GLProfile::Core);
    video.gl_attr().set_context_version(3, 3);
    let mut builder = video.window("Controls — Rust / OpenGL", 940, 640);
    builder.opengl().resizable().high_pixel_density();
    if smoke {
        builder.hidden();
    }
    let window = builder.build()?;
    let _context = window.gl_create_context()?;
    let _ = video.gl_set_swap_interval(sdl3::video::SwapInterval::VSync);
    // SAFETY: the host context is current and outlives the renderer.
    let gl = Rc::new(unsafe {
        glow::Context::from_loader_function(|name| {
            video
                .gl_get_proc_address(name)
                .map_or(std::ptr::null(), |p| p as *const _)
        })
    });
    let mut backend = unsafe { OpenGlBackend::new(gl.clone())? };
    backend.register_image_rgba(
        "first",
        2,
        2,
        &[
            255, 90, 90, 255, 70, 150, 255, 255, 80, 210, 150, 255, 255, 210, 80, 255,
        ],
    )?;
    backend.register_image_rgba(
        "second",
        2,
        2,
        &[
            80, 210, 150, 255, 255, 210, 80, 255, 255, 90, 90, 255, 70, 150, 255, 255,
        ],
    )?;
    let mut ui = Ui::from_html_css_with_viewport(HTML, &format!("{THEME}{CSS}"), 940., 640.)?;
    let mut alternate = false;

    let mut events = sdl.event_pump()?;
    'running: for frame in 0.. {
        if smoke && frame == 8 {
            break;
        }
        for event in events.poll_iter() {
            if matches!(event, sdl3::event::Event::Quit { .. }) {
                break 'running;
            }
            native_ui_sdl3::handle_window_event(&mut ui, &event, &window)?;
        }
        let (w, h) = window.size();
        let (pw, ph) = window.size_in_pixels();
        if w == 0 || h == 0 {
            continue;
        }
        ui.set_viewport(w as f32, h as f32)?;
        native_ui_sdl3::sync_text_input(&ui, &window);
        update(&mut ui, &mut alternate)?;
        unsafe {
            gl.viewport(0, 0, pw as i32, ph as i32);
            gl.clear_color(243. / 255., 243. / 255., 243. / 255., 1.);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        // Draw the game here; the adapter restores GL state after the UI.
        backend.render_with_canvases(
            &ui,
            Surface::new([w as f32, h as f32], [pw, ph])?,
            |info, painter| {
                let c = info.region.content;
                painter.fill(
                    native_ui::Rect {
                        x: c.x + 16.,
                        y: c.y + 16.,
                        w: 64.,
                        h: 64.,
                    },
                    native_ui::Color(54, 95, 221, 255),
                );
                Ok(())
            },
        )?;
        window.gl_swap_window();
        ui.end_frame();
    }
    Ok(())
}
