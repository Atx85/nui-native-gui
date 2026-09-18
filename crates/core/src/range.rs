use crate::{
    Color, Control, ControlKind, Error, Key, Rect, Renderer, Ui, css,
    decoration::{Appearance, paint_box},
    html::Node,
    parts::metric,
};
#[derive(Debug)]
pub(crate) struct RangeInput {
    pub min: f64,
    pub max: f64,
    pub step: Option<f64>,
    pub value: f64,
    pub text: String,
    pub drag_offset: f32,
}
fn number(s: &str) -> Result<f64, Error> {
    s.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| Error::new("range numbers must be finite"))
}
impl RangeInput {
    pub fn compile(node: &Node) -> Result<Self, Error> {
        if ["checked", "readonly", "placeholder", "maxlength"]
            .iter()
            .any(|a| node.attr(a).is_some())
        {
            return Err(Error::new(
                "range does not support text or checkbox attributes",
            ));
        }
        let attr = |n, default| node.attr(n).map(|s| number(&s)).unwrap_or(Ok(default));
        let min = attr("min", 0.0)?;
        let max = attr("max", 100.0)?;
        if max < min || !(max - min).is_finite() {
            return Err(Error::new(
                "range max must be at least min, with a finite span",
            ));
        }
        let step = match node.attr("step").as_deref() {
            Some("any") => None,
            Some(s) => Some(number(s)?),
            None => Some(1.0),
        };
        if step.is_some_and(|s| s <= 0.0 || (max - min) / s > 1e15) {
            return Err(Error::new(
                "range step must be positive and provide at most 1e15 increments",
            ));
        }
        let mut r = Self {
            min,
            max,
            step,
            value: min,
            text: String::new(),
            drag_offset: 0.0,
        };
        r.set(attr("value", min + (max - min) / 2.0)?);
        Ok(r)
    }
    fn normalize(&self, v: f64) -> f64 {
        let v = v.clamp(self.min, self.max);
        let v = if let Some(s) = self.step {
            let count = ((self.max - self.min) / s + 1e-9).floor();
            self.min + ((v - self.min) / s).round().clamp(0.0, count) * s
        } else {
            v
        };
        v.clamp(self.min, self.max)
    }
    pub fn set(&mut self, v: f64) -> bool {
        let next = self.normalize(v);
        let changed = next != self.value;
        self.value = next;
        self.text = if next == 0.0 {
            "0".into()
        } else {
            next.to_string()
        };
        changed
    }
    pub fn set_text(&mut self, s: &str) -> Result<bool, Error> {
        Ok(self.set(number(s)?))
    }
    pub fn fraction(&self) -> f32 {
        if self.max == self.min {
            0.0
        } else {
            ((self.value - self.min) / (self.max - self.min)) as f32
        }
    }
    pub fn key(&mut self, key: Key) -> bool {
        let step = self.step.unwrap_or((self.max - self.min) / 100.0);
        let value = match key {
            Key::Left | Key::Down => self.value - step,
            Key::Right | Key::Up => self.value + step,
            Key::PageDown => self.value - step * 10.0,
            Key::PageUp => self.value + step * 10.0,
            Key::Home => self.min,
            Key::End => self.max,
            _ => return false,
        };
        self.set(value)
    }
}
impl Control {
    pub(crate) fn range_geometry(&self) -> (Rect, Rect) {
        let c = self.content_rect();
        let tw = metric(&self.parts.range_thumb, 0, 16.0).min(c.w);
        let th = metric(&self.parts.range_thumb, 1, 16.0).min(c.h);
        let w = metric(&self.parts.range_track, 0, (c.w - tw).max(0.0)).min((c.w - tw).max(0.0));
        let h = metric(&self.parts.range_track, 1, 4.0).min(c.h);
        let track = Rect {
            x: c.x + (c.w - w) / 2.0,
            y: c.y + (c.h - h) / 2.0,
            w,
            h,
        };
        let fraction = if let ControlKind::Range(r) = &self.kind {
            r.fraction()
        } else {
            0.0
        };
        (
            track,
            Rect {
                x: track.x + fraction * w - tw / 2.0,
                y: c.y + (c.h - th) / 2.0,
                w: tw,
                h: th,
            },
        )
    }
}
impl Ui {
    pub(crate) fn range_pointer(&mut self, i: usize, x: f32, down: bool) {
        let (track, thumb) = self.controls[i].range_geometry();
        let on_thumb = self.pointer.is_some_and(|(x, y)| thumb.contains(x, y));
        if let ControlKind::Range(r) = &mut self.controls[i].kind {
            if down {
                r.drag_offset = if on_thumb {
                    x - (thumb.x + thumb.w / 2.0)
                } else {
                    0.0
                };
            }
            if down && on_thumb {
                return;
            }
            let f = if track.w > 0.0 {
                ((x - r.drag_offset - track.x) / track.w).clamp(0.0, 1.0) as f64
            } else {
                0.0
            };
            let changed = r.set(r.min + (r.max - r.min) * f);
            self.controls[i].changed |= changed;
            if changed {
                self.invalidate_paint();
            }
        }
    }
}
pub(crate) fn paint<R: Renderer>(r: &mut R, c: &Control, state: u8) -> Result<(), R::Error> {
    let (track, thumb) = c.range_geometry();
    let f = if let ControlKind::Range(v) = &c.kind {
        v.fraction()
    } else {
        0.0
    };
    for (part, rect, fallback) in [
        (&c.parts.range_track, track, Color(180, 185, 193, 255)),
        (
            &c.parts.range_fill,
            Rect {
                w: track.w * f,
                ..track
            },
            Color(55, 120, 220, 255),
        ),
        (&c.parts.range_thumb, thumb, Color(240, 242, 245, 255)),
    ] {
        let appearance = part.as_ref().map_or(
            Appearance {
                radius: 8.0,
                ..Default::default()
            },
            |p| p.states[state as usize],
        );
        let border = part.as_ref().map_or(0.0, |p| p.base.border_width);
        let fallback = if state & css::DISABLED != 0 {
            Color(190, 190, 190, 255)
        } else {
            fallback
        };
        paint_box(
            r,
            rect,
            border,
            appearance,
            fallback,
            Color(130, 135, 140, 255),
        )?;
    }
    Ok(())
}
