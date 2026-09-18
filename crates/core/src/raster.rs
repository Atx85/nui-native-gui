//! Coverage rasterization of CSS boxes on the backend's physical-pixel grid.
//! Only curve/border transition cells need separate spans: a flat interior remains
//! one rectangle per row, independent of its width. Border and fill are integrated
//! together so their shared antialiased edge cannot leave a translucent seam.
use crate::{
    Color, Rect, Renderer,
    decoration::{Appearance, Gradient, InsetShadow, inset},
};

#[derive(Clone, Copy)]
struct Span {
    left: f32,
    right: f32,
    color: [f32; 4],
}
fn premultiplied(Color(r, g, b, a): Color) -> [f32; 4] {
    let a = a as f32 / 255.;
    [r as f32 * a, g as f32 * a, b as f32 * a, a]
}
fn over(fg: Color, bg: Color) -> Color {
    let f = premultiplied(fg);
    let b = premultiplied(bg);
    straight(std::array::from_fn(|i| f[i] + b[i] * (1. - f[3])))
}
fn straight(p: [f32; 4]) -> Color {
    if p[3] <= 0. {
        return Color(0, 0, 0, 0);
    }
    Color(
        (p[0] / p[3]).round() as u8,
        (p[1] / p[3]).round() as u8,
        (p[2] / p[3]).round() as u8,
        (p[3] * 255.).round() as u8,
    )
}
fn extent(r: Rect, radius: f32, y: f32) -> Option<(f32, f32)> {
    if y < r.y || y >= r.y + r.h || r.w <= 0. || r.h <= 0. {
        return None;
    }
    let radius = radius.max(0.).min(r.w / 2.).min(r.h / 2.);
    let d = (radius - (y - r.y).min(r.y + r.h - y)).max(0.);
    let cut = radius - (radius * radius - d * d).max(0.).sqrt();
    Some((r.x + cut, r.x + r.w - cut))
}
fn span(spans: &mut Vec<Span>, left: f32, right: f32, color: Color) {
    if right > left && color.3 > 0 {
        spans.push(Span {
            left,
            right,
            color: premultiplied(color),
        });
    }
}
struct Interior {
    rect: Rect,
    radius: f32,
    background: Color,
    image: Option<Gradient>,
    shadow: Option<InsetShadow>,
}
impl Interior {
    fn spans(&self, spans: &mut Vec<Span>, left: f32, right: f32, y: f32) {
        let Self {
            rect: inner,
            radius,
            background,
            image,
            shadow,
        } = *self;
        let color = image.map_or(background, |g| {
            over(g.sample((y - inner.y) / inner.h), background)
        });
        if let Some(shadow) = shadow.filter(|s| s.color.3 > 0) {
            let mut hole = inset(inner, shadow.spread);
            hole.x += shadow.x;
            hole.y += shadow.y;
            let shade = over(shadow.color, color);
            if let Some((hl, hr)) = extent(hole, radius - shadow.spread, y) {
                let hl = hl.clamp(left, right);
                let hr = hr.clamp(hl, right);
                span(spans, left, hl, shade);
                span(spans, hl, hr, color);
                span(spans, hr, right, shade);
            } else {
                span(spans, left, right, shade);
            }
        } else {
            span(spans, left, right, color);
        }
    }
}

