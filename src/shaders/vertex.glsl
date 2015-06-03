#version 120
attribute vec2 coords;
attribute vec4 color;
uniform mat4 matrix;

varying vec4 v_color;

void main(void) {
     v_color = color;
     gl_Position = matrix * vec4(coords, 0, 1);
}