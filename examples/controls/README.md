# One control showcase per renderer and language

Each program contains all eleven feature areas: buttons, text input, checkboxes,
checkbox groups, sliders, selects, labels, images, native canvas, layout, and
scrolling. The application source includes its HTML/CSS, event handling,
application-driven updates, setup, frame loop, and cleanup. All use the canonical
macOS light theme; source comments explain Windows 11 and XP selection.

**Recommended: C and Rust, with host-owned windows and frame loops.**
Go, Zig and Jai are unverified standalone references using older owning connectors.
See [verification and toolchains](../../docs/languages.md) before choosing a language.

| Renderer | C — host-owned | Rust — host-owned |
| --- | --- | --- |
| SDL3 | [showcase.c](sdl3/c/showcase.c) | [showcase.rs](sdl3/rust/showcase.rs) |
| OpenGL | [showcase.c](opengl/c/showcase.c) | [showcase.rs](opengl/rust/showcase.rs) |
| raylib | [showcase.c](raylib/c/showcase.c) | [showcase.rs](raylib/rust/showcase.rs) |

| Unverified standalone references | Go | Zig | Jai |
| --- | --- | --- | --- |
| SDL3 | [source](sdl3/go/main.go) | [source](sdl3/zig/showcase.zig) | [source](sdl3/jai/showcase.jai) |
| OpenGL | [source](opengl/go/main.go) | [source](opengl/zig/showcase.zig) | [source](opengl/jai/showcase.jai) |
| raylib | [source](raylib/go/main.go) | [source](raylib/zig/showcase.zig) | [source](raylib/jai/showcase.jai) |

For short per-control instructions, use [C control recipes](../../docs/controls.md).
The reference covers [HTML/CSS and renderer behavior](../../docs/reference.md).

## Run

C requires host framework CMake packages; set `CMAKE_PREFIX_PATH` if needed.
Rust uses Cargo and builds framework dependencies from source.

```sh
python3 tools/build-sdk.py --out target/showcase-sdk
python3 tools/build-language-example.py c --backend raylib --sdk target/showcase-sdk --run
```

Choose `c` or `rust`, and `sdl3`, `opengl`, or `raylib`. The default
example is `showcase`; `--example all` remains an alias. The old per-control
application names have been replaced by the recipes. From a packaged SDK, use
`python3 build-language.py c --backend raylib --run`.

Rust can also run through Cargo:

```sh
cargo run --bin showcase
cargo run -p native-ui-opengl-examples --bin opengl-showcase
cargo run -p native-ui-raylib-examples --bin raylib-showcase
```

## What to copy

C: copy one `showcase.c`, link the SDK and your host framework (SDL3 or raylib).
The public headers supply the renderer adapters; no private runner is required.
Rust: copy the application and theme module and add the native crates listed in
its Cargo manifest. Framework setup, event routing, drawing and presentation are
visible in each main function. Tests remain in separate files.

Destroy the UI renderer before its host context. The borrowed raylib C header
calls the application's raylib, so host objects never enter the SDK's bundled
instance. The host owns text queue routing. SDL3 and raylib use
`nui_renderer_render` for clipped canvas callbacks, OpenGL uses `nui_gl_render`.
The SDK-owned connectors remain available for standalone applications.

## Tests and maintenance

```sh
python3 tools/build-language-example.py c --backend raylib --sdk target/showcase-sdk --check
python3 tools/build-language-example.py c --backend raylib --sdk target/showcase-sdk --smoke-test
cargo test -p native-ui-examples -p native-ui-opengl-examples -p native-ui-raylib-examples
python3 tools/sync-control-examples.py --check
```

C tests compile a separate harness around the actual application source. Smoke
checks run eight hidden frames and switch the displayed image. Rust, Go, and
Zig have separate state tests; their regular binaries support the smoke mode.
Jai remains source-only and unverified; see its language folder instructions.

Edit `ui.html` and `layout.css` here for the shared markup/layout, and the SDL3
application source for each language's behavior. C/Rust keep independent native
setup/drawing code; their shared action blocks are synchronized. Run
`python3 tools/sync-control-examples.py` to update embedded assets and derive the
OpenGL/raylib versions. The resulting application sources are still self-contained.
The unverified raylib bindings retain their older connector calls. Canonical themes are generated
from `examples/showcase/ui/themes/` by `tools/sync-example-themes.py`.
