# Examples

Start with [one complete control showcase](controls/README.md) for your renderer
and language. There are **15 application sources**, covering three renderers and
five languages. **C and Rust are the verified host-owned integrations; Go, Zig
and Jai are unverified standalone references.**

Use [C control recipes](../docs/controls.md) to learn a single control's markup,
user events, reading values, and updating data. The runnable showcases demonstrate
those operations together. They default to the canonical macOS light theme, with
comments showing how to choose Windows 11 or Windows XP.

| Collection | Purpose |
| --- | --- |
| [Controls](controls/README.md) | All controls, events, and host updates in one file per renderer/language. |
| [Existing SDL3 game](integration/sdl3/README.md) | Borrow a game's renderer; the host owns the SDL setup and loop. |
| [OpenGL integration](integration/opengl/README.md) | Use a host-owned OpenGL context. |
| [Linking](integration/linking/README.md) | CMake and direct C/C++ SDK consumers. |
| [Optional standalone host adapters](integration/host/README.md) | Existing convenience window connectors. |
| [Rendering demos](showcase/README.md) | Repository rendering fixtures, themed studios, and benchmarks. |

```sh
python3 tools/build-sdk.py --out target/my-sdk
python3 tools/build-language-example.py c --backend raylib --sdk target/my-sdk --run
```

Choose `c` or `rust`; Go/Zig/Jai remain unverified references. Use `--check`
for separate input/state tests, or `--smoke-test` for short hidden rendering
checks. See [language setup](../docs/languages.md) for toolchains and ownership.

The recommended C/Rust showcases use host-owned windows on every backend.
Go/Zig/Jai are prominently labelled unverified standalone reference sources.
See [language status](../docs/languages.md) for the actual verification scope.
