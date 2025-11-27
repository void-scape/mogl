use crate::shader::{Shader, uniform, with_shader};
use crate::{HEIGHT, WIDTH, compile_shader};
use glam::{Mat4, Quat, Vec3};
use glazer::gl;

#[derive(Default)]
pub struct Memory {
    startup: bool,
    model_shader: u32,
    model: Model,
    x: f32,
}

#[unsafe(no_mangle)]
pub fn handle_input(platform_input: glazer::PlatformInput<Memory>) {
    crate::default_handle_input(platform_input);
}

#[unsafe(no_mangle)]
pub fn update_and_render(
    glazer::PlatformUpdate { memory, delta, .. }: glazer::PlatformUpdate<Memory>,
) {
    if !memory.startup {
        memory.startup = true;
        memory.model_shader = shader();
        memory.model = Model::from_obj("assets/xyzrgb-dragon.obj");
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

        memory
            .model
            .render(memory.model_shader, Vec3::ZERO, model_matrix);

        crate::report_errors();
    }
}

pub fn shader() -> Shader {
    // TODO: Should this be explicit? Will this be a problem for other shaders?
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
    }

    let shader = compile_shader!("shaders/model.vert", "shaders/model.frag");
    with_shader(shader, || {
        uniform(shader, c"proj_matrix", |location| {
            let proj_matrix = Mat4::perspective_rh_gl(
                90f32.to_radians(),
                WIDTH as f32 / HEIGHT as f32,
                1.0,
                1_000.0,
            );
            unsafe {
                gl::UniformMatrix4fv(
                    location,
                    1,
                    gl::FALSE,
                    core::ptr::from_ref(&proj_matrix).cast(),
                );
            }
        });

        uniform(shader, c"light_source", |location| unsafe {
            gl::Uniform3f(location, 10.0, 10.0, 5.0);
        });

        uniform(shader, c"ambient_brightness", |location| unsafe {
            gl::Uniform1f(location, 0.05);
        });
    });
    shader
}

#[derive(Default)]
pub struct Model {
    vao: u32,
    _vbo: u32,
    _ebo: u32,
    indices: i32,
}

impl Model {
    // https://learnopengl.com/code_viewer_gh.php?code=src/1.getting_started/5.1.transformations/transformations.cpp
    pub fn from_obj(path: &str) -> Self {
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

            Self {
                vao,
                _vbo: vbo,
                _ebo: ebo,
                indices: indices.len() as i32,
            }
        }
    }

    pub fn render(&self, shader: Shader, camera_position: Vec3, model_matrix: Mat4) {
        with_shader(shader, || {
            uniform(shader, c"camera_position", |location| unsafe {
                gl::Uniform3f(
                    location,
                    camera_position.x,
                    camera_position.y,
                    camera_position.z,
                );
            });

            uniform(shader, c"model_matrix", |location| unsafe {
                gl::UniformMatrix4fv(
                    location,
                    1,
                    gl::FALSE,
                    core::ptr::from_ref(&model_matrix).cast(),
                );
            });

            unsafe {
                gl::BindVertexArray(self.vao);
                gl::DrawElements(
                    gl::TRIANGLES,
                    self.indices,
                    gl::UNSIGNED_INT,
                    core::ptr::null(),
                );
            }
        });
    }
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
