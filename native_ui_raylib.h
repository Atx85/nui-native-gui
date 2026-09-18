#ifndef NATIVE_UI_RAYLIB_H
#define NATIVE_UI_RAYLIB_H
#include "native_ui_renderer.h"
#include <raylib.h>
#include <rlgl.h>
#include <stdlib.h>
#include <math.h>

/* Borrow the application's raylib (5.5+). Call create after InitWindow and destroy
   before CloseWindow. Render between BeginDrawing/EndDrawing in screen space,
   outside camera/shader/custom blend modes. Uses the current target and respects
   an existing scissor; never changes either. No frame begin/end, window lifetime,
   exit-key policy, input queue draining, or pacing is hidden in this adapter.
   For render textures or scaled UI use nui_renderer_create with your own scale
   callback; the default reports the window's framebuffer/logical size ratio. */
typedef NuiHostRenderer NuiRaylibRenderer;
static inline const char *nui_rl_error(void) { return "raylib texture allocation failed or window not ready"; }
static inline void *nui_rl_upload(void *host, uint32_t w, uint32_t h, const uint8_t *rgba) {
    (void)host;
    Texture2D *texture = (Texture2D *)malloc(sizeof *texture);
    if (!texture) return NULL;
    Image image = {(void *)rgba, (int)w, (int)h, 1, PIXELFORMAT_UNCOMPRESSED_R8G8B8A8};
    *texture = LoadTextureFromImage(image);
    if (!texture->id) { free(texture); return NULL; }
    SetTextureFilter(*texture, TEXTURE_FILTER_BILINEAR);
    return texture;
}
static inline void nui_rl_release(void *p) {
    Texture2D *texture = (Texture2D *)p;
    /* Cached text can be evicted mid-frame; flush draws that still reference it. */
    rlDrawRenderBatchActive();
    UnloadTexture(*texture);
    free(texture);
}
static inline int32_t nui_rl_scale(void *host, float *x, float *y) {
    (void)host;
    int w = GetScreenWidth(), h = GetScreenHeight();
    if (w <= 0 || h <= 0) return 0;
    *x = (float)GetRenderWidth() / (float)w;
    *y = (float)GetRenderHeight() / (float)h;
    return 1;
}
static inline int32_t nui_rl_fill(void *host, NuiRect r, NuiColor c) {
    (void)host;
    Rectangle rect = {r.x,r.y,r.w,r.h};
    Color color = {c.r,c.g,c.b,c.a};
    DrawRectangleRec(rect, color);
    return 1;
}
static inline int32_t nui_rl_copy(void *host, void *texture, NuiRect s, NuiRect d, NuiRect clip, NuiColor c) {
    (void)host;
    /* Crop both rectangles instead of changing raylib's scissor state. */
    float x = fmaxf(d.x,clip.x), y = fmaxf(d.y,clip.y);
    float right = fminf(d.x+d.w,clip.x+clip.w), bottom = fminf(d.y+d.h,clip.y+clip.h);
    if (d.w <= 0 || d.h <= 0 || right <= x || bottom <= y) return 1;
    Rectangle src = {s.x+(x-d.x)*s.w/d.w, s.y+(y-d.y)*s.h/d.h,
                     (right-x)*s.w/d.w, (bottom-y)*s.h/d.h};
    Rectangle dst = {x,y,right-x,bottom-y};
    Vector2 origin = {0,0};
    Color color = {c.r,c.g,c.b,c.a};
    DrawTexturePro(*(Texture2D *)texture,src,dst,origin,0,color);
    return 1;
}
static inline NuiRaylibRenderer *nui_raylib_create(void) {
    const NuiRendererHost host = {1,nui_rl_upload,nui_rl_release,nui_rl_fill,nui_rl_copy,nui_rl_scale,nui_rl_error};
    /* The bridge needs a non-null identity; no SDK code dereferences this token. */
    static int identity;
    if (!IsWindowReady()) { nui_sdl_host_error("create raylib adapter after InitWindow"); return NULL; }
    return nui_renderer_create(&identity,&host);
}
static inline int32_t nui_raylib_destroy(NuiRaylibRenderer *renderer) { return nui_renderer_destroy(renderer); }
static inline int32_t nui_raylib_draw(NuiRaylibRenderer *renderer, const NuiUi *ui) { return nui_renderer_render(renderer,ui,NULL,NULL); }