pub(crate) fn scale<R: Renderer>(r: &R) -> (f32, f32) {
    let valid = |v: f32| if v.is_finite() && v > 0. { v } else { 1. };
    let (x, y) = r.raster_scale();
    (valid(x), valid(y))
}
fn transpose(r: Rect) -> Rect {
    Rect {
        x: r.y,
        y: r.x,
        w: r.h,
        h: r.w,
    }
}
pub(crate) fn paint_box<R: Renderer>(
    r: &mut R,
    mut rect: Rect,
    border: f32,
    mut appearance: Appearance,
    background: Color,
    border_color: Color,
) -> Result<(), R::Error> {
    if rect.w <= 0. || rect.h <= 0. {
        return Ok(());
    }
    let mut scale = scale(r);
    // Horizontal gradients use the same span algorithm with transposed axes;
    // their cost is proportional to width rather than width * height.
    let horizontal = appearance.image.is_some_and(|g| g.horizontal);
    if horizontal {
        rect = transpose(rect);
        scale = (scale.1, scale.0);
        appearance.borders = [
            appearance.borders[3],
            appearance.borders[2],
            appearance.borders[1],
            appearance.borders[0],
        ];
        if let Some(s) = appearance.shadow.as_mut() {
            std::mem::swap(&mut s.x, &mut s.y);
        }
    }
    let border = border.max(0.).min(rect.w / 2.).min(rect.h / 2.);
    let radius = appearance.radius.max(0.).min(rect.w / 2.).min(rect.h / 2.);
    let inner = inset(rect, border);
    let inner_radius = (radius - border).max(0.);
    let colors = appearance.borders.map(|c| c.unwrap_or(border_color));
    let background = appearance.background.unwrap_or(background);
    // Exact, grid-aligned square boxes need no coverage rasterization.
    if radius == 0.
        && appearance.image.is_none()
        && appearance.shadow.is_none()
        && [
            rect.x * scale.0,
            (rect.x + rect.w) * scale.0,
            inner.x * scale.0,
            (inner.x + inner.w) * scale.0,
            rect.y * scale.1,
            (rect.y + rect.h) * scale.1,
            inner.y * scale.1,
            (inner.y + inner.h) * scale.1,
        ]
        .iter()
        .all(|v| v.fract().abs() < 0.0001)
    {
        for (rect, color) in [
            (Rect { h: border, ..rect }, colors[0]),
            (
                Rect {
                    y: rect.y + rect.h - border,
                    h: border,
                    ..rect
                },
                colors[2],
            ),
            (
                Rect {
                    y: inner.y,
                    w: border,
                    h: inner.h,
                    ..rect
                },
                colors[3],
            ),
            (
                Rect {
                    x: rect.x + rect.w - border,
                    y: inner.y,
                    w: border,
                    h: inner.h,
                },
                colors[1],
            ),
            (inner, background),
        ] {
            emit(r, rect, color, horizontal)?;
        }
        return Ok(());
    }
    let interior = Interior {
        rect: inner,
        radius: inner_radius,
        background,
        image: appearance.image,
        shadow: appearance.shadow,
    };
    let top = (rect.y * scale.1).floor();
    let bottom = ((rect.y + rect.h) * scale.1).ceil();
    // Preserve the previous safety bound for enormous host-provided boxes.
    let step = ((bottom - top) / 8192.).ceil().max(1.);
    let mut row = top;
    let mut spans = Vec::with_capacity(48);
    let mut edges = Vec::with_capacity(192);
    const SAMPLES: usize = 32;
    while row < bottom {
        let height = step.min(bottom - row);
        spans.clear();
        edges.clear();
        for sample in 0..SAMPLES {
            let y = (row + (sample as f32 + 0.5) * height / SAMPLES as f32) / scale.1;
            let Some((left, right)) = extent(rect, radius, y) else {
                continue;
            };
            if let Some((il, ir)) = extent(inner, inner_radius, y) {
                let il = il.clamp(left, right);
                let ir = ir.clamp(il, right);
                span(&mut spans, left, il, colors[3]);
                interior.spans(&mut spans, il, ir, y);
                span(&mut spans, ir, right, colors[1]);
            } else {
                span(
                    &mut spans,
                    left,
                    right,
                    if y < inner.y { colors[0] } else { colors[2] },
                );
            }
        }
        for s in &mut spans {
            s.left *= scale.0;
            s.right *= scale.0;
            edges.extend([
                s.left.floor(),
                s.left.ceil(),
                s.right.floor(),
                s.right.ceil(),
            ]);
        }
        edges.sort_unstable_by(f32::total_cmp);
        edges.dedup();
        let mut pending: Option<(Rect, Color)> = None;
        for ends in edges.windows(2) {
            let (left, right) = (ends[0], ends[1]);
            if right <= left {
                continue;
            }
            let mut sum = [0.; 4];
            for s in &spans {
                let weight = (right.min(s.right) - left.max(s.left)).max(0.)
                    / (right - left)
                    / SAMPLES as f32;
                for (out, value) in sum.iter_mut().zip(s.color) {
                    *out += value * weight;
                }
            }
            let color = straight(sum);
            let cell = Rect {
                x: left / scale.0,
                y: row / scale.1,
                w: (right - left) / scale.0,
                h: height / scale.1,
            };
            if let Some((p, c)) = pending.as_mut()
                && *c == color
                && (p.x + p.w - cell.x).abs() < 0.0001
            {
                p.w = cell.x + cell.w - p.x;
            } else {
                if let Some((p, c)) = pending.take() {
                    emit(r, p, c, horizontal)?;
                }
                pending = Some((cell, color));
            }
        }
        if let Some((p, c)) = pending {
            emit(r, p, c, horizontal)?;
        }
        row += height;
    }
    Ok(())
}
fn emit<R: Renderer>(
    r: &mut R,
    rect: Rect,
    color: Color,
    transposed: bool,
) -> Result<(), R::Error> {
    if rect.w > 0. && rect.h > 0. {
        r.draw(crate::PaintCommand::FillRect {
            rect: if transposed { transpose(rect) } else { rect },
            color,
        })?;
    }
    Ok(())
}
