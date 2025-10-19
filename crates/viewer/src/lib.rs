pub mod context;
pub mod config;
pub mod types;

pub use context::{Context, ViewBackend, DEFAULT_SCALE};
pub use config::{Config, Icon, view_box};
pub use types::{Emitter, Interactive};

use pathfinder_geometry::vector::Vector2I;

pub fn round_to_16(i: i32) -> i32 {
    (i + 15) & !0xf
}

pub fn round_v_to_16(v: Vector2I) -> Vector2I {
    Vector2I::new(round_to_16(v.x()), round_to_16(v.y()))
}
