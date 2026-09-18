# API and rendering reference

For a quick start, use the [control showcases](../examples/controls/README.md).
[Language integration](languages.md) covers runtime selection in the SDK and toolchains.
The [larger showcases](../examples/showcase/README.md) contain the themed demos,
animated canvases and benchmarks discussed below. Commands run from the repository root.

The core lives in `crates/core/`, SDL in `crates/sdl3/`, and the C ABI in
`crates/c-api/`. Plain `cargo run` opens the SDL3 control showcase. See [C control recipes](controls.md) for individual widgets, events, and data updates.

For a game that already owns SDL3, the [borrowed C renderer adapter](../examples/integration/sdl3/README.md)
provides `nui_sdl_create/event/draw/destroy` through `native_ui_sdl3.h`.
It leaves event routing, text-input activation, logical sizing and presentation to
the game. `nui_end_frame` explicitly resets transient actions after the game reads them.

## Supported controls

The compiler supports `<button>`, `<input type="text">` (also the default input type),
`<input type="checkbox">`, single `<select>` controls with `<option>` children, and explicit
`<label for="control-id">` elements. `<canvas>` embeds native application drawing. `<img>` displays host-registered image assets. Labels have their own CSS rectangle, focus/activate their
associated control when clicked, and are skipped by Tab. A label without `for` displays text.
Wrapping controls inside labels is not supported.

`html`, `head`, `body`, `div` and `span` can supply structure; button and label text can contain
spans, entities and Unicode text. Document metadata and comments are accepted.
Controls accept `id`, `class` and inline `style`. Buttons, inputs and selects also accept `disabled`.
Buttons also accept `type=button`.
All supplied IDs, including structural IDs, must be globally unique. Anonymous controls render
but have no native ID. HTML5 parsing happens once and its temporary tree is dropped afterward.

CSS selectors: tag, `#id`, `.class`, attribute presence/equality (`[readonly]`,
`input[type="checkbox"]`), compounds such as `button.primary`, and comma lists.
`:not()` currently accepts one attribute selector, e.g. `input:not([type="checkbox"])`.
Attribute names are case-insensitive; input/button `type` values follow HTML's ASCII
case-insensitive matching. Other values are case-sensitive unless an explicit `i` flag is
provided; `s` explicitly requests case-sensitive matching. Attribute selectors and their
negations have class-level specificity. Other attribute operators and more general negations
are rejected.
Classes are for CSS only; the native API accepts plain IDs. Cascade resolution applies
specificity, source order, inline styles and `!important` independently to each property.

The original `from_html_css` constructor uses absolute layout; each control and label requires these five geometry properties:

```css
#save {
    position: absolute;
    left: 20px;
    top: 20px;
    width: 140px;
    height: 48px;
    padding: 10px 18px;
    background-color: #4170e6;
    color: white;
}
```

- Lengths use `px` or unitless zero. Left/top may be negative; sizes and padding cannot.
- `padding` accepts one to four values in CSS order; individual `padding-top/right/bottom/left`
  declarations participate in the cascade independently. Button text centers; form text aligns left.
  All text clips within the content area.
- `background-color`, solid-color `background`, and `color` support 3/4/6/8-digit hex,
  `rgb()` / `rgba()` (commas or space-separated components, percentages and alpha), `transparent`,
  the 16 basic CSS named colors, gray/grey, cyan/magenta, orange and rebeccapurple.
- `box-sizing` defaults to `border-box` for controls. Explicit `content-box` adds padding and the
  CSS border width to the specified width and height. Borders default to 1px on controls and zero
  on labels. A border-box cannot be smaller than its padding and border; its content area clamps
  to zero when there is no room for text.
- With the absolute constructor, coordinates are relative to the host UI origin. For nested containers and resizeable layouts, use the viewport constructor described below.
- `:hover`, `:active`, `:focus`, `:disabled`, and `:checked` support background colors/gradients,
  text color, border colors/radius, inset shadows, and outlines. They combine with tag/ID/class
  selectors and each other,
  such as `button.primary:hover:active`. Pseudo-classes contribute class-level specificity;
  normal cascade rules, inline styles, and `!important` still apply. Geometry, padding, border
  width/style and font size in state rules are rejected so hit areas and text metrics stay stable.
  Use `border-color` in state rules; the `border` shorthand also changes width/style.
- `:hover` follows the topmost pointer target, including disabled controls. `:active` lasts while
  the primary pointer press is captured or Space is held on the focused control. Release/focus loss
  clears activation; a drag-out release still cancels the click. `:focus` follows pointer or keyboard
  focus; `:disabled` follows the HTML attribute; `:checked` follows persistent checkbox state. There is no implicit priority between states:
  specificity and declaration order decide which colors win.
- State appearances are compiled at load time. The runtime paints resolved styles without consulting a DOM or
  stylesheet for painting, and selects the combination matching input state. Parsed rules remain available for explicit element and select-list updates.
- Without matching state rules, interaction does not recolor the control itself. The core still supplies
  defaults for the border, font size and colors absent from CSS. The demo declares its complete
  themes in `examples/showcase/ui/themes/`; the renderer has no platform-theme-specific branches.
- External CSS is supplied explicitly to `Ui::load` / `from_html_css`; HTML links never trigger fetching.
  Embedded style blocks are applied after the supplied CSS, in document order, then inline styles.

Unsupported elements, attributes and CSS features return errors rather than being silently misrendered.
There is no JavaScript, live DOM, event listener API, browser navigation or network access.

## Box decoration and themes

