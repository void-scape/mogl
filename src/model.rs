use crate::shader::{compile_shader, uniform};
use crate::{HEIGHT, WIDTH};
use glam::{Mat4, Quat, Vec3};
use glazer::glow::{self, HasContext};
use std::io::BufReader;

#[derive(Default)]
pub struct Memory {
    startup: bool,
    model_shader: Option<glow::Program>,
    model: Option<Model>,
    x: f32,
}

#[unsafe(no_mangle)]
pub fn handle_input(platform_input: glazer::PlatformInput<Memory>) {
    crate::default_handle_input(platform_input);
}

#[unsafe(no_mangle)]
pub fn update_and_render(
    glazer::PlatformUpdate {
        memory, delta, gl, ..
    }: glazer::PlatformUpdate<Memory>,
) {
    if !memory.startup {
        memory.startup = true;
        memory.model_shader = Some(shader(gl));
        memory.model = Some(Model::from_obj(
            gl,
            include_bytes!("../assets/xyzrgb-dragon.obj").as_slice(),
        ));
    }

    memory.x += delta * 0.2;
    let model_matrix = Mat4::from_scale_rotation_translation(
        Vec3::splat(0.05),
        Quat::from_rotation_y(memory.x),
        Vec3::new(0.0, -1.0, -9.0),
    );

    unsafe {
        gl.clear_color(0.1, 0.1, 0.1, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        if let (Some(model), Some(shader)) = (&memory.model, memory.model_shader) {
            model.render(gl, shader, Vec3::ZERO, model_matrix);
        }
        crate::report_errors(gl);
    }
}

pub fn shader(gl: &glow::Context) -> glow::Program {
    // TODO: Should this be explicit? Will this be a problem for other shaders?
    unsafe {
        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::LESS);

        let shader = compile_shader(
            gl,
            include_str!("shaders/model.vert"),
            include_str!("shaders/model.frag"),
        );
        gl.use_program(Some(shader));
        uniform(gl, shader, "proj_matrix", |location| {
            let proj_matrix = Mat4::perspective_rh_gl(
                90f32.to_radians(),
                WIDTH as f32 / HEIGHT as f32,
                1.0,
                1_000.0,
            );
            gl.uniform_matrix_4_f32_slice(location, false, &proj_matrix.to_cols_array());
        });

        uniform(gl, shader, "light_source", |location| {
            gl.uniform_3_f32(location, 10.0, 10.0, 5.0);
        });

        uniform(gl, shader, "ambient_brightness", |location| {
            gl.uniform_1_f32(location, 0.05);
        });

        shader
    }
}

pub struct Model {
    vao: glow::VertexArray,
    _vbo: glow::Buffer,
    _ebo: glow::Buffer,
    indices: i32,
}

impl Model {
    // https://learnopengl.com/code_viewer_gh.php?code=src/1.getting_started/5.1.transformations/transformations.cpp
    pub fn from_obj(gl: &glow::Context, bytes: &[u8]) -> Self {
        unsafe {
            let (vertices, indices) = load_obj(bytes);

            let vao = gl.create_vertex_array().unwrap();
            let vbo = gl.create_buffer().unwrap();
            let ebo = gl.create_buffer().unwrap();

            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let data =
                core::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * 4);
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            let data =
                core::slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * 4);
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, data, glow::STATIC_DRAW);

            let stride = 6 * 4;
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 3 * 4);
            gl.enable_vertex_attrib_array(1);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Self {
                vao,
                _vbo: vbo,
                _ebo: ebo,
                indices: indices.len() as i32,
            }
        }
    }

    pub fn render(
        &self,
        gl: &glow::Context,
        shader: glow::Program,
        camera_position: Vec3,
        model_matrix: Mat4,
    ) {
        unsafe {
            gl.use_program(Some(shader));
            uniform(gl, shader, "camera_position", |location| {
                gl.uniform_3_f32(
                    location,
                    camera_position.x,
                    camera_position.y,
                    camera_position.z,
                );
            });

            uniform(gl, shader, "model_matrix", |location| {
                gl.uniform_matrix_4_f32_slice(location, false, &model_matrix.to_cols_array());
            });

            gl.bind_vertex_array(Some(self.vao));
            gl.draw_elements(glow::TRIANGLES, self.indices, glow::UNSIGNED_INT, 0);
        }
    }
}

fn load_obj(bytes: &[u8]) -> (Vec<f32>, Vec<u32>) {
    let (models, _materials) = tobj::load_obj_buf(
        &mut BufReader::new(bytes),
        &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        },
        |_| Err(tobj::LoadError::OpenFileFailed),
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
