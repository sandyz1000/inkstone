pub mod dom;
pub mod draw;
pub mod text;
pub mod web;
pub mod shared;
pub mod types;

// Re-export commonly used items
pub use draw::*;
pub use dom::prelude as dom_prelude;
pub use types::SvgGlyph;

// Create a prelude that combines draw's prelude
pub mod prelude {
    pub use crate::draw::prelude::*;
}
