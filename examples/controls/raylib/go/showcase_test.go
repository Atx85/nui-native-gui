package main

import (
    "runtime"
    "testing"
    "native-ui-raylib-examples/support"
)

func TestControls(t *testing.T) {
    runtime.LockOSThread()
    defer runtime.UnlockOSThread()
    ui := support.NewUi(html, css, 940, 640)
    defer ui.Close()
    alternate := false
    ui.Click("name")
    ui.TextInput("!")
    update(ui, &alternate)
    if ui.Value("echo") != ui.Value("name") || !ui.Changed("name") { t.Fatal("text edit") }
    ui.EndFrame()
    ui.SetValue("name", "Host update")
    if ui.Changed("name") { t.Fatal("setter generated a user event") }
    ui.Click("svg")
    update(ui, &alternate)
    if ui.GroupCount("formats") != 2 { t.Fatal("group selection") }
    ui.EndFrame()
    ui.Click("options")
    update(ui, &alternate)
    ui.SetValue("mode", "draft")
    if ui.Value("mode") != "draft" { t.Fatal("option replacement") }
    ui.EndFrame()
    ui.Click("reset")
    update(ui, &alternate)
    if ui.Value("name") != "Ada" || ui.Value("volume") != "55" || ui.GroupCount("formats") != 1 || !ui.Checked("enabled") { t.Fatal("reset") }
    ui.EndFrame()
    ui.Click("apply")
    update(ui, &alternate)
    if ui.Value("status") != "Quality: normal / volume: 55" { t.Fatal("read values") }
    ui.EndFrame()
    ui.Click("swap")
    update(ui, &alternate)
    if !alternate { t.Fatal("image switch") }
    ui.EndFrame()
    ui.Click("scene")
    update(ui, &alternate)
    if ui.Value("status") != "Canvas pressed" { t.Fatal("canvas input") }
    ui.EndFrame()
    ui.Click("bottom")
    update(ui, &alternate)
    if ui.ScrollPosition("list").Y <= 0 { t.Fatal("scrolling") }
}
