//! Image assets belong to the host. The UI only retains source keys and geometry.
use crate::{ControlKind, Error, Rect, Ui};

/// A registered image draw, in document order. `source` uses image pixels;
/// `destination` and `clip` use logical UI coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageDraw<'a> {
    pub src: &'a str,
    pub source: Rect,
    pub destination: Rect,
    pub clip: Rect,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum ObjectFit {
    #[default]
    Fill,
    Contain,
    Cover,
}
impl ObjectFit {
    pub fn place(self, src: &str, size: (u32, u32), content: Rect) -> ImageDraw<'_> {
        let mut source = Rect {
            x: 0.0,
            y: 0.0,
            w: size.0 as f32,
            h: size.1 as f32,
        };
        let mut destination = content;
        match self {
            Self::Fill => {}
            Self::Contain => {
                let scale =
                    (content.w as f64 / source.w as f64).min(content.h as f64 / source.h as f64);
                destination.w = ((source.w as f64 * scale) as f32).min(content.w);
                destination.h = ((source.h as f64 * scale) as f32).min(content.h);
                destination.x += (content.w - destination.w) / 2.0;
                destination.y += (content.h - destination.h) / 2.0;
            }
            Self::Cover => {
                let scale =
                    (content.w as f64 / source.w as f64).max(content.h as f64 / source.h as f64);
                let w = ((content.w as f64 / scale) as f32).min(source.w);
                let h = ((content.h as f64 / scale) as f32).min(source.h);
                source.x = (source.w - w) / 2.0;
                source.y = (source.h - h) / 2.0;
                source.w = w;
                source.h = h;
            }
        }
        ImageDraw {
            src,
            source,
            destination,
            clip: content,
        }
    }
}
impl Ui {
    /// Source keys in document order, including duplicates. No assets are loaded.
    pub fn image_sources(&self) -> impl Iterator<Item = &str> {
        self.controls.iter().filter_map(|c| match &c.kind {
            ControlKind::Image { src, .. } => Some(src.as_str()),
            _ => None,
        })
    }
    pub fn image_source(&self, id: &str) -> Option<&str> {
        match &self.controls[*self.ids.get(id)?].kind {
            ControlKind::Image { src, .. } => Some(src),
            _ => None,
        }
    }
    /// Change a source key without loading an asset or changing layout.
    pub fn set_image_source(&mut self, id: &str, src: &str) -> Result<(), Error> {
        let Some(&index) = self.ids.get(id) else {
            return Err(Error::new(format!("unknown image: {id}")));
        };
        let ControlKind::Image { src: current, .. } = &mut self.controls[index].kind else {
            return Err(Error::new(format!("not an image: {id}")));
        };
        if current != src {
            src.clone_into(current);
            self.invalidate_paint();
        }
        Ok(())
    }
}