/* Forward key/mouse snapshot once per host frame. Does not consume key or text
   queues. The host can still read the same input and decides which system gets
   text: call GetCharPressed yourself and forward chosen codepoints below.
   Set viewport explicitly when your logical UI size changes. */
static inline int32_t nui_raylib_input(NuiUi *ui) {
    if (!IsWindowFocused()) return nui_pointer(ui,NUI_CANCEL,0,0);
    Vector2 p = GetMousePosition();
    if (IsCursorOnScreen() || IsMouseButtonDown(MOUSE_BUTTON_LEFT)) {
        if (!nui_pointer(ui,NUI_MOVE,p.x,p.y)) return 0;
    }
    if (!IsCursorOnScreen() && !nui_pointer(ui,NUI_LEAVE,0,0)) return 0;
    if (IsMouseButtonPressed(MOUSE_BUTTON_LEFT) && !nui_pointer(ui,NUI_DOWN,p.x,p.y)) return 0;
    if (IsMouseButtonReleased(MOUSE_BUTTON_LEFT) && !nui_pointer(ui,NUI_UP,p.x,p.y)) return 0;
    Vector2 wheel = GetMouseWheelMoveV();
    if ((wheel.x || wheel.y) && !nui_scroll_wheel(ui,-wheel.x*40,-wheel.y*40)) return 0;
    int shift = IsKeyDown(KEY_LEFT_SHIFT) || IsKeyDown(KEY_RIGHT_SHIFT);
    int shortcut = IsKeyDown(KEY_LEFT_CONTROL) || IsKeyDown(KEY_RIGHT_CONTROL) ||
                   IsKeyDown(KEY_LEFT_SUPER) || IsKeyDown(KEY_RIGHT_SUPER);
    const int native[] = {KEY_TAB,KEY_ENTER,KEY_KP_ENTER,KEY_SPACE,KEY_LEFT,KEY_RIGHT,KEY_UP,KEY_DOWN,
        KEY_HOME,KEY_END,KEY_BACKSPACE,KEY_DELETE,KEY_ESCAPE,KEY_PAGE_UP,KEY_PAGE_DOWN,KEY_A};
    const uint32_t keys[] = {(uint32_t)(shift?NUI_BACK_TAB:NUI_TAB),NUI_ENTER,NUI_ENTER,NUI_SPACE,NUI_LEFT,NUI_RIGHT,
        NUI_UP_KEY,NUI_DOWN_KEY,NUI_HOME,NUI_END,NUI_BACKSPACE,NUI_DELETE,NUI_ESCAPE,NUI_PAGE_UP,NUI_PAGE_DOWN,NUI_SELECT_ALL};
    for (size_t i=0;i<sizeof native/sizeof native[0];i++) {
        if (native[i] == KEY_A && !shortcut) continue;
        int pressed = IsKeyPressed(native[i]), repeat = IsKeyPressedRepeat(native[i]);
        if ((pressed || repeat) && !nui_key(ui,keys[i],1,!pressed && repeat,shift)) return 0;
        if (IsKeyReleased(native[i]) && !nui_key(ui,keys[i],0,0,shift)) return 0;
    }
    return 1;
}
/* raylib owns the temporary UTF-8 buffer; nui_text_input copies it immediately. */
static inline int32_t nui_raylib_text(NuiUi *ui, int codepoint) {
    if (codepoint < 32 || codepoint == 127) return 1;
    if (codepoint > 0x10ffff || (codepoint >= 0xd800 && codepoint <= 0xdfff))
        return nui_sdl_host_error("invalid Unicode codepoint");
    int bytes;
    const char *text = CodepointToUTF8(codepoint,&bytes);
    char utf8[5] = {0};
    for (int i=0;i<bytes;i++) utf8[i]=text[i];
    return nui_text_input(ui,utf8);
}
#endif
