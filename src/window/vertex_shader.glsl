#version 330 core

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
uniform float distance;

out vec2 v_uv;

void main() {
    float dist;
    if(distance > 0.0) {
        dist = distance;
    } else {
        dist = 1.0;
    }
    gl_Position = vec4(vec2(pos.x, pos.y+(1.0-dist)), 0.0, dist);
    v_uv = vec2(uv);
}
