use std::sync::Arc;
use std::fmt;
use crate::dom::{ Svg, Item };

/// SVG Glyph representation
#[derive(Clone)]
pub struct SvgGlyph {
    pub svg: Arc<Svg>,
    pub item: Arc<Item>,
}

impl fmt::Debug for SvgGlyph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SVG Glyph")
    }
}
