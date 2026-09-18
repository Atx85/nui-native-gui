# raylib connector

`native-ui-raylib` connects the same HTML/CSS UI core to raylib 6.0. It provides
text and primitive painting, registered RGBA images, clipped native canvas
callbacks, and polled mouse/keyboard/Unicode input with clipboard shortcuts.
It uses the same bundled Noto Sans font and text metrics as SDL3.

Run the example:

```sh
cargo run -p native-ui-raylib --example controls
```

For the existing Canvas Studio with the shared macOS theme:

```sh
cargo run -p native-ui-raylib --example studio -- --theme macos
```

This reuses `examples/showcase/ui/studio.html`, `studio.css`, and
`themes/macos-light.css`, including the original image, responsive sidebar,
patterns, palettes, sliders and canvas interaction. It also accepts `--theme xp`
and `--theme win11`. Capture a deterministic frame with
`--screenshot target/raylib-studio-macos.png`; this renders eight hidden frames
and exits. `--smoke-test` does the same without saving an image.
Measure uncapped frame times with `--benchmark 120`. It reports frame, UI and
canvas timings after 20 warmup frames, plus paint-cache counters.

The demo includes a button, text field, checkbox, slider, select, image and animated
native canvas. Cargo builds raylib from source, so a system raylib installation is
optional for Rust applications. Build prerequisites are Rust, CMake, a C/C++
compiler and libclang. On macOS, install Xcode command line tools and CMake.
For native C/C++ projects, `brew install raylib` installs raylib's headers and library.
See the [raylib Rust binding documentation](https://docs.rs/raylib/6.0.0/raylib/).

## Host integration

Add `native-ui`, `native-ui-raylib` and `raylib = "6.0.0"` to your application's
dependencies, using local paths for the two native-ui crates.

```rust,no_run
use native_ui::Ui;
use native_ui_raylib::{BackendError, RaylibBackend, handle_input};
use raylib::prelude::*;

fn main() -> Result<(), BackendError> {
    let (mut rl, thread) = raylib::init()
        .size(640, 480).title("My app").resizable().highdpi().build();
    rl.set_target_fps(60);
    rl.set_exit_key(None); // Escape belongs to the UI, including select popups.
    let mut backend = RaylibBackend::new(&rl, &thread)?;
    let mut ui = Ui::from_html_css_with_viewport(
        "<button id=save>Save</button>",
        "button { position:absolute; left:20px; top:20px; width:140px; height:40px; }",
        640.0, 480.0,
    )?;
    while !rl.window_should_close() {
        handle_input(&mut ui, &mut rl)?;
        if ui.clicked("save") {
            // Save application state here.
        }
        {
            let mut draw = rl.begin_drawing(&thread);
            draw.clear_background(Color::RAYWHITE);
            backend.render(&ui, &mut draw)?;
        }
        ui.end_frame();
    }
    Ok(())
}
```

Create the backend after the raylib window and drop it before closing that window.
All operations run on raylib's owning thread. The host owns the window, frame
lifecycle and quit behavior. Call `handle_input` once each frame; it updates the UI
viewport, consumes raylib's character queue and leaves key/mouse polling available.
Coordinates use logical window pixels. Focus loss cancels active UI input.

Call `render` in an ordinary screen-space drawing frame, outside camera, render
texture, shader, blend and scissor modes. For a game overlay, finish the game's
camera/3D scope before rendering the UI. This adapter does not capture or restore
arbitrary host graphics state. Text textures, UI paint commands and text metrics
are cached. UI changes, viewport/DPI changes and image replacements invalidate
the retained commands. Canvas callbacks still run every frame in paint order,
so animation does not repeatedly rasterize the surrounding CSS controls.
`cache_stats()` reports command rebuilds, reuse and text measurements;
`render_uncached_with_canvases` provides a reference path for diagnostics.

`register_image_rgba(src, width, height, pixels)` uploads an asset once, keyed by
the HTML image's exact `src`. Pixels are tightly packed straight-alpha RGBA8.
Replacing/removing assets takes effect on the next render. Missing assets use the
UI's alt fallback. No files or URLs are fetched implicitly.

Use `render_with_canvases` to draw with raylib inside a `<canvas>`. Its callback
receives `CanvasRegion` and a clipped raylib drawing handle. Use `region.content`
as the drawing origin. Do not clear/present, modify scissor state, or destroy the
window inside the callback. Balanced camera/shader/blend scopes are supported.
The clip is released on return, including callback errors.

For C games, include `native_ui_raylib.h` and link your host's raylib. Create the
adapter after `InitWindow` with `nui_raylib_create`, forward snapshots with
`nui_raylib_input`, route `GetCharPressed` yourself through `nui_raylib_text`, and
draw with `nui_raylib_draw` or `nui_renderer_render` for native canvas callbacks.
Destroy with `nui_raylib_destroy` before `CloseWindow`. The adapter never starts
or ends a frame, changes the exit key, or sets a frame rate. It calls the host's
raylib through callbacks; the default SDK contains no raylib or SDL runtime.

Rust offers `handle_input_snapshot` when the game owns character-queue routing;
`handle_input` remains a convenience that also drains that queue.

The older `nui_raylib_open/poll/clear/render/present/close` connector remains for
standalone applications via the separate `standalone/` compatibility library
(`native_ui::standalone`, or Cargo feature `owned-windows`). Link exactly one
runtime. Go/Zig/Jai references still use that compatibility API and are unverified.
See [the C and Rust showcases](../../examples/controls/raylib/README.md).

## Checks

```sh
cargo test -p native-ui-raylib --features desktop-tests --test render
cargo test -p native-ui-raylib --features desktop-tests --test render -- --highdpi
cargo run -p native-ui-raylib --example controls -- --smoke-test
```

These commands use hidden windows and need a working graphics session. The first
checks pixels, including image fitting, Unicode text, canvas clipping and cleanup
after callback errors. It runs on the main thread for macOS/GLFW compatibility.
The second renders eight desktop frames. Pixel tests are opt-in so ordinary
workspace tests still work without a display. Raylib's experimental software
renderer is not supported by this connector.
