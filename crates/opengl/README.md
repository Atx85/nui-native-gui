# OpenGL adapter

Draw Native UI and your own graphics into the **same framebuffer** with desktop
OpenGL 3.3 or newer. The adapter handles UI text, images, CSS decoration, canvas
viewports, scrolling clips and high-DPI scaling. It neither creates a window nor
owns your event loop. No canvas readback or intermediate scene texture is required.

Run the complete Rust example:

```sh
cargo run -p native-ui-opengl --example triangle
cargo run -p native-ui-opengl --example triangle -- --theme macos
```

It creates a host-owned OpenGL context using SDL, draws a rotating triangle using
normal shaders, and places working UI controls over the canvas. It loads the
existing Windows 11 skin by default; `--theme xp|macos|win11` selects our shared
skins. The C triangle example accepts the same option and reads the canonical
CSS files from the repository or the SDK’s `themes` folder. SDL is only the
window/input provider in this example; it does not render the UI or triangle.

## Existing Rust application

Dependencies: `native-ui`, `native-ui-opengl`, and your window/context library.
The adapter re-exports its `glow` version so you can use one set of GL bindings.

```rust,ignore
use native_ui_opengl::{OpenGlBackend, Surface, glow};
use std::rc::Rc;

// Your window library provides get_proc_address. Its GL context is already current.
let gl = Rc::new(unsafe { glow::Context::from_loader_function(get_proc_address) });
let mut renderer = unsafe { OpenGlBackend::new(gl.clone())? };

// Each frame, after updating input and any layout size:
ui.set_viewport(window_width, window_height)?;
let surface = Surface::new(
    [window_width, window_height],       // logical UI units
    [framebuffer_width, framebuffer_height], // physical pixels
)?;
renderer.render_cached_with_canvases(&ui, surface, |canvas, painter| {
    if canvas.region.id == Some("scene") {
        let [width, height] = canvas.pixel_size();
        camera.set_aspect_ratio(width as f32 / height as f32);
        draw_scene(painter.gl(), &camera); // your ordinary OpenGL calls
    }
    Ok(())
})?;
// Your app swaps/presents once, after the entire UI has been painted.
ui.end_frame();
```

Create GPU resources once, then reuse them in the callback. `examples/triangle.rs`
includes shader compilation, resource cleanup, event handling and the actual
`draw_arrays` call; the sketch above omits application-specific setup.

`CanvasInfo.region.content` contains the full logical drawing rectangle, excluding
CSS border/padding. `pixel_size()` gives its physical pixel dimensions.
`viewport` is that full rectangle converted to bottom-left GL pixels; `scissor`
is the visible portion after ancestor overflow and framebuffer clipping.
Scrolling clips the scene without changing its camera aspect ratio. Fully clipped
canvases receive no callback. A zero physical surface size skips the whole frame.
On resize, pass fresh logical and physical dimensions; do not multiply pointer
coordinates by the display scale. SDL hosts can reuse
`native_ui_sdl3::handle_window_event` for logical mouse, keyboard, Unicode and
clipboard input. Other hosts route events through the core `Ui` input API.

## C and other languages

The same adapter is exported through `native_ui.h` in the default host-owned SDK runtime:

```c
// Existing host context, already current. load_gl is your function loader.
NuiGl *renderer = nui_gl_create(load_gl, app);
// Per frame: logical dimensions match the UI; pixels match the bound framebuffer.
nui_gl_render(renderer, ui, width, height, pixel_width, pixel_height,
              draw_canvas, app);
// Before destroying the context:
nui_gl_destroy(renderer);
```

For applications starting from scratch, the optional window helper is simpler:

```c
NuiWindow *window = nui_window_open("My app", 900, 600, 0, NUI_BACKEND_OPENGL);
// Poll, clear, render, present using nui_window_*.
```

The `examples/controls/opengl/` collection contains one complete control showcase
per language in C, Rust, Go, Zig and Jai. They select OpenGL directly:

```sh
python3 tools/build-language-example.py c --backend opengl --example showcase --run
python3 tools/build-language-example.py rust --backend opengl --example showcase --run
cargo run -p native-ui-opengl-examples --bin opengl-showcase
```

