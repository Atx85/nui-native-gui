// Package support contains C ABI bindings; each example owns its frame loop.
package support

/*
#include "native_ui.h"
#include <stdlib.h>
extern int32_t exampleDraw(void*,void*,NuiString,NuiRect,NuiRect,NuiRect);
static inline int32_t render(NuiWindow *w,NuiUi *ui,int canvas) {
	return nui_window_render(w,ui,canvas ? exampleDraw : NULL,NULL);
}
*/
import "C"
import (
	"runtime"
	"strings"
	"unsafe"
)

func init() { runtime.LockOSThread() }
func c(s string) (*C.char, func()) {
	if strings.ContainsRune(s, 0) {
		panic("NUL in string")
	}
	p := C.CString(s)
	return p, func() { C.free(unsafe.Pointer(p)) }
}
func ok(n C.int32_t) {
	if n != 1 {
		panic(C.GoString(C.nui_last_error()))
	}
}
func flag(n C.int32_t) bool {
	if n < 0 {
		panic(C.GoString(C.nui_last_error()))
	}
	return n == 1
}
func Require(v bool) {
	if !v {
		panic("Element check failed")
	}
}

type Ui struct{ p *C.NuiUi }
type Window struct{ p *C.NuiWindow }
type Renderer struct{ p unsafe.Pointer }
type Rect struct{ X, Y, W, H float32 }
type Color struct{ R, G, B, A uint8 }

func rect(r C.NuiRect) Rect          { return Rect{float32(r.x), float32(r.y), float32(r.w), float32(r.h)} }
func (u *Ui) Clicked(id string) bool { p, f := c(id); defer f(); return flag(C.nui_clicked(u.p, p)) }
func (u *Ui) Changed(id string) bool { p, f := c(id); defer f(); return flag(C.nui_changed(u.p, p)) }
func (u *Ui) Checked(id string) bool { p, f := c(id); defer f(); return flag(C.nui_checked(u.p, p)) }
func (u *Ui) GroupChanged(id string) bool {
	p, f := c(id)
	defer f()
	return flag(C.nui_checkbox_group_changed(u.p, p))
}
func (u *Ui) GroupCount(id string) int {
	p, f := c(id)
	defer f()
	var n C.size_t
	ok(C.nui_checked_value_count(u.p, p, &n))
	return int(n)
}
func (u *Ui) Value(id string) string {
	p, f := c(id)
	defer f()
	n := C.nui_get_value(u.p, p, nil, 0)
	Require(n > 0)
	b := C.malloc(n)
	Require(b != nil)
	defer C.free(b)
	Require(C.nui_get_value(u.p, p, (*C.char)(b), n) == n)
	return C.GoString((*C.char)(b))
}
func (u *Ui) SetChecked(id string, checked bool) {
    p, free := c(id); defer free()
    var value C.int32_t
    if checked { value = 1 }
    ok(C.nui_set_checked(u.p, p, value))
}
func (u *Ui) SetCheckedValues(id string, values []string) {
    p, free := c(id); defer free()
    // C storage avoids passing a Go pointer-containing slice through cgo.
    memory := C.malloc(C.size_t(len(values)) * C.size_t(unsafe.Sizeof(uintptr(0))))
    if len(values) > 0 { Require(memory != nil) }
    defer C.free(memory)
    pointers := unsafe.Slice((**C.char)(memory), len(values))
    for i, value := range values {
        text, release := c(value); defer release(); pointers[i] = text
    }
    ok(C.nui_set_checked_values(u.p, p, (**C.char)(memory), C.size_t(len(values))))
}
func (u *Ui) ReplaceOptions(id string, options [][2]string) {
    p, free := c(id); defer free()
    memory := C.calloc(C.size_t(len(options)), C.size_t(unsafe.Sizeof(C.NuiSelectOption{})))
    if len(options) > 0 { Require(memory != nil) }
    defer C.free(memory)
    items := unsafe.Slice((*C.NuiSelectOption)(memory), len(options))
    for i, option := range options {
        value, a := c(option[0]); defer a()
        label, b := c(option[1]); defer b()
        items[i].value = value; items[i].label = label
        if i == 0 { items[i].selected = 1 }
    }
    ok(C.nui_set_select_options(u.p, p, (*C.NuiSelectOption)(memory), C.size_t(len(options))))
}
func (u *Ui) SetValue(id, value string) {
	p, f := c(id)
	defer f()
	v, g := c(value)
	defer g()
	ok(C.nui_set_value(u.p, p, v))
}
func (u *Ui) ImageSource(id, value string) {
	p, f := c(id)
	defer f()
	v, g := c(value)
	defer g()
	ok(C.nui_set_image_source(u.p, p, v))
}
func (u *Ui) Bounds(id string) Rect {
	p, f := c(id)
	defer f()
	var r C.NuiRect
	ok(C.nui_bounds(u.p, p, &r))
	return rect(r)
}
func (u *Ui) Pressed(id string) bool {
	p, f := c(id)
	defer f()
	var r C.NuiCanvasInput
	ok(C.nui_canvas_input(u.p, p, &r))
	return r.flags&C.NUI_PRESSED != 0
}
func (u *Ui) PointerDown(x, y float32) {
	ok(C.nui_pointer(u.p, C.NUI_DOWN, C.float(x), C.float(y)))
}
func (u *Ui) Click(id string) {
	r := u.Bounds(id)
	u.PointerDown(r.X+r.W/2, r.Y+r.H/2)
	ok(C.nui_pointer(u.p, C.NUI_UP, C.float(r.X+r.W/2), C.float(r.Y+r.H/2)))
}
func (u *Ui) TextInput(s string)    { p, f := c(s); defer f(); ok(C.nui_text_input(u.p, p)) }
func (u *Ui) Viewport(w, h float32) { ok(C.nui_set_viewport(u.p, C.float(w), C.float(h))) }
func (u *Ui) ScrollTo(id string, y float32) {
	p, f := c(id)
	defer f()
	ok(C.nui_scroll_to(u.p, p, 0, C.float(y)))
}
func (u *Ui) ScrollPosition(id string) Rect {
	p, f := c(id)
	defer f()
	var r C.NuiRect
	ok(C.nui_scroll_position(u.p, p, &r))
	return rect(r)
}
func (w *Window) Image(id string, width, height uint32, pixels []byte) {
	Require(len(pixels) > 0)
	p, f := c(id)
	defer f()
	ok(C.nui_window_register_image(w.p, p, C.uint32_t(width), C.uint32_t(height), (*C.uint8_t)(unsafe.Pointer(&pixels[0])), C.size_t(len(pixels))))
}
func (r *Renderer) Fill(b Rect, color Color) {
	ok(C.nui_draw_fill(r.p, C.NuiRect{x: C.float(b.X), y: C.float(b.Y), w: C.float(b.W), h: C.float(b.H)}, C.NuiColor{r: C.uint8_t(color.R), g: C.uint8_t(color.G), b: C.uint8_t(color.B), a: C.uint8_t(color.A)}))
}

