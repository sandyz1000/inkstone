use egl::Instance;
/// EGL-based OpenGL context for Linux and other platforms
use khronos_egl as egl;

pub struct EGLContext {
    egl: Instance<egl::Static>,
    display: egl::Display,
    surface: egl::Surface,
    context: egl::Context,
    previous_context: Option<egl::Context>,
    previous_draw_surface: Option<egl::Surface>,
    previous_read_surface: Option<egl::Surface>,
}

impl EGLContext {
    pub fn new() -> Self {
        let egl = egl::Instance::new(egl::Static);

        let display = unsafe {
            egl.get_display(egl::DEFAULT_DISPLAY)
                .expect("Failed to get EGL display")
        };
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
            previous_context: None,
            previous_draw_surface: None,
            previous_read_surface: None,
        }
    }

    pub fn make_current(&mut self) {
        // Save the current context before switching
        let current_context = self.egl.get_current_context();
        let current_draw_surface = self.egl.get_current_surface(egl::DRAW);
        let current_read_surface = self.egl.get_current_surface(egl::READ);

        // Only save if there's actually a current context and it's different from ours
        if current_context.is_some() && current_context != Some(self.context) {
            self.previous_context = current_context;
            self.previous_draw_surface = current_draw_surface;
            self.previous_read_surface = current_read_surface;
        }

        self.egl
            .make_current(
                self.display,
                Some(self.surface),
                Some(self.surface),
                Some(self.context),
            )
            .expect("Failed to make EGL context current");
    }

    pub fn restore_previous(&mut self) {
        if let Some(prev_context) = self.previous_context {
            let _ = self.egl.make_current(
                self.display,
                self.previous_draw_surface,
                self.previous_read_surface,
                Some(prev_context),
            );
            log::debug!("✓ Previous EGL context restored");

            // Clear the saved context
            self.previous_context = None;
            self.previous_draw_surface = None;
            self.previous_read_surface = None;
        } else {
            // Clear the current context
            let _ = self.egl.make_current(self.display, None, None, None);
        }
    }
}

impl Drop for EGLContext {
    fn drop(&mut self) {
        // Restore previous context before destroying
        self.restore_previous();

        let _ = self.egl.terminate(self.display);
    }
}
