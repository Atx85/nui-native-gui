// UNVERIFIED reference: compiler not tested; older SDK-owned standalone connector.
// Prefer the C/Rust showcases for host-owned game integration. See docs/languages.md.
// Controls, events and application updates. Experimental: compiler verification required.
// Use THEME_WINDOWS_11 or THEME_WINDOWS_XP in support.jai to change the skin.
#load "support.jai";
// BEGIN UI
HTML :: #string END_HTML
<body>
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
END_HTML
CSS :: #string END_CSS
/* Geometry only; the canonical macOS theme supplies all control appearance. */
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
END_CSS
;
// END UI

alternate: bool;

update :: (ui: *NuiUi) {
    if changed(ui, "name") {
        text := value(ui, "name");
        defer free(text.data);
        set_value(ui, "echo", text);
    }
    if changed(ui, "enabled")
        set_value(ui, "status", ifx checked(ui, "enabled") then "Sound on" else "Sound off");
    if nui_checkbox_group_changed(ui, "formats\0".data) == 1 {
        count: u64;
        assert(nui_checked_value_count(ui, "formats\0".data, *count) == 1);
        set_value(ui, "status", tprint("% formats selected", count));
    }
    if changed(ui, "volume") || changed(ui, "mode") || clicked(ui, "apply") {
        quality := value(ui, "mode"); defer free(quality.data);
        volume := value(ui, "volume"); defer free(volume.data);
        set_value(ui, "status", tprint("Quality: % / volume: %", quality, volume));
    }
    if clicked(ui, "reset") {
        set_value(ui, "name", "Ada");
        set_value(ui, "echo", "Ada");
        assert(nui_set_checked(ui, "enabled\0".data, 1) == 1);
        set_value(ui, "volume", "55");
        set_value(ui, "mode", "normal");
        formats: [1]*u8 = .["png\0".data];
        assert(nui_set_checked_values(ui, "formats\0".data, formats.data, 1) == 1);
        set_value(ui, "status", "Values reset");
    }
    if clicked(ui, "options") {
        options: [3]NuiSelectOption;
        options[0] = .{value="normal\0".data, label="Balanced\0".data, selected=1};
        options[1] = .{value="high\0".data, label="Best quality\0".data};
        options[2] = .{value="draft\0".data, label="Draft\0".data};
        assert(nui_set_select_options(ui, "mode\0".data, options.data, 3) == 1);
    }
    if clicked(ui, "swap") {
        alternate = !alternate;
        source := to_c_string(ifx alternate then "second" else "first");
        defer free(source);
        assert(nui_set_image_source(ui, "picture\0".data, source) == 1);
    }
    if clicked(ui, "top") scroll_to(ui, "list", 0);
    if clicked(ui, "bottom") scroll_to(ui, "list", 10000);
    input: NuiCanvasInput;
    assert(nui_canvas_input(ui, "scene\0".data, *input) == 1);
    if input.flags & NUI_PRESSED != 0 set_value(ui, "status", "Canvas pressed");
}

draw :: (data: *void, renderer: *void, id: NuiString, bounds: NuiRect, content: NuiRect, clip: NuiRect) -> s32 #c_call {
    square: NuiRect = .{x=content.x+16, y=content.y+16, w=64, h=64};
    blue: NuiColor = .{r=54, g=95, b=221, a=255};
    return ifx nui_raylib_fill(renderer, square, blue) == 1 then 0 else 1;
}

main :: () {
    h := to_c_string(HTML); defer free(h);
    styles := to_c_string(tprint("%%", THEME, CSS)); defer free(styles);
    ui := nui_create(h, styles, 940, 640);
    assert(ui != null);
    defer nui_destroy(ui);
    #if CHECK_ONLY { check(ui); return; }
    title := to_c_string("Controls — Jai / raylib"); defer free(title);
    window := nui_raylib_open(title, 940, 640, ifx SMOKE_TEST then 1 else 0);
    assert(window != null);
    defer nui_raylib_close(window);
    first: [16]u8 = .[255,90,90,255, 70,150,255,255, 80,210,150,255, 255,210,80,255];
    second: [16]u8 = .[80,210,150,255, 255,210,80,255, 255,90,90,255, 70,150,255,255];
    assert(nui_raylib_register_image(window, "first\0".data, 2, 2, first.data, 16) == 1);
    assert(nui_raylib_register_image(window, "second\0".data, 2, 2, second.data, 16) == 1);
    frame := 0;
    while !SMOKE_TEST || frame < 8 {
        event := nui_raylib_poll(window, ui);
        assert(event >= 0);
        if event == 1 break;
        update(ui);
        background: NuiColor = .{r=243, g=243, b=243, a=255};
        assert(nui_raylib_clear(window, background) == 1);
        assert(nui_raylib_render(window, ui, draw, null) == 1);
        assert(nui_raylib_present(window) == 1);
        assert(nui_end_frame(ui) == 1);
        frame += 1;
    }
}
#if CHECK_ONLY { #load "tests/checks.jai"; }
