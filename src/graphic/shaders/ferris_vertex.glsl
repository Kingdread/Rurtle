#version 120
attribute vec2 coords;
attribute vec2 tex_coords;

uniform mat4 matrix;
uniform mat4 rotation_matrix;
uniform float tip_x;
uniform float tip_y;

varying vec2 v_tex_coords;

void main(void) {
    v_tex_coords = tex_coords;
    mat4 translate_origin = mat4(
        vec4(1, 0, 0, 0),
        vec4(0, 1, 0, 0),
        vec4(0, 0, 1, 0),
        vec4(-tip_x, -tip_y, 0, 1));
    mat4 translate_back = mat4(
        vec4(1, 0, 0, 0),
        vec4(0, 1, 0, 0),
        vec4(0, 0, 1, 0),
        vec4(tip_x, tip_y, 0, 1));
    // Read from bottom to top:
    // 1. Translate and move it to the origin
    // 2. Rotate
    // 3. Move back
    // 4. Translate coordinates to the range [0; 1]
    gl_Position = (matrix *
                   translate_back *
                   rotation_matrix *
                   translate_origin *
                   vec4(coords, 0, 1));
}