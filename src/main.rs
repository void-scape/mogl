use glam::{Mat4, Quat, Vec3};
use glazer::gl;
use std::io::Write;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

fn main() {
    glazer::run_opengl(
        Memory::default(),
        WIDTH,
        HEIGHT,
        handle_input,
        update_and_render,
        None,
    );
}

#[derive(Default)]
struct Memory {
    startup: bool,
    model_shader: u32,
    model: Model,
    x: f32,
}

fn handle_input(glazer::PlatformInput { input, .. }: glazer::PlatformInput<Memory>) {
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

fn compute_normals(positions: &[f32], indices: &[u32]) -> Vec<f32> {
    let mut normals = vec![0.0f32; positions.len()];
    for tri in indices.chunks_exact(3) {
        let i0 = tri[0] as usize * 3;
        let i1 = tri[1] as usize * 3;
        let i2 = tri[2] as usize * 3;
        let v0 = Vec3::new(positions[i0], positions[i0 + 1], positions[i0 + 2]);
        let v1 = Vec3::new(positions[i1], positions[i1 + 1], positions[i1 + 2]);
        let v2 = Vec3::new(positions[i2], positions[i2 + 1], positions[i2 + 2]);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let face_normal = edge1.cross(edge2);

        for idx in tri[..3].iter() {
            let i = *idx as usize * 3;
            normals[i] += face_normal.x;
            normals[i + 1] += face_normal.y;
            normals[i + 2] += face_normal.z;
        }
    }
    for i in (0..normals.len()).step_by(3) {
        let normal = Vec3::new(normals[i], normals[i + 1], normals[i + 2]);
        let normalized = normal.normalize();
        normals[i] = normalized.x;
        normals[i + 1] = normalized.y;
        normals[i + 2] = normalized.z;
    }
    normals
}

fn load_obj(path: &str) -> (Vec<f32>, Vec<u32>) {
    let (models, _materials) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut index_offset = 0u32;
    for model in models.iter() {
        let mesh = &model.mesh;
        let normals = if mesh.normals.is_empty() {
            &compute_normals(&mesh.positions, &mesh.indices)
        } else {
            &mesh.normals
        };
        for i in 0..mesh.positions.len() / 3 {
            vertices.push(mesh.positions[i * 3]);
            vertices.push(mesh.positions[i * 3 + 1]);
            vertices.push(mesh.positions[i * 3 + 2]);
            vertices.push(normals[i * 3]);
            vertices.push(normals[i * 3 + 1]);
            vertices.push(normals[i * 3 + 2]);
        }
        for idx in &mesh.indices {
            indices.push(idx + index_offset);
        }
        index_offset += (mesh.positions.len() / 3) as u32;
    }
    (vertices, indices)
}

#[derive(Default)]
struct Model {
    vao: u32,
    _vbo: u32,
    _ebo: u32,
    indices: i32,
}

impl Model {
    // https://learnopengl.com/code_viewer_gh.php?code=src/1.getting_started/5.1.transformations/transformations.cpp
    fn from_obj(path: &str) -> Self {
        unsafe {
            let (vertices, indices) = load_obj(path);
            let mut vao = 0;
            let mut vbo = 0;
            let mut ebo = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                vertices.len() as isize * 4,
                vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                indices.len() as isize * 4,
                indices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            let stride = 6 * 4;
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, core::ptr::null_mut());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                ((3 * 4) as *const u8).cast(),
            );
            gl::EnableVertexAttribArray(1);

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            loop {
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!(
                        "[ERROR] OpenGL initialization failed with error code: {}",
                        error
                    );
                } else {
                    break;
                }
            }

            Self {
                vao,
                _vbo: vbo,
                _ebo: ebo,
                indices: indices.len() as i32,
            }
        }
    }
}

// https://learnopengl.com/Getting-started/Shaders
fn basic_model_shader() -> u32 {
    unsafe {
        let shaders = [
            concat!(include_str!("shader.vert"), '\0'),
            concat!(include_str!("shader.frag"), '\0'),
        ];

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

        gl::UseProgram(shader);
        let proj_matrix = Mat4::perspective_rh_gl(
            90f32.to_radians(),
            WIDTH as f32 / HEIGHT as f32,
            1.0,
            1_000.0,
        );
        let location = gl::GetUniformLocation(shader, c"proj_matrix".as_ptr());
        gl::UniformMatrix4fv(
            location,
            1,
            gl::FALSE,
            core::ptr::from_ref(&proj_matrix).cast(),
        );

        let location = gl::GetUniformLocation(shader, c"light_source".as_ptr());
        gl::Uniform3f(location, 10.0, 10.0, 5.0);

        let location = gl::GetUniformLocation(shader, c"ambient_brightness".as_ptr());
        gl::Uniform1f(location, 0.05);

        shader
    }
}

fn update_and_render(
    glazer::PlatformUpdateGL { memory, delta, .. }: glazer::PlatformUpdateGL<Memory>,
) {
    if !memory.startup {
        memory.startup = true;
        memory.model_shader = basic_model_shader();
        memory.model = Model::from_obj("assets/xyzrgb-dragon.obj");

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
        }
    }

    memory.x += delta * 0.2;
    let model_matrix = Mat4::from_scale_rotation_translation(
        Vec3::splat(0.05),
        Quat::from_rotation_y(memory.x),
        Vec3::new(0.0, -1.0, -9.0),
    );

    unsafe {
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        let location = gl::GetUniformLocation(memory.model_shader, c"camera_position".as_ptr());
        gl::Uniform3f(location, 0.0, 0.0, 0.0);

        let location = gl::GetUniformLocation(memory.model_shader, c"model_matrix".as_ptr());
        gl::UniformMatrix4fv(
            location,
            1,
            gl::FALSE,
            core::ptr::from_ref(&model_matrix).cast(),
        );

        gl::UseProgram(memory.model_shader);
        gl::BindVertexArray(memory.model.vao);
        gl::DrawElements(
            gl::TRIANGLES,
            memory.model.indices,
            gl::UNSIGNED_INT,
            core::ptr::null(),
        );
    }
}
