#version 120
attribute vec2 coords;
attribute vec2 tex_coords;
uniform mat4 matrix;

varying vec2 v_tex_coords;

void main(void) {
     v_tex_coords = tex_coords;
     gl_Position = matrix * vec4(coords, 0, 1);
}