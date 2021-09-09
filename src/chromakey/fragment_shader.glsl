#version 330 core

in vec2 v_uv;

out vec4 frag_color;

uniform vec4 keyRGBA;
uniform sampler2D u_buffer;
uniform bool highlight;

uniform float distance;
uniform vec2 offset;

const vec2 pixel_size = vec2(1.0 / 640.0, 1.0 / 480.0);
const int antialias_radius = 4;

const int sample_radius = 6;

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

vec2 toUVCoords(vec2 viewportCoords) {
	return viewportCoords * distance + 0.5 * ((1.0 - distance) * vec2(1, 1) - offset * vec2(1, -1));
}

vec4 background() {
	if(highlight) {
		return vec4(0.6, 0., 0., 0.2);
	} else {
		return vec4(0, 0, 0, 0);
	}
}

vec3 sample(vec2 uv) {
	vec3 mixColor = vec3(0, 0, 0);
	for(int i = -sample_radius; i < sample_radius; i++) {
		for(int j = -sample_radius; j < sample_radius; j++) {
			mixColor += texture(u_buffer, uv + pixel_size * vec2(i, j)).rgb;
		}
	}
	return mixColor / pow(sample_radius, 2);
}

float sampleKeyHue() {
	vec3 topLeft = rgb2hsv(sample(toUVCoords(vec2(0.05, 0.05))));
	vec3 topRight = rgb2hsv(sample(toUVCoords(vec2(0.95, 0.05))));
	if( topLeft.s > 0.3 && abs(topLeft.x - topRight.x) < 0.05) {
		return (topLeft.x + topRight.x) / 2.0;
	} else {
		return -1.0;
	}
}

float chromaKey(vec3 color, float keyHue) {
	vec3 color_hsv = rgb2hsv(color);
	if(color_hsv.y < 0.2 || color_hsv.z < 0.3) {
		return 0.0;
	}
	float dist = abs(color_hsv.x - keyHue);
	return 1 - smoothstep(0., 0.15, dist);
}

float blackKeyAntialias() {
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

vec4 blackKey(vec3 color) {
	if(color == vec3(0., 0., 0.)) {
		return background();
	} else {
		return vec4(color, blackKeyAntialias());
	}
}

vec4 desaturate(vec3 color, float keyHue) {
	vec3 hsv = rgb2hsv(color);
	float sat = smoothstep(0, 0.4, abs(hsv.x - keyHue));
	float luma = dot(vec3(0.213, 0.715, 0.072) * color, vec3(1.));
	vec4 result = vec4(mix(vec3(luma), color, sat), 1.0);
	return result;
}

void main() {
	vec4 color = texture(u_buffer, v_uv);
	float keyHue = sampleKeyHue();
	if(keyRGBA.rgb == vec3(0., 0., 0.)) {
		color = blackKey(color.rgb);
	} else if(keyHue != -1) {
		float incrustation = chromaKey(color.rgb, keyHue);
		//color = desaturate(color.rgb, keyHue);
		color = mix(color, background(), incrustation);
	}
	frag_color = color;
}
