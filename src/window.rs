use mini_gl_fb::BasicInput;
use mini_gl_fb::core;
use mini_gl_fb::glutin::dpi::LogicalSize;
use mini_gl_fb::glutin::event_loop::EventLoop;
use mini_gl_fb::glutin::window::WindowBuilder;
use mini_gl_fb::glutin::ContextBuilder;
use mini_gl_fb::glutin::PossiblyCurrent;
use mini_gl_fb::glutin::WindowedContext;
use mini_gl_fb::BufferFormat;
use mini_gl_fb::MiniGlFb;
use mini_gl_fb::Framebuffer;

pub fn create<F: FnMut(&mut Framebuffer, &mut BasicInput) -> bool>(handler: F) {
    let mut event_loop = EventLoop::new();

    let window_title = String::from("Headesk");
    let window_size = LogicalSize::new(640, 480);
    let buffer_size = LogicalSize::new(640, 480);

    let window = WindowBuilder::new()
        .with_decorations(false)
        .with_always_on_top(true)
        .with_transparent(true)
        .with_title(window_title.to_string())
        .with_inner_size(window_size)
        .with_resizable(true);

    let context: WindowedContext<PossiblyCurrent> = unsafe {
        ContextBuilder::new()
            .build_windowed(window, &event_loop)
            .unwrap()
            .make_current()
            .unwrap()
    };

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    let (vp_width, vp_height) = context.window().inner_size().into();

    let fb = core::init_framebuffer(
        buffer_size.width,
        buffer_size.height,
        vp_width,
        vp_height,
        false,
    );

    let mut fb = MiniGlFb {
        internal: core::Internal { context, fb },
    };

    const FRAGMENT_SOURCE: &str = r"
    #version 330 core

    in vec2 v_uv;

    out vec4 frag_color;

    uniform sampler2D u_buffer;

    void main() {
        frag_color = vec4(texture(u_buffer, v_uv).rgb, 0.5);
    }
";
    fb.internal.fb.use_fragment_shader(FRAGMENT_SOURCE);
    fb.change_buffer_format::<u8>(BufferFormat::BGR);

    fb.glutin_handle_basic_input(&mut event_loop, handler);

    fb.persist(&mut event_loop);
}
