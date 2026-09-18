/* Host-owned SDL3 window and OpenGL context. */
#include "native_ui_sdl3.h"
#include "native_ui_themes.h"
#include <stdio.h>
#include <stdlib.h>

// Use NUI_THEME_WINDOWS_11 or NUI_THEME_WINDOWS_XP to change the skin.
#define EXAMPLE_THEME NUI_THEME_MACOS
// BEGIN UI
static const char *HTML =
    "<body>\n"
    "  <section id=\"controls\" class=\"panel\">\n"
    "    <label class=\"title\">Controls</label>\n"
    "    <label for=\"name\">Player name</label>\n"
    "    <input id=\"name\" value=\"Ada\" maxlength=\"40\" placeholder=\"Your name\">\n"
    "    <input id=\"echo\" value=\"Ada\" readonly>\n"
    "    <div class=\"row\"><input id=\"enabled\" type=\"checkbox\" checked><label class=\"row-label\" for=\"enabled\">Enable sound</label></div>\n"
    "    <label>Export formats</label>\n"
    "    <div class=\"row\">\n"
    "      <input id=\"png\" name=\"formats\" type=\"checkbox\" value=\"png\" checked><label class=\"row-label\" for=\"png\">PNG</label>\n"
    "      <input id=\"svg\" name=\"formats\" type=\"checkbox\" value=\"svg\"><label class=\"row-label\" for=\"svg\">SVG</label>\n"
    "    </div>\n"
    "    <label for=\"volume\">Volume</label>\n"
    "    <input id=\"volume\" type=\"range\" min=\"0\" max=\"100\" step=\"5\" value=\"55\">\n"
    "    <label for=\"mode\">Quality</label>\n"
    "    <select id=\"mode\"><option value=\"normal\" selected>Normal</option><option value=\"high\">High</option></select>\n"
    "    <div class=\"row\"><button id=\"apply\" class=\"action primary\">Read values</button><button id=\"reset\" class=\"action\">Reset values</button></div>\n"
    "    <div class=\"row\"><button id=\"options\" class=\"action\">Replace options</button><button class=\"action\" disabled>Unavailable</button></div>\n"
    "  </section>\n"
    "  <section id=\"preview\" class=\"panel\">\n"
    "    <label class=\"title\">Images, canvas and scrolling</label>\n"
    "    <div class=\"row\"><img id=\"picture\" src=\"first\" alt=\"Color tiles\"><button id=\"swap\" class=\"action\">Switch image</button></div>\n"
    "    <canvas id=\"scene\" tabindex=\"0\"></canvas>\n"
    "    <label>Click the native canvas above.</label>\n"
    "    <div id=\"list\" tabindex=\"0\">\n"
    "      <label class=\"list-item\">Row 1</label><label class=\"list-item\">Row 2</label><label class=\"list-item\">Row 3</label><label class=\"list-item\">Row 4</label>\n"
    "      <label class=\"list-item\">Row 5</label><label class=\"list-item\">Row 6</label><label class=\"list-item\">Row 7</label><label class=\"list-item\">Row 8</label>\n"
    "    </div>\n"
    "    <div class=\"row\"><button id=\"top\" class=\"action\">Scroll to top</button><button id=\"bottom\" class=\"action\">Scroll to bottom</button></div>\n"
    "    <input id=\"status\" value=\"Ready\" readonly>\n"
    "  </section>\n"
    "</body>\n";
static const char *CSS = EXAMPLE_THEME
    "/* Geometry only; the canonical macOS theme supplies all control appearance. */\n"
    "body { display: flex; gap: 24px; padding: 24px; }\n"
    ".panel { padding: 16px; display: flex; flex-direction: column; gap: 8px; overflow: auto; }\n"
    "#controls { width: 360px; }\n"
    "#preview { flex-grow: 1; min-width: 280px; }\n"
    ".title { font-size: 18px; height: 28px; }\n"
    ".row { display: flex; align-items: center; gap: 8px; height: 32px; }\n"
    ".row-label { width: 76px; }\n"
    ".action { flex-grow: 1; }\n"
    "#picture { width: 80px; height: 32px; object-fit: contain; }\n"
    "#scene { height: 110px; }\n"
    "#list { height: 96px; overflow: auto; display: flex; flex-direction: column; }\n"
    ".list-item { height: 32px; }\n";
// END UI

