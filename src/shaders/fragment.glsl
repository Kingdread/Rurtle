#version 140
out vec4 color;

flat in vec4 v_color;

void main(void) {
    color = v_color;
}