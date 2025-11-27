#version 300 es

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;

uniform mat4 proj_matrix;
uniform mat4 model_matrix;

out vec3 frag_position;
out vec3 frag_normal;

void main() {
	gl_Position = proj_matrix * model_matrix * vec4(position, 1.0);
	frag_position = vec3(model_matrix * vec4(position, 1.0));
	frag_normal = mat3(transpose(inverse(model_matrix))) * normal;
}
