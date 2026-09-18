//! Backend-independent box decoration. Shapes lower to the existing graphics primitives.
use crate::{Color, PaintCommand, Rect, Renderer};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Stop {
    pub position: f32,
    pub color: Color,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Gradient {
    pub stops: [Stop; 8],
    pub count: usize,
    pub horizontal: bool,
    pub reverse: bool,
}
impl Gradient {
    pub(crate) fn sample(&self, position: f32) -> Color {
        let p = if self.reverse {
            1.0 - position
        } else {
            position
        };
        let stops = &self.stops[..self.count];
        if p < stops[0].position {
            return stops[0].color;
        }
        for pair in stops.windows(2) {
            if p < pair[1].position {
                let t = (p - pair[0].position) / (pair[1].position - pair[0].position);
                let Color(r1, g1, b1, a1) = pair[0].color;
                let Color(r2, g2, b2, a2) = pair[1].color;
                let alpha = a1 as f32 * (1.0 - t) + a2 as f32 * t;
                let mix = |v1: u8, v2: u8| {
                    if alpha == 0.0 {
                        0
                    } else {
                        ((v1 as f32 * a1 as f32 * (1.0 - t) + v2 as f32 * a2 as f32 * t) / alpha)
                            .round() as u8
                    }
                };
                return Color(mix(r1, r2), mix(g1, g2), mix(b1, b2), alpha.round() as u8);
            }
        }
        stops.last().unwrap().color
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct InsetShadow {
    pub x: f32,
    pub y: f32,
    pub spread: f32,
    pub color: Color,
}
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Outline {
    pub width: f32,
    pub offset: f32,
    pub color: Option<Color>,
    pub dotted: bool,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Icon {
    Check,
    ChevronDown,
    ChevronUpDown,
}
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Appearance {
    pub object_fit: crate::image::ObjectFit,
    pub icon: Option<Icon>,
    pub icon_size: f32,
    pub icon_stroke: f32,
    pub content: Option<char>,
    pub background: Option<Color>,
    pub image: Option<Gradient>,
    pub text: Option<Color>,
    pub borders: [Option<Color>; 4],
    pub radius: f32,
    pub shadow: Option<InsetShadow>,
    pub outline: Outline,
}

pub(crate) fn inset(rect: Rect, amount: f32) -> Rect {
    let x = amount.min(rect.w / 2.0);
    let y = amount.min(rect.h / 2.0);
    Rect {
        x: rect.x + x,
        y: rect.y + y,
        w: (rect.w - 2.0 * amount).max(0.0),
        h: (rect.h - 2.0 * amount).max(0.0),
    }
}
fn fill<R: Renderer>(r: &mut R, rect: Rect, color: Color) -> Result<(), R::Error> {
    if rect.w > 0.0 && rect.h > 0.0 {
        r.draw(PaintCommand::FillRect { rect, color })?;
    }
    Ok(())
}
fn ring<R: Renderer>(
    r: &mut R,
    rect: Rect,
    radius: f32,
    width: f32,
    colors: [Color; 4],
) -> Result<(), R::Error> {
    crate::raster::paint_box(
        r,
        rect,
        width,
        Appearance {
            radius,
            borders: colors.map(Some),
            ..Default::default()
        },
        Color(0, 0, 0, 0),
        Color(0, 0, 0, 0),
    )
}
pub(crate) fn paint_box<R: Renderer>(
    r: &mut R,
    rect: Rect,
    border: f32,
    appearance: Appearance,
    background: Color,
    border_color: Color,
) -> Result<(), R::Error> {
    crate::raster::paint_box(r, rect, border, appearance, background, border_color)?;
    let outline = appearance.outline;
    if outline.width > 0.0 {
        let rect = inset(rect, -outline.offset - outline.width);
        let color = outline.color.or(appearance.text).unwrap_or(border_color);
        if outline.dotted {
            let dot = outline.width.min(rect.w / 2.0).min(rect.h / 2.0);
            if dot <= 0.0 {
                return Ok(());
            }
            for horizontal in [true, false] {
                let length = if horizontal { rect.w } else { rect.h };
                let count = (length / (2.0 * dot)).ceil().min(8192.0) as usize;
                for i in 0..count {
                    let offset = i as f32 * 2.0 * dot;
                    let span = dot.min(length - offset);
                    if horizontal {
                        for y in [rect.y, rect.y + rect.h - dot] {
                            fill(
                                r,
                                Rect {
                                    x: rect.x + offset,
                                    y,
                                    w: span,
                                    h: dot,
                                },
                                color,
                            )?;
                        }
                    } else if offset >= dot && offset + span <= rect.h - dot {
                        for x in [rect.x, rect.x + rect.w - dot] {
                            fill(
                                r,
                                Rect {
                                    x,
                                    y: rect.y + offset,
                                    w: dot,
                                    h: span,
                                },
                                color,
                            )?;
                        }
                    }
                }
            }
        } else {
            ring(
                r,
                rect,
                (appearance.radius + outline.offset + outline.width).max(0.0),
                outline.width,
                [color; 4],
            )?;
        }
    }
    Ok(())
}

/// Font-independent, round-ended strokes, shared by every theme. Coverage is
/// sampled once per pixel across all segments, so joins do not darken from overlap.
pub(crate) fn paint_icon<R: Renderer>(
    renderer: &mut R,
    area: Rect,
    icon: Icon,
    size: f32,
    stroke: f32,
    color: Color,
) -> Result<(), R::Error> {
    let aspect = match icon {
        Icon::Check => 0.8,
        Icon::ChevronDown => 0.55,
        Icon::ChevronUpDown => 1.25,
    };
    let width = size.min(area.w).min(area.h / aspect);
    if width <= 0.0 || color.3 == 0 {
        return Ok(());
    }
    let height = width * aspect;
    let stroke = stroke.min(width).min(height);
    let x = area.x + (area.w - width) / 2.0 + stroke / 2.0;
    let y = area.y + (area.h - height) / 2.0 + stroke / 2.0;
    let w = width - stroke;
    let h = height - stroke;
    let segments = match icon {
        Icon::Check => [
            (0.0, 0.45, 0.35, 0.9),
            (0.35, 0.9, 1.0, 0.05),
            (0.0, 0.45, 0.35, 0.9),
            (0.35, 0.9, 1.0, 0.05),
        ],
        Icon::ChevronDown => [
            (0.0, 0.0, 0.5, 1.0),
            (0.5, 1.0, 1.0, 0.0),
            (0.0, 0.0, 0.5, 1.0),
            (0.5, 1.0, 1.0, 0.0),
        ],
        Icon::ChevronUpDown => [
            (0.0, 0.3, 0.5, 0.0),
            (0.5, 0.0, 1.0, 0.3),
            (0.0, 0.7, 0.5, 1.0),
            (0.5, 1.0, 1.0, 0.7),
        ],
    }
    .map(|(ax, ay, bx, by)| (x + ax * w, y + ay * h, x + bx * w, y + by * h));
    let (sx, sy) = crate::raster::scale(renderer);
    let left = ((x - stroke / 2.0) * sx).floor() / sx;
    let top = ((y - stroke / 2.0) * sy).floor() / sy;
    let right = ((x + w + stroke / 2.0) * sx).ceil() / sx;
    let bottom = ((y + h + stroke / 2.0) * sy).ceil() / sy;
    for row in 0..((bottom - top) * sy).round() as usize {
        for col in 0..((right - left) * sx).round() as usize {
            let cell = Rect {
                x: left + col as f32 / sx,
                y: top + row as f32 / sy,
                w: 1.0 / sx,
                h: 1.0 / sy,
            };
            let mut covered = 0;
            for sy in 0..4 {
                for sx in 0..4 {
                    let px = cell.x + (sx as f32 + 0.5) * cell.w / 4.0;
                    let py = cell.y + (sy as f32 + 0.5) * cell.h / 4.0;
                    if area.contains(px, py)
                        && segments.iter().any(|&(ax, ay, bx, by)| {
                            let dx = bx - ax;
                            let dy = by - ay;
                            let length_squared = dx * dx + dy * dy;
                            let t = if length_squared > 0.0 {
                                (((px - ax) * dx + (py - ay) * dy) / length_squared).clamp(0.0, 1.0)
                            } else {
                                0.0
                            };
                            (px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2)
                                <= (stroke / 2.0).powi(2)
                        })
                    {
                        covered += 1;
                    }
                }
            }
            if covered > 0 {
                fill(
                    renderer,
                    cell,
                    Color(
                        color.0,
                        color.1,
                        color.2,
                        ((color.3 as u32 * covered + 8) / 16) as u8,
                    ),
                )?;
            }
        }
    }
    Ok(())
}
