# Controls: events, values, and updates (C)

Start with one complete [control showcase](../examples/controls/README.md), then
copy the small recipe you need below. Each showcase includes the markup, layout,
macOS theme, event handling, application updates, images, and native drawing.
No custom `run`, `require`, `value`, or synthetic-click helpers are needed in C.

The snippets below go inside your existing application flow and assume a valid
`NuiUi *ui`. Include `native_ui.h`; use the full examples for setup, cleanup, and
error paths. These are the actual current API names, not proposed adapters.

## Frame and event rules

1. Feed host input to the UI before reading its actions. For a game-owned SDL3
   renderer, forward each `SDL_Event` through `nui_sdl_event(overlay, ui, &event)`.
2. Read clicks/changes and run your application logic.
3. Draw the UI and present through your host renderer.
4. Call `nui_end_frame(ui)` once after all input consumers finish.

`nui_clicked` and `nui_changed` are flags for the current frame. Reading them does
not consume them. They clear at `nui_end_frame`; values and checkbox selections
persist. Application setters do **not** emit user-change events. If your reset
button sets a field and a status label, update both explicitly.

Most setters return **1 on success, 0 on error**. Click/change/checked queries
return **0 or 1, -1 on error**. Use `nui_last_error()` for the error message.
Never interpret a negative query result as a true event. The header documents
exceptions such as string lengths, selected-index results, and draw callbacks.

## Button

```html
<button id="save" class="primary">Save</button>
<button disabled>Unavailable</button>
```

```c
int clicked = nui_clicked(ui, "save");
if (clicked == 1) {
    puts("Save requested");
}
```

`class="primary"` uses the theme's accent button. `disabled` is a markup
attribute; there is currently no public C setter for a button's disabled state
or label. Do not invent an update function for unsupported operations.

## Text input

```html
<label for="name">Name</label>
<input id="name" value="Ada" maxlength="40" placeholder="Your name">
<input id="echo" value="Ada" readonly>
```

```c
if (nui_changed(ui, "name") == 1) {
    char name[256]; // maxlength=40; also accommodates multibyte UTF-8.
    size_t needed = nui_get_value(ui, "name", name, sizeof name);
    if (needed > 0 && needed <= sizeof name) {
        nui_set_value(ui, "echo", name);
    }
}

// An application update; it does not set nui_changed(ui, "name").
nui_set_value(ui, "name", "Grace");
```

`nui_get_value` returns the required bytes **including the terminating NUL**;
zero means error. An undersized buffer is left untouched. For unbounded text,
query with `nui_get_value(ui, "name", NULL, 0)`, allocate that many bytes, read,
and free the buffer after use. All operations on a UI stay on its owning thread.
The library copies text supplied to setters, so it does not retain your buffer.

With borrowed SDL3 integration, your host also owns text-input activation. Use
`nui_wants_text_input(ui)` together with your game's text-entry needs to decide
when to call `SDL_StartTextInput(window)` / `SDL_StopTextInput(window)`.
Forward text-input events as well as pointer and keyboard events. The adapter
does not take over your text-input session.

## Checkbox

```html
<input id="sound" type="checkbox" checked>
<label for="sound">Enable sound</label>
```

```c
if (nui_changed(ui, "sound") == 1) {
    int checked = nui_checked(ui, "sound");
    if (checked >= 0) printf("Sound: %s\n", checked ? "on" : "off");
}
nui_set_checked(ui, "sound", 0); // Application turns it off.
```

Read the checked state with `nui_checked`; the checkbox's string `value` is a
separate submission value, not its on/off state. Clicking its associated label
also toggles it.

## Checkbox group

```html
<input id="png" name="formats" type="checkbox" value="png" checked>
<label for="png">PNG</label>
<input id="svg" name="formats" type="checkbox" value="svg">
<label for="svg">SVG</label>
```

```c
if (nui_checkbox_group_changed(ui, "formats") == 1) {
    size_t count;
    if (nui_checked_value_count(ui, "formats", &count)) {
        for (size_t i = 0; i < count; ++i) {
            char format[16];
            size_t needed = nui_checked_value(ui, "formats", i,
                                              format, sizeof format);
            if (needed > 0 && needed <= sizeof format) puts(format);
        }
    }
}
const char *selected[] = {"png", "svg"};
nui_set_checked_values(ui, "formats", selected, 2);
nui_set_checked_values(ui, "formats", NULL, 0); // Clear all selections.
```

The shared `name` identifies the group; individual `id`s identify its controls.
Selected strings are returned in HTML order. Unknown requested values are
rejected atomically. Changing the group from code does not emit a group-change
flag. This is multiple checkbox selection, not a multi-select dropdown.

## Slider

```html
<label for="volume">Volume</label>
<input id="volume" type="range" min="0" max="100" step="5" value="55">
```

```c
if (nui_changed(ui, "volume") == 1) {
    char text[32];
    size_t needed = nui_get_value(ui, "volume", text, sizeof text);
    if (needed > 0 && needed <= sizeof text) {
        float volume = strtof(text, NULL); // <stdlib.h>
        printf("Volume: %.0f\n", volume);
    }
}
nui_set_value(ui, "volume", "75");
```

Values use the same string API as other inputs. The library validates numeric
values, clamps them to the range, and applies the step. Arrow keys and Home/End
work when focused. Negative bounds and fractional steps are supported too.

## Select: read selection and replace options

```html
<select id="quality">
  <option value="normal" selected>Normal</option>
  <option value="high">High</option>
</select>
```

