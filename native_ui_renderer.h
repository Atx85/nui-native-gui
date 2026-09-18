#ifndef NATIVE_UI_RENDERER_H
#define NATIVE_UI_RENDERER_H
#include "native_ui.h"
#ifdef __cplusplus
extern "C" {
#endif
/* Shared host-renderer bridge. Framework headers supply these callbacks.
   Main thread only. Destroy before the host renderer/context. No window, input
   polling, frame timing, clear or present operations are performed here.
   Callbacks return 1 on success. Their code must outlive the adapter, must not
   throw/longjmp, and must not mutate/destroy the UI or adapter during rendering. */
typedef struct NuiSdlRenderer NuiHostRenderer;
typedef struct NuiSdlRenderer NuiSdlRenderer;
typedef struct {
    uint32_t version;
    void *(*upload)(void *, uint32_t, uint32_t, const uint8_t *);
    void (*release)(void *);
    int32_t (*fill)(void *, NuiRect, NuiColor);
    int32_t (*copy)(void *, void *, NuiRect, NuiRect, NuiRect, NuiColor);
    int32_t (*scale)(void *, float *, float *);
    const char *(*last_error)(void);
} NuiSdlHost;
NUI_API NuiSdlRenderer *nui_sdl_create_host(void *, const NuiSdlHost *);
NUI_API void *nui_sdl_native_renderer(const NuiSdlRenderer *);
NUI_API int32_t nui_sdl_host_error(const char *);


NUI_API int32_t nui_sdl_destroy(NuiSdlRenderer *);
NUI_API int32_t nui_sdl_invalidate(NuiSdlRenderer *);
NUI_API int32_t nui_sdl_renderer_register_image(NuiSdlRenderer *, const char *, uint32_t, uint32_t, const uint8_t *, size_t);
NUI_API int32_t nui_sdl_renderer_remove_image(NuiSdlRenderer *, const char *);
/* Canvas callbacks return 0 on success. Their painter is a temporary token,
   usable only with nui_renderer_fill; it is not a native renderer pointer. */
NUI_API int32_t nui_renderer_render(NuiHostRenderer *, const NuiUi *, NuiCanvasCallback, void *);
NUI_API int32_t nui_renderer_fill(void *, NuiRect, NuiColor);
typedef NuiSdlHost NuiRendererHost;
static inline NuiHostRenderer *nui_renderer_create(void *host, const NuiRendererHost *ops) { return nui_sdl_create_host(host, ops); }
static inline int32_t nui_renderer_destroy(NuiHostRenderer *r) { return nui_sdl_destroy(r); }
static inline int32_t nui_renderer_invalidate(NuiHostRenderer *r) { return nui_sdl_invalidate(r); }
static inline int32_t nui_renderer_register_image(NuiHostRenderer *r, const char *src, uint32_t w, uint32_t h, const uint8_t *p, size_t n) { return nui_sdl_renderer_register_image(r,src,w,h,p,n); }
static inline int32_t nui_renderer_remove_image(NuiHostRenderer *r, const char *src) { return nui_sdl_renderer_remove_image(r,src); }
#ifdef __cplusplus
}
#endif
#endif
