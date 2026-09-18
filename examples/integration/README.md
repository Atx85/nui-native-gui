# Integration examples

- [Existing SDL3 game](sdl3/README.md): borrow your renderer, forward events and
  draw an overlay while your game keeps its own loop and SDL ownership.
- [Small C integration](host/README.md): copy a host adapter and keep your own
  application loop; input and frame drawing each take one call.
- [Link the library](linking/README.md): a standalone C application with CMake
  and direct compiler instructions for macOS, Linux and Windows.
- [OpenGL canvas](opengl/README.md): raw shader drawing inside a UI canvas.

For small control examples in all five languages, start with
[control showcases](../controls/README.md).
