use pathfinder_gl::{ GLDevice, GLVersion };
use pathfinder_renderer::{
    concurrent::rayon::RayonExecutor,
    concurrent::scene_proxy::SceneProxy,
    gpu::{
        options::{ DestFramebuffer, RendererLevel, RendererMode, RendererOptions },
        renderer::Renderer,
    },
    scene::Scene,
    options::{ BuildOptions, RenderTransform },
};
use pathfinder_gpu::{ Device, TextureData, RenderTarget };
use pathfinder_geometry::{ vector::{ Vector2F, Vector2I }, rect::RectI, transform2d::Transform2F };
use pathfinder_color::ColorF;
use pathfinder_resources::embedded::EmbeddedResourceLoader;

use khronos_egl as egl;
use image::RgbaImage;
use egl::Instance;

pub struct Rasterizer {
    egl: Instance<egl::Static>,
    display: egl::Display,
    surface: egl::Surface,
    context: egl::Context,
    renderer: Option<(Renderer<GLDevice>, Vector2I, Option<ColorF>)>,
}

impl Rasterizer {
    pub fn new() -> Self {
        let egl = egl::Instance::new(egl::Static);

        let display = unsafe { egl.get_display(egl::DEFAULT_DISPLAY).expect("display") };
        let (_major, _minor) = egl.initialize(display).expect("init");

        let attrib_list = [
            egl::SURFACE_TYPE,
            egl::PBUFFER_BIT,
            egl::BLUE_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::RED_SIZE,
            8,
            egl::DEPTH_SIZE,
            8,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_BIT,
            egl::NONE,
        ];

        let config = egl.choose_first_config(display, &attrib_list).unwrap().unwrap();

        let pbuffer_attrib_list = [egl::WIDTH, 1, egl::HEIGHT, 1, egl::NONE];
        let surface = egl.create_pbuffer_surface(display, config, &pbuffer_attrib_list).unwrap();

        egl.bind_api(egl::OPENGL_API).expect("unable to select OpenGL API");

        let context = egl.create_context(display, config, None, &[egl::NONE]).unwrap();
        egl.make_current(display, Some(surface), Some(surface), Some(context)).unwrap();

        // Setup Open GL.
        gl::load_with(|name| egl.get_proc_address(name).unwrap() as *const std::ffi::c_void);

        Rasterizer {
            egl,
            display,
            surface,
            context,
            renderer: None,
        }
    }

    fn make_current(&self) {
        self.egl
            .make_current(self.display, Some(self.surface), Some(self.surface), Some(self.context))
            .unwrap();
    }

    fn renderer_for_size(
        &mut self,
        size: Vector2I,
        background: Option<ColorF>
    ) -> &mut Renderer<GLDevice> {
        let size = Vector2I::new((size.x() + 15) & !15, (size.y() + 15) & !15);

        // Check if we need to recreate the renderer
        let needs_recreation = self.renderer
            .as_ref()
            .map_or(true, |(_, current_size, current_bg)| {
                size != *current_size || background != *current_bg
            });

        if needs_recreation {
            let resource_loader = EmbeddedResourceLoader::new();
            let renderer_gl_version = GLVersion::GLES3;
            let device = GLDevice::new(renderer_gl_version, 0);

            let render_mode = RendererMode {
                level: RendererLevel::D3D9,
            };
            let dest = DestFramebuffer::full_window(size);
            let render_options = RendererOptions {
                dest,
                background_color: background,
                show_debug_ui: false,
            };

            let renderer = Renderer::new(device, &resource_loader, render_mode, render_options);
            self.renderer = Some((renderer, size, background));
        }

        &mut self.renderer.as_mut().unwrap().0
    }

    pub fn rasterize(&mut self, scene: Scene, background: Option<ColorF>) -> RgbaImage {
        self.make_current();

        let view_box = dbg!(scene.view_box());
        let size = view_box.size().ceil().to_i32();
        let transform = Transform2F::from_translation(-view_box.origin());

        let renderer = self.renderer_for_size(size, background);

        let options = BuildOptions {
            transform: RenderTransform::Transform2D(transform),
            dilation: Vector2F::default(),
            subpixel_aa_enabled: false,
        };

        // Use SceneProxy for building and rendering
        let mut proxy = SceneProxy::from_scene(scene, RendererLevel::D3D9, RayonExecutor);
        proxy.build_and_render(renderer, options);

        // Access device as a method (git version API)
        let texture_data_receiver = renderer
            .device()
            .read_pixels(&RenderTarget::Default, RectI::new(Vector2I::zero(), size));
        let pixels = match renderer.device().recv_texture_data(&texture_data_receiver) {
            TextureData::U8(pixels) => pixels,
            _ => panic!("Unexpected pixel format for default framebuffer!"),
        };

        RgbaImage::from_raw(size.x() as u32, size.y() as u32, pixels).unwrap()
    }
}

impl Drop for Rasterizer {
    fn drop(&mut self) {
        self.egl.terminate(self.display).unwrap();
    }
}

#[test]
fn test_render() {
    use pathfinder_geometry::rect::RectF;

    let mut scene = Scene::new();
    scene.set_view_box(RectF::new(Vector2F::zero(), Vector2F::new(100.0, 100.0)));
    Rasterizer::new().rasterize(scene, None);
}
