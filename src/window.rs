use std::cmp::min;
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
use winit::dpi::PhysicalPosition;
use winit::dpi::PhysicalSize;
use winit::event::MouseScrollDelta;

use crate::camera::Camera;
use crate::chromakey;

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
        .with_visible(false)
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

    let mut glfb = MiniGlFb {
        internal: core::Internal { context, fb },
    };

    glfb.set_resizable(true);
    glfb.change_buffer_format::<u8>(BufferFormat::BGR);

    let GlutinBreakout { context, mut fb } = glfb.glutin_breakout();

    let mut chromakey = chromakey::new(&mut fb);

    let mut last_frame_instant = Instant::now();
    let mut last_mouse_wheel = Instant::now();

    let mut current_window_size = context.window().inner_size();

    let started = Instant::now();

    event_loop.run(move |event, _, flow| {
        let mut redraw = false;
        let window = context.window();

        if Instant::now() > last_frame_instant + Duration::from_millis(10) {
            camera.read(|data| {
                chromakey.calibrate(data, buffer_size.width);
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
                let position = window.outer_position().unwrap();
                window.set_outer_position(PhysicalPosition::new(
                    position.x + (current_window_size.width as i32 - size.width as i32) / 2,
                    position.y + (current_window_size.height as i32 - size.height as i32),
                ));
                current_window_size = size;
                context.resize(size);
                fb.resize_viewport(size.width, size.height);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, .. },
                ..
            } => {
                if state == ElementState::Pressed {
                    window.drag_window().unwrap();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                if Instant::now() > last_mouse_wheel + Duration::from_millis(25) {
                    let current_size = window.inner_size();
                    let elapsed = ((Instant::now() - last_mouse_wheel).as_millis() - 20) as u32;
                    let accel = 3 - min(2, elapsed / 20);
                    let h_step = 15 * accel;
                    let w_step = 20 * accel;
                    if match delta {
                        MouseScrollDelta::LineDelta(_, y) => y > 0.0,
                        MouseScrollDelta::PixelDelta(pos) => pos.y > 0.0,
                    } {
                        if current_size.height < 960 {
                            window.set_inner_size(PhysicalSize::new(
                                current_size.width + w_step,
                                current_size.height + h_step,
                            ));
                        }
                    } else {
                        let visible_y =
                            window.current_monitor().unwrap().size().height as i32 - 200;
                        if current_size.width > 200
                            && window.outer_position().unwrap().y < visible_y
                        {
                            window.set_inner_size(PhysicalSize::new(
                                current_size.width - w_step,
                                current_size.height - h_step,
                            ));
                        }
                    };
                    last_mouse_wheel = Instant::now();
                }
            }
            Event::RedrawRequested(_) => {
                redraw = true;
            }
            _ => {}
        }

        if redraw {
            if Instant::now() > started + Duration::from_millis(200) {
                context.window().set_visible(true);
            }
            fb.redraw();
            context.swap_buffers().unwrap();
        }
    });
}
