//! Native drawing regions. The host owns graphics resources and the frame loop.
use crate::{Control, ControlKind, Rect, Ui};

/// CSS-space rectangles for one native drawing callback. No pixels or GPU resources
/// are owned by this value. `content` excludes the CSS border and padding.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasRegion<'a> {
    pub id: Option<&'a str>,
    pub bounds: Rect,
    pub content: Rect,
    /// Visible content after ancestor overflow clipping. Bounds/content retain their full geometry.
    pub clip: Rect,
}
impl CanvasRegion<'_> {
    /// Convert UI coordinates to coordinates relative to the drawing content.
    /// Values outside the region are useful during captured drags.
    pub fn to_local(self, x: f32, y: f32) -> (f32, f32) {
        (x - self.content.x, y - self.content.y)
    }
}

/// Polling state for the primary pointer. Edge flags last until `Ui::end_frame`.
/// Positions are relative to the content box, in CSS pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CanvasInput {
    pub pointer: Option<(f32, f32)>,
    pub hovered: bool,
    pub captured: bool,
    pub focused: bool,
    pub pressed: Option<(f32, f32)>,
    pub released: Option<(f32, f32)>,
    pub cancelled: bool,
}

#[derive(Debug, Default)]
pub(crate) struct CanvasState {
    pub id: Option<String>,
    pub tab_stop: bool,
    pub pressed: Option<(f32, f32)>,
    pub released: Option<(f32, f32)>,
    pub cancelled: bool,
}
impl Control {
    pub(crate) fn content_rect(&self) -> Rect {
        let inner = crate::decoration::inset(self.rect, self.style.border_width);
        let [top, right, bottom, left] = self.style.padding;
        Rect {
            x: inner.x + left,
            y: inner.y + top,
            w: (inner.w - left - right).max(0.0),
            h: (inner.h - top - bottom).max(0.0),
        }
    }
    pub(crate) fn canvas_region(&self) -> Option<CanvasRegion<'_>> {
        let ControlKind::Canvas(state) = &self.kind else {
            return None;
        };
        Some(CanvasRegion {
            id: state.id.as_deref(),
            bounds: self.rect,
            content: self.content_rect(),
            clip: self.clip.intersect(self.content_rect()),
        })
    }
}
impl Ui {
    /// None for an unknown ID or a non-canvas control.
    pub fn canvas_region(&self, id: &str) -> Option<CanvasRegion<'_>> {
        self.controls.get(*self.ids.get(id)?)?.canvas_region()
    }
    pub fn canvas_input(&self, id: &str) -> Option<CanvasInput> {
        let index = *self.ids.get(id)?;
        let control = &self.controls[index];
        let ControlKind::Canvas(state) = &control.kind else {
            return None;
        };
        let region = control.canvas_region()?;
        let hovered = self.pointer_target() == Some(index)
            && self
                .pointer
                .is_some_and(|(x, y)| region.content.contains(x, y) && region.clip.contains(x, y));
        let captured = self.pointer_capture == Some(index);
        Some(CanvasInput {
            pointer: self
                .pointer
                .filter(|_| hovered || captured)
                .map(|(x, y)| region.to_local(x, y)),
            hovered,
            captured,
            focused: self.focused == Some(index),
            pressed: state.pressed,
            released: state.released,
            cancelled: state.cancelled,
        })
    }
}
