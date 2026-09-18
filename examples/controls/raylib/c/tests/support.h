/* Test harness only. No example includes this header. */
#include "native_ui.h"
static int test_frames;
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static inline void require(int condition) {
    if (!condition) {
        fprintf(stderr, "Element check failed: %s\n", nui_last_error());
        exit(1);
    }
}
static inline void ok(int result) { require(result == 1); }
static inline const char *value(NuiUi *ui, const char *id) {
    static char buffer[4096];
    size_t size = nui_get_value(ui, id, buffer, sizeof buffer);
    require(size > 0 && size <= sizeof buffer);
    return buffer;
}
static inline NuiRect bounds(NuiUi *ui, const char *id) {
    NuiRect r;
    ok(nui_bounds(ui, id, &r));
    return r;
}
static inline void click(NuiUi *ui, const char *id) {
    NuiRect r = bounds(ui, id);
    ok(nui_pointer(ui, NUI_DOWN, r.x + r.w / 2, r.y + r.h / 2));
    ok(nui_pointer(ui, NUI_UP, r.x + r.w / 2, r.y + r.h / 2));
}

#include "native_ui_raylib.h"
static void test_init(int w, int h, const char *title) {
    SetConfigFlags(FLAG_WINDOW_HIDDEN | FLAG_WINDOW_RESIZABLE | FLAG_WINDOW_HIGHDPI);
    InitWindow(w,h,title);
}
#define InitWindow test_init
#define WindowShouldClose() (test_frames >= 8 || WindowShouldClose())
/* Hooks live only in this harness. The application uses real framework calls. */
static int32_t test_end_frame(NuiUi *ui) {
    int result = nui_end_frame(ui);
    const char *mode = getenv("NUI_EXAMPLE_MODE");
    if (mode && strcmp(mode,"smoke") == 0 && ++test_frames == 3) click(ui,"swap");
    return result;
}
#define nui_end_frame test_end_frame
