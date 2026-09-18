use crate::{
    CanvasRegion, Color, Control, ControlKind, Error, ImageDraw, Key, PaintCommand, Rect, Renderer,
    Ui, css,
    decoration::{Appearance, inset, paint_box},
    parts::metric,
};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Overflow {
    #[default]
    Visible,
    Hidden,
    Clip,
    Auto,
    Scroll,
}
impl Overflow {
    pub fn clips(self) -> bool {
        self != Self::Visible
    }
    pub fn scrolls(self) -> bool {
        matches!(self, Self::Hidden | Self::Auto | Self::Scroll)
    }
    fn wheel(self) -> bool {
        matches!(self, Self::Auto | Self::Scroll)
    }
}
pub(crate) fn normalize(mut v: [Overflow; 2]) -> [Overflow; 2] {
    if v.iter()
        .any(|v| !matches!(v, Overflow::Visible | Overflow::Clip))
    {
        for axis in &mut v {
            *axis = match *axis {
                Overflow::Visible => Overflow::Auto,
                Overflow::Clip => Overflow::Hidden,
                v => v,
            };
        }
    }
    v
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Clip {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}
impl Default for Clip {
    fn default() -> Self {
        Self {
            left: f32::NEG_INFINITY,
            top: f32::NEG_INFINITY,
            right: f32::INFINITY,
            bottom: f32::INFINITY,
        }
    }
}
impl Clip {
    pub fn intersect(self, r: Rect) -> Rect {
        let x = r.x.max(self.left);
        let y = r.y.max(self.top);
        Rect {
            x,
            y,
            w: (r.x + r.w).min(self.right).max(x) - x,
            h: (r.y + r.h).min(self.bottom).max(y) - y,
        }
    }
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
    fn add(mut self, r: Rect, axes: [Overflow; 2]) -> Self {
        if axes[0].clips() {
            self.left = self.left.max(r.x);
            self.right = self.right.min(r.x + r.w)
        }
        if axes[1].clips() {
            self.top = self.top.max(r.y);
            self.bottom = self.bottom.min(r.y + r.h)
        }
        self
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct ScrollState {
    pub offset: [f32; 2],
    pub max: [f32; 2],
    pub viewport: Rect,
    pub gutter: [f32; 2],
    pub shown: [bool; 2],
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct Drag {
    pub index: usize,
    pub axis: usize,
    pub grab: f32,
}
#[derive(Clone, Copy)]
pub(crate) struct Bar {
    pub index: usize,
    pub axis: usize,
    pub track: Rect,
    pub thumb: Rect,
}
fn viewport(c: &Control, rect: Rect, g: [f32; 2]) -> Rect {
    let mut r = inset(rect, c.style.border_width);
    r.w = (r.w - g[0]).max(0.0);
    r.h = (r.h - g[1]).max(0.0);
    r
}
/// Determine scroll extents from unscrolled boxes. Clipped descendants do not
/// inflate an ancestor's extent beyond the clipping descendant's own box.
pub(crate) fn metrics(
    controls: &[Control],
    tree: &crate::layout::Tree,
    rects: &[Rect],
    gutters: &[[f32; 2]],
) -> Vec<ScrollState> {
    let mut extents: Vec<_> = rects.iter().map(|r| [r.x + r.w, r.y + r.h]).collect();
    let mut states = vec![ScrollState::default(); controls.len()];
    for i in (0..controls.len()).rev() {
        let c = &controls[i];
        let r = rects[i];
        let view = viewport(c, r, gutters[i]);
        let mut end = [view.x, view.y];
        for &child in &tree.nodes[i].children {
            let cr = rects[child];
            let cc = &controls[child];
            let outer = [
                cr.x + cr.w + cc.style.layout.margin[1],
                cr.y + cr.h + cc.style.layout.margin[2],
            ];
            for a in 0..2 {
                end[a] = end[a].max(if cc.style.overflow[a].clips() {
                    outer[a]
                } else {
                    outer[a].max(extents[child][a])
                });
            }
        }
        if !tree.nodes[i].children.is_empty() {
            end[0] += c.style.padding[1];
            end[1] += c.style.padding[2];
        }
        let max = [
            (end[0] - view.x - view.w).max(0.0),
            (end[1] - view.y - view.h).max(0.0),
        ];
        let shown = [0, 1].map(|a| {
            matches!(c.style.overflow[a], Overflow::Scroll)
                || c.style.overflow[a] == Overflow::Auto && max[a] > 0.01
        });
        let inner = inset(r, c.style.border_width);
        let gutter = [
            if shown[1] {
                metric(&c.parts.scrollbar, 0, 14.0).min(inner.w)
            } else {
                0.0
            },
            if shown[0] {
                metric(&c.parts.scrollbar, 1, 14.0).min(inner.h)
            } else {
                0.0
            },
        ];
        states[i] = ScrollState {
            offset: c.scroll.offset,
            max: [0, 1].map(|a| {
                if c.style.overflow[a].scrolls() {
                    max[a]
                } else {
                    0.0
                }
            }),
            viewport: view,
            gutter,
            shown,
        };
        extents[i][0] = extents[i][0].max(end[0]);
        extents[i][1] = extents[i][1].max(end[1]);
    }
    states
}
impl Ui {
    pub(crate) fn paint_order(&self) -> impl DoubleEndedIterator<Item = (usize, bool)> + '_ {
        (0..if self.layout.is_none() {
            self.controls.len()
        } else {
            0
        })
            .map(|i| (i, false))
            .chain(
                self.layout
                    .iter()
                    .flat_map(|t| t.paint_order.iter().copied()),
            )
    }
    pub(crate) fn refresh_scroll_positions(&mut self) {
        let Some(tree) = &self.layout else { return };
        let mut shifts = vec![[0.0, 0.0]; self.controls.len()];
        for i in 0..self.controls.len() {
            let (shift, clip) = if let Some(p) = tree.nodes[i].parent {
                let parent = &self.controls[p];
                (
                    [
                        shifts[p][0] + parent.scroll.offset[0],
                        shifts[p][1] + parent.scroll.offset[1],
                    ],
                    parent
                        .clip
                        .add(parent.scroll.viewport, parent.style.overflow),
                )
            } else {
                ([0.0; 2], Clip::default())
            };
            shifts[i] = shift;
            let c = &mut self.controls[i];
            c.rect = Rect {
                x: c.layout_rect.x - shift[0],
                y: c.layout_rect.y - shift[1],
                ..c.layout_rect
            };
            c.clip = clip;
            c.scroll.viewport = viewport(c, c.rect, c.scroll.gutter);
            for a in 0..2 {
                c.scroll.offset[a] = c.scroll.offset[a].clamp(0.0, c.scroll.max[a]);
            }
            if let ControlKind::TextInput(input) = &c.kind {
                input.hit_positions.borrow_mut().clear();
            }
        }
    }
    pub fn scroll_offset(&self, id: &str) -> Option<(f32, f32)> {
        let c = &self.controls[*self.ids.get(id)?];
        matches!(c.kind, ControlKind::Container).then_some((c.scroll.offset[0], c.scroll.offset[1]))
    }
    pub fn scroll_max(&self, id: &str) -> Option<(f32, f32)> {
        let c = &self.controls[*self.ids.get(id)?];
        matches!(c.kind, ControlKind::Container).then_some((c.scroll.max[0], c.scroll.max[1]))
    }
    pub fn set_scroll_offset(&mut self, id: &str, x: f32, y: f32) -> Result<(), Error> {
        if !x.is_finite() || !y.is_finite() {
            return Err(Error::new("scroll offsets must be finite"));
        }
        let i = *self
            .ids
            .get(id)
            .ok_or_else(|| Error::new("unknown scroll container"))?;
        if !matches!(self.controls[i].kind, ControlKind::Container) {
            return Err(Error::new("scrolling requires a container"));
        }
        if self.move_scroll(i, [x, y]) {
            self.cancel_layout_captures();
            self.refresh_scroll_positions();
            self.invalidate_paint();
        }
        Ok(())
    }
    fn move_scroll(&mut self, i: usize, next: [f32; 2]) -> bool {
        let c = &mut self.controls[i];
        let next = [0, 1].map(|a| next[a].clamp(0.0, c.scroll.max[a]));
        if next == c.scroll.offset {
            return false;
        }
        c.scroll.offset = next;
        true
    }
    pub(crate) fn bars(&self, i: usize) -> Vec<Bar> {
        let c = &self.controls[i];
        let s = c.scroll;
        let inner = inset(c.rect, c.style.border_width);
        let mut bars = Vec::new();
        for a in 0..2 {
            if !s.shown[a] {
                continue;
            }
            let track = if a == 0 {
                Rect {
                    x: inner.x,
                    y: inner.y + s.viewport.h,
                    w: s.viewport.w,
                    h: s.gutter[1],
                }
            } else {
                Rect {
                    x: inner.x + s.viewport.w,
                    y: inner.y,
                    w: s.gutter[0],
                    h: s.viewport.h,
                }
            };
            let length = if a == 0 { track.w } else { track.h };
            let view = if a == 0 { s.viewport.w } else { s.viewport.h };
            let thumb_length = (length * view / (view + s.max[a]).max(1.0))
                .max(metric(&c.parts.scroll_thumb, a, 20.0))
                .min(length);
            let start = if s.max[a] > 0.0 {
                (length - thumb_length) * s.offset[a] / s.max[a]
            } else {
                0.0
            };
            let thumb = if a == 0 {
                Rect {
                    x: track.x + start,
                    w: thumb_length,
                    ..track
                }
            } else {
                Rect {
                    y: track.y + start,
                    h: thumb_length,
                    ..track
                }
            };
            bars.push(Bar {
                index: i,
                axis: a,
                track,
                thumb,
            });
        }
        bars
    }
    pub(crate) fn scrollbar_hit(&self, x: f32, y: f32) -> Option<Bar> {
        for (i, bar) in self.paint_order().rev() {
            let c = &self.controls[i];
            if !c.clip.contains(x, y) {
                continue;
            }
            if bar {
                if let Some(b) = self.bars(i).into_iter().find(|b| b.track.contains(x, y)) {
                    return Some(b);
                }
            } else if c.rect.contains(x, y) {
                return None;
            }
        }
        None
    }
    pub(crate) fn scrollbar_down(&mut self, x: f32, y: f32) -> bool {
        let Some(bar) = self.scrollbar_hit(x, y) else {
            return false;
        };
        let a = bar.axis;
        let i = bar.index;
        self.focused = Some(i);
        if bar.thumb.contains(x, y) {
            let grab = if a == 0 {
                x - bar.thumb.x
            } else {
                y - bar.thumb.y
            };
            self.scroll_capture = Some(Drag {
                index: i,
                axis: a,
                grab,
            });
        } else {
            let s = self.controls[i].scroll;
            let p = if a == 0 { x } else { y };
            let start = if a == 0 { bar.thumb.x } else { bar.thumb.y };
            let size = if a == 0 { s.viewport.w } else { s.viewport.h };
            let mut next = s.offset;
            next[a] += if p < start { -size } else { size };
            if self.move_scroll(i, next) {
                self.refresh_scroll_positions();
            }
        }
        self.invalidate_paint();
        true
    }
    pub(crate) fn scrollbar_move(&mut self, x: f32, y: f32) -> bool {
        let Some(d) = self.scroll_capture else {
            return false;
        };
        let Some(bar) = self.bars(d.index).into_iter().find(|b| b.axis == d.axis) else {
            return false;
        };
        let (p, start, len, thumb) = if d.axis == 0 {
            (x, bar.track.x, bar.track.w, bar.thumb.w)
        } else {
            (y, bar.track.y, bar.track.h, bar.thumb.h)
        };
        let mut next = self.controls[d.index].scroll.offset;
        next[d.axis] = if len > thumb {
            ((p - d.grab - start) / (len - thumb)).clamp(0.0, 1.0)
                * self.controls[d.index].scroll.max[d.axis]
        } else {
            0.0
        };
        if self.move_scroll(d.index, next) {
            self.refresh_scroll_positions();
            self.invalidate_paint();
        }
        true
    }
    /// Logical-pixel wheel deltas. Positive moves content toward its right/bottom.
    /// Nested containers consume only the distance they can scroll, then chain outward.
    pub fn scroll_wheel(&mut self, dx: f32, dy: f32) {
        if !dx.is_finite() || !dy.is_finite() {
            return;
        }
        if self.popup.is_some() {
            if dy != 0.0 {
                self.scroll((dy / 40.0).round() as i32)
            }
            return;
        }
        let Some(mut i) = self.pointer.and_then(|(x, y)| self.hit(x, y)) else {
            return;
        };
        let mut delta = [dx, dy];
        let mut changed = false;
        loop {
            let c = &self.controls[i];
            let old = c.scroll.offset;
            let mut next = old;
            for a in 0..2 {
                if c.style.overflow[a].wheel() {
                    next[a] = (old[a] + delta[a]).clamp(0.0, c.scroll.max[a]);
                    delta[a] -= next[a] - old[a];
                }
            }
            changed |= self.move_scroll(i, next);
            let Some(parent) = self.layout.as_ref().and_then(|t| t.nodes[i].parent) else {
                break;
            };
            i = parent;
        }
        if changed {
            self.cancel_layout_captures();
            self.refresh_scroll_positions();
            self.invalidate_paint();
        }
    }
    pub(crate) fn scroll_key(&mut self, i: usize, key: Key) -> bool {
        if !matches!(self.controls[i].kind, ControlKind::Container) {
            return false;
        }
        let s = self.controls[i].scroll;
        let mut next = s.offset;
        match key {
            Key::Left => next[0] -= 40.0,
            Key::Right => next[0] += 40.0,
            Key::Up => next[1] -= 40.0,
            Key::Down => next[1] += 40.0,
            Key::PageUp => next[1] -= s.viewport.h,
            Key::PageDown | Key::Space => next[1] += s.viewport.h,
            Key::Home => next = [0.0; 2],
            Key::End => next = s.max,
            _ => return false,
        }
        for (a, value) in next.iter_mut().enumerate() {
            if !self.controls[i].style.overflow[a].wheel() {
                *value = s.offset[a];
            }
        }
        if self.move_scroll(i, next) {
            self.refresh_scroll_positions();
            self.invalidate_paint();
        }
        true
    }
    pub(crate) fn reveal_focus(&mut self) {
        let Some(target) = self.focused else { return };
        let mut i = target;
        let mut changed = false;
        while let Some(p) = self.layout.as_ref().and_then(|t| t.nodes[i].parent) {
            let c = &self.controls[p];
            let view = c.scroll.viewport;
            let r = self.controls[target].rect;
            let mut next = c.scroll.offset;
            for (a, value) in next.iter_mut().enumerate() {
                if c.style.overflow[a].scrolls() {
                    let (start, end, vs, ve) = if a == 0 {
                        (r.x, r.x + r.w, view.x, view.x + view.w)
                    } else {
                        (r.y, r.y + r.h, view.y, view.y + view.h)
                    };
                    *value += if start < vs {
                        start - vs
                    } else if end > ve {
                        (end - ve).min(start - vs)
                    } else {
                        0.0
                    };
                }
            }
            if self.move_scroll(p, next) {
                changed = true;
                self.refresh_scroll_positions();
            }
            i = p;
        }
        if changed {
            self.invalidate_paint();
        }
    }
    pub(crate) fn paint_scrollbars<R: Renderer>(
        &self,
        r: &mut R,
        i: usize,
    ) -> Result<(), R::Error> {
        let c = &self.controls[i];
        if !c.scroll.shown.iter().any(|s| *s) {
            return Ok(());
        }
        let mut renderer = Clipped {
            inner: r,
            clip: c.clip,
        };
        for b in self.bars(i) {
            let hovered = self
                .pointer
                .is_some_and(|(x, y)| b.track.contains(x, y) && c.clip.contains(x, y));
            let active = self
                .scroll_capture
                .is_some_and(|d| d.index == i && d.axis == b.axis);
            let state = (if hovered { css::HOVER } else { 0 })
                | (if active { css::ACTIVE } else { 0 })
                | (if self.focused == Some(i) {
                    css::FOCUS
                } else {
                    0
                });
            for (part, rect, color) in [
                (&c.parts.scrollbar, b.track, Color(238, 239, 241, 255)),
                (&c.parts.scroll_track, b.track, Color(0, 0, 0, 0)),
                (&c.parts.scroll_thumb, b.thumb, Color(153, 158, 165, 255)),
            ] {
                let a = part.as_ref().map_or(
                    Appearance {
                        radius: if std::ptr::eq(part, &c.parts.scroll_thumb) {
                            6.0
                        } else {
                            0.0
                        },
                        ..Default::default()
                    },
                    |p| p.states[state as usize],
                );
                let border = part.as_ref().map_or(0.0, |p| p.base.border_width);
                paint_box(
                    &mut renderer,
                    rect,
                    border,
                    a,
                    color,
                    Color(150, 150, 150, 255),
                )?;
            }
        }
        if c.scroll.shown == [true, true] {
            let v = c.scroll.viewport;
            let corner = Rect {
                x: v.x + v.w,
                y: v.y + v.h,
                w: c.scroll.gutter[0],
                h: c.scroll.gutter[1],
            };
            let p = &c.parts.scroll_corner;
            paint_box(
                &mut renderer,
                corner,
                p.as_ref().map_or(0.0, |p| p.base.border_width),
                p.as_ref().map_or(Appearance::default(), |p| p.states[0]),
                Color(238, 239, 241, 255),
                Color(150, 150, 150, 255),
            )?;
        }
        Ok(())
    }
}
/// Clip primitive commands centrally, so all renderers get identical scrolling.
/// Native canvas bounds remain unchanged; its separate clip carries visibility.
pub(crate) struct Clipped<'a, R> {
    pub inner: &'a mut R,
    pub clip: Clip,
}
impl<R: Renderer> Renderer for Clipped<'_, R> {
    type Error = R::Error;
    fn raster_scale(&self) -> (f32, f32) {
        self.inner.raster_scale()
    }
    fn measure_text(&mut self, t: &str, s: f32) -> Result<(f32, f32), R::Error> {
        self.inner.measure_text(t, s)
    }
    fn draw(&mut self, cmd: PaintCommand<'_>) -> Result<(), R::Error> {
        let cmd = match cmd {
            PaintCommand::FillRect { rect, color } => {
                let rect = self.clip.intersect(rect);
                if rect.w <= 0.0 || rect.h <= 0.0 {
                    return Ok(());
                }
                PaintCommand::FillRect { rect, color }
            }
            PaintCommand::Text {
                text,
                x,
                y,
                size,
                color,
                clip,
            } => {
                let clip = self.clip.intersect(clip);
                if clip.w <= 0.0 || clip.h <= 0.0 {
                    return Ok(());
                }
                PaintCommand::Text {
                    text,
                    x,
                    y,
                    size,
                    color,
                    clip,
                }
            }
        };
        self.inner.draw(cmd)
    }
    fn image_size(&mut self, src: &str) -> Option<(u32, u32)> {
        self.inner.image_size(src)
    }
    fn draw_image(&mut self, mut i: ImageDraw<'_>) -> Result<(), R::Error> {
        i.clip = self.clip.intersect(i.clip);
        if i.clip.w <= 0.0 || i.clip.h <= 0.0 {
            return Ok(());
        }
        self.inner.draw_image(i)
    }
    fn draw_canvas(&mut self, mut c: CanvasRegion<'_>) -> Result<(), R::Error> {
        c.clip = self.clip.intersect(c.clip);
        if c.clip.w <= 0.0 || c.clip.h <= 0.0 {
            return Ok(());
        }
        self.inner.draw_canvas(c)
    }
}
