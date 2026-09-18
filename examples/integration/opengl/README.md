# OpenGL canvas integration

[triangle.c](triangle.c) draws an animated triangle with raw OpenGL shader calls
inside a UI canvas. It links to the Native UI SDK and loads OpenGL functions through
the window helper. No separate OpenGL loader or SDL headers are needed.

For a smaller canvas example using portable drawing helpers, see the
[OpenGL showcase](../../controls/opengl/README.md).

This example opens an SDK-owned window and requires the **standalone/**
compatibility library. Existing games use the [borrowed OpenGL showcase](../../controls/opengl/c/showcase.c)
and the default runtime instead. Link exactly one runtime.

## Build from the SDK folder

macOS (match `arm64` or `x86_64` to the downloaded SDK):

```sh
cc -arch arm64 -std=c11 examples/integration/opengl/triangle.c -I. -Lstandalone -lnative_ui \
  -Wl,-rpath,@loader_path -o standalone/triangle
./standalone/triangle
```

Linux:

```sh
cc -std=c11 examples/integration/opengl/triangle.c -I. -Lstandalone -lnative_ui \
  '-Wl,-rpath,$ORIGIN' -o standalone/triangle
./standalone/triangle
```

Windows, in an x64 Developer PowerShell for Visual Studio:

```powershell
cl /nologo /utf-8 /std:c11 examples/integration/opengl/triangle.c /I. /Fe:standalone/triangle.exe /link /LIBPATH:standalone native_ui.lib
.\standalone\triangle.exe
```

Keep the runtime library beside the executable and run from the SDK folder so it
can find `themes/`. A working graphics session with desktop OpenGL 3.3+ is required.
The SDK's `OPENGL.md` describes the rendering and callback contract; in the
repository, that guide is `crates/opengl/README.md`.