The reusable skins contain no demo IDs or positions:

- `examples/showcase/ui/themes/windows-xp-luna.css`: cream surfaces, blue borders, gold hover accents and dotted focus.
- `examples/showcase/ui/themes/macos-light.css`: flat surfaces, compact fields and checkbox, an inset blue popup indicator with paired chevrons, primary button styling and rounded focus rings.
- `examples/showcase/ui/themes/windows-11-fluent.css`: Fluent-inspired metrics, subtle lower borders and inset focus indicators.

Themes target `input:not([type="checkbox"])`, `input[type="checkbox"]`, `select`,
`button`, `label` and `option`. This includes text inputs with no explicit `type`, controls
with arbitrary IDs, and anonymous controls. Use `class="primary"` on any button to opt into
primary styling; ordinary buttons need no special class. IDs identify application actions
and optional per-control overrides, never the skin.

The demo's positions, field/button widths and action ordering live in `examples/showcase/ui/layout.css`
and `examples/showcase/ui/layouts/`. All three themes keep the same HTML, labels and actions. The Mac demo
places its primary action last; the Windows demos place it before Reset.

For a different application, load a theme followed by your own layout; do not load the demo layouts:

```html
<input id="nickname" placeholder="Nickname">
<input id="subscribe" type="checkbox">
<button id="save" class="primary">Save</button>
```

```css
/* App layout, loaded after the chosen theme. Heights/padding come from the skin. */
input, button { position: absolute; left: 24px; top: 24px; }
#nickname { width: 280px; }
#subscribe { top: 80px; }
#save { top: 124px; width: 100px; }
```

The checkbox gets its width from the theme. App-specific rules can override height, width,
padding or appearance through the normal CSS cascade. The viewport constructor additionally supports block flow and a focused flex subset; CSS Grid remains unsupported.

The palettes and supported decoration are CSS-driven. Shared renderer constants still include the
checkmark/default triangle shapes, the 22px picker area, a 1px caret, popup label padding, eight visible popup
rows, half-opacity placeholder/default disabled text, and fallback colors from `Theme::default()`.
The SDL backend uses bundled Noto Sans (replaceable with `SdlBackend::with_font`), not Tahoma, Segoe UI or
San Francisco; font-family is not a CSS property yet. The host still defines window size, and the
core owns input/focus behavior. These are inspired skins, not operating-system-native widgets.

```css
button {
    /* Supply position/left/top/width/height as above. */
    font-size: 13px;
    color: black;
    background: linear-gradient(to bottom, white, #f5f4ea 45%, #e6e3d5);
    border: 1px solid #003c74;
    border-radius: 3px;
    box-shadow: inset 0 0 0 1px white;
}
button:hover { box-shadow: inset 0 0 0 2px #f6c65a; }
button:focus { outline: 1px dotted #383838; outline-offset: -5px; }
```

Supported additions:

- `background-image: linear-gradient(...)` or `none`, and gradient `background` shorthand.
  Gradients accept two to eight color stops with optional positions from 0% to 100%.
  Omitted positions are interpolated; coincident stops produce hard edges. Directions are
  `to bottom` (default), `to top`, `to left`, `to right`, or 0/90/180/270deg.
  Colors interpolate with premultiplied alpha. A solid `background` resets the gradient;
  `background-color` alone preserves it. Images paint over the background color.
- `border: <width> solid <color>` or `none`; uniform `border-width`; `border-style: solid/none`;
  `border-color` with one to four colors; individual `border-top/right/bottom/left-color`.
  Shorthands resolve each longhand independently in the cascade, including `!important`.
- `border-radius` with one nonnegative pixel length, clamped to fit the rectangle.
  Border rings leave transparent interiors and rounded corners visible to the host.
- `box-shadow: inset <x> <y> 0 <spread> <color>` or `none` for crisp inner highlights.
  Signed offsets allow individual edge shading (e.g. `inset 0 -1px 0 0 blue` along the bottom).
  Blur, multiple shadows and outer shadows are deliberately unsupported.
- `outline: <width> solid|dotted <color>` or `none`, with `outline-width`, `outline-color`,
  `outline-style`, and signed `outline-offset`. Outlines are visual and never enlarge hit areas.
  Negative offsets put keyboard focus dots inside a control. Solid outlines follow the border
  radius; dotted outlines stay rectangular.
- `font-size` in px (greater than zero, up to 256px). Text, caret measurement, selection and
  pointer text positioning use this size. The bundled Noto Sans font remains in use. Font-family/weight are not implemented.
- `select::picker-icon` styles the existing 22px-wide arrow area using backgrounds, borders,
  radius, shadows, outlines and color. States precede the pseudo-element, e.g.
  `select:hover::picker-icon`. Optional positive `width` and `height` center a smaller indicator
  inside that area (clamped to fit); they do not change the select's hit area or label space.
  Position and padding are not supported on the part.
- Picker vector icons use explicit renderer extensions (not standard browser CSS):
  `-native-ui-icon: chevron-down | chevron-up-down`, `-native-ui-icon-size: 10px` and
  `-native-ui-icon-stroke: 1.5px`. Size is the icon width; the paired icon is 1.25 times as tall.
  Both lengths must be positive and at most 64px. The renderer draws round-ended segments with
  sampled edge coverage, using `color`, independently of the font. Mac and Windows 11 use these
  shapes; XP retains its filled triangle. Shape, size, stroke and colors may change with state.
  These are shared geometric primitives, not operating-system branches.
- A single-character `content` string is still available on `::picker-icon` for text symbols;
  it cannot be combined with a vector icon. `font-size` sizes that character independently.
