// UNVERIFIED reference: compiler not tested; older SDK-owned standalone connector.
// Prefer the C/Rust showcases for host-owned game integration. See docs/languages.md.
//! Controls, events and application updates using the SDK-owned OpenGL connector.
// Select themes.windows_11 or themes.windows_xp in support.zig for another skin.
const std = @import("std");
pub const app = @import("support.zig");
pub const c = app.c;
// BEGIN UI
pub const html =
    \\<body>
    \\  <section id="controls" class="panel">
    \\    <label class="title">Controls</label>
    \\    <label for="name">Player name</label>
    \\    <input id="name" value="Ada" maxlength="40" placeholder="Your name">
    \\    <input id="echo" value="Ada" readonly>
    \\    <div class="row"><input id="enabled" type="checkbox" checked><label class="row-label" for="enabled">Enable sound</label></div>
    \\    <label>Export formats</label>
    \\    <div class="row">
    \\      <input id="png" name="formats" type="checkbox" value="png" checked><label class="row-label" for="png">PNG</label>
    \\      <input id="svg" name="formats" type="checkbox" value="svg"><label class="row-label" for="svg">SVG</label>
    \\    </div>
    \\    <label for="volume">Volume</label>
    \\    <input id="volume" type="range" min="0" max="100" step="5" value="55">
    \\    <label for="mode">Quality</label>
    \\    <select id="mode"><option value="normal" selected>Normal</option><option value="high">High</option></select>
    \\    <div class="row"><button id="apply" class="action primary">Read values</button><button id="reset" class="action">Reset values</button></div>
    \\    <div class="row"><button id="options" class="action">Replace options</button><button class="action" disabled>Unavailable</button></div>
    \\  </section>
    \\  <section id="preview" class="panel">
    \\    <label class="title">Images, canvas and scrolling</label>
    \\    <div class="row"><img id="picture" src="first" alt="Color tiles"><button id="swap" class="action">Switch image</button></div>
    \\    <canvas id="scene" tabindex="0"></canvas>
    \\    <label>Click the native canvas above.</label>
    \\    <div id="list" tabindex="0">
    \\      <label class="list-item">Row 1</label><label class="list-item">Row 2</label><label class="list-item">Row 3</label><label class="list-item">Row 4</label>
    \\      <label class="list-item">Row 5</label><label class="list-item">Row 6</label><label class="list-item">Row 7</label><label class="list-item">Row 8</label>
    \\    </div>
    \\    <div class="row"><button id="top" class="action">Scroll to top</button><button id="bottom" class="action">Scroll to bottom</button></div>
    \\    <input id="status" value="Ready" readonly>
    \\  </section>
    \\</body>
;
pub const css =
    \\/* Geometry only; the canonical macOS theme supplies all control appearance. */
    \\body { display: flex; gap: 24px; padding: 24px; }
    \\.panel { padding: 16px; display: flex; flex-direction: column; gap: 8px; overflow: auto; }
    \\#controls { width: 360px; }
    \\#preview { flex-grow: 1; min-width: 280px; }
    \\.title { font-size: 18px; height: 28px; }
    \\.row { display: flex; align-items: center; gap: 8px; height: 32px; }
    \\.row-label { width: 76px; }
    \\.action { flex-grow: 1; }
    \\#picture { width: 80px; height: 32px; object-fit: contain; }
    \\#scene { height: 110px; }
    \\#list { height: 96px; overflow: auto; display: flex; flex-direction: column; }
    \\.list-item { height: 32px; }
;
// END UI

