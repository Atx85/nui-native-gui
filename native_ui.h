#ifndef NATIVE_UI_H
#define NATIVE_UI_H
#include <stddef.h>
#include <stdint.h>
#ifdef _WIN32
#define NUI_API __declspec(dllimport)
#else
#define NUI_API
#endif
#ifdef __cplusplus
extern "C" {
#endif
/* ABI version 2. Canvas callbacks now receive an additional visible clip rectangle. UTF-8, NUL-terminated input strings. Handles are thread-confined;
   SDL calls belong on the main thread. All pointers must be valid for the call.
   Never destroy handles or mutate a UI from its render callback. Read-only UI
   queries are allowed there. No callback may throw/longjmp across this interface.
   Normal operations return 1 on success, 0 on failure unless stated otherwise.
   nui_last_error is thread-local and valid until the next error on that thread. */
typedef struct NuiUi NuiUi;
typedef struct NuiSdl NuiSdl;
typedef struct { float x,y,w,h; } NuiRect;
typedef struct { uint8_t r,g,b,a; } NuiColor;
/* Borrowed UTF-8 span, NOT NUL-terminated. Valid only during the callback. */
typedef struct { const uint8_t *data; size_t len; } NuiString;
typedef struct {
    uint32_t flags;
    float x,y,press_x,press_y,release_x,release_y;
} NuiCanvasInput;
enum {
    NUI_HOVERED=1, NUI_CAPTURED=2, NUI_FOCUSED=4, NUI_PRESSED=8,
    NUI_RELEASED=16, NUI_CANCELLED=32, NUI_HAS_POINTER=64
};
enum { NUI_MOVE, NUI_DOWN, NUI_UP, NUI_LEAVE, NUI_CANCEL };
enum {
    NUI_TAB, NUI_BACK_TAB, NUI_ENTER, NUI_SPACE, NUI_LEFT, NUI_RIGHT,
    NUI_UP_KEY, NUI_DOWN_KEY, NUI_HOME, NUI_END, NUI_BACKSPACE, NUI_DELETE,
    NUI_SELECT_ALL, NUI_ESCAPE, NUI_PAGE_UP, NUI_PAGE_DOWN
};
NUI_API uint32_t nui_abi_version(void);
NUI_API const char *nui_last_error(void);
NUI_API NuiUi *nui_create(const char *html,const char *css,float width,float height);
NUI_API int32_t nui_destroy(NuiUi *ui);
NUI_API int32_t nui_set_viewport(NuiUi *ui,float width,float height);
NUI_API int32_t nui_end_frame(NuiUi *ui);
/* 0/1, -1 on error. Host combines this with its own text-input needs before
   starting/stopping platform text input; borrowed adapters never toggle it. */
NUI_API int32_t nui_wants_text_input(const NuiUi *ui);
/* Queries return 0/1, or -1 on error. Unknown click/change IDs return 0. */
NUI_API int32_t nui_clicked(const NuiUi *ui,const char *id);
NUI_API int32_t nui_changed(const NuiUi *ui,const char *id);
NUI_API int32_t nui_checked(const NuiUi *ui,const char *id);
NUI_API int32_t nui_set_checked(NuiUi *ui,const char *id,int32_t value);
NUI_API int32_t nui_set_value(NuiUi *ui,const char *id,const char *value);
/* Checkbox groups: input type=checkbox name="group" value="item".
   Values are read in HTML order, including disabled checked members. Missing
   value defaults to "on". Duplicate values are returned per checked member.
   Setter copies input, checks all matching values, unchecks the rest (including
   disabled members), and rejects unknown groups/values atomically. NULL + 0
   clears the group. Host updates do not emit changed events.
   checked_value uses nui_get_value's buffer contract; index is among checked items.
   group_changed returns 0/1, -1 for an unknown group/error. */
NUI_API int32_t nui_checked_value_count(const NuiUi *,const char *group,size_t *out);
NUI_API size_t nui_checked_value(const NuiUi *,const char *group,size_t index,char *buffer,size_t capacity);
NUI_API int32_t nui_set_checked_values(NuiUi *,const char *group,const char *const *values,size_t count);
NUI_API int32_t nui_checkbox_group_changed(const NuiUi *,const char *group);
/* Element tokens remain valid until their owning Ui is destroyed. Copy freely;
   never forge tokens, reuse them with another Ui, or use them after Ui destruction.
   No separate free is required. get reports unknown IDs as an error.
   Descriptions may be zero-initialized; only id is required. NULL label generates
   a readable label; "" hides it. All strings are copied. Only containers accept
   children. Addition failure leaves the UI unchanged. A successful insertion
   reflows layout and cancels open popups and pointer captures, like resizing. */
typedef struct { NuiUi *ui; size_t index; } NuiElement;
typedef struct {
    const char *id, *label, *class_name, *name, *value;
    int32_t checked, disabled;
} NuiCheckbox;
typedef struct { const char *id, *label, *class_name; int32_t disabled; } NuiButton;
NUI_API int32_t nui_get(NuiUi *,const char *id,NuiElement *out);
NUI_API int32_t nui_add_checkbox(NuiUi *,const NuiCheckbox *);
NUI_API int32_t nui_add_button(NuiUi *,const NuiButton *);
NUI_API int32_t nui_element_add_checkbox(NuiElement,const NuiCheckbox *);
NUI_API int32_t nui_element_add_button(NuiElement,const NuiButton *);
NUI_API int32_t nui_element_clicked(NuiElement);
NUI_API int32_t nui_element_checked(NuiElement);
NUI_API int32_t nui_element_set_checked(NuiElement,int32_t);
NUI_API size_t nui_element_get_value(NuiElement,char *buffer,size_t capacity);
NUI_API int32_t nui_element_set_value(NuiElement,const char *value);
/* Runtime select data: value/label required, class_name optional (NULL = none).
   All strings are UTF-8 C strings, copied during the call. Labels are plain text.
   Last selected flag wins; else retain the old value if enabled; else first enabled.
   Explicit selected may select a disabled item, matching HTML. Duplicate values
   are allowed; set_value uses the first match, preservation the first enabled match.
   NULL items with count=0 clears the list. No user changed event is generated.
   Replacement is atomic on error and closes this select's open popup. */
typedef struct {
    const char *value, *label, *class_name;
    int32_t disabled, selected;
} NuiSelectOption;
NUI_API int32_t nui_set_select_options(NuiUi *,const char *id,const NuiSelectOption *items,size_t count);
NUI_API int32_t nui_select_option_count(const NuiUi *,const char *id,size_t *out);
/* selected_index: 1=selected, 0=none (out untouched), -1=error. */
NUI_API int32_t nui_selected_index(const NuiUi *,const char *id,size_t *out);
/* Required buffer bytes INCLUDING NUL; 0 on error. NULL queries required size.
   An undersized buffer is left untouched; compare return value with capacity. */
NUI_API size_t nui_get_value(const NuiUi *ui,const char *id,char *buffer,size_t capacity);
/* Legacy ABI 2 spelling; new code should use nui_get_value. */
NUI_API size_t nui_value(const NuiUi *,const char *id,char *buffer,size_t capacity);
NUI_API int32_t nui_bounds(const NuiUi *ui,const char *id,NuiRect *out);
NUI_API int32_t nui_canvas_input(const NuiUi *ui,const char *id,NuiCanvasInput *out);
NUI_API int32_t nui_set_image_source(NuiUi *ui,const char *id,const char *src);
NUI_API int32_t nui_pointer(NuiUi *ui,uint32_t kind,float x,float y);
NUI_API int32_t nui_text_input(NuiUi *ui,const char *text);
NUI_API int32_t nui_scroll(NuiUi *ui,int32_t rows);
/* Positive logical-pixel deltas scroll right/down under the pointer. */
NUI_API int32_t nui_scroll_wheel(NuiUi *,float dx,float dy);
NUI_API int32_t nui_scroll_to(NuiUi *,const char *id,float x,float y);
/* out x/y are current offsets; w/h are maximum offsets. */
NUI_API int32_t nui_scroll_position(const NuiUi *,const char *id,NuiRect *out);
NUI_API int32_t nui_key(NuiUi *ui,uint32_t key,int32_t down,int32_t repeat,int32_t shift);
/* Custom engine integration: the host owns all graphics and event processing.
   Required: measure/fill/text. Optional: canvas and both image callbacks.
   Draw callbacks return 0 on success; image_size returns 1 if found, 0 if missing.
   Rectangles use logical UI coordinates except the image source (image pixels).
   The host must apply the supplied clipping rectangles and restore its state. */
typedef struct {
    int32_t (*measure)(void *,NuiString,float,float *,float *);
    int32_t (*fill)(void *,NuiRect,NuiColor);
    int32_t (*text)(void *,NuiString,float,float,float,NuiColor,NuiRect);
    /* Canvas rectangles: full bounds, full content, visible clip (ABI 2). */
    int32_t (*canvas)(void *,NuiString,NuiRect,NuiRect,NuiRect);
    int32_t (*image_size)(void *,NuiString,uint32_t *,uint32_t *);
    int32_t (*image)(void *,NuiString,NuiRect,NuiRect,NuiRect);
} NuiRenderer;
NUI_API int32_t nui_render(const NuiUi *,const NuiRenderer *,void *userdata);

/* Optional bundled SDL convenience layer. No SDL headers or separate SDL binary
   required. The app explicitly creates/destroys its window and runs the loop.
   New owned windows request display-synchronized presentation when supported.
   This adapter creates a window. Existing SDL3 games use native_ui_sdl3.h;
   other engines can use nui_render above.
   One active convenience SDL event pump per process. Do not mix this bundled SDL
   instance with a separately linked SDL instance. The raw renderer passed to the
   canvas callback belongs to this library; use the provided drawing functions.
   Nothing is cleared, polled or presented automatically by nui_sdl_render. */
/* The window-owning functions below require the standalone compatibility
   runtime (Cargo: owned-windows; CMake: native_ui::standalone). The default
   runtime has no embedded SDL/raylib. Borrowed APIs are provided separately by
   native_ui_sdl3.h, native_ui_raylib.h and nui_gl_*. Link one runtime only. */
NUI_API NuiSdl *nui_sdl_open(const char *title,uint32_t width,uint32_t height,int32_t hidden);
NUI_API int32_t nui_sdl_close(NuiSdl *sdl);
NUI_API int32_t nui_sdl_set_minimum_size(NuiSdl *,uint32_t width,uint32_t height);
/* poll: -1 error, 0 continue, 1 quit. Routes input and updates viewport on resize. */
NUI_API int32_t nui_sdl_poll(NuiSdl *sdl,NuiUi *ui);
NUI_API int32_t nui_sdl_clear(NuiSdl *sdl,NuiColor color);
NUI_API int32_t nui_sdl_present(NuiSdl *sdl);
/* Canvas callback: userdata, borrowed renderer, ID, bounds, full content, visible clip; 0=success.
   Drawing is clipped to content. Do not clear/present/destroy the renderer or
   remove its clip. Coordinates are logical UI coordinates. */
typedef int32_t (*NuiCanvasCallback)(void *,void *,NuiString,NuiRect,NuiRect,NuiRect);
NUI_API int32_t nui_sdl_render(NuiSdl *,const NuiUi *,NuiCanvasCallback,void *userdata);
NUI_API int32_t nui_sdl_register_image(NuiSdl *,const char *src,uint32_t width,uint32_t height,const uint8_t *rgba,size_t bytes);
NUI_API int32_t nui_sdl_remove_image(NuiSdl *,const char *src);
NUI_API int32_t nui_sdl_line(void *renderer,float x1,float y1,float x2,float y2,NuiColor);
NUI_API int32_t nui_sdl_fill(void *renderer,NuiRect,NuiColor);
typedef struct { uint64_t rebuilds,reuses,image_uploads; } NuiCacheStats;
NUI_API int32_t nui_sdl_cache_stats(NuiSdl *,NuiCacheStats *out);

/* Bundled raylib convenience layer. One window initialization per process;
   open, all operations and close must run on the application's main thread.
   Use poll -> clear (BeginDrawing) -> render -> present (EndDrawing) -> end_frame.
   close also finishes an open frame before releasing textures and the window.
   A callback failure leaves the frame open: present or close it to clean up.
   Callback renderer tokens are only valid during the callback on that thread;
   use nui_raylib_fill/line, never SDL helpers or a separately linked raylib.
   No separate raylib binary or headers are required for SDK consumers.
   UI paint commands are retained until invalidated; canvas callbacks run each frame. */
typedef struct NuiRaylib NuiRaylib;
NUI_API NuiRaylib *nui_raylib_open(const char *title,uint32_t width,uint32_t height,int32_t hidden);
NUI_API int32_t nui_raylib_close(NuiRaylib *);
NUI_API int32_t nui_raylib_set_minimum_size(NuiRaylib *,uint32_t width,uint32_t height);
NUI_API int32_t nui_raylib_poll(NuiRaylib *,NuiUi *);
NUI_API int32_t nui_raylib_clear(NuiRaylib *,NuiColor);
NUI_API int32_t nui_raylib_render(NuiRaylib *,const NuiUi *,NuiCanvasCallback,void *userdata);
NUI_API int32_t nui_raylib_present(NuiRaylib *);
NUI_API int32_t nui_raylib_register_image(NuiRaylib *,const char *src,uint32_t width,uint32_t height,const uint8_t *rgba,size_t bytes);
NUI_API int32_t nui_raylib_remove_image(NuiRaylib *,const char *src);
NUI_API int32_t nui_raylib_line(void *renderer,float x1,float y1,float x2,float y2,NuiColor);
NUI_API int32_t nui_raylib_fill(void *renderer,NuiRect,NuiColor);
/* Optional window helper: same UI and drawing helpers with either backend.
   SDL remains the default chosen by examples. OpenGL needs desktop GL 3.3+.
   All operations are main-thread only. One window/context owner per handle.
   Callbacks receive a borrowed NuiDraw in their void * painter argument, NOT
   an SDL_Renderer. Use nui_draw_* below, or preloaded raw GL calls on GL windows.
   Never clear/present/change contexts or disable clipping inside a callback. */
typedef struct NuiWindow NuiWindow;
enum { NUI_BACKEND_SDL=0, NUI_BACKEND_OPENGL=1 };
NUI_API NuiWindow *nui_window_open(const char *title,uint32_t width,uint32_t height,int32_t hidden,uint32_t backend);
NUI_API int32_t nui_window_close(NuiWindow *);
NUI_API int32_t nui_window_poll(NuiWindow *,NuiUi *); /* -1 error, 0 continue, 1 quit */
NUI_API int32_t nui_window_clear(NuiWindow *,NuiColor);
NUI_API int32_t nui_window_present(NuiWindow *);
NUI_API int32_t nui_window_render(NuiWindow *,const NuiUi *,NuiCanvasCallback,void *userdata);
NUI_API int32_t nui_window_register_image(NuiWindow *,const char *src,uint32_t width,uint32_t height,const uint8_t *rgba,size_t bytes);
NUI_API int32_t nui_window_remove_image(NuiWindow *,const char *src);
/* Retained paint-cache counters for either SDL or OpenGL. */
NUI_API int32_t nui_window_cache_stats(NuiWindow *,NuiCacheStats *out);
/* Load raw GL functions before rendering. Returns NULL on SDL or missing names. */
NUI_API const void *nui_window_gl_proc_address(NuiWindow *,const char *name);
/* Painter is valid only during a nui_window_render/nui_gl_render callback.
   Fill coordinates are logical UI coordinates; pixel size is the full canvas
   content viewport, unaffected by ancestor clipping. */
NUI_API int32_t nui_draw_fill(void *painter,NuiRect,NuiColor);
NUI_API int32_t nui_draw_canvas_pixel_size(void *painter,uint32_t *width,uint32_t *height);

/* Adapter for an EXISTING host-owned desktop OpenGL 3.3+ context/framebuffer.
   No SDL window, polling, clearing or presentation is performed here.
   The same context must remain current on its owning thread for every operation,
   including destruction. Loader returns valid GL function addresses by name.
   Callback viewport/scissor are prepared automatically, including HiDPI.
   Callback must keep scissor enabled and bind its own GL program/VAO. Common
   graphics state is restored even on failure; see OPENGL.md for the exact list.
   logical dimensions must match nui_set_viewport; physical dimensions describe
   the currently bound draw framebuffer. A zero physical size skips rendering. */
typedef struct NuiGl NuiGl;
typedef const void *(*NuiGlLoader)(void *userdata,const char *name);
NUI_API NuiGl *nui_gl_create(NuiGlLoader,void *userdata);
NUI_API int32_t nui_gl_destroy(NuiGl *);
NUI_API int32_t nui_gl_render(NuiGl *,const NuiUi *,float logical_width,float logical_height,uint32_t pixel_width,uint32_t pixel_height,NuiCanvasCallback,void *userdata);
NUI_API int32_t nui_gl_register_image(NuiGl *,const char *src,uint32_t width,uint32_t height,const uint8_t *rgba,size_t bytes);
NUI_API int32_t nui_gl_remove_image(NuiGl *,const char *src);
/* Rendering caches unchanged UI; native canvas callbacks remain live each frame. */
NUI_API int32_t nui_gl_cache_stats(NuiGl *,NuiCacheStats *out);
/* Clear derived UI/text caches, preserving registered images. Requires current context. */
NUI_API int32_t nui_gl_clear_caches(NuiGl *);

#ifdef __cplusplus
}
#endif
#endif
