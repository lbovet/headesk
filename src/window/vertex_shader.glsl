#version 330 core

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;

out vec2 v_uv;

uniform float distance;
uniform vec2 offset;

void main() {
    float dist = distance > 0.0 ? distance : 1.0;
    gl_Position = vec4(vec2(pos.x, pos.y) + offset, 0.0, dist);
    v_uv = vec2(uv);
}
