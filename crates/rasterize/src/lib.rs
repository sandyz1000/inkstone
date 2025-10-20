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
use pathfinder_geometry::{ vector::{ Vector2F, Vector2I }, transform2d::Transform2F };
use pathfinder_color::ColorF;
use pathfinder_resources::embedded::EmbeddedResourceLoader;
use image::RgbaImage;

// Platform-specific OpenGL context management
#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
mod egl_backend;

// Platform-specific context wrapper
#[cfg(target_os = "macos")]
use macos::MacOSGLContext as GLContext;

#[cfg(not(target_os = "macos"))]
use egl_backend::EGLContext as GLContext;

pub struct Rasterizer {
    context: GLContext,
    renderer: Option<(Renderer<GLDevice>, Vector2I, Option<ColorF>, u32, u32, u32)>, // FBO, color_tex, depth_rb
}

impl Rasterizer {
    pub fn new() -> Self {
        let context = GLContext::new();
        
        Rasterizer {
            context,
            renderer: None,
        }
    }

    fn make_current(&mut self) {
        self.context.make_current();
    }
    
    fn restore_context(&mut self) {
        self.context.restore_previous();
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
            .map_or(true, |(_, current_size, current_bg, _, _, _)| {
                size != *current_size || background != *current_bg
            });

        if needs_recreation {
            // Clean up old FBO if it exists
            if let Some((_, _, _, old_fbo, old_tex, old_rb)) = self.renderer.take() {
                unsafe {
                    gl::DeleteFramebuffers(1, &old_fbo);
                    gl::DeleteTextures(1, &old_tex);
                    gl::DeleteRenderbuffers(1, &old_rb);
                }
            }

            // Create FBO with color and depth attachments before renderer
            let (fbo, color_texture, depth_renderbuffer) = unsafe {
                let mut fbo = 0;
                gl::GenFramebuffers(1, &mut fbo);
                gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

                // Create color texture
                let mut color_texture = 0;
                gl::GenTextures(1, &mut color_texture);
                gl::BindTexture(gl::TEXTURE_2D, color_texture);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    size.x(),
                    size.y(),
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    std::ptr::null(),
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    color_texture,
                    0,
                );

                // Create depth renderbuffer
                let mut depth_renderbuffer = 0;
                gl::GenRenderbuffers(1, &mut depth_renderbuffer);
                gl::BindRenderbuffer(gl::RENDERBUFFER, depth_renderbuffer);
                gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT24, size.x(), size.y());
                gl::FramebufferRenderbuffer(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_ATTACHMENT,
                    gl::RENDERBUFFER,
                    depth_renderbuffer,
                );

                // Check framebuffer status
                let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
                if status != gl::FRAMEBUFFER_COMPLETE {
                    panic!("Framebuffer is not complete: 0x{:x}", status);
                }

                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

                (fbo, color_texture, depth_renderbuffer)
            };

            let resource_loader = EmbeddedResourceLoader::new();
            let renderer_gl_version = GLVersion::GL3;
            let device = GLDevice::new(renderer_gl_version, fbo);

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
            self.renderer = Some((renderer, size, background, fbo, color_texture, depth_renderbuffer));
        }

        &mut self.renderer.as_mut().unwrap().0
    }

    pub fn rasterize(&mut self, scene: Scene, background: Option<ColorF>) -> RgbaImage {
        // Make our CGL context current
        self.make_current();
        
        let view_box = scene.view_box();
        let size = view_box.size().ceil().to_i32();
        let transform = Transform2F::from_translation(-view_box.origin());

        // Get renderer and FBO separately to avoid borrow issues
        {
            let _ = self.renderer_for_size(size, background);
        }
        
        let fbo = self.renderer.as_ref().map(|(_, _, _, fbo, _, _)| *fbo).unwrap();

        // Bind and clear the framebuffer
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            
            // Clear the framebuffer
            if let Some(bg) = background {
                gl::ClearColor(bg.r(), bg.g(), bg.b(), bg.a());
            } else {
                gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            }
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let options = BuildOptions {
            transform: RenderTransform::Transform2D(transform),
            dilation: Vector2F::default(),
            subpixel_aa_enabled: false,
        };

        // Use SceneProxy for building and rendering
        let mut proxy = SceneProxy::from_scene(scene, RendererLevel::D3D9, RayonExecutor);
        let renderer = &mut self.renderer.as_mut().unwrap().0;
        proxy.build_and_render(renderer, options);

        // Read pixels from the framebuffer
        let pixels = unsafe {
            let mut pixels = vec![0u8; (size.x() * size.y() * 4) as usize];
            gl::ReadPixels(
                0,
                0,
                size.x(),
                size.y(),
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr() as *mut _,
            );
            
            // Check for GL errors
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                panic!("GL error after ReadPixels: 0x{:x}", error);
            }
            
            pixels
        };

        // Unbind framebuffer
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            
            // Flush and finish all GL commands before returning
            gl::Finish();
        }
        
        // Restore the previous OpenGL context
        self.restore_context();

        // Create image and flip it vertically to correct OpenGL coordinate system
        // OpenGL has origin at bottom-left, but images have origin at top-left
        let mut img = RgbaImage::from_raw(size.x() as u32, size.y() as u32, pixels).unwrap();
        image::imageops::flip_vertical_in_place(&mut img);
        img
    }
}

impl Default for Rasterizer {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test_render() {
    use pathfinder_geometry::rect::RectF;

    let mut scene = Scene::new();
    scene.set_view_box(RectF::new(Vector2F::zero(), Vector2F::new(100.0, 100.0)));
    Rasterizer::new().rasterize(scene, None);
}