- `input::selection` supports solid background and color for selected text. For example,
  `#name::selection { background: #316ac5; color: white; }`.
- `option` accepts box decoration, color and font size for existing popup rows. `:hover` tracks
  the keyboard/pointer highlight, `:checked` the committed selection, and `:disabled` disabled
  options. Rows keep the select's dimensions; no option layout or padding is supported.
- `body { background-color: #ece9d8; }` exposes the canvas color through `Ui::background_color()`.
  The host still owns clearing/render targets; the demo uses this color when clearing its canvas.
  Only solid backgrounds are supported on the body, and body styles do not inherit into controls.

Decoration lengths are bounded at 4096px. Unsupported variants return diagnostics. Decoration
is lowered to `FillRect` commands in the core, so existing primitive backends need no new widget
API. Gradients use bounded one-CSS-pixel bands; rounded corners and focus dots keep the crisp
raster appearance of the theme. HTML control types and input behavior are unchanged; theme CSS can override base layout metrics.

## Form values and editing

```html
<input id="name" type="text" placeholder="Your name" maxlength="80">
<input id="enabled" type="checkbox" checked>
<select id="mode">
    <option value="basic">Basic</option>
    <option value="advanced" selected>Advanced</option>
    <option value="soon" disabled>Coming soon</option>
</select>
```

Supply absolute geometry or place these elements in a viewport layout. Poll or update by ID:

```rust
let name: Option<&str> = ui.get_value("name");
let mode: Option<&str> = ui.get_value("mode"); // option value, not its display label
let enabled: Option<bool> = ui.checked("enabled");
if ui.changed("name") {
    println!("Name edited: {:?}", ui.get_value("name"));
}
ui.set_value("name", "Ada")?;
ui.set_value("mode", "basic")?;
ui.set_checked("enabled", true)?;
```

`value`, `checked`, and `selected_index` return `None` for unknown IDs, incompatible controls,
or a select without a selection. `changed` stays true after a user edit until `end_frame`,
including across repeated reads. Values persist across frames. Host setters validate IDs,
control types, option values and text length; they do not emit user change events.
`clicked` reports button clicks; `get` retrieves an element handle.

- Text inputs support `value`, `placeholder`, `readonly`, `disabled` and `maxlength`.
  Click to place the caret, drag or Shift+Left/Right/Home/End to select, and use Backspace/Delete
  to edit. Ctrl/Cmd+A selects all. The SDL adapter handles Ctrl/Cmd+C/X/V through the system clipboard.
  Text comes from `Ui::text_input` / SDL committed text events; Space is not inserted twice.
  Editing and maxlength use Unicode scalar values, not grapheme clusters. Controls and line
  separators are stripped. Readonly fields can be focused and copied; disabled fields cannot.
  Long text scrolls to keep the caret visible. Text hit positions come from the last render;
  before the first render or after an unrendered edit, clicking places the caret at the end.
- Checkboxes support `checked` and `disabled`. Click or press/release Space to toggle;
  dragging out or losing focus cancels a press. `:checked` can style checked colors.
- Selects use `value`, `selected` and `disabled` on options. Omitted option values use normalized
  label text. The last explicitly selected option wins; otherwise the first enabled option is
  selected. Empty and entirely disabled lists may have no selection.
  Click, Enter or Space opens the popup. Up/Down/Home/End navigate past disabled options;
  Enter/Space or an option click commits. Escape, Tab, outside click or focus loss dismisses it.
  Navigation in a closed select changes the value immediately. Popups show up to eight rows;
  mouse wheel scrolls and keyboard navigation reveals the highlighted option. Popups paint
  above other controls and consume outside dismissal clicks. They open below the control,
  within the host's existing clip; leave space below selects in fixed layouts.

Runtime select lists can be replaced without recreating the UI:

```rust
use native_ui::SelectItem;
ui.set_select_options("files", &[
    SelectItem::new("report.txt", "Report"),
    SelectItem { disabled: true, ..SelectItem::new("busy.txt", "Busy file") },
])?;
ui.set_value("files", "report.txt")?;
```

The C API (also used by C++, Go, Zig and Jai) adds `NuiSelectOption`,
`nui_set_select_options`, `nui_select_option_count` and `nui_selected_index`.
Pass an empty slice, or NULL/count 0 in C, to clear the list. Strings are copied.
The last `selected` flag wins; otherwise an enabled option matching the old value is
preserved, otherwise the first enabled option is selected. An empty/all-disabled list
has no selection unless explicitly selected. Duplicate values are allowed: value lookup
uses the first match, while preservation uses the first enabled match.

Replacement is atomic on errors, closes that select's popup and cancels its pending
press. It keeps the select's geometry/focus and other control values, invalidates paint,
and does not emit a user `changed` event. Labels are literal single-line text, never HTML.
New options support `value`, `label`, `disabled`, `selected` and CSS `class` (C: `class_name`).
The engine retains parsed CSS for typed additions and option updates; rules are applied on
explicit list updates, including class/value/disabled selectors. Rendering still uses
precomputed styles. This is an additive ABI 2 extension; rebuild the library for new symbols.

Password, multiline and other input types, multiple selects, option groups, live IME preedit,
word navigation, undo/redo and full grapheme-aware editing are not implemented.

## Relative layout and the canvas studio

```sh
cargo run -p native-ui-demo --example studio -- --theme macos
# xp and win11 work too.
```

