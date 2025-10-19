use pathfinder_geometry::vector::Vector2F;
use pathfinder_renderer::scene::Scene;

use crate::context::{Context, ViewBackend};

pub struct Emitter<E> {
    pub inner: E,
}

impl<E: Clone> Clone for Emitter<E> {
    fn clone(&self) -> Self {
        Emitter {
            inner: self.inner.clone(),
        }
    }
}

/// Core trait for interactive PDF viewers
/// Implementations must handle scene rendering and user interactions
pub trait Interactive: 'static {
    type Event: std::fmt::Debug + Send + 'static;
    type Backend: ViewBackend;

    /// Generate the scene to render
    fn scene(&mut self, ctx: &mut Context<Self::Backend>) -> Scene;

    /// Handle single character input
    fn char_input(&mut self, _ctx: &mut Context<Self::Backend>, _input: char) {}

    /// Handle text input (default: process char by char)
    fn text_input(&mut self, ctx: &mut Context<Self::Backend>, input: String) {
        for c in input.chars() {
            self.char_input(ctx, c);
        }
    }

    /// Handle cursor movement
    fn cursor_moved(&mut self, _ctx: &mut Context<Self::Backend>, _pos: Vector2F) {}

    /// Handle exit/close request
    fn exit(&mut self, _ctx: &mut Context<Self::Backend>) {}

    /// Get window title
    fn title(&self) -> String {
        "PDF Viewer".into()
    }

    /// Handle custom events
    fn event(&mut self, _ctx: &mut Context<Self::Backend>, _event: Self::Event) {}

    /// Initialize the viewer
    fn init(&mut self, ctx: &mut Context<Self::Backend>, sender: Emitter<Self::Event>);

    /// Handle idle state
    fn idle(&mut self, _ctx: &mut Context<Self::Backend>) {}

    /// Suggest initial window size
    fn window_size_hint(&self) -> Option<Vector2F> {
        None
    }
}
