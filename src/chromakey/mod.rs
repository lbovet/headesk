use mini_gl_fb::gl;
use mini_gl_fb::Framebuffer;
pub struct ChromaKey {
    program: gl::types::GLuint,
    highlight: gl::types::GLint,
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
            program: fb.internal.program,
            highlight: gl::GetUniformLocation(
                fb.internal.program,
                b"highlight\0".as_ptr() as *const _,
            ),
        }
    }
}

impl ChromaKey {
    pub fn set_highlight(&mut self, value: bool) {
        unsafe {
            gl::ProgramUniform1i(self.program, self.highlight, value as i32);
        }
    }
}
