#version 330 core

uniform float ambient_brightness;
uniform vec3 light_source;
uniform vec3 camera_position;

in vec3 frag_position;
in vec3 frag_normal;
out vec4 c;

// phong lighting (ambient + diffuse + specular)
// 
// https://learnopengl.com/Lighting/Basic-Lighting
void main() {
	float specular_strength = 0.75;
	vec3 light_color = vec3(1.0);
	vec3 color = vec3(1.0, 0.5, 0.2);

	// diffuse
	vec3 norm = normalize(frag_normal);
	vec3 light_dir = normalize(light_source - frag_position);  
	float diff = max(dot(norm, light_dir), 0.0);
	vec3 diffuse = diff * light_color;

	// specular
	vec3 view_dir = normalize(camera_position - frag_position);
	vec3 reflect_dir = reflect(-light_dir, norm); 
	float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32);
	vec3 specular = specular_strength * spec * light_color;  

	vec3 result = (ambient_brightness + diffuse + specular) * color;
    c = vec4(result, 1.0);
} 