The studio places a growing native canvas on the left and a fixed-width `<aside>` on the right.
Its controls change the pattern, colours, animation speed, amplitude, detail and grid; the marker can be dragged.
Drag the sliders, scroll the options panel, and resize the window to see the layout adapt. This is separate from both earlier demos.

```rust
let mut ui = Ui::from_html_css_with_viewport(html, css, 1000.0, 680.0)?;
// Use logical window dimensions, not high-DPI framebuffer dimensions:
ui.set_viewport(window_width as f32, window_height as f32)?;
```

```html
<main class="workspace">
    <canvas id="scene"></canvas>
    <aside class="options" tabindex="0">
        <label for="rate">Speed</label>
        <input id="rate" type="range" min="0.1" max="4" step="0.1" value="1">
    </aside>
</main>
```

```css
body { padding: 24px; }
.workspace { display: flex; gap: 20px; height: 100%; }
canvas { flex-grow: 1; min-width: 220px; }
.options { width: 268px; padding: 20px; overflow: auto; }
label { margin-bottom: 6px; }
input { margin-bottom: 16px; }
```

Load the reusable theme before these application rules. In viewport mode, `body`, `div`,
`main`, `aside`, `section`, `header` and `footer` become retained layout/decoration boxes.
They are skipped by Tab unless given `tabindex="0"`; this makes scroll panels keyboard accessible. The root body fills the viewport by default. HTML and
CSS are parsed once. Layout retains a small tree of computed settings and control indices;
parsed CSS rules remain available for typed element additions and select-list updates.
An unchanged viewport is a no-op, so normal animation frames do not rerun layout. Resizing
updates painting, canvas content rectangles and hit testing together. Values and keyboard focus
survive; open select popups, pointer captures and cached text hit positions are invalidated.
An invalid reflow returns an error before changing any rectangles.

Supported relative layout:

- Default block flow stacks children vertically. Auto width fills the parent's content width;
  auto container height fits its in-flow children. Leaf controls normally get their height from
  the theme; otherwise it uses font size plus padding/borders (canvas and img have no intrinsic height).
- `margin` and individual margin sides accept px or zero, including negative margins. Margins
  are additive, never collapsed; `auto` margins are not supported. Padding stays inside the box.
- `width`/`height` accept px, percentages or `auto`. Percentages use parent content dimensions.
  Percentage heights under an auto-height parent use zero during intrinsic measurement; give
  parents definite heights for percentage-height designs. Percent padding/margins are unsupported.
- `min-width`, `min-height`, `max-width`, `max-height` take nonnegative px values. All constraints
  follow `box-sizing`; border-box is the default. Minimums cannot exceed maximums.
- `display: flex` supports one row (default) or column through `flex-direction`. Positive
  `flex-grow` weights share remaining main-axis space, respecting size limits. `gap` adds space
  between children, in flex containers and as a convenience in vertical block containers.
- `align-items` supports `stretch`, `start`/`flex-start`, `center` and `end`/`flex-end`.
  Stretch affects auto cross-axis sizes. Auto widths in rows start at padding/borders or their
  minimum width; use explicit widths or `flex-grow` rather than expecting text intrinsic sizing.
- `position: relative` offsets a flow box using `left`/`top` without changing its reserved slot.
  `position: absolute` removes it from flow and uses the immediate layout parent's content
  origin. This deliberately does not implement browser containing-block ancestor selection.

This subset does not implement shrinking, wrapping, flex-basis/shorthand, justify-content,
CSS Grid, text wrapping/intrinsic text measurement or automatic breakpoints. If fixed/minimum
sizes exceed available space, boxes overflow according to their container’s `overflow` setting.
Set appropriate host window minimums (the studio uses 700×600). In a growing flex column,
use `height: 0; flex-grow: 1` on a scrollable region to give it the remaining height.

The original `from_html_css`/`load` absolute path stays compatible and rejects relative layout
properties with a diagnostic directing callers to the viewport constructor.

## Sliders and overflow

A horizontal slider is a regular `<input type="range">`, independent of its ID:

```html
<input id="volume" type="range" min="0" max="100" step="1" value="35">
```

Read and write it through `value`/`set_value`, and poll `changed` just like a text input.
Values are clamped and snapped to `min`-based steps. Defaults are 0–100, step 1 and the
midpoint; `step="any"` enables continuous values. Finite numeric values and a positive step
are required. Drag the thumb or click the track; arrows change one step, Page Up/Down ten,
and Home/End move to the endpoints. Labels focus their slider. `disabled` prevents input.

```css
input[type="range"] { height: 30px; background: transparent; border: none; }
input[type="range"]::slider-track { height: 4px; background: #ccc; border-radius: 2px; }
input[type="range"]::slider-fill { background: #087aff; border-radius: 2px; }
input[type="range"]::slider-thumb {
    width: 18px; height: 18px; background: white;
    border: 1px solid #aaa; border-radius: 9px;
}
input[type="range"]:active::slider-thumb { background: #eee; }
.options { width: 268px; height: 400px; overflow: auto; }
.options::scrollbar { width: 12px; height: 12px; background: #f3f3f3; }
.options::scrollbar-track { background: transparent; }
.options::scrollbar-thumb {
    width: 24px; height: 24px; background: #aaa;
    border: 3px solid #f3f3f3; border-radius: 6px;
}
.options::scrollbar-thumb:hover { background: #888; }
.options::scrollbar-thumb:active { background: #666; }
.options::scrollbar-corner { background: #f3f3f3; }
```