var activeDraw func(*Renderer, Rect)

//export exampleDraw
func exampleDraw(userdata unsafe.Pointer, renderer unsafe.Pointer, id C.NuiString, bounds C.NuiRect, content C.NuiRect, clip C.NuiRect) (result C.int32_t) {
	// A panic must never unwind through C. No Go pointer is retained by the library.
	defer func() {
		if recover() != nil {
			result = 1
		}
	}()
	activeDraw(&Renderer{renderer}, rect(content))
	return 0
}
func NewUi(markup, styles string, width, height int) *Ui {
	Require(C.nui_abi_version() == 2)
	html, a := c(markup)
	defer a()
	css, b := c(baseCSS + styles)
	defer b()
	ui := &Ui{C.nui_create(html, css, C.float(width), C.float(height))}
	Require(ui.p != nil)
	return ui
}
func (u *Ui) Close() { ok(C.nui_destroy(u.p)) }
func (u *Ui) EndFrame() { ok(C.nui_end_frame(u.p)) }
func OpenWindow(name string, width, height int, smoke bool) *Window {
	title, free := c(name)
	defer free()
	var hidden C.int32_t
	if smoke {
		hidden = 1
	}
	backend := C.uint32_t(C.NUI_BACKEND_SDL)
	window := &Window{C.nui_window_open(title, C.uint32_t(width), C.uint32_t(height), hidden, backend)}
	Require(window.p != nil)
	return window
}
func (w *Window) Close() { ok(C.nui_window_close(w.p)) }
func (w *Window) Poll(ui *Ui) bool {
	event := C.nui_window_poll(w.p, ui.p)
	Require(event >= 0)
	return event == 0
}
func (w *Window) Clear(color Color) {
	ok(C.nui_window_clear(w.p, C.NuiColor{r: C.uint8_t(color.R), g: C.uint8_t(color.G), b: C.uint8_t(color.B), a: C.uint8_t(color.A)}))
}
func (w *Window) Render(ui *Ui, draw func(*Renderer, Rect)) {
	activeDraw = draw
	defer func() { activeDraw = nil }()
	var canvas C.int
	if draw != nil {
		canvas = 1
	}
	ok(C.render(w.p, ui.p, canvas))
}
func (w *Window) CheckCache() {
	var stats C.NuiCacheStats
	ok(C.nui_window_cache_stats(w.p, &stats))
	Require(stats.reuses > 0)
}
func (w *Window) Present() { ok(C.nui_window_present(w.p)) }

// Change themeMACOS to themeWINDOWS_11 or themeWINDOWS_XP to switch skins.
// themes.go is generated from the shared theme CSS; the rest is layout only.
const baseCSS = themeMACOS