// BEGIN ACTIONS
static int update(NuiUi *ui) {
    // User edits: read data with the public API, then update another control.
    if (nui_changed(ui, "name") == 1) {
        char name[256]; // maxlength=40, including space for multibyte UTF-8.
        size_t bytes = nui_get_value(ui, "name", name, sizeof name);
        if (!bytes || bytes > sizeof name) return 0;
        if (!nui_set_value(ui, "echo", name)) return 0;
    }
    if (nui_changed(ui, "enabled") == 1) {
        int enabled = nui_checked(ui, "enabled");
        if (enabled < 0) return 0;
        if (!nui_set_value(ui, "status", enabled ? "Sound on" : "Sound off")) return 0;
    }
    if (nui_checkbox_group_changed(ui, "formats") == 1) {
        size_t count;
        char message[64];
        if (!nui_checked_value_count(ui, "formats", &count)) return 0;
        snprintf(message, sizeof message, "%zu formats selected", count);
        if (!nui_set_value(ui, "status", message)) return 0;
    }
    if (nui_changed(ui, "volume") == 1 || nui_changed(ui, "mode") == 1 ||
        nui_clicked(ui, "apply") == 1) {
        char mode[32], volume[16], message[96];
        size_t a = nui_get_value(ui, "mode", mode, sizeof mode);
        size_t b = nui_get_value(ui, "volume", volume, sizeof volume);
        if (!a || a > sizeof mode || !b || b > sizeof volume) return 0;
        snprintf(message, sizeof message, "Quality: %s / volume: %s", mode, volume);
        if (!nui_set_value(ui, "status", message)) return 0;
    }

    // Application changes: these setters do not generate user-change events.
    if (nui_clicked(ui, "reset") == 1) {
        const char *formats[] = {"png"};
        if (!nui_set_value(ui, "name", "Ada") || !nui_set_value(ui, "echo", "Ada") ||
            !nui_set_checked(ui, "enabled", 1) || !nui_set_value(ui, "volume", "55") ||
            !nui_set_value(ui, "mode", "normal") ||
            !nui_set_checked_values(ui, "formats", formats, 1) ||
            !nui_set_value(ui, "status", "Values reset")) return 0;
    }
    if (nui_clicked(ui, "options") == 1) {
        const NuiSelectOption options[] = {
            {"normal", "Balanced", NULL, 0, 1},
            {"high", "Best quality", NULL, 0, 0},
            {"draft", "Draft", NULL, 0, 0},
        };
        if (!nui_set_select_options(ui, "mode", options, 3)) return 0;
    }
    if (nui_clicked(ui, "swap") == 1) {
        static int alternate;
        alternate = !alternate;
        if (!nui_set_image_source(ui, "picture", alternate ? "second" : "first")) return 0;
    }
    if (nui_clicked(ui, "top") == 1 && !nui_scroll_to(ui, "list", 0, 0)) return 0;
    if (nui_clicked(ui, "bottom") == 1 && !nui_scroll_to(ui, "list", 0, 10000)) return 0;
    NuiCanvasInput input;
    if (!nui_canvas_input(ui, "scene", &input)) return 0;
    if ((input.flags & NUI_PRESSED) && !nui_set_value(ui, "status", "Canvas pressed")) return 0;
    return 1;
}

// END ACTIONS

#include <SDL3/SDL_opengl.h>
static const void *load_gl(void *data, const char *name) {
    (void)data;
    return (const void *)SDL_GL_GetProcAddress(name);
}

// The adapter clips this canvas painter; 0 means callback success.
static int32_t draw(void *data, void *painter, NuiString id, NuiRect bounds,
                    NuiRect content, NuiRect clip) {
    (void)data; (void)id; (void)bounds; (void)clip;
    NuiRect square = {content.x + 16, content.y + 16, 64, 64};
    NuiColor blue = {54, 95, 221, 255};
    return nui_draw_fill(painter, square, blue) == 1 ? 0 : 1;
}

int main(void) {
    int result = 1;
    SDL_Window *window = NULL;
    SDL_GLContext context = NULL;
    NuiGl *renderer = NULL;
    NuiUi *ui = nui_create(HTML, CSS, 940, 640);
    if (!ui || !SDL_Init(SDL_INIT_VIDEO)) goto error;
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);
    window = SDL_CreateWindow("Controls — C / OpenGL", 940, 640,
        SDL_WINDOW_OPENGL | SDL_WINDOW_RESIZABLE | SDL_WINDOW_HIGH_PIXEL_DENSITY);
    if (!window) goto error;
    context = SDL_GL_CreateContext(window);
    if (!context) goto error;
    SDL_GL_SetSwapInterval(1);
    renderer = nui_gl_create(load_gl, NULL);
    if (!renderer) goto error;
    const uint8_t first[] = {255,90,90,255, 70,150,255,255, 80,210,150,255, 255,210,80,255};
    const uint8_t second[] = {80,210,150,255, 255,210,80,255, 255,90,90,255, 70,150,255,255};
    if (!nui_gl_register_image(renderer, "first", 2, 2, first, sizeof first) ||
        !nui_gl_register_image(renderer, "second", 2, 2, second, sizeof second)) goto error;

    bool running = true;
    while (running) {
        SDL_Event event;
        while (SDL_PollEvent(&event)) {
            if (event.type == SDL_EVENT_QUIT) running = false;
            // Your game can inspect the same event here.
            if (!nui_sdl_window_event(ui, window, &event)) goto error;
        }
        if (!running) break;
        int width, height;
        if (!SDL_GetWindowSize(window, &width, &height)) goto error;
        if (width <= 0 || height <= 0) continue;
        if (!nui_set_viewport(ui, (float)width, (float)height)) goto error;
        int typing = nui_wants_text_input(ui);
        if (typing < 0) goto error;
        if (typing && !SDL_TextInputActive(window)) SDL_StartTextInput(window);
        if (!typing && SDL_TextInputActive(window)) SDL_StopTextInput(window);
        if (!update(ui)) goto error;
        int pw, ph;
        if (!SDL_GetWindowSizeInPixels(window, &pw, &ph)) goto error;
        glViewport(0,0,pw,ph);
        glClearColor(243.0f/255,243.0f/255,243.0f/255,1);
        glClear(GL_COLOR_BUFFER_BIT);
        // Draw your game here. UI rendering restores the host's GL state.
        if (!nui_gl_render(renderer,ui,(float)width,(float)height,(uint32_t)pw,(uint32_t)ph,draw,NULL)) goto error;
        if (!SDL_GL_SwapWindow(window)) goto error;
        if (!nui_end_frame(ui)) goto error;
    }
    result = 0;
    goto cleanup;
error:
    fprintf(stderr, "UI: %s\nSDL: %s\n", nui_last_error(), SDL_GetError());
cleanup:
    nui_gl_destroy(renderer); // Release UI textures while the host is alive.
    nui_destroy(ui);
    if (context) SDL_GL_DestroyContext(context);
    SDL_DestroyWindow(window);
    SDL_Quit();
    return result;
}
