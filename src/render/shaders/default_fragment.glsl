#version 330

in vec3 vertex_normal;
in vec3 vertex_position;

out vec4 color;

uniform vec3 light_direction;
uniform vec3 diffuse_color;

const vec3 u_ambient_intensity = vec3(0.05, 0.05, 0.05);
const vec3 u_diffuse_intensity = vec3(0.6, 0.6, 0.6);
const vec3 u_specular_intensity = vec3(0.90, 0.90, 0.90);
const float shininess = 32.0;

vec3 specular_lighting(vec3 v_normal, vec3 light_dir, vec3 camera_dir) {
    vec3 half_direction = normalize(light_dir + camera_dir);

    float specular_weight = 0;
    if (dot(v_normal, light_dir) > 0) {
        specular_weight = pow(max(dot(half_direction, v_normal), 0.0), shininess);
    }

    return specular_weight * u_specular_intensity;
}

vec3 diffuse_lighting(vec3 v_normal, vec3 light_dir) {
    float diffuse_weight = max(dot(v_normal, light_dir), 0.0);
    return diffuse_weight * u_diffuse_intensity * diffuse_color;
}

vec3 ambient_lighting() {
    return u_ambient_intensity * u_diffuse_intensity * diffuse_color;
}

void main() {
    vec3 v_normal = normalize(vertex_normal);
    vec3 l_dir = normalize(light_direction);
    vec3 camera_dir = normalize(-vertex_position);

    vec3 ambient = ambient_lighting();
    vec3 diffuse = diffuse_lighting(v_normal, l_dir);
    vec3 specular = specular_lighting(v_normal, l_dir, camera_dir);

    color = vec4(ambient + diffuse + specular, 1.0);
}