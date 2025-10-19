use gpui::Window;
use pathfinder_geometry::vector::Vector2F;
use viewer::{ ViewBackend, Icon };

/// GPUI backend implementation for the viewer crate
/// Bridges GPUI window management with viewer abstractions
pub struct GpuiBackend {
    pixel_scroll_factor: Vector2F,
    line_scroll_factor: Vector2F,
    icon: Option<Icon>,
}

impl GpuiBackend {
    pub fn new() -> Self {
        Self {
            pixel_scroll_factor: Vector2F::splat(1.0),
            line_scroll_factor: Vector2F::splat(10.0),
            icon: None,
        }
    }
}

impl ViewBackend for GpuiBackend {
    fn resize(&mut self, _size: Vector2F) {
        // Size changes are handled by GPUI's layout system
        // We don't need to do anything here as GPUI manages window resizing
    }

    fn get_scroll_factors(&self) -> (Vector2F, Vector2F) {
        (self.pixel_scroll_factor, self.line_scroll_factor)
    }

    fn set_icon(&mut self, icon: Icon) {
        self.icon = Some(icon);
        // Note: GPUI 0.2 window icon setting might need window handle
        // For now, we just store it
    }
}

impl Default for GpuiBackend {
    fn default() -> Self {
        Self::new()
    }
}