```c
if (nui_changed(ui, "quality") == 1) {
    char quality[32];
    size_t needed = nui_get_value(ui, "quality", quality, sizeof quality);
    if (needed > 0 && needed <= sizeof quality) puts(quality);
}
nui_set_value(ui, "quality", "high");

const NuiSelectOption options[] = {
    {"normal", "Balanced", NULL, 0, 1}, // value, label, class, disabled, selected
    {"high",   "Best quality", NULL, 0, 0},
    {"draft",  "Draft", NULL, 0, 0},
};
nui_set_select_options(ui, "quality", options, 3);
```

The value is the machine-readable option value, not its displayed label.
Replacement copies all strings; your array can be temporary. Explicit selection
wins; otherwise the library preserves the old value when possible, then selects
the first enabled option. Replacement closes an open popup and emits no user
change event. Pass `NULL, 0` to clear options. `nui_selected_index` returns 0 when
there is no selection; check this before interpreting a selected value.

## Labels

```html
<label for="name">Player name</label>
<label class="caption">Changes apply immediately.</label>
```

Labels display text and can focus or activate their associated control. They do
not have an independent changed event or a public text-update setter. For a
changing status field, use `<input id="status" readonly>` and
`nui_set_value(ui, "status", "Saved")`, as the showcase does.

## Images

```html
<img id="picture" src="first" alt="Preview" style="width:80px;height:60px">
```

For a borrowed SDL3 or raylib renderer (include its public adapter header):

```c
const uint8_t pixels[] = {255, 90, 90, 255, 70, 150, 255, 255};
nui_renderer_register_image(renderer, "first", 2, 1, pixels, sizeof pixels);
// Register a second asset from your loader the same way, then switch:
nui_set_image_source(ui, "picture", "second");
```

For borrowed OpenGL use `nui_gl_register_image`. The older standalone connectors
use `nui_window_register_image` or `nui_raylib_register_image`.
Register valid RGBA bytes for the chosen dimensions. Registration copies the
pixels; the renderer retains uploaded assets. Re-register a key to replace its
pixels. Use `object-fit: contain`, `cover`, or `fill` to control fitting.
Images have no dedicated click event; use a button or interactive canvas when
interaction is required. A missing asset uses its alt-text fallback.

## Native canvas

```html
<canvas id="scene" tabindex="0" style="height:110px"></canvas>
```

```c
NuiCanvasInput input;
if (nui_canvas_input(ui, "scene", &input) && (input.flags & NUI_PRESSED)) {
    printf("Canvas pressed at %.0f, %.0f\n", input.x, input.y);
}
```

Pointer coordinates are local to the canvas content. The flags include hover,
press, release, capture, focus, and cancellation. Press/release edges clear at
`nui_end_frame`. See the [canvas reference](reference.md#native-canvas-embedding)
for coordinate and clipping rules.

The borrowed adapter invokes your draw callback in UI paint order:

```c
static int32_t draw(void *data, void *renderer, NuiString id,
                    NuiRect bounds, NuiRect content, NuiRect clip) {
    (void)data; (void)id; (void)bounds; (void)clip;
    NuiRect square = {content.x + 16, content.y + 16, 64, 64};
    NuiColor blue = {54, 95, 221, 255};
    return nui_renderer_fill(renderer, square, blue) == 1 ? 0 : 1;
}
// Inside the normal frame loop:
nui_renderer_render(renderer, ui, draw, NULL);
```

Canvas callbacks return **0 on success**, unlike most setters. SDL3/raylib's
borrowed adapters provide a temporary painter token for `nui_renderer_fill`,
which clips drawing to the canvas. Do not cast it to a framework pointer.
For borrowed OpenGL use `nui_gl_render` and `nui_draw_fill` in the callback.
Simple `nui_sdl_draw`/`nui_raylib_draw` calls omit custom canvas content.

## Layout and scrolling

```html
<div id="list" tabindex="0">
  <label class="row">First row</label>
  <label class="row">Second row</label>
</div>
```

```css
#list { height: 96px; display: flex; flex-direction: column; overflow: auto; }
.row { height: 64px; }
```

```c
nui_scroll_to(ui, "list", 0, 10000); // Library clamps to the bottom.
nui_scroll_to(ui, "list", 0, 0);     // Back to the top.
NuiRect scroll;
if (nui_scroll_position(ui, "list", &scroll)) {
    printf("Offset %.0f / maximum %.0f\n", scroll.y, scroll.h);
}
```

There is no separate `nui_changed` event for scrolling. Read the scroll position
and compare it with the previous frame if your application needs that signal.
Wheel/keyboard input works through the host's normal event routing. Layout
responds to viewport updates; call `nui_set_viewport(ui, width, height)` when your
host's logical UI dimensions change. Standalone window connectors do this while
polling resize events; borrowed integrations leave that decision to the host.

## Creating a button or checkbox from C

HTML is not required for these two control types. You can also add them to an
existing UI using the typed API:

```c
NuiCheckbox sound = {0};
sound.id = "sound";
sound.label = "Enable sound";
sound.checked = 1;
nui_add_checkbox(ui, &sound);

NuiButton save = {0};
save.id = "save";
save.label = "Save";
save.class_name = "primary";
nui_add_button(ui, &save);
```

Use the same `nui_clicked`, `nui_changed`, and value/state APIs afterward. The
library copies the descriptions' strings. To insert into a specific container,
get its `NuiElement` with `nui_get` and use `nui_element_add_button` or
`nui_element_add_checkbox`. Typed insertion currently supports these two types;
it is not a general DOM mutation API.

## Runnable programs and tests

[Choose a renderer and language](../examples/controls/README.md). The showcase's
Reset, Replace options, Switch image, and scrolling buttons demonstrate host
updates; editing controls demonstrates user events. Tests are separate files,
not a framework the application must implement. Use `--check` for state/input
checks and `--smoke-test` for short hidden rendering checks.
