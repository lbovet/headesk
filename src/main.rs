use std::process;
use std::time::Duration;
use std::time::Instant;

use mini_gl_fb::core;
use mini_gl_fb::glutin::dpi::LogicalSize;
use mini_gl_fb::glutin::event::VirtualKeyCode;
use mini_gl_fb::glutin::event_loop::EventLoop;
use mini_gl_fb::glutin::window::WindowBuilder;
use mini_gl_fb::glutin::ContextBuilder;
use mini_gl_fb::glutin::PossiblyCurrent;
use mini_gl_fb::glutin::WindowedContext;
use mini_gl_fb::BufferFormat;
use mini_gl_fb::MiniGlFb;

mod camera;

fn main() {
    let mut event_loop = EventLoop::new();

    let window_title = String::from("Hello world!");
    let window_size = LogicalSize::new(640, 480);
    let buffer_size = LogicalSize::new(640, 480);

    let mut cam = camera::Camera::init(1);

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
        frag_color = texture(u_buffer, v_uv).rgbb;
    }
";
    fb.internal.fb.use_fragment_shader(FRAGMENT_SOURCE);
    fb.change_buffer_format::<u8>(BufferFormat::BGR);

    let mut update_id: Option<u32> = None;

    fb.glutin_handle_basic_input(&mut event_loop, |fb, input| {
        input.wait = true;
        if update_id.is_none() {
            update_id = Some(input.schedule_wakeup(Instant::now() + Duration::from_millis(10)))
        } else if let Some(mut wakeup) = input.wakeup {
            if Some(wakeup.id) == update_id {
                cam.read(&mut |data| {
                    fb.update_buffer(&data);
                });
                wakeup.when = Instant::now() + Duration::from_millis(5);
                input.reschedule_wakeup(wakeup);
            }
            return true;
        }

        if input.key_is_down(VirtualKeyCode::Escape) {
            cam.close();
            process::exit(0);
        }

        true
    });

    fb.persist(&mut event_loop);
}
