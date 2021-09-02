use opencv::{core::Mat, prelude::*, videoio::VideoCapture, videoio::CAP_ANY};

pub struct Camera {
    video: VideoCapture
}

impl Camera {
    pub fn init(index: i32) -> Camera {
        #[cfg(ocvrs_opencv_branch_32)]
        let mut cam = VideoCapture::new_default(0).unwrap();
        #[cfg(not(ocvrs_opencv_branch_32))]
        let cam = VideoCapture::new(index, CAP_ANY).unwrap();
        let opened = VideoCapture::is_opened(&cam).unwrap();
        if !opened {
            panic!("Unable to open default camera!");
        }
        Camera {
            video: cam
        }
    }

    pub fn read(&mut self, mat: &mut Mat) -> bool {
        self.video.read(mat).unwrap()
    }

}
