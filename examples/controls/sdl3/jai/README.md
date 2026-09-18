# Jai showcase (experimental)

No Jai compiler is available here. The application, callbacks, and generated
bindings must be verified with the official toolchain before claiming support.

From the SDK directory:

```sh
jai examples/controls/sdl3/jai/generate.jai
jai examples/controls/sdl3/jai/showcase.jai
```

The generator reads `native_ui.h` and writes `native_ui.jai` next to the
showcase. Build for the SDK architecture and keep the executable beside its
native library from the SDK's `standalone/` directory (these bindings use the
older window-owning APIs). The complete application loop is in `showcase.jai`.
`support.jai` provides bindings/string conversion and theme selection. Set
`CHECK_ONLY` to run the separate state checks, or `SMOKE_TEST` for eight hidden
frames. Leave both false for interactive use.
