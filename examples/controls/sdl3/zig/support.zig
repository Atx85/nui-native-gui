const std = @import("std");
pub const c = @import("native_ui");
pub extern "c" fn getenv([*:0]const u8) ?[*:0]const u8;
pub extern "c" fn usleep(c_uint) c_int;
pub fn ok(result: i32) !void {
    if (result != 1) {
        std.debug.print("native-ui: {s}\n", .{c.nui_last_error()});
        return error.NativeUI;
    }
}
pub fn require(condition: bool) !void {
    if (!condition) return error.ElementCheck;
}
pub fn value(ui: *c.NuiUi, id: [*:0]const u8, buffer: []u8) ![:0]const u8 {
    const n = c.nui_get_value(ui, id, buffer.ptr, buffer.len);
    if (n == 0 or n > buffer.len) return error.Value;
    return buffer[0 .. n - 1 :0];
}
pub fn bounds(ui: *c.NuiUi, id: [*:0]const u8) !c.NuiRect {
    var r: c.NuiRect = undefined;
    try ok(c.nui_bounds(ui, id, &r));
    return r;
}
pub fn click(ui: *c.NuiUi, id: [*:0]const u8) !void {
    const r = try bounds(ui, id);
    try ok(c.nui_pointer(ui, c.NUI_DOWN, r.x + r.w / 2, r.y + r.h / 2));
    try ok(c.nui_pointer(ui, c.NUI_UP, r.x + r.w / 2, r.y + r.h / 2));
}
pub fn expectValue(ui: *c.NuiUi, id: [*:0]const u8, expected: []const u8) !void {
    var buffer: [4096]u8 = undefined;
    try require(std.mem.eql(u8, try value(ui, id, &buffer), expected));
}
// Use themes.windows_11 or themes.windows_xp to select another canonical skin.
const themes = @import("themes.zig");
pub const base_css = themes.macos;
