#ifndef NATIVE_UI_SDL3_H
#define NATIVE_UI_SDL3_H
#include "native_ui_renderer.h"
#include <SDL3/SDL.h>

/* Borrowed SDL3 renderer adapter (SDL 3.2+).
   All calls belong on the main thread. The renderer must outlive this adapter.
   No SDL init/quit, event polling, text-input toggling, clearing or presentation.
   Drawing uses the current target, viewport, scale and clip and restores state.
   nui_sdl_event returns 1 on success (including ignored events), 0 on error;
   it never consumes/removes/modifies an event. The host chooses input routing.
   Call nui_set_viewport with your UI's logical dimensions when those change.
   Destroy the adapter before destroying the SDL renderer or unloading this code.
   NULL destroy is safe; otherwise handles must be live and destroyed only once. */
#ifdef __cplusplus
extern "C" {
#endif
typedef struct NuiSdlRenderer NuiSdlRenderer;
NUI_API int32_t nui_sdl_destroy(NuiSdlRenderer *);
NUI_API int32_t nui_sdl_draw(NuiSdlRenderer *, const NuiUi *);
/* Discard GPU caches after a host device reset. Registered image pixels survive. */
NUI_API int32_t nui_sdl_invalidate(NuiSdlRenderer *);
NUI_API int32_t nui_sdl_renderer_register_image(NuiSdlRenderer *, const char *src,
    uint32_t width, uint32_t height, const uint8_t *rgba, size_t bytes);
NUI_API int32_t nui_sdl_renderer_remove_image(NuiSdlRenderer *, const char *src);

/* Bridge ABI for language bindings. The inline helpers below supply it for C/C++.
   Callbacks return 1 on success; the table is copied on create. Callbacks and
   borrowed renderer must stay valid until destroy. Never mutate/destroy handles
   or throw/longjmp from callbacks. These callbacks call the HOST's SDL instance,
   so host pointers are used only by the host's SDL. The default SDK embeds no SDL. No user callbacks
   are needed to use nui_sdl_create/event/draw/destroy. */
static inline void *nui_sdl3_upload(void *p, uint32_t w, uint32_t h, const uint8_t *rgba) {
    SDL_Texture *t = SDL_CreateTexture((SDL_Renderer *)p, SDL_PIXELFORMAT_RGBA32,
                                       SDL_TEXTUREACCESS_STATIC, (int)w, (int)h);
    if (!t) return NULL;
    if (!SDL_UpdateTexture(t, NULL, rgba, (int)w * 4) ||
        !SDL_SetTextureBlendMode(t, SDL_BLENDMODE_BLEND) ||
        !SDL_SetTextureScaleMode(t, SDL_SCALEMODE_LINEAR)) {
        SDL_DestroyTexture(t);
        return NULL;
    }
    return t;
}
static inline void nui_sdl3_release(void *p) { SDL_DestroyTexture((SDL_Texture *)p); }
static inline const char *nui_sdl3_error(void) { return SDL_GetError(); }
static inline int32_t nui_sdl3_scale(void *p, float *x, float *y) {
    SDL_Renderer *r = (SDL_Renderer *)p;
    int w, h;
    SDL_RendererLogicalPresentation mode;
    if (!SDL_GetRenderScale(r, x, y) ||
        !SDL_GetRenderLogicalPresentation(r, &w, &h, &mode)) return 0;
    if (mode != SDL_LOGICAL_PRESENTATION_DISABLED && w > 0 && h > 0) {
        SDL_FRect output;
        if (!SDL_GetRenderLogicalPresentationRect(r, &output)) return 0;
        /* Rasterize text at the actual pixel density, including letterboxing. */
        if (output.w > 0 && output.h > 0) {
            *x *= output.w / (float)w;
            *y *= output.h / (float)h;
        }
    }
    return 1;
}
static inline int32_t nui_sdl3_fill(void *p, NuiRect rect, NuiColor c) {
    SDL_Renderer *r = (SDL_Renderer *)p;
    float red, green, blue, alpha;
    SDL_BlendMode blend;
    if (!SDL_GetRenderDrawColorFloat(r, &red, &green, &blue, &alpha) ||
        !SDL_GetRenderDrawBlendMode(r, &blend)) return 0;
    SDL_FRect dst = {rect.x, rect.y, rect.w, rect.h};
    int ok = SDL_SetRenderDrawBlendMode(r, SDL_BLENDMODE_BLEND) &&
             SDL_SetRenderDrawColor(r, c.r, c.g, c.b, c.a) && SDL_RenderFillRect(r, &dst);
    /* Restore both even when the drawing operation failed. */
    int color_ok = SDL_SetRenderDrawColorFloat(r, red, green, blue, alpha);
    int blend_ok = SDL_SetRenderDrawBlendMode(r, blend);
    return ok && color_ok && blend_ok;
}
static inline int32_t nui_sdl3_copy(void *p, void *texture, NuiRect source,
                                   NuiRect dest, NuiRect clipping, NuiColor c) {
    SDL_Renderer *r = (SDL_Renderer *)p;
    SDL_Texture *t = (SDL_Texture *)texture;
    SDL_Rect previous;
    int clipped = SDL_RenderClipEnabled(r);
    if (clipped && !SDL_GetRenderClipRect(r, &previous)) return 0;
    /* Round inward, safely bounding casts even for very large layout coordinates. */
    int x = (int)SDL_clamp(SDL_ceilf(clipping.x), -1000000000.0f, 1000000000.0f);
    int y = (int)SDL_clamp(SDL_ceilf(clipping.y), -1000000000.0f, 1000000000.0f);
    int right = (int)SDL_clamp(SDL_floorf(clipping.x + clipping.w), -1000000000.0f, 1000000000.0f);
    int bottom = (int)SDL_clamp(SDL_floorf(clipping.y + clipping.h), -1000000000.0f, 1000000000.0f);
    if (right <= x || bottom <= y) return 1;
    SDL_Rect clip = {x, y, right - x, bottom - y};
    if (clipped) {
        SDL_Rect intersection;
        if (!SDL_GetRectIntersection(&previous, &clip, &intersection)) return 1;
        clip = intersection;
    }
    SDL_FRect src = {source.x, source.y, source.w, source.h};
    SDL_FRect dst = {dest.x, dest.y, dest.w, dest.h};
    int ok = SDL_SetRenderClipRect(r, &clip) &&
             SDL_SetTextureColorMod(t, c.r, c.g, c.b) && SDL_SetTextureAlphaMod(t, c.a) &&
             SDL_RenderTexture(r, t, &src, &dst);
    int restored = SDL_SetRenderClipRect(r, clipped ? &previous : NULL);
    return ok && restored;
}
static inline NuiSdlRenderer *nui_sdl_create(SDL_Renderer *renderer) {
    const NuiSdlHost host = {1, nui_sdl3_upload, nui_sdl3_release, nui_sdl3_fill,
                            nui_sdl3_copy, nui_sdl3_scale, nui_sdl3_error};
    return nui_sdl_create_host(renderer, &host);
}
static inline int nui_sdl3_key(SDL_Keycode key, SDL_Keymod modifiers) {
    switch (key) {
    case SDLK_TAB: return (modifiers & SDL_KMOD_SHIFT) ? NUI_BACK_TAB : NUI_TAB;
    case SDLK_RETURN: case SDLK_KP_ENTER: return NUI_ENTER;
    case SDLK_SPACE: return NUI_SPACE;
    case SDLK_LEFT: return NUI_LEFT;
    case SDLK_RIGHT: return NUI_RIGHT;
    case SDLK_UP: return NUI_UP_KEY;
    case SDLK_DOWN: return NUI_DOWN_KEY;
    case SDLK_HOME: return NUI_HOME;
    case SDLK_END: return NUI_END;
    case SDLK_BACKSPACE: return NUI_BACKSPACE;
    case SDLK_DELETE: return NUI_DELETE;
    case SDLK_ESCAPE: return NUI_ESCAPE;
    case SDLK_PAGEUP: return NUI_PAGE_UP;
    case SDLK_PAGEDOWN: return NUI_PAGE_DOWN;
    case SDLK_A: return (modifiers & (SDL_KMOD_CTRL | SDL_KMOD_GUI)) ? NUI_SELECT_ALL : -1;
    default: return -1;
    }
}
static inline bool nui_sdl3_coordinates(SDL_Renderer *r, float wx, float wy, float *x, float *y) {
    if (r) return SDL_RenderCoordinatesFromWindow(r,wx,wy,x,y);
    *x=wx; *y=wy; return true;
}
static inline int32_t nui_sdl_forward(NuiSdlRenderer *adapter, SDL_Renderer *renderer, SDL_Window *window, NuiUi *ui, const SDL_Event *event) {
    if (nui_wants_text_input(ui) < 0) return 0;
    if (!event) return nui_sdl_host_error("null SDL event");
    if (event->type == SDL_EVENT_RENDER_DEVICE_RESET || event->type == SDL_EVENT_RENDER_TARGETS_RESET) {
        if (adapter && (event->render.windowID == 0 || (window && event->render.windowID == SDL_GetWindowID(window))))
            return nui_sdl_invalidate(adapter);
        return 1;
    }
    /* No global input, quit handling, window closing, or application policy. */
    if (!window || SDL_GetWindowFromEvent(event) != window) return 1;
    float x, y;
    switch (event->type) {
    case SDL_EVENT_MOUSE_MOTION:
        if (!nui_sdl3_coordinates(renderer, event->motion.x, event->motion.y, &x, &y))
            return nui_sdl_host_error(SDL_GetError());
        return nui_pointer(ui, NUI_MOVE, x, y);
    case SDL_EVENT_MOUSE_BUTTON_DOWN: case SDL_EVENT_MOUSE_BUTTON_UP:
        if (event->button.button != SDL_BUTTON_LEFT) return 1;
        if (!nui_sdl3_coordinates(renderer, event->button.x, event->button.y, &x, &y))
            return nui_sdl_host_error(SDL_GetError());
        return nui_pointer(ui, event->type == SDL_EVENT_MOUSE_BUTTON_DOWN ? NUI_DOWN : NUI_UP, x, y);
    case SDL_EVENT_MOUSE_WHEEL: {
        if (!nui_sdl3_coordinates(renderer, event->wheel.mouse_x, event->wheel.mouse_y, &x, &y))
            return nui_sdl_host_error(SDL_GetError());
        if (!nui_pointer(ui, NUI_MOVE, x, y)) return 0;
        float sign = event->wheel.direction == SDL_MOUSEWHEEL_FLIPPED ? 40.0f : -40.0f;
        return nui_scroll_wheel(ui, event->wheel.x * sign, event->wheel.y * sign);
    }
    case SDL_EVENT_TEXT_INPUT: return nui_text_input(ui, event->text.text);
    case SDL_EVENT_WINDOW_FOCUS_LOST: return nui_pointer(ui, NUI_CANCEL, 0, 0);
    case SDL_EVENT_WINDOW_MOUSE_LEAVE: return nui_pointer(ui, NUI_LEAVE, 0, 0);
    case SDL_EVENT_KEY_DOWN: case SDL_EVENT_KEY_UP: {
        int key = nui_sdl3_key(event->key.key, event->key.mod);
        return key < 0 ? 1 : nui_key(ui, (uint32_t)key, event->type == SDL_EVENT_KEY_DOWN,
                                   event->key.repeat, (event->key.mod & SDL_KMOD_SHIFT) != 0);
    }
    default: return 1;
    }
}
/* For a host-owned OpenGL window, events use window logical coordinates. */
static inline int32_t nui_sdl_window_event(NuiUi *ui, SDL_Window *window, const SDL_Event *event) {
    return nui_sdl_forward(NULL,NULL,window,ui,event);
}
static inline int32_t nui_sdl_event(NuiSdlRenderer *adapter, NuiUi *ui, const SDL_Event *event) {
    SDL_Renderer *renderer = (SDL_Renderer *)nui_sdl_native_renderer(adapter);
    if (!renderer) return 0;
    return nui_sdl_forward(adapter,renderer,SDL_GetRenderWindow(renderer),ui,event);
}
#ifdef __cplusplus
}
#endif
#endif
