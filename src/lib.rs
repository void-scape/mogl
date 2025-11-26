use glazer::gl;

#[cfg_attr(not(feature = "model"), allow(unused))]
mod model;
mod shader;

#[cfg(feature = "model")]
pub use model::{Memory, handle_input, update_and_render};

pub const WIDTH: usize = 1280;
pub const HEIGHT: usize = 720;

pub fn report_errors() {
    loop {
        let error = unsafe { gl::GetError() };
        if error != gl::NO_ERROR {
            println!("[ERROR] OpenGL error code: {}", error);
        } else {
            break;
        }
    }
}

pub fn default_handle_input<Memory>(
    glazer::PlatformInput { input, .. }: glazer::PlatformInput<Memory>,
) {
    if matches!(
        input,
        glazer::Input::Key {
            code: glazer::KeyCode::Escape,
            ..
        }
    ) {
        std::process::exit(0);
    }
}

#[unsafe(no_mangle)]
pub fn initialize_opengl(loader: &dyn Fn(&'static str) -> *const core::ffi::c_void) {
    gl::load_with(loader);
}
