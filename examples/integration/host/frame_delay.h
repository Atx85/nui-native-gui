#ifndef UI_HOST_FRAME_DELAY_H
#define UI_HOST_FRAME_DELAY_H
/* Optional pacing for these small examples; use your own frame scheduler in an app.
   On Linux define _DEFAULT_SOURCE before including any system headers. */
#ifdef _WIN32
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#else
#include <unistd.h>
#endif
static inline void ui_host_wait(void) {
#ifdef _WIN32
    Sleep(16);
#else
    usleep(16000);
#endif
}
#endif
