use opencv::{core::Mat, prelude::*, videoio::VideoCapture, videoio::CAP_ANY};

pub struct Camera {
    video: VideoCapture,
}

impl Camera {
    pub fn init(index: i32) -> Camera {
        let cam = VideoCapture::new(index, CAP_ANY).unwrap();
        let opened = VideoCapture::is_opened(&cam).unwrap();
        if !opened {
            panic!("Unable to open default camera!");
        }
        Camera { video: cam }
    }

    pub fn read<F: FnMut(&[u8]) -> ()>(&mut self, mut func: F) {
        let mut frame = Mat::default();
        match self.video.read(&mut frame) {
            Ok(true) => unsafe {
                match Mat::data_typed_unchecked::<u8>(&frame.reshape(1, 1).unwrap()) {
                    Ok(data) => {
                        func(&data);
                    }
                    Err(why) => panic!("{}", why),
                }
            },
            Ok(false) => {
                // no frame, do nothing
            }
            Err(why) => {
                panic!("{}", why);
            }
        }
    }

    pub fn close(&mut self) {
        self.video.release().unwrap_or_default();
    }
}
