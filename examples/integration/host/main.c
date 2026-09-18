/* Copy this file and one host header into your app, then link the native UI SDK. */
#define _DEFAULT_SOURCE
#ifndef UI_HOST_HEADER
#define UI_HOST_HEADER "sdl3.h"
#endif
#include UI_HOST_HEADER
#include "frame_delay.h"
#include "native_ui_themes.h"
#include <stdio.h>

int main(void) {
    if (nui_abi_version() != 2) {
        fputs("This example needs native UI ABI 2\n", stderr);
        return 1;
    }
    // Parse markup and styles once; this UI retains control state between frames.
    /* Switch NUI_THEME_MACOS to NUI_THEME_WINDOWS_11 or NUI_THEME_WINDOWS_XP.
       The shared skin supplies appearance; the appended CSS only sets layout. */
    NuiUi *ui = nui_create("<button id='hello' class='primary'>Click me</button>",
        NUI_THEME_MACOS "button { width: 180px; margin: 24px; }", 320, 160);
    if (!ui) {
        fprintf(stderr, "%s\n", nui_last_error());
        return 1;
    }
    UiHost host = ui_host_open(ui, "Hello UI", 320, 160, 0);
    int failed = !host.window;
    while (!failed) {
        // Feed window input into the UI and update its layout when resized.
        int event = ui_host_poll(&host);
        if (event != 0) {
            failed = event < 0;
            break;
        }
        // Read this frame's actions before drawing clears the transient input flags.
        if (nui_clicked(ui, "hello") == 1)
            puts("Hello from my application!");
        NuiColor background = {243, 243, 243, 255};
        // Clear, render, present, then end the UI frame. NULL means no canvas callback.
        failed = !ui_host_draw(&host, background, NULL, NULL);
        if (!failed)
            ui_host_wait();
    }
    if (failed)
        fprintf(stderr, "%s\n", nui_last_error());
    // The host borrows ui, so close its window before releasing the UI.
    int closed = ui_host_close(&host);
    int destroyed = nui_destroy(ui);
    return failed || !closed || !destroyed;
}
