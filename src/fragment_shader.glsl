#version 330 core

in vec2 v_uv;

out vec4 frag_color;

uniform sampler2D u_buffer;

void main() {
    frag_color = vec4(texture(u_buffer, v_uv).rgb, 0.5);
}