//! Small retained layout tree; no DOM or stylesheet is needed during resize.
use crate::{Control, ControlKind, Error, Rect, Ui};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum Dimension {
    #[default]
    Auto,
    Px(f32),
    Percent(f32),
}
impl Dimension {
    fn resolve(self, available: f32) -> Option<f32> {
        match self {
            Self::Auto => None,
            Self::Px(v) => Some(v),
            Self::Percent(v) => Some(available * v),
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Position {
    #[default]
    Static,
    Relative,
    Absolute,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Display {
    #[default]
    Block,
    Flex,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Direction {
    #[default]
    Row,
    Column,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Align {
    #[default]
    Stretch,
    Start,
    Center,
    End,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Style {
    pub position: Position,
    pub width: Dimension,
    pub height: Dimension,
    pub margin: [f32; 4],
    pub display: Display,
    pub direction: Direction,
    pub grow: f32,
    pub gap: f32,
    pub align: Align,
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub content_box: bool,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            position: Position::Static,
            width: Dimension::Auto,
            height: Dimension::Auto,
            margin: [0.0; 4],
            display: Display::Block,
            direction: Direction::Row,
            grow: 0.0,
            gap: 0.0,
            align: Align::Stretch,
            min: [0.0; 2],
            max: [f32::INFINITY; 2],
            content_box: false,
        }
    }
}
#[derive(Debug)]
pub(crate) struct Node {
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}
#[derive(Debug, Default)]
pub(crate) struct Tree {
    pub nodes: Vec<Node>,
    pub viewport: Option<(f32, f32)>,
    pub paint_order: Vec<(usize, bool)>,
}
impl Tree {
    pub fn rebuild_order(&mut self) {
        // Physical indices stay stable as controls are appended. Paint and hit-test
        // order follows the tree, including children inserted into earlier parents.
        self.paint_order.clear();
        let mut stack: Vec<_> = self
            .nodes
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, n)| n.parent.is_none())
            .map(|(i, _)| (i, false))
            .collect();
        while let Some((i, closing)) = stack.pop() {
            self.paint_order.push((i, closing));
            if !closing {
                stack.push((i, true));
                stack.extend(self.nodes[i].children.iter().rev().map(|&c| (c, false)));
            }
        }
    }

    pub fn push(&mut self, parent: Option<usize>) {
        let index = self.nodes.len();
        self.nodes.push(Node {
            parent,
            children: Vec::new(),
        });
        if let Some(parent) = parent {
            self.nodes[parent].children.push(index);
        }
    }
}
impl Ui {
    /// Reflow only when the logical viewport size changes. Values and keyboard focus
    /// survive resizing; popup/captures and old text hit positions are cancelled.
    pub fn set_viewport(&mut self, width: f32, height: f32) -> Result<(), Error> {
        Rect {
            x: 0.0,
            y: 0.0,
            w: width,
            h: height,
        }
        .validate()?;
        let Some(tree) = &self.layout else {
            return Err(Error::new(
                "viewport layout requires from_html_css_with_viewport",
            ));
        };
        if tree.viewport == Some((width, height)) {
            return Ok(());
        }
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            w: width,
            h: height,
        };
        let mut gutters = vec![[0.0; 2]; self.controls.len()];
        let mut solved = None;
        for _ in 0..(self.controls.len() * 2 + 4) {
            let mut solver = Solver {
                controls: &self.controls,
                tree,
                rects: vec![Rect::default(); self.controls.len()],
                gutters: &gutters,
            };
            for (index, node) in tree.nodes.iter().enumerate() {
                if node.parent.is_none() {
                    solver.place_root(index, viewport)?;
                }
            }
            let states = crate::scrolling::metrics(&self.controls, tree, &solver.rects, &gutters);
            let next: Vec<_> = states.iter().map(|s| s.gutter).collect();
            if next == gutters {
                solved = Some((solver.rects, states));
                break;
            }
            gutters = next;
        }
        let (rects, states) =
            solved.ok_or_else(|| Error::new("scrollbar layout did not converge"))?;
        if states.iter().any(|s| s.max.iter().any(|v| !v.is_finite())) {
            return Err(Error::new("scroll extent must be finite"));
        }
        // Commit only once both layout and scrollbar gutters are valid.
        for ((control, rect), scroll) in self.controls.iter_mut().zip(rects).zip(states) {
            control.layout_rect = rect;
            control.rect = rect;
            control.scroll = scroll;
        }
        self.layout.as_mut().unwrap().rebuild_order();
        self.refresh_scroll_positions();
        self.invalidate_paint();
        self.cancel_layout_captures();
        self.layout.as_mut().unwrap().viewport = Some((width, height));
        Ok(())
    }
}
struct Solver<'a> {
    controls: &'a [Control],
    tree: &'a Tree,
    rects: Vec<Rect>,
    gutters: &'a [[f32; 2]],
}
impl Solver<'_> {
    fn edges(&self, i: usize) -> [f32; 2] {
        let s = &self.controls[i].style;
        [
            s.padding[1] + s.padding[3] + 2.0 * s.border_width + self.gutters[i][0],
            s.padding[0] + s.padding[2] + 2.0 * s.border_width + self.gutters[i][1],
        ]
    }
    fn constrain(&self, i: usize, axis: usize, value: f32) -> f32 {
        let s = self.controls[i].style.layout;
        let edges = self.edges(i)[axis];
        let add = if s.content_box { edges } else { 0.0 };
        value
            .max(s.min[axis] + add)
            .min(s.max[axis] + add)
            .max(edges)
    }
    fn explicit(&self, i: usize, axis: usize, available: f32) -> Option<f32> {
        let s = self.controls[i].style.layout;
        let value = if axis == 0 { s.width } else { s.height };
        value.resolve(available).map(|v| {
            self.constrain(
                i,
                axis,
                v + if s.content_box {
                    self.edges(i)[axis]
                } else {
                    0.0
                },
            )
        })
    }
    fn measure(
        &self,
        i: usize,
        available: [f32; 2],
        fill_width: bool,
        depth: usize,
    ) -> Result<[f32; 2], Error> {
        if depth > 128 {
            return Err(Error::new("layout nesting exceeds 128 containers"));
        }
        let c = &self.controls[i];
        let s = c.style.layout;
        let edges = self.edges(i);
        let width = self.explicit(i, 0, available[0]).unwrap_or_else(|| {
            self.constrain(
                i,
                0,
                if fill_width {
                    available[0] - s.margin[1] - s.margin[3]
                } else {
                    edges[0]
                },
            )
        });
        let mut height = self.explicit(i, 1, available[1]);
        if height.is_none() {
            let content_w = (width - edges[0]).max(0.0);
            let mut used: f32 = 0.0;
            let mut count = 0;
            for &child in &self.tree.nodes[i].children {
                let cs = self.controls[child].style.layout;
                if cs.position == Position::Absolute {
                    continue;
                }
                // Percent heights inside an auto-height container use a zero base;
                // there is no circular parent/child percentage resolution.
                let size = self.measure(child, [content_w, 0.0], true, depth + 1)?;
                let outer = size[1] + cs.margin[0] + cs.margin[2];
                if s.display == Display::Flex && s.direction == Direction::Row {
                    used = used.max(outer);
                } else {
                    used += outer;
                }
                count += 1;
            }
            if !(s.display == Display::Flex && s.direction == Direction::Row) {
                used += s.gap * (count as f32 - 1.0).max(0.0);
            }
            let leaf = if matches!(
                c.kind,
                ControlKind::Container | ControlKind::Canvas(_) | ControlKind::Image { .. }
            ) {
                0.0
            } else {
                c.style.font_size.unwrap_or(18.0)
            };
            height = Some(self.constrain(i, 1, edges[1] + if count > 0 { used } else { leaf }));
        }
        Ok([width, height.unwrap()])
    }
    fn place_root(&mut self, i: usize, area: Rect) -> Result<(), Error> {
        let s = self.controls[i].style.layout;
        let m = s.margin;
        let measured = self.measure(i, [area.w, area.h], true, 0)?;
        let size = if matches!(self.controls[i].kind, ControlKind::Container) {
            [
                self.explicit(i, 0, area.w)
                    .unwrap_or_else(|| self.constrain(i, 0, area.w - m[1] - m[3])),
                self.explicit(i, 1, area.h)
                    .unwrap_or_else(|| self.constrain(i, 1, area.h - m[0] - m[2])),
            ]
        } else {
            measured
        };
        self.place(i, area.x + m[3], area.y + m[0], size, 0)
    }
    fn place(
        &mut self,
        i: usize,
        x: f32,
        y: f32,
        size: [f32; 2],
        depth: usize,
    ) -> Result<(), Error> {
        if depth > 128 {
            return Err(Error::new("layout nesting exceeds 128 containers"));
        }
        let c = &self.controls[i];
        let s = c.style.layout;
        let (dx, dy) = if s.position != Position::Static {
            (c.style.rect.x, c.style.rect.y)
        } else {
            (0.0, 0.0)
        };
        let rect = Rect {
            x: x + dx,
            y: y + dy,
            w: size[0],
            h: size[1],
        };
        rect.validate()?;
        if matches!(c.kind, ControlKind::Select { .. }) {
            Rect {
                y: rect.y + rect.h,
                h: rect.h * crate::controls::POPUP_ROWS as f32,
                ..rect
            }
            .validate()?;
        }
        self.rects[i] = rect;
        let p = c.style.padding;
        let b = c.style.border_width;
        let area = Rect {
            x: rect.x + b + p[3],
            y: rect.y + b + p[0],
            w: (rect.w - 2.0 * b - p[1] - p[3] - self.gutters[i][0]).max(0.0),
            h: (rect.h - 2.0 * b - p[0] - p[2] - self.gutters[i][1]).max(0.0),
        };
        let row = s.display == Display::Flex && s.direction == Direction::Row;
        let axis = usize::from(!row);
        let cross = 1 - axis;
        let available = [area.w, area.h];
        let mut sizes = Vec::with_capacity(self.tree.nodes[i].children.len());
        let mut occupied = 0.0;
        let mut grow = 0.0;
        let mut count = 0;
        for &child in &self.tree.nodes[i].children {
            let cs = self.controls[child].style.layout;
            let child_size = self.measure(child, available, !row, depth + 1)?;
            sizes.push((child, child_size));
            if cs.position == Position::Absolute {
                continue;
            }
            occupied += child_size[axis]
                + if row {
                    cs.margin[1] + cs.margin[3]
                } else {
                    cs.margin[0] + cs.margin[2]
                };
            grow += cs.grow;
            count += 1;
        }
        occupied += s.gap * (count as f32 - 1.0).max(0.0);
        // Positive free space is shared by grow weights. Min/max constrained items
        // are frozen and the remaining space is redistributed among the others.
        if s.display == Display::Flex && grow > 0.0 {
            let mut remaining = (available[axis] - occupied).max(0.0);
            for _ in 0..=sizes.len() {
                let weight: f32 = sizes
                    .iter()
                    .filter(|(j, z)| {
                        self.controls[*j].style.layout.position != Position::Absolute
                            && self.constrain(*j, axis, z[axis] + remaining) > z[axis]
                    })
                    .map(|(j, _)| self.controls[*j].style.layout.grow)
                    .sum();
                if remaining <= 0.001 || weight <= 0.0 {
                    break;
                }
                let mut used = 0.0;
                for (j, z) in &mut sizes {
                    let cs = self.controls[*j].style.layout;
                    if cs.position == Position::Absolute || cs.grow == 0.0 {
                        continue;
                    }
                    let next = self.constrain(*j, axis, z[axis] + remaining * cs.grow / weight);
                    used += next - z[axis];
                    z[axis] = next;
                }
                if used <= 0.001 {
                    break;
                }
                remaining = (remaining - used).max(0.0);
            }
        }
        let mut cursor = 0.0;
        for (child, mut child_size) in sizes {
            let cs = self.controls[child].style.layout;
            let m = cs.margin;
            if cs.position == Position::Absolute {
                self.place(child, area.x + m[3], area.y + m[0], child_size, depth + 1)?;
                continue;
            }
            let before = if row { m[3] } else { m[0] };
            let after = if row { m[1] } else { m[2] };
            let cross_before = if row { m[0] } else { m[3] };
            let cross_after = if row { m[2] } else { m[1] };
            let mut cross_offset = cross_before;
            if s.display == Display::Flex {
                let auto = matches!(
                    if cross == 0 { cs.width } else { cs.height },
                    Dimension::Auto
                );
                if s.align == Align::Stretch && auto {
                    child_size[cross] =
                        self.constrain(child, cross, available[cross] - cross_before - cross_after);
                }
                let extra =
                    (available[cross] - child_size[cross] - cross_before - cross_after).max(0.0);
                cross_offset += match s.align {
                    Align::Center => extra / 2.0,
                    Align::End => extra,
                    _ => 0.0,
                };
            }
            let (x, y) = if row {
                (area.x + cursor + before, area.y + cross_offset)
            } else {
                (area.x + cross_offset, area.y + cursor + before)
            };
            self.place(child, x, y, child_size, depth + 1)?;
            cursor += before + child_size[axis] + after + s.gap;
        }
        Ok(())
    }
}