These parts use the existing CSS colors, gradients, borders, rounded corners, shadows and
outlines. Thumb/track dimensions are fixed px; state rules change appearance, not geometry.
The slider fill follows its track; scrollbar tracks and corners follow the scrollbar gutters.
Set sizes on the owning track/scrollbar; these dependent parts reject width/height.
Slider track width defaults to available width minus the thumb width. Scrollbar width sets
the vertical gutter, height the horizontal gutter; thumb width/height set minimum thumb
lengths along the corresponding axis. Actual scrollbar thumb length reflects visible content.
Zero scrollbar thickness hides that bar while preserving wheel/programmatic scrolling.
`::-webkit-slider-runnable-track`, `::-webkit-slider-thumb`, and the corresponding
`::-webkit-scrollbar*` names are accepted aliases for the parts above. Parts work with class,
attribute and global selectors; they have no dependency on application-specific IDs.

In viewport layouts, containers support `overflow`, `overflow-x` and `overflow-y`:

| Value | Clips children | Wheel / scrollbar | Programmatic scroll |
| --- | --- | --- | --- |
| `visible` (default) | No | No | No |
| `hidden` | Yes | No | Yes |
| `clip` | Yes | No | No |
| `auto` | Yes | Bars when needed | Yes |
| `scroll` | Yes | Bars always present | Yes |

The shorthand accepts one value or two (x then y). As in CSS, pairing `visible` or `clip`
with an axis that scrolls computes them to `auto` or `hidden`, respectively. Scrollbars
reserve space and reflow children; offsets clamp on resize. Nested wheel input consumes
available distance before continuing in the parent. Thumb dragging, track clicks and
keyboard arrows/Page Up/Down/Home/End are supported. Use `tabindex="0"` to focus a panel
with Tab; tabbing to an offscreen control reveals it automatically.

```rust
ui.scroll_wheel(delta_x, delta_y); // logical pixels; positive means right/down
ui.set_scroll_offset("options", 0.0, 100.0)?;
let offset = ui.scroll_offset("options");
let maximum = ui.scroll_max("options");
```

The SDL adapter forwards horizontal and vertical wheel/trackpad deltas. The C API exposes
`nui_scroll_wheel`, `nui_scroll_to` and `nui_scroll_position`. Scrolling invalidates the paint
cache only when offsets change. It moves/clips controls, images and native canvases together
and cancels old pointer captures. Canvas callbacks retain full content dimensions and receive
a separate `CanvasRegion::clip`; custom renderers must apply that clip. This adds a clip
argument to C canvas callbacks in **ABI 2**; rebuild older clients with the supplied header.

This is rectangular overflow clipping, including when a container has rounded corners.
Select popups remain a top layer. Overlay/fading scrollbars, arrow buttons, smooth/inertial
scrolling, vertical sliders and tick marks are not implemented. The three included themes
supply different slider geometry and scrollbar treatments using only these CSS parts.

## Images

`<img>` uses an application-owned asset key. The core never opens files, fetches URLs,
decodes formats or owns GPU textures. Images participate in the same CSS layout as
other elements; no particular ID, class or source name is required.

```html
<img id="preview" src="project-preview" alt="Preview unavailable">
```

```css
img {
    width: 100%;
    height: 160px;
    margin-bottom: 12px;
    padding: 6px;
    border: 1px solid #d9dde2;
    background: #ffffff;
    object-fit: contain;
    color: #333333;
}
```

Register packed, straight-alpha RGBA pixels in the SDL backend once, before rendering.
The host can get those pixels from its existing image decoder or asset pipeline.

```rust
backend.register_image_rgba("project-preview", width, height, &rgba_pixels)?;
backend.render_cached(&ui, &mut canvas)?;
// Later: switch the element to another registered asset.
ui.set_image_source("preview", "another-preview")?;
// Or replace the pixels under an existing key / remove that key.
backend.remove_image("project-preview");
ui.invalidate_paint(); // request repaint in an event-driven host after asset changes
```

Registration uploads one texture shared by every element using that key. Successful
replacement/removal invalidates the backend's retained paint layers, including any cached
missing-image fallback. Invalid registration preserves the existing asset. Asset mutation
does not wake the host event loop: arrange a repaint when changing assets. Registered
pixels are retained for texture recreation after `clear_caches()` / a renderer reset;
normal frames neither decode nor upload them. `cache_stats().image_uploads` counts uploads.
Source asset memory is separate from the 64 MiB derived layer cache budget; remove unused
assets to release their pixels and texture.

Supported fitting modes are `fill` (default, stretches to the content box), `contain`
(centered, preserves the whole image), and `cover` (centered, crops to fill). Borders and
padding surround the content box. Clipping is rectangular; border-radius decorates the
box but does not mask image pixels. Missing assets draw `alt` as clipped single-line text;
empty/missing alt draws no content. Images block click-through but do not activate or take
keyboard focus.

Size comes from CSS and layout, not the decoded asset: give images a height or a flex size.
Intrinsic sizing, `object-position`, `srcset`, inline images inside text/buttons, SVG decoding
and automatic file/URL loading are outside this first implementation. The existing studio
example registers a small transparent bitmap once and displays it in the header:

```sh
cargo run -p native-ui-demo --example studio -- --theme macos
```

Custom renderers implement `Renderer::image_size` and `Renderer::draw_image(ImageDraw)`.
The core supplies source rectangles in image pixels, destination and clip rectangles in
logical UI coordinates. Images draw in document order before select popups. Renderers
without image support keep working and show alt text. `Ui::image_sources()` lists source
keys in document order (including duplicates); `image_source(id)` reads an element's key.

