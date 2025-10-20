/// EGL-based OpenGL context for Linux and other platforms
use khronos_egl as egl;
use egl::Instance;

pub struct EGLContext {
    egl: Instance<egl::Static>,
    display: egl::Display,
    surface: egl::Surface,
    context: egl::Context,
}

impl EGLContext {
    pub fn new() -> Self {
        let egl = egl::Instance::new(egl::Static);

        let display = unsafe { egl.get_display(egl::DEFAULT_DISPLAY).expect("Failed to get EGL display") };
        let (_major, _minor) = egl.initialize(display).expect("Failed to initialize EGL");

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

        let config = egl
            .choose_first_config(display, &attrib_list)
            .expect("Failed to choose EGL config")
            .expect("No suitable EGL config found");

        let pbuffer_attrib_list = [egl::WIDTH, 1, egl::HEIGHT, 1, egl::NONE];
        let surface = egl
            .create_pbuffer_surface(display, config, &pbuffer_attrib_list)
            .expect("Failed to create pbuffer surface");

        egl.bind_api(egl::OPENGL_API)
            .expect("Failed to bind OpenGL API");

        let context = egl
            .create_context(display, config, None, &[egl::NONE])
            .expect("Failed to create EGL context");

        egl.make_current(display, Some(surface), Some(surface), Some(context))
            .expect("Failed to make EGL context current");

        // Load OpenGL function pointers
        gl::load_with(|name| egl.get_proc_address(name).unwrap() as *const std::ffi::c_void);

        log::info!("✓ EGL context created successfully");

        EGLContext {
            egl,
            display,
            surface,
            context,
        }
    }

    pub fn make_current(&self) {
        self.egl
            .make_current(
                self.display,
                Some(self.surface),
                Some(self.surface),
                Some(self.context),
            )
            .expect("Failed to make EGL context current");
    }
}

impl Drop for EGLContext {
    fn drop(&mut self) {
        let _ = self.egl.terminate(self.display);
    }
}
