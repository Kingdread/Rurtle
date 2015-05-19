#version 140
in vec2 coords;
in vec4 color;
uniform mat4 matrix;

flat out vec4 v_color;

void main(void) {
     v_color = color;
     gl_Position = matrix * vec4(coords, 0, 1);
}