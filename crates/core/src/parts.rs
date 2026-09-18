use crate::{
    Error,
    css::{self, CompiledPart, Part},
    html::Node,
};
#[derive(Debug, Default)]
pub(crate) struct Parts {
    pub range_track: Option<Box<CompiledPart>>,
    pub range_fill: Option<Box<CompiledPart>>,
    pub range_thumb: Option<Box<CompiledPart>>,
    pub scrollbar: Option<Box<CompiledPart>>,
    pub scroll_track: Option<Box<CompiledPart>>,
    pub scroll_thumb: Option<Box<CompiledPart>>,
    pub scroll_corner: Option<Box<CompiledPart>>,
}
impl Parts {
    pub fn compile(node: &Node, rules: &[css::Rule], flow: bool) -> Result<Self, Error> {
        let part = |p| css::compile_part(node, rules, p, flow);
        Ok(Self {
            range_track: part(Part::SliderTrack)?,
            range_fill: part(Part::SliderFill)?,
            range_thumb: part(Part::SliderThumb)?,
            scrollbar: part(Part::Scrollbar)?,
            scroll_track: part(Part::ScrollTrack)?,
            scroll_thumb: part(Part::ScrollThumb)?,
            scroll_corner: part(Part::ScrollCorner)?,
        })
    }
}
pub(crate) fn metric(part: &Option<Box<CompiledPart>>, axis: usize, default: f32) -> f32 {
    part.as_ref().map_or(default, |p| {
        match if axis == 0 {
            p.base.layout.width
        } else {
            p.base.layout.height
        } {
            crate::layout::Dimension::Px(v) => v,
            _ => default,
        }
    })
}
