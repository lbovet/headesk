use std::cmp::max;
use std::cmp::min;
use std::process;
use std::time::Duration;
use std::time::Instant;

use mini_gl_fb::core;
use mini_gl_fb::glutin::dpi::LogicalSize;
use mini_gl_fb::glutin::event::{Event, ElementState, WindowEvent};
use mini_gl_fb::glutin::event::MouseButton;
use mini_gl_fb::glutin::event::VirtualKeyCode;
use mini_gl_fb::glutin::event_loop::EventLoop;
use mini_gl_fb::glutin::window::WindowBuilder;
use mini_gl_fb::glutin::ContextBuilder;
use mini_gl_fb::glutin::PossiblyCurrent;
use mini_gl_fb::glutin::WindowedContext;
use mini_gl_fb::BufferFormat;
use mini_gl_fb::MiniGlFb;
use mini_gl_fb::GlutinBreakout;
use mini_gl_fb::glutin::event::WindowEvent::KeyboardInput;
use mini_gl_fb::glutin::event_loop::ControlFlow;

use crate::camera::Camera;

pub fn create(mut camera: Camera) {
    let event_loop = EventLoop::new();

    let window_title = String::from("Headesk");
    let window_size = LogicalSize::new(640, 480);
    let buffer_size = LogicalSize::new(640, 480);

    let window_builder = WindowBuilder::new()
        .with_decorations(false)
        .with_always_on_top(true)
        .with_transparent(true)
        .with_title(window_title.to_string())
        .with_inner_size(window_size)
        .with_resizable(true);

    let context: WindowedContext<PossiblyCurrent> = unsafe {
        ContextBuilder::new()
            .build_windowed(window_builder, &event_loop)
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

    fb.internal
        .fb
        .use_fragment_shader(include_str!("./fragment_shader.glsl"));
    fb.change_buffer_format::<u8>(BufferFormat::BGR);

    let GlutinBreakout {
        context,
        mut fb,
    } = fb.glutin_breakout();

    let mut last_frame_instant = Instant::now();

    event_loop.run(move |event, _, flow| {
        if Instant::now() > last_frame_instant + Duration::from_millis(10) {
            camera.read( |data| {
                fb.update_buffer(data);
                fb.redraw();
                context.swap_buffers().unwrap();
            });
            last_frame_instant = Instant::now();
        }
        *flow = ControlFlow::WaitUntil(Instant::now()+Duration::from_millis(5));

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: KeyboardInput { input, .. }, .. } => {
                if let Some(k) = input.virtual_keycode {
                    if k == VirtualKeyCode::Escape && input.state == ElementState::Pressed {
                        *flow = ControlFlow::Exit;
                    }
                }
            }
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                context.resize(size);
                context.window().request_redraw();
            }
            Event::WindowEvent { event: WindowEvent::MouseInput { state, .. }, .. } => {
                if state == ElementState::Pressed {
                    context.window().drag_window().unwrap();
                }
            }
            Event::RedrawRequested(_) => {
                fb.redraw();
                context.swap_buffers().unwrap();
            }
            _ => {}
        }
    });
}
