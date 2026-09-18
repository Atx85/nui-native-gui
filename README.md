# Native UI SDK

The default library contains the UI engine, fonts and borrowed renderer adapters.
It embeds no SDL or raylib runtime: graphics calls use the application's framework.
The optional standalone/ library preserves the older window-owning helpers.
Link exactly one runtime: native_ui::native_ui (default) or native_ui::standalone
(compatibility). Do not load the standalone runtime into an existing SDL/raylib game.
Build your application using its language compiler, native_ui.h and this library.
C showcases link the host SDL3/raylib development package (set CMAKE_PREFIX_PATH if needed).
Rust showcases use Cargo and the native Rust crates included under crates/.
The core C ABI alone still needs only native_ui.h and the shared library.
Examples default to the canonical macOS theme. native_ui_themes.h contains
NUI_THEME_MACOS, NUI_THEME_WINDOWS_11 and NUI_THEME_WINDOWS_XP for C/C++.
Example source comments explain how to select an alternative in each language.

Start with the control showcase:

    python3 build-language.py c --example showcase --run

Choose c, rust, go or zig. Each program demonstrates all controls and their updates.
For example: `python3 build-language.py go --example showcase --run`.
Add `--backend opengl` or `--backend raylib` to choose the renderer.
Read docs/controls.md for per-control C recipes and examples/controls/README.md for the source index.
OpenGL canvas integration: OPENGL.md.
Existing SDL3 games: include native_ui_sdl3.h and see examples/integration/sdl3.
Existing raylib games: include native_ui_raylib.h and see examples/controls/raylib/c/showcase.c.
This optional adapter borrows your renderer and uses your game's SDL3 library.
C and Rust showcases borrow host renderers. Go, Zig and Jai are unverified
standalone reference sources using older SDK-owned connectors; see verification.json.
See LANGUAGES.md for toolchains and Jai binding generation.

    python3 build-language.py rust --example all --check
    python3 build-language.py c --example showcase --smoke-test

The first command checks state/input without a window. The second requires a
working graphics session and checks hidden native frames. A built executable
opens its example window by default. Keep it beside the native library.

C ABI version: 2. See native_ui.h for ownership, errors and thread rules.
The bundle targets the packaging machine's OS/architecture. Packaging supports
macOS, Linux and Windows (MSVC). NOTICES.txt contains licenses.

Include the library in a standalone C or C++ project with CMake:

    cmake -S examples/integration/linking -B build -Dnative_ui_DIR=/absolute/path/to/this/sdk/cmake
    cmake --build build --config Release
    ctest --test-dir build -C Release --output-on-failure

Run build/hello-ui on macOS/Linux or build/Release/hello-ui.exe with Visual Studio.
See examples/integration/linking/README.md for direct compiler commands and deployment details.
For explicit host-owned loops start with examples/controls/README.md.
examples/integration/host contains older standalone convenience wrappers.
On Windows use an x64 Developer PowerShell for Visual Studio to run build-language.py.
The Windows helper builds C and Rust examples; Go and Zig need their own
compatible Windows integration toolchains. The DLL keeps its native_ui_c.dll name;
native_ui.lib is its import library, not a second runtime or a static library.
