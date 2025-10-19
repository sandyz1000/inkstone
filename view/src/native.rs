// Refer this: https://github.com/rust-windowing/glutin/blob/master/glutin_examples/src/lib.rs
// https://github.com/rust-windowing/winit/blob/master/examples/window.rs#L127
// On resumed create the glwindow
use crate::config::{ Config, Icon };
use crate::context::{ Context, ViewBackend };

use crate::round_v_to_16;
use crate::{ Emitter, Interactive };

use pathfinder_geometry::vector::vec2f;
use pathfinder_renderer::options::{ BuildOptions, RenderTransform };
use pathfinder_renderer::scene::Scene;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::dpi::{ PhysicalPosition, PhysicalSize };
use winit::event::{
    ElementState as WinitElementState,
    MouseButton,
    MouseScrollDelta,
    RawKeyEvent,
    WindowEvent,
};
use winit::event_loop::{ ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy };
use winit::keyboard::ModifiersState;
use winit::window::Window;

use std::{ ffi::CStr, num::NonZeroU32 };

use pathfinder_geometry::{ rect::RectF, vector::{ Vector2F, Vector2I } };
use pathfinder_gl::{ GLDevice, GLVersion };
use pathfinder_renderer::{
    concurrent::{ executor::SequentialExecutor, rayon::RayonExecutor, scene_proxy::SceneProxy },
    gpu::{
        options::{ DestFramebuffer, RendererLevel, RendererMode, RendererOptions },
        renderer::Renderer,
    },
};
use winit::raw_window_handle::HasWindowHandle;

use gl;

use glutin::{
    config::{ Api, ConfigTemplateBuilder, GlConfig },
    context::NotCurrentGlContext,
    context::{ PossiblyCurrentContext, Version },
    display::{ GetGlDisplay, GlDisplay },
    surface::{ GlSurface, Surface, WindowSurface },
};
use glutin_winit::{ DisplayBuilder, GlWindow as GlutinGlWindow };
// use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::Window};

struct AppState {
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    framebuffer_size: Vector2I,
    window_size: Vector2F,
    // NOTE: Window should be dropped after all resources created using its
    // raw-window-handle.
    window: Rc<Window>,
    ctx: Context<NativeBackend>,
    modifiers: ModifiersState,
    cursor_pos: Vector2F,
    dragging: bool,
    proxy: SceneProxy,
    renderer: Renderer<GLDevice>,
}

struct App<I> {
    template: ConfigTemplateBuilder,
    display_builder: DisplayBuilder,
    gl_version: GLVersion,
    config: Rc<Config>,
    window_size: Vector2F,
    // NOTE: `AppState` carries the `Window`, thus it should be dropped after everything else.
    item: I,
    state: Option<AppState>,
}

impl AppState {
    pub fn render(&mut self, mut scene: Scene, options: BuildOptions) {
        scene.set_view_box(RectF::new(Vector2F::default(), self.framebuffer_size.to_f32()));
        self.proxy.replace_scene(scene);

        self.proxy.build_and_render(&mut self.renderer, options);
        self.gl_surface.swap_buffers(&self.gl_context).unwrap();
    }

    // size changed, update GL context
    pub fn resized(&mut self, size: Vector2F) {
        // pathfinder does not like scene sizes that are now a multiple of the tile size (16).
        let new_framebuffer_size = round_v_to_16(size.to_i32());
        if new_framebuffer_size != self.framebuffer_size {
            self.framebuffer_size = new_framebuffer_size;
            self.gl_surface.resize(
                &self.gl_context,
                NonZeroU32::new(self.framebuffer_size.x() as u32).unwrap(),
                NonZeroU32::new(self.framebuffer_size.y() as u32).unwrap()
            );
            self.renderer.options_mut().dest = DestFramebuffer::full_window(new_framebuffer_size);
        }
    }