## Native canvas embedding

```sh
cargo run -p native-ui-demo --example canvas -- --theme macos
# Also supports --theme xp and --theme win11.
```

The separate example draws an animated plot with ordinary SDL commands, lets you drag a marker,
and puts themed controls alongside it. The existing form demo is unchanged.

```html
<canvas id="scene" class="viewport" tabindex="0"></canvas>
```

```css
.viewport {
    position: absolute;
    left: 24px; top: 24px; width: 640px; height: 360px;
    padding: 12px;
    background: #172331;
    border: 1px solid #33465a;
    border-radius: 8px;
}
```

A canvas is a native drawing region, not a browser Canvas2D API or an owned pixel buffer.
Its ID is arbitrary and optional. CSS supplies position, size and decoration through the normal
cascade; the default is transparent with no border. `CanvasRegion` exposes the border box and
content box (excluding border and padding), plus the visible `clip` after ancestor overflow,
in UI coordinates. HTML `width`/`height` attributes,
fallback content and nested controls are currently rejected; use CSS dimensions.

```rust
backend.render_with_canvases(&ui, &mut canvas, |region, native| {
    if region.id == Some("scene") {
        let area = region.content;
        native.set_draw_color(sdl3::pixels::Color::RGB(80, 180, 240));
        native.fill_rect(sdl3::render::FRect::new(
            area.x + 10.0, area.y + 10.0, 40.0, 40.0,
        ))?;
    }
    Ok(())
})?;
```

The callback receives the existing SDL renderer and target. Coordinates remain in UI space,
under the host's scale, viewport and logical presentation. The backend intersects the native
`CanvasRegion::clip` with the host clip; it rounds fractional clip edges inward to SDL's integer clip
coordinates. Empty content or an empty effective clip skips the callback. Clipping is rectangular:
`border-radius` styles the surrounding box but does not mask native drawing to a rounded shape.

Callbacks run in document paint order after canvas decoration, before later controls and all
select popups. Target, logical presentation, viewport (including automatic sizing), scale, clip,
blend mode and draw color are restored after each callback, including returned errors. Native
callbacks are trusted: do not clear/present the frame, destroy its renderer or target, or bypass
clipping to draw outside the region. `SDL_RenderClear` ignores the clip; fill the content rectangle
instead. Error returns are handled; panic unwinding is not a supported callback error mechanism.
The host alone owns resource lifetime and presentation.

The direct `render_with_canvases` dispatch path adds no intermediate texture, CPU readback, pixel upload, or per-canvas heap
allocation. It does perform callback dispatch and clip/state management; zero overhead or a
specific frame rate is not promised. Application rendering cost is still the application's own.

`ui.canvas_region("scene")` returns its current geometry. After feeding input, poll
`ui.canvas_input("scene")` for content-local pointer coordinates, hover/capture/focus and primary
press/release positions. Presses must begin in the content box; captures retain outside positions
until release. Covering controls and popup dismissal consume their own presses. `pressed`,
`released` and `cancelled` persist until `end_frame`; repeated polling does not consume them.
These fields aggregate a frame (the last press/release position wins), not an ordered event queue.
Tab or window focus loss cancels captures. Clicking content focuses it; `tabindex="0"` also
includes it in keyboard traversal. Omitted tabindex or `-1` skips Tab traversal. The host handles
its own keyboard, wheel, secondary-pointer and application events, using `focused` or
`ui.canvas_at(x, y)` to determine routing. Canvas input never produces button clicks/form changes.

The core extension is `Renderer::draw_canvas(CanvasRegion)`. It contains no SDL types; another
backend can implement it with its own native context and clipping/state rules. Its default method
is a no-op, so existing renderers remain compatible. Likewise, SDL's ordinary `render` draws only
canvas decoration; use `render_with_canvases` for application content. The versioned C API exposes this SDL integration through `crates/c-api/`.

## Rendering and input

The core calls graphics primitives (`FillRect` and `Text`), font measurement, and the optional native-canvas and registered-image hooks. A backend does not
implement `draw_button`, and it does not own the application loop. The SDL adapter borrows the host
canvas for rendering, respects its scale/viewport/clip and restores drawing state afterward.
Rounded boxes, borders, shadows, focus rings, checkmarks and picker icons use
coverage antialiasing on the backend's physical-pixel grid. Borders and fills are
sampled together to avoid gaps along their shared edge. The painter emits ordinary
RGBA rectangle spans; flat interiors remain wide spans rather than per-pixel draws.
Custom Rust backends should implement `Renderer::raster_scale()` with physical
pixels per logical unit; its default is `(1.0, 1.0)`. Layout and pointer coordinates
remain logical units. SDL, OpenGL and raylib supply their current scale, and SDL's
retained command/texture cache uses that scale as part of its existing cache key.

