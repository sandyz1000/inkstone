/// macOS-specific OpenGL context using CGL (Core OpenGL)
use std::os::raw::{c_int, c_void};
use std::ptr;

// CGL types and constants
#[repr(C)]
#[derive(Copy, Clone)]
struct CGLPixelFormatObj(*mut c_void);

#[repr(C)]
#[derive(Copy, Clone)]
struct CGLContextObj(*mut c_void);

type CGLPixelFormatAttribute = u32;
type CGLError = c_int;

#[allow(non_upper_case_globals)]
const kCGLPFAAccelerated: CGLPixelFormatAttribute = 73;
#[allow(non_upper_case_globals)]
const kCGLPFAOpenGLProfile: CGLPixelFormatAttribute = 99;
#[allow(non_upper_case_globals)]
const kCGLOGLPVersion_3_2_Core: CGLPixelFormatAttribute = 0x3200;
#[allow(non_upper_case_globals)]
const kCGLPFAColorSize: CGLPixelFormatAttribute = 8;
#[allow(non_upper_case_globals)]
const kCGLPFAAlphaSize: CGLPixelFormatAttribute = 11;
#[allow(non_upper_case_globals)]
const kCGLPFADepthSize: CGLPixelFormatAttribute = 12;
#[allow(non_upper_case_globals)]
const kCGLPFAStencilSize: CGLPixelFormatAttribute = 13;

#[link(name = "OpenGL", kind = "framework")]
extern "C" {
    fn CGLChoosePixelFormat(
        attribs: *const CGLPixelFormatAttribute,
        pix: *mut CGLPixelFormatObj,
        npix: *mut c_int,
    ) -> CGLError;
    
    fn CGLCreateContext(
        pix: CGLPixelFormatObj,
        share: CGLContextObj,
        ctx: *mut CGLContextObj,
    ) -> CGLError;
    
    fn CGLDestroyPixelFormat(pix: CGLPixelFormatObj) -> CGLError;
    
    fn CGLSetCurrentContext(ctx: CGLContextObj) -> CGLError;
    
    fn CGLDestroyContext(ctx: CGLContextObj) -> CGLError;
    
    fn CGLGetCurrentContext() -> CGLContextObj;
}

pub struct MacOSGLContext {
    pixel_format: CGLPixelFormatObj,
    context: CGLContextObj,
    previous_context: Option<CGLContextObj>,
}

impl MacOSGLContext {
    pub fn new() -> Self {
        unsafe {
            // Define pixel format attributes for OpenGL 3.2 Core Profile
            let attribs: [CGLPixelFormatAttribute; 12] = [
                kCGLPFAAccelerated,
                kCGLPFAOpenGLProfile,
                kCGLOGLPVersion_3_2_Core,
                kCGLPFAColorSize, 24,
                kCGLPFAAlphaSize, 8,
                kCGLPFADepthSize, 24,
                kCGLPFAStencilSize, 8,
                0, // Terminator
            ];

            let mut pixel_format = CGLPixelFormatObj(ptr::null_mut());
            let mut npix: c_int = 0;

            let result = CGLChoosePixelFormat(attribs.as_ptr(), &mut pixel_format, &mut npix);
            if result != 0 {
                panic!("Failed to choose pixel format: error code {}", result);
            }

            if pixel_format.0.is_null() {
                panic!("No suitable pixel format found");
            }

            log::info!("✓ macOS CGL pixel format created successfully");

            // Create OpenGL context
            let mut context = CGLContextObj(ptr::null_mut());
            let result = CGLCreateContext(
                pixel_format,
                CGLContextObj(ptr::null_mut()), // No shared context
                &mut context,
            );

            if result != 0 {
                CGLDestroyPixelFormat(pixel_format);
                panic!("Failed to create CGL context: error code {}", result);
            }

            if context.0.is_null() {
                CGLDestroyPixelFormat(pixel_format);
                panic!("CGL context is null");
            }

            log::info!("✓ macOS CGL context created successfully");

            MacOSGLContext {
                pixel_format,
                context,
                previous_context: None,
            }
        }
    }

    pub fn make_current(&mut self) {
        unsafe {
            // Save current context before switching
            let current = CGLGetCurrentContext();
            if !current.0.is_null() && current.0 != self.context.0 {
                self.previous_context = Some(current);
            }
            
            let result = CGLSetCurrentContext(self.context);
            if result != 0 {
                log::warn!("Failed to make CGL context current: error code {}", result);
            }
            
            // Load OpenGL function pointers if not already loaded
            gl::load_with(|name| {
                let symbol_name = format!("{}\0", name);
                let symbol = libc::dlsym(libc::RTLD_DEFAULT, symbol_name.as_ptr() as *const i8);
                symbol as *const c_void
            });

            log::debug!("✓ CGL context made current");
        }
    }
    
    pub fn restore_previous(&mut self) {
        unsafe {
            if let Some(prev) = self.previous_context {
                let result = CGLSetCurrentContext(prev);
                if result != 0 {
                    log::warn!("Failed to restore previous CGL context: error code {}", result);
                } else {
                    log::debug!("✓ Previous CGL context restored");
                }
                self.previous_context = None;
            } else {
                // Clear the context
                CGLSetCurrentContext(CGLContextObj(ptr::null_mut()));
            }
        }
    }
}

impl Drop for MacOSGLContext {
    fn drop(&mut self) {
        unsafe {
            // Restore previous context before destroying
            self.restore_previous();
            
            if !self.context.0.is_null() {
                CGLDestroyContext(self.context);
            }
            if !self.pixel_format.0.is_null() {
                CGLDestroyPixelFormat(self.pixel_format);
            }
        }
    }
}

// Safety: CGL contexts are thread-safe when properly synchronized
unsafe impl Send for MacOSGLContext {}
unsafe impl Sync for MacOSGLContext {}
