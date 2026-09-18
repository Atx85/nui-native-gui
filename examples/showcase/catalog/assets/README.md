# Rendering catalogue fixtures

These HTML/CSS assets exercise control variants in `crates/sdl3/tests/catalog.rs`.
The old language-specific catalogue applications were replaced by the
[single-file control showcases](../../../controls/README.md).

```sh
cargo test -p native-ui-sdl3 --test catalog
```

Use the [C recipes](../../../../docs/controls.md) for events and application updates.

## Coverage

This is a catalogue of the engine's supported control options, not of all HTML
or CSS features. The fixtures include the variants below.

| Element | Demonstrated options |
| --- | --- |
| Text input | explicit/implicit text type, value, placeholder, maxlength, readonly, disabled; host read/write |
| Checkbox | checked/unchecked, disabled in both states; host read/write, associated label; name/value groups, multiple values, group change events, preset and clear |
| Range input | min/max/value/step, negative and fractional values, step=any, defaults, equal endpoints, disabled; host clamping/read/write |
| Select/option | static and host-created lists, value/label fallback, selected, disabled select/option, empty and all-disabled lists, selected disabled item, long popup; count/index/value, replace/clear, option class/value CSS |
| Button | normal/primary/disabled, type=button, click handling |
| Label/span | for association, text, inline span, Unicode/entities, inline style |
| Canvas | multiple surfaces, tabindex 0/-1, CSS size/border/padding, native drawing, bounds/content/clip, pointer events |
| Image | src/alt, registered RGBA pixels, missing-source fallback, object-fit contain/cover/fill, source replacement |
| Containers | body/header/main/section/aside/div/footer, block/flex, grow/gap/padding/margins, resize |
| Overflow | visible, hidden, clip, auto, scroll, independent axes, keyboard focus, programmatic scroll position |

Tests load a canonical theme before this layout stylesheet. The repo
also tests this catalogue with the classic desktop and Windows 11-inspired themes.
IDs locate controls in host code; reusable control styling uses tag/class/attribute
selectors. An input does not need a particular ID to receive its appearance.

Disabled/readonly/maxlength and range bounds/step are creation-time attributes;
there is no generic runtime attribute setter. The typed insertion API currently creates checkboxes and buttons. Runtime APIs change values, checks,
select lists, checkbox-group selections, image sources, scroll position and viewport dimensions.
Programmatic setters do not emit user `changed` events. The host can poll `changed`
and `clicked` once per frame, then call `nui_end_frame` to reset transient state.

The catalogue does not imply browser features such as password/number inputs,
multiple select, textarea, form submission, grid or arbitrary DOM mutation.
These are optional test assets; applications can embed their own HTML/CSS.
