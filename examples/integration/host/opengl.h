#ifndef UI_HOST_H
#define UI_HOST_H
#include "native_ui.h"

/* Copyable adapter for a bundled window. Include one host header per application.
   The caller owns ui and its loop; keep ui alive until after ui_host_close.
   host owns window: do not copy an open host or close its window separately.
   All calls belong on the main thread. No theme, test modes or application loop. */
typedef struct {
    NuiUi *ui;
    NuiWindow *window;
} UiHost;

static inline UiHost ui_host_open_backend(NuiUi *ui, const char *title, uint32_t width,
                                          uint32_t height, int hidden, uint32_t backend) {
    UiHost host = {ui, nui_window_open(title, width, height, hidden, backend)};
    return host; /* Check host.window before using it; errors use nui_last_error(). */
}
static inline UiHost ui_host_open(NuiUi *ui, const char *title, uint32_t width,
                                  uint32_t height, int hidden) {
    return ui_host_open_backend(ui, title, width, height, hidden, NUI_BACKEND_OPENGL);
}

/* -1 = error, 0 = continue, 1 = quit. Routes input and updates layout on resize. */
static inline int ui_host_poll(UiHost *host) {
    return nui_window_poll(host->window, host->ui);
}

/* Draw one complete frame; pass NULL for draw/userdata when no canvas is needed.
   Even if drawing fails, finish the opened frame and clear the UI's input edges.
   Returns 1 on success, 0 on error. Clearing/presentation belong to this helper;
   existing engine overlays should use the lower-level renderer APIs instead. */
static inline int ui_host_draw(UiHost *host, NuiColor background,
                                NuiCanvasCallback draw, void *userdata) {
    if (!nui_window_clear(host->window, background))
        return 0;
    int rendered = nui_window_render(host->window, host->ui, draw, userdata);
    int presented = nui_window_present(host->window);
    int ended = nui_end_frame(host->ui);
    return rendered && presented && ended;
}

/* Does not destroy the borrowed UI. Safe after a failed open or an earlier close. */
static inline int ui_host_close(UiHost *host) {
    if (!host->window)
        return 1;
    int closed = nui_window_close(host->window);
    if (closed)
        host->window = NULL;
    return closed;
}
#endif
