// UNVERIFIED reference: compiler not tested; older SDK-owned standalone connector.
// Prefer the C/Rust showcases for host-owned game integration. See docs/languages.md.
// Controls, events and application updates using the SDK-owned SDL3 connector.
// Change themeMACOS in support/support.go to themeWINDOWS_11 or themeWINDOWS_XP.
package main

import (
    "fmt"
    "os"
    "native-ui-examples/support"
)

// BEGIN UI
const html = `<body>
  <section id="controls" class="panel">
    <label class="title">Controls</label>
    <label for="name">Player name</label>
    <input id="name" value="Ada" maxlength="40" placeholder="Your name">
    <input id="echo" value="Ada" readonly>
    <div class="row"><input id="enabled" type="checkbox" checked><label class="row-label" for="enabled">Enable sound</label></div>
    <label>Export formats</label>
    <div class="row">
      <input id="png" name="formats" type="checkbox" value="png" checked><label class="row-label" for="png">PNG</label>
      <input id="svg" name="formats" type="checkbox" value="svg"><label class="row-label" for="svg">SVG</label>
    </div>
    <label for="volume">Volume</label>
    <input id="volume" type="range" min="0" max="100" step="5" value="55">
    <label for="mode">Quality</label>
    <select id="mode"><option value="normal" selected>Normal</option><option value="high">High</option></select>
    <div class="row"><button id="apply" class="action primary">Read values</button><button id="reset" class="action">Reset values</button></div>
    <div class="row"><button id="options" class="action">Replace options</button><button class="action" disabled>Unavailable</button></div>
  </section>
  <section id="preview" class="panel">
    <label class="title">Images, canvas and scrolling</label>
    <div class="row"><img id="picture" src="first" alt="Color tiles"><button id="swap" class="action">Switch image</button></div>
    <canvas id="scene" tabindex="0"></canvas>
    <label>Click the native canvas above.</label>
    <div id="list" tabindex="0">
      <label class="list-item">Row 1</label><label class="list-item">Row 2</label><label class="list-item">Row 3</label><label class="list-item">Row 4</label>
      <label class="list-item">Row 5</label><label class="list-item">Row 6</label><label class="list-item">Row 7</label><label class="list-item">Row 8</label>
    </div>
    <div class="row"><button id="top" class="action">Scroll to top</button><button id="bottom" class="action">Scroll to bottom</button></div>
    <input id="status" value="Ready" readonly>
  </section>
</body>
`
const css = `/* Geometry only; the canonical macOS theme supplies all control appearance. */
body { display: flex; gap: 24px; padding: 24px; }
.panel { padding: 16px; display: flex; flex-direction: column; gap: 8px; overflow: auto; }
#controls { width: 360px; }
#preview { flex-grow: 1; min-width: 280px; }
.title { font-size: 18px; height: 28px; }
.row { display: flex; align-items: center; gap: 8px; height: 32px; }
.row-label { width: 76px; }
.action { flex-grow: 1; }
#picture { width: 80px; height: 32px; object-fit: contain; }
#scene { height: 110px; }
#list { height: 96px; overflow: auto; display: flex; flex-direction: column; }
.list-item { height: 32px; }
`
// END UI

func update(ui *support.Ui, alternate *bool) {
    if ui.Changed("name") { ui.SetValue("echo", ui.Value("name")) }
    if ui.Changed("enabled") {
        message := "Sound off"
        if ui.Checked("enabled") { message = "Sound on" }
        ui.SetValue("status", message)
    }
    if ui.GroupChanged("formats") {
        ui.SetValue("status", fmt.Sprintf("%d formats selected", ui.GroupCount("formats")))
    }
    if ui.Changed("volume") || ui.Changed("mode") || ui.Clicked("apply") {
        ui.SetValue("status", fmt.Sprintf("Quality: %s / volume: %s", ui.Value("mode"), ui.Value("volume")))
    }
    if ui.Clicked("reset") {
        ui.SetValue("name", "Ada")
        ui.SetValue("echo", "Ada")
        ui.SetChecked("enabled", true)
        ui.SetValue("volume", "55")
        ui.SetValue("mode", "normal")
        ui.SetCheckedValues("formats", []string{"png"})
        ui.SetValue("status", "Values reset")
    }
    if ui.Clicked("options") {
        ui.ReplaceOptions("mode", [][2]string{{"normal", "Balanced"}, {"high", "Best quality"}, {"draft", "Draft"}})
    }
    if ui.Clicked("swap") {
        *alternate = !*alternate
        source := "first"
        if *alternate { source = "second" }
        ui.ImageSource("picture", source)
    }
    if ui.Clicked("top") { ui.ScrollTo("list", 0) }
    if ui.Clicked("bottom") { ui.ScrollTo("list", 10000) }
    if ui.Pressed("scene") { ui.SetValue("status", "Canvas pressed") }
}

func draw(renderer *support.Renderer, content support.Rect) {
    renderer.Fill(support.Rect{X: content.X+16, Y: content.Y+16, W: 64, H: 64},
                  support.Color{R: 54, G: 95, B: 221, A: 255})
}

func main() {
    ui := support.NewUi(html, css, 940, 640)
    defer ui.Close()
    smoke := os.Getenv("NUI_EXAMPLE_MODE") == "smoke"
    window := support.OpenWindow("Controls — Go / SDL3", 940, 640, smoke)
    defer window.Close()
    window.Image("first", 2, 2, []byte{255,90,90,255, 70,150,255,255, 80,210,150,255, 255,210,80,255})
    window.Image("second", 2, 2, []byte{80,210,150,255, 255,210,80,255, 255,90,90,255, 70,150,255,255})
    alternate := false
    for frame := 0; !smoke || frame < 8; frame++ {
        if !window.Poll(ui) { break }
        update(ui, &alternate)
        window.Clear(support.Color{R: 243, G: 243, B: 243, A: 255})
        window.Render(ui, draw)
        window.Present()
        ui.EndFrame()
    }
    if smoke { window.CheckCache() }
}
