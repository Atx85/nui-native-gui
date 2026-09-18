# UI inside an existing SDL3 game

The game owns SDL initialization, its window and renderer, all events, simulation,
clearing, presentation and shutdown. Native UI borrows the renderer and manages
only its own font/image textures. Creating or destroying the adapter does not
pause the game. The same `NuiUi` can survive adapter destruction and recreation.

Include `native_ui_sdl3.h` and link the SDK plus **your game's SDL3** (3.2 or newer).
No SDL_ttf, font setup, drawing callbacks or replacement game loop is required.

The example includes `native_ui_themes.h` and uses `NUI_THEME_MACOS`.
The comment beside `nui_create` shows how to select `NUI_THEME_WINDOWS_11` or
`NUI_THEME_WINDOWS_XP`. Only the page background is made transparent so the
game remains visible; the controls retain the selected theme's appearance.

```c
NuiUi *ui = nui_create(html, css, logical_width, logical_height);
NuiSdlRenderer *overlay = nui_sdl_create(renderer);

/* Inside your existing event loop. Return value is success, NOT consumption. */
if (!nui_sdl_event(overlay, ui, &event)) /* handle nui_last_error() */;
game_handle_event(&event);

/* Inside your existing frame. */
game_update(dt);
if (nui_clicked(ui, "debug") == 1) toggle_debug();
game_draw(renderer);
if (!nui_sdl_draw(overlay, ui)) /* handle nui_last_error() */;
nui_end_frame(ui);
SDL_RenderPresent(renderer);

/* When the temporary UI is no longer needed. */
nui_sdl_destroy(overlay);
nui_destroy(ui);
```

Check all operation results in real code; [main.c](main.c) is a complete example.
The moving square continues behind the UI. Click the button to change its color;
F1 destroys/recreates the adapter while the SDL renderer and game stay alive.

`nui_sdl_event` forwards a supplied, unmodified event. It returns **1 on success,
including ignored events, or 0 on error**, not whether the UI consumed it. It never
polls the queue, handles quit, closes windows, or prevents the game receiving input.
Only input for this renderer's window is forwarded. Pass original SDL window-space
events, before your game converts their coordinates; the helper accounts for the
current viewport, scale and logical presentation. Set the renderer's intended UI
coordinate system before forwarding events and drawing.

Call `nui_set_viewport` when your logical UI dimensions change. If using SDL's fixed
logical presentation as in the example, those dimensions remain fixed on resize.
`nui_sdl_draw` preserves the host target, viewport, scale, logical presentation,
clip, blend mode and draw color, including on drawing errors. It does not clear,
present, or reset UI click/change flags. Call `nui_end_frame` after reading them,
even on frames where you choose not to draw the UI.

For text fields, the game retains control of `SDL_StartTextInput`/`SDL_StopTextInput`.
Combine `nui_wants_text_input(ui)` with the game's own text-input needs. Text events,
editing/navigation keys and Ctrl/Cmd+A are forwarded; clipboard shortcuts and IME
composition display remain host integrations. The adapter does not change the
cursor, grab the mouse, toggle relative mouse mode, or install SDL event hooks.

Destroy the adapter **before** its renderer, on the main thread. Destroyed handles
cannot be reused; set them to NULL if storing them after cleanup. When hiding a UI
while preserving its state, cancel its pending input with `nui_pointer(ui, NUI_CANCEL,
0, 0)`. A null adapter can safely be passed to `nui_sdl_destroy`.

Fonts use the same bundled rasterizer as the existing SDL backend. Text textures
are cached (up to 256 entries / 64 MiB); paint commands are submitted each draw.
Image registration is available through `nui_sdl_renderer_register_image` and
`nui_sdl_renderer_remove_image`. Forward renderer-reset events or call
`nui_sdl_invalidate` to recreate cached textures; registered image pixels survive.
Canvas elements draw their UI decoration; draw native game content yourself before
the overlay. The simple `nui_sdl_draw` call omits canvas callbacks. Use
`nui_renderer_render(adapter, ui, callback, data)` and `nui_renderer_fill` for
clipped native canvas content, as shown in the [C showcase](../../controls/sdl3/c/showcase.c).

## Build

```sh
cmake -S examples/integration/sdl3 -B build-overlay \
  -Dnative_ui_DIR=/absolute/path/to/sdk/cmake \
  -DSDL3_DIR=/absolute/path/to/your/SDL3/cmake
cmake --build build-overlay --config Release
```

Run `build-overlay/game-ui` (or `build-overlay/Release/game-ui.exe`).
`--smoke-test` runs eight hidden frames. The build copies the Native UI runtime;
deploy your game's SDL runtime with your usual SDL packaging process.

## How the borrowed boundary works

The small inline bridge in `native_ui_sdl3.h` calls the SDL instance linked by your
application. Its function table is copied into the SDK adapter, which calls back
through it to upload, draw and release textures. Host SDL pointers never go to
a second SDL runtime. Link the SDK root library (`native_ui::native_ui`), which
contains no SDL or raylib. The optional `standalone/` library is for older
window-owning applications and must not be loaded into this host-owned game.

Other language bindings can wrap this header in a compiled C shim; they do not
need to implement fonts or drawing. Keep the shim loaded until all adapters using
it are destroyed. The bridge ABI is versioned independently from the UI ABI.
