#version 140
in vec2 coords;
uniform mat4 matrix;

void main(void) {
     gl_Position = matrix * vec4(coords, 0, 1);
}