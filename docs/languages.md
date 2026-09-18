# Language support and verification

**Start with C or Rust.** Their SDL3, OpenGL and raylib showcases use host-owned
windows and frame loops. Go, Zig and Jai are **unverified reference sources**;
they still use the older SDK-owned standalone connectors and are not the recommended
game-integration pattern. Source availability does not imply tested support.

| Language | Integration | Verification |
| --- | --- | --- |
| C | Host SDL3 renderer, OpenGL context, or raylib | Builds and control tests checked on macOS x86_64; see packaged verification.json for the actual build runner. |
| Rust | Native framework crates; host owns the loop | Builds and control tests checked on macOS x86_64; see packaged verification.json. |
| Go | Older standalone C ABI bindings | Unverified: compiler unavailable in this development session. |
| Zig | Older standalone C ABI bindings | Unverified: compiler unavailable in this development session. |
| Jai | Older standalone C ABI bindings | Experimental and unverified; official compiler required. |

CI defines macOS, Linux and Windows jobs; a configured job is not evidence of a
successful run. SDK packages record completed builds and state checks in
`verification.json`. Graphics smoke tests require a display and are reported
separately from headless checks.

On 2026-09-18, all six C/Rust showcases also passed eight-frame rendering checks
on macOS x86_64. The borrowed raylib C/C++ tests checked pixel output, clipping,
cache invalidation, callback errors, and destroy/recreate while the host stayed open.
The default SDK runtime no longer embeds SDL or raylib. The older window-owning
connectors live in `standalone/`, which must not be loaded alongside a host's SDL
or raylib. This separation prevents duplicate macOS Objective-C class registration.

## Build and run

```sh
python3 tools/build-sdk.py --out target/my-sdk
python3 tools/build-language-example.py c --backend raylib --sdk target/my-sdk --run
cargo run -p native-ui-raylib-examples --bin raylib-showcase
```

The SDK output directory must be new. C examples require your framework's headers
and libraries: SDL3 3.2+ for SDL3/OpenGL, raylib 5.5+ for raylib, plus OpenGL for
the OpenGL example. Install their CMake packages or set `CMAKE_PREFIX_PATH`.
The SDK packager uses the development packages built by its native dependencies;
a downloaded SDK does not include those host framework development packages.

From inside a downloaded SDK:

```sh
python3 build-language.py c --backend raylib --run
python3 build-language.py rust --backend sdl3 --check
```

C needs CMake and a C11 compiler. Rust needs Cargo/Rust 2024 and the native build
toolchain; the SDK includes the Rust crates and Cargo manifests. Cargo builds
SDL3/raylib dependencies from source. Rust applications use native Rust APIs,
so they do not need the SDK shared library at runtime.

`--check` builds and runs separate state tests. `--smoke-test` draws eight hidden
frames. `--example all` is an alias for the single combined showcase. For C,
`--compiler` selects a C compiler; for Rust it selects Cargo.

## Runtime selection

Link `native_ui::native_ui` (or the library at the SDK root) for a host-owned
application. It includes the core, fonts, borrowed SDL/raylib bridge and OpenGL
adapter, with no embedded window runtime. Native Rust examples already link only
their host framework.

Older `nui_sdl_open`, `nui_window_*` and `nui_raylib_open` APIs require the
compatibility library in `standalone/`. CMake users select `native_ui::standalone`;
source builders enable `native-ui-c/owned-windows`. Both libraries implement the
same core ABI, so link exactly one. The default library deliberately does not
export those window-owning entry points. Never load the compatibility library
into an existing SDL/raylib game.

The example builder selects `standalone/` for the unverified Go/Zig reference
bindings and writes their executables there. Jai users must likewise link/deploy
that library. This compatibility choice does not change their unverified status.

## Ownership

C: copy one showcase source, the public SDK headers/library, and link your normal
framework. Rust: copy the source and generated theme module and use the native
crates listed in its Cargo.toml. Neither needs a private run/window wrapper.
All examples default to macOS; comments explain Windows 11/XP alternatives.

The host initializes and closes the framework, handles quit/resize/text routing,
draws its scene, draws the UI, and presents. Destroy UI renderer resources before
the host renderer/context. `nui_end_frame`/`Ui::end_frame` only clears transient UI
flags. Canvas callbacks return zero on success; ordinary C mutations return one.
The borrowed C canvas painter is a temporary token for `nui_renderer_fill`.

SDL events are forwarded without consuming them. Raylib key/mouse snapshots
remain readable by the game; the host explicitly drains and routes the character
queue. Borrowed drawing never clears, presents, sleeps, or owns a frame loop.

## Unverified reference languages

Go sources require Go 1.24+, cgo and a C compiler. Zig sources target Zig 0.16.
Jai needs the official compiler and Bindings_Generator. Their folders and source
headers carry the unverified status. They remain useful porting references, but
have not yet been migrated to the host-owned C/Rust integration pattern.
`--language all` attempts installed Go/Zig compilers during packaging and records
successful builds; missing compilers remain unverified. Jai remains source-only.
The Go/Zig build helper targets macOS/Linux; Windows needs a separate compatible
toolchain. Go pins its OS thread and its bridge must not retain Go pointers.

[Showcase sources](../examples/controls/README.md) · [C control recipes](controls.md)