Text uses cached font textures and actual glyph dimensions; long labels clip within their padding.
Fontdue provides basic Unicode glyph layout, not full bidirectional/complex-script shaping or font fallback.
The bundled Noto Sans font and its OFL license are in `crates/text/assets/`, sourced from
[Google Fonts](https://github.com/google/fonts/tree/main/ofl/notosans).

Input targets the topmost painted control. Presses are captured until release; drag-out release and
focus loss cancel activation. Disabled controls block click-through and are skipped during keyboard
navigation. Hit rectangles have exclusive right/bottom edges. Mouse input uses SDL's coordinate
conversion so it follows host transforms and high-DPI scaling. The SDL event adapter starts/stops
platform text input as focus changes. Other backends should feed `text_input`, route editing keys
with `key_down_with_shift`, and use `wants_text_input` / `selected_text` for platform integration.
`Renderer::measure_text` should include trailing-space advance for correct caret positioning.

## Redrawing and paint caching

The form demo now waits for SDL events while idle. It repaints on UI changes and window
exposure/resizing instead of waking every 16ms. `Ui::paint_revision()` is an opaque token
that changes with relevant input, host value updates, and layout changes. Compare tokens
for equality; they also distinguish newly loaded UI instances. `end_frame()` clears input
flags without invalidating paint. Hosts must still request repaint for window exposure,
display changes, or their own drawing. `Ui::invalidate_paint()` explicitly advances the token.
This is conservative change tracking: some input that leaves pixels unchanged can still
advance it, but pointer motion within the same canvas/control does not, unless it changes
a text selection or popup highlight.

For a game or animated visualization, use the optional SDL cache:

```rust
// Every game frame: clear the host target, then draw live content and cached UI.
backend.render_cached_with_canvases(&ui, &mut canvas, |region, native| {
    draw_game(region, native)
})?;
canvas.present();
ui.end_frame();
```

`render_cached` provides the same caching without native drawing callbacks. The canvas
demo uses `render_cached_with_canvases`; its native drawing still runs each frame.
UI paint commands and text hit positions are calculated only on cache rebuilds.
On accelerated SDL renderers, static commands are painted into transparent target textures,
with separate layers before/between/after native canvases to preserve document order and
popup overlays. Warm frames composite these layers and call the live canvas callbacks.
Native scene pixels are never stored in the UI cache. Premultiplied blending preserves
transparent controls and text edges, with possible small 8-bit rounding differences.

Changing the UI token, output size, scale, viewport, logical presentation, or host clip
rebuilds the cache. The initial implementation refreshes all UI layers on a change.
Target textures use four bytes per output pixel per layer, capped at 64 MiB per backend;
layers over that budget fall back to recorded commands. Software renderers also replay
recorded commands to preserve text transparency and avoid full-window CPU image copies.
Unsupported target allocation/blending falls back the same way. Command replay still
draws individual primitives, but skips decoration generation and text measurement.
Existing `render` / `render_with_canvases` remain available without a paint cache.

Call `backend.clear_caches()` after SDL renderer/device reset events; both demos do this.
The caller retains ownership of clearing, presentation, animation timing and renderer
lifetime. Cached drawing restores host rendering state, including when a callback returns
an error. Custom callbacks retain the same clipping and state contract as direct rendering.
Changing native game content alone does not invalidate the UI. External native drawing
still needs its own event-driven repaint decisions when a game is paused.

`backend.cache_stats()` reports rebuilds, reuse, paint-command generation, text-measurement
counts and retained layer memory. Run the repeatable comparison with:

```sh
cargo run --release -p native-ui-demo --example cache_bench
cargo run --release -p native-ui-demo --example cache_bench -- --software
```

It checks cached/direct pixels and measures repeated frames on the selected renderer.
Results depend on the renderer, resolution, UI complexity and frequency of changes;
caching trades memory and rebuild cost for cheaper unchanged frames.

## Validation and build requirements

```sh
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p native-ui
# Optional native GPU check: image blending, fitting, cached layers and live canvases.
cargo run -p native-ui-sdl3 --example image_cache_check
```

SDL builds statically from source, requiring CMake and a C toolchain (already present on this Mac).
The first SDL build takes longer; later runs reuse it. Core-only consumers do not need SDL or CMake.

OpenGL/raylib canvas adapters, general standalone text, additional form controls, CSS Grid and broader flex support, broader CSS pseudo-class support,
accessibility integration and a build-time UI compiler remain future milestones.

### Example coverage

The [example index](../examples/README.md) provides individual controls in C,
Rust, Go, Zig and experimental Jai. Larger all-controls source examples remain in
`examples/showcase/catalog/` as integration references. Their
[coverage table](../examples/showcase/catalog/assets/README.md) lists control
variants and runtime APIs.

### Floating UI over a native canvas

`examples/showcase/ui/overlay.html` places the full-window native canvas first, followed by
a floating header and scrollable options panel. `overlay.css` sizes the canvas
to 100% and uses normal layout for the controls above it. HTML order determines
painting order; no z-index or special overlay renderer is needed. Even empty
space inside the floating panel blocks canvas clicks. The remaining canvas area
stays interactive. The panel uses CSS alpha transparency, not backdrop blur.

Build the example against the existing single-library SDK:

```sh
python3 tools/build-overlay-example.py
./target/native-ui-sdk/overlay
```

The generated `overlay.c` embeds the markup/styles and uses the same native SDL
curve drawing as the C studio. It animates beneath cached controls; `--check`
runs twelve native frames and checks cache reuse.

### Checkbox groups (multiple selection)

Give checkboxes the same `name` and distinct `value` attributes. They remain
normal checkboxes with independent IDs, labels, styles, focus and checked states:

```html
<input type="checkbox" id="png" name="formats" value="png" checked>
<label for="png">PNG</label>
<input type="checkbox" id="svg" name="formats" value="svg">
<label for="svg">SVG</label>
<input type="checkbox" id="pdf" name="formats" value="pdf" checked>
<label for="pdf">PDF</label>
```

```rust
let selected = ui.checked_values("formats"); // Some(vec!["png", "pdf"])
ui.set_checked_values("formats", &["png", "svg"])?;
ui.set_checked_values("formats", &[])?; // clear the group
if ui.checkbox_group_changed("formats") == Some(true) {
    // Read the current values after a user toggle, then end_frame as usual.
}
```

C ABI clients use `nui_checked_value_count`, `nui_checked_value` (one checked item
by index, using the usual caller-owned string buffer), `nui_set_checked_values`,
and `nui_checkbox_group_changed`. These are additive ABI 2 functions; rebuild the
SDK to use them. C, Rust, Go, Zig and experimental Jai catalogue examples
include the same multi-selection group, preset and clear actions.

Group names apply across the whole UI, including separate containers. Each checkbox’s group
name and item value are fixed when it is created. Typed additions can join
existing groups. `name` currently applies only
to checkboxes and must be nonempty. A missing `value` defaults to `on`; explicit
values are recommended. `get_value(id)` returns that value regardless of checked state.
An unnamed checkbox still works independently as before.

Reads follow HTML order and include disabled checked members. Duplicate member
values are returned per checked member; setting a value checks every matching
member. An empty selection is different from an unknown group. Unknown values
are rejected before any state changes. Host setters may change disabled members,
just like `set_checked`, and emit no user change events. The group event is true
when any member has a user change in the current frame; reads do not consume it.
Repeated identical setters do not invalidate painting. This is raw UI state,
not browser form serialization. The group has no minimum/maximum selection limit
and does not make the existing single-select dropdown a multiple-select control.

### Element handles and controls created from code

The Rust API now uses `clicked(id)` for click polling and `get_value(id)` /
`set_value(id, value)` for values. `get(id)?` returns an element handle or an error
for a missing ID. Update old `ui.get(id)` click checks to `ui.clicked(id)`, and old
`ui.value(id)` reads to `ui.get_value(id)`. The low-level absolute button constructor
is now `add_button_at(id, text, rect)`.

```rust
use native_ui::{Checkbox, Button};

let mut parent = ui.get("options")?;
parent.add_checkbox(Checkbox::new("showFPS"))?; // Label: Show FPS
parent.add_checkbox(Checkbox::new("showGrid").label("Display grid").checked(true))?;
parent.add_button(Button::new("resetView").class("primary"))?;

// When parent is no longer used, the Ui can be borrowed again.
ui.get("showGrid")?.set_checked(false)?;
if ui.get("resetView")?.clicked() { /* reset the view */ }
ui.add_checkbox(Checkbox::new("enable_shadows"))?; // Root/body; Enable Shadows
```

The first typed descriptors are `Checkbox` and `Button`. Only `id` is required.
Both support label, class and disabled overrides; checkboxes additionally support
checked, group name and value. Their public fields can also be initialized using
`..Default::default()`. An omitted label is generated from camelCase, underscores
and hyphens, preserving acronyms; an explicitly empty label hides it. Checkbox
labels are associated with their control automatically. They use a flex row with
class `native-checkbox-row`; normal checkbox and label CSS still applies.

Use these additions with `from_html_css_with_viewport`; they require initialized
viewport layout. Only containers accept children. Typed data compiles directly
through the same style/control compiler as HTML, without parsing markup strings.
Loaded CSS (including embedded style rules) styles new controls too. The engine
retains parsed CSS for mutations, but normal painting and resizing do not parse CSS.

Successful additions preserve existing values, focus, group membership and scroll
position (clamped to current bounds). As with resize, they cancel open select popups
and active pointer captures. They rebuild the UI cache once; subsequent frames
reuse it. Invalid IDs, duplicate IDs, invalid CSS and failed layouts roll back the
entire addition. Paint, hit-test and Tab order follow the parent tree.

Rust element handles borrow the UI mutably: use them within a scope and reacquire
with `get` as needed. They are not persistent DOM nodes. Removal and generic tree
mutation are not part of this initial typed API.

C ABI callers use `nui_get(ui, id, &element)`, then
`nui_element_add_checkbox(element, &description)` or `nui_element_add_button`.
Zero-initialize `NuiCheckbox` / `NuiButton` and set only needed fields. NULL label
generates text; an empty string hides it. `nui_add_checkbox` / `nui_add_button`
add to the root. Element tokens need no separate free, remain stable across
insertions and live only as long as their owning UI. `nui_get_value` is the new
canonical value-reader name; the exported `nui_value` alias remains for existing
ABI 2 binaries. All current examples use the new spelling.

The rendering catalogue test (`crates/sdl3/tests/catalog.rs`) constructs a checkbox
and button in its **Created from code** section. The C recipes show these APIs too.

## OpenGL adapter

`native-ui-opengl` implements UI painting and native canvas callbacks for an
existing desktop OpenGL 3.3+ context. Canvas callbacks receive logical bounds and
physical viewport/scissor dimensions, already applied. C applications can use
`nui_gl_create`/`nui_gl_render` with their own context, or `nui_window_open` with
`NUI_BACKEND_OPENGL` for the bundled window helper.

See the [OpenGL guide](../crates/opengl/README.md) for complete examples, context
ownership, HiDPI handling and the exact graphics-state restoration contract.

### OpenGL retained UI caching

`OpenGlBackend::render_cached_with_canvases` retains static UI layers and invokes
native canvas callbacks each frame in document order. The C OpenGL/window helpers
and language examples use it automatically. UI edits, image changes and surface
size/DPI changes rebuild the layers; unchanged frames skip paint generation and
text measurements. Textures have a 64 MiB budget, with retained command fallback
for excess layers or host sRGB rendering. See the [OpenGL cache guide](../crates/opengl/README.md#retained-ui-cache)
for diagnostics, explicit clearing, and the uncached rendering option.