    pub fn scale_factor(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn framebuffer_size(&self) -> Vector2I {
        self.framebuffer_size
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}

impl<'a, I> ApplicationHandler for App<I> where I: Interactive<Backend = NativeBackend, Event = ()> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent
    ) {
        let app_state = self.state.as_mut().unwrap();

        match event {
            WindowEvent::RedrawRequested => {
                let options = BuildOptions {
                    transform: RenderTransform::default(),
                    dilation: Vector2F::default(),
                    subpixel_aa_enabled: false,
                };

                app_state.resized(app_state.ctx.window_size);
                let scene = self.item.scene(&mut app_state.ctx);
                app_state.render(scene, options);
                app_state.ctx.redraw_requested = false;
            }

            WindowEvent::ScaleFactorChanged { scale_factor, mut inner_size_writer } => {
                app_state.ctx.set_scale_factor(scale_factor as f32);
                let width = app_state.ctx.window_size.x().ceil() as u32;
                let height = app_state.ctx.window_size.y().ceil() as u32;
                app_state.ctx.set_window_size(Vector2F::new(width as f32, height as f32));
                let new_inner_size = PhysicalSize::new(width, height);
                inner_size_writer
                    .request_inner_size(new_inner_size)
                    .map_err(|e| log::error!("{:?}", e))
                    .unwrap();
                app_state.ctx.request_redraw();
            }

            WindowEvent::Focused { .. } => app_state.ctx.request_redraw(),

            WindowEvent::Resized(PhysicalSize { width, height }) => {
                let physical_size = Vector2F::new(width as f32, height as f32);
                app_state.ctx.window_size = physical_size;
                app_state.ctx.check_bounds();
                app_state.ctx.request_redraw();
            }

            WindowEvent::ModifiersChanged(new_modifiers) => {
                app_state.modifiers = new_modifiers.state();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                let raw_kevt = RawKeyEvent {
                    state: event.state,
                    physical_key: event.physical_key,
                };
                self.item.keyboard_input(&mut app_state.ctx, app_state.modifiers, raw_kevt);
            }

            WindowEvent::CursorMoved { position: PhysicalPosition { x, y }, .. } => {
                let new_pos = Vector2F::new(x as f32, y as f32);
                let cursor_delta = new_pos - app_state.cursor_pos;
                app_state.cursor_pos = new_pos;

                if app_state.dragging {
                    app_state.ctx.move_by(cursor_delta * (-1.0 / app_state.ctx.scale));
                } else {
                    self.item.cursor_moved(&mut app_state.ctx, new_pos);
                }
            }

            WindowEvent::MouseInput { button: MouseButton::Left, state, .. } =>
                match (state, app_state.modifiers.shift_key()) {
                    (WinitElementState::Pressed, true) if app_state.ctx.config.pan => {
                        app_state.dragging = true;
                    }
                    (WinitElementState::Released, _) if app_state.dragging => {
                        app_state.dragging = false;
                    }
                    _ => {
                        let page_nr = app_state.ctx.page_nr;
                        self.item.mouse_input(
                            &mut app_state.ctx,
                            page_nr,
                            app_state.cursor_pos,
                            state
                        );
                    }
                }

            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    MouseScrollDelta::PixelDelta(PhysicalPosition { x: dx, y: dy }) => {
                        Vector2F::new(dx as f32, dy as f32) * app_state.ctx.pixel_scroll_factor
                    }
                    MouseScrollDelta::LineDelta(dx, dy) => {
                        Vector2F::new(dx as f32, dy as f32) * app_state.ctx.line_scroll_factor
                    }
                };
                if app_state.ctx.config.zoom && app_state.modifiers.control_key() {
                    app_state.ctx.zoom_by(-0.02 * delta.y());
                } else if app_state.ctx.config.pan {
                    app_state.ctx.move_by(delta * (-1.0 / app_state.ctx.scale));
                }
            }

            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                app_state.ctx.close();
            }

            _ => (),
        }
    }
}

