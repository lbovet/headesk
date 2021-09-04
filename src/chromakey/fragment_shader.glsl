#version 330 core

uniform sampler2D u_buffer;
in vec2 v_uv;
uniform vec4 keyRGBA;    // key color as rgba
uniform vec2 keyCC;      // the CC part of YCC color model of key color
uniform vec2 range;      // the smoothstep range

out vec4 frag_color;

vec3 rgb2hsv(vec3 rgb) {
	float Cmax = max(rgb.r, max(rgb.g, rgb.b));
	float Cmin = min(rgb.r, min(rgb.g, rgb.b));
	float delta = Cmax - Cmin;

	vec3 hsv = vec3(0., 0., Cmax);

	if(Cmax > Cmin) {
		hsv.y = delta / Cmax;

		if(rgb.r == Cmax)
			hsv.x = (rgb.g - rgb.b) / delta;
		else {
			if(rgb.g == Cmax)
				hsv.x = 2. + (rgb.b - rgb.r) / delta;
			else
				hsv.x = 4. + (rgb.r - rgb.g) / delta;
		}
		hsv.x = fract(hsv.x / 6.);
	}
	return hsv;
}

vec2 RGBAToCC(vec3 color) {
	float y = 0.299 * color.r + 0.587 * color.g + 0.114 * color.b;
	return vec2((color.b - y) * 0.565, (color.r - y) * 0.713);
};

float chromaKey(vec3 color) {
	vec3 backgroundColor = keyRGBA.rgb;
	vec4 weights = vec4(7., 1., 0., 3.);

	vec3 hsv = rgb2hsv(color);
	vec3 target = rgb2hsv(backgroundColor);
	float rgb_dist = length(color.rgb - backgroundColor.rgb);
	float dist = length(weights * vec4((target - hsv).x, rgb_dist, RGBAToCC(color) - RGBAToCC(keyRGBA.rgb)));
	return 1. - smoothstep(0., 1., 3. * dist - 1);
}

vec4 blackKey(vec3 color) {
	if(color == vec3(0., 0., 0.)) {
		return vec4(0., 0., 0., 0.);
	} else {
		return vec4(color, 1.);
	}
}

vec4 desaturate(vec3 color) {
	vec3 hsv = rgb2hsv(color);
	vec3 target = rgb2hsv(keyRGBA.rgb);
	float sat = smoothstep(0, 0.4, length(target.x - hsv.x));
	float luma = dot(vec3(0.213, 0.715, 0.072) * color, vec3(1.));
	vec4 result = vec4(mix(vec3(luma), color, sat), 1.0);
	return result;
}

void main() {
	vec4 color = texture(u_buffer, v_uv);

	if(keyRGBA.rgb == vec3(0., 0., 0.)) {
		color = blackKey(color.rgb);
	}
	else if(keyRGBA != vec4(1., 1., 1., 1.)) {
		float incrustation = chromaKey(color.rgb);
		color = desaturate(color.rgb);
		color = mix(color, vec4(0, 0, 0, 0), incrustation);
	}

	frag_color = color;
}
