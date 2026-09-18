# Include the compiled library in an application

This folder is a complete C application. It uses `native_ui.h` and the downloaded
library; no Rust installation, Cargo build or SDL SDK is needed. C++ applications
use the same header and linking target.

This older example uses the **standalone/** compatibility runtime because it
opens its own window. Existing games should use the borrowed showcases with
`native_ui::native_ui`; never link both runtimes.

The example also includes `native_ui_themes.h` and uses `NUI_THEME_MACOS`.
The comment beside `nui_create` shows how to switch to `NUI_THEME_WINDOWS_11`
or `NUI_THEME_WINDOWS_XP`; those skins are compiled in, so no CSS file is needed
at runtime.

## CMake (macOS, Linux and Windows)

Install CMake 3.20+ and a C/C++ compiler. Download and extract the SDK for your OS
and architecture. From the SDK folder:

```sh
cmake -S examples/integration/linking -B build -Dnative_ui_DIR=/absolute/path/to/sdk/cmake
cmake --build build --config Release
ctest --test-dir build -C Release --output-on-failure
```

Use your actual SDK path. On Windows, use an x64 Visual Studio toolchain and a path
such as `-Dnative_ui_DIR=C:/libraries/native-ui/cmake`.

Run `build/hello-ui` on macOS/Linux, or `build/Release/hello-ui.exe` when using the
Visual Studio generator. Click the button to print a message in the terminal.
`--check` validates linking and button interaction without opening a window.

For your own application, these are the important lines:

```cmake
find_package(native_ui CONFIG REQUIRED COMPONENTS standalone)
add_executable(my-app main.c)
target_link_libraries(my-app PRIVATE native_ui::standalone)
```

And in `main.c`:

```c
#include "native_ui.h"
```

Use the runtime-copy command and relative library search path from
[CMakeLists.txt](CMakeLists.txt) when shipping your application. The sample copies
the library beside the executable automatically.

## Direct compiler commands

These commands run from the SDK folder and put the executable beside the
compatibility library in `standalone/`.

macOS (select `arm64` or `x86_64` to match the downloaded SDK):

```sh
cc -arch arm64 -std=c11 examples/integration/linking/main.c -I. -Lstandalone -lnative_ui \
  -Wl,-rpath,@loader_path -o standalone/hello-ui
./standalone/hello-ui
```

Linux x86_64:

```sh
cc -std=c11 examples/integration/linking/main.c -I. -Lstandalone -lnative_ui \
  '-Wl,-rpath,$ORIGIN' -o standalone/hello-ui
./standalone/hello-ui
```

Windows x86_64, in an x64 Developer PowerShell for Visual Studio:

```powershell
cl /nologo /utf-8 /std:c11 examples/integration/linking/main.c /I. /Fe:standalone/hello-ui.exe /link /LIBPATH:standalone native_ui.lib
.\standalone\hello-ui.exe
```

## What to distribute

| Platform | Link against | Ship beside the executable |
| --- | --- | --- |
| macOS | `libnative_ui.dylib` | `libnative_ui.dylib` |
| Linux | `libnative_ui.so` | `libnative_ui.so` |
| Windows (MSVC) | `native_ui.lib` | `native_ui_c.dll` |

`native_ui.lib` is a Windows **import library**: it tells the linker how to call
functions in the DLL. It is not a static copy of the UI engine. Keep the DLL's
original name because that name is recorded in the import library.

For this example, ship the library from `standalone/`, not the default library
at the SDK root. The compatibility library includes the UI engine, parsers,
font and SDL renderer. Normal
OS graphics libraries are still required. The Linux build uses glibc (Ubuntu 22.04
build baseline, glibc 2.35+) and needs a working X11 or Wayland desktop. Windows
builds use the MSVC runtime; install the matching Microsoft Visual C++ Redistributable
when deploying to a machine without it. The release Mac libraries require macOS 15+ and are ad-hoc signed for
linking; sign your final app and its embedded library for your distribution method.

Ship `NOTICES.txt` with your application. The header and import library are build
dependencies and do not have to be installed with the final app. The C ABI is
version 2; check `nui_abi_version()` before using it.
