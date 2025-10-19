mod context;
pub mod config;
pub mod native;
pub mod wasm;

use pathfinder_geometry::vector::Vector2I;
use pathfinder_geometry::vector::Vector2F;
use pathfinder_renderer::scene::Scene;
use winit::{
    event::{ElementState, KeyEvent, RawKeyEvent},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
};
use crate::context::{ViewBackend, Context, DEFAULT_SCALE};


pub struct Emitter<E> {
    pub inner: E
}

impl<E: Clone> Clone for Emitter<E> {
    fn clone(&self) -> Self {
        Emitter {
            inner: self.inner.clone()
        }
    }
}

pub trait Interactive: 'static {
    type Event: std::fmt::Debug + Send + 'static;
    type Backend: ViewBackend;

    fn scene(&mut self, ctx: &mut Context<Self::Backend>) -> Scene;

    fn char_input(&mut self, ctx: &mut Context<Self::Backend>, input: char) {

    }
    
    fn text_input(&mut self, ctx: &mut Context<Self::Backend>, input: String) {
        for c in input.chars() {
            self.char_input(ctx, c);
        }
    }

    fn keyboard_input(&mut self, ctx: &mut Context<Self::Backend>, modifiers: ModifiersState, event: RawKeyEvent) {
        match (event.state, modifiers.control_key(), event.physical_key) {
            (ElementState::Pressed, false, PhysicalKey::Code(KeyCode::PageDown)) => ctx.next_page(),
            (ElementState::Pressed, false, PhysicalKey::Code(KeyCode::PageUp)) => ctx.prev_page(),
            (ElementState::Pressed, true, PhysicalKey::Code(KeyCode::Digit1)) => ctx.zoom_by(0.2),
            (ElementState::Pressed, true, PhysicalKey::Code(KeyCode::Digit2)) => ctx.zoom_by(-0.2),
            (ElementState::Pressed, true, PhysicalKey::Code(KeyCode::Digit0)) => {
                ctx.set_zoom(DEFAULT_SCALE)
            }
            _ => return,
        }
    }

    fn mouse_input(&mut self, ctx: &mut Context<Self::Backend>, page: usize, pos: Vector2F, state: ElementState) {}
    
    fn cursor_moved(&mut self, ctx: &mut Context<Self::Backend>, pos: Vector2F) {}
    
    fn exit(&mut self, ctx: &mut Context<Self::Backend>) {}
    
    fn title(&self) -> String {
        "A fantastic window!".into()
    }
    
    fn event(&mut self, _ctx: &mut Context<Self::Backend>, event: Self::Event) {}
    
    fn init(&mut self, ctx: &mut Context<Self::Backend>, sender: Emitter<Self::Event>);
    
    fn idle(&mut self, _ctx: &mut Context<Self::Backend>) {}
    
    fn window_size_hint(&self) -> Option<Vector2F> {
        None
    }
}


fn round_to_16(i: i32) -> i32 {
    (i + 15) & !0xf
}

pub fn round_v_to_16(v: Vector2I) -> Vector2I {
    Vector2I::new(round_to_16(v.x()), round_to_16(v.y()))
}
