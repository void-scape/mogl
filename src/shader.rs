use glazer::gl;
use std::{ffi::CStr, io::Write};

pub type Shader = u32;
pub type UniformLocation = i32;

#[macro_export]
macro_rules! compile_shader {
    ($vertex:literal, $fragment:literal) => {
        $crate::shader::compile_shader(
            concat!(include_str!($vertex), '\0'),
            concat!(include_str!($fragment), '\0'),
        )
    };
}

pub fn with_shader<F: FnOnce()>(shader: Shader, f: F) {
    unsafe { gl::UseProgram(shader) };
    f()
}

pub fn uniform<F: FnOnce(UniformLocation)>(shader: Shader, uniform_ident: &CStr, f: F) {
    f(unsafe { gl::GetUniformLocation(shader, uniform_ident.as_ptr()) })
}

// https://learnopengl.com/Getting-started/Shaders
pub fn compile_shader(vertex: &str, fragment: &str) -> Shader {
    unsafe {
        let shaders = [vertex, fragment];
        let vert = gl::CreateShader(gl::VERTEX_SHADER);
        let frag = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(vert, 1, shaders.as_ptr().cast(), core::ptr::null());
        gl::ShaderSource(frag, 1, shaders.as_ptr().add(1).cast(), core::ptr::null());

        gl::CompileShader(vert);
        let mut success = 0;
        gl::GetShaderiv(vert, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut buf = [0; 512];
            gl::GetShaderInfoLog(vert, 512, core::ptr::null_mut(), buf.as_mut_ptr().cast());
            let end = buf.iter().position(|c| *c == 0).unwrap_or(511);
            println!("[ERROR] failed to compile vertex shader: ");
            _ = std::io::stdout().lock().write(&buf[..end]);
            println!();
            std::process::exit(1);
        }

        gl::CompileShader(frag);
        let mut success = 0;
        gl::GetShaderiv(frag, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut buf = [0; 512];
            gl::GetShaderInfoLog(frag, 512, core::ptr::null_mut(), buf.as_mut_ptr().cast());
            let end = buf.iter().position(|c| *c == 0).unwrap_or(511);
            println!("[ERROR] failed to compile fragment shader: ");
            _ = std::io::stdout().lock().write(&buf[..end]);
            println!();
            std::process::exit(1);
        }

        let shader = gl::CreateProgram();
        gl::AttachShader(shader, vert);
        gl::AttachShader(shader, frag);
        gl::LinkProgram(shader);
        let mut success = 0;
        gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut buf = [0; 512];
            gl::GetProgramInfoLog(shader, 512, core::ptr::null_mut(), buf.as_mut_ptr().cast());
            let end = buf.iter().position(|c| *c == 0).unwrap_or(511);
            println!("[ERROR] failed to link shaders: ");
            _ = std::io::stdout().lock().write(&buf[..end]);
            println!();
            std::process::exit(1);
        }

        gl::DeleteShader(vert);
        gl::DeleteShader(frag);

        shader
    }
}
