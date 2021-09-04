#version 330 core

uniform sampler2D u_buffer;
in vec2 v_uv;
uniform vec4 keyRGBA;    // key color as rgba
uniform vec2 keyCC;      // the CC part of YCC color model of key color
uniform vec2 range;      // the smoothstep range

out vec4 frag_color;

vec2 RGBToCC(vec4 rgba) {
    float Y = 0.299 * rgba.r + 0.587 * rgba.g + 0.114 * rgba.b;
    return vec2((rgba.b - Y) * 0.565, (rgba.r - Y) * 0.713);
}
void main() {
    vec4 color = texture2D(u_buffer,  v_uv);
    vec2 CC = RGBToCC(color);
    float dist = sqrt(pow(keyCC.x - CC.x, 2.0) + pow(keyCC.y - CC.y, 2.0));
    float mask = dist < 0.1 ? 0.0: 1.0;
    //smoothstep(range.x, range.y, dist);
    if (mask == 0.0) { frag_color = color; }
    else if (mask == 1.0) { frag_color =  vec4(0,0,0,0); }
    else { frag_color = max(color - mask * keyRGBA, 0.0); }
}