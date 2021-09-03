use std::process;
use std::time::Duration;
use std::time::Instant;

use mini_gl_fb::glutin::event::VirtualKeyCode;

mod camera;
mod window;

fn main() {
    let mut video = camera::Camera::init(0);
    let mut update_id: Option<u32> = None;

    window::create(
    |fb, input| {
        input.wait = true;
        if update_id.is_none() {
            update_id = Some(input.schedule_wakeup(Instant::now() + Duration::from_millis(10)))
        } else if let Some(mut wakeup) = input.wakeup {
            if Some(wakeup.id) == update_id {
                video.read( |data| {
                    fb.update_buffer(data);
                });
                wakeup.when = Instant::now() + Duration::from_millis(5);
                input.reschedule_wakeup(wakeup);
            }
        }

        if input.key_is_down(VirtualKeyCode::Escape) {
            video.close();
            process::exit(0);
        }

        true
    });
}
