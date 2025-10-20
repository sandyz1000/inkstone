use viewer::{ ViewBackend, Icon };
use pathfinder_geometry::vector::Vector2F;

/// WebGL backend for Dioxus-based PDF viewer
pub struct DioxusBackend {
    window_size: Vector2F,
}

impl DioxusBackend {
    pub fn new() -> Self {
        Self {
            window_size: Vector2F::new(800.0, 600.0),
        }
    }
}

impl ViewBackend for DioxusBackend {
    fn resize(&mut self, size: Vector2F) {
        self.window_size = size;
    }

    fn get_scroll_factors(&self) -> (Vector2F, Vector2F) {
        // For web:
        // - pixel scroll factor: direct pixel scrolling (1:1)
        // - line scroll factor: typical browser line height (~20 pixels)
        (
            Vector2F::new(1.0, 1.0), // pixel scroll
            Vector2F::new(20.0, 20.0), // line scroll
        )
    }

    fn set_icon(&mut self, _icon: Icon) {
        // In web context, this could set the favicon
        // For now, we'll leave it as a no-op
        // Future: Could use web-sys to update the favicon
    }
}