impl<'a, I> App<I> where I: Interactive<Backend = NativeBackend, Event = ()> {
    pub fn new(title: String, window_size: Vector2F, config: Config, item: I) -> Self {
        let config = Rc::new(config);
        let window_builder = Window::default_attributes()
            .with_title(title)
            .with_decorations(config.borders)
            .with_inner_size(PhysicalSize::new(window_size.x() as f64, window_size.y() as f64))
            .with_transparent(config.transparent);
        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_builder));

        // Use GL 3.0 as default to match RendererLevel::D3D9
        let (_glutin_gl_version, renderer_gl_version, api) = match config.render_level {
            RendererLevel::D3D9 => (Version::new(3, 0), GLVersion::GL3, Api::OPENGL),
            RendererLevel::D3D11 => (Version::new(4, 3), GLVersion::GL4, Api::OPENGL),
        };

        let template_builder = ConfigTemplateBuilder::new().with_alpha_size(8).with_api(api);

        App {
            display_builder,
            config,
            window_size,
            template: template_builder,
            gl_version: renderer_gl_version,
            state: None,
            item,
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Option<Rc<Window>> {
        let (window, gl_config) = self.display_builder
            .clone()
            .build(event_loop, self.template.clone(), |configs| {
                configs
                    .reduce(|accum, config| {
                        let transparency_check =
                            config.supports_transparency().unwrap_or(false) &
                            !accum.supports_transparency().unwrap_or(false);

                        if transparency_check || config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .unwrap();
        let window = Rc::new(window.unwrap());

        let raw_window_handle = window.window_handle().unwrap().as_raw();

        // XXX The display could be obtained from any object created by it, so we can
        // query it from the config.
        let gl_display = gl_config.display();

        // The context creation part.
        let context_attributes = glutin::context::ContextAttributesBuilder
            ::new()
            .build(Some(raw_window_handle));

        let attrs = window.build_surface_attributes(<_>::default()).unwrap();
        let gl_surface = unsafe {
            gl_config.display().create_window_surface(&gl_config, &attrs).unwrap()
        };

        let windowed_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .expect("failed to create context")
        };
        let gl_context = windowed_context.make_current(&gl_surface).unwrap();

        gl::load_with(|ptr: &str| {
            gl_display.get_proc_address(unsafe { CStr::from_ptr(ptr.as_ptr().cast()) })
        });

        let dpi = window.scale_factor() as f32;
        let proxy = match self.config.threads {
            true => SceneProxy::new(self.config.render_level, RayonExecutor),
            false => SceneProxy::new(self.config.render_level, SequentialExecutor),
        };

        // Create a Pathfinder renderer.
        let render_mode = RendererMode {
            level: self.config.render_level,
        };
        let framebuffer_size = (self.window_size * dpi).to_i32();
        let dest = DestFramebuffer::full_window(framebuffer_size);

        let render_options = RendererOptions {
            dest,
            background_color: Some(self.config.background),
            show_debug_ui: false,
        };

        let renderer = Renderer::new(
            GLDevice::new(self.gl_version, 0),
            &*self.config.resource_loader,
            render_mode,
            render_options
        );
        let backend = NativeBackend::new(window.clone(), self.window_size);
        let mut ctx = Context::new(self.config.clone(), backend);
        let scale_factor = window.scale_factor() as f32;
        ctx.set_scale_factor(scale_factor);
        ctx.request_redraw();
        ctx.window_size = self.window_size;

        let app_state = AppState {
            gl_context,
            gl_surface,
            framebuffer_size,
            window: window.clone(),
            renderer,
            proxy,
            window_size: self.window_size,
            ctx,
            modifiers: ModifiersState::default(),
            cursor_pos: Vector2F::default(),
            dragging: false,
        };

        self.state.replace(app_state);

        Some(window)
    }
}

impl<U: 'static> Emitter<EventLoopProxy<U>> {
    pub fn send(&self, event: U) {
        let _ = self.inner.send_event(event);
    }
}

pub struct NativeBackend {
    window: Rc<Window>,
    window_size: Vector2F,
}

impl NativeBackend {
    pub fn new(window: Rc<Window>, window_size: Vector2F) -> NativeBackend {
        NativeBackend { window, window_size }
    }
}

impl Interactive for Scene {
    type Event = ();
    type Backend = NativeBackend;

    fn init(&mut self, ctx: &mut Context<Self::Backend>, sender: Emitter<Self::Event>) {
        ctx.set_view_box(self.view_box());
    }

    fn scene(&mut self, ctx: &mut Context<Self::Backend>) -> Scene {
        self.clone()
    }

    fn window_size_hint(&self) -> Option<Vector2F> {
        let size = self.view_box().size();
        if size.is_zero() {
            None
        } else {
            Some(size)
        }
    }
}

impl ViewBackend for NativeBackend {
    fn resize(&mut self, size: Vector2F) {
        if size != self.window_size {
            let physical_size = PhysicalSize::new(size.x() as u32, size.y() as u32);
            self.window.set_max_inner_size(Some(physical_size));
            self.window.request_redraw();
            self.window_size = size;
        }
    }

    fn get_scroll_factors(&self) -> (Vector2F, Vector2F) {
        (
            env_vec("PIXEL_SCROLL_FACTOR").unwrap_or(Vector2F::new(1.0, 1.0)),
            env_vec("LINE_SCROLL_FACTOR").unwrap_or(Vector2F::new(10.0, -10.0)),
        )
    }

    fn set_icon(&mut self, icon: Icon) {
        self.window.set_window_icon(
            Some(winit::window::Icon::from_rgba(icon.data, icon.width, icon.height).unwrap())
        );
    }
}

fn env_vec(name: &str) -> Option<Vector2F> {
    use tuple::{ Map, TupleElements, T2 };
    let val = std::env::var(name).ok()?;
    let t2 = T2::from_iter(val.splitn(2, ","))?;
    let T2(x, y) = t2.map(|s: &str| s.parse().ok()).collect()?;
    Some(Vector2F::new(x, y))
}

pub fn show<T>(item: T, config: Config) where T: Interactive<Backend = NativeBackend, Event = ()> {
    log::info!("creating event loop");
    let event_loop = EventLoop::<()>::with_user_event().build().unwrap();
    let window_size = item.window_size_hint().unwrap_or(vec2f(600.0, 400.0));

    log::info!("entering the event loop");

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(item.title(), window_size, config, item);
    let _ = event_loop.run_app(&mut app).unwrap();
}