The resulting `opengl-` executables also use OpenGL when launched directly.
Jai remains compiler-unverified.
The C raw-shader example is `examples/integration/opengl/triangle.c` (repository/SDK root).
A callback receives a borrowed painter, not an SDL renderer. `nui_draw_fill` is a
portable convenience; raw OpenGL calls work inside OpenGL callbacks. Load their
addresses before rendering with `nui_window_gl_proc_address`. Query the full canvas
pixel dimensions with `nui_draw_canvas_pixel_size`. Callbacks return 0 on success;
ordinary API operations return 1. Existing `nui_sdl_*` APIs remain available.

## Context and state contract

The context must stay alive and current on its owning thread while the adapter
is created, used, and dropped. The host owns framebuffer attachments, including
any depth buffer. Render into a complete framebuffer using one color attachment.
Finish transform-feedback and conditional-render scopes before calling the UI.
The adapter does not switch contexts, create depth attachments or present frames.

Canvas callbacks run in document order; later controls and select popups cover
them. On entry, viewport/scissor are set, scissor is enabled, color writes are
on, and depth/stencil/culling/blending are off. Depth writes are off. Bind your
own shader and VAO; enable depth testing **and depth writes** when needed for 3D.
Do not disable or enlarge scissor, change contexts, present, delete adapter-owned
objects, or modify their VAO/program contents. A GL clear respects the scissor,
but enabling depth writes is required before clearing depth. Rounded CSS corners
do not mask native drawing; canvas clipping is rectangular.

Rendering and callbacks restore these common state values, including on errors
and Rust unwinding: program, VAO, array buffer, draw/read framebuffer bindings,
viewport, scissor, active texture unit, unit-0 2D texture/sampler, blending factors
and equations, blend/cull/depth/stencil/scissor/rasterizer-discard/primitive-restart/
color-logic enables, polygon mode, color/depth write masks, depth function, clear
color, and pixel-unpack buffer/alignment/row-length/skips. Callbacks must restore
other state they change themselves (for example other texture units, stencil
functions, cull direction, depth range or clear depth). Host sRGB policy is retained.
This is a documented state boundary, not a snapshot of every possible GL extension.

RGBA images are registered explicitly with `register_image_rgba` / `nui_gl_register_image`;
no asset paths or URLs are opened implicitly. Text uses bundled Noto Sans and a
cache cleared between frames when it exceeds 256 entries. Compatible paint quads
are batched; canvases flush pending UI draws before native drawing.

## Retained UI cache

Use `render_cached` or `render_cached_with_canvases` to reuse unchanged UI layers.
The Rust triangle demo and C/window helpers use this path, so all five language
example sets receive caching automatically. Direct `render` / `render_with_canvases`
remain available for callers who do not want a retained paint cache.

UI commands, text measurements and antialiased edge coverage are generated only
when the UI paint revision or surface dimensions change. Image replacement/removal
also invalidates the cache. A change rebuilds all UI layers, as in the SDL adapter.
Native canvas callbacks run on every frame, interleaved with the cached layers in
original paint order; native scene pixels never enter the UI cache.

Static layers use premultiplied RGBA textures with a 64 MiB total budget. Layers
that exceed the budget or cannot use a framebuffer texture retain commands instead.
When the host enables `FRAMEBUFFER_SRGB`, all layers retain commands to preserve
its color conversion behavior. Command fallback still avoids regenerating UI paint
and measurements, but redraws its primitives. `set_cache_texture_limit(0)` explicitly
selects command-only caching. No CPU pixel readback occurs in either path.

`cache_stats()` reports rebuilds, reuses, generated commands, text measurements,
image uploads, current texture bytes, and texture/command layer counts.
`clear_caches()` releases derived UI/text resources and preserves registered images;
use it only with the original context current. Recreate the backend after context
loss. `clear_text_cache()` also invalidates paint layers.

C consumers can inspect `nui_gl_cache_stats` / `nui_window_cache_stats` and clear an
existing-context adapter with `nui_gl_clear_caches`. The `NuiCacheStats` ABI is unchanged.

OpenGL ES, Vulkan and WebGL are not supported by this adapter.

## Verification

```sh
cargo test -p native-ui-opengl --features desktop-tests --test render
cargo run -p native-ui-opengl --example triangle -- --smoke-test
```

The pixel test needs a desktop GL session and checks text, registered images,
canvas padding/borders/ancestor clipping, overlay order, HiDPI conversion,
minimized surfaces and state restoration after callbacks and callback errors.
Cache checks compare direct/cached pixels at 1×, 1.5×, 2× and 3×, exercise live
canvases under transparent UI, popup and value changes, image replacement, budget
fallback, sRGB, and confirm unchanged frames generate no paint or measurements.
