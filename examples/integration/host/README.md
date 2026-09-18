# Optional standalone window wrappers

For recommended host-owned game integration, start with the
[C/Rust showcases](../../controls/README.md). This older convenience example
creates its own window and manages clearing/presentation. Its CMake target
explicitly links `native_ui::standalone`, the compatibility runtime in
`standalone/`. Use that library alone, never alongside a host SDL/raylib.

Start with [main.c](main.c). It includes a host adapter, creates its UI once, and
keeps the application loop visible. No `support.h`, test runner, shared theme or
update callback is required. `native_ui_themes.h` supplies the shared macOS
skin; the comment at `nui_create` shows how to switch to Windows 11 or XP.

The two integration calls inside that loop are:

```c
int event = ui_host_poll(&host); // -1: error, 0: continue, 1: quit
// After a continue result, read controls and update your application here.
if (nui_clicked(ui, "hello") == 1)
    puts("Hello from my application!");
int painted = ui_host_draw(&host, background, NULL, NULL); // 1: success, 0: error
```

The complete example checks these results and cleans up on failure. `poll` handles
window events and viewport resizing. `draw` clears, renders, presents and ends the
UI frame. Optional canvas drawing and user data can replace its two `NULL` arguments.
If rendering fails, it still finishes the opened frame and clears transient input.

Copy **one** of [sdl3.h](sdl3.h), [opengl.h](opengl.h) or [raylib.h](raylib.h)
into your application and include it alongside `native_ui.h`. Each uses the same
`UiHost`, `ui_host_open`, `ui_host_poll`, `ui_host_draw` and `ui_host_close` interface.
The headers are small source helpers, not a new binary API. Their implementations
are checked as standalone SDK consumers.

The caller owns `NuiUi`; the adapter borrows it and owns the window. Check
`host.window` after opening, close the host before destroying the UI, and keep all
calls on the main thread. Keep image registration and other backend-specific calls
on `host.window`. Read `nui_last_error()` when an operation fails.

The optional [frame_delay.h](frame_delay.h) keeps this demo from spinning. Copy it
too when copying `main.c`, or replace `ui_host_wait()` with your application's
frame scheduler. There is no CSS or test infrastructure in the adapter.

This helper creates a **bundled window** and owns clearing/presentation. To embed
UI into an existing SDL3 game, use the [borrowed renderer adapter](../sdl3/README.md).
For raylib use [the borrowed C showcase](../../controls/raylib/c/showcase.c).
For other engines, use `nui_render` with your renderer callbacks,
or `nui_gl_create` / `nui_gl_render` for an existing OpenGL context. See
[OpenGL integration](../opengl/README.md) for that route.

## Build

From a downloaded SDK, or this repository with a built SDK:

```sh
cmake -S examples/integration/host -B build-host \
  -Dnative_ui_DIR=/absolute/path/to/sdk/cmake -DUI_BACKEND=sdl3
cmake --build build-host --config Release
```

Set `UI_BACKEND` to `opengl` or `raylib` to build the same application with that
adapter. Run `build-host/hello-host` on macOS/Linux or
`build-host/Release/hello-host.exe` with Visual Studio. The build copies the runtime
library beside the executable. Click the button to print a message.

The original [linking example](../linking/README.md) shows the full lower-level
window API without this helper, along with direct compiler and deployment details.
