use crate::{
    CanvasRegion, ControlKind, Rect, Ui,
    controls::{POPUP_ROWS, TextInput},
    css,
    decoration::{Appearance, inset, paint_box, paint_icon},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);
/// Defaults for properties absent from CSS. The core owns widget appearance, not the backend.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: Color,
    pub text: Color,
    pub border: Color,
    pub font_size: f32,
}
impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color(65, 112, 230, 255),
            text: Color(255, 255, 255, 255),
            border: Color(25, 37, 65, 255),
            font_size: 18.0,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum PaintCommand<'a> {
    FillRect {
        rect: Rect,
        color: Color,
    },
    Text {
        text: &'a str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        clip: Rect,
    },
}
/// Backends provide graphics primitives and font measurement, never widget drawing.
/// The host retains ownership of its window, render target, and frame lifecycle.
pub trait Renderer {
    type Error;
    /// Physical pixels per logical UI unit, used to antialias curves and strokes.
    /// Custom renderers default to 1x; this does not alter layout coordinates.
    fn raster_scale(&self) -> (f32, f32) {
        (1.0, 1.0)
    }
    fn measure_text(&mut self, text: &str, size: f32) -> Result<(f32, f32), Self::Error>;
    fn draw(&mut self, command: PaintCommand<'_>) -> Result<(), Self::Error>;
    /// Registered asset dimensions. Returning None paints the img alt text instead.
    /// Implementations must not load files or upload textures during this lookup.
    fn image_size(&mut self, _src: &str) -> Option<(u32, u32)> {
        None
    }
    /// Draw an existing asset; source pixels and logical destination are precomputed.
    fn draw_image(&mut self, _image: crate::ImageDraw<'_>) -> Result<(), Self::Error> {
        Ok(())
    }
    /// Called in document paint order after canvas decoration, before later controls
    /// and popup overlays. Backends clip native drawing to `region.content`.
    /// The default draws no native content, keeping primitive-only backends compatible.
    fn draw_canvas(&mut self, _region: CanvasRegion<'_>) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl Ui {
    pub fn render<R: Renderer>(&self, renderer: &mut R) -> Result<(), R::Error> {
        self.render_with_theme(renderer, &Theme::default())
    }
    pub fn render_with_theme<R: Renderer>(
        &self,
        renderer: &mut R,
        theme: &Theme,
    ) -> Result<(), R::Error> {
        let pointer_target = self.pointer_target();
        for (index, scrollbar) in self.paint_order() {
            if scrollbar {
                self.paint_scrollbars(renderer, index)?;
                continue;
            }
            let control = &self.controls[index];
            let mut clipped = crate::scrolling::Clipped {
                inner: renderer,
                clip: control.clip,
            };
            let renderer = &mut clipped;
            let rect = control.rect;
            if rect.w <= 0.0 || rect.h <= 0.0 {
                continue;
            }
            let mut state = 0;
            if pointer_target == Some(index) {
                state |= css::HOVER;
            }
            if self.pointer_capture == Some(index) || self.keyboard_capture == Some(index) {
                state |= css::ACTIVE;
            }
            if self.focused == Some(index) {
                state |= css::FOCUS;
            }
            if control.disabled {
                state |= css::DISABLED;
            }
            if matches!(control.kind, ControlKind::Checkbox { checked: true, .. }) {
                state |= css::CHECKED;
            }
            let colors = control.state_styles[state as usize];
            let transparent_default = matches!(
                control.kind,
                ControlKind::Label { .. }
                    | ControlKind::Canvas(_)
                    | ControlKind::Container
                    | ControlKind::Image { .. }
                    | ControlKind::Range(_)
            );
            let font_size = control.style.font_size.unwrap_or(theme.font_size);
            let border = control.style.border_width;
            paint_box(
                renderer,
                rect,
                border,
                colors,
                if transparent_default {
                    Color(0, 0, 0, 0)
                } else {
                    theme.background
                },
                theme.border,
            )?;
            let inner = inset(rect, border);
            let content = control.content_rect();
            if content.w <= 0.0 || content.h <= 0.0 {
                continue;
            }
            let text_color = colors.text.unwrap_or(theme.text);
            match &control.kind {
                ControlKind::Container => {}
                ControlKind::Range(_) => crate::range::paint(renderer, control, state)?,
                ControlKind::Image { src, alt } => {
                    if let Some(size) = renderer.image_size(src).filter(|&(w, h)| w > 0 && h > 0) {
                        renderer.draw_image(colors.object_fit.place(src, size, content))?;
                    } else if !alt.is_empty() {
                        paint_label(renderer, alt, content, font_size, text_color)?;
                    }
                }
                ControlKind::Canvas(_) => renderer.draw_canvas(control.canvas_region().unwrap())?,
                ControlKind::Button { text } => {
                    let (width, height) = renderer.measure_text(text, font_size)?;
                    renderer.draw(PaintCommand::Text {
                        text,
                        x: content.x + (content.w - width) / 2.0,
                        y: content.y + (content.h - height) / 2.0,
                        size: font_size,
                        color: text_color,
                        clip: content,
                    })?;
                }
                ControlKind::TextInput(input) => paint_input(
                    renderer,
                    input,
                    content,
                    self.focused == Some(index),
                    font_size,
                    text_color,
                    control.selection.as_ref().map(|s| s.states[state as usize]),
                )?,
                ControlKind::Checkbox { checked: true, .. } => {
                    paint_icon(
                        renderer,
                        content,
                        crate::decoration::Icon::Check,
                        content.w.min(content.h) * 0.9,
                        (content.w.min(content.h) * 0.13).max(1.25),
                        text_color,
                    )?;
                }

                ControlKind::Checkbox { checked: false, .. } => {}
                ControlKind::Label { text, .. } => {
                    paint_label(renderer, text, content, font_size, text_color)?
                }
                ControlKind::Select { options, selected } => {
                    // The indicator is an intrinsic part of the existing select, not a new control.
                    let arrow_width = inner.w.min(22.0);
                    let arrow_area = if control.picker.is_some() {
                        Rect {
                            x: inner.x + inner.w - arrow_width,
                            y: inner.y,
                            w: arrow_width,
                            h: inner.h,
                        }
                    } else {
                        Rect {
                            x: content.x + content.w - content.w.min(18.0),
                            w: content.w.min(18.0),
                            ..content
                        }
                    };
                    let label_area = Rect {
                        w: (arrow_area.x - 4.0 - content.x).max(0.0),
                        ..content
                    };
                    if let Some(index) = selected {
                        paint_label(
                            renderer,
                            &options[*index].label,
                            label_area,
                            font_size,
                            text_color,
                        )?;
                    }
                    let mut arrow_icon = None;
                    let mut arrow_color = text_color;
                    let mut arrow_character = None;
                    let mut arrow_font_size = font_size;
                    let mut arrow_content = arrow_area;
                    if let Some(picker) = &control.picker {
                        let style = picker.states[state as usize];
                        let area = Rect {
                            w: if picker.base.rect.w > 0.0 {
                                picker.base.rect.w.min(arrow_area.w)
                            } else {
                                arrow_area.w
                            },
                            h: if picker.base.rect.h > 0.0 {
                                picker.base.rect.h.min(arrow_area.h)
                            } else {
                                arrow_area.h
                            },
                            ..arrow_area
                        };
                        let area = Rect {
                            x: area.x + (arrow_area.w - area.w) / 2.0,
                            y: area.y + (arrow_area.h - area.h) / 2.0,
                            ..area
                        };
                        paint_box(
                            renderer,
                            area,
                            picker.base.border_width,
                            style,
                            Color(0, 0, 0, 0),
                            theme.border,
                        )?;
                        arrow_color = style.text.unwrap_or(text_color);
                        arrow_icon = style
                            .icon
                            .map(|icon| (icon, style.icon_size, style.icon_stroke));
                        arrow_character = style.content;
                        arrow_font_size = picker.base.font_size.unwrap_or(font_size);
                        arrow_content = inset(area, picker.base.border_width);
                    }
                    if let Some((icon, size, stroke)) = arrow_icon {
                        paint_icon(renderer, arrow_content, icon, size, stroke, arrow_color)?;
                    } else if let Some(character) = arrow_character {
                        let mut buffer = [0; 4];
                        let text = character.encode_utf8(&mut buffer);
                        let (w, h) = renderer.measure_text(text, arrow_font_size)?;
                        renderer.draw(PaintCommand::Text {
                            text,
                            x: arrow_content.x + (arrow_content.w - w) / 2.0,
                            y: arrow_content.y + (arrow_content.h - h) / 2.0,
                            size: arrow_font_size,
                            color: arrow_color,
                            clip: arrow_content,
                        })?;
                    } else {
                        let unit = (arrow_content.w / 7.0).min(arrow_content.h / 4.0).min(1.0);
                        for row in 0..3 {
                            renderer.draw(PaintCommand::FillRect {
                                rect: Rect {
                                    x: arrow_content.x + arrow_content.w / 2.0
                                        - (3 - row) as f32 * unit,
                                    y: arrow_content.y
                                        + arrow_content.h / 2.0
                                        + (row as f32 - 1.5) * unit,
                                    w: (6 - 2 * row) as f32 * unit,
                                    h: unit,
                                },
                                color: arrow_color,
                            })?;
                        }
                    }
                }
            }
        }
        // The open popup is painted last, above every control, matching modal hit testing.
        if let Some(popup) = &self.popup
            && let ControlKind::Select { options, .. } = &self.controls[popup.control].kind
        {
            let control = &self.controls[popup.control];
            let colors = control.state_styles[css::FOCUS as usize];
            let Color(r, g, b, _) = colors.background.unwrap_or(theme.background);
            let background = Color(r, g, b, 255);
            for (index, option) in options
                .iter()
                .enumerate()
                .skip(popup.first)
                .take(POPUP_ROWS)
            {
                let rect = self.option_rect(popup.control, index - popup.first);
                let highlighted = popup.highlighted == Some(index);
                let mut state = if highlighted { css::HOVER } else { 0 };
                if option.disabled {
                    state |= css::DISABLED;
                }
                if let ControlKind::Select { selected, .. } = &control.kind
                    && *selected == Some(index)
                {
                    state |= css::CHECKED;
                }
                let style = option.states[state as usize];
                let border_color = colors.borders[0].unwrap_or(theme.border);
                paint_box(
                    renderer,
                    rect,
                    option.style.border_width,
                    style,
                    if highlighted {
                        theme.border
                    } else {
                        background
                    },
                    border_color,
                )?;
                let inner = inset(rect, option.style.border_width);
                let mut color = style.text.or(colors.text).unwrap_or(theme.text);
                if option.disabled && style.text.is_none() {
                    color.3 /= 2;
                }
                paint_label(
                    renderer,
                    &option.label,
                    Rect {
                        x: inner.x + 6.0,
                        w: (inner.w - 12.0).max(0.0),
                        ..inner
                    },
                    option
                        .style
                        .font_size
                        .or(control.style.font_size)
                        .unwrap_or(theme.font_size),
                    color,
                )?;
            }
            let first_row = self.option_rect(popup.control, 0);
            let rows = options.len().saturating_sub(popup.first).min(POPUP_ROWS);
            let popup_rect = Rect {
                h: first_row.h * rows as f32,
                ..first_row
            };
            paint_box(
                renderer,
                popup_rect,
                control.style.border_width,
                Appearance {
                    borders: colors.borders,
                    ..Appearance::default()
                },
                Color(0, 0, 0, 0),
                theme.border,
            )?;
        }
        Ok(())
    }
}

fn paint_label<R: Renderer>(
    renderer: &mut R,
    text: &str,
    content: Rect,
    size: f32,
    color: Color,
) -> Result<(), R::Error> {
    if content.w <= 0.0 || content.h <= 0.0 || text.is_empty() {
        return Ok(());
    }
    let (_, height) = renderer.measure_text(text, size)?;
    renderer.draw(PaintCommand::Text {
        text,
        x: content.x,
        y: content.y + (content.h - height) / 2.0,
        size,
        color,
        clip: content,
    })
}

fn text_width<R: Renderer>(renderer: &mut R, text: &str, size: f32) -> Result<f32, R::Error> {
    if text.is_empty() {
        Ok(0.0)
    } else {
        Ok(renderer.measure_text(text, size)?.0)
    }
}

fn paint_input<R: Renderer>(
    renderer: &mut R,
    input: &TextInput,
    content: Rect,
    focused: bool,
    font_size: f32,
    color: Color,
    selection_style: Option<Appearance>,
) -> Result<(), R::Error> {
    // Build only the visible slice, so long edited values never require huge font textures.
    let available = (content.w - 1.0).max(0.0);
    let mut start = 0;
    if focused {
        start = input.cursor;
        for (index, _) in input.value[..input.cursor].char_indices().rev() {
            if text_width(renderer, &input.value[index..input.cursor], font_size)? > available {
                break;
            }
            start = index;
        }
    }
    let mut end = start;
    for (offset, character) in input.value[start..].char_indices() {
        end = start + offset + character.len_utf8();
        if text_width(renderer, &input.value[start..end], font_size)? > available {
            break;
        }
    }
    let mut positions = input.hit_positions.borrow_mut();
    positions.clear();
    positions.push((start, content.x));
    for (offset, ch) in input.value[start..end].char_indices() {
        let index = start + offset + ch.len_utf8();
        let x = text_width(renderer, &input.value[start..index], font_size)?;
        if x <= available {
            positions.push((index, content.x + x));
        }
    }
    drop(positions);
    let selection = input.selection();
    if focused && !selection.is_empty() {
        let left = text_width(
            renderer,
            &input.value[start..selection.start.clamp(start, end)],
            font_size,
        )?
        .min(available);
        let right = text_width(
            renderer,
            &input.value[start..selection.end.clamp(start, end)],
            font_size,
        )?
        .min(available);
        renderer.draw(PaintCommand::FillRect {
            rect: Rect {
                x: content.x + left,
                w: (right - left).max(0.0),
                ..content
            },
            color: selection_style
                .and_then(|s| s.background)
                .unwrap_or(Color(100, 150, 240, 120)),
        })?;
    }
    if input.value.is_empty() {
        let mut placeholder_color = color;
        placeholder_color.3 /= 2;
        paint_label(
            renderer,
            &input.placeholder,
            content,
            font_size,
            placeholder_color,
        )?;
    } else {
        paint_label(
            renderer,
            &input.value[start..end],
            content,
            font_size,
            color,
        )?;
    }
    if focused
        && !selection.is_empty()
        && let Some(selected_color) = selection_style.and_then(|s| s.text)
    {
        let left = text_width(
            renderer,
            &input.value[start..selection.start.clamp(start, end)],
            font_size,
        )?
        .min(available);
        let right = text_width(
            renderer,
            &input.value[start..selection.end.clamp(start, end)],
            font_size,
        )?
        .min(available);
        let (_, height) = renderer.measure_text(&input.value[start..end], font_size)?;
        renderer.draw(PaintCommand::Text {
            text: &input.value[start..end],
            x: content.x,
            y: content.y + (content.h - height) / 2.0,
            size: font_size,
            color: selected_color,
            clip: Rect {
                x: content.x + left,
                w: (right - left).max(0.0),
                ..content
            },
        })?;
    }
    if focused && !input.readonly {
        let cursor =
            text_width(renderer, &input.value[start..input.cursor], font_size)?.min(available);
        let height = content.h.min(font_size);
        renderer.draw(PaintCommand::FillRect {
            rect: Rect {
                x: content.x + cursor,
                y: content.y + (content.h - height) / 2.0,
                w: content.w.min(1.0),
                h: height,
            },
            color,
        })?;
    }
    Ok(())
}
