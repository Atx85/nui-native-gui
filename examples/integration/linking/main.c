/* A complete application: include the SDK header and link its shared library. */
#define _DEFAULT_SOURCE
#include "native_ui.h"
#include "native_ui_themes.h"
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <unistd.h>
#endif

int main(int argc, char **argv) {
    if (nui_abi_version() != 2)
        return 1;
    /* Switch NUI_THEME_MACOS to NUI_THEME_WINDOWS_11 or NUI_THEME_WINDOWS_XP.
       The shared skin supplies appearance; the appended CSS only sets layout. */
    NuiUi *ui = nui_create("<button id='hello' class='primary'>Click me</button>",
                           NUI_THEME_MACOS "button { width: 180px; margin: 24px; }",
                           320, 160);
    if (!ui) {
        fprintf(stderr, "%s\n", nui_last_error());
        return 1;
    }
    if (argc == 2 && strcmp(argv[1], "--check") == 0) {
        NuiRect rect;
        int success = nui_bounds(ui, "hello", &rect) == 1 &&
                      nui_pointer(ui, NUI_DOWN, rect.x + 10, rect.y + 10) == 1 &&
                      nui_pointer(ui, NUI_UP, rect.x + 10, rect.y + 10) == 1 &&
                      nui_clicked(ui, "hello") == 1;
        nui_destroy(ui);
        return success ? 0 : 1;
    }
    NuiSdl *window = nui_sdl_open("Hello UI", 320, 160, 0);
    if (!window) {
        fprintf(stderr, "%s\n", nui_last_error());
        nui_destroy(ui);
        return 1;
    }
    int result = 0;
    for (;;) {
        int event = nui_sdl_poll(window, ui);
        if (event < 0) {
            result = 1;
            break;
        }
        if (event == 1)
            break;
        if (nui_clicked(ui, "hello") == 1)
            puts("Hello from the shared library!");
        NuiColor background = {243, 243, 243, 255};
        if (!nui_sdl_clear(window, background) || !nui_sdl_render(window, ui, NULL, NULL) ||
            !nui_sdl_present(window) || !nui_end_frame(ui)) {
            result = 1;
            break;
        }
#ifdef _WIN32
        Sleep(16);
#else
        usleep(16000);
#endif
    }
    if (result)
        fprintf(stderr, "%s\n", nui_last_error());
    nui_sdl_close(window);
    nui_destroy(ui);
    return result;
}
