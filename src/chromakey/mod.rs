use std::time::{Duration, Instant};

use color::{rgb, Hsv, Rgb, ToHsv};
use mini_gl_fb::gl;
use mini_gl_fb::Framebuffer;

const WHITE: Rgb<u8> = rgb!(255, 255, 255);
const BLACK: Rgb<u8> = rgb!(0, 0, 0);

pub struct ChromaKey {
    calibration_key: Rgb, // candidate for color key from the previous iteration
    last_calibration_time: Instant,
    program: gl::types::GLuint,
    key_rgba_loc: gl::types::GLint,
}

/// Creates a chroma key context. It automatically configures the OpenGL context.
pub fn new(fb: &mut Framebuffer) -> ChromaKey {
    fb.use_fragment_shader(include_str!("./fragment_shader.glsl"));
    unsafe {
        let range_loc =
            gl::GetUniformLocation(fb.internal.program, b"range\0".as_ptr() as *const _);
        gl::ProgramUniform2f(fb.internal.program, range_loc, 0.01, 0.19);

        gl::BindTexture(gl::TEXTURE_2D, fb.internal.texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        gl::BindTexture(gl::TEXTURE_2D, 0);

        ChromaKey {
            calibration_key: BLACK,
            last_calibration_time: Instant::now() - Duration::from_millis(2000),
            program: fb.internal.program,
            key_rgba_loc: gl::GetUniformLocation(
                fb.internal.program,
                b"keyRGBA\0".as_ptr() as *const _,
            ),
        }
    }
}

impl ChromaKey {
    /// Requests calibration of the chroma key. Should be called on every frame.
    pub fn calibrate(&mut self, data: &[u8], width: u32) {
        if Instant::now() > self.last_calibration_time + Duration::from_millis(200) {
            let p = ((width * 3 as u32) - 3) as usize;
            let samples = [
                // buffer is BGR, we swap red and blue
                rgb!(data[2], data[1], data[0]),
                rgb!(data[p - 1], data[p - 2], data[p - 3]),
                self.calibration_key,
            ];

            if samples[0] == BLACK && samples[1] == BLACK {
                // for virtual cameras setting the background to pure black
                self.set_key(BLACK);
            } else if let Some(color) = compute_key(&samples) {
                let hsv: Hsv<f32> = rgb!(color.r as f32, color.g as f32, color.b as f32).to_hsv();
                // ignore dark and greyish/redish backgrounds
                if hsv.s > 0.2 && hsv.v > 80.0 && color.r < 80 {
                    self.set_key(color);
                    self.calibration_key = color;
                }
            } else {
                // no calibration, no chroma key
                self.calibration_key = samples[0];
                self.set_key(WHITE);
            }
            self.last_calibration_time = Instant::now()
        }

        /// Computes an average color if both top corners and the candidate color are similar.
        fn compute_key(samples: &[Rgb]) -> Option<Rgb> {
            let mut result = samples[0];
            for i in 1..samples.len() {
                if similar(result, samples[i], 7.0) {
                    result = mix(result, samples[i]);
                } else {
                    return None;
                }
            }
            Some(result)
        }

        fn mix(c1: Rgb, c2: Rgb) -> Rgb {
            let r = (c1.r as u32 + c2.r as u32) / 2;
            let g = (c1.g as u32 + c2.g as u32) / 2;
            let b = (c1.b as u32 + c2.b as u32) / 2;
            rgb!(r as u8, g as u8, b as u8)
        }

        fn similar(c1: Rgb, c2: Rgb, tolerance: f32) -> bool {
            let (cb1, cr1) = color_to_cc(c1);
            let (cb2, cr2) = color_to_cc(c2);

            let db = cb1 - cb2;
            let dr = cr1 - cr2;
            (db * db + dr * dr).sqrt() < tolerance
        }

        fn color_to_cc(rgb: Rgb) -> (f32, f32) {
            let r = rgb.r as f32;
            let g = rgb.g as f32;
            let b = rgb.b as f32;
            rgb_to_cc(r, g, b)
        }
    }

    fn set_key(&mut self, color: Rgb) {
        unsafe {
            let r = color.r as f32 / 255.0;
            let g = color.g as f32 / 255.0;
            let b = color.b as f32 / 255.0;
            gl::ProgramUniform4f(self.program, self.key_rgba_loc, r, g, b, 1.0);
        }
    }
}

fn rgb_to_cc(r: f32, g: f32, b: f32) -> (f32, f32) {
    let y = 0.299 * r + 0.587 * g + 0.114 * b;
    ((b - y) * 0.565, (r - y) * 0.713)
}
