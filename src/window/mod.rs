use std::cmp::min;
use std::time::Duration;
use std::time::Instant;

use mini_gl_fb::core;
use mini_gl_fb::gl;
use mini_gl_fb::glutin::dpi::LogicalSize;
use mini_gl_fb::glutin::dpi::PhysicalPosition;
use mini_gl_fb::glutin::dpi::PhysicalSize;
use mini_gl_fb::glutin::event::MouseButton;
use mini_gl_fb::glutin::event::MouseScrollDelta;
use mini_gl_fb::glutin::event::VirtualKeyCode;
use mini_gl_fb::glutin::event::WindowEvent::KeyboardInput;
use mini_gl_fb::glutin::event::{ElementState, Event, WindowEvent};
use mini_gl_fb::glutin::event_loop::ControlFlow;
use mini_gl_fb::glutin::event_loop::EventLoop;
use mini_gl_fb::glutin::window::CursorIcon;
use mini_gl_fb::glutin::window::Icon;
use mini_gl_fb::glutin::window::WindowBuilder;
use mini_gl_fb::glutin::ContextBuilder;
use mini_gl_fb::glutin::PossiblyCurrent;
use mini_gl_fb::glutin::WindowedContext;
use mini_gl_fb::BufferFormat;
use mini_gl_fb::GlutinBreakout;
use mini_gl_fb::MiniGlFb;

use crate::camera::CameraSwitcher;
use crate::chromakey;

