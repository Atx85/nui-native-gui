/* A running game with a temporary UI. No custom font or drawing callbacks. */
#include "native_ui_sdl3.h"
#include "native_ui_themes.h"
#include <stdio.h>
#include <string.h>

int main(int argc, char **argv) {
    int smoke = argc > 1 && strcmp(argv[1], "--smoke-test") == 0;
    SDL_Window *window = NULL;
    SDL_Renderer *renderer = NULL;
    NuiUi *ui = NULL;
    NuiSdlRenderer *overlay = NULL;
    int failed = 1;
    if (!SDL_Init(SDL_INIT_VIDEO)) goto done;
    if (!SDL_CreateWindowAndRenderer("Game + temporary UI (F1 toggles UI)", 800, 600,
            SDL_WINDOW_RESIZABLE | (smoke ? SDL_WINDOW_HIDDEN : 0), &window, &renderer)) goto done;
    if (!SDL_SetRenderLogicalPresentation(renderer, 800, 600, SDL_LOGICAL_PRESENTATION_LETTERBOX)) goto done;

    /* Switch NUI_THEME_MACOS to NUI_THEME_WINDOWS_11 or NUI_THEME_WINDOWS_XP.
       The shared skin supplies appearance; the appended CSS only sets layout.
       A transparent page lets the game remain visible behind the controls. */
    ui = nui_create("<button id='debug' class='primary'>Toggle debug</button>",
        NUI_THEME_MACOS "body { background: transparent; } button { width: 180px; margin: 24px; }", 800, 600);
    if (!ui) goto done;
    overlay = nui_sdl_create(renderer);
    if (!overlay) goto done;

    int running = 1, debug = 0, frames = 0;
    while (running && (!smoke || frames < 8)) {
        SDL_Event event;
        while (SDL_PollEvent(&event)) {
            if (overlay && !nui_sdl_event(overlay, ui, &event)) goto done;
            /* The game receives the same event, even while the UI is active. */
            if (event.type == SDL_EVENT_QUIT) running = 0;
            if (event.type == SDL_EVENT_KEY_DOWN && event.key.key == SDLK_F1 && !event.key.repeat) {
                if (overlay) {
                    if (!nui_sdl_destroy(overlay)) goto done;
                    overlay = NULL;
                    if (!nui_pointer(ui, NUI_CANCEL, 0, 0)) goto done;
                } else {
                    overlay = nui_sdl_create(renderer);
                    if (!overlay) goto done;
                }
            }
        }
        if (!running) break;
        if (nui_clicked(ui, "debug") == 1) debug = !debug;

        /* Game simulation and drawing continue on every frame. */
        float x = (float)(SDL_GetTicks() % 4000) * 0.18f;
        SDL_FRect player = {x, 280, 64, 64};
        if (!SDL_SetRenderDrawColor(renderer, 24, 26, 32, 255) || !SDL_RenderClear(renderer)) goto done;
        if (!SDL_SetRenderDrawColor(renderer, debug ? 255 : 70, 200, 130, 255) ||
            !SDL_RenderFillRect(renderer, &player)) goto done;
        if (overlay && !nui_sdl_draw(overlay, ui)) goto done;
        if (!nui_end_frame(ui) || !SDL_RenderPresent(renderer)) goto done;
        SDL_Delay(16); /* The game's frame scheduler. */
        frames++;
    }
    failed = 0;
done:
    if (failed) fprintf(stderr, "Native UI: %s\nSDL: %s\n", nui_last_error(), SDL_GetError());
    if (!nui_sdl_destroy(overlay)) failed = 1;
    if (!nui_destroy(ui)) failed = 1;
    SDL_DestroyRenderer(renderer);
    SDL_DestroyWindow(window);
    SDL_Quit();
    return failed;
}
