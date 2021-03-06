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

#[cfg(target_os = "windows")]
use mini_gl_fb::glutin::platform::windows::WindowBuilderExtWindows;

use crate::camera::CameraSwitcher;
use crate::chromakey;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct View {
    pub camera_index: i32,
    pub size: (u32, u32),
    pub position: (i32, i32),
    pub distance: f32,
    pub offset: (f32, f32),
}

pub fn create<F: 'static + FnMut(View) -> ()>(
    mut camera_switcher: CameraSwitcher,
    view: Option<View>,
    mut store: F,
) {
    let mut view = if let Some(view) = view {
        view
    } else {
        View {
            camera_index: 0,
            size: (camera_switcher.width, camera_switcher.height),
            position: (800, 600),
            distance: 1.0,
            offset: (0.0, 0.0),
        }
    };
    camera_switcher.set_current(view.camera_index);

    let event_loop = EventLoop::new();

    let window_title = String::from("Headesk");
    let window_size = LogicalSize::new(view.size.0, view.size.1);
    let buffer_size = LogicalSize::new(camera_switcher.width, camera_switcher.height);

    let icon_png = include_bytes!("../../images/small-icon-48.png");
    let image = image::load_from_memory(icon_png);
    let image_bytes = image.unwrap().as_rgba8().unwrap().as_raw().to_vec();

    #[allow(unused_mut)]
    let mut window_builder = WindowBuilder::new()
        .with_position(PhysicalPosition::new(view.position.0, view.position.1))
        .with_decorations(false)
        .with_always_on_top(true)
        .with_transparent(true)
        .with_title(window_title.to_string())
        .with_inner_size(window_size)
        .with_visible(false)
        .with_resizable(true)
        .with_window_icon(Some(Icon::from_rgba(image_bytes, 48, 48).unwrap()));

    #[cfg(target_os = "windows")]
    {
        let icon_png = include_bytes!("../../images/icon-512.png");
        let image = image::load_from_memory(icon_png);
        let image_bytes = image.unwrap().as_rgba8().unwrap().as_raw().to_vec();
        window_builder =
            window_builder.with_taskbar_icon(Some(Icon::from_rgba(image_bytes, 512, 512).unwrap()));
    }

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
    let distance_loc =
        unsafe { gl::GetUniformLocation(fb.internal.program, b"distance\0".as_ptr() as *const _) };
    let offset_loc =
        unsafe { gl::GetUniformLocation(fb.internal.program, b"offset\0".as_ptr() as *const _) };

    let mut chromakey = chromakey::new(&mut fb);
    set_geometry(&fb, view, distance_loc, offset_loc);

    let mut last_frame_instant = Instant::now();

    let mut is_shift = false;
    let mut left_pressed = false;
    let mut mouse_pos: Option<PhysicalPosition<f64>> = None;
    let mut drag_start: Option<PhysicalPosition<f64>> = None;
    let mut offset_delta: (f32, f32) = (0.0, 0.0);

    let mut last_zoom_event = Instant::now();

    let mut first_loop = true;

    event_loop.run(move |event, _, flow| {
        let mut redraw = false;
        let window = context.window();

        // grab a frame from the camera periodically
        if Instant::now() > last_frame_instant + Duration::from_millis(10) {
            camera_switcher.read(|data| {
                fb.update_buffer(data);
                redraw = true;
            });
            context.window().set_cursor_icon(CursorIcon::Default);

            last_frame_instant = Instant::now();
        }

        *flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(5));

        match event {
            Event::LoopDestroyed => {
                view.position = window
                    .outer_position()
                    .map_or(view.position, |p| (p.x, p.y));
                view.camera_index = camera_switcher.current().unwrap_or_default();
                store(view);
                camera_switcher.close();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: KeyboardInput { input, .. },
                ..
            } => {
                if let Some(k) = input.virtual_keycode {
                    if k == VirtualKeyCode::Escape && input.state == ElementState::Pressed {
                        *flow = ControlFlow::Exit;
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(state),
                ..
            } => {
                is_shift = state.shift();
                chromakey.set_highlight(state.shift());
            }
            Event::WindowEvent {
                event: WindowEvent::Moved(position),
                ..
            } => {
                // avoid messing up with zoom events because of weird X11 event ordering bug
                if Instant::now() > last_zoom_event + Duration::from_millis(100) {
                    view.position = (position.x, position.y);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                if button == MouseButton::Left && state == ElementState::Pressed {
                    if is_shift {
                        if let Some(position) = mouse_pos {
                            drag_start = Some(position);
                        }
                    } else {
                        window.drag_window().unwrap();
                    }
                }
                if button == MouseButton::Left && state == ElementState::Released {
                    view.offset = (
                        view.offset.0 + offset_delta.0,
                        view.offset.1 + offset_delta.1,
                    );
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
                if is_shift && left_pressed {
                    if let Some(start) = drag_start {
                        let rel_dx = (position.x - start.x) / window.inner_size().width as f64;
                        let rel_dy = -(position.y - start.y) / window.inner_size().height as f64;
                        let mut offset_dx = 2.0 * view.distance * rel_dx as f32;
                        let mut offset_dy = 2.0 * view.distance * rel_dy as f32;
                        let offset_x = view.offset.0 + offset_dx;
                        let offset_y = view.offset.1 + offset_dy;

                        if offset_x.abs() > 1.0 - view.distance {
                            offset_dx = (1.0 - view.distance) * offset_x.signum() - view.offset.0;
                        }
                        if offset_y.abs() > 1.0 - view.distance {
                            offset_dy = (1.0 - view.distance) * offset_y.signum() - view.offset.1;
                        }

                        offset_delta = (offset_dx, offset_dy);
                        unsafe {
                            gl::ProgramUniform2f(
                                fb.internal.program,
                                offset_loc,
                                view.offset.0 + offset_delta.0,
                                view.offset.1 + offset_delta.1,
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
                if is_shift {
                    let mut increment: f32 = if match delta {
                        MouseScrollDelta::LineDelta(_, y) => y > 0.0,
                        MouseScrollDelta::PixelDelta(pos) => pos.y > 0.0,
                    } {
                        -0.05
                    } else {
                        0.05
                    };
                    if view.distance + increment > 1.0 {
                        increment = 1.0 - view.distance;
                        view.distance = 1.0
                    } else if view.distance + increment < 0.2 {
                        increment = 0.2 - view.distance;
                        view.distance = 0.2
                    } else {
                        view.distance += increment;
                    }
                    let mut offset_x = view.offset.0;
                    let mut offset_y = view.offset.1 - increment; // snap to the bottom
                    if offset_x.abs() > (1.0 - view.distance) {
                        offset_x = (1.0 - view.distance) * offset_x.signum();
                    }
                    if offset_y.abs() > (1.0 - view.distance) {
                        offset_y = (1.0 - view.distance) * offset_y.signum();
                    }
                    view.offset = (offset_x, offset_y);
                    set_geometry(&fb, view, distance_loc, offset_loc);
                } else {
                    let h_step = 15;
                    let w_step = 20;
                    if match delta {
                        MouseScrollDelta::LineDelta(_, y) => y > 0.0,
                        MouseScrollDelta::PixelDelta(pos) => pos.y > 0.0,
                    } {
                        if view.size.1 < 960 {
                            view.size = (view.size.0 + w_step, view.size.1 + h_step);
                            view.position = (
                                view.position.0 - w_step as i32 / 2,
                                view.position.1 - h_step as i32,
                            );
                        }
                    } else {
                        let visible_y =
                            window.current_monitor().unwrap().size().height as i32 - 200;
                        if view.size.0 > 200 && window.outer_position().unwrap().y < visible_y {
                            view.size = (view.size.0 - w_step, view.size.1 - h_step);
                            view.position = (
                                view.position.0 + w_step as i32 / 2,
                                view.position.1 + h_step as i32,
                            );
                        }
                    }
                    window.set_outer_position(PhysicalPosition::new(
                        view.position.0,
                        view.position.1,
                    ));
                    fb.resize_viewport(view.size.0, view.size.1);
                    window.set_inner_size(PhysicalSize::new(view.size.0, view.size.1));
                    window.request_redraw();
                    last_zoom_event = Instant::now();
                }
            }

            Event::RedrawRequested(_) => {
                redraw = true;
            }
            _ => {}
        }

        if redraw {
            if first_loop {
                window.set_inner_size(PhysicalSize::new(view.size.0, view.size.1));
                window.set_outer_position(PhysicalPosition::new(view.position.0, view.position.1));
                context.window().set_visible(true);
                first_loop = false;
            }
            fb.redraw();
            context.swap_buffers().unwrap();
        }
    });
}

fn set_geometry(
    fb: &mini_gl_fb::Framebuffer,
    view: View,
    distance_loc: gl::types::GLint,
    offset_loc: gl::types::GLint,
) {
    unsafe {
        gl::ProgramUniform1f(fb.internal.program, distance_loc, view.distance);
        gl::ProgramUniform2f(
            fb.internal.program,
            offset_loc,
            view.offset.0,
            view.offset.1,
        );
    }
}