pub fn create(mut camera_switcher: CameraSwitcher) {
    let event_loop = EventLoop::new();

    let window_title = String::from("Headesk");
    let window_size = LogicalSize::new(camera_switcher.width, camera_switcher.height);
    let buffer_size = LogicalSize::new(camera_switcher.width, camera_switcher.height);

    let icon_png = include_bytes!("../../images/small-icon-48.png");
    let image = image::load_from_memory(icon_png);
    let image_bytes = image.unwrap().as_rgba8().unwrap().as_raw().to_vec();

    let window_builder = WindowBuilder::new()
        .with_decorations(false)
        .with_always_on_top(true)
        .with_transparent(true)
        .with_title(window_title.to_string())
        .with_inner_size(window_size)
        .with_visible(false)
        .with_resizable(true)
        .with_window_icon(Some(Icon::from_rgba(image_bytes, 48, 48).unwrap()));

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

    context.window().set_cursor_icon(CursorIcon::Wait);

    fb.use_vertex_shader(include_str!("./vertex_shader.glsl"));
    let mut distance: f32 = 1.0;
    let mut offset: (f32, f32) = (0.0, 0.0);
    let distance_loc =
        unsafe { gl::GetUniformLocation(fb.internal.program, b"distance\0".as_ptr() as *const _) };
    let offset_loc =
        unsafe { gl::GetUniformLocation(fb.internal.program, b"offset\0".as_ptr() as *const _) };

    let mut chromakey = chromakey::new(&mut fb);

    let mut last_frame_instant = Instant::now();
    let mut last_mouse_wheel = Instant::now();

    let mut current_window_size = context.window().inner_size();

    let mut is_ctrl = false;
    let mut left_pressed = false;
    let mut mouse_pos: Option<PhysicalPosition<f64>> = None;
    let mut drag_start: Option<PhysicalPosition<f64>> = None;
    let mut offset_delta: (f32, f32) = (0.0, 0.0);

    event_loop.run(move |event, _, flow| {
        let mut redraw = false;
        let window = context.window();

        // grab a frame from the camera periodically
        if Instant::now() > last_frame_instant + Duration::from_millis(10) {
            camera_switcher.read(|data| {
                chromakey.calibrate(data, buffer_size.width);
                fb.update_buffer(data);
                redraw = true;
            });
            context.window().set_cursor_icon(CursorIcon::Default);
            last_frame_instant = Instant::now();
        }
        *flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(5));

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                camera_switcher.close();
                *flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: KeyboardInput { input, .. },
                ..
            } => {
                if let Some(k) = input.virtual_keycode {
                    if k == VirtualKeyCode::Escape && input.state == ElementState::Pressed {
                        camera_switcher.close();
                        *flow = ControlFlow::Exit;
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(state),
                ..
            } => {
                is_ctrl = state.ctrl();
                chromakey.set_highlight(state.ctrl());
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
                fb.resize_viewport(size.width, size.height);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                if button == MouseButton::Left && state == ElementState::Pressed {
                    if is_ctrl {
                        if let Some(position) = mouse_pos {
                            drag_start = Some(position);
                        }
                    } else {
                        window.drag_window().unwrap();
                    }
                }
                if button == MouseButton::Left && state == ElementState::Released {
                    offset = (offset.0 + offset_delta.0, offset.1 + offset_delta.1);
                    offset_delta = (0.0, 0.0);
                }
                if button == MouseButton::Right && state == ElementState::Released {
                    context.window().set_cursor_icon(CursorIcon::Wait);
                    camera_switcher.next();
                    drag_start = None;
                }
                left_pressed = button == MouseButton::Left && state == ElementState::Pressed;
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                if is_ctrl && left_pressed {
                    if let Some(start) = drag_start {
                        let rel_dx = (position.x - start.x) / window.inner_size().width as f64;
                        let rel_dy = -(position.y - start.y) / window.inner_size().height as f64;
                        let mut offset_dx = 2.0 * distance * rel_dx as f32;
                        let mut offset_dy = 2.0 * distance * rel_dy as f32;
                        let offset_x = offset.0 + offset_dx;
                        let offset_y = offset.1 + offset_dy;

                        if offset_x.abs() > 1.0 - distance {
                            offset_dx = (1.0 - distance) * offset_x.signum() - offset.0;
                        }
                        if offset_y.abs() > 1.0 - distance {
                            offset_dy = (1.0 - distance) * offset_y.signum() - offset.1;
                        }

                        offset_delta = (offset_dx, offset_dy);
                        unsafe {
                            gl::ProgramUniform2f(
                                fb.internal.program,
                                offset_loc,
                                offset.0 + offset_delta.0,
                                offset.1 + offset_delta.1,
                            );
                        }
                    }
                } else {
                    mouse_pos = Some(position);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                if is_ctrl {
                    let mut increment: f32 = if match delta {
                        MouseScrollDelta::LineDelta(_, y) => y > 0.0,
                        MouseScrollDelta::PixelDelta(pos) => pos.y > 0.0,
                    } {
                        -0.05
                    } else {
                        0.05
                    };
                    if distance + increment > 1.0 {
                        increment = 1.0 - distance;
                        distance = 1.0
                    } else if distance + increment < 0.2 {
                        increment = 0.2 - distance;
                        distance = 0.2
                    } else {
                        distance += increment;
                    }
                    let mut offset_x = offset.0;
                    let mut offset_y = offset.1 - increment; // snap to the bottom
                    if offset_x.abs() > (1.0 - distance) {
                        offset_x = (1.0 - distance) * offset_x.signum();
                    }
                    if offset_y.abs() > (1.0 - distance) {
                        offset_y = (1.0 - distance) * offset_y.signum();
                    }
                    offset = (offset_x, offset_y);
                    unsafe {
                        gl::ProgramUniform1f(fb.internal.program, distance_loc, distance);
                        gl::ProgramUniform2f(fb.internal.program, offset_loc, offset.0, offset.1);
                    }
                } else {
                    if Instant::now() > last_mouse_wheel + Duration::from_millis(25) {
                        let elapsed = ((Instant::now() - last_mouse_wheel).as_millis()) as u32;
                        let accel = 3 - min(2, elapsed / 20);
                        let h_step = 15 * accel;
                        let w_step = 20 * accel;
                        let current_size = window.inner_size();
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
                    }
                    last_mouse_wheel = Instant::now();
                }
            }

            Event::RedrawRequested(_) => {
                redraw = true;
            }
            _ => {}
        }

        if redraw {
            context.window().set_visible(true);
            fb.redraw();
            context.swap_buffers().unwrap();
        }
    });
}