pub fn update(ui: *c.NuiUi, alternate: *bool) !void {
    var text: [256]u8 = undefined;
    if (c.nui_changed(ui, "name") == 1)
        try app.ok(c.nui_set_value(ui, "echo", (try app.value(ui, "name", &text)).ptr));
    if (c.nui_changed(ui, "enabled") == 1)
        try app.ok(c.nui_set_value(ui, "status", if (c.nui_checked(ui, "enabled") == 1) "Sound on" else "Sound off"));
    if (c.nui_checkbox_group_changed(ui, "formats") == 1) {
        var count: usize = 0;
        try app.ok(c.nui_checked_value_count(ui, "formats", &count));
        const message = try std.fmt.bufPrintZ(&text, "{d} formats selected", .{count});
        try app.ok(c.nui_set_value(ui, "status", message));
    }
    if (c.nui_changed(ui, "volume") == 1 or c.nui_changed(ui, "mode") == 1 or c.nui_clicked(ui, "apply") == 1) {
        var mode: [32]u8 = undefined;
        var volume: [16]u8 = undefined;
        const message = try std.fmt.bufPrintZ(&text, "Quality: {s} / volume: {s}",
            .{ try app.value(ui, "mode", &mode), try app.value(ui, "volume", &volume) });
        try app.ok(c.nui_set_value(ui, "status", message));
    }
    if (c.nui_clicked(ui, "reset") == 1) {
        try app.ok(c.nui_set_value(ui, "name", "Ada"));
        try app.ok(c.nui_set_value(ui, "echo", "Ada"));
        try app.ok(c.nui_set_checked(ui, "enabled", 1));
        try app.ok(c.nui_set_value(ui, "volume", "55"));
        try app.ok(c.nui_set_value(ui, "mode", "normal"));
        const formats = [_][*c]const u8{"png"};
        try app.ok(c.nui_set_checked_values(ui, "formats", &formats, formats.len));
        try app.ok(c.nui_set_value(ui, "status", "Values reset"));
    }
    if (c.nui_clicked(ui, "options") == 1) {
        const options = [_]c.NuiSelectOption{
            .{ .value = "normal", .label = "Balanced", .class_name = null, .disabled = 0, .selected = 1 },
            .{ .value = "high", .label = "Best quality", .class_name = null, .disabled = 0, .selected = 0 },
            .{ .value = "draft", .label = "Draft", .class_name = null, .disabled = 0, .selected = 0 },
        };
        try app.ok(c.nui_set_select_options(ui, "mode", &options, options.len));
    }
    if (c.nui_clicked(ui, "swap") == 1) {
        alternate.* = !alternate.*;
        try app.ok(c.nui_set_image_source(ui, "picture", if (alternate.*) "second" else "first"));
    }
    if (c.nui_clicked(ui, "top") == 1) try app.ok(c.nui_scroll_to(ui, "list", 0, 0));
    if (c.nui_clicked(ui, "bottom") == 1) try app.ok(c.nui_scroll_to(ui, "list", 0, 10000));
    var input: c.NuiCanvasInput = undefined;
    try app.ok(c.nui_canvas_input(ui, "scene", &input));
    if (input.flags & c.NUI_PRESSED != 0) try app.ok(c.nui_set_value(ui, "status", "Canvas pressed"));
}

fn draw(_: ?*anyopaque, renderer: ?*anyopaque, _: c.NuiString, _: c.NuiRect, content: c.NuiRect, _: c.NuiRect) callconv(.c) i32 {
    const square: c.NuiRect = .{ .x = content.x + 16, .y = content.y + 16, .w = 64, .h = 64 };
    return if (c.nui_draw_fill(renderer, square, .{ .r = 54, .g = 95, .b = 221, .a = 255 }) == 1) 0 else 1;
}

pub fn main() !void {
    const styles = try std.fmt.allocPrintSentinel(std.heap.page_allocator, "{s}{s}", .{ app.base_css, css }, 0);
    defer std.heap.page_allocator.free(styles);
    const ui = c.nui_create(html, styles, 940, 640) orelse return error.Create;
    defer _ = c.nui_destroy(ui);
    const mode = if (app.getenv("NUI_EXAMPLE_MODE")) |m| std.mem.span(m) else "";
    const smoke = std.mem.eql(u8, mode, "smoke");
    const window = c.nui_window_open("Controls — Zig / OpenGL", 940, 640, @intFromBool(smoke), c.NUI_BACKEND_OPENGL) orelse return error.Window;
    defer _ = c.nui_window_close(window);
    const first = [_]u8{255,90,90,255, 70,150,255,255, 80,210,150,255, 255,210,80,255};
    const second = [_]u8{80,210,150,255, 255,210,80,255, 255,90,90,255, 70,150,255,255};
    try app.ok(c.nui_window_register_image(window, "first", 2, 2, &first, first.len));
    try app.ok(c.nui_window_register_image(window, "second", 2, 2, &second, second.len));
    var alternate = false;
    var frame: usize = 0;
    while (!smoke or frame < 8) : (frame += 1) {
        const event = c.nui_window_poll(window, ui);
        if (event < 0) return error.Poll;
        if (event == 1) break;
        try update(ui, &alternate);
        try app.ok(c.nui_window_clear(window, .{ .r = 243, .g = 243, .b = 243, .a = 255 }));
        try app.ok(c.nui_window_render(window, ui, draw, null));
        try app.ok(c.nui_window_present(window));
        try app.ok(c.nui_end_frame(ui));
    }
}
