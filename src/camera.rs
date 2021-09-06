use std::time::{Duration, Instant};

use opencv::{core::Mat, core::Size, prelude::*, videoio::VideoCapture, videoio::CAP_ANY};

/// Iterating provider to let the user change the active camera.
pub struct CameraSwitcher {
    pub width: u32,
    pub height: u32,
    current: Option<Camera>,
}
pub struct Camera {
    width: u32,
    height: u32,
    index: i32,
    video: VideoCapture,
}

impl CameraSwitcher {
    pub fn new() -> CameraSwitcher {
        CameraSwitcher {
            width: 640,
            height: 480,
            current: None,
        }
    }

    pub fn current(&self) -> Option<i32> {
        self.current.as_ref().map(|provider| provider.index)
    }

    pub fn set_current(&mut self, index: i32) -> bool {
        if let Some(camera) = &mut self.current {
            camera.close()
        }
        if let Some(camera) = Camera::init(index, self.width, self.height) {
            self.current.replace(camera);
            return true;
        }
        false
    }

    pub fn next(&mut self) {
        let start = self
            .current
            .as_ref()
            .map(|camera| camera.index + 1)
            .unwrap_or(0);
        let mut index = start;
        loop {
            if self.set_current(index) {
                break;
            }
            index += 1;
            if index > 10 {
                index = 0;
            }
            if index == start {
                eprintln!("Could not find any camera");
                break;
            }
        }
    }

    pub fn read<F: FnMut(&[u8]) -> ()>(&mut self, func: F) {
        if let Some(camera) = &mut self.current {
            if !camera.read(func) {
                // switch to next camera if it cannot read or frames are not the right size
                self.next();
            }
        }
    }

    pub fn close(&mut self) {
        if let Some(camera) = &mut self.current {
            camera.close()
        }
    }
}

impl Camera {
    pub fn init(index: i32, width: u32, height: u32) -> Option<Camera> {
        match VideoCapture::new(index, CAP_ANY) {
            Ok(video) => {
                if VideoCapture::is_opened(&video).unwrap() {
                    Some(Camera {
                        index,
                        video,
                        width,
                        height,
                    })
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    pub fn read<F: FnMut(&[u8]) -> ()>(&mut self, mut func: F) -> bool {
        let mut frame = Mat::default();
        let grab_start = Instant::now();
        match self.video.read(&mut frame) {
            Ok(true) => unsafe {
                let bad_read_time = Instant::now() > grab_start + Duration::from_millis(1000);
                let wrong_size =
                    frame.size().unwrap() != Size::new(self.width as i32, self.height as i32);
                if bad_read_time || wrong_size {
                    return false;
                }
                match Mat::data_typed_unchecked::<u8>(&frame.reshape(1, 1).unwrap()) {
                    Ok(data) => {
                        func(&data);
                    }
                    Err(why) => {
                        eprintln!(
                            "Could not initialize frame for camera {}: {}",
                            self.index, why
                        );
                    }
                }
            },
            Ok(false) => {
                return false;
            }
            Err(why) => {
                eprintln!("Could not read frame from camera {}: {}", self.index, why);
                return false;
            }
        }
        true
    }

    fn close(&mut self) {
        self.video.release().unwrap_or_default();
    }
}
