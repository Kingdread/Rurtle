#version 120
varying vec2 v_tex_coords;

uniform sampler2D texture_data;

void main(void) {
    gl_FragColor = texture2D(texture_data, v_tex_coords);
    if (gl_FragColor.a < 0.5) {
        discard;
    }
}