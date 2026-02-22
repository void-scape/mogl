use glazer::glow::{self, HasContext};

pub use model::{Memory, handle_input, update_and_render};

mod model;
mod shader;

pub const WIDTH: usize = 1280;
pub const HEIGHT: usize = 720;

pub fn report_errors(gl: &glow::Context) {
    loop {
        let error = unsafe { gl.get_error() };
        if error != glow::NO_ERROR {
            glazer::log!("[ERROR] OpenGL error code: {}", error);
        } else {
            break;
        }
    }
}

pub fn default_handle_input<Memory>(
    glazer::PlatformInput { input, .. }: glazer::PlatformInput<Memory>,
) {
    use glazer::winit;
    if matches!(
        input,
        glazer::Input::Window(winit::event::WindowEvent::KeyboardInput {
            event: winit::event::KeyEvent {
                physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                ..
            },
            ..
        })
    ) {
        std::process::exit(0);
    }
}
