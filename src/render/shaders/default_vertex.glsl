
#version 330
in vec3 pos;
in vec3 normal;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

out vec3 vertex_normal;
out vec3 vertex_position;

void main() {
    mat4 modelview = view * model;
    gl_Position = perspective * modelview * vec4(pos, 1.0);
    vertex_normal = transpose(inverse(mat3(modelview))) * normal;
    vertex_position = vec3(modelview * vec4(pos, 1.0));
}