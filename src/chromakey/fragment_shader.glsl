#version 330 core

in vec2 v_uv;

out vec4 frag_color;

uniform sampler2D u_buffer;
uniform bool highlight;

uniform float distance;
uniform vec2 offset;

const vec4 highlight_color = vec4(0.6, 0.0, 0.0, 0.2);
const vec3 black = vec3(0.0, 0.0, 0.0);
const vec4 transparent = vec4(black, 0.0);
const vec2 pixel_size = vec2(1.0 / 640.0, 1.0 / 480.0);
const int antialias_radius = 4;

vec3 rgb_to_hsv(vec3 c) {
	vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
	vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
	vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

	float d = q.x - min(q.w, q.y);
	float e = 1.0e-10;
	return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec2 to_uv_coords(vec2 vp_coords) {
	return vp_coords * distance + 0.5 * ((1.0 - distance) * vec2(1.0, 1.0) - offset * vec2(1.0, -1.0));;
}

vec4 background() {
	return highlight ? highlight_color : transparent;
}

vec3 sample(sampler2D image, vec2 uv) {
	return texture(image, uv + pixel_size * vec2(-1, -1)).rgb +
		texture(image, uv + pixel_size * vec2(-1, 0)).rgb +
		texture(image, uv + pixel_size * vec2(-1, 1)).rgb +
		texture(image, uv + pixel_size * vec2(0, -1)).rgb +
		texture(image, uv + pixel_size * vec2(0, 0)).rgb +
		texture(image, uv + pixel_size * vec2(0, 1)).rgb +
		texture(image, uv + pixel_size * vec2(1, -1)).rgb +
		texture(image, uv + pixel_size * vec2(1, 0)).rgb +
		texture(image, uv + pixel_size * vec2(1, 1)).rgb / 9;
}

float key_hue(sampler2D image) {
	vec3 top_left_color = rgb_to_hsv(sample(image, to_uv_coords(vec2(0.05, 0.05))));
	vec3 top_right_color = rgb_to_hsv(sample(image, to_uv_coords(vec2(0.95, 0.05))));
	if(top_left_color.s > 0.3 && abs(top_left_color.x - top_right_color.x) < 0.05) {
		return (top_left_color.x + top_right_color.x) / 2.0;
	} else {
		return -1.0;
	}
}

float chroma_key(vec3 color, float key_hue) {
	vec3 color_hsv = rgb_to_hsv(color);
	if(color_hsv.y < 0.3 || color_hsv.z < 0.2) {
		return 0.0;
	}
	float diff = color_hsv.x - key_hue;
	if(abs(diff) < 0.11 && length(vec2(color_hsv.y, color_hsv.z)) > 0.7) {
		return 1 - smoothstep(0.08, 0.11, abs(diff));
	}
	return 0.0;
}

float black_key_antialias() {
	float transparency = 0.0;
	for(int i = -antialias_radius; i < antialias_radius; i++) {
		for(int j = -antialias_radius; j < antialias_radius; j++) {
			if(texture(u_buffer, v_uv + pixel_size * vec2(i, j)).rgb == vec3(0., 0., 0.)) {
				transparency++;
			}
		}
	}
	return 1.0 - transparency / pow(antialias_radius, 2);
}

vec4 black_key(vec3 color) {
	return color == black ? background() : vec4(color, black_key_antialias());
}

vec4 desaturate(vec3 color, float keyHue) {
	vec3 hsv = rgb_to_hsv(color);
	float sat = smoothstep(0, 0.3, abs(hsv.x - keyHue));
	float luma = dot(vec3(0.213, 0.715, 0.072) * color, vec3(1.));
	return vec4(mix(vec3(luma), color, sat), 1.0);
}

void main() {
	vec4 color = texture(u_buffer, v_uv);
	if(texture(u_buffer, to_uv_coords(vec2(0.05, 0.05))).rgb == black) {
		color = black_key(color.rgb);
	} else {
		float chroma_hue = key_hue(u_buffer);
		if(chroma_hue != -1) {
			float incrustation = chroma_key(color.rgb, chroma_hue);
			color = desaturate(color.rgb, chroma_hue);
			color = mix(color, background(), incrustation);
		}
	}
	frag_color = color;
}
