use opencv::{core::Mat, prelude::*, videoio::VideoCapture, videoio::CAP_ANY};

/// Iterating provider to let the user change the active camera.
pub struct CameraSwitcher {
    current: Option<Camera>,
}
pub struct Camera {
    index: i32,
    video: VideoCapture,
}

impl CameraSwitcher {
    pub fn new(index: i32) -> CameraSwitcher {
        let mut result = CameraSwitcher { current: None };
        result.set_current(index);
        if let None = result.current {
            result.next();
        }
        result
    }

    pub fn _current(&self) -> Option<i32> {
        self.current.as_ref().map(|provider| provider.index)
    }

    pub fn set_current(&mut self, index: i32) -> bool {
        if let Some(camera) = &mut self.current {
            camera.close()
        }
        if let Some(camera) = Camera::init(index) {
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
            camera.read(func)
        }
    }

    pub fn close(&mut self) {
        if let Some(camera) = &mut self.current {
            camera.close()
        }
    }
}

impl Camera {
    pub fn init(index: i32) -> Option<Camera> {
        match VideoCapture::new(index, CAP_ANY) {
            Ok(cam) => {
                if VideoCapture::is_opened(&cam).unwrap() {
                    Some(Camera { index, video: cam })
                } else {
                    eprintln!("Camera {} could not be opened", index);
                    None
                }
            }
            Err(why) => {
                eprintln!("Cannot create camera {}: {}", index, why);
                None
            }
        }
    }

    pub fn read<F: FnMut(&[u8]) -> ()>(&mut self, mut func: F) {
        let mut frame = Mat::default();
        match self.video.read(&mut frame) {
            Ok(true) => unsafe {
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
                // no frame, do nothing
            }
            Err(why) => {
                eprintln!("Could not read frame from camera {}: {}", self.index, why);
            }
        }
    }

    fn close(&mut self) {
        self.video.release().unwrap_or_default();
    }
}
