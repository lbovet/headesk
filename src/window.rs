use std::time::Duration;
use std::time::Instant;

use mini_gl_fb::core;
use mini_gl_fb::glutin::dpi::LogicalSize;
use mini_gl_fb::glutin::event::VirtualKeyCode;
use mini_gl_fb::glutin::event::WindowEvent::KeyboardInput;
use mini_gl_fb::glutin::event::{ElementState, Event, WindowEvent};
use mini_gl_fb::glutin::event_loop::ControlFlow;
use mini_gl_fb::glutin::event_loop::EventLoop;
use mini_gl_fb::glutin::window::WindowBuilder;
use mini_gl_fb::glutin::ContextBuilder;
use mini_gl_fb::glutin::PossiblyCurrent;
use mini_gl_fb::glutin::WindowedContext;
use mini_gl_fb::BufferFormat;
use mini_gl_fb::GlutinBreakout;
use mini_gl_fb::MiniGlFb;
use winit::dpi::PhysicalSize;
use winit::event::MouseScrollDelta;

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

    fb.set_resizable(true);

    fb.internal
        .fb
        .use_fragment_shader(include_str!("./fragment_shader.glsl"));
    fb.change_buffer_format::<u8>(BufferFormat::BGR);

    let GlutinBreakout { context, mut fb } = fb.glutin_breakout();

    let mut last_frame_instant = Instant::now();

    event_loop.run(move |event, _, flow| {
        let mut redraw = false;

        if Instant::now() > last_frame_instant + Duration::from_millis(10) {
            camera.read(|data| {
                fb.update_buffer(data);
                redraw = true;
            });
            last_frame_instant = Instant::now();
        }
        *flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(5));

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                camera.close();
                *flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: KeyboardInput { input, .. },
                ..
            } => {
                if let Some(k) = input.virtual_keycode {
                    if k == VirtualKeyCode::Escape && input.state == ElementState::Pressed {
                        camera.close();
                        *flow = ControlFlow::Exit;
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                context.resize(size);
                fb.resize_viewport(size.width, size.height);
                context.window().request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, .. },
                ..
            } => {
                if state == ElementState::Pressed {
                    context.window().drag_window().unwrap();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                let current_size = context.window().inner_size();
                if current_size.width > 200 {
                    let h_step = 15;
                    let w_step = 20;
                    if match delta {
                        MouseScrollDelta::LineDelta(_, y) => y > 0.0,
                        MouseScrollDelta::PixelDelta(pos) => pos.y > 0.0,
                    } {
                        context.window().set_inner_size(PhysicalSize::new(
                            current_size.width + w_step,
                            current_size.height + h_step,
                        ));
                    } else {
                        context.window().set_inner_size(PhysicalSize::new(
                            current_size.width - w_step,
                            current_size.height - h_step,
                        ));
                    };
                }
            }
            Event::RedrawRequested(_) => {
                redraw = true;
            }
            _ => {}
        }

        if redraw {
            fb.redraw();
            context.swap_buffers().unwrap();
        }
    });
}
